import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import {
  collectInputs,
  exampleInputs,
  isCandidate,
  isMessage,
  rustStringLiterals,
  testSection,
  unescapeRust,
} from './extract-parity-corpus.mjs';

const values = (source) => rustStringLiterals(source).map((literal) => literal.value);

test('decodes Rust string escapes', () => {
  assert.equal(unescapeRust('a\\"b\\\\c\\n\\u{20BD}\\x41'), 'a"b\\c\n₽A');
  assert.equal(unescapeRust('line \\\n    continued'), 'line continued');
});

test('reads plain, raw and byte-raw literals and skips comments and chars', () => {
  const source = `
    // calc("in a comment")
    /* "block comment" */
    let c = '"';
    let q = '\\'';
    calc("2 + 3");
    calc(r#"say "hi""#);
    calc(br"raw bytes");
  `;
  assert.deepEqual(values(source), ['2 + 3', 'say "hi"', 'raw bytes']);
});

test('records the call and argument position of each literal', () => {
  const literals = rustStringLiterals('assert_eq!(calc("1 + 1"), "2", "sum of {}", x); value.expect("must parse");');
  assert.deepEqual(literals, [
    { value: '1 + 1', call: 'calc', argument: 0 },
    { value: '2', call: 'assert_eq!', argument: 1 },
    { value: 'sum of {}', call: 'assert_eq!', argument: 2 },
    { value: 'must parse', call: 'expect', argument: 0 },
  ]);
});

test('treats assertion, panic and format messages as messages, not inputs', () => {
  const [condition, message] = rustStringLiterals('assert!(ok("5 USD"), "Result should be in USD");');
  assert.equal(isMessage(condition), false);
  assert.equal(isMessage(message), true);
  const [expected, eqMessage] = rustStringLiterals('assert_eq!(x, "10 RUB", "conversion");');
  assert.equal(isMessage(expected), false);
  assert.equal(isMessage(eqMessage), true);
  assert.equal(isMessage(rustStringLiterals('panic!("bad input");')[0]), true);
  assert.equal(isMessage(rustStringLiterals('let s = format!("{} USD", n);')[0]), true);
  assert.equal(isMessage(rustStringLiterals('let v = vec!["1 + 2", "3 * 4"];')[0]), false);
});

test('accepts one-line inputs and rejects templates, blanks and long text', () => {
  assert.equal(isCandidate('2 + 3'), true);
  assert.equal(isCandidate('{"usd": 1.0}'), true);
  assert.equal(isCandidate('   '), false);
  assert.equal(isCandidate('a\nb'), false);
  assert.equal(isCandidate('{v} + 2 = 4'), false);
  assert.equal(isCandidate('value {:?}'), false);
  assert.equal(isCandidate('x'.repeat(201)), false);
});

test('takes whole test files and the cfg(test) part of other sources', () => {
  assert.equal(testSection('tests/a.rs', 'all'), 'all');
  assert.equal(testSection('src/grammar/lexer_tests.rs', 'all'), 'all');
  assert.equal(testSection('src/types/value/tests.rs', 'all'), 'all');
  assert.equal(testSection('src/lib.rs', 'code\n#[cfg(test)]\nmod tests {}'), '#[cfg(test)]\nmod tests {}');
  assert.equal(testSection('src/lib.rs', 'no tests here'), '');
});

test('reads the example expressions of data/examples.lino', () => {
  const source = '(examples\n  (example (expression "2 + 3") (description "add"))\n  (example (expression "say \\"x\\""))\n)';
  assert.deepEqual(exampleInputs(source), ['2 + 3', 'say "x"']);
});

test('parity/corpus.json contains every input the Rust tests use', () => {
  const corpus = JSON.parse(readFileSync(new URL('../parity/corpus.json', import.meta.url), 'utf8'));
  const known = new Set(corpus.inputs);
  const missing = [...collectInputs().keys()].filter((input) => !known.has(input));
  assert.deepEqual(missing, [], 'run node scripts/extract-parity-corpus.mjs --write and regenerate the goldens');
  assert.ok(corpus.scenarios.default, 'the default scenario is required');
  assert.match(corpus.clock, /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$/);
});
