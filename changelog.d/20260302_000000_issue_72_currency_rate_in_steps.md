---
bump: patch
---

### Fixed
- Currency conversion steps now always show the exchange rate source, effective date, and exact rate value for `as`/`in`/`to` unit conversion expressions (e.g. `1 ETH in EUR`, `100 USD as EUR`)
- Previously, the rate metadata was only shown in steps for arithmetic currency expressions (e.g. `0 RUB + 1 USD`), but was silently omitted for direct unit-conversion syntax
- Both fiat-to-fiat (USD→EUR) and crypto-to-fiat (ETH→USD, ETH→EUR via cross-rate) conversions are now covered uniformly
