// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `num_rational::Ratio<BigInt>` (`BigRational`): an immutable numerator and
// |  * denominator. `Ratio.new` reduces; `Ratio.newRaw` keeps the fields as given,
// |  * and every method then behaves as num-rational does on such a value. Binary
// |  * operators take a `Ratio` or a `bigint` (Rust's `Ratio<T> op T`).
// |  */
// | export class Ratio {
// |   /**
// |    * `Ratio::new_raw`.
// |    * @param {bigint} numer
// |    * @param {bigint} denom
// |    */
// |   constructor(numer, denom) {
// |     /** @readonly */
// |     this._numer = numer;
// |     /** @readonly */
// |     this._denom = denom;
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * `Ratio::new`: reduced, with a positive denominator; panics
// |    * "denominator == 0".
// |    * @param {bigint} numer
// |    * @param {bigint} denom
// |    * @returns {Ratio}
// |    */
// |   static new(numer, denom) {
// |     return reduced(numer, denom);
// |   }
// |
// |   /**
// |    * `Ratio::new_raw`: no reduction, no checks.
// |    * @param {bigint} numer
// |    * @param {bigint} denom
// |    * @returns {Ratio}
// |    */
// |   static newRaw(numer, denom) {
// |     return new Ratio(numer, denom);
// |   }
// |
// |   /**
// |    * `Ratio::from_integer` (and `From<BigInt>`): `t/1`.
// |    * @param {bigint} t
// |    * @returns {Ratio}
// |    */
// |   static fromInteger(t) {
// |     return new Ratio(t, 1n);
// |   }
// |
// |   /** `Zero::zero`: 0/1. @returns {Ratio} */
// |   static zero() {
// |     return ZERO;
// |   }
// |
// |   /** `One::one`: 1/1. @returns {Ratio} */
// |   static one() {
// |     return ONE;
// |   }
// |
// |   /** `Ratio::numer`. @returns {bigint} */
// |   numer() {
// |     return this._numer;
// |   }
// |
// |   /** `Ratio::denom`. @returns {bigint} */
// |   denom() {
// |     return this._denom;
// |   }
// |
// |   /** `Ratio::is_integer`: `denom == 1`. @returns {boolean} */
// |   isInteger() {
// |     return this._denom === 1n;
// |   }
// |
// |   /** `Zero::is_zero`: `numer == 0`. @returns {boolean} */
// |   isZero() {
// |     return this._numer === 0n;
// |   }
// |
// |   /** `One::is_one`: `numer == denom`. @returns {boolean} */
// |   isOne() {
// |     return this._numer === this._denom;
// |   }
// |
// |   /** `Signed::is_positive`. @returns {boolean} */
// |   isPositive() {
// |     const n = this._numer;
// |     const d = this._denom;
// |     return (n > 0n && d > 0n) || (n < 0n && d < 0n);
// |   }
// |
// |   /** `Signed::is_negative`. @returns {boolean} */
// |   isNegative() {
// |     const n = this._numer;
// |     const d = this._denom;
// |     return (n < 0n && d > 0n) || (n > 0n && d < 0n);
// |   }
// |
// |   /** `Neg`: `new_raw(-numer, denom)`. @returns {Ratio} */
// |   neg() {
// |     return new Ratio(-this._numer, this._denom);
// |   }
// |
// |   /** `Signed::abs`. @returns {Ratio} */
// |   abs() {
// |     return this.isNegative() ? this.neg() : this;
// |   }
// |
// |   /** `Signed::signum`: 1/1, 0/1 or -1/1. @returns {Ratio} */
// |   signum() {
// |     if (this.isPositive()) {
// |       return ONE;
// |     }
// |     if (this.isZero()) {
// |       return ZERO;
// |     }
// |     return ONE.neg();
// |   }
// |
// |   /**
// |    * `Ratio::recip`: swaps the fields, moving the sign to the numerator;
// |    * panics "division by zero".
// |    * @returns {Ratio}
// |    */
// |   recip() {
// |     const n = this._numer;
// |     const d = this._denom;
// |     if (n === 0n) {
// |       panic('division by zero');
// |     }
// |     return n > 0n ? new Ratio(d, n) : new Ratio(-d, -n);
// |   }
// |
// |   /**
// |    * `Add`.
// |    * @param {Ratio | bigint} rhs
// |    * @returns {Ratio}
// |    */
// |   add(rhs) {
// |     return arith(this, rhs, plus);
// |   }
// |
// |   /**
// |    * `Sub`.
// |    * @param {Ratio | bigint} rhs
// |    * @returns {Ratio}
// |    */
// |   sub(rhs) {
// |     return arith(this, rhs, minus);
// |   }
// |
// |   /**
// |    * `Rem`: the remainder of truncating division, with the sign of `self`.
// |    * @param {Ratio | bigint} rhs
// |    * @returns {Ratio}
// |    */
// |   rem(rhs) {
// |     return arith(this, rhs, bigintRem);
// |   }
// |
// |   /**
// |    * `Mul`.
// |    * @param {Ratio | bigint} rhs
// |    * @returns {Ratio}
// |    */
// |   mul(rhs) {
// |     if (typeof rhs === 'bigint') {
// |       const g = gcd(this._denom, rhs);
// |       const denom = bigintDiv(this._denom, g);
// |       return reduced(this._numer * bigintDiv(rhs, g), denom);
// |     }
// |     const gcdAd = gcd(this._numer, rhs._denom);
// |     const gcdBc = gcd(this._denom, rhs._numer);
// |     return reduced(
// |       bigintDiv(this._numer, gcdAd) * bigintDiv(rhs._numer, gcdBc),
// |       bigintDiv(this._denom, gcdBc) * bigintDiv(rhs._denom, gcdAd),
// |     );
// |   }
// |
// |   /**
// |    * `Div`.
// |    * @param {Ratio | bigint} rhs
// |    * @returns {Ratio}
// |    */
// |   div(rhs) {
// |     if (typeof rhs === 'bigint') {
// |       const g = gcd(this._numer, rhs);
// |       const numer = bigintDiv(this._numer, g);
// |       return reduced(numer, this._denom * bigintDiv(rhs, g));
// |     }
// |     const gcdAc = gcd(this._numer, rhs._numer);
// |     const gcdBd = gcd(this._denom, rhs._denom);
// |     return reduced(
// |       bigintDiv(this._numer, gcdAc) * bigintDiv(rhs._denom, gcdBd),
// |       bigintDiv(this._denom, gcdBd) * bigintDiv(rhs._numer, gcdAc),
// |     );
// |   }
// |
// |   /**
// |    * `Pow<u32>` / `Pow<i32>` (and the 64-bit forms): `new_raw(numer^e,
// |    * denom^e)`, without reducing; 1/1 for 0; the `recip` of the positive power
// |    * for a negative exponent.
// |    * @param {number} exp an integer
// |    * @returns {Ratio}
// |    */
// |   pow(exp) {
// |     if (exp < 0) {
// |       const e = -exp;
// |       return new Ratio(bigintPow(this._numer, e), bigintPow(this._denom, e)).recip();
// |     }
// |     if (exp === 0) {
// |       return ONE;
// |     }
// |     return new Ratio(bigintPow(this._numer, exp), bigintPow(this._denom, exp));
// |   }
// |
// |   /** `Ratio::floor`: rounds toward minus infinity. @returns {Ratio} */
// |   floor() {
// |     const n = this._numer;
// |     const d = this._denom;
// |     if (this.cmp(ZERO) < 0) {
// |       return new Ratio(bigintDiv(n - d + 1n, d), 1n);
// |     }
// |     return new Ratio(bigintDiv(n, d), 1n);
// |   }
// |
// |   /** `Ratio::ceil`: rounds toward plus infinity. @returns {Ratio} */
// |   ceil() {
// |     const n = this._numer;
// |     const d = this._denom;
// |     if (this.cmp(ZERO) < 0) {
// |       return new Ratio(bigintDiv(n, d), 1n);
// |     }
// |     return new Ratio(bigintDiv(n + d - 1n, d), 1n);
// |   }
// |
// |   /** `Ratio::round`: to the nearest integer, half away from zero. @returns {Ratio} */
// |   round() {
// |     let fractional = this.fract();
// |     if (fractional.cmp(ZERO) < 0) {
// |       fractional = ZERO.sub(fractional);
// |     }
// |     const halfOrLarger = bigintIsEven(fractional._denom)
// |       ? fractional._numer >= fractional._denom / 2n
// |       : fractional._numer >= fractional._denom / 2n + 1n;
// |     if (halfOrLarger) {
// |       return this.cmp(ZERO) >= 0 ? this.trunc().add(ONE) : this.trunc().sub(ONE);
// |     }
// |     return this.trunc();
// |   }
// |
// |   /** `Ratio::trunc`: rounds toward zero. @returns {Ratio} */
// |   trunc() {
// |     return new Ratio(bigintDiv(this._numer, this._denom), 1n);
// |   }
// |
// |   /**
// |    * `Ratio::fract`: `new_raw(numer % denom, denom)`, with the sign of `self`.
// |    * @returns {Ratio}
// |    */
// |   fract() {
// |     return new Ratio(bigintRem(this._numer, this._denom), this._denom);
// |   }
// |
// |   /** `Ratio::to_integer`: `trunc().numer`. @returns {bigint} */
// |   toInteger() {
// |     return bigintDiv(this._numer, this._denom);
// |   }
// |
// |   /**
// |    * `ToPrimitive::to_f64` of num-rational's `num-bigint` feature (which the
// |    * calculator's build does not enable): correctly rounded, `null` for 0/0.
// |    * @returns {number | null}
// |    */
// |   toF64() {
// |     const f = ratioToF64(this._numer, this._denom);
// |     return Number.isNaN(f) ? null : f;
// |   }
// |
// |   /** `ToPrimitive::to_i64` (feature impl): `to_integer().to_i64()`. @returns {bigint | null} */
// |   toI64() {
// |     return bigintToI64(this.toInteger());
// |   }
// |
// |   /** `ToPrimitive::to_u64` (feature impl). @returns {bigint | null} */
// |   toU64() {
// |     return bigintToU64(this.toInteger());
// |   }
// |
// |   /** `ToPrimitive::to_i128` (feature impl). @returns {bigint | null} */
// |   toI128() {
// |     return bigintToI128(this.toInteger());
// |   }
// |
// |   /** `ToPrimitive::to_u128` (feature impl). @returns {bigint | null} */
// |   toU128() {
// |     return bigintToU128(this.toInteger());
// |   }
// |
// |   /**
// |    * `Ord::cmp`, num-rational's algorithm, which also orders unreduced and
// |    * negative-denominator values (and panics on a zero denominator where it
// |    * has to divide).
// |    * @param {Ratio} other
// |    * @returns {-1 | 0 | 1}
// |    */
// |   cmp(other) {
// |     /** @type {Ratio} */
// |     let a = this;
// |     let b = other;
// |     let flip = false;
// |     for (;;) {
// |       if (a._denom === b._denom) {
// |         const ord = cmpBigint(a._numer, b._numer);
// |         return reverseIf(ord, flip !== a._denom < 0n);
// |       }
// |       if (a._numer === b._numer) {
// |         if (a._numer === 0n) {
// |           return 0;
// |         }
// |         const ord = cmpBigint(a._denom, b._denom);
// |         return reverseIf(ord, flip !== !(a._numer < 0n));
// |       }
// |       const [aInt, aRem] = bigintDivModFloor(a._numer, a._denom);
// |       const [bInt, bRem] = bigintDivModFloor(b._numer, b._denom);
// |       const ord = cmpBigint(aInt, bInt);
// |       if (ord !== 0) {
// |         return reverseIf(ord, flip);
// |       }
// |       if (aRem === 0n) {
// |         return reverseIf(bRem === 0n ? 0 : -1, flip);
// |       }
// |       if (bRem === 0n) {
// |         return reverseIf(1, flip);
// |       }
// |       // Compare the reciprocals of the remaining fractions in reverse.
// |       a = new Ratio(a._denom, aRem);
// |       b = new Ratio(b._denom, bRem);
// |       flip = !flip;
// |     }
// |   }
// |
// |   /**
// |    * `PartialEq`: `cmp(other) == Equal`, so 1/2 equals 2/4.
// |    * @param {unknown} other
// |    * @returns {boolean}
// |    */
// |   equals(other) {
// |     return other instanceof Ratio && this.cmp(other) === 0;
// |   }
// |
// |   /** `Display`: "numer" when `denom == 1`, else "numer/denom". @returns {string} */
// |   toString() {
// |     return this._denom === 1n ? `${this._numer}` : `${this._numer}/${this._denom}`;
// |   }
// |
// |   /** derived `Debug`. @returns {string} */
// |   fmtDebug() {
// |     return `Ratio { numer: ${this._numer}, denom: ${this._denom} }`;
// |   }
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const ZERO = new Ratio(0n, 1n);
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const ONE = new Ratio(1n, 1n);

