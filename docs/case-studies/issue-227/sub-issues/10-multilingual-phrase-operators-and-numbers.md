---
title: "Multilingual phrase operators, spelled numbers and CJK tokenization"
---
Part of #227. Requirement: scientific sentences in many languages must compute. Current coverage per language: en 21.6%, de 7.1%, fr 7.7%, zh 7.7%, ru 5.3%, and 0% for es, hi, it, ja, ko, pt, tr, ar.

## Evidence

| Expression | Lang | Expected | Actual |
| --- | --- | --- | --- |
| `3的平方加4的平方` | zh | 25 | `3 的平方加4的平方` (CJK run becomes a unit) |
| `3の二乗足す4の二乗` | ja | 25 | `3 の二乗足す4の二乗` |
| `2の10乗` | ja | 1024 | `2 の10乗` |
| `十の三乗` | ja | 1000 | `10 の三乗` |
| `700的百分之三十` | zh | 210 | `700 的百分之三十` |
| `30度の正弦` | ja | 0.5 | `30 度の正弦` |
| `700 увеличить на 30 процентов` | ru | 910 | `700 увеличить` |
| `2 ампера умножить на 5 ом` | ru | 10 В | `10 ампера` |
| `2 amperios por 5 ohmios` | es | 10 V | `10 amperios` |
| `3 लाख किमी/सेकंड` | hi | 300000 km/s | `Unexpected trailing input` |
| Arabic, Korean, Turkish, Portuguese, Italian rows | | | 0 of 24 pass |

`data/words/arithmetic-words.lino` covers cardinal numbers and a few operator words for en, ru, hi, zh, es only; the lexer has no CJK word segmentation, so any CJK run after a number is absorbed as a unit.

## Acceptance criteria

- Spelled cardinals, ordinals and decimals for at least en, de, fr, es, pt, it, ru, uk, hi, zh, ja, ko, ar, tr, nl, pl (text2num 2.8.0, MIT, is the reference list and ships test data for en, da, nl, es, fr, de, it, pt, ru, ca; Duckling covers 49 languages).
- Operator and function phrases for the same languages (`plus`, `mal`, `fois`, `умножить на`, `加`, `足す`, `의 제곱`, `kare`), percent phrases, and root/power phrases, all loaded from LINO data.
- CJK tokenization: digits followed by CJK characters are split at vocabulary boundaries using longest-match over the LINO phrase table (no external segmenter needed because the vocabulary is closed).
- The per-language table in the coverage report shows at least 80% for every seeded language on the theorem and paper-phrasing groups.

## Solution options

1. Grow `arithmetic-words.lino` with a `phrase` link type per language and implement longest-match segmentation in `src/grammar/arithmetic_words.rs`. Recommended; zero new dependencies and WASM-safe.
2. Port text2num's per-language grammar tables (MIT) into LINO data with a generator script in `experiments/`. Good source for numbers, but it has no operator phrases.
3. Use ICU RBNF spell-out rules from CLDR (`icu` 2.3.1) in reverse. Rejected: RBNF formats numbers to words; parsing words back is not provided.

## Blockers

- Blocked by: #237 (English phrase grammar defines the structure), #233 (locale decimal/grouping conventions per language), #236 (scale words such as `万`, `лакх`, `Mrd.`).
