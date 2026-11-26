import json
import os
import shutil
import subprocess
from pathlib import Path

import pytest

REPOSITORY = Path(__file__).resolve().parents[2]
EXAMPLES = REPOSITORY / "examples"


@pytest.fixture(scope="session")
def cli() -> Path:
    """The Rust CLI to compare with: `CHARTLET_BIN`, or a release build of the repository."""
    if os.environ.get("CHARTLET_BIN"):
        return Path(os.environ["CHARTLET_BIN"])
    if not (REPOSITORY / "Cargo.toml").is_file() or shutil.which("cargo") is None:
        pytest.skip("needs the chartlet repository and cargo, or CHARTLET_BIN")
    subprocess.run(["cargo", "build", "--release", "--quiet"], cwd=REPOSITORY, check=True)
    metadata = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=REPOSITORY,
        check=True,
        capture_output=True,
        text=True,
    )
    return Path(json.loads(metadata.stdout)["target_directory"]) / "release" / "chartlet"


@pytest.fixture
def cli_render(cli, tmp_path):
    """Renders a specification file with the CLI and returns what it writes with `-o`."""

    def run(spec: Path, *arguments: str) -> str:
        output = tmp_path / "output"
        subprocess.run([cli, "render", spec, *arguments, "-o", output], check=True)
        return output.read_text(encoding="utf-8")

    return run
