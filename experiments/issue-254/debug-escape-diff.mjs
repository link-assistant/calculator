// Lists the code points where js/src/rust/str.js escapes differently from
// Rust's `{:?}` (js/test/vectors/std.json), grouped into ranges with their
// general category, to tune the `is_printable` emulation.
import { readFileSync } from 'node:fs';
import { debugChar, debugStr } from '../../js/src/rust/str.js';

const vectors = JSON.parse(readFileSync(new URL('../../js/test/vectors/std.json', import.meta.url), 'utf8'));
const name = process.argv[2] ?? 'debug_str_escaped';
const escaped = new Set();
for (const [from, to] of vectors.chars[name]) {
  for (let code = from; code <= to; code += 1) escaped.add(code);
}
const jsEscapes = name === 'debug_str_escaped'
  ? (c) => debugStr(c) !== `"${c}"`
  : (c) => debugChar(c) !== `'${c}'`;
const categories = ['Lu', 'Ll', 'Lt', 'Lm', 'Lo', 'Mn', 'Mc', 'Me', 'Nd', 'Nl', 'No', 'Pc', 'Pd', 'Ps', 'Pe', 'Pi', 'Pf', 'Po', 'Sm', 'Sc', 'Sk', 'So', 'Zs', 'Zl', 'Zp', 'Cc', 'Cf', 'Cs', 'Co', 'Cn'];
const category = (c) => categories.find((gc) => new RegExp(`^\\p{gc=${gc}}$`, 'u').test(c));
const diffs = [];
for (let code = 0; code <= 0x10ffff; code += 1) {
  if (code >= 0xd800 && code <= 0xdfff) continue;
  const c = String.fromCodePoint(code);
  const rust = escaped.has(code);
  const js = jsEscapes(c);
  if (rust !== js) {
    const key = `${rust ? 'rust-only' : 'js-only'} ${category(c)}`;
    const last = diffs[diffs.length - 1];
    if (last && last.key === key && last.to === code - 1) last.to = code;
    else diffs.push({ key, from: code, to: code });
  }
}
const summary = {};
for (const d of diffs) summary[d.key] = (summary[d.key] ?? 0) + d.to - d.from + 1;
console.log(summary);
for (const d of diffs.slice(0, Number(process.argv[3] ?? 60))) {
  console.log(d.key, d.from.toString(16), d.to.toString(16));
}
