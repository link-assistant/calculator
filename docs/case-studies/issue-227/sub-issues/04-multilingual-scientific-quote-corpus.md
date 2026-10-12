---
title: "Grow the multilingual scientific-quote corpus beyond the 13 seeded languages"
---
Part of #227. Requirement: "quotes from scientific papers in natural language in different languages" must be computable.

## Problem

`docs/case-studies/issue-227/corpus/natural-language-quotes.tsv` seeds 162 rows in 13 languages (en 49, ru 19, de 14, fr 13, es 13, zh 13, ja 9, pt 8, hi 8, ar 7, it 5, ko 2, tr 2). Group A takes verbatim sentences from Wikipedia "speed of light" articles (CC BY-SA 4.0, cited per row), groups B-D are derived conversions, theorems stated in words and paper phrasing (percent, fold, orders of magnitude, uncertainty). Current coverage of these rows is 0% for Wikipedia quotes and theorems and 11.6% for paper phrasing. The corpus is too small to be representative: no two rows per language cover the same construct, and open-access papers themselves are not sampled.

## Acceptance criteria

- At least 50 rows per language for the 13 seeded languages plus nl, pl, uk, sv, fa, id, vi, th, he, bn (the Duckling and Recognizers-Text language sets), each row citing a CC-BY or public-domain source (arXiv abstracts under CC BY, Wikipedia, CODATA tables, NIST SP 811).
- Each construct in the table below has at least one row per language: number with thousands/decimal separators, SI unit with prefix, scientific notation, uncertainty, percentage change, fold change, order of magnitude, ratio in words, named constant, power in words, root in words, unit conversion request.
- A `lang` and `note` column convention is documented in `natural-language-corpus.md`, and the coverage report keeps the per-language table.

## Solution options

1. Semi-automatic: fetch Wikipedia REST summaries and arXiv abstracts per language (script in `experiments/issue-227/`), extract sentences containing a number and a unit with a regex over Unicode digits and SI symbols, then review manually. Recommended.
2. Hand-write all rows. Not scalable to 23 languages.

## Blockers

None.
