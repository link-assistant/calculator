# Issue 154 Case Study: `12:30 по МСК`

## Summary

Input:

```text
12:30 по МСК
```

Observed behavior in version 0.15.4:

```text
Result: 12:30:00 MSK
Links Notation: (12:30:00 MSK)
```

The parser recognized the Russian MSK expression, but the result view only showed
the source timezone. It did not show the same instant in UTC or in the user's
browser-local timezone.

## Collected Data

- `issue.json`: raw GitHub issue data for issue #154.
- `issue-comments.json`: raw issue comments. The issue had no comments when investigated.
- `pr-155.json`: prepared PR metadata.
- `pr-conversation-comments.json`, `pr-review-comments.json`, `pr-reviews.json`: PR discussion data.
- `ci-runs.json`: recent CI runs for the prepared branch. No runs existed when investigated.

## External Facts

- Timeanddate lists Moscow Standard Time (MSK) as UTC+3:
  https://www.timeanddate.com/time/zones/msk
- MDN documents `Intl.DateTimeFormat.prototype.formatToParts()` as a browser API
  for building formatted date/time strings, including timezone-name parts:
  https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DateTimeFormat/formatToParts

The calculator already modeled `MSK` as UTC+3, so the issue was not a timezone
offset data problem.

## Timeline

- 2026-05-09T07:38:13.716Z: user reproduced the behavior in the web calculator.
- 2026-05-09T07:39:09Z: GitHub issue #154 was created.
- 2026-05-09T07:55:02Z: PR #155 was opened from branch `issue-154-20a345fec0fc`.
- 2026-05-09T08:00Z: local regression test reproduced the missing UTC conversion step.

## Requirements

- Keep parsing `12:30 по МСК` as a valid Moscow-time value.
- Preserve the displayed source time: `12:30:00 MSK`.
- Show the equivalent UTC time by default.
- Show the equivalent browser-local time by default in the web UI.
- Keep Links Notation consuming and normalizing the full timezone expression.
- Add automated coverage for the reported expression and the UI conversion rows.
- Record issue data and analysis in `docs/case-studies/issue-154`.

## Root Cause

Issues #149 and #152 fixed parsing for Russian `по мск` and `HH:MM по МСК`
forms. After those fixes, issue #154 was already parsed into a timezone-aware
`DateTime` value.

The remaining gap was in result presentation:

- `DateTime` display rendered one string in the source timezone.
- `CalculationResult` did not expose structured datetime metadata for the web UI.
- The web UI could not safely calculate browser-local time from only a formatted
  string such as `12:30:00 MSK`.
- Calculation steps did not include the UTC equivalent, so issue reports also
  lacked conversion context.

## Solution Options

- Add `as UTC` to the user's expression automatically. This would change the
  main result and lose the source-time display the user already expects.
- Parse the formatted result string in TypeScript. This is brittle because it
  depends on display text rather than structured data.
- Expose structured datetime metadata from Rust and let the browser format the
  local timezone. This is the chosen solution because the browser is the only
  reliable source for the user's local timezone.

## Fix

- Added UTC conversion formatting methods to `DateTime`.
- Added a `UTC equivalent: ...` calculation step for standalone timezone-aware
  datetime values.
- Added `datetime_result` metadata to `CalculationResult`, including the UTC
  timestamp, source display, UTC display, source timezone, offset, and component
  flags.
- Added frontend formatting with `Intl.DateTimeFormat.formatToParts()` to display
  browser-local and UTC conversion rows under timezone datetime results.
- Added localized labels for the conversion rows.
- Added Rust, TypeScript utility, and React rendering tests.

## Verification

```text
cargo fmt --check
node scripts/check-file-size.mjs
cargo clippy --all-targets --all-features
cargo test --all-features --verbose
cargo test --test issue_154_timezone_conversion_display_tests -- --nocapture
cargo test --test issue_152_russian_msk_colon_time_tests -- --nocapture
cargo test --test issue_149_russian_msk_time_tests -- --nocapture
cargo test --test issue_115_timezone_conversion_tests
wasm-pack build --target web --out-dir web/public/pkg
npm run build
npm test
```

The browser verification screenshot is saved as `issue-154-after.png`.
