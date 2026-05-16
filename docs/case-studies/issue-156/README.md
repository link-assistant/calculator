# Issue 156 Case Study: Exportable Rust Library Surface

## Summary

Issue [#156](https://github.com/link-assistant/calculator/issues/156) asks us to
double-check that every public-ish concept in the calculator is reachable when
the crate is consumed as a normal Rust library from
[crates.io](https://crates.io/crates/link-calculator). The crate is already
published (version `0.15.5`, 40 releases, last update 2026-05-09) and the
release workflow automates further publishes, so the real question is whether
the **API surface** is rich enough for downstream code (in particular
[github.com/link-assistant/formal-ai](https://github.com/link-assistant/formal-ai))
to:

1. Drive the calculator end-to-end.
2. Capture every step *and* the final result in a way that can be re-encoded as
   Links Notation, with no output lost in translation.
3. Reach into the internals — parser, evaluator, value algebra — so that users
   can reconstruct or replay a calculation in their own way.

## Collected Data

- `issue.json`: raw GitHub issue data for issue #156.
- `issue-comments.json`: raw issue comments. The issue had no comments when
  investigated.
- `pr-157.json`: prepared PR metadata.
- `pr-conversation-comments.json`, `pr-review-comments.json`, `pr-reviews.json`:
  PR discussion data. All empty at the start of the investigation.

## External Facts

- The crate is already on crates.io as
  [`link-calculator`](https://crates.io/crates/link-calculator) — published
  2026-01-25, 40 versions, 535 lifetime downloads, current version `0.15.5`.
- The release pipeline at `.github/workflows/release.yml` invokes
  `scripts/publish-crate.mjs`, which runs `cargo publish` with
  `CARGO_REGISTRY_TOKEN` / `CARGO_TOKEN`.
- `Cargo.toml` declares `crate-type = ["cdylib", "rlib"]`, so the same source
  produces both a WebAssembly module and a normal Rust `rlib` for downstream
  Cargo consumers.
- [formal-ai](https://github.com/link-assistant/formal-ai) is the planned
  downstream consumer. It is a deterministic, symbolic assistant that exposes
  OpenAI-shaped interfaces; it will lean on this crate for the math/conversion
  parts of its rule engine, so it needs a programmatic Rust API rather than the
  wasm-bindgen JSON bridge used by the web UI.
- The Links Notation reference is the [`lino-rs`](https://github.com/foundationDB/lino-rs)
  family of projects from `linksplatform`; the calculator already emits LINO
  via `Expression::to_lino()` and consumes it for rate files via
  `Calculator::load_rates_from_consolidated_lino`.

## Timeline

- 2026-05-16T10:35:27Z: GitHub issue #156 was filed.
- 2026-05-16T10:36:07Z: PR #157 was opened from branch
  `issue-156-492574dbd3ca` as a WIP placeholder.
- 2026-05-16T10:39Z: case study data captured under
  `docs/case-studies/issue-156`.
- 2026-05-16T10:40Z: audit of public surface began; this README records the
  result.

## Requirements (parsed from the issue body)

1. **Crate must be exportable as a Rust library on crates.io.** The release
   pipeline already does this for the WebAssembly artefacts; verify the
   `rlib` build is fully usable from a plain `cargo add link-calculator`
   consumer.
2. **Every calculation step must be reachable as Links Notation.** The current
   `CalculationResult.steps` field exposes human-readable strings only; the
   final result is exposed as a display string. For a downstream consumer that
   wants to re-encode in LINO, the `Expression` (input AST) and `Value`
   (output) must be reachable too.
3. **The final result must not be lost.** `calculate_internal` already returns
   the formatted result; we need an equally simple way to recover the
   *structured* `Value` it came from so the consumer can format it however
   they like (LINO, LaTeX, JSON, etc.).
4. **All internal functions should be public** so users can reconstruct
   computations differently — e.g., re-run a single binary operation, replay
   a step list, or substitute a sub-expression.
5. **Plan/execute everything in a single PR** (PR #157, this branch).

## Audit of the Existing API Surface

### Already public

- `Calculator` struct with `wasm-bindgen` constructor and the JSON-string
  facades `plan`, `execute`, `calculate`, `version`,
  `update_rates_from_api`, `update_cbr_rates_from_api`,
  `update_crypto_rates_from_api`, `load_rates_from_consolidated_lino`.
- `Calculator::plan_internal` and `Calculator::calculate_internal` return
  typed `CalculationPlan` / `CalculationResult` values for Rust consumers.
- `Calculator::parse`, `Calculator::evaluate`,
  `Calculator::load_rate_from_lino`, `Calculator::load_rates_batch`,
  `Calculator::load_rates_from_consolidated_lino_impl`.
- The result/plan types:
  `CalculationResult`, `CalculationStep`, `CalculationPlan`, `PlotData`,
  `RepeatingDecimalFormats`, `RateSource`, plus the `VERSION` constant.
- Module roots:
  `crypto_api`, `currency_api`, `error`, `grammar`, `lino`, `plan`,
  `types`, `wasm`.
- Grammar re-exports: `DateTimeGrammar`, `ExpressionParser`,
  `evaluate_indefinite_integral`, `symbolic_result_to_latex`,
  `try_symbolic_integral`, `Lexer`, `Token`, `TokenKind`, `NumberGrammar`,
  `evaluate_function`, `integrate`, `is_math_function`.
- Type re-exports: `Currency`, `CurrencyDatabase`, `ExchangeRateInfo`,
  `DateTime`, `DateTimeResult`, `Decimal`, `BinaryOp`, `Expression`,
  `Rational`, `RepeatingDecimal`, `DataSizeUnit`, `DurationUnit`,
  `MassUnit`, `Unit`, `Value`, `ValueKind`.

### Gaps preventing full reconstruction

1. **`ExpressionParser` cannot be driven step-by-step from outside.** The
   tracked methods `evaluate_with_steps`, `evaluate_expr`,
   `evaluate_expr_with_steps`, `apply_binary_op`, `evaluate_integrate`,
   `evaluate_at`, `evaluate_expr_with_var` are all `fn` (private). A consumer
   that has a parsed `Expression` cannot ask the parser to evaluate it while
   recording steps without going back through `parse_and_evaluate`, which
   re-parses and discards the AST.
2. **`evaluate_power` is a private top-level function** — useful as a
   primitive when users compose their own evaluator.
3. **`utils::generate_issue_link`** is referenced for `CalculationResult`
   creation but is hidden inside a private module. Downstream tooling that
   wants to render the same issue link for its own failure cases cannot reach
   it.
4. **`Calculator` has no way to return the structured `Value` and step list
   together with the AST.** `calculate_internal` returns a `CalculationResult`
   tuned for the web UI; programmatic consumers end up re-parsing and
   re-evaluating to get the underlying `Expression`/`Value`.
5. **No integration test verifies the surface a downstream crate sees.** All
   tests assume `Calculator`'s JSON facade or `calculate_internal`. Nothing
   exercises `ExpressionParser::evaluate_with_steps` directly or the round-
   trip Expression → LINO → re-parse → re-evaluate.
6. **No example targeting downstream Rust users.** `examples/basic_usage.rs`
   uses `calculate_internal` only — it does not demonstrate the
   reconstruction story formal-ai needs.

## Root Cause

The crate was originally designed to back the WebAssembly web calculator, so
its API tapered down to the few methods the web worker calls. The internal
evaluator was kept private because the web layer never needed to drive it
directly. When the same crate is consumed by a Rust program that wants to
inspect or replay every step, the private layer becomes the ceiling.

## Solution Options

1. **Re-expose the entire `ExpressionParser`/`utils` surface as `pub`.**
   Lowest cost, highest reach. Cost: the surface becomes part of the public
   contract and changes are SemVer-breaking thereafter. Mitigation: keep the
   methods documented and stable; they have been the de facto API for the
   internal Calculator for many versions.
2. **Add a separate trait or facade** (e.g., `Evaluator`) and forward to the
   existing private methods. Bigger refactor for limited benefit; the
   existing methods are already shaped for downstream use.
3. **Publish a sibling crate** (`link-calculator-core`) that exposes the
   internals. Over-engineered for one consumer.

We pick option 1. To minimise the SemVer footprint we group all newly-public
methods behind doc comments labelling them as "low-level" so consumers
understand the contract.

## Fix

- Expose the internal evaluator on `ExpressionParser`:
  `evaluate_with_steps`, `evaluate_expr`, `evaluate_expr_with_steps`,
  `apply_binary_op`, `evaluate_integrate`, `evaluate_at`,
  `evaluate_expr_with_var` become `pub`.
- Lift `evaluate_power` from a free function to a `pub fn` exported via
  `crate::grammar::evaluate_power`.
- Promote `mod utils;` to `pub mod utils;` so `generate_issue_link` and
  `truncate` are reachable; re-export `generate_issue_link` from the crate
  root.
- Add a high-level convenience method `Calculator::calculate_with_value`
  that returns `(Expression, Value, Vec<String>, String)` so callers get the
  AST, structured value, raw step list, and Links Notation in one call —
  no information lost.
- Add a new integration test
  `tests/issue_156_library_surface_tests.rs` that exercises the new public
  paths: drive `ExpressionParser` directly, re-emit Links Notation from a
  `Value`, replay a calculation by manually walking the AST.
- Add a new Rust example `examples/library_consumer.rs` modelling the
  formal-ai consumption pattern: parse → plan → evaluate-with-steps →
  re-encode as LINO → render final result.
- Bump version to `0.15.6` so the next release cycle picks up the expanded
  surface; add a changelog entry.

## Verification

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features`
- `cargo test --all-features --verbose`
- `cargo test --test issue_156_library_surface_tests -- --nocapture`
- `cargo run --example library_consumer`
- `cargo package --no-verify --allow-dirty` (confirms publishable manifest)
- `cargo publish --dry-run` (confirms publishability under crates.io rules)

## Related Work

- [Issue #154 case study](../issue-154/README.md) introduced the
  `datetime_result` metadata on `CalculationResult`; the same pattern of
  surfacing structured data for downstream consumers applies here.
- [Issue #18](https://github.com/link-assistant/calculator/issues/18) added
  the `update_rates_from_api` family of methods, which this audit also
  re-confirms as public.
- The existing `scripts/publish-crate.mjs` and `release.yml` workflow remain
  unchanged; they already handle the publish step.
