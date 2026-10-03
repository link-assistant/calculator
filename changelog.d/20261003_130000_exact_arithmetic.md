---
bump: minor
---

### Fixed
- `n!` / `factorial(n)` is computed exactly with big integers (`30!` used to return `0`, `25!` was rounded) (#222).
- Function results outside the decimal range (e.g. `exp(70)`) now report an overflow error instead of silently returning `0`.
- Unary minus binds looser than `^`: `-2^2` is `-4` (was `4`), matching standard notation; `2^-1` and `(-2)^2` are unchanged.

### Added
- `evaluate_function_values` evaluates a function on `Value` arguments, using exact arithmetic where available.
