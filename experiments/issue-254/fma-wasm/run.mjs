// Calls the wasm32 build of `f64::mul_add` on the libm-fma-sticky inputs.
import { readFileSync } from 'node:fs';

const wasm = new URL(
  '../../../target/wasm32-unknown-unknown/release/fma_wasm.wasm',
  import.meta.url,
);
const { instance } = await WebAssembly.instantiate(readFileSync(wasm));
const x = 1 + 2 ** -26;
const y = 1 - 2 ** -26 + 2 ** -52;
const z = 2 ** 53;
const result = instance.exports.mul_add(x, y, z);
console.log(`wasm32 f64::mul_add = ${result}`);
console.log(`expected (2^53 + 2) = ${z + 2}`);
console.log(result === z + 2 ? 'OK: correctly rounded' : 'BUG: the sticky bit was lost');
