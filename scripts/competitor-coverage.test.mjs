import test from 'node:test';
import assert from 'node:assert/strict';
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { compareWithBaseline, decodeEscapes, normalizeText, parseCorpus, resultsMatch, STATUSES } from './competitor-coverage.mjs';

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

test('matches documented compact large-number displays without confusing units', () => {
  assert.equal(resultsMatch('300000', '300k'), true);
  assert.equal(resultsMatch('1.5M', '1500000'), true);
  assert.equal(resultsMatch('2G USD', '2000000000 USD'), true);
  assert.equal(resultsMatch('300', '300k'), false);
  assert.equal(resultsMatch('1 km', '1000 m'), false);
  assert.equal(resultsMatch('1 kg', '1000 g'), false);
});

test('compares equivalent radix and decimal displays numerically', () => {
  assert.equal(resultsMatch('16', '0x10'), true);
  assert.equal(resultsMatch('4.125', '0b100.001'), true);
  assert.equal(resultsMatch('64.001953125', '0o100.001'), true);
  assert.equal(resultsMatch('0 X10', '0x10'), false);
});

function runCoverage(t, previous, rows, { separateOutput = false, rejects = false, sourceFilter = null } = {}) {
  const dir = mkdtempSync(join(tmpdir(), 'coverage-gate-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const binary = join(dir, 'calculator.mjs');
  writeFileSync(binary, `#!/usr/bin/env node\nconsole.log(${JSON.stringify(rejects ? 'Error: rejected' : 'Result: 4')});\n`);
  chmodSync(binary, 0o755);
  const corpus = join(dir, 'corpus.tsv');
  writeFileSync(corpus, rows.map((row) => `fixture\ten\t${row.expression}\t${row.expected}\n`).join(''));
  const baseline = join(dir, 'coverage-baseline.json');
  writeFileSync(baseline, JSON.stringify({ results: previous.map((row) => ({
    source: 'fixture', lang: 'en', ...row,
  })) }));
  const out = separateOutput ? join(dir, 'output') : dir;
  const run = spawnSync(process.execPath, [
    'scripts/competitor-coverage.mjs', '--binary', binary, '--corpus', corpus,
    '--out', out, '--baseline', baseline, '--quiet', '--timeout', '1000',
    ...(sourceFilter ? ['--source', sourceFilter] : []),
  ], { encoding: 'utf8', timeout: 10000 });
  assert.ifError(run.error);
  return { ...run, out, baseline };
}

test('fails a regression before overwriting the baseline in the output directory', { timeout: 15000 }, (t) => {
  const run = runCoverage(t, [{ expression: '2 + 2', status: 'supported' }], [
    { expression: '2 + 2', expected: '5' },
  ]);
  assert.equal(run.status, 1, run.stderr);
  assert.match(run.stderr, /2 \+ 2 -> different/);
  assert.match(readFileSync(join(run.out, 'coverage-report.md'), 'utf8'), /## Different \(1\)/);
});

test('fails when a previously supported expression disappears from the corpus', { timeout: 15000 }, (t) => {
  const run = runCoverage(t, [{ expression: 'removed', status: 'supported' }], [
    { expression: '2 + 2', expected: '4' },
  ], { separateOutput: true });
  assert.equal(run.status, 1, run.stderr);
  assert.match(run.stderr, /removed.*missing/);
});

test('fails when a different result becomes unsupported', { timeout: 15000 }, (t) => {
  const run = runCoverage(t, [{ expression: '2 + 2', status: 'different' }], [
    { expression: '2 + 2', expected: '4' },
  ], { rejects: true });
  assert.equal(run.status, 1, run.stderr);
  assert.match(run.stderr, /2 \+ 2 -> unsupported/);
});

test('allows unchanged statuses and improvements', { timeout: 15000 }, (t) => {
  const run = runCoverage(t, [
    { expression: 'unchanged', status: 'supported' },
    { expression: 'improved', status: 'unsupported' },
  ], [
    { expression: 'unchanged', expected: '4' },
    { expression: 'improved', expected: '4' },
  ]);
  assert.equal(run.status, 0, run.stderr);
  assert.match(run.stdout, /No regressions/);
});

test('applies the source filter to the baseline when checking for missing expressions', { timeout: 15000 }, (t) => {
  const run = runCoverage(t, [
    { expression: '2 + 2', status: 'supported' },
    { source: 'another engine', expression: 'ignored', status: 'supported' },
  ], [{ expression: '2 + 2', expected: '4' }], { sourceFilter: 'FIXTURE' });
  assert.equal(run.status, 0, run.stderr);
  assert.match(run.stdout, /No regressions/);
});

test('orders all status transitions and keeps languages separate', () => {
  const row = { source: 'fixture', lang: 'en', expression: '2 + 2' };
  for (const [oldRank, oldStatus] of STATUSES.entries()) {
    for (const [newRank, newStatus] of STATUSES.entries()) {
      const regressions = compareWithBaseline([{ ...row, status: newStatus }], {
        results: [{ ...row, status: oldStatus }],
      });
      assert.equal(regressions.length, newRank > oldRank ? 1 : 0, `${oldStatus} -> ${newStatus}`);
    }
  }
  const regressions = compareWithBaseline([
    { ...row, status: 'supported' },
    { ...row, lang: 'ru', status: 'unsupported' },
  ], { results: [{ ...row, status: 'supported' }, { ...row, lang: 'ru', status: 'unsupported' }] });
  assert.deepEqual(regressions, []);
});
