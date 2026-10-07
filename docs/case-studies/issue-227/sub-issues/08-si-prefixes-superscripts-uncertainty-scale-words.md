---
title: "Scientific notation as written in papers: superscripts, SI prefixes, ± uncertainty, scale words after digits"
---
Part of #227. Scientific prose writes numbers in forms the lexer rejects outright.

## Evidence

| Expression | Lang | Source | Expected | Actual |
| --- | --- | --- | --- | --- |
| `3×10⁸ м/с` | ru | Wikipedia | 300000000 м/с | `Unexpected trailing input 'м'` (superscript exponent not attached to 10) |
| `π²/6` | en | Basel problem | 1.644934 | `Unexpected identifier: π²` |
| `1⁄299792458 second` | en | Wikipedia | 3.3356e-9 s | `Unexpected character '⁄'` |
| `1/299792458秒` | ja | Wikipedia | 3.3356e-9 s | 3.33564e-9 (unit dropped) |
| `12.3 ± 4.5` | en | paper phrasing | 12.3 ± 4.5 | `Unexpected character '±'` |
| `3 billion` | en | paper phrasing | 3000000000 | `Unexpected trailing input 'billion'` |
| `1,08 Mrd. km/h` | de | Wikipedia | 1.08e9 km/h | `Unexpected trailing input ','` |
| `3 लाख किमी/सेकंड` | hi | Wikipedia | 300000 km/s | `Unexpected trailing input 'किमी'` |
| `three orders of magnitude larger than 5` | en | paper phrasing | 5000 | `Unexpected trailing input 'of'` |
| `a 2.5-fold increase over 40` | en | paper phrasing | 100 | `Unexpected trailing input '2.5'` |
| `nano` | en | fend | 0.000000001 | `Unexpected identifier: nano` |
| `¼`, `‰`, `½ of 10` | en | fend/paper | 0.25, 0.001, 5 | `Unexpected character` |

Superscript digits are already accepted as powers after a number (`2³`), but not after `10` in `×10⁸`, after identifiers (`π²`, `m²`) or with a minus sign (`s⁻¹`, `s−1`).

## Acceptance criteria

- Superscript exponents with optional `⁻`/`−` sign bind to the preceding number, constant, parenthesis or unit.
- `×10ⁿ`, `x10^n`, `·10ⁿ`, `e±n`, `E±n` and `10^n` are equivalent scientific notation.
- Scale words after a number (`3 billion`, `2 million`, `1,08 Mrd.`, `3 लाख`, `5 crore`, `2 тыс.`, `3万`) multiply the number; they are loaded from `data/words/arithmetic-words.lino` (`scale` entries already exist for word numbers).
- `a ± b` evaluates to an interval or uncertainty value; `(12.3 ± 4.5) * 2` prints `24.6 ± 9`. Qalculate! interval arithmetic is the reference.
- Vulgar fractions `¼ ½ ¾ ⅓ ⅔ ⅛`, the fraction slash `⁄`, and `‰`/`‱` are tokens.
- `k-fold`, `n orders of magnitude`, `n times larger/smaller than`, `half/third/quarter of` are phrase operators (coordinate with #237 for the English phrase grammar).

## Solution options

1. Lexer-level Unicode normalization table plus a small `scale`/`fold` grammar in `src/grammar/`, with vocabulary in the LINO words file. Recommended.
2. Pre-normalize with `unicode-normalization` NFKC. Rejected on its own: NFKC turns `²` into `2`, which would change `m²` into `m2`.

## Blockers

- Blocked by: #233 (locale separators: `1,08 Mrd.` and `12,3 ± 4,5`), #235 (unit tokens for `m⋅s−1`, `км/ч`, `किमी/सेकंड`).
