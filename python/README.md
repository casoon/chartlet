# chartlet

Compile structured data into deterministic, accessible static charts and diagrams, from Python.

```sh
pip install chartlet
```

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

`render` returns the same bytes as `chartlet render` with the same options. The package carries
the renderer compiled to WebAssembly and runs it with wasmtime; it needs no Rust toolchain, no
browser and no network.

- **Jupyter:** `chartlet.Chart(spec)` shows the chart's HTML figure in a notebook.
- **Sphinx:** `extensions = ["chartlet.sphinx"]` adds the `chartlet` directive, for
  reStructuredText and MyST Markdown.
- **MkDocs:** `markdown_extensions: [chartlet]` renders fenced `chartlet` blocks.

Documentation: [Python](https://casoon.github.io/chartlet/docs/guides/python/) and
[Markdown and documentation sites](https://casoon.github.io/chartlet/docs/guides/markdown/).

## License

MIT.
