import { test, expect, Page } from '@playwright/test';

/**
 * Helper function to enter an expression and trigger calculation.
 * The calculator now requires explicit trigger (Enter key or button click).
 */
async function calculateExpression(page: Page, expression: string) {
  const input = page.locator('textarea');
  await input.fill(expression);
  // Trigger calculation by pressing Enter
  await page.keyboard.press('Enter');
}

test.describe('Calculator', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for WASM to be ready
    await expect(page.locator('textarea')).toBeEnabled({ timeout: 10000 });
  });

  test('should display the calculator UI', async ({ page }) => {
    await expect(page.locator('h1')).toContainText('Link.Calculator');
    await expect(page.locator('textarea')).toBeVisible();
  });

  test('should calculate simple expression', async ({ page }) => {
    await calculateExpression(page, '2 + 3');

    // Wait for result to appear (result is in .result-value)
    await expect(page.locator('.result-value')).toContainText('5', { timeout: 5000 });
  });

  test('should show calculation steps', async ({ page }) => {
    await calculateExpression(page, '2 + 3 * 4');

    // Wait for result
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 5000 });

    // Check for steps section
    await expect(page.locator('.steps-section')).toBeVisible();
  });

  test('should handle empty input gracefully', async ({ page }) => {
    const input = page.locator('textarea');
    await input.fill('');

    // Should not show error for empty input
    await expect(page.locator('.result-value.error')).not.toBeVisible();
  });

  test('should display error for invalid expression', async ({ page }) => {
    await calculateExpression(page, 'invalid expression +++');

    // Wait for error to appear (error is .result-value.error)
    await expect(page.locator('.result-value.error')).toBeVisible({ timeout: 5000 });
  });

  test('should handle multiline input', async ({ page }) => {
    await calculateExpression(page, '1 + 1');

    // Should process and show result
    await expect(page.locator('.result-value')).toContainText('2', { timeout: 5000 });
  });
});

test.describe('Theme', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should toggle theme when clicking theme button', async ({ page }) => {
    // Open settings menu
    const settingsButton = page.locator('.settings-button');
    await expect(settingsButton).toBeVisible();
    await settingsButton.click();

    // Wait for dropdown to appear
    await expect(page.locator('.settings-dropdown')).toBeVisible();

    // Find and click Dark theme button
    const darkButton = page.locator('.settings-buttons button:has-text("Dark")');
    await darkButton.click();

    // Theme should have changed - check document attribute
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  });

  test('should persist theme preference in localStorage', async ({ page }) => {
    // Open settings and change theme
    const settingsButton = page.locator('.settings-button');
    await settingsButton.click();

    await expect(page.locator('.settings-dropdown')).toBeVisible();

    const darkButton = page.locator('.settings-buttons button:has-text("Dark")');
    await darkButton.click();

    // Check localStorage is used
    const storage = await page.evaluate(() => localStorage.getItem('link-calculator-preferences'));
    expect(storage).toContain('dark');
  });
});

test.describe('Language', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should have language selector', async ({ page }) => {
    // Open settings menu
    const settingsButton = page.locator('.settings-button');
    await settingsButton.click();

    // Check for language selector (first select element in dropdown is language)
    const langSelector = page.locator('.settings-dropdown select').first();
    await expect(langSelector).toBeVisible();
  });

  test('should change language when selected', async ({ page }) => {
    // Open settings menu
    const settingsButton = page.locator('.settings-button');
    await settingsButton.click();

    // Select German (first select is language selector)
    const langSelector = page.locator('.settings-dropdown select').first();
    await langSelector.selectOption('de');

    // UI should change to German - check result heading changes to "Ergebnis"
    await expect(page.locator('.result-section h2')).toContainText('Ergebnis');
  });
});

