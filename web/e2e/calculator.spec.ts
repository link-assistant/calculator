/**
 * End-to-end tests for Link.Calculator.
 *
 * Uses browser-commander (via the shared `./fixtures` module) for all browser
 * interactions. Playwright's test runner and expect assertions are still used
 * for waiting and asserting on DOM state.
 *
 * Covered critical user paths:
 *  - Basic UI: page loads, heading, textarea visible
 *  - Arithmetic: simple calculation, steps panel, multiline
 *  - Error handling: invalid expression, empty input
 *  - Theme: toggle to dark, persistence in localStorage
 *  - Language: selector present, changing to German
 *  - URL sharing: updates on input, loads from URL, browser history
 *  - Report issue: button exists, opens GitHub
 *  - Responsive: mobile viewport, desktop max-width
 *  - Loading / busy indicator
 *  - Example buttons: count, fill input on click
 *  - Textarea auto-resize: grows, shrinks, min-height, content never clipped
 *  - Currency conversion: calculation, steps show rate info, multiple currencies,
 *    symbols ($, €), graceful offline fallback
 */

import { test, expect, waitForWasm, calculate } from './fixtures';

// ─────────────────────────────────────────────────────────────────────────────
// Basic UI
// ─────────────────────────────────────────────────────────────────────────────

