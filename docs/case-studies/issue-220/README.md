# Issue 220 implementation and requirement trace

Research and behavior were checked on 2026-09-15. The umbrella issue, all three
child issues, every issue comment feed, and all three PR feedback channels were
read in full. Issues 217-220 had no comments; PR #221 subsequently requested a
broader conventional/open-calculator expression corpus.

## Complete requirement inventory

| ID | Source | Requirement | Resolution in this pull request |
| --- | --- | --- | --- |
| R1 | #220 | Read and implement every requirement in child issues #217, #218, and #219, including comments. | All four issue bodies, their complete comment feeds, and the later PR feedback were reviewed. The concrete bugs are covered by issue-specific integration tests; the market request is covered by a sourced audit, compatibility suite, conventional/natural notation, and a prioritized gap plan. |
| R2 | #220 | Apply the requirements across the codebase, not as separate partial fixes. | The changes are in the shared lexer, expression parser, locale-number fallback, and datetime combiner used by the library, CLI, and WASM entry points. |
| R3 | #220 | Deliver the child work in one pull request. | All work is delivered through PR #221. |
| R4 | #220 | Close each issue explicitly. | The final PR description contains separate `Fixes #217`, `Fixes #218`, `Fixes #219`, and `Fixes #220` lines. |
| R5 | #220 | Explicitly identify anything already fixed or not reproducible. | None of the two reported expressions was fixed or non-reproducible on the starting revision. Both failed before the change and have saved regression tests. |
| R6 | #217 | `15\u202f847 / 17\u202f256` must calculate without leaving `847` as trailing input. | Common typographic grouping spaces are normalized only when every group is structurally valid. The exact report and five common space separators are tested. |
| R7 | #217 | Do not corrupt ordinary whitespace-separated expressions while accepting grouping spaces. | The normalization requires a 1-3 digit leading group followed only by 3-digit groups. `12 34`, `1234 567`, and `1\u202f23` remain invalid. |
| R8 | #218 | Research similar calculators, preferring open-source projects and their public docs/tests. | The companion [competitor audit](competitor-audit.md) surveys five open-source engines and five commercial/general products from primary sources. |
| R9 | #218 | Add competitors' published examples so compatible expressions remain working. | `issue_218_competitor_compatibility_tests.rs` now locks more than 40 deterministic examples and guards spanning Numbat, fend, Qalculate!, math.js, Numi, Parsify, Soulver, Raycast, and Wolfram\|Alpha. Unsupported examples remain recorded rather than being claimed as parity. |
| R10 | #218 | Expand natural-language expression coverage. | The shared grammar supports documented word operators, scientific notation, `_` digit separators, Unicode arithmetic/root/power symbols, `**`, implicit products, functions without parentheses, function aliases, `mod(a, b)`, and natural root/power phrases. |
| R11 | #218 | Work toward a free/open alternative with broader coverage than paid options and Wolfram|Alpha. | This is an ongoing product goal rather than a finite acceptance condition. The audit makes it measurable: it lists current coverage, exact gaps, candidate components, and an ordered implementation plan including paid-only feature classes. |
| R12 | #219 | `00:35 16 september 2026 IST` must parse without leaving `:` as trailing input. | A trailing timezone is associated with the preceding time/date pair before combination. The exact report, all supported word orders, numeric dates, and 12-hour input are tested. |
| R13 | #219 | Preserve the stated wall clock and apply the timezone exactly once. | Every tested order displays `2026-09-16 00:35:00 IST` and records `2026-09-15 19:05:00 UTC` with offset +05:30. This also fixes a pre-existing double-offset defect in `time timezone date` order. |
| R14 | PR feedback | Support popular conventional expressions from open calculators and scientific writing, without adopting proprietary-language syntax. | Primary-source syntax pages were re-reviewed and their shared scalar grammar promoted into the executable corpus. Ambiguity guards preserve units, SI suffixes, dates, strict separators, and compact placeholder equations; Mathematica/product scripting syntax remains out of scope. |

