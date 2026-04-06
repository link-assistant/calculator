/**
 * Shared Playwright fixtures that provide browser-commander integration.
 *
 * Usage:
 *   import { test, expect } from './fixtures';
 *   // All tests get a `commander` fixture wrapping the Playwright page with
 *   // browser-commander for consistent browser control.
 */

import { test as base, expect } from '@playwright/test';
import { makeBrowserCommander } from 'browser-commander';
import type { Page } from '@playwright/test';

export type BrowserCommander = ReturnType<typeof makeBrowserCommander>;

export type Fixtures = {
  /** browser-commander instance wrapping the Playwright page */
  commander: BrowserCommander;
};

export const test = base.extend<Fixtures>({
  commander: async ({ page }, use) => {
    const commander = makeBrowserCommander({
      page,
      verbose: false,
      enableNetworkTracking: false,
      enableNavigationManager: false,
    });
    await use(commander);
    await commander.destroy();
  },
});

export { expect };

/**
 * Wait for the WASM calculator to finish loading.
 * Polls until the textarea is enabled, then waits for exchange rates to settle.
 */
export async function waitForWasm(page: Page, { ratesMs = 2000 } = {}) {
  await page.waitForSelector('textarea', { state: 'attached', timeout: 15000 });
  await page.waitForFunction(
    () => {
      const el = document.querySelector('textarea');
      return el && !el.disabled;
    },
    { timeout: 15000 },
  );
  if (ratesMs > 0) {
    await page.waitForTimeout(ratesMs);
  }
}

/**
 * Fill the calculator input and trigger calculation.
 */
export async function calculate(commander: BrowserCommander, expression: string) {
  await commander.fillTextArea({ selector: 'textarea', text: expression });
  await commander.pressKey({ key: 'Enter' });
}
