// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=96841b8c72589ae2d473ed178e39d9bcecb59f0f51a7aebe46114204b4760cfa bytes=4983

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// Logarithms: a port of libm 0.2.16 src/math/{log,log10,log2}.rs (FreeBSD
// e_log.c, e_log10.c, e_log2.c).

// meta-language:carried JavaScript import_statement (unsupported)
// | import { combineWords, fromBits, getHighWord, getLowWord, withSetLowWord } from './bits.js';

// meta-language:translated JavaScript lexical_declaration items=1 sha256=5b9c88edfc3b0f416c089aad5fc89ac95b303dfb191448a4038ab5479b715103
// | const LN2_HI = 6.93147180369123816490e-1;
pub const LN2_HI: f64 = 0.6931471803691238f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=bb9e5118418ef03fe8eb95ad90cebdf4c9700876e6d3c87c248ab828c8bb5407
// | /* 3fe62e42 fee00000 */
// | const LN2_LO = 1.90821492927058770002e-10;
pub const LN2_LO: f64 = 1.9082149292705877e-10f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=b38d219c12f28d65fa1cdbea86141b3d0793d29187e4852cc9cc1b3733477562
// | /* 3dea39ef 35793c76 */
// | const IVLN10HI = 4.34294481878168880939e-1;
pub const IVLN10HI: f64 = 0.4342944818781689f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=d4801608221e41ad3f476d7d60dc121572fd7c59f90700cb17f68622c1d48930
// | /* 0x3fdbcb7b, 0x15200000 */
// | const IVLN10LO = 2.50829467116452752298e-11;
pub const IVLN10LO: f64 = 2.5082946711645275e-11f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=faef307191f2a1207727c9607bb9cbd0e6889e45490fad3ff25acb2e3b9f0804
// | /* 0x3dbb9438, 0xca9aadd5 */
// | const LOG10_2HI = 3.01029995663611771306e-1;
pub const LOG10_2HI: f64 = 0.30102999566361177f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=38e05ac9878090ff26dc1b1331ac10a93c34ba7b85ebb29badcb27cf2f91000d
// | /* 0x3FD34413, 0x509F6000 */
// | const LOG10_2LO = 3.69423907715893078616e-13;
pub const LOG10_2LO: f64 = 3.694239077158931e-13f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=95563988ee0d55b62ad611429aeade68967b1746c03e707b82cabfd1b92be2ac
// | /* 0x3D59FEF3, 0x11F12B36 */
// | const IVLN2HI = 1.44269504072144627571e+0;
pub const IVLN2HI: f64 = 1.4426950407214463f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=8749afd54d50a5f1a3fe5fe51d7be9d15935ce08f4eb6546fa3c019bcfc01e05
// | /* 0x3ff71547, 0x65200000 */
// | const IVLN2LO = 1.67517131648865118353e-10;
pub const IVLN2LO: f64 = 1.6751713164886512e-10f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=04e4a935db6b8f6ba875e547bc09f4fedc58058227b76bb9e0a9a9a5faa2894c
// | /* 0x3de705fc, 0x2eefa200 */
// | const LG1 = 6.666666666666735130e-1;
pub const LG1: f64 = 0.6666666666666735f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=d9522f5d749eef5fa990dbf1a2b264b8339654736a431173617708f6b55dbfc6
// | /* 3FE55555 55555593 */
// | const LG2 = 3.999999999940941908e-1;
pub const LG2: f64 = 0.3999999999940942f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=66eb5302d9be3fcfcbae246ab675486fe7304dd4d26d941f250c761b6953a8ea
// | /* 3FD99999 9997FA04 */
// | const LG3 = 2.857142874366239149e-1;
pub const LG3: f64 = 0.2857142874366239f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=312a0b399bff202f5408e98c12d92339cd3fccc56d712b4d84351b73b632f2df
// | /* 3FD24924 94229359 */
// | const LG4 = 2.222219843214978396e-1;
pub const LG4: f64 = 0.22222198432149784f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=a53540d9a829e860b8676e16225ea7ec2b232a53821ddc5aa164a01f3092a38c
// | /* 3FCC71C5 1D8E78AF */
// | const LG5 = 1.818357216161805012e-1;
pub const LG5: f64 = 0.1818357216161805f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=57a4d328b0cd5ae24c918275324db9eb139bf3458e341d29ee80b54cd64b9c22
// | /* 3FC74664 96CB03DE */
// | const LG6 = 1.531383769920937332e-1;
pub const LG6: f64 = 0.15313837699209373f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=87add29cc0c9b38897ebd889b77e8aaa54f5e539ee50ebca1004e0c4a691b078
// | /* 3FC39A09 D078C69F */
// | const LG7 = 1.479819860511658591e-1;
pub const LG7: f64 = 0.14798198605116586f64;
/* 3FC2F112 DF3E5244 */

