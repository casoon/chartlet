import { renderChart } from '@casoon/chartlet';
import type { ShowcaseExample } from '@casoon/pages-theme/showcase';

type Spec = Record<string, unknown>;

// The canonical specifications in examples/ are also the golden-file test inputs, so every
// chart on this site has exactly one source of truth. They are rendered here at build time
// with @casoon/chartlet, the same renderer as the CLI and the crate.
const specs = import.meta.glob<Spec>('../../examples/*.json', { import: 'default', eager: true });

export function spec(slug: string): Spec {
  const found = specs[`../../examples/${slug}.json`];
  if (!found) throw new Error(`Unknown example: ${slug}`);
  return found;
}

/**
 * Renders a chart and wraps it in a light surface: chartlet ships a light palette only
 * (dark mode is planned, see the support matrix), so the chart keeps its contrast in the
 * site's dark theme.
 */
export function render(
  source: Spec,
  id: string,
  format: 'svg' | 'html' = 'html',
): { html: string; warnings: string[] } {
  const { content, warnings } = renderChart(source, { format, table: 'details', idPrefix: id });
  return {
    html: `<div style="background:#fff;color:#172033;color-scheme:light;padding:16px;border-radius:10px">${content}</div>`,
    warnings: warnings.map((warning: string) => warning.trim()),
  };
}

const catalogue = [
  {
    slug: 'monthly-revenue',
    chart: 'Bar (vertical)',
    useCase: 'Compare categories',
    blurb: 'A single series of revenue values, one bar per month.',
  },
  {
    slug: 'operating-costs',
    chart: 'Bar (vertical)',
    useCase: 'Track a cost trend',
    blurb: 'Operating costs over half a year with a value axis title.',
  },
  {
    slug: 'quarterly-change',
    chart: 'Bar (horizontal)',
    useCase: 'Show change and deviation',
    blurb: 'Positive and negative change, formatted as percent, with a deliberately long label.',
  },
  {
    slug: 'signup-conversion',
    chart: 'Bar (horizontal)',
    useCase: 'Compare rates',
    blurb: 'Signup conversion per landing page, sorted from lowest to highest.',
  },
  {
    slug: 'budget-vs-actual',
    chart: 'Grouped bar',
    useCase: 'Plan versus actual',
    blurb:
      'Budget and actual costs side by side, including a missing value. Use the checkboxes to hide a series, or hover a bar for its value.',
  },
  {
    slug: 'revenue-by-channel',
    chart: 'Grouped bar',
    useCase: 'Break down by segment',
    blurb: 'Revenue split across three channels over four quarters.',
  },
  {
    slug: 'csat-by-region',
    chart: 'Grouped bar (horizontal)',
    useCase: 'Compare across groups',
    blurb: 'Customer satisfaction scores for two years, four regions.',
  },
  {
    slug: 'monthly-trend',
    chart: 'Line',
    useCase: 'Show a trend',
    blurb: 'Monthly active users, with a gap where a month has no data.',
  },
  {
    slug: 'support-volume',
    chart: 'Line',
    useCase: 'Track a metric',
    blurb: 'Weekly support tickets with a mid-period dip.',
  },
  {
    slug: 'headcount',
    chart: 'Line',
    useCase: 'Show growth',
    blurb:
      'Steady headcount growth from January to August. Switch between the full period and each half with the pre-rendered zoom steps.',
  },
];

export const examples: ShowcaseExample[] = catalogue.map(({ slug, chart, useCase, blurb }) => {
  const source = spec(slug);
  const { html, warnings } = render(source, `showcase-${slug}`);
  return {
    slug,
    title: String(source.title),
    description:
      warnings.length > 0 ? `${blurb} Build-time warnings: ${warnings.join(' · ')}` : blurb,
    file: `examples/${slug}.json`,
    tags: [chart, useCase],
    input: { code: JSON.stringify(source, null, 2), lang: 'json' },
    output: { html, kind: 'panel' },
  };
});
