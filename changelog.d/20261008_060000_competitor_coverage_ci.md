---
bump: patch
---

### Added
- Gate CI on competitor expression coverage and upload the regenerated report, with a weekly coverage gap issue containing source and language summaries.

### Fixed
- Read coverage baselines before regenerating them, and detect worsening statuses and removed corpus expressions.

### Changed
- Refresh web dependencies and Rust lockfile entries to satisfy the existing dependency freshness CI gate.
