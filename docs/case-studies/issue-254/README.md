# Case Study: Issue #254 — JavaScript First, Rust Gated

## Overview

**Issue:** https://github.com/link-assistant/calculator/issues/254
**Pull request:** https://github.com/link-assistant/calculator/pull/261
**Parent standard:** link-assistant/router#759 (requirement 9)

The calculator existed only as Rust (about 32.7k lines in `src/`); the web UI
called it through WASM, and all CI was one Rust-first workflow. The issue asks
for a native JavaScript engine at parity, Rust generated from it, a CI split in
which JavaScript runs first and gates Rust, a bounded Rust build load, agent
push discipline, and the checks of the two pipeline templates.

Data collected for this study:

- [`data/issue-254.md`](data/issue-254.md): the issue text as audited on 2026-10-10.
- [`runs-per-pr.md`](runs-per-pr.md): CI runs per pull request before the split.
- [`upstream/`](upstream/): reports filed with other projects.

## Timeline

| Date | Event |
| --- | --- |
| 2026-10-10 | Audit against link-assistant/router#759; issue #254 filed with findings 1-8. |
| 2026-10-13 | Toolchain pin, Dependabot, `.cargo/config.toml`, `bounded-cargo.mjs`, `AGENTS.md`/`CLAUDE.md` and the push gate land on the PR branch. |
| 2026-10-13 | Parity corpus (2279 inputs) and goldens recorded at a fixed clock; `js/` package scaffolded with exact emulations of Rust `str`/`f64`/integer/Debug semantics, `num-rational`, `rust_decimal`, `libm` 0.2.16 and the `chrono` subset. |
| 2026-10-13 | Porting libm's `fma` exposed a sticky-bit bug in the generic soft-float path that `f64::mul_add` hits on wasm32: filed as rust-lang/compiler-builtins#1353 ([report](upstream/compiler-builtins-fma-sticky.md)). |
| 2026-10-13 | CI split: `release.yml` keeps only JavaScript jobs and calls `rust.yml` from one gated `rust-stage` job; `check-javascript-first.mjs` enforces it. |
| 2026-10-13 | First run of the split pipeline: JavaScript stage green; the Rust stage's fresh-merge simulation failed because `CALCULATOR_CARGO_JOBS=4` from the Rust stage leaked into a script test that expected the local default of 2. Fixed by pinning the variable in the test. |

## Requirements and Solutions

### R1. Native JavaScript engine at parity

**Root cause of the gap:** the engine was written once, in Rust; `web/src/worker.ts`
imports the WASM `Calculator`.

**Solution.** `js/` (`link-calculator`, no runtime dependencies, Node 20+, Bun
and browsers) is a module-by-module port that follows the conventions in
[`js/PORTING.md`](../../../js/PORTING.md). Exactness is the hard part: the
results must match Rust bit for bit, so the port re-implements the semantics
JavaScript does not share with Rust instead of using JavaScript built-ins:

| Rust dependency | JavaScript replacement | Checked by |
| --- | --- | --- |
| `rust_decimal` | `js/src/rust/decimal.js` | Rust-computed vectors |
| `num-rational`, `num-bigint` | `js/src/rust/ratio.js` (on `bigint`) | Rust-computed vectors |
| `libm` 0.2.16 (`f64::sin`, `exp`, …) | `js/src/rust/libm.js` | Rust-computed vectors, fuzzing |
| `chrono` 0.4.45 subset | `js/src/rust/chrono.js` | Rust-computed vectors |
| `str`, `char`, `{:?}`, `f64` Display | `js/src/rust/str.js`, `fmt.js`, `num.js` | unit tests and vectors |

Existing libraries were evaluated and rejected where they differ from Rust:
`decimal.js`/`big.js` round and format differently from `rust_decimal`;
`Math.sin` and friends are implementation-defined in ECMAScript (V8 and
JavaScriptCore differ from libm in the last bit); `Intl`/`Temporal` parse and
format differently from `chrono`. Exact ports have no runtime dependency and
behave the same in every engine.

The vector generators in `experiments/issue-254/rust-vectors/` include the
calculator's own Rust sources unchanged, so the vectors are the Rust results.
`parity/corpus.json` (2279 inputs extracted from tests and docs) and
`parity/goldens/*.json` (Rust results at a fixed clock and rate cutoff) are the
end-to-end parity manifest: `tests/parity_goldens.rs` checks the Rust side and
`npm run parity` in `js/` checks the JavaScript side.

