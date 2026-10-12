// After `wasm-pack build --target web --out-dir web/public/pkg`, run:
// node experiments/verify_number_separators_wasm.mjs
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import init, { Calculator } from '../web/public/pkg/link_calculator.js';

await init({ module_or_path: readFileSync(new URL('../web/public/pkg/link_calculator_bg.wasm', import.meta.url)) });
const calculator = new Calculator();
try {
  for (const [input, expected] of [
    ['3,000 minus 12', '2988'],
    ['1,000 divided by 200', '5'],
    ['100,000 + 200,000', '300000'],
    ['56.7% of 1,234 participants', '699.678 participants'],
    ['1,1', '11'],
    ['12,3 ± 4,5', '12.3 ± 4.5'],
    ['1,08 Mrd. km/h', '1080000000 km/h'],
    ['max(1,2)', '2'],
    ['0b1e10', '4'],
  ]) {
    const result = JSON.parse(calculator.execute(input));
    assert.equal(result.success, true, `${input}: ${result.error}`);
    assert.equal(result.result, expected, input);
    console.log(`${input} = ${result.result}`);
  }
  assert.equal(JSON.parse(calculator.execute('5 kg * 9.8 m/s^2')).success, false);
} finally {
  calculator.free();
}
