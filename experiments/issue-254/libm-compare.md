# Built-in `Math` against the `libm` crate (issue #254)

Vectors: `js/test/vectors/libm.json`, from `rust-vectors/src/bin/libm.rs`
(libm 0.2.16 and the Rust `f64` methods on x86_64 Linux, i.e. glibc). The
counts are the vectors whose result differs bit for bit; any NaN equals any NaN.
JavaScript has no fused multiply-add, so the `Math` column for fma is `a * b + c`.
The JS port is `js/src/rust/libm.js`, with the same counts on all three runtimes.

| function | vectors | Node 26 `Math` vs libm | Node 20 `Math` vs libm | Bun `Math` vs libm | std (glibc) vs libm | JS port vs libm |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| sin | 2070 | 19 | 19 | 47 | 47 | 0 |
| cos | 2070 | 20 | 20 | 45 | 45 | 0 |
| tan | 2070 | 0 | 0 | 70 | 70 | 0 |
| asin | 2113 | 0 | 0 | 108 | 108 | 0 |
| acos | 2113 | 0 | 0 | 96 | 96 | 0 |
| atan | 1690 | 0 | 0 | 48 | 48 | 0 |
| atan2 | 2165 | 3 | 3 | 230 | 230 | 0 |
| sinh | 1687 | 6 | 6 | 27 | 27 | 0 |
| cosh | 1687 | 18 | 18 | 93 | 93 | 0 |
| tanh | 1687 | 141 | 141 | 125 | 125 | 0 |
| exp | 1749 | 1 | 1 | 110 | 110 | 0 |
| log | 2298 | 13 | 13 | 45 | 45 | 0 |
| log10 | 2298 | 241 | 241 | 234 | 234 | 0 |
| log2 | 2298 | 0 | 0 | 10 | 10 | 0 |
| pow | 3618 | 201 | 117 | 360 | 197 | 0 |
| cbrt | 2394 | 145 | 145 | 0 | 0 | 0 |
| sqrt | 567 | 0 | 0 | 0 | 0 | 0 |
| fma | 3208 | 1062 | 1062 | 1062 | 0 | 0 |
| **total** | **37782** | **1870** | **1786** | **2710** | **1485** | **0** |

Runtimes: Node 26.11.1 (V8 14.6), Node 20.20.2 (V8 11.3), Bun 1.4.3
(JavaScriptCore). Notes:

- V8 ships its own fdlibm port, so Node is close to libm except for tanh,
  log10, pow and cbrt. Bun calls the C library, so it matches glibc on
  everything except pow. The two Node versions differ from each other on pow.
- `Math.pow(1, ±Infinity)` and `Math.pow(-1, ±Infinity)` are NaN by the
  ECMAScript spec. Rust and libm return 1.
- The JS port matches libm on every vector on all three runtimes. NaN payloads
  are not compared.

Reproduce from the repository root:

```sh
node scripts/bounded-cargo.mjs run --release --manifest-path experiments/issue-254/rust-vectors/Cargo.toml --bin libm > js/test/vectors/libm.json
node experiments/issue-254/libm-compare.mjs
~/.bun/bin/bun experiments/issue-254/libm-compare.mjs
npx -y node@20 experiments/issue-254/libm-compare.mjs
```
