---
bump: minor
---

### Changed
- Updated `num-bigint` to 0.5 and `thiserror` to 2.0, and refreshed `Cargo.lock` (#223).
- `num-rational` no longer enables its `num-bigint` feature, which pinned `num-bigint` 0.4.

### Fixed
- Publishing to crates.io failed with `413 Payload Too Large` because a stray wasm-pack archive was packaged; the crate now ships an explicit `include` list (#223).
- `scripts/publish-crate.mjs` reported success when `cargo publish` failed, so 0.21.0 was never published (#223).
