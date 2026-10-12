---
title: "Run the competitor coverage checker in CI and fail on regressions"
---
Part of #227. Measured by `scripts/competitor-coverage.mjs` (added in #228).

## Problem

PR #228 added three expression corpora (3055 rows) and a runner that classifies each row as `supported`, `different`, `unsupported` or `timeout`. Nothing executes it in CI yet, so a change that breaks one of the 640 currently supported competitor expressions would go unnoticed, and the per-source coverage numbers in `docs/case-studies/issue-227/results/coverage-report.md` go stale silently.

## Acceptance criteria

- A CI job builds the release binary and runs `node scripts/competitor-coverage.mjs --baseline docs/case-studies/issue-227/results/coverage-baseline.json --quiet`; any row whose status got worse than the baseline fails the job.
- The regenerated `coverage-report.md` is uploaded as a workflow artifact so reviewers can inspect `different` rows without a local build.
- A scheduled (weekly) workflow regenerates the report and opens or updates a single "coverage gap report" issue with the per-source and per-language tables, so the gap is visible without a PR.
- The baseline is updated in the same PR as any engine change that improves coverage (document this in `CONTRIBUTING.md`).
- The job has a timeout of at most 15 minutes; the full corpus takes about 24 seconds locally.

## Solution options

1. Add a `coverage` job to `.github/workflows/release.yml` after the `test` job, reusing the cargo cache. Simplest, recommended.
2. A separate `coverage.yml` workflow with both `pull_request` and `schedule` triggers. Cleaner separation, but duplicates the Rust setup steps.

## Blockers

None. This is the gate every engine sub-issue of #227 reports against.
