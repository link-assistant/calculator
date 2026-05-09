# Issue 149 Case Study: `11 по мск`

## Summary

Input:

```text
11 по мск
```

Expected behavior: interpret the expression as 11:00 in Moscow time (MSK) and show it as a datetime/time value rather than a custom unit.

Observed behavior in version 0.15.2:

```text
Result: 11 по
Links Notation: (11 по)
```

The trailing `мск` token was not part of the parsed expression.

## Timeline

- 2026-05-08T06:50:37.663Z: user reported the expression from the web calculator.
- 2026-05-08T06:51:35Z: GitHub issue #149 was created.
- PR #150 was opened from branch `issue-149-0d91bb1ef22e`.

## Requirements

- Parse Russian timezone phrasing `N по мск`.
- Treat Cyrillic `мск` as the existing MSK timezone.
- Preserve the requested local hour in the displayed timezone.
- Prevent the timezone token from being silently ignored in Links Notation.
- Add an automated regression test.

## Root Cause

The token parser only tried numeric datetime parsing for these number-leading patterns:

- `N:MM`
- `N AM/PM`
- `N <month/date token>`

For `11 по мск`, the lexer did not classify `по` as a datetime cue and the parser interpreted `11 по` as a number with a custom unit. `мск` remained unconsumed, so it disappeared from the result.

A secondary issue appeared after adding the Russian parse path: 24-hour times with timezone offsets were not converted to UTC internally before display. That made `11:00 MSK` display as `14:00:00 MSK`.

## Fix

- Added `по` as a Russian temporal keyword.
- Added a parser branch for `N по <timezone>` that constructs `N:00 <timezone>`.
- Added Cyrillic timezone normalization for `мск` to `MSK`.
- Applied the same UTC adjustment used by 12-hour timezone parsing to 24-hour `%H:%M` and `%H:%M:%S` timezone parsing.
- Added `tests/issue_149_russian_msk_time_tests.rs`.

## Verification

```text
cargo test --test issue_149_russian_msk_time_tests -- --nocapture
cargo test --test issue_115_timezone_conversion_tests
cargo test --test datetime_issues_tests
cargo test --test issue_91_tests
```
