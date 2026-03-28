### Added
- Timezone conversion support: expressions like "6 PM GMT as MSK" now convert times between named timezones
- 80+ timezone abbreviations supported (MSK, ICT, WITA, BRT, EAT, NZST, and many more)
- Half-hour and 45-minute timezone offsets: IST (+5:30), NPT (+5:45), ACST (+9:30)
- Time expressions without colon now recognized: "6 PM", "6 PM GMT", "9 AM PST"

### Fixed
- Time expressions like "6 PM GMT" previously dropped the timezone and returned just "6 PM"
- "GTM" typo now handled as "GMT" (Greenwich Mean Time)
- Timezone abbreviation names now displayed in results (e.g., "MSK") instead of raw offsets (e.g., "+03:00")
