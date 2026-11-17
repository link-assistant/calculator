#!/usr/bin/env node
/**
 * Smoke test of the package entry points (Node and Bun): imports the native
 * API and the wasm-compatible API and evaluates a few expressions.
 *
 *   node scripts/smoke.mjs
 *   bun scripts/smoke.mjs
 */

import assert from 'node:assert/strict';
import { Calculator, VERSION } from '../src/index.js';
import init, { Calculator as WasmCalculator, get_version, health_check, parse_consolidated_lino_rates } from '../src/wasm.js';

const runtime = typeof globalThis.Bun === 'undefined' ? `node ${process.version}` : `bun ${globalThis.Bun.version}`;

// Native API.
const calculator = Calculator.new();
/** @type {[string, string][]} */
const cases = [
  ['2 + 3', '5'],
  ['(1 + 2) * 4', '12'],
  ['1/3', '0.3333333333333333'],
  ['2^10', '1024'],
  ['10 m in cm', '1000 cm'],
];
for (const [input, expected] of cases) {
  const result = calculator.calculateInternal(input);
  assert.equal(result.success, true, `${input}: ${result.error}`);
  assert.equal(result.result, expected, input);
}
assert.equal(calculator.calculateInternal('1/3').fraction, '1/3');
assert.equal(calculator.calculateInternal('2 +').success, false);
assert.equal(Calculator.version(), VERSION);

// wasm-compatible API, as web/src/worker.ts uses it.
await init();
assert.equal(health_check(), true);
assert.equal(get_version(), VERSION);
const wasm = new WasmCalculator();
wasm.set_timezone_offset(0);
const plan = JSON.parse(wasm.plan('100 USD in EUR'));
assert.equal(plan.success, true);
assert.deepEqual(plan.required_sources, ['ecb']);
assert.equal(wasm.update_rates_from_api('USD', '2026-01-25', '{"eur": 0.5}'), 1);
const converted = JSON.parse(wasm.execute('100 USD in EUR'));
assert.equal(converted.success, true, converted.error);
assert.equal(converted.result, '50 EUR');
assert.equal(JSON.parse(wasm.calculate('6 * 7')).result, '42');
const consolidated = JSON.parse(parse_consolidated_lino_rates('conversion:\n  from USD\n  to RUB\n  rates:\n    2026-01-25 76.03\n'));
assert.equal(consolidated.success, true);
wasm.clear_timezone_offset();
wasm.free();

console.log(`smoke: ok (${runtime}, version ${VERSION})`);
