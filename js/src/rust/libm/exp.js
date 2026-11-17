// exp, expm1 and the hyperbolic functions: a port of libm 0.2.16
// src/math/{exp,expm1,expo2,k_expo2,sinh,cosh,tanh}.rs (FreeBSD e_exp.c,
// s_expm1.c, k_exp.c, e_sinh.c, e_cosh.c, s_tanh.c).

import { combineWords, fromBits, getHighWord, scalbn, withSetHighWord } from './bits.js';

// ---------------------------------------------------------------------------
// exp.rs

const HALF = [0.5, -0.5];
const LN2HI = 6.93147180369123816490e-1; /* 0x3fe62e42, 0xfee00000 */
const LN2LO = 1.90821492927058770002e-10; /* 0x3dea39ef, 0x35793c76 */
const INVLN2 = 1.44269504088896338700e+0; /* 0x3ff71547, 0x652b82fe */
const P1 = 1.66666666666666019037e-1; /* 0x3FC55555, 0x5555553E */
const P2 = -2.77777777770155933842e-3; /* 0xBF66C16C, 0x16BEBD93 */
const P3 = 6.61375632143793436117e-5; /* 0x3F11566A, 0xAF25DE2C */
const P4 = -1.65339022054652515390e-6; /* 0xBEBBBD41, 0xC5D26BF1 */
const P5 = 4.13813679705723846039e-8; /* 0x3E663769, 0x72BEA4D0 */

const X1P1023 = fromBits(0x7fe0000000000000n); // 2^1023

/**
 * `libm::exp`.
 * @param {number} x
 * @returns {number}
 */
export function exp(x) {
  let hi;
  let lo;
  let k;

  let hx = getHighWord(x);
  const sign = hx >>> 31;
  hx &= 0x7fffffff; /* high word of |x| */

  /* special cases */
  if (hx >= 0x4086232b) {
    /* if |x| >= 708.39... */
    if (Number.isNaN(x)) {
      return x;
    }
    if (x > 709.782712893383973096) {
      /* overflow if x!=inf */
      x *= X1P1023;
      return x;
    }
    if (x < -708.3964185322641) {
      /* -708.39641853226410622 in the source */
      /* underflow if x!=-inf */
      if (x < -745.13321910194110842) {
        return 0;
      }
    }
  }

  /* argument reduction */
  if (hx > 0x3fd62e42) {
    /* if |x| > 0.5 ln2 */
    if (hx >= 0x3ff0a2b2) {
      /* if |x| >= 1.5 ln2 */
      k = (INVLN2 * x + HALF[sign]) | 0;
    } else {
      k = 1 - sign - sign;
    }
    hi = x - k * LN2HI; /* k*ln2hi is exact here */
    lo = k * LN2LO;
    x = hi - lo;
  } else if (hx > 0x3e300000) {
    /* if |x| > 2**-28 */
    k = 0;
    hi = x;
    lo = 0;
  } else {
    /* inexact if x!=0 */
    return 1 + x;
  }

  /* x is now in primary range */
  const xx = x * x;
  const c = x - xx * (P1 + xx * (P2 + xx * (P3 + xx * (P4 + xx * P5))));
  const y = 1 + (x * c / (2 - c) - lo + hi);
  return k === 0 ? y : scalbn(y, k);
}

// ---------------------------------------------------------------------------
// expm1.rs

const O_THRESHOLD = 7.09782712893383973096e+2; /* 0x40862E42, 0xFEFA39EF */
const LN2_HI = 6.93147180369123816490e-1; /* 0x3fe62e42, 0xfee00000 */
const LN2_LO = 1.90821492927058770002e-10; /* 0x3dea39ef, 0x35793c76 */
/* Scaled Q's: Qn_here = 2**n * Qn_above, for R(2*z) where z = hxs = x*x/2: */
const Q1 = -3.33333333333331316428e-2; /* BFA11111 111110F4 */
const Q2 = 1.58730158725481460165e-3; /* 3F5A01A0 19FE5585 */
const Q3 = -7.93650757867487942473e-5; /* BF14CE19 9EAADBB7 */
const Q4 = 4.00821782732936239552e-6; /* 3ED0CFCA 86E65239 */
const Q5 = -2.01099218183624371326e-7; /* BE8AFDB7 6E09C32D */

/**
 * `libm::expm1`: e^x - 1.
 * @param {number} x
 * @returns {number}
 */
