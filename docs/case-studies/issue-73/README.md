# Case Study: Issue #73 — Expression `10 RUB + 10 USD + 11 INR` Uses Hardcoded Rates Instead of CBR

## Problem Statement

**Input:** `10 RUB + 10 USD + 11 INR`
**Reported Result:** `916.38150289017341 RUB`
**URL:** https://link-assistant.github.io/calculator/?q=KGV4cHJlc3Npb24lMjAlMjIxMCUyMFJVQiUyMCUyQiUyMDEwJTIwVVNEJTIwJTJCJTIwMTElMjBJTlIlMjIp

**Reported Steps from the App:**
```
5. Exchange rate: 1 USD = 89.5 RUB (source: default (hardcoded), date: unknown)
9. Exchange rate: 1 INR = 1.0346820809248554 RUB (source: default (hardcoded) (via USD), date: unknown)
```

**Questions from Issue Author:**
1. Why aren't CBR rates tested in CI/CD?
2. Why aren't `usd-rub.lino` and `rub-usd.lino` files re-exported at a public URL (e.g., `https://link-assistant.github.io/calculator/data/currency/rub-usd.lino`)?
3. Is there a CI/CD error preventing the app from accessing updated rates?

---

## Timeline and Sequence of Events

### Phase 1: Initial State (before Issue #70)
- Calculator used **hardcoded default rates** for all currency conversions.
- `1 USD = 89.5 RUB` — hardcoded constant.
- `1 INR = 1.034... RUB` — derived via triangulation: INR→USD (ECB) × USD→RUB (hardcoded).

