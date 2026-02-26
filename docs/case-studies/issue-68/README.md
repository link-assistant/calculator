# Case Study: Issue #68 - Unrecognized input: `UTC time`

## Timeline of Events

1. **User Input**: `UTC time`
2. **Error Received**: `Invalid datetime format: Could not parse 'UTC time' as a date or time`
3. **Root Cause**: The parser recognizes `UTC` as a datetime indicator but cannot parse the combined phrase "UTC time" as a valid datetime.

## Root Cause Analysis

The input `UTC time` is tokenized as two identifiers: `Identifier("UTC")` and `Identifier("time")`.

In `DateTimeGrammar::looks_like_datetime()`, the check `input.contains("utc")` returns true, so the parser attempts to parse it as a datetime. However, `DateTime::parse("UTC time")` fails because:
- It doesn't match any date format
- It doesn't match any time format (no colon or am/pm)
- It doesn't match any datetime format

The phrase "UTC time" is a natural language query meaning "What is the current time in UTC?"

## Expected Behavior

Per the issue:
1. Should be interpreted as current UTC time, showing current UTC date and time
2. Should also provide a term interpretation linking to the Wikipedia article about Coordinated Universal Time

## Proposed Solution

### Primary Interpretation: Current UTC DateTime
- Detect phrases like `UTC time`, `time UTC`, `current UTC time`, `UTC now` as requests for current UTC datetime
- Return `Utc::now()` as a DateTime value with UTC timezone

### Implementation Approach
- In `DateTime::parse()`, add early detection for `now`/`time`/`current time` keywords combined with timezone indicators
- Handle variations: `UTC time`, `time UTC`, `UTC now`, `now UTC`, `current time UTC`

## Related Concepts

### Coordinated Universal Time (UTC)
- Primary time standard for civil timekeeping
- Successor to Greenwich Mean Time (GMT)
- Basis for all timezone offsets worldwide
- Wikipedia: https://en.wikipedia.org/wiki/Coordinated_Universal_Time

## Related Libraries

- **chrono** (Rust): `Utc::now()` returns current UTC time
- Most programming languages provide a `now()` function for UTC time

## Data Files

- Error log: `Invalid datetime format: Could not parse 'UTC time' as a date or time`
- Input expression: `UTC time`
