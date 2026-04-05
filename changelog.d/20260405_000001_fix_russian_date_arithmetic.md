---
bump: minor
---

### Fixed
- Fixed parsing of Russian-language date arithmetic expressions like `17 февраля 2027 - 6 месяцев` (issue #125)
- Added Russian month names (all grammatical cases) to datetime recognition and parsing
- Added Russian duration unit names to all grammatical cases: месяц/месяца/месяцев (months), день/дня/дней (days), неделя/недели/недель (weeks), час/часа/часов (hours), год/года/лет (years)
- Fixed `try_parse_datetime_from_tokens` to try progressively shorter token prefixes when datetime parsing fails, enabling better backtracking
- Added support for date formats without commas: `Feb 17 2027` and `17 February 2027`
