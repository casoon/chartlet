export type Status = 'verified' | 'designed' | 'pending' | 'not-supported';

export interface MatrixGroup {
  heading: string;
  rows: { context: string; status: Status; note: string }[];
}

export const statusLabels: Record<Status, string> = {
  verified: 'Verified',
  designed: 'Designed, not yet verified',
  pending: 'Planned',
  'not-supported': 'Not supported',
};

export const supportMatrix: MatrixGroup[] = [
  {
    heading: 'Embedding',
    rows: [
      {
        context: 'Standalone SVG',
        status: 'verified',
        note: 'Byte-identical on macOS and Linux; covered by the golden-file tests.',
      },
      {
        context: 'HTML figure (caption, source, data table)',
        status: 'verified',
        note: 'The data table is always present, in a native disclosure by default.',
      },
      {
        context: 'Astro component',
        status: 'verified',
        note: 'Renders at build time and ships no chart JavaScript to the browser.',
      },
      {
        context: 'CLI and Rust API',
        status: 'verified',
        note: 'The same renderer backs the CLI, the crate, and the npm package.',
      },
      {
        context: '<img src="chart.svg">',
        status: 'designed',
        note: 'Title and description survive; the figure wrapper and data table do not apply.',
      },
      { context: 'Markdown', status: 'pending', note: 'Not yet evaluated.' },
      { context: 'PDF and print', status: 'pending', note: 'Not yet evaluated.' },
      { context: 'E-mail clients', status: 'pending', note: 'Not yet evaluated.' },
    ],
  },
  {
    heading: 'Browsers',
    rows: [
      {
        context: 'Chromium (Chrome, Edge)',
        status: 'verified',
        note: 'Automated accessibility scan (axe) and manual review during development.',
      },
      {
        context: 'Firefox',
        status: 'pending',
        note: 'Standard SVG is expected to work; the accessibility tree is not yet tested.',
      },
      {
        context: 'Safari',
        status: 'pending',
        note: 'Standard SVG is expected to work; the accessibility tree is not yet tested.',
      },
    ],
  },
  {
    heading: 'Screen readers',
    rows: [
      {
        context: 'VoiceOver (macOS)',
        status: 'pending',
        note: 'The title and description are exposed as the accessible name and description; not yet tested.',
      },
      { context: 'NVDA (Windows)', status: 'pending', note: 'Not yet tested.' },
      { context: 'JAWS (Windows)', status: 'pending', note: 'Not yet tested.' },
    ],
  },
  {
    heading: 'Themes',
    rows: [
      {
        context: 'Light (default)',
        status: 'verified',
        note: 'The default palette keeps at least 4.5:1 contrast on white.',
      },
      {
        context: 'Dark',
        status: 'pending',
        note: 'Planned via CSS custom properties; not yet shipped.',
      },
      {
        context: 'Forced colors / high contrast',
        status: 'pending',
        note: 'Planned; not yet shipped.',
      },
    ],
  },
  {
    heading: 'Distribution and interactions',
    rows: [
      {
        context: 'Browser WASM build',
        status: 'not-supported',
        note: 'Planned after the MVP.',
      },
      {
        context: 'Native Node bindings',
        status: 'not-supported',
        note: 'Planned; the alpha npm package shells out to the CLI.',
      },
      {
        context: 'Series filtering and stepped zoom',
        status: 'designed',
        note: 'Available in the HTML profile with native controls and CSS; broader browser and assistive-technology verification is pending.',
      },
      {
        context: 'Mark tooltips',
        status: 'designed',
        note: 'Native SVG titles provide hover hints; the description and table remain the accessible alternatives.',
      },
    ],
  },
];
