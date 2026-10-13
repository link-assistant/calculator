#!/usr/bin/env node
/**
 * Issue #258: show what a version of scripts/update-coverage-gap-issue.mjs
 * sends to GitHub when the coverage gap issue was closed (for example by a
 * repository-mode pull request with `Fixes #258`).
 *
 * Usage:
 *   git show 745c4345:scripts/update-coverage-gap-issue.mjs > /tmp/old/update-coverage-gap-issue.mjs
 *   node experiments/issue-258/reopen-repro.mjs /tmp/old/update-coverage-gap-issue.mjs
 *   node experiments/issue-258/reopen-repro.mjs scripts/update-coverage-gap-issue.mjs
 */
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const script = process.argv[2];
if (!script) {
  console.error('usage: reopen-repro.mjs <path to update-coverage-gap-issue.mjs>');
  process.exit(2);
}
const { ISSUE_MARKER, upsertCoverageGapIssue } = await import(pathToFileURL(resolve(script)).href);

// A fake `gh`: one closed marked issue, then echo the write request.
const requests = [];
const run = (command, args, options) => {
  if (!args.includes('--method')) {
    return JSON.stringify([[{ number: 258, title: 'Coverage gap report', body: ISSUE_MARKER, state: 'closed' }]]);
  }
  requests.push({ endpoint: args[1], method: args[args.indexOf('--method') + 1], payload: JSON.parse(options.input) });
  return JSON.stringify({ html_url: 'https://github.com/link-assistant/calculator/issues/258' });
};

upsertCoverageGapIssue('link-assistant/calculator', 'weekly body', run, console.log);
for (const { method, endpoint, payload } of requests) {
  console.log(`${method} ${endpoint} state=${payload.state ?? '(unchanged)'}`);
  console.log(payload.state === 'open' ? 'REOPENS the closed report' : 'keeps the closed report closed');
}
