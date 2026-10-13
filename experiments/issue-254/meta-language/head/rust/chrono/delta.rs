// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=4fdf0385a1929ed70731acf02c10469c8e80ab63c91af77dd92c785693884f93 bytes=16166

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * chrono 0.4.45 `TimeDelta` (alias `Duration`, src/time_delta.rs), `Months`
// |  * (src/month.rs) and the subset of `std::time::Duration` that
// |  * `TimeDelta::to_std` returns.
// |  *
// |  * `i64` values (`secs`, `num_seconds()`, ...) are bigints; `i32`/`u32` values
// |  * are numbers. Arguments typed `i64` accept a number or a bigint.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=ed0c33f0945575ce66bd32d6f90adaee2df0fac7a1ee6cba0c39326bfa9a4520
// | import { debugStruct, debugTuple } from '../fmt.js';
use crate::rust::fmt::{debugStruct, debugTuple};
// meta-language:translated JavaScript import_statement items=1 sha256=045406a9ac61ab96be5a2cc4ef3512bd77df5bdf62647ec698647e8dca73209b
// | import { err, expect, panic } from '../core.js';
use crate::rust::core_::{err, expect, panic};

// meta-language:translated JavaScript lexical_declaration items=1 sha256=ea8434aae58864aff5b02fd02b07bc746bcc109534a527fdd56e8bb0f7ca038e
// | const NANOS_PER_MICRO = 1000;
pub const NANOS_PER_MICRO: f64 = 1000f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=826c64f336f0eccb4c6f462f37230656042267517d342d566ba66d31487b06f6
// | const NANOS_PER_MILLI = 1_000_000;
pub const NANOS_PER_MILLI: f64 = 1000000f64;
// meta-language:translated JavaScript lexical_declaration items=1 sha256=24e699860d603060e952d8983ab05c9f27decbbf6edc773905f9bcb1fa97fed9
// | const NANOS_PER_SEC = 1_000_000_000;
pub const NANOS_PER_SEC: f64 = 1000000000f64;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const I64_MIN = -(2n ** 63n);
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const I64_MAX = 2n ** 63n - 1n;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const I32_MIN = -(2 ** 31);
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const I32_MAX = 2 ** 31 - 1;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const U32_MAX = 2 ** 32 - 1;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * An `i64` argument as a bigint.
// |  * @param {number | bigint} value
// |  * @returns {bigint}
// |  */
// | export function toI64(value) {
// |   return typeof value === 'bigint' ? value : BigInt(value);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `i64::checked_mul`.
// |  * @param {bigint} a
// |  * @param {bigint} b
// |  * @returns {bigint | null}
// |  */
// | function checkedMulI64(a, b) {
// |   const product = a * b;
// |   return product < I64_MIN || product > I64_MAX ? null : product;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `(this.div_euclid(other), this.rem_euclid(other))` for a positive divisor.
// |  * @param {bigint} value
// |  * @param {bigint} divisor
// |  * @returns {[bigint, bigint]}
// |  */
// | function divModFloor(value, divisor) {
// |   let rem = value % divisor;
// |   if (rem < 0n) {
// |     rem += divisor;
// |   }
// |   return [(value - rem) / divisor, rem];
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** `secs: -i64::MAX / 1000 - 1`, `nanos: 1e9 + (-i64::MAX % 1000) * 1e6`. */
// | const MIN_SECS = -I64_MAX / 1000n - 1n;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const MIN_NANOS = NANOS_PER_SEC + Number(-I64_MAX % 1000n) * NANOS_PER_MILLI;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** `secs: i64::MAX / 1000`, `nanos: (i64::MAX % 1000) * 1e6`. */
// | const MAX_SECS = I64_MAX / 1000n;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const MAX_NANOS = Number(I64_MAX % 1000n) * NANOS_PER_MILLI;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * chrono `TimeDelta`: a signed duration of `secs` seconds plus `nanos`
// |  * (0 <= nanos < 1e9) nanoseconds, so -1.5 s is `{ secs: -2, nanos: 5e8 }`.
// |  */
// | export class TimeDelta {
// |   /** @param {{ secs: bigint, nanos: number }} fields */
// |   constructor({ secs, nanos }) {
// |     /** @type {bigint} */
// |     this.secs = secs;
// |     /** @type {number} */
// |     this.nanos = nanos;
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * `TimeDelta::new(secs, nanos)`: `None` outside `MIN..=MAX` or for
// |    * `nanos >= 1e9`.
// |    * @param {number | bigint} secs
// |    * @param {number} nanos a `u32`
// |    * @returns {TimeDelta | null}
// |    */
// |   static new(secs, nanos) {
// |     const s = toI64(secs);
// |     if (s < MIN_SECS || s > MAX_SECS || nanos >= NANOS_PER_SEC
// |       || (s === MAX_SECS && nanos > MAX_NANOS) || (s === MIN_SECS && nanos < MIN_NANOS)) {
// |       return null;
// |     }
// |     return new TimeDelta({ secs: s, nanos });
// |   }
// |
// |   /**
// |    * `TimeDelta::try_weeks`.
// |    * @param {number | bigint} weeks
// |    * @returns {TimeDelta | null}
// |    */
// |   static tryWeeks(weeks) {
// |     const secs = checkedMulI64(toI64(weeks), 604_800n);
// |     return secs === null ? null : TimeDelta.trySeconds(secs);
// |   }
// |
// |   /**
// |    * `TimeDelta::weeks`; panics out of bounds.
// |    * @param {number | bigint} weeks
// |    */
// |   static weeks(weeks) {
// |     return expect(TimeDelta.tryWeeks(weeks), 'TimeDelta::weeks out of bounds');
// |   }
// |
// |   /**
// |    * `TimeDelta::try_days`.
// |    * @param {number | bigint} days
// |    * @returns {TimeDelta | null}
// |    */
// |   static tryDays(days) {
// |     const secs = checkedMulI64(toI64(days), 86_400n);
// |     return secs === null ? null : TimeDelta.trySeconds(secs);
// |   }
// |
// |   /**
// |    * `TimeDelta::days`; panics out of bounds.
// |    * @param {number | bigint} days
// |    */
// |   static days(days) {
// |     return expect(TimeDelta.tryDays(days), 'TimeDelta::days out of bounds');
// |   }
// |
// |   /**
// |    * `TimeDelta::try_hours`.
// |    * @param {number | bigint} hours
// |    * @returns {TimeDelta | null}
// |    */
// |   static tryHours(hours) {
// |     const secs = checkedMulI64(toI64(hours), 3600n);
// |     return secs === null ? null : TimeDelta.trySeconds(secs);
// |   }
// |
// |   /**
// |    * `TimeDelta::hours`; panics out of bounds.
// |    * @param {number | bigint} hours
// |    */
// |   static hours(hours) {
// |     return expect(TimeDelta.tryHours(hours), 'TimeDelta::hours out of bounds');
// |   }
// |
// |   /**
// |    * `TimeDelta::try_minutes`.
// |    * @param {number | bigint} minutes
// |    * @returns {TimeDelta | null}
// |    */
// |   static tryMinutes(minutes) {
// |     const secs = checkedMulI64(toI64(minutes), 60n);
// |     return secs === null ? null : TimeDelta.trySeconds(secs);
// |   }
// |
// |   /**
// |    * `TimeDelta::minutes`; panics out of bounds.
// |    * @param {number | bigint} minutes
// |    */
// |   static minutes(minutes) {
// |     return expect(TimeDelta.tryMinutes(minutes), 'TimeDelta::minutes out of bounds');
// |   }
// |
// |   /**
// |    * `TimeDelta::try_seconds`.
// |    * @param {number | bigint} seconds
// |    * @returns {TimeDelta | null}
// |    */
// |   static trySeconds(seconds) {
// |     return TimeDelta.new(seconds, 0);
// |   }
// |
// |   /**
// |    * `TimeDelta::seconds`; panics out of bounds.
// |    * @param {number | bigint} seconds
// |    */
// |   static seconds(seconds) {
// |     return expect(TimeDelta.trySeconds(seconds), 'TimeDelta::seconds out of bounds');
// |   }
// |
// |   /**
// |    * `TimeDelta::try_milliseconds`.
// |    * @param {number | bigint} milliseconds
// |    * @returns {TimeDelta | null}
// |    */
// |   static tryMilliseconds(milliseconds) {
// |     const ms = toI64(milliseconds);
// |     if (ms < -I64_MAX) {
// |       return null;
// |     }
// |     const [secs, millis] = divModFloor(ms, 1000n);
// |     return new TimeDelta({ secs, nanos: Number(millis) * NANOS_PER_MILLI });
// |   }
// |
// |   /**
// |    * `TimeDelta::milliseconds`; panics out of bounds.
// |    * @param {number | bigint} milliseconds
// |    */
// |   static milliseconds(milliseconds) {
// |     return expect(TimeDelta.tryMilliseconds(milliseconds), 'TimeDelta::milliseconds out of bounds');
// |   }
// |
// |   /**
// |    * `TimeDelta::microseconds`.
// |    * @param {number | bigint} microseconds
// |    */
// |   static microseconds(microseconds) {
// |     const [secs, micros] = divModFloor(toI64(microseconds), 1_000_000n);
// |     return new TimeDelta({ secs, nanos: Number(micros) * NANOS_PER_MICRO });
// |   }
// |
// |   /**
// |    * `TimeDelta::nanoseconds`.
// |    * @param {number | bigint} nanoseconds
// |    */
// |   static nanoseconds(nanoseconds) {
// |     const [secs, nanos] = divModFloor(toI64(nanoseconds), 1_000_000_000n);
// |     return new TimeDelta({ secs, nanos: Number(nanos) });
// |   }
// |
// |   /** `TimeDelta::zero`. */
// |   static zero() {
// |     return new TimeDelta({ secs: 0n, nanos: 0 });
// |   }
// |
// |   /** `TimeDelta::default` (zero). */
// |   static default() {
// |     return TimeDelta.zero();
// |   }
// |
// |   /** `TimeDelta::MIN` (-i64::MAX milliseconds). */
// |   static get MIN() {
// |     return new TimeDelta({ secs: MIN_SECS, nanos: MIN_NANOS });
// |   }
// |
// |   /** `TimeDelta::MAX` (i64::MAX milliseconds). */
// |   static get MAX() {
// |     return new TimeDelta({ secs: MAX_SECS, nanos: MAX_NANOS });
// |   }
// |
// |   /**
// |    * `TimeDelta::from_std`; throws `Err(OutOfRangeError)`.
// |    * @param {StdDuration} duration
// |    * @returns {TimeDelta}
// |    */
// |   static fromStd(duration) {
// |     if (duration.secs > MAX_SECS) {
// |       return err(new OutOfRangeError());
// |     }
// |     const delta = TimeDelta.new(duration.secs, duration.nanos);
// |     return delta === null ? err(new OutOfRangeError()) : delta;
// |   }
// |
// |   /** `num_weeks`. @returns {bigint} */
// |   numWeeks() {
// |     return this.numDays() / 7n;
// |   }
// |
// |   /** `num_days`. @returns {bigint} */
// |   numDays() {
// |     return this.numSeconds() / 86_400n;
// |   }
// |
// |   /** `num_hours`. @returns {bigint} */
// |   numHours() {
// |     return this.numSeconds() / 3600n;
// |   }
// |
// |   /** `num_minutes`. @returns {bigint} */
// |   numMinutes() {
// |     return this.numSeconds() / 60n;
// |   }
// |
// |   /** `num_seconds` (truncated toward zero). @returns {bigint} */
// |   numSeconds() {
// |     return this.secs < 0n && this.nanos > 0 ? this.secs + 1n : this.secs;
// |   }
// |
// |   /** `subsec_nanos`: the signed nanoseconds `numSeconds()` leaves. */
// |   subsecNanos() {
// |     return this.secs < 0n && this.nanos > 0 ? this.nanos - NANOS_PER_SEC : this.nanos;
// |   }
// |
// |   /** `subsec_millis`. */
// |   subsecMillis() {
// |     return Math.trunc(this.subsecNanos() / NANOS_PER_MILLI);
// |   }
// |
// |   /** `num_milliseconds`. @returns {bigint} */
// |   numMilliseconds() {
// |     return this.numSeconds() * 1000n + BigInt(Math.trunc(this.subsecNanos() / NANOS_PER_MILLI));
// |   }
// |
// |   /** `num_microseconds`. @returns {bigint | null} */
// |   numMicroseconds() {
// |     const secsPart = checkedMulI64(this.numSeconds(), 1_000_000n);
// |     if (secsPart === null) {
// |       return null;
// |     }
// |     const total = secsPart + BigInt(Math.trunc(this.subsecNanos() / NANOS_PER_MICRO));
// |     return total < I64_MIN || total > I64_MAX ? null : total;
// |   }
// |
// |   /** `num_nanoseconds`. @returns {bigint | null} */
// |   numNanoseconds() {
// |     const secsPart = checkedMulI64(this.numSeconds(), 1_000_000_000n);
// |     if (secsPart === null) {
// |       return null;
// |     }
// |     const total = secsPart + BigInt(this.subsecNanos());
// |     return total < I64_MIN || total > I64_MAX ? null : total;
// |   }
// |
// |   /** `as_seconds_f64`. */
// |   asSecondsF64() {
// |     return Number(this.secs) + this.nanos / NANOS_PER_SEC;
// |   }
// |
// |   /**
// |    * `checked_add`.
// |    * @param {TimeDelta} rhs
// |    * @returns {TimeDelta | null}
// |    */
// |   checkedAdd(rhs) {
// |     let secs = this.secs + rhs.secs;
// |     let nanos = this.nanos + rhs.nanos;
// |     if (nanos >= NANOS_PER_SEC) {
// |       nanos -= NANOS_PER_SEC;
// |       secs += 1n;
// |     }
// |     return TimeDelta.new(secs, nanos);
// |   }
// |
// |   /**
// |    * `checked_sub`.
// |    * @param {TimeDelta} rhs
// |    * @returns {TimeDelta | null}
// |    */
// |   checkedSub(rhs) {
// |     let secs = this.secs - rhs.secs;
// |     let nanos = this.nanos - rhs.nanos;
// |     if (nanos < 0) {
// |       nanos += NANOS_PER_SEC;
// |       secs -= 1n;
// |     }
// |     return TimeDelta.new(secs, nanos);
// |   }
// |
// |   /**
// |    * `checked_mul(i32)`. Like chrono it only rejects results outside the
// |    * `i64` range of seconds, not outside `MIN..=MAX`.
// |    * @param {number} rhs an `i32`
// |    * @returns {TimeDelta | null}
// |    */
// |   checkedMul(rhs) {
// |     const factor = BigInt(rhs);
// |     const [extraSecs, nanos] = divModFloor(BigInt(this.nanos) * factor, 1_000_000_000n);
// |     const secs = this.secs * factor + extraSecs;
// |     if (secs <= I64_MIN || secs >= I64_MAX) {
// |       return null;
// |     }
// |     return new TimeDelta({ secs, nanos: Number(nanos) });
// |   }
// |
// |   /**
// |    * `checked_div(i32)`: `None` when dividing by zero.
// |    * @param {number} rhs an `i32`
// |    * @returns {TimeDelta | null}
// |    */
// |   checkedDiv(rhs) {
// |     if (rhs === 0) {
// |       return null;
// |     }
// |     const divisor = BigInt(rhs);
// |     let secs = this.secs / divisor;
// |     const carry = this.secs % divisor;
// |     const extraNanos = Number((carry * 1_000_000_000n) / divisor);
// |     let nanos = Math.trunc(this.nanos / rhs) + extraNanos;
// |     if (nanos < 0) {
// |       secs -= 1n;
// |       nanos += NANOS_PER_SEC;
// |     } else if (nanos >= NANOS_PER_SEC) {
// |       secs += 1n;
// |       nanos -= NANOS_PER_SEC;
// |     }
// |     return new TimeDelta({ secs, nanos });
// |   }
// |
// |   /**
// |    * `TimeDelta + TimeDelta`; panics on overflow.
// |    * @param {TimeDelta} rhs
// |    */
// |   add(rhs) {
// |     return expect(this.checkedAdd(rhs), '`TimeDelta + TimeDelta` overflowed');
// |   }
// |
// |   /**
// |    * `TimeDelta - TimeDelta`; panics on overflow.
// |    * @param {TimeDelta} rhs
// |    */
// |   sub(rhs) {
// |     return expect(this.checkedSub(rhs), '`TimeDelta - TimeDelta` overflowed');
// |   }
// |
// |   /**
// |    * `TimeDelta * i32`; panics on overflow.
// |    * @param {number} rhs
// |    */
// |   mul(rhs) {
// |     return expect(this.checkedMul(rhs), '`TimeDelta * i32` overflowed');
// |   }
// |
// |   /**
// |    * `TimeDelta / i32`; panics when dividing by zero.
// |    * @param {number} rhs
// |    */
// |   div(rhs) {
// |     return expect(this.checkedDiv(rhs), '`i32` is zero');
// |   }
// |
// |   /** `-delta`. */
// |   neg() {
// |     if (this.nanos === 0) {
// |       return new TimeDelta({ secs: -this.secs, nanos: 0 });
// |     }
// |     return new TimeDelta({ secs: -this.secs - 1n, nanos: NANOS_PER_SEC - this.nanos });
// |   }
// |
// |   /** `abs`. */
// |   abs() {
// |     if (this.secs < 0n && this.nanos !== 0) {
// |       const secs = this.secs + 1n;
// |       return new TimeDelta({ secs: secs < 0n ? -secs : secs, nanos: NANOS_PER_SEC - this.nanos });
// |     }
// |     return new TimeDelta({ secs: this.secs < 0n ? -this.secs : this.secs, nanos: this.nanos });
// |   }
// |
// |   /** `is_zero`. */
// |   isZero() {
// |     return this.secs === 0n && this.nanos === 0;
// |   }
// |
// |   /**
// |    * `to_std`; throws `Err(OutOfRangeError)` for a negative delta.
// |    * @returns {StdDuration}
// |    */
// |   toStd() {
// |     if (this.secs < 0n) {
// |       return err(new OutOfRangeError());
// |     }
// |     return new StdDuration({ secs: this.secs, nanos: this.nanos });
// |   }
// |
// |   /**
// |    * Derived `PartialEq`.
// |    * @param {TimeDelta} other
// |    */
// |   equals(other) {
// |     return other instanceof TimeDelta && this.secs === other.secs && this.nanos === other.nanos;
// |   }
// |
// |   /**
// |    * Derived `Ord` (on `secs`, then `nanos`).
// |    * @param {TimeDelta} other
// |    * @returns {-1 | 0 | 1}
// |    */
// |   cmp(other) {
// |     if (this.secs !== other.secs) {
// |       return this.secs < other.secs ? -1 : 1;
// |     }
// |     if (this.nanos !== other.nanos) {
// |       return this.nanos < other.nanos ? -1 : 1;
// |     }
// |     return 0;
// |   }
// |
// |   /** `Display`: the ISO 8601 duration, e.g. `PT1.5S`, `-PT86400S`, `P0D`. */
// |   toString() {
// |     const [abs, sign] = this.secs < 0n ? [this.neg(), '-'] : [this, ''];
// |     if (abs.secs === 0n && abs.nanos === 0) {
// |       return `${sign}P0D`;
// |     }
// |     let text = `${sign}PT${abs.secs}`;
// |     if (abs.nanos > 0) {
// |       text += `.${String(abs.nanos).padStart(9, '0').replace(/0+$/, '')}`;
// |     }
// |     return `${text}S`;
// |   }
// |
// |   /** Derived `Debug`: `TimeDelta { secs: 1, nanos: 0 }`. */
// |   fmtDebug() {
// |     return debugStruct('TimeDelta', [['secs', String(this.secs)], ['nanos', String(this.nanos)]]);
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** `chrono::Duration`, an alias of `TimeDelta`. */
// | export const Duration = TimeDelta;

