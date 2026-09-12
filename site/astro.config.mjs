// @ts-check
import casoonPages from '@casoon/pages-theme';
import { defineConfig } from 'astro/config';

// Project page: https://casoon.github.io/chartlet/ — `base` is the GitHub Pages path.
export default defineConfig({
  site: 'https://casoon.github.io/chartlet',
  base: '/chartlet/',
  integrations: [
    casoonPages({
      name: 'chartlet',
      description:
        'Compiles a small JSON specification into a deterministic, accessible SVG or HTML chart at build time.',
      repo: 'casoon/chartlet',
      version: '0.1.0-alpha.3',
      license: 'MIT',
      packages: [
        { label: 'crates.io', href: 'https://crates.io/crates/chartlet' },
        { label: 'npm', href: 'https://www.npmjs.com/package/@casoon/chartlet' },
        { label: 'docs.rs', href: 'https://docs.rs/chartlet' },
      ],
      docsGroups: {
        'getting-started': 'Getting started',
        guides: 'Guides',
        reference: 'Reference',
      },
      // chartlet has no CHANGELOG.md yet.
      changelog: false,
    }),
  ],
});
