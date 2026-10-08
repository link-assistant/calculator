# Percentage arithmetic (#234)

## Reproduction and cause

The parser previously lowered every postfix percentage to division by 100.
The evaluator therefore received ordinary fractions and could neither apply
relative addition nor retain a percentage display.

| Input | Before | After |
| --- | --- | --- |
| `200 + 10%` | `200.1` | `220` |
| `200 - 10%` | `199.9` | `180` |
| `6 - 50%` | `5.5` | `3` |
| `100 + 10% + 10%` | `100.2` | `121` |
| `10% + 20%` | `0.3` | `30%` |
| `5% + 1` | `1.05` | `105%` |
| `180 is what % off 200` | Parse error | `10%` |
| `120% 2` | `0` | `2.4` |

The first six regression groups in
[`issue_234_percent_arithmetic_tests.rs`](../../../tests/issue_234_percent_arithmetic_tests.rs)
failed before the fix. They exercise relative arithmetic, display, exact
fractions, queries and conversions, multilingual phrases, modulo compatibility,
and invalid inputs. Another group checks serialization and notation round trips.

## Arithmetic and grammar

`ValueKind::Percent` stores an exact rational fraction: `10%` contains `1/10`.
A non-percent left operand plus or minus a percent scales the left operand,
retaining its currency or unit. A percent left operand adds or subtracts
percentage points, including when the right operand is a plain scalar.
Multiplying a percent by a scalar returns a scalar; multiplying two percents
retains percentage display. Division returns a scalar.

Percentage queries are lowered into existing arithmetic expressions and exact
ratio constructors. Fraction, reciprocal and multiplier conversions retain
their requested display. Equations also understand percentage constructors
and relative addition.

All added multilingual vocabulary lives in
[`arithmetic-words.lino`](../../../data/words/arithmetic-words.lino).
Percent words have a distinct token from modulo words. CJK possession markers
are recognized as percentage syntax only when followed by percent vocabulary,
preserving existing custom-unit fallbacks. Adjacent `%` followed by a separated
number permits implicit multiplication (`120% 2`); ordinary binary modulo
(`5%4`, `8 % 3`, `50%(13)`, `8 mod 3`) remains available.

`56.7% of 1,234 participants` returns `699.678 participants`, preserving the
population label as required by the number-separator fix merged from #252.
Numeric separators use the shared lexer conventions. Percentage scaling also
retains compound units: `20% of 50 km/h` returns `10 km/h`. The integration
regression failed before resolving the merged function guard; unsupported
compound-unit algebra still reports an error.

## Intentional compatibility changes

Existing standalone `50%` expectations in `expression_parser_tests`,
`issue_145_percent_of_tests`, and `issue_158_modulo_and_strict_parse_tests`
change from `0.5` to `50%`. The parser test for `100 + 10%` changes from
`100.1` to `110`. The issue #218 compatibility expectations and existing
percent-of notation assertions remain unchanged.

The upstream corpus has incompatible conventions. fend expects
`0.1 + 5% = 0.15`, but issue #234 requires relative addition, giving `0.105`.
This row intentionally moves from supported to different. Its original expected
value stays in the corpus, and the mismatch remains visible in the regenerated
report. A direct regression asserts `0.105`.

Other pre-existing dialect gaps include math.js's `3%+100 = 3` and SpeedCrunch's
`100+(10%+10%) = 100.11`; neither agrees with the typed percentage policy.
Financial phrases, per-mille, temperature suffixes, bitwise syntax, and bracketed
physical units require separate features. The issue's historical count of
32 different and 20 unsupported rows is not a count of universally compatible
percentage arithmetic: the corpus has since expanded, and includes these other
constructs and conflicting conventions.

The coverage checker compares percent and decimal output using the same
dimensionless numeric value (for example, `15%` and `0.15`), while preserving
physical-unit and wrong-value mismatches. It recognizes Rink's explicit
`(dimensionless)` label. Regression tests prevent precision rounding from
accepting `0%` as `0.001`.

Across 8,298 corpus rows, supported cases increase from 1,100 on the merged
`main` to 1,222: 123 newly matching rows and the one intentional fend conflict.
Four additional rows improve from unsupported to different. Compared with the
pre-merge percentage implementation, 54 more rows match and none regress.
Among the 198 rows selected by
the percentage inspection script, supported cases increase from 60 to 152,
different results drop from 69 to 18, and unsupported cases drop from 69 to 28.
All Soulver examples from its percentages syntax page now match.

## Verification

```sh
cargo test --all-features --no-fail-fast
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
node --test scripts/*.test.mjs
node scripts/check-file-size.mjs
cargo run --example percentage_arithmetic
cargo build --release --locked --bin link-calculator
node scripts/competitor-coverage.mjs --baseline docs/case-studies/issue-227/results/coverage-baseline.json --out ci-logs/competitor-coverage --quiet
```

The web's `npm test` and `npm run build` and `wasm-pack build --target web`
also validate integration with the shared calculator engine. Local investigation
logs are kept under the ignored `ci-logs` directory. The reusable example is
[`percentage_arithmetic.rs`](../../../examples/percentage_arithmetic.rs), and
[`inspect-corpus.py`](../../../experiments/issue-234/inspect-corpus.py) lists
percentage rows and their committed coverage statuses.
[`compare-coverage.py`](../../../experiments/issue-234/compare-coverage.py)
compares saved baselines and reports every status decline.

The new `web/e2e/percentages.spec.ts` regression fails with `main`'s WASM
(`200.1` instead of `220`) and passes with this implementation. The broader
browser suite exposes existing test-helper failures in theme selection,
expression history, multiline textarea filling, and empty textarea filling.
They reproduce against `main`'s WASM with identical frontend code. The first
run also hit two currency assertions that inspect visible but still-empty result
elements; those passed on the baseline run. These optional browser tests are
outside the CI unit/build checks and are recorded rather than changing unrelated
frontend behavior or increasing timeouts.
