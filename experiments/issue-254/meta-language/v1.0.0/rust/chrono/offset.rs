// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=9968f6afd70795bb42e43be1e365ba800754bc3f975ab87fbf9d7d802ab99a61 bytes=13684

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * chrono 0.4.45 time zones and `DateTime`: `Utc` (src/offset/utc.rs),
// |  * `FixedOffset` (src/offset/fixed.rs), `LocalResult` and the `TimeZone`
// |  * methods (src/offset/mod.rs) and `DateTime` (src/datetime/mod.rs).
// |  */

// meta-language:carried JavaScript import_statement (unsupported)
// | import { expect, panic } from '../core.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { Months } from './delta.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { NaiveDate } from './date.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { NaiveTime } from './time.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { formatToString, writeRfc3339 } from './formatting.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { parseRfc3339 } from './parse.js';

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {import('./datetime.js').NaiveDateTime} NaiveDateTime */
// | /** @typedef {import('./delta.js').TimeDelta} TimeDelta */

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * chrono `LocalResult` (`MappedLocalTime`): `None`, `Single(T)` or
// |  * `Ambiguous(T, T)`.
// |  * @template T
// |  */
// | export class LocalResult {
// |   /**
// |    * @param {'None' | 'Single' | 'Ambiguous'} tag
// |    * @param {{ _0?: T, _1?: T }} [fields]
// |    */
// |   constructor(tag, fields = {}) {
// |     this.tag = tag;
// |     /** @type {T | undefined} */
// |     this._0 = fields._0;
// |     /** @type {T | undefined} */
// |     this._1 = fields._1;
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * `LocalResult::None`.
// |    * @template T
// |    * @returns {LocalResult<T>}
// |    */
// |   static None() {
// |     return new LocalResult('None');
// |   }
// |
// |   /**
// |    * `LocalResult::Single`.
// |    * @template T
// |    * @param {T} value
// |    * @returns {LocalResult<T>}
// |    */
// |   static Single(value) {
// |     return new LocalResult('Single', { _0: value });
// |   }
// |
// |   /** `single`. @returns {T | null} */
// |   single() {
// |     return this.tag === 'Single' ? /** @type {T} */ (this._0) : null;
// |   }
// |
// |   /** `earliest`. @returns {T | null} */
// |   earliest() {
// |     return this.tag === 'None' ? null : /** @type {T} */ (this._0);
// |   }
// |
// |   /** `latest`. @returns {T | null} */
// |   latest() {
// |     if (this.tag === 'None') {
// |       return null;
// |     }
// |     return /** @type {T} */ (this.tag === 'Single' ? this._0 : this._1);
// |   }
// |
// |   /** `unwrap`; panics unless `Single`. @returns {T} */
// |   unwrap() {
// |     if (this.tag === 'None') {
// |       return panic('No such local time');
// |     }
// |     if (this.tag === 'Ambiguous') {
// |       const debug = (/** @type {any} */ v) => v.fmtDebug();
// |       return panic(`Ambiguous local time, ranging from ${debug(this._0)} to ${debug(this._1)}`);
// |     }
// |     return /** @type {T} */ (this._0);
// |   }
// | }

