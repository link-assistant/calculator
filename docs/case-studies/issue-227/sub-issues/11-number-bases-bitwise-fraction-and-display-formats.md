---
title: "Number bases, bitwise operators, repeating decimals and display formats (fraction, roman, words, eng, hex)"
---
Part of #227.

## Evidence

| Expression | Source | Expected | Actual |
| --- | --- | --- | --- |
| `0xff` | Rink | 255 | **`0 XFF`** |
| `0b101010` | Rink | 42 | **`0 b101010`** |
| `0b1010 + 0xA` | Calca | 20 | `Unit mismatch: cannot add 'b1010' and 'XA'` |
| `0xff to bin` | Numi | 0b11111111 | `No exchange rate available` |
| `0xffffffffi32` | math.js | -1 | `0 xffffffffi32` |
| `1 to roman` | fend | I | `Cannot convert to ROMAN` (30 rows) |
| `1 2/3 to fraction` | fend | 5/3 | `Unexpected trailing input '2'` |
| `1/7 to fraction` | Rink | 1/7 | `Unknown unit 'fraction'` |
| `0.(3) to float` | fend | 0.(3) | `Unexpected character '.'` |
| `1/3 to 2 dp` | Soulver | 0.33 | `Expected a unit name after 'to'` |
| `5 & 3`, `5 | 3`, `~5`, `1 << 4`, `5 xor 3` | fend/Qalculate!/Soulver | 1, 7, -6, 16, 6 | `Unexpected character` (`&` 24 rows, `|` 34, `~` 40) |
| `to words`, `to eng`, `to hex`, `to float`, `to bool` | fend | | `Cannot convert` (18 + 17 + 4 + 8 + 3 rows) |

The `to <unit>` conversion is routed through the exchange-rate/unit converter, so every display format word is reported as a missing unit.

## Acceptance criteria

- Literals `0x`, `0b`, `0o`, `0d`, fend's `6#12` base prefix, Qalculate!'s `base 16` suffix, and `0.(3)` repeating decimals parse to exact rationals.
- Bitwise `& | xor ^^ ~ << >>` and words `and or xor not` on integers; `and`/`or` on booleans are delegated to #242.
- `to fraction`, `to mixed`, `to float`, `to decimal`, `to roman`, `to words`, `to eng`, `to sci`, `to hex/bin/oct/base N`, `to N dp/sf` are display formats handled before unit conversion and also exposed as output formats in the WASM result object.
- Mixed numbers `1 2/3` parse when the context is a fraction (fend rule).

## Solution options

1. A `Format` enum in the expression tree with `to <format>` parsed before unit conversion; exact rational display via existing `num-rational`. Recommended.
2. String post-processing in the CLI. Rejected: the WASM/React consumer needs the same formats.

## Blockers

- Blocked by: #229.
