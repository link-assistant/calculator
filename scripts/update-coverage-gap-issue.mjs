#!/usr/bin/env node

/** Refresh the single weekly coverage gap issue from the regenerated report. */
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const ISSUE_TITLE = 'Coverage gap report';
const ISSUE_MARKER = '<!-- calculator:coverage-gap-report -->';

function renderIssueBody(report, { repository, sha, runId, generated = new Date().toISOString() }) {
  const start = report.indexOf('## Coverage by source\n');
  if (start === -1) {
    throw new Error('Coverage report is missing the source summary.');
  }
  // Keep summary tables; the full row details are available in the artifact.
  const summary = report.slice(start).split(/\n## (?:Different|Unsupported|Timeout) \(/)[0].trim();
  const runUrl = `https://github.com/${repository}/actions/runs/${runId}`;
  return `${ISSUE_MARKER}
# Coverage gap report

Generated ${generated} for [commit ${sha}](https://github.com/${repository}/commit/${sha}).
The full report, including expressions with different results, is in the
\`competitor-coverage-report\` artifact of [this workflow run](${runUrl}).

${summary}

This issue is updated weekly by the Coverage Gap Report workflow. Coverage improvements
must update the baseline in the same PR; see [CONTRIBUTING.md](https://github.com/${repository}/blob/main/CONTRIBUTING.md#competitor-coverage).
`;
}

function upsertCoverageGapIssue(repository, body, run = execFileSync) {
  const endpoint = `repos/${repository}/issues`;
  const pages = JSON.parse(run('gh', [
    'api', `${endpoint}?state=all&per_page=100`, '--paginate', '--slurp',
  ], { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 }));
  const issues = pages.flat().filter((issue) => !issue.pull_request);
  const existing = issues.find((issue) => issue.body?.includes(ISSUE_MARKER))
    ?? issues.find((issue) => issue.title === ISSUE_TITLE);
  const target = existing ? `${endpoint}/${existing.number}` : endpoint;
  const payload = { title: ISSUE_TITLE, body, ...(existing ? { state: 'open' } : {}) };
  const result = JSON.parse(run('gh', [
    'api', target, '--method', existing ? 'PATCH' : 'POST', '--input', '-',
  ], { encoding: 'utf8', input: JSON.stringify(payload) }));
  return result.html_url;
}

function main() {
  const repository = process.env.GITHUB_REPOSITORY;
  const sha = process.env.GITHUB_SHA;
  const runId = process.env.GITHUB_RUN_ID;
  if (!repository || !sha || !runId) {
    throw new Error('GITHUB_REPOSITORY, GITHUB_SHA and GITHUB_RUN_ID are required.');
  }
  const report = readFileSync('docs/case-studies/issue-227/results/coverage-report.md', 'utf8');
  const body = renderIssueBody(report, { repository, sha, runId });
  console.log(`Coverage gap issue: ${upsertCoverageGapIssue(repository, body)}`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  try {
    main();
  } catch (error) {
    console.error(`update-coverage-gap-issue: ${error.message}`);
    process.exitCode = 1;
  }
}

export { ISSUE_MARKER, ISSUE_TITLE, renderIssueBody, upsertCoverageGapIssue };