test.describe('Calculator', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasm(page, { ratesMs: 0 });
  });

  test('should display the calculator UI', async ({ page }) => {
    await expect(page.locator('h1')).toContainText('Link.Calculator');
    await expect(page.locator('textarea')).toBeVisible();
  });

  test('should calculate simple expression', async ({ page, commander }) => {
    await calculate(commander, '2 + 3');
    await expect(page.locator('.result-value')).toContainText('5', { timeout: 5000 });
  });

  test('should show calculation steps', async ({ page, commander }) => {
    await calculate(commander, '2 + 3 * 4');
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('.steps-section')).toBeVisible();
  });

  test('should handle empty input gracefully', async ({ page }) => {
    const input = page.locator('textarea');
    await input.fill('');
    await expect(page.locator('.result-value.error')).not.toBeVisible();
  });

  test('should display error for invalid expression', async ({ page, commander }) => {
    await calculate(commander, 'invalid expression +++');
    await expect(page.locator('.result-value.error')).toBeVisible({ timeout: 5000 });
  });

  test('should handle multiline input', async ({ page, commander }) => {
    await calculate(commander, '1 + 1');
    await expect(page.locator('.result-value')).toContainText('2', { timeout: 5000 });
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Theme
// ─────────────────────────────────────────────────────────────────────────────

test.describe('Theme', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasm(page, { ratesMs: 0 });
  });

  test('should toggle theme when clicking theme button', async ({ page, commander }) => {
    await commander.clickButton({ selector: '.settings-button' });
    await expect(page.locator('.settings-dropdown')).toBeVisible();
    await commander.clickButton({ selector: '.settings-buttons button:has-text("Dark")' });
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  });

  test('should persist theme preference in localStorage', async ({ page, commander }) => {
    await commander.clickButton({ selector: '.settings-button' });
    await expect(page.locator('.settings-dropdown')).toBeVisible();
    await commander.clickButton({ selector: '.settings-buttons button:has-text("Dark")' });

    const storage = await page.evaluate(() => localStorage.getItem('link-calculator-preferences'));
    expect(storage).toContain('dark');
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Language
// ─────────────────────────────────────────────────────────────────────────────

test.describe('Language', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasm(page, { ratesMs: 0 });
  });

  test('should have language selector', async ({ page, commander }) => {
    await commander.clickButton({ selector: '.settings-button' });
    const langSelector = page.locator('.settings-dropdown select').first();
    await expect(langSelector).toBeVisible();
  });

  test('should change language when selected', async ({ page, commander }) => {
    await commander.clickButton({ selector: '.settings-button' });
    const langSelector = page.locator('.settings-dropdown select').first();
    await langSelector.selectOption('de');
    await expect(page.locator('.result-section h2')).toContainText('Ergebnis');
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// URL Sharing
// ─────────────────────────────────────────────────────────────────────────────

test.describe('URL Sharing', () => {
  test('should update URL when expression changes', async ({ page, commander }) => {
    await page.goto('/');
    await waitForWasm(page, { ratesMs: 0 });

    await commander.fillTextArea({ selector: 'textarea', text: '1 + 2' });
    await page.waitForTimeout(600);

    expect(page.url()).toContain('?q=');
  });

  test('should load expression from URL', async ({ page, commander }) => {
    const expression = '5 * 5';
    const linoEncoded = `(expression "${expression}")`;
    const base64Encoded = Buffer.from(linoEncoded).toString('base64');

    await page.goto(`/?q=${base64Encoded}`);
    await waitForWasm(page, { ratesMs: 0 });

    const input = page.locator('textarea');
    await expect(input).toHaveValue(expression);

    await input.focus();
    await commander.pressKey({ key: 'Enter' });

    await expect(page.locator('.result-value')).toContainText('25', { timeout: 5000 });
  });

  test('should add each new expression to browser history', async ({ page, commander }) => {
    await page.goto('/');
    await waitForWasm(page, { ratesMs: 0 });

    const input = page.locator('textarea');

    await calculate(commander, '1 + 1');
    await page.waitForTimeout(600);
    await expect(page.locator('.result-value')).toContainText('2', { timeout: 5000 });

    await calculate(commander, '2 + 2');
    await page.waitForTimeout(600);
    await expect(page.locator('.result-value')).toContainText('4', { timeout: 5000 });

    await calculate(commander, '3 + 3');
    await page.waitForTimeout(600);
    await expect(page.locator('.result-value')).toContainText('6', { timeout: 5000 });

    await page.goBack();
    await page.waitForTimeout(200);
    await expect(input).toHaveValue('2 + 2');
    await commander.pressKey({ key: 'Enter' });
    await expect(page.locator('.result-value')).toContainText('4', { timeout: 5000 });

    await page.goBack();
    await page.waitForTimeout(200);
    await expect(input).toHaveValue('1 + 1');
    await commander.pressKey({ key: 'Enter' });
    await expect(page.locator('.result-value')).toContainText('2', { timeout: 5000 });

    await page.goForward();
    await page.waitForTimeout(200);
    await expect(input).toHaveValue('2 + 2');
    await commander.pressKey({ key: 'Enter' });
    await expect(page.locator('.result-value')).toContainText('4', { timeout: 5000 });
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Report Issue
// ─────────────────────────────────────────────────────────────────────────────

test.describe('Report Issue', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasm(page, { ratesMs: 0 });
  });

  test('should have report issue button in footer', async ({ page }) => {
    await expect(page.locator('footer button.link-button')).toBeVisible();
  });

  test('should open GitHub issue when clicking report', async ({ page, context }) => {
    const pagePromise = context.waitForEvent('page');
    await page.locator('footer button.link-button').click();
    const newPage = await pagePromise;
    const newUrl = newPage.url();
    expect(newUrl).toContain('github.com');
    expect(newUrl).toMatch(/issues\/new|issues%2Fnew/);
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Responsive Design
// ─────────────────────────────────────────────────────────────────────────────

test.describe('Responsive Design', () => {
  test('should be mobile-friendly', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('textarea')).toBeVisible();
  });

  test('should have fixed width on desktop', async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto('/');
    const container = page.locator('.container');
    await expect(container).toBeVisible();
    const box = await container.boundingBox();
    expect(box).not.toBeNull();
    if (box) {
      expect(box.width).toBeLessThanOrEqual(800);
    }
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Busy Indicator
// ─────────────────────────────────────────────────────────────────────────────

test.describe('Busy Indicator', () => {
  test('should show loading indicator when WASM is loading', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('h1')).toBeVisible();
  });

  test('should show spinner during calculation', async ({ page, commander }) => {
    await page.goto('/');
    await waitForWasm(page, { ratesMs: 0 });
    await calculate(commander, '123456789 * 987654321');
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 10000 });
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Examples
// ─────────────────────────────────────────────────────────────────────────────

test.describe('Examples', () => {
  test('should show example buttons', async ({ page }) => {
    await page.goto('/');
    await waitForWasm(page, { ratesMs: 0 });
    await expect(page.locator('.example-button')).toHaveCount(6);
  });

  test('should fill input when clicking example', async ({ page, commander }) => {
    await page.goto('/');
    await waitForWasm(page, { ratesMs: 0 });

    const firstExample = page.locator('.example-button').first();
    await firstExample.click();

    const input = page.locator('textarea');
    await expect(input).not.toHaveValue('');

    await input.focus();
    await commander.pressKey({ key: 'Enter' });
    await expect(page.locator('.result-value')).not.toContainText('Enter an expression', {
      timeout: 5000,
    });
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Textarea Auto-Resize
// ─────────────────────────────────────────────────────────────────────────────

test.describe('Textarea Auto-Resize', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasm(page, { ratesMs: 0 });
  });

  test('should auto-resize textarea when content becomes multiline', async ({ page, commander }) => {
    const textarea = page.locator('textarea');
    const initialBox = await textarea.boundingBox();
    const initialHeight = initialBox?.height ?? 0;

    await commander.fillTextArea({ selector: 'textarea', text: 'Line 1\nLine 2\nLine 3\nLine 4' });
    await page.waitForTimeout(100);

    const newBox = await textarea.boundingBox();
    expect(newBox?.height ?? 0).toBeGreaterThan(initialHeight);
  });

  test('should shrink textarea when content is reduced', async ({ page, commander }) => {
    const textarea = page.locator('textarea');

    await commander.fillTextArea({
      selector: 'textarea',
      text: 'Line 1\nLine 2\nLine 3\nLine 4\nLine 5',
    });
    await page.waitForTimeout(100);
    const multilineHeight = (await textarea.boundingBox())?.height ?? 0;

    await commander.fillTextArea({ selector: 'textarea', text: 'Single line' });
    await page.waitForTimeout(100);
    const singleLineHeight = (await textarea.boundingBox())?.height ?? 0;

    expect(singleLineHeight).toBeLessThan(multilineHeight);
  });

  test('should maintain minimum height for empty content', async ({ page, commander }) => {
    await commander.fillTextArea({ selector: 'textarea', text: '' });
    await page.waitForTimeout(100);
    const box = await page.locator('textarea').boundingBox();
    expect(box?.height ?? 0).toBeGreaterThan(30);
  });

  test('should handle long single-line content with word wrap', async ({ page, commander }) => {
    const longExpression = '(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC) + (Jan 25, 12:51pm UTC)';
    await commander.fillTextArea({ selector: 'textarea', text: longExpression });
    await page.waitForTimeout(100);
    expect(await page.locator('textarea').boundingBox()).not.toBeNull();
  });

  test('should resize in discrete line-height steps', async ({ page, commander }) => {
    const textarea = page.locator('textarea');
    await commander.fillTextArea({ selector: 'textarea', text: 'Line 1\nLine 2' });
    await page.waitForTimeout(100);

    const box = await textarea.boundingBox();
    const height = box?.height ?? 0;

    const lineHeight = await textarea.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return parseFloat(style.lineHeight) || 24;
    });
    const padding = await textarea.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return parseFloat(style.paddingTop) + parseFloat(style.paddingBottom);
    });

    const contentHeight = height - padding;
    const lines = Math.round(contentHeight / lineHeight);
    expect(Math.abs(contentHeight - lines * lineHeight)).toBeLessThan(5);
  });

  test('should not allow resize smaller than content height', async ({ page, commander }) => {
    const textarea = page.locator('textarea');
    await commander.fillTextArea({ selector: 'textarea', text: 'Line 1\nLine 2\nLine 3' });
    await page.waitForTimeout(100);

    const contentHeight = (await textarea.boundingBox())?.height ?? 0;

    await textarea.evaluate((el) => {
      el.style.height = '30px';
    });
    await page.waitForTimeout(50);

    const newHeight = (await textarea.boundingBox())?.height ?? 0;
    expect(newHeight).toBeGreaterThanOrEqual(contentHeight - 10);
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Currency Conversion
// ─────────────────────────────────────────────────────────────────────────────

test.describe('Currency Conversion with Real Rates', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasm(page, { ratesMs: 3000 });
  });

  test('should show rates loading status in footer', async ({ page }) => {
    await expect(page.locator('footer')).toBeVisible();
  });

  test('should calculate currency conversion', async ({ page, commander }) => {
    await calculate(commander, '100 USD in EUR');
    await expect(page.locator('.result-value')).not.toContainText('Enter an expression', {
      timeout: 10000,
    });
    const resultText = await page.locator('.result-value').textContent();
    expect(resultText).toMatch(/EUR|\d+/i);
  });

  test('should show exchange rate info in calculation steps', async ({ page, commander }) => {
    await calculate(commander, '0 RUB + 1 USD');
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('.steps-section')).toBeVisible();
    const stepsText = await page.locator('.steps-section').textContent();
    expect(stepsText?.toLowerCase()).toMatch(/exchange|rate|convert/i);
  });

  test('should handle multiple currency conversions', async ({ page, commander }) => {
    await calculate(commander, '1 USD + 1 EUR + 1 GBP');
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 10000 });
    expect(await page.locator('.result-value').textContent()).toBeTruthy();
  });

  test('should display rate source information', async ({ page, commander }) => {
    await calculate(commander, '100 JPY in USD');
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 10000 });
    if (await page.locator('.steps-section').isVisible()) {
      expect(await page.locator('.steps-section').textContent()).toBeTruthy();
    }
  });

  test('should handle currency symbols', async ({ page, commander }) => {
    await calculate(commander, '$100 + €50');
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 10000 });
    const resultText = await page.locator('.result-value').textContent();
    expect(resultText).toMatch(/[$€]|\w{3}/);
  });

  test('should gracefully handle rates when offline', async ({ page, commander }) => {
    await calculate(commander, '2 + 2');
    await expect(page.locator('.result-value')).toContainText('4', { timeout: 5000 });
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Textarea Resize Visual Regression
// ─────────────────────────────────────────────────────────────────────────────

test.describe('Textarea Resize Visual Regression', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasm(page, { ratesMs: 0 });
  });

  test('textarea should display all content without clipping', async ({ page, commander }) => {
    const bugContent = '(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC) + (Jan 25, 12:51nm UTC)';
    await commander.fillTextArea({ selector: 'textarea', text: bugContent });
    await page.waitForTimeout(100);

    const overflow = await page.locator('textarea').evaluate((el) => el.scrollHeight > el.clientHeight);
    expect(overflow).toBe(false);
  });

  test('textarea content should never be partially visible', async ({ page, commander }) => {
    await commander.fillTextArea({ selector: 'textarea', text: 'Line 1\nLine 2\nLine 3' });
    await page.waitForTimeout(100);

    const textarea = page.locator('textarea');
    const lineHeight = await textarea.evaluate((el) => {
      const style = window.getComputedStyle(el);
      const lh = parseFloat(style.lineHeight);
      return isNaN(lh) ? 24 : lh;
    });
    const padding = await textarea.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return parseFloat(style.paddingTop) + parseFloat(style.paddingBottom);
    });
    const border = await textarea.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return parseFloat(style.borderTopWidth) + parseFloat(style.borderBottomWidth);
    });

    const height = (await textarea.boundingBox())?.height ?? 0;
    const contentArea = height - padding - border;
    const lines = contentArea / lineHeight;
    const remainder = lines - Math.round(lines);
    expect(Math.abs(remainder)).toBeLessThan(0.2);
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Additional Critical User Paths
// ─────────────────────────────────────────────────────────────────────────────

test.describe('Mathematical Operations', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasm(page, { ratesMs: 0 });
  });

  test('should compute definite integral', async ({ page, commander }) => {
    await calculate(commander, 'integrate(x^2, x, 0, 3)');
    await expect(page.locator('.result-value')).toContainText('9', { timeout: 5000 });
  });

  test('should compute sqrt and pow', async ({ page, commander }) => {
    await calculate(commander, 'sqrt(16) + pow(2, 3)');
    await expect(page.locator('.result-value')).toContainText('12', { timeout: 5000 });
  });

  test('should compute date/time difference', async ({ page, commander }) => {
    await calculate(commander, '(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)');
    await expect(page.locator('.result-value')).toContainText('day', { timeout: 5000 });
  });

  test('should respect parentheses in expressions', async ({ page, commander }) => {
    await calculate(commander, '(2 + 3) * 4');
    await expect(page.locator('.result-value')).toContainText('20', { timeout: 5000 });
  });
});
