# Case Study: Issue #136 — Temporal Modifier Ignored in Unit Conversion

## Issue Summary

**Title**: Issue with expression: `22822 рублей в рупиях на 11 апреля 2026`  
**Reported**: 2026-04-25  
**Version**: 0.14.0  
**Labels**: bug

The expression `22822 рублей в рупиях на 11 апреля 2026` (Russian: "22822 rubles in rupees on April 11, 2026") produces:
- **Actual result**: 27996.41 INR (using the rate from 2026-04-17)
- **Expected result**: Should use the exchange rate effective on 2026-04-11

The lino notation showed `((22822 RUB) as INR)` instead of the expected `(((22822 RUB) as INR) at 2026-04-11)`.

---

## Input Expression Decoded

The URL parameter decodes to:
```
22822 рублей в рупиях на 11 апреля 2026
```

Broken down:
- `22822 рублей` = 22822 RUB (rubles)
- `в рупиях` = "in rupees" → `in INR` (currency conversion, "в" is already mapped to `In` keyword)
- `на` = "on/at" for dates → should map to `At` keyword (was MISSING)
- `11 апреля 2026` = April 11, 2026 (datetime, "апреля" is already supported)

---

## Timeline of Events

1. **Expression entered**: User types `22822 рублей в рупиях на 11 апреля 2026`
2. **Lexing**: "в" → `TokenKind::In` ✓, "на" → `TokenKind::Identifier("на")` ✗ (bug)
3. **Parsing**: Parser sees `22822 RUB in INR`, stops — remaining `на 11 апреля 2026` is ignored
4. **Evaluation**: `convert_to_unit("INR")` is called without any date context
5. **Rate lookup**: Current rate (2026-04-17) is used instead of historical (2026-04-11)
6. **Result**: 27996.41 INR (wrong rate) instead of the expected historical rate result

---

## Root Cause Analysis

### Root Cause 1: Parser doesn't handle "X as Y at date" pattern

**File**: `src/grammar/token_parser.rs`, function `parse_additive()`

The parse order was:
1. Parse arithmetic (`+`, `-`)
2. Check for `"at"` keyword → parse temporal modifier
3. Check for `"as"/"in"/"to"` keyword → parse unit conversion

For the expression `22822 RUB in INR at Apr 11, 2026`:
- Step 2: "at" check fails (next token is "in")
- Step 3: "in" check succeeds → builds `UnitConversion { 22822 RUB, INR }`
- **Parser returns** — the trailing `at Apr 11, 2026` is never consumed

The fix is to add a second "at" check after the unit conversion is parsed.

### Root Cause 2: `convert_to_unit` ignores date context

**File**: `src/types/value/mod.rs`, function `convert_to_unit()`

Even when an `AtTime` wrapper is present in the AST (e.g., `(22822 RUB as INR) at Apr 11, 2026`), the evaluation of `UnitConversion` calls `currency_db.convert()` without passing the date:

```rust
// BEFORE (broken):
Expression::UnitConversion { value, target_unit } => {
    let val = self.evaluate_expr(value)?;
    val.convert_to_unit(target_unit, &mut self.currency_db)  // No date!
}
```

Binary operations (`+`, `-`) already pass `current_date_context` via `add_at_date()` / `subtract_at_date()`, but `UnitConversion` did not.

### Root Cause 3: Russian "на" not mapped to `TokenKind::At`

**File**: `src/grammar/lexer.rs`

The keyword mapping table handled Russian "в" → `In` but had no entry for "на" (the Russian preposition meaning "at/on" for dates). This caused "на" to be tokenized as `Identifier("на")`, which is then ignored by the parser.

---

## Requirements

1. **R1**: The expression `X as Y at date` (and variants with `in`, `to`) must parse correctly, producing an `AtTime(UnitConversion(...))` AST node.

2. **R2**: When evaluating `UnitConversion` inside an `AtTime` context, the historical exchange rate for the specified date must be used.

