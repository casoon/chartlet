# @casoon/chartlet

Build static, accessible SVG or HTML charts during an Astro build. The component runs only on
the server and ships no chart JavaScript to the browser.

The current alpha adapter calls the `chartlet` CLI. Install the CLI and make it available on
`PATH`, or set `CHARTLET_BIN` to its absolute path.

```sh
npm install @casoon/chartlet
cargo install chartlet --version 0.1.0-alpha.1
```

```astro
---
import Chart from '@casoon/chartlet/astro';
import chart from '../data/monthly-revenue.json';
---

<Chart id="monthly-revenue" spec={chart} />
```

`id` must be stable and unique within the page. It prevents accessibility ID collisions when
the same specification is embedded more than once.

Use `format="svg"` for the graphic alone or `format="html"` for the default figure, caption,
source, and data-table output. The HTML table uses a native disclosure by default; set
`table="visible"` to show it permanently.
