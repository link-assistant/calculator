# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- changelog-insert-here -->









## [0.5.1] - 2026-03-02

### Fixed

- Improved Links Notation for indefinite integrals to be more explicit and unambiguous. The differential variable is now shown as `(differential of (x))` and the multiplication between the integrand and differential is made explicit. For example, `integrate cos(x) dx` now produces `(integrate ((cos (x)) * (differential of (x))))` instead of `(integrate (cos (x)) dx)`.

## [0.5.0] - 2026-03-01

### Added
- Support for `=` as an equality check operator in expressions (e.g., `1 * (2 / 3) = (1 * 2) / 3` returns `true`)
- Previously, using `=` in an expression would throw `Parse error: Unexpected character '=' at position N`
- Both sides of the equality are evaluated and compared, returning `true` or `false`

## [0.4.1] - 2026-02-28

### Fixed
- Clicking an alternative interpretation now triggers recalculation instead of only visually highlighting the selected button
- Unified `.lino-alt-button` appearance with `.lino-value`: consistent font-family, font-size, text color in dark mode, and larger gap between items

## [0.4.0] - 2026-02-27

### Fixed
- Function calls now render in proper links notation: `integrate(x^2, x, 0, 3)` → `(integrate ((x ^ 2) x 0 3))` instead of keeping mathematical notation
- Power expressions now wrap in parentheses with spaces: `2^3` → `(2 ^ 3)` instead of `2^3`
- All compound expressions are now wrapped in outer `()` in links notation for consistency
- Zero-argument functions render as `(pi)` instead of bare `pi`
- Indefinite integrals use proper lino in symbolic results: `(integrate (x ^ 2) dx)`

### Added
- Alternative interpretation support for ambiguous expressions
  - Expressions with mixed operator precedence show alternative groupings (e.g., `2 + 3 * 4` shows both `(2 + (3 * 4))` and `((2 + 3) * 4)`)
  - Function calls show both links notation and traditional mathematical notation
- UI allows clicking between alternative interpretations with visual selection indicator
- New examples for expressions with multiple interpretations

## [0.3.0] - 2026-02-26

