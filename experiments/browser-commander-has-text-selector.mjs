// Reproduce: browser-commander normalizes `.parent button:has-text("B")` to an
// unscoped `button:nth-of-type(2)`, so clicking hits the first such button in
// the document instead of the one inside `.parent` (issue #256).
// Usage (from web/): node ../experiments/browser-commander-has-text-selector.mjs
import { chromium } from '@playwright/test';
import { makeBrowserCommander } from 'browser-commander';

const browser = await chromium.launch();
const page = await browser.newPage();
await page.setContent(`
  <nav><button>Home</button><button onclick="window.clicked='nav-2'">About</button></nav>
  <div class="parent">
    <button onclick="window.clicked='A'">A</button>
    <button onclick="window.clicked='B'">B</button>
  </div>`);
const commander = makeBrowserCommander({ page, verbose: false, enableNetworkTracking: false, enableNavigationManager: false });
await commander.clickButton({ selector: '.parent button:has-text("B")' });
const clicked = await page.evaluate(() => window.clicked);
console.log(`expected B, clicked ${clicked}`);
await commander.destroy();
await browser.close();
process.exitCode = clicked === 'B' ? 0 : 1;
