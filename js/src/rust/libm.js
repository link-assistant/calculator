// A bit-exact port of the f64 functions of the Rust `libm` crate, version
// 0.2.16 (https://github.com/rust-lang/compiler-builtins/tree/master/libm).
//
// The Rust engine calls `libm` instead of the `f64` methods, whose results come
// from the platform C library and differ between targets. This module gives
// the same bits on every JavaScript engine: it uses only IEEE 754 double
// arithmetic, typed-array bit casts, integer operations and bigint, and never
// a transcendental `Math` function (their results differ between V8,
// JavaScriptCore and SpiderMonkey).
//
// The implementation chosen is the one libm selects on x86_64 (and any generic
// target): the FreeBSD/musl ports, and core-math for cbrt. `sqrt` and `fma` are
// the correctly rounded IEEE operations, which libm computes with the `sqrtsd`
// and FMA instructions on x86_64: `sqrt` is `Math.sqrt` (correctly rounded on
// every engine), `fma` is the generic `fma_round` with one sticky-bit fix that
// makes it correctly rounded like the instruction (see libm/fma.js).
//
// Every function here maps a NaN argument to a NaN result like libm does, but
// the NaN payload and sign are not guaranteed (engines may canonicalise NaNs).
//
// Source files, under libm-0.2.16/src/math/:
//   sin, cos, tan        sin.rs, cos.rs, tan.rs, k_sin.rs, k_cos.rs, k_tan.rs,
//                        rem_pio2.rs, rem_pio2_large.rs   -> libm/trig.js, libm/rem_pio2.js
//   asin, acos, atan,    asin.rs, acos.rs, atan.rs, atan2.rs -> libm/atrig.js
//   atan2
//   exp, sinh, cosh,     exp.rs, expm1.rs, k_expo2.rs, sinh.rs, cosh.rs,
//   tanh                 tanh.rs                          -> libm/exp.js
//   log, log10, log2     log.rs, log10.rs, log2.rs        -> libm/log.js
//   pow                  pow.rs                           -> libm/pow.js
//   cbrt                 cbrt.rs                          -> libm/cbrt.js
//   fma                  generic/fma.rs                   -> libm/fma.js
//   (helpers)            generic/scalbn.rs, mod.rs words  -> libm/bits.js

export { acos, asin, atan, atan2 } from './libm/atrig.js';
export { cbrt } from './libm/cbrt.js';
export { cosh, exp, sinh, tanh } from './libm/exp.js';
export { fma } from './libm/fma.js';
export { log, log10, log2 } from './libm/log.js';
export { pow } from './libm/pow.js';
export { cos, sin, tan } from './libm/trig.js';

/**
 * `libm::sqrt` (Rust `f64::sqrt`): the correctly rounded square root, which
 * libm computes with `sqrtsd` on x86_64 and ECMAScript engines compute exactly
 * the same way.
 * @param {number} x
 * @returns {number}
 */
export function sqrt(x) {
  return Math.sqrt(x);
}