## Root causes and solution choices

### Unicode digit grouping (#217)

The lexer correctly treats Unicode whitespace as a token boundary. The locale
fallback previously retried only comma/dot decimal conventions, so U+202F
split the first number and left the next numeric token unconsumed.

Options considered:

1. Make grouping whitespace part of every numeric token. This makes the core
   lexer locale-aware and risks consuming expression separators.
2. Delete every space between digits. This accepts malformed and ambiguous
   input such as `12 34`.
3. Extend the existing locale fallback with validated grouped integers.

Option 3 was selected. It composes with comma/dot locale normalization, keeps
the strict first parse unchanged, and accepts ASCII space, no-break space,
figure space, thin space, and narrow no-break space. This matches the CLDR
model in which grouping symbols are locale-dependent, including U+202F.

### Competitor compatibility (#218)

The request's “all competitors/all examples” universe changes continuously and
includes non-computational queries, random expressions, network-dependent
rates, copyrighted/private test suites, and mutually incompatible semantics.
A one-time assertion of universal parity would therefore be unverifiable.

Options considered:

1. Replace the engine with `fend_core` or Numbat. This gives broad units and
   precision quickly but loses this project's LINO AST, equation behavior,
   datetime/currency model, and result metadata.
2. Send unknown input to a hosted knowledge engine. This conflicts with the
   offline, free, deterministic, and privacy goals.
3. Maintain a dated primary-source capability audit, promote compatible
   deterministic examples into tests, and implement gaps by capability.

Option 3 was selected. The implementation now covers the conventional scalar
notation shared by the surveyed engines: scientific numbers, digit separators,
Unicode operators and roots, three power forms, implicit multiplication,
parenthesis-free unary functions, aliases, function-form modulo, and natural
root/power phrases. Context-sensitive parsing preserves the established `**`
placeholder equation syntax and distinguishes known one-letter units from
coefficient variables. The audit remains explicit about unsupported examples
and semantic conflicts, so future work can move rows from “gap” to the
executable corpus without overstating compatibility.

### Time + date + timezone ordering (#219)

The datetime combiner recognized a timezone adjacent to a time, but not a
timezone after a time-first date. It also reused an already offset-adjusted
time in one supported order, applying the offset a second time.

Options considered:

1. Add one special-case format for the reported string. This would duplicate
   the bug across numeric dates, 12-hour clocks, and other timezones.
2. Expand the datetime parser into a monolithic regex set. This would bypass
   the established date/time parsers and make precedence harder to reason about.
3. Centralize date/time construction and relocate an optional trailing
   timezone beside the time before testing supported splits.

Option 3 was selected. All date-first/time-first/comma paths now use one
combiner and one offset application.

## Whole-codebase verification

- `Calculator::plan_internal` and `Calculator::calculate_internal` both use
  the shared `ExpressionParser`; the CLI and WASM wrappers call those APIs.
- Locale retries happen at the expression-parser boundary, so validated
  grouping works for arithmetic, planning, CLI, library, and browser callers.
- Conventional notation is normalized in the shared lexer and token parser;
  scientific values remain decimal-backed and implicit products use the same
  expression tree and precedence as explicit multiplication.
- Date/time collection happens in the shared token parser and datetime type,
  so every front end accepts the same token orders.
- The compatibility suite exercises public `Calculator` behavior for display
  contracts and `ExpressionParser` numeric values for irrational comparisons,
  providing end-to-end contracts without rounding-dependent assertions.
- No version was edited: the repository release workflow derives the next
  version from changelog fragments.

## Tests

- `tests/issue_217_unicode_grouping_tests.rs`
- `tests/issue_218_competitor_compatibility_tests.rs`
- `tests/issue_219_time_date_timezone_tests.rs`
- Existing locale-number, datetime, parser, library, CLI-target, and WASM-target
  tests are exercised by the full all-features suite.
