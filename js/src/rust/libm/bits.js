// Bit-level helpers shared by the libm port (libm 0.2.16, src/math/mod.rs and
// src/math/generic/scalbn.rs).
//
// The f64 <-> word casts go through one shared Float64Array, viewed as two u32
// words and as one u64. The u32 view follows the platform byte order, so the
// index of the high word is detected once. Only NaN payloads are not
// guaranteed to survive a typed-array round trip (ECMAScript lets engines
// canonicalise NaN); every other bit pattern, including -0 and subnormals,
// does.

const F64 = new Float64Array(1);
const U32 = new Uint32Array(F64.buffer);
const U64 = new BigUint64Array(F64.buffer);
const LITTLE_ENDIAN = new Uint8Array(new Uint32Array([1]).buffer)[0] === 1;
const HI = LITTLE_ENDIAN ? 1 : 0;
const LO = LITTLE_ENDIAN ? 0 : 1;

/**
 * `(x.to_bits() >> 32) as u32`, as a number in [0, 2^32).
 * @param {number} x
 * @returns {number}
 */
export function getHighWord(x) {
  F64[0] = x;
  return U32[HI];
}

/**
 * `x.to_bits() as u32`, as a number in [0, 2^32).
 * @param {number} x
 * @returns {number}
 */
export function getLowWord(x) {
  F64[0] = x;
  return U32[LO];
}

/**
 * Replace the high 32 bits of `f`.
 * @param {number} f
 * @param {number} hi u32 (or i32; only the low 32 bits are used)
 * @returns {number}
 */
export function withSetHighWord(f, hi) {
  F64[0] = f;
  U32[HI] = hi;
  return F64[0];
}

/**
 * Replace the low 32 bits of `f`.
 * @param {number} f
 * @param {number} lo u32 (or i32; only the low 32 bits are used)
 * @returns {number}
 */
export function withSetLowWord(f, lo) {
  F64[0] = f;
  U32[LO] = lo;
  return F64[0];
}

/**
 * `f64::from_bits(((hi as u64) << 32) | lo as u64)`.
 * @param {number} hi
 * @param {number} lo
 * @returns {number}
 */
export function combineWords(hi, lo) {
  U32[HI] = hi;
  U32[LO] = lo;
  return F64[0];
}

/**
 * `f64::to_bits`.
 * @param {number} x
 * @returns {bigint}
 */
export function toBits(x) {
  F64[0] = x;
  return U64[0];
}

/**
 * `f64::from_bits` (the argument is reduced modulo 2^64).
 * @param {bigint} bits
 * @returns {number}
 */
export function fromBits(bits) {
  U64[0] = BigInt.asUintN(64, bits);
  return F64[0];
}

/**
 * `hf64!("0x1.<13 hex digits>p<exp>")` for normal numbers: a hexadecimal
 * float literal decoded straight into its bit pattern (no rounding).
 * @param {string} s
 * @returns {number}
 */
export function hf64(s) {
  const m = /^(-?)0x1(?:\.([0-9a-f]{1,13}))?p([+-]?\d+)$/.exec(s);
  if (m === null) {
    throw new Error(`bad hex float literal: ${s}`);
  }
  const exp = Number(m[3]) + 1023;
  if (exp < 1 || exp > 2046) {
    throw new Error(`hex float literal out of the normal range: ${s}`);
  }
  const frac = BigInt(`0x${(m[2] ?? '').padEnd(13, '0')}`);
  const sign = m[1] === '-' ? 1n : 0n;
  return fromBits((sign << 63n) | (BigInt(exp) << 52n) | frac);
}

const F_EXP_MAX = combineWords(0x7fe00000, 0); // 2^1023
const F_EXP_MIN_TIMES_POW_SUBNORM = combineWords(0x03600000, 0); // 2^-1022 * 2^53 = 2^-969

/**
 * `scalbn(x, n)`: x * 2^n with one final rounding (generic/scalbn.rs, f64).
 * @param {number} x
 * @param {number} n i32
 * @returns {number}
 */
export function scalbn(x, n) {
  const EXP_MAX = 1023;
  const EXP_MIN = -1022;
  if (n > EXP_MAX) {
    x *= F_EXP_MAX;
    n -= EXP_MAX;
    if (n > EXP_MAX) {
      x *= F_EXP_MAX;
      n -= EXP_MAX;
      if (n > EXP_MAX) {
        n = EXP_MAX;
      }
    }
  } else if (n < EXP_MIN) {
    const mul = F_EXP_MIN_TIMES_POW_SUBNORM;
    const add = -EXP_MIN - 53;
    x *= mul;
    n += add;
    if (n < EXP_MIN) {
      x *= mul;
      n += add;
      if (n < EXP_MIN) {
        n = EXP_MIN;
      }
    }
  }
  const scale = combineWords((1023 + n) << 20, 0);
  return x * scale;
}
