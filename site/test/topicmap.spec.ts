import { expect, test } from '@playwright/test';

const page_url = 'showcase/topicmap-sample/';
const accent = 'rgb(37, 99, 235)';
const quiet = 'rgb(255, 255, 255)';

test.describe('topic map selection', () => {
  test('the picker is a real radio group, with the first area selected', async ({ page }) => {
    await page.goto(page_url);
    const radios = page.locator('.chartlet-topic-picker input[type="radio"]');
    await expect(radios).toHaveCount(5);
    await expect(radios.first()).toBeChecked();
    await expect(page.locator('.chartlet-topic-picker legend')).toHaveText('Area');
    // One group: checking another area unchecks the first, without any script on the page.
    await expect(page.locator('script[src]')).toHaveCount(0);
  });

  test('choosing an area brings it forward and quiets the rest', async ({ page }) => {
    await page.goto(page_url);
    const chosen = page.locator('.chartlet-topic-area.chartlet-topic-2');
    const other = page.locator('.chartlet-topic-area.chartlet-topic-0');

    // The first area is selected on load.
    await expect(other).toHaveCSS('fill', accent);
    await expect(chosen).toHaveCSS('fill', quiet);

    await page.locator('.chartlet-topic-picker input.topic-2').check();
    await expect(chosen).toHaveCSS('fill', accent);
    await expect(other).toHaveCSS('fill', quiet);
    // The label beside the map follows its area, so the selection is readable in the text too.
    await expect(page.locator('text.chartlet-topic-outside.chartlet-topic-2')).toHaveCSS(
      'fill',
      accent,
    );
  });

  test('the selection can be moved with the keyboard alone', async ({ page }) => {
    await page.goto(page_url);
    const first = page.locator('.chartlet-topic-picker input.topic-0');
    await first.focus();
    await expect(first).toBeFocused();

    await page.keyboard.press('ArrowDown');
    await expect(page.locator('.chartlet-topic-picker input.topic-1')).toBeChecked();
    await expect(page.locator('.chartlet-topic-area.chartlet-topic-1')).toHaveCSS('fill', accent);
  });

  test('detail panels a host page places beside the chart follow the selection', async ({
    page,
  }) => {
    await page.goto(page_url);
    // chartlet never emits panels — the entries belong to the page. This is the markup contract
    // the emitted rules promise to work with: panels as siblings of the wrapper.
    await page.evaluate(() => {
      const wrapper = document.querySelector('.chartlet-wrapper');
      for (const index of [0, 2]) {
        const panel = document.createElement('p');
        panel.className = `chartlet-topic-panel chartlet-topic-panel-${index}`;
        panel.dataset.testid = `panel-${index}`;
        panel.textContent = `Entries for area ${index}`;
        wrapper?.after(panel);
      }
    });

    await expect(page.getByTestId('panel-0')).toBeVisible();
    await expect(page.getByTestId('panel-2')).toBeHidden();

    await page.locator('.chartlet-topic-picker input.topic-2').check();
    await expect(page.getByTestId('panel-2')).toBeVisible();
    await expect(page.getByTestId('panel-0')).toBeHidden();
  });

  test('on a phone the map stays readable as text', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 800 });
    await page.goto(page_url);

    // The drawing scales down until its type is too small to read, but the parts that carry the
    // data in words do not depend on the drawing at all.
    await expect(page.locator('.chartlet-topic-picker input[type="radio"]')).toHaveCount(5);
    const table = page.locator('.chartlet-data table').first();
    for (const area of ['AI in practice', 'Engineering', 'Cloud native', 'WASM']) {
      await expect(table).toContainText(area);
    }
    await expect(table).toContainText('34.52%');

    // Nothing spills sideways out of the page.
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
  });
});

test('the map carries its own text alternative', async ({ page }) => {
  await page.goto(page_url);
  // This example writes its own description, which is what a specification's `description` is
  // for; the generated fallback is covered by the crate's own tests.
  const description = page.locator('.chartlet-root desc').first();
  await expect(description).toContainText('Which topics carry the most published articles');
  await expect(page.locator('.chartlet-root')).toHaveAttribute('role', 'img');

  const table = page.locator('.chartlet-data table').first();
  await expect(table.locator('th[scope="col"]')).toHaveText([
    'Topic',
    'Entries',
    'Paths',
    'Share',
  ]);
  await expect(table.locator('tbody tr')).toHaveCount(6);
});