### R2. Rust generated from JavaScript

See [Translation](#translation-r2) below.

### R3. JavaScript stage first, Rust gated

**Root cause:** one workflow whose web and script tests ran after the Rust
jobs, behind `paths:` filters.

**Solution.**

- `release.yml` has no `paths:` filters. Its jobs are JavaScript only:
  detect-changes, changelog and version checks, secrets scan, npm freshness,
  script checks, `js/` lint, type check and data regeneration check, and the
  `js/` tests on Node 20, Node 24 and Bun. The `JavaScript Stage` job reports
  the result.
- `rust-stage` `needs:` every other job and runs only when
  `needs.javascript-stage.result == 'success'`. It calls `rust.yml`
  (`workflow_call` only), which holds every Cargo, wasm-pack and CodeQL-for-Rust
  job, plus the workflows that need Rust (`update-screenshots.yml`,
  `coverage-gap-report.yml`, also `workflow_call` only).
- `scripts/check-javascript-first.mjs` (run in Script Checks, tested by
  `scripts/check-javascript-first.test.mjs` with mutations of the real
  workflows) fails when Rust tooling appears anywhere else, directly or through
  a local composite action, when a Rust workflow gains a trigger, when another
  job calls `rust.yml`, or when the gate is weakened.
- Required checks: `.github/main-ruleset.json` requires `JavaScript Stage` and
  `Rust Stage / Rust Stage`. A pull request cannot change repository settings,
  so `scripts/check-branch-ruleset.mjs` warns on pull requests and fails on
  `main` with the command an admin must run:
  `gh api -X POST repos/link-assistant/calculator/rulesets --input .github/main-ruleset.json`.

### R4. Cancellation only in the JavaScript stage

**Root cause:** the workflow-level `concurrency` cancelled every in-progress run
on a branch, including Rust builds that had nearly finished.

**Solution.** No workflow-level concurrency. Each JavaScript job has its own
group with a literal `cancel-in-progress: true`; `rust-stage` uses the
per-branch group `rust-${{ github.ref }}` with `cancel-in-progress: false`, and
no job inside `rust.yml` may cancel. `scripts/ci-runs-per-pr.mjs` measures runs
per pull request; the baseline (every run started Cargo) is in
[`runs-per-pr.md`](runs-per-pr.md), to be extended after the next five merged
pull requests.

### R5. Bounded Rust build load

`.cargo/config.toml` (two jobs, `CARGO_INCREMENTAL=0`, reduced debug info),
`scripts/bounded-cargo.mjs` (lock file, `prlimit` address-space cap, disk
budget with pruning before and after the build), and the
`.github/actions/setup-rust` composite action (toolchain from
`rust-toolchain.toml`, cache keyed by job, toolchain and every `Cargo.lock`,
with no restore keys so `target` never accumulates). Local agents run the
JavaScript tests by default (`AGENTS.md`, `CONTRIBUTING.md`).

### R6. Agent push discipline

`AGENTS.md` (with `CLAUDE.md` pointing to it) and the CONTRIBUTING "Multi-Agent
Work" section: drafting sub-agents never push; one CI/CD agent runs
`scripts/ci-push-gate.mjs`, which refuses while a run is in progress on the
branch and saves the failed logs of the latest run to `ci-logs/`, then pushes
once.

### R7. Template workflows

`links.yml` with `lychee.toml`, `security.yml` (cargo-audit in the Rust stage,
npm audit for `js/` and `web/`, secretlint, CodeQL), `workflows.yml`
(actionlint and zizmor), Dependabot for cargo, npm and GitHub Actions with a
seven-day cooldown, and `rust-toolchain.toml`. Third-party actions are pinned
to commit SHAs. `.github/zizmor.yml` documents the namespaces that may keep a
release tag because their owners keep tags immutable (`actions/*`, `github/*`,
`docker/*`, `astral-sh/*`, `lycheeverse/*`, `zizmorcore/*`), and zizmor fails
on any other unpinned action.

## Translation (R2)

The status of generating Rust from the JavaScript source with
link-foundation/meta-language is recorded in
[`experiments/issue-254/meta-language/README.md`](../../../experiments/issue-254/meta-language/README.md).

## Upstream Reports

- rust-lang/compiler-builtins#1353: the generic `fma` loses the sticky bit, so
  `f64::mul_add` on wasm32 can round differently from native targets.
