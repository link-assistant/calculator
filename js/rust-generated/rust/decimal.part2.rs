// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `rem_impl` (ops/rem.rs): the dividend's sign, and the larger of the two
// |  * scales. A dividend with the smaller scale is scaled up in 96 bits (as far
// |  * as `find_scale` allows) or in `rem_full`'s wider buffer; the limb
// |  * arithmetic, wrapping included, is reproduced.
// |  * @param {RustDecimal} d1 @param {RustDecimal} d2
// |  * @returns {RustDecimal | string}
// |  */
// | function rem(d1, d2) {
// |   if (d2.magnitude === 0n) return DIV_BY_ZERO;
// |   if (d1.magnitude === 0n) return RustDecimal.ZERO;
// |   // `cmp_internal` ignores the signs and is exact.
// |   const top = Math.max(d1.places, d2.places);
// |   const a = d1.magnitude * POW10[top - d1.places];
// |   const b = d2.magnitude * POW10[top - d2.places];
// |   if (a === b) return RustDecimal.ZERO;
// |   if (a < b) return d1;
// |   let x = d1.magnitude;
// |   let xScale = d1.places;
// |   let y = d2.magnitude;
// |   let scale = d1.places - d2.places;
// |   if (scale > 0) {
// |     // The divisor is scaled up with a wrapping u64 multiply (mod 2^96).
// |     y = (y * POW10[scale]) & U96_MASK;
// |     scale = 0;
// |   }
// |   for (;;) {
// |     if (scale < 0) {
// |       for (;;) {
// |         const powerScale = findScale(x, MAX_SCALE + scale);
// |         if (powerScale === null) return OVERFLOW;
// |         if (powerScale === 0) break;
// |         scale += powerScale;
// |         x = (x * POW10[powerScale]) & U96_MASK;
// |         if (powerScale !== 9 || scale >= 0) break;
// |       }
// |       xScale = d2.places;
// |     }
// |     if (x <= U64_MASK) {
// |       const y64 = y & U64_MASK;
// |       if (y64 === 0n) return panic('attempt to calculate the remainder with a divisor of zero');
// |       return fromParts(x % y64, d1.negative, xScale);
// |     }
// |     if (y >= U32) return remFull(x, xScale, d1.negative, y, scale);
// |     x = ((((x >> 32n) % y) << 32n) | (x & M32)) % y;
// |     if (scale >= 0) return fromParts(x, d1.negative, xScale);
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `rem_full` (ops/rem.rs), for a divisor `y` wider than 32 bits: the dividend
// |  * `x` is shifted by the divisor's leading zeros, scaled by `10^-scale` in a
// |  * 192-bit buffer plus an overflow word, then reduced by `partial_divide_64`
// |  * or `partial_divide_96` steps. The result has `xScale` and `negative`.
// |  * @param {bigint} x @param {number} xScale @param {boolean} negative @param {bigint} y @param {number} scale
// |  */
// | function remFull(x, xScale, negative, y, scale) {
// |   const yHi = y >> 64n;
// |   const shift = BigInt(Math.clz32(Number(yHi === 0n ? y >> 32n : yHi)));
// |   const low64 = (x << shift) & U64_MASK;
// |   const mid64 = x >> 32n >> (32n - shift);
// |   // The `Buf24` words, then the `overflow` word.
// |   const buf = [low64 & M32, low64 >> 32n, mid64 & M32, mid64 >> 32n, 0n, 0n, 0n];
// |   let upper = 3;
// |   while (scale < 0) {
// |     const power = POW10[Math.min(-scale, 9)];
// |     const last = upper === 6 ? 6 : Math.min(upper, 5);
// |     let t = 0n;
// |     for (let i = 0; i <= last; i++) buf[i] = (t = (t >> 32n) + buf[i] * power) & M32;
// |     // Keeps the top word's high bit clear.
// |     if (t > 0x7fffffffn) {
// |       upper += 1;
// |       buf[Math.min(upper, 6)] = t >> 32n;
// |     }
// |     scale += 9;
// |   }
// |   const divisor = (y << shift) & U64_MASK;
// |   const start = upper >= 4 && upper <= 6 ? upper - 2 : 1;
// |   if (yHi === 0n) {
// |     for (let at = start; at >= 0; at--) partialDivide64(buf, at, divisor);
// |     return fromParts((buf[0] | (buf[1] << 32n)) >> shift, negative, xScale);
// |   }
// |   const words = [divisor & M32, divisor >> 32n, (y >> 32n >> (32n - shift)) & M32];
// |   for (let at = start - 1; at >= 0; at--) partialDivide96(buf, at, words);
// |   const low = ((buf[0] | (buf[1] << 32n)) >> shift) + ((buf[2] << (32n - shift) << 32n) & U64_MASK);
// |   return fromParts(low + ((buf[2] >> shift) << 64n), negative, xScale);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Buf16::partial_divide_64` (ops/div.rs) on the words `b[at..at + 3]`: the
// |  * remainder by a normalized 64-bit divisor replaces the low two words (the
// |  * quotient is not needed).
// |  * @param {bigint[]} b @param {number} at @param {bigint} divisor
// |  */
// | function partialDivide64(b, at, divisor) {
// |   const low64 = b[at] | (b[at + 1] << 32n);
// |   const divisorHi = divisor >> 32n;
// |   let r;
// |   if (b[at + 2] === 0n) {
// |     if (low64 < divisor) return;
// |     r = low64 % divisor;
// |   } else if (b[at + 2] >= divisorHi) {
// |     r = (low64 - ((divisor << 32n) & U64_MASK) + divisor) & U64_MASK;
// |     while (r >= divisor) r = (r + divisor) & U64_MASK;
// |   } else {
// |     const mid64 = b[at + 1] | (b[at + 2] << 32n);
// |     if (mid64 < divisorHi) return;
// |     const q = mid64 / divisorHi;
// |     const product = q * (divisor & M32);
// |     r = ((b[at] | ((mid64 - q * divisorHi) << 32n)) - product) & U64_MASK;
// |     if (r > U64_MASK - product) do r = (r + divisor) & U64_MASK; while (r >= divisor);
// |   }
// |   b[at] = r & M32;
// |   b[at + 1] = r >> 32n;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Buf16::partial_divide_96` (ops/div.rs) on the words `b[at..at + 4]`: the
// |  * remainder by a normalized 96-bit divisor (`words`) replaces the low three
// |  * words. The estimated quotient digit is truncated to 32 bits, as in Rust.
// |  * @param {bigint[]} b @param {number} at @param {bigint[]} words
// |  */
// | function partialDivide96(b, at, words) {
// |   const dividend = b[at + 2] | (b[at + 3] << 32n);
// |   const divisorHi = words[2];
// |   if (dividend < divisorHi) return;
// |   const quo = (dividend / divisorHi) & M32;
// |   let r = ((dividend & M32) - quo * divisorHi) & M32;
// |   let prod1 = quo * words[0];
// |   let prod2 = quo * words[1] + (prod1 >> 32n);
// |   prod1 = (prod1 & M32) | ((prod2 << 32n) & U64_MASK);
// |   prod2 >>= 32n;
// |   let num = ((b[at] | (b[at + 1] << 32n)) - prod1) & U64_MASK;
// |   r = (r - prod2) & M32;
// |   let settled;
// |   if (num > U64_MASK - prod1) {
// |     r = (r - 1n) & M32;
// |     settled = r < M32 - prod2;
// |   } else {
// |     settled = r <= M32 - prod2;
// |   }
// |   // The remainder went negative: add the divisor back until it carries.
// |   const divisorLow = words[0] | (words[1] << 32n);
// |   while (!settled) {
// |     num = (num + divisorLow) & U64_MASK;
// |     r = (r + divisorHi) & M32;
// |     if (num < divisorLow) {
// |       const before = r;
// |       r = (r + 1n) & M32;
// |       if (before < divisorHi) break;
// |     }
// |     settled = r < divisorHi;
// |   }
// |   b[at] = num & M32;
// |   b[at + 1] = num >> 32n;
// |   b[at + 2] = r;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `Decimal::rescale` (ops/array.rs `rescale`): rounds half away from zero
// |  * when dropping digits; keeps the sign flag.
// |  * @param {RustDecimal} d @param {number} newScale
// |  */
// | function rescale(d, newScale) {
// |   if (d.places === newScale) return d;
// |   let m = d.magnitude;
// |   if (m === 0n) return new RustDecimal(0n, Math.min(newScale, MAX_SCALE), d.negative);
// |   if (d.places > newScale) {
// |     m /= POW10[d.places - newScale - 1];
// |     if (m !== 0n) {
// |       const digit = m % 10n;
// |       m /= 10n;
// |       if (digit >= 5n) m += 1n;
// |     }
// |     return new RustDecimal(m, newScale, d.negative);
// |   }
// |   let remaining = newScale - d.places;
// |   for (; remaining > 0 && m * 10n < U96; remaining--) m *= 10n;
// |   return new RustDecimal(m, newScale - remaining, d.negative);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** `Decimal::set_scale`: keeps the mantissa and the sign flag. @param {RustDecimal} d @param {number} scale */
// | function setScale(d, scale) {
// |   if (scale > MAX_SCALE) err(DecimalError.ScaleExceedsMaximumPrecision(scale));
// |   return new RustDecimal(d.magnitude, scale, d.negative);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `str.parse::<u32>()`: an optional '+', then at least one ASCII digit.
// |  * @param {string} s
// |  * @returns {number | null}
// |  */
// | function parseU32(s) {
// |   const digits = s.startsWith('+') ? s.slice(1) : s;
// |   if (!/^[0-9]+$/.test(digits)) return null;
// |   const value = Number(digits);
// |   return value <= 4294967295 ? value : null;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `from_scientific` and `from_scientific_lossy` (decimal.rs).
// |  * @param {string} value @param {boolean} lossy
// |  * @returns {RustDecimal}
// |  */
// | function fromScientific(value, lossy) {
// |   const split = value.search(/[eE]/);
// |   if (split < 0) return err(DecimalError.ErrorString('Failed to parse'));
// |   let ret = RustDecimal.fromStr(value.slice(0, split));
// |   const exp = value.slice(split + 1);
// |   const currentScale = ret.places;
// |   const negativeExp = exp.startsWith('-');
// |   const e = parseU32(negativeExp ? exp.slice(1) : exp);
// |   if (e === null) return err(DecimalError.ErrorString('Failed to parse'));
// |   if (negativeExp) {
// |     if (e > MAX_SCALE) err(DecimalError.ScaleExceedsMaximumPrecision(e));
// |     if (lossy && currentScale + e > MAX_SCALE) {
// |       return setScale(rescale(ret, MAX_SCALE - e), MAX_SCALE);
// |     }
// |     return setScale(ret, currentScale + e);
// |   }
// |   if (e <= currentScale) return setScale(ret, currentScale - e);
// |   if (e > MAX_SCALE) err(DecimalError.ScaleExceedsMaximumPrecision(e));
// |   let rest = e;
// |   while (rest > 0) {
// |     const step = Math.min(rest, 19);
// |     rest -= step;
// |     const r = mul(ret, new RustDecimal(POW10[step], 0, false));
// |     if (!(r instanceof RustDecimal)) err(DecimalError.ExceedsMaximumPossibleValue);
// |     ret = /** @type {RustDecimal} */ (r);
// |   }
// |   return ret.normalize();
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const UTF8 = new TextEncoder();
// meta-language:translated JavaScript lexical_declaration items=1 sha256=17c7203f94adbe768523c7b70daa2208b8a37cefcfff925559491affdafaadf1
// | const DOT = 0x2e;
pub const DOT: f64 = 46f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=ed9d163509979d0e7a2227e9ee194cff2190e926b67524a95950c8503206994c
// | const UNDERSCORE = 0x5f;
pub const UNDERSCORE: f64 = 95f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=b04f022f08eb64d49266a1b875558663310e18ac7479d61ce48290f0e3b9de4d
// | const MINUS = 0x2d;
pub const MINUS: f64 = 45f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=e6647f10d3bdf58683a17572e441f3d09483c1c37f93aff88d8a49f9900db30f
// | const PLUS = 0x2b;
pub const PLUS: f64 = 43f64;

