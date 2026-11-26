"""Compile structured data into deterministic, accessible static charts.

The renderer is chartlet compiled to WebAssembly, the module the npm package ships, run with
wasmtime: `render` returns the same bytes as `chartlet render -o` with the same options.
"""

from __future__ import annotations

import json
import os
import threading
import warnings
from collections.abc import Iterable, Mapping
from dataclasses import dataclass, field
from importlib.metadata import version
from pathlib import Path
from typing import Any, Literal, Union

__version__ = version("chartlet")

__all__ = [
    "Chart",
    "ChartletError",
    "ChartletWarning",
    "Diagnostic",
    "Rendered",
    "render",
    "render_detailed",
    "stylesheet",
]

Spec = Union[Mapping[str, Any], str, "os.PathLike[str]"]
"""A specification: a mapping, JSON text, or the path of a JSON file."""

Theme = Union[Literal["light", "dark", "blueprint", "classic", "editorial"], Mapping[str, Any]]
"""A bundled theme by its name, or a theme file as a mapping."""


@dataclass(frozen=True)
class Diagnostic:
    """An error or warning as the CLI reports it with `--diagnostics json`."""

    code: str
    path: str | None
    """A JSON Pointer into the specification; `None` for option errors and `strict_warnings`."""
    message: str

    def __str__(self) -> str:
        return self.message if self.path is None else f"{self.code} at {self.path}: {self.message}"


class ChartletError(ValueError):
    """A specification or option the renderer rejected, with the warnings it reported first."""

    def __init__(self, error: Diagnostic, warnings: Iterable[Diagnostic] = ()) -> None:
        super().__init__(str(error))
        self.code = error.code
        self.path = error.path
        self.message = error.message
        self.warnings = list(warnings)


class ChartletWarning(UserWarning):
    """A warning of a render, issued by `render` with `warnings.warn`."""

    def __init__(self, diagnostic: Diagnostic) -> None:
        super().__init__(f"warning[{diagnostic.code}] at {diagnostic.path}: {diagnostic.message}")
        self.diagnostic = diagnostic


@dataclass(frozen=True)
class Rendered:
    """A successful render with everything the renderer reported."""

    content: str
    style_hashes: list[str]
    """The CSP source expressions (`'sha256-…'`) of the inline `<style>` elements in `content`."""
    warnings: list[Diagnostic]
    manifest: dict[str, Any] | None = None
    """The provenance manifest, with `manifest=True`."""
    alternative: dict[str, Any] | None = None
    """The text alternative, `{"description", "table": {"caption", "columns", "rows"}}`."""
    alternatives: list[dict[str, Any]] | None = None
    """With `alternative=True` and `all_views=True`: that of every view on the page."""
    help: list[dict[str, Any]] = field(default_factory=list)
    """With `advice=True`: the structured help for layout warnings."""


def render(
    spec: Spec,
    *,
    format: Literal["svg", "html"] = "svg",
    table: Literal["details", "visible"] | None = None,
    id_prefix: str | None = None,
    variant: Literal["desktop", "mobile", "print", "social"] | None = None,
    view: str | None = None,
    all_views: bool = False,
    perspective: str | None = None,
    theme: Theme | None = None,
    styles: Literal["inline", "external"] | None = None,
    focus: bool | None = None,
    hooks: bool = False,
    strict: bool = False,
    allow_warnings: Iterable[str] = (),
) -> str:
    """Renders a specification to SVG or HTML.

    The options are those of `chartlet render`. Each warning is issued as a `ChartletWarning`;
    an invalid specification or option raises `ChartletError`.
    """
    rendered = render_detailed(
        spec,
        format=format,
        table=table,
        id_prefix=id_prefix,
        variant=variant,
        view=view,
        all_views=all_views,
        perspective=perspective,
        theme=theme,
        styles=styles,
        focus=focus,
        hooks=hooks,
        strict=strict,
        allow_warnings=allow_warnings,
    )
    for warning in rendered.warnings:
        warnings.warn(ChartletWarning(warning), stacklevel=2)
    return rendered.content


