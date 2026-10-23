// sin, cos and tan: a port of libm 0.2.16 src/math/{k_sin,k_cos,k_tan,sin,
// cos,tan}.rs (FreeBSD k_sin.c, k_cos.c, k_tan.c, s_sin.c, s_cos.c, s_tan.c).

import { getHighWord, withSetLowWord } from './bits.js';
import { remPio2 } from './rem_pio2.js';

// ---------------------------------------------------------------------------
// k_sin.rs

const S1 = -1.66666666666666324348e-1; /* 0xBFC55555, 0x55555549 */
const S2 = 8.33333333332248946124e-3; /* 0x3F811111, 0x1110F8A6 */
const S3 = -1.98412698298579493134e-4; /* 0xBF2A01A0, 0x19C161D5 */
const S4 = 2.75573137070700676789e-6; /* 0x3EC71DE3, 0x57B1FE7D */
const S5 = -2.50507602534068634195e-8; /* 0xBE5AE5E6, 0x8A2B9CEB */
const S6 = 1.58969099521155010221e-10; /* 0x3DE5D93A, 0x5ACFD57C */

/**
 * Kernel sin on [-pi/4, pi/4]; `y` is the tail of x, `iy` says whether it is
 * used.
 * @param {number} x
 * @param {number} y
 * @param {number} iy
 * @returns {number}
 */
export function kSin(x, y, iy) {
  const z = x * x;
  const w = z * z;
  const r = S2 + z * (S3 + z * S4) + z * w * (S5 + z * S6);
  const v = z * x;
  if (iy === 0) {
    return x + v * (S1 + z * r);
  } else {
    return x - ((z * (0.5 * y - v * r) - y) - v * S1);
  }
}

// ---------------------------------------------------------------------------
// k_cos.rs

const C1 = 4.16666666666666019037e-2; /* 0x3FA55555, 0x5555554C */
const C2 = -1.38888888888741095749e-3; /* 0xBF56C16C, 0x16C15177 */
const C3 = 2.48015872894767294178e-5; /* 0x3EFA01A0, 0x19CB1590 */
const C4 = -2.75573143513906633035e-7; /* 0xBE927E4F, 0x809C52AD */
const C5 = 2.08757232129817482790e-9; /* 0x3E21EE9E, 0xBDB4B1C4 */
const C6 = -1.13596475577881948265e-11; /* 0xBDA8FAE9, 0xBE8838D4 */

/**
 * Kernel cos on [-pi/4, pi/4]; `y` is the tail of x.
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function kCos(x, y) {
  const z = x * x;
  let w = z * z;
  const r = z * (C1 + z * (C2 + z * C3)) + w * w * (C4 + z * (C5 + z * C6));
  const hz = 0.5 * z;
  w = 1.0 - hz;
  return w + (((1.0 - w) - hz) + (z * r - x * y));
}

// ---------------------------------------------------------------------------
// k_tan.rs

const T = [
  3.33333333333334091986e-1, /* 3FD55555, 55555563 */
  1.33333333333201242699e-1, /* 3FC11111, 1110FE7A */
  5.39682539762260521377e-2, /* 3FABA1BA, 1BB341FE */
  2.18694882948595424599e-2, /* 3F9664F4, 8406D637 */
  8.86323982359930005737e-3, /* 3F8226E3, E96E8493 */
  3.59207910759131235356e-3, /* 3F6D6D22, C9560328 */
  1.45620945432529025516e-3, /* 3F57DBC8, FEE08315 */
  5.88041240820264096874e-4, /* 3F4344D8, F2F26501 */
  2.46463134818469906812e-4, /* 3F3026F7, 1A8D1068 */
  7.81794442939557092300e-5, /* 3F147E88, A03792A6 */
  7.14072491382608190305e-5, /* 3F12B80F, 32F0A7E9 */
  -1.85586374855275456654e-5, /* BEF375CB, DB605373 */
  2.59073051863633712884e-5, /* 3EFB2A70, 74BF7AD4 */
];
const PIO4 = 7.85398163397448278999e-1; /* 3FE921FB, 54442D18 */
const PIO4_LO = 3.06161699786838301793e-17; /* 3C81A626, 33145C07 */

/**
 * @param {number} x
 * @returns {number}
 */
function zeroLowWord(x) {
  return withSetLowWord(x, 0);
}

