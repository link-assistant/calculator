# Case Study: Issue #104 — Ambiguous Unit "ton" (Mass vs Cryptocurrency)

## Overview

**Issue:** https://github.com/link-assistant/calculator/issues/104
**Status:** Root cause found and fixed
**Severity:** Medium — expressions using "ton" produced only one interpretation instead of surfacing the ambiguity

### Symptoms

1. Expression `19 ton` produced only `(19 TON)` — interpreted solely as Toncoin cryptocurrency
2. No alternative interpretation for metric ton (1000 kg mass unit) was shown
3. Users had no way to know the calculator chose one interpretation over another

---

## Timeline

| Date | Event |
|------|-------|
| 2026-03-21T23:40 | Issue #104 filed with expression `19 ton`, environment v0.8.0 |
| 2026-03-22 | Root cause analysis and fix implemented |

---

## Root Cause Analysis

### The Ambiguity

The word "ton" has two valid interpretations in this calculator:

1. **Metric ton (mass):** 1 ton = 1000 kg. The tonne (symbol: t) is a non-SI unit of mass accepted for use with SI, equal to 1000 kilograms. Commonly used in international trade for measuring large quantities of goods.
   - Source: [Wikipedia — Tonne](https://en.wikipedia.org/wiki/Tonne)

2. **Toncoin (cryptocurrency):** TON is the native cryptocurrency of The Open Network (TON), a decentralized layer-1 blockchain originally developed by Telegram founders. As of March 2026, TON trades at ~$1.26 with a market cap of ~$3.1B.
   - Source: [CoinMarketCap — Toncoin](https://coinmarketcap.com/currencies/toncoin/)
   - Source: [Wikipedia — TON (blockchain)](https://en.wikipedia.org/wiki/TON_(blockchain))

### Root Cause: Intentional Exclusion Without Alternative

In `src/types/unit.rs`, `MassUnit::parse()` intentionally excluded "ton" (singular) from matching:

```rust
// Note: "ton" (singular) is intentionally omitted here to avoid ambiguity with
// the TON cryptocurrency. Use "tons", "tonne", or "tonnes" for mass.
"t" | "tons" | "tonne" | "tonnes" | "metric_ton" | "metric_tons" => {
    Some(Self::MetricTon)
}
```

This caused `NumberGrammar::parse_unit("ton")` to skip the mass interpretation entirely and fall through to `CurrencyDatabase::parse_currency("ton")`, which matched "TON" as a 3-letter alphabetic currency code. The result was a single interpretation with no alternative shown.

### The Data Flow

```
Input: "19 ton"
  1. Lexer: [Number("19"), Identifier("ton"), Eof]
  2. TokenParser::parse_primary sees Number + Identifier
  3. Calls NumberGrammar::parse_unit("ton")
  4. MassUnit::parse("ton") → None (excluded)
  5. CurrencyDatabase::parse_currency("ton") → Some("TON")
  6. Result: Expression::Number { value: 19, unit: Currency("TON") }
  7. No alternative interpretations generated
```

---

## Solution

### Approach: Unit Ambiguity Detection and Contextual Resolution

Rather than choosing one interpretation over the other, the fix introduces a system that:

1. **Detects ambiguity** — when a unit identifier matches both a mass unit and a well-known cryptocurrency
2. **Surfaces alternatives** — shows both interpretations to the user via the existing `alternative_lino` mechanism
3. **Resolves contextually** — when a conversion target provides context (e.g., "19 ton in usd" → crypto, "19 ton in kg" → mass)

### Changes

1. **`MassUnit::parse`** — Re-added "ton" (singular) as a valid mass unit match
2. **`Expression::Number`** — Added `alternative_units: Vec<Unit>` field to carry ambiguous interpretations
3. **`NumberGrammar::parse_unit_with_alternatives`** — New method that returns both primary unit and alternatives when ambiguity is detected; only flags well-known currencies (via `crypto_api::coingecko_id`) to avoid false positives
4. **`Expression::collect_alternatives`** — Extended to generate alternative LINO strings for expressions with ambiguous units
5. **`TokenParser::resolve_unit_ambiguity_for_conversion`** — When a conversion keyword ("in"/"to"/"as") provides context, swaps the primary unit to the compatible alternative
6. **`Unit::is_same_category`** — New helper to check if two units belong to the same category (both currencies, both mass, etc.)

### Behavior After Fix

| Expression | Primary | Alternatives |
|-----------|---------|-------------|
| `19 ton` | `(19 t)` (mass) | `(19 TON)` (crypto) |
| `19 TON` | `(19 TON)` (crypto) | `(19 t)` (mass) |
| `19 ton in usd` | `((19 TON) as USD)` | none (context resolved) |
| `19 ton in kg` | `((19 t) as kg)` | none (context resolved) |
| `19 tons` | `(19 t)` (mass) | none (unambiguous) |
| `19 tonne` | `(19 t)` (mass) | none (unambiguous) |

---

## References

- [Toncoin (TON) — CoinMarketCap](https://coinmarketcap.com/currencies/toncoin/)
- [TON (blockchain) — Wikipedia](https://en.wikipedia.org/wiki/TON_(blockchain))
- [Tonne — Wikipedia](https://en.wikipedia.org/wiki/Tonne)
- [Ton — Wikipedia](https://en.wikipedia.org/wiki/Ton)
- [CoinGecko — Toncoin API](https://www.coingecko.com/en/coins/toncoin)
- [The Open Network — ton.org](https://ton.org/en/toncoin)
