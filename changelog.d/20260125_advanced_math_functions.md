---
bump: minor
---

### Added
- Native support for advanced mathematical functions computed in Rust/WebAssembly:
  - Trigonometric functions: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sinh`, `cosh`, `tanh`
  - Logarithmic functions: `ln`, `log`, `log2`, `log10`, `exp`
  - Math functions: `sqrt`, `cbrt`, `abs`, `floor`, `ceil`, `round`, `pow`, `factorial`
  - Numerical integration: `integrate(expr, var, lower, upper)` using Simpson's rule
  - Mathematical constants: `pi()`, `e()`
  - Angle conversion: `deg()`, `rad()`
  - Min/max functions with variable arguments
  - Power operator `^` for exponentiation
- Domain error handling for invalid inputs (e.g., `sqrt(-1)`, `ln(-1)`)
- Unknown function error messages for unsupported function names
