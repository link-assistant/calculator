/**
 * Screenshot generation script using browser-commander.
 *
 * Takes full-page screenshots of all calculator use cases and saves them to
 * docs/use-cases/ and docs/screenshots/ directories.
 *
 * Usage:
 *   node web/scripts/take-screenshots.mjs
 *
 * Requires:
 *   - A running web server at http://localhost:4173 (npm run preview)
 *   - browser-commander and playwright installed as dependencies
 */

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { launchBrowser, makeBrowserCommander } from 'browser-commander';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const BASE_URL = process.env.BASE_URL || 'http://localhost:4173';
const USE_CASES_DIR = path.resolve(__dirname, '../../docs/use-cases');
const SCREENSHOTS_DIR = path.resolve(__dirname, '../../docs/screenshots');

fs.mkdirSync(USE_CASES_DIR, { recursive: true });
fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });

/**
 * Poll until conditionFn returns true or timeout is reached.
 * @param {import('playwright').Page} page
 * @param {() => Promise<boolean>} conditionFn
 * @param {number} timeoutMs
 */
async function waitFor(page, conditionFn, timeoutMs = 10000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await conditionFn()) return;
    await page.waitForTimeout(200);
  }
  throw new Error(`Condition not met within ${timeoutMs}ms`);
}

/**
 * Take a full-page screenshot via commander.page (official escape hatch).
 * @param {Object} commander
 * @param {string} outputPath
 */
async function screenshot(commander, outputPath) {
  await commander.page.screenshot({ path: outputPath, fullPage: true });
  console.log(`  ✓ ${path.relative(process.cwd(), outputPath)}`);
}