export function expm1(x) {
  let hi;
  let lo;
  let k;
  let c;
  let t;
  let y;

  const hxw = getHighWord(x);
  const hx = hxw & 0x7fffffff;
  const sign = hxw >>> 31;

  /* filter out huge and non-finite argument */
  if (hx >= 0x4043687a) {
    /* if |x|>=56*ln2 */
    if (Number.isNaN(x)) {
      return x;
    }
    if (sign !== 0) {
      return -1.0;
    }
    if (x > O_THRESHOLD) {
      x *= X1P1023;
      return x;
    }
  }

  /* argument reduction */
  if (hx > 0x3fd62e42) {
    /* if  |x| > 0.5 ln2 */
    if (hx < 0x3ff0a2b2) {
      /* and |x| < 1.5 ln2 */
      if (sign === 0) {
        hi = x - LN2_HI;
        lo = LN2_LO;
        k = 1;
      } else {
        hi = x + LN2_HI;
        lo = -LN2_LO;
        k = -1;
      }
    } else {
      k = (INVLN2 * x + (sign !== 0 ? -0.5 : 0.5)) | 0;
      t = k;
      hi = x - t * LN2_HI; /* t*ln2_hi is exact here */
      lo = t * LN2_LO;
    }
    x = hi - lo;
    c = (hi - x) - lo;
  } else if (hx < 0x3c900000) {
    /* |x| < 2**-54, return x */
    return x;
  } else {
    c = 0.0;
    k = 0;
  }

  /* x is now in primary range */
  const hfx = 0.5 * x;
  const hxs = x * hfx;
  const r1 = 1.0 + hxs * (Q1 + hxs * (Q2 + hxs * (Q3 + hxs * (Q4 + hxs * Q5))));
  t = 3.0 - r1 * hfx;
  let e = hxs * ((r1 - t) / (6.0 - x * t));
  if (k === 0) {
    /* c is 0 */
    return x - (x * e - hxs);
  }
  e = x * (e - c) - c;
  e -= hxs;
  /* exp(x) ~ 2^k (x_reduced - e + 1) */
  if (k === -1) {
    return 0.5 * (x - e) - 0.5;
  }
  if (k === 1) {
    if (x < -0.25) {
      return -2.0 * (e - (x + 0.5));
    }
    return 1.0 + 2.0 * (x - e);
  }
  const twopk = combineWords((0x3ff + k) << 20, 0); /* 2^k */
  if (!(k >= 0 && k <= 56)) {
    /* suffice to return exp(x)-1 */
    y = x - e + 1.0;
    if (k === 1024) {
      y = y * 2.0 * X1P1023;
    } else {
      y = y * twopk;
    }
    return y - 1.0;
  }
  const uf = combineWords((0x3ff - k) << 20, 0); /* 2^-k */
  if (k < 20) {
    y = (x - e + (1.0 - uf)) * twopk;
  } else {
    y = (x - (e + uf) + 1.0) * twopk;
  }
  return y;
}

// ---------------------------------------------------------------------------
// k_expo2.rs (expo2.rs is the same function)

/* k is such that k*ln2 has minimal relative error and x - kln2 > log(DBL_MIN) */
const K = 2043;
const K_LN2 = fromBits(0x40962066151add8bn);
/* note that k is odd and scale*scale overflows */
const EXPO2_SCALE = combineWords((0x3ff + Math.trunc(K / 2)) << 20, 0);

/**
 * exp(x)/2 for x >= log(DBL_MAX).
 * @param {number} x
 * @returns {number}
 */
function kExpo2(x) {
  /* exp(x - k ln2) * 2**(k-1) */
  return exp(x - K_LN2) * EXPO2_SCALE * EXPO2_SCALE;
}

// ---------------------------------------------------------------------------
// sinh.rs, cosh.rs, tanh.rs

/**
 * |x| by clearing the sign bit (`ui &= !1 / 2`).
 * @param {number} x
 * @returns {number}
 */
function clearSign(x) {
  return withSetHighWord(x, getHighWord(x) & 0x7fffffff);
}

/**
 * `libm::sinh`.
 * @param {number} x
 * @returns {number}
 */
export function sinh(x) {
  let h = 0.5;
  if (getHighWord(x) >>> 31 !== 0) {
    h = -h;
  }
  /* |x| */
  const absx = clearSign(x);
  const w = getHighWord(absx);

  /* |x| < log(DBL_MAX) */
  if (w < 0x40862e42) {
    const t = expm1(absx);
    if (w < 0x3ff00000) {
      if (w < 0x3ff00000 - (26 << 20)) {
        /* note: this branch avoids spurious underflow */
        return x;
      }
      return h * (2.0 * t - t * t / (t + 1.0));
    }
    /* note: |x|>log(0x1p26)+eps could be just h*exp(x) */
    return h * (t + t / (t + 1.0));
  }

  /* |x| > log(DBL_MAX) or nan */
  /* note: the result is stored to handle overflow */
  return 2.0 * h * kExpo2(absx);
}

/**
 * `libm::cosh`.
 * @param {number} x
 * @returns {number}
 */
export function cosh(x) {
  /* |x| */
  x = clearSign(x);
  const w = getHighWord(x);

  /* |x| < log(2) */
  if (w < 0x3fe62e42) {
    if (w < 0x3ff00000 - (26 << 20)) {
      return 1;
    }
    const t = expm1(x); // exponential minus 1
    return 1 + t * t / (2 * (1 + t));
  }

  /* |x| < log(DBL_MAX) */
  if (w < 0x40862e42) {
    const t = exp(x);
    /* note: if x>log(0x1p26) then the 1/t is not needed */
    return 0.5 * (t + 1 / t);
  }

  /* |x| > log(DBL_MAX) or nan */
  return kExpo2(x);
}

/**
 * `libm::tanh`.
 * @param {number} x
 * @returns {number}
 */
export function tanh(x) {
  let t;

  /* x = |x| */
  const sign = getHighWord(x) >>> 31 !== 0;
  x = clearSign(x);
  const w = getHighWord(x);

  if (w > 0x3fe193ea) {
    /* |x| > log(3)/2 ~= 0.5493 or nan */
    if (w > 0x40340000) {
      /* |x| > 20 or nan */
      /* note: this branch avoids raising overflow */
      t = 1.0 - 0.0 / x;
    } else {
      t = expm1(2.0 * x);
      t = 1.0 - 2.0 / (t + 2.0);
    }
  } else if (w > 0x3fd058ae) {
    /* |x| > log(5/3)/2 ~= 0.2554 */
    t = expm1(2.0 * x);
    t = t / (t + 2.0);
  } else if (w >= 0x00100000) {
    /* |x| >= 0x1p-1022, up to 2ulp error in [0.1,0.2554] */
    t = expm1(-2.0 * x);
    t = -t / (t + 2.0);
  } else {
    /* |x| is subnormal */
    t = x;
  }
  return sign ? -t : t;
}