// ----------------------------------------------------------------------------
// num-rational's private `ratio_to_f64` and `ldexp`.

// meta-language:translated JavaScript lexical_declaration items=1 sha256=2ae56ef9d6d48acd77669a26d41c2dc2f46307a540e8054ac38979ea7715784e
// | const MIN_POSITIVE = 2.2250738585072014e-308;
pub const MIN_POSITIVE: f64 = 2.2250738585072014e-308f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=37dd72dab009b97a0595266d0ecd28d286dd2628a7b61dceb8766f9adcd4a79f
// | const TWO_POW_53 = 9007199254740992;
pub const TWO_POW_53: f64 = 9007199254740992f64;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const EXACT = 1n << 53n;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const view = new DataView(new ArrayBuffer(8));

// meta-language:translated JavaScript function_declaration items=1 sha256=73e68a189e84b0ccce1e2aeb1b907b2547563fc3b469c45b93a4de695aef997d
// | /**
// |  * `f64::is_normal`.
// |  * @param {number} x
// |  */
// | function isNormal(x) {
// |   return Number.isFinite(x) && Math.abs(x) >= MIN_POSITIVE;
// | }
pub fn is_normal(x: f64) -> bool {
    (x.is_finite() && ((x.abs()) >= (MIN_POSITIVE)))
}

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `ratio_to_f64`. num-rational runs it on `i128` when both fields fit `i64`
// |  * and on `BigInt` otherwise; nothing overflows `i128` there, so one bigint
// |  * version gives the same bits.
// |  * @param {bigint} numer
// |  * @param {bigint} denom
// |  * @returns {number}
// |  */
// | function ratioToF64(numer, denom) {
// |   const floSign = Number(bigintSignum(numer)) / Number(bigintSignum(denom));
// |   if (!isNormal(floSign)) {
// |     return floSign;
// |   }
// |   if (numer >= -EXACT && numer <= EXACT && denom >= -EXACT && denom <= EXACT) {
// |     return Number(numer) / Number(denom);
// |   }
// |
// |   let n = bigintAbs(numer);
// |   let d = bigintAbs(denom);
// |   const diff = bigintBits(n) - bigintBits(d);
// |   if (diff > 1024) {
// |     return Infinity * floSign;
// |   }
// |   if (-diff > 1021 + 53 + 1) {
// |     return 0 * floSign;
// |   }
// |
// |   const shift = Math.max(diff, -1021) - 53 - 2;
// |   if (shift >= 0) {
// |     d <<= BigInt(shift);
// |   } else {
// |     n <<= BigInt(-shift);
// |   }
// |   let quotient = n / d;
// |   const remainder = n % d;
// |
// |   const nRoundingBits = BigInt(Math.max(bigintBits(quotient), -1021 - shift) - 53);
// |   const roundingBitMask = (1n << nRoundingBits) - 1n;
// |   const lsBit = (quotient & (1n << nRoundingBits)) !== 0n;
// |   const msRoundingBit = (quotient & (1n << (nRoundingBits - 1n))) !== 0n;
// |   const lsRoundingBits = (quotient & (roundingBitMask >> 1n)) !== 0n;
// |   if (msRoundingBit && (lsBit || lsRoundingBits || remainder !== 0n)) {
// |     quotient += 1n << nRoundingBits;
// |   }
// |   quotient &= ~roundingBitMask;
// |
// |   return ldexp(Number(quotient) * floSign, shift);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * num-rational's `ldexp`: `x * 2^exp` by editing the exponent bits.
// |  * @param {number} x
// |  * @param {number} exp
// |  * @returns {number}
// |  */
// | function ldexp(x, exp) {
// |   if (x === 0 || !Number.isFinite(x)) {
// |     return x;
// |   }
// |   const sign = x < 0 ? -1 : 1;
// |   if (exp > 3 * 1024) {
// |     return Infinity * sign;
// |   }
// |   if (exp < -3 * 1024) {
// |     return 0 * sign;
// |   }
// |
// |   let currExp;
// |   if (isNormal(x)) {
// |     view.setFloat64(0, x);
// |     currExp = (view.getUint32(0) >>> 20) & 0x7ff;
// |   } else {
// |     view.setFloat64(0, x * TWO_POW_53);
// |     currExp = ((view.getUint32(0) >>> 20) & 0x7ff) - 53;
// |   }
// |   const newExp = currExp + exp;
// |
// |   if (newExp > 0x7fe) {
// |     return Infinity * sign;
// |   }
// |   if (newExp > 0) {
// |     setExponent(newExp);
// |     return view.getFloat64(0);
// |   }
// |   if (newExp >= -53) {
// |     setExponent(newExp + 53);
// |     return view.getFloat64(0) / TWO_POW_53;
// |   }
// |   return 0 * sign;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Replaces the biased exponent of the f64 held in `view`.
// |  * @param {number} biased
// |  */
// | function setExponent(biased) {
// |   view.setUint32(0, ((view.getUint32(0) & 0x800fffff) | (biased << 20)) >>> 0);
// | }
