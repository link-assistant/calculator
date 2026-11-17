---
bump: minor
---

### Added
- The JavaScript engine in `js/` is complete (issue #254): the lexer, token and expression parsers, grammars, `Calculator`, substitution and the WASM-compatible API (`link-calculator/wasm`). `npm run parity` matches the Rust goldens for every one of the 2474 corpus cases on Node.js and Bun, in both CI stages.
- `scripts/sync-js-version.mjs` keeps `js/`'s version equal to `Cargo.toml`'s; the release scripts run it and Script Checks verifies it.

### Changed
- The web app runs the JavaScript engine instead of the WASM build, and its lint, type check, tests, build and Playwright suite run in the JavaScript stage. `rust.yml` keeps a `WASM Build` job for the Rust bindings in `src/wasm.rs`.
- `js/scripts/translate-rust.mjs` generates `js/rust-generated/` from `js/src` with link-foundation/meta-language at a pinned commit. `--verify` runs in `npm test`, and the new `JS to Rust Translation` job retranslates and diffs; carried items must be listed in `js/translation-allowlist.lino`.
