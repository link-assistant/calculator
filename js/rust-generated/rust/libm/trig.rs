// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=f0b74f339447eebb3e028b53aeaf4cee145b77628cc4939034e01b128ee46ee4 bytes=6343

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// sin, cos and tan: a port of libm 0.2.16 src/math/{k_sin,k_cos,k_tan,sin,
// cos,tan}.rs (FreeBSD k_sin.c, k_cos.c, k_tan.c, s_sin.c, s_cos.c, s_tan.c).

// meta-language:translated JavaScript import_statement items=1 sha256=a2ea5cb77584815b0027a9c5574011c80289db5719ba42aef7532ca2518f2630
// | import { getHighWord, withSetLowWord } from './bits.js';
use crate::rust::libm::bits::{getHighWord, withSetLowWord};
// meta-language:translated JavaScript import_statement items=1 sha256=a828745f62380065d52236b4917953c4c1fa3be71e5ceae83ec1dbb4f37e6337
// | import { remPio2 } from './rem_pio2.js';
use crate::rust::libm::rem_pio2::remPio2;

// ---------------------------------------------------------------------------
// k_sin.rs

// meta-language:translated JavaScript lexical_declaration items=1 sha256=61e6011c24fcba1cded86644dfd413cf3f78348044462c5f9cc625d286729bfc
// | const S1 = -1.66666666666666324348e-1;
pub const S1: f64 = (-0.16666666666666632f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=0ba33796ef5fb7a9b024f4e6dd3016196818d8fbd1ab71cc3ea6cd486eeeb7db
// | /* 0xBFC55555, 0x55555549 */
// | const S2 = 8.33333333332248946124e-3;
pub const S2: f64 = 0.00833333333332249f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=4f3916577c511f2a9ea0bd3a44e23fc88d9ed7cc00fdd9320a253ce40693801d
// | /* 0x3F811111, 0x1110F8A6 */
// | const S3 = -1.98412698298579493134e-4;
pub const S3: f64 = (-0.0001984126982985795f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=31e64954567bb644a1a01d532ae6d57a5fc85d70720cdfd5d83e2165b26b7621
// | /* 0xBF2A01A0, 0x19C161D5 */
// | const S4 = 2.75573137070700676789e-6;
pub const S4: f64 = 0.0000027557313707070068f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=82e5f4b2484e0fa301fbf7a10080aeb805b1bb56f18185139f9d32a4ff043bbb
// | /* 0x3EC71DE3, 0x57B1FE7D */
// | const S5 = -2.50507602534068634195e-8;
pub const S5: f64 = (-2.5050760253406863e-8f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=851e95c018ca9718e4424c87675dc6a6ca962b07a307f1228ccdc1a9975e0f64
// | /* 0xBE5AE5E6, 0x8A2B9CEB */
// | const S6 = 1.58969099521155010221e-10;
pub const S6: f64 = 1.58969099521155e-10f64;
/* 0x3DE5D93A, 0x5ACFD57C */

// meta-language:translated JavaScript export_statement items=1 sha256=72fd0e9353d84ed837264ea68e32a54f9b9e04db28a84589bc2365d3f1094320
// | /**
// |  * Kernel sin on [-pi/4, pi/4]; `y` is the tail of x, `iy` says whether it is
// |  * used.
// |  * @param {number} x
// |  * @param {number} y
// |  * @param {number} iy
// |  * @returns {number}
// |  */
// | export function kSin(x, y, iy) {
// |   const z = x * x;
// |   const w = z * z;
// |   const r = S2 + z * (S3 + z * S4) + z * w * (S5 + z * S6);
// |   const v = z * x;
// |   if (iy === 0) {
// |     return x + v * (S1 + z * r);
// |   } else {
// |     return x - ((z * (0.5 * y - v * r) - y) - v * S1);
// |   }
// | }
pub fn k_sin(x: f64, y: f64, iy: f64) -> f64 {
    {
        let z = (x * x);
        {
            let w = (z * z);
            {
                let r = ((S2 + (z * (S3 + (z * S4)))) + ((z * w) * (S5 + (z * S6))));
                {
                    let v = (z * x);
                    if (iy == (0f64)) {
                        (x + (v * (S1 + (z * r))))
                    } else {
                        (x - (((z * ((0.5f64 * y) - (v * r))) - y) - (v * S1)))
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// k_cos.rs

// meta-language:translated JavaScript lexical_declaration items=1 sha256=235888259e1da9fd216b236b411a477606978ae3d12e1f3e74facc1d923b685f
// | const C1 = 4.16666666666666019037e-2;
pub const C1: f64 = 0.0416666666666666f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=ebcf51abcb7e3a91e84dfcd81630263e4e2f2427ee1dde66c1bdf0de69ecbeea
// | /* 0x3FA55555, 0x5555554C */
// | const C2 = -1.38888888888741095749e-3;
pub const C2: f64 = (-0.001388888888887411f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=f9d5dc754c0d28626f68dbc768c3dafd88c235a6ff42364e7cd4ff028cd0b0e6
// | /* 0xBF56C16C, 0x16C15177 */
// | const C3 = 2.48015872894767294178e-5;
pub const C3: f64 = 0.00002480158728947673f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=18a33d70d13da407b1e39835677d7145c73bce28cf2651f4685dbeeca980343e
// | /* 0x3EFA01A0, 0x19CB1590 */
// | const C4 = -2.75573143513906633035e-7;
pub const C4: f64 = (-2.7557314351390663e-7f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=27ff6d1c058660ed716144f956c7a3da0bb4eea6ebe69eb64462feab61528f9b
// | /* 0xBE927E4F, 0x809C52AD */
// | const C5 = 2.08757232129817482790e-9;
pub const C5: f64 = 2.087572321298175e-9f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=5c29ecd9c056baa4a72ce1686753c839c3376ea77b080393847747d50a1dfb20
// | /* 0x3E21EE9E, 0xBDB4B1C4 */
// | const C6 = -1.13596475577881948265e-11;
pub const C6: f64 = (-1.1359647557788195e-11f64);
/* 0xBDA8FAE9, 0xBE8838D4 */

// meta-language:translated JavaScript export_statement items=1 sha256=77842e2c2d1e9cff005d5267c3d26ebffe96a02e216abaf4f7660e0358b96324
// | /**
// |  * Kernel cos on [-pi/4, pi/4]; `y` is the tail of x.
// |  * @param {number} x
// |  * @param {number} y
// |  * @returns {number}
// |  */
// | export function kCos(x, y) {
// |   const z = x * x;
// |   let w = z * z;
// |   const r = z * (C1 + z * (C2 + z * C3)) + w * w * (C4 + z * (C5 + z * C6));
// |   const hz = 0.5 * z;
// |   w = 1.0 - hz;
// |   return w + (((1.0 - w) - hz) + (z * r - x * y));
// | }
pub fn k_cos(x: f64, y: f64) -> f64 {
    {
        let z = (x * x);
        {
            let w = (z * z);
            {
                let r = ((z * (C1 + (z * (C2 + (z * C3))))) + ((w * w) * (C4 + (z * (C5 + (z * C6))))));
                {
                    let hz = (0.5f64 * z);
                    {
                        let w_2 = (1f64 - hz);
                        (w_2 + (((1f64 - w_2) - hz) + ((z * r) - (x * y))))
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// k_tan.rs

// meta-language:translated JavaScript lexical_declaration items=1 sha256=48dfb93a601bc68a317a13e038c1a19a5df1a0e0eb99453bacd3fb20c61c0bd3
// | const T = [
// |   3.33333333333334091986e-1, /* 3FD55555, 55555563 */
// |   1.33333333333201242699e-1, /* 3FC11111, 1110FE7A */
// |   5.39682539762260521377e-2, /* 3FABA1BA, 1BB341FE */
// |   2.18694882948595424599e-2, /* 3F9664F4, 8406D637 */
// |   8.86323982359930005737e-3, /* 3F8226E3, E96E8493 */
// |   3.59207910759131235356e-3, /* 3F6D6D22, C9560328 */
// |   1.45620945432529025516e-3, /* 3F57DBC8, FEE08315 */
// |   5.88041240820264096874e-4, /* 3F4344D8, F2F26501 */
// |   2.46463134818469906812e-4, /* 3F3026F7, 1A8D1068 */
// |   7.81794442939557092300e-5, /* 3F147E88, A03792A6 */
// |   7.14072491382608190305e-5, /* 3F12B80F, 32F0A7E9 */
// |   -1.85586374855275456654e-5, /* BEF375CB, DB605373 */
// |   2.59073051863633712884e-5, /* 3EFB2A70, 74BF7AD4 */
// | ];
pub static T: std::sync::LazyLock<Vec<f64>> = std::sync::LazyLock::new(|| vec![0.3333333333333341f64, 0.13333333333320124f64, 0.05396825397622605f64, 0.021869488294859542f64, 0.0088632398235993f64, 0.0035920791075913124f64, 0.0014562094543252903f64, 0.0005880412408202641f64, 0.0002464631348184699f64, 0.00007817944429395571f64, 0.00007140724913826082f64, (-0.000018558637485527546f64), 0.00002590730518636337f64]);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=41adfb17671646e9485c81f423c10cbb5480399dee24323fda0e37c13a7c3901
// | const PIO4 = 7.85398163397448278999e-1;
pub const PIO4: f64 = 0.7853981633974483f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=47c6d500f4c4b40b26d675dae2dc311fec3fb447e6e92cc635168097e3ba0de2
// | /* 3FE921FB, 54442D18 */
// | const PIO4_LO = 3.06161699786838301793e-17;
pub const PIO4_LO: f64 = 3.061616997868383e-17f64;
/* 3C81A626, 33145C07 */

// meta-language:carried JavaScript function_declaration (type)
// | /**
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | function zeroLowWord(x) {
// |   return withSetLowWord(x, 0);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Kernel tan on [-pi/4, pi/4]; returns -1/tan when `odd` is 1.
// |  * @param {number} x
// |  * @param {number} y
// |  * @param {number} odd
// |  * @returns {number}
// |  */
// | export function kTan(x, y, odd) {
// |   const hx = getHighWord(x);
// |   const big = (hx & 0x7fffffff) >= 0x3fe59428; /* |x| >= 0.6744 */
// |   if (big) {
// |     const sign = hx >>> 31;
// |     if (sign !== 0) {
// |       x = -x;
// |       y = -y;
// |     }
// |     x = (PIO4 - x) + (PIO4_LO - y);
// |     y = 0.0;
// |   }
// |   const z = x * x;
// |   const w = z * z;
// |   /*
// |    * Break x^5*(T[1]+x^2*T[2]+...) into
// |    * x^5(T[1]+x^4*T[3]+...+x^20*T[11]) +
// |    * x^5(x^2*(T[2]+x^4*T[4]+...+x^22*[T12]))
// |    */
// |   let r = T[1] + w * (T[3] + w * (T[5] + w * (T[7] + w * (T[9] + w * T[11]))));
// |   let v = z * (T[2] + w * (T[4] + w * (T[6] + w * (T[8] + w * (T[10] + w * T[12])))));
// |   const s = z * x;
// |   r = y + z * (s * (r + v) + y) + s * T[0];
// |   const w2 = x + r;
// |   if (big) {
// |     const sign = hx >>> 31;
// |     const s = 1.0 - 2.0 * odd;
// |     const v = s - 2.0 * (x + (r - w2 * w2 / (w2 + s)));
// |     return sign !== 0 ? -v : v;
// |   }
// |   if (odd === 0) {
// |     return w2;
// |   }
// |   /* -1.0/(x+r) has up to 2ulp error, so compute it accurately */
// |   const w0 = zeroLowWord(w2);
// |   v = r - (w0 - x); /* w0+v = r+x */
// |   const a = -1.0 / w2;
// |   const a0 = zeroLowWord(a);
// |   return a0 + a * (1.0 + a0 * w0 + a0 * v);
// | }

// ---------------------------------------------------------------------------
// sin.rs, cos.rs, tan.rs

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::sin`.
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function sin(x) {
// |   /* High word of x. */
// |   const ix = getHighWord(x) & 0x7fffffff;
// |
// |   /* |x| ~< pi/4 */
// |   if (ix <= 0x3fe921fb) {
// |     if (ix < 0x3e500000) {
// |       /* |x| < 2**-26 */
// |       return x;
// |     }
// |     return kSin(x, 0.0, 0);
// |   }
// |
// |   /* sin(Inf or NaN) is NaN */
// |   if (ix >= 0x7ff00000) {
// |     return x - x;
// |   }
// |
// |   /* argument reduction needed */
// |   const [n, y0, y1] = remPio2(x);
// |   switch (n & 3) {
// |     case 0:
// |       return kSin(y0, y1, 1);
// |     case 1:
// |       return kCos(y0, y1);
// |     case 2:
// |       return -kSin(y0, y1, 1);
// |     default:
// |       return -kCos(y0, y1);
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::cos`.
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function cos(x) {
// |   const ix = getHighWord(x) & 0x7fffffff;
// |
// |   /* |x| ~< pi/4 */
// |   if (ix <= 0x3fe921fb) {
// |     if (ix < 0x3e46a09e) {
// |       /* if x < 2**-27 * sqrt(2) */
// |       /* raise inexact if x != 0 */
// |       if ((x | 0) === 0) {
// |         return 1.0;
// |       }
// |     }
// |     return kCos(x, 0.0);
// |   }
// |
// |   /* cos(Inf or NaN) is NaN */
// |   if (ix >= 0x7ff00000) {
// |     return x - x;
// |   }
// |
// |   /* argument reduction needed */
// |   const [n, y0, y1] = remPio2(x);
// |   switch (n & 3) {
// |     case 0:
// |       return kCos(y0, y1);
// |     case 1:
// |       return -kSin(y0, y1, 1);
// |     case 2:
// |       return -kCos(y0, y1);
// |     default:
// |       return kSin(y0, y1, 1);
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::tan`.
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function tan(x) {
// |   const ix = getHighWord(x) & 0x7fffffff;
// |
// |   /* |x| ~< pi/4 */
// |   if (ix <= 0x3fe921fb) {
// |     if (ix < 0x3e400000) {
// |       /* |x| < 2**-27 */
// |       return x;
// |     }
// |     return kTan(x, 0.0, 0);
// |   }
// |
// |   /* tan(Inf or NaN) is NaN */
// |   if (ix >= 0x7ff00000) {
// |     return x - x;
// |   }
// |
// |   /* argument reduction */
// |   const [n, y0, y1] = remPio2(x);
// |   return kTan(y0, y1, n & 1);
// | }