3. **R3**: The Russian preposition "на" must be recognized as the `At` keyword, enabling `22822 рублей в рупиях на 11 апреля 2026` to work.

4. **R4**: The lino notation must reflect the temporal modifier: `(((22822 RUB) as INR) at 2026-04-11)`.

5. **R5**: All other supported languages (French, German, Chinese, Hindi, Arabic) should be verified that their equivalent of "at" (for dates) works. Only Russian was identified as missing.

6. **R6**: Non-currency unit conversions (data size, mass) are unaffected by date context (no historical rates concept applies).

---

## Solutions Implemented

### Fix 1: Parser handles "X as Y at date"

In `src/grammar/token_parser.rs`, added a second "at" check after the unit conversion block:

```rust
// Check for "as", "in", or "to" keyword
if self.check_as() || self.check_in() || self.check_to() {
    self.advance();
    let target_unit = self.parse_unit_for_conversion()?;
    left = Self::resolve_unit_ambiguity_for_conversion(left, &target_unit);
    left = Expression::unit_conversion(left, target_unit);

    // NEW: check for trailing "at <date>" after unit conversion
    if self.check_at() {
        self.advance();
        let time = self.parse_primary()?;
        left = Expression::at_time(left, time);
    }
}
```

### Fix 2: `convert_to_unit_at_date` threads date context

In `src/types/value/mod.rs`:
- Added `convert_to_unit_at_date(&self, target_unit, currency_db, date: Option<&DateTime>)` 
- `convert_to_unit` now delegates to `convert_to_unit_at_date(…, None)`
- For currency conversions, calls `currency_db.convert_at_date()` when `date` is provided

In `src/grammar/expression_parser.rs`, all three `UnitConversion` evaluation paths now pass `self.current_date_context`:

```rust
Expression::UnitConversion { value, target_unit } => {
    let val = self.evaluate_expr(value)?;
    val.convert_to_unit_at_date(
        target_unit,
        &mut self.currency_db,
        self.current_date_context.as_ref(),  // NEW
    )
}
```

### Fix 3: Russian "на" → `TokenKind::At`

In `src/grammar/lexer.rs`:

```rust
// Russian: "на" means "at/on" for temporal context
"на" => TokenKind::At,
```

---

## Verification

Created test file: `tests/issue_136_temporal_unit_conversion_tests.rs`

All 10 tests pass:
- `test_unit_conversion_with_trailing_at_parses` — lino contains "at"
- `test_unit_conversion_at_date_lino_format` — lino format correct
- `test_unit_conversion_at_date_uses_historical_rate` — uses rate 1.5 not 1.2
- `test_unit_conversion_different_dates_give_different_rates` — different dates → different results
- `test_unit_conversion_in_keyword_with_at_date` — "in" keyword works
- `test_unit_conversion_to_keyword_with_at_date` — "to" keyword works
- `test_russian_na_as_at_keyword` — "на" mapped correctly
- `test_russian_rub_in_inr_at_date` — full Russian expression
- `test_original_issue_russian_date_parses` — original issue expression
- `test_data_size_conversion_ignores_date_context` — no regression on non-currency conversions

All pre-existing tests continue to pass (0 regressions across all test suites).

---

## Related Issues

- **Issue #75**: Added Russian "в" → `In` keyword (same pattern as this fix)
- **Issue #134**: Russian partial date subtraction (similar datetime parsing context)
- **Issues #51-53**: Currency symbol and multilingual currency support

---

## Files Changed

| File | Change |
|------|--------|
| `src/grammar/lexer.rs` | Added "на" → `TokenKind::At` |
| `src/grammar/token_parser.rs` | Added "at" check after unit conversion parsing |
| `src/types/value/mod.rs` | Added `convert_to_unit_at_date()` with date parameter |
| `src/grammar/expression_parser.rs` | Pass `current_date_context` to all `UnitConversion` handlers |
| `tests/issue_136_temporal_unit_conversion_tests.rs` | New test file (10 tests) |
| `docs/case-studies/issue-136/README.md` | This case study |
