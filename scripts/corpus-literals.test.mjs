import test from 'node:test';
import assert from 'node:assert/strict';
import { calls, decodeLiteral, displayLiteral, singleLine, splitArguments, stringLiteral, stripComments } from './corpus-literals.mjs';
import { parseArgs } from './import-competitor-corpus.mjs';

test('decodes text literals and rejects executable templates or expressions', () => {
  assert.equal(decodeLiteral('\\u03c0 \\u{1f600} \\x41 \\n \\t \\\\'), 'π 😀 A \n \t \\');
  assert.equal(stringLiteral('"https://example.com/*text*/"'), 'https://example.com/*text*/');
  assert.equal(stringLiteral('QString::fromUtf8("µm")'), 'µm');
  assert.equal(stringLiteral('u8"Ω"'), 'Ω');
  assert.equal(stringLiteral('`sqrt(4)`'), 'sqrt(4)');
  assert.equal(stringLiteral('`sqrt(${process.exit()})`'), null);
  assert.equal(stringLiteral('"2" + "3"'), null);
  assert.equal(displayLiteral('answer'), null);
  assert.equal(displayLiteral('2 + 3'), null);
  assert.equal(displayLiteral('[2, [3, 4]]'), '[2, [3, 4]]');
});

test('skips comments, regexes and quoted assertion examples without changing source line numbers', () => {
  const text = stripComments(`// assert.strictEqual(sqrt(4), 99)
    const example = 'assert.strictEqual(sqrt(4), 99)'
    assert.throws(() => sqrt('no'), /Function 'sqrt' doesn't apply/)
    /* assert.strictEqual(sqrt(4), 99) */
    assert.strictEqual(sqrt(4), 2)
    assert.strictEqual(unit('http://example.com'), 'http://example.com')
  `);
  const found = [...calls(text, ['assert\\.strictEqual'])];
  assert.equal(found.length, 2);
  assert.equal(found[0].line, 5);
  assert.deepEqual(found[0].args, ['sqrt(4)', '2']);
  assert(text.includes('http://example.com'));
  assert.equal(stripComments('-- expectOutput\' "99" "2 + 2"\nexpectOutput\' "4" "2 + 2"', 'purs').split('\n')[1], 'expectOutput\' "4" "2 + 2"');
});

test('separates nested literal arguments and excludes multiline/control inputs', () => {
  assert.deepEqual(splitArguments('unit(5, "m"), [1, [2, 3]], "a,b"'), ['unit(5, "m")', '[1, [2, 3]]', '"a,b"']);
  assert(singleLine('2 + 2'));
  for (const value of ['', '2\n3', '2\r3', '\0', '\u007f']) assert(!singleLine(value));
});

test('validates required command arguments without running an import on module load', () => {
  assert.throws(() => parseArgs([]), /checkouts/);
  assert.throws(() => parseArgs(['--checkouts']), /Missing value/);
  assert.throws(() => parseArgs(['--checkouts', '--out', 'x']), /Missing value/);
  assert.throws(() => parseArgs(['--unknown']), /Unknown argument/);
  assert.equal(parseArgs(['--help']), null);
});