### Phase 2: Issue #70 Fix — CBR Integration Added (2026-02-25 ~12:39 UTC)
- PR merged as commit `59ffb2e0` introduced `fetch_cbr_rates()` WASM function.
- CI runs [22397233029](https://github.com/link-assistant/calculator/actions/runs/22397233029) and following failed due to **clippy lint errors** before the fix was stable.
- Final successful merge at commit `290c4235` (2026-02-25T17:05:15Z).

### Phase 3: Issue #73 Reported (2026-02-25T17:12:55.553Z)
- The app still shows **hardcoded rates** for `10 RUB + 10 USD + 11 INR`.
- This is despite the CBR integration code existing in the codebase.

---

## Root Cause Analysis

### Root Cause #1: CBR API Does NOT Support CORS

**The primary root cause** is that `https://www.cbr.ru/scripts/XML_daily.asp` does NOT return
`Access-Control-Allow-Origin` headers.

**Verified evidence:**
```bash
$ curl -I "https://www.cbr.ru/scripts/XML_daily.asp" -H "Origin: https://link-assistant.github.io"
HTTP/1.1 200 OK
allow: GET,POST
x-frame-options: SAMEORIGIN
# No Access-Control-Allow-Origin header!
```

**Impact:** Any browser-based JavaScript (including WASM running in a Web Worker) that tries
`fetch("https://www.cbr.ru/scripts/XML_daily.asp")` will receive a CORS error:
```
Access to fetch at 'https://www.cbr.ru/scripts/XML_daily.asp' from origin
'https://link-assistant.github.io' has been blocked by CORS policy: No
'Access-Control-Allow-Origin' header is present on the requested resource.
```

The web worker's `fetchCbrRates()` call silently falls back (logs a `console.warn`) and the
calculator continues with hardcoded defaults.

**Contrast with Frankfurter API** (ECB), which DOES support CORS:
```bash
$ curl -I "https://api.frankfurter.app/latest?from=USD" -H "Origin: https://link-assistant.github.io"
access-control-allow-origin: *
access-control-allow-methods: GET, OPTIONS
```

### Root Cause #2: Data Files Not Served via GitHub Pages

The `.lino` rate files in `data/currency/` (e.g., `usd-rub.lino`, `rub-usd.lino`) contain
historical CBR rates updated weekly by the `update-currency-rates.yml` workflow.

However, these files are **not accessible via GitHub Pages** because:

1. The Vite build only serves files from `web/public/` and source files — it does NOT copy `data/currency/`.
2. The GitHub Pages deployment uploads only `web/dist/` (the Vite build output).
3. The `data/currency/` directory lives at the repo root, not in `web/public/`.

**Verified:**
```bash
$ curl -s -o /dev/null -w "%{http_code}" "https://link-assistant.github.io/calculator/data/currency/usd-rub.lino"
404
```

This means the app has no alternative way to load CBR-sourced rates in the browser. The web
worker could theoretically load `.lino` files via `fetch()` from a same-origin URL, but these
files are simply not deployed.

### Root Cause #3: INR Missing from CBR Historical Download

The `scripts/download_historical_rates.py` script only downloads 6 currencies from CBR:
```python
CBR_CURRENCIES = {
    "R01235": "USD",
    "R01239": "EUR",
    "R01035": "GBP",
    "R01820": "JPY",
    "R01775": "CHF",
    "R01375": "CNY",
}
```

**INR (Indian Rupee, code `R01270`) is not included.** The CBR API does provide INR rates, as
confirmed by the live API response:
```xml
<Valute ID="R01270">
  <CharCode>INR</CharCode>
  <Nominal>100</Nominal>
  <Value>84,0766</Value>  <!-- 100 INR = 84.0766 RUB, so 1 INR ≈ 0.840766 RUB -->
</Valute>
```

Without a `inr-rub.lino` or `rub-inr.lino` file, the app falls back to triangulation via USD
(ECB INR→USD rate × hardcoded USD→RUB), which gives the inaccurate result `1 INR = 1.034... RUB`.

### Root Cause #4: CBR Rates Not Tested Against Real Calculation Steps

The existing tests in `tests/issue_70_cbr_rates_tests.rs` inject mock CBR rates via
`update_cbr_rates_from_api()` — they don't test that the web worker actually fetches live CBR
rates before a user performs a calculation. There's a race condition: if the user calculates
before `fetchCbrRates()` completes (or if CBR is blocked by CORS), the calculator uses defaults.

---

## Data Collected

### Live CBR Rate (2026-02-26)
```
1 USD = 76.4678 RUB
1 EUR = 90.3211 RUB
1 INR = 0.840766 RUB  (100 INR = 84.0766 RUB)
```

### Hardcoded Rate in Codebase (issue-causing default)
```
1 USD = 89.5 RUB  (outdated/incorrect)
1 INR = 1.034... RUB  (via USD triangulation with hardcoded USD rate)
```

### Expected Correct Result for `10 RUB + 10 USD + 11 INR` (using CBR 2026-02-26)
```
10 RUB
+ 10 USD × 76.4678 = 764.678 RUB
+ 11 INR × 0.840766 = 9.248426 RUB
= 783.926426 RUB  (approximately)
```
vs. reported hardcoded result: `916.38 RUB` — a difference of ~132 RUB (~14%)!

### CI/CD Failure Logs (from Phase 2 above)

CI runs [22397233029](https://github.com/link-assistant/calculator/actions/runs/22397233029),
[22397267940](https://github.com/link-assistant/calculator/actions/runs/22397267940),
[22397392929](https://github.com/link-assistant/calculator/actions/runs/22397392929) all failed
with clippy lint errors during PR #71 integration. Full logs: see `ci-logs/` directory.

The `update-currency-rates.yml` workflow runs **weekly (Sunday 00:00 UTC)** and successfully
downloads and commits updated `.lino` files to the repo's `data/currency/` directory. The CI
itself is not broken — the files are being updated. The problem is that these files are never
served to the browser.

---

## Proposed Solutions

### Solution A (Recommended): Serve `.lino` Files via GitHub Pages

**Copy `data/currency/` to `web/public/data/currency/`** so that Vite includes them in the
build output and they become accessible at:
`https://link-assistant.github.io/calculator/data/currency/usd-rub.lino`

**Steps:**
1. Add a build step in `web-build` CI job to copy `data/currency/` into `web/public/data/currency/`
   before running `npm run build`.
2. Modify the web worker to try loading `.lino` files as a fallback when CBR CORS fails.
3. Add INR to `CBR_CURRENCIES` in `scripts/download_historical_rates.py` so `inr-rub.lino`
   and `rub-inr.lino` are downloaded and served.

**Advantages:**
- Works around CBR's CORS restriction entirely.
- Uses official CBR-sourced data (already downloaded by CI).
- Same-origin request from browser — no CORS issues.
- Data is up to date (weekly CI updates + commit).
- Enables future offline-capable PWA features.

**Implementation:**
- In `web-build` workflow step, add: `cp -r data/currency web/public/data/currency`
- In the web worker, after CBR fetch fails, attempt `fetch('/calculator/data/currency/usd-rub.lino')`
  and parse the `.lino` file using the existing `parse_consolidated_lino_rates` WASM function.

### Solution B: Use a CORS Proxy for CBR

Deploy a simple proxy (e.g., Cloudflare Worker, AWS Lambda) that fetches from `cbr.ru` and
adds CORS headers. The calculator then fetches from the proxy instead of directly from CBR.

**Known proxy options:**
- [cors-anywhere](https://github.com/Rob--W/cors-anywhere) — open source CORS proxy
- [allorigins.win](https://allorigins.win) — public CORS proxy (reliability concerns)
- Cloudflare Workers — first 100k requests/day free

**Disadvantages:**
- Introduces a third-party dependency.
- Public proxies are unreliable.
- Self-hosted proxy requires maintenance.
- Not necessary if Solution A is implemented.

### Solution C: Add CORS Headers to CBR (External Report)

Report the CORS issue to the Central Bank of Russia so they add `Access-Control-Allow-Origin: *`
to their XML API responses.

**Reference:** The CBR has a feedback form at https://www.cbr.ru/about/contacts/

**Issue details for the report:**
- API endpoint: `https://www.cbr.ru/scripts/XML_daily.asp`
- Problem: Missing `Access-Control-Allow-Origin` header prevents browser-based applications
  from fetching exchange rates.
- Request: Add `Access-Control-Allow-Origin: *` header to allow web applications to use the API.
- Workaround in the meantime: Use server-side fetching or CORS proxies.

Note: Government agencies are unlikely to respond quickly or implement this change.

### Solution D: Add Debug/Verbose Logging for Rate Fetch Failures

Add a `ratesStatus` message to the web worker that tells the UI when CBR fetch fails and why.
This would allow users and developers to see the actual CORS error in the UI rather than silently
falling back to hardcoded defaults.

Currently, CBR failures are only logged as `console.warn` — invisible to users.

---

## Implementation Plan (Selected Solutions)

This case study implements **Solution A** (serve `.lino` files via GitHub Pages) combined with
**Solution D** (verbose error logging):

1. **Add INR to CBR download** (`scripts/download_historical_rates.py`) — adds `inr-rub.lino` and `rub-inr.lino` to the data directory.
2. **Copy data files to web/public** in the CI `web-build` step — makes them accessible at the GitHub Pages URL.
3. **Fallback in web worker** — when CBR CORS fails, load `.lino` files from GitHub Pages instead.
4. **Add verbose error logging** — log CBR CORS error details to console and send to UI.
5. **Add tests** for issue #73 expression with step verification.

---

## References

- Issue #73: https://github.com/link-assistant/calculator/issues/73
- Issue #70 (CBR integration): https://github.com/link-assistant/calculator/issues/70
- PR #71 (CBR fix merged): https://github.com/link-assistant/calculator/pull/71
- CBR XML API: https://www.cbr.ru/scripts/XML_daily.asp
- CBR Dynamic XML API: http://www.cbr.ru/scripts/XML_dynamic.asp
- Frankfurter API (ECB): https://frankfurter.dev/
- CORS specification: https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS
- `src/currency_api.rs` — CBR XML parsing and rate fetching
- `web/src/worker.ts` — Web worker that calls `fetch_cbr_rates()`
- `scripts/download_historical_rates.py` — CBR rate downloader for CI
- `data/currency/usd-rub.lino` — Historical USD→RUB rates from CBR
- `.github/workflows/update-currency-rates.yml` — Weekly rate update CI job
- `.github/workflows/release.yml` — Main CI/CD pipeline with `web-build` and `deploy-pages` jobs
