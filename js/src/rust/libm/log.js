// Logarithms: a port of libm 0.2.16 src/math/{log,log10,log2}.rs (FreeBSD
// e_log.c, e_log10.c, e_log2.c).

import { combineWords, fromBits, getHighWord, getLowWord, withSetLowWord } from './bits.js';

const LN2_HI = 6.93147180369123816490e-1; /* 3fe62e42 fee00000 */
const LN2_LO = 1.90821492927058770002e-10; /* 3dea39ef 35793c76 */
const IVLN10HI = 4.34294481878168880939e-1; /* 0x3fdbcb7b, 0x15200000 */
const IVLN10LO = 2.50829467116452752298e-11; /* 0x3dbb9438, 0xca9aadd5 */
const LOG10_2HI = 3.01029995663611771306e-1; /* 0x3FD34413, 0x509F6000 */
const LOG10_2LO = 3.69423907715893078616e-13; /* 0x3D59FEF3, 0x11F12B36 */
const IVLN2HI = 1.44269504072144627571e+0; /* 0x3ff71547, 0x65200000 */
const IVLN2LO = 1.67517131648865118353e-10; /* 0x3de705fc, 0x2eefa200 */
const LG1 = 6.666666666666735130e-1; /* 3FE55555 55555593 */
const LG2 = 3.999999999940941908e-1; /* 3FD99999 9997FA04 */
const LG3 = 2.857142874366239149e-1; /* 3FD24924 94229359 */
const LG4 = 2.222219843214978396e-1; /* 3FCC71C5 1D8E78AF */
const LG5 = 1.818357216161805012e-1; /* 3FC74664 96CB03DE */
const LG6 = 1.531383769920937332e-1; /* 3FC39A09 D078C69F */
const LG7 = 1.479819860511658591e-1; /* 3FC2F112 DF3E5244 */

const X1P54 = fromBits(0x4350000000000000n); // 2^54

/**
 * The argument handling shared by log, log10 and log2: either a final result
 * (`done` with `value`), or `x` reduced into [sqrt(2)/2, sqrt(2)] with its
 * exponent `k`.
 * @typedef {{ done: boolean, value: number, x: number, k: number }} Reduced
 */

/**
 * @param {number} x
 * @returns {Reduced}
 */
function reduce(x) {
  let hx = getHighWord(x);
  let lx = getLowWord(x);
  let k = 0;

  if (hx < 0x00100000 || hx >>> 31 !== 0) {
    /* x < 2**-126  */
    if (((hx & 0x7fffffff) | lx) === 0) {
      return { done: true, value: -1 / (x * x), x: 0, k: 0 }; /* log(+-0)=-inf */
    }
    if (hx >>> 31 !== 0) {
      return { done: true, value: (x - x) / 0.0, x: 0, k: 0 }; /* log(-#) = NaN */
    }
    /* subnormal number, scale x up */
    k -= 54;
    x *= X1P54;
    hx = getHighWord(x);
    lx = getLowWord(x);
  } else if (hx >= 0x7ff00000) {
    return { done: true, value: x, x: 0, k: 0 };
  } else if (hx === 0x3ff00000 && lx === 0) {
    return { done: true, value: 0, x: 0, k: 0 };
  }

  /* reduce x into [sqrt(2)/2, sqrt(2)] */
  hx += 0x3ff00000 - 0x3fe6a09e;
  k += (hx >>> 20) - 0x3ff;
  hx = (hx & 0x000fffff) + 0x3fe6a09e;
  x = combineWords(hx, lx);
  return { done: false, value: 0, x, k };
}

/**
 * `libm::log` (Rust `f64::ln`).
 * @param {number} x
 * @returns {number}
 */
export function log(x) {
  const red = reduce(x);
  if (red.done) {
    return red.value;
  }
  const k = red.k;
  x = red.x;
  const f = x - 1.0;
  const hfsq = 0.5 * f * f;
  const s = f / (2.0 + f);
  const z = s * s;
  const w = z * z;
  const t1 = w * (LG2 + w * (LG4 + w * LG6));
  const t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
  const r = t2 + t1;
  const dk = k;
  return s * (hfsq + r) + dk * LN2_LO - hfsq + f + dk * LN2_HI;
}

/**
 * `libm::log10`.
 * @param {number} x
 * @returns {number}
 */
export function log10(x) {
  const red = reduce(x);
  if (red.done) {
    return red.value;
  }
  const k = red.k;
  x = red.x;
  const f = x - 1.0;
  const hfsq = 0.5 * f * f;
  const s = f / (2.0 + f);
  const z = s * s;
  let w = z * z;
  const t1 = w * (LG2 + w * (LG4 + w * LG6));
  const t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
  const r = t2 + t1;

  /* See log2.c for details. */
  /* hi+lo = f - hfsq + s*(hfsq+R) ~ log(1+f) */
  const hi = withSetLowWord(f - hfsq, 0);
  const lo = f - hi - hfsq + s * (hfsq + r);

  /* val_hi+val_lo ~ log10(1+f) + k*log10(2) */
  let valHi = hi * IVLN10HI;
  const dk = k;
  const y = dk * LOG10_2HI;
  let valLo = dk * LOG10_2LO + (lo + hi) * IVLN10LO + lo * IVLN10HI;

  /*
   * Extra precision in for adding y is not strictly needed
   * since there is no very large cancellation near x = sqrt(2) or
   * x = 1/sqrt(2), but we do it anyway since it costs little on CPUs
   * with some parallelism and it reduces the error for many args.
   */
  w = y + valHi;
  valLo += (y - w) + valHi;
  valHi = w;

  return valLo + valHi;
}

/**
 * `libm::log2`.
 * @param {number} x
 * @returns {number}
 */
export function log2(x) {
  const red = reduce(x);
  if (red.done) {
    return red.value;
  }
  const k = red.k;
  x = red.x;
  const f = x - 1.0;
  const hfsq = 0.5 * f * f;
  const s = f / (2.0 + f);
  const z = s * s;
  let w = z * z;
  const t1 = w * (LG2 + w * (LG4 + w * LG6));
  const t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
  const r = t2 + t1;

  /* hi+lo = f - hfsq + s*(hfsq+R) ~ log(1+f) */
  const hi = withSetLowWord(f - hfsq, 0);
  const lo = f - hi - hfsq + s * (hfsq + r);

  let valHi = hi * IVLN2HI;
  let valLo = (lo + hi) * IVLN2LO + lo * IVLN2HI;

  /* spadd(val_hi, val_lo, y), except for not using double_t: */
  const y = k;
  w = y + valHi;
  valLo += (y - w) + valHi;
  valHi = w;

  return valLo + valHi;
}
