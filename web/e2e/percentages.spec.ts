import { test, expect, waitForWasm } from './fixtures';

test('percentage values survive calculations in the WASM worker', async ({ page }) => {
  await page.goto('/');
  await waitForWasm(page, { ratesMs: 0 });
  for (const [expression, expected] of [
    ['200 + 10%', '220'],
    ['200 + Percent(10)', '220'],
    ['100 + 10% + 10%', '121'],
    ['10% + 20%', '30%'],
    ['180 is what % off 200', '10%'],
    ['20% is 500, what is 750', '30%'],
    ['if 20 is 30%, what is 60%', '40'],
    ['20% of 50 km/h', '10 km/h'],
    ['56.7% of 1,234 participants', '699.678 participants'],
  ]) {
    await page.locator('textarea').fill(expression);
    await page.locator('textarea').press('Enter');
    await expect(page.locator('.result-value')).toHaveText(expected);
  }
});
