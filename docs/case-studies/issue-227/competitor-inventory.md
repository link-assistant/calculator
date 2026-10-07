# Competitor inventory for issue #227

Checked on 2026-10-07. Open-source projects were cloned and their published
test suites were imported; closed-source products were sampled from their
documentation. The issue-220 [competitor audit](../issue-220/competitor-audit.md)
describes each product's published feature surface; this file records exactly
which artifacts were turned into executable corpus rows and what the current
build does with them. Coverage numbers come from
[results/coverage-report.md](results/coverage-report.md).

## Open-source engines (tests and code)

| Project | Language, license | Revision used | Imported artifact | Rows | Supported | Different | Unsupported |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: |
| [fend](https://github.com/printfn/fend) | Rust, MIT | d9cb764 (2026-08-28) | `core/tests/integration_tests.rs` (`test_eval`, `test_eval_simple`) | 1098 | 369 (33.6%) | 67 | 662 |
| [libqalculate](https://github.com/Qalculate/libqalculate) | C++, GPL-2.0-or-later | b245c41 (2026-10-06) | `tests/*.batch` (bitwise, calculus, dates, explog, geometry, limits, matrixvector, numberbase, operators, parser, percentages, polynomial, solver, stats, strings, units, variables) | 656 | 61 (9.3%) | 14 | 581 |
| [Numbat](https://github.com/sharkdp/numbat) | Rust, MIT OR Apache-2.0 | 49fccb1 (2026-08-25) | `numbat/tests/interpreter.rs` (`expect_output`) | 289 | 65 (22.5%) | 22 | 202 |
| [Rink](https://github.com/tiffany352/rink-rs) | Rust, MPL-2.0 (GPL data file) | abf3042 (2026-04-24) | `core/tests/query.rs` (`test`) | 210 | 0 (0.0%) | 33 | 177 |
| [math.js](https://github.com/josdejong/mathjs) | JavaScript, Apache-2.0 | 8d214e0 (2026-08-10) | `test/unit-tests/expression/parse.test.js` (`parseAndEval` with literal result) | 297 | 80 (26.9%) | 22 | 195 |

Not yet imported (tracked by the open-source import sub-issue): Insect
(PureScript, MIT, d19e4f1), kalker (Rust, MIT, 87cd078), Hurmet (JavaScript,
MIT, a41258b), Numara (Electron over math.js, MIT, 21d486f), Mathics (Python,
GPL-3.0, 9586a89), GNU Units (manual examples only), SpeedCrunch.

Unit database sizes, for the dimensional-units sub-issue: Numbat defines 201
units in `numbat/modules/units/*.nbt`, fend's `core/src/units/builtin.rs` is
1003 lines of definitions, and Rink's `core/definitions.units` (derived from
GNU Units) is 7431 lines. This project's `Unit` enum has five families
(currency, duration, data size, mass, timezone) plus an untyped `Custom`.

## Closed-source and documentation-only products

| Product | Source used | Rows | Supported | Different | Unsupported |
| --- | --- | ---: | ---: | ---: | ---: |
| [Soulver 5](https://documentation.soulver.app/) | 40 syntax-reference pages fetched as Markdown (`llms.txt` index), verbatim `expression | answer` pairs | 266 | 25 (9.4%) | 32 | 209 |
| [Numi](https://github.com/nikolaeu/numi/wiki) | wiki examples (hand-picked) | 23 | 10 (43.5%) | 1 | 12 |
| [Wolfram\|Alpha](https://www.wolframalpha.com/examples/mathematics) | example gallery (hand-picked, deterministic) | 20 | 11 (55.0%) | 0 | 9 |
| Google Search calculator | public help examples | 9 | 2 (22.2%) | 1 | 6 |
| [Calca](http://calca.io/) | manual examples | 8 | 4 (50.0%) | 0 | 4 |
| [Raycast](https://manual.raycast.com/calculator) | manual examples | 7 | 3 (42.9%) | 1 | 3 |
| Apple Math Notes | Apple support examples | 5 | 0 (0.0%) | 0 | 5 |
| [Frink](https://frinklang.org/) | documentation examples | 5 | 0 (0.0%) | 0 | 5 |

Soulver's documentation is the most complete public specification of a
natural-language calculator and is therefore the main closed-source reference
(percentages, dates, time zones, units, rates, finance, bases, rounding,
averages, large-number symbols, timecode). Examples whose answers depend on the
current date, live exchange rates or CPI data keep an empty expected value and
only have to evaluate.

## Reference components that are not calculators

These were cloned to evaluate reuse for the natural-language requirements.

| Component | What it offers | Finding |
| --- | --- | --- |
| [text2num](https://github.com/allo-media/text2num) (Rust crate 2.8.0, MIT, 56k downloads) | Spelled cardinals, ordinals and decimals to digits for en, da, nl, es, fr, de, it, pt, ru, ca; token-stream API | Best data source for spelled numbers. No operator or unit phrases, so it cannot replace the LINO vocabulary; port its tables or depend on it behind a feature. |
| [Duckling](https://github.com/facebook/duckling) (Haskell, BSD-3) | Numeral, ordinal, quantity, distance, volume, temperature, duration, time and amount-of-money rules for 49 languages | Not embeddable in Rust/WASM. Its per-language rule files are a checklist for which surface forms each language needs. |
| [Microsoft Recognizers-Text](https://github.com/microsoft/Recognizers-Text) (C#/JS/Python/Java, MIT) | YAML pattern definitions for numbers, units, dates and currencies in 14 languages (`Patterns/*.yaml`) | Patterns are data and MIT; a generator could translate the number and unit YAML into LINO entries. |
| [quantulum3](https://github.com/nielstron/quantulum3) (Python, MIT) | Quantity extraction from English prose with a unit database (`_lang/en_US`) | English only; its unit alias list is a useful cross-check for aliases. |
| [MathCAT](https://github.com/NSoiffer/MathCAT) (Rust, MIT) | MathML to semantic tree with 16 language rule sets | Candidate parser for MathML input in the LaTeX/MathML sub-issue. |
| [Speech Rule Engine](https://github.com/Speech-Rule-Engine/speech-rule-engine) (JavaScript, Apache-2.0) | MathML semantic parsing, 15 locales | Same role as MathCAT for a JavaScript front end. |
| [Mathics](https://github.com/Mathics3/mathics-core) (Python, GPL-3.0) | Wolfram-language CAS | Oracle for symbolic expectations only; license and runtime rule out embedding. |

## Rust crates evaluated (crates.io, 2026-10-07)

| Crate | Version | Role in the plan |
| --- | --- | --- |
| `chrono-tz` | 0.10.4 | IANA zones for the dates sub-issue, behind a feature flag for WASM size. |
| `icu_decimal` / `icu` | 2.3.0 / 2.3.1 | Locale grouping and decimal patterns if a locale profile API is added. |
| `uom` | 0.38.0 | Compile-time dimensions; rejected because calculator dimensions are dynamic. |
| `fend-core` | 1.5.8 | Test oracle for units and bases in `experiments/`; not a backend. |
| `rink-core` | 0.9.0 | Same; its definitions file is GPL-derived and needs a license review before data generation. |
| `numbat` | 1.24.0 | Design reference for the dimension type system. |
| `kalk` | 3.2.3 | Reference for complex numbers and user functions. |
| `num-rational` / `rust_decimal` | 0.4.2 / 1.43.0 | Already dependencies; exact fractions and decimal display. |
| `unicode-normalization` | 0.1.25 | Only for targeted normalization; NFKC would destroy superscripts. |
| `latex2mathml` | 0.2.3 | Size reference for a hand-written LaTeX tokenizer. |
| `chumsky` / `nom` | 0.13.0 / 8.0.0 | Not needed; the existing hand-written lexer and token parser are kept. |
