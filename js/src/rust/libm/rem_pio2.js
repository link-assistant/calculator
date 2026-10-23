// Argument reduction for the trigonometric functions: a port of libm 0.2.16
// src/math/rem_pio2.rs and src/math/rem_pio2_large.rs (FreeBSD e_rem_pio2.c
// and k_rem_pio2.c).

import { combineWords, getHighWord, getLowWord, scalbn } from './bits.js';

// ---------------------------------------------------------------------------
// rem_pio2_large.rs

/** Initial value for jk, indexed by `prec`. */
const INIT_JK = [3, 4, 4, 6];

/**
 * 2/pi in 24-bit chunks. libm keeps 690 entries on 64-bit targets and these
 * first 66 on 32-bit ones (wasm32); f64 arguments never need more than 66
 * (e0 <= 1000, so jv <= 41 and jv + jz stays well below 66).
 */
const IPIO2 = [
  0xa2f983, 0x6e4e44, 0x1529fc, 0x2757d1, 0xf534dd, 0xc0db62, 0x95993c, 0x439041, 0xfe5163,
  0xabdebb, 0xc561b7, 0x246e3a, 0x424dd2, 0xe00649, 0x2eea09, 0xd1921c, 0xfe1deb, 0x1cb129,
  0xa73ee8, 0x8235f5, 0x2ebb44, 0x84e99c, 0x7026b4, 0x5f7e41, 0x3991d6, 0x398353, 0x39f49c,
  0x845f8b, 0xbdf928, 0x3b1ff8, 0x97ffde, 0x05980f, 0xef2f11, 0x8b5a0a, 0x6d1f6d, 0x367ecf,
  0x27cb09, 0xb74f46, 0x3f669e, 0x5fea2d, 0x7527ba, 0xc7ebe5, 0xf17b3d, 0x0739f7, 0x8a5292,
  0xea6bfb, 0x5fb11f, 0x8d5d08, 0x560330, 0x46fc7b, 0x6babf0, 0xcfbc20, 0x9af436, 0x1da9e3,
  0x91615e, 0xe61b08, 0x659985, 0x5f14a0, 0x68408d, 0xffd880, 0x4d7327, 0x310606, 0x1556ca,
  0x73a8c9, 0x60e27b, 0xc08c6b,
];

const PIO2 = [
  1.57079625129699707031e+0, /* 0x3FF921FB, 0x40000000 */
  7.54978941586159635335e-8, /* 0x3E74442D, 0x00000000 */
  5.39030252995776476554e-15, /* 0x3CF84698, 0x80000000 */
  3.28200341580791294123e-22, /* 0x3B78CC51, 0x60000000 */
  1.27065575308067607349e-29, /* 0x39F01B83, 0x80000000 */
  1.22933308981111328932e-36, /* 0x387A2520, 0x40000000 */
  2.73370053816464559624e-44, /* 0x36E38222, 0x80000000 */
  2.16741683877804819444e-51, /* 0x3569F31D, 0x00000000 */
];

const X1P24 = combineWords(0x41700000, 0); // 2^24
const X1P_24 = combineWords(0x3e700000, 0); // 2^-24

/**
 * Return the last three digits of N with y = x - N*pi/2 so that |y| < pi/2.
 *
 * `x` holds the positive input in 24-bit pieces with `x[0] * 2^e0` matching
 * its top 24 bits; the result goes into `y` (1, 2, 2 or 3 entries for
 * `prec` 0, 1, 2 or 3).
 * @param {number[]} x
 * @param {number[]} y
 * @param {number} e0
 * @param {number} prec
 * @returns {number}
 */
