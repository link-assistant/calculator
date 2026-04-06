/**
 * Screenshot generation tests for USE-CASES.md documentation.
 *
 * Uses browser-commander for all browser control. Screenshots are full-page
 * so they capture the complete calculator output including plots/steps.
 *
 * Output:
 *   docs/use-cases/01-initial-state.png … 09-parentheses.png
 */

import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { test, expect, waitForWasm, calculate } from './fixtures';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SCREENSHOT_DIR = path.resolve(__dirname, '../../docs/use-cases');

// ─────────────────────────────────────────────────────────────────────────────
// Helper
// ─────────────────────────────────────────────────────────────────────────────

async function screenshot(commander: import('./fixtures').BrowserCommander, name: string) {
  const filePath = path.join(SCREENSHOT_DIR, name);
  await commander.page.screenshot({ path: filePath, fullPage: true });
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

test.describe('Screenshot Generation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasm(page);
  });

  test('01-initial-state', async ({ commander }) => {
    await screenshot(commander, '01-initial-state.png');
  });

  test('02-simple-arithmetic', async ({ page, commander }) => {
    await calculate(commander, '2 + 3');
    await expect(page.locator('.result-value')).toContainText('5', { timeout: 5000 });
    await screenshot(commander, '02-simple-arithmetic.png');
  });

  test('03-currency-conversion', async ({ page, commander }) => {
    await calculate(commander, '84 USD - 34 EUR');
    await expect(page.locator('.result-value')).not.toContainText('Enter an expression', {
      timeout: 10000,
    });
    await screenshot(commander, '03-currency-conversion.png');
  });

  test('04-integration', async ({ page, commander }) => {
    await calculate(commander, 'integrate sin(x)/x dx');
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(1000);
    await screenshot(commander, '04-integration.png');
  });

  test('05-dark-theme', async ({ page, commander }) => {
    await calculate(commander, 'integrate sin(x)/x dx');
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(500);

    await commander.clickButton({ selector: '.settings-button' });
    await expect(page.locator('.settings-dropdown')).toBeVisible();
    await commander.clickButton({ selector: '.settings-buttons button:has-text("Dark")' });
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');

    await screenshot(commander, '05-dark-theme.png');
  });

  test('06-definite-integral', async ({ page, commander }) => {
    await commander.clickButton({ selector: '.settings-button' });
    await expect(page.locator('.settings-dropdown')).toBeVisible();
    await commander.clickButton({ selector: '.settings-buttons button:has-text("Dark")' });
    await commander.pressKey({ key: 'Escape' });

    await calculate(commander, 'integrate(x^2, x, 0, 3)');
    await expect(page.locator('.result-value')).toContainText('9', { timeout: 5000 });
    await screenshot(commander, '06-definite-integral.png');
  });

  test('07-math-functions', async ({ page, commander }) => {
    await commander.clickButton({ selector: '.settings-button' });
    await expect(page.locator('.settings-dropdown')).toBeVisible();
    await commander.clickButton({ selector: '.settings-buttons button:has-text("Dark")' });
    await commander.pressKey({ key: 'Escape' });

    await calculate(commander, 'sqrt(16) + pow(2, 3)');
    await expect(page.locator('.result-value')).toContainText('12', { timeout: 5000 });
    await screenshot(commander, '07-math-functions.png');
  });

  test('08-datetime', async ({ page, commander }) => {
    await calculate(commander, '(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)');
    await expect(page.locator('.result-value')).toContainText('day', { timeout: 5000 });
    await screenshot(commander, '08-datetime.png');
  });

  test('09-parentheses', async ({ page, commander }) => {
    await calculate(commander, '(2 + 3) * 4');
    await expect(page.locator('.result-value')).toContainText('20', { timeout: 5000 });
    await screenshot(commander, '09-parentheses.png');
  });
});
