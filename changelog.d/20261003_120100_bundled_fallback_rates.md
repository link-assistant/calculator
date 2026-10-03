---
bump: minor
---

### Fixed
- `35usd- 200cny- 1hkd` failed with "Cannot convert HKD to USD: No exchange rate available" (#224). The latest rate of every `data/currency/*.lino` file is now bundled at build time, so HKD, AUD, CAD, SEK, KRW and the other ECB currencies convert without a live rate fetch; live rates still take precedence.
- Uppercase `SEK` is now the Swedish krona instead of German `sek` (seconds); lowercase `sek` stays seconds unless a currency conversion target is given.
- The CLI no longer loops forever at end of input, and accepts an expression as command-line arguments (`link-calculator "1 HKD in USD"`).
