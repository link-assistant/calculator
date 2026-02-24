---
bump: minor
---

### Added
- Cryptocurrency price conversions via CoinGecko API (free tier, no API key required):
  - Expressions: `19 ton in usd`, `19 ton to usd`, `19 ton as usd`, `19 ton in dollars`
  - Natural language crypto names: `toncoin`, `bitcoin`, `ethereum`, `solana`, `dogecoin`, etc.
  - Supports TON, BTC, ETH, BNB, SOL, XRP, ADA, DOGE, DOT, LTC, LINK, UNI and more
  - `in` and `to` keywords for unit conversion (in addition to existing `as`)
- Mass/weight unit conversions: `10 tons to kg`, `1 kg as pounds`, `1000 g as kg`
  - Units: milligrams (mg), grams (g), kilograms (kg), metric tons/tonnes (t), pounds (lb), ounces (oz)
  - Full-name aliases: `grams`, `kilograms`, `tonnes`, `pounds`, `ounces`
  - Disambiguation: singular `ton` = TON cryptocurrency; plural `tons`/`tonnes` = metric mass unit
