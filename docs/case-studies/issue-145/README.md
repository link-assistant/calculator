# Issue 145 Case Study

## Summary

Issue #145 reports that `8% of $50` evaluates to `0.08` instead of `4 USD`.
The expression `8% of $50` should be interpreted as `(8 / 100) * (50 USD)`,
the same as the already-working form `8% * $50`.

## Collected Artifacts

- `issue.json`: issue #145 title, body, timestamps, and GitHub metadata.
- `issue-comments.json`: all issue #145 comments fetched via the GitHub API.
- `pr-146.json`: PR #146 metadata for the fix branch.
- `ci-runs.json`: recent CI runs for branch `issue-145-7ec822e4c743`.

## Environment from Issue Report

- **Version**: 0.15.0
- **URL**: https://link-assistant.github.io/calculator/?q=KGV4cHJlc3Npb24lMjAlMjI4JTI1JTIwb2YlMjAlMjQ1MCUyMik
- **User Agent**: Chrome 147 / macOS 10.15.7
- **WASM Ready**: Yes
- **Timestamp**: 2026-04-29T19:35:16.916Z

## Observed Behaviour

| Input       | Expected | Actual |
|-------------|----------|--------|
| `8% of $50` | `4 USD`  | `0.08` |
| `8% * $50`  | `4 USD`  | `4 USD` (already correct) |

The reported **Links Notation** for `8% of $50` was:

```
(8 / 100)
```

The expected **Links Notation** is:

```
((8 / 100) * (50 USD))
```

## Timeline / Sequence of Events

1. `8%` is lexed as `Number(8)`, `Percent`.
2. `of` is lexed as `Identifier("of")` (not a recognized keyword).
3. `$50` is lexed as `Identifier("$")`, `Number(50)`.
4. `parse_unary()` in `token_parser.rs:118–126` handles the `%` postfix:
   it returns `8 / 100 = 0.08` and advances past `%`.
5. The top-level `parse_and_evaluate` then attempts to reconcile the remainder
   `of $50`.  Because `of` is an unknown identifier and the expression parser
   never saw it, **it was silently dropped** — the parser returns 0.08 as the
   final result.

The root cause is **the absence of an `Of` token kind** in the lexer and
**no grammar rule** to handle `expr% of rhs`.

## Root Cause Analysis

### R1 — Missing `Of` keyword

`src/grammar/lexer.rs` defines keyword tokens (`At`, `As`, `In`, `To`,
`Until`) but did not include `Of`.  The string `"of"` therefore fell through
to `TokenKind::Identifier("of")`.

### R2 — No percent-of grammar rule

`token_parser.rs::parse_unary()` handled `expr%` → `expr / 100` but did not
check for a following `of <rhs>` pattern to produce
`(expr / 100) * rhs`.

### R3 — Silent partial parse

`parse_and_evaluate` calls `parse()` then evaluates the AST.  The tokenizer
silently produced tokens for `of $50`, but since `parse_expression()` returned
after consuming `8 / 100`, the trailing tokens were discarded without an error.

## Requirements from the Issue

1. `8% of $50` → `4 USD` (= `(8/100) * 50 USD`).
2. The result and Links Notation must match those of `8% * $50`.
3. All existing percentage expressions (`50%`, `3% * 50`, `(3+2)% * 10`,
   `100 + 10%`) must remain unchanged.

## Proposed Solution (implemented)

### Approach: add `Of` token + percent-of grammar rule

**Step 1 — Lexer (`src/grammar/lexer.rs`)**

Add `Of` to `TokenKind`:

```rust
/// The "of" keyword for percent-of expressions (e.g., `8% of $50`).
Of,
```

Recognize it in `scan_identifier`:

```rust
"of" => TokenKind::Of,
```

**Step 2 — Parser (`src/grammar/token_parser.rs`)**

Extend `parse_unary()` to desugar `N% of X` → `(N / 100) * X`:

```rust
if matches!(self.current_kind(), Some(TokenKind::Percent)) {
    self.advance();
    let percent_expr = Expression::binary(expr, BinaryOp::Divide, Expression::number(Decimal::new(100)));
    if matches!(self.current_kind(), Some(TokenKind::Of)) {
        self.advance(); // consume "of"
        let rhs = self.parse_primary()?;
        return Ok(Expression::binary(percent_expr, BinaryOp::Multiply, rhs));
    }
    return Ok(percent_expr);
}
```

This is minimal, backwards-compatible, and follows the same pattern as the
`Bang` (`!`) token added in issue #132.

### Alternative considered

Parse `% of` entirely at the lexer level as a single compound token
(`PercentOf`). Rejected: it complicates the lexer with look-ahead and
provides no benefit over handling it in the parser.

### Known limitation

`of` is now a reserved keyword.  Any expression using `of` as a variable name
will fail to parse.  This is acceptable: `of` has no mathematical meaning as a
variable and is already used in natural language by English-speaking users to
mean "percent of a value".

## Online Research

- **Natural language calculators** (WolframAlpha, Google Calculator, macOS
  Calculator) all interpret `N% of X` as `(N/100) * X`.
  - WolframAlpha: https://www.wolframalpha.com/input?i=8%25+of+50
  - Google: searching "8% of 50" in the browser URL bar yields 4.
- **JavaScript `Intl.NumberFormat`** has no built-in "percent of" operator;
  all implementations desugar to multiplication.
- **Existing `%` handling** in this codebase (issue #59) already establishes
  the convention that `N%` = `N/100`, so `N% of X = (N/100) * X` is the
  natural extension.

## Fix Verification

All seven tests in `tests/issue_145_percent_of_tests.rs` pass:

```
test test_issue_145_8_percent_of_50_usd         ... ok  (core bug)
test test_issue_145_8_percent_times_50_usd      ... ok  (equivalent form)
test test_issue_145_10_percent_of_200           ... ok
test test_issue_145_50_percent_of_80            ... ok
test test_issue_145_100_percent_of_42           ... ok
test test_issue_145_percent_standalone_unchanged ... ok  (backward compat)
test test_issue_145_percent_times_number_unchanged ... ok (backward compat)
```

Full test suite: **0 failures**.
