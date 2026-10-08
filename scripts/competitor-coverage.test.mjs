import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { decodeEscapes, normalizeText, parseCorpus, resultsMatch, STATUSES } from './competitor-coverage.mjs';

test('exposes the four coverage statuses', () => {
  assert.deepEqual(STATUSES, ['supported', 'different', 'unsupported', 'timeout']);
});

test('parses tab-separated corpus rows and skips comments', () => {
  const dir = mkdtempSync(join(tmpdir(), 'corpus-'));
  const file = join(dir, 'sample.tsv');
  writeFileSync(
    file,
    [
      '# comment line',
      '',
      'fend\ten\t2 + 2\t4\tbasic',
      'Soulver\ten\ttab\\there\\nnext\t\tmust evaluate',
      'Wikipedia quote\tru\t3×10⁸ м/с\t300000000 m/s',
    ].join('\n')
  );
  const cases = parseCorpus(file);
  assert.equal(cases.length, 3);
  assert.deepEqual(
    cases.map((c) => [c.line, c.source, c.lang, c.expression, c.expected, c.note]),
    [
      [3, 'fend', 'en', '2 + 2', '4', 'basic'],
      [4, 'Soulver', 'en', 'tab\there\nnext', '', 'must evaluate'],
      [5, 'Wikipedia quote', 'ru', '3×10⁸ м/с', '300000000 m/s', ''],
    ]
  );
});

test('rejects rows with fewer than three columns', () => {
  const dir = mkdtempSync(join(tmpdir(), 'corpus-'));
  const file = join(dir, 'broken.tsv');
  writeFileSync(file, 'fend\t2 + 2\n');
  assert.throws(() => parseCorpus(file), /expected at least 3 tab-separated columns/);
});

test('decodes the corpus escape sequences', () => {
  assert.equal(decodeEscapes('a\\tb\\nc\\\\d'), 'a\tb\nc\\d');
});

test('normalizes display text before comparison', () => {
  assert.equal(normalizeText('approx. 1,234.50 KG'), '1234.5 kg');
  assert.equal(normalizeText('  2.000  '), '2');
  assert.equal(normalizeText('1.0 m/s'), '1 m/s');
});

test('matches numbers at the expected precision and with units', () => {
  assert.equal(resultsMatch('anything', ''), true, 'empty expectation means "must evaluate"');
  assert.equal(resultsMatch('3.14159265', '3.1416'), true);
  assert.equal(resultsMatch('3.14159265', '3.1415'), false);
  assert.equal(resultsMatch('0.3333333333', '1/3'), false);
  assert.equal(resultsMatch('1000000', '1e6'), true);
  assert.equal(resultsMatch('2.54 cm', '2.54 cm'), true);
  assert.equal(resultsMatch('2.54 cm', '2.54 m'), false);
  assert.equal(resultsMatch('220', '200.1'), false);
  assert.equal(resultsMatch('-2 hours', '2 hours'), false);
});
