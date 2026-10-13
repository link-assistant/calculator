// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=3ad46e51f16c2f08621b4a2c2a326d0a0af9f5a009e854f9aa2851692d2b3864 bytes=8277

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * chrono 0.4.45 `NaiveTime` (src/naive/time/mod.rs).
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=0722404aebba2d1feb797ebe8c6db6bbd16639545c83e2e55f10440d15f6dda2
// | import { expect } from '../core.js';
use crate::rust::core_::expect;
// meta-language:translated JavaScript import_statement items=1 sha256=f4e2b548428a5ec66da16b6e638e15fb757964dc24b9d0500169131afdbefd2a
// | import { NANOS_PER_SEC, TimeDelta } from './delta.js';
use crate::rust::chrono::delta::{NANOS_PER_SEC, TimeDelta};
// meta-language:translated JavaScript import_statement items=1 sha256=000b26919872bdff906c5f75a867d10f081546a6a76b667b722e2291d71673e7
// | import { formatToString } from './formatting.js';
use crate::rust::chrono::formatting::formatToString;
// meta-language:translated JavaScript import_statement items=1 sha256=c7768fe2fcab274580080d6a04db0d925d2273becb91c78460075ec3aad01a2b
// | import { Parsed, parse } from './parse.js';
use crate::rust::chrono::parse::{Parsed, parse};
// meta-language:translated JavaScript import_statement items=1 sha256=860482e701847c72b98b1ed244ed02b720c2c8f9ea10bc6d91ae354229e4fb37
// | import { StrftimeItems } from './strftime.js';
use crate::rust::chrono::strftime::StrftimeItems;
// meta-language:translated JavaScript import_statement items=1 sha256=2c2bf693adc4b27a334e13a7cc10b5e4fe3221b26e5370e32c6f1ff9409d1a58
// | import { U32_MAX, autoSiFraction, divModFloor, isU32, two } from './util.js';
use crate::rust::chrono::util::{U32_MAX, autoSiFraction, divModFloor, isU32, two};

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Anything with a fixed UTC offset in seconds (`FixedOffset`, `Utc`).
// |  * @typedef {{ localMinusUtc(): number }} FixedOffsetLike
// |  */

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * chrono `NaiveTime`: seconds from midnight plus nanoseconds; `frac` reaches
// |  * up to 2e9 - 1 to represent a leap second at second 59.
// |  */
// | export class NaiveTime {
// |   /** @param {{ secs: number, frac: number }} fields */
// |   constructor({ secs, frac }) {
// |     /** @type {number} */
// |     this.secs = secs;
// |     /** @type {number} */
// |     this.frac = frac;
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * `NaiveTime::from_hms_opt`.
// |    * @param {number} hour
// |    * @param {number} min
// |    * @param {number} sec
// |    * @returns {NaiveTime | null}
// |    */
// |   static fromHmsOpt(hour, min, sec) {
// |     return NaiveTime.fromHmsNanoOpt(hour, min, sec, 0);
// |   }
// |
// |   /**
// |    * `NaiveTime::from_hms_milli_opt`.
// |    * @param {number} hour
// |    * @param {number} min
// |    * @param {number} sec
// |    * @param {number} milli
// |    * @returns {NaiveTime | null}
// |    */
// |   static fromHmsMilliOpt(hour, min, sec, milli) {
// |     if (!isU32(milli) || milli * 1_000_000 > U32_MAX) {
// |       return null;
// |     }
// |     return NaiveTime.fromHmsNanoOpt(hour, min, sec, milli * 1_000_000);
// |   }
// |
// |   /**
// |    * `NaiveTime::from_hms_micro_opt`.
// |    * @param {number} hour
// |    * @param {number} min
// |    * @param {number} sec
// |    * @param {number} micro
// |    * @returns {NaiveTime | null}
// |    */
// |   static fromHmsMicroOpt(hour, min, sec, micro) {
// |     if (!isU32(micro) || micro * 1000 > U32_MAX) {
// |       return null;
// |     }
// |     return NaiveTime.fromHmsNanoOpt(hour, min, sec, micro * 1000);
// |   }
// |
// |   /**
// |    * `NaiveTime::from_hms_nano_opt`: `nano` may reach 2e9 - 1 at second 59
// |    * (a leap second).
// |    * @param {number} hour
// |    * @param {number} min
// |    * @param {number} sec
// |    * @param {number} nano
// |    * @returns {NaiveTime | null}
// |    */
// |   static fromHmsNanoOpt(hour, min, sec, nano) {
// |     if (!isU32(hour) || !isU32(min) || !isU32(sec) || !isU32(nano)
// |       || hour >= 24 || min >= 60 || sec >= 60
// |       || (nano >= NANOS_PER_SEC && sec !== 59) || nano >= 2 * NANOS_PER_SEC) {
// |       return null;
// |     }
// |     return new NaiveTime({ secs: hour * 3600 + min * 60 + sec, frac: nano });
// |   }
// |
// |   /**
// |    * `NaiveTime::from_num_seconds_from_midnight_opt`.
// |    * @param {number} secs
// |    * @param {number} nano
// |    * @returns {NaiveTime | null}
// |    */
// |   static fromNumSecondsFromMidnightOpt(secs, nano) {
// |     if (!isU32(secs) || !isU32(nano) || secs >= 86_400 || nano >= 2 * NANOS_PER_SEC
// |       || (nano >= NANOS_PER_SEC && secs % 60 !== 59)) {
// |       return null;
// |     }
// |     return new NaiveTime({ secs, frac: nano });
// |   }
// |
// |   /**
// |    * `NaiveTime::parse_from_str`; throws `Err(ParseError)`.
// |    * @param {string} s
// |    * @param {string} fmt
// |    * @returns {NaiveTime}
// |    */
// |   static parseFromStr(s, fmt) {
// |     const parsed = new Parsed();
// |     parse(parsed, s, new StrftimeItems(fmt));
// |     return parsed.toNaiveTime();
// |   }
// |
// |   /** `NaiveTime::MIN` (midnight). */
// |   static get MIN() {
// |     return new NaiveTime({ secs: 0, frac: 0 });
// |   }
// |
// |   /** `NaiveTime::MAX` (crate-private): 23:59:59.999999999. */
// |   static get MAX() {
// |     return new NaiveTime({ secs: 86_399, frac: 999_999_999 });
// |   }
// |
// |   /** `Default` (midnight). */
// |   static default() {
// |     return NaiveTime.MIN;
// |   }
// |
// |   /** `hour`. */
// |   hour() {
// |     return Math.floor(this.secs / 3600);
// |   }
// |
// |   /** `minute`. */
// |   minute() {
// |     return Math.floor(this.secs / 60) % 60;
// |   }
// |
// |   /** `second`. */
// |   second() {
// |     return this.secs % 60;
// |   }
// |
// |   /** `nanosecond` (>= 1e9 during a leap second). */
// |   nanosecond() {
// |     return this.frac;
// |   }
// |
// |   /** `num_seconds_from_midnight`. */
// |   numSecondsFromMidnight() {
// |     return this.secs;
// |   }
// |
// |   /**
// |    * `overflowing_add_signed`: the wrapped time and the seconds (a multiple
// |    * of 86400) that did not fit into the day.
// |    * @param {TimeDelta} rhs
// |    * @returns {[NaiveTime, bigint]}
// |    */
// |   overflowingAddSigned(rhs) {
// |     let secs = BigInt(this.secs);
// |     let frac = this.frac;
// |     const secsToAdd = rhs.numSeconds();
// |     const fracToAdd = rhs.subsecNanos();
// |     if (frac >= NANOS_PER_SEC) {
// |       if (secsToAdd > 0n || (fracToAdd > 0 && frac >= 2 * NANOS_PER_SEC - fracToAdd)) {
// |         frac -= NANOS_PER_SEC;
// |       } else if (secsToAdd < 0n) {
// |         frac -= NANOS_PER_SEC;
// |         secs += 1n;
// |       } else {
// |         return [new NaiveTime({ secs: this.secs, frac: frac + fracToAdd }), 0n];
// |       }
// |     }
// |     secs += secsToAdd;
// |     frac += fracToAdd;
// |     if (frac < 0) {
// |       frac += NANOS_PER_SEC;
// |       secs -= 1n;
// |     } else if (frac >= NANOS_PER_SEC) {
// |       frac -= NANOS_PER_SEC;
// |       secs += 1n;
// |     }
// |     let secsInDay = secs % 86_400n;
// |     if (secsInDay < 0n) {
// |       secsInDay += 86_400n;
// |     }
// |     return [new NaiveTime({ secs: Number(secsInDay), frac }), secs - secsInDay];
// |   }
// |
// |   /**
// |    * `overflowing_sub_signed`.
// |    * @param {TimeDelta} rhs
// |    * @returns {[NaiveTime, bigint]}
// |    */
// |   overflowingSubSigned(rhs) {
// |     const [time, remainder] = this.overflowingAddSigned(rhs.neg());
// |     return [time, -remainder];
// |   }
// |
// |   /**
// |    * `signed_duration_since`, aware of leap seconds.
// |    * @param {NaiveTime} rhs
// |    * @returns {TimeDelta}
// |    */
// |   signedDurationSince(rhs) {
// |     let secs = this.secs - rhs.secs;
// |     const frac = this.frac - rhs.frac;
// |     if (this.secs > rhs.secs && rhs.frac >= NANOS_PER_SEC) {
// |       secs += 1;
// |     } else if (this.secs < rhs.secs && this.frac >= NANOS_PER_SEC) {
// |       secs -= 1;
// |     }
// |     const [secsFromFrac, fracRem] = divModFloor(frac, NANOS_PER_SEC);
// |     return expect(TimeDelta.new(secs + secsFromFrac, fracRem), 'must be in range');
// |   }
// |
// |   /**
// |    * `overflowing_add_offset` (crate-private): the wrapped time and the day
// |    * change (-1, 0 or 1).
// |    * @param {FixedOffsetLike} offset
// |    * @returns {[NaiveTime, number]}
// |    */
// |   overflowingAddOffset(offset) {
// |     const [days, secs] = divModFloor(this.secs + offset.localMinusUtc(), 86_400);
// |     return [new NaiveTime({ secs, frac: this.frac }), days];
// |   }
// |
// |   /**
// |    * `overflowing_sub_offset` (crate-private).
// |    * @param {FixedOffsetLike} offset
// |    * @returns {[NaiveTime, number]}
// |    */
// |   overflowingSubOffset(offset) {
// |     const [days, secs] = divModFloor(this.secs - offset.localMinusUtc(), 86_400);
// |     return [new NaiveTime({ secs, frac: this.frac }), days];
// |   }
// |
// |   /**
// |    * `NaiveTime + TimeDelta` (wraps around midnight).
// |    * @param {TimeDelta} rhs
// |    */
// |   add(rhs) {
// |     return this.overflowingAddSigned(rhs)[0];
// |   }
// |
// |   /**
// |    * `NaiveTime - TimeDelta` (wraps) or `NaiveTime - NaiveTime` (a `TimeDelta`).
// |    * @template {TimeDelta | NaiveTime} T
// |    * @param {T} rhs
// |    * @returns {T extends NaiveTime ? TimeDelta : NaiveTime}
// |    */
// |   sub(rhs) {
// |     if (rhs instanceof NaiveTime) {
// |       return /** @type {any} */ (this.signedDurationSince(rhs));
// |     }
// |     return /** @type {any} */ (this.overflowingSubSigned(/** @type {TimeDelta} */ (rhs))[0]);
// |   }
// |
// |   /**
// |    * `format(fmt).to_string()`.
// |    * @param {string} fmt
// |    */
// |   format(fmt) {
// |     return formatToString(null, this, null, fmt);
// |   }
// |
// |   /**
// |    * Derived `PartialEq`.
// |    * @param {NaiveTime} other
// |    */
// |   equals(other) {
// |     return other instanceof NaiveTime && this.secs === other.secs && this.frac === other.frac;
// |   }
// |
// |   /**
// |    * Derived `Ord`.
// |    * @param {NaiveTime} other
// |    * @returns {-1 | 0 | 1}
// |    */
// |   cmp(other) {
// |     if (this.secs !== other.secs) {
// |       return this.secs < other.secs ? -1 : 1;
// |     }
// |     if (this.frac !== other.frac) {
// |       return this.frac < other.frac ? -1 : 1;
// |     }
// |     return 0;
// |   }
// |
// |   /** `Debug`: `23:56:04`, `23:59:60.5` for a leap second. */
// |   fmtDebug() {
// |     let sec = this.second();
// |     let nano = this.frac;
// |     if (nano >= NANOS_PER_SEC) {
// |       sec += 1;
// |       nano -= NANOS_PER_SEC;
// |     }
// |     return `${two(this.hour())}:${two(this.minute())}:${two(sec)}${autoSiFraction(nano)}`;
// |   }
// |
// |   /** `Display` (same as `Debug`). */
// |   toString() {
// |     return this.fmtDebug();
// |   }
// |
// |   /** `Serialize`: the `Display` string. */
// |   toJSON() {
// |     return this.toString();
// |   }
// | }
