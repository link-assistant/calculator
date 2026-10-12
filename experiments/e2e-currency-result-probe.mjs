// Probe what `.result-value` shows over time after a currency calculation, to
// understand why the currency e2e tests read an empty string (issue #256).
// Usage: start `npm run preview` in web/, then from web/:
//   node ../experiments/e2e-currency-result-probe.mjs ["100 USD in EUR"]
import { chromium } from '@playwright/test';

const expression = process.argv[2] ?? '100 USD in EUR';
const browser = await chromium.launch();
const page = await browser.newPage();
page.on('console', (msg) => console.log(`[console.${msg.type()}] ${msg.text()}`));
await page.goto('http://localhost:4173/');
await page.waitForFunction(() => !document.querySelector('textarea')?.disabled);
await page.waitForTimeout(3000);
await page.locator('textarea').fill(expression);
await page.keyboard.press('Enter');
for (let i = 0; i < 10; i++) {
  const html = await page.locator('.result-section').innerHTML().catch(() => '(none)');
  console.log(`t=${i * 300}ms ${html.replace(/\s+/g, ' ').slice(0, 400)}`);
  await page.waitForTimeout(300);
}
await browser.close();