// meta-language:carried JavaScript class_declaration (unsupported)
// | /** Methods shared by `Utc` and `FixedOffset` (`TimeZone`). */
// | class TimeZoneBase {
// |   /** @returns {number} */
// |   localMinusUtc() {
// |     return panic('abstract');
// |   }
// |
// |   /**
// |    * `from_local_datetime`.
// |    * @param {NaiveDateTime} local
// |    * @returns {LocalResult<DateTime>}
// |    */
// |   fromLocalDatetime(local) {
// |     const utc = local.checkedSubOffset(this);
// |     return utc === null ? LocalResult.None() : LocalResult.Single(new DateTime(utc, /** @type {any} */ (this)));
// |   }
// |
// |   /**
// |    * `from_utc_datetime`.
// |    * @param {NaiveDateTime} utc
// |    */
// |   fromUtcDatetime(utc) {
// |     return new DateTime(utc, /** @type {any} */ (this));
// |   }
// |
// |   /**
// |    * `with_ymd_and_hms`.
// |    * @param {number} year
// |    * @param {number} month
// |    * @param {number} day
// |    * @param {number} hour
// |    * @param {number} min
// |    * @param {number} sec
// |    * @returns {LocalResult<DateTime>}
// |    */
// |   withYmdAndHms(year, month, day, hour, min, sec) {
// |     const local = NaiveDate.fromYmdOpt(year, month, day)?.andHmsOpt(hour, min, sec) ?? null;
// |     return local === null ? LocalResult.None() : this.fromLocalDatetime(local);
// |   }
// |
// |   /**
// |    * `timestamp_opt`.
// |    * @param {number} secs
// |    * @param {number} nsecs
// |    * @returns {LocalResult<DateTime>}
// |    */
// |   timestampOpt(secs, nsecs) {
// |     const naive = naiveFromTimestamp(secs, nsecs);
// |     return naive === null ? LocalResult.None() : LocalResult.Single(this.fromUtcDatetime(naive));
// |   }
// |
// |   /**
// |    * `timestamp_millis_opt`.
// |    * @param {number | bigint} millis an `i64` (a number must be a safe integer)
// |    * @returns {LocalResult<DateTime>}
// |    */
// |   timestampMillisOpt(millis) {
// |     const ms = BigInt(millis);
// |     let secs = ms / 1000n;
// |     let rem = ms % 1000n;
// |     if (rem < 0n) {
// |       secs -= 1n;
// |       rem += 1000n;
// |     }
// |     // Out of i32-day range anyway once |secs| exceeds 2^53.
// |     if (secs > 9_007_199_254_740_991n || secs < -9_007_199_254_740_991n) {
// |       return LocalResult.None();
// |     }
// |     return this.timestampOpt(Number(secs), Number(rem) * 1_000_000);
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `DateTime::from_timestamp(secs, nsecs).naive_utc()`.
// |  * @param {number} secs
// |  * @param {number} nsecs
// |  * @returns {NaiveDateTime | null}
// |  */
// | function naiveFromTimestamp(secs, nsecs) {
// |   if (!Number.isSafeInteger(secs)) {
// |     return null;
// |   }
// |   const days = Math.floor(secs / 86_400);
// |   const secsOfDay = secs - days * 86_400;
// |   const date = NaiveDate.fromNumDaysFromCeOpt(days + 719_163);
// |   const time = NaiveTime.fromNumSecondsFromMidnightOpt(secsOfDay, nsecs);
// |   return date === null || time === null ? null : date.andTime(time);
// | }

// meta-language:carried JavaScript class_declaration (unsupported)
// | /** chrono `Utc` (both the zone and its offset). */
// | class UtcZone extends TimeZoneBase {
// |   constructor() {
// |     super();
// |     Object.freeze(this);
// |   }
// |
// |   /** `Offset::fix().local_minus_utc()` = 0. */
// |   localMinusUtc() {
// |     return 0;
// |   }
// |
// |   /** `fix`. */
// |   fix() {
// |     return FixedOffset.UTC;
// |   }
// |
// |   /** `Utc::now()`: millisecond precision from `Date.now()`, like the wasm build. */
// |   now() {
// |     return this.timestampMillisOpt(Date.now()).unwrap();
// |   }
// |
// |   /** `PartialEq`. @param {unknown} other */
// |   equals(other) {
// |     return other === this;
// |   }
// |
// |   /** `Display`. */
// |   toString() {
// |     return 'UTC';
// |   }
// |
// |   /** `Debug`. */
// |   fmtDebug() {
// |     return 'Z';
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** The `Utc` singleton. */
// | export const Utc = new UtcZone();

// meta-language:carried JavaScript export_statement (unsupported)
// | /** chrono `FixedOffset`: seconds east of UTC in `-86399..=86399`. */
// | export class FixedOffset extends TimeZoneBase {
// |   /** @param {number} localMinusUtc */
// |   constructor(localMinusUtc) {
// |     super();
// |     /** @type {number} */
// |     this._0 = localMinusUtc;
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * `FixedOffset::east_opt`.
// |    * @param {number} secs
// |    * @returns {FixedOffset | null}
// |    */
// |   static eastOpt(secs) {
// |     return Number.isInteger(secs) && secs > -86_400 && secs < 86_400 ? new FixedOffset(secs + 0) : null;
// |   }
// |
// |   /**
// |    * `FixedOffset::west_opt`.
// |    * @param {number} secs
// |    * @returns {FixedOffset | null}
// |    */
// |   static westOpt(secs) {
// |     return FixedOffset.eastOpt(-secs);
// |   }
// |
// |   /** UTC as a `FixedOffset`. */
// |   static get UTC() {
// |     return new FixedOffset(0);
// |   }
// |
// |   /** `local_minus_utc`. */
// |   localMinusUtc() {
// |     return this._0;
// |   }
// |
// |   /** `utc_minus_local`. */
// |   utcMinusLocal() {
// |     return -this._0;
// |   }
// |
// |   /** `fix`. */
// |   fix() {
// |     return this;
// |   }
// |
// |   /** `PartialEq`. @param {unknown} other */
// |   equals(other) {
// |     return other instanceof FixedOffset && other._0 === this._0;
// |   }
// |
// |   /** `Debug`: `+05:30`, `-00:00:30`. */
// |   fmtDebug() {
// |     const sign = this._0 < 0 ? '-' : '+';
// |     const off = Math.abs(this._0);
// |     const sec = off % 60;
// |     const mins = Math.floor(off / 60);
// |     const two = (/** @type {number} */ v) => String(v).padStart(2, '0');
// |     const base = `${sign}${two(Math.floor(mins / 60))}:${two(mins % 60)}`;
// |     return sec === 0 ? base : `${base}:${two(sec)}`;
// |   }
// |
// |   /** `Display` (same as `Debug`). */
// |   toString() {
// |     return this.fmtDebug();
// |   }
// | }

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {UtcZone | FixedOffset} Offset */

