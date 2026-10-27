import { renderChart } from '@casoon/chartlet';
import type { ShowcaseExample } from '@casoon/pages-theme/showcase';
import qrLink from '../../examples/qr/link.svg?raw';

type Spec = Record<string, unknown>;

// The canonical specifications in examples/ are also the golden-file test inputs, so every
// chart on this site has exactly one source of truth. They are rendered here at build time
// with @casoon/chartlet, the same renderer as the CLI and the crate.
const specs = import.meta.glob<Spec>('../../examples/*.json', { import: 'default', eager: true });

/** The number of examples, which are also the golden-file test inputs. */
export const exampleCount = Object.keys(specs).length;

export function spec(slug: string): Spec {
  const found = specs[`../../examples/${slug}.json`];
  if (!found) throw new Error(`Unknown example: ${slug}`);
  return found;
}

/**
 * Every variable of chartlet's dark theme, for a light chart shown on a dark page — read from
 * chartlet's own dark output rather than copied, so new variables arrive with the renderer.
 * Each chart family declares its own: the base palette and the node and role colors of diagrams.
 * Copying only the base palette left diagram labels light on light nodes.
 */
const DARK_VARS = darkVariables(['monthly-revenue', 'order-flow']);

function darkVariables(references: string[]): string {
  const variables = new Map<string, string>();
  for (const slug of references) {
    const { content } = renderChart({ ...spec(slug), theme: 'dark' }, { format: 'svg' });
    for (const rule of content.matchAll(/\.chartlet-theme-dark\{([^}]*)\}/g)) {
      for (const declaration of rule[1].split(';')) {
        const colon = declaration.indexOf(':');
        const name = declaration.slice(0, colon).trim();
        if (name.startsWith('--chartlet-')) variables.set(name, declaration.slice(colon + 1).trim());
      }
    }
  }
  for (const required of ['--chartlet-text', '--chartlet-node', '--chartlet-role-blue-fill']) {
    if (!variables.has(required)) throw new Error(`chartlet's dark theme no longer declares ${required}`);
  }
  return [...variables].map(([name, value]) => `${name}:${value}`).join(';');
}

/**
 * The surface a light chart sits on: light by default, and dark with the dark palette where the
 * site's theme is dark, by the toggle (`data-theme`) or by the system setting.
 */
const SURFACE_STYLE = `<style>.cl-surface{background:#fff;color:#172033;color-scheme:light;padding:16px;border-radius:10px}:root[data-theme=dark] .cl-surface{background:#0e131c;color:#e8edf6;color-scheme:dark}:root[data-theme=dark] .cl-surface .chartlet-root{${DARK_VARS}}@media (prefers-color-scheme:dark){:root:not([data-theme=light]) .cl-surface{background:#0e131c;color:#e8edf6;color-scheme:dark}:root:not([data-theme=light]) .cl-surface .chartlet-root{${DARK_VARS}}}</style>`;

/**
 * Renders a chart for the showcase.
 *
 * A light chart sits on a surface that follows the site's theme, with chartlet's dark palette on
 * a dark site. A chart with `"theme": "dark"` paints its own dark background and is left as it is.
 * `view` picks the view of an architecture model with `views`; without it, its default view.
 * `variant` picks the layout variant, e.g. `mobile` for a specification with a `mobile` block.
 */
export function render(
  source: Spec,
  id: string,
  format: 'svg' | 'html' = 'html',
  view?: string,
  variant?: 'desktop' | 'mobile',
): { html: string; warnings: string[] } {
  const { content, warnings } = renderChart(source, {
    format,
    table: 'details',
    idPrefix: id,
    view,
    variant,
  });
  const dark = source.theme === 'dark';
  return {
    html: dark ? content : `${SURFACE_STYLE}<div class="cl-surface">${content}</div>`,
    warnings: warnings.map((warning: string) => warning.trim()),
  };
}

