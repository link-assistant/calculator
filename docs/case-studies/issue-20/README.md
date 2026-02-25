# Case Study: Issue #20 — Unrecognized input: `2 UF + 1 USD`

## Problem Statement

**Input:** `2 UF + 1 USD`
**Error (original):** `Unit mismatch: cannot add 'UF' and 'USD'`
**Error (after partial fix):** `Cannot convert USD to UF: No exchange rate available`
**Expected:** The calculator should treat `UF` as a currency (Unidad de Fomento, ISO 4217 code `CLF`) and convert between it and USD using exchange rates.

---

## Timeline and Sequence of Events

1. User enters `2 UF + 1 USD`.
2. The token parser calls `CurrencyDatabase::parse_currency("UF")`.
3. **Original behavior:** `UF` is only 2 characters and did not match the generic 3-letter code or named rules — it was parsed as `Unit::Custom("UF")` (a dimensionless custom unit).
4. Adding `Unit::Custom("UF")` to `Unit::Currency("USD")` fails with `UnitMismatch`.
5. **After the 2-letter generic code was added:** `UF` is parsed as `Unit::Currency("UF")`.
6. Two `Currency` units can be added, but the `CurrencyDatabase` is queried for a USD→UF exchange rate.
7. No rate exists in the database → `CurrencyConversion` error: "No exchange rate available".

---

## Root Cause Analysis

### 1. UF is not in the default currency database

The `initialize_default_currencies()` function in `src/types/currency.rs` (lines 166-181) only registers 8 currencies: USD, EUR, GBP, JPY, CHF, CNY, RUB, INR.

The Unidad de Fomento (ISO 4217: CLF) was not among them.

### 2. No exchange rate for CLF/UF in the default rates

The `initialize_default_rates()` function (lines 183-226) sets up rates only for the 8 registered currencies. Since CLF was absent, no rate exists to convert between CLF and any other currency.

### 3. The generic 2-letter catch-all might parse UF as a currency but exchange rates are still missing

Since the code at lines 525-529 of `src/types/currency.rs` accepts any 2-5 letter all-alphabetic input as a currency code:
```rust
if input.len() >= 2 && input.len() <= 5 && input.chars().all(|c| c.is_ascii_alphabetic()) {
    return Some(input);
}
```
...UF would be recognized as `Unit::Currency("UF")`, but there's no exchange rate registered for `"UF"`.

---

## Key Facts About UF (Unidad de Fomento)

- **Full name:** Unidad de Fomento (Unit of Development)
- **ISO 4217 Code:** CLF (Chilean Unidad de Fomento, funds code)
- **Symbol:** UF (user-visible name, not an ISO 4217 symbol)
- **Country:** Chile
- **Type:** Inflation-indexed unit of account (not legal tender, but widely used in contracts, mortgages, savings)
- **Decimals:** 4 (precision is important due to inflation indexing)
- **Exchange rate (February 2026):** 1 CLF ≈ 37–46 USD; 1 USD ≈ 0.022–0.027 CLF

Sources:
- [CLF to USD - fx-rate.net](https://fx-rate.net/CLF/USD/)
- [ISO 4217 - Wikipedia](https://en.wikipedia.org/wiki/ISO_4217)
- [Unidad de Fomento - Wikipedia](https://en.wikipedia.org/wiki/Unidad_de_Fomento)

---

## Proposed Solution

### Option 1: Add CLF to default currencies with a hardcoded default rate (Recommended)

This follows the same pattern used for INR (issue #53):
1. Add `Currency::clf()` factory method in the `Currency` struct.
2. Add CLF to `initialize_default_currencies()`.
3. Add a default USD→CLF rate in `initialize_default_rates()`.
4. Add explicit recognition of "UF", "CLF", "unidad de fomento" in `parse_currency()`.
5. Create `data/currency/usd-clf.lino` with historical rates.

### Option 2: Rely on the generic catch-all + add a rate file only

Simpler — just add `data/currency/usd-clf.lino`. But this doesn't help when no lino files are loaded (e.g., in tests), and doesn't provide proper currency metadata (name, symbol, decimals).

---

## Chosen Fix

**Option 1** — Add CLF as a first-class supported currency with a default rate and dedicated parsing, following the same pattern as INR (issue #53).

The internal code uses `CLF` (ISO 4217), but user input `UF` is recognized as an alias for `CLF`.

---

## References

- [Unidad de Fomento - Wikipedia](https://en.wikipedia.org/wiki/Unidad_de_Fomento)
- [ISO 4217 - Wikipedia](https://en.wikipedia.org/wiki/ISO_4217)
- [CLF to USD - fx-rate.net](https://fx-rate.net/CLF/USD/)
- `src/types/currency.rs` — Currency database and parsing
- `tests/currency_issues_tests.rs` — Existing test patterns for currency issues
- Related issues: #51, #52, #53 (parent #54) — Previous currency support fixes