// meta-language:carried JavaScript export_statement (unsupported)
// | /** `chrono::OutOfRangeError`, returned by `TimeDelta::to_std`/`from_std`. */
// | export class OutOfRangeError {
// |   constructor() {
// |     Object.freeze(this);
// |   }
// |
// |   /** `Display`. */
// |   toString() {
// |     return 'Source duration value is out of range for the target type';
// |   }
// |
// |   /** Derived `Debug`. */
// |   fmtDebug() {
// |     return 'OutOfRangeError(())';
// |   }
// |
// |   /**
// |    * Derived `PartialEq`.
// |    * @param {unknown} other
// |    */
// |   equals(other) {
// |     return other instanceof OutOfRangeError;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * The subset of `std::time::Duration` that `TimeDelta::to_std` produces:
// |  * `secs` is a `u64` bigint, `nanos` a `u32` below 1e9.
// |  */
// | export class StdDuration {
// |   /** @param {{ secs: bigint, nanos: number }} fields */
// |   constructor({ secs, nanos }) {
// |     /** @type {bigint} */
// |     this.secs = secs;
// |     /** @type {number} */
// |     this.nanos = nanos;
// |     Object.freeze(this);
// |   }
// |
// |   /** `Duration::default()` (zero). */
// |   static default() {
// |     return new StdDuration({ secs: 0n, nanos: 0 });
// |   }
// |
// |   /** `as_secs`. @returns {bigint} */
// |   asSecs() {
// |     return this.secs;
// |   }
// |
// |   /** `subsec_nanos`. */
// |   subsecNanos() {
// |     return this.nanos;
// |   }
// |
// |   /** `as_millis` (a `u128`). @returns {bigint} */
// |   asMillis() {
// |     return this.secs * 1000n + BigInt(Math.trunc(this.nanos / NANOS_PER_MILLI));
// |   }
// |
// |   /** `as_secs_f64`. */
// |   asSecsF64() {
// |     return Number(this.secs) + this.nanos / NANOS_PER_SEC;
// |   }
// |
// |   /** `is_zero`. */
// |   isZero() {
// |     return this.secs === 0n && this.nanos === 0;
// |   }
// |
// |   /**
// |    * Derived `PartialEq`.
// |    * @param {StdDuration} other
// |    */
// |   equals(other) {
// |     return other instanceof StdDuration && this.secs === other.secs && this.nanos === other.nanos;
// |   }
// |
// |   /**
// |    * Derived `Ord`.
// |    * @param {StdDuration} other
// |    * @returns {-1 | 0 | 1}
// |    */
// |   cmp(other) {
// |     if (this.secs !== other.secs) {
// |       return this.secs < other.secs ? -1 : 1;
// |     }
// |     if (this.nanos !== other.nanos) {
// |       return this.nanos < other.nanos ? -1 : 1;
// |     }
// |     return 0;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** chrono `Months(u32)` (src/month.rs). */
// | export class Months {
// |   /** @param {number} months a `u32` */
// |   constructor(months) {
// |     if (!Number.isInteger(months) || months < 0 || months > U32_MAX) {
// |       panic(`Months: ${months} is not a u32`);
// |     }
// |     /** @type {number} */
// |     this._0 = months;
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * `Months::new`.
// |    * @param {number} months a `u32`
// |    */
// |   static new(months) {
// |     return new Months(months);
// |   }
// |
// |   /** `as_u32`. */
// |   asU32() {
// |     return this._0;
// |   }
// |
// |   /**
// |    * Derived `PartialEq`.
// |    * @param {Months} other
// |    */
// |   equals(other) {
// |     return other instanceof Months && this._0 === other._0;
// |   }
// |
// |   /** Derived `Debug`: `Months(3)`. */
// |   fmtDebug() {
// |     return debugTuple('Months', [String(this._0)]);
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | export { I32_MAX, I32_MIN, NANOS_PER_SEC };
