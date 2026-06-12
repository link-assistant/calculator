---
bump: minor
---

### Added
- Support `?` and operand-position `*` as single-unknown placeholders in
  supported linear equations, with structured equation-solution results and
  derivation steps.
- Support symbolic solutions for linear equations with multiple variables, such
  as `x + y = 10` -> `x = 10 - y`, including detailed step-by-step derivations.
- Support exact real rational roots for single-variable polynomial equations
  with nonnegative integer powers, including quadratic and higher-power cases
  such as `x^2 = 4` -> `x = -2 or x = 2`.
