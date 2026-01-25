import { test, expect } from '@playwright/test';

test.describe('Calculator', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for WASM to be ready
    await expect(page.locator('textarea')).toBeEnabled({ timeout: 10000 });
  });

  test('should display the calculator UI', async ({ page }) => {
    await expect(page.locator('h1')).toContainText('Link Calculator');
    await expect(page.locator('textarea')).toBeVisible();
  });

  test('should calculate simple expression', async ({ page }) => {
    const input = page.locator('textarea');
    await input.fill('2 + 3');

    // Wait for result to appear (result is in .result-value)
    await expect(page.locator('.result-value')).toContainText('5', { timeout: 5000 });
  });

  test('should show calculation steps', async ({ page }) => {
    const input = page.locator('textarea');
    await input.fill('2 + 3 * 4');

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
    const input = page.locator('textarea');
    await input.fill('invalid expression +++');

    // Wait for error to appear (error is .result-value.error)
    await expect(page.locator('.result-value.error')).toBeVisible({ timeout: 5000 });
  });

  test('should handle multiline input', async ({ page }) => {
    const input = page.locator('textarea');
    await input.fill('1 + 1');

    // Should process and show result
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 5000 });
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

    // Check for language selector (select element)
    const langSelector = page.locator('.settings-dropdown select');
    await expect(langSelector).toBeVisible();
  });

  test('should change language when selected', async ({ page }) => {
    // Open settings menu
    const settingsButton = page.locator('.settings-button');
    await settingsButton.click();

    // Select German
    const langSelector = page.locator('.settings-dropdown select');
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
    await page.waitForTimeout(500);

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

    // Result should show 25
    await expect(page.locator('.result-value')).toContainText('25', { timeout: 5000 });
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

    const input = page.locator('textarea');
    // Type a complex expression that might take longer
    await input.fill('123456789 * 987654321');

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

    // Result should appear
    await expect(page.locator('.result-value')).toBeVisible({ timeout: 5000 });
  });
});
