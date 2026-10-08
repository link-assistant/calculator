---
title: "Complex numbers, vectors and matrices, infinity/NaN and the missing special functions"
---
Part of #227.

## Evidence

| Expression | Source | Expected | Actual |
| --- | --- | --- | --- |
| `i^2`, `2 + 3i`, `sqrt(-1)` | fend/Numbat/Qalculate! | -1, 2 + 3i, i or NaN | `undefined variable: i` (43 rows) |
| `infinity`, `inf`, `-infinity + 1` | fend/Numbat | ∞ | `Unexpected identifier: infinity` (78 + 3 rows) |
| `NaN`, `NaN < NaN` | Numbat | NaN, false | `Unexpected identifier` (23 rows) |
| `[1 2; 4 5] * 2` | Qalculate! | [2 4; 8 10] | `Unexpected character '['` (104 rows) |
| `fib 10`, `fac(5)`, `gcd(36, 48)`, `atanh 0`, `asinh`, `acosh`, `cis`, `identity`, `matrix`, `vector`, `hex` | fend/Rink/Wolfram\|Alpha | | `Unknown function` / `Unexpected identifier` (12 + 11 + 5 + 5 + 3 + 3 + 4 + 3 + 2 + 2 + 2 rows) |
| `mod(-1, 4)` | Numbat | 3 | **-1** (sign convention) |
| `3 mod -2` | Qalculate! | -1 | **-1.97** |
| `round(3.14159, 2)` | Numi | 3.14 | `round` only accepts 1 argument |
| `min(1,2,3)` | Qalculate! | 1 | `expected at most 2 arguments` (10 rows) |

## Acceptance criteria

- `ValueKind` gains `Complex` (exact rational parts), `Infinity`/`NaN`, and `Matrix`/`Vector`; the display, LINO, serde and operation matrices are tested per kind, one kind per PR.
- Functions: `fib`, `fac`/`factorial`, `gcd`, `lcm`, `atanh`, `asinh`, `acosh`, `cis`, `identity`, `det`, `transpose`, variadic `min`/`max`/`sum`/`mean`/`median`, two-argument `round`, `floor`/`ceil` with units.
- `mod` follows the floored convention for `mod(a, b)` and documents the `%` convention (Numbat and Qalculate! differ; both are in the corpus).

## Solution options

1. Native implementation on top of `num-rational`/`num-bigint`; matrices as `Vec<Vec<Value>>` with dimension checks. Recommended.
2. Embed a CAS. Not needed for value types; see #241.

## Blockers

- Blocked by: #229, #239 (bitwise/base literal lexing shares the numeric token changes).
