---
bump: minor
---

### Added
- Exchange rate transparency: show source, date, and fetch time for currency conversions
- Real-time exchange rate fetching from official Central Bank APIs (ECB via frankfurter.dev, CBR via cbr.ru)
- WASM bindings for `fetch_exchange_rates` and `fetch_historical_rates` functions
- Exchange rate loading indicator in the web UI
- E2E tests for currency conversion with real rates

### Changed
- Currency calculations now display exchange rate info in calculation steps
- CurrencyDatabase now tracks the last used rate information for transparency
