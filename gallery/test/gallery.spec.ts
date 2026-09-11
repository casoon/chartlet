import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

const routes = [
  '',
  'support/',
  'examples/monthly-revenue/',
  'examples/operating-costs/',
  'examples/quarterly-change/',
  'examples/signup-conversion/',
  'examples/budget-vs-actual/',
  'examples/revenue-by-channel/',
  'examples/csat-by-region/',
  'examples/monthly-trend/',
  'examples/support-volume/',
  'examples/headcount/',
];

for (const route of routes) {
  test(`${route || 'index'} is accessible and stays within the project base`, async ({ page }) => {
    await page.goto(route);

    const accessibility = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag22aa'])
      .analyze();
    expect(accessibility.violations).toEqual([]);
    await expect(page.locator('script')).toHaveCount(0);

    const internalPaths = await page.locator('a[href]').evaluateAll((links) =>
      links
        .map((link) => new URL((link as HTMLAnchorElement).href))
        .filter((url) => url.origin === window.location.origin)
        .map((url) => url.pathname),
    );
    expect(internalPaths.every((path) => path.startsWith('/chartlet/'))).toBe(true);
  });
}
