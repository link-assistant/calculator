# Multilingual scientific-quote corpus

File: [corpus/natural-language-quotes.tsv](corpus/natural-language-quotes.tsv),
162 rows, 13 languages. Built on 2026-10-07. Every row is one expression that
a reader could copy from scientific prose and expect a calculator to evaluate.

## Groups

| Group | Source column | Rows | What the rows contain | Current result |
| --- | --- | ---: | --- | --- |
| A | `Wikipedia quote` | 26 | Verbatim numeric fragments from the "Speed of light" article in en, ru, de, fr, es, it, pt, zh, ja, ko, hi, ar, tr (CC BY-SA 4.0, page URL in the note column). Examples: `299792458 m⋅s−1`, `1⁄299792458 second`, `3×10⁸ м/с`, `1,08 Mrd. km/h`, `3 लाख किमी/सेकंड`, `1/299792458秒`. | 0 supported, 1 different, 25 unsupported |
| B | `Wikipedia quote (derived)` | 10 | Group A fragments followed by a conversion request in the same language (`… в метрах в секунду`, `… को मीटर प्रति सेकंड में`). | 0 supported |
| C | `Theorem in words` | 40 | Pythagoras, Kepler's third law, the Basel problem, mass-energy equivalence and Ohm's law stated with numbers in words (en, ru, de, fr, es, zh, ja, pt, it, hi, ar, ko, tr). Examples: `3 squared plus 4 squared`, `3の二乗足す4の二乗`, `2 ампера умножить на 5 ом`, `π²/6`. | 0 supported, 11 different, 29 unsupported |
| D | `Paper phrasing` | 86 | Constructs common in papers: percentages (`56.7% of 1,234 participants`, `700 увеличить на 30 процентов`), fold change, orders of magnitude, uncertainty (`12.3 ± 4.5`), temperature and SI prefix conversion (`25 °C in K`, `5 µm in nm`), superscripts, vulgar fractions, spelled fractions and decimals, multilingual number words. | 10 supported, 8 different, 68 unsupported |

Per-language totals: en 49, ru 19, de 14, fr 13, es 13, zh 13, ja 9, pt 8,
hi 8, ar 7, it 5, ko 2, tr 2.

## Why these rows fail today

| Cause | Example | Sub-issue |
| --- | --- | --- |
| No length/speed/temperature/electrical units; `m` is milli or minutes | `299792458 m⋅s−1` → `undefined variable: m`; `2 amperes times 5 ohms` → `10 amperes` | dimensional unit algebra |
| Superscripts and scale words not attached to numbers | `3×10⁸ м/с`, `π²/6`, `3 billion`, `1,08 Mrd.` | scientific notation |
| Decimal commas and thousands separators | `12,3 ± 4,5`, `56.7% of 1,234 participants` → `0.699678` | thousands separators |
| Unknown English words become units | `3 squared plus 4 squared` → `7 squared` | English phrase operators |
| CJK runs after a digit become a unit; no vocabulary for ja/de/fr/pt/it/ar/ko/tr | `3的平方加4的平方` → `3 的平方加4的平方` | multilingual phrases |
| Percent phrases | `700 увеличить на 30 процентов` → `700 увеличить` | percent semantics |
| `±`, `⁄`, `¼`, `‰` characters | `Unexpected character` | scientific notation, LaTeX/Unicode input |

## Corpus conventions

- Columns: `source`, `lang` (ISO 639-1), `expression`, `expected`, `note`.
  Tabs, newlines and backslashes inside a column are escaped as `\t`, `\n`, `\\`.
- An empty `expected` means "must evaluate without error"; used for rows whose
  answer depends on the current date, live rates or display precision.
- `note` carries provenance: the Wikipedia page URL, the theorem name, or the
  phrasing category.
- Expected values are written in the display form competitors use (`300000000 m/s`,
  `89875517873681764 J`), and the runner compares numbers at the expected
  precision and the unit text loosely (case and spacing insensitive).

## How to extend

1. Add rows to the TSV (or a new TSV in `corpus/`; every `*.tsv` file in the
   directory is loaded).
2. Run `node scripts/competitor-coverage.mjs` and commit the regenerated
   `results/coverage-report.md` and `results/coverage-baseline.json`.
3. Cite the source in the `note` column; prefer CC-BY/CC-BY-SA or public-domain
   texts (Wikipedia, arXiv abstracts under CC BY, CODATA, NIST SP 811).
