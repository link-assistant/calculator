## Problem

`normalizeSelector` (`src/elements/selectors.js`) turns a Playwright text selector such as `.parent button:has-text("B")` into a CSS selector. It does this by finding the element and returning only `${tagName}:nth-of-type(${index + 1})`. The ancestor part of the selector (`.parent`) is dropped.

`clickButton` and the other interactions then act on the **first element in the whole document** that matches `button:nth-of-type(2)`, which may be a completely different button. The Puppeteer text-selector branch builds the same unscoped selector.

Downstream, link-assistant/calculator's e2e test clicked a navigation button instead of the intended button inside its container.

## Reproduction (browser-commander 0.31.1, latest)

```js
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
console.log(`expected B, clicked ${await page.evaluate(() => window.clicked)}`);
await commander.destroy();
await browser.close();
```

```console
$ node repro.mjs
expected B, clicked nav-2
```

Script: https://github.com/link-assistant/calculator/blob/issue-256-9c021bdb8c9d/experiments/browser-commander-has-text-selector.mjs

## Fix

In order of preference:

1. For `engine === 'playwright'`, do not rewrite the selector at all. Playwright locators support `:has-text()` and `:text-is()` natively, so `page.locator(selector)` is exact.
2. Otherwise, build a selector that is unique in the document. One option is the ancestor path with `:nth-of-type` at each level up to an `id` or `body`; `src/traces/page-capture.js` already builds such a path for traces. Another is to tag the matched element with a temporary `data-*` attribute and return `[data-…="…"]`.
3. Add a test with two containers whose buttons share a tag and an index.

## Workaround

Target elements with a unique attribute (`[data-testid="…"]`), or use `page.locator('.parent button:has-text("B")').click()` directly.

Found while working on link-assistant/calculator#256 (commit 2c7978f7).
