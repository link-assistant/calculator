// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=ccc15df0077eb1c28b625f463574d94583924493b4e31007ef00c49f00051254 bytes=8207

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// exp, expm1 and the hyperbolic functions: a port of libm 0.2.16
// src/math/{exp,expm1,expo2,k_expo2,sinh,cosh,tanh}.rs (FreeBSD e_exp.c,
// s_expm1.c, k_exp.c, e_sinh.c, e_cosh.c, s_tanh.c).

// meta-language:carried JavaScript import_statement (unsupported)
// | import { combineWords, fromBits, getHighWord, scalbn, withSetHighWord } from './bits.js';

// ---------------------------------------------------------------------------
// exp.rs

// meta-language:translated JavaScript lexical_declaration items=1 sha256=5da305ab04bca7e8d99f73368258ee7cc80e31d9fab35872fa1d177de3f23de7
// | const HALF = [0.5, -0.5];
pub static HALF: std::sync::LazyLock<Vec<f64>> = std::sync::LazyLock::new(|| vec![0.5f64, (-0.5f64)]);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=88fb21f850190e5546201921be2a204a40918ee43fcb176d00f078319ac4fbd6
// | const LN2HI = 6.93147180369123816490e-1;
pub const LN2HI: f64 = 0.6931471803691238f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=5519d94333e0f396b2b2a58d76eff219d19c58e6e3f6b9713645d14b2da6ff4f
// | /* 0x3fe62e42, 0xfee00000 */
// | const LN2LO = 1.90821492927058770002e-10;
pub const LN2LO: f64 = 1.9082149292705877e-10f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=d1e45805a806c41917679612c4baf8761f69af5109070b1010122c208bc568de
// | /* 0x3dea39ef, 0x35793c76 */
// | const INVLN2 = 1.44269504088896338700e+0;
pub const INVLN2: f64 = 1.4426950408889634f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=eac7abc939b1df61f2371bf875ace6c0a393ab8cb4201c8f0f2f476e14075fde
// | /* 0x3ff71547, 0x652b82fe */
// | const P1 = 1.66666666666666019037e-1;
pub const P1: f64 = 0.16666666666666602f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=b2fff89f5e53f45c480901765a1bd809c085ee24854bb6bba26770b82835902a
// | /* 0x3FC55555, 0x5555553E */
// | const P2 = -2.77777777770155933842e-3;
pub const P2: f64 = (-0.0027777777777015593f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=79744faef559d2d0406a3a2f619bcd28bfb1b8021e917b384c34eefccfb140eb
// | /* 0xBF66C16C, 0x16BEBD93 */
// | const P3 = 6.61375632143793436117e-5;
pub const P3: f64 = 0.00006613756321437934f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=e4415439906c39f71b72ac0593a63fd5b0c502f29e572485ddbdfbfdaa4ea197
// | /* 0x3F11566A, 0xAF25DE2C */
// | const P4 = -1.65339022054652515390e-6;
pub const P4: f64 = (-0.0000016533902205465252f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=d8c07de4f8770f5a3cfdb34365f8219d80f293dc68350b6876c1a2698e1af755
// | /* 0xBEBBBD41, 0xC5D26BF1 */
// | const P5 = 4.13813679705723846039e-8;
pub const P5: f64 = 4.1381367970572385e-8f64;
/* 0x3E663769, 0x72BEA4D0 */

// meta-language:carried JavaScript lexical_declaration (type)
// | const X1P1023 = fromBits(0x7fe0000000000000n);
// 2^1023

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::exp`.
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function exp(x) {
// |   let hi;
// |   let lo;
// |   let k;
// |
// |   let hx = getHighWord(x);
// |   const sign = hx >>> 31;
// |   hx &= 0x7fffffff; /* high word of |x| */
// |
// |   /* special cases */
// |   if (hx >= 0x4086232b) {
// |     /* if |x| >= 708.39... */
// |     if (Number.isNaN(x)) {
// |       return x;
// |     }
// |     if (x > 709.782712893383973096) {
// |       /* overflow if x!=inf */
// |       x *= X1P1023;
// |       return x;
// |     }
// |     if (x < -708.3964185322641) {
// |       /* -708.39641853226410622 in the source */
// |       /* underflow if x!=-inf */
// |       if (x < -745.13321910194110842) {
// |         return 0;
// |       }
// |     }
// |   }
// |
// |   /* argument reduction */
// |   if (hx > 0x3fd62e42) {
// |     /* if |x| > 0.5 ln2 */
// |     if (hx >= 0x3ff0a2b2) {
// |       /* if |x| >= 1.5 ln2 */
// |       k = (INVLN2 * x + HALF[sign]) | 0;
// |     } else {
// |       k = 1 - sign - sign;
// |     }
// |     hi = x - k * LN2HI; /* k*ln2hi is exact here */
// |     lo = k * LN2LO;
// |     x = hi - lo;
// |   } else if (hx > 0x3e300000) {
// |     /* if |x| > 2**-28 */
// |     k = 0;
// |     hi = x;
// |     lo = 0;
// |   } else {
// |     /* inexact if x!=0 */
// |     return 1 + x;
// |   }
// |
// |   /* x is now in primary range */
// |   const xx = x * x;
// |   const c = x - xx * (P1 + xx * (P2 + xx * (P3 + xx * (P4 + xx * P5))));
// |   const y = 1 + (x * c / (2 - c) - lo + hi);
// |   return k === 0 ? y : scalbn(y, k);
// | }

// ---------------------------------------------------------------------------
// expm1.rs

// meta-language:translated JavaScript lexical_declaration items=1 sha256=c292d909c8afd60d3bb830f29b77cc859466a1e953d54a6e2d2e015d3267a02c
// | const O_THRESHOLD = 7.09782712893383973096e+2;
pub const O_THRESHOLD: f64 = 709.782712893384f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=5b9c88edfc3b0f416c089aad5fc89ac95b303dfb191448a4038ab5479b715103
// | /* 0x40862E42, 0xFEFA39EF */
// | const LN2_HI = 6.93147180369123816490e-1;
pub const LN2_HI: f64 = 0.6931471803691238f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=bb9e5118418ef03fe8eb95ad90cebdf4c9700876e6d3c87c248ab828c8bb5407
// | /* 0x3fe62e42, 0xfee00000 */
// | const LN2_LO = 1.90821492927058770002e-10;
pub const LN2_LO: f64 = 1.9082149292705877e-10f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=32a3344881e8a7db2d751b18639844e0fdeb3b1c1e23961bd2f4d903133f3e1a
// | /* 0x3dea39ef, 0x35793c76 */
// | /* Scaled Q's: Qn_here = 2**n * Qn_above, for R(2*z) where z = hxs = x*x/2: */
// | const Q1 = -3.33333333333331316428e-2;
pub const Q1: f64 = (-0.03333333333333313f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=08813f7aa47aeb1ccf5c8955f0b109dca1bb32e137aa1d032711eac63b1604f6
// | /* BFA11111 111110F4 */
// | const Q2 = 1.58730158725481460165e-3;
pub const Q2: f64 = 0.0015873015872548146f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=8b982e4ba0ba830d59b802f51097c5cda69dfed444f94da3a36b941663589d56
// | /* 3F5A01A0 19FE5585 */
// | const Q3 = -7.93650757867487942473e-5;
pub const Q3: f64 = (-0.0000793650757867488f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=c02b76841a6a3f0e3edc46c94754a28c677aefcf1434f02138b6c4bb5983a65d
// | /* BF14CE19 9EAADBB7 */
// | const Q4 = 4.00821782732936239552e-6;
pub const Q4: f64 = 0.000004008217827329362f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=1c22d7b4e173a278fc464231f7e6337b43119b99df7945023ed3ecfa94840e35
// | /* 3ED0CFCA 86E65239 */
// | const Q5 = -2.01099218183624371326e-7;
pub const Q5: f64 = (-2.0109921818362437e-7f64);
/* BE8AFDB7 6E09C32D */

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::expm1`: e^x - 1.
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function expm1(x) {
// |   let hi;
// |   let lo;
// |   let k;
// |   let c;
// |   let t;
// |   let y;
// |
// |   const hxw = getHighWord(x);
// |   const hx = hxw & 0x7fffffff;
// |   const sign = hxw >>> 31;
// |
// |   /* filter out huge and non-finite argument */
// |   if (hx >= 0x4043687a) {
// |     /* if |x|>=56*ln2 */
// |     if (Number.isNaN(x)) {
// |       return x;
// |     }
// |     if (sign !== 0) {
// |       return -1.0;
// |     }
// |     if (x > O_THRESHOLD) {
// |       x *= X1P1023;
// |       return x;
// |     }
// |   }
// |
// |   /* argument reduction */
// |   if (hx > 0x3fd62e42) {
// |     /* if  |x| > 0.5 ln2 */
// |     if (hx < 0x3ff0a2b2) {
// |       /* and |x| < 1.5 ln2 */
// |       if (sign === 0) {
// |         hi = x - LN2_HI;
// |         lo = LN2_LO;
// |         k = 1;
// |       } else {
// |         hi = x + LN2_HI;
// |         lo = -LN2_LO;
// |         k = -1;
// |       }
// |     } else {
// |       k = (INVLN2 * x + (sign !== 0 ? -0.5 : 0.5)) | 0;
// |       t = k;
// |       hi = x - t * LN2_HI; /* t*ln2_hi is exact here */
// |       lo = t * LN2_LO;
// |     }
// |     x = hi - lo;
// |     c = (hi - x) - lo;
// |   } else if (hx < 0x3c900000) {
// |     /* |x| < 2**-54, return x */
// |     return x;
// |   } else {
// |     c = 0.0;
// |     k = 0;
// |   }
// |
// |   /* x is now in primary range */
// |   const hfx = 0.5 * x;
// |   const hxs = x * hfx;
// |   const r1 = 1.0 + hxs * (Q1 + hxs * (Q2 + hxs * (Q3 + hxs * (Q4 + hxs * Q5))));
// |   t = 3.0 - r1 * hfx;
// |   let e = hxs * ((r1 - t) / (6.0 - x * t));
// |   if (k === 0) {
// |     /* c is 0 */
// |     return x - (x * e - hxs);
// |   }
// |   e = x * (e - c) - c;
// |   e -= hxs;
// |   /* exp(x) ~ 2^k (x_reduced - e + 1) */
// |   if (k === -1) {
// |     return 0.5 * (x - e) - 0.5;
// |   }
// |   if (k === 1) {
// |     if (x < -0.25) {
// |       return -2.0 * (e - (x + 0.5));
// |     }
// |     return 1.0 + 2.0 * (x - e);
// |   }
// |   const twopk = combineWords((0x3ff + k) << 20, 0); /* 2^k */
// |   if (!(k >= 0 && k <= 56)) {
// |     /* suffice to return exp(x)-1 */
// |     y = x - e + 1.0;
// |     if (k === 1024) {
// |       y = y * 2.0 * X1P1023;
// |     } else {
// |       y = y * twopk;
// |     }
// |     return y - 1.0;
// |   }
// |   const uf = combineWords((0x3ff - k) << 20, 0); /* 2^-k */
// |   if (k < 20) {
// |     y = (x - e + (1.0 - uf)) * twopk;
// |   } else {
// |     y = (x - (e + uf) + 1.0) * twopk;
// |   }
// |   return y;
// | }

// ---------------------------------------------------------------------------
// k_expo2.rs (expo2.rs is the same function)

// meta-language:translated JavaScript lexical_declaration items=1 sha256=06bf68cc295139fdb68fde56704fa499d418847f676b6529cdd69d063616d043
// | /* k is such that k*ln2 has minimal relative error and x - kln2 > log(DBL_MIN) */
// | const K = 2043;
pub const K: f64 = 2043f64;
// meta-language:carried JavaScript lexical_declaration (type)
// | const K_LN2 = fromBits(0x40962066151add8bn);
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /* note that k is odd and scale*scale overflows */
// | const EXPO2_SCALE = combineWords((0x3ff + Math.trunc(K / 2)) << 20, 0);

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * exp(x)/2 for x >= log(DBL_MAX).
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | function kExpo2(x) {
// |   /* exp(x - k ln2) * 2**(k-1) */
// |   return exp(x - K_LN2) * EXPO2_SCALE * EXPO2_SCALE;
// | }

// ---------------------------------------------------------------------------
// sinh.rs, cosh.rs, tanh.rs

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * |x| by clearing the sign bit (`ui &= !1 / 2`).
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | function clearSign(x) {
// |   return withSetHighWord(x, getHighWord(x) & 0x7fffffff);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::sinh`.
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function sinh(x) {
// |   let h = 0.5;
// |   if (getHighWord(x) >>> 31 !== 0) {
// |     h = -h;
// |   }
// |   /* |x| */
// |   const absx = clearSign(x);
// |   const w = getHighWord(absx);
// |
// |   /* |x| < log(DBL_MAX) */
// |   if (w < 0x40862e42) {
// |     const t = expm1(absx);
// |     if (w < 0x3ff00000) {
// |       if (w < 0x3ff00000 - (26 << 20)) {
// |         /* note: this branch avoids spurious underflow */
// |         return x;
// |       }
// |       return h * (2.0 * t - t * t / (t + 1.0));
// |     }
// |     /* note: |x|>log(0x1p26)+eps could be just h*exp(x) */
// |     return h * (t + t / (t + 1.0));
// |   }
// |
// |   /* |x| > log(DBL_MAX) or nan */
// |   /* note: the result is stored to handle overflow */
// |   return 2.0 * h * kExpo2(absx);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::cosh`.
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function cosh(x) {
// |   /* |x| */
// |   x = clearSign(x);
// |   const w = getHighWord(x);
// |
// |   /* |x| < log(2) */
// |   if (w < 0x3fe62e42) {
// |     if (w < 0x3ff00000 - (26 << 20)) {
// |       return 1;
// |     }
// |     const t = expm1(x); // exponential minus 1
// |     return 1 + t * t / (2 * (1 + t));
// |   }
// |
// |   /* |x| < log(DBL_MAX) */
// |   if (w < 0x40862e42) {
// |     const t = exp(x);
// |     /* note: if x>log(0x1p26) then the 1/t is not needed */
// |     return 0.5 * (t + 1 / t);
// |   }
// |
// |   /* |x| > log(DBL_MAX) or nan */
// |   return kExpo2(x);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::tanh`.
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function tanh(x) {
// |   let t;
// |
// |   /* x = |x| */
// |   const sign = getHighWord(x) >>> 31 !== 0;
// |   x = clearSign(x);
// |   const w = getHighWord(x);
// |
// |   if (w > 0x3fe193ea) {
// |     /* |x| > log(3)/2 ~= 0.5493 or nan */
// |     if (w > 0x40340000) {
// |       /* |x| > 20 or nan */
// |       /* note: this branch avoids raising overflow */
// |       t = 1.0 - 0.0 / x;
// |     } else {
// |       t = expm1(2.0 * x);
// |       t = 1.0 - 2.0 / (t + 2.0);
// |     }
// |   } else if (w > 0x3fd058ae) {
// |     /* |x| > log(5/3)/2 ~= 0.2554 */
// |     t = expm1(2.0 * x);
// |     t = t / (t + 2.0);
// |   } else if (w >= 0x00100000) {
// |     /* |x| >= 0x1p-1022, up to 2ulp error in [0.1,0.2554] */
// |     t = expm1(-2.0 * x);
// |     t = -t / (t + 2.0);
// |   } else {
// |     /* |x| is subnormal */
// |     t = x;
// |   }
// |   return sign ? -t : t;
// | }
