import { readdirSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

// Every built page, read from dist/ so new docs pages and examples are covered automatically.
function routes(dir = 'dist'): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) return routes(path);
    if (entry !== 'index.html') return [];
    const route = relative('dist', dir).split(sep).join('/');
    return [route === '' ? '' : `${route}/`];
  });
}

for (const route of routes()) {
  for (const colorScheme of ['light', 'dark'] as const) {
    test(`${route || 'index'} (${colorScheme}) is accessible and stays within the project base`, async ({
      page,
    }) => {
      await page.emulateMedia({ colorScheme });
      await page.goto(route);

      const accessibility = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag22aa'])
        .analyze();
      expect(accessibility.violations).toEqual([]);

      // The theme's two small inline scripts only; nothing is loaded from elsewhere.
      await expect(page.locator('script[src]')).toHaveCount(0);

      const internalPaths = await page.locator('a[href]').evaluateAll((links) =>
        links
          .map((link) => new URL((link as HTMLAnchorElement).href))
          .filter((url) => url.origin === window.location.origin)
          .map((url) => url.pathname),
      );
      expect(internalPaths.every((path) => path.startsWith('/chartlet/'))).toBe(true);
    });
  }
}
