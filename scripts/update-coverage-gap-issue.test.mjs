import test from 'node:test';
import assert from 'node:assert/strict';
import {
  ISSUE_MARKER,
  ISSUE_TITLE,
  TRACKING_PARENT,
  dryRunGitHub,
  listTrackingIssues,
  renderIssueBody,
  stepSummary,
  summarizeReport,
  untrackedGapMessage,
  upsertCoverageGapIssue,
} from './update-coverage-gap-issue.mjs';

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

test('updates the marked issue found on a later page without reopening it', () => {
  const mock = github([
    [{ title: ISSUE_TITLE, number: 7 }],
    [{ title: 'renamed report', body: ISSUE_MARKER, number: 123, state: 'closed' }],
  ]);
  const log = [];
  assert.equal(upsertCoverageGapIssue(repository, 'updated body', mock.run, (line) => log.push(line)), issueUrl);
  assert.deepEqual(mock.calls[1].args, ['api', `repos/${repository}/issues/123`, '--method', 'PATCH', '--input', '-']);
  // Issue #258: reopening the closed report every week put it back in the
  // hive-mind repository-mode queue as a work item.
  assert.deepEqual(JSON.parse(mock.calls[1].options.input), { title: ISSUE_TITLE, body: 'updated body' });
  assert.ok(log.some((line) => /#123 .*closed.*not reopened/.test(line)), log.join('\n'));
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

const reportWithTotals = `# Competitor coverage report

## Coverage by source

| Source | Cases | Supported | Different | Unsupported | Timeout | Coverage |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| fend | 1098 | 423 | 83 | 592 | 0 | 38.5% |
| **All** | 13975 | 2010 | 307 | 11658 | 0 | 14.4% |

## Coverage by language

| Source | Cases | Supported | Different | Unsupported | Timeout | Coverage |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| en | 13975 | 2010 | 307 | 11658 | 0 | 14.4% |
| **All** | 13975 | 2010 | 307 | 11658 | 0 | 14.4% |
`;

const tracking = [
  { number: 236, title: 'Scientific notation as written in papers', state: 'open' },
  { number: 246, title: 'Milestone: every corpus row is supported', state: 'open' },
];

test('reads the overall totals from the source table', () => {
  assert.deepEqual(summarizeReport(reportWithTotals), {
    cases: 13975, supported: 2010, different: 307, unsupported: 11658, timeout: 0, coverage: '14.4%',
  });
  assert.throws(() => summarizeReport('## Coverage by source\n\n| a |\n'), /missing the \*\*All\*\* row/);
});

test('lists the sub-issues of the tracking parent across pages', () => {
  const calls = [];
  const run = (command, args) => {
    calls.push(args);
    return JSON.stringify([
      [{ number: 229, title: 'Run the checker in CI', state: 'closed' }, { number: 236, title: 'Notation', state: 'open' }],
      [{ number: 246, title: 'Milestone', state: 'open', pull_request: {} }],
    ]);
  };
  assert.deepEqual(listTrackingIssues(repository, TRACKING_PARENT, run), [
    { number: 229, title: 'Run the checker in CI', state: 'closed' },
    { number: 236, title: 'Notation', state: 'open' },
  ]);
  assert.deepEqual(calls[0], [
    'api', `repos/${repository}/issues/${TRACKING_PARENT}/sub_issues?per_page=100`, '--paginate', '--slurp',
  ]);
});

test('says the report is a status page and lists the open tracking issues', () => {
  const body = renderIssueBody(reportWithTotals, {
    repository, sha: 'abc123', runId: '456', generated: '2026-10-13',
    trackingIssues: [...tracking, { number: 229, title: 'Done', state: 'closed' }],
  });
  assert.ok(body.includes('## Tracking issues'));
  assert.ok(body.includes('status page, not a work item'));
  assert.ok(body.includes('never reopens'));
  assert.ok(body.includes(`- #236 Scientific notation as written in papers`));
  assert.ok(body.includes(`- #246 Milestone: every corpus row is supported`));
  assert.ok(!body.includes('- #229'));
  assert.ok(body.includes(`1 sub-issue of #${TRACKING_PARENT} is already closed`));
});

test('says when no open issue tracks the remaining gaps', () => {
  const body = renderIssueBody(reportWithTotals, { repository, sha: 'abc123', runId: '456', trackingIssues: [] });
  assert.ok(body.includes(`No open sub-issue of #${TRACKING_PARENT} tracks the remaining gaps`));
});

test('fails only when gaps remain and no tracking issue is open', () => {
  const totals = summarizeReport(reportWithTotals);
  assert.equal(untrackedGapMessage(totals, tracking), null);
  assert.match(untrackedGapMessage(totals, []), /11965 of 13975 .*no sub-issue of #227 is open.*open one/);
  assert.equal(untrackedGapMessage({ ...totals, supported: totals.cases }, []), null);
});

test('writes the report to the job summary without the issue marker', () => {
  const body = renderIssueBody(reportWithTotals, { repository, sha: 'abc123', runId: '456', trackingIssues: tracking });
  const summary = stepSummary(body);
  assert.ok(!summary.includes(ISSUE_MARKER));
  assert.ok(summary.startsWith('# Coverage gap report'));
});

test('dry run reads from GitHub but only prints the write', () => {
  const reads = [];
  const printed = [];
  const run = dryRunGitHub((command, args) => {
    reads.push(args);
    return '[]';
  }, (line) => printed.push(line));
  assert.equal(run('gh', ['api', `repos/${repository}/issues?state=all`, '--paginate', '--slurp'], {}), '[]');
  const result = JSON.parse(run('gh', ['api', `repos/${repository}/issues/258`, '--method', 'PATCH', '--input', '-'], {
    input: JSON.stringify({ title: ISSUE_TITLE, body: 'b' }),
  }));
  assert.equal(reads.length, 1);
  assert.match(result.html_url, /dry run/);
  assert.match(printed.join('\n'), /PATCH repos\/link-assistant\/calculator\/issues\/258/);
});
