"""A Python-Markdown extension that renders fenced `chartlet` blocks, for MkDocs and any
Python-Markdown pipeline.

    markdown_extensions:
      - chartlet:
          table: visible

A block becomes the chart's HTML figure, the same bytes as `chartlet render --format html`.
Fences in other languages, and `chartlet` fences inside them, stay as they are.
"""

from __future__ import annotations

import logging
import re
from typing import Any

from markdown import Markdown
from markdown.extensions import Extension
from markdown.preprocessors import Preprocessor

from ._fence import fence_settings, render_block

# Inside MkDocs, warnings go to its log, so `mkdocs build --strict` fails on them.
_LOGGER = logging.getLogger("mkdocs.extensions.chartlet")
_OPEN = re.compile(r"^(?P<indent> *)(?P<fence>`{3,}|~{3,})(?P<info>.*)$")

# The render options that apply to every block, with defaults of the type Python-Markdown parses
# the value as: it reads a value as a boolean when the default is a boolean or `None`.
_OPTIONS = {
    "table": ["", "details (default) or visible for the data table"],
    "theme": ["", "a bundled theme by its name, or a theme file as a mapping"],
    "styles": ["", "inline (default) or external"],
    "focus": [None, "whether the figure of a diagram carries its focus control"],
    "hooks": [False, "add the data-* hooks of the interactive layer"],
    "strict": [False, "fail on any warning, except those listed in allow_warnings"],
    "allow_warnings": [[], "warning codes that strict lets through"],
}


class ChartletExtension(Extension):
    def __init__(self, **kwargs: Any) -> None:
        self.config = {"lang": ["chartlet", "the fence language"]}
        self.config.update({key: list(value) for key, value in _OPTIONS.items()})
        super().__init__(**kwargs)

    def extendMarkdown(self, md: Markdown) -> None:  # noqa: N802 (the Python-Markdown API)
        config = self.getConfigs()
        options = {key: config[key] for key in _OPTIONS if config[key] not in ("", None)}
        # After whitespace normalization (30), before fenced code (25) and raw HTML (20).
        md.preprocessors.register(ChartletPreprocessor(md, config["lang"], options), "chartlet", 27)


class ChartletPreprocessor(Preprocessor):
    def __init__(self, md: Markdown, lang: str, options: dict[str, Any]) -> None:
        super().__init__(md)
        self.lang = lang
        self.options = options

    def run(self, lines: list[str]) -> list[str]:
        result: list[str] = []
        index = 0
        block = 0
        while index < len(lines):
            opening = _OPEN.match(lines[index])
            if not opening or (opening["fence"][0] == "`" and "`" in opening["info"]):
                result.append(lines[index])
                index += 1
                continue
            indent, fence = opening["indent"], opening["fence"]
            closing = re.compile(rf"^ *{re.escape(fence[0])}{{{len(fence)},}}\s*$")
            end = index + 1
            while end < len(lines) and not closing.match(lines[end]):
                end += 1
            language, _, meta = opening["info"].strip().partition(" ")
            if language != self.lang:
                result.extend(lines[index : end + 1])
                index = end + 1
                continue
            block += 1
            source = "\n".join(_dedent(line, len(indent)) for line in lines[index + 1 : end])
            try:
                rendered = render_block(source, fence_settings(meta), self.options)
            except ValueError as error:
                raise ValueError(f"chartlet block {block}: {error}") from error
            for warning in rendered.warnings:
                _LOGGER.warning(
                    "chartlet block %d: warning[%s] at %s: %s",
                    block,
                    warning.code,
                    warning.path,
                    warning.message,
                )
            result.extend(["", indent + self.md.htmlStash.store(rendered.content), ""])
            index = end + 1
        return result


def _dedent(line: str, width: int) -> str:
    """Removes up to `width` leading spaces, the indentation of the opening fence."""
    stripped = line.lstrip(" ")
    return line[min(width, len(line) - len(stripped)) :]


def makeExtension(**kwargs: Any) -> ChartletExtension:  # noqa: N802 (the Python-Markdown API)
    return ChartletExtension(**kwargs)
