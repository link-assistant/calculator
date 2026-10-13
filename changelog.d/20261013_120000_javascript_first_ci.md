---
bump: minor
---

### Added
- Native JavaScript engine in `js/` (issue #254), ported module by module from the Rust sources: errors, utilities, dimensions, locale number scanning, the `rust_decimal` and chrono subsets the calculator uses, and data modules generated from `data/` (`cd js && npm run generate`). The tests compare against vectors printed by the Rust code and run on Node.js 20, Node.js 24 and Bun.
- `scripts/check-javascript-first.mjs`: fails CI when Rust tooling can start outside `rust.yml`, when the Rust stage does not need every JavaScript job, or when a cancellation could kill a running Rust build.

### Changed
- CI runs JavaScript first (issue #254). `release.yml` holds the JavaScript, Python and shell jobs; each cancels its superseded run and none has path filters. Its `rust-stage` job calls the new `rust.yml` only after all of them passed, in a per-branch group that queues instead of cancelling.
- `update-screenshots.yml` and `coverage-gap-report.yml` run only from the Rust stage (on push to `main`, weekly, or by dispatching `release.yml` with `release_mode` `screenshots` or `coverage-gap-report`). `cargo audit` and CodeQL for Rust move from `security.yml` into `rust.yml`; `npm audit` and the npm freshness check now cover `js/` too.
- Rust CI jobs install the toolchain pinned in `rust-toolchain.toml` through `.github/actions/setup-rust`, with a cache keyed per job, toolchain and `Cargo.lock` and no restore fallback, and build through `scripts/bounded-cargo.mjs`.
