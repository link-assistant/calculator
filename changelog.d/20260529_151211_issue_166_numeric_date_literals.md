---
bump: minor
---

### Added
- Recognize numeric date literals inside expressions, e.g. `15.10.2025 + 180 days`
  and `15/10/2025 + 180 days`. Dot- (`DD.MM.YYYY`, `YYYY.MM.DD`), slash-, and
  dash-separated dates in day-first, month-first, and ISO orderings are now
  parsed as dates instead of arithmetic (issue #166).

### Fixed
- Stop rejecting `15.10.2025 + 180 days` with "Unexpected trailing input
  '.2025'" and stop silently evaluating numeric dates such as `15/10/2025` and
  `2021-02-08` as division/subtraction. A strict 4-digit-year + valid-calendar
  guard keeps ordinary arithmetic (`2026 - 1 - 22`, `1/3 * 3`, `10/2`,
  `3.14 + 2.86`) untouched.
