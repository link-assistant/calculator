import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { parseCorpus } from './competitor-coverage.mjs';
import { unstable } from '../experiments/issue-227/documentation.mjs';

const corpus = 'docs/case-studies/issue-227/corpus/closed-source-docs.tsv';
const products = ['Soulver', 'Numi', 'Calca', 'Raycast', 'Wolfram|Alpha', 'Google',
  'Apple Math Notes', 'Frink', 'Parsify', 'Hurmet', 'SpeedCrunch',
  'Windows Calculator', 'Samsung Calculator', 'Xiaomi Calculator', 'DuckDuckGo'];

test('every documentation product is represented or has an audited zero-example reference', () => {
  const rows = parseCorpus(corpus);
  const audit = JSON.parse(readFileSync('docs/case-studies/issue-227/documentation-sources.json', 'utf8'));
  for (const product of products) {
    const source = audit.sources.find((source) => source.product === product);
    assert.ok(source, `missing documentation audit for ${product}`);
    assert.ok(source.pages.length > 0, `missing fetched reference for ${product}`);
    assert.equal(rows.filter((row) => row.source === product).length, source.rows);
    for (const page of source.pages) {
      assert.match(page.url, /^https?:\/\//);
      assert.match(page.fetched, /^\d{4}-\d{2}-\d{2}$/);
    }
    assert.ok(source.rows > 0 || source.zeroExamples, `missing examples for ${product}`);
  }
});

test('all documentation rows have full page attribution', () => {
  for (const row of parseCorpus(corpus)) {
    assert.match(row.note, /https?:\/\//, `${row.source}: ${row.expression}`);
  }
});

test('refresh preserves historical regression identifiers and clears unstable answers', () => {
  const rows = parseCorpus(corpus);
  const identity = (row) => JSON.stringify([row.source, row.lang, row.expression]);
  const ids = new Set(rows.map(identity));
  assert.equal(ids.size, rows.length, 'duplicate corpus identifiers');
  for (const row of parseCorpus('experiments/issue-227/legacy-examples.tsv')) {
    assert.ok(ids.has(identity(row)), `missing historical example: ${row.expression}`);
  }
  for (const row of rows) {
    if (unstable(row.expression, row.expected, row.note)) assert.equal(row.expected, '', row.expression);
  }
});

test('current timestamp, timezone and locale-sensitive examples do not assert stale values', () => {
  const rows = parseCorpus(corpus);
  for (const expression of ['current timestamp', 'Tokyo time', '6pm Sydney in Chicago',
    '12/02/1988 + 32 years', '$300 + VAT']) {
    const row = rows.find((row) => row.expression === expression);
    assert.ok(row, `missing existing example ${expression}`);
    assert.equal(row.expected, '', expression);
  }
});
