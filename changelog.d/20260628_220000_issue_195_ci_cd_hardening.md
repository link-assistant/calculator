### Fixed
- Hardened CI/CD jobs that generate commits on `main` against non-fast-forward push races by serializing main-writer jobs and retrying pushes after `git pull --rebase`.
- Removed avoidable CI warning noise by pinning `wasm-pack`, using Cargo's native registry token environment variable, setting Git's default initial branch in Actions, and documenting the expected Vite bundle threshold.
