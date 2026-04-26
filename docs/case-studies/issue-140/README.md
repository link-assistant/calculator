# Issue 140 Case Study

## Summary

Issue #140 reported that this input failed:

```text
22822 рублей в рупиях на 11 апреля 2026
```

Error:
```
No exchange rate available for RUB/INR on 2026-04-11
```

The parser recognized the expression, but historical currency conversion failed because
historical rates from `.lino` files were never actually loaded in the browser — due to the
WASM export being missing. A second root cause was that the database's exact-date lookup had
no fallback to the previous available rate on weekends/holidays.

## Collected Artifacts

- `issue.json`: issue title, body, timestamps, and inline GitHub issue metadata.
- `issue-comments.json`: issue comments at time of first fix (PR #141).
- `issue-comments-updated.json`: issue comments including the post-v0.14.3 report.
- `pr-141.json`: PR #141 metadata (first fix: prior-date fallback in Rust).
- `pr-141-final.json`: PR #141 after merge.
- `ci-runs.json`: CI runs for `issue-140-4a0789606d78`.
- `cbr-xml-api.html`: CBR XML API documentation snapshot.
- `cbr-xml-daily-2026-04-11.xml`: CBR XML response for `date_req=11/04/2026`.

## Timeline

1. **2026-04-26 17:58 UTC**: Issue #140 opened with the failing Russian RUB→INR expression.
2. **2026-04-26 17:59 UTC**: Draft PR #141 created for branch `issue-140-4a0789606d78`.
3. **2026-04-26 18:03 UTC**: PR #141 commit `e61c6100` — `fix: use prior historical currency rates`.
   - Added `get_historical_rate_info` with fallback to the latest prior date.
   - Fixed `CurrencyDatabase::convert_at_date` to find the closest earlier rate.
4. **2026-04-26 18:38 UTC**: PR #141 merged.
5. **2026-04-26 18:40 UTC**: v0.14.3 released (contains PR #141 fix).
6. **2026-04-26 19:36 UTC**: Issue comment: "It was not fixed in v0.14.3".
   - URL provided to reproduce: expression still failing in the live calculator.
7. **2026-04-26 (PR #144)**: Root cause 2 identified: `load_rates_from_consolidated_lino`
   missing `#[wasm_bindgen]` — historical rates never loaded in the browser.
   - Fix: Added `#[wasm_bindgen]`-annotated method in the wasm impl block.

## Requirements

1. Use the exact historical rate when it exists for the requested date.
2. Fall back to the latest available rate before the requested date (for weekends/holidays).
3. Fail with a clear error if no rate exists on or before the requested date.
4. Do not silently fall back to current/default rates for explicitly dated conversions.
5. Historical `.lino` files must be loaded into the WASM calculator via the web worker.
6. Keep rate metadata in calculation steps so users see which effective date was used.

## Root Causes

### Root Cause 1: Missing prior-date fallback in `CurrencyDatabase` (fixed in PR #141)

`CurrencyDatabase::convert_at_date` only looked up `(from, to, exact_date)` in
`historical_rates`. If that key was missing (e.g., Saturday 2026-04-11), it returned
`NoHistoricalRate` instead of using the latest known prior rate.

**Fix**: Added `get_historical_rate_info` helper that first tries the exact date, then scans
for the maximum rate date ≤ requested date for the same currency pair.

### Root Cause 2: `load_rates_from_consolidated_lino` not exported via WASM (fixed in PR #144)

The method `load_rates_from_consolidated_lino` was defined in a plain `impl Calculator` block
(not annotated with `#[wasm_bindgen]`), so it was never compiled into the WASM binary.

In the browser, the web worker called:
```js
loadedHistoricalRateCount += calculator.load_rates_from_consolidated_lino(content);
```
This silently threw `TypeError: calculator.load_rates_from_consolidated_lino is not a function`,
which was caught by the try/catch in `loadCbrRatesFromLinoFiles` and logged only as a debug
message. Historical rates were never loaded. Every dated RUB conversion then failed with
`No exchange rate available` even though the data was present in the `.lino` files.

**Evidence**: After building WASM before the fix:
```
$ grep "load_rates_from_consolidated_lino" web/public/pkg/link_calculator.d.ts
(no output — method not present)
```
After the fix:
```
$ grep "load_rates_from_consolidated_lino" web/public/pkg/link_calculator.d.ts
    load_rates_from_consolidated_lino(content: string): number;
```

**Fix**: Added a `#[wasm_bindgen]` wrapper in the `#[wasm_bindgen] impl Calculator` block:
```rust
#[wasm_bindgen]
pub fn load_rates_from_consolidated_lino(&mut self, content: &str) -> usize {
    match self.load_rates_from_consolidated_lino_impl(content) {
        Ok(n) => n,
        Err(_) => 0,
    }
}
```
The return type is `usize` (0 on error) instead of `Result<usize, String>` for WASM
compatibility. The internal implementation keeps its `Result` return type and is called by
the WASM wrapper.

## External Data Notes

The CBR XML response in `cbr-xml-daily-2026-04-11.xml` confirms CBR does publish rates for
April 11, 2026 (a Saturday — unusual). The `inr-rub.lino` data file also contains a rate for
that exact date. Both root causes needed to be fixed: the WASM export (so data is loaded at
all) and the prior-date fallback (for future dates where no rate is available).

## Online Resources Consulted

- CBR XML API: `https://www.cbr.ru/scripts/XML_daily.asp?date_req=11/04/2026`
- CBR API documentation: `https://www.cbr.ru/development/SXML/`

## Test Coverage

### PR #141 (Rust fallback logic)
- `current_cbr_rate_is_not_used_for_a_different_historical_date`
- `current_cbr_rate_is_available_for_its_own_effective_date`
- `original_issue_expression_uses_april_11_cbr_lino_rate`
- `missing_weekend_rate_uses_previous_available_business_day_rate`
- `missing_historical_rate_before_first_known_date_still_fails`

### PR #144 (WASM export fix)
- `load_rates_returns_count_not_result` — verifies the usize return type
- `issue_140_rub_to_inr_on_april_11_2026_succeeds` — end-to-end regression test
- `load_rates_returns_zero_for_empty_content` — error handling
