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
 * Renders a chart for the showcase.
 *
 * A light chart is wrapped in a light surface because chartlet's light palette assumes one and
 * the site's own theme may be dark. A chart with `"theme": "dark"` paints its own dark
 * background and is left as it is.
 */
export function render(
  source: Spec,
  id: string,
  format: 'svg' | 'html' = 'html',
): { html: string; warnings: string[] } {
  const { content, warnings } = renderChart(source, { format, table: 'details', idPrefix: id });
  const dark = source.theme === 'dark';
  return {
    html: dark
      ? content
      : `<div style="background:#fff;color:#172033;color-scheme:light;padding:16px;border-radius:10px">${content}</div>`,
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
    slug: 'ticket-backlog',
    chart: 'Bar (horizontal)',
    useCase: 'Compare against a target',
    blurb: 'Open tickets per region against a target line, with thousands separated.',
  },
  {
    slug: 'energy-mix',
    chart: 'Bar (stacked)',
    useCase: 'Show a total and its parts',
    blurb: 'Electricity generation by source, stacked by value, with the total above each bar.',
  },
  {
    slug: 'audit-outcomes',
    chart: 'Bar (100 % stacked)',
    useCase: 'Compare shares',
    blurb: 'Outcomes of accessibility checks per page as shares of all checks.',
  },
  {
    slug: 'population-and-emissions',
    chart: 'Bar (grouped, outlined series)',
    useCase: 'Compare two shares per category',
    blurb: 'Two series told apart by fill and outline as well as by color.',
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
    slug: 'visitors-by-channel',
    chart: 'Line (several series)',
    useCase: 'Compare trends',
    blurb: 'Three channels over six weeks, told apart by color and line pattern, one value missing.',
  },
  {
    slug: 'resting-heart-rate',
    chart: 'Time series (points and line)',
    useCase: 'Show readings around a trend',
    blurb: 'Morning readings as dots, scattered around the line of their weekly mean.',
  },
  {
    slug: 'quake-frequency',
    chart: 'Line (logarithmic axis)',
    useCase: 'Show values across orders of magnitude',
    blurb: 'Earthquakes per year by magnitude, from thousands to one, on a logarithmic axis.',
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
  {
    slug: 'daily-orders',
    chart: 'Time series',
    useCase: 'Plot against a real time axis',
    blurb:
      'Orders per day on a calendar axis: the ticks sit on day boundaries and the weekend dips show up as local minima.',
  },
  {
    slug: 'revenue-vs-forecast',
    chart: 'Time series (dark)',
    useCase: 'Compare two series over time',
    blurb:
      'Weekly revenue against its forecast on a dark theme, with both layers naming their own color — one as a hex value, one as a CSS variable the host page owns.',
  },
  {
    slug: 'temperature-projection',
    chart: 'Time series with uncertainty',
    useCase: 'Separate measurement from projection',
    blurb:
      'A measured line and two modeled scenarios, each with its uncertainty band. Modeled layers are dashed and hatched and say so in legend, description and table, so the difference never rests on color alone.',
  },
  {
    slug: 'annual-mean-threshold',
    chart: 'Time series with reference line',
    useCase: 'Measure against a threshold',
    blurb:
      'Single years as a thin line under their 20-year mean, with a horizontal threshold and a vertical date marker. Illustrative values.',
  },
  {
    slug: 'sensor-readings',
    chart: 'Time series with area and gaps',
    useCase: 'Show several measures and their outages',
    blurb:
      'Five layers on one axis: a total filled down to zero, a bold, a dotted and a dashed line, and one layer in its own color beside the four palette colors. Missing readings break the lines and the area, and two zoom steps switch between the whole month and its last week. Illustrative values.',
  },
  {
    slug: 'release-incidents',
    chart: 'Time series with zones and markers',
    useCase: 'Mark events and ranges on a timeline',
    blurb:
      'A daily error rate with a shaded range of values, a maintenance window and three point markers in different shapes. Every zone and marker carries its own label, so the meaning never rests on color alone. Illustrative values.',
  },
  {
    slug: 'share-price',
    chart: 'Candlesticks with stacked panes',
    useCase: 'Show prices and trading volume on one time axis',
    blurb:
      'Daily candles with a 20-day average and a marker for quarterly results, above a volume pane filled down to zero. Both panes share one time axis, on which weekends and holidays take no space, and have their own value axis; a rising candle is hollow and a falling one filled, so the direction never rests on color alone. Illustrative values for a fictional company.',
  },
  {
    slug: 'warming-stripes',
    chart: 'Stripes',
    useCase: 'Show a long trend at a glance',
    blurb:
      'One stripe per year on a diverging scale around a reference value. Illustrative values; the data table carries every year.',
  },
  {
    slug: 'daily-anomaly-calendar',
    chart: 'Calendar heatmap',
    useCase: 'Find patterns across a year',
    blurb:
      'One cell per day of a year, colored on the same diverging scale as the stripes, with a key for minimum, reference and maximum. Illustrative values.',
  },
  {
    slug: 'warming-contributions',
    chart: 'Range bars',
    useCase: 'Compare estimates with their uncertainty',
    blurb:
      'Each factor as a span from low to high with its central estimate; modeled spans are hatched. The scale does not force zero, because a span is not a bar.',
  },
  {
    slug: 'emission-pathways',
    chart: 'Small multiples',
    useCase: 'Compare the same measure across groups',
    blurb:
      'One small time chart per sector on a shared value axis, with one legend for all panels. Illustrative values.',
  },
  {
    slug: 'deep-sea-isotopes',
    chart: 'Time series (numeric axis)',
    useCase: 'Show deep time or a profile',
    blurb: 'Oxygen isotopes over 66 million years, oldest on the left, warmer up.',
  },
  {
    slug: 'acceleration-indicators',
    chart: 'Small multiples (own axes)',
    useCase: 'Compare shapes across units',
    blurb: 'Four indicators in different units, each panel on its own value axis.',
  },
  {
    slug: 'mobile-revenue',
    chart: 'Mobile variant',
    useCase: 'Stay readable on a phone',
    blurb:
      'Three sales channels over eighteen months, laid out twice: at 800 × 450 and at 360 × 420 for containers narrower than 640 pixels, where the legend wraps into two rows. A container query switches between them; caption, source and data table appear once. The output panel on this page is narrower than 640 pixels at every window size, so it shows the mobile variant. Illustrative values.',
  },
  {
    slug: 'topicmap-sample',
    chart: 'Topic map',
    useCase: 'Compare how much there is of each subject',
    blurb:
      'Published entries per subject, drawn as a map: the area of each landmass is its share of the whole, the points are the paths through it, and the dashed route joins the two subjects that overlap most. Pick an area to bring it forward — radio buttons and CSS, no script.',
  },
  {
    slug: 'knowledge-landscape',
    chart: 'Knowledge landscape',
    useCase: 'Show how subjects lie next to each other',
    blurb:
      'Documentation as one continuous land: realms of neighbouring regions, sized by how much they hold but damped so the largest does not swallow the map, with the terrain rising where the writing is densest and every marked place a page worth starting from. Where a region lies says as much as how large it is — kinship across realms leaves a subject on the border facing its kin.',
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
