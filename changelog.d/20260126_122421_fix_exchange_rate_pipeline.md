---
bump: patch
---

### Fixed
- Exchange rates fetched from API are now actually applied to the Calculator (fixes issue #18)
- Worker now calls `update_rates_from_api` method after fetching rates, replacing hardcoded fallback rates
- Removed suspicious hardcoded 89.5 USD/RUB rate from data/currency/usd-rub.lino

### Added
- New WASM method `update_rates_from_api(base, date, rates_json)` on Calculator class
- Integration tests for API rate updates