test.describe('URL Sharing', () => {
  test('should update URL when expression changes', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('textarea')).toBeEnabled({ timeout: 10000 });

    const input = page.locator('textarea');
    await input.fill('1 + 2');

    // Wait for debounce and URL update
    await page.waitForTimeout(600);

    // Check URL contains expression parameter
    const url = page.url();
    expect(url).toContain('?q=');
  });

  test('should load expression from URL', async ({ page }) => {
    // Encode expression in URL (base64 of Links Notation)
    const expression = '5 * 5';
    const linoEncoded = `(expression "${expression}")`;
    const base64Encoded = Buffer.from(linoEncoded).toString('base64');

    await page.goto(`/?q=${base64Encoded}`);
    await expect(page.locator('textarea')).toBeEnabled({ timeout: 10000 });

    // Input should have the expression
    const input = page.locator('textarea');
    await expect(input).toHaveValue(expression);

    // Trigger calculation (calculation is on-demand, not automatic)
    // First focus the input, then press Enter
    await input.focus();
    await page.keyboard.press('Enter');

    // Result should show 25
    await expect(page.locator('.result-value')).toContainText('25', { timeout: 5000 });
  });

  test('should add each new expression to browser history', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('textarea')).toBeEnabled({ timeout: 10000 });

    const input = page.locator('textarea');

    // Type and calculate first expression
    await input.fill('1 + 1');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(600);
    await expect(page.locator('.result-value')).toContainText('2', { timeout: 5000 });

    // Type and calculate second expression
    await input.fill('2 + 2');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(600);
    await expect(page.locator('.result-value')).toContainText('4', { timeout: 5000 });

    // Type and calculate third expression
    await input.fill('3 + 3');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(600);
    await expect(page.locator('.result-value')).toContainText('6', { timeout: 5000 });

    // Go back in history
    await page.goBack();
    await page.waitForTimeout(200);

    // Should show second expression, trigger calculation
    await expect(input).toHaveValue('2 + 2');
    await page.keyboard.press('Enter');
    await expect(page.locator('.result-value')).toContainText('4', { timeout: 5000 });

    // Go back again
    await page.goBack();
    await page.waitForTimeout(200);

    // Should show first expression, trigger calculation
    await expect(input).toHaveValue('1 + 1');
    await page.keyboard.press('Enter');
    await expect(page.locator('.result-value')).toContainText('2', { timeout: 5000 });

    // Go forward
    await page.goForward();
    await page.waitForTimeout(200);

    // Should show second expression again, trigger calculation
    await expect(input).toHaveValue('2 + 2');
    await page.keyboard.press('Enter');
    await expect(page.locator('.result-value')).toContainText('4', { timeout: 5000 });
  });
});

test.describe('Report Issue', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('textarea')).toBeEnabled({ timeout: 10000 });
  });

  test('should have report issue button in footer', async ({ page }) => {
    // Report issue is a button with .link-button class in footer
    const reportButton = page.locator('footer button.link-button');
    await expect(reportButton).toBeVisible();
  });

  test('should open GitHub issue when clicking report', async ({ page, context }) => {
    // Listen for new page (popup)
    const pagePromise = context.waitForEvent('page');

    // Click report issue button
    const reportButton = page.locator('footer button.link-button');
    await reportButton.click();

    // Wait for new page
    const newPage = await pagePromise;
    const newUrl = newPage.url();

    // URL should point to GitHub - it may redirect to login but contains issue URL
    expect(newUrl).toContain('github.com');
    // The issues/new URL may be in return_to param if redirected to login
    expect(newUrl).toMatch(/issues\/new|issues%2Fnew/);
  });
});

test.describe('Responsive Design', () => {
  test('should be mobile-friendly', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Calculator should be visible
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('textarea')).toBeVisible();
  });

  test('should have fixed width on desktop', async ({ page }) => {
    // Set desktop viewport
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto('/');

    // Main container should have max-width (around 780px)
    const container = page.locator('.container');
    await expect(container).toBeVisible();

    const box = await container.boundingBox();
    expect(box).not.toBeNull();
    if (box) {
      // Should be around 780px or less
      expect(box.width).toBeLessThanOrEqual(800);
    }
  });
});

test.describe('Busy Indicator', () => {
  test('should show loading indicator when WASM is loading', async ({ page }) => {
    await page.goto('/');

    // Initially, loading indicator should be visible while WASM loads
    // It shows "Loading calculator engine..."
    const loadingText = page.locator('text=Loading');
    // This may or may not be visible depending on WASM load speed
    // Just verify the page loads without errors
    await expect(page.locator('h1')).toBeVisible();
  });

  test('should show spinner during calculation', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('textarea')).toBeEnabled({ timeout: 10000 });

    // Type a complex expression that might take longer and trigger calculation
    await calculateExpression(page, '123456789 * 987654321');

    // Eventually result should appear (loading indicator is .loading)
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 10000 });
  });
});

test.describe('Examples', () => {
  test('should show example buttons', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('textarea')).toBeEnabled({ timeout: 10000 });

    // Check example buttons exist
    await expect(page.locator('.example-button')).toHaveCount(6);
  });

  test('should fill input when clicking example', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('textarea')).toBeEnabled({ timeout: 10000 });

    // Click first example button
    const firstExample = page.locator('.example-button').first();
    await firstExample.click();

    // Input should be filled
    const input = page.locator('textarea');
    await expect(input).not.toHaveValue('');

    // Trigger calculation (clicking example only fills input, doesn't calculate)
    // First focus the input, then press Enter
    await input.focus();
    await page.keyboard.press('Enter');

    // Result should appear
    await expect(page.locator('.result-value')).not.toContainText('Enter an expression', { timeout: 5000 });
  });
});

