### Fixed
- Fixed error "Cannot add duration and datetime" when adding a duration to a datetime (issue #8)
  - The expression `(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC) + (Jan 25, 12:51pm UTC)` now works correctly
  - Addition of Duration + DateTime is now supported (previously only DateTime + Duration worked)
