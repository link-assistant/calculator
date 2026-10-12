---
title: "English noun-phrase operators: squared, cubed, sum of, half of, dozen, rounded, log base"
---
Part of #227.

## Evidence

| Expression | Source | Expected | Actual |
| --- | --- | --- | --- |
| `3 squared plus 4 squared` | theorem in words | 25 | **`7 squared`** (treated as a custom unit) |
| `4 cubed` | theorem in words | 64 | **`4 CUBED`** |
| `one dozen` | fend | 12 | **`1 DOZEN`** |
| `log 20 base 4` | Soulver | 2.1609640474 | **5.204** (`base 4` ignored) |
| `5.5 rounded` | Soulver | 6 | `5.5 rounded` |
| `5.5 rounded down` / `rounded up` | Soulver | 5 / 6 | `Unexpected trailing input 'down'` |
| `half of 175` | Soulver | 87.5 | `Unexpected identifier: half` |
| `half of three quarters` | paper phrasing | 0.375 | `Unexpected identifier: half` |
| `gcd of 20 and 30` | Soulver | 10 | `Unexpected identifier: gcd` |
| `average of 3, 8, 13` | Wolfram\|Alpha | 8 | `Unexpected identifier: average` |
| `dozen` | fend | 12 | `Unexpected identifier: dozen` |
| `sum of 1, 2 and 3` | Soulver | 6 | unsupported |

Silent custom-unit absorption (`7 squared`) is the dangerous case: a word that is not in the vocabulary becomes a unit name and the arithmetic proceeds without error.

## Acceptance criteria

- Postfix power words (`squared`, `cubed`, `to the power of`, `to the nth`), prefix function phrases (`square root of`, `cube root of`, `half of`, `a third of`, `quarter of`, `double`, `triple`, `average/mean/median of`, `sum of`, `gcd/lcm of`), multi-value lists with `and`/`,`, and rounding phrases (`rounded`, `rounded up/down`, `rounded to 2 dp/places`) parse.
- `log x base b`, `log base b of x`, `ln of` parse; the `log` default base stays documented (base 10 here, natural log in Numbat and Parsify; the coverage report lists the 7 affected Numbat rows as an accepted `different`).
- Count nouns (`dozen`, `pair`, `score`, `gross`, `half dozen`) and SI prefix words (`nano`, `kilo`, `mega`) evaluate to numbers.
- A word that is not in any vocabulary and not an SI/unit alias must produce an error, never a silent custom unit, unless the expression is an explicit unit definition (document the custom-unit rule).

## Solution options

1. Extend the LINO vocabulary with `(phrase en <op> '<phrase>')` entries and a grammar for postfix/prefix phrase operators in `src/grammar/arithmetic_words.rs`. Recommended; it also gives #238 the structure to translate.
2. Hard-code English phrases in the parser. Rejected: the issue requires multilingual support.

## Blockers

- Blocked by: #229.