// meta-language:carried JavaScript lexical_declaration (type)
// | const X1P54 = fromBits(0x4350000000000000n);
// 2^54

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * The argument handling shared by log, log10 and log2: either a final result
// |  * (`done` with `value`), or `x` reduced into [sqrt(2)/2, sqrt(2)] with its
// |  * exponent `k`.
// |  * @typedef {{ done: boolean, value: number, x: number, k: number }} Reduced
// |  */

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {number} x
// |  * @returns {Reduced}
// |  */
// | function reduce(x) {
// |   let hx = getHighWord(x);
// |   let lx = getLowWord(x);
// |   let k = 0;
// |
// |   if (hx < 0x00100000 || hx >>> 31 !== 0) {
// |     /* x < 2**-126  */
// |     if (((hx & 0x7fffffff) | lx) === 0) {
// |       return { done: true, value: -1 / (x * x), x: 0, k: 0 }; /* log(+-0)=-inf */
// |     }
// |     if (hx >>> 31 !== 0) {
// |       return { done: true, value: (x - x) / 0.0, x: 0, k: 0 }; /* log(-#) = NaN */
// |     }
// |     /* subnormal number, scale x up */
// |     k -= 54;
// |     x *= X1P54;
// |     hx = getHighWord(x);
// |     lx = getLowWord(x);
// |   } else if (hx >= 0x7ff00000) {
// |     return { done: true, value: x, x: 0, k: 0 };
// |   } else if (hx === 0x3ff00000 && lx === 0) {
// |     return { done: true, value: 0, x: 0, k: 0 };
// |   }
// |
// |   /* reduce x into [sqrt(2)/2, sqrt(2)] */
// |   hx += 0x3ff00000 - 0x3fe6a09e;
// |   k += (hx >>> 20) - 0x3ff;
// |   hx = (hx & 0x000fffff) + 0x3fe6a09e;
// |   x = combineWords(hx, lx);
// |   return { done: false, value: 0, x, k };
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::log` (Rust `f64::ln`).
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function log(x) {
// |   const red = reduce(x);
// |   if (red.done) {
// |     return red.value;
// |   }
// |   const k = red.k;
// |   x = red.x;
// |   const f = x - 1.0;
// |   const hfsq = 0.5 * f * f;
// |   const s = f / (2.0 + f);
// |   const z = s * s;
// |   const w = z * z;
// |   const t1 = w * (LG2 + w * (LG4 + w * LG6));
// |   const t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
// |   const r = t2 + t1;
// |   const dk = k;
// |   return s * (hfsq + r) + dk * LN2_LO - hfsq + f + dk * LN2_HI;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::log10`.
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function log10(x) {
// |   const red = reduce(x);
// |   if (red.done) {
// |     return red.value;
// |   }
// |   const k = red.k;
// |   x = red.x;
// |   const f = x - 1.0;
// |   const hfsq = 0.5 * f * f;
// |   const s = f / (2.0 + f);
// |   const z = s * s;
// |   let w = z * z;
// |   const t1 = w * (LG2 + w * (LG4 + w * LG6));
// |   const t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
// |   const r = t2 + t1;
// |
// |   /* See log2.c for details. */
// |   /* hi+lo = f - hfsq + s*(hfsq+R) ~ log(1+f) */
// |   const hi = withSetLowWord(f - hfsq, 0);
// |   const lo = f - hi - hfsq + s * (hfsq + r);
// |
// |   /* val_hi+val_lo ~ log10(1+f) + k*log10(2) */
// |   let valHi = hi * IVLN10HI;
// |   const dk = k;
// |   const y = dk * LOG10_2HI;
// |   let valLo = dk * LOG10_2LO + (lo + hi) * IVLN10LO + lo * IVLN10HI;
// |
// |   /*
// |    * Extra precision in for adding y is not strictly needed
// |    * since there is no very large cancellation near x = sqrt(2) or
// |    * x = 1/sqrt(2), but we do it anyway since it costs little on CPUs
// |    * with some parallelism and it reduces the error for many args.
// |    */
// |   w = y + valHi;
// |   valLo += (y - w) + valHi;
// |   valHi = w;
// |
// |   return valLo + valHi;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::log2`.
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function log2(x) {
// |   const red = reduce(x);
// |   if (red.done) {
// |     return red.value;
// |   }
// |   const k = red.k;
// |   x = red.x;
// |   const f = x - 1.0;
// |   const hfsq = 0.5 * f * f;
// |   const s = f / (2.0 + f);
// |   const z = s * s;
// |   let w = z * z;
// |   const t1 = w * (LG2 + w * (LG4 + w * LG6));
// |   const t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
// |   const r = t2 + t1;
// |
// |   /* hi+lo = f - hfsq + s*(hfsq+R) ~ log(1+f) */
// |   const hi = withSetLowWord(f - hfsq, 0);
// |   const lo = f - hi - hfsq + s * (hfsq + r);
// |
// |   let valHi = hi * IVLN2HI;
// |   let valLo = (lo + hi) * IVLN2LO + lo * IVLN2HI;
// |
// |   /* spadd(val_hi, val_lo, y), except for not using double_t: */
// |   const y = k;
// |   w = y + valHi;
// |   valLo += (y - w) + valHi;
// |   valHi = w;
// |
// |   return valLo + valHi;
// | }
