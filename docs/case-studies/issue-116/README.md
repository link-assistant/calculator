# Case Study: Issue #116 — Vietnamese Dong to Russian Rubles conversion fails

## Summary

Expression `2340000 донгов СРВ в рублях` fails to convert Vietnamese Dong (VND) to Russian Rubles (RUB). The calculator returns the input as a literal value (`2340000 донгов`) without performing any conversion.

## Environment

- **Version**: 0.8.4
- **URL**: https://link-assistant.github.io/calculator/?q=KGV4cHJlc3Npb24lMjAlMjIyMzQwMDAwJTIwJUQwJUI0JUQwJUJFJUQwJUJEJUQwJUIzJUQwJUJFJUQwJUIyJTIwJUQwJUExJUQwJUEwJUQwJTkyJTIwJUQwJUIyJTIwJUQxJTgwJUQxJTgzJUQwJUIxJUQwJUJCJUQxJThGJUQxJTg1JTIyKQ
- **Timestamp**: 2026-03-28T16:15:10.991Z

## Input

```
2340000 донгов СРВ в рублях
```

## Expected Result

Conversion of 2,340,000 Vietnamese Dong to Russian Rubles (approximately 7,568 RUB at current CBR rates where 10,000 VND ≈ 32.33 RUB).

## Actual Result

```
Result: 2340000 донгов
Links Notation: (2340000 донгов)
Steps:
  1. Input expression: 2340000 донгов
  2. Literal value: 2340000 донгов
  3. Final result: 2340000 донгов
```

The calculator:
1. Parsed `2340000` as a number
2. Parsed `донгов` as an unrecognized identifier → treated as `Custom("донгов")` unit
3. Parsed `СРВ` as a separate identifier (not recognized)
4. The `в` keyword was consumed as `TokenKind::In` (conversion operator)
5. `рублях` was correctly parsed as RUB
6. But since the source unit was `Custom("донгов")` (not `Currency("VND")`), the `UnitConversion` either failed silently or was never created

## Timeline / Sequence of Events

### Parsing Phase (Lexer)
```
"2340000 донгов СРВ в рублях"
  ↓ lexer.tokenize()
[Number("2340000"), Identifier("донгов"), Identifier("СРВ"), In, Identifier("рублях"), EOF]
```

### AST Construction (Token Parser)
```
Token: Number("2340000") + Identifier("донгов")
  → parse_unit_with_alternatives("донгов")
  → DataSizeUnit::parse("донгов") → None
  → MassUnit::parse("донгов") → None
  → DurationUnit::parse("донгов") → None
  → CurrencyDatabase::parse_currency("донгов") → None (NOT FOUND!)
  → Falls through to Custom("донгов")

Result: Expression::Number { value: 2340000, unit: Custom("донгов") }

Token: Identifier("СРВ")
  → Parsed as standalone expression (separate from the number)
  → This breaks the expected "number unit conversion-keyword target" pattern

Token: In (from "в")
  → Would trigger unit conversion, but AST structure is wrong at this point

Token: Identifier("рублях")
  → CurrencyDatabase::parse_currency("рублях") → Some("RUB") ✓
```

The core issue is that `донгов` is never recognized as VND, so the expression cannot form a valid `UnitConversion` node.

## Root Causes

### Root Cause 1: Missing Russian-language aliases for VND

**File**: `src/types/currency.rs` (lines 469-682)
**Function**: `CurrencyDatabase::parse_currency()`

The function has comprehensive Russian-language coverage for:
- RUB (рубль, рубля, рубле, рубли, рублей, рублям, рублю, рублём, рублем, рублями, рублях)
- USD (доллар, доллара, долларе, доллары, долларов, долларам, доллару, долларом, долларами, долларах)
- EUR (евро)
- GBP (фунт + all cases)
- CNY (юань + all cases)
- JPY (иена + all cases)
- INR (рупия + all cases)

But **no entries exist for VND** in any language:
- No "донг/донга/донге/донги/донгов/донгам/донгу/донгом/донгами/донгах" (Russian declensions)
- No "dong/đồng" (English/Vietnamese)
- No "₫" symbol

### Root Cause 2: "СРВ" qualifier not handled