def render_detailed(
    spec: Spec,
    *,
    format: Literal["svg", "html"] = "svg",
    table: Literal["details", "visible"] | None = None,
    id_prefix: str | None = None,
    variant: Literal["desktop", "mobile", "print", "social"] | None = None,
    view: str | None = None,
    all_views: bool = False,
    perspective: str | None = None,
    theme: Theme | None = None,
    styles: Literal["inline", "external"] | None = None,
    focus: bool | None = None,
    hooks: bool = False,
    strict: bool = False,
    allow_warnings: Iterable[str] = (),
    manifest: bool = False,
    alternative: bool = False,
    advice: bool = False,
) -> Rendered:
    """Renders like `render` and returns the warnings, the CSP hashes of the inline styles and,
    when asked for, the manifest, the text alternative and the help for layout warnings."""
    response = _send(
        {
            "spec": _spec_text(spec),
            "options": {
                "format": format,
                "table": table,
                "idPrefix": id_prefix or None,
                "variant": variant,
                "view": view or None,
                "allViews": all_views,
                "perspective": perspective or None,
                "theme": theme or None,
                "styles": styles,
                "focus": focus,
                "hooks": hooks,
                "strict": strict,
                "allowWarnings": list(allow_warnings),
                "manifest": manifest,
                "alternative": alternative,
                "advice": advice,
            },
        }
    )
    diagnostics = [Diagnostic(**warning) for warning in response["warnings"]]
    if not response["ok"]:
        raise ChartletError(Diagnostic(**response["error"]), diagnostics)
    return Rendered(
        content=response["content"],
        style_hashes=response["styleHashes"],
        warnings=diagnostics,
        manifest=response.get("manifest"),
        alternative=response.get("alternative"),
        alternatives=response.get("alternatives"),
        help=response.get("help", []),
    )


def stylesheet(
    types: Iterable[str] | None = None,
    *,
    common: bool = True,
    breakpoints: Iterable[int] = (),
) -> str:
    """The shared stylesheet that charts rendered with `styles="external"` rely on: for every
    chart type, or only for `types`. `chartlet stylesheet` writes the same rules."""
    response = _send(
        {
            "stylesheet": True,
            "types": None if types is None else list(types),
            "common": common,
            "breakpoints": list(breakpoints),
        }
    )
    if not response["ok"]:
        raise ChartletError(Diagnostic(**response["error"]))
    return response["stylesheet"]


class Chart:
    """A chart that shows itself in Jupyter and other IPython front ends.

    The options are those of `render`; the format defaults to HTML, the figure with its caption
    and data table.
    """

    def __init__(self, spec: Spec, **options: Any) -> None:
        self.spec = spec
        self.options = options

    def _repr_html_(self) -> str:
        return render(self.spec, **{"format": "html", **self.options})


def _spec_text(spec: Spec) -> str:
    if isinstance(spec, str):
        return spec
    if isinstance(spec, os.PathLike):
        return Path(spec).read_text(encoding="utf-8")
    return json.dumps(spec, ensure_ascii=False)


class _Renderer:
    """One instance of the WebAssembly module; a lock keeps calls from different threads apart."""

    def __init__(self) -> None:
        import wasmtime

        engine = wasmtime.Engine()
        self._store = wasmtime.Store(engine)
        module = wasmtime.Module.from_file(engine, str(Path(__file__).with_name("chartlet.wasm")))
        exports = wasmtime.Instance(self._store, module, []).exports(self._store)
        self._memory = exports["memory"]
        self._alloc = exports["alloc"]
        self._dealloc = exports["dealloc"]
        self._render = exports["render"]
        self._result_len = exports["result_len"]

    def send(self, message: dict[str, Any]) -> dict[str, Any]:
        store = self._store
        request = json.dumps(message, ensure_ascii=False).encode()
        pointer = self._alloc(store, len(request))
        self._memory.write(store, request, pointer)
        output = self._render(store, pointer, len(request))
        length = self._result_len(store)
        response = bytes(self._memory.read(store, output, output + length))
        self._dealloc(store, pointer, len(request))
        self._dealloc(store, output, length)
        return json.loads(response)


_lock = threading.Lock()
_instance: _Renderer | None = None


def _send(message: dict[str, Any]) -> dict[str, Any]:
    global _instance
    with _lock:
        if _instance is None:
            _instance = _Renderer()
        try:
            return _instance.send(message)
        except Exception:
            # A trap leaves the module's memory in an unknown state: start over next time.
            _instance = None
            raise
