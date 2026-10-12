---
title: "Complete the closed-source calculator documentation corpus"
---
Part of #227.

## Problem

`docs/case-studies/issue-227/corpus/closed-source-docs.tsv` has 343 rows: 266 verbatim examples from the Soulver 5 documentation (fetched 2026-10-07 from `documentation.soulver.app/*.md`) and only 77 hand-picked examples for Numi, Wolfram|Alpha, Google, Calca, Raycast, Apple Math Notes and Frink. Several documented products are missing entirely (Parsify, Numi full wiki, Calca manual, Hurmet manual, SpeedCrunch book, Windows Calculator, Samsung/Xiaomi calculators, DuckDuckGo instant answers).

## Acceptance criteria

- Every product in the inventory table of `docs/case-studies/issue-227/competitor-inventory.md` has at least every documented example of its syntax reference in the corpus, with the page URL in the note column and the fetch date in the file header.
- Date-, currency- and locale-dependent examples keep an empty expected value (meaning "must evaluate") so they are not marked `different` for the wrong reason.
- An extraction script per documentation source lives in `experiments/issue-227/` (the Soulver one already exists) so the corpus can be re-fetched.
- Fair-use scope: examples are short expression/answer pairs with attribution; no prose is copied.

## Solution options

1. Extend `experiments/issue-227/extract-soulver-examples.mjs` into a generic "expression | answer" extractor for Markdown documentation and apply it to each product. Recommended.
2. Hand-curate. Slower and loses provenance.

## Blockers

None.
