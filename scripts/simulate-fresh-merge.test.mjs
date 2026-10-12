import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const script = fileURLToPath(new URL('./simulate-fresh-merge.sh', import.meta.url));

// A bare origin whose main moved after the pull request branched: the same
// shape as a PR whose base advanced after its checks ran.
function fixture(t, { conflict = false } = {}) {
  const dir = mkdtempSync(join(tmpdir(), 'fresh-merge-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const origin = join(dir, 'origin.git');
  const work = join(dir, 'work');
  const git = (cwd, ...args) =>
    execFileSync('git', ['-c', 'init.defaultBranch=main', '-c', 'user.name=Merge Test',
      '-c', 'user.email=merge@example.com', '-C', cwd, ...args], { encoding: 'utf8' }).trim();
  const commit = (file, content) => {
    writeFileSync(join(work, file), content);
    git(work, 'add', '.');
    git(work, 'commit', '--quiet', '-m', `${file}: ${content}`);
  };
  git(dir, 'init', '--quiet', '--bare', origin);
  git(dir, 'init', '--quiet', work);
  git(work, 'remote', 'add', 'origin', origin);
  commit('shared.txt', 'base');
  git(work, 'push', '--quiet', 'origin', 'main');
  git(work, 'checkout', '--quiet', '-b', 'feature');
  commit(conflict ? 'shared.txt' : 'feature.txt', 'feature');
  git(work, 'checkout', '--quiet', 'main');
  commit(conflict ? 'shared.txt' : 'main.txt', 'advanced');
  git(work, 'push', '--quiet', 'origin', 'main');
  git(work, 'checkout', '--quiet', 'feature');
  return { work, head: () => git(work, 'rev-parse', 'HEAD'), git };
}

function run(cwd, env = {}) {
  return spawnSync('bash', [script], {
    cwd,
    encoding: 'utf8',
    env: { ...process.env, GITHUB_BASE_REF: 'main', FRESH_MERGE_RETRY_DELAY_SECONDS: '0', ...env },
  });
}

test('runs the checks on the merge with the current base tip', { timeout: 15000 }, (t) => {
  const { work, head } = fixture(t);
  const before = head();
  // Both files exist only in the merged tree, so this proves where checks run.
  const result = run(work, { FRESH_MERGE_CHECKS: 'test -f main.txt\ntest -f feature.txt' });
  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.match(result.stdout, /::group::test -f main\.txt/);
  assert.match(result.stdout, /::group::test -f feature\.txt/);
  assert.equal(head(), before, 'the pull request checkout is restored');
});

test('reports a check that fails only on the merge as a semantic conflict', { timeout: 15000 }, (t) => {
  const { work, head } = fixture(t);
  const before = head();
  const result = run(work, { FRESH_MERGE_CHECKS: 'true\ntest ! -f main.txt' });
  assert.equal(result.status, 1);
  assert.match(result.stdout, /::error::Checks failed on the simulated merge.*semantic merge conflict/);
  assert.equal(head(), before);
});

test('fails a textual conflict and leaves the checkout clean', { timeout: 15000 }, (t) => {
  const { work, head, git } = fixture(t, { conflict: true });
  const before = head();
  const result = run(work, { FRESH_MERGE_CHECKS: 'true' });
  assert.equal(result.status, 1);
  assert.match(result.stdout, /::error::Textual merge conflict with origin\/main/);
  assert.equal(head(), before);
  assert.equal(git(work, 'status', '--porcelain'), '');
});

test('skips a head that the base already contains', { timeout: 15000 }, (t) => {
  const { work, git } = fixture(t);
  git(work, 'checkout', '--quiet', 'main');
  const result = run(work, { FRESH_MERGE_CHECKS: 'false' });
  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.match(result.stdout, /nothing to simulate/);
});

test('retries the base fetch a bounded number of times', { timeout: 15000 }, (t) => {
  const { work, git } = fixture(t);
  git(work, 'remote', 'set-url', 'origin', join(work, 'missing.git'));
  const result = run(work, { FRESH_MERGE_FETCH_ATTEMPTS: '2', FRESH_MERGE_CHECKS: 'true' });
  assert.equal(result.status, 1);
  assert.match(result.stdout, /attempt 1 of 2/);
  assert.match(result.stderr, /Could not fetch origin\/main after 2 attempts/);
});

test('rejects an attempt count of zero', { timeout: 15000 }, (t) => {
  const { work } = fixture(t);
  const result = run(work, { FRESH_MERGE_FETCH_ATTEMPTS: '0' });
  assert.equal(result.status, 2);
  assert.match(result.stderr, /must be a positive integer/);
});
