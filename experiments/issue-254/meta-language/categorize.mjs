// usage: node categorize.mjs census.json  -> prints category counts + examples
import { readFileSync } from 'node:fs';
const rows = JSON.parse(readFileSync(process.argv[2], 'utf8'));
const rules = [
  [/^function value |unknown name crate\./, 'reads/calls a sibling or imported item that is itself carried (cascade)', 'CASCADE'],
  [/^export "class"|throw new \w+: throw an Error/, 'exported classes / custom Error subclasses', '#210'],
  [/^export "[{*]"/, 're-exports export { a } from / export * from', 'NEW-G'],
  [/^assignment of [A-Z_a-z]+: only local variables/, 'module-level mutable state (top-level let assigned in a function)', 'NEW-H'],
  [/namespace property/, 'frozen constant namespaces with non-method values', '#206'],
  [/startsWith\(\) with 2/, 'string methods / length', '#209'],
  [/destructured parameter/, 'statement gaps (switch on strings, try, typeof, let, destructuring)', '#212'],
  [/expected string but found float/, 'heterogeneous array literal (tuple)', '#205'],
  [/used as a string and as an array/, 'type inferred from one body', '#213'],
  [/unknown type crate\.never|import\(/, 'JSDoc generic / imported typedef type', 'NEW-C'],
  [/unknown type crate\.\w+/, 'JSDoc union of non-null types (incl. literal unions)', 'NEW-B'],
  [/comment the target cannot hold/, 'free-standing /** */ comment', 'NEW-A'],
  [/^export \w+: only function and const/, 'export class / non-function export', '#210'],
  [/new expression|top-level class|class /, 'class / new', '#210'],
  [/JSDoc type \{[^}]*null|^null|undefined|optional chaining|\?\?/, 'null / undefined / Option', '#204'],
  [/JSDoc type \{\[|JSDoc type \{\(|JSDoc type \{\{|Map<|Set<|Record<|@param needs/, 'JSDoc tuple/function/record type, [opt] param', '#205'],
  [/JSDoc type \{[^}]*\|/, 'JSDoc union of non-null types (incl. literal unions)', 'NEW-B'],
  [/JSDoc type \{(unknown|object|\*|any)\}/, 'JSDoc unknown/object/any', '#205'],
  [/JSDoc type \{[A-Z]\w*\}|JSDoc type \{\w+<|unknown type/, 'JSDoc generic / imported typedef type', 'NEW-C'],
  [/object without a \$ tag|object literal|Object\./, 'plain object records', '#206'],
  [/arrow function|function expression|function value/, 'functions/values not portable (arrow, fn value, namespace const)', '#207'],
  [/method call \.(map|filter|find|some|every|push|join|reduce|forEach|includes|indexOf|sort|reverse|concat|fill)\(|Array\./, 'array methods', '#208'],
  [/method call|length of a string|String\.|for…of over a string/, 'string methods / length', '#209'],
  [/regular expression|malformed number|unterminated|escape/, 'regex / lexer', '#211'],
  [/case test|try statement|typeof|for with const|without a value|array destructuring|destructuring/, 'statement gaps (switch on strings, try, typeof, let, destructuring)', '#212'],
  [/bitwise|shift/, 'bitwise and shift operators', 'NEW-D'],
  [/exponentiation/, 'exponentiation **', 'NEW-E'],
  [/^(Number|BigInt|Math|BigInt64Array|Float64Array|DataView|Symbol|Infinity|NaN)\b|standard library/, 'JS standard library globals (Number.*, BigInt(), Math.*, typed arrays)', 'NEW-F'],
  [/toString with a radix/, 'toString(radix)', 'NEW-F'],
  [/async|await/, 'async', '#223'],
  [/^import \{|unknown name crate\.|unknown name/, 'unbound import / sibling name (cascade)', '#220'],
  [/getter|setter|static|this\b/, 'class members', '#210'],
];
const out = {}; const ex = {};
for (const r of rows) {
  const msg = r.message ?? r.reason;
  const rule = rules.find(([re]) => re.test(msg)) ?? [null, `other: ${r.reason}: ${msg}`, '?'];
  const key = `${rule[2]}\t${rule[1]}`; out[key] = (out[key] ?? 0) + 1; (ex[key] ??= []).push(`${r.module} ${r.name}: ${msg}`);
}
for (const [k, n] of Object.entries(out).sort((a, b) => b[1] - a[1])) { console.log(String(n).padStart(4), k); if (process.argv[3]) for (const e of ex[k].slice(0, +process.argv[3])) console.log('       ', e.slice(0, 160)); }
console.log(rows.length, 'carried items');
