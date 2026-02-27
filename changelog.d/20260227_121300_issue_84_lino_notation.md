---
bump: minor
---

### Fixed
- Function calls now render in proper links notation: `integrate(x^2, x, 0, 3)` → `(integrate ((x ^ 2) x 0 3))` instead of keeping mathematical notation
- Power expressions now wrap in parentheses with spaces: `2^3` → `(2 ^ 3)` instead of `2^3`
- All compound expressions are now wrapped in outer `()` in links notation for consistency
- Zero-argument functions render as `(pi)` instead of bare `pi`
- Indefinite integrals use proper lino in symbolic results: `(integrate (x ^ 2) dx)`

### Added
- Alternative interpretation support for ambiguous expressions
  - Expressions with mixed operator precedence show alternative groupings (e.g., `2 + 3 * 4` shows both `(2 + (3 * 4))` and `((2 + 3) * 4)`)
  - Function calls show both links notation and traditional mathematical notation
- UI allows clicking between alternative interpretations with visual selection indicator
- New examples for expressions with multiple interpretations
