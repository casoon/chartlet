// The canonical example specifications live in `<repo>/examples/` and are also the
// golden-file test inputs. The gallery imports them directly so there is exactly one
// source of truth for each chart.

import monthlyRevenue from '../../../examples/monthly-revenue.json';
import quarterlyChange from '../../../examples/quarterly-change.json';
import budgetVsActual from '../../../examples/budget-vs-actual.json';
import monthlyTrend from '../../../examples/monthly-trend.json';
import operatingCosts from '../../../examples/operating-costs.json';
import supportVolume from '../../../examples/support-volume.json';
import revenueByChannel from '../../../examples/revenue-by-channel.json';
import signupConversion from '../../../examples/signup-conversion.json';
import headcount from '../../../examples/headcount.json';
import csatByRegion from '../../../examples/csat-by-region.json';

export interface Example {
  slug: string;
  /** Human-friendly chart-type label. */
  chart: string;
  /** The business use case the example illustrates. */
  useCase: string;
  /** One or two sentences describing the scenario. */
  blurb: string;
  spec: Record<string, unknown>;
}

export const examples: Example[] = [
  {
    slug: 'monthly-revenue',
    chart: 'Bar (vertical)',
    useCase: 'Compare categories',
    blurb: 'A single series of revenue values, one bar per month.',
    spec: monthlyRevenue,
  },
  {
    slug: 'operating-costs',
    chart: 'Bar (vertical)',
    useCase: 'Track a cost trend',
    blurb: 'Operating costs over half a year with a value axis title.',
    spec: operatingCosts,
  },
  {
    slug: 'quarterly-change',
    chart: 'Bar (horizontal)',
    useCase: 'Show change and deviation',
    blurb: 'Positive and negative change, formatted as percent, with a deliberately long label.',
    spec: quarterlyChange,
  },
  {
    slug: 'signup-conversion',
    chart: 'Bar (horizontal)',
    useCase: 'Compare rates',
    blurb: 'Signup conversion per landing page, sorted from lowest to highest.',
    spec: signupConversion,
  },
  {
    slug: 'budget-vs-actual',
    chart: 'Grouped bar',
    useCase: 'Plan versus actual',
    blurb:
      'Budget and actual costs side by side, including a missing value. Use the checkboxes to hide a series, or hover a bar for its value.',
    spec: budgetVsActual,
  },
  {
    slug: 'revenue-by-channel',
    chart: 'Grouped bar',
    useCase: 'Break down by segment',
    blurb: 'Revenue split across three channels over four quarters.',
    spec: revenueByChannel,
  },
  {
    slug: 'csat-by-region',
    chart: 'Grouped bar (horizontal)',
    useCase: 'Compare across groups',
    blurb: 'Customer satisfaction scores for two years, four regions.',
    spec: csatByRegion,
  },
  {
    slug: 'monthly-trend',
    chart: 'Line',
    useCase: 'Show a trend',
    blurb: 'Monthly active users, with a gap where a month has no data.',
    spec: monthlyTrend,
  },
  {
    slug: 'support-volume',
    chart: 'Line',
    useCase: 'Track a metric',
    blurb: 'Weekly support tickets with a mid-period dip.',
    spec: supportVolume,
  },
  {
    slug: 'headcount',
    chart: 'Line',
    useCase: 'Show growth',
    blurb:
      'Steady headcount growth from January to August. Switch between the full period and each half with the pre-rendered zoom steps.',
    spec: headcount,
  },
];

export function getExample(slug: string): Example | undefined {
  return examples.find((example) => example.slug === slug);
}