// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=a40e8efb584a5bc1169b4753a7b0c615d5192a4751a2caf919e8ea65e3d23119 bytes=8967

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// asin, acos, atan and atan2: a port of libm 0.2.16 src/math/{asin,acos,atan,
// atan2}.rs (FreeBSD e_asin.c, e_acos.c, s_atan.c, e_atan2.c).

// meta-language:carried JavaScript import_statement (unsupported)
// | import { fromBits, getHighWord, getLowWord, withSetLowWord } from './bits.js';

// ---------------------------------------------------------------------------
// asin.rs

// meta-language:translated JavaScript lexical_declaration items=1 sha256=74f007d117456b9a2e3f9f71c13fb6aaea3b9658bf941b13d40dd47d74564229
// | const PIO2_HI = 1.57079632679489655800e+0;
pub const PIO2_HI: f64 = 1.5707963267948966f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=5852e11cdd6bffb2d1acf61d4f364e20f0c178e25a0850df0fd570006894a397
// | /* 0x3FF921FB, 0x54442D18 */
// | const PIO2_LO = 6.12323399573676603587e-17;
pub const PIO2_LO: f64 = 6.123233995736766e-17f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=1744e5a66f5a9e5c3a3dc77ef00847bc13625368aaac40f8c066ce3f4f5a51c9
// | /* 0x3C91A626, 0x33145C07 */
// | /* coefficients for R(x^2) */
// | const P_S0 = 1.66666666666666657415e-1;
pub const P_S0: f64 = 0.16666666666666666f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=6420458574ebc583975e3d7f5e40d0a269cfe7a9d7b0c77eb93ff4ff5f0169ca
// | /* 0x3FC55555, 0x55555555 */
// | const P_S1 = -3.25565818622400915405e-1;
pub const P_S1: f64 = (-0.3255658186224009f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=5cac2bb84b1a26e69f68550bcfa1a539c41bd9a5e926272cc516e733de8798fa
// | /* 0xBFD4D612, 0x03EB6F7D */
// | const P_S2 = 2.01212532134862925881e-1;
pub const P_S2: f64 = 0.20121253213486293f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=2613b019edd733d7c72289834e2aba1252da6df883c2ab1f71ccd4c5dc2360ce
// | /* 0x3FC9C155, 0x0E884455 */
// | const P_S3 = -4.00555345006794114027e-2;
pub const P_S3: f64 = (-0.04005553450067941f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=906b420e0f9d564f2e5dc0ee2a5b38371de6db7985afdcd22d79f77bb89753cf
// | /* 0xBFA48228, 0xB5688F3B */
// | const P_S4 = 7.91534994289814532176e-4;
pub const P_S4: f64 = 0.0007915349942898145f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=538d0d0f89f8e34afc08b1858df1dee03b9d725dca375773ac28de6ad27b7053
// | /* 0x3F49EFE0, 0x7501B288 */
// | const P_S5 = 3.47933107596021167570e-5;
pub const P_S5: f64 = 0.00003479331075960212f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=a3de49744111899b4490959bda42df1a4a710ca8af0753343bdfa0bb2b9c5827
// | /* 0x3F023DE1, 0x0DFDF709 */
// | const Q_S1 = -2.40339491173441421878e+0;
pub const Q_S1: f64 = (-2.403394911734414f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=51a85c14a3c21f4c80b92f1a597d4cb3ecc661e812822040be1d00bf3dfbfe24
// | /* 0xC0033A27, 0x1C8A2D4B */
// | const Q_S2 = 2.02094576023350569471e+0;
pub const Q_S2: f64 = 2.0209457602335057f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=29020276311e0e2e7c87ac23cd8fa957d1477720d83e8de3cc01e058525f7ec1
// | /* 0x40002AE5, 0x9C598AC8 */
// | const Q_S3 = -6.88283971605453293030e-1;
pub const Q_S3: f64 = (-0.6882839716054533f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=0c537de1f0f42873667ebf365a60d4af74cd64dbddd23fdf7d29a08a3e244289
// | /* 0xBFE6066C, 0x1B8D0159 */
// | const Q_S4 = 7.70381505559019352791e-2;
pub const Q_S4: f64 = 0.07703815055590194f64;
/* 0x3FB3B8C5, 0xB12E9282 */

// meta-language:carried JavaScript lexical_declaration (type)
// | const X1P_120 = fromBits(0x3870000000000000n);
// 2^-120

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * R(z) of asin.rs and acos.rs (the two copies are identical).
// |  * @param {number} z
// |  * @returns {number}
// |  */
// | function compR(z) {
// |   const p = z * (P_S0 + z * (P_S1 + z * (P_S2 + z * (P_S3 + z * (P_S4 + z * P_S5)))));
// |   const q = 1.0 + z * (Q_S1 + z * (Q_S2 + z * (Q_S3 + z * Q_S4)));
// |   return p / q;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::asin`.
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function asin(x) {
// |   const hx = getHighWord(x);
// |   const ix = hx & 0x7fffffff;
// |   /* |x| >= 1 or nan */
// |   if (ix >= 0x3ff00000) {
// |     const lx = getLowWord(x);
// |     if (((ix - 0x3ff00000) | lx) === 0) {
// |       /* asin(1) = +-pi/2 with inexact */
// |       return x * PIO2_HI + X1P_120;
// |     } else {
// |       return 0.0 / (x - x);
// |     }
// |   }
// |   /* |x| < 0.5 */
// |   if (ix < 0x3fe00000) {
// |     /* if 0x1p-1022 <= |x| < 0x1p-26, avoid raising underflow */
// |     if (ix >= 0x00100000 && ix < 0x3e500000) {
// |       return x;
// |     } else {
// |       return x + x * compR(x * x);
// |     }
// |   }
// |   /* 1 > |x| >= 0.5 */
// |   const z = (1.0 - Math.abs(x)) * 0.5;
// |   const s = Math.sqrt(z);
// |   const r = compR(z);
// |   if (ix >= 0x3fef3333) {
// |     /* if |x| > 0.975 */
// |     x = PIO2_HI - (2 * (s + s * r) - PIO2_LO);
// |   } else {
// |     /* f+c = sqrt(z) */
// |     const f = withSetLowWord(s, 0);
// |     const c = (z - f * f) / (s + f);
// |     x = 0.5 * PIO2_HI - (2.0 * s * r - (PIO2_LO - 2.0 * c) - (0.5 * PIO2_HI - 2.0 * f));
// |   }
// |   return hx >>> 31 !== 0 ? -x : x;
// | }

// ---------------------------------------------------------------------------
// acos.rs

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::acos`.
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function acos(x) {
// |   const hx = getHighWord(x);
// |   const ix = hx & 0x7fffffff;
// |   /* |x| >= 1 or nan */
// |   if (ix >= 0x3ff00000) {
// |     const lx = getLowWord(x);
// |     if (((ix - 0x3ff00000) | lx) === 0) {
// |       /* acos(1)=0, acos(-1)=pi */
// |       if (hx >>> 31 !== 0) {
// |         return 2 * PIO2_HI + X1P_120;
// |       }
// |       return 0;
// |     }
// |     return 0 / (x - x);
// |   }
// |   /* |x| < 0.5 */
// |   if (ix < 0x3fe00000) {
// |     if (ix <= 0x3c600000) {
// |       /* |x| < 2**-57 */
// |       return PIO2_HI + X1P_120;
// |     }
// |     return PIO2_HI - (x - (PIO2_LO - x * compR(x * x)));
// |   }
// |   /* x < -0.5 */
// |   if (hx >>> 31 !== 0) {
// |     const z = (1.0 + x) * 0.5;
// |     const s = Math.sqrt(z);
// |     const w = compR(z) * s - PIO2_LO;
// |     return 2 * (PIO2_HI - (s + w));
// |   }
// |   /* x > 0.5 */
// |   const z = (1.0 - x) * 0.5;
// |   const s = Math.sqrt(z);
// |   const df = withSetLowWord(s, 0);
// |   const c = (z - df * df) / (s + df);
// |   const w = compR(z) * s + c;
// |   return 2 * (df + w);
// | }

// ---------------------------------------------------------------------------
// atan.rs

// meta-language:translated JavaScript lexical_declaration items=1 sha256=c88f261accd75365c5cbaed3308c242797cd301ea5ad8ba46c6f23d8593e8789
// | const ATANHI = [
// |   4.63647609000806093515e-1, /* atan(0.5)hi 0x3FDDAC67, 0x0561BB4F */
// |   7.85398163397448278999e-1, /* atan(1.0)hi 0x3FE921FB, 0x54442D18 */
// |   9.82793723247329054082e-1, /* atan(1.5)hi 0x3FEF730B, 0xD281F69B */
// |   1.57079632679489655800e+0, /* atan(inf)hi 0x3FF921FB, 0x54442D18 */
// | ];
pub static ATANHI: std::sync::LazyLock<Vec<f64>> = std::sync::LazyLock::new(|| vec![0.4636476090008061f64, 0.7853981633974483f64, 0.982793723247329f64, 1.5707963267948966f64]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=aba7092b898578b1eb5d04cb2e00e85754a29498bc650e46933b1aa58c8527e2
// | const ATANLO = [
// |   2.26987774529616870924e-17, /* atan(0.5)lo 0x3C7A2B7F, 0x222F65E2 */
// |   3.06161699786838301793e-17, /* atan(1.0)lo 0x3C81A626, 0x33145C07 */
// |   1.39033110312309984516e-17, /* atan(1.5)lo 0x3C700788, 0x7AF0CBBD */
// |   6.12323399573676603587e-17, /* atan(inf)lo 0x3C91A626, 0x33145C07 */
// | ];
pub static ATANLO: std::sync::LazyLock<Vec<f64>> = std::sync::LazyLock::new(|| vec![2.2698777452961687e-17f64, 3.061616997868383e-17f64, 1.3903311031230998e-17f64, 6.123233995736766e-17f64]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=13dc44a08b42d7bac94f90a05edef95582b884f294b35154faa936b287c8cda5
// | const AT = [
// |   3.33333333333329318027e-1, /* 0x3FD55555, 0x5555550D */
// |   -1.99999999998764832476e-1, /* 0xBFC99999, 0x9998EBC4 */
// |   1.42857142725034663711e-1, /* 0x3FC24924, 0x920083FF */
// |   -1.11111104054623557880e-1, /* 0xBFBC71C6, 0xFE231671 */
// |   9.09088713343650656196e-2, /* 0x3FB745CD, 0xC54C206E */
// |   -7.69187620504482999495e-2, /* 0xBFB3B0F2, 0xAF749A6D */
// |   6.66107313738753120669e-2, /* 0x3FB10D66, 0xA0D03D51 */
// |   -5.83357013379057348645e-2, /* 0xBFADDE2D, 0x52DEFD9A */
// |   4.97687799461593236017e-2, /* 0x3FA97B4B, 0x24760DEB */
// |   -3.65315727442169155270e-2, /* 0xBFA2B444, 0x2C6A6C2F */
// |   1.62858201153657823623e-2, /* 0x3F90AD3A, 0xE322DA11 */
// | ];
pub static AT: std::sync::LazyLock<Vec<f64>> = std::sync::LazyLock::new(|| vec![0.3333333333333293f64, (-0.19999999999876483f64), 0.14285714272503466f64, (-0.11111110405462356f64), 0.09090887133436507f64, (-0.0769187620504483f64), 0.06661073137387531f64, (-0.058335701337905735f64), 0.049768779946159324f64, (-0.036531572744216916f64), 0.016285820115365782f64]);

// meta-language:carried JavaScript lexical_declaration (type)
// | /** `f64::from_bits(0x0380_0000)`: a tiny subnormal (libm's comment says 0x1p-120f). */
// | const ATAN_TINY = fromBits(0x03800000n);

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::atan`.
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function atan(x) {
// |   let ix = getHighWord(x);
// |   const sign = ix >>> 31;
// |   ix &= 0x7fffffff;
// |   if (ix >= 0x44100000) {
// |     if (Number.isNaN(x)) {
// |       return x;
// |     }
// |     const z = ATANHI[3] + ATAN_TINY;
// |     return sign !== 0 ? -z : z;
// |   }
// |   let id;
// |   if (ix < 0x3fdc0000) {
// |     /* |x| < 0.4375 */
// |     if (ix < 0x3e400000) {
// |       /* |x| < 2^-27 */
// |       return x;
// |     }
// |     id = -1;
// |   } else {
// |     x = Math.abs(x);
// |     if (ix < 0x3ff30000) {
// |       /* |x| < 1.1875 */
// |       if (ix < 0x3fe60000) {
// |         /* 7/16 <= |x| < 11/16 */
// |         x = (2 * x - 1) / (2 + x);
// |         id = 0;
// |       } else {
// |         /* 11/16 <= |x| < 19/16 */
// |         x = (x - 1) / (x + 1);
// |         id = 1;
// |       }
// |     } else if (ix < 0x40038000) {
// |       /* |x| < 2.4375 */
// |       x = (x - 1.5) / (1 + 1.5 * x);
// |       id = 2;
// |     } else {
// |       /* 2.4375 <= |x| < 2^66 */
// |       x = -1 / x;
// |       id = 3;
// |     }
// |   }
// |   const z = x * x;
// |   const w = z * z;
// |   /* break sum from i=0 to 10 AT[i]z**(i+1) into odd and even poly */
// |   const s1 = z * (AT[0] + w * (AT[2] + w * (AT[4] + w * (AT[6] + w * (AT[8] + w * AT[10])))));
// |   const s2 = w * (AT[1] + w * (AT[3] + w * (AT[5] + w * (AT[7] + w * AT[9]))));
// |   if (id < 0) {
// |     return x - x * (s1 + s2);
// |   }
// |   const r = ATANHI[id] - (x * (s1 + s2) - ATANLO[id] - x);
// |   return sign !== 0 ? -r : r;
// | }

// ---------------------------------------------------------------------------
// atan2.rs

// meta-language:translated JavaScript lexical_declaration items=1 sha256=6e54bd4deeeaaf41350000cf8aebf264d06f0c0540263f2f49ad726e61a745c6
// | const PI = 3.1415926535897931160e+0;
pub const PI: f64 = 3.141592653589793f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=8300552dd783898a2d5b64513d9ebd0c92741cab73e3adb2f769ada4313c2a4e
// | /* 0x400921FB, 0x54442D18 */
// | const PI_LO = 1.2246467991473532e-16;
pub const PI_LO: f64 = 1.2246467991473532e-16f64;
/* 1.2246467991473531772e-16, 0x3CA1A626, 0x33145C07 */

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::atan2(y, x)`.
// |  * @param {number} y
// |  * @param {number} x
// |  * @returns {number}
// |  */
// | export function atan2(y, x) {
// |   if (Number.isNaN(x) || Number.isNaN(y)) {
// |     return x + y;
// |   }
// |   let ix = getHighWord(x);
// |   const lx = getLowWord(x);
// |   let iy = getHighWord(y);
// |   const ly = getLowWord(y);
// |   if (((ix - 0x3ff00000) | lx) === 0) {
// |     /* x = 1.0 */
// |     return atan(y);
// |   }
// |   const m = ((iy >>> 31) & 1) | ((ix >>> 30) & 2); /* 2*sign(x)+sign(y) */
// |   ix &= 0x7fffffff;
// |   iy &= 0x7fffffff;
// |
// |   /* when y = 0 */
// |   if ((iy | ly) === 0) {
// |     switch (m) {
// |       case 0:
// |       case 1:
// |         return y; /* atan(+-0,+anything)=+-0 */
// |       case 2:
// |         return PI; /* atan(+0,-anything) = PI */
// |       default:
// |         return -PI; /* atan(-0,-anything) =-PI */
// |     }
// |   }
// |   /* when x = 0 */
// |   if ((ix | lx) === 0) {
// |     return (m & 1) !== 0 ? -PI / 2.0 : PI / 2.0;
// |   }
// |   /* when x is INF */
// |   if (ix === 0x7ff00000) {
// |     if (iy === 0x7ff00000) {
// |       switch (m) {
// |         case 0:
// |           return PI / 4.0; /* atan(+INF,+INF) */
// |         case 1:
// |           return -PI / 4.0; /* atan(-INF,+INF) */
// |         case 2:
// |           return (3.0 * PI) / 4.0; /* atan(+INF,-INF) */
// |         default:
// |           return (-3.0 * PI) / 4.0; /* atan(-INF,-INF) */
// |       }
// |     } else {
// |       switch (m) {
// |         case 0:
// |           return 0.0; /* atan(+...,+INF) */
// |         case 1:
// |           return -0.0; /* atan(-...,+INF) */
// |         case 2:
// |           return PI; /* atan(+...,-INF) */
// |         default:
// |           return -PI; /* atan(-...,-INF) */
// |       }
// |     }
// |   }
// |   /* |y/x| > 0x1p64 */
// |   if ((ix + (64 << 20)) >>> 0 < iy || iy === 0x7ff00000) {
// |     return (m & 1) !== 0 ? -PI / 2.0 : PI / 2.0;
// |   }
// |
// |   /* z = atan(|y/x|) without spurious underflow */
// |   let z;
// |   if ((m & 2) !== 0 && (iy + (64 << 20)) >>> 0 < ix) {
// |     /* |y/x| < 0x1p-64, x<0 */
// |     z = 0.0;
// |   } else {
// |     z = atan(Math.abs(y / x));
// |   }
// |   switch (m) {
// |     case 0:
// |       return z; /* atan(+,+) */
// |     case 1:
// |       return -z; /* atan(-,+) */
// |     case 2:
// |       return PI - (z - PI_LO); /* atan(+,-) */
// |     default:
// |       return (z - PI_LO) - PI; /* atan(-,-) */
// |   }
// | }
