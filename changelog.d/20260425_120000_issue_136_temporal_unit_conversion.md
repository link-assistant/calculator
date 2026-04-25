---
bump: patch
---

### Fixed
- Temporal modifier (`at <date>`) is now correctly applied to unit conversion expressions (e.g. `22822 RUB as INR at Apr 11, 2026` now uses the historical exchange rate for that date instead of the current rate). Fixes #136.
- Russian preposition "на" (meaning "at/on" for dates) is now recognized as the temporal `at` keyword, enabling expressions like `22822 рублей в рупиях на 11 апреля 2026` to use historical rates correctly.