export function remPio2Large(x, y, e0, prec) {
  const nx = x.length;
  let fw;
  let n;
  let ih;
  let z;
  const f = new Float64Array(20);
  const fq = new Float64Array(20);
  const q = new Float64Array(20);
  const iq = new Int32Array(20);

  /* initialize jk*/
  const jk = INIT_JK[prec];
  const jp = jk;

  /* determine jx,jv,q0, note that 3>q0 */
  const jx = nx - 1;
  let jv = Math.trunc((e0 - 3) / 24);
  if (jv < 0) {
    jv = 0;
  }
  let q0 = e0 - 24 * (jv + 1);

  /* set up f[0] to f[jx+jk] where f[jx+jk] = ipio2[jv+jk] */
  let j = jv - jx;
  const m = jx + jk;
  for (let i = 0; i <= m; i++) {
    f[i] = j < 0 ? 0 : IPIO2[j];
    j += 1;
  }

  /* compute q[0],q[1],...q[jk] */
  for (let i = 0; i <= jk; i++) {
    fw = 0;
    for (let j = 0; j <= jx; j++) {
      fw += x[j] * f[jx + i - j];
    }
    q[i] = fw;
  }

  let jz = jk;

  recompute: while (true) {
    /* distill q[] into iq[] reversingly */
    let i = 0;
    z = q[jz];
    for (let j = jz; j >= 1; j--) {
      fw = (X1P_24 * z) | 0;
      iq[i] = (z - X1P24 * fw) | 0;
      z = q[j - 1] + fw;
      i += 1;
    }

    /* compute n */
    z = scalbn(z, q0); /* actual value of z */
    z -= 8.0 * Math.floor(z * 0.125); /* trim off integer >= 8 */
    n = z | 0;
    z -= n;
    ih = 0;
    if (q0 > 0) {
      /* need iq[jz-1] to determine n */
      i = iq[jz - 1] >> (24 - q0);
      n += i;
      iq[jz - 1] -= i << (24 - q0);
      ih = iq[jz - 1] >> (23 - q0);
    } else if (q0 === 0) {
      ih = iq[jz - 1] >> 23;
    } else if (z >= 0.5) {
      ih = 2;
    }

    if (ih > 0) {
      /* q > 0.5 */
      n += 1;
      let carry = 0;
      for (let i = 0; i < jz; i++) {
        /* compute 1-q */
        const j = iq[i];
        if (carry === 0) {
          if (j !== 0) {
            carry = 1;
            iq[i] = 0x1000000 - j;
          }
        } else {
          iq[i] = 0xffffff - j;
        }
      }
      if (q0 > 0) {
        /* rare case: chance is 1 in 12 */
        switch (q0) {
          case 1:
            iq[jz - 1] &= 0x7fffff;
            break;
          case 2:
            iq[jz - 1] &= 0x3fffff;
            break;
          default:
            break;
        }
      }
      if (ih === 2) {
        z = 1 - z;
        if (carry !== 0) {
          z -= scalbn(1, q0);
        }
      }
    }

    /* check if recomputation is needed */
    if (z === 0) {
      let j = 0;
      for (let i = jz - 1; i >= jk; i--) {
        j |= iq[i];
      }
      if (j === 0) {
        /* need recomputation */
        let k = 1;
        while (iq[jk - k] === 0) {
          k += 1; /* k = no. of terms needed */
        }

        for (let i = jz + 1; i <= jz + k; i++) {
          /* add q[jz+1] to q[jz+k] */
          f[jx + i] = IPIO2[jv + i];
          fw = 0;
          for (let j = 0; j <= jx; j++) {
            fw += x[j] * f[jx + i - j];
          }
          q[i] = fw;
        }
        jz += k;
        continue recompute;
      }
    }

    break;
  }

  /* chop off zero terms */
  if (z === 0) {
    jz -= 1;
    q0 -= 24;
    while (iq[jz] === 0) {
      jz -= 1;
      q0 -= 24;
    }
  } else {
    /* break z into 24-bit if necessary */
    z = scalbn(z, -q0);
    if (z >= X1P24) {
      fw = (X1P_24 * z) | 0;
      iq[jz] = (z - X1P24 * fw) | 0;
      jz += 1;
      q0 += 24;
      iq[jz] = fw | 0;
    } else {
      iq[jz] = z | 0;
    }
  }

  /* convert integer "bit" chunk to floating-point value */
  fw = scalbn(1, q0);
  for (let i = jz; i >= 0; i--) {
    q[i] = fw * iq[i];
    fw *= X1P_24;
  }

  /* compute PIo2[0,...,jp]*q[jz,...,0] */
  for (let i = jz; i >= 0; i--) {
    fw = 0;
    let k = 0;
    while (k <= jp && k <= jz - i) {
      fw += PIO2[k] * q[i + k];
      k += 1;
    }
    fq[jz - i] = fw;
  }

  /* compress fq[] into y[] */
  switch (prec) {
    case 0:
      fw = 0;
      for (let i = jz; i >= 0; i--) {
        fw += fq[i];
      }
      y[0] = ih === 0 ? fw : -fw;
      break;
    case 1:
    case 2:
      fw = 0;
      for (let i = jz; i >= 0; i--) {
        fw += fq[i];
      }
      y[0] = ih === 0 ? fw : -fw;
      fw = fq[0] - fw;
      for (let i = 1; i <= jz; i++) {
        fw += fq[i];
      }
      y[1] = ih === 0 ? fw : -fw;
      break;
    case 3:
      /* painful */
      for (let i = jz; i >= 1; i--) {
        fw = fq[i - 1] + fq[i];
        fq[i] += fq[i - 1] - fw;
        fq[i - 1] = fw;
      }
      for (let i = jz; i >= 2; i--) {
        fw = fq[i - 1] + fq[i];
        fq[i] += fq[i - 1] - fw;
        fq[i - 1] = fw;
      }
      fw = 0;
      for (let i = jz; i >= 2; i--) {
        fw += fq[i];
      }
      if (ih === 0) {
        y[0] = fq[0];
        y[1] = fq[1];
        y[2] = fw;
      } else {
        y[0] = -fq[0];
        y[1] = -fq[1];
        y[2] = -fw;
      }
      break;
    default:
      break;
  }
  return n & 7;
}

