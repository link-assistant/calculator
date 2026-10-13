import { describe, expect, test } from 'vitest';
import {
  ISSUE_URL,
  cascadeOf,
  compareAllowlist,
  construct,
  criticalPath,
  itemName,
  mergeAllowlist,
  moduleDirectory,
  parseAllowlist,
  parseLedger,
  partName,
  patchSelfTranslation,
  planModules,
  readHeader,
  regressions,
  renderAllowlist,
  renderLedger,
  requiredEntries,
  sha256,
  splitOutput,
  suggestUpstream,
  uniqueNames,
  validUpstream,
} from '../scripts/translate-rust-lib.mjs';
import { verify } from '../scripts/translate-rust.mjs';

describe('translate-rust (js/scripts/translate-rust-lib.mjs)', () => {
  test('the committed Rust, ledger and allowlist match js/src (--verify)', () => {
    expect(verify()).toEqual([]);
  });

  test('plans modules in import order and binds only earlier modules', () => {
    const sources = new Map([
      ['a.js', "import { b } from './b.js';\nexport const a = 1;"],
      ['b.js', "import { c } from './sub/c.js';\nimport { a } from './a.js';"],
      ['sub/c.js', "export * from '../d.js';"],
      ['d.js', ''],
    ]);
    const { order, deps } = planModules(sources);
    expect(order).toEqual(['d.js', 'sub/c.js', 'b.js', 'a.js']);
    expect(deps.get('b.js')).toEqual([{ specifier: './sub/c.js', module: 'sub/c.js' }]);
    expect(deps.get('a.js')).toEqual([{ specifier: './b.js', module: 'b.js' }]);
    expect(deps.get('sub/c.js')).toEqual([{ specifier: '../d.js', module: 'd.js' }]);
  });

  test('ranks the critical path first', () => {
    const deps = new Map([['d', []], ['c', [{ module: 'd' }]], ['b', []], ['a', [{ module: 'c' }, { module: 'b' }]]]);
    const rank = criticalPath(['d', 'c', 'b', 'a'], deps, () => 1);
    expect(Object.fromEntries(rank)).toEqual({ a: 1, c: 2, b: 2, d: 3 });
  });

  test('module directories and part names', () => {
    expect(moduleDirectory('index.js')).toEqual([]);
    expect(moduleDirectory('rust/libm/log.js')).toEqual(['rust', 'libm']);
    expect(partName('rust/decimal.js', 0)).toBe('rust/decimal.rs');
    expect(partName('rust/decimal.js', 1)).toBe('rust/decimal.part2.rs');
  });

  test('splits output at block markers, under the line limit, losslessly', () => {
    const blocks = Array.from({ length: 30 }, (_, i) => `// meta-language:carried JavaScript lexical_declaration (unsupported)\n${'// | x\n'.repeat(i % 7)}`);
    const code = `// header\n\n${blocks.join('\n')}\n`;
    const parts = splitOutput(code, 20);
    expect(parts.join('')).toBe(code);
    expect(parts.length).toBeGreaterThan(1);
    for (const part of parts) expect(part.split('\n').length).toBeLessThanOrEqual(20);
    for (const part of parts.slice(1)) expect(part.startsWith('// meta-language:carried')).toBe(true);
    expect(splitOutput('a\nb\n', 20)).toEqual(['a\nb\n']);
    const long = `${'x\n'.repeat(50)}`;
    expect(splitOutput(long, 10).join('')).toBe(long);
  });

  test('reads the self-translation header', () => {
    const hash = 'a'.repeat(64);
    expect(readHeader(`// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=${hash} bytes=12\n`)).toEqual({ sha256: hash, bytes: 12 });
    expect(readHeader('fn main() {}')).toBeNull();
  });

  test('names items', () => {
    expect(itemName('export function foo(a) {}', 'export_statement')).toBe('foo');
    expect(itemName('export async function bar() {}', 'export_statement')).toBe('bar');
    expect(itemName('const X = 1;', 'lexical_declaration')).toBe('X');
    expect(itemName('export class Panic extends Error {}', 'export_statement')).toBe('Panic');
    expect(itemName("export { a, b } from './m.js';", 'export_statement')).toBe('export{}:./m.js');
    expect(itemName("export * from './m.js';", 'export_statement')).toBe('export*:./m.js');
    expect(itemName("import { a } from './m.js';", 'import_statement')).toBe('import:./m.js');
    expect(itemName('CACHE = build();', 'expression_statement')).toBe('expression_statement:CACHE');
    expect(uniqueNames([{ text: 'let x;', term: 't' }, { text: 'let x = 1;', term: 't' }])).toEqual(['x', 'x#2']);
  });

  test('normalises diagnostics and finds cascades', () => {
    expect(construct('bitwise &: bitwise operators are outside the portable core at 34..35', 'unsupported'))
      .toBe('bitwise &: bitwise operators are outside the portable core');
    expect(construct('export "{": only function and const declarations are exported', 'unsupported'))
      .toBe('export {: only function and const declarations are exported');
    expect(construct('import("./m.js").Value', 'type')).toBe('import(X).Value');
    expect(construct(null, 'top-level statement')).toBe('top-level statement');
    expect(cascadeOf('unknown name crate.combineWords')).toBe('combineWords');
    expect(cascadeOf('function value F64: functions and namespaces are only portable when a function is called')).toBe('F64');
    expect(cascadeOf('malformed number 9a')).toBeNull();
  });

  test('suggests upstream issues', () => {
    expect(suggestUpstream('new expression: outside the portable core')).toBe(ISSUE_URL(210));
    expect(suggestUpstream('bitwise &: bitwise operators are outside the portable core')).toBe('drafted:D');
    expect(suggestUpstream('malformed number 9a')).toBe(ISSUE_URL(211));
    expect(suggestUpstream('comment the target cannot hold')).toBe('drafted:A');
    expect(validUpstream(ISSUE_URL(204))).toBe(true);
    expect(validUpstream('drafted:B')).toBe(true);
    expect(validUpstream('')).toBe(false);
    expect(validUpstream('TODO')).toBe(false);
  });

  test('derives the allowlist entries of a module', () => {
    const source = '/** head */\n\nconst A = 1 & 2;\nconst B = A;\nconst C = 3;\n';
    const at = (text) => [source.indexOf(text), source.indexOf(text) + text.length];
    const items = [
      { term: 'comment', status: 'carried', reason: 'comment the target cannot hold', diagnostic: null, ...span(at('/** head */')) },
      { term: 'lexical_declaration', status: 'carried', reason: 'unsupported', diagnostic: 'bitwise &: outside at 3..4', ...span(at('const A = 1 & 2;')) },
      { term: 'lexical_declaration', status: 'carried', reason: 'type', diagnostic: 'unknown name crate.A', ...span(at('const B = A;')) },
      { term: 'lexical_declaration', status: 'translated', reason: null, diagnostic: null, ...span(at('const C = 3;')) },
    ];
    expect(requiredEntries('m.js', source, items)).toEqual([
      { kind: 'item', module: 'm.js', name: 'A', construct: 'bitwise &: outside' },
      { kind: 'cascade', module: 'm.js', name: 'B', needs: 'A' },
      { kind: 'comments', module: 'm.js', count: 1 },
    ]);
  });

  test('the allowlist round-trips and is compared both ways', () => {
    const required = [
      { kind: 'item', module: 'm.js', name: 'A', construct: 'bitwise &: outside' },
      { kind: 'cascade', module: 'm.js', name: 'B', needs: 'A' },
      { kind: 'comments', module: 'm.js', count: 1 },
    ];
    const merged = mergeAllowlist(required, [{ kind: 'item', module: 'm.js', name: 'A', construct: 'bitwise &: outside', upstream: 'drafted:Z', note: 'kept' }]);
    expect(merged[0]).toEqual({ ...required[0], upstream: 'drafted:Z', note: 'kept' });
    expect(merged[2].upstream).toBe('drafted:A');
    const text = renderAllowlist(merged);
    expect(parseAllowlist(text)).toEqual(merged);
    expect(compareAllowlist(required, merged)).toEqual([]);

    const stale = [...merged, { kind: 'item', module: 'm.js', name: 'Gone', construct: 'x', upstream: 'drafted:D' }];
    expect(compareAllowlist(required, stale)).toEqual(['stale allowlist entry (now translates or is gone): item m.js Gone']);
    expect(compareAllowlist(required, merged.slice(1))).toEqual(['carried but not in the allowlist: item m.js A (bitwise &: outside)']);
    const noIssue = merged.map((e) => (e.kind === 'comments' ? { ...e, upstream: '' } : e));
    expect(compareAllowlist(required, noIssue)).toEqual(['no upstream issue: comments m.js']);
    const changed = merged.map((e) => (e.kind === 'item' ? { ...e, construct: 'shift' } : e));
    expect(compareAllowlist(required, changed)[0]).toMatch(/^construct changed/);
  });

  test('the ledger round-trips and catches regressions', () => {
    const row = { source: sha256('a'), output: sha256('b'), parts: 1, translated: 3, carried: 2, comment: 1, allowlisted: 2 };
    const ledger = { commit: 'eb0574d5528babd3b4aadece81590d8e2230b99b', modules: new Map([['x/y.js', row]]) };
    const text = renderLedger(ledger);
    expect(parseLedger(text)).toEqual(ledger);
    expect(text).toContain('  translated 3\n');
    expect(regressions(ledger.modules, new Map([['x/y.js', { translated: 2 }]]))).toEqual(['x/y.js: translated 3 -> 2']);
    expect(regressions(ledger.modules, new Map([['x/y.js', { translated: 4 }]]))).toEqual([]);
  });

  test('patches self-translation.js only where expected', () => {
    const source = 'a\nreturn carry(text, term, from, error.kind);\nb\nrecorded.push({ term, start, end, status: out.status, reason: out.reason ?? null });\n';
    const patched = patchSelfTranslation(source);
    expect(patched).toContain('diagnostic: error.reason ?? error.message');
    expect(patched).toContain('diagnostic: out.diagnostic ?? null');
    expect(patched).toContain('export { translateModule as __translateModule };');
    expect(() => patchSelfTranslation('nothing')).toThrow(/update SELF_TRANSLATION_PATCHES/);
  });
});

/** @param {number[]} range */
function span([start, end]) {
  return { start, end };
}
