---
bump: minor
---

### Added
- Support `now` keyword in expressions, e.g. `(Jan 27, 8:59am UTC) - (now UTC)` (issue #34)
- Support `until <datetime>` syntax for countdown durations, e.g. `until Jan 27, 11:59pm UTC` (issue #23)
- Parse timezone abbreviations (EST, PST, CET, etc.) in datetime expressions (issue #23)
- Parse ordinal date suffixes (1st, 2nd, 3rd, 26th) and strip day names (Monday, Tuesday, etc.) (issue #23)
- Support `UTC time`, `time UTC`, `current time`, and `current UTC time` inputs (issue #68)
- Show "Time since" / "Time until" elapsed/remaining duration for standalone datetime inputs (issue #45)
- Wrap standalone datetime values in parentheses in links notation (issue #45)
