"""Puts the WebAssembly renderer at src/chartlet/chartlet.wasm before a build.

- `CHARTLET_WASM` names a module that is built already, such as the one CI built for the npm
  package: it is copied.
- In the repository, the npm package's build script compiles the workspace crate `wasm/` (and
  runs wasm-opt when it is on PATH); its output is copied.
- From an sdist, the module it carries is used.
"""

import os
import shutil
import subprocess
from pathlib import Path

from hatchling.builders.hooks.plugin.interface import BuildHookInterface


class WasmHook(BuildHookInterface):
    def initialize(self, version, build_data):
        root = Path(self.root)
        target = root / "src" / "chartlet" / "chartlet.wasm"
        script = root.parent / "packages" / "chartlet" / "scripts" / "build-wasm.mjs"
        if os.environ.get("CHARTLET_WASM"):
            shutil.copyfile(os.environ["CHARTLET_WASM"], target)
        elif script.is_file():
            subprocess.run(["node", str(script)], check=True)
            shutil.copyfile(script.parent.parent / "src" / "chartlet.wasm", target)
        elif not target.is_file():
            raise RuntimeError(
                "src/chartlet/chartlet.wasm is missing: build from the chartlet repository "
                "or set CHARTLET_WASM to a built module"
            )
