---
bump: minor
---

### Added
- Added multilingual mass unit aliases for all 7 app languages (Russian, Chinese, Hindi, Arabic, German, French) — e.g., "кг" (kg), "г" (g), "тонна" (ton), "公斤" (kg), "किलोग्राम" (kg), "كيلوغرام" (kg), and many more including all grammatical forms
- Added Unicode combining mark support in the lexer for scripts like Devanagari (Hindi) and Arabic where diacritical marks are integral parts of words

### Fixed
- Fixed compilation error in `number_grammar.rs` where `DurationUnit::parse` returned wrong type
