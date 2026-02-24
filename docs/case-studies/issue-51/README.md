# Case Study: Issue #51 — Unrecognized input: `10 рублей + $10 + 10 рупий`

## Problem Statement

**Input:** `10 рублей + $10 + 10 рупий`
**Error:** `Parse error: Unexpected character '$' at position 12`
**Expected:** Should be treated as `10 RUB + 10 USD + 10 INR`

---

## Timeline and Sequence of Events

1. User enters expression with a mix of Russian currency names and a `$` currency symbol prefix.
2. The Lexer (`src/grammar/lexer.rs`) encounters `$` and fails with "Unexpected character" because `$` is not in the set of handled characters.
3. The expression never reaches the parser.

---

## Root Cause Analysis

### 1. Lexer Does Not Support Currency Symbol Characters

File: `src/grammar/lexer.rs`, lines 172–178

```rust
_ if ch.is_ascii_digit() || ch == '.' => self.scan_number(),
_ if ch.is_alphabetic() => self.scan_identifier(),
_ => {
    return Err(CalculatorError::parse(format!(
        "Unexpected character '{ch}' at position {start}"
    )));
}
```

The `$` character (U+0024, DOLLAR SIGN) is neither alphabetic nor a digit, so it falls through to the error case. Other currency symbols like `€` (U+20AC), `£` (U+00A3), `¥` (U+00A5) would fail similarly.

### 2. Currency Symbols as Prefix Notation

In standard international usage, the `$` symbol appears *before* the number (e.g., `$10`, meaning "ten US dollars"). This is fundamentally different from the postfix unit notation used elsewhere in the calculator (e.g., `10 USD`).

Key currency prefix symbols:
- `$` → USD (US Dollar)
- `€` → EUR (Euro)
- `£` → GBP (British Pound)
- `¥` → JPY (Japanese Yen) / CNY (Chinese Yuan)
- `₽` → RUB (Russian Ruble)
- `₹` → INR (Indian Rupee)
- `₩` → KRW (Korean Won)

---

## Related Facts

- ISO 4217 standard defines 3-letter codes for all currencies.
- The Unicode character `$` is defined as "DOLLAR SIGN" and has been used since the late 18th century.
- Several currencies use `$` including USD, CAD, AUD, NZD, SGD, HKD.
- When `$` is used without qualification, it conventionally means US Dollar in most international contexts.
- The `CurrencyDatabase::parse_currency()` function (in `src/types/currency.rs`) already handles `"$"` as an alias for `"USD"` — but the lexer never gets a chance to emit it as a token.

Source: [Dollar sign - Wikipedia](https://en.wikipedia.org/wiki/Dollar_sign)

---

## Proposed Solution

### Option 1: Scan Currency Symbols as Identifiers (Recommended)

Modify the lexer to recognize known Unicode currency symbol characters (`$`, `€`, `£`, `¥`, `₽`, `₹`, `₩`) as single-character identifiers. The existing `CurrencyDatabase::parse_currency()` already maps these to ISO codes.

The change in the lexer's `next_token()` method:
```rust
_ if ch.is_alphabetic() || is_currency_symbol(ch) => self.scan_identifier_or_currency_symbol(),
```

For prefix notation like `$10`, the token parser would need to handle `<currency-symbol> <number>` as `<number> <currency>`.

### Option 2: Pre-process Input

Normalize input before lexing, replacing `$10` → `10 USD`, `€5` → `5 EUR`, etc. Simpler but less transparent.

### Option 3: Use a Preprocessing Step in the Lexer

Scan currency symbols as special prefix tokens and flip the order when building the AST.

---

## Chosen Fix

We extend the lexer to treat common currency symbol characters as identifier characters, and update the token parser to handle prefix currency notation (e.g., `$10` → value `10` with unit `USD`).

---

## References

- [Dollar sign - Wikipedia](https://en.wikipedia.org/wiki/Dollar_sign)
- [ISO 4217 - Wikipedia](https://en.wikipedia.org/wiki/ISO_4217)
- [iso4217parse Python library](https://github.com/tammoippen/iso4217parse)
- [price-parser - extracting price and currency from text](https://github.com/scrapinghub/price-parser)
- `src/grammar/lexer.rs` — Lexer implementation
- `src/types/currency.rs:430` — `CurrencyDatabase::parse_currency()`
