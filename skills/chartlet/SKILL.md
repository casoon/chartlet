---
name: chartlet
description: Make accessible static charts and software diagrams (bar, line, time series, scatter, heatmap, flow, sequence, state, architecture, class, tree and more) as SVG or HTML with chartlet. Use when the user asks for a chart, plot, graph, diagram, flow chart, architecture or sequence diagram, or to redraw an existing diagram, and wants a file they can embed. Not for interactive dashboards.
---

# chartlet

chartlet compiles a JSON specification into static SVG or HTML with a text alternative and a data
table. You draft the specification; chartlet validates and draws it deterministically. The skill
only tells you the route; the specification itself is documented by the schema and the type
guide, which you read instead of guessing field names.

## The four parts

| Part | What it is | When you need it |
| --- | --- | --- |
| This skill | Instructions: the route from request to artifact | Always, to know what to do |
| MCP server `@casoon/chartlet-mcp` | Tools: inspect data, starters, validate, render, explain, check coverage, theme from brand | Preferred when the host has it |
| MCP App view | The rendered figure shown inside the chat | Optional; hosts without MCP Apps just return text |
| Compiler (`chartlet` CLI, `@casoon/chartlet`) | The renderer both of the above call | Fallback when there is no MCP server |

Check which tools you have: if `chartlet_validate_spec` is available, use the MCP route;
otherwise the CLI route in [references/cli.md](references/cli.md). Installation per host:
[references/hosts.md](references/hosts.md).

## Route

1. **Understand the request.** Read the data (a file the user names, pasted values) or the
   description. With MCP: `chartlet_inspect_data` on CSV or rows; it suggests types with columns
   and a reason. Read the resource `chartlet://guide/types` (MCP) or `docs/guides/chart-types.md`
   to choose the type; the schema is `chartlet://schema` or `schema/chartlet.schema.json`.
2. **Decide whether to draw.** A short list is a table, one number is a sentence. Say so instead
   of drawing.
3. **State your plan in one message**: type, size, what is shown and what is not. Proceed on
   routine choices (size, colors, label wording) with the stated default. Ask only when data are
   missing or two readings change the meaning.
4. **Start from a starter**: `chartlet_diagram_starter` (sequence, flow, state, architecture,
   tree, classes) or `chartlet_chart_starter` (waterfall, heatmap, sankey, scatter and other
   block charts); table charts are drafted from the data. Fill in the real values.
5. **Validate** (`chartlet_validate_spec`, or `chartlet render ... --diagnostics json`) until
   `ok`. Fix the field the error `path` points to. Treat warnings as findings: `text_truncated`,
   `label_overlap`, `dense_chart`, `diagram_over_budget` mean the reader loses something.
6. **Render** (`chartlet_render` with `outputPath` when the server has a root, else the CLI
   with `-o`). Add a `mobile` block for narrow screens; use the `print` variant for PDF.
7. **Deliver**: the editable specification (JSON), the rendered file, the warnings that remain,
   and one line of what the chart shows. With `chartlet_explain` you get computed facts (first,
   last, min, max, missing) to quote; state conclusions as your own.
8. **Corrections**: change the specification, never the output. Re-validate and re-render; say
   what changed.

## Never

- Aggregate, round, drop rows, invent data or remove relationships to make something fit or to
  silence a warning. Propose the cut and let the user decide.
- Redraw an existing diagram silently: after a redraw call `chartlet_check_coverage` with the
  inventory you read (say it is your extraction) and show the report: kept, changed, missing.
- Claim a font or brand match: chartlet has no font setting; `chartlet_theme_from_brand` records
  fonts as not applied.
- Fetch websites through chartlet; read the brand values yourself, then pass them in.

## Look of the project

A `chartlet.theme.json` in the working directory (CLI) or `CHARTLET_MCP_ROOT` (MCP) applies to
every render. To create one from brand colors: read `paper`, `ink`, `muted`, `accent` (and `link`)
from the site or style guide, call `chartlet_theme_from_brand` with `write: true`, read the receipt
(adjusted values and why) and show it to the user, then render a chart to check.

## References

- [references/cli.md](references/cli.md): the CLI route and its prerequisites.
- [references/hosts.md](references/hosts.md): install, update, uninstall, troubleshooting for
  Claude Code and Codex, and what has and has not been verified.
- Documentation: <https://casoon.github.io/chartlet/docs/>
