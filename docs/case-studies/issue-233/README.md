# Issue #233: numeric separators before expression parsing

Issue: https://github.com/link-assistant/calculator/issues/233
Pull request: https://github.com/link-assistant/calculator/pull/252

## Reproduction and root cause

On the original branch, `3,000 minus 12` returned `-9` and
`1,000 divided by 200` returned `0.005`. The locale parser tried ordinary
tokenization first, then rewrote numeric separators only after a parse failure.
An expression that successfully parsed with the wrong comma interpretation
therefore never reached normalization. Tests for the issue's examples were
added and run before the lexer change; grouping, uncertainty, and German scales
failed as reported.

The lexer now validates and canonicalizes each numeric token before parsing.
It retains the original token text and character positions. Western and Indian
grouping, Swiss apostrophes, decimal commas, mixed punctuation, Unicode grouping
spaces, underscores, and scientific notation share this path. Function-call
commas remain argument separators; date components retain token boundaries.
Planning exposes a decimal-comma alternative while calculation uses the primary
interpretation.

## Explicit convention decisions

- A single comma before three digits groups thousands when the leading group
  has one to three digits. Other single commas are decimal marks.
- The issue also requires fend's `1,1` → `11` outlier, which conflicts with that
  general rule. The exact spelling is a narrow compatibility exception in the
  primary interpretation; planning also exposes `1.1`. Other short fractions
  such as `1,2`, `12,3`, and `4,5` remain decimal. Inside functions, `max(1,1)`
  still has two arguments.
- The rightmost punctuation mark is decimal when both comma and dot occur.
- `round(1,234)` has two arguments and produces an argument-count error. Use
  `round(1'234.5)` for a grouped numeric argument.
- Participant counts retain their custom unit. The corpus expectation is
  `699.678 participants`, matching the input and the existing unit behavior.
- Equivalent compact displays such as `300k` and radix displays such as `0x10`
  compare numerically in the coverage checker; unit mismatches still fail.

## Additional syntax needed by the reported examples

`12,3 ± 4,5` produces a measured value with a non-negative absolute uncertainty.
Its central value and uncertainty are exact rationals and must have matching
units. Negation retains a positive error bound. Arithmetic on these values
reports an error until uncertainty propagation is implemented.

`Mio.` and `Mrd.` multiply by one million and one billion, respectively.
Contiguous distance/time units such as `km/h` retain their spelling as custom
units; physical unit conversions are outside this change. Multipliers use
checked arithmetic so oversized literals report overflow instead of panicking.
Scalar scaling and arithmetic with matching compound units preserve the unit.
Powers, mathematical functions, mixed-unit multiplication, and unmatched
division report an error because dimensional algebra is not implemented.

## Regression protection

The existing full corpus gate initially identified space-grouped fractions,
mixed underscore/space grouping, and radix fractions as regressions. Minimal
tests reproduced them before fixes. Space grouping now also works after a
decimal dot. Binary, octal, and hexadecimal literals are parsed numerically,
including fractions, rather than becoming custom units through a fallback.
Radix conversion uses bounded checked integer arithmetic and the existing
decimal literal grammar's 28-place limit. Trailing decimal points and fend's
radix exponents retain their meaning: `0b1e10` means **4**, with both exponent
digits and powers interpreted in base two. Oversized values report overflow.

Regression tests are in `tests/issue_233_number_separator_tests.rs`,
`tests/issue_197_locale_number_tests.rs`, and
`scripts/competitor-coverage.test.mjs`. Run the original examples with
`cargo run --example number_separators`.

Validate the full corpus against the previous committed baseline before
refreshing it:

```sh
cargo build --release --locked --bin link-calculator
node scripts/competitor-coverage.mjs --baseline docs/case-studies/issue-227/results/coverage-baseline.json --out ci-logs/competitor-coverage --quiet
```

The output directory keeps the previous baseline intact during the check.

## Validation results

Before synchronizing with main, the release binary evaluated all 3,055 corpus expressions with zero timeouts
and zero regressions against the previous baseline's 640 supported rows.
The refreshed baseline contains 689 supported, 185 different, and 2,181
unsupported rows. All seven expressions listed in #233 are supported. The
uncertainty row now checks its expected value instead of only requiring a
successful evaluation.

Main then expanded the corpus through #249 and #251. Merge those updates,
retain all licensed provenance, update the two corrected seed expectations,
and regenerate the scientific TSV offline. The combined release check passes
against main's 8,298-case baseline with no regressions: **1,100 supported**
(previously 1,051), **307 different**, **6,891 unsupported**, and **zero timeouts**.
All seven issue rows remain supported.

Local validation passed: 999 Rust tests including the doc test, 42 script tests,
229 web tests, formatting, Clippy with warnings denied, the file-size check,
the release binary build, WASM build, TypeScript checking, and production web
build. The changelog fragment requests the next patch release through the
repository's automatic release pipeline.
The WASM runtime also verifies every reported example using
`node experiments/verify_number_separators_wasm.mjs` after the WASM build.
