---
bump: patch
---

### Fixed
- Fixed currency rates CI/CD pipeline: migrated Frankfurter API to `api.frankfurter.dev/v1`, set proper User-Agent header to avoid CDN 403 blocks, added error detection for silent failures
- Upgraded GitHub Actions from Node.js 20 to Node.js 24 (`actions/checkout@v6`, `actions/setup-python@v6`)

### Changed
- Currency rates update schedule changed from weekly to daily (weekdays at 17:00 UTC)
