import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { checkoutSuites } from './checkout-competitor-suites.mjs';

// Use a local upstream with two revisions: no network or upstream tools needed.
test('checks out the pin, refreshes to HEAD on request, and preserves dirty work', { timeout: 15000 }, (t) => {
  const dir = mkdtempSync(join(tmpdir(), 'checkout-corpus-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const repo = join(dir, 'upstream');
  mkdirSync(repo);
  const git = (...args) => execFileSync('git', ['-C', repo, ...args], { encoding: 'utf8' }).trim();
  git('init', '--quiet');
  const commit = (content) => {
    writeFileSync(join(repo, 'tests.txt'), content);
    git('add', '.');
    git('-c', 'user.name=Corpus Test', '-c', 'user.email=corpus@example.com', 'commit', '--quiet', '-m', content);
    return git('rev-parse', 'HEAD');
  };
  const revision = commit('first');
  const newest = commit('second');
  const lock = join(dir, 'lock.json');
  writeFileSync(lock, JSON.stringify({ fixture: { repo, revision } }));
  const checkouts = join(dir, 'checkouts');
  const root = join(checkouts, 'fixture');
  const head = () => execFileSync('git', ['-C', root, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
  checkoutSuites({ checkouts, lock });
  assert.equal(head(), revision);
  checkoutSuites({ checkouts, lock });
  assert.equal(head(), revision);
  checkoutSuites({ checkouts, lock, latest: true });
  assert.equal(head(), newest);
  writeFileSync(join(root, 'tests.txt'), 'local edits');
  assert.throws(() => checkoutSuites({ checkouts, lock }), /dirty/);
  assert.equal(head(), newest);
  assert.equal(readFileSync(join(root, 'tests.txt'), 'utf8'), 'local edits');
});
