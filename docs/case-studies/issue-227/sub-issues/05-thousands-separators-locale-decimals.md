---
title: "Thousands separators and locale decimal commas give silently wrong answers"
---
Part of #227. Highest-priority finding: wrong results, not errors.

## Evidence (verbatim competitor examples, current output)

| Expression | Source | Expected | Actual |
| --- | --- | --- | --- |
| `3,000 minus 12` | Soulver | 2,988 | **-9** |
| `1,000 divided by 200` | Soulver | 5 | **0.005** |
| `100,000 + 200,000` | Soulver | 300k | **300** |
| `56.7% of 1,234 participants` | paper phrasing | 699.678 | **0.699678 participants** |
| `1,1` | fend | 11 | 1.1 |
| `12,3 ± 4,5` (ru) | paper phrasing | 12.3 ± 4.5 | `Unexpected character '±'` |
| `1,08 Mrd. km/h` (de) | Wikipedia | 1 080 000 000 km/h | `Unexpected trailing input ','` |

`3,000` is read as the two numbers `3` and `000` and the comma is treated as a separator, so the first operand becomes `3`. Issue #217 fixed Unicode grouping spaces (`15 847`) but not ASCII commas, and the locale fallback in `src/grammar/locale_numbers.rs` only triggers when the whole expression fails.

## Acceptance criteria

- All rows above pass in `scripts/competitor-coverage.mjs`; no `supported` row of the baseline regresses.
- `1,234.5`, `1.234,5`, `1 234,5`, `1'234.5` (Swiss), `12,34,567` (Indian lakh grouping) resolve deterministically; ambiguous forms such as `1,234` with no decimal part follow a documented rule (fend/Soulver/Numi: thousands separator when exactly three digits follow).
- The rule is implemented in the lexer or a single pre-pass, not as a fallback after a parse failure, so that `3,000 minus 12` never reaches the old interpretation.
- Argument separation in function calls (`max(1,2)`) still works; `round(1,234)` is documented.

## Solution options

1. Lexer rule: a comma followed by exactly three digits and not followed by another digit is a grouping separator; a comma followed by one, two or four or more digits is a decimal separator if the expression has no `.` decimals. This is what fend does (`1,1` → 11 is its documented outlier). Recommended; small change in `src/lexer`.
2. Locale profile selected by the caller (`en`, `de`, `ru`, `hi`) with ICU-style patterns via `icu_decimal` 2.3. More correct for `1.234,5` but needs an API change and a CLI flag; can be layered on option 1 later.

## Blockers

- Blocked by: #229 (regression gate must be in place because this touches the lexer used by every corpus row).
