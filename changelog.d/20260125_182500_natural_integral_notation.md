---
bump: minor
---

### Added
- Natural integral notation support: `integrate sin(x)/x dx` now parses correctly
- Symbolic integration results for common functions (sin, cos, exp, polynomials, sin(x)/x -> Si(x))
- LaTeX formula rendering using KaTeX for mathematical expressions
- Canvas-based function plotting for integral visualizations
- New `IndefiniteIntegral` expression type for symbolic integrals
- New `MathRenderer` and `FunctionPlot` React components

### Changed
- Examples in the calculator UI now include `integrate sin(x)/x dx`
- `CalculationResult` extended with `latex_input`, `latex_result`, `is_symbolic`, and `plot_data` fields

### Fixed
- Issue #3: "integrate sin(x)/x dx" no longer returns "Parse error: Unexpected identifier: integrate"
