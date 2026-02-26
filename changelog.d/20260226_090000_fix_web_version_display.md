---
bump: minor
---

### Added

- Added `deploy-after-release` CI/CD job that rebuilds and redeploys the web app with the correct version immediately after auto-release bumps `Cargo.toml` (fixes version display in web footer)
- Added crates.io badge to README

### Fixed

- Fixed web app footer showing stale version (e.g., `v0.1.0`) after a release: the WASM now gets recompiled with the updated `CARGO_PKG_VERSION` after each version bump, so the deployed web app always displays the current release version
- Fixed `scripts/version-and-commit.mjs` not updating `Cargo.lock` after bumping the version in `Cargo.toml`, which caused `cargo package --list` to fail with "files in the working directory contain changes that were not yet committed"
- Fixed CI/CD badge URL in README to match the actual workflow name
