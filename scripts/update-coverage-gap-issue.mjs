#!/usr/bin/env node

/**
 * Refresh the single coverage gap issue from the regenerated report.
 *
 * The issue is a status page, not a work item (#258). An existing issue is
 * updated in place and never reopened: reopening it every week put the closed
 * report back in the hive-mind repository-mode queue, where a pull request
 * closed it again. The missing features are tracked by the sub-issues of
 * #227, which the body lists.
 *
 * Usage:
 *   node scripts/update-coverage-gap-issue.mjs [--verbose] [--dry-run]
 *
 * --verbose (or COVERAGE_GAP_VERBOSE=1) logs the issue that was found and
 * what was done with it. --dry-run reads from GitHub but only prints the
 * create or update request; outside GitHub Actions it uses the `gh` default
 * repository and the local HEAD.
 */
import { execFileSync } from 'node:child_process';
import { appendFileSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const ISSUE_TITLE = 'Coverage gap report';
const ISSUE_MARKER = '<!-- calculator:coverage-gap-report -->';
const REPORT_PATH = 'docs/case-studies/issue-227/results/coverage-report.md';
// The competitor parity plan; its sub-issues implement the missing features.
const TRACKING_PARENT = 227;

function sourceSummary(report) {
  const start = report.indexOf('## Coverage by source\n');
  if (start === -1) {
    throw new Error('Coverage report is missing the source summary.');
  }
  // Keep summary tables; the full row details are available in the artifact.
  return report.slice(start).split(/\n## (?:Different|Unsupported|Timeout) \(/)[0].trim();
}

function summarizeReport(report) {
  const row = sourceSummary(report).split('\n').find((line) => line.startsWith('| **All** |'));
  if (!row) {
    throw new Error('Coverage report source table is missing the **All** row.');
  }
  const [cases, supported, different, unsupported, timeout, coverage] = row.split('|').slice(2, 8)
    .map((cell) => cell.trim());
  return {
    cases: Number(cases),
    supported: Number(supported),
    different: Number(different),
    unsupported: Number(unsupported),
    timeout: Number(timeout),
    coverage,
  };
}

function listTrackingIssues(repository, parent = TRACKING_PARENT, run = execFileSync) {
  const pages = JSON.parse(run('gh', [
    'api', `repos/${repository}/issues/${parent}/sub_issues?per_page=100`, '--paginate', '--slurp',
  ], { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 }));
  return pages.flat().filter((issue) => !issue.pull_request)
    .map(({ number, title, state }) => ({ number, title, state }));
}

function renderTracking(trackingIssues, parent) {
  const open = trackingIssues.filter((issue) => issue.state === 'open');
  const closed = trackingIssues.length - open.length;
  const lines = [
    '## Tracking issues',
    '',
    'This issue is a status page, not a work item: the workflow updates it in place',
    'and never reopens it, so it may stay closed. The missing features are implemented',
    `through the open sub-issues of #${parent}:`,
    '',
  ];
  if (open.length === 0) {
    lines.push(`No open sub-issue of #${parent} tracks the remaining gaps. Open one for the next missing feature.`);
  } else {
    lines.push(...open.map((issue) => `- #${issue.number} ${issue.title}`));
  }
  if (closed > 0) {
    lines.push('', `${closed} sub-issue${closed === 1 ? '' : 's'} of #${parent} ${closed === 1 ? 'is' : 'are'} already closed.`);
  }
  return lines.join('\n');
}

function renderIssueBody(report, {
  repository, sha, runId, generated = new Date().toISOString(), trackingIssues = [], parent = TRACKING_PARENT,
}) {
  const summary = sourceSummary(report);
  const runUrl = `https://github.com/${repository}/actions/runs/${runId}`;
  return `${ISSUE_MARKER}
# Coverage gap report

Generated ${generated} for [commit ${sha}](https://github.com/${repository}/commit/${sha}).
The full report, including expressions with different results, is in the
\`competitor-coverage-report\` artifact of [this workflow run](${runUrl}).

${summary}

${renderTracking(trackingIssues, parent)}

This issue is updated weekly, and when the coverage baseline changes on \`main\`, by the
Coverage Gap Report workflow. Coverage improvements must update the baseline in the same PR;
see [CONTRIBUTING.md](https://github.com/${repository}/blob/main/CONTRIBUTING.md#competitor-coverage).
`;
}

function untrackedGapMessage(totals, trackingIssues, parent = TRACKING_PARENT) {
  const gaps = totals.cases - totals.supported;
  if (gaps <= 0 || trackingIssues.some((issue) => issue.state === 'open')) {
    return null;
  }
  return `${gaps} of ${totals.cases} corpus expressions are not supported, but no sub-issue of #${parent} `
    + 'is open: open one for the next missing feature (see the Coverage gap report issue) so the gaps stay tracked.';
}

function stepSummary(body) {
  return `${body.replace(`${ISSUE_MARKER}\n`, '')}\n`;
}

function upsertCoverageGapIssue(repository, body, run = execFileSync, log = () => {}) {
  const endpoint = `repos/${repository}/issues`;
  const pages = JSON.parse(run('gh', [
    'api', `${endpoint}?state=all&per_page=100`, '--paginate', '--slurp',
  ], { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 }));
  const issues = pages.flat().filter((issue) => !issue.pull_request);
  const existing = issues.find((issue) => issue.body?.includes(ISSUE_MARKER))
    ?? issues.find((issue) => issue.title === ISSUE_TITLE);
  if (existing) {
    log(`Updating #${existing.number} (${existing.state ?? 'unknown state'}); its state is kept, a closed report is not reopened.`);
  } else {
    log(`No issue has the marker or the title "${ISSUE_TITLE}"; creating it.`);
  }
  const target = existing ? `${endpoint}/${existing.number}` : endpoint;
  // Never send `state`: whoever closed the report decides when it reopens.
  const payload = { title: ISSUE_TITLE, body };
  const result = JSON.parse(run('gh', [
    'api', target, '--method', existing ? 'PATCH' : 'POST', '--input', '-',
  ], { encoding: 'utf8', input: JSON.stringify(payload) }));
  return result.html_url;
}

function dryRunGitHub(run = execFileSync, print = console.log) {
  return (command, args, options) => {
    const method = args[args.indexOf('--method') + 1];
    if (!args.includes('--method') || method === 'GET') {
      return run(command, args, options);
    }
    print(`[dry run] ${method} ${args[1]}:\n${JSON.parse(options.input).body}`);
    return JSON.stringify({ html_url: `(dry run: ${method} ${args[1]})` });
  };
}

function parseArgs(argv, env) {
  const options = { verbose: env.COVERAGE_GAP_VERBOSE === '1', dryRun: false };
  for (const arg of argv) {
    if (arg === '--verbose') {
      options.verbose = true;
    } else if (arg === '--dry-run') {
      options.dryRun = true;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  return options;
}

function localDefault(command, args) {
  return execFileSync(command, args, { encoding: 'utf8' }).trim();
}

function main(argv = process.argv.slice(2), env = process.env) {
  const options = parseArgs(argv, env);
  const log = options.verbose ? (line) => console.log(line) : () => {};
  const run = options.dryRun ? dryRunGitHub() : execFileSync;
  const repository = env.GITHUB_REPOSITORY
    ?? (options.dryRun ? localDefault('gh', ['repo', 'view', '--json', 'nameWithOwner', '--jq', '.nameWithOwner']) : null);
  const sha = env.GITHUB_SHA ?? (options.dryRun ? localDefault('git', ['rev-parse', 'HEAD']) : null);
  const runId = env.GITHUB_RUN_ID ?? (options.dryRun ? 'local' : null);
  if (!repository || !sha || !runId) {
    throw new Error('GITHUB_REPOSITORY, GITHUB_SHA and GITHUB_RUN_ID are required (or pass --dry-run).');
  }
  const report = readFileSync(REPORT_PATH, 'utf8');
  const totals = summarizeReport(report);
  log(`Report totals: ${JSON.stringify(totals)}`);
  const trackingIssues = listTrackingIssues(repository, TRACKING_PARENT, run);
  log(`Sub-issues of #${TRACKING_PARENT}: ${trackingIssues.map((issue) => `#${issue.number} ${issue.state}`).join(', ') || 'none'}`);
  const body = renderIssueBody(report, { repository, sha, runId, trackingIssues });
  if (env.GITHUB_STEP_SUMMARY) {
    appendFileSync(env.GITHUB_STEP_SUMMARY, stepSummary(body));
  }
  console.log(`Coverage gap issue: ${upsertCoverageGapIssue(repository, body, run, log)}`);
  const untracked = untrackedGapMessage(totals, trackingIssues);
  if (untracked) {
    throw new Error(untracked);
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  try {
    main();
  } catch (error) {
    const prefix = process.env.GITHUB_ACTIONS === 'true' ? '::error title=Coverage gap report::' : '';
    console.error(`${prefix}update-coverage-gap-issue: ${error.message}`);
    process.exitCode = 1;
  }
}

export {
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
};
