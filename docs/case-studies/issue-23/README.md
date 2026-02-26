# Case Study: Issue #23 - Unrecognized input: `until 11:59pm EST on Monday, January 26th`

## Timeline of Events

1. **User Input**: `until 11:59pm EST on Monday, January 26th`
2. **Error Received**: `Parse error: Unexpected identifier: until`
3. **Root Cause**: The parser does not recognize `until` as a keyword, and cannot parse the natural date format with day names and ordinal suffixes.

## Root Cause Analysis

Multiple parsing failures compound:

### Problem 1: `until` keyword not recognized
The lexer tokenizes `until` as `Identifier("until")`, and the parser has no handling for this keyword. It's not in the keyword list (at, as, in, to).

### Problem 2: Timezone abbreviation `EST` not supported
The `DateTime::extract_timezone()` method only supports:
- `UTC` and `GMT` suffixes
- Explicit offsets like `+05:00` or `-08:00`

Common US timezone abbreviations (EST, CST, MST, PST, EDT, CDT, MDT, PDT) are not recognized.

### Problem 3: Day name `Monday` not supported
The datetime parser has no handling for day-of-week names.

### Problem 4: Ordinal suffix `26th` not supported
The parser cannot handle ordinal suffixes (`1st`, `2nd`, `3rd`, `4th`-`31st`).

### Problem 5: `on` preposition not handled
The `on` keyword between time and date is not recognized.

## Expected Behavior

Per the issue:
- `until 11:59pm EST on Monday, January 26th` should calculate the duration from now to that date/time
- `11:59pm EST on Monday, January 26th` should be parsable as a standalone datetime
- By default, show the difference between now and that time

## Proposed Solution

### 1. Add `until` keyword support
- Interpret `until <datetime>` as `<datetime> - now` (time remaining until that point)

### 2. Add timezone abbreviation support
Common US timezone abbreviations with their UTC offsets:
| Abbreviation | Name | Offset |
|---|---|---|
| EST | Eastern Standard Time | UTC-5 |
| EDT | Eastern Daylight Time | UTC-4 |
| CST | Central Standard Time | UTC-6 |
| CDT | Central Daylight Time | UTC-5 |
| MST | Mountain Standard Time | UTC-7 |
| MDT | Mountain Daylight Time | UTC-6 |
| PST | Pacific Standard Time | UTC-8 |
| PDT | Pacific Daylight Time | UTC-7 |
| AKST | Alaska Standard Time | UTC-9 |
| HST | Hawaii Standard Time | UTC-10 |

Note: Timezone abbreviations are inherently ambiguous (e.g., CST = Central Standard Time OR China Standard Time). We limit support to common unambiguous US/global abbreviations.

### 3. Add `on` preposition and day name support
- Strip day names (Monday-Sunday) as they're redundant with the date
- Handle `on` as a separator between time and date parts

### 4. Add ordinal suffix support
- Strip ordinal suffixes (st, nd, rd, th) from day numbers

## Related Libraries

- **chrono-tz** (Rust): Full IANA timezone database
- **date_time_parser** (Rust): Natural language date parsing
- **dateutil** (Python): Handles ordinal dates, day names, and timezone abbreviations

## Data Files

- Error log: `Parse error: Unexpected identifier: until`
- Input expression: `until 11:59pm EST on Monday, January 26th`
