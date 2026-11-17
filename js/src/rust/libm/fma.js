// fma: a port of libm 0.2.16 src/math/generic/fma.rs (`fma_round` for f64,
// origin musl src/math/fma.c).
//
// On x86_64 and aarch64 libm uses the FMA instruction, which is correctly
// rounded. This port is correctly rounded too: it follows generic/fma.rs line
// by line except for one sticky bit (see below) where the Rust code loses
// precision. The u64/u128 integer arithmetic of the Rust code is done with
// bigint, reduced modulo 2^64 wherever Rust wraps or truncates.

import { fromBits, scalbn, toBits } from './bits.js';

const MASK64 = 0xffffffffffffffffn;
const SIG_MASK = 0x000fffffffffffffn;
const IMPLICIT_BIT = 0x0010000000000000n;

/* Norm::<f64>::EXP_UNBIAS = EXP_BIAS + SIG_BITS + 1 */
const EXP_UNBIAS = 1023 + 52 + 1;
/* Norm::<f64>::ZERO_INF_NAN = EXP_SAT - EXP_UNBIAS */
const ZERO_INF_NAN = 2047 - EXP_UNBIAS;
/* F::from_parts(false, BITS - 1 + EXP_BIAS, 0) = 2^63 */
const SCALE_F = fromBits(BigInt(63 + 1023) << 52n);
/* max_pow = BITS - 1 + EXP_BIAS */
const MAX_POW = 64 - 1 + 1023;
/* F::from_parts(false, max_pow, 0) = 2^63 */
const C_MAX_POW = fromBits(BigInt(MAX_POW) << 52n);
const MIN_POSITIVE_NORMAL = fromBits(0x0010000000000000n); // 2^-1022

/**
 * `u64::leading_zeros` for a non-zero value below 2^64.
 * @param {bigint} v
 * @returns {number}
 */
function clz64(v) {
  let n = 0;
  if (v >> 32n === 0n) {
    n += 32;
    v <<= 32n;
  }
  if (v >> 48n === 0n) {
    n += 16;
    v <<= 16n;
  }
  if (v >> 56n === 0n) {
    n += 8;
    v <<= 8n;
  }
  if (v >> 60n === 0n) {
    n += 4;
    v <<= 4n;
  }
  if (v >> 62n === 0n) {
    n += 2;
    v <<= 2n;
  }
  if (v >> 63n === 0n) {
    n += 1;
  }
  return n;
}

/**
 * `(x << s) as u64` with `0 <= s < 64`.
 * @param {bigint} x
 * @param {number} s
 * @returns {bigint}
 */
function shl(x, s) {
  return (x << BigInt(s)) & MASK64;
}

/**
 * `x >> s` with `0 <= s < 64`.
 * @param {bigint} x
 * @param {number} s
 * @returns {bigint}
 */
function shr(x, s) {
  return x >> BigInt(s);
}

/**
 * @typedef {{ m: bigint, e: number, neg: boolean }} Norm
 */

/**
 * `Norm::from_float`: the significand with the implicit bit and a guard bit,
 * and the exponent such that `m * 2^e = |x|`; zero gets the sentinel exponent
 * `(1 << EXP_BITS) - EXP_UNBIAS`.
 * @param {number} x
 * @returns {Norm}
 */
function norm(x) {
  let ix = toBits(x);
  let e = Number((ix >> 52n) & 0x7ffn);
  const neg = ix >> 63n !== 0n;
  if (e === 0) {
    // Normalize subnormals by multiplication
    const scaled = x * SCALE_F;
    ix = toBits(scaled);
    e = Number((ix >> 52n) & 0x7ffn);
    e = e === 0 ? 1 << 11 : e - 63;
  }

  e -= EXP_UNBIAS;

  // Absolute value, set the implicit bit, and shift to create a guard bit
  ix &= SIG_MASK;
  ix |= IMPLICIT_BIT;
  ix <<= 1n;

  return { m: ix, e, neg };
}

/**
 * `libm::fma` (Rust `f64::mul_add`): `x * y + z` with a single rounding.
 * @param {number} x
 * @param {number} y
 * @param {number} z
 * @returns {number}
 */