// ---------------------------------------------------------------------------
// rem_pio2.rs

const EPS = 2.2204460492503131e-16;
const TO_INT = 1.5 / EPS;
const INV_PIO2 = 6.36619772367581382433e-1; /* 0x3FE45F30, 0x6DC9C883 */
const PIO2_1 = 1.57079632673412561417e+0; /* 0x3FF921FB, 0x54400000 */
const PIO2_1T = 6.07710050650619224932e-11; /* 0x3DD0B461, 0x1A626331 */
const PIO2_2 = 6.07710050630396597660e-11; /* 0x3DD0B461, 0x1A600000 */
const PIO2_2T = 2.02226624879595063154e-21; /* 0x3BA3198A, 0x2E037073 */
const PIO2_3 = 2.02226624871116645580e-21; /* 0x3BA3198A, 0x2E000000 */
const PIO2_3T = 8.47842766036889956997e-32; /* 0x397B839A, 0x252049C1 */

/**
 * @param {number} x
 * @param {number} ix
 * @returns {[number, number, number]}
 */
function medium(x, ix) {
  /* rint(x/(pi/2)), Assume round-to-nearest. */
  const tmp = x * INV_PIO2 + TO_INT;
  const fN = tmp - TO_INT;
  const n = fN | 0;
  let r = x - fN * PIO2_1;
  let w = fN * PIO2_1T; /* 1st round, good to 85 bits */
  let y0 = r - w;
  const ey = (getHighWord(y0) >>> 20) & 0x7ff;
  const ex = ix >>> 20;
  if (ex - ey > 16) {
    /* 2nd round, good to 118 bits */
    let t = r;
    w = fN * PIO2_2;
    r = t - w;
    w = fN * PIO2_2T - ((t - r) - w);
    y0 = r - w;
    const ey = (getHighWord(y0) >>> 20) & 0x7ff;
    if (ex - ey > 49) {
      /* 3rd round, good to 151 bits, covers all cases */
      t = r;
      w = fN * PIO2_3;
      r = t - w;
      w = fN * PIO2_3T - ((t - r) - w);
      y0 = r - w;
    }
  }
  const y1 = (r - y0) - w;
  return [n, y0, y1];
}

/**
 * Reduce `x` modulo pi/2: returns `[n, y0, y1]` with x = n*pi/2 + (y0 + y1).
 * @param {number} x
 * @returns {[number, number, number]}
 */
