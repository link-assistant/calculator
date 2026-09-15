---
bump: minor
---

### Added

- Support natural-language arithmetic aliases such as `plus`, `times`, `multiplied by`, `divided by`, and `mod`.
- Support shared open-calculator and scientific notation: decimal exponents,
  digit separators, Unicode operators/roots/superscripts, implicit
  multiplication, parenthesis-free unary functions, and natural root/power
  phrases.
- Add a primary-source competitor compatibility audit and executable example corpus.

### Fixed

- Parse numbers grouped with common typographic spaces, including narrow no-break space, while rejecting malformed digit groups.
- Parse time-first date expressions with a trailing timezone and apply timezone offsets exactly once in every supported word order.
