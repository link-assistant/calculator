# Issue 134 Case Study: Russian Partial Date Subtraction

## Source Data

- Issue: <https://github.com/link-assistant/calculator/issues/134>
- Pull request: <https://github.com/link-assistant/calculator/pull/135>
- Local artifacts:
  - `issue.json`
  - `issue-comments.json`
  - `pr-135.json`
  - `pr-conversation-comments.json`
  - `pr-review-comments.json`
  - `pr-reviews.json`
  - `ci-runs.json`
  - `focused-test.log`
  - `full-cargo-test.log`

No issue comments were present when this case study was prepared. `ci-runs.json` was empty for branch `issue-134-bbb0be33c9fa`, so there were no branch CI logs to download.

## Timeline

- 2026-04-18: Issue #134 reported that `18 апреля - 28 марта` failed with `Unit mismatch: cannot subtract 'апреля' and 'марта'`.
- 2026-04-18: PR #135 existed as a draft solution branch for the issue.
- 2026-04-18: Local reproduction showed the expression fell through to generic unit arithmetic instead of date interval arithmetic.
- 2026-04-18: Regression tests were added for the reported expression and standalone partial Russian date parsing.
- 2026-04-18: Parser fix was implemented and verified with focused date tests plus the full cargo test suite.

## Requirements

1. `18 апреля - 28 марта` must parse as subtraction between two dates, not subtraction between custom units.
2. Partial dates without an explicit year should be accepted when the day and month are present.
3. Russian month names in genitive form, such as `апреля` and `марта`, must continue to work through the existing multilingual month translation path.
4. The result should be a valid time interval.
5. Existing full Russian date arithmetic, such as `17 февраля 2027 - 6 месяцев`, must keep working.
6. All collected issue and PR data should be stored under `docs/case-studies/issue-134`.

## Root Cause

The date parser already had a helper for partial dates (`parse_partial_date`) that fills the missing year from the current UTC year. However, `DateTime::parse` only used that helper inside date-time combinations such as `Jan 27, 8:59am UTC`; it did not use it for standalone date inputs.

There was a second gap in `parse_partial_date`: it supported day plus abbreviated month (`27 Jan`) but not day plus full month (`27 January`). Russian `18 апреля` is translated to `18 April`, so it still failed after translation.

Finally, `DateTimeGrammar::try_parse_datetime_subtraction` only handled the explicit parenthesized form `(date) - (date)`. The reported input was unparenthesized, so the parser interpreted `18 апреля` and `28 марта` as numbers with custom units and then raised a unit mismatch.

## Online And Library Research

The implementation uses `chrono::NaiveDate` for concrete calendar dates. Chrono's documented parsing API works with concrete dates through `NaiveDate::parse_from_str`; partial dates require the application to provide missing context such as the year before creating a `NaiveDate`. This matches the repository's existing approach: `parse_partial_date` supplies the current UTC year explicitly.

Potential libraries for broader natural-language date parsing include `rustling-ontology` and higher-level date parsers, but this issue does not need a new dependency. The existing parser already translates multilingual month names and has a local partial-date helper; extending those paths is lower risk than adding a new parsing engine.

## Solution

The fix has three parts:

1. `DateTime::parse` now calls `parse_partial_date` for standalone date input after full date format attempts.
2. `parse_partial_date` now accepts full month names in day-month order (`%d %B %Y`), covering translated Russian inputs like `18 April`.
3. `DateTimeGrammar::try_parse_datetime_subtraction` now also recognizes unparenthesized `date - date` when both sides independently parse as `DateTime`.

The unparenthesized subtraction path is intentionally guarded by successful parsing of both sides as dates. If either side is not a date, normal expression parsing continues.

## Verification

- `cargo fmt --check`
- `cargo test --test issue_134_russian_partial_date_subtraction_tests -- --nocapture`
- `cargo test --test issue_125_russian_date_arithmetic_tests -- --nocapture`
- `cargo test --test datetime_issues_tests -- --nocapture`
- `cargo test --test issue_128_calendar_month_arithmetic_tests -- --nocapture`
- `cargo test -- --nocapture`

The focused regression test verifies that `18 апреля - 28 марта` returns `21 days` on the 2026 current-year context used during this work.
