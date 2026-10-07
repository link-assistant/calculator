---
title: "Percent arithmetic differs from every competitor"
---
Part of #227.

## Evidence

Every surveyed engine (Soulver, Numi, Qalculate!, fend, Calca, Raycast, Google) treats `x + p%` as "increase x by p percent"; this project adds the fraction.

| Expression | Source | Expected | Actual |
| --- | --- | --- | --- |
| `200 + 10%` | Soulver | 220 | 200.1 |
| `200 - 10%` | Soulver | 180 | 199.9 |
| `6 - 50%` | Numi | 3 | 5.5 |
| `100 + 10% + 10%` | Qalculate! | 121 | 100.2 |
| `10% + 20%` | Soulver | 30% | 0.3 |
| `5% + 1` | fend | 105% | 1.05 |
| `20/200 %` | Soulver | 10% | 10 |
| `100% + 2 + 30%` | Soulver | 330% | 3.3 |
| `180 is what % off 200` | Soulver | 10% | `Unexpected trailing input 'what'` |
| `50 to 75 is what %` | Soulver | 50% | `Expected a unit name after 'to'` |
| `120% 2` | Rink | 2.4 | 0 |
| `700 увеличить на 30 процентов` (ru) | paper phrasing | 910 | `700 увеличить` |

The Soulver percentages page (`syntax-reference/percentages`) documents the full set: `% of`, `% on`, `% off`, `x + y%`, `x as a % of y`, `x is what % of y`, `x to y is what %`, `% change`.

## Acceptance criteria

- The 32 `different` and 20 `unsupported` percent rows in the current report pass.
- Percent values keep a percent display type (`10% + 20%` prints `30%`).
- Existing `% of` behaviour and the `tests/issue_218_competitor_compatibility_tests.rs` expectations stay green, except where this issue explicitly changes them (list them in the PR).
- Multilingual phrase forms (`увеличить на`, `百分之`, `パーセント`) are added through `data/words/arithmetic-words.lino`, not code.

## Solution options

1. Introduce a `Percent` value kind in `ValueKind`; binary `+`/`-` with a percent right operand and a non-percent left operand scales the left operand; percent-with-percent adds points. This matches Soulver and Qalculate!. Recommended.
2. Keep fractions and special-case the display. Rejected: produces `100 + 10% + 10% = 100.2`.

## Blockers

- Blocked by: #229.
