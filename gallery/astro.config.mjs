// @ts-check
import { defineConfig } from 'astro/config';

// The gallery is a project page: https://<user>.github.io/chartlet/.
// `site` keeps generated URLs absolute; `base` is the GitHub Pages path.
export default defineConfig({
  site: 'https://casoon.github.io/chartlet',
  base: '/chartlet/',
  output: 'static',
  vite: {
    server: {
      // Allow importing the canonical example specs from the repository root
      // (../examples) during `astro dev`.
      fs: { allow: ['..'] },
    },
  },
});
