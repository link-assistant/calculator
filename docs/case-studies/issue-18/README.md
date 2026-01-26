# Case Study: Issue #18 - Currency Exchange Rate Source Transparency

## Overview

**Issue URL:** https://github.com/link-assistant/calculator/issues/18
**Title:** Проблема с выражением: 0 RUB + 1 USD (Problem with expression: 0 RUB + 1 USD)
**Reported:** 2026-01-25
**Reporter:** @andchir

## Problem Statement

When calculating `0 RUB + 1 USD`, the calculator returns `89.5 RUB` but it's unclear:
1. Where the exchange rate (89.5 RUB per USD) comes from
2. What date the rate is from
3. What source provided the rate

Users need transparency about currency conversion rates for:
- Trust and verification
- Financial accuracy
- Audit purposes
- Educational understanding

## Timeline of Events

| Timestamp | Event |
|-----------|-------|
| 2026-01-25 17:29:01 UTC | Issue reported with expression `0 RUB + 1 USD` |
| 2026-01-25 17:33:21 UTC | @konard commented with technical requirements |

## Root Cause Analysis

### Primary Issue: Web Worker Context Incompatibility

**Critical Finding**: The currency API fetch code used `web_sys::window()` which returns `None` in a Web Worker context. Web Workers don't have a `window` object - they use `WorkerGlobalScope` instead.

In `src/currency_api.rs` (before fix):
```rust
async fn fetch_json(url: &str) -> Result<(String, HashMap<String, f64>), CurrencyApiError> {
    let window = web_sys::window()
        .ok_or_else(|| CurrencyApiError::NetworkError("No window object available".to_string()))?;
    // This always fails in a Web Worker!
    // ...
}
```

The fix requires checking for both Window and WorkerGlobalScope contexts:
```rust
let global = js_sys::global();
let resp_value = if let Some(window) = global.dyn_ref::<web_sys::Window>() {
    JsFuture::from(window.fetch_with_request(&request)).await
} else if let Some(worker) = global.dyn_ref::<web_sys::WorkerGlobalScope>() {
    JsFuture::from(worker.fetch_with_request(&request)).await
} else {
    return Err(CurrencyApiError::NetworkError("Neither Window nor WorkerGlobalScope available".to_string()));
};
```

### Secondary Issue: Missing Rate Application Pipeline (Fixed in PR #38)

The web worker originally fetched exchange rates from the API but **never applied them to the Calculator instance**. This was fixed by adding `update_rates_from_api` call in `web/src/worker.ts`.

### Tertiary Issue: Hardcoded Fallback Rates

The calculator uses **hardcoded exchange rates** in `src/types/currency.rs:188`:

```rust
fn initialize_default_rates(&mut self) {
    // Default fallback rates - these are used only when API rates are unavailable.
    self.set_rate_with_info("USD", "RUB", ExchangeRateInfo::default_rate(89.5));
    // ...
}
```

This creates rates with `source: "default (hardcoded)"` and `date: "unknown"`.

### Tertiary Issue: Data Quality

The `data/currency/usd-rub.lino` file contains a suspicious rate:
- Previous rates (2025-12-30 to 2026-01-24): 75-80 RUB/USD range (legitimate CBR data)
- Last entry (2026-01-25): **89.5 RUB/USD** (matches hardcoded default, NOT real data)

### Additional Issues
1. Historical rates from `.lino` files are not automatically loaded
2. No rate source attribution shown to users
3. Rates are static and don't reflect real-world changes

## Current vs Actual Exchange Rate

| Currency Pair | Hardcoded Rate | Real Rate (fawazahmed0 API) | Discrepancy |
|---------------|---------------|----------------------------|-------------|
| USD/RUB | 89.5 | ~75.75 | ~18% overvalued |

*Note: Real rate fetched from fawazahmed0/currency-api on 2026-01-25*

## Technical Requirements (from @konard)

