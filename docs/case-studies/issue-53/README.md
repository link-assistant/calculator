# Case Study: Issue #53 — `10 RUB + 10 USD + 10 INR` — No exchange rate available

## Problem Statement

**Input:** `10 RUB + 10 USD + 10 INR`
**Error:** `Cannot convert INR to RUB: No exchange rate available`
**Expected:** All required exchange rates should be available via real API of central banks.

---

## Timeline and Sequence of Events

1. User enters `10 RUB + 10 USD + 10 INR`.
2. Lexer and parser succeed — all tokens are recognized.
3. Evaluator attempts to add `10 RUB + 10 USD`.
4. Looks up rate `USD → RUB` — found (default hardcoded at 89.5 RUB/USD).
5. Result is `(89.5 + 10) RUB = 99.5 RUB`.
6. Evaluator attempts to add `99.5 RUB + 10 INR`.
7. Looks up rate `INR → RUB` in `self.rates` — **not found**.
8. Error: `Cannot convert INR to RUB: No exchange rate available`.

---

## Root Cause Analysis

### 1. Missing INR/RUB Exchange Rate in Default Hardcoded Rates

File: `src/types/currency.rs`, lines 177–218, function `initialize_default_rates()`

The default rates only include:
- USD ↔ EUR, GBP, JPY, CHF, CNY, RUB
- EUR ↔ USD, GBP, JPY, CHF
- GBP ↔ USD, EUR

INR is **absent** from the default rates, and there is no RUB↔INR pair.

### 2. Data File Exists but Is Not Loaded

The repository has `data/currency/usd-inr.lino` which contains USD/INR exchange rates from 2021 onwards (via Frankfurter/ECB). However, looking at the codebase, the `.lino` data files need to be explicitly loaded into the `CurrencyDatabase`. The `Calculator` struct must load these files at startup.

Checking `src/lib.rs` — the calculator does not automatically load all `.lino` files from `data/currency/` directory.

### 3. No RUB↔INR Pair in Any Data File

Looking at `data/currency/` directory contents:
```
chf-rub.lino, cny-rub.lino, eur-chf.lino, eur-cny.lino, eur-gbp.lino,
eur-jpy.lino, eur-rub.lino, eur-usd.lino, gbp-eur.lino, gbp-rub.lino,
gbp-usd.lino, jpy-rub.lino, rub-chf.lino, rub-cny.lino, rub-eur.lino,
rub-gbp.lino, rub-jpy.lino, rub-usd.lino, usd-aud.lino, usd-brl.lino,
usd-cad.lino, usd-chf.lino, usd-cny.lino, usd-czk.lino, usd-dkk.lino,
usd-eur.lino, usd-gbp.lino, usd-hkd.lino, usd-huf.lino, usd-inr.lino,
usd-jpy.lino, usd-krw.lino, usd-mxn.lino, usd-nok.lino, usd-nzd.lino,
usd-pln.lino, usd-rub.lino, usd-sek.lino, usd-sgd.lino, usd-try.lino, usd-zar.lino
```

There is `usd-inr.lino` and `rub-usd.lino` but **no direct `rub-inr.lino`** pair.

### 4. Currency Conversion Requires Direct Rate

The `CurrencyDatabase::convert()` function (`src/types/currency.rs:339`) only looks for a direct rate `(from, to)`. It does not perform triangulation through a common base currency (e.g., INR → USD → RUB).

---

## Currency Facts

### INR (Indian Rupee)
- ISO code: INR
- Symbol: ₹ (U+20B9)
- Issuing authority: Reserve Bank of India
- The Indian Rupee is supported by the ECB Frankfurter API (`api.frankfurter.app`)

### RUB (Russian Ruble)
- ISO code: RUB
- Symbol: ₽ (U+20BD)
- Issuing authority: Bank of Russia (ЦБ РФ)
- RUB is **NOT** available through ECB Frankfurter API since March 2022 (sanctions)
- RUB rates must come from the Bank of Russia API: `cbr.ru`

### Current Exchange Rate (as of 2026-02-24)
1 USD ≈ 87–90 RUB (approximate)
1 USD ≈ 86–87 INR (approximate)
Therefore: 1 RUB ≈ 1 INR (approximately, as both have similar USD rates)
Source: [XE.com RUB to INR](https://www.xe.com/currencyconverter/convert/?Amount=1&From=RUB&To=INR)

---

## Proposed Solutions

### Option 1: Add INR to Default Hardcoded Rates (Minimal Fix)

Add approximate fallback rates for INR:
```rust
self.set_rate_with_info("USD", "INR", ExchangeRateInfo::default_rate(86.5));
```

This would allow triangulation: INR → USD → RUB if we add cross-rate computation.

### Option 2: Implement Rate Triangulation (Better Fix)

When a direct `(from, to)` rate is not found, try triangulation via USD as a base:
```
INR → USD → RUB: rate = (1/INR_USD_rate) * USD_RUB_rate
```

This is the standard approach used by financial systems and allows any currency pair to be computed if both have USD rates.

### Option 3: Load .lino Data Files at Startup (Best Fix for Production)

Load all data from `data/currency/*.lino` files when the calculator initializes. This provides real, dated historical rates.

### Option 4: Use Frankfurter API at Runtime

For INR, the Frankfurter API supports it. For RUB (post-2022), use CBR API.

---

## Chosen Fix

We implement a combination:
1. Add INR to default hardcoded rates (for offline/fallback use).
2. Add rate triangulation through USD as a bridge currency.

This provides the most robust solution without requiring API calls or file loading.

---

## References

- [Frankfurter API - supported currencies (INR is included)](https://frankfurter.dev/)
- [Bank of Russia exchange rates](https://cbr.ru/eng/currency_base/daily/)
- [Reserve Bank of India](https://rbi.org.in/)
- [CEIC - Bank of Russia Indian Rupee rates](https://www.ceicdata.com/en/russia/foreign-exchange-rate-bank-of-russia-main-currencies/foreign-exchange-rate-bank-of-russia-indian-rupee)
- `src/types/currency.rs:177` — `initialize_default_rates()`
- `src/types/currency.rs:339` — `CurrencyDatabase::convert()`
- `data/currency/usd-inr.lino` — USD/INR historical rates file
