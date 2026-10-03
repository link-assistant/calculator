---
bump: minor
---

### Added

- CI job `Dependency Freshness` (`scripts/check-dependency-freshness.mjs`)
  fails when a direct Rust or web dependency is behind its latest release or
  `cargo update` has pending lockfile updates, unless the blocker is documented
  with an issue URL (a `Cargo.toml` comment, or `dependencyBlockers` in
  `package.json`) (issue #223).
- The Build job fails when the packaged `.crate` would exceed the 10 MiB
  crates.io upload limit, which had blocked the v0.20.4..v0.21.0 releases.