// meta-language:translated JavaScript function_declaration items=1 sha256=eb0fb3dd538ac7673e35527149bf17d0675176f7630ad85c02d4bd3dd1496f32
// | /** @param {number} b */
// | function isDigit(b) {
// |   return b >= 0x30 && b <= 0x39;
// | }
pub fn is_digit(b: f64) -> bool {
    ((b >= (48f64)) && (b <= (57f64)))
}

// meta-language:carried JavaScript function_declaration (type)
// | /** @param {string} message */
// | function parseError(message) {
// |   return DecimalError.ErrorString(`Invalid decimal: ${message}`);
// | }

// meta-language:carried JavaScript function_declaration (type)
// | /** `tail_invalid_digit` (str.rs). @param {number} b */
// | function tailInvalidDigit(b) {
// |   if (b === DOT) return parseError('two decimal points');
// |   if (b === UNDERSCORE) return parseError('must start lead with a number');
// |   return parseError('unknown character');
// | }

// meta-language:carried JavaScript function_declaration (type)
// | /**
// |  * `handle_data` (str.rs).
// |  * @param {bigint} data @param {number} scale @param {boolean} negative @param {boolean} hasDigits
// |  */
// | function handleData(data, scale, negative, hasDigits) {
// |   if (!hasDigits) return parseError('no digits found');
// |   return fromParts(data, negative, scale);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `maybe_round` (str.rs): rounds half up on `bytes[at]`, then checks that
// |  * the rest of the text is only digits, underscores and at most one point.
// |  * @param {bigint} data @param {Uint8Array} bytes @param {number} at
// |  * @param {number} scale @param {boolean} point @param {boolean} negative
// |  */
// | function maybeRound(data, bytes, at, scale, point, negative) {
// |   const next = bytes[at];
// |   let digit;
// |   if (isDigit(next)) digit = next - 0x30;
// |   else if (next === UNDERSCORE || (next === DOT && !point)) digit = 0;
// |   else return tailInvalidDigit(next);
// |   if (digit >= 5) {
// |     data += 1n;
// |     if (data >= U96) {
// |       if (scale === 0) return parseError('overflow from mantissa after rounding');
// |       data = (data + 4n) / 10n;
// |       scale -= 1;
// |     }
// |   }
// |   let seenDot = point || next === DOT;
// |   for (let i = at + 1; i < bytes.length; i++) {
// |     const b = bytes[i];
// |     if (isDigit(b) || b === UNDERSCORE) continue;
// |     if (b === DOT && !seenDot) seenDot = true;
// |     else if (b === DOT) return parseError('two decimal points');
// |     else return tailInvalidDigit(b);
// |   }
// |   return handleData(data, scale, negative, true);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `handle_full_128` (str.rs): the digits past `WILL_OVERFLOW_U64`, from
// |  * `bytes[at]` on, accumulated up to 96 bits.
// |  * @param {bigint} data @param {Uint8Array} bytes @param {number} at
// |  * @param {number} scale @param {boolean} point @param {boolean} negative
// |  */
// | function handleFull128(data, bytes, at, scale, point, negative) {
// |   const n = bytes.length;
// |   for (let i = at; ; i++) {
// |     const b = bytes[i];
// |     if (isDigit(b)) {
// |       const next = data * 10n + BigInt(b - 0x30);
// |       if (next >= U96) {
// |         if (!point) return parseError('overflow from too many digits');
// |         return maybeRound(data, bytes, i, scale, point, negative);
// |       }
// |       data = next;
// |       if (point) scale += 1;
// |       if (i + 1 >= n) return handleData(data, scale, negative, true);
// |       if (point && scale >= MAX_SCALE) {
// |         let j = i + 1;
// |         if (bytes[j] === UNDERSCORE) {
// |           while (j < n && bytes[j] === UNDERSCORE) j++;
// |           if (j >= n) return handleData(data, scale, negative, true);
// |         }
// |         return maybeRound(data, bytes, j, scale, point, negative);
// |       }
// |       continue;
// |     }
// |     if (b === DOT && !point) point = true;
// |     else if (b !== UNDERSCORE) return tailInvalidDigit(b);
// |     if (i + 1 >= n) return handleData(data, scale, negative, true);
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `parse_str_radix_10` (str.rs) without the scientific fallback: the
// |  * `dispatch_next` state machine on 64 bits, then `handle_full_128` once
// |  * the value may overflow a `u64` (only for texts of 18 bytes or more).
// |  * @param {string} s
// |  * @returns {RustDecimal | DecimalError}
// |  */
// | function parseStrRadix10(s) {
// |   const bytes = UTF8.encode(s);
// |   const n = bytes.length;
// |   if (n === 0) return parseError('empty');
// |   const big = n >= BYTES_TO_OVERFLOW_U64;
// |   let point = false;
// |   let negative = false;
// |   let hasDigits = false;
// |   let data = 0n;
// |   let scale = 0;
// |   for (let i = 0; ; i++) {
// |     const b = bytes[i];
// |     if (isDigit(b)) {
// |       data = data * 10n + BigInt(b - 0x30);
// |       scale = point ? scale + 1 : 0;
// |       hasDigits = true;
// |       if (i + 1 >= n) return handleData(data, scale, negative, true);
// |       if (point && big && scale >= MAX_SCALE) return maybeRound(data, bytes, i + 1, scale, point, negative);
// |       if (big && data >= WILL_OVERFLOW_U64) return handleFull128(data, bytes, i + 1, scale, point, negative);
// |       continue;
// |     }
// |     if (b === DOT && !point) point = true;
// |     else if ((b === MINUS || b === PLUS) && i === 0) negative = b === MINUS;
// |     else if (!(b === UNDERSCORE && hasDigits)) return tailInvalidDigit(b);
// |     if (i + 1 >= n) return handleData(data, scale, negative, hasDigits);
// |   }
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const F64_VIEW = new DataView(new ArrayBuffer(8));

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `from_f64` with `remove_excess_bits = true` and `base2_to_decimal`
// |  * (decimal.rs): the exact binary value is converted to a 96-bit mantissa
// |  * times a power of ten, then trimmed to at most 28 places and to the 53
// |  * significant bits of the float.
// |  * @param {number} n
// |  * @returns {RustDecimal | null}
// |  */
// | function fromF64(n) {
// |   if (!Number.isFinite(n)) return null;
// |   F64_VIEW.setFloat64(0, n);
// |   const raw = F64_VIEW.getBigUint64(0);
// |   const positive = raw >> 63n === 0n;
// |   const biasedExponent = Number((raw >> 52n) & 0x7ffn);
// |   let bits = raw & 0xfffffffffffffn;
// |   if (biasedExponent === 0 && bits === 0n) return positive ? RustDecimal.ZERO : new RustDecimal(0n, 0, true);
// |   let exponent2 = biasedExponent - 1023;
// |   if (biasedExponent === 0) exponent2 += 1;
// |   else bits |= 1n << 52n;
// |   exponent2 -= 52;
// |
// |   let exponent5 = -exponent2;
// |   let exponent10 = exponent2;
// |   // An odd mantissa is multiplied by 5 unless that overflows; otherwise halve.
// |   for (; exponent5 > 0; exponent5--) {
// |     if ((bits & 1n) === 0n || bits * 5n >= U96) {
// |       exponent10 += 1;
// |       bits >>= 1n;
// |     } else {
// |       bits *= 5n;
// |     }
// |   }
// |   for (; exponent5 < 0; exponent5++) {
// |     if (bits < 1n << 95n) {
// |       exponent10 -= 1;
// |       bits <<= 1n;
// |     } else if (exponent10 * 2 > -exponent5) {
// |       return null;
// |     } else {
// |       bits /= 5n;
// |     }
// |   }
// |   for (; exponent10 > 0; exponent10--) if ((bits *= 10n) >= U96) return null;
// |   // Drop digits, rounding half up, to at most 28 places (a zero mantissa
// |   // resets the exponent), then to the float's 53 bits.
// |   while (exponent10 < -MAX_SCALE || (exponent10 < 0 && bits >= 1n << 52n)) {
// |     const digit = bits % 10n;
// |     bits /= 10n;
// |     exponent10 += 1;
// |     if (bits === 0n) exponent10 = 0;
// |     else if (digit >= 5n) bits += 1n;
// |   }
// |   for (; exponent10 < 0 && bits % 10n === 0n; exponent10++) bits /= 10n;
// |   return new RustDecimal(bits, -exponent10, !positive);
// | }
