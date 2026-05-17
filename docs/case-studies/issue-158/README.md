# Issue 158 Case Study: Binary Modulo and Strict Parser Consumption

## Summary

Issue [#158](https://github.com/link-assistant/calculator/issues/158)
reports that link-calculator v0.16.0 silently accepts only the valid prefix of:

```text
100 - 25 % 7
```

The calculator returned `99.75` with Links Notation `(100 - (25 / 100))`,
which means it interpreted `25%` as postfix percent and dropped the trailing
`7`. The requested behavior is to support `%` as binary remainder/modulo in
math-like infix expressions, or at minimum reject the expression instead of
silently accepting a prefix.

The issue also calls out `2 + 2 please`, which returned `4 please` because the
unknown word was parsed as a custom unit. That is useful for real custom units
when deliberate, but bad for natural-language wrappers passed in by
downstream callers.

## Collected Data

- `issue.json`: raw issue #158 data, including title, body, comments, and
  timestamps.
- `issue-comments.json`: issue conversation fetched with
  `gh api repos/link-assistant/calculator/issues/158/comments --paginate`.
- `pr-159.json`: initial PR #159 metadata.
- `pr-review-comments.json`, `pr-conversation-comments.json`, `pr-reviews.json`:
  PR discussion data. All were empty when the investigation started.
- `pr-159-initial.diff`: initial placeholder PR diff.
- `formal-ai-issue-96.json`, `formal-ai-issue-96-comments.json`: downstream
  formal-ai report referenced by this issue.
- `related-pr-146-percent-of.json`: recent percent-of parser fix used as
  related parser history.
- `related-pr-157-library-surface.json`: recent library-surface work for the
  formal-ai consumer.
- `ci-runs.json`: recent CI runs for branch `issue-158-31cf69f63b70`; no runs
  existed when checked, so `ci-logs/` has no downloaded run logs.
- `failing-test-before-fix.log`: saved reproduction test output before the fix.
- `focused-test-after-fix.log`, `focused-test-after-refactor.log`,
  `issue-145-percent-test-after-fix.log`, `expression-parser-test-after-fix.log`,
  `issue-156-library-surface-test-after-fix.log`, `lino-rate-test-after-fix.log`:
  saved focused verification logs after the fix.
- `cargo-fmt-check.log`, `file-size-check.log`,
  `cargo-clippy-all-targets-all-features.log`, `cargo-test-all-features.log`:
  final local check logs.

## Timeline

- 2026-05-16T23:18:21Z: formal-ai issue
  [#96](https://github.com/link-assistant/formal-ai/issues/96) was opened for
  the downstream fallback gap.
- 2026-05-16T23:33:25Z: calculator issue #158 was opened with the concrete
  `100 - 25 % 7` reproduction.
- 2026-05-17T09:33:16Z: issue comment requested a full case study, downloaded
  artifacts, and online research.
- 2026-05-17T09:34:05Z: PR #159 was opened as a WIP placeholder from branch
  `issue-158-31cf69f63b70`.
- 2026-05-17T09:39Z: CI history was checked; no branch runs existed yet.
- 2026-05-17T09:40Z: a failing regression test was added and saved in
  `failing-test-before-fix.log`.
- 2026-05-17T09:43Z: implementation and focused verification completed.

## Requirements

1. Parse binary `%` as remainder/modulo for math-like infix prompts.
2. Return `96` for `100 - 25 % 7`, with Links Notation
   `(100 - (25 % 7))`.
3. Give binary `%` the same precedence as `*` and `/`, so
   `100 - 25 % 7 * 2` evaluates as `100 - ((25 % 7) * 2)`.
4. Preserve postfix percent behavior for expressions such as `50%` and
   `3% * 50`.
5. Preserve percent-of behavior added in issue #145, e.g. `8% of $50`.
6. Reject trailing tokens after a complete expression instead of silently
   dropping them.
7. Reject the accidental custom-unit result for `2 + 2 please` where the left
   operand is unitless and the right side is an arbitrary custom unit.
8. Keep formal-ai's use case covered by automated tests and PR documentation.

## Root Cause

### R1: `%` existed only as postfix percent

`TokenKind::Percent` was handled in `TokenParser::parse_unary()` only as
`expr% -> expr / 100`. There was no `BinaryOp::Modulo`, no multiplicative
parser branch for `%`, and no evaluator operation for remainder.

For `100 - 25 % 7`, the parser read `25%` as `25 / 100`, returned that AST,
and left the trailing `7` unconsumed.

### R2: top-level parsing did not require EOF

`ExpressionParser::parse()` called `TokenParser::parse_expression()` directly.
That method is intentionally reusable for nested grammar contexts, so it stops
when the next token is not part of the current expression. The top-level entry
point never checked that the next token was EOF, allowing valid prefixes to
become accepted full calculations.

### R3: unknown unit fallback was too permissive around unitless arithmetic

The number grammar deliberately allows custom units: `2 apples + 3 apples`
should remain representable. The problem case was more specific: arithmetic
between a unitless value and a value carrying an unknown custom unit accepted
the custom unit and returned `4 please`.

That behavior made natural-language filler look like a successful calculator
unit.

## External Facts and Existing Components

- The Rust Reference documents `%` as the remainder operator and places it in
  the same precedence group as `*` and `/`:
  https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html
- This crate already uses `num-rational` and `num-bigint` for exact rational
  arithmetic, so the fix can implement remainder without adding a new
  dependency.
- Recent calculator PR
  [#146](https://github.com/link-assistant/calculator/pull/146) added
  `N% of X` support by desugaring percent syntax in the parser. This fix keeps
  that postfix/percent-of model and adds binary `%` only when `%` is followed
  by a right-hand expression.
- Recent PR
  [#157](https://github.com/link-assistant/calculator/pull/157) expanded the
  Rust library surface for formal-ai. This issue is a concrete downstream
  compatibility fix for that integration.

## Solution Options

### Option 1: reject all `%` followed by non-operator trailing input

This would satisfy the "do not accept a prefix" fallback requirement, but would
leave formal-ai's binary remainder prompt unsupported and keep the expression
on its local fallback.

### Option 2: add binary modulo and strict top-level EOF

Add `BinaryOp::Modulo`, parse `%` as a multiplicative operator when it is
followed by a right-hand expression, keep postfix `%` otherwise, and require
the top-level parser to consume all tokens. This directly addresses the issue
without removing existing percent syntax.

### Option 3: split percent and modulo into separate tokens by whitespace

This could treat `25%` as postfix percent and `25 % 7` as binary modulo, but
it would make parser behavior depend on spacing. The existing lexer and grammar
mostly avoid whitespace-sensitive arithmetic, so this was rejected.

## Implemented Fix

- Added `BinaryOp::Modulo` with display, Links Notation, LaTeX, and
  multiplicative precedence.
- Added `TokenParser::parse_complete_expression()` and switched
  `ExpressionParser::parse()` to use it at the top level.
- Updated `%` parsing so:
  - `50%` remains postfix percent.
  - `3% * 50` remains postfix percent followed by multiplication.
  - `8% of $50` remains percent-of.
  - `25 % 7` becomes binary modulo because `%` is followed by a right-hand
    expression.
- Added `Rational::remainder()` using truncation toward zero, matching
  Rust-like signed remainder behavior such as `-5 % 2 == -1`.
- Added `Value::modulo()` for unitless numeric values and routed
  `ExpressionParser::apply_binary_op()` through it.
- Rejected unitless/custom-unit addition and subtraction so
  `2 + 2 please` no longer succeeds as `4 please`, while deliberate matching
  custom units can still work.
- Added `tests/issue_158_modulo_and_strict_parse_tests.rs` as the regression
  suite.

## Verification

Before the fix, `failing-test-before-fix.log` showed five failures:

- `100 - 25 % 7` returned `99.75` instead of `96`.
- `100 - 25 % 7 * 2` returned `99.75` instead of `92`.
- `-5 % 2` returned `-0.05` instead of `-1`.
- `50% please` silently returned `0.5`.
- `2 + 2 please` silently returned `4 please`.

After the fix, these focused checks pass:

```text
cargo test --test issue_158_modulo_and_strict_parse_tests -- --nocapture
cargo test --test issue_145_percent_of_tests -- --nocapture
cargo test --test expression_parser_tests -- --nocapture
cargo test --test issue_156_library_surface_tests -- --nocapture
```

The final local check pass also succeeded:

```text
cargo fmt --check
node scripts/check-file-size.mjs
cargo clippy --all-targets --all-features
cargo test --all-features
```

## Related Work

- [Issue #145 case study](../issue-145/README.md): percent-of parsing and the
  earlier silent-prefix root cause.
- [Issue #156 case study](../issue-156/README.md): formal-ai library surface
  audit and downstream consumption pattern.
- [formal-ai #96](https://github.com/link-assistant/formal-ai/issues/96): the
  downstream report that motivated keeping binary `%` on formal-ai fallback
  until this upstream fix lands.
