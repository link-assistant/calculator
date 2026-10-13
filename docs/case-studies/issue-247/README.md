# A planning PR closed the implementation issues it only planned (#247)

PR #228 was the case study and work plan for #227. It added no Rust changes.
Its own description said:

> Merging this PR closes only #227. The open sub-issues #236 to #246 stay open,
> so this description deliberately has no `Fixes` lines for them.

The same description ended with a `Fixes` line for each of #229 to #246. GitHub
acts on the keywords, not on the prose. On 2026-10-12 hive-mind auto-merged the
PR, and the merge closed #236 to #246 as completed. None of those 11
implementation issues had been started. Issues #229 to #235 had already been
closed by their own PRs, so the stray lines did nothing to them.

Issue #247 reported the first symptom on 2026-10-08, before the merge. hive's
`--skip-issues-with-prs` filter counted #228 as an existing solution for every
open issue. It skipped all 19 of them, and two hive runs completed zero tasks.

## Data

Everything here was downloaded with `gh` on 2026-10-13 into [`data/`](data/):

| File | Content |
| --- | --- |
| [`issue-247.md`](data/issue-247.md) | The issue as filed |
| [`pr-228-body-at-merge.md`](data/pr-228-body-at-merge.md) | PR #228's description when it was merged |
| [`pr-228-description-edits.json`](data/pr-228-description-edits.json) | Every revision of the description (GraphQL `userContentEdits`) with its `Fixes` lines |
| [`pr-228-comments.json`](data/pr-228-comments.json) | The 11 PR comments: session summaries, logs, auto-restart, ready-to-merge, auto-merge |
| [`closed-events-227-246.json`](data/closed-events-227-246.json) | For #227 and #229 to #246: state, `closedAt`, and the PR and merge commit that closed it |
| [`hive-mind-link-repair-repro.txt`](data/hive-mind-link-repair-repro.txt), [`hive-mind-3174.md`](data/hive-mind-3174.md) | The hive-mind reproduction's output and the upstream report |
| [`pr-228-body-corrected.md`](data/pr-228-body-corrected.md) | PR #228's corrected description |
| [`negated-keyword-probe.md`](data/negated-keyword-probe.md) | A probe on PR #261 showing that "does not close #N" still closes #N |
| [`closing-references-audit.txt`](data/closing-references-audit.txt) | The new checker replayed over the descriptions of all 117 PRs in the repository |
| [`closing-references-vs-github.txt`](data/closing-references-vs-github.txt) | The issues the checker parses compared with GitHub's `closingIssuesReferences` for all 117 PRs |

## Timeline

All times are UTC.

| Time | Event | Source |
| --- | --- | --- |
| 2026-10-07 09:27:30 | PR #228 opened for #227 | edits[0] |
| 10:03–10:18 | The agent rewrites the description as a plan, with no `Fixes` lines for sub-issues | edits[1–3] |
| 10:30:40 | Working session summary | comment 6036057126 |
| 10:31:07 | hive-mind appends 18 lines, `Fixes #229` to `Fixes #246`. Session log: `Repaired 18 required issue-closing reference(s) in PR #228` | edits[4], #247 |
| 10:31:41 | Auto-restart 1/5 because of uncommitted changes | comment 6036072359 |
| 10:33:43 | The restarted agent removes the 18 lines | edits[6] |
| 10:44:47 | The post-session repair appends them again | edits[7] |
| 10:48:37 | "✅ Ready to merge" (`--auto-restart-until-mergeable`) | comment 6036328202 |
| 2026-10-08 01:44 | #247 is filed: hive skips all 19 open issues because of #228 | #247 |
| 2026-10-08 05:54 to 2026-10-12 | #229 to #235 are implemented and closed by PRs #248 to #253 and #259 | closed-events |
| 2026-10-12 16:48:36 | Maintainer asks to "resolve conflicts and double check that we implemented exactly all requirements" | comment 6128653311 |
| 19:14:42 | New session (solve v2.35.8, claude/opus) | comment 6130932988 |
| 19:16:26, 19:23:12 | The agent removes the `Fixes` lines again | edits[9–10] |
| 19:39:59 | Working session summary | comment 6131339274 |
| 19:40:26 | The post-session repair appends the 18 lines a third time | edits[11] |
| 19:44:21 | hive-mind auto-merges #228 (merge commit `da77fafe`) | comment 6131407266 |
| 19:44:22–28 | GitHub closes #227 and #236 to #246, all with `closer` = PR #228 | closed-events |

Every agent session removed the lines, and every post-session repair added
them back. The last repair ran four minutes before the merge.

## Requirements

From #247 and the repository-mode issue #260:

1. **R1.** Keep the planning PR linked to #227 only. Mention the implementation
   issues without closing keywords.
2. **R2.** Undo the damage: #236 to #246 were closed without being
   implemented. #229 to #235 were implemented by their own PRs and stay
   closed.
