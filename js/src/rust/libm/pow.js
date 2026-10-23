// pow: a port of libm 0.2.16 src/math/pow.rs (FreeBSD e_pow.c).
//
// The high words are i32 in Rust (`(x.to_bits() >> 32) as i32`), so they are
// read with `| 0`; the low words stay u32 numbers.

import { combineWords, getHighWord, getLowWord, scalbn, withSetHighWord, withSetLowWord } from './bits.js';

const BP = [1.0, 1.5];
const DP_H = [0.0, 5.84962487220764160156e-1]; /* 0x3fe2b803_40000000 */
const DP_L = [0.0, 1.35003920212974897128e-8]; /* 0x3E4CFDEB, 0x43CFD006 */
const TWO53 = 9007199254740992.0; /* 0x43400000_00000000 */
const HUGE = 1.0e300;
const TINY = 1.0e-300;

// poly coefs for (3/2)*(log(x)-2s-2/3*s**3:
const L1 = 5.99999999999994648725e-1; /* 0x3fe33333_33333303 */
const L2 = 4.28571428578550184252e-1; /* 0x3fdb6db6_db6fabff */
const L3 = 3.33333329818377432918e-1; /* 0x3fd55555_518f264d */
const L4 = 2.72728123808534006489e-1; /* 0x3fd17460_a91d4101 */
const L5 = 2.30660745775561754067e-1; /* 0x3fcd864a_93c9db65 */
const L6 = 2.06975017800338417784e-1; /* 0x3fca7e28_4a454eef */
const P1 = 1.66666666666666019037e-1; /* 0x3fc55555_5555553e */
const P2 = -2.77777777770155933842e-3; /* 0xbf66c16c_16bebd93 */
const P3 = 6.61375632143793436117e-5; /* 0x3f11566a_af25de2c */
const P4 = -1.65339022054652515390e-6; /* 0xbebbbd41_c5d26bf1 */
const P5 = 4.13813679705723846039e-8; /* 0x3e663769_72bea4d0 */
const LG2 = 6.93147180559945286227e-1; /* 0x3fe62e42_fefa39ef */
const LG2_H = 6.93147182464599609375e-1; /* 0x3fe62e43_00000000 */
const LG2_L = -1.90465429995776804525e-9; /* 0xbe205c61_0ca86c39 */
const OVT = 8.008566259537294e-17; /* 8.0085662595372944372e-17 = -(1024-log2(ovfl+.5ulp)) */
const CP = 9.61796693925975554329e-1; /* 0x3feec709_dc3a03fd =2/(3ln2) */
const CP_H = 9.61796700954437255859e-1; /* 0x3feec709_e0000000 =(float)cp */
const CP_L = -7.02846165095275826516e-9; /* 0xbe3e2fe0_145b01f5 =tail of cp_h*/
const IVLN2 = 1.44269504088896338700e+0; /* 0x3ff71547_652b82fe =1/ln2 */
const IVLN2_H = 1.44269502162933349609e+0; /* 0x3ff71547_60000000 =24b 1/ln2*/
const IVLN2_L = 1.92596299112661746887e-8; /* 0x3e54ae0b_f85ddf44 =1/ln2 tail*/