1. ✅ Use real API for current prices
2. ✅ Use presaved .lino files for historical data
3. ✅ Call API only from Rust in background worker
4. ✅ UI thread should not be blocked
5. ✅ Busy indicator should work as expected
6. ✅ Cover with unit tests and e2e tests

## API Research

### Selected API: fawazahmed0/exchange-api

**Reasons for selection:**
- Supports RUB (Russian Ruble) - Frankfurter API does not
- 200+ currencies including crypto and precious metals
- No API key required
- No rate limits
- JSON format
- Free and open source
- CDN-hosted for reliability

**API Endpoints:**
- Latest rates: `https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/usd.json`
- Historical: `https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@{YYYY-MM-DD}/v1/currencies/usd.json`
- Fallback: `https://{date}.currency-api.pages.dev/v1/currencies/{code}.json`

### Alternative Considered: Frankfurter API

**Pros:**
- ECB (European Central Bank) as data source
- Very reliable
- Good documentation

**Cons:**
- Does NOT support RUB (Russian Ruble)
- Limited to ~30 currencies

## Solution Design

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        UI Layer                              │
│  ┌─────────────┐    ┌────────────┐    ┌────────────────┐   │
│  │ Input Field │    │ Result     │    │ Loading        │   │
│  │             │    │ Display    │    │ Indicator      │   │
│  └─────────────┘    └────────────┘    └────────────────┘   │
└───────────────────────────┬─────────────────────────────────┘
                            │ Web Worker Messages
┌───────────────────────────┼─────────────────────────────────┐
│                     Web Worker                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                    WASM Module                       │   │
│  │  ┌─────────────┐  ┌──────────────┐  ┌────────────┐  │   │
│  │  │ Calculator  │  │ Currency     │  │ Currency   │  │   │
│  │  │ Engine      │  │ Database     │  │ API Client │  │   │
│  │  └─────────────┘  └──────────────┘  └────────────┘  │   │
│  └─────────────────────────────────────────────────────┘   │
└───────────────────────────┬─────────────────────────────────┘
                            │ HTTP/Fetch
┌───────────────────────────┼─────────────────────────────────┐
│                    External APIs                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ fawazahmed0/currency-api (via jsDelivr CDN)          │   │
│  │ - Current rates: @latest/v1/currencies/usd.json      │   │
│  │ - Historical: @{date}/v1/currencies/usd.json         │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Key Components

1. **ExchangeRateInfo** - Struct to track rate metadata
2. **CurrencyApiClient** - Async HTTP client for API calls
3. **Enhanced CurrencyDatabase** - Track rate source and date
4. **Updated Calculation Steps** - Show rate info in steps
5. **Rate Loading State** - Track when rates are being fetched

### Data Flow

1. Calculator initializes with default/cached rates
2. Background worker fetches fresh rates from API
3. Rates are updated with source and timestamp
4. Calculations show rate info: "Using rate 1 USD = 75.75 RUB (fawazahmed0, 2026-01-25)"
5. UI shows loading indicator during rate fetch

## Implementation Details

### Files Modified

1. `src/types/currency.rs` - Add ExchangeRateInfo, rate tracking
2. `src/currency_api.rs` - New module for API client
3. `src/grammar/expression_parser.rs` - Show rate info in steps
4. `web/src/worker.ts` - Handle rate loading messages
5. `web/src/App.tsx` - Show rate loading state

### Tests Added

1. Unit tests for ExchangeRateInfo
2. Unit tests for API parsing
3. Integration tests for rate loading
4. E2E tests for currency conversion display

## References

- [fawazahmed0/exchange-api](https://github.com/fawazahmed0/exchange-api)
- [Frankfurter API](https://frankfurter.dev/)
- [European Central Bank Exchange Rates](https://www.ecb.europa.eu/stats/policy_and_exchange_rates/euro_reference_exchange_rates/html/index.en.html)
