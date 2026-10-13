import test from 'node:test';
import assert from 'node:assert/strict';
import { classifyRuns, logFileName, runGate } from './ci-push-gate.mjs';

const run = (databaseId, headSha, status, conclusion, createdAt, workflowName = 'CI/CD Pipeline') => ({
  databaseId,
  headSha,
  status,
  conclusion,
  createdAt,
  workflowName,
  url: `https://github.com/link-assistant/calculator/actions/runs/${databaseId}`,
});

function fakeRun(runs, { commits = ['bbb', 'aaa'], logs = {} } = {}) {
  const calls = [];
  return {
    calls,
    run(command, args) {
      calls.push([command, ...args]);
      if (command === 'git' && args[0] === 'branch') {
        return 'issue-260\n';
      }
      if (command === 'git' && args[0] === 'rev-list') {
        return `${commits.join('\n')}\n`;
      }
      if (command === 'gh' && args[1] === 'list') {
        return JSON.stringify(runs);
      }
      if (command === 'gh' && args[1] === 'view') {
        return logs[args[2]] ?? '';
      }
      throw new Error(`unexpected command ${command} ${args.join(' ')}`);
    },
  };
}

test('blocks while a run on the commit range is in progress', () => {
  const { active, failures } = classifyRuns(
    [run(3, 'bbb', 'in_progress', '', '2026-10-13T10:05:00Z'), run(2, 'aaa', 'completed', 'failure', '2026-10-13T10:00:00Z')],
    new Set(['aaa', 'bbb']),
  );
  assert.deepEqual(active.map((entry) => entry.databaseId), [3]);
  assert.deepEqual(failures, []);
});

test('ignores runs of commits outside the branch range', () => {
  const { active, latestSha } = classifyRuns(
    [run(4, 'zzz', 'queued', '', '2026-10-13T11:00:00Z'), run(2, 'aaa', 'completed', 'success', '2026-10-13T10:00:00Z')],
    new Set(['aaa']),
  );
  assert.deepEqual(active, []);
  assert.equal(latestSha, 'aaa');
});

test('collects every failure of the latest commit, not of older ones', () => {
  const { latestSha, failures } = classifyRuns(
    [
      run(5, 'bbb', 'completed', 'failure', '2026-10-13T10:10:00Z', 'JavaScript'),
      run(6, 'bbb', 'completed', 'cancelled', '2026-10-13T10:11:00Z', 'Links'),
      run(7, 'bbb', 'completed', 'skipped', '2026-10-13T10:12:00Z', 'Security'),
      run(1, 'aaa', 'completed', 'failure', '2026-10-13T09:00:00Z'),
    ],
    new Set(),
  );
  assert.equal(latestSha, 'bbb');
  assert.deepEqual(failures.map((entry) => entry.databaseId), [6, 5]);
});

test('names log files after the workflow and run', () => {
  assert.equal(logFileName({ databaseId: 42, workflowName: 'CI/CD Pipeline' }), 'ci-cd-pipeline-42.log');
});

test('the gate exits 1 and names the active runs', () => {
  const lines = [];
  const gh = fakeRun([run(3, 'bbb', 'queued', '', '2026-10-13T10:05:00Z')]);
  assert.equal(runGate([], { run: gh.run, print: (line) => lines.push(line) }), 1);
  assert.match(lines[0], /^::error title=CI push gate::Do not push: 1 run\(s\) on issue-260/);
  assert.match(lines[1], /queued CI\/CD Pipeline bbb/);
});

test('the gate downloads the failed-job logs of the latest commit', () => {
  const lines = [];
  const written = {};
  const gh = fakeRun(
    [run(5, 'bbb', 'completed', 'failure', '2026-10-13T10:10:00Z'), run(1, 'aaa', 'completed', 'failure', '2026-10-13T09:00:00Z')],
    { logs: { 5: 'test failed\n' } },
  );
  const code = runGate(['--logs-dir', 'ci-logs', '--repo', 'link-assistant/calculator'], {
    run: gh.run,
    write: (path, text) => {
      written[path] = text;
    },
    print: (line) => lines.push(line),
  });
  assert.equal(code, 0);
  assert.deepEqual(written, { 'ci-logs/ci-cd-pipeline-5.log': 'test failed\n' });
  assert.match(lines[0], /1 failing run\(s\) of bbb on issue-260; fix all of them, then push once/);
  assert.ok(gh.calls.some((call) => call.join(' ') === 'gh run view 5 --repo link-assistant/calculator --log-failed'));
});

test('the gate passes when the latest commit is green', () => {
  const lines = [];
  const gh = fakeRun([run(5, 'bbb', 'completed', 'success', '2026-10-13T10:10:00Z')]);
  assert.equal(runGate([], { run: gh.run, print: (line) => lines.push(line) }), 0);
  assert.match(lines[0], /Every run of bbb on issue-260 passed/);
});

test('rejects unknown arguments', () => {
  assert.throws(() => runGate(['--force'], { run: fakeRun([]).run }), /unknown or incomplete argument: --force/);
});