/**
 * `libm::pow` (Rust `f64::powf`).
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function pow(x, y) {
  let t1;
  let t2;

  const hx = getHighWord(x) | 0;
  const lx = getLowWord(x);
  const hy = getHighWord(y) | 0;
  const ly = getLowWord(y);

  let ix = hx & 0x7fffffff;
  const iy = hy & 0x7fffffff;

  /* x**0 = 1, even if x is NaN */
  if ((iy | ly) === 0) {
    return 1.0;
  }

  /* 1**y = 1, even if y is NaN */
  if (hx === 0x3ff00000 && lx === 0) {
    return 1.0;
  }

  /* NaN if either arg is NaN */
  if (
    ix > 0x7ff00000 ||
    (ix === 0x7ff00000 && lx !== 0) ||
    iy > 0x7ff00000 ||
    (iy === 0x7ff00000 && ly !== 0)
  ) {
    return x + y;
  }

  /* determine if y is an odd int when x < 0
   * yisint = 0       ... y is not an integer
   * yisint = 1       ... y is an odd int
   * yisint = 2       ... y is an even int
   */
  let yisint = 0;
  let k;
  let j;
  if (hx < 0) {
    if (iy >= 0x43400000) {
      yisint = 2; /* even integer y */
    } else if (iy >= 0x3ff00000) {
      k = (iy >> 20) - 0x3ff; /* exponent */

      if (k > 20) {
        j = (ly >>> (52 - k)) | 0;

        if (j << (52 - k) === (ly | 0)) {
          yisint = 2 - (j & 1);
        }
      } else if (ly === 0) {
        j = iy >> (20 - k);

        if (j << (20 - k) === iy) {
          yisint = 2 - (j & 1);
        }
      }
    }
  }

  if (ly === 0) {
    /* special value of y */
    if (iy === 0x7ff00000) {
      /* y is +-inf */
      if (((ix - 0x3ff00000) | lx) === 0) {
        /* (-1)**+-inf is 1 */
        return 1.0;
      } else if (ix >= 0x3ff00000) {
        /* (|x|>1)**+-inf = inf,0 */
        return hy >= 0 ? y : 0.0;
      } else {
        /* (|x|<1)**+-inf = 0,inf */
        return hy >= 0 ? 0.0 : -y;
      }
    }

    if (iy === 0x3ff00000) {
      /* y is +-1 */
      return hy >= 0 ? x : 1.0 / x;
    }

    if (hy === 0x40000000) {
      /* y is 2 */
      return x * x;
    }

    if (hy === 0x3fe00000) {
      /* y is 0.5 */
      if (hx >= 0) {
        /* x >= +0 */
        return Math.sqrt(x);
      }
    }
  }

  let ax = Math.abs(x);
  if (lx === 0) {
    /* special value of x */
    if (ix === 0x7ff00000 || ix === 0 || ix === 0x3ff00000) {
      /* x is +-0,+-inf,+-1 */
      let z = ax;

      if (hy < 0) {
        /* z = (1/|x|) */
        z = 1.0 / z;
      }

      if (hx < 0) {
        if (((ix - 0x3ff00000) | yisint) === 0) {
          z = (z - z) / (z - z); /* (-1)**non-int is NaN */
        } else if (yisint === 1) {
          z = -z; /* (x<0)**odd = -(|x|**odd) */
        }
      }

      return z;
    }
  }

  let s = 1.0; /* sign of result */
  if (hx < 0) {
    if (yisint === 0) {
      /* (x<0)**(non-int) is NaN */
      return (x - x) / (x - x);
    }

    if (yisint === 1) {
      /* (x<0)**(odd int) */
      s = -1.0;
    }
  }

  /* |y| is HUGE */
  if (iy > 0x41e00000) {
    /* if |y| > 2**31 */
    if (iy > 0x43f00000) {
      /* if |y| > 2**64, must o/uflow */
      if (ix <= 0x3fefffff) {
        return hy < 0 ? HUGE * HUGE : TINY * TINY;
      }

      if (ix >= 0x3ff00000) {
        return hy > 0 ? HUGE * HUGE : TINY * TINY;
      }
    }

    /* over/underflow if x is not close to one */
    if (ix < 0x3fefffff) {
      return hy < 0 ? s * HUGE * HUGE : s * TINY * TINY;
    }
    if (ix > 0x3ff00000) {
      return hy > 0 ? s * HUGE * HUGE : s * TINY * TINY;
    }

    /* now |1-x| is TINY <= 2**-20, suffice to compute
    log(x) by x-x^2/2+x^3/3-x^4/4 */
    const t = ax - 1.0; /* t has 20 trailing zeros */
    /* 0.3333333333333333 is the source literal 0.3333333333333333333333 */
    const w = (t * t) * (0.5 - t * (0.3333333333333333 - t * 0.25));
    const u = IVLN2_H * t; /* ivln2_h has 21 sig. bits */
    const v = t * IVLN2_L - w * IVLN2;
    t1 = withSetLowWord(u + v, 0);
    t2 = v - (t1 - u);
  } else {
    let n = 0;

    if (ix < 0x00100000) {
      /* take care subnormal number */
      ax *= TWO53;
      n -= 53;
      ix = getHighWord(ax) | 0;
    }

    n += (ix >> 20) - 0x3ff;
    j = ix & 0x000fffff;

    /* determine interval */
    let k;
    ix = j | 0x3ff00000; /* normalize ix */
    if (j <= 0x3988e) {
      /* |x|<sqrt(3/2) */
      k = 0;
    } else if (j < 0xbb67a) {
      /* |x|<sqrt(3)   */
      k = 1;
    } else {
      k = 0;
      n += 1;
      ix -= 0x00100000;
    }
    ax = withSetHighWord(ax, ix);

    /* compute ss = s_h+s_l = (x-1)/(x+1) or (x-1.5)/(x+1.5) */
    let u = ax - BP[k]; /* bp[0]=1.0, bp[1]=1.5 */
    let v = 1.0 / (ax + BP[k]);
    const ss = u * v;
    const sH = withSetLowWord(ss, 0);

    /* t_h=ax+bp[k] High */
    let tH = combineWords(((ix >>> 1) | 0x20000000) + 0x00080000 + (k << 18), 0);
    let tL = ax - (tH - BP[k]);
    const sL = v * ((u - sH * tH) - sH * tL);

    /* compute log(ax) */
    let s2 = ss * ss;
    let r = s2 * s2 * (L1 + s2 * (L2 + s2 * (L3 + s2 * (L4 + s2 * (L5 + s2 * L6)))));
    r += sL * (sH + ss);
    s2 = sH * sH;
    tH = withSetLowWord(3.0 + s2 + r, 0);
    tL = r - ((tH - 3.0) - s2);

    /* u+v = ss*(1+...) */
    u = sH * tH;
    v = sL * tH + tL * ss;

    /* 2/(3log2)*(ss+...) */
    const pH = withSetLowWord(u + v, 0);
    const pL = v - (pH - u);
    const zH = CP_H * pH; /* cp_h+cp_l = 2/(3*log2) */
    const zL = CP_L * pH + pL * CP + DP_L[k];

    /* log2(ax) = (ss+..)*2/(3*log2) = n + dp_h + z_h + z_l */
    const t = n;
    t1 = withSetLowWord(((zH + zL) + DP_H[k]) + t, 0);
    t2 = zL - (((t1 - t) - DP_H[k]) - zH);
  }

  /* split up y into y1+y2 and compute (y1+y2)*(t1+t2) */
  const y1 = withSetLowWord(y, 0);
  const pL = (y - y1) * t1 + y * t2;
  let pH = y1 * t1;
  let z = pL + pH;
  j = getHighWord(z) | 0;
  let i = getLowWord(z) | 0;
  if (j >= 0x40900000) {
    /* z >= 1024 */
    if (((j - 0x40900000) | i) !== 0) {
      /* if z > 1024 */
      return s * HUGE * HUGE; /* overflow */
    }

    if (pL + OVT > z - pH) {
      return s * HUGE * HUGE; /* overflow */
    }
  } else if ((j & 0x7fffffff) >= 0x4090cc00) {
    /* z <= -1075 */
    if ((((j >>> 0) - 0xc090cc00) | i) !== 0) {
      /* z < -1075 */
      return s * TINY * TINY; /* underflow */
    }

    if (pL <= z - pH) {
      return s * TINY * TINY; /* underflow */
    }
  }

  /* compute 2**(p_h+p_l) */
  i = j & 0x7fffffff;
  k = (i >> 20) - 0x3ff;
  let n = 0;

  if (i > 0x3fe00000) {
    /* if |z| > 0.5, set n = [z+0.5] */
    n = j + (0x00100000 >> (k + 1));
    k = ((n & 0x7fffffff) >> 20) - 0x3ff; /* new k for n */
    const t = combineWords(n & ~(0x000fffff >> k), 0);
    n = ((n & 0x000fffff) | 0x00100000) >> (20 - k);
    if (j < 0) {
      n = -n;
    }
    pH -= t;
  }

  const t = withSetLowWord(pL + pH, 0);
  const u = t * LG2_H;
  const v = (pL - (t - pH)) * LG2 + t * LG2_L;
  z = u + v;
  const w = v - (z - u);
  const tt = z * z;
  const tt1 = z - tt * (P1 + tt * (P2 + tt * (P3 + tt * (P4 + tt * P5))));
  const r = (z * tt1) / (tt1 - 2.0) - (w + z * w);
  z = 1.0 - (r - z);
  j = getHighWord(z) | 0;
  j += n << 20;

  if (j >> 20 <= 0) {
    /* subnormal output */
    z = scalbn(z, n);
  } else {
    z = withSetHighWord(z, j);
  }

  return s * z;
}
