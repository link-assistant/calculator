Closes #227

## Summary

This is the case study and work plan for parity with every calculator on natural-language math. The 18 sub-issues (#229 to #246) are filed with GitHub blocker dependencies.

The coverage checker, corpus importer and corpora started in this PR. Sub-issues #229 to #232 then grew them and merged them into `main` (#248 to #251). When resolving the merge conflicts, this branch therefore took `main`'s version of every shared file. It now adds only:

- `docs/case-studies/issue-227/README.md`, which covers:
  - the requirement inventory (R1 to R7) with how each one is resolved
  - the method, and the initial measurement of 3055 rows on v0.22.0
  - the families of silently wrong answers and the error classes
  - solution options per requirement, the component and library evaluation, the dependency graph, and the assumptions
  - a new **Progress since the plan was filed** section measured on v0.24.0
- `docs/case-studies/issue-227/sub-issues/`, the 18 sub-issue specifications exactly as filed, plus `issues.json`.
- `docs/case-studies/issue-227/issue.json`, the issue snapshot.
- `experiments/issue-227/create-sub-issues.mjs`, an idempotent script that creates the sub-issues, links them as sub-issues and registers the blockers.
- A changelog fragment.

## Requirement check (2026-10-12)

| Requirement from #227 | Where it is met |
| --- | --- |
| Quotes from scientific papers in different languages are computable | Corpus of 1356 rows in 23 languages (#232, merged). The feature work is in #236, #237, #238 and #245 (open). |
| All compute features of open-source competitors, based on their tests and code | 7432 rows imported from the test suites of 9 engines with pinned revisions (#230, merged). |
| All features of closed-source competitors, based on their documentation | 5187 attributed documentation rows from 9 products (#231, merged). |
| Automated scripts that show exactly what we have and what is missing | `scripts/competitor-coverage.mjs`, the `Competitor Coverage` CI gate and the weekly gap report (#229, merged). |
| All work planned as sub-issues, with blockers marked | 18 sub-issues. GraphQL `blockedBy` re-verified today: 37 edges, unchanged. |
| Data and case study in `docs/case-studies/issue-227`, online research, solution plans, existing libraries | `docs/case-studies/issue-227/` |

## Current state (v0.24.0, from the committed baseline)

| Corpus | Rows | Supported | Different | Unsupported |
| --- | ---: | ---: | ---: | ---: |
| Open-source test suites | 7432 | 1294 (17.4%) | 212 | 5926 |
| Closed-source documentation | 5187 | 654 (12.6%) | 25 | 4508 |
| Multilingual scientific statements | 1356 | 62 (4.6%) | 70 | 1224 |
| **All** | **13975** | **2010 (14.4%)** | **307** | **11658** |

On the original 3055 rows, 891 are now supported (640 on 2026-10-07), and 195 rows give a silently different answer (213 before). The examples from the original report, `3,000 minus 12`, `200 + 10%`, `10m × 10m` and `2 amperes times 5 ohms`, now give 2988, 220, 100 m² and 10 V. Five rows moved to a worse status. Each of those changes is deliberate and documented in the case study of #234 or #235.

## Sub-issues

| # | Title | Blocked by | State |
| --- | --- | --- | --- |
| #229 | Coverage checker in CI | none | closed by #248 |
| #230 | Open-source suites | none | closed by #249 |
| #231 | Closed-source documentation corpus | none | closed by #250 |
| #232 | Multilingual scientific-quote corpus | none | closed by #251 |
| #233 | Thousands separators and locale decimals | #229 | closed by #252 |
| #234 | Percent arithmetic | #229 | closed by #253 |
| #235 | Dimensional unit algebra | #229, #233 | closed by #259 |
| #236 | Scientific notation as written in papers | #233, #235 | open |
| #237 | English noun-phrase operators | #229 | open |
| #238 | Multilingual phrase operators, spelled numbers, CJK | #237, #233, #236 | open |
| #239 | Number bases, bitwise, display formats | #229 | open |
| #240 | Complex numbers, matrices, special values | #229, #239 | open |
| #241 | Symbolic algebra | #240 | open |
| #242 | Notepad state, booleans, strings | #229 | open |
| #243 | Date and time parity | #229 | open |
| #244 | Finance, statistics and rates | #234, #235, #237 | open |
| #245 | LaTeX, MathML and Unicode math input | #236, #242 | open |
| #246 | Milestone: every row supported or documented | #230 to #245 | open |

Merging this PR closes only #227. The open sub-issues #236 to #246 stay open, so this description deliberately has no `Fixes` lines for them.

## How to reproduce

```sh
cargo build --release --locked --bin link-calculator
node scripts/competitor-coverage.mjs \
  --baseline docs/case-studies/issue-227/results/coverage-baseline.json \
  --out ci-logs/competitor-coverage --quiet
node --test scripts/*.test.mjs
```

Results on this branch: `No regressions against docs/case-studies/issue-227/results/coverage-baseline.json`, 129 of 129 script tests pass, and `check-required-docs` and `check-file-size` pass.

## Notes

- No Rust code changes.
- Upstream test data is used as an executable reference under the quoted licenses and is not relicensed.
- The assumptions are listed at the end of the case-study README: how "beat exactly" is measured, rows that use live data only have to evaluate, and conflicting competitor conventions are settled in each sub-issue.


---

**Correction (2026-10-13, #247):** hive-mind's post-session link repair had appended `Fixes #229` to `Fixes #246` here. GitHub acts on those keywords, not on the sentence above, so merging it marked #236 to #246 as completed before they were implemented (their own PRs had already completed #229 to #235). The lines are removed and #236 to #246 are reopened. See [docs/case-studies/issue-247](https://github.com/link-assistant/calculator/blob/1b91a43a/docs/case-studies/issue-247/README.md) and PR #261.
