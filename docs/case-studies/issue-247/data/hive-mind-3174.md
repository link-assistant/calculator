## Summary

`solve` treats every native sub-issue of the source issue as an issue the PR must close. When the source issue asks for a **plan** (analysis plus creating implementation issues), the session attaches the implementation issues as sub-issues. From then on the post-session repair appends `Fixes #N` for each of them after **every** session, even when the agent deliberately removed those lines. Auto-merge then closes issues that were never implemented.

That is what happened in link-assistant/calculator:

- PR link-assistant/calculator#228 was the plan for link-assistant/calculator#227. Its description said: `Merging this PR closes only #227. The open sub-issues #236 to #246 stay open, so this description deliberately has no Fixes lines for them.`
- The repair appended `Fixes #229` … `Fixes #246` at 2026-10-07T10:31:07Z. The agent removed them at 10:33:43Z, and the repair re-added them at 10:44:47Z.
- On 2026-10-12 the agent removed them twice more (19:16:26Z and 19:23:12Z). The repair re-added them at 19:40:26Z.
- hive-mind auto-merged at 19:44:21Z. GitHub then closed link-assistant/calculator#236 to link-assistant/calculator#246, which were not implemented. They have now been reopened by hand.
- Before the merge, the same lines made `--skip-issues-with-prs` skip every open issue in the repository (link-assistant/calculator#247, #2685).

The full timeline, every description revision and the closed events are in [docs/case-studies/issue-247](https://github.com/link-assistant/calculator/blob/1b91a43a/docs/case-studies/issue-247/README.md).

## Root cause (at ed6a155, v2.35.8)

1. [`fetchRequiredIssueScope`](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/issue-link-verification.lib.mjs#L34-L53) walks `…/sub_issues` recursively ([L44](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/issue-link-verification.lib.mjs#L44)) and marks every node as required. Nothing separates "sub-issues to implement" from "sub-issues created as the plan's output".
2. [`repairRequiredIssueLinks`](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/pr-issue-link-repair.lib.mjs#L18-L44) appends a `Fixes` line for each missing one ([L32](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/pr-issue-link-repair.lib.mjs#L32)). It runs in the [`finally` block](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/solve.restart-shared.lib.mjs#L551-L555) after every session, so it always overrides the agent. It ignores a description that explicitly says the issue stays open.
3. Auto-merge does not compare what GitHub will close (`closingIssuesReferences`) with what the PR implements.

### Second problem: the shared parser and GitHub disagree on negation

[`extractClosingIssueReferences`](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/github-linking.lib.mjs#L38-L53) skips a keyword whose clause contains `not`, `never`, `won't` and so on. The comment there cites a PR body that said `does not close #322`. GitHub does not skip such keywords. On link-assistant/calculator#261, adding `Probe for #247: this pull request does not close #258.` made `gh pr view 261 --json closingIssuesReferences` return `[258, 260]`. Merging would have closed link-assistant/calculator#258. GitLab documents the same behaviour: https://gitlab.com/gitlab-org/gitlab-foss/-/work_items/55970. So hive-mind's linking checks can say "#N is not closed" while the merge closes it.

## Reproduction

The script below calls hive-mind's own `repairRequiredIssueLinks` and `extractClosingIssueReferences`, with a fake `gh` runner. It needs a checkout at ed6a155 with `npm ci --ignore-scripts --omit=dev`, then `node repro.mjs <checkout>`.

<details><summary>repro.mjs</summary>

```js
// Reproduces link-assistant/calculator#247 against hive-mind's own modules.
// Usage: node repro.mjs <path to hive-mind checkout>
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const src = (file) => pathToFileURL(resolve(process.argv[2], 'src', file)).href;
const { repairRequiredIssueLinks } = await import(src('pr-issue-link-repair.lib.mjs'));
const { extractClosingIssueReferences } = await import(src('github-linking.lib.mjs'));

// #1 asks for a plan; #2 and #3 are the implementation issues the session
// created and attached as native sub-issues.
const issues = {
  1: "Analyse the problem and create one issue per part of the work. This issue's own result is the analysis and the issue creation.",
  2: 'Implement part A.',
  3: 'Implement part B.',
};
const subIssues = { 1: [2, 3], 2: [], 3: [] };
let prBody = 'Closes #1\n\nThis PR only plans the work: #2 and #3 stay open.';

const ok = (value) => ({ code: 0, stdout: JSON.stringify(value), stderr: '' });
const run = async (args) => {
  const [command, target] = args;
  if (command === 'api' && /issues\/\d+\/sub_issues$/.test(target)) {
    const number = Number(target.split('/').at(-2));
    return ok([subIssues[number].map((n) => ({ number: n, repository_url: 'https://api.github.com/repos/o/r' }))]);
  }
  if (command === 'api' && /issues\/\d+$/.test(target)) {
    const number = Number(target.split('/').at(-1));
    return ok({ number, body: issues[number] });
  }
  if (command === 'api' && /pulls\/10$/.test(target)) return ok({ number: 10, body: prBody });
  if (command === 'pr' && target === 'edit') {
    prBody = readFileSync(args[args.indexOf('--body-file') + 1], 'utf8');
    return { code: 0, stdout: '', stderr: '' };
  }
  throw new Error(`unexpected gh ${args.join(' ')}`);
};

const repair = () => repairRequiredIssueLinks({ owner: 'o', repo: 'r', issueNumber: 1, prNumber: 10, run, logger: async (m) => console.log('  log:', m) });

console.log('1. Session ends; the finally-block repair runs:');
await repair();
console.log(JSON.stringify(prBody));

console.log('2. The next session removes the lines again; the repair re-adds them:');
prBody = prBody.replace(/\n\n---\n\nFixes #2\nFixes #3$/, '');
await repair();
console.log(JSON.stringify(prBody));

console.log('3. hive-mind and GitHub disagree on negated keywords:');
const text = 'This PR does not close #2.';
console.log(`  extractClosingIssueReferences(${JSON.stringify(text)}) =`, JSON.stringify(extractClosingIssueReferences(text)));
console.log('  GitHub: closingIssuesReferences lists #2, and merging closes it (probe on link-assistant/calculator#261).');
```

</details>

Output:

```
1. Session ends; the finally-block repair runs:
  log: 🔗 Repaired 2 required issue-closing reference(s) in PR #10.
"Closes #1\n\nThis PR only plans the work: #2 and #3 stay open.\n\n---\n\nFixes #2\nFixes #3"
2. The next session removes the lines again; the repair re-adds them:
  log: 🔗 Repaired 2 required issue-closing reference(s) in PR #10.
"Closes #1\n\nThis PR only plans the work: #2 and #3 stay open.\n\n---\n\nFixes #2\nFixes #3"
3. hive-mind and GitHub disagree on negated keywords:
  extractClosingIssueReferences("This PR does not close #2.") = []
  GitHub: closingIssuesReferences lists #2, and merging closes it (probe on link-assistant/calculator#261).
```

## Workarounds

- In repositories hive-mind auto-merges, add a required check that fails when the description's closing keywords contradict its prose. link-assistant/calculator#261 adds one: [`scripts/check-closing-references.mjs`](https://github.com/link-assistant/calculator/blob/1b91a43a/scripts/check-closing-references.mjs), run on `pull_request: [opened, edited, synchronize, reopened]`. With that check failing, hive-mind does not auto-merge.
- For planning issues, run without auto-merge, and remove the appended lines by hand right before merging.
- Do not write "does not close #N". Write "#N stays open" without a keyword.

## Suggested fix

1. **Scope.** In `fetchRequiredIssueScope`, do not treat a native sub-issue as required when either of these holds:
   - it was created after the PR was opened (plan output), or
   - another PR already closes it (`closedByPullRequestsReferences`).

   Keep the explicit repository-mode block (`parseRequiredClosingReferences`) and the current all-sub-issues behaviour when the parent asks for implementation, for example with an opt-in marker or flag.
2. **Respect the description.** In `repairRequiredIssueLinks`, do not add `Fixes #N` when the body mentions `#N` without a keyword, for example in "Part of #N", "#N stays open" or "#N to #M stay open". Log it instead: `⚠️ #N is in the sub-issue tree but the description keeps it open; not adding a closing reference.`
3. **Never undo the agent.** If the agent removed a reference the previous repair added, do not re-add it in the `finally` block. Report the conflict in the session summary instead.
4. **Parser parity.** Make `extractClosingIssueReferences` return negated references, because GitHub closes them, and report them as errors ("'does not close #N' still closes #N"). Alternatively, use GraphQL `closingIssuesReferences` as the source of truth for repair, skip and merge checks. Before auto-merge, refuse when `closingIssuesReferences` includes an issue that is still open, is in the sub-issue tree, and has no commits in the PR (this overlaps #2406).
5. **Regression test.** Use the link-assistant/calculator#227 wording ("this issue's own result is the analysis and issue creation") and assert that the implementation sub-issues created during the session are not added.

Related: #2685 / #2686 (the no-work report this caused), #2406 (complete delivery before auto-merge), #2615 (sub-issue and blocker awareness in `/hive`), #2891 (kill notices posted to the parent plan PR), link-assistant/calculator#247, link-assistant/calculator#260.
