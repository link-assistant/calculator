# Issue 138 Case Study: Dated RUB to INR Conversion Used Latest CBR Rate

## Source Data

- Issue: <https://github.com/link-assistant/calculator/issues/138>
- Pull request: <https://github.com/link-assistant/calculator/pull/139>
- Bank of Russia daily page for 11.04.2026: <https://www.cbr.ru/eng/currency_base/daily/?UniDbQuery.Posted=True&UniDbQuery.To=11.04.2026>
- Bank of Russia XML API documentation: <https://www.cbr.ru/development/SXML/>
- Local artifacts:
  - `issue.json`
  - `issue-comments.json`
  - `pr-139.json`
  - `pr-conversation-comments.json`
  - `pr-review-comments.json`
  - `pr-reviews.json`
  - `pr-diff.patch`
  - `ci-runs.json`
  - `cbr-daily-2026-04-11.html`
  - `cbr-xml-daily-2026-04-11.xml`

No issue comments, PR comments, PR reviews, or branch CI runs were present when this case study was prepared.

## Timeline

- 2026-04-25: PR #137 merged a core fix for parsing temporal unit conversions and passing date context into currency conversion.
- 2026-04-26 08:05 UTC: Version 0.14.1 in Safari evaluated `22822 рублей в рупиях на 11 апреля 2026`.
- 2026-04-26 08:07 UTC: Issue #138 reported that the expression was interpreted with `at (2026-04-11)` but the exchange-rate step used the 2026-04-24 CBR rate.
- 2026-04-26: Local investigation found the browser CBR `.lino` fallback loaded only each file's latest row as a current rate.
- 2026-04-26: Regression tests and the fix were added on branch `issue-138-15a486ad2ff3`.

## Requirements

1. `22822 рублей в рупиях на 11 апреля 2026` must use the CBR rate effective on 2026-04-11.
2. The calculation steps must show `date: 2026-04-11`, not the latest available CBR date.
3. Browser CBR fallback loading must preserve historical `.lino` rows, not only latest current rates.
4. A dated conversion must not silently use a current/latest rate for a different requested date.
5. Current API rates should remain usable for their own effective date.
6. Issue and PR data, plus relevant external CBR data, should be stored under `docs/case-studies/issue-138`.
7. The fix must include automated regression coverage.

## Root Cause

The prior parser and evaluator fix produced the right AST: `(((22822 RUB) as INR) at (2026-04-11))`. The remaining failure was data loading.

In `web/src/worker.ts`, the CBR fallback fetched local `.lino` files such as `inr-rub.lino`, parsed all rows, then kept only the final row and called `update_cbr_rates_from_api()` with that latest value. For the reported case, that latest row was 2026-04-24. No 2026-04-11 historical rate was loaded into the WASM calculator.

The core currency database then made the bug visible by falling back from `convert_at_date()` to current rates when no exact historical rate existed. That fallback produced a successful but date-inaccurate result instead of either using historical data or reporting a missing historical rate.

## Online Research

Bank of Russia documents the XML daily endpoint with `date_req=dd/mm/yyyy`; if the parameter is absent, the latest registered date is returned. This confirms historical CBR requests are date-specific.

The official 11.04.2026 Bank of Russia daily page lists INR as `100 Indian Rupee = 83.0794 RUB`. The repository's `data/currency/inr-rub.lino` stores that as `1 INR = 0.830794 RUB`, and the inverse used by the issue expression is `1 RUB = 1.203667816570654 INR`.

## Solution

1. Added `web/src/worker-cbr-lino-loader.ts` so CBR fallback loading is testable and preserves the full `.lino` content.
2. The worker now calls `calculator.load_rates_from_consolidated_lino(content)` for each CBR `.lino` file, loading all historical rows into WASM.
3. The worker still applies latest `.lino` rows as current fallback rates when direct CBR fetch fails.
4. When direct CBR fetch succeeds, the worker loads local `.lino` history without overwriting fresher direct current rates.
5. `CurrencyDatabase::convert_at_date()` no longer falls back to current rates for a different requested date.
6. API rate updates now also register the response date as a historical rate, so the current API value works for its own effective date.

## Verification

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features`
- `cargo test --test issue_138_historical_cbr_rates_tests -- --nocapture`
- `cargo test --test issue_136_temporal_unit_conversion_tests -- --nocapture`
- `cargo test --test lino_rate_tests -- --nocapture`
- `cargo test --all-features --verbose`
- `node scripts/check-file-size.mjs`
- `cd web && npx tsc --noEmit`
- `cd web && npm test -- --run src/worker-cbr-lino-loader.test.ts`
- `cd web && npm test`

The new original-issue regression verifies that the result uses 2026-04-11 and evaluates to approximately `27470.106909775466 INR`, not the old 2026-04-24 result `28691.14713044528 INR`.