async function main() {
  console.log(`Taking screenshots from ${BASE_URL} ...`);

  // Use a temporary user data dir so each run is isolated
  const userDataDir = path.join(os.tmpdir(), `calculator-screenshots-${Date.now()}`);

  const { browser, page } = await launchBrowser({
    engine: 'playwright',
    headless: true,
    userDataDir,
    slowMo: 0,
    verbose: false,
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
  });

  // Set a consistent viewport for screenshots
  await page.setViewportSize({ width: 1280, height: 800 });

  const commander = makeBrowserCommander({
    page,
    verbose: false,
    enableNetworkTracking: false,
    enableNavigationManager: false,
  });

  try {
    // ── Helper: navigate and wait for WASM to be ready ──────────────────────
    async function loadPage() {
      await page.goto(BASE_URL);
      // Wait for the textarea to become enabled (WASM loaded)
      await page.waitForSelector('textarea', { state: 'attached', timeout: 15000 });
      await waitFor(
        page,
        async () => {
          const disabled = await page.$eval('textarea', (el) => el.disabled);
          return !disabled;
        },
        15000,
      );
      // Allow exchange rates to load
      await page.waitForTimeout(2000);
    }

    // ── 01 Initial State ─────────────────────────────────────────────────────
    console.log('01-initial-state');
    await loadPage();
    await screenshot(commander, path.join(USE_CASES_DIR, '01-initial-state.png'));

    // ── 02 Simple Arithmetic ─────────────────────────────────────────────────
    console.log('02-simple-arithmetic');
    await loadPage();
    await commander.fillTextArea({ selector: 'textarea', text: '2 + 3' });
    await commander.keyboard.press('Enter');
    await waitFor(page, async () => {
      const el = await page.$('.result-value');
      const text = el ? await el.textContent() : '';
      return (text || '').includes('5');
    }, 5000);
    await screenshot(commander, path.join(USE_CASES_DIR, '02-simple-arithmetic.png'));

    // ── 03 Currency Conversion ───────────────────────────────────────────────
    console.log('03-currency-conversion');
    await loadPage();
    await commander.fillTextArea({ selector: 'textarea', text: '84 USD - 34 EUR' });
    await commander.keyboard.press('Enter');
    await waitFor(page, async () => {
      const el = await page.$('.result-value');
      const text = el ? await el.textContent() : '';
      return !!(text && !text.includes('Enter an expression'));
    }, 10000);
    await screenshot(commander, path.join(USE_CASES_DIR, '03-currency-conversion.png'));

    // ── 04 Integration ───────────────────────────────────────────────────────
    console.log('04-integration');
    await loadPage();
    await commander.fillTextArea({ selector: 'textarea', text: 'integrate sin(x)/x dx' });
    await commander.keyboard.press('Enter');
    await waitFor(page, async () => !!(await page.$('.result-value')), 10000);
    await page.waitForTimeout(1000);
    await screenshot(commander, path.join(USE_CASES_DIR, '04-integration.png'));

    // ── 05 Dark Theme ────────────────────────────────────────────────────────
    console.log('05-dark-theme');
    await loadPage();
    await commander.fillTextArea({ selector: 'textarea', text: 'integrate sin(x)/x dx' });
    await commander.keyboard.press('Enter');
    await waitFor(page, async () => !!(await page.$('.result-value')), 10000);
    await page.waitForTimeout(500);
    await commander.clickButton({ selector: '.settings-button' });
    await waitFor(page, async () => !!(await page.$('.settings-dropdown')), 3000);
    await commander.clickButton({ selector: '.settings-buttons button:has-text("Dark")' });
    await waitFor(page, async () => {
      const attr = await page.$eval('html', (el) => el.getAttribute('data-theme'));
      return attr === 'dark';
    }, 3000);
    await screenshot(commander, path.join(USE_CASES_DIR, '05-dark-theme.png'));

    // ── 06 Definite Integral ─────────────────────────────────────────────────
    console.log('06-definite-integral');
    await loadPage();
    await commander.clickButton({ selector: '.settings-button' });
    await waitFor(page, async () => !!(await page.$('.settings-dropdown')), 3000);
    await commander.clickButton({ selector: '.settings-buttons button:has-text("Dark")' });
    await commander.keyboard.press('Escape');
    await commander.fillTextArea({ selector: 'textarea', text: 'integrate(x^2, x, 0, 3)' });
    await commander.keyboard.press('Enter');
    await waitFor(page, async () => {
      const el = await page.$('.result-value');
      const text = el ? await el.textContent() : '';
      return (text || '').includes('9');
    }, 5000);
    await screenshot(commander, path.join(USE_CASES_DIR, '06-definite-integral.png'));

    // ── 07 Math Functions ────────────────────────────────────────────────────
    console.log('07-math-functions');
    await loadPage();
    await commander.clickButton({ selector: '.settings-button' });
    await waitFor(page, async () => !!(await page.$('.settings-dropdown')), 3000);
    await commander.clickButton({ selector: '.settings-buttons button:has-text("Dark")' });
    await commander.keyboard.press('Escape');
    await commander.fillTextArea({ selector: 'textarea', text: 'sqrt(16) + pow(2, 3)' });
    await commander.keyboard.press('Enter');
    await waitFor(page, async () => {
      const el = await page.$('.result-value');
      const text = el ? await el.textContent() : '';
      return (text || '').includes('12');
    }, 5000);
    await screenshot(commander, path.join(USE_CASES_DIR, '07-math-functions.png'));

    // ── 08 Date/Time ─────────────────────────────────────────────────────────
    console.log('08-datetime');
    await loadPage();
    await commander.fillTextArea({ selector: 'textarea', text: '(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)' });
    await commander.keyboard.press('Enter');
    await waitFor(page, async () => {
      const el = await page.$('.result-value');
      const text = el ? await el.textContent() : '';
      return (text || '').toLowerCase().includes('day');
    }, 5000);
    await screenshot(commander, path.join(USE_CASES_DIR, '08-datetime.png'));

    // ── 09 Parentheses ───────────────────────────────────────────────────────
    console.log('09-parentheses');
    await loadPage();
    await commander.fillTextArea({ selector: 'textarea', text: '(2 + 3) * 4' });
    await commander.keyboard.press('Enter');
    await waitFor(page, async () => {
      const el = await page.$('.result-value');
      const text = el ? await el.textContent() : '';
      return (text || '').includes('20');
    }, 5000);
    await screenshot(commander, path.join(USE_CASES_DIR, '09-parentheses.png'));

    // ── Copy selected screenshots to docs/screenshots/ ────────────────────
    const copies = [
      ['01-initial-state.png', 'calculator-main.png'],
      ['02-simple-arithmetic.png', 'calculator-arithmetic.png'],
      ['03-currency-conversion.png', 'calculator-currency.png'],
      ['08-datetime.png', 'calculator-datetime.png'],
      ['09-parentheses.png', 'calculator-parentheses.png'],
    ];
    for (const [src, dest] of copies) {
      const srcPath = path.join(USE_CASES_DIR, src);
      const destPath = path.join(SCREENSHOTS_DIR, dest);
      if (fs.existsSync(srcPath)) {
        fs.copyFileSync(srcPath, destPath);
        console.log(`  ✓ copied ${dest}`);
      }
    }

    console.log('\nAll screenshots taken successfully.');
  } finally {
    await commander.destroy();
    await browser.close();
    // Clean up temporary user data dir
    fs.rmSync(userDataDir, { recursive: true, force: true });
  }
}

main().catch((err) => {
  console.error('Screenshot script failed:', err);
  process.exit(1);
});
