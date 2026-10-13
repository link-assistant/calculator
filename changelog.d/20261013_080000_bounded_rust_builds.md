---
bump: patch
---

### Changed
- Rust builds are bounded locally and in CI (#254): `.cargo/config.toml` limits Cargo to two parallel rustc processes, turns off incremental compilation and reduces debug info, and `node scripts/bounded-cargo.mjs <cargo arguments>` queues concurrent builds behind a lock, caps memory with `prlimit` and prunes the target directory (or a shared one in `CALCULATOR_CARGO_TARGET_DIR`) when it exceeds its disk budget. Local agent runs execute the JavaScript tests by default; see CONTRIBUTING.md, "Rust Build Load".
