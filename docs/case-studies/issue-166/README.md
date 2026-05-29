# Issue 166 Case Study: Numeric Date Literals in Expressions

## Summary

Issue [#166](https://github.com/link-assistant/calculator/issues/166) reports
that link-calculator v0.17.2 fails to parse:

```text
15.10.2025 + 180 days
```

with the error:

```text
Parse error: Unexpected trailing input '.2025' at position 5
```

The user expects both of these to work and to compute the date 180 days after
15 October 2025:

```text
15.10.2025 + 180 days
15/10/2025 + 180 days
```

and, more broadly, asks for "the widest possible parsing support for dates, in
all locales and languages."

The root cause is that the **lexer never produced a date token for numeric
dates**. Only month-name dates (`Feb 8, 2021`) ever reached the date parser;
numeric dates were shredded into arithmetic tokens (`15 / 10 / 2025`) or, with
dots, rejected outright. The fix teaches the lexer to recognise a complete
numeric date literal — with a strict disambiguation rule so that ordinary
arithmetic is never misread as a date — and adds the missing dot- and
dash-separated date formats to the date parser.

## Collected Data

- `issue.json`: raw issue #166 data (title, body, state, author, labels),
  fetched with `gh issue view 166 --json ...`.
- `issue-comments.json`: issue conversation fetched with
  `gh api repos/link-assistant/calculator/issues/166/comments --paginate`
  (empty — the issue had no comments when the investigation started).
- `pr-167.json`: PR #167 metadata (the single PR tracking this work).
- `ci-runs.json`: recent CI runs for branch `issue-166-b2fb36372b96`
  (empty until the fix is pushed).
- `cargo-test-after-fix.log`: full `cargo test` run after the fix (all suites
  pass).
- `issue-166-focused-tests.log`: focused run of the new regression suite
  `tests/issue_166_numeric_date_parsing_tests.rs`.
- `cargo-clippy-after-fix.log`, `cargo-fmt-check.log`: lint/format gate logs.

## Timeline / Sequence of Events

1. **2026-05-29** — Issue #166 opened by `konard` with the concrete
   reproduction `15.10.2025 + 180 days` and the
   `Unexpected trailing input '.2025'` error, plus the meta-request for a full
   case study, online research, root-cause analysis, codebase-wide fix, and
   downstream issue reports.
2. **Investigation** — Traced the calculation pipeline
   (lexer → token parser → expression parser → evaluator). Confirmed via the
   existing comment in `tests/datetime_issues_tests.rs` and the workaround notes
   in `tests/lino_rate_tests.rs` that ISO dates (`2021-02-08`) were *already
   known* to tokenize as `2021 - 02 - 08` arithmetic rather than as a date —
   i.e. the bug is broader than dots.
3. **Reproduction** — Reproduced all three failure shapes:
   - dot dates → parse error (`.` is not a valid arithmetic operator after a
     number),
   - slash dates → silently computed as division (`15/10/2025 ≈ 0.00074`),
   - dash dates → silently computed as subtraction (`2021-02-08 = 2011`).
4. **Fix** — Added a `DateLiteral` token, lexer recognition with strict
   disambiguation, token-parser handling, and the missing `.`/`-` date formats.
5. **Verification** — Added a 20-test regression suite, updated the two obsolete
   "limitation" tests, and confirmed the full suite, clippy, and fmt are clean.

## Requirements (every requirement extracted from the issue)

| # | Requirement | Status |
|---|-------------|--------|
| R1 | `15.10.2025 + 180 days` must parse and compute (`2026-04-13`). | ✅ Done |
| R2 | `15/10/2025 + 180 days` must parse and compute (`2026-04-13`). | ✅ Done |
| R3 | Support the widest practical range of numeric date formats / locales (dot, slash, dash; day-first, month-first, year-first). | ✅ Done (numeric); month-name formats already existed |
| R4 | Do not break ordinary arithmetic that *looks* like a date (`2026 - 1 - 22`, `1/3 * 3`, `3.14 + 2.86`, `10/2`). | ✅ Done (disambiguation rule) |
| R5 | Compile issue data into `./docs/case-studies/issue-166` and write a deep case-study analysis. | ✅ This document |
| R6 | Search online for additional facts (existing libraries/components). | ✅ See "Library / Component Survey" |
| R7 | Add debug output / verbose mode if root cause cannot be found. | N/A — root cause was found directly from source; no extra tracing needed |
| R8 | If the issue relates to other repositories, file reports there with reproducible examples, workarounds, and fix suggestions. | N/A — the defect is entirely inside this repository's Rust core (see "Codebase-Wide Verification") |
| R9 | Apply the fix everywhere the problem occurs (not just the dot case). | ✅ Done — fixed at the lexer, which covers dot, slash, dash, and the `at <date>` clause |
| R10 | Do everything in the single PR #167 on branch `issue-166-b2fb36372b96`. | ✅ |
| R11 | Add reproducing tests before the fix; document reproduction + tests in the PR. | ✅ |
| R12 | Update the release trigger (version) so the fix ships. | ✅ Version bump in `Cargo.toml` + changelog entry |

## Root Cause Analysis

The calculation pipeline is:

```text
input ──▶ Lexer ──▶ tokens ──▶ TokenParser ──▶ Expression AST ──▶ evaluator
         (src/grammar/lexer.rs)  (src/grammar/token_parser.rs)
```

### Problem 1 — dot dates rejected (the reported symptom)

In `src/grammar/lexer.rs`, the digit branch of `next_token` routed both digits
*and* a leading `.` straight into `scan_number`:

```rust
_ if ch.is_ascii_digit() || ch == '.' => self.scan_number(),
```

`scan_number` reads `15`, stops at the first `.`, and `scan_number` cannot
continue past a second `.`. The `.2025` tail then has no valid interpretation,
so the parser reports **`Unexpected trailing input '.2025' at position 5`**.

### Problem 2 — slash and dash dates silently mis-evaluated

`15/10/2025` lexes cleanly as `15 / 10 / 2025` and evaluates to a tiny fraction;
`2021-02-08` lexes as `2021 - 02 - 08` and evaluates to `2011`. No error is
raised, which is arguably worse than Problem 1 because the wrong answer looks
plausible. This is the same root cause: **the lexer had no concept of a numeric
date literal**, so any digit-separator-digit sequence became arithmetic.

The date parser in `src/types/datetime.rs` *was* capable of parsing
`%Y-%m-%d`, `%m/%d/%Y`, and `%d/%m/%Y`, but it only ran on text that the
*token parser* had already classified as a possible date — which, for numeric
input, never happened. Month-name dates worked only because words don't lex as
arithmetic operators.

### Problem 3 — missing dot/dash format strings

Even once a numeric date reaches `DateTime::parse`, the parser had no
`%d.%m.%Y` / `%Y.%m.%d` / `%d-%m-%Y` formats, so dot and non-ISO dash dates
would still fail.

## Solution

The fix is applied at the **lexer** layer so that every downstream consumer
(standalone dates, `date + duration`, `date - date`, and the `... at <date>`
historical-rate clause) benefits from a single change.

### 1. New token kind — `src/grammar/lexer.rs`

A `DateLiteral(String)` variant was added to `TokenKind`.

### 2. Lexer recognition with strict disambiguation — `src/grammar/lexer.rs`

The digit branch now tries `try_scan_date` first and only falls back to
`scan_number` when it returns `None`. The `.` arm was split out so a bare
leading dot still scans as a number/decimal:

```rust
_ if ch.is_ascii_digit() => {
    if let Some((text, end)) = self.try_scan_date() {
        let token = Token::new(TokenKind::DateLiteral(text.clone()), start, end, text);
        self.pos = end;
        token
    } else {
        self.scan_number()
    }
}
_ if ch == '.' => self.scan_number(),
```

`try_scan_date` accepts a sequence of **exactly three digit groups joined by a
single, consistent separator** (`-`, `/`, or `.`) and applies these guards so
arithmetic is never misclassified:

- All three separators must be identical (`15/10.2025` is rejected).
- It must **not** continue into a fourth group or extra digit (so version
  strings like `1.2.3.4` and numbers like `12.3456` are left alone).
- **Exactly one outer group must be a 4-digit year** (`l1==4 && 1..=2,1..=2`
  for year-first, or `l3==4 && 1..=2,1..=2` for year-last). This is the key
  rule that keeps `2026 - 1 - 22` (spaces ⇒ arithmetic), `1/3`, `10/2`, and
  `3.14` as arithmetic.
- The candidate must parse as a real calendar date via
  `crate::types::DateTime::parse`, so `2026-20-5` (month 20) falls back to
  arithmetic.

Note that the lexer only sees a contiguous run with no spaces, so anything the
user spaces out (`2026 - 1 - 22`) is never even a candidate.

### 3. Token-parser handling — `src/grammar/token_parser.rs`

`parse_primary` consumes a `DateLiteral` before the `Number` branch and builds
an `Expression::DateTime`:

```rust
if let Some(TokenKind::DateLiteral(s)) = self.current_kind() {
    let s = s.clone();
    self.advance();
    return crate::types::DateTime::parse(&s).map(Expression::DateTime);
}
```

### 4. Additional date formats — `src/types/datetime.rs`

`try_parse_date_formats` gained `%d.%m.%Y` (European/German/Russian dot),
`%m.%d.%Y` (US dot), `%Y.%m.%d` (ISO dot), `%d-%m-%Y` (non-ISO day-first dash),
and `%m-%d-%Y` (US dash), in addition to the pre-existing `%Y-%m-%d`,
`%m/%d/%Y`, `%d/%m/%Y`, and the month-name formats.

### Format coverage after the fix

| Separator | Year-first | Day-first | Month-first |
|-----------|-----------|-----------|-------------|
| `-` | `2026-01-22` (ISO) | `15-10-2025` | `01-22-2026` |
| `/` | — | `22/01/2026` | `01/22/2026` |
| `.` | `2025.10.15` | `15.10.2025` | `10.15.2025` |
| month name | — | `22 Jan 2026` / `Jan 22, 2026` (pre-existing) | |

## Codebase-Wide Verification (R9)

- The defect lives entirely in the Rust core. A `grep` for date-parsing logic
  (`parse_from_str`, `%d/%m`, `%m/%d`, `DateLiteral`, `looks_like_datetime`)
  across the repository matches only the Rust source files
  (`src/grammar/lexer.rs`, `src/grammar/token_parser.rs`,
  `src/grammar/datetime_grammar.rs`, `src/types/datetime.rs`,
  `src/types/datetime_parse.rs`).
- The `web/` frontend has **no** date-parsing code; it calls the WASM-compiled
  Rust engine, so fixing the core fixes the web UI as well.
- Because the fix is at the lexer, every numeric-date entry point is covered:
  standalone dates, `date ± duration`, `date - date` differences, and the
  `(... ) at <date>` historical-rate clause (previously documented as a
  limitation in `tests/lino_rate_tests.rs`, now corrected).

## Library / Component Survey (R6)

We deliberately **did not** add a new dependency. The existing `chrono`
dependency already parses every format we need via `NaiveDate::parse_from_str`;
the bug was that numeric dates never reached it. The disambiguation problem
(date vs. arithmetic) is specific to this calculator's grammar and is not
something a general date library solves for us. For reference, the Rust
ecosystem options considered were:

- **[dateparser](https://crates.io/crates/dateparser)** — recognises many
  common formats and returns `chrono::DateTime<Utc>`. Heavier than needed and
  oriented to timestamps/RFC formats; would not solve the
  arithmetic-vs-date ambiguity.
- **[date_time_parser](https://docs.rs/date_time_parser)** — natural-language
  date parsing built on `chrono` + `regex`. Aimed at English prose, not our
  numeric-locale formats.
- **[parse_datetime](https://lib.rs/crates/parse_datetime)** — human-readable
  relative-time strings. Overlaps with our existing duration handling, not the
  numeric-date need.

Conclusion: keeping `chrono` and recognising the literal at the lexer is the
smallest, lowest-risk change.

## Tests

New regression suite `tests/issue_166_numeric_date_parsing_tests.rs` (20 tests),
covering:

- the two exact issue examples (dot and slash) → `2026-04-13`;
- every separator and ordering (ISO dash, US/European slash, European/ISO dot,
  European dash);
- `date + duration`, `date - duration`, and `date - date` (duration result);
- ambiguity guards proving arithmetic is untouched: `2026 - 1 - 22` = `2003`,
  `1/3 * 3` = `1`, `3.14 + 2.86` = `6`, `10/2` = `5`, and `2026-20-5` = `2001`.

Unit tests were added in `src/grammar/lexer.rs` (`test_tokenize_numeric_date_literals`,
`test_tokenize_date_in_expression`, `test_arithmetic_not_treated_as_date`) and in
`src/types/datetime.rs` (`test_parse_dot_date_european`, `test_parse_dot_date_iso`).
The obsolete `test_iso_date_format_limitation` in `tests/lino_rate_tests.rs` was
rewritten as `test_iso_date_format_in_at_clause` to assert the now-correct
behaviour.

## Sources

- [dateparser — crates.io](https://crates.io/crates/dateparser)
- [date_time_parser — docs.rs](https://docs.rs/date_time_parser)
- [parse_datetime — lib.rs](https://lib.rs/crates/parse_datetime)
- [Date and time crates — lib.rs](https://lib.rs/date-and-time)