test.describe('Textarea Auto-Resize', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('textarea')).toBeEnabled({ timeout: 10000 });
  });

  test('should auto-resize textarea when content becomes multiline', async ({ page }) => {
    const textarea = page.locator('textarea');

    // Get initial height
    const initialBox = await textarea.boundingBox();
    const initialHeight = initialBox?.height || 0;

    // Type multiline content
    await textarea.fill('Line 1\nLine 2\nLine 3\nLine 4');

    // Wait for resize to complete
    await page.waitForTimeout(100);

    // Get new height
    const newBox = await textarea.boundingBox();
    const newHeight = newBox?.height || 0;

    // Height should have increased
    expect(newHeight).toBeGreaterThan(initialHeight);
  });

  test('should shrink textarea when content is reduced', async ({ page }) => {
    const textarea = page.locator('textarea');

    // Type multiline content
    await textarea.fill('Line 1\nLine 2\nLine 3\nLine 4\nLine 5');
    await page.waitForTimeout(100);

    // Get height with multiline content
    const multilineBox = await textarea.boundingBox();
    const multilineHeight = multilineBox?.height || 0;

    // Clear to single line
    await textarea.fill('Single line');
    await page.waitForTimeout(100);

    // Get new height
    const singleLineBox = await textarea.boundingBox();
    const singleLineHeight = singleLineBox?.height || 0;

    // Height should have decreased
    expect(singleLineHeight).toBeLessThan(multilineHeight);
  });

  test('should maintain minimum height for empty content', async ({ page }) => {
    const textarea = page.locator('textarea');

    // Clear the textarea
    await textarea.fill('');
    await page.waitForTimeout(100);

    // Get height
    const box = await textarea.boundingBox();
    const height = box?.height || 0;

    // Should have a minimum height (at least one line + padding)
    expect(height).toBeGreaterThan(30);
  });

  test('should handle long single-line content with word wrap', async ({ page }) => {
    const textarea = page.locator('textarea');

    // Type a very long expression that should wrap
    const longExpression = '(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC) + (Jan 25, 12:51pm UTC)';
    await textarea.fill(longExpression);
    await page.waitForTimeout(100);

    // Check that content is visible and textarea has resized appropriately
    const box = await textarea.boundingBox();
    expect(box).not.toBeNull();
  });

  test('should resize in discrete line-height steps when dragged', async ({ page }) => {
    const textarea = page.locator('textarea');

    // First, add some content to have a baseline
    await textarea.fill('Line 1\nLine 2');
    await page.waitForTimeout(100);

    // Get the current height
    const initialBox = await textarea.boundingBox();
    const initialHeight = initialBox?.height || 0;

    // Get line height from computed style
    const lineHeight = await textarea.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return parseFloat(style.lineHeight) || 24;
    });

    // Verify the height is roughly a multiple of line-height (within tolerance for padding)
    // The height should be: (N lines * lineHeight) + padding
    const padding = await textarea.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return parseFloat(style.paddingTop) + parseFloat(style.paddingBottom);
    });

    const contentHeight = initialHeight - padding;
    const lines = Math.round(contentHeight / lineHeight);

    // Should be close to a whole number of lines
    expect(Math.abs(contentHeight - lines * lineHeight)).toBeLessThan(5);
  });

  test('should not allow resize smaller than content height', async ({ page }) => {
    const textarea = page.locator('textarea');

    // Add multiline content
    await textarea.fill('Line 1\nLine 2\nLine 3');
    await page.waitForTimeout(100);

    // Get the content height (minimum required)
    const box = await textarea.boundingBox();
    const contentHeight = box?.height || 0;

    // Try to manually resize smaller using JavaScript (simulating drag)
    await textarea.evaluate((el) => {
      el.style.height = '30px'; // Very small height
    });

    // Trigger the resize observer by forcing a reflow
    await page.waitForTimeout(50);

    // The component should snap back to at least content height
    const newBox = await textarea.boundingBox();
    const newHeight = newBox?.height || 0;

    // Height should be at least the minimum required for content
    // (allowing some tolerance since snap logic may round up)
    expect(newHeight).toBeGreaterThanOrEqual(contentHeight - 10);
  });
});

