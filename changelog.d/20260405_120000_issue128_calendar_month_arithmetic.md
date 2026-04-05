---
bump: patch
---

### Fixed
- Fixed date arithmetic for months and years to use proper calendar semantics instead of
  fixed-second approximations (e.g. `17 February 2027 - 6 months` now correctly returns
  `2026-08-17` instead of `2026-08-21`). The day-of-month is now preserved, with clamping
  to the last day of the month when needed (e.g. `31 January + 1 month = 28 February`).
- Fixed duration unit display to use full English words (`months`, `years`, `days`, etc.)
  instead of abbreviations (`mo`, `y`, `d`, etc.) in calculation steps and lino notation.
