---
bump: patch
---

### Fixed
- CI no longer fails pushes to `main` because an upstream dependency released after the pull request merged; dependency freshness now blocks pull requests only and warns on push. Updated the stale dependencies and the audit findings.
- The Playwright end-to-end suite runs in CI again, with its eight broken tests fixed, and the web lint script runs `oxlint` instead of a missing ESLint setup.
- A failed job can no longer hide behind skipped dependants: every workflow ends with a pipeline-status gate that also fails on jobs killed by their timeout.
- Corpus refresh pull requests opened without `CORPUS_REFRESH_TOKEN` now get CI through a dispatched `checks` run instead of an approval-pending run that never started, and changes to `.github/actions/` trigger the jobs that use them.
- Workflows are hardened (read-only default permissions, pinned runners and third-party actions, no persisted checkout credentials, no template injection) and linted by actionlint and zizmor; a new security workflow runs cargo audit, npm audit, CodeQL and dependency review.
- `cargo package` no longer warns about experiment examples that are not part of the published crate.
- Markdown and HTML links are checked by lychee, with transient failures re-checked and Web Archive replacements suggested, and checkouts no longer print git's default-branch hint.
- Pull requests are also checked for committed credentials (secretlint) and re-tested on a fresh merge with the current `main`, so a semantic merge conflict fails the pull request instead of `main`.
- Releases prove the crates.io token can publish before pushing the version bump to `main`, so a revoked token can no longer leave `main` bumped on a version crates.io never received.
- Required documents and the sections other files link to (for example `CONTRIBUTING.md#competitor-coverage`) are checked on every pull request.
- Pull requests that change only Python or shell scripts or `web/package.json` now count as code changes, so they need a changelog fragment and run lint, build and the web tests; the CI Python scripts are linted with ruff.
