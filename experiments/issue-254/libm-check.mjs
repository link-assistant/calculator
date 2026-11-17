// Checks the JavaScript libm port (js/src/rust/libm.js) against every vector in
// js/test/vectors/libm.json, bit for bit (NaN only as a class). Runs on Node 20+
// and Bun:
//
//   node experiments/issue-254/libm-check.mjs
//   ~/.bun/bin/bun experiments/issue-254/libm-check.mjs
//   npx -y node@20 experiments/issue-254/libm-check.mjs
//
// Exits with status 1 if any vector differs.

import { readFileSync } from 'node:fs';
import * as libm from '../../js/src/rust/libm.js';

const data = JSON.parse(readFileSync(new URL('../../js/test/vectors/libm.json', import.meta.url), 'utf8'));

const F64 = new Float64Array(1);
const U64 = new BigUint64Array(F64.buffer);

/** @param {string} hex */
function fromHex(hex) {
  U64[0] = BigInt(`0x${hex}`);
  return F64[0];
}

/** @param {number} x */
function toHex(x) {
  F64[0] = x;
  return U64[0].toString(16).padStart(16, '0');
}

/** @param {number} actual @param {string} expected */
function same(actual, expected) {
  const want = fromHex(expected);
  return Number.isNaN(want) ? Number.isNaN(actual) : toHex(actual) === expected;
}

const runtime = typeof globalThis.Bun !== 'undefined'
  ? `Bun ${globalThis.Bun.version}`
  : `Node ${process.versions.node} (V8 ${process.versions.v8})`;
console.log(`runtime: ${runtime}`);

let total = 0;
let failed = 0;
const start = performance.now();
for (const [name, vectors] of Object.entries(data.functions)) {
  const f = libm[name];
  if (typeof f !== 'function') {
    console.log(`${name}: no export`);
    failed += vectors.length;
    continue;
  }
  let bad = 0;
  const examples = [];
  for (const v of vectors) {
    const args = v.slice(0, -2).map(fromHex);
    const got = f(...args);
    if (!same(got, v[v.length - 2])) {
      bad += 1;
      if (examples.length < 3) {
        examples.push(`(${v.slice(0, -2).join(', ')}) -> ${toHex(got)}, libm ${v[v.length - 2]}`);
      }
    }
  }
  total += vectors.length;
  failed += bad;
  console.log(`${name.padEnd(6)} ${String(vectors.length).padStart(5)} vectors, ${bad} mismatches`);
  for (const e of examples) {
    console.log(`  ${e}`);
  }
}
const ms = (performance.now() - start).toFixed(0);
console.log(`total: ${total} vectors, ${failed} mismatches (${ms} ms)`);
process.exitCode = failed === 0 ? 0 : 1;
