# @casoon/chartlet

Build static, accessible SVG or HTML charts during an Astro build. The component runs only on
the server and ships no chart JavaScript to the browser.

The current alpha adapter calls the `chartlet` CLI. Install the CLI and make it available on
`PATH`, or set `CHARTLET_BIN` to its absolute path.

Neither the package nor the CLI is published to a registry yet. Install the CLI from the release
tag, and the package from a local clone of the repository:

```sh
cargo install --git https://github.com/casoon/chartlet --tag v0.1.0-alpha.2
git clone --branch v0.1.0-alpha.2 https://github.com/casoon/chartlet.git
npm install ./chartlet/packages/chartlet
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