export function fma(x, y, z) {
  const nx = norm(x);
  const ny = norm(y);
  const nz = norm(z);

  if (nx.e >= ZERO_INF_NAN || ny.e >= ZERO_INF_NAN) {
    // Value will overflow, defer to non-fused operations.
    return x * y + z;
  }

  if (nz.e >= ZERO_INF_NAN) {
    if (nz.e > ZERO_INF_NAN) {
      // Empty add component means we only need to multiply.
      return x * y;
    }
    // `z` is NaN or infinity, which sets the result.
    return z;
  }

  // multiply: r = x * y
  let zhi;
  let zlo;
  const prod = nx.m * ny.m;
  let rlo = prod & MASK64;
  let rhi = prod >> 64n;

  // Exponent result of multiplication
  let e = nx.e + ny.e;
  // Needed shift to align `z` to the multiplication result
  let d = nz.e - e;
  const sbits = 64;

  // Scale `z`. Shift `z <<= kz`, `r >>= kr`, so `kz+kr == d`, set `e = e+kr` (== ez-kz)
  if (d > 0) {
    // The magnitude of `z` is larger than `x * y`
    if (d < sbits) {
      zlo = shl(nz.m, d);
      zhi = shr(nz.m, sbits - d);
    } else {
      zlo = 0n;
      zhi = nz.m;
      d -= sbits;

      // `z`'s exponent is large enough that it now needs to be taken into account.
      e = nz.e - sbits;

      if (d === 0) {
        // Exactly `sbits`, nothing to do
      } else if (d < sbits) {
        // Remaining shift fits within `sbits`. Leave `z` in place, shift `x * y`
        // and set the sticky bit from the bits shifted out of `rlo`.
        //
        // Deliberate deviation: generic/fma.rs tests the *updated* `rlo`
        // (`rlo = ...; rlo |= (rlo << (sbits - d)) != 0`), which drops the
        // sticky bit and rounds e.g. fma(1 + 2^-26, 1 - 2^-26 + 2^-52, 2^53)
        // to 2^53 instead of 2^53 + 2. musl's fma.c, the FMA instruction that
        // libm uses on x86_64 and aarch64, and IEEE 754 all give 2^53 + 2, so
        // this follows them.
        const sticky = shl(rlo, sbits - d) !== 0n ? 1n : 0n;
        rlo = shl(rhi, sbits - d) | shr(rlo, d) | sticky;
        rhi = shr(rhi, d);
      } else {
        // `z`'s magnitude is enough that `x * y` is irrelevant. It was nonzero, so set
        // the sticky bit.
        rlo = 1n;
        rhi = 0n;
      }
    }
  } else {
    // `z`'s magnitude once shifted fits entirely within `zlo`
    zhi = 0n;
    d = -d;
    if (d === 0) {
      // No shift needed
      zlo = nz.m;
    } else if (d < sbits) {
      // Shift s.t. `nz.m` fits into `zlo`
      const sticky = shl(nz.m, sbits - d) !== 0n ? 1n : 0n;
      zlo = shr(nz.m, d) | sticky;
    } else {
      // Would be entirely shifted out, only set the sticky bit
      zlo = 1n;
    }
  }

  /* addition */

  let neg = nx.neg !== ny.neg;
  const samesign = !neg !== nz.neg;
  let rhiNonzero = true;

  if (samesign) {
    // r += z
    rlo = (rlo + zlo) & MASK64;
    rhi = (rhi + zhi + (rlo < zlo ? 1n : 0n)) & MASK64;
  } else {
    // r -= z
    const borrow = rlo < zlo ? 1n : 0n;
    rlo = (rlo - zlo) & MASK64;
    rhi = (rhi - ((zhi + borrow) & MASK64)) & MASK64;
    if (rhi >> 63n !== 0n) {
      rlo = -rlo & MASK64;
      rhi = (-rhi & MASK64) - (rlo !== 0n ? 1n : 0n);
      neg = !neg;
    }
    rhiNonzero = rhi !== 0n;
  }

  /* Construct result */

  // Shift result into `rhi`, left-aligned. Last bit is sticky
  if (rhiNonzero) {
    // `d` > 0, need to shift both `rhi` and `rlo` into result
    e += sbits;
    d = clz64(rhi) - 1;
    rhi = shl(rhi, d) | shr(rlo, sbits - d);
    // Update sticky
    rhi |= shl(rlo, d) !== 0n ? 1n : 0n;
  } else if (rlo !== 0n) {
    // `rhi` is zero, `rlo` is the entire result and needs to be shifted
    d = clz64(rlo) - 1;
    if (d < 0) {
      // Shift and set sticky
      rhi = (rlo >> 1n) | (rlo & 1n);
    } else {
      rhi = shl(rlo, d);
    }
  } else {
    // exact +/- 0.0
    return x * y + z;
  }

  e -= d;

  // Use int->float conversion to populate the significand.
  // i is in [1 << (BITS - 2), (1 << (BITS - 1)) - 1]
  let i = rhi;

  if (neg) {
    i = -i;
  }

  // `|r|` is in `[0x1p62,0x1p63]` for `f64`; Number(bigint) rounds to nearest, ties to even
  let r = Number(i);

  /* Account for subnormal and rounding */

  if (e < -(MAX_POW - 2)) {
    // Result is subnormal before rounding
    if (e === -(MAX_POW - 1)) {
      let c = C_MAX_POW;
      if (neg) {
        c = -c;
      }

      if (r === c) {
        // Min normal after rounding,
        return r < 0 ? -MIN_POSITIVE_NORMAL : MIN_POSITIVE_NORMAL;
      }

      if (shl(rhi, 52 + 1) !== 0n) {
        // Account for truncated bits. One bit will be lost in the `scalbn` call, add
        // another top bit to avoid double rounding if inexact.
        const iu = (rhi >> 1n) | (rhi & 1n) | (1n << 62n);
        i = iu;

        if (neg) {
          i = -i;
        }

        r = Number(i);

        // Remove the top bit
        r = 2 * r - c;
      }
    } else {
      // Only round once when scaled
      d = 11 - 1;
      const sticky = shl(rhi, sbits - d) !== 0n ? 1n : 0n;
      i = ((rhi >> BigInt(d)) | sticky) << BigInt(d);

      if (neg) {
        i = -i;
      }

      r = Number(i);
    }
  }

  // Use our exponent to scale the final value.
  return scalbn(r, e);
}
