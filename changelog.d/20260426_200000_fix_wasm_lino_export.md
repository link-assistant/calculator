---
bump: patch
---

### Fixed
- Export `load_rates_from_consolidated_lino` via WASM bindings so the web worker can load historical CBR rates from local `.lino` files. Previously this method was missing from the WASM binary, causing all dated RUB currency conversions (e.g., `22822 рублей в рупиях на 11 апреля 2026`) to silently fail with "No exchange rate available" even though the rate data existed locally.
