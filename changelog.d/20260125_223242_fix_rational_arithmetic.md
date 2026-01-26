---
bump: minor
---

### Added
- New Rational type for exact fractional arithmetic using `num-rational` crate
- Repeating decimal detection algorithm for proper display of fractions
- Extended ValueKind enum with Rational variant for symbolic computation

### Fixed
- Expression `(1/3)*3` now correctly returns `1` instead of `0.9999999999999999...`
- All fractional expressions like `(2/3)*3`, `(1/6)*6`, `(1/7)*7` now return exact results
- Reduced excessive parentheses in links notation output