### Added
- Support `now` keyword in expressions, e.g. `(Jan 27, 8:59am UTC) - (now UTC)` (issue #34)
- Support `until <datetime>` syntax for countdown durations, e.g. `until Jan 27, 11:59pm UTC` (issue #23)
- Parse timezone abbreviations (EST, PST, CET, etc.) in datetime expressions (issue #23)
- Parse ordinal date suffixes (1st, 2nd, 3rd, 26th) and strip day names (Monday, Tuesday, etc.) (issue #23)
- Support `UTC time`, `time UTC`, `current time`, and `current UTC time` inputs (issue #68)
- Show "Time since" / "Time until" elapsed/remaining duration for standalone datetime inputs (issue #45)
- Wrap standalone datetime values in parentheses in links notation (issue #45)

## [0.2.1] - 2026-02-26

### Fixed
- Fixed `deploy-after-release` CI/CD job failing with 409 Conflict when uploading the GitHub Pages artifact (issue #78)
  - Root cause: `actions/upload-pages-artifact` always uses the artifact name `github-pages` by default. When both `deploy-pages` and `deploy-after-release` jobs run in the same workflow, the second upload fails because the name is already taken.
  - Fix: pass `name: github-pages-after-release` to `upload-pages-artifact` and `artifact_name: github-pages-after-release` to `deploy-pages` in the `deploy-after-release` job, so it uses a unique artifact name.

## [0.2.0] - 2026-02-26

### Added

- Added `deploy-after-release` CI/CD job that rebuilds and redeploys the web app with the correct version immediately after auto-release bumps `Cargo.toml` (fixes version display in web footer)
- Added crates.io badge to README

### Fixed

- Fixed web app footer showing stale version (e.g., `v0.1.0`) after a release: the WASM now gets recompiled with the updated `CARGO_PKG_VERSION` after each version bump, so the deployed web app always displays the current release version
- Fixed `scripts/version-and-commit.mjs` not updating `Cargo.lock` after bumping the version in `Cargo.toml`, which caused `cargo package --list` to fail with "files in the working directory contain changes that were not yet committed"
- Fixed CI/CD badge URL in README to match the actual workflow name

## [0.1.1] - 2026-02-26

### Fixed

- Fixed auto-release pipeline never committing version bumps to git (issue #78)
  - Root cause: `command-stream`'s `$` has `errexit: false` by default, so `git diff --cached --quiet` never threw an exception even when there were staged changes
  - Fix: replaced `try/catch` approach with explicit `diffResult.code === 0` check in `scripts/version-and-commit.mjs`
- Cleaned up accumulated changelog fragments that were never committed to git
- Updated `CHANGELOG.md` with all accumulated changes
- Added case study analysis in `docs/case-studies/issue-78/`

## [0.2.0] - 2026-02-25

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Added
- Implement Link Calculator with Rust WebAssembly core
- Grammar-based expression parser supporting arithmetic operations (+, -, *, /)
- DateTime parsing with multiple formats (ISO, US, European, month names)
- Currency support with exchange rates and temporal awareness
- Links notation representation for all expressions
- Step-by-step calculation explanations
- Issue prefill links for unrecognized input
- React frontend with Web Worker for async WASM calculations
- GitHub Actions CI/CD pipeline for testing and GitHub Pages deployment
- Currency database with historical exchange rates in links-notation format

### Fixed
- Fixed error "Cannot add duration and datetime" when adding a duration to a datetime (issue #8)
  - The expression `(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC) + (Jan 25, 12:51pm UTC)` now works correctly
  - Addition of Duration + DateTime is now supported (previously only DateTime + Duration worked)

### Fixed
- Fixed settings dropdown menu appearing behind calculator card due to CSS stacking context issue (#10)

### Added
- Full i18n (internationalization) support for error messages in all 7 languages (English, Russian, Chinese, Hindi, Arabic, German, French)
- Error information with translation keys (`error_info`) in `CalculationResult` for frontend localization
- Calculation step translation keys (`steps_i18n`) for future step-by-step translation support
- GitHub issue report localization - issue titles and report content are now translated to the user's selected language
- New translation sections: `errors` (16 error types), `steps` (15 step types), `issueReport` (19 labels), and `common` (3 common terms)

### Changed
- Links notation interpretation is now displayed before the result value in the UI (similar to Wolfram Alpha layout)
- Error messages in the UI are now translated using i18n keys when available, with fallback to raw error messages
- `CalculatorError` now includes `to_error_info()` method that provides translation keys and interpolation parameters
- GitHub issue URL generation now accepts an optional translation function for localized reports

### Fixed
- Error messages were previously displayed in English only - now they respect the user's language preference

### Added
- Natural integral notation support: `integrate sin(x)/x dx` now parses correctly
- Symbolic integration results for common functions (sin, cos, exp, polynomials, sin(x)/x -> Si(x))
- LaTeX formula rendering using KaTeX for mathematical expressions
- Canvas-based function plotting for integral visualizations
- New `IndefiniteIntegral` expression type for symbolic integrals
- New `MathRenderer` and `FunctionPlot` React components

### Changed
- Examples in the calculator UI now include `integrate sin(x)/x dx`
- `CalculationResult` extended with `latex_input`, `latex_result`, `is_symbolic`, and `plot_data` fields

### Fixed
- Issue #3: "integrate sin(x)/x dx" no longer returns "Parse error: Unexpected identifier: integrate"

### Added
- Exchange rate transparency: show source, date, and fetch time for currency conversions
- Real-time exchange rate fetching from official Central Bank APIs (ECB via frankfurter.dev, CBR via cbr.ru)
- WASM bindings for `fetch_exchange_rates` and `fetch_historical_rates` functions
- Exchange rate loading indicator in the web UI
- E2E tests for currency conversion with real rates

### Changed
- Currency calculations now display exchange rate info in calculation steps
- CurrencyDatabase now tracks the last used rate information for transparency

### Added
- New Rational type for exact fractional arithmetic using `num-rational` crate
- Repeating decimal detection algorithm for proper display of fractions
- Extended ValueKind enum with Rational variant for symbolic computation

### Fixed
- Expression `(1/3)*3` now correctly returns `1` instead of `0.9999999999999999...`
- All fractional expressions like `(2/3)*3`, `(1/6)*6`, `(1/7)*7` now return exact results
- Reduced excessive parentheses in links notation output

### Added
- Native support for advanced mathematical functions computed in Rust/WebAssembly:
  - Trigonometric functions: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sinh`, `cosh`, `tanh`
  - Logarithmic functions: `ln`, `log`, `log2`, `log10`, `exp`
  - Math functions: `sqrt`, `cbrt`, `abs`, `floor`, `ceil`, `round`, `pow`, `factorial`
  - Numerical integration: `integrate(expr, var, lower, upper)` using Simpson's rule
  - Mathematical constants: `pi()`, `e()`
  - Angle conversion: `deg()`, `rad()`
  - Min/max functions with variable arguments
  - Power operator `^` for exponentiation
- Domain error handling for invalid inputs (e.g., `sqrt(-1)`, `ln(-1)`)
- Unknown function error messages for unsupported function names

### Changed
- Consolidated currency exchange rate data from 22,147 individual `.lino` files into 41 single files per currency pair
- Changed data format from individual date-based files (`data/currency/{from}/{to}/{date}.lino`) to consolidated files (`data/currency/{from}-{to}.lino`)
- Updated `download_historical_rates.py` script to generate consolidated format
- Added `consolidate_rates.py` script to migrate existing data to new format

### Added
- New `load_rates_from_consolidated_lino()` method in Calculator for loading the consolidated format
- New `parse_consolidated_lino_rates()` WASM binding for parsing consolidated format in web app

### Fixed
- Fixed .lino rate file parser to support the new format (`conversion:` as root, `rates:` for data section)
- The parser now correctly handles both the new format and the legacy format (`rates:` as root, `data:` for data section)

### Changed
- Updated `load_rates_from_consolidated_lino()` to support both new and legacy .lino formats
- Updated `parse_consolidated_lino_rates()` WASM binding to support both formats
- Added read-only `currency_db()` accessor to ExpressionParser for testing

### Added
- Added tests for new .lino format parsing
- Added test to verify historical rates can be loaded and retrieved from the database

### Fixed
- Fixed extra parentheses in Links notation for datetime subtraction expressions (issue #30). The output now correctly uses 2 outer parentheses instead of 3: `((datetime1) - (datetime2))` instead of `(((datetime1) - (datetime2)))`.

### Changed
- Add path filters to CI workflow (`ci.yml`) to only trigger on changes to `src/`, `tests/`, `data/`, `scripts/`, `web/`, `Cargo.toml`, `Cargo.lock`, and `.github/workflows/ci.yml`
- Add path filters to deploy workflow (`deploy.yml`) to only trigger on changes to `src/`, `data/`, `web/`, `Cargo.toml`, `Cargo.lock`, and `.github/workflows/deploy.yml`
- Add path filters to release workflow (`release.yml`) to only trigger on changes to `src/`, `tests/`, `data/`, `scripts/`, `web/`, `Cargo.toml`, `Cargo.lock`, and `.github/workflows/release.yml`
- This prevents CI/CD from running unnecessarily on documentation-only changes, changelog fragments, or other non-code files (like `CLAUDE.md`)

### Fixed
- Exchange rates fetched from API are now actually applied to the Calculator (fixes issue #18)
- Worker now calls `update_rates_from_api` method after fetching rates, replacing hardcoded fallback rates
- Removed suspicious hardcoded 89.5 USD/RUB rate from data/currency/usd-rub.lino

### Added
- New WASM method `update_rates_from_api(base, date, rates_json)` on Calculator class
- Integration tests for API rate updates

### Fixed
- Currency API fetch now works in Web Worker context (fixes the root cause of issue #18)
- The `fetch_json` function now uses `js_sys::global()` to detect and use either `Window` or `WorkerGlobalScope` context
- Previously, `web_sys::window()` returned `None` in Web Workers, causing silent API fetch failures

### Changed
- Added `WorkerGlobalScope` feature to web-sys dependency in Cargo.toml

### Added
- Browser history support: Each new calculation is now added to browser history, allowing users to navigate between calculations using the browser's back and forward buttons

### Changed
- Replaced unofficial fawazahmed0/currency-api with official Central Bank APIs:
  - European Central Bank (ECB) via frankfurter.dev for most currencies
  - Central Bank of Russia (CBR) via cbr.ru for RUB rates
- Updated currency rate source attribution throughout the codebase to reflect official sources

### Added
- GitHub Actions workflow for weekly automated currency rate updates from Central Banks
- Manual trigger support for on-demand rate updates

### Fixed
- EUR-CNY rate now sourced from ECB instead of unofficial API

### Changed

- Rebranded to "Link.Calculator" with new SVG logo and updated tagline "Free open-source calculator dedicated to public domain"
- Renamed "System" theme option to "Auto" for clarity
- Added "Automatic" option as first choice in language selector for auto-detection
- Moved input interpretation section before the result section and renamed it to "Input"
- Removed "Expression" label from input field for cleaner UI
- Changed input field from reactive updates to calculate-on-command: now requires clicking equals button or pressing Enter
- Disabled manual resize indicator on textarea (auto-resize only)

### Added

- Calculate button (=) in the input field to trigger computation
- Enter key support to submit calculation
- Preferred currency setting with major fiat currencies and top 10 cryptocurrencies
- Computation time display showing how long calculations take
- Window resize handler for textarea auto-resize

### Added
- Examples section showing 6 random examples from a centralized examples.lino file
- React unit tests for App.tsx component (23 new tests covering branding, input, settings, examples, and footer)
- USE-CASES.md documentation with screenshots of calculator features
- CI/CD workflow for auto-updating screenshots when web code changes
- New data/examples.lino file containing categorized calculator examples (arithmetic, currency, datetime, functions, integration)
- E2E screenshot generation tests for documentation

### Changed
- Updated E2E tests to use explicit calculation trigger (Enter key or button click) instead of reactive calculation
- Examples are now randomly selected from examples.lino on each page load for variety

### Changed
- Replace standard base64 URL encoding with URL-safe base64url encoding (RFC 4648 Section 5)
- URLs now use only `a-zA-Z0-9`, `-`, and `_` characters, avoiding URL encoding issues with `+`, `/`, and `=`

### Added
- Auto-calculate when expression is loaded from URL (calculation triggered immediately on page load)
- Backward compatibility: old base64 URLs are automatically redirected to new base64url format
- Prevent duplicate browser history entries when same expression is recalculated

### Added
- Data size unit conversions support: KB, MB, GB, TB, PB (SI decimal) and KiB, MiB, GiB, TiB, PiB (IEC binary), plus bit variants (b, Kb, Mb, etc.)
- `as` keyword for unit conversion syntax: `741 KB as MB`, `741 KB as mebibytes`, `741 KiB as MiB`
- Arithmetic with data size units: `(500 KB + 241 KB) as MB`
- Cross-standard conversions between SI and IEC systems (e.g., `1 GiB as GB`)
- Full-name unit support: `kilobytes`, `mebibytes`, `gibibytes`, etc.
- Case study for issue #55 at `docs/case-studies/issue-55/case-study.md`

### Added
- Cryptocurrency price conversions via CoinGecko API (free tier, no API key required):
  - Expressions: `19 ton in usd`, `19 ton to usd`, `19 ton as usd`, `19 ton in dollars`
  - Natural language crypto names: `toncoin`, `bitcoin`, `ethereum`, `solana`, `dogecoin`, etc.
  - Supports TON, BTC, ETH, BNB, SOL, XRP, ADA, DOGE, DOT, LTC, LINK, UNI and more
  - `in` and `to` keywords for unit conversion (in addition to existing `as`)
- Mass/weight unit conversions: `10 tons to kg`, `1 kg as pounds`, `1000 g as kg`
  - Units: milligrams (mg), grams (g), kilograms (kg), metric tons/tonnes (t), pounds (lb), ounces (oz)
  - Full-name aliases: `grams`, `kilograms`, `tonnes`, `pounds`, `ounces`
  - Disambiguation: singular `ton` = TON cryptocurrency; plural `tons`/`tonnes` = metric mass unit

### Added
- Support for currency symbol prefix notation: `$10`, `€5`, `£3`, `₽100`, `₹10` are now parsed as `10 USD`, `5 EUR`, `3 GBP`, `100 RUB`, `10 INR` respectively (fixes #51)
- Russian language currency name support: grammatical case forms of рубль (→ RUB) and рупия (→ INR) (fixes #52)
- INR (Indian Rupee) to the default currency database with USD→INR exchange rate (86.5) (fixes #53)
- USD triangulation for cross-currency conversions (e.g. RUB↔INR via USD bridge) when no direct rate exists (fixes #53)

### Added
- Support for the percentage operator (`%`): expressions like `3% * 50` are now parsed as `0.03 * 50 = 1.5`

### Fixed

- Add support for UF (Unidad de Fomento, ISO 4217: CLF) currency unit #20
  - `2 UF + 1 USD` now correctly converts between CLF and USD using default exchange rates
  - Both `UF` and `CLF` are recognized as the Chilean Unidad de Fomento
  - Natural language names ("unidad de fomento", "unidad fomento") are also supported
  - Added default USD/CLF exchange rate (1 USD ≈ 0.022 CLF, i.e. 1 CLF ≈ 45 USD)
  - Added historical rate data file `data/currency/usd-clf.lino`

### Fixed

- Fixed auto-release pipeline not bumping version when a git tag already existed from a previous partial release (version-and-commit.mjs now uses Cargo.toml as source of truth instead of git tags)
- Fixed changelog fragments not being deleted after collection in auto-release, causing duplicate release attempts on every push
- Fixed tag force-update to allow retrying a failed release without manual tag deletion
- Fixed changelog fragment check not requiring fragments for web/ frontend changes
- Fixed code change detection not recognizing TypeScript (.ts, .tsx), CSS, and HTML files as code changes requiring changelog entries

### Changed
- Serve CBR (Central Bank of Russia) RUB exchange rate data via GitHub Pages
- CORS fallback: when direct CBR API is blocked in browser, use locally-cached `.lino` files from GitHub Pages
- Updated `update-currency-rates.yml` to publish CBR rates as `.lino` files to `data/currency/`
- RUB conversions now always have recent official rates (up to 1 week old via CDN)

### Fixed
- Fix CI npm registry 403 errors by pinning `command-stream@0.9.4` and `lino-arguments@0.2.5` in all scripts
- When `use-m` loads packages by version (not `@latest`), it skips the `npm show` registry call that was causing 403 errors in GitHub Actions CI

## [0.1.0] - 2025-01-XX

### Added

- Initial project structure
- Basic example functions (add, multiply, delay)
- Comprehensive test suite
- Code quality tools (rustfmt, clippy)
- Pre-commit hooks configuration
- GitHub Actions CI/CD pipeline
- Changelog fragment system (similar to Changesets/Scriv)
- Release automation (GitHub releases)
- Template structure for AI-driven Rust development
