# Case study: issue #258, the weekly coverage gap report as a work item

Issue: https://github.com/link-assistant/calculator/issues/258
Parent: https://github.com/link-assistant/calculator/issues/260 (hive-mind repository mode)
Pull request: https://github.com/link-assistant/calculator/pull/261

## Summary

#258 is not a request. The scheduled **Coverage Gap Report** workflow (#229)
creates it and keeps it updated. On 2026-10-12 it reported 1857 of 13,975
competitor corpus expressions as supported (13.3%). The day after, hive-mind
repository mode (`/solve https://github.com/link-assistant/calculator`) listed
it in #260 as one of three issues to close with `Fixes #258`.

Closing it would not have lasted. `scripts/update-coverage-gap-issue.mjs`
patched the existing issue with `state: 'open'` on every run, so the next
Monday's run would reopen it. The next repository-mode run would then select it
again, and another pull request would close it again. Each round costs a solver
session that changes no code. The gaps it reports, 11,965 unsupported or
different expressions, are the subject of eleven implementation issues (#236 to
#246). No single pull request can close them.

This pull request keeps `Fixes #258` and fixes the loop:

- The workflow updates the report issue in place and never reopens it.
- The issue lists the open sub-issues of #227 that track the gaps, so readers
  and agents can see where the work is.
- The run also writes the tables to its job summary.
- The workflow also runs when the coverage baseline changes on `main`.
- It fails when gaps remain but no sub-issue of #227 is open to track them.

The gaps themselves stay open in #236 to #246. The #247 part of this pull
request reopened them.

## Data

| File | What it is |
| --- | --- |
| [`data/issue-258-as-created.md`](data/issue-258-as-created.md) | The issue as the workflow created it (commit `0e388a9`, 13.3%) |
| [`data/issue-258-timeline.json`](data/issue-258-timeline.json) | Its timeline: cross-referenced by #260, attached as a sub-issue, cross-referenced by #261 and hive-mind#3174 |
| [`data/coverage-gap-report-runs.json`](data/coverage-gap-report-runs.json) | All runs of the workflow before this PR (one scheduled run) |
| [`data/run-38262647800.log`](data/run-38262647800.log), [`data/run-38262647800-jobs.json`](data/run-38262647800-jobs.json) | The run that created #258 |
| [`data/sub-issues-227.tsv`](data/sub-issues-227.tsv) | Sub-issues of #227 on 2026-10-13: #229 to #235 closed, #236 to #246 open |
| [`data/reopen-repro.txt`](data/reopen-repro.txt) | [`experiments/issue-258/reopen-repro.mjs`](../../../experiments/issue-258/reopen-repro.mjs) before and after the fix |
| [`data/hive-mind-dashboard-selection-repro.txt`](data/hive-mind-dashboard-selection-repro.txt) | [`experiments/issue-258/hive-mind-dashboard-selection-repro.mjs`](../../../experiments/issue-258/hive-mind-dashboard-selection-repro.mjs): hive-mind's own selection code produces #260's closing block |
| [`data/dry-run-after.log`](data/dry-run-after.log) | `node scripts/update-coverage-gap-issue.mjs --dry-run --verbose` against the live repository after the fix |

## Timeline (UTC)

| Time | Event |
| --- | --- |
| 2026-10-08 05:15 | `af379249` adds `coverage-gap-report.yml` and `update-coverage-gap-issue.mjs` for #229 ("opens or updates a single 'coverage gap report' issue"). PR #248 merges it at 05:54 |
| 2026-10-11 18:49–19:15 | `822df334` and `2e65a7be` (#256) harden the workflow. The issue logic does not change |
| 2026-10-12 06:00 | The Monday cron is due |
| 2026-10-12 14:24:13 | Run [38262647800](https://github.com/link-assistant/calculator/actions/runs/38262647800) starts, 8 h 24 min late (GitHub may delay scheduled runs under load). It measures `0e388a9` (v0.23.6) |
| 2026-10-12 14:25:19 | The run creates #258: 1857 of 13,975 supported, 13.3% |
| 2026-10-12 18:06 | `78718cba` (#235, dimensional units) updates the baseline to 2010 supported, 14.4%. PR #259 merges at 18:55, so #258 is already stale |
| 2026-10-12 19:44 | PR #228 merges. Its `Fixes` lines close #229 to #246, including the unimplemented #236 to #246 (see [issue-247](../issue-247/README.md)) |
| 2026-10-13 05:18:25 | Repository mode creates #260. Only #247, #254 and #258 are open, and it requires `Fixes #258` |
| 2026-10-13 | This PR reopens #236 to #246 (#247) and stops the weekly reopen of the report (#258) |
| 2026-10-19 06:00 | Next scheduled run. Before this PR, it would reopen #258 once a PR had closed it |

## Requirements

#258 itself has no explicit requirements. #260 sets these for every listed
issue, and the case for #258 adds the derived ones below.

| # | Requirement | Source | Status |
| --- | --- | --- | --- |
| R1 | Implement what the issue asks, in this PR; keep `Fixes #258`. If the issue is already resolved or not reproducible, say so | #260 requirements 1–5 | The issue asks for nothing beyond being a report. The PR says so and keeps `Fixes #258` |
| R2 | Closing #258 must stick, so repository mode does not get it back every week | Derived: a weekly reopen would undo R1 | **Done.** The workflow never sends `state` ([`update-coverage-gap-issue.mjs`](../../../scripts/update-coverage-gap-issue.mjs)). Test: "updates the marked issue found on a later page without reopening it" |
| R3 | Keep #229's guarantee: one issue, created or updated, with the source and language tables, so the gap is visible without a PR | #229 acceptance criteria | **Kept.** The existing issue is found by marker or title in any state and updated in place. A missing issue is still created. The tables are also in the job summary |
| R4 | Say where the work on the gaps is, so readers and agents do not treat the report as the work item | Derived: #260 listed the report as work | **Done.** A "Tracking issues" section lists the open sub-issues of #227, read from the sub-issues API |
| R5 | Keep the report current | Derived: #258 showed 13.3% while `main` was at 14.4% | **Done.** It also runs on pushes to `main` that change `coverage-baseline.json` |
| R6 | If the gaps have no owner, say what to do instead of reopening a status page | Derived from R2, and the repository rule that a check needing human action fails on `main` with an explicit message | **Done.** After updating the issue, the run fails when gaps remain and no sub-issue of #227 is open, with the message "open one for the next missing feature" |
| R7 | Add verbose output where the cause is unclear | #254 footer (applies to the whole PR) | **Done.** `--verbose` (or `COVERAGE_GAP_VERBOSE=1`) logs the totals, the sub-issue states and the issue found. `--dry-run` reads from GitHub and prints the write. The workflow passes `--verbose` |
| R8 | Fix it everywhere | #254 footer | **Audited.** No other workflow or script creates, edits or reopens issues (`grep -rn "issues: write\|state: 'open'" .github scripts`) |
| R9 | Report upstream problems with a reproduction, a workaround and a suggested fix | #254 footer | hive-mind: see [Upstream](#upstream) |

The coverage itself (14.4% on `main`) is not something one PR can fix. The
milestone #246 and the feature issues #236 to #245 track it.

## Root causes

1. **The workflow reopened a closed report every week.** At `745c4345`,
   [`scripts/update-coverage-gap-issue.mjs` line 42](https://github.com/link-assistant/calculator/blob/745c4345b5f11512011125193e8c77ab6edc5a33/scripts/update-coverage-gap-issue.mjs#L42)
   was:

   ```js
   const payload = { title: ISSUE_TITLE, body, ...(existing ? { state: 'open' } : {}) };
   ```

   The test "updates and reopens the marked issue found on a later page"
   locked this in. #229 asked for one issue that the workflow "opens or
   updates", and the code read "opens" as "reopens". GitHub's
   [Update an issue](https://docs.github.com/en/rest/issues/issues#update-an-issue)
   endpoint changes the state only when `state` is sent, so dropping the field
   keeps whatever state a person or a PR chose.
2. **Repository mode treats every open issue as work.** At hive-mind
   `ed6a155` (v2.35.8), [`selectOldestOpenIssues`](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/solve.repository-mode.lib.mjs#L109-L131)
   filters only pull requests. It has an `exclude` option, but
   [`prepareRepositoryModeIssue`](https://github.com/link-assistant/hive-mind/blob/ed6a15520e49aafd49c06b49733ae8c28a15c4a6/src/solve.repository-mode.run.lib.mjs#L137-L141)
   never passes it. No label, author or marker excludes a status page.
   `buildCombinedIssueBody` then requires `Fixes #258`
   ([reproduction](data/hive-mind-dashboard-selection-repro.txt)).
3. **The two together make a loop.** A pull request closes #258, the next
   Monday's run reopens it, and the next repository-mode run selects it again.
   Neither part is wrong alone. A reopening dashboard suits a repository
   without automated solvers, and selecting all open issues suits a repository
   without dashboards.
4. **The report did not say it was a report.** The body ended with "This issue
   is updated weekly" but did not say where the work is. Nothing told a reader
   or an agent that #236 to #246 own the gaps.
5. **The report went stale between runs.** The workflow ran only weekly. #235
   moved the baseline about 4.5 hours after #258 was created. Until the next
   Monday the issue would show 13.3% for a `main` at 14.4%.

## Solution

| Change | File |
| --- | --- |
| Update the existing issue without `state`. Log whether it was open or closed | `scripts/update-coverage-gap-issue.mjs` (`upsertCoverageGapIssue`) |
| Read the sub-issues of #227 (`GET repos/{owner}/{repo}/issues/227/sub_issues`, paginated) and list the open ones under "Tracking issues". The section says the issue is a status page that is never reopened | `listTrackingIssues`, `renderIssueBody` |
| Parse the `**All**` row and fail after the update when gaps remain with no open tracking issue | `summarizeReport`, `untrackedGapMessage` |
| Write the body, without the marker, to `$GITHUB_STEP_SUMMARY` | `stepSummary` |
| `--verbose`, `--dry-run` | `parseArgs`, `dryRunGitHub` |
| Run on pushes to `main` that change the baseline; pass `--verbose` | `.github/workflows/coverage-gap-report.yml` |
| Tests for all of the above (12 tests) | `scripts/update-coverage-gap-issue.test.mjs` |
| Process documentation | [CONTRIBUTING.md](../../../CONTRIBUTING.md#competitor-coverage), [issue-227](../issue-227/README.md) |

Before and after, with a fake `gh` that returns a closed report
([`data/reopen-repro.txt`](data/reopen-repro.txt)):

```text
## before (745c4345)
PATCH repos/link-assistant/calculator/issues/258 state=open
REOPENS the closed report

## after
Updating #258 (closed); its state is kept, a closed report is not reopened.
PATCH repos/link-assistant/calculator/issues/258 state=(unchanged)
keeps the closed report closed
```

### Alternatives considered

| Option | Why not |
| --- | --- |
| Keep reopening, and have solvers skip the issue by label | No such filter exists in hive-mind (root cause 2). It would work only for one tool. Reported upstream as the long-term fix |
| Never create an issue: job summary, artifact or a committed file only | Breaks #229's criterion that the gap is visible without opening a run or a PR. The job summary is added as a second place, not a replacement |
| A GitHub Discussion instead of an issue | Discussions are not enabled in this repository, and the API is GraphQL-only. Solvers would not see it, but neither would people who follow the issue list |
| Reopen only when coverage drops | The Competitor Coverage CI job already fails a PR on any status regression. A drop on `main` would come from outside the code, such as rate- or time-dependent results, and a reopen would still bring the loop back |
| Close the report automatically after each update | It would hide the report from people who look at open issues, and fight anyone who reopens it on purpose |

### Existing components

Each row cites the primary documentation, read on 2026-10-13.

| Component | What it does | Fit for #258 |
| --- | --- | --- |
| [peter-evans/create-issue-from-file](https://github.com/peter-evans/create-issue-from-file) | Creates an issue from a file, or updates one given as `issue-number` ("The issue number of an existing issue to update"). Inputs: `title`, `content-filepath`, `labels`, `assignees` | It does not look up the issue, so a step would still have to find the marked issue. The README does not say what happens to the state of a closed issue. Adding an action dependency does not remove the lookup script |
| [JasonEtco/create-an-issue](https://github.com/JasonEtco/create-an-issue) | Creates an issue from a template. `update_existing: true` updates "an open issue with the **exact same title**". `search_existing` can search `open`, `closed` or `all` | Matches by title only, not by a marker. The README does not say whether a matched closed issue is reopened. Labels come from template front matter |
| [Renovate Dependency Dashboard](https://docs.renovatebot.com/key-concepts/dashboard/) | The best-known bot-maintained status issue. [`dependencyDashboardAutoclose`](https://docs.renovatebot.com/configuration-options/) (default `false`) closes it "if there are no more updates". `dependencyDashboardLabels` are restored "on each run" | Same pattern as the coverage report, and it is on by default in `config:recommended`. The docs do not describe reopening a dashboard that a person closed; only the Config Warning issue has a reuse/reopen option (`configWarningReuseIssue`, default `false`). Any repository with a dashboard has the same hive-mind exposure, which is why the upstream fix should not be specific to this repository |
| [REST: List sub-issues](https://docs.github.com/en/rest/issues/sub-issues) | `GET /repos/{owner}/{repo}/issues/{issue_number}/sub_issues`, `per_page` up to 100 | Used to list the tracking issues. The script passes `--paginate --slurp` |
| [REST: Update an issue](https://docs.github.com/en/rest/issues/issues#update-an-issue) | `PATCH /repos/{owner}/{repo}/issues/{issue_number}`. No body parameter is required; `state_reason` is "Ignored unless state is changed" | The page does not say outright that omitting `state` keeps it, but nothing is required and PATCH changes only the fields sent. The script now sends only `title` and `body`; [`data/reopen-repro.txt`](data/reopen-repro.txt) checks that the request has no `state` |

None of these tools decides whether a status issue should be reopened. That
depends on the repository, so the fix stays in the script, which already knew
the marker and the title.

## How to reproduce

```bash
# The weekly reopen, before and after the fix:
mkdir -p /tmp/old258
git show 745c4345:scripts/update-coverage-gap-issue.mjs > /tmp/old258/update-coverage-gap-issue.mjs
node experiments/issue-258/reopen-repro.mjs /tmp/old258/update-coverage-gap-issue.mjs
node experiments/issue-258/reopen-repro.mjs scripts/update-coverage-gap-issue.mjs

# hive-mind repository mode selecting the report:
git clone https://github.com/link-assistant/hive-mind /tmp/hive-mind
git -C /tmp/hive-mind checkout ed6a15520e49aafd49c06b49733ae8c28a15c4a6
node experiments/issue-258/hive-mind-dashboard-selection-repro.mjs /tmp/hive-mind

# The body the workflow would write now, without writing it:
node scripts/update-coverage-gap-issue.mjs --dry-run --verbose

node --test scripts/update-coverage-gap-issue.test.mjs
```

## Upstream

<!-- Filled after filing. -->
