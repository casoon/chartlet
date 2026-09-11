import { renderChart } from '@casoon/chartlet';

export interface RenderedChart {
  /** Full SVG or HTML output from the chartlet renderer. */
  content: string;
  /** The accessible description chartlet placed in `<desc>`, unescaped. */
  description: string;
  /** Layout warnings reported to stderr by the CLI. */
  warnings: string[];
}

/**
 * Renders a specification with the `@casoon/chartlet` package and extracts the
 * accessible description for display. Runs purely at build time.
 */
export function render(
  spec: Record<string, unknown>,
  id: string,
  options: { format?: 'svg' | 'html'; table?: 'details' | 'visible' } = {},
): RenderedChart {
  const result = renderChart(spec, {
    format: options.format ?? 'html',
    table: options.table ?? 'visible',
    idPrefix: id,
  });
  return {
    content: result.content,
    description: extractDescription(result.content),
    warnings: result.warnings,
  };
}

/** Pulls chartlet's generated `<desc>` out of the rendered SVG or HTML. */
function extractDescription(content: string): string {
  const match = /<desc[^>]*>([\s\S]*?)<\/desc>/.exec(content);
  if (!match) return '';
  return decodeEntities(match[1]);
}

/** chartlet only ever emits the five named XML entities. */
function decodeEntities(text: string): string {
  return text
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&amp;/g, '&');
}