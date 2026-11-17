import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { checkClosingReferences, stripIgnoredMarkdown } from './check-closing-references.mjs';

const repository = 'link-assistant/calculator';
const script = fileURLToPath(new URL('./check-closing-references.mjs', import.meta.url));
// PR #228's description when it was merged (docs/case-studies/issue-247).
const pr228 = readFileSync(
  new URL('../docs/case-studies/issue-247/data/pr-228-body-at-merge.md', import.meta.url),
  'utf8'
);

const numbers = (references) => references.map((reference) => Number(reference.split('#')[1]));
const range = (low, high) => Array.from({ length: high - low + 1 }, (_, i) => low + i);

function errorsBy(body, rule) {
  return numbers(checkClosingReferences(body, repository).errors.filter((e) => e.rule === rule).map((e) => e.reference));
}

test('PR #228: the issues it says stay open are the ones its Fixes lines close', () => {
  const result = checkClosingReferences(pr228, repository);
  assert.deepEqual(numbers(result.closes), [227, ...range(229, 246)]);
  assert.deepEqual(numbers(result.keepOpen), range(236, 246));
  assert.deepEqual(errorsBy(pr228, 'keep-open'), range(236, 246));
  assert.deepEqual(errorsBy(pr228, 'closes-only'), range(229, 246));
});

test('PR #228 without the appended Fixes lines is consistent', () => {
  const planned = pr228.replace(/^Fixes #\d+\n?/gm, '');
  const result = checkClosingReferences(planned, repository);
  assert.deepEqual(result.errors, []);
  assert.deepEqual(numbers(result.closes), [227]);
});

test('one keyword per issue: a comma or "and" list closes only the first issue (PRs #61 and #82)', () => {
  assert.deepEqual(errorsBy('Fixes #51, #52, #53 (parent issue #54).\n\nFixes #54', 'missing-keyword'), [52, 53]);
  assert.deepEqual(errorsBy('Fixes #36, #34, #23, #68, #45', 'missing-keyword'), [34, 23, 68, 45]);
  assert.deepEqual(errorsBy('Closes #1 and #2', 'missing-keyword'), [2]);
  assert.deepEqual(checkClosingReferences('Resolves #10, resolves #123', repository).errors, []);
  assert.deepEqual(checkClosingReferences('Fixes #1, #2\nFixes #2', repository).errors, []);
});

test('the repository-mode block of #260 is consistent', () => {
  const body = 'Fixes #260\nFixes #247\nFixes #254\nFixes #258\n\nThe implementation issues #236 to #246 stay open.';
  const result = checkClosingReferences(body, repository);
  assert.deepEqual(result.errors, []);
  assert.deepEqual(numbers(result.closes), [260, 247, 254, 258]);
});

test('a closing target in the same sentence is not the issue kept open', () => {
  assert.deepEqual(checkClosingReferences('Fixes #5, so #6 stays open.', repository).errors, []);
  assert.deepEqual(errorsBy('Fixes #5, so #6 stays open.\nFixes #6', 'keep-open'), [6]);
});

test('keep-open phrasings and ranges', () => {
  for (const sentence of [
    '#7 remains open.',
    'This PR does not implement #7, so it stays open.',
    'Issue #7 will not be closed by this PR.',
    'We keep #7 open for the follow-up.',
    '#6 to #8 stay open.',
    '#6–#8 remain open.',
    '#6..#8 are left open.',
  ]) {
    assert.ok(errorsBy(`${sentence}\n\nFixes #7`, 'keep-open').includes(7), sentence);
  }
  assert.deepEqual(errorsBy('#6 to #8 stay open.\n\nFixes #9', 'keep-open'), []);
});

test('a negated keyword still closes the issue (probe on PR #261)', () => {
  assert.deepEqual(errorsBy('This does not close #7.', 'negated-keyword'), [7]);
  assert.deepEqual(errorsBy("Won't fix #8 here", 'negated-keyword'), [8]);
  assert.deepEqual(errorsBy('It does not yet resolve #9', 'negated-keyword'), [9]);
  assert.deepEqual(errorsBy('Without closing #9 the work continues.', 'negated-keyword'), []);
  assert.deepEqual(checkClosingReferences('Notably fixes #10', repository).errors, []);
  // PR #95: a negation that ends the previous line does not apply.
  assert.deepEqual(checkClosingReferences('- [x] All keys disabled when WASM not ready\n\nFixes #48', repository).errors, []);
});

test('Part of / Refs and a closing keyword for the same issue contradict', () => {
  assert.deepEqual(errorsBy('Part of #227.\n\nFixes #227', 'part-of'), [227]);
  assert.deepEqual(errorsBy('Part of #227.\n\nFixes #229', 'part-of'), []);
});

test('full URLs and cross-repository references are keyed by repository', () => {
  const body = 'Fixes https://github.com/link-assistant/calculator/issues/12\nFixes other/repo#12\n\nother/repo#12 stays open.';
  const result = checkClosingReferences(body, repository);
  assert.deepEqual(result.closes, ['link-assistant/calculator#12', 'other/repo#12']);
  assert.deepEqual(result.errors.map((e) => e.reference), ['other/repo#12']);
});

test('keywords in code blocks, inline code and HTML comments are ignored', () => {
  const body = 'Text\n\n```\nFixes #1\n```\n\nUse `Fixes #2` lines.\n<!-- Fixes #3 -->\nCloses: #4';
  assert.equal(stripIgnoredMarkdown(body).includes('#1'), false);
  assert.deepEqual(numbers(checkClosingReferences(body, repository).closes), [4]);
});

test('prefix words are not keywords', () => {
  assert.deepEqual(checkClosingReferences('Prefixes #1 and suffixes #2', repository).closes, []);
});

test('CLI reads the pull_request event payload and fails with annotations', (t) => {
  const dir = mkdtempSync(join(tmpdir(), 'closing-references-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const event = join(dir, 'event.json');
  writeFileSync(event, JSON.stringify({ pull_request: { body: pr228 } }));
  const failed = spawnSync(process.execPath, [script, '--verbose'], {
    encoding: 'utf8',
    env: { ...process.env, GITHUB_EVENT_PATH: event, GITHUB_REPOSITORY: repository },
  });
  assert.equal(failed.status, 1);
  assert.match(failed.stdout, /::error title=Closing reference \(keep-open\)::line \d+: #236 has a closing keyword/);
  assert.match(failed.stdout, /Declared open: #236, #237/);

  writeFileSync(event, JSON.stringify({ pull_request: { body: 'Fixes #260\nFixes #247' } }));
  const passed = spawnSync(process.execPath, [script], {
    encoding: 'utf8',
    env: { ...process.env, GITHUB_EVENT_PATH: event, GITHUB_REPOSITORY: repository },
  });
  assert.equal(passed.status, 0, passed.stderr);
  assert.match(passed.stdout, /2 issue\(s\) closed on merge/);

  const body = join(dir, 'body.md');
  writeFileSync(body, 'Fixes #1, #2');
  const file = spawnSync(process.execPath, [script, '--body-file', body], { encoding: 'utf8' });
  assert.equal(file.status, 1);
  assert.match(file.stdout, /missing-keyword/);
});
