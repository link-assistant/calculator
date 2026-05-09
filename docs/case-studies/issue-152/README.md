# Issue 152 Case Study: `11:00 по МСК`

## Summary

Input:

```text
11:00 по МСК
```

Expected behavior: interpret the expression as 11:00 in Moscow time (MSK) and show it as a time value.

Observed behavior in version 0.15.3:

```text
Error: Parse error: Unexpected identifier: МСК
```

## Collected Data

- `issue.json`: raw GitHub issue data for issue #152.
- `issue-comments.json`: raw issue comments. The issue had no comments when investigated.
- `pr-153.json`: initial PR metadata for the prepared branch.

## External Facts

- Timeanddate lists Moscow Standard Time (MSK) as UTC+3: https://www.timeanddate.com/time/zones/msk
- Public IANA-oriented timezone references map `Europe/Moscow` to abbreviation `MSK` and offset UTC+3: https://date.is/iana-timezones/Europe%2FMoscow

The calculator already models `MSK` as UTC+3, so the bug was not a timezone data issue.

## Timeline

- 2026-05-09T05:01:54.947Z: user reproduced the failure in the web calculator.
- 2026-05-09T05:03:43Z: GitHub issue #152 was created.
- 2026-05-09T05:04:28Z: PR #153 was opened from branch `issue-152-4bc1493a5ff1`.
- 2026-05-09T05:10Z: local investigation reproduced the same parse error.

## Requirements

- Parse the exact reported phrase `11:00 по МСК`.
- Preserve support added by issue #149 for `11 по мск`.
- Treat Cyrillic `мск`/`МСК` as the existing `MSK` timezone.
- Consume the timezone token so Links Notation reflects the complete input.
- Add automated regression coverage for colon-time Russian MSK expressions.
- Record issue data and analysis in `docs/case-studies/issue-152`.

## Root Cause

Issue #149 added a parser branch for the shorthand pattern `N по <timezone>`, where `N` means the hour and the parser constructs `N:00 <timezone>`.

The issue #152 input starts with a colon time. That goes through a different parser branch, `try_parse_time_starting_with_number()`. This branch collected only numbers, identifiers, commas, and colons. It stopped when it reached the `по` token because the lexer classifies `по` as the temporal `At` keyword.

As a result, the parser accepted only `11:00` as the first expression, left `по МСК` unconsumed, and later reported `МСК` as an unexpected identifier.

## Solution Options

- Extend the colon-time token collector to consume `по <timezone>` when the timezone is recognized. This is the smallest fix and reuses existing timezone normalization.
- Preprocess datetime strings in `DateTime::parse()` by removing Russian timezone prepositions. This is broader and could hide syntax that the grammar should handle explicitly.
- Replace abbreviation handling with full IANA timezone support. This could help future historical timezone work, but it is unnecessary for a fixed-abbreviation parse bug and would expand the change substantially.

## Fix

The chosen fix extends the colon-time token collector so `HH:MM` and `HH:MM:SS` style expressions can consume a following temporal preposition plus recognized timezone abbreviation. The collected datetime string becomes `11:00 МСК`, which the existing timezone normalization parses and displays as `11:00:00 MSK`.

## Verification

```text
cargo test --test issue_152_russian_msk_colon_time_tests -- --nocapture
cargo test --test issue_149_russian_msk_time_tests -- --nocapture
cargo test --test issue_115_timezone_conversion_tests
cargo test --test datetime_issues_tests
cargo test --test issue_91_tests
```
