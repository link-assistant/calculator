---
bump: minor
---

### Fixed
- CBR exchange rates now load correctly in the browser via local `.lino` file fallback. The direct cbr.ru API call fails with a CORS error in browsers (cbr.ru does not return `Access-Control-Allow-Origin` headers), so the web worker now falls back to loading pre-downloaded `.lino` files served from GitHub Pages at `/calculator/data/currency/`. Fixes #73.

### Added
- Currency data files (`data/currency/*.lino`) are now served via GitHub Pages at `https://link-assistant.github.io/calculator/data/currency/`. A copy step was added to the `web-build` CI job to include these files in the Vite build output.
- Indian Rupee (INR) added to the CBR historical rates download script (`scripts/download_historical_rates.py`), generating `inr-rub.lino` and `rub-inr.lino` files with direct CBR rates. Previously INR→RUB required triangulation via USD, now uses a direct CBR rate.
- Deep case study analysis for issue #73 documenting the CORS root cause, timeline reconstruction, data analysis, and proposed solutions. See `docs/case-studies/issue-73/README.md`.
- Tests for the issue #73 expression `10 RUB + 10 USD + 11 INR` verifying CBR rate injection, step source attribution, and regression against the incorrect hardcoded result.
