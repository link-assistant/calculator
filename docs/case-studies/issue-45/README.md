# Case Study: Issue #45 - Issue with expression: `Jan 27, 9:33am UTC`

## Timeline of Events

1. **User Input**: `Jan 27, 9:33am UTC`
2. **Result**: `2026-01-27 09:33:00 +00:00` (successfully parsed)
3. **Issue**: When input is a standalone date, the result should show a countdown (future) or elapsed time (past), not just echo back the datetime

## Root Cause Analysis

The parser correctly parses `Jan 27, 9:33am UTC` as a DateTime value. However, the evaluator simply returns the DateTime as-is without computing any relationship to the current time.

Currently, `Expression::DateTime(dt)` evaluates to `Value::datetime(dt.clone())`, and `Value::datetime` displays as the formatted datetime string. There is no logic to compute the difference from "now".

### Secondary Issue: Links Notation
The issue also states that composite datetime expressions should be enclosed in parentheses in links notation, like `(2026-01-27 09:33:00 +00:00)`. Currently, `Expression::DateTime` in `to_lino()` just returns `dt.to_string()` without wrapping.

## Expected Behavior

Per the issue:
1. **Date-only input**: When a user enters just a date/time, show the time difference from now
   - If in the future: show countdown ("in X days, Y hours, Z minutes")
   - If in the past: show elapsed time ("X days, Y hours, Z minutes ago")
2. **Links notation**: Always enclose dates as links like `(2026-01-27 09:33:00 +00:00)` in links notation

## Proposed Solution

### 1. Show time difference for standalone datetime input
When the expression is a single DateTime (not part of an arithmetic operation):
- Calculate the difference between the input datetime and current UTC time
- Show both the parsed datetime AND the duration from now in the steps
- The result remains the datetime, but steps include "Time from now: X days, Y hours ago" or "Time until: X days, Y hours"

### 2. Update links notation for DateTime
In `Expression::to_lino()`, wrap DateTime values in parentheses: `(datetime_string)`.

## Implementation Approach

Since the frontend handles dynamic ticking (as mentioned in the issue: "interactive ticking onwards"), the backend should:
1. Return the parsed DateTime value as the result
2. Include the current difference in the steps for initial display
3. The frontend can use the DateTime value to compute live updates

## Related Concepts

- Time duration formatting: days, hours, minutes, seconds
- Relative time display (e.g., "2 days ago", "in 3 hours")
- Live/ticking countdowns require frontend JavaScript implementation

## Data Files

- Input expression: `Jan 27, 9:33am UTC`
- Parsed result: `2026-01-27 09:33:00 +00:00`
- Environment: Version 0.1.0, Safari on macOS