const catalogue = [
  {
    slug: 'monthly-revenue',
    group: 'Bar charts',
    chart: 'Bar (vertical)',
    useCase: 'Compare categories',
    blurb: 'A single series of revenue values, one bar per month.',
  },
  {
    slug: 'quarterly-change',
    group: 'Bar charts',
    chart: 'Bar (horizontal)',
    useCase: 'Show change and deviation',
    blurb: 'Positive and negative change, formatted as percent, with a deliberately long label.',
  },
  {
    slug: 'ticket-backlog',
    group: 'Bar charts',
    chart: 'Bar (horizontal)',
    useCase: 'Compare against a target',
    blurb: 'Open tickets per region against a target line, with thousands separated.',
  },
  {
    slug: 'energy-mix',
    group: 'Bar charts',
    chart: 'Bar (stacked)',
    useCase: 'Show a total and its parts',
    blurb: 'Electricity generation by source, stacked by value, with the total above each bar.',
  },
  {
    slug: 'audit-outcomes',
    group: 'Bar charts',
    chart: 'Bar (100 % stacked)',
    useCase: 'Compare shares',
    blurb: 'Outcomes of accessibility checks per page as shares of all checks.',
  },
  {
    slug: 'budget-vs-actual',
    group: 'Bar charts',
    chart: 'Grouped bar',
    useCase: 'Plan versus actual',
    blurb:
      'Budget and actual costs side by side, including a missing value. Use the checkboxes to hide a series, or hover a bar for its value.',
  },
  {
    slug: 'population-and-emissions',
    group: 'Bar charts',
    chart: 'Bar (grouped, outlined series)',
    useCase: 'Compare two shares per category',
    blurb: 'Two series told apart by fill and outline as well as by color.',
  },
  {
    slug: 'benefit-and-harm',
    group: 'Bar charts',
    chart: 'Bar chart (groups)',
    useCase: 'Set two kinds of effect apart',
    blurb:
      'Five effects as horizontal bars, colored by group: benefit and harm, each with a legend entry. The groups are not series — every category has one bar, one value and one group, and the data table says which. Illustrative values.',
  },
  {
    slug: 'satisfaction-scores',
    group: 'Bar charts',
    chart: 'Bar chart with error bars',
    useCase: 'Show how sure an estimate is',
    blurb:
      'Satisfaction by plan with its 95 % confidence interval as an error bar through the end of each bar: Enterprise scores 4.0 but could be anywhere from 3.3 to 4.7. The interval is also in the tooltip and in an extra column of the table. Illustrative survey.',
  },
  {
    slug: 'satisfaction-by-quarter',
    group: 'Bar charts',
    chart: 'Grouped bars (error bars)',
    useCase: 'Compare groups with their uncertainty',
    blurb:
      'Satisfaction in four quarters for two plans: each bar of a group has its own 95% confidence interval, the scale reaches every limit, and the table gains an interval column for each plan. Illustrative survey, with a phone variant.',
  },
  {
    slug: 'monthly-trend',
    group: 'Line charts',
    chart: 'Line',
    useCase: 'Show a trend',
    blurb: 'Monthly active users, with a gap where a month has no data.',
  },
  {
    slug: 'visitors-by-channel',
    group: 'Line charts',
    chart: 'Line (several series)',
    useCase: 'Compare trends',
    blurb: 'Three channels over six weeks, told apart by color and line pattern, one value missing.',
  },
  {
    slug: 'quake-frequency',
    group: 'Line charts',
    chart: 'Line (logarithmic axis)',
    useCase: 'Show values across orders of magnitude',
    blurb: 'Earthquakes per year by magnitude, from thousands to one, on a logarithmic axis.',
  },
  {
    slug: 'monthly-satisfaction',
    group: 'Line charts',
    chart: 'Line chart (error bars)',
    useCase: 'Show a trend with its uncertainty',
    blurb:
      'Mean satisfaction by month as a line with a 95% confidence interval through every point: the scale reaches every limit, the value label sits above the bar, and the tooltip and the table give the interval. Illustrative survey, with a phone variant.',
  },
  {
    slug: 'headcount',
    group: 'Line charts',
    chart: 'Line',
    useCase: 'Show growth',
    blurb:
      'Steady headcount growth from January to August. Switch between the full period and each half with the pre-rendered zoom steps.',
  },
  {
    slug: 'daily-orders',
    group: 'Time series',
    chart: 'Time series',
    useCase: 'Plot against a real time axis',
    blurb:
      'Orders per day on a calendar axis: the ticks sit on day boundaries and the weekend dips show up as local minima.',
  },
  {
    slug: 'revenue-vs-forecast',
    group: 'Time series',
    chart: 'Time series (dark)',
    useCase: 'Compare two series over time',
    blurb:
      'Weekly revenue against its forecast on a dark theme, with both layers naming their own color — one as a hex value, one as a CSS variable the host page owns.',
  },
  {
    slug: 'temperature-projection',
    group: 'Time series',
    chart: 'Time series with uncertainty',
    useCase: 'Separate measurement from projection',
    blurb:
      'A measured line and two modeled scenarios, each with its uncertainty band. Modeled layers are dashed and hatched and say so in legend, description and table, so the difference never rests on color alone.',
  },
  {
    slug: 'annual-mean-threshold',
    group: 'Time series',
    chart: 'Time series with reference line',
    useCase: 'Measure against a threshold',
    blurb:
      'Single years as a thin line under their 20-year mean, with a horizontal threshold and a vertical date marker. Illustrative values.',
  },
  {
    slug: 'sensor-readings',
    group: 'Time series',
    chart: 'Time series with area and gaps',
    useCase: 'Show several measures and their outages',
    blurb:
      'Five layers on one axis: a total filled down to zero, a bold, a dotted and a dashed line, and one layer in its own color beside the four palette colors. Missing readings break the lines and the area, and two zoom steps switch between the whole month and its last week. Illustrative values.',
  },
  {
    slug: 'generation-mix',
    group: 'Time series',
    chart: 'Time series (stacked areas)',
    useCase: 'Show a total and its parts over time',
    blurb: 'Electricity by source, each area on top of the ones below, named where it ends.',
  },
  {
    slug: 'resting-heart-rate',
    group: 'Time series',
    chart: 'Time series (points and line)',
    useCase: 'Show readings around a trend',
    blurb: 'Morning readings as dots, scattered around the line of their weekly mean.',
  },
  {
    slug: 'release-incidents',
    group: 'Time series',
    chart: 'Time series with zones and markers',
    useCase: 'Mark events and ranges on a timeline',
    blurb:
      'A daily error rate with a shaded range of values, a maintenance window and three point markers in different shapes. Every zone and marker carries its own label, so the meaning never rests on color alone. Illustrative values.',
  },
  {
    slug: 'share-price',
    group: 'Time series',
    chart: 'Candlesticks with stacked panes',
    useCase: 'Show prices and trading volume on one time axis',
    blurb:
      'Daily candles with a 20-day average and a marker for quarterly results, above a volume pane filled down to zero. Both panes share one time axis, on which weekends and holidays take no space, and have their own value axis; a rising candle is hollow and a falling one filled, so the direction never rests on color alone. Illustrative values for a fictional company.',
  },
  {
    slug: 'deep-sea-isotopes',
    group: 'Time series',
    chart: 'Time series (numeric axis)',
    useCase: 'Show deep time or a profile',
    blurb: 'Oxygen isotopes over 66 million years, oldest on the left, warmer up.',
  },
  {
    slug: 'emission-pathways',
    group: 'Small multiples',
    chart: 'Small multiples',
    useCase: 'Compare the same measure across groups',
    blurb:
      'One small time chart per sector on a shared value axis, with one legend for all panels. Illustrative values.',
  },
  {
    slug: 'acceleration-indicators',
    group: 'Small multiples',
    chart: 'Small multiples (own axes)',
    useCase: 'Compare shapes across units',
    blurb: 'Four indicators in different units, each panel on its own value axis.',
  },
  {
    slug: 'framework-benchmarks',
    group: 'Small multiples',
    chart: 'Small multiples of bars (phone variant)',
    useCase: 'Compare candidates on unlike measures',
    blurb:
      'Four frameworks on three measures that share no unit: requests per second, memory and cold start on a logarithmic axis. One panel per measure, each with its own value axis, unit and verdict, over the same categories. On a phone the panels stack in one column. Illustrative values.',
  },
  {
    slug: 'warming-causes',
    group: 'Small multiples',
    chart: 'Small multiples (panel notes)',
    useCase: 'Compare candidates against one measurement',
    blurb: 'Measured warming beside three model runs, each panel with its verdict under the title — the one that matches set apart — and one column on phones.',
  },
  {
    slug: 'warming-stripes',
    group: 'Stripes, calendars and timelines',
    chart: 'Stripes',
    useCase: 'Show a long trend at a glance',
    blurb:
      'One stripe per year on a diverging scale around a reference value. Illustrative values; the data table carries every year.',
  },
  {
    slug: 'daily-anomaly-calendar',
    group: 'Stripes, calendars and timelines',
    chart: 'Calendar heatmap',
    useCase: 'Find patterns across a year',
    blurb:
      'One cell per day of a year, colored on the same diverging scale as the stripes, with a key for minimum, reference and maximum. Illustrative values.',
  },
  {
    slug: 'product-roadmap',
    group: 'Stripes, calendars and timelines',
    chart: 'Timeline',
    useCase: 'Plan phases, milestones and dependencies',
    blurb:
      'Seven items over nine months in three groups: phases as bars, milestones as diamonds, a dashed marker for today and arrows from each item to the ones that follow it. The description and the table list every item with its days and what it follows; on a phone the dates move into the tooltips. Illustrative plan.',
  },
  {
    slug: 'warming-contributions',
    group: 'Ranges and distributions',
    chart: 'Range bars',
    useCase: 'Compare estimates with their uncertainty',
    blurb:
      'Each factor as a span from low to high with its central estimate; modeled spans are hatched. The scale does not force zero, because a span is not a bar.',
  },
  {
    slug: 'soil-animals',
    group: 'Ranges and distributions',
    chart: 'Range bars (groups)',
    useCase: 'Compare spans across orders of magnitude',
    blurb:
      'How many soil animals live under a square metre, as spans on a logarithmic axis, colored by size class with a legend.',
  },
  {
    slug: 'trial-effects',
    group: 'Ranges and distributions',
    chart: 'Forest plot',
    useCase: 'Pool the results of several studies',
    blurb:
      'Five studies and their pooled result: each effect with its confidence interval as a thin line, a square on the effect whose area follows the weight of the study, a diamond for the overall result and a dashed line at no effect. Effect and interval are written beside every span, the weights are in the table. Illustrative meta-analysis, with a phone variant.',
  },
  {
    slug: 'response-times',
    group: 'Ranges and distributions',
    chart: 'Box plot',
    useCase: 'Compare distributions, not averages',
    blurb:
      'Four endpoints side by side: three boxes computed from their measurements (quartiles, whiskers to the last value within 1.5 boxes, a point for the stray 140 ms) and one drawn from the five numbers of a report that was computed elsewhere. The table has every box with its outliers. Illustrative measurements, with a phone variant.',
  },
  {
    slug: 'latency-by-region',
    group: 'Ranges and distributions',
    chart: 'Box plot (groups)',
    useCase: 'Compare groups across categories',
    blurb:
      'Response times of three endpoints measured in two regions: the boxes of a region share a color and a legend entry, so the two regions read side by side, and a diamond marks each mean. The table gains a Group column. Illustrative measurements, with a phone variant.',
  },
  {
    slug: 'satisfaction-change',
    group: 'Ranges and distributions',
    chart: 'Dumbbell chart',
    useCase: 'Compare two values per category, such as before and after',
    blurb:
      'Satisfaction scores of six support areas before and after a redesign: an open dot for the score before, a filled dot for the score after, a line between them, both values written beside the dots, and the area with the widest change first. The table lists both values per area. Illustrative survey scores, with a phone variant.',
  },
  {
    slug: 'response-distribution',
    group: 'Ranges and distributions',
    chart: 'Violin plot',
    useCase: 'Show the shape of a distribution',
    blurb:
      'The same kind of data as the box plot, drawn as violins: three endpoints with sixty response times each, where the outline shows what a box hides — a long tail for search, two peaks for checkout — with the quartile box and the median inside. A strip shows every observation instead. Illustrative measurements, with a phone variant.',
  },
  {
    slug: 'response-swarm',
    group: 'Ranges and distributions',
    chart: 'Beeswarm plot',
    useCase: 'Show every observation without hiding any behind another',
    blurb:
      'The same three endpoints as the violins, now every one of the sixty requests as a dot, moved sideways only as far as it must so that no dot overlaps another, with the box and the median under them. The width of the swarm shows where the requests pile up. Illustrative measurements, with a phone variant.',
  },
  {
    slug: 'response-ridges',
    group: 'Ranges and distributions',
    chart: 'Ridgeline plot',
    useCase: 'Compare the shapes of many distributions on one axis',
    blurb:
      'Five services, one density curve each on a shared millisecond axis, stacked with a slight overlap: a single peak for search, two for checkout, a long tail for uploads. All curves share one scale, so their heights compare, and a line up to each curve marks its median. Illustrative measurements, with a phone variant.',
  },
  {
    slug: 'topic-streams',
    group: 'Time series',
    chart: 'Streamgraph',
    useCase: 'Show how the mix of a total shifts over time',
    blurb:
      'Four support topics stacked around a moving baseline instead of zero, so the shape of the whole reads as a stream: each band is named at its right end, and a scale bar stands in for the value axis because only thickness carries the value. The table lists each topic per fortnight. Illustrative figures, with a phone variant.',
  },
  {
    slug: 'gene-expression',
    group: 'Scatter plots and survival',
    chart: 'Volcano plot',
    useCase: 'Find what stands out among many points',
    blurb:
      'Seven hundred genes by their change in expression and their significance: three groups in the palette colors, dashed lines at the cutoffs, and names on the four that stand out. The description counts the points per group and above each line; the table lists only the named points, so it stays short. Illustrative data, with a phone variant.',
  },
  {
    slug: 'genome-scan',
    group: 'Scatter plots and survival',
    chart: 'Manhattan plot',
    useCase: 'Find the hits along a genome',
    blurb:
      'Significance along twenty-two chromosomes: the points alternate between two colors by chromosome, the chromosome numbers stand at the middle of their stretch, the dashed line marks the genome-wide threshold, and the axes end exactly at the genome and at 10. The table lists only the labeled points, so it stays short. Illustrative data, with a phone variant.',
  },
  {
    slug: 'allometry',
    group: 'Scatter plots and survival',
    chart: 'Scatter plot (regression, log axes)',
    useCase: 'Fit a power law',
    blurb:
      'Metabolic rate against body mass for two groups of animals on logarithmic axes, with a least-squares line for each group: fitted to the logarithms, so a power law is a straight line. The description gives each line\'s slope, intercept and r². Illustrative data, with a phone variant.',
  },
  {
    slug: 'feature-priorities',
    group: 'Scatter plots and survival',
    chart: 'Scatter plot (quadrants)',
    useCase: 'Sort features by impact and effort',
    blurb:
      'Ten features placed by effort and impact, divided at 5 on both axes into quick wins, major projects, fill-ins and thankless tasks. Each area is named quietly in a corner that no point or name takes; the description lists the features of every area and the table gains a Quadrant column. Illustrative estimates, with a phone variant.',
  },
  {
    slug: 'overall-survival',
    group: 'Scatter plots and survival',
    chart: 'Kaplan-Meier curves',
    useCase: 'Compare survival between groups',
    blurb:
      'Two treatment arms of forty patients each as step curves, with a tick where a patient left the study, the 95% confidence bands and the number still at risk under the axis. The description gives every arm\'s events and median, the table the survival and the number at risk at every tick. Illustrative trial data, with a phone variant.',
  },
  {
    slug: 'revenue-to-profit',
    group: 'Ranges and distributions',
    chart: 'Waterfall chart',
    useCase: 'Explain how a total comes about',
    blurb:
      'From revenue to operating profit in eight steps: a start bar, costs that take the running total down, a license income that lifts it, and two totals. A dashed line carries the total from bar to bar; every change is written with its sign. The table gives the change and the total after each step. Illustrative figures, with a phone variant.',
  },
  {
    slug: 'energy-sources',
    group: 'Parts of a whole',
    chart: 'Waffle chart',
    useCase: 'Make a share countable',
    blurb:
      'A hundred squares for one year of electricity: wind, solar, gas and coal each take as many squares as percent, and the 12 squares that are left stand for the rest. Every square has a tooltip, the legend names value and share, and the table counts the squares. Illustrative shares, with a phone variant.',
  },
  {
    slug: 'regional-slope',
    group: 'Line charts',
    chart: 'Slope chart',
    useCase: 'Compare two moments, series by series',
    blurb:
      'Four regions between 2024 and 2025: one line each with its name and value at both ends, so that no legend is needed. Lines differ in pattern and weight as well as color, and the description names the biggest rise and the biggest fall. Illustrative figures, with a phone variant.',
  },
  {
    slug: 'support-rank',
    group: 'Line charts',
    chart: 'Rank chart',
    useCase: 'Show who overtakes whom',
    blurb:
      'The rank of four support channels across four quarters, 1 at the top: lines cross where the order changes, and the names and ranks stand at both ends. The description lists every rank and names the largest rise and fall in rank. Illustrative counts, with a phone variant.',
  },
  {
    slug: 'weekly-response',
    group: 'Ranges and distributions',
    chart: 'Heatmap',
    useCase: 'Find the hours that stand out',
    blurb:
      'Median first response of a support team by weekday and time of day: six blues from the quickest to the slowest, the minutes written into every cell, and a dashed box where no tickets were answered. The key names the unit and both ends of the scale, and the table has a row for every cell. Illustrative data, with a phone variant.',
  },
  {
    slug: 'permission-matrix',
    group: 'Ranges and distributions',
    chart: 'Heatmap (categories)',
    useCase: 'Show who may do what',
    blurb:
      'Four roles against four areas of a project: read, write and none, each with its own fill and symbol so that color is never the only cue, and a dashed box for the combination nobody defined. Illustrative roles, with a phone variant.',
  },
  {
    slug: 'election-result',
    group: 'Parts of a whole',
    chart: 'Parliament chart',
    useCase: 'Show who holds the seats',
    blurb:
      'Two hundred seats in six parties, in blocks from left to right, with a dashed line at the 101st seat and a ring round the seats of a four-party coalition. The legend, the description and the table give each party with its seats and share and say whether the coalition has a majority. Illustrative result, with a phone variant.',
  },
  {
    slug: 'budget-by-department',
    group: 'Parts of a whole',
    chart: 'Treemap',
    useCase: 'Show many parts of a whole',
    blurb:
      'A budget of thirteen items in four groups: the area of each rectangle follows its amount, the colors name the group, and the names and amounts stand inside. The description names the biggest items and the shares of the groups, the table lists every item with value, share and group. Illustrative amounts, with a phone variant.',
  },
  {
    slug: 'budget-by-sector',
    group: 'Parts of a whole',
    chart: 'Treemap (nested)',
    useCase: 'Show parts inside parts',
    blurb:
      'The same kind of budget, one level deeper: four sectors drawn as frames, and the departments inside them as rectangles by their amount. Every sector keeps one color, the legend names the sectors, and the table lists every department with its path. Illustrative amounts, with a phone variant.',
  },
  {
    slug: 'energy-flow',
    group: 'Flows',
    chart: 'Sankey diagram',
    useCase: 'Follow a quantity from source to use',
    blurb:
      'Energy from five sources through power plants and heating to households, industry and transport, with the losses on the way: every band is as thick as its flow and takes the color of where it comes from. The description names the biggest links, the table lists every link with value and share. Illustrative flows, with a phone variant.',
  },
  {
    slug: 'water-cycle',
    group: 'Flows',
    chart: 'Sankey diagram (flow back)',
    useCase: 'Show a cycle in a flow',
    blurb:
      'Water from intake through treatment to households and industry, and on to wastewater, with part of it recycled back to treatment: the flow back is marked on its link and drawn as a loop under the diagram, as thick as its flow. Illustrative flows.',
  },
  {
    slug: 'voter-movement',
    group: 'Flows',
    chart: 'Sankey diagram (fixed columns)',
    useCase: 'Follow voters between two elections',
    blurb:
      'Where the voters of three parties went between two elections: the nodes are pinned to two columns in a listed order, and each party keeps its color on both sides, so a band that changes color is a voter who changed party. Illustrative survey, with a phone variant.',
  },
  {
    slug: 'cache-lookup',
    group: 'Sequence and state diagrams',
    chart: 'Sequence diagram',
    useCase: 'Explain a request flow',
    blurb:
      'Five participants, each kind in its own shape, and ten numbered messages: calls with filled heads activate their receiver until it replies with a dashed line, asynchronous messages have open heads, and an alt fragment frames the cached and the uncached branch. The description and the data table list every message in order.',
  },
  {
    slug: 'async-export',
    group: 'Sequence and state diagrams',
    chart: 'Sequence diagram (landscape)',
    useCase: 'Fit a flow into a wide format',
    blurb:
      'The same notation turned on its side: participants one below the other, time running right, for a wide, low place on the page. With the default orientation, auto, a diagram takes this form by itself wherever only it fits the canvas.',
  },
  {
    slug: 'sign-in-sequence',
    group: 'Sequence and state diagrams',
    chart: 'Sequence diagram with a fragment and a phone variant',
    useCase: 'Explain a sign-in across services',
    blurb:
      'A browser, a backend for frontend, an identity provider, an API and a database: fifteen numbered messages through an OIDC sign-in, a self-call that creates the session, and an `opt` fragment around the asynchronous back-channel logout. The description lists the messages in order and names the fragment with the messages it spans. Illustrative reference architecture, with a phone variant.',
  },
  {
    slug: 'ticket-states',
    group: 'Sequence and state diagrams',
    chart: 'State diagram',
    useCase: 'Explain a lifecycle',
    blurb:
      'The states of a support ticket: a dot marks where it starts, a diamond the choice by severity, double outlines the two ways it ends. Transitions read event [guard] / action, a reminder loops on its state, and reopening runs back against the flow. The table lists every transition with its parts in columns of their own.',
  },
  {
    slug: 'release-flow',
    group: 'Flow charts',
    chart: 'Flow chart (lanes)',
    useCase: 'Explain a process across teams',
    blurb:
      'Ten steps in three lanes, laid out by chartlet from nothing but the connections: the main path runs straight in the accent color, two loops back — a retry and a request for changes — are drawn against the flow, and a group frames the quality gates. The description and the table follow the steps in reading order.',
  },
  {
    slug: 'order-flow',
    group: 'Flow charts',
    chart: 'Flow chart (landscape)',
    useCase: 'Fit a process into a wide format',
    blurb:
      'Without lanes, on a wide and low canvas: auto orientation turns the flow to run right. Each kind of step has its own shape — start and end, decisions, input and output, a subprocess and a data store.',
  },
  {
    slug: 'contract-review',
    group: 'Flow charts',
    chart: 'Flow chart with port sides',
    useCase: 'Keep loops back apart from the way forward',
    blurb:
      'A contract from request to signature with two loops back: a failed legal review returns to the draft, failed negotiations to the terms. `fromSide` and `toSide` send the first loop down the right-hand side and the second down the left, so they run straight past the steps between instead of crossing the main path; the signed contract leaves to the archive on the left. The sides hold in either orientation and on the phone variant. The main path is given by its ends. Illustrative values.',
  },
  {
    slug: 'order-fulfilment',
    group: 'Flow charts',
    chart: 'Flow chart with shared trunks',
    useCase: 'Show parallel work without a tangle',
    blurb:
      'An order split into four parallel checks in a group, which meet again at packing. With `"sharedTrunks": true` the three checks off the main path leave the split on one branching line and reach packing on one, with a dot at every branch; the payment check on the main path and the two branches of the decision keep their own lines. Every edge is still drawn end to end with its tooltip; the description and the table are unchanged. Illustrative values.',
  },
  {
    slug: 'claim-handling',
    group: 'Flow charts',
    chart: 'Flow chart with the shares of each step',
    useCase: 'Show what happens to the cases at each step',
    blurb:
      'Insurance claims through a first check, an assessment and an appeal. A stripe inside each box splits it into the claims settled, rejected or passed on; one legend names the parts, and the description and the table give every part with its count and share. The main path is given by its ends. Illustrative values.',
  },
  {
    slug: 'study-selection',
    group: 'Flow charts',
    chart: 'Flow chart (PRISMA)',
    useCase: 'Report how studies were selected',
    blurb:
      'The PRISMA 2020 flow of a systematic review as a flow chart: records from databases and registers, duplicates removed, screening, reports sought and assessed, and the studies included, with every exclusion in a dashed box beside the main path and the three phases as groups. Counts in the sublabels; the description and the table list every step and edge. Illustrative counts, with a phone variant.',
  },
  {
    slug: 'court-instances',
    group: 'Flow charts',
    chart: 'Flow chart (Instanzenzug)',
    useCase: 'Show a chain of appeals',
    blurb:
      'The path of a civil case through the German courts, from Amtsgericht and Landgericht by appeal to the Oberlandesgericht and by Revision to the Bundesgerichtshof, in German with the conditions on the edges. A preset on the flow chart, not a type of its own. Simplified, with a phone variant.',
  },
  {
    slug: 'shop-architecture',
    group: 'Architecture diagrams',
    chart: 'Architecture diagram',
    useCase: 'Show where the parts of a system run',
    blurb:
      'Ten components of a web shop in three nested boundaries — a cloud region, a private network, a data zone — with every kind in its own shape. Connections say what they do and, in brackets, how. Components outside a boundary stay out of its frame; the table names each component with its boundaries and connections.',
  },
  {
    slug: 'tenant-architecture',
    group: 'Architecture diagrams',
    chart: 'Architecture diagram with a main path',
    useCase: 'Show the path of a request through a system',
    blurb:
      'Eleven components of a multi-tenant school application in three nested boundaries: a data center, the application network and a data zone. The main path from the user to the domain data is drawn straight and thicker, connections say what they do and over which protocol, and the description names the most connected component and what nothing leads into. Illustrative reference architecture, with a phone variant.',
  },
  {
    slug: 'checkout-calls',
    group: 'Architecture diagrams',
    chart: 'Architecture diagram with numbered steps',
    useCase: 'Show the order of calls through a system',
    blurb:
      'Five connections of a checkout, numbered 1 to 5 in front of their labels. The description lists the steps in order with where each one goes, and a step number is used once. Laid out top down, so that the numbers read like a sequence. Illustrative values.',
  },
  {
    slug: 'shop-deployment',
    group: 'Architecture diagrams',
    chart: 'Deployment diagram in the blueprint look',
    useCase: 'Show what runs where',
    blurb:
      'A load balancer as a node, container images as artifacts on two nodes inside a cluster, and a volume. `node` is a box seen from a corner, `artifact` a box with a folded corner; the boundaries are the hosts. The blueprint theme draws pale lines on a deep blue ground. Illustrative values.',
  },
  {
    slug: 'request-volume',
    group: 'Architecture diagrams',
    chart: 'Architecture diagram with weighted connections',
    useCase: 'Show where the load goes',
    blurb:
      'Requests per second between the services of a shop. Every connection has a `weight`; its line is as much stronger as it carries more, from 1 px for nothing to 5 px for the heaviest, while the arrowheads keep their size. The description names the heaviest connection and the table gives every weight. The main path is given by its ends only, `{ "from": "web", "to": "pay" }`, and stays in the accent color. Illustrative values.',
  },
  {
    slug: 'data-tier',
    group: 'Architecture diagrams',
    chart: 'Architecture diagram with a shared rank (phone variant)',
    useCase: 'Line up what belongs to one tier',
    blurb:
      'Three services, each with its own database. Left to itself, every database follows its service, so they land at three depths and the Data tier boundary around them stretches over the queue and the worker. `"sameRank": [["catalogDb", "ordersDb", "billingDb"]]` puts the three at one position along the flow: the frame is one layer deep and the main path stays straight. A layout hint, not an edge; laid out again for the phone, where the tier is a row at the bottom. Illustrative values.',
  },
  {
    slug: 'service-fan-out',
    group: 'Architecture diagrams',
    chart: 'Architecture diagram with shared trunks',
    useCase: 'Draw a fan-out without a tangle',
    blurb:
      'A gateway that calls four services, each of which writes to one log store. With `"sharedTrunks": true` the calls to Catalog, Search and Cart leave the gateway on one line that branches, and the four dashed log writes reach the store on one, with a dot at every branch. Only connections that read the same share a line: the call to Orders lies on the main path and the token check has a label, so both keep their own. Every connection is still drawn end to end with its tooltip and listed in the description and the table. Illustrative values.',
  },
  {
    slug: 'shop-symbols',
    group: 'Architecture diagrams',
    chart: 'Architecture diagram with symbols',
    useCase: 'Make components recognizable at a glance',
    blurb:
      'The web shop on one cloud region with a generic `symbol` above the label of nine components — a browser, a content network, a container, a cache, a message broker, a function, object storage, identity and a cloud. The kind keeps its shape; each symbol is a plain stroke drawing, defined once per SVG, and the description and the table name it after the kind. Illustrative values.',
  },
  {
    slug: 'payment-interfaces',
    group: 'Architecture diagrams',
    chart: 'Architecture diagram with interfaces',
    useCase: 'Show which interface a call depends on',
    blurb:
      'A checkout and the interfaces it relies on. Orders, Payments and the card processor list what they `provides`, each drawn as a ball on a stem with its label; a connection with `toInterface` ends in a socket around that ball, so Support visibly requires the Refunds API rather than Payments as a whole. The description and the table say it in words — “requires Payments API”, a Provides column. With a phone variant. Illustrative values.',
  },
  {
    slug: 'payment-notes',
    group: 'Architecture diagrams',
    chart: 'Architecture diagram with notes',
    useCase: 'Write down what the boxes do not say',
    blurb:
      'Payments with three `notes`: one on the token vault, joined to it by a dotted line, one inside the card data zone at the head of its frame, and one without `on` for the whole diagram, in a band below it. Notes add no connection, so structure and main path stay the same; the description gives every note after the elements. With a phone variant. Illustrative values.',
  },
  {
    slug: 'billing-migration',
    group: 'Architecture diagrams',
    chart: 'Architecture diagram with roles',
    useCase: 'Show what changes in a migration',
    blurb:
      'Billing moves out of a monolith. The old path is `deprecated` (dotted, gray), the new service and its connections are `planned` (dashed, violet), the gateway, the accounts database and the payment call are `critical` (strong red). Each role also carries a badge or a line pattern, so it reads without color; the legend names only the roles used, and the description and the table name every role. Illustrative values.',
  },
  {
    slug: 'shop-views',
    group: 'Architecture models: views and levels',
    chart: 'Architecture views',
    useCase: 'Show one model at several levels of detail',
    blurb:
      'One model of a web shop with eleven components, shown through two `views`. The overview, shown here, keeps eight components with the connections between them and the boundaries that still hold one; the description says it shows 8 of the model\'s 11 components. The checkout view lists ten `steps` along existing connections, each with its own text, and renders as a numbered diagram. `--view checkout` renders it, `--all-views` both on one HTML page with links between them. Illustrative values.',
  },
  {
    slug: 'service-perspectives',
    group: 'Architecture models: views and levels',
    chart: 'Architecture diagram with perspectives',
    useCase: 'Show who owns what, or where the debt is',
    blurb:
      'A booking platform with two `perspectives`, the owning team and the technical debt as the teams rated it. Each view selects one; this one shows the teams: every box gains a stripe in its category\'s color and names the category, a legend lists the categories, and the guest and the payment provider, which belong to none, are shown as not stated. The description gives a sentence per category, the table a column. `--perspective debt` shows the other. Nothing is derived or rated by chartlet. Illustrative values.',
  },
  {
    slug: 'booking-context',
    group: 'Architecture models: views and levels',
    chart: 'System context',
    useCase: 'Show who uses a system and what it depends on',
    blurb:
      'A course booking system as one box, the learners and trainers who use it and the three external systems it calls — the `context` template of the MCP diagram starter. No boundary and nothing inside the system: the diagram answers who uses it and what it relies on, with the main path from the learner to the payment provider. Illustrative values.',
  },
  {
    slug: 'booking-containers',
    group: 'Architecture models: views and levels',
    chart: 'Containers',
    useCase: 'Show the deployable parts of a system',
    blurb:
      'The same course booking system one level down — the `containers` template: a boundary around its separately running parts, a web app, an API, a queue with its worker, a database and a file store, with the learner and the external systems outside. Sublabels say what each part is, connections how they talk. The description names the API and the queue as the bottlenecks. Illustrative values.',
  },
  {
    slug: 'booking-deployment',
    group: 'Architecture models: views and levels',
    chart: 'Deployment view (default view)',
    useCase: 'Document a deployment',
    blurb:
      'The `deployment` template: a logical model of the images, the managed queue, database and object storage, and a view of production — a cloud region holding a container cluster with two nodes, one API instance on each, and a node pool for the worker. `defaultView` makes the production view the one that renders; `--view logical` shows the model. Illustrative values.',
  },
  {
    slug: 'api-environments',
    group: 'Architecture models: views and levels',
    view: 'production',
    chart: 'Deployment view',
    useCase: 'Show where a model runs in each environment',
    blurb:
      'Four components of an orders API and three views of them: the components alone, a developer laptop, and production, shown here with `--view production`. A view with an `environment` nests `hosts` like boundaries — a region with two nodes and a managed database — and draws `instances` of components, labelled with their id, such as Orders API (api-1). Only the `instanceConnections` it lists are drawn, each with the label and technology of the model connection it names. The description names the environment, the table every instance. Illustrative values.',
  },
  {
    slug: 'order-model',
    group: 'Class and entity diagrams',
    chart: 'Class diagram',
    useCase: 'Document a domain model',
    blurb:
      'Seven classes of a shop: attributes and methods in their own compartments, an interface and an enumeration with their stereotypes, a composition with a filled diamond, a realization with a hollow triangle, dependencies dashed, and cardinalities at the ends. chartlet lays it out; the description and the table give every class with its members and relations.',
  },
  {
    slug: 'shop-schema',
    group: 'Class and entity diagrams',
    chart: 'Entity-relationship diagram',
    useCase: 'Show a database schema',
    blurb:
      'Four tables of a shop as entities with their keys, joined by relations with cardinalities: the same diagram type as the class diagram, without methods. The table lists every entity with its columns and relations.',
  },
  {
    slug: 'shop-modules',
    group: 'Class and entity diagrams',
    chart: 'Class diagram with packages',
    useCase: 'Show how a domain model is divided into modules',
    blurb:
      'Seven classes of a shop in four `packages`: Ordering and Catalog nested inside Shop, Billing beside it, and the customer outside all of them. Classes name their package with `in`; relations run across package borders as before. The description names every package with what it holds, and the table gains a Package column such as Shop › Ordering.',
  },
  {
    slug: 'invoice-model',
    group: 'Class and entity diagrams',
    chart: 'Class diagram with an oriented package',
    useCase: 'Lay out one part of a model its own way',
    blurb:
      'Orders and their invoices in two packages. The diagram runs top down, but the Billing package sets `"orientation": "landscape"`, so invoice, invoice lines and tax rate lie side by side as one block that the diagram places like a class; relations from outside meet the block where their class lies. The reminder stays outside both packages.',
  },
  {
    slug: 'subscription-model',
    group: 'Class and entity diagrams',
    chart: 'Class diagram with notes',
    useCase: 'Record rules a class box cannot show',
    blurb:
      'A subscription model with three `notes`: when a subscription renews, that plan prices are net, and that a card stores only a token. Each note is a pale box with a folded corner beside its class, joined by a dotted line that only says “explains” — it is no relation, so the structure stays the same. The description gives every note after the classes, the table a Notes column.',
  },
  {
    slug: 'payment-redesign',
    group: 'Class and entity diagrams',
    chart: 'Class diagram with roles',
    useCase: 'Show what a redesign changes in a model',
    blurb:
      'Payments after a redesign: the order and its dependency on the payment interface are `critical`, the wallet payment and the composition with the receipt `planned`, the paper invoice and the relation to it `deprecated` — drawn with the same colors, badges and line patterns as roles in flow and architecture diagrams. A relation keeps its marks: the planned composition with the receipt keeps its filled diamond. The legend names the roles used, the description lists them.',
  },
  {
    slug: 'ownership-structure',
    group: 'Trees and networks',
    chart: 'Tree (phone variant)',
    useCase: 'Show who owns what',
    blurb:
      'A group of companies as a tree: ten nodes in four levels, a share written on every link, parents centered over their children, and subtrees pushed together so that a narrow branch tucks in under a broad one. Laid out twice, at 1000 pixels and at 448 for a phone. The description names the root and what hangs below every node; the table lists each node with its level, parent, link and children. Illustrative values.',
  },
  {
    slug: 'family-tree',
    group: 'Trees and networks',
    chart: 'Tree (couples)',
    useCase: 'Draw a family or a household',
    blurb:
      'Three generations: persons in their own shape, couples side by side with their children hanging from the line between them, and a dashed box for something outside the family. The table gains a Partner column; the description names every couple as one. Illustrative family.',
  },
  {
    slug: 'product-strategy',
    group: 'Trees and networks',
    chart: 'Mind map (radial tree)',
    useCase: 'Lay out themes around one centre',
    blurb:
      'A product strategy as a mind map: the platform in the middle, four themes on the ring around it and their measures on the ring outside, every theme in a sector as wide as its share of the measures, branches clockwise from twelve o\'clock. The description calls it a mind map and names the centre and what hangs below every theme. Illustrative values.',
  },
  {
    slug: 'climate-similarity',
    group: 'Trees and networks',
    chart: 'Dendrogram from a distance matrix',
    useCase: 'Show which items belong together',
    blurb:
      'Twelve capitals and how far apart their climates are. chartlet joins the two closest again and again (average linkage) and draws every join at its distance; the warm south, the west and centre, and the north fall apart into their groups. The description names the closest pair and the last join, the table every join. Illustrative values.',
  },
  {
    slug: 'department-budgets',
    group: 'Trees and networks',
    chart: 'Tree with boxes colored by value',
    useCase: 'Show where the money goes in an organization',
    blurb:
      'An organization chart whose boxes are filled by budget, from light for the smallest to dark for the largest, with a key below. The text in every box keeps its contrast on the fill; the description and the table give every budget, so the color is never the only sign. Illustrative values.',
  },
  {
    slug: 'co-authors',
    group: 'Trees and networks',
    chart: 'Network on a circle',
    useCase: 'Show who works with whom',
    blurb:
      'Fourteen researchers of four institutes and their joint papers. The groups stand together around the circle, ordered so that few edges cross; every edge is as strong as its number of papers and bends a little towards the middle. The description names the most connected people and every person with their co-authors. With a phone variant. Illustrative values.',
  },
  {
    slug: 'service-dependencies',
    group: 'Trees and networks',
    chart: 'Network in rings around one node',
    useCase: 'Show what one thing depends on',
    blurb:
      'The checkout service in the middle, the services it calls on the first ring, theirs on the second, and so on; arrowheads say which way a dependency runs. The description lists the rings with their services. Illustrative values.',
  },
  {
    slug: 'improvement-cycle',
    group: 'Trees and networks',
    chart: 'Cycle with the labels inside boxes',
    useCase: 'Show a loop that never ends',
    blurb:
      'The improvement cycle of a support team: five steps in boxes on a ring, an arrow from each to the next and one from the last back to the first. A note stands under two labels; one step is `critical` and one `planned`, drawn with the badges and outline patterns of roles. The description reads the steps in order and names the way back, the table has one row per step. With a phone variant. Illustrative steps.',
  },
  {
    slug: 'growth-flywheel',
    group: 'Trees and networks',
    chart: 'Cycle around a hub',
    useCase: 'Show a flywheel and what it feeds',
    blurb:
      'Five steps on a ring with numbered dots and their labels outside, and a hub in the middle that every step feeds through a dotted link with a label of its own. In the HTML profile each step is a focus link that keeps its neighbours and the hub and dims the rest. The description names the hub and every link label. With a phone variant. Illustrative steps.',
  },
  {
    slug: 'mobile-revenue',
    group: 'Mobile variant',
    chart: 'Mobile variant',
    useCase: 'Stay readable on a phone',
    blurb:
      'Three sales channels over eighteen months, laid out twice: at 800 × 450 and at 360 × 420 for containers narrower than 640 pixels, where the legend wraps into two rows. A container query switches between them; caption, source and data table appear once. The output panel on this page is narrower than 640 pixels at every window size, so it shows the mobile variant. Illustrative values.',
  },
];

