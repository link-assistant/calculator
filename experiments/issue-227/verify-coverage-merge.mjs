#!/usr/bin/env node
// Verify a generated baseline preserves the union of unchanged-engine snapshots,
// including duplicate assertions and each assertion's exact coverage status.
// Usage: node experiments/issue-227/verify-coverage-merge.mjs merged.json before-a.json before-b.json
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { compareWithBaseline, STATUSES } from '../../scripts/competitor-coverage.mjs';

const [mergedFile, ...previousFiles] = process.argv.slice(2);
assert.ok(mergedFile && previousFiles.length >= 2, 'Provide a merged baseline and at least two previous snapshots');
const read = (file) => JSON.parse(readFileSync(file, 'utf8'));
const merged = read(mergedFile);

function inventory(snapshot) {
  const counts = new Map();
  for (const { source, lang, expression, status } of snapshot.results) {
    assert.ok(STATUSES.includes(status), `Unknown status ${status}`);
    const key = JSON.stringify([source, lang, expression, status]);
    counts.set(key, (counts.get(key) || 0) + 1);
  }
  return counts;
}

const expected = new Map();
for (const file of previousFiles) {
  const previous = read(file);
  assert.deepEqual(compareWithBaseline(merged.results, previous), [], `${file}: coverage regression`);
  for (const [key, count] of inventory(previous)) {
    expected.set(key, Math.max(expected.get(key) || 0, count));
  }
  console.log(`Preserved ${previous.results.length} cases from ${file}`);
}
assert.deepEqual(inventory(merged), expected, 'Merged cases/statuses differ from the union of previous snapshots');
assert.equal(merged.total.total, merged.results.length);
for (const status of STATUSES) {
  assert.equal(merged.total[status], merged.results.filter((row) => row.status === status).length);
}
console.log(`Verified merged totals: ${JSON.stringify(merged.total)}`);