export function remPio2(x) {
  const hx = getHighWord(x);
  const sign = hx >>> 31;
  const ix = hx & 0x7fffffff;

  if (ix <= 0x400f6a7a) {
    /* |x| ~<= 5pi/4 */
    if ((ix & 0xfffff) === 0x921fb) {
      /* |x| ~= pi/2 or 2pi/2 */
      return medium(x, ix); /* cancellation -- use medium case */
    }
    if (ix <= 0x4002d97c) {
      /* |x| ~<= 3pi/4 */
      if (sign === 0) {
        const z = x - PIO2_1; /* one round good to 85 bits */
        const y0 = z - PIO2_1T;
        const y1 = (z - y0) - PIO2_1T;
        return [1, y0, y1];
      } else {
        const z = x + PIO2_1;
        const y0 = z + PIO2_1T;
        const y1 = (z - y0) + PIO2_1T;
        return [-1, y0, y1];
      }
    } else if (sign === 0) {
      const z = x - 2.0 * PIO2_1;
      const y0 = z - 2.0 * PIO2_1T;
      const y1 = (z - y0) - 2.0 * PIO2_1T;
      return [2, y0, y1];
    } else {
      const z = x + 2.0 * PIO2_1;
      const y0 = z + 2.0 * PIO2_1T;
      const y1 = (z - y0) + 2.0 * PIO2_1T;
      return [-2, y0, y1];
    }
  }
  if (ix <= 0x401c463b) {
    /* |x| ~<= 9pi/4 */
    if (ix <= 0x4015fdbc) {
      /* |x| ~<= 7pi/4 */
      if (ix === 0x4012d97c) {
        /* |x| ~= 3pi/2 */
        return medium(x, ix);
      }
      if (sign === 0) {
        const z = x - 3.0 * PIO2_1;
        const y0 = z - 3.0 * PIO2_1T;
        const y1 = (z - y0) - 3.0 * PIO2_1T;
        return [3, y0, y1];
      } else {
        const z = x + 3.0 * PIO2_1;
        const y0 = z + 3.0 * PIO2_1T;
        const y1 = (z - y0) + 3.0 * PIO2_1T;
        return [-3, y0, y1];
      }
    } else {
      if (ix === 0x401921fb) {
        /* |x| ~= 4pi/2 */
        return medium(x, ix);
      }
      if (sign === 0) {
        const z = x - 4.0 * PIO2_1;
        const y0 = z - 4.0 * PIO2_1T;
        const y1 = (z - y0) - 4.0 * PIO2_1T;
        return [4, y0, y1];
      } else {
        const z = x + 4.0 * PIO2_1;
        const y0 = z + 4.0 * PIO2_1T;
        const y1 = (z - y0) + 4.0 * PIO2_1T;
        return [-4, y0, y1];
      }
    }
  }
  if (ix < 0x413921fb) {
    /* |x| ~< 2^20*(pi/2), medium size */
    return medium(x, ix);
  }
  /*
   * all other (large) arguments
   */
  if (ix >= 0x7ff00000) {
    /* x is inf or NaN */
    const y0 = x - x;
    const y1 = y0;
    return [0, y0, y1];
  }
  /* set z = scalbn(|x|,-ilogb(x)+23) */
  let z = combineWords((hx & 0x000fffff) | ((0x3ff + 23) << 20), getLowWord(x));
  const tx = [0, 0, 0];
  for (let i = 0; i < 2; i++) {
    tx[i] = z | 0;
    z = (z - tx[i]) * X1P24;
  }
  tx[2] = z;
  /* skip zero terms, first term is non-zero */
  let i = 2;
  while (i !== 0 && tx[i] === 0) {
    i -= 1;
  }
  const ty = [0, 0, 0];
  const n = remPio2Large(tx.slice(0, i + 1), ty, (ix >> 20) - (0x3ff + 23), 1);
  if (sign !== 0) {
    return [-n, -ty[0], -ty[1]];
  }
  return [n, ty[0], ty[1]];
}