"СРВ" (Социалистическая Республика Вьетнам — Socialist Republic of Vietnam) is a Russian abbreviation used to disambiguate the dong currency. The parser has no mechanism for:
- Compound currency names with country qualifiers (like "донгов СРВ")
- The "СРВ" abbreviation itself

Note: The generic 2-5 letter catch-all pattern in `parse_currency()` only matches ASCII alphabetic characters (`c.is_ascii_alphabetic()`), so Cyrillic "СРВ" returns `None`.

### Root Cause 3: VND not in rate source mappings

**File**: `src/plan.rs` (lines 58-169)

Even if VND were parsed correctly:
- `primary_source("VND")` would return `RateSource::Ecb` (default for non-RUB, non-crypto)
- But **ECB/Frankfurter does not provide VND rates** (only 32 currencies, VND not included)
- VND is not listed in `can_also_serve()` CBR fiat list (lines 115-144), so CBR wouldn't be used as fallback
- **CBR does provide VND rates** (R01150, Nominal=10000, current rate ≈ 32.33 RUB per 10,000 VND)

### Root Cause 4: No VND fallback .lino data files

**Directory**: `data/currency/`

No `rub-vnd.lino` or `vnd-rub.lino` files exist. The CBR data download script (`scripts/download_historical_rates.py`) only includes 7 currency pairs for CBR (USD, EUR, GBP, JPY, CHF, CNY, INR), missing VND.

## Data Sources

### CBR (Central Bank of Russia) — provides VND
- **API**: https://www.cbr.ru/scripts/XML_daily.asp
- **VND Valute ID**: R01150
- **Nominal**: 10,000 VND
- **Current Rate**: ~32.33 RUB per 10,000 VND
- **Computed per-unit rate**: 1 VND ≈ 0.003233 RUB

### ECB/Frankfurter — does NOT provide VND
- **API**: https://api.frankfurter.app/currencies
- **VND**: Not in the list of 32 supported currencies

### Real-world reference
At CBR rate of 32.3284 RUB per 10,000 VND:
- 2,340,000 VND = 2,340,000 × (32.3284 / 10,000) = **~7,564.84 RUB**

## Proposed Solutions

### Solution 1: Add Russian-language VND aliases to `parse_currency()`

Add all grammatical cases of "донг" (dong) in Russian:
```rust
// Russian language names for VND (Vietnamese Dong, all grammatical cases)
// Nominative: донг, донги; Genitive: донга, донгов;
// Dative: донгу, донгам; Instrumental: донгом, донгами;
// Prepositional: донге, донгах
"донг" | "донга" | "донге" | "донги" | "донгов" | "донгам" | "донгу" | "донгом"
| "донгами" | "донгах" => return Some("VND".to_string()),
```

Also add English and Vietnamese aliases:
```rust
"dong" | "dongs" | "đồng" => return Some("VND".to_string()),
```

And the ₫ symbol in the symbol section.

### Solution 2: Add VND to CBR rate source mappings

In `plan.rs`, add "VND" to the `can_also_serve()` CBR fiat currency list so that VND↔RUB conversions use CBR rates (since ECB doesn't have VND).

Additionally, make VND's primary source CBR (since ECB doesn't provide it).

### Solution 3: Add VND to CBR data pipeline

In `scripts/download_historical_rates.py`, add VND to the `CBR_CURRENCIES` dictionary:
```python
"R01150": "VND",  # Vietnamese Dong (10000 VND = X RUB nominal)
```

### Solution 4: Handle "СРВ" qualifier (lower priority)

Options:
- Add "донгов срв" / "донг срв" as compound aliases → VND
- Or simply rely on "донгов" alone being sufficient (most users will not include "СРВ")

## Related Components

| Component | File | What needs changing |
|-----------|------|-------------------|
| Currency parser | `src/types/currency.rs` | Add VND aliases |
| Rate source mapping | `src/plan.rs` | Add VND to CBR coverage |
| CBR data script | `scripts/download_historical_rates.py` | Add VND to CBR_CURRENCIES |
| Default rates | `src/types/currency.rs` | Add VND default rate |
| Lexer | `src/grammar/lexer.rs` | Add ₫ symbol support |
