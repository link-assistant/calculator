---
bump: minor
---

### Changed
- Replaced unofficial fawazahmed0/currency-api with official Central Bank APIs:
  - European Central Bank (ECB) via frankfurter.dev for most currencies
  - Central Bank of Russia (CBR) via cbr.ru for RUB rates
- Updated currency rate source attribution throughout the codebase to reflect official sources

### Added
- GitHub Actions workflow for weekly automated currency rate updates from Central Banks
- Manual trigger support for on-demand rate updates

### Fixed
- EUR-CNY rate now sourced from ECB instead of unofficial API
