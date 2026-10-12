import test from 'node:test';
import assert from 'node:assert/strict';
import { ISSUE_MARKER, ISSUE_TITLE, renderIssueBody, upsertCoverageGapIssue } from './update-coverage-gap-issue.mjs';

const repository = 'link-assistant/calculator';
const issueUrl = `https://github.com/${repository}/issues/123`;

function github(pages) {
  const calls = [];
  return {
    calls,
    run(command, args, options) {
      assert.equal(command, 'gh');
      calls.push({ args, options });
      return JSON.stringify(calls.length === 1 ? pages : { html_url: issueUrl });
    },
  };
}

test('renders source and language tables with provenance and the artifact link', () => {
  const report = `# Competitor coverage report

## Coverage by source

| Source | Cases |
| --- | ---: |
| fend | 2 |

## Coverage by language

| Source | Cases |
| --- | ---: |
| en | 1 |
| ru | 1 |

## Different (1)

large row details
`;
  const body = renderIssueBody(report, { repository, sha: 'abc123', runId: '456', generated: '2026-10-08' });
  assert.ok(body.includes(ISSUE_MARKER));
  assert.ok(body.includes('## Coverage by source'));
  assert.ok(body.includes('## Coverage by language'));
  assert.ok(body.includes('| ru | 1 |'));
  assert.ok(body.includes(`/commit/abc123`));
  assert.ok(body.includes('/actions/runs/456'));
  assert.ok(body.includes('competitor-coverage-report'));
  assert.ok(!body.includes('large row details'));
  assert.throws(() => renderIssueBody('broken report', { repository }), /missing the source summary/);
});

test('creates the gap issue when none exists, ignoring similarly named PRs', () => {
  const mock = github([[{ title: ISSUE_TITLE, number: 99, pull_request: {} }]]);
  assert.equal(upsertCoverageGapIssue(repository, 'new body', mock.run), issueUrl);
  assert.deepEqual(mock.calls[0].args, [
    'api', `repos/${repository}/issues?state=all&per_page=100`, '--paginate', '--slurp',
  ]);
  assert.deepEqual(mock.calls[1].args, ['api', `repos/${repository}/issues`, '--method', 'POST', '--input', '-']);
  assert.deepEqual(JSON.parse(mock.calls[1].options.input), { title: ISSUE_TITLE, body: 'new body' });
});

test('updates and reopens the marked issue found on a later page', () => {
  const mock = github([
    [{ title: ISSUE_TITLE, number: 7 }],
    [{ title: 'renamed report', body: ISSUE_MARKER, number: 123, state: 'closed' }],
  ]);
  assert.equal(upsertCoverageGapIssue(repository, 'updated body', mock.run), issueUrl);
  assert.deepEqual(mock.calls[1].args, ['api', `repos/${repository}/issues/123`, '--method', 'PATCH', '--input', '-']);
  assert.deepEqual(JSON.parse(mock.calls[1].options.input), {
    title: ISSUE_TITLE, body: 'updated body', state: 'open',
  });
});

test('adopts an existing issue with the exact title instead of creating a duplicate', () => {
  const mock = github([[{ title: ISSUE_TITLE, number: 123 }]]);
  upsertCoverageGapIssue(repository, 'body', mock.run);
  assert.ok(mock.calls[1].args.includes(`repos/${repository}/issues/123`));
  assert.ok(mock.calls[1].args.includes('PATCH'));
});

test('fails on GitHub errors so the workflow cannot report a false success', () => {
  assert.throws(() => upsertCoverageGapIssue(repository, 'body', () => {
    throw new Error('GitHub API failed');
  }), /GitHub API failed/);
});
