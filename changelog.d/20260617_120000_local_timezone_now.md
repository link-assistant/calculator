---
bump: minor
---

### Fixed
- `now-12:30` and similar `DateTime - DateTime` subtractions no longer collapse a
  negative difference to `0 seconds`; the signed duration is now returned
  (issue #185).

### Added
- The web app now interprets `now` and timezone-less times (e.g. `12:30`) in the
  user's local timezone by default, so `now-12:30` matches the wall clock.
  Times with an explicit timezone (e.g. `12:30 UTC`) are still honored as given.
- New `Calculator::set_timezone_offset(offset_minutes)` and
  `Calculator::clear_timezone_offset()` WASM/library methods to control the
  local timezone offset used when resolving `now` and bare times.
