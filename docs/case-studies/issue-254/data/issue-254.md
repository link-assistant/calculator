## Problem

link-assistant/router#759 sets a standard for large Rust projects: JavaScript first with a native implementation at parity, Rust CI/CD gated on green JavaScript CI/CD, bounded Rust build load, single-push-gate agent rules, and the checks from the two pipeline templates and `docs/CI-CD-BEST-PRACTICES.md`. This repository (about 32.7k lines of Rust in `src/`, plus a React web UI in `web/`) was audited against that checklist on 2026-10-10. The calculator exists only as Rust, the web UI only calls it through WASM, all CI is one Rust-first workflow, and several template workflows and agent rules are absent.

## Findings

1. **No native JavaScript implementation and no parity manifest.** `web/src/worker.ts:15` imports `init, { Calculator, ... } from '@wasm/link_calculator'`. The `web/` package (`link-calculator-web`) has only UI code, hooks and utils; no calculation logic is written in JavaScript. There is no `parity/` manifest and no parity checker in `scripts/`.
2. **No meta-language translation and no regenerate-and-diff check.** Rust in `src/` and `tests/` is hand-written. No workflow or script generates Rust from JavaScript.
3. **No JavaScript stage gates Rust.** `.github/workflows/release.yml` is the only CI workflow. The chain is `detect-changes`, `changelog`, `test` (cargo), `coverage`, then `wasm-build` and `web-build` (lines 374-448). The web tests (`npm test`, line 435) and type check run only after the Rust `lint`, `test` and `coverage` jobs, so JavaScript never runs first. `npm run lint` (ESLint, `web/package.json`) and `npm run test:e2e` (Playwright) are not run in any CI job; Playwright appears only in `update-screenshots.yml:70`. The `scripts/*.test.mjs` tests run inside the Rust `lint` job (line ~55). `push`/`pull_request` use `paths:` filters (lines 4-28), so a change outside the listed paths skips CI entirely.
4. **Push-per-fix waste.** The workflow `concurrency` (lines 54-56) cancels in-progress runs on every non-main ref, including the Rust stage. `gh run list` of 100 recent runs: branch `issue-225-*` had 4 failed, 2 cancelled, 2 green `CI/CD Pipeline` runs; `issue-227-*` (PR #228) 2 failed, 2 green; `issue-234-*` 2 cancelled, 4 green; `issue-229-*` 1 cancelled, 2 green. `release.yml` over its history: 16 failures and 4 cancelled of 100 runs.
5. **Rust build load is unmanaged.** There is no `.cargo/config.toml`. `release.yml` caches `target` together with `~/.cargo/registry` under a key of only `hashFiles('**/Cargo.lock')` (lines 184, 264, 302, 344), with no pruning (`cargo-sweep`, `Swatinem/rust-cache` or equivalent), and sets no `CARGO_INCREMENTAL=0`, no `debug` reduction and no `-j` limit for local agent builds. `cargo test --all-features` builds 55 integration test binaries from `tests/` separately.
6. **No agent instructions.** There is no `AGENTS.md` or `CLAUDE.md`, and `CONTRIBUTING.md` says nothing about bulk drafting sub-agents, a single push gate or waiting for in-progress runs.
7. **Template differences.** The Rust template has `links.yml` (with `lychee.toml`), `security.yml` and `workflows.yml`; the JS template has the same three. This repository has none of them: no link check, no security scan (no cargo-audit/deny, npm audit, secrets scan or CodeQL anywhere in `.github`, `scripts`, `Makefile` or `.pre-commit-config.yaml`), no workflow linting, and no `.github/dependabot.yml` or `CODEOWNERS`. Actions are pinned by major tag only (`actions/checkout@v6`) and `dtolnay/rust-toolchain@stable` floats (10 uses).
8. **Sub-issue closing links on PR #228** are tracked separately in #247 and are not repeated here.

Checked and fine: no recent failure on `main` (last failed `release.yml` run on main was 2026-07-16); the only open PR is #228 (created 2026-10-07, not stale); `docs/case-studies/` exists.

## Requirements

1. Add a native JavaScript implementation of the calculator (Node 20+ and Bun, no WASM) and use it in `web/` for the same operations the WASM `Calculator` exposes. Add `parity/*.json` plus a checker script that fails CI if a feature exists in only one language, run by both a JavaScript job and a Rust job.
2. Generate the Rust code from the JavaScript source with link-foundation/meta-language, with a CI step that regenerates and diffs it. Report each untranslatable construct to meta-language with a minimal example and list it in a documented allowlist.
3. Split CI so the JavaScript stage (ESLint, type check, vitest, scripts tests, parity, regenerate-and-diff, and a Playwright smoke test) runs first, and every Rust job `needs:` all of its jobs. Verify with a CI check that fails if a workflow can start a Rust job without this gate. Parity and translation checks must run regardless of `paths:` filters. Required status checks in branch protection must include both stages.
4. Keep `cancel-in-progress` only for the JavaScript stage. The Rust stage uses a per-branch concurrency group without cancellation. Report before/after for runs per PR (green/failed/cancelled) for the next five PRs.
5. Add `.cargo/config.toml` (shared bounded `target-dir` option, `CARGO_INCREMENTAL=0`, reduced `debug`), a cache keyed so `target` does not grow without bound, and pruning of old artifacts in CI and in the documented local workflow. Document that local agent runs execute JavaScript tests by default.
6. Add `AGENTS.md`/`CLAUDE.md` and a `CONTRIBUTING.md` section: drafting sub-agents never push; one CI/CD sub-agent pushes once after collecting all failures of the latest run and never while a run is in progress on the same commit range.
7. Port from the two templates: `links.yml` with `lychee.toml`, `security.yml` (dependency audit for Cargo and npm, secrets scan), `workflows.yml` (actionlint), Dependabot for cargo, npm and github-actions, and pin third-party actions to commit SHAs (or document why tags are accepted). Pin the Rust toolchain via `rust-toolchain.toml`.

## Related

- link-assistant/router#759 (parent standard, requirement 9)
- #247 (PR #228 closing references), #227 and #235-#246 (feature work; not about CI/CD, no overlap)
- No open issue in this repository covers findings 1-7.

---
We need to download all logs and data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), in which we will reconstruct timeline/sequence of events, list of each and all requirements from the issue, find root causes of the each problem, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

If there is not enough data to find actual root cause, add debug output and verbose mode if not present, that will allow us to find root cause on next iteration.

If issue related to any other repository/project, where we can report issues on GitHub, please do so. Each issue must contain reproducible examples, workarounds and suggestions for fix the issue in code. Also double check to fully apply requirements to entire codebase, so if we have issue in multiple places, it should be fixed in all them.

Please plan and execute everything in this single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.


