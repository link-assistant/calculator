// usage: node repro.mjs <ml js/src>
import { join } from 'node:path'; import { pathToFileURL } from 'node:url';
const ml = await import(pathToFileURL(join(process.argv[2], 'self-translation.js')));
const { parseJavaScript } = await import(pathToFileURL(join(process.argv[2], 'translation/javascript.js')));
const { checkProgram } = await import(pathToFileURL(join(process.argv[2], 'translation/check.js')));
const cases = {
  A_module_doc_comment: `/**\n * Utility helpers.\n */\n\n/** @param {number} x @returns {number} */\nexport function twice(x) { return x * 2; }\n`,
  A_typedef_import_comment: `/** @typedef {import('./m.js').Thing} Thing */\n\n/** @param {number} x @returns {number} */\nexport function twice(x) { return x * 2; }\n`,
  B_union_number_bigint: `/** @param {number | bigint} x @returns {number} */\nexport function toNumber(x) { return 0; }\n`,
  B_string_literal_union: `/** @param {'Hours' | 'Minutes'} unit @returns {number} */\nexport function scale(unit) { return unit === 'Hours' ? 3600 : 60; }\n`,
  B_number_literal_union: `/** @param {number} a @param {number} b @returns {-1 | 0 | 1} */\nexport function cmp(a, b) { return a < b ? -1 : a > b ? 1 : 0; }\n`,
  C_import_type: `/** @param {import('./m.js').Thing} t @returns {number} */\nexport function f(t) { return 1; }\n`,
  C_never: `/** @param {string} m @returns {never} */\nexport function fail(m) { throw new Error('unreachable'); }\n`,
  D_bitwise: `/** @param {number} x @returns {number} */\nexport function low(x) { return x & 0xff; }\n`,
  D_shift: `/** @param {number} x @returns {number} */\nexport function hi(x) { return x >>> 16; }\n`,
  D_bigint_shift: `/** @param {bigint} x @returns {bigint} */\nexport function hi64(x) { return (x >> 32n) & 0xffffffffn; }\n`,
  E_pow: `export const I64_MAX = 2n ** 63n - 1n;\n`,
  E_pow_number: `/** @param {number} x @returns {number} */\nexport function sq(x) { return x ** 2; }\n`,
  F_BigInt_call: `/** @param {number} x @returns {bigint} */\nexport function big(x) { return BigInt(x); }\n`,
  F_Number_call: `/** @param {bigint} x @returns {number} */\nexport function num(x) { return Number(x); }\n`,
  F_toString_radix: `/** @param {bigint} x @returns {number} */\nexport function bits(x) { return x.toString(2).length; }\n`,
  G_reexport: `export { twice } from './m.js';\n`,
  G_reexport_star: `export * from './m.js';\n`,
  H_module_state: `let cache = 0;\n/** @returns {number} */\nexport function next() { cache = cache + 1; return cache; }\n`,
  I_destructured_param: `/** @param {{a: number, b: number}} p @returns {number} */\nexport function sum({ a, b }) { return a + b; }\n`,
};
for (const [name, src] of Object.entries(cases)) {
  const r = ml.selfTranslate(src, 'JavaScript', 'Rust');
  const items = r.items.map((i) => `${i.term}:${i.status}${i.reason ? `(${i.reason})` : ''}`).join(' ');
  let diag = '';
  if (r.items.some((i) => ["unsupported","type","syntax"].includes(i.reason))) {
    try { checkProgram(parseJavaScript(src)); } catch (e) { diag += ` | ${e.message}`; }
  }
  console.log(`${name}: ${items}${diag}`);
}
