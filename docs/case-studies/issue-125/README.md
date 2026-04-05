# Case Study: Issue #125 — Russian Date Arithmetic Expressions

## Overview

**Issue:** https://github.com/link-assistant/calculator/issues/125
**Status:** Root cause found and fixed
**Severity:** High — Russian-language date arithmetic expressions silently returned wrong results instead of computing the expected date

### Symptoms

1. Expression `17 февраля 2027 - 6 месяцев` returned `17 февраля` (incomplete result)
2. Links notation showed `(17 февраля)` — only the first two tokens were parsed
3. Steps showed `Literal value: 17 февраля` — the entire expression was treated as a literal string, not as date arithmetic
4. The expected behavior: interpret the expression as `(17 февраля 2027) - 6 месяцев` → `2026-08-17`

---

## Timeline

| Date | Event |
|------|-------|
| 2026-04-05T12:19:30 | Issue #125 filed with expression `17 февраля 2027 - 6 месяцев`, version 0.11.1 |
| 2026-04-05 | Root cause analysis: 4 distinct root causes identified |
| 2026-04-05 | Fix implemented, all existing tests pass, 17 new tests added |

---

## Root Cause Analysis

### The Four Root Causes

The expression `17 февраля 2027 - 6 месяцев` failed due to four separate, compounding issues in the parsing pipeline.

#### Root Cause 1: Russian Month Names Not Recognized in `looks_like_datetime()`

`DateTimeGrammar::looks_like_datetime()` (in `src/grammar/datetime_grammar.rs`) checked only English month names to determine whether a token looks like part of a datetime expression:

```rust
let month_names = [
    "jan", "feb", "mar", "apr", "may", "jun",
    "jul", "aug", "sep", "oct", "nov", "dec",
    "january", "february", "march", "april", "june",
    "july", "august", "september", "october", "november", "december",
];
```

Russian month names like `февраля` (February, genitive), `января` (January, genitive), etc. were entirely absent. This function is used in `parse_primary()` to decide when to try datetime parsing; without Russian month names, the parser never attempted to parse `февраля` as a datetime component.

#### Root Cause 2: Number + Month-Name Identifier Not Tried as Datetime

In `TokenParser::parse_primary()` (in `src/grammar/token_parser.rs`), when the parser encounters `Number(17)` followed by `Identifier(февраля)`, it tries:
1. `HH:MM` colon time pattern → not applicable
2. `N AM/PM` pattern → not applicable
3. **Unit parsing** → `февраля` doesn't match any known unit, so `Unit::Custom("февраля")` is returned

The parser never checked whether the identifier after a number might be a month name (datetime component). The result was `Expression::Number { value: 17, unit: Custom("февраля") }`, and the rest of the expression (`2027 - 6 месяцев`) was ignored or became part of a different sub-expression.

#### Root Cause 3: Russian Duration Units Not Recognized

`DurationUnit::parse()` (in `src/types/unit.rs`) only supported English duration unit names:

```rust
"mo" | "month" | "months" => Some(Self::Months),
```

Russian words like `месяцев` (months, genitive plural), `месяца` (genitive singular), `месяц` (nominative), `дней` (days), `недель` (weeks), `лет` (years), etc. were absent. Even if the date part parsed correctly, `6 месяцев` would fail to be recognized as "6 months".

#### Root Cause 4: `try_parse_datetime_from_tokens` Did Not Backtrack

`TokenParser::try_parse_datetime_from_tokens()` collected all following tokens (numbers, identifiers, commas, colons) into a string and attempted to parse it as a datetime. If parsing failed, it returned an error — it did not try progressively shorter prefixes.

This caused expressions like `22 Jan 2027 - 6 months` to fail:
- Parser sees `Jan` as a datetime-looking identifier
- Collects `Jan 22 2027` (stops before `-`)
- Tries to parse `"Jan 22 2027"` — this failed because only `"Jan 22, 2027"` (with comma) was supported

Additionally, `DateTime::try_parse_date_formats()` was missing the formats `"Mon DD YYYY"` and `"DD Mon YYYY"` (without comma).

### Parsing Flow Trace

For `17 февраля 2027 - 6 месяцев`:

```
Lexer output: [Number("17"), Ident("февраля"), Number("2027"), Minus, Number("6"), Ident("месяцев"), EOF]

TokenParser::parse_primary():
  1. Sees Number("17"), advances
  2. Sees Ident("февраля"):
     - Is it a colon? No.
     - Is it AM/PM? No.
     - Is it a datetime identifier? (looks_like_datetime("февраля")) → FALSE (bug!)
     - Treat as unit: DurationUnit::parse("февраля") → None
                      CurrencyDatabase::parse_currency("февраля") → None
                      → Unit::Custom("февраля")
  3. Returns Expression::Number { value: 17, unit: Custom("февраля") }
  4. Parser stops — next token Number("2027") cannot follow a complete expression at top level

Result: "17 февраля" (literal), rest of expression ignored
```

### What the Fix Does

For the same input after the fix:

```
TokenParser::parse_primary():
  1. Sees Number("17"), advances
  2. Sees Ident("февраля"):
     - looks_like_datetime("февраля") → TRUE (fix #1: Russian months added)
     - Tries try_parse_datetime_from_tokens("17"):
         Collects: ["17", "февраля", "2027"] (stops at Minus)
         Tries "17 февраля 2027":
           DateTime::parse("17 февраля 2027"):
             translate_russian_months("17 февраля 2027") → "17 February 2027"
             try_parse_date_formats("17 February 2027"):
               NaiveDate::parse_from_str("17 February 2027", "%d %B %Y") → 2027-02-17 ✓
         Returns Expression::DateTime(2027-02-17)

parse_additive():
  Sees Minus
  parse_primary() for right side:
    Sees Number("6"), advances
    Sees Ident("месяцев"):
      DurationUnit::parse("месяцев") → Some(Months) (fix #3: Russian durations added)
    Returns Expression::Number { value: 6, unit: Duration(Months) }
  Binary subtraction: DateTime(2027-02-17) - 6_months

Evaluation:
  DateTime(2027-02-17) subtract 6 months → 2026-08-17
```

---

## Fix Summary

### Changes Made

#### 1. `src/grammar/datetime_grammar.rs` — Add Russian month names to `looks_like_datetime()`

Added all Russian month names in both nominative and genitive forms to the month_names array:
- Nominative: январь, февраль, март, апрель, май, июнь, июль, август, сентябрь, октябрь, ноябрь, декабрь
- Genitive (most common in date expressions): января, февраля, марта, апреля, июня, июля, августа, сентября, октября, ноября, декабря

#### 2. `src/types/datetime.rs` — Add Russian month name translation

Added a new `translate_russian_months()` method that converts Russian month names to their English equivalents before parsing. This is called at the start of `DateTime::parse()` as a preprocessing step.

Also added missing date formats `"%b %d %Y"` and `"%B %d %Y"` (month-day-year without comma) to `try_parse_date_formats()`.

#### 3. `src/types/unit.rs` — Add Russian duration units to `DurationUnit::parse()`

Added all Russian duration unit names in all grammatical cases:
- Milliseconds: мс, миллисекунда, миллисекунды, миллисекунд, ...
- Seconds: с, сек, секунда, секунды, секунд, ...
- Minutes: мин, минута, минуты, минут, ...
- Hours: ч, час, часа, часов, ...
- Days: д, день, дня, дней, ...
- Weeks: нед, неделя, недели, недель, ...
- Months: мес, месяц, месяца, месяцев, ...
- Years: г, год, года, лет, ...

#### 4. `src/grammar/token_parser.rs` — Fix datetime parsing for number + month patterns and backtracking

Two sub-fixes:
1. When a number is followed by a datetime-looking identifier (month name), try parsing the whole sequence as a datetime before falling through to unit parsing.
2. Fixed `try_parse_datetime_from_tokens` to try progressively shorter token prefixes when the full collected string fails to parse, enabling backtracking.

### Tests Added

File: `tests/issue_125_russian_date_arithmetic_tests.rs` — 17 new tests covering:
- The exact expression from the issue: `17 февраля 2027 - 6 месяцев`
- Russian month names in all commonly used forms (nominative, genitive)
- Russian duration unit names in various grammatical cases
- English date formats without comma: `Feb 17 2027 - 6 months`, `17 February 2027 - 6 months`
- Addition and subtraction with Russian dates and durations
- Edge cases: different months, years, days

---

## Related Issues and Prior Art

### Related Calculator Issues

- **Issue #75** — Russian currency conversion (`1000 рублей в долларах`): Added Russian preposition "в" as `In` keyword and Russian currency names in all grammatical cases. The same pattern of adding multilingual support was followed here.
- **Issue #83** — Duration arithmetic with `now`: Fixed `now - duration` expressions, establishes the DateTime subtraction pattern.
- **Issue #45** — Datetime countdown expressions: Established datetime arithmetic pattern.

### Linguistic Context

Russian is a highly inflected language with 6 grammatical cases. Month names and duration words decline differently depending on their syntactic role:

| Case | месяц (month) | февраль (February) |
|------|--------------|-------------------|
| Nominative (subject) | месяц | февраль |
| Genitive (possession, "of") | месяца | февраля |
| Genitive plural | месяцев | — |
| Dative ("to") | месяцу | февралю |
| Accusative (object) | месяц | февраль |
| Instrumental ("by/with") | месяцем | февралём |
| Prepositional ("about/in") | месяце | феврале |

In date expressions, the genitive case is most common:
- `17 февраля 2027` = "17 of February 2027" (genitive: февраля)
- `6 месяцев` = "6 of months" (genitive plural: месяцев)

### External References

- [Russian grammar: noun declension](https://en.wikipedia.org/wiki/Russian_grammar#Nouns)
- [Chrono date formatting](https://docs.rs/chrono/latest/chrono/format/strftime/index.html): `%B` = full month name, `%b` = abbreviated
- [Unicode month names](https://www.unicode.org/cldr/charts/46/summary/ru.html): CLDR Russian locale month name forms

---

## Lessons Learned

1. **Multilingual date support requires all grammatical cases.** Russian (and many other inflected languages) express the same concept with different word forms depending on grammar. A parser handling Russian dates must recognize all common forms, not just the nominative.

2. **Greedy token collection needs backtracking.** The original `try_parse_datetime_from_tokens` collected all eligible tokens and tried to parse the whole string. A better approach tries the full string first, then shorter prefixes, enabling the expression `Jan 22 2027 - 6 months` to correctly parse `Jan 22 2027` as a date even though the format without comma wasn't supported at the parse level.

3. **Number + month-name disambiguation.** When a number is followed by a month name (or other datetime-looking identifier), it should be tried as a datetime before being treated as a number with a custom unit.

4. **Test all related patterns when adding multilingual support.** Adding Russian months also revealed that English date formats without commas (`Feb 17 2027`) were missing, even though they would naturally be expected to work.
