# Case Study: Issue #106 - CI/CD of Currency Rates Update Must Be Fixed

## Issue Description

The automated CI/CD workflow for updating currency exchange rates was silently failing. The workflow appeared to succeed (green check) but the Frankfurter API (ECB data source) returned HTTP 403 Forbidden for all 28 currency pairs. Only the CBR (Russian Central Bank) data was being updated. Additionally, the workflow used deprecated Node.js 20 GitHub Actions and ran only weekly instead of daily.

## Timeline of Events

1. **Workflow runs on schedule** (weekly, Sundays at 00:00 UTC)
2. **Frankfurter API calls fail** with HTTP 403 Forbidden for all 28 ECB currency pairs
3. **CBR API calls succeed** - 14 RUB-based currency pair files get updated
4. **Script exits with code 0** despite Frankfurter failures — errors are printed to stderr but not treated as fatal
5. **Workflow reports success** because `git diff` detects CBR changes, commits, and pushes
6. **Node.js 20 deprecation warning** is logged but hidden in annotations — not visible in normal CI output
7. **Net result**: ECB data goes stale silently; only RUB pairs get weekly updates

Reference CI/CD run: https://github.com/link-assistant/calculator/actions/runs/23376106513

## Root Cause Analysis

### Root Cause 1: Frankfurter API Blocks Python urllib User-Agent

The primary failure was `HTTP Error 403: Forbidden` from the Frankfurter API at `api.frankfurter.app`.

**Investigation**: Testing showed that:
- `curl` requests to both `api.frankfurter.app` and `api.frankfurter.dev/v1/` succeed (HTTP 200)
- Python `urllib.request.urlopen()` gets 403 from both domains

The difference: Python's `urllib` sends a default `User-Agent: Python-urllib/3.x` header. The Frankfurter API (hosted behind Cloudflare or similar CDN) blocks requests with bot-like User-Agent strings.

**Evidence**:
```
# Fails (403):
urllib.request.urlopen("https://api.frankfurter.dev/v1/latest?from=USD&to=EUR")

# Works (200):
req = urllib.request.Request(url, headers={"User-Agent": "calculator-rates-updater/1.0"})
urllib.request.urlopen(req)
```

### Root Cause 2: Silent Failure Mode

The script's error handling printed failures to stderr but always exited with code 0. The workflow's "Check for changes" step saw CBR changes and treated the run as successful. This masked the complete failure of the ECB data source.

### Root Cause 3: Deprecated API Domain

The script used `api.frankfurter.app` while the Frankfurter project has migrated to `api.frankfurter.dev/v1/` as the primary API endpoint (see https://frankfurter.dev/).

### Root Cause 4: Node.js 20 Deprecation

`actions/checkout@v4` and `actions/setup-python@v5` run on Node.js 20, which GitHub will force-migrate to Node.js 24 starting June 2, 2026. This was logged as a workflow annotation but not visible in normal output.

### Root Cause 5: Weekly Schedule Too Infrequent

ECB publishes rates daily around 16:00 CET. The workflow ran weekly (Sundays at 00:00 UTC), meaning 4-5 days of rate updates were missed each week.

## Solutions Implemented

### Fix 1: Set Proper User-Agent Header
Added explicit `User-Agent: calculator-rates-updater/1.0` and `Accept: application/json` headers to `fetch_json()` to avoid being blocked by CDN bot protection.

### Fix 2: Migrate to New API Domain
Changed API URL from `https://api.frankfurter.app/` to `https://api.frankfurter.dev/v1/`.

### Fix 3: Add Error Detection
Added validation in `main()` that checks whether each data source returned at least some data. If either source fails completely, the script exits with code 1, causing the workflow to fail visibly.

### Fix 4: Add Verbose/Debug Mode
Added `--verbose` / `-v` flag and `VERBOSE=true` environment variable support. When enabled, logs request URLs, HTTP status codes, response structure, and retry attempts to stderr.

### Fix 5: Upgrade GitHub Actions
- `actions/checkout@v4` → `actions/checkout@v6` (Node.js 24 support)
- `actions/setup-python@v5` → `actions/setup-python@v6` (Node.js 24 support)
- Added `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true` environment variable

### Fix 6: Daily Schedule
Changed cron from `0 0 * * 0` (weekly Sunday midnight) to `0 17 * * 1-5` (weekdays at 17:00 UTC, after ECB publishes around 16:00 CET).

## Lessons Learned

1. **Bot protection breaks automated scripts**: APIs behind CDNs (Cloudflare, etc.) increasingly block default `urllib`/`requests` User-Agent strings. Always set a descriptive User-Agent.
2. **Silent failures are worse than loud failures**: A script that prints errors but exits 0 gives a false sense of reliability. Data pipelines should fail loudly when a data source is unavailable.
3. **GitHub Actions deprecation warnings are easy to miss**: They appear only as annotations, not in the main log output. Proactive upgrades prevent surprise breakage.
4. **Schedule should match data source frequency**: ECB publishes daily; updating weekly means stale data for most of the week.

## References

- Frankfurter API: https://frankfurter.dev/
- ECB exchange rate publication schedule: https://www.ecb.europa.eu/stats/policy_and_exchange_rates/euro_reference_exchange_rates/html/index.en.html
- GitHub Actions Node.js 20 deprecation: https://github.blog/changelog/2025-09-19-deprecation-of-node-20-on-github-actions-runners/
- actions/checkout releases: https://github.com/actions/checkout/releases
- actions/setup-python releases: https://github.com/actions/setup-python/releases