/**
 * The QR code is a Rust API, not a chart type: it has no specification, so its example is the
 * committed output of the code shown, which `tests/qr.rs` keeps equal to what that code draws.
 */
const qrExample: ShowcaseExample = {
  slug: 'qr-code',
  group: 'QR codes',
  title: 'QR code for a link',
  description:
    'A link as a QR code of version 5, drawn as a small, deterministic SVG: one unit per module, a quiet zone of four modules and the title as the accessible name. `chartlet::qr` is a Rust API, not a chart type — there is no specification and no command-line option for it.',
  file: 'examples/qr/link.svg',
  tags: ['QR code', 'Link a page from print'],
  input: {
    code: `use chartlet::qr::{ErrorCorrection, QrCode};

let code = QrCode::encode("https://casoon.github.io/chartlet/", ErrorCorrection::Medium)?;
let svg = code.to_svg("Link to the chartlet documentation", 4);`,
    lang: 'rust',
  },
  output: { html: qrLink, kind: 'panel' },
};

export const examples: ShowcaseExample[] = catalogue.map(({ slug, view, group, chart, useCase, blurb }): ShowcaseExample => {
  const source = spec(slug);
  const { html, warnings } = render(source, `showcase-${slug}`, 'html', view);
  return {
    slug,
    group,
    title: String(source.title),
    description: warnings.length > 0 ? `${blurb} Build-time warnings: ${warnings.join(' · ')}` : blurb,
    file: `examples/${slug}.json`,
    tags: [chart, useCase],
    input: { code: JSON.stringify(source, null, 2), lang: 'json' },
    output: { html, kind: 'panel' },
  };
}).concat(qrExample);
