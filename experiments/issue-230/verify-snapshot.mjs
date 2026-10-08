// Compare real import snapshots without executing upstream code.
// Usage: node experiments/issue-230/verify-snapshot.mjs <old.tsv> <new.tsv> [old-baseline.json new-baseline.json]
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { parseCorpus } from '../../scripts/competitor-coverage.mjs';
const [oldFile, newFile, oldBaseline, newBaseline] = process.argv.slice(2);
assert(oldFile && newFile, 'Give old and new corpus paths');
const oldRows = parseCorpus(oldFile);
const newRows = parseCorpus(newFile);
const key = ({ source, lang, expression, expected }) => JSON.stringify([source, lang, expression, expected]);
const counts = new Map();
for (const row of newRows) counts.set(key(row), (counts.get(key(row)) || 0) + 1);
for (const row of oldRows) {
  const id = key(row);
  assert(counts.get(id) > 0, `Original assertion removed: ${id}`);
  counts.set(id, counts.get(id) - 1);
}
const totals = {};
for (const row of newRows) totals[row.source] = (totals[row.source] || 0) + 1;
console.log(JSON.stringify({ retained: oldRows.length, added: newRows.length - oldRows.length, total: newRows.length, sources: totals }, null, 2));
if (oldBaseline || newBaseline) {
  assert(oldBaseline && newBaseline, 'Give both baseline paths');
  const before = JSON.parse(readFileSync(oldBaseline, 'utf8'));
  const after = JSON.parse(readFileSync(newBaseline, 'utf8'));
  const statusKey = ({ source, lang, expression, status }) => JSON.stringify([source, lang, expression, status]);
  const statuses = new Map();
  for (const row of after.results) statuses.set(statusKey(row), (statuses.get(statusKey(row)) || 0) + 1);
  for (const row of before.results) {
    const id = statusKey(row);
    assert(statuses.get(id) > 0, `Original baseline status changed: ${id}`);
    statuses.set(id, statuses.get(id) - 1);
  }
  console.log(`All ${before.results.length} original baseline statuses are unchanged.`);
}
