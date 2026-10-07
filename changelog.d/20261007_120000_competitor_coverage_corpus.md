---
bump: patch
---

### Added
- `scripts/competitor-coverage.mjs`: runs every expression of the competitor and scientific-quote corpora through the CLI, reports supported/different/unsupported/timeout per source and language, and fails on regressions against a committed baseline (#227).
- `scripts/import-competitor-corpus.mjs`: imports single-expression assertions from the fend, libqalculate, Numbat, Rink and math.js test suites into the corpus format.
- Case study `docs/case-studies/issue-227` with 3055 corpus rows, the measured coverage report (20.9% supported) and the 18 planned sub-issues with blockers.

### Changed

- Update web dependencies to their latest releases (vite 8.3.3, @vitejs/plugin-react 6.1.2, jsdom 30.1.2, react-i18next 17.0.16, links-notation 0.23.0, lino-objects-codec 0.9.0, browser-commander 0.26.2) so the dependency freshness check passes.
