---
title: Python
description: Render charts from Python, show them in Jupyter, and write them in Sphinx and MkDocs pages, with the WebAssembly renderer in the chartlet package.
order: 3
---

The Python package `chartlet` contains the renderer compiled to WebAssembly, the module the npm
package ships, and runs it with [wasmtime](https://pypi.org/project/wasmtime/). It produces the
same bytes as the CLI and needs no Rust toolchain, no browser and no network.

```sh
pip install chartlet
```

It needs Python 3.10 or newer. wasmtime comes as a wheel for Linux (glibc and musl), macOS and
Windows on x86-64 and ARM64.

## Rendering

```python
import chartlet

svg = chartlet.render({
    "schemaVersion": 1,
    "type": "bar",
    "title": "Monthly revenue",
    "data": [
        {"label": "January", "value": 120},
        {"label": "February", "value": 180},
    ],
})
```

`render` returns what `chartlet render -o` writes with the same options: SVG by default, the
HTML figure with `format="html"`. The specification is a dictionary, JSON text, or a
`pathlib.Path` of a JSON file; a plain string is always read as JSON text, not as a path.

The options are those of the CLI, in snake case:

| Option | CLI |
| --- | --- |
| `format` | `--format svg\|html` |
| `table` | `--table details\|visible` |
| `id_prefix` | `--id-prefix` |
| `variant` | `--variant desktop\|mobile\|print\|social` |
| `view`, `all_views` | `--view`, `--all-views` |
| `perspective` | `--perspective` |
| `theme` | `--theme`: a bundled theme by its name, or a theme file as a dictionary |
| `styles` | `--styles inline\|external` |
| `focus` | `--focus`, `--no-focus` |
| `hooks` | `--hooks` |
| `strict`, `allow_warnings` | `--strict`, `--allow-warning` |

`--size` is the `size` field of the specification. PNG output, fonts, watching and the preview
stay with the Rust CLI.

An invalid specification or option raises `chartlet.ChartletError`, a `ValueError` with the
`code`, `path` and `message` the CLI reports, and the warnings that came before it. Each warning
of a successful render is issued as a `chartlet.ChartletWarning` through Python's `warnings`
module, so the usual filters apply:

```python
import warnings

warnings.simplefilter("error", chartlet.ChartletWarning)  # treat every warning as an error
```

`render_detailed` takes the same options and returns the warnings as data instead, together with
the CSP hashes of the inline styles (`style_hashes`) and, when asked for, the provenance manifest
(`manifest=True`), the [text alternative](../javascript/#text-alternative) (`alternative=True`)
and the help for layout warnings (`advice=True`):

```python
rendered = chartlet.render_detailed(spec, format="html", manifest=True)
rendered.content, rendered.warnings, rendered.manifest
```

`chartlet.stylesheet()` returns the shared stylesheet that charts rendered with
`styles="external"` rely on; `types=["bar", "time"]`, `common` and `breakpoints` work as in
[`stylesheet`](../javascript/) of the npm package.

The renderer loads once per process, on the first call, and takes well under a second to
compile; every render after it takes milliseconds. Calls from several threads are serialized.

## Jupyter

`chartlet.Chart` shows its HTML figure in a notebook cell, with the caption, the chart and the
data table:

```python
chartlet.Chart(spec)
chartlet.Chart(spec, table="visible", theme="dark")
```

It takes the options of `render`; the format defaults to HTML. The figure needs no script. Two charts of the same specification in one notebook
share their IDs; give each its own `id_prefix`.

## Sphinx

```python
# conf.py
extensions = ["chartlet.sphinx"]
```

The `chartlet` directive takes the specification as its content:

```rst
.. chartlet::
   :id: revenue

   {
     "schemaVersion": 1,
     "type": "bar",
     "title": "Monthly revenue",
     "data": [{ "label": "January", "value": 120 }, { "label": "February", "value": 180 }]
   }
```

With MyST Markdown, the same directive is a fence. Add `chartlet` to `myst_fence_as_directive`,
and a plain ` ```chartlet ` block with the [fence settings](../markdown/#fence-settings) after the
language works as it does in every other generator:

```python
# conf.py
extensions = ["myst_parser", "chartlet.sphinx"]
myst_fence_as_directive = ["chartlet"]
```

The settings `id`, `view` and `perspective` are options (`:id: revenue`) or the directive's
argument (`.. chartlet:: id=revenue`). `chartlet_options` in `conf.py` applies the options of
`render` to every chart, such as `{"table": "visible", "theme": "dark"}`.

An invalid specification is an error at the line of the directive; `sphinx-build -W` turns it into
a failed build. A warning is a Sphinx warning of type `chartlet` with the warning's code as
subtype, so `suppress_warnings = ["chartlet.dense_chart"]` silences one code. The chart is HTML;
builders without HTML output, such as LaTeX, leave it out.

## MkDocs

```yaml
# mkdocs.yml
markdown_extensions:
  - chartlet
```

Every fenced block in the language `chartlet` becomes the chart's figure, with the
[fence settings](../markdown/#fence-settings) after the language. Options under the extension
apply to every chart:

```yaml
markdown_extensions:
  - chartlet:
      table: visible
      theme: dark
```

The options are `table`, `theme` (a name, or a theme file written as a mapping), `styles`,
`focus`, `hooks`, `strict` and `allow_warnings`, plus `lang` for another fence language.

The extension works with the default fenced code blocks and alongside a fence extension that
highlights code: it renders `chartlet` blocks before the others see them, leaves every other fence
as it is, and ignores a `chartlet` block shown inside a longer fence. A block may be indented, as
in a list item, an admonition or a content tab.

An invalid specification stops the build with the page and the number of the block on it. Warnings
go to the MkDocs log, so `mkdocs build --strict` fails on them. Outside MkDocs, the extension
works in any Python-Markdown pipeline as `markdown.markdown(text, extensions=["chartlet"])`.
