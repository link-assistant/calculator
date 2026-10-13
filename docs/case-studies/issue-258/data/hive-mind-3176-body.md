## Problem

`/solve <repository-url>` (repository mode) selects **every** open issue as work, including status pages that a bot or a scheduled workflow maintains (coverage reports, dependency dashboards, release checklists). The combined issue then requires `Fixes #<status page>` in the pull request.

Such issues are often reopened or recreated by their workflow, so this becomes a loop: the PR closes the status page, the workflow reopens it on its next run, the next repository-mode run selects it again, and another solver session spends a full run on an issue that has nothing to implement.

Real case: link-assistant/calculator#258 ("Coverage gap report") is created and updated weekly by `github-actions[bot]`. On 2026-10-13, `/solve https://github.com/link-assistant/calculator` created link-assistant/calculator#260 with three open issues, and its required closing block contained `Fixes #258`. Until link-assistant/calculator#261, that repository's weekly workflow sent `state: 'open'` with every update, so a close would have lasted only until the next Monday.

## Reproduction

hive-mind `ed6a15520e49aafd49c06b49733ae8c28a15c4a6` (v2.35.8), `src/solve.repository-mode.lib.mjs`:

```js
import { selectOldestOpenIssues, buildCombinedIssueBody, REQUIRED_CLOSING_REFERENCES_HEADING } from './src/solve.repository-mode.lib.mjs';

const entries = [
  { number: 247, id: 1, title: 'A real bug', created_at: '2026-10-08T00:00:00Z', user: { login: 'konard', type: 'User' } },
  { number: 254, id: 2, title: 'A real feature', created_at: '2026-10-10T00:00:00Z', user: { login: 'konard', type: 'User' } },
  { number: 258, id: 3, title: 'Coverage gap report', created_at: '2026-10-12T14:25:19Z',
    user: { login: 'github-actions[bot]', type: 'Bot' },
    body: '<!-- calculator:coverage-gap-report -->\n# Coverage gap report\n...' },
];
const { selected } = selectOldestOpenIssues(entries);
console.log(selected.map((issue) => issue.number)); // [ 247, 254, 258 ]
const body = buildCombinedIssueBody({
  repository: { owner: 'link-assistant', repo: 'calculator', fullName: 'link-assistant/calculator', url: 'https://github.com/link-assistant/calculator' },
  issues: selected, totalOpen: entries.length, attachmentCount: selected.length,
});
console.log(body.slice(body.indexOf(REQUIRED_CLOSING_REFERENCES_HEADING)));
// ```
// Fixes #247
// Fixes #254
// Fixes #258   <- the bot-maintained status page
// ```
```

The full script and its output: https://github.com/link-assistant/calculator/blob/09410d73/experiments/issue-258/hive-mind-dashboard-selection-repro.mjs and https://github.com/link-assistant/calculator/blob/09410d73/docs/case-studies/issue-258/data/hive-mind-dashboard-selection-repro.txt

## Root cause

- [`selectOldestOpenIssues`](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/solve.repository-mode.lib.mjs#L109-L131) filters out only pull requests. It accepts an `exclude` list of issue numbers, but [`prepareRepositoryModeIssue`](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/solve.repository-mode.run.lib.mjs#L137-L141) never passes it, and no CLI option sets it.
- Nothing looks at labels, the author type, or a body marker, so a repository cannot say "this issue is not work".
- [`buildCombinedIssueBody`](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/solve.repository-mode.lib.mjs) puts every selected issue in the required closing block. Combined with the post-session link repair (#3174), the pull request ends up closing the status page whatever the agent decides.

## Workarounds

- In the repository: make the status workflow update its issue without reopening it, so one close sticks. Example: https://github.com/link-assistant/calculator/pull/261 (`scripts/update-coverage-gap-issue.mjs` no longer sends `state`; the body lists the issues that track the actual work).
- Close the status page as "not planned" before running repository mode, if its workflow does not reopen it.
- Solve the real issues one by one (`/solve <issue-url>`) instead of using repository mode.

## Suggested fix

1. **Let a repository opt issues out.** Skip open issues that carry a label from a configurable list (for example `--skip-issue-labels`, defaulting to something like `hive-mind:skip`, `dashboard`, `status`) or a body marker such as `<!-- hive-mind:skip -->`. Pass the result through the existing `exclude` option of `selectOldestOpenIssues`.
2. **Treat bot-authored issues as opt-in, or at least flag them.** Issues with `user.type === 'Bot'` are mostly reports and dashboards (Renovate's "Dependency Dashboard", coverage and benchmark reports). Either skip them unless a flag includes them, or keep them but list them separately in the combined issue ("maintained by a bot, close only if the PR changes what it reports").
3. **Show what was left out.** `buildCombinedIssueBody` already reports issues over the sub-issue limit. Add a line for issues skipped by label, marker or author, with the reason, so the scope stays reviewable.
4. **Regression tests.** `selectOldestOpenIssues` with a labeled issue, a marked issue and a bot-authored issue. `prepareRepositoryModeIssue` passing the exclusions. The combined body listing the skipped issues and leaving them out of the closing block.

Related: #3174 (post-session link repair re-adds `Fixes` lines for the whole sub-issue tree).