// meta-language:carried JavaScript export_statement (unsupported)
// | /** chrono `DateTime<Utc>` / `DateTime<FixedOffset>`: a UTC time and an offset. */
// | export class DateTime {
// |   /**
// |    * @param {NaiveDateTime} datetime the UTC time
// |    * @param {Offset} offsetValue
// |    */
// |   constructor(datetime, offsetValue) {
// |     /** @type {NaiveDateTime} */
// |     this.datetime = datetime;
// |     /** @type {Offset} */
// |     this.offsetValue = offsetValue;
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * `DateTime::from_naive_utc_and_offset`.
// |    * @param {NaiveDateTime} datetime
// |    * @param {Offset} offset
// |    */
// |   static fromNaiveUtcAndOffset(datetime, offset) {
// |     return new DateTime(datetime, offset);
// |   }
// |
// |   /**
// |    * `DateTime::parse_from_rfc3339`; throws `Err(ParseError)`.
// |    * @param {string} s
// |    * @returns {DateTime}
// |    */
// |   static parseFromRfc3339(s) {
// |     return parseRfc3339(s);
// |   }
// |
// |   /** `naive_utc`. */
// |   naiveUtc() {
// |     return this.datetime;
// |   }
// |
// |   /** `naive_local`; panics out of range. */
// |   naiveLocal() {
// |     return expect(this.datetime.checkedAddOffset(this.offsetValue),
// |       'Local time out of range for `NaiveDateTime`');
// |   }
// |
// |   /** `overflowing_naive_local` (crate-private). */
// |   overflowingNaiveLocal() {
// |     return this.datetime.overflowingAddOffset(this.offsetValue);
// |   }
// |
// |   /** `offset`. */
// |   offset() {
// |     return this.offsetValue;
// |   }
// |
// |   /** `timezone`. */
// |   timezone() {
// |     return this.offsetValue;
// |   }
// |
// |   /** `fixed_offset`. */
// |   fixedOffset() {
// |     return new DateTime(this.datetime, this.offsetValue.fix());
// |   }
// |
// |   /**
// |    * `with_timezone`.
// |    * @param {Offset} tz
// |    */
// |   withTimezone(tz) {
// |     return tz.fromUtcDatetime(this.datetime);
// |   }
// |
// |   /** `date_naive`. */
// |   dateNaive() {
// |     return this.naiveLocal().date();
// |   }
// |
// |   /** `time`. */
// |   time() {
// |     return this.datetime.time().overflowingAddOffset(this.offsetValue)[0];
// |   }
// |
// |   /** `year`. */
// |   year() {
// |     return this.naiveLocal().year();
// |   }
// |
// |   /** `month`. */
// |   month() {
// |     return this.naiveLocal().month();
// |   }
// |
// |   /** `day`. */
// |   day() {
// |     return this.naiveLocal().day();
// |   }
// |
// |   /** `ordinal`. */
// |   ordinal() {
// |     return this.naiveLocal().ordinal();
// |   }
// |
// |   /** `hour`. */
// |   hour() {
// |     return this.time().hour();
// |   }
// |
// |   /** `minute`. */
// |   minute() {
// |     return this.time().minute();
// |   }
// |
// |   /** `second`. */
// |   second() {
// |     return this.time().second();
// |   }
// |
// |   /** `nanosecond`. */
// |   nanosecond() {
// |     return this.time().nanosecond();
// |   }
// |
// |   /** `timestamp` (seconds since the epoch). */
// |   timestamp() {
// |     const days = this.datetime.date().numDaysFromCe() - 719_163;
// |     return days * 86_400 + this.datetime.time().numSecondsFromMidnight();
// |   }
// |
// |   /** `timestamp_millis`. */
// |   timestampMillis() {
// |     return this.timestamp() * 1000 + Math.floor(this.datetime.nanosecond() / 1_000_000);
// |   }
// |
// |   /** `to_rfc3339`. */
// |   toRfc3339() {
// |     return expect(writeRfc3339(this.overflowingNaiveLocal(), this.offsetValue.localMinusUtc(), 'AutoSi', false),
// |       'writing rfc3339 datetime to string should never fail');
// |   }
// |
// |   /**
// |    * `to_rfc3339_opts`.
// |    * @param {string} secform a `SecondsFormat`
// |    * @param {boolean} useZ
// |    */
// |   toRfc3339Opts(secform, useZ) {
// |     return expect(writeRfc3339(this.overflowingNaiveLocal(), this.offsetValue.localMinusUtc(), secform, useZ),
// |       'writing rfc3339 datetime to string should never fail');
// |   }
// |
// |   /**
// |    * `format(fmt).to_string()`.
// |    * @param {string} fmt
// |    */
// |   format(fmt) {
// |     const local = this.overflowingNaiveLocal();
// |     return formatToString(local.date(), local.time(),
// |       { name: this.offsetValue.toString(), fix: this.offsetValue.localMinusUtc() }, fmt);
// |   }
// |
// |   /**
// |    * `checked_add_signed`.
// |    * @param {TimeDelta} rhs
// |    * @returns {DateTime | null}
// |    */
// |   checkedAddSigned(rhs) {
// |     const utc = this.datetime.checkedAddSigned(rhs);
// |     return utc === null ? null : new DateTime(utc, this.offsetValue);
// |   }
// |
// |   /**
// |    * `checked_sub_signed`.
// |    * @param {TimeDelta} rhs
// |    * @returns {DateTime | null}
// |    */
// |   checkedSubSigned(rhs) {
// |     const utc = this.datetime.checkedSubSigned(rhs);
// |     return utc === null ? null : new DateTime(utc, this.offsetValue);
// |   }
// |
// |   /**
// |    * `checked_add_months`.
// |    * @param {Months} rhs
// |    * @returns {DateTime | null}
// |    */
// |   checkedAddMonths(rhs) {
// |     return this.overflowingNaiveLocal().checkedAddMonths(rhs)?.andLocalTimezone(this.offsetValue).single() ?? null;
// |   }
// |
// |   /**
// |    * `checked_sub_months`.
// |    * @param {Months} rhs
// |    * @returns {DateTime | null}
// |    */
// |   checkedSubMonths(rhs) {
// |     return this.overflowingNaiveLocal().checkedSubMonths(rhs)?.andLocalTimezone(this.offsetValue).single() ?? null;
// |   }
// |
// |   /**
// |    * `signed_duration_since`.
// |    * @param {DateTime} rhs
// |    */
// |   signedDurationSince(rhs) {
// |     return this.datetime.signedDurationSince(rhs.datetime);
// |   }
// |
// |   /**
// |    * `DateTime + TimeDelta` / `+ Months`; panics on overflow.
// |    * @param {TimeDelta | Months} rhs
// |    * @returns {DateTime}
// |    */
// |   add(rhs) {
// |     if (rhs instanceof Months) {
// |       return expect(this.checkedAddMonths(rhs), '`DateTime + Months` out of range');
// |     }
// |     return expect(this.checkedAddSigned(rhs), '`DateTime + TimeDelta` overflowed');
// |   }
// |
// |   /**
// |    * `DateTime - TimeDelta` / `- Months` (panics on overflow) or
// |    * `DateTime - DateTime` (a `TimeDelta`).
// |    * @template {TimeDelta | Months | DateTime} T
// |    * @param {T} rhs
// |    * @returns {T extends DateTime ? TimeDelta : DateTime}
// |    */
// |   sub(rhs) {
// |     /** @type {any} */
// |     let result;
// |     if (rhs instanceof DateTime) {
// |       result = this.signedDurationSince(rhs);
// |     } else if (rhs instanceof Months) {
// |       result = expect(this.checkedSubMonths(rhs), '`DateTime - Months` out of range');
// |     } else {
// |       result = expect(this.checkedSubSigned(/** @type {TimeDelta} */ (rhs)), '`DateTime - TimeDelta` overflowed');
// |     }
// |     return result;
// |   }
// |
// |   /**
// |    * `PartialEq`: compares the instants.
// |    * @param {unknown} other
// |    */
// |   equals(other) {
// |     return other instanceof DateTime && this.datetime.equals(other.datetime);
// |   }
// |
// |   /**
// |    * `Ord`: compares the instants.
// |    * @param {DateTime} other
// |    */
// |   cmp(other) {
// |     return this.datetime.cmp(other.datetime);
// |   }
// |
// |   /** `Debug`: `2026-03-15T12:34:56Z`. */
// |   fmtDebug() {
// |     return this.overflowingNaiveLocal().fmtDebug() + this.offsetValue.fmtDebug();
// |   }
// |
// |   /** `Display`: `2026-03-15 12:34:56 UTC`. */
// |   toString() {
// |     return `${this.overflowingNaiveLocal().toString()} ${this.offsetValue.toString()}`;
// |   }
// |
// |   /** `Serialize`: RFC 3339 with `Z` for UTC. */
// |   toJSON() {
// |     return expect(writeRfc3339(this.naiveLocal(), this.offsetValue.localMinusUtc(), 'AutoSi', true),
// |       'chrono::DateTime::serialize');
// |   }
// | }
