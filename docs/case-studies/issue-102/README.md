# Case Study: Issue #102 — Redundant Exchange Rate Source in Calculation Plan

## Summary

The calculation plan for expressions involving RUB, crypto, and USD currencies
unnecessarily fetches ECB (European Central Bank) rates when only CBR (Central
Bank of Russia) and CoinGecko crypto rates are needed.

## Reproduction

**Expression:**
```
(1000 рублей + 500 рублей + 2000 рублей + 20 TON + 1000 рублей) в USD
```

**Parsed as:**
```
((((((1000 RUB) + (500 RUB)) + (2000 RUB)) + (20 TON)) + (1000 RUB)) as USD)
```

**Currencies detected:** `{RUB, TON, USD}`

**Observed rate sources fetched:** ECB, CBR, Crypto (3 sources)
**Expected rate sources fetched:** CBR, Crypto (2 sources)

**Extra rate fetch:** ECB (Frankfurter API) — unnecessary network request.

## Timeline / Sequence of Events

1. User enters expression with RUB amounts, a TON crypto amount, and requests
   USD conversion.
2. `Calculator::plan()` parses the AST and calls `collect_currencies()`,
   finding `{RUB, TON, USD}`.
3. `currency_to_sources()` maps each currency:
   - RUB → `Cbr`
   - TON → `Crypto`
   - USD → `Ecb`
4. After mapping, `sources_set = {Cbr, Crypto, Ecb}`.
5. The plan then has an **additional heuristic** (plan.rs lines 97–100):
   ```rust
   if currencies.len() >= 2 && !sources_set.is_empty() {
       sources_set.insert(RateSource::Ecb);
   }
   ```
   This adds ECB unconditionally for any multi-currency expression,
   even when ECB is already present or not needed at all.
6. Result: 3 sources are requested when 2 would suffice.

## Root Cause Analysis

There are **two issues** contributing to the redundancy:

### Issue 1: Unconditional ECB injection (plan.rs:97–100)

The heuristic `if currencies.len() >= 2 { add ECB }` was intended to ensure
ECB rates are available for "triangulation" (converting between currencies
via USD as a bridge). However, this is unnecessary because:

- CBR already provides USD↔RUB rates (covers all RUB-involving conversions)
- CoinGecko crypto rates are denominated in USD (covers crypto→USD directly)
- If a currency genuinely needs ECB (like EUR, GBP, JPY), it's already mapped
  to ECB by `currency_to_sources()`

The per-currency source mapping already produces the correct set of sources
without this heuristic.

### Issue 2: USD always maps to ECB (plan.rs:73)

`currency_to_sources("USD")` returns `[Ecb]`. While USD is technically an
ECB-provided currency, in practice:

- When CBR is already required (because RUB is present), CBR provides
  USD↔RUB rates — no ECB needed for the USD↔RUB conversion path.
- When Crypto is already required, crypto rates are in USD — no ECB needed
  for the crypto↔USD conversion path.

USD as a **target** currency (via `as USD` / `in USD`) only needs a source
that can provide X→USD rates. If CBR or Crypto already provide those rates
for the currencies in the expression, ECB is redundant.

## Actual Conversion Path (Runtime)

During evaluation of `(1000 RUB + 500 RUB + 2000 RUB + 20 TON + 1000 RUB) as USD`:

| Step | Operation | Rate Source Used |
|------|-----------|-----------------|
| 1 | 1000 RUB + 500 RUB = 1500 RUB | None (same currency) |
| 2 | 1500 RUB + 2000 RUB = 3500 RUB | None (same currency) |
| 3 | 3500 RUB + 20 TON | Triangulate: TON→USD (CoinGecko) + USD→RUB (CBR) |
| 4 | 5616.75 RUB + 1000 RUB = 6616.75 RUB | None (same currency) |
| 5 | 6616.75 RUB → USD | RUB→USD (CBR inverse) |

**ECB rates are never used at runtime** for this expression. The plan fetches
them unnecessarily.

## Solution

1. **Remove the unconditional ECB injection** (plan.rs:97–100). The
   per-currency mapping in `currency_to_sources()` is already sufficient.

2. **Make USD source context-aware**: When CBR is already required (RUB is
   in the expression), USD doesn't need ECB because CBR provides USD↔RUB
   rates. Similarly, when only Crypto sources are needed alongside USD,
   the crypto rates (denominated in USD) already cover the conversion path.

### Impact

- Fewer unnecessary network requests (1 fewer API call for RUB+crypto+USD
  expressions)
- Faster plan execution for expressions that don't need ECB
- No change to calculation results — only the plan's rate source list changes

## Related Components

- `src/plan.rs` — Plan generation with source mapping
- `src/types/currency.rs` — `CurrencyDatabase::convert()` with USD
  triangulation
- `src/crypto_api.rs` — CoinGecko rate fetching (USD-denominated)
- `web/src/worker-rate-coordination.ts` — Rate source orchestration
