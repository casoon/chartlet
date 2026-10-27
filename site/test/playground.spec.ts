import { existsSync } from 'node:fs';
import { expect, test } from '@playwright/test';

// The compiler is copied into the site at build time when packages/chartlet/src/chartlet.wasm
// exists (the Pages workflow builds it). Which half of this file runs depends on that.
const compiler = existsSync('dist/playground/chartlet.wasm');

test.describe('without the compiler', () => {
  test.skip(compiler, 'chartlet.wasm is built into this site');

  test('the page says so, keeps the editor and the JSON download', async ({ page }) => {
    await page.goto('playground/');
    await expect(page.getByRole('alert')).toContainText('could not be loaded');
    await expect(page.getByLabel('Specification as JSON')).toBeEditable();
    await expect(page.getByRole('button', { name: 'Download JSON' })).toBeEnabled();
  });
});

test.describe('with the compiler', () => {
  test.skip(!compiler, 'chartlet.wasm is not built into this site');

  test('an example renders, an error is located, corrected and exported', async ({ page }) => {
    await page.goto('playground/');
    const status = page.getByRole('status');
    await expect(status).toContainText('Rendered');
    await expect(page.locator('#pg-preview svg')).toBeVisible();

    await page.getByLabel('Start from').selectOption('error');
    await expect(status).toContainText('duplicate_label');
    await page.getByRole('button', { name: 'Go to field' }).click();
    await expect(page.getByLabel('Specification as JSON')).toBeFocused();

    // Correct it in the form: the second January becomes February.
    await page.getByRole('button', { name: 'Form' }).click();
    await page.getByLabel('Label of row 2').fill('February');
    await expect(status).toContainText('Rendered');

    await page.getByRole('button', { name: 'JSON' }).click();
    await expect(page.getByLabel('Specification as JSON')).toHaveValue(/"February"/);
    const download = page.waitForEvent('download');
    await page.getByRole('button', { name: 'Download SVG' }).click();
    expect((await download).suggestedFilename()).toBe('chartlet.svg');
  });

  test('a narrow canvas offers a checked correction that is applied only on request', async ({
    page,
  }) => {
    await page.goto('playground/');
    await page.getByLabel('Start from').selectOption('narrow');
    await expect(page.getByRole('list', { name: 'Diagnostics' })).toContainText('canvas_too_small');
    await expect(page.getByLabel('Specification as JSON')).toHaveValue(/"width": 320/);
    await page.getByRole('button', { name: 'Apply this change' }).first().click();
    await expect(page.getByLabel('Specification as JSON')).not.toHaveValue(/"width": 320/);
    await page.getByRole('button', { name: 'Undo' }).click();
    await expect(page.getByLabel('Specification as JSON')).toHaveValue(/"width": 320/);
  });
});
