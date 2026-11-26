"""What the Markdown extension and the Sphinx directive share: the settings of a block and
rendering one."""

from __future__ import annotations

import json
import re
from collections.abc import Mapping
from typing import Any

from . import Rendered, render_detailed

# The settings a block may carry after its language, such as ```chartlet id=revenue view=checkout,
# by the name of the render option each sets.
FENCE_SETTINGS = {"id": "id_prefix", "view": "view", "perspective": "perspective"}
_SETTING = re.compile(r"""\s*([^\s=]+)=(?:"([^"]*)"|'([^']*)'|(\S+))""")


def fence_settings(meta: str) -> dict[str, str]:
    """Reads `key=value` settings, quoted when the value has spaces, as render options."""
    settings = {}
    rest = meta.strip()
    while rest:
        match = _SETTING.match(rest)
        if not match:
            raise ValueError(
                f"cannot read {json.dumps(rest, ensure_ascii=False)} after the fence language; "
                f"write key=value with {', '.join(FENCE_SETTINGS)}"
            )
        key = match[1]
        if key not in FENCE_SETTINGS:
            raise ValueError(
                f"unknown fence setting {json.dumps(key, ensure_ascii=False)}; "
                f"the settings are {', '.join(FENCE_SETTINGS)}"
            )
        settings[FENCE_SETTINGS[key]] = next(v for v in match.group(2, 3, 4) if v is not None)
        rest = rest[match.end() :].strip()
    return settings


def render_block(source: str, settings: Mapping[str, str], options: Mapping[str, Any]) -> Rendered:
    """Renders the specification of a block as the HTML figure, with the options of every block
    and the block's own settings. An invalid specification raises `ChartletError`."""
    return render_detailed(source, **{"format": "html", **options, **settings})
