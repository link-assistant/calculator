---
bump: minor
---

### Added
- Support for currency symbol prefix notation: `$10`, `€5`, `£3`, `₽100`, `₹10` are now parsed as `10 USD`, `5 EUR`, `3 GBP`, `100 RUB`, `10 INR` respectively (fixes #51)
- Russian language currency name support: grammatical case forms of рубль (→ RUB) and рупия (→ INR) (fixes #52)
- INR (Indian Rupee) to the default currency database with USD→INR exchange rate (86.5) (fixes #53)
- USD triangulation for cross-currency conversions (e.g. RUB↔INR via USD bridge) when no direct rate exists (fixes #53)
