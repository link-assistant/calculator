---
bump: minor
---

### Added

- Spelled numbers and operator words as a data-driven front end (issue #222):
  `data/words/arithmetic-words.lino` lists number words, scale words and
  operator phrases for English, Russian, Hindi, Chinese and Spanish, so
  `two plus two`, `шесть умножить на семь`, `दस बटा दो`, `二十三加五` and
  `10 dividido por 2` evaluate directly. Compound numbers (`one hundred
  twenty-three`, `сто двадцать три`, `三百二十`) are composed exactly.
  Adding a language only needs new lines in the data file.
- `≠`, `≤` and `≥` work as comparison operators.

### Changed

- The English operator words (`plus`, `times`, `multiplied by`, …) moved
  from the lexer into the same data file. All-uppercase currency codes such as
  `ONE` or `DOS` keep their currency meaning.
