import { test, expect } from '@playwright/test';

/**
 * Screenshot generation tests for USE-CASES.md documentation.
 * These tests generate screenshots of various calculator use cases.
 */

const SCREENSHOT_DIR = '../docs/use-cases';

test.describe('Screenshot Generation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for WASM to be ready
    await expect(page.locator('textarea')).toBeEnabled({ timeout: 15000 });
    // Wait for exchange rates to load
    await page.waitForTimeout(2000);
  });

  test('01-initial-state', async ({ page }) => {
    await page.screenshot({
      path: `${SCREENSHOT_DIR}/01-initial-state.png`,
      fullPage: false,
    });
  });

  test('02-simple-arithmetic', async ({ page }) => {
    const input = page.locator('textarea');
    await input.fill('2 + 3');
    await page.keyboard.press('Enter');
    await expect(page.locator('.result-value')).toContainText('5', { timeout: 5000 });

    await page.screenshot({
      path: `${SCREENSHOT_DIR}/02-simple-arithmetic.png`,
      fullPage: false,
    });
  });

  test('03-currency-conversion', async ({ page }) => {
    const input = page.locator('textarea');
    await input.fill('84 USD - 34 EUR');
    await page.keyboard.press('Enter');
    await expect(page.locator('.result-value')).not.toContainText('Enter an expression', { timeout: 10000 });

    await page.screenshot({
      path: `${SCREENSHOT_DIR}/03-currency-conversion.png`,
      fullPage: false,
    });
  });

  test('04-integration', async ({ page }) => {
    const input = page.locator('textarea');
    await input.fill('integrate sin(x)/x dx');
    await page.keyboard.press('Enter');
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 10000 });
    // Wait for plot to render
    await page.waitForTimeout(1000);

    await page.screenshot({
      path: `${SCREENSHOT_DIR}/04-integration.png`,
      fullPage: true,
    });
  });

  test('05-dark-theme', async ({ page }) => {
    // First enter an expression
    const input = page.locator('textarea');
    await input.fill('integrate sin(x)/x dx');
    await page.keyboard.press('Enter');
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(500);

    // Switch to dark theme
    const settingsButton = page.locator('.settings-button');
    await settingsButton.click();
    await expect(page.locator('.settings-dropdown')).toBeVisible();

    const darkButton = page.locator('.settings-buttons button:has-text("Dark")');
    await darkButton.click();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');

    await page.screenshot({
      path: `${SCREENSHOT_DIR}/05-dark-theme.png`,
      fullPage: true,
    });
  });

  test('06-definite-integral', async ({ page }) => {
    // Switch to dark theme first for consistency
    const settingsButton = page.locator('.settings-button');
    await settingsButton.click();
    const darkButton = page.locator('.settings-buttons button:has-text("Dark")');
    await darkButton.click();
    // Close dropdown
    await page.keyboard.press('Escape');

    const input = page.locator('textarea');
    await input.fill('integrate(x^2, x, 0, 3)');
    await page.keyboard.press('Enter');
    await expect(page.locator('.result-value')).toContainText('9', { timeout: 5000 });

    await page.screenshot({
      path: `${SCREENSHOT_DIR}/06-definite-integral.png`,
      fullPage: false,
    });
  });

  test('07-math-functions', async ({ page }) => {
    // Switch to dark theme first for consistency
    const settingsButton = page.locator('.settings-button');
    await settingsButton.click();
    const darkButton = page.locator('.settings-buttons button:has-text("Dark")');
    await darkButton.click();
    // Close dropdown
    await page.keyboard.press('Escape');

    const input = page.locator('textarea');
    await input.fill('sqrt(16) + pow(2, 3)');
    await page.keyboard.press('Enter');
    await expect(page.locator('.result-value')).toContainText('12', { timeout: 5000 });

    await page.screenshot({
      path: `${SCREENSHOT_DIR}/07-math-functions.png`,
      fullPage: false,
    });
  });
});
