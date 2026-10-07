---
bump: patch
---

### Added
- `scripts/competitor-coverage.mjs`: runs every expression of the competitor and scientific-quote corpora through the CLI, reports supported/different/unsupported/timeout per source and language, and fails on regressions against a committed baseline (#227).
- `scripts/import-competitor-corpus.mjs`: imports single-expression assertions from the fend, libqalculate, Numbat, Rink and math.js test suites into the corpus format.
- Case study `docs/case-studies/issue-227` with 3055 corpus rows, the measured coverage report (20.9% supported) and the 18 planned sub-issues with blockers.