3. **R3.** Stop it from happening again in this repository. A description that
   contradicts its closing keywords must fail CI, because hive-mind
   auto-merges only after all checks pass.
4. **R4.** Report the orchestration cause to hive-mind with a reproduction, a
   workaround and a suggested fix. The cause is the required-issue scope that
   expands the whole sub-issue tree, plus the `finally` repair. Distinguish
   sub-issues to implement from sub-issues created as a plan, and use the same
   scope for repair, pre-merge verification and issue closing.
5. **R5.** Apply the fix everywhere: audit every PR description in the
   repository for the same class of mistake, and document the rule for
   contributors and agents.

## Root causes

### 1. hive-mind treats every native sub-issue as required

[`fetchRequiredIssueScope`](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/issue-link-verification.lib.mjs#L34-L53)
walks the whole sub-issue tree of the source issue
([line 44](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/issue-link-verification.lib.mjs#L44)
reads `…/sub_issues`) and marks every node as an issue the PR must close.
#227 asked for an analysis and for issues to be created, so #229 to #246 were
created as its sub-issues. From then on they counted as required.

[`repairRequiredIssueLinks`](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/pr-issue-link-repair.lib.mjs#L18-L44)
appends a `Fixes #N` line for every required issue that is missing
([line 32](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/pr-issue-link-repair.lib.mjs#L32)).
It runs in a
[`finally` block](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/solve.restart-shared.lib.mjs#L551-L555)
after every session, so it always has the last word. When the agent removed
the lines, that was undone each time.

There is no way to say "these sub-issues are the plan, not the scope". The
issue body's prose ("this issue's own result is the analysis and issue
creation") is not read.

### 2. GitHub closes on keywords, whatever the prose says

GitHub closes every issue referenced by a closing keyword in a PR merged into
the default branch
([docs](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/linking-a-pull-request-to-an-issue)).
It does not read the sentence around the keyword. Negation is ignored too.
Draft PR #261 temporarily said "this pull request does not close #258", and
`closingIssuesReferences` then listed #258
([probe](data/negated-keyword-probe.md)). GitLab
[documents the same behaviour](https://gitlab.com/gitlab-org/gitlab-foss/-/work_items/55970).
A careful sentence can therefore never repair a stray `Fixes` line.

### 3. Nothing in CI compared the description with its keywords

The release pipeline checks code, changelog fragments, links, security and
workflows. Nothing checked the description, which here was the only artifact
that mattered. The `ready-to-merge` and `auto-merge` steps saw green checks and
merged.

### 4. Secondary: the hive schedule was blocked

hive's `--skip-issues-with-prs` counts an open PR with a closing reference as
the solution for that issue. With 18 stray lines, one planning PR blocked
the whole repository. That was the symptom #247 reported. hive-mind#2685 and
PR hive-mind#2686 made it report "no work" instead of success, but they did not
change the scope.

## Repository-wide audit

[`experiments/issue-247/audit-closing-references.mjs`](../../../experiments/issue-247/audit-closing-references.mjs)
replays the new checker over every PR description in the repository
(117 PRs, from `gh api repos/link-assistant/calculator/pulls?state=all`). The
[result](data/closing-references-audit.txt) has three hits:

| PR | Rule | Effect |
| --- | --- | --- |
| #228 | `keep-open`, `closes-only` | Closed #236 to #246 unimplemented (this issue) |
| #82 | `missing-keyword` | `Fixes #36, #34, #23, #68, #45` closes only #36. #34, #23, #68 and #45 were later closed by hand |
| #61 | `missing-keyword` | `Fixes #51, #52, #53` closes only #51. #52 and #53 were later closed by hand |

The other 114 descriptions pass. #82 and #61 needed no further action: all the
issues they meant to close are closed. They show the opposite failure, where a
keyword list closes fewer issues than intended. That is why the checker
covers that case as well.

## Solution

| Requirement | Change |
| --- | --- |
| R3, R5 | [`scripts/check-closing-references.mjs`](../../../scripts/check-closing-references.mjs) parses closing keywords the way GitHub does. It ignores keywords in code, inline code and HTML comments; handles `#N`, `owner/repo#N` and issue URLs; and needs one keyword per issue. It fails on five rules: `keep-open`, `closes-only`, `part-of`, `missing-keyword` and `negated-keyword`. Each error is an `::error` annotation with the line number, and `--verbose` prints every reference it parsed. |
| R3 | [`.github/workflows/pr-description.yml`](../../../.github/workflows/pr-description.yml) runs it on `opened`, `edited`, `synchronize` and `reopened`, so an automated description edit re-runs the check. It has no path filter and read-only permissions, and reads the description from the event payload, never from `${{ }}` interpolation. |
| R3 | [`scripts/check-closing-references.test.mjs`](../../../scripts/check-closing-references.test.mjs) uses PR #228's real description as a fixture and expects exactly the 11 `keep-open` and 18 `closes-only` errors. The same text without the appended lines must pass. Other fixtures: PRs #61, #82 and #95, and the repository-mode block of #260. |
| R1, R5 | `CONTRIBUTING.md` gains a [Closing Issues](../../../CONTRIBUTING.md#closing-issues) section: one keyword per issue on its own line; `Part of #N` or "#N stays open" for issues a PR only plans; remove automatically appended references before merging. `scripts/check-required-docs.mjs` keeps that section from being removed. |
| R2 | #236 to #246 were reopened on 2026-10-13, each with a comment explaining the mistake and linking to this case study. |
| R1 | PR #228's description was corrected ([new text](data/pr-228-body-corrected.md)). The stray `Fixes #229` to `Fixes #246` lines are replaced by a note explaining the mistake, and `Closes #227` stays. `closingIssuesReferences` now lists only #227. The checker caught a first draft of that note: "so this merge closed #236" is itself a closing keyword. |
| R4 | [link-assistant/hive-mind#3174](https://github.com/link-assistant/hive-mind/issues/3174) ([copy](data/hive-mind-3174.md)) reports the plan-versus-implement scope, the `finally` repair that overrides the agent, and the negation mismatch between hive-mind's shared parser and GitHub. It includes a reproduction, workarounds and a suggested fix. The reproduction, [`experiments/issue-247/hive-mind-link-repair-repro.mjs`](../../../experiments/issue-247/hive-mind-link-repair-repro.mjs), calls hive-mind's own `repairRequiredIssueLinks` with a fake `gh`. It shows that the repair re-adds the lines after the agent removes them, and that `extractClosingIssueReferences` ignores "does not close #2" ([output](data/hive-mind-link-repair-repro.txt)). |

The workflow would have stopped #228. The first `edited` event at 10:31:07
would have failed the check. hive-mind does not merge a PR with failing
checks, so #228 would have stayed open until someone removed the lines. The
`finally` repair would add them again after every session, but the check
would fail every time instead of letting the merge through.

## Alternatives considered

- **Query GitHub's `closingIssuesReferences` instead of parsing the
  description.** That is the ground truth, and it also covers issues linked
  by hand in the Development sidebar. However, it needs a token with
  `pull-requests: read`. It is also updated asynchronously after an `edited`
  event, so a check that runs immediately can see stale data. Run locally, it
  needs a PR number before the PR exists. Instead,
  [`experiments/issue-247/compare-closing-references.mjs`](../../../experiments/issue-247/compare-closing-references.mjs)
  compares the parser with `closingIssuesReferences` for all 117 PRs. They
  [agree on all 117](data/closing-references-vs-github.txt), including #228
  after the merge. The remaining gap is sidebar links: they never appear in
  the description. The `gh pr view N --json closingIssuesReferences` command
  in the CONTRIBUTING section shows them.
- **Existing actions.**
  [nearform-actions/github-action-check-linked-issues](https://github.com/nearform-actions/github-action-check-linked-issues)
  and
  [HarshCasper/validate-issues-over-pull-requests](https://github.com/HarshCasper/validate-issues-over-pull-requests)
  require at least one linked issue.
  [gpsgate/pr-linked-issues-action](https://github.com/gpsgate/pr-linked-issues-action)
  lists the issues a PR would close. None of them compares the keywords with
  what the description says should stay open. That comparison is the failure
  here. The repository's pinning policy would also require hash pins for all
  three.
- **Only fix hive-mind.** That is necessary (R4), but it does not protect this
  repository from a person writing `Fixes #1, #2`, from "does not close #N",
  or from another tool.

## Reproduce

```bash
# The contradiction in #228's description at merge:
node scripts/check-closing-references.mjs \
  --body-file docs/case-studies/issue-247/data/pr-228-body-at-merge.md --verbose

# Which PR closed each issue:
gh api graphql -f query='{repository(owner:"link-assistant",name:"calculator"){
  issue(number:236){timelineItems(itemTypes:CLOSED_EVENT,first:5){nodes{
  ... on ClosedEvent{createdAt closer{... on PullRequest{number}}}}}}}}'

# Replay the checker over every PR description:
gh api 'repos/link-assistant/calculator/pulls?state=all' --paginate \
  --jq '.[] | {number,state,merged_at,title,body}' > /tmp/prs.jsonl
node experiments/issue-247/audit-closing-references.mjs /tmp/prs.jsonl

# Compare the parser with GitHub's closingIssuesReferences:
gh pr list --repo link-assistant/calculator --state all --limit 1000 \
  --json number,state,baseRefName,body,closingIssuesReferences > /tmp/prs.json
node experiments/issue-247/compare-closing-references.mjs /tmp/prs.json

node --test scripts/check-closing-references.test.mjs
```
