# Case Study: Issue #34 - Unrecognized input: `(Jan 27, 8:59am UTC) - (now UTC)`

## Timeline of Events

1. **User Input**: `(Jan 27, 8:59am UTC) - (now UTC)`
2. **Error Received**: `Parse error: Unexpected identifier: now`
3. **Root Cause**: The parser does not recognize `now` as a keyword or datetime expression.

## Root Cause Analysis

The calculator's `TokenParser::parse_primary()` method handles identifiers by checking:
1. If it's a prefix currency symbol
2. If it's a function call (identifier + left paren)
3. If it starts with "integrate"
4. If it looks like a datetime (via `DateTimeGrammar::looks_like_datetime()`)
5. If it's a math constant
6. If it's a single-letter variable

The keyword `now` fails all these checks because:
- `DateTimeGrammar::looks_like_datetime("now")` returns `false` - it checks for month names, am/pm, utc/gmt, ISO patterns, and time patterns with colons
- `now` is not a math function or single-letter variable
- Result: falls through to `Err(CalculatorError::parse("Unexpected identifier: now"))`

## Expected Behavior

The issue requests support for these patterns:
- `(now UTC)` - current time in UTC
- `(utc now)` - current time in UTC (alternative ordering)
- `(now)` - current time
- `now` - current time as standalone

## Proposed Solution

1. **Lexer level**: No changes needed - `now` will be tokenized as `Identifier("now")`
2. **DateTime Grammar**: Add `now` detection in `looks_like_datetime()`
3. **DateTime parsing**: Add `now` handling in `DateTime::parse()` that returns `Utc::now()`
4. **Token Parser**: The existing `try_parse_datetime_from_tokens` path will handle `now UTC` combinations once the grammar recognizes it

## Related Libraries

- **chrono** (Rust): `Utc::now()` provides current UTC datetime
- **dateparser** (Python): Supports "now" keyword natively
- **natural-date-parser** (Rust): Handles "now" expressions
- **Temporal API** (JavaScript): `Temporal.Now.zonedDateTimeISO()`

## Data Files

- Error log: `Parse error: Unexpected identifier: now`
- Input expression: `(Jan 27, 8:59am UTC) - (now UTC)`
