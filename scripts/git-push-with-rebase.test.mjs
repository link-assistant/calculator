import { mkdtempSync, rmSync, writeFileSync, readFileSync, existsSync } from 'fs';
import { tmpdir } from 'os';
import { join, resolve } from 'path';
import { spawnSync } from 'child_process';
import test from 'node:test';
import assert from 'node:assert/strict';

const helperPath = resolve('scripts/git-push-with-rebase.mjs');

function run(command, args, cwd, options = {}) {
  const result = spawnSync(command, args, {
    cwd,
    encoding: 'utf8',
    env: {
      ...process.env,
      GIT_AUTHOR_NAME: 'CI Bot',
      GIT_AUTHOR_EMAIL: 'ci@example.invalid',
      GIT_COMMITTER_NAME: 'CI Bot',
      GIT_COMMITTER_EMAIL: 'ci@example.invalid',
    },
    ...options,
  });

  if (result.status !== 0) {
    throw new Error(
      `${command} ${args.join(' ')} failed\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`
    );
  }

  return result;
}

function commitFile(repo, fileName, content, message) {
  writeFileSync(join(repo, fileName), content);
  run('git', ['add', fileName], repo);
  run('git', ['commit', '-m', message], repo);
}

test('push helper rebases and retries after a non-fast-forward rejection', () => {
  const root = mkdtempSync(join(tmpdir(), 'git-push-with-rebase-'));

  try {
    const remote = join(root, 'remote.git');
    const first = join(root, 'first');
    const second = join(root, 'second');
    const verify = join(root, 'verify');

    run('git', ['init', '--bare', '--initial-branch=main', remote], root);
    run('git', ['clone', remote, first], root);
    commitFile(first, 'initial.txt', 'initial\n', 'initial commit');
    run('git', ['push', 'origin', 'HEAD:main'], first);

    run('git', ['clone', remote, second], root);

    commitFile(second, 'remote.txt', 'remote advanced first\n', 'remote update');
    run('git', ['push', 'origin', 'HEAD:main'], second);

    commitFile(first, 'local.txt', 'local generated change\n', 'local update');
    run('node', [helperPath, '--branch', 'main'], first);

    run('git', ['clone', remote, verify], root);
    assert.equal(readFileSync(join(verify, 'remote.txt'), 'utf8'), 'remote advanced first\n');
    assert.equal(readFileSync(join(verify, 'local.txt'), 'utf8'), 'local generated change\n');
    assert.equal(existsSync(join(verify, 'initial.txt')), true);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
