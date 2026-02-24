# Case Study: Issue #52 — Unrecognized input: `10 рублей + 10 USD + 10 рупий`

## Problem Statement

**Input:** `10 рублей + 10 USD + 10 рупий`
**Error:** `Unit mismatch: cannot add 'рублей' and 'USD'`
**Expected:** Should be treated as `10 RUB + 10 USD + 10 INR`

---

## Timeline and Sequence of Events

1. User enters expression with Russian-language currency names mixed with ISO currency codes.
2. The Lexer successfully tokenizes `рублей` and `рупий` as `Identifier` tokens because `is_alphabetic()` returns `true` for Cyrillic characters (Unicode alphabetic).
3. The parser sends them to `NumberGrammar::parse_unit()`.
4. `CurrencyDatabase::parse_currency()` does not recognize `рублей` or `рупий`.
5. They are stored as `Unit::Custom("рублей")` and `Unit::Custom("рупий")` respectively.
6. When the expression attempts `10 рублей + 10 USD`, the value system raises a unit mismatch error because `Custom("рублей") != Currency("USD")`.

---

## Root Cause Analysis

### 1. Missing Russian Currency Name Aliases

File: `src/types/currency.rs`, lines 444–471

The `parse_currency()` function handles English names:
```rust
"ruble" | "rubles" | "rouble" | "roubles" => return Some("RUB".to_string()),
```

But does NOT handle Russian-language names:
- `рубль` — nominative singular ("ruble" in Russian)
- `рубля` — genitive singular (after 2–4)
- `рублей` — genitive plural (after 5+, 11–14, 0)
- `рублём` — instrumental singular
- `рублю` — dative singular

Similarly for Indian Rupee in Russian:
- `рупия` — nominative singular
- `рупии` — genitive singular / plural
- `рупий` — genitive plural

### 2. Russian Grammatical Cases (Declension)

Russian is a highly inflected language with 6 grammatical cases. The word "рубль" changes form based on the number:
- 1 рубль (nominative)
- 2–4 рубля (genitive singular)
- 5–10+ рублей (genitive plural)
- 11–14 рублей (genitive plural, irregular)

This means a calculator dealing with Russian-language input must recognize multiple forms of the same currency word.

Source: [Russian ruble - Wikipedia](https://en.wikipedia.org/wiki/Russian_ruble)

### 3. Indian Rupee Russian Names

The word "рупия" (rupiya) is the Russian transliteration of the Indian Rupee (from Sanskrit रुपया "rūpya"):
- `рупия` — nominative singular
- `рупии` — genitive singular / nominative plural (2–4 rupees)
- `рупий` — genitive plural (5+ rupees)

Source: [Rupee - Wikipedia](https://en.wikipedia.org/wiki/Rupee)

---

## Related Facts

- The ISO 4217 code for the Russian Ruble is RUB (previously RUR before 1998 redenomination).
- The ISO 4217 code for the Indian Rupee is INR.
- The ruble symbol is ₽ (U+20BD, RUBLE SIGN, introduced to Unicode in 2014).
- The rupee symbol is ₹ (U+20B9, INDIAN RUPEE SIGN, introduced 2010).

---

## Proposed Solution

Extend `CurrencyDatabase::parse_currency()` to recognize Russian-language currency name variants:

```rust
// Russian language names for RUB
"рубль" | "рубля" | "рублей" | "рублю" | "рублём" | "рублем" => return Some("RUB".to_string()),
// Russian language names for INR
"рупия" | "рупии" | "рупий" | "рупию" | "рупией" => return Some("INR".to_string()),
```

The comparison should be case-insensitive for Cyrillic characters using `.to_lowercase()`.

---

## References

- [Russian ruble - Wikipedia](https://en.wikipedia.org/wiki/Russian_ruble)
- [Rupee - Wikipedia](https://en.wikipedia.org/wiki/Rupee)
- [Indian rupee - Wikipedia](https://en.wikipedia.org/wiki/Indian_rupee)
- `src/types/currency.rs:430` — `CurrencyDatabase::parse_currency()`
- `src/grammar/number_grammar.rs:50` — `NumberGrammar::parse_unit()`
