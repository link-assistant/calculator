#!/usr/bin/env node
/**
 * Issue #258: hive-mind repository mode (`/solve <repository-url>`) selects
 * every open issue, including a status page that a scheduled workflow
 * maintains, and asks the pull request to close it with `Fixes`.
 *
 * Usage:
 *   node experiments/issue-258/hive-mind-dashboard-selection-repro.mjs <hive-mind checkout at ed6a155>
 */
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const checkout = process.argv[2];
if (!checkout) {
  console.error('usage: hive-mind-dashboard-selection-repro.mjs <hive-mind checkout>');
  process.exit(2);
}
const lib = await import(pathToFileURL(resolve(checkout, 'src/solve.repository-mode.lib.mjs')).href);

// The open issues of link-assistant/calculator on 2026-10-13, as the REST
// `/issues?state=open` endpoint returns them (fields trimmed).
const entries = [
  { number: 247, id: 1, title: 'PR #228 closes implementation sub-issues that it only plans, blocking hive scheduling', created_at: '2026-10-08T00:00:00Z', user: { login: 'konard', type: 'User' } },
  { number: 254, id: 2, title: 'Adopt the JavaScript-first, gated CI/CD standard and close the pipeline-template gaps', created_at: '2026-10-10T00:00:00Z', user: { login: 'konard', type: 'User' } },
  {
    number: 258, id: 3, title: 'Coverage gap report', created_at: '2026-10-12T14:25:19Z',
    user: { login: 'github-actions[bot]', type: 'Bot' },
    body: '<!-- calculator:coverage-gap-report -->\n# Coverage gap report\n\nGenerated 2026-10-12T14:25:17.468Z ...',
  },
];

const { selected } = lib.selectOldestOpenIssues(entries);
console.log(`selected: ${selected.map((issue) => `#${issue.number}`).join(', ')}`);
const dashboard = selected.find((issue) => issue.number === 258);
console.log(dashboard
  ? 'the bot-maintained status issue #258 is selected as work (no label, author or marker filter)'
  : 'the status issue is not selected');
const body = lib.buildCombinedIssueBody({
  repository: { owner: 'link-assistant', repo: 'calculator', fullName: 'link-assistant/calculator', url: 'https://github.com/link-assistant/calculator' },
  issues: selected,
  totalOpen: entries.length,
  attachmentCount: selected.length,
});
const block = body.slice(body.indexOf(lib.REQUIRED_CLOSING_REFERENCES_HEADING));
console.log(block.trim());
console.log(/Fixes #258/.test(block)
  ? 'the combined issue requires `Fixes #258`; the next scheduled run reopened it (before calculator#261), so the next /solve run picks it up again'
  : 'no Fixes line for #258');
