---
bump: patch
---

### Fixed

- Fixed auto-release pipeline never committing version bumps to git (issue #78)
  - Root cause: `command-stream`'s `$` has `errexit: false` by default, so `git diff --cached --quiet` never threw an exception even when there were staged changes
  - Fix: replaced `try/catch` approach with explicit `diffResult.code === 0` check in `scripts/version-and-commit.mjs`
- Cleaned up accumulated changelog fragments that were never committed to git
- Updated `CHANGELOG.md` with all accumulated changes
- Added case study analysis in `docs/case-studies/issue-78/`
