// cbrt: a port of libm 0.2.16 src/math/cbrt.rs (`cbrt_round` with
// `Round::Nearest`; origin core-math src/binary64/cbrt/cbrt.c by Alexei
// Sibidanov, ported to Rust by Trevor Gross). The u64 bit manipulation is done
// with bigint; the inner `f64::fma` calls go to the libm fma port.

import { fromBits, hf64, toBits } from './bits.js';
import { fma } from './fma.js';

const MASK64 = 0xffffffffffffffffn;
const SIG_MASK = 0x000fffffffffffffn;
const ABS_MASK = 0x7fffffffffffffffn;

const ESCALE = [
  1.0,
  hf64('0x1.428a2f98d728bp+0'), /* 2^(1/3) */
  hf64('0x1.965fea53d6e3dp+0'), /* 2^(2/3) */
];

/* the polynomial c0+c1*x+c2*x^2+c3*x^3 approximates x^(1/3) on [1,2]
with maximal error < 9.2e-5 (attained at x=2) */
const C = [
  hf64('0x1.1b0babccfef9cp-1'),
  hf64('0x1.2c9a3e94d1da5p-1'),
  hf64('-0x1.4dc30b1a1ddbap-3'),
  hf64('0x1.7a8d3e4ec9b07p-6'),
];

const U0 = hf64('0x1.5555555555555p-2');
const U1 = hf64('0x1.c71c71c71c71cp-3');

const RSC = [1.0, -1.0, 0.5, -0.5, 0.25, -0.25];

/* off[Round::Nearest] */
const OFF = hf64('0x1p-53');

const P2M52 = hf64('0x1p-52');
const P2M60 = hf64('0x1p-60');
const P2M75 = hf64('0x1p-75');
const P2M98 = hf64('0x1p-98');

const AZZ_A = hf64('0x1.9b78223aa307cp+1');
const Y1_A = hf64('0x1.79d15d0e8d59cp+0'); // ~ 0x1.79d15d0e8d59b80000000000000ffc3dp+0
const AZZ_B = hf64('0x1.a202bfc89ddffp+2');
const Y1_B = hf64('0x1.de87aa837820fp+0'); // ~ 0x1.de87aa837820e80000000000001c0f08p+0

/**
 * `u64::leading_zeros` for a non-zero value below 2^64.
 * @param {bigint} v
 * @returns {number}
 */
function clz64(v) {
  return 64 - v.toString(2).length;
}

/**
 * `libm::cbrt`.
 * @param {number} x
 * @returns {number}
 */
