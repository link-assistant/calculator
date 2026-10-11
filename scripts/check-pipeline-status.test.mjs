import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const script = path.join(path.dirname(fileURLToPath(import.meta.url)), 'check-pipeline-status.sh');

function gate(needs, env = {}) {
  const result = spawnSync('bash', [script], {
    encoding: 'utf8',
    env: { PATH: process.env.PATH, NEEDS_JSON: JSON.stringify(needs), ...env },
  });
  return { status: result.status, output: result.stdout + result.stderr };
}

function workflow(cancelInProgress) {
  const dir = mkdtempSync(path.join(tmpdir(), 'pipeline-status-'));
  const file = path.join(dir, 'release.yml');
  writeFileSync(
    file,
    `jobs:\n  lint:\n    concurrency:\n      group: lint\n      cancel-in-progress: ${cancelInProgress}\n    runs-on: ubuntu-24.04\n`
  );
  return file;
}

test('passes when every job succeeded or was skipped', () => {
  const { status, output } = gate({ lint: { result: 'success' }, deploy: { result: 'skipped' } });
  assert.equal(status, 0);
  assert.match(output, /All required jobs succeeded/);
});

test('fails on a failed job that a skipped descendant would hide (issue #256)', () => {
  const { status, output } = gate({ test: { result: 'failure' }, build: { result: 'skipped' } });
  assert.equal(status, 1);
  assert.match(output, /::error title=Pipeline failed::Failing jobs: test/);
});

test('fails on a cancelled job in a run that is still the branch head (timeout)', () => {
  const { status, output } = gate(
    { lint: { result: 'cancelled' } },
    { RUN_SHA: 'abc', BRANCH_HEAD_SHA: 'abc', WORKFLOW_FILE: workflow('true') }
  );
  assert.equal(status, 1);
  assert.match(output, /Pipeline has cancelled jobs::lint/);
});

test('excuses a cancelled job of a superseded run that cancels in progress', () => {
  const { status, output } = gate(
    { lint: { result: 'cancelled' } },
    { RUN_SHA: 'abc', BRANCH_HEAD_SHA: 'def', WORKFLOW_FILE: workflow('true') }
  );
  assert.equal(status, 0);
  assert.match(output, /Cancelled jobs in a superseded run::lint/);
});

test('fails closed when cancel-in-progress is an expression it cannot evaluate', () => {
  const { status } = gate(
    { lint: { result: 'cancelled' } },
    { RUN_SHA: 'abc', BRANCH_HEAD_SHA: 'def', WORKFLOW_FILE: workflow("${{ github.ref != 'refs/heads/main' }}") }
  );
  assert.equal(status, 1);
});
