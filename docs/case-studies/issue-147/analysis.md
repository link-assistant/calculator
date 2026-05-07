# Case Study: Issue #147 — Unrecognized input: `300000 ms in seconds`

## Issue Details

- **Repository**: https://github.com/link-assistant/calculator
- **Issue**: #147
- **Title**: Unrecognized input: 300000 ms in seconds
- **State**: Open
- **Labels**: bug
- **Author**: konard (Konstantin Diachenko)

## Input that Failed to Parse

```
300000 ms in seconds
```

## Error Message

```
Parse error: Unknown unit 'seconds'. Supported conversions: data sizes (B, KB, MB, GB, KiB, MiB, GiB, ...), mass (g, kg, tons, lb, oz), currencies (USD, EUR, GBP, TON, BTC, ETH, ...) and natural language aliases (dollars, euros, bitcoin, toncoin, ...), timezones (UTC, GMT, EST, MSK, JST, ...).
```

## Expected Behavior

`ms` (milliseconds) to `seconds` conversion should work.

## Timeline / Sequence of Events

1. User inputs `300000 ms in seconds`
2. Lexer tokenizes the input: `[Number(300000), Identifier(ms), In, Identifier(seconds)]`
3. Parser sees `300000 ms` → recognized as `Expression::Number { value: 300000, unit: Unit::Duration(DurationUnit::Milliseconds) }`
4. Parser sees `in` keyword → calls `parse_unit_for_conversion()`
5. `parse_unit_for_conversion()` tries:
   - `DataSizeUnit::parse("seconds")` → None
   - `MassUnit::parse("seconds")` → None
   - Case-insensitive data size → None
   - Case-insensitive mass → None
   - Timezone check → None
   - `CurrencyDatabase::parse_currency("seconds")` → None
6. Returns error: `Unknown unit 'seconds'`

## Root Cause Analysis

### Root Cause 1: `parse_unit_for_conversion` omits Duration units

**File**: `src/grammar/token_parser.rs`, function `parse_unit_for_conversion` (line ~816)

The function handles unit conversions after `as`/`in`/`to` keywords. It tries DataSize, Mass, Timezone, and Currency units — but **completely omits `DurationUnit`**. This means time unit names like "seconds", "ms", "minutes", "hours" are never recognized as valid conversion targets.

**Evidence**: `DurationUnit::parse()` in `src/types/unit.rs` (line ~553) correctly handles all these aliases:
```rust
"ms" | "millisecond" | "milliseconds" => Some(Self::Milliseconds),
"s" | "sec" | "secs" | "second" | "seconds" => Some(Self::Seconds),
"min" | "mins" | "minute" | "minutes" => Some(Self::Minutes),
"h" | "hr" | "hrs" | "hour" | "hours" => Some(Self::Hours),
```

### Root Cause 2: `convert_to_unit_at_date` lacks Duration→Duration case

**File**: `src/types/value/mod.rs`, function `convert_to_unit_at_date` (line ~631)

The function handles DataSize→DataSize, Currency→Currency, Mass→Mass, and Timezone conversions. It has **no handler for Duration→Duration** conversions.

## Solution Plan

### Fix 1: Add `DurationUnit` check in `parse_unit_for_conversion`

In `src/grammar/token_parser.rs`, add before the error return:

```rust
// Try duration/time unit (e.g., "seconds", "ms", "minutes", "hours")
if let Some(duration) = crate::types::DurationUnit::parse(&unit_str) {
    return Ok(Unit::Duration(duration));
}
```

### Fix 2: Add Duration→Duration case in `convert_to_unit_at_date`

In `src/types/value/mod.rs`, add a new match arm for `Duration(from) → Duration(to)`:

```rust
(Unit::Duration(from), Unit::Duration(to)) => {
    let value_f64 = self.as_decimal().ok_or_else(|| {
        CalculatorError::InvalidOperation(
            "duration conversion requires a numeric value".into(),
        )
    })?;
    // Convert: from → seconds → to
    let secs = from.to_secs(value_f64.to_f64());
    let result = to.secs_to_unit(secs);
    Ok(Value::number_with_unit(
        Decimal::from_f64(result),
        Unit::Duration(*to),
    ))
}
```

## Test Cases

- `300000 ms in seconds` → `300` (300,000 ms / 1000 = 300 s)
- `5 minutes in seconds` → `300`
- `2 hours in minutes` → `120`
- `1 hour in ms` → `3600000`

## Related Code Locations

- `src/grammar/token_parser.rs:816` — `parse_unit_for_conversion()`
- `src/types/unit.rs:548` — `DurationUnit::parse()`
- `src/types/value/mod.rs:631` — `convert_to_unit_at_date()`
- `src/grammar/number_grammar.rs:93` — duration unit parsing for source (already works)
