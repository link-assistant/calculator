---
bump: patch
---

### Changed
- Trigonometric, hyperbolic, exponential, logarithmic, `pow` and `cbrt` results now come from the `libm` crate instead of the platform's C library (#254), so the CLI, the WASM build and the native JavaScript port (`js/src/rust/libm.js`) return the same bits on Linux, macOS, Windows and in the browser. A few results change in the last digit (for example plotted points of `integrate sin(x) dx`).
