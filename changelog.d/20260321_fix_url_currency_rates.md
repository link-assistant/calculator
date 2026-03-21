---
bump: minor
---

### Changed
- Introduced plan→execute architecture: `calculator.plan(expr)` parses the expression and determines required rate sources (ECB, CBR, CoinGecko) from the AST before execution. This replaces the previous TypeScript string heuristic with authoritative Rust-based detection.
- The worker now sends the plan to the UI immediately, showing the expression interpretation while rate sources are being fetched.

### Fixed
- Fixed currency conversion failing on page load when expression is loaded from URL. Expressions containing currencies (e.g., RUB, TON, USD) would show "No exchange rate available" because rates hadn't been fetched yet. The worker now awaits required rate sources before executing.

### Added
- `ARCHITECTURE.md` documenting the plan→execute pipeline, rate sources, module structure, and data flow.
- `Calculator::plan()` WASM API for planning calculations without executing them.
- `Calculator::execute()` WASM API (equivalent to `calculate()`, named for clarity in the pipeline).
- `Expression::collect_currencies()` for extracting currency codes from the parsed AST.
- `RateSource` enum and `CalculationPlan` struct in the Rust core.
