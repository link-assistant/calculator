---
bump: minor
---

### Added
- Arbitrary-precision integer arithmetic using BigInt-based Rational numbers
- Exact computation for integer exponentiation (e.g., `10^100` now returns all 101 digits exactly)
- Numbers beyond the i128 range (~1.7×10^38) are now representable without loss of precision