/**
 * Kernel tan on [-pi/4, pi/4]; returns -1/tan when `odd` is 1.
 * @param {number} x
 * @param {number} y
 * @param {number} odd
 * @returns {number}
 */
export function kTan(x, y, odd) {
  const hx = getHighWord(x);
  const big = (hx & 0x7fffffff) >= 0x3fe59428; /* |x| >= 0.6744 */
  if (big) {
    const sign = hx >>> 31;
    if (sign !== 0) {
      x = -x;
      y = -y;
    }
    x = (PIO4 - x) + (PIO4_LO - y);
    y = 0.0;
  }
  const z = x * x;
  const w = z * z;
  /*
   * Break x^5*(T[1]+x^2*T[2]+...) into
   * x^5(T[1]+x^4*T[3]+...+x^20*T[11]) +
   * x^5(x^2*(T[2]+x^4*T[4]+...+x^22*[T12]))
   */
  let r = T[1] + w * (T[3] + w * (T[5] + w * (T[7] + w * (T[9] + w * T[11]))));
  let v = z * (T[2] + w * (T[4] + w * (T[6] + w * (T[8] + w * (T[10] + w * T[12])))));
  const s = z * x;
  r = y + z * (s * (r + v) + y) + s * T[0];
  const w2 = x + r;
  if (big) {
    const sign = hx >>> 31;
    const s = 1.0 - 2.0 * odd;
    const v = s - 2.0 * (x + (r - w2 * w2 / (w2 + s)));
    return sign !== 0 ? -v : v;
  }
  if (odd === 0) {
    return w2;
  }
  /* -1.0/(x+r) has up to 2ulp error, so compute it accurately */
  const w0 = zeroLowWord(w2);
  v = r - (w0 - x); /* w0+v = r+x */
  const a = -1.0 / w2;
  const a0 = zeroLowWord(a);
  return a0 + a * (1.0 + a0 * w0 + a0 * v);
}

// ---------------------------------------------------------------------------
// sin.rs, cos.rs, tan.rs

/**
 * `libm::sin`.
 * @param {number} x
 * @returns {number}
 */
export function sin(x) {
  /* High word of x. */
  const ix = getHighWord(x) & 0x7fffffff;

  /* |x| ~< pi/4 */
  if (ix <= 0x3fe921fb) {
    if (ix < 0x3e500000) {
      /* |x| < 2**-26 */
      return x;
    }
    return kSin(x, 0.0, 0);
  }

  /* sin(Inf or NaN) is NaN */
  if (ix >= 0x7ff00000) {
    return x - x;
  }

  /* argument reduction needed */
  const [n, y0, y1] = remPio2(x);
  switch (n & 3) {
    case 0:
      return kSin(y0, y1, 1);
    case 1:
      return kCos(y0, y1);
    case 2:
      return -kSin(y0, y1, 1);
    default:
      return -kCos(y0, y1);
  }
}

/**
 * `libm::cos`.
 * @param {number} x
 * @returns {number}
 */
export function cos(x) {
  const ix = getHighWord(x) & 0x7fffffff;

  /* |x| ~< pi/4 */
  if (ix <= 0x3fe921fb) {
    if (ix < 0x3e46a09e) {
      /* if x < 2**-27 * sqrt(2) */
      /* raise inexact if x != 0 */
      if ((x | 0) === 0) {
        return 1.0;
      }
    }
    return kCos(x, 0.0);
  }

  /* cos(Inf or NaN) is NaN */
  if (ix >= 0x7ff00000) {
    return x - x;
  }

  /* argument reduction needed */
  const [n, y0, y1] = remPio2(x);
  switch (n & 3) {
    case 0:
      return kCos(y0, y1);
    case 1:
      return -kSin(y0, y1, 1);
    case 2:
      return -kCos(y0, y1);
    default:
      return kSin(y0, y1, 1);
  }
}

/**
 * `libm::tan`.
 * @param {number} x
 * @returns {number}
 */
export function tan(x) {
  const ix = getHighWord(x) & 0x7fffffff;

  /* |x| ~< pi/4 */
  if (ix <= 0x3fe921fb) {
    if (ix < 0x3e400000) {
      /* |x| < 2**-27 */
      return x;
    }
    return kTan(x, 0.0, 0);
  }

  /* tan(Inf or NaN) is NaN */
  if (ix >= 0x7ff00000) {
    return x - x;
  }

  /* argument reduction */
  const [n, y0, y1] = remPio2(x);
  return kTan(y0, y1, n & 1);
}