export function cbrt(x) {
  /* rm=0 for rounding to nearest, and other values for directed roundings */
  const hx = toBits(x);
  let mant = hx & SIG_MASK;
  const sign = Number(hx >> 63n);

  let e = Number(hx >> 52n) & 0x7ff;

  if (((e + 1) & 0x7ff) < 2) {
    const ix = hx & ABS_MASK;

    /* 0, inf, nan: we return x + x instead of simply x,
    to that for x a signaling NaN, it correctly triggers
    the invalid exception. */
    if (e === 0x7ff || ix === 0n) {
      return x + x;
    }

    const nz = clz64(ix) - 11; /* subnormal */
    mant <<= BigInt(nz);
    mant &= SIG_MASK;
    e = (e - (nz - 1)) >>> 0; // u32 wrapping_sub
  }

  e = (e + 3072) >>> 0; // u32 wrapping_add
  const cvt1 = mant | (0x3ffn << 52n);
  let cvt5 = cvt1;

  const et = Math.trunc(e / 3);
  const it = e % 3;

  /* 2^(3k+it) <= x < 2^(3k+it+1), with 0 <= it <= 3 */
  cvt5 += BigInt(it) << 52n;
  cvt5 |= BigInt(sign) << 63n;
  const zz = fromBits(cvt5);

  /* cbrt(x) = cbrt(zz)*2^(et-1365) where 1 <= zz < 8 */
  let isc = toBits(ESCALE[it]);
  isc |= BigInt(sign) << 63n;
  const cvt2 = isc;
  const z = fromBits(cvt1);

  /* cbrt(zz) = cbrt(z)*isc, where isc encodes 1, 2^(1/3) or 2^(2/3),
  and 1 <= z < 2 */
  const r = 1.0 / z;
  const rr = r * RSC[(it << 1) | sign];
  const z2 = z * z;
  const c0 = C[0] + z * C[1];
  const c2 = C[2] + z * C[3];
  let y = c0 + z2 * c2;
  let y2 = y * y;

  /* y is an approximation of z^(1/3) */
  let h = y2 * (y * r) - 1.0;

  /* h determines the error between y and z^(1/3) */
  y -= (h * y) * (U0 - U1 * h);

  /* The correction y -= (h*y)*(u0 - u1*h) corresponds to a cubic variant
  of Newton's method, with the function f(y) = 1-z/y^3. */
  y *= fromBits(cvt2);

  /* Now y is an approximation of zz^(1/3),
   * and rr an approximation of 1/zz. We now perform another iteration of
   * Newton-Raphson, this time with a linear approximation only. */
  y2 = y * y;
  let y2l = fma(y, y, -y2);

  /* y2 + y2l = y^2 exactly */
  let y3 = y2 * y;
  let y3l = fma(y, y2, -y3) + y * y2l;

  /* y3 + y3l approximates y^3 with about 106 bits of accuracy */
  h = ((y3 - zz) + y3l) * rr;
  let dy = h * (y * U0);

  /* the approximation of zz^(1/3) is y - dy */
  let y1 = y - dy;
  dy = (y - y1) - dy;

  /* the approximation of zz^(1/3) is now y1 + dy, where |dy| < 1/2 ulp(y)
   * (for rounding to nearest) */
  let ady = Math.abs(dy);

  /* For directed roundings, ady0 is tiny when dy is tiny, or ady0 is near
   * from ulp(1);
   * for rounding to nearest, ady0 is tiny when dy is near from 1/2 ulp(1),
   * or from 3/2 ulp(1). */
  let ady0 = Math.abs(ady - OFF);
  let ady1 = Math.abs(ady - (P2M52 + OFF));

  if (ady0 < P2M75 || ady1 < P2M75) {
    y2 = y1 * y1;
    y2l = fma(y1, y1, -y2);
    y3 = y2 * y1;
    y3l = fma(y1, y2, -y3) + y1 * y2l;
    h = ((y3 - zz) + y3l) * rr;
    dy = h * (y1 * U0);
    y = y1 - dy;
    dy = (y1 - y) - dy;
    y1 = y;
    ady = Math.abs(dy);
    ady0 = Math.abs(ady - OFF);
    ady1 = Math.abs(ady - (P2M52 + OFF));

    if (ady0 < P2M98 || ady1 < P2M98) {
      const azz = Math.abs(zz);

      // `copysign(c, zz)`: zz carries the sign of x
      if (azz === AZZ_A) {
        y1 = sign !== 0 ? -Y1_A : Y1_A;
      }

      if (azz === AZZ_B) {
        y1 = sign !== 0 ? -Y1_B : Y1_B;
      }

      // The `round != Round::Nearest` table is not needed here.
    }
  }

  let cvt3 = toBits(y1);
  cvt3 = (cvt3 + ((BigInt((et - 342 - 1023) >>> 0) << 52n) & MASK64)) & MASK64;
  const m0 = (cvt3 << 30n) & MASK64;
  const m1 = m0 >> 63n;

  if ((m0 ^ m1) <= 1n << 30n) {
    let cvt4 = toBits(y1);
    cvt4 = (cvt4 + (164n << 15n)) & 0xffffffffffff0000n;

    if (Math.abs((fromBits(cvt4) - y1) - dy) < P2M60 || Math.abs(zz) === 1.0) {
      cvt3 = (cvt3 + (1n << 15n)) & 0xffffffffffff0000n;
    }
  }

  return fromBits(cvt3);
}
