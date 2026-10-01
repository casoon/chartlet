import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

const page_url = 'interactive/';

test.describe('optional interactive module', () => {
  test('the crosshair moves with the keyboard and closes with Escape', async ({ page }) => {
    await page.goto(page_url);
    const svg = page.locator('#sensors-z0');
    const values = page.locator('#sensors-z0 + .chartlet-crosshair-value');
    await expect(svg).toHaveAttribute('tabindex', '0');
    await expect(values).toHaveAttribute('role', 'status');
    await expect(values).toBeHidden();

    await svg.focus();
    await page.keyboard.press('ArrowRight');
    await expect(values).toBeVisible();
    // The values come from the data table, as the chart writes them.
    await expect(values.locator('strong')).toHaveText('2026-03-01');
    await expect(values).toContainText('Servers: 22.0');
    await expect(svg.locator('.chartlet-crosshair line')).toHaveCount(1);

    await page.keyboard.press('ArrowRight');
    await expect(values.locator('strong')).toHaveText('2026-03-02');
    await page.keyboard.press('End');
    await expect(values.locator('strong')).toHaveText('2026-03-28');
    await page.keyboard.press('Escape');
    await expect(values).toBeHidden();
    await expect(svg.locator('.chartlet-crosshair line')).toHaveCount(0);
  });

  test('the crosshair follows the pointer and stays while hovered', async ({ page }) => {
    await page.goto(page_url);
    const svg = page.locator('#sensors-z0');
    const values = page.locator('#sensors-z0 + .chartlet-crosshair-value');
    await svg.scrollIntoViewIfNeeded();
    const box = await svg.boundingBox();
    if (!box) throw new Error('the chart is not laid out');
    await page.mouse.move(box.x + box.width * 0.5, box.y + box.height * 0.5);
    await expect(values).toBeVisible();
    const first = await values.locator('strong').textContent();
    await page.mouse.move(box.x + box.width * 0.9, box.y + box.height * 0.5);
    await expect(values.locator('strong')).not.toHaveText(first ?? '');
    // Moving onto the values keeps them open (WCAG 1.4.13).
    const overlay = await values.boundingBox();
    if (!overlay) throw new Error('the values are not laid out');
    await page.mouse.move(overlay.x + 4, overlay.y + 4);
    await expect(values).toBeVisible();
    await page.mouse.move(0, 0);
    await expect(values).toBeHidden();
  });

  test('small multiples share one crosshair', async ({ page }) => {
    await page.goto(page_url);
    const svg = page.locator('#pathways');
    await svg.focus();
    await page.keyboard.press('ArrowRight');
    const panes = await svg.locator('[data-chartlet-plot]').count();
    expect(panes).toBeGreaterThan(1);
    await expect(svg.locator('.chartlet-crosshair line')).toHaveCount(panes);
  });

  test('a series can be switched off', async ({ page }) => {
    await page.goto(page_url);
    const filter = page.locator('section:has(#sensors-z0) .chartlet-filter');
    await expect(filter.locator('legend')).toHaveText('Series');
    const servers = filter.getByLabel('Servers');
    await expect(servers).toBeChecked();
    await servers.uncheck();
    await expect(page.locator('#sensors-z0 g[data-name="Servers"]').first()).toHaveAttribute(
      'display',
      'none',
    );
    const svg = page.locator('#sensors-z0');
    await svg.focus();
    await page.keyboard.press('ArrowRight');
    const values = page.locator('#sensors-z0 + .chartlet-crosshair-value');
    await expect(values).toContainText('Cooling');
    await expect(values).not.toContainText('Servers');
    await servers.check();
    await expect(page.locator('#sensors-z0 g[data-name="Servers"]').first()).not.toHaveAttribute(
      'display',
      'none',
    );
  });

  test('playback can be paused and shows everything again', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'no-preference' });
    await page.goto(page_url);
    const controls = page.locator('.chartlet-play');
    const toggle = controls.getByRole('button', { name: 'Play' });
    await toggle.click();
    await expect(controls.getByRole('button', { name: 'Pause' })).toBeVisible();
    await expect(page.locator('#projection g[data-series][clip-path]').first()).toBeAttached();
    await controls.getByRole('button', { name: 'Pause' }).click();
    await expect(controls.getByRole('status')).not.toHaveText('');
    await controls.getByRole('button', { name: 'Show all' }).click();
    await expect(page.locator('#projection [clip-path]')).toHaveCount(0);
  });

  test('with reduced motion there is no playback, only steps', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await page.goto(page_url);
    const controls = page.locator('.chartlet-play');
    await expect(controls.getByRole('button', { name: 'Play' })).toHaveCount(0);
    await controls.getByRole('button', { name: 'Step forward' }).click();
    await expect(controls.getByRole('status')).not.toHaveText('');
    await expect(page.locator('#projection g[data-series][clip-path]').first()).toBeAttached();
    await controls.getByRole('button', { name: 'Show all' }).click();
    await expect(page.locator('#projection [clip-path]')).toHaveCount(0);
  });

  test('a scroll station brings its series forward', async ({ page }) => {
    await page.goto(page_url);
    await page.locator('[data-chartlet-highlight="20-year mean"]').scrollIntoViewIfNeeded();
    await page.locator('[data-chartlet-highlight="20-year mean"]').evaluate((station) =>
      window.scrollBy(0, station.getBoundingClientRect().top - window.innerHeight / 2 + 20),
    );
    await expect(page.locator('#threshold g[data-name="Single years"]').first()).toHaveAttribute(
      'opacity',
      '0.25',
    );
    await expect(page.locator('#threshold g[data-name="20-year mean"]').first()).not.toHaveAttribute(
      'opacity',
      /./,
    );
  });

  for (const colorScheme of ['light', 'dark'] as const) {
    test(`stays accessible with the values open (${colorScheme})`, async ({ page }) => {
      await page.emulateMedia({ colorScheme });
      await page.goto(page_url);
      await page.locator('#sensors-z0').focus();
      await page.keyboard.press('ArrowRight');
      await expect(page.locator('#sensors-z0 + .chartlet-crosshair-value')).toBeVisible();
      // The theme's translucent header would otherwise be measured over the chart's surface.
      await page.evaluate(() => window.scrollTo(0, 0));
      const accessibility = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag22aa'])
        .analyze();
      expect(accessibility.violations).toEqual([]);
    });
  }

  test('loads only its own module script', async ({ page }) => {
    await page.goto(page_url);
    const sources = await page
      .locator('script[src]')
      .evaluateAll((scripts) => scripts.map((script) => (script as HTMLScriptElement).src));
    for (const source of sources) {
      expect(new URL(source).origin).toBe(new URL(page.url()).origin);
    }
  });
});

test.describe('without JavaScript', () => {
  test.use({ javaScriptEnabled: false });

  test('every chart stays a complete static figure', async ({ page }) => {
    await page.goto(page_url);
    await expect(page.locator('figure.chartlet-figure')).toHaveCount(4);
    await expect(page.locator('table[data-chartlet-table]')).toHaveCount(4);
    await expect(page.locator('.chartlet-crosshair-value')).toHaveCount(0);
    await expect(page.locator('#sensors-z0')).not.toHaveAttribute('tabindex', /./);
  });
});
