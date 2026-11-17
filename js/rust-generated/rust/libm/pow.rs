// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=59a4063f8d094e27fd26e6b4b0bfb43762552f713db3752c2d4dba97e1058ba9 bytes=9232

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// pow: a port of libm 0.2.16 src/math/pow.rs (FreeBSD e_pow.c).
//
// The high words are i32 in Rust (`(x.to_bits() >> 32) as i32`), so they are
// read with `| 0`; the low words stay u32 numbers.

// meta-language:translated JavaScript import_statement items=1 sha256=4c93fc6ffa41c039adebfc386272485cb25b4d9f8d2610835288b6909d2d5c23
// | import { combineWords, getHighWord, getLowWord, scalbn, withSetHighWord, withSetLowWord } from './bits.js';
use crate::rust::libm::bits::{combineWords, getHighWord, getLowWord, scalbn, withSetHighWord, withSetLowWord};

// meta-language:translated JavaScript lexical_declaration items=1 sha256=2bb06408739aa15485e5837a8eea0cbad2c588d5606c7bfdcb6fad37578325ab
// | const BP = [1.0, 1.5];
pub static BP: std::sync::LazyLock<Vec<f64>> = std::sync::LazyLock::new(|| vec![1f64, 1.5f64]);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=25aaeaab01c7b15580e32a4f2e2426790545c39ec1c9387124a608dffc9a1871
// | const DP_H = [0.0, 5.84962487220764160156e-1];
pub static DP_H: std::sync::LazyLock<Vec<f64>> = std::sync::LazyLock::new(|| vec![0f64, 0.5849624872207642f64]);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=fb0e9ecaa04ceac008c34174ba9b7021b3a323ffdbdf5026572c640a042a866e
// | /* 0x3fe2b803_40000000 */
// | const DP_L = [0.0, 1.35003920212974897128e-8];
pub static DP_L: std::sync::LazyLock<Vec<f64>> = std::sync::LazyLock::new(|| vec![0f64, 1.350039202129749e-8f64]);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=931c3f947fa0bf72114e35c307ec0b058c8890a295760758c3bd15fb67662ddb
// | /* 0x3E4CFDEB, 0x43CFD006 */
// | const TWO53 = 9007199254740992.0;
pub const TWO53: f64 = 9007199254740992f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=cf9815d9aa099d0622119642cbf10c5799820412a7e7285c97b19bc0a7d8a8a7
// | /* 0x43400000_00000000 */
// | const HUGE = 1.0e300;
pub const HUGE: f64 = 1e+300f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=fa1c351d73df11bda7d2f39f3f8b5005573f5cd6285cf70721acd1198e51ccf2
// | const TINY = 1.0e-300;
pub const TINY: f64 = 1e-300f64;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=c25410afb937a54a9297922f8906b49dc71b87d539af3c34e79db3084de8657c
// | // poly coefs for (3/2)*(log(x)-2s-2/3*s**3:
// | const L1 = 5.99999999999994648725e-1;
pub const L1: f64 = 0.5999999999999946f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=a71f06cb1616107f87b393f62ea5e72c977a803a498bf708c69be0a44404efab
// | /* 0x3fe33333_33333303 */
// | const L2 = 4.28571428578550184252e-1;
pub const L2: f64 = 0.4285714285785502f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=09e17104cfe02a8e7a3fc897b23022bb2792dbcae8be983be738073039c3e6de
// | /* 0x3fdb6db6_db6fabff */
// | const L3 = 3.33333329818377432918e-1;
pub const L3: f64 = 0.33333332981837743f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=a8ab4aa9960248d5687d35566ddb87901404da4e4cb385ad2ebe3307170d148e
// | /* 0x3fd55555_518f264d */
// | const L4 = 2.72728123808534006489e-1;
pub const L4: f64 = 0.272728123808534f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=7e778fd03b98615b39f587244810b9ba6ec166be9a5cb77c0adcd6bad3fb7129
// | /* 0x3fd17460_a91d4101 */
// | const L5 = 2.30660745775561754067e-1;
pub const L5: f64 = 0.23066074577556175f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=b565c2eff5e7408f3fe1288349c8ef98295e2e8a47ccb017ec9c56e8543b5274
// | /* 0x3fcd864a_93c9db65 */
// | const L6 = 2.06975017800338417784e-1;
pub const L6: f64 = 0.20697501780033842f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=eac7abc939b1df61f2371bf875ace6c0a393ab8cb4201c8f0f2f476e14075fde
// | /* 0x3fca7e28_4a454eef */
// | const P1 = 1.66666666666666019037e-1;
pub const P1: f64 = 0.16666666666666602f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=b2fff89f5e53f45c480901765a1bd809c085ee24854bb6bba26770b82835902a
// | /* 0x3fc55555_5555553e */
// | const P2 = -2.77777777770155933842e-3;
pub const P2: f64 = (-0.0027777777777015593f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=79744faef559d2d0406a3a2f619bcd28bfb1b8021e917b384c34eefccfb140eb
// | /* 0xbf66c16c_16bebd93 */
// | const P3 = 6.61375632143793436117e-5;
pub const P3: f64 = 0.00006613756321437934f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=e4415439906c39f71b72ac0593a63fd5b0c502f29e572485ddbdfbfdaa4ea197
// | /* 0x3f11566a_af25de2c */
// | const P4 = -1.65339022054652515390e-6;
pub const P4: f64 = (-0.0000016533902205465252f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=d8c07de4f8770f5a3cfdb34365f8219d80f293dc68350b6876c1a2698e1af755
// | /* 0xbebbbd41_c5d26bf1 */
// | const P5 = 4.13813679705723846039e-8;
pub const P5: f64 = 4.1381367970572385e-8f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=9cc8b4078095904888b03b5486a42a33fb1db23f19b0699a8621158f380589f9
// | /* 0x3e663769_72bea4d0 */
// | const LG2 = 6.93147180559945286227e-1;
pub const LG2: f64 = 0.6931471805599453f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=181a8089981fb6929efde7c4c6113872c6265b90fb80f31e26ede25eaf03ec74
// | /* 0x3fe62e42_fefa39ef */
// | const LG2_H = 6.93147182464599609375e-1;
pub const LG2_H: f64 = 0.6931471824645996f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=4fc73747d82551885490d3045281fe9293c18954aeff4986c4a0f8fd5d374d51
// | /* 0x3fe62e43_00000000 */
// | const LG2_L = -1.90465429995776804525e-9;
pub const LG2_L: f64 = (-1.904654299957768e-9f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=8505ed661f70ac7a3452703d65f72e7ef766209a49de442ab54eab28dcc06c4c
// | /* 0xbe205c61_0ca86c39 */
// | const OVT = 8.008566259537294e-17;
pub const OVT: f64 = 8.008566259537294e-17f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=77f60513814a2a7d788416563c635e5d179064d55f30be57f8c037422bc9ba78
// | /* 8.0085662595372944372e-17 = -(1024-log2(ovfl+.5ulp)) */
// | const CP = 9.61796693925975554329e-1;
pub const CP: f64 = 0.9617966939259756f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=05f316a654990855ea9b83e80c9b1b72b3d7247fe26a79fdbecddde94b714b64
// | /* 0x3feec709_dc3a03fd =2/(3ln2) */
// | const CP_H = 9.61796700954437255859e-1;
pub const CP_H: f64 = 0.9617967009544373f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=0aeb9d115bb432059ed0f49009a1326bcd220a89dd292d7b0eafb359d68e35c3
// | /* 0x3feec709_e0000000 =(float)cp */
// | const CP_L = -7.02846165095275826516e-9;
pub const CP_L: f64 = (-7.028461650952758e-9f64);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=118d9c1b7150cc08e9bef86891f335847f521dd477f4dad9abfe001ad4f0793d
// | /* 0xbe3e2fe0_145b01f5 =tail of cp_h*/
// | const IVLN2 = 1.44269504088896338700e+0;
pub const IVLN2: f64 = 1.4426950408889634f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=4f2c3960eff1c8f8c204e2704fc46797b96ffd39e07999dde9f9bf1091eca46c
// | /* 0x3ff71547_652b82fe =1/ln2 */
// | const IVLN2_H = 1.44269502162933349609e+0;
pub const IVLN2_H: f64 = 1.4426950216293335f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=d391a64692727d1add60d95bbac54d86a376515d6804078961f407e4255d4cc8
// | /* 0x3ff71547_60000000 =24b 1/ln2*/
// | const IVLN2_L = 1.92596299112661746887e-8;
pub const IVLN2_L: f64 = 1.9259629911266175e-8f64;
/* 0x3e54ae0b_f85ddf44 =1/ln2 tail*/

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `libm::pow` (Rust `f64::powf`).
// |  * @param {number} x
// |  * @param {number} y
// |  * @returns {number}
// |  */
// | export function pow(x, y) {
// |   let t1;
// |   let t2;
// |
// |   const hx = getHighWord(x) | 0;
// |   const lx = getLowWord(x);
// |   const hy = getHighWord(y) | 0;
// |   const ly = getLowWord(y);
// |
// |   let ix = hx & 0x7fffffff;
// |   const iy = hy & 0x7fffffff;
// |
// |   /* x**0 = 1, even if x is NaN */
// |   if ((iy | ly) === 0) {
// |     return 1.0;
// |   }
// |
// |   /* 1**y = 1, even if y is NaN */
// |   if (hx === 0x3ff00000 && lx === 0) {
// |     return 1.0;
// |   }
// |
// |   /* NaN if either arg is NaN */
// |   if (
// |     ix > 0x7ff00000 ||
// |     (ix === 0x7ff00000 && lx !== 0) ||
// |     iy > 0x7ff00000 ||
// |     (iy === 0x7ff00000 && ly !== 0)
// |   ) {
// |     return x + y;
// |   }
// |
// |   /* determine if y is an odd int when x < 0
// |    * yisint = 0       ... y is not an integer
// |    * yisint = 1       ... y is an odd int
// |    * yisint = 2       ... y is an even int
// |    */
// |   let yisint = 0;
// |   let k;
// |   let j;
// |   if (hx < 0) {
// |     if (iy >= 0x43400000) {
// |       yisint = 2; /* even integer y */
// |     } else if (iy >= 0x3ff00000) {
// |       k = (iy >> 20) - 0x3ff; /* exponent */
// |
// |       if (k > 20) {
// |         j = (ly >>> (52 - k)) | 0;
// |
// |         if (j << (52 - k) === (ly | 0)) {
// |           yisint = 2 - (j & 1);
// |         }
// |       } else if (ly === 0) {
// |         j = iy >> (20 - k);
// |
// |         if (j << (20 - k) === iy) {
// |           yisint = 2 - (j & 1);
// |         }
// |       }
// |     }
// |   }
// |
// |   if (ly === 0) {
// |     /* special value of y */
// |     if (iy === 0x7ff00000) {
// |       /* y is +-inf */
// |       if (((ix - 0x3ff00000) | lx) === 0) {
// |         /* (-1)**+-inf is 1 */
// |         return 1.0;
// |       } else if (ix >= 0x3ff00000) {
// |         /* (|x|>1)**+-inf = inf,0 */
// |         return hy >= 0 ? y : 0.0;
// |       } else {
// |         /* (|x|<1)**+-inf = 0,inf */
// |         return hy >= 0 ? 0.0 : -y;
// |       }
// |     }
// |
// |     if (iy === 0x3ff00000) {
// |       /* y is +-1 */
// |       return hy >= 0 ? x : 1.0 / x;
// |     }
// |
// |     if (hy === 0x40000000) {
// |       /* y is 2 */
// |       return x * x;
// |     }
// |
// |     if (hy === 0x3fe00000) {
// |       /* y is 0.5 */
// |       if (hx >= 0) {
// |         /* x >= +0 */
// |         return Math.sqrt(x);
// |       }
// |     }
// |   }
// |
// |   let ax = Math.abs(x);
// |   if (lx === 0) {
// |     /* special value of x */
// |     if (ix === 0x7ff00000 || ix === 0 || ix === 0x3ff00000) {
// |       /* x is +-0,+-inf,+-1 */
// |       let z = ax;
// |
// |       if (hy < 0) {
// |         /* z = (1/|x|) */
// |         z = 1.0 / z;
// |       }
// |
// |       if (hx < 0) {
// |         if (((ix - 0x3ff00000) | yisint) === 0) {
// |           z = (z - z) / (z - z); /* (-1)**non-int is NaN */
// |         } else if (yisint === 1) {
// |           z = -z; /* (x<0)**odd = -(|x|**odd) */
// |         }
// |       }
// |
// |       return z;
// |     }
// |   }
// |
// |   let s = 1.0; /* sign of result */
// |   if (hx < 0) {
// |     if (yisint === 0) {
// |       /* (x<0)**(non-int) is NaN */
// |       return (x - x) / (x - x);
// |     }
// |
// |     if (yisint === 1) {
// |       /* (x<0)**(odd int) */
// |       s = -1.0;
// |     }
// |   }
// |
// |   /* |y| is HUGE */
// |   if (iy > 0x41e00000) {
// |     /* if |y| > 2**31 */
// |     if (iy > 0x43f00000) {
// |       /* if |y| > 2**64, must o/uflow */
// |       if (ix <= 0x3fefffff) {
// |         return hy < 0 ? HUGE * HUGE : TINY * TINY;
// |       }
// |
// |       if (ix >= 0x3ff00000) {
// |         return hy > 0 ? HUGE * HUGE : TINY * TINY;
// |       }
// |     }
// |
// |     /* over/underflow if x is not close to one */
// |     if (ix < 0x3fefffff) {
// |       return hy < 0 ? s * HUGE * HUGE : s * TINY * TINY;
// |     }
// |     if (ix > 0x3ff00000) {
// |       return hy > 0 ? s * HUGE * HUGE : s * TINY * TINY;
// |     }
// |
// |     /* now |1-x| is TINY <= 2**-20, suffice to compute
// |     log(x) by x-x^2/2+x^3/3-x^4/4 */
// |     const t = ax - 1.0; /* t has 20 trailing zeros */
// |     /* 0.3333333333333333 is the source literal 0.3333333333333333333333 */
// |     const w = (t * t) * (0.5 - t * (0.3333333333333333 - t * 0.25));
// |     const u = IVLN2_H * t; /* ivln2_h has 21 sig. bits */
// |     const v = t * IVLN2_L - w * IVLN2;
// |     t1 = withSetLowWord(u + v, 0);
// |     t2 = v - (t1 - u);
// |   } else {
// |     let n = 0;
// |
// |     if (ix < 0x00100000) {
// |       /* take care subnormal number */
// |       ax *= TWO53;
// |       n -= 53;
// |       ix = getHighWord(ax) | 0;
// |     }
// |
// |     n += (ix >> 20) - 0x3ff;
// |     j = ix & 0x000fffff;
// |
// |     /* determine interval */
// |     let k;
// |     ix = j | 0x3ff00000; /* normalize ix */
// |     if (j <= 0x3988e) {
// |       /* |x|<sqrt(3/2) */
// |       k = 0;
// |     } else if (j < 0xbb67a) {
// |       /* |x|<sqrt(3)   */
// |       k = 1;
// |     } else {
// |       k = 0;
// |       n += 1;
// |       ix -= 0x00100000;
// |     }
// |     ax = withSetHighWord(ax, ix);
// |
// |     /* compute ss = s_h+s_l = (x-1)/(x+1) or (x-1.5)/(x+1.5) */
// |     let u = ax - BP[k]; /* bp[0]=1.0, bp[1]=1.5 */
// |     let v = 1.0 / (ax + BP[k]);
// |     const ss = u * v;
// |     const sH = withSetLowWord(ss, 0);
// |
// |     /* t_h=ax+bp[k] High */
// |     let tH = combineWords(((ix >>> 1) | 0x20000000) + 0x00080000 + (k << 18), 0);
// |     let tL = ax - (tH - BP[k]);
// |     const sL = v * ((u - sH * tH) - sH * tL);
// |
// |     /* compute log(ax) */
// |     let s2 = ss * ss;
// |     let r = s2 * s2 * (L1 + s2 * (L2 + s2 * (L3 + s2 * (L4 + s2 * (L5 + s2 * L6)))));
// |     r += sL * (sH + ss);
// |     s2 = sH * sH;
// |     tH = withSetLowWord(3.0 + s2 + r, 0);
// |     tL = r - ((tH - 3.0) - s2);
// |
// |     /* u+v = ss*(1+...) */
// |     u = sH * tH;
// |     v = sL * tH + tL * ss;
// |
// |     /* 2/(3log2)*(ss+...) */
// |     const pH = withSetLowWord(u + v, 0);
// |     const pL = v - (pH - u);
// |     const zH = CP_H * pH; /* cp_h+cp_l = 2/(3*log2) */
// |     const zL = CP_L * pH + pL * CP + DP_L[k];
// |
// |     /* log2(ax) = (ss+..)*2/(3*log2) = n + dp_h + z_h + z_l */
// |     const t = n;
// |     t1 = withSetLowWord(((zH + zL) + DP_H[k]) + t, 0);
// |     t2 = zL - (((t1 - t) - DP_H[k]) - zH);
// |   }
// |
// |   /* split up y into y1+y2 and compute (y1+y2)*(t1+t2) */
// |   const y1 = withSetLowWord(y, 0);
// |   const pL = (y - y1) * t1 + y * t2;
// |   let pH = y1 * t1;
// |   let z = pL + pH;
// |   j = getHighWord(z) | 0;
// |   let i = getLowWord(z) | 0;
// |   if (j >= 0x40900000) {
// |     /* z >= 1024 */
// |     if (((j - 0x40900000) | i) !== 0) {
// |       /* if z > 1024 */
// |       return s * HUGE * HUGE; /* overflow */
// |     }
// |
// |     if (pL + OVT > z - pH) {
// |       return s * HUGE * HUGE; /* overflow */
// |     }
// |   } else if ((j & 0x7fffffff) >= 0x4090cc00) {
// |     /* z <= -1075 */
// |     if ((((j >>> 0) - 0xc090cc00) | i) !== 0) {
// |       /* z < -1075 */
// |       return s * TINY * TINY; /* underflow */
// |     }
// |
// |     if (pL <= z - pH) {
// |       return s * TINY * TINY; /* underflow */
// |     }
// |   }
// |
// |   /* compute 2**(p_h+p_l) */
// |   i = j & 0x7fffffff;
// |   k = (i >> 20) - 0x3ff;
// |   let n = 0;
// |
// |   if (i > 0x3fe00000) {
// |     /* if |z| > 0.5, set n = [z+0.5] */
// |     n = j + (0x00100000 >> (k + 1));
// |     k = ((n & 0x7fffffff) >> 20) - 0x3ff; /* new k for n */
// |     const t = combineWords(n & ~(0x000fffff >> k), 0);
// |     n = ((n & 0x000fffff) | 0x00100000) >> (20 - k);
// |     if (j < 0) {
// |       n = -n;
// |     }
// |     pH -= t;
// |   }
// |
// |   const t = withSetLowWord(pL + pH, 0);
// |   const u = t * LG2_H;
// |   const v = (pL - (t - pH)) * LG2 + t * LG2_L;
// |   z = u + v;
// |   const w = v - (z - u);
// |   const tt = z * z;
// |   const tt1 = z - tt * (P1 + tt * (P2 + tt * (P3 + tt * (P4 + tt * P5))));
// |   const r = (z * tt1) / (tt1 - 2.0) - (w + z * w);
// |   z = 1.0 - (r - z);
// |   j = getHighWord(z) | 0;
// |   j += n << 20;
// |
// |   if (j >> 20 <= 0) {
// |     /* subnormal output */
// |     z = scalbn(z, n);
// |   } else {
// |     z = withSetHighWord(z, j);
// |   }
// |
// |   return s * z;
// | }
