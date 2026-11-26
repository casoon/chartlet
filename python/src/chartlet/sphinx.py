"""A Sphinx extension with the `chartlet` directive, for reStructuredText and MyST Markdown.

    extensions = ["chartlet.sphinx"]
    chartlet_options = {"table": "visible"}

The directive's content is the specification; the settings `id`, `view` and `perspective` are
options (`:id: revenue`) or the argument (`.. chartlet:: id=revenue`), which is how MyST passes
the rest of a fence's info string with `myst_fence_as_directive = ["chartlet"]`.
"""

from __future__ import annotations

from typing import Any

from docutils import nodes
from docutils.parsers.rst import directives
from sphinx.application import Sphinx
from sphinx.util import logging
from sphinx.util.docutils import SphinxDirective

from . import __version__
from ._fence import FENCE_SETTINGS, fence_settings, render_block

_LOGGER = logging.getLogger(__name__)


class ChartletDirective(SphinxDirective):
    has_content = True
    optional_arguments = 1
    final_argument_whitespace = True
    option_spec = {key: directives.unchanged_required for key in FENCE_SETTINGS}

    def run(self) -> list[nodes.Node]:
        try:
            settings = fence_settings(self.arguments[0] if self.arguments else "")
            settings.update({FENCE_SETTINGS[key]: value for key, value in self.options.items()})
            rendered = render_block("\n".join(self.content), settings, self.config.chartlet_options)
        except ValueError as error:
            raise self.error(f"chartlet: {error}") from error
        for warning in rendered.warnings:
            _LOGGER.warning(
                "chartlet: warning[%s] at %s: %s",
                warning.code,
                warning.path,
                warning.message,
                location=self.get_location(),
                type="chartlet",
                subtype=warning.code,
            )
        # The figure is HTML; other builders leave it out.
        return [nodes.raw("", rendered.content, format="html")]


def setup(app: Sphinx) -> dict[str, Any]:
    app.add_config_value("chartlet_options", {}, "env", types=frozenset({dict}))
    app.add_directive("chartlet", ChartletDirective)
    return {"version": __version__, "parallel_read_safe": True, "parallel_write_safe": True}
