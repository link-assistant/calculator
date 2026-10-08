import assert from 'node:assert/strict';
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';
import { parseCorpus } from './competitor-coverage.mjs';
import { buildCorpus } from '../experiments/issue-227/build-natural-language-corpus.mjs';

const languages = 'en ru de fr es zh ja pt hi ar it ko tr nl pl uk sv fa id vi th he bn'.split(' ');
const constructs = [
  'separators', 'si-prefix', 'scientific-notation', 'uncertainty',
  'percentage-change', 'fold-change', 'order-of-magnitude', 'ratio',
  'named-constant', 'power', 'root', 'unit-conversion',
];
const corpus = new URL('../docs/case-studies/issue-227/corpus/natural-language-quotes.tsv', import.meta.url);
const rows = readFileSync(corpus, 'utf8').split('\n')
  .filter((line) => line && !line.startsWith('#'))
  .map((line) => line.split('\t'));

test('scientific corpus has 50 distinct expressions and every construct in each required language', () => {
  for (const lang of languages) {
    const localized = rows.filter((row) => row[1] === lang);
    assert.ok(new Set(localized.map((row) => row[2])).size >= 50, `${lang}: fewer than 50 distinct expressions`);
    for (const construct of constructs) {
      assert.ok(localized.some((row) => row[4]?.includes(`construct=${construct};`)), `${lang}: missing ${construct}`);
    }
  }
});

test('every scientific row carries explicit provenance and an allowed source license', () => {
  const identities = new Set();
  for (const [source, lang, expression, expected, note, ...extra] of rows) {
    assert.equal(extra.length, 0, `${lang}: extra TSV columns`);
    assert.ok(languages.includes(lang), `unexpected language ${lang}`);
    assert.ok(source && expression && note, `${lang}: incomplete row`);
    assert.match(note, /https:\/\//, `${lang}: missing citation for ${expression}`);
    assert.match(note, /license=(CC-BY-4\.0|CC-BY-SA-4\.0|public-domain);/, `${lang}: missing license`);
    assert.match(note, /kind=(verbatim|translated|derived|legacy-quote);/, `${lang}: missing provenance kind`);
    assert.match(note, /construct=[\w-]+;/, `${lang}: missing construct`);
    if (expected === '') assert.match(note, /oracle=evaluate-only;/, `${lang}: unexplained empty expected`);
    const identity = JSON.stringify([source, lang, expression]);
    assert.ok(!identities.has(identity), `${lang}: duplicate expression ${expression}`);
    identities.add(identity);
  }
});

test('corpus includes real open-access paper excerpts and fresh Wikipedia sentences', () => {
  const papers = rows.filter((row) => row[0] === 'Open-access paper quote');
  assert.ok(papers.length >= 3, 'missing sampled open-access papers');
  assert.ok(new Set(papers.map((row) => row[4].match(/https:\/\/arxiv\.org\/abs\/[\d.v]+/)?.[0])).size >= 3);
  for (const lang of languages) {
    assert.ok(rows.some((row) => row[1] === lang && row[0] === 'Wikipedia sentence'), `${lang}: no sourced sentence`);
  }
});

test('offline regeneration is deterministic and preserves all 162 seed inputs and oracles', () => {
  assert.equal(buildCorpus(), readFileSync(corpus, 'utf8'));
  const seeds = parseCorpus(new URL('../experiments/issue-227/seed-natural-language.tsv', import.meta.url));
  assert.equal(seeds.length, 162);
  const current = parseCorpus(corpus);
  for (const seed of seeds) {
    assert.ok(current.some((row) => ['source', 'lang', 'expression', 'expected'].every((key) => row[key] === seed[key])),
      `missing or changed seed: ${seed.lang} ${seed.expression}`);
  }
});

test('new quotations are exact excerpts of retained, explicitly licensed sources', () => {
  for (const row of parseCorpus(corpus)) {
    let file;
    if (row.source === 'Wikipedia sentence') file = `wikipedia-${row.lang}`;
    if (row.source === 'Open-access paper quote') file = `arxiv-${row.note.match(/arxiv\.org\/abs\/([\d.]+v\d+)/)[1]}`;
    if (!file) continue;
    const source = JSON.parse(readFileSync(new URL(`../experiments/issue-227/sources/${file}.json`, import.meta.url)));
    assert.ok(source.extract.includes(row.expression), `${file}: quote differs from source`);
    assert.ok(row.note.includes(source.license) && row.note.includes(source.revisionUrl || source.url));
    assert.match(row.expression, /\p{Nd}/u, `${file}: missing numeric content`);
  }
});

test('coverage runner retains a per-language table for all 23 languages', { timeout: 15000 }, (t) => {
  const directory = mkdtempSync(join(tmpdir(), 'scientific-language-report-'));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const binary = join(directory, 'calculator.mjs');
  writeFileSync(binary, '#!/usr/bin/env node\nconsole.log("Result: 4");\n');
  chmodSync(binary, 0o755);
  const fixture = join(directory, 'corpus.tsv');
  writeFileSync(fixture, languages.map((lang) => `fixture\t${lang}\t2 + 2\t4\tfixture\n`).join(''));
  const run = spawnSync(process.execPath, [
    fileURLToPath(new URL('./competitor-coverage.mjs', import.meta.url)),
    '--binary', binary, '--corpus', fixture, '--out', directory, '--quiet',
  ], { encoding: 'utf8', timeout: 10000 });
  assert.ifError(run.error);
  assert.equal(run.status, 0, run.stderr);
  const report = readFileSync(join(directory, 'coverage-report.md'), 'utf8');
  const table = report.split('## Coverage by language\n')[1];
  assert.ok(table, 'per-language table disappeared');
  for (const lang of languages) assert.ok(table.includes(`| ${lang} | 1 | 1 | 0 | 0 | 0 | 100.0% |`), lang);
});
