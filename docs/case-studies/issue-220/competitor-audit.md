# Calculator competitor compatibility audit

Snapshot date: 2026-09-15.

This is a living, evidence-based interpretation of issue #218. “All
competitors” cannot be a closed set, so this baseline covers prominent
open-source engines first, followed by commercial text calculators and the
explicit Wolfram|Alpha benchmark. Only official documentation, repositories,
and product pages are used.

## Method

1. Inventory the public feature surface and published examples of each product.
2. Run deterministic single-expression examples through `Calculator`.
3. Promote examples that have the same meaning and result to the integration
   test corpus.
4. Record unsupported or semantically different examples as exact gaps.
5. Prioritize reusable capability work; rerun and expand the corpus whenever a
   gap is implemented or an upstream source materially changes.

Live rates, “now”, random/dice output, uploads, private suites, UI-only actions,
and general-knowledge questions cannot have stable literal result assertions.
They are inventoried as capabilities and need mocked integration or UI tests
when implemented.

## Primary-source survey

### Open-source projects (reviewed first)

| Project and sources | Published surface | Current overlap | Representative gaps |
| --- | --- | --- | --- |
| [Qalculate! features](https://qalculate.github.io/features.html) and [expression syntax](https://qalculate.github.io/manual/qalculate-expressions.html) | Arbitrary precision, complex/infinite numbers, interval uncertainty, bases, symbolic algebra/calculus, equations, matrices/vectors, units, dates, plots, customizable functions/variables. | Arithmetic, exact fractions, equations, selected functions, implicit products such as `5x` and `5(2 + 3)`, numeric integration/plots, limited units/dates. | Complex/interval values, matrices, broad symbolic transforms, bases/Roman numerals, general dimensional units, custom functions. |
| [Numbat repository](https://github.com/sharkdp/numbat), [syntax examples](https://numbat.dev/docs/examples/example-numbat_syntax/), and [tutorial](https://numbat.dev/docs/tutorial/) | Statically checked physical dimensions, extensive first-class/custom units, constants, functions, variables, procedures, assertions, REPL and web UI. | Scientific notation, Unicode multiplication/division, `^`/`**`/superscript powers, implicit constant multiplication, modulo, deterministic scalar functions, mass/time/data units, currencies. | `30 km/h -> mph`, `5 in + 2 ft -> cm`, compound dimensions, typed variables, custom units, assertions. |
| [fend repository](https://github.com/printfn/fend) and [manual](https://printfn.github.io/fend/documentation/) | Arbitrary-precision rational/complex arithmetic, units and temperatures, bases, bitwise operators, dice, dates, variables, lambdas, formats, scripting. | Arithmetic, digit separators, scientific notation, implicit `2pi`, functions without parentheses, fractions, factorial/modulo, common functions, mass/time/data units, dates/currencies. | `1 ft to cm`, `0.(3) to fraction`, `0b1001 + 3`, complex numbers, temperature, dice, persistent variables/lambdas. |
| [math.js expression syntax](https://mathjs.org/docs/expressions/syntax.html), [data types](https://mathjs.org/docs/), and [units](https://mathjs.org/docs/datatypes/units.html) | Scalar/big/fraction/complex values, strings/booleans, matrices/objects, extensive functions, variables, symbolic work, broad/custom dimensional units. | Scalar/fraction arithmetic, comparisons, functions, and documented implicit multiplication forms such as `(1 + 2)(3 + 4)`. | `45 cm + 0.1m`, `cos(45 deg)`, matrices, complex/boolean/string/object values, custom units. |
| [GNU Units manual](https://www.gnu.org/software/units/manual/units.html) | Large and user-extensible unit database, nonlinear/piecewise units, compound dimensions, physical constants, localization, currency and CPI data. | Mass/time/data/currency conversions. | General SI/imperial dimensions, compound/nonlinear units, physical constants, user unit files. |

### Commercial and general products

| Project and sources | Published surface | Current overlap | Representative gaps |
| --- | --- | --- | --- |
| [Numi official wiki](https://github.com/nikolaeu/numi/wiki) | Word operators, currencies/timezones, bases, percentages/scales, variables, functions, units, line references/totals, labels/comments, import/export, JavaScript extensions. | Arithmetic/functions, aliases such as `fact`/`arcsin`, functions without parentheses, adjacent-parenthesis multiplication, percent-of, currencies, fixed timezone abbreviations, mass/time/data, comparisons. | `1 meter 20 cm = 120 cm`, bases, percent-on/off, city zones, persistent variables, line totals, custom extensions. |
| [Parsify getting started](https://parsify.notion.site/Getting-started-be7132e43e844bd88fe2ad48918b43d7) and [desktop repository](https://github.com/parsify-dev/desktop) | Word arithmetic, variables, angle-aware functions, broad/custom units, currency, calendars/timezones, plugins, labels/comments. | Arithmetic, word operators added here, common functions, mass/time/data/currency. | `sin(30 deg)`, variables across lines, broad/custom units, plugins. Parsify's `log(pi)` means natural log (about 1.145), while this project preserves its existing base-10 `log` result. |
| [Soulver getting started](https://documentation.soulver.app/documentation/getting-started), [variables](https://documentation.soulver.app/documentation/variables), and [unit reference](https://documentation.soulver.app/syntax-reference/units-and-conversions/unit-reference) | Natural-language notepad, phrase variables, labels/comments, percent semantics, calendar/timezone math, 200+ units, financial/rate functions, line references/subtotals. | Arithmetic, percent-of, dates/durations, currencies, history, limited units. | Soulver treats `120 + 30%` as a 30% increase (156), while this project currently adds 0.3 (120.3); phrase variables, city zones, financial functions, broad units and timecode are absent. |
| [Raycast calculator manual](https://manual.raycast.com/calculator) | Natural-language arithmetic plus unit/currency/crypto conversion, percentages, city timezones, relative calendar dates. | Arithmetic, natural `square root of`/`power` phrases, percent-of, currencies, fixed timezone abbreviations, dates. | `10ft in m`, `5pm ldn in sf`, `time in tokyo`, `monday in 3 weeks`. |
| [Wolfram|Alpha units](https://www.wolframalpha.com/examples/science-and-technology/units-and-measures), [dates/times](https://www.wolframalpha.com/examples/society-and-culture/dates-and-times), and [Pro](https://www.wolframalpha.com/pro/) | Computational knowledge across domains; broad units, calendars/timezones and natural-language queries. Paid features include step-by-step solutions, uploads, downloadable data, more compute, and customizable interactive visuals. | Arithmetic/functions, equation solving, numeric integration/plots, limited units/dates/currencies, calculation steps. | `20mL in drams`, `10 miles + 14 kilometers`, calendar systems/holidays/city zones, general knowledge, richer symbolic steps, 60+ upload formats, downloads and customizable visuals. `175lb vs 100kg` is supported. |

[SpeedCrunch](https://www.speedcrunch.org/introduction.html) was also checked
as a desktop scientific-calculator peer. Its main differentiators are a large
formula/constants book, high precision, autocomplete, and searchable session
history rather than a distinct natural-language grammar.

## Executable seed corpus

The following verbatim expressions have equivalent deterministic semantics and
are locked by `tests/issue_218_competitor_compatibility_tests.rs`:

| Source | Expressions |
| --- | --- |
| Numbat | `1920 / 16 * 9`; `2^32` |
| Parsify | `12+5`; `2*(3/4)` |
| Numi | `8 times 9`; `20% of $10` |
| Soulver | `30% of 700` |
| fend | `1 + 3 * 4`; `16^2`; `5!` |
| Wolfram|Alpha | `175lb vs 100kg` |

The expanded conventional-notation corpus also locks these published grammar
families:

| Source | Expressions or forms now covered |
| --- | --- |
| Numbat | Decimal scientific notation; `1920 ÷ 16 × 9`; `6 · 7`; `6 ⋅ 7`; `2**3`; `2³`; `2⁻³`; `2¹⁰`; `mod(17, 4)`; `2 pi` |
| fend | `_` digit separators; decimal scientific notation; `2pi`; `sqrt 2`; `sqrt 16` |
| Qalculate! | `2(3 + 4)` and coefficient multiplication |
| math.js | `(1 + 2)(3 + 4)`; `(4 - 1)2`; `sqrt(4)(1 + 2)` |
| Numi | `cbrt 8`; `fact 5`; `arcsin 1`; `6 (3) = 18` |
| Raycast | `square root of 625`; `2 power 10` |

Conventional scientific writing adds Unicode minus and root signs (`−`, `√`,
`∛`), `π`, and both compact and spaced coefficients (`2x`, `2 x`). The parser
keeps established meanings at ambiguity boundaries: `2h` remains a duration,
`2k USD` remains an SI-scaled currency amount, malformed `1__0` stays invalid,
and leading/operand-less `**` remains the existing equation placeholder syntax.
Implicit multiplication has ordinary multiplicative precedence; parentheses
are recommended whenever adjacency could obscure the intended grouping.

The word-operator test additionally covers all unambiguous aliases in the Numi
operations table and the four Parsify word forms: `plus`, `with`, `minus`,
`subtract`, `without`, `times`, `multiplied by`, `mul`, `divide`, `divided by`,
`mod`, and `modulo`. Numi's `and` is intentionally not an addition alias here:
this grammar already uses it as the separator in `compare A and B`.

The corpus deliberately targets conventional mathematical notation and the
documented surface of open calculators. It does not add Mathematica syntax,
product-specific scripting languages, or proprietary/internal test cases.

## Capability gap plan

| Priority | Capability slice | Example acceptance corpus | Implementation plan |
| --- | --- | --- | --- |
| P0 | General dimensional unit algebra | `1 ft to cm`; `10 miles + 14 kilometers`; `20mL in drams`; `90 km/h to m/s`; `cos(45 deg)` | Introduce dimension vectors and affine conversion metadata; import a reviewed unit data set; keep currency/date units separate; add generated aliases and cross-engine fixtures. |
| P0 | Locale-safe natural input | Additional CLDR grouped/decimal samples; word operators in supported locales | Keep strict parsing first, make normalization locale profiles explicit, and add corpus-driven ambiguity tests before aliases. |
| P1 | IANA/city timezone and relative calendar grammar | `5pm Europe/London in America/Los_Angeles`; `monday in 3 weeks` | Add IANA-zone identity and DST-aware resolution, then a deterministic relative-date layer with caller-supplied reference time. |
| P1 | Notepad state | `x = 2` followed by `x * 4`; `prev`; `sum`; labels/comments | Separate assignment statements from equation solving, add an explicit evaluation context, then persist context only at the UI/session boundary. |
| P1 | Precision and value types | repeating decimals, bases, complex values, vectors/matrices | Generalize `ValueKind`; preserve exact rational paths; add one type at a time with serialization, LINO, display and operation matrices tested. |
| P2 | Symbolic mathematics | simplification, derivatives, richer equation/inequality solving and steps | Extend the expression tree with symbolic transforms and proof-step objects; avoid string-only rewrites. Evaluate a dedicated CAS boundary if native rules become unmaintainable. |
| P2 | Free equivalents of paid workflow features | import CSV, export results, downloadable/interactive plots, detailed steps | Start with open formats and deterministic local processing. Add UI tests and sanitized fixture files; keep uploads opt-in and offline-capable. |
| P3 | Computational knowledge | holidays, constants, domain calculators, structured datasets | Use versioned open datasets and typed domain adapters. Keep provenance in results and do not silently turn arbitrary text into web search. |

## Existing components evaluated

| Component | Useful for | Decision |
| --- | --- | --- |
| [`chrono-tz`](https://docs.rs/chrono-tz/latest/chrono_tz/) | IANA timezone database and DST-aware `chrono::TimeZone` values. | Strong candidate for P1. Not needed for #219's fixed IST offset and adds WASM size, so defer until named zones are implemented. |
| [`dateparser`](https://docs.rs/dateparser/latest/dateparser/) | Common date formats and timezone-aware parsing. | Useful reference/fallback, but it does not replace multilingual calculator grammar or preserve this project's AST/LINO interpretation. |
| [`rust_dateparser`](https://docs.rs/rust_dateparser/latest/rust_dateparser/) | Localized absolute/relative dates and non-Gregorian calendars with caller-provided reference time. | Candidate for a separately benchmarked P1/P3 prototype; dependency size, WASM compatibility, ambiguity policy, and license must be reviewed first. |
| [`fend_core`](https://docs.rs/fend-core/latest/fend_core/) | Embeddable arbitrary-precision unit-aware calculator. | Excellent compatibility oracle or optional backend experiment. Direct replacement would bypass LINO, custom currency history, dates and current result metadata. |
| [Numbat](https://github.com/sharkdp/numbat) | Dimension type system, unit database and language architecture. | Best design/data reference for P0. Embedding the full language would introduce a second parser and incompatible statement model. |
| [GNU Units](https://www.gnu.org/software/units/manual/units.html) | Comprehensive open unit definitions, including nonlinear units and constants. | Investigate its data/license boundary for generated definitions; do not copy data until redistribution and attribution requirements are documented. |

## Standards note for #217

Unicode spaces between digits are not interchangeable decoration. The
[Unicode CLDR number-format specification](https://unicode.org/reports/tr35/tr35-numbers.html)
defines locale-specific grouping symbols and explicitly includes narrow
no-break space behavior. The implementation therefore validates grouping shape
before removing separators; it does not globally strip whitespace.