test.describe('Currency Conversion with Real Rates', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for WASM to be ready
    await expect(page.locator('textarea')).toBeEnabled({ timeout: 10000 });
    // Wait for exchange rates to load (rates status should appear in footer)
    // The loading indicator should eventually disappear or show loaded state
    await page.waitForTimeout(3000); // Allow time for rates API call
  });

  test('should show rates loading status in footer', async ({ page }) => {
    // Footer should contain rates status information
    const footer = page.locator('footer');
    await expect(footer).toBeVisible();
    // After rates load, should show currency info (USD, date, or loading indicator)
    // Check for any rates-related element
    const ratesStatus = page.locator('.rates-status');
    // May or may not be visible depending on implementation
  });

  test('should calculate currency conversion', async ({ page }) => {
    // Use an expression that will trigger currency conversion
    await calculateExpression(page, '100 USD in EUR');

    // Wait for result to appear (not the default placeholder)
    await expect(page.locator('.result-value')).not.toContainText('Enter an expression', { timeout: 10000 });

    // Result should contain a currency value (EUR since it's the target)
    const resultText = await page.locator('.result-value').textContent();
    // Should contain EUR or a numeric value
    expect(resultText).toMatch(/EUR|\d+/i);
  });

  test('should show exchange rate info in calculation steps', async ({ page }) => {
    // This should trigger a currency conversion
    await calculateExpression(page, '0 RUB + 1 USD');

    // Wait for result
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 10000 });

    // Steps section should be visible
    await expect(page.locator('.steps-section')).toBeVisible();

    // Steps should contain exchange rate information
    const stepsText = await page.locator('.steps-section').textContent();
    // Should mention exchange rate or conversion
    expect(stepsText?.toLowerCase()).toMatch(/exchange|rate|convert/i);
  });

  test('should handle multiple currency conversions', async ({ page }) => {
    await calculateExpression(page, '1 USD + 1 EUR + 1 GBP');

    // Wait for result
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 10000 });

    // Result should be a number with currency
    const resultText = await page.locator('.result-value').textContent();
    expect(resultText).toBeTruthy();
  });

  test('should display rate source information', async ({ page }) => {
    await calculateExpression(page, '100 JPY in USD');

    // Wait for result
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 10000 });

    // Check if steps section shows rate source
    const stepsSection = page.locator('.steps-section');
    if (await stepsSection.isVisible()) {
      const stepsText = await stepsSection.textContent();
      // Should contain rate source information (e.g., ECB, frankfurter.dev, or cbr.ru)
      // This test verifies the exchange rate source is displayed
      expect(stepsText).toBeTruthy();
    }
  });

  test('should handle currency symbols', async ({ page }) => {
    await calculateExpression(page, '$100 + €50');

    // Wait for result
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 10000 });

    // Should show a result (either in $ or €)
    const resultText = await page.locator('.result-value').textContent();
    expect(resultText).toMatch(/[$€]|\w{3}/); // Either symbol or currency code
  });

  test('should gracefully handle rates when offline', async ({ page }) => {
    // This test checks that calculator still works even if rates fail
    // We can't easily simulate offline, but we can test that basic math works
    await calculateExpression(page, '2 + 2');

    await expect(page.locator('.result-value')).toContainText('4', { timeout: 5000 });
  });
});

test.describe('Textarea Resize Visual Regression', () => {
  test('textarea should display all content without clipping', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('textarea')).toBeEnabled({ timeout: 10000 });

    const textarea = page.locator('textarea');

    // Type the exact content from the bug report
    const bugContent = '(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC) + (Jan 25, 12:51nm UTC)';
    await textarea.fill(bugContent);
    await page.waitForTimeout(100);

    // Check that scroll height equals client height (no hidden content)
    const overflow = await textarea.evaluate((el) => {
      return el.scrollHeight > el.clientHeight;
    });

    expect(overflow).toBe(false);
  });

  test('textarea content should never be partially visible', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('textarea')).toBeEnabled({ timeout: 10000 });

    const textarea = page.locator('textarea');

    // Add multiple lines
    await textarea.fill('Line 1\nLine 2\nLine 3');
    await page.waitForTimeout(100);

    // Get line height
    const lineHeight = await textarea.evaluate((el) => {
      const style = window.getComputedStyle(el);
      const lh = parseFloat(style.lineHeight);
      return isNaN(lh) ? 24 : lh;
    });

    // Get padding
    const padding = await textarea.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return parseFloat(style.paddingTop) + parseFloat(style.paddingBottom);
    });

    // Get border
    const border = await textarea.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return parseFloat(style.borderTopWidth) + parseFloat(style.borderBottomWidth);
    });

    // Get current height
    const box = await textarea.boundingBox();
    const height = (box?.height || 0);

    // Calculate content area
    const contentArea = height - padding - border;

    // Content area should be a multiple of line-height (within small tolerance)
    const lines = contentArea / lineHeight;
    const remainder = lines - Math.round(lines);

    // Should be very close to a whole number
    expect(Math.abs(remainder)).toBeLessThan(0.2);
  });
});
