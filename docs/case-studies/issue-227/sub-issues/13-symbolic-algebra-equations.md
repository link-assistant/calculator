---
title: "Symbolic algebra: variables in expressions, non-polynomial equations, calculus and simplification"
---
Part of #227.

## Evidence

- 143 Qalculate! rows fail with `undefined variable: x` (`solver.batch`, `polynomial.batch`, `calculus.batch`, `limits.batch`): expressions such as `(x+1)^2`, `diff(x^3)`, `integrate(x^2)`, `solve(x^2 = 4)`.
- 22 rows fail with "only polynomial equations with one unknown are supported" and 3 with "polynomial equation has no supported form".
- `tests/issue_218_competitor_compatibility_tests.rs` already locks the current linear and polynomial solver behaviour.

## Acceptance criteria

- Free symbols are first-class values: `(x+1)^2` prints `x^2 + 2x + 1`, `2x + 3x` prints `5x`.
- `solve`, `diff`/`derivative`, `integrate` with and without bounds, `limit`, `factor`, `expand`, `simplify` on univariate expressions; systems of linear equations.
- Numeric fallback (bisection/Newton) for non-polynomial equations with one unknown, with a documented precision.
- Every Qalculate! `solver.batch` and `polynomial.batch` row passes or is listed as an accepted `different` with a reason.

## Solution options

1. Extend the expression tree with symbolic rewrite rules (polynomial canonical form over `num-rational`, product/sum normalization, rule-based differentiation) as the issue-220 audit planned. Recommended; keeps LINO output and `no_std` goals.
2. Bind a CAS: `symbolica` (commercial license), Mathics (Python, GPL, not WASM), `libqalculate` (C++, GPL, no WASM build). All rejected on licensing or target platform.

## Blockers

- Blocked by: #240 (complex values and matrices are needed for solution sets).
