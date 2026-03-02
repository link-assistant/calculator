---
bump: minor
---

### Added
- Enhanced display format for time expressions like "UTC time", "current UTC time", "EST time": now shows as `('current UTC time': 2026-03-02 20:40:13 UTC (+00:00))` with outer parentheses, timezone name, and offset
- Support for all known timezone abbreviations in "X time" and "time X" patterns (e.g., "EST time", "PST time", "JST time")
- `is_live_time` flag in `CalculationResult` to indicate expressions that represent the current time and should auto-refresh
- Reactive auto-refresh in the web frontend: time expressions update every second when `is_live_time` is true
