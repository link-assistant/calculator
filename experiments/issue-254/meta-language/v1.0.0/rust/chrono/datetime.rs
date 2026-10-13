// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=3247d46a49f8fb8f26ed5df2cbc19437fa41ea77906fe8825b4970cb9798f9ea bytes=9257

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * chrono 0.4.45 `NaiveDateTime` (src/naive/datetime/mod.rs).
// |  */

// meta-language:carried JavaScript import_statement (unsupported)
// | import { expect } from '../core.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { Months, TimeDelta } from './delta.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { formatToString } from './formatting.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { DateTime, Utc } from './offset.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { Parsed, parse } from './parse.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { StrftimeItems } from './strftime.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { NaiveDate } from './date.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { NaiveTime } from './time.js';

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {import('./time.js').FixedOffsetLike} FixedOffsetLike */

// meta-language:carried JavaScript export_statement (unsupported)
// | /** chrono `NaiveDateTime`: a `NaiveDate` and a `NaiveTime`. */
// | export class NaiveDateTime {
// |   /**
// |    * The fields are not named `date` and `time` because those are methods.
// |    * @param {{ dateValue: NaiveDate, timeValue: NaiveTime }} fields
// |    */
// |   constructor({ dateValue, timeValue }) {
// |     /** @type {NaiveDate} */
// |     this.dateValue = dateValue;
// |     /** @type {NaiveTime} */
// |     this.timeValue = timeValue;
// |     Object.freeze(this);
// |   }
// |
// |   /**
// |    * `NaiveDateTime::new`.
// |    * @param {NaiveDate} date
// |    * @param {NaiveTime} time
// |    */
// |   static new(date, time) {
// |     return new NaiveDateTime({ dateValue: date, timeValue: time });
// |   }
// |
// |   /**
// |    * `NaiveDateTime::parse_from_str`; throws `Err(ParseError)`.
// |    * @param {string} s
// |    * @param {string} fmt
// |    * @returns {NaiveDateTime}
// |    */
// |   static parseFromStr(s, fmt) {
// |     const parsed = new Parsed();
// |     parse(parsed, s, new StrftimeItems(fmt));
// |     return parsed.toNaiveDatetimeWithOffset(0);
// |   }
// |
// |   /** `NaiveDateTime::MIN`. */
// |   static get MIN() {
// |     return new NaiveDateTime({ dateValue: NaiveDate.MIN, timeValue: NaiveTime.MIN });
// |   }
// |
// |   /** `NaiveDateTime::MAX`. */
// |   static get MAX() {
// |     return new NaiveDateTime({ dateValue: NaiveDate.MAX, timeValue: NaiveTime.MAX });
// |   }
// |
// |   /** `Default`: 1970-01-01 00:00:00. */
// |   static default() {
// |     return new NaiveDateTime({ dateValue: NaiveDate.default(), timeValue: NaiveTime.MIN });
// |   }
// |
// |   /** `date`. */
// |   date() {
// |     return this.dateValue;
// |   }
// |
// |   /** `time`. */
// |   time() {
// |     return this.timeValue;
// |   }
// |
// |   /** `year`. */
// |   year() {
// |     return this.dateValue.year();
// |   }
// |
// |   /** `month`. */
// |   month() {
// |     return this.dateValue.month();
// |   }
// |
// |   /** `month0`. */
// |   month0() {
// |     return this.dateValue.month0();
// |   }
// |
// |   /** `day`. */
// |   day() {
// |     return this.dateValue.day();
// |   }
// |
// |   /** `ordinal`. */
// |   ordinal() {
// |     return this.dateValue.ordinal();
// |   }
// |
// |   /** `hour`. */
// |   hour() {
// |     return this.timeValue.hour();
// |   }
// |
// |   /** `minute`. */
// |   minute() {
// |     return this.timeValue.minute();
// |   }
// |
// |   /** `second`. */
// |   second() {
// |     return this.timeValue.second();
// |   }
// |
// |   /** `nanosecond`. */
// |   nanosecond() {
// |     return this.timeValue.nanosecond();
// |   }
// |
// |   /**
// |    * `checked_add_signed`.
// |    * @param {TimeDelta} rhs
// |    * @returns {NaiveDateTime | null}
// |    */
// |   checkedAddSigned(rhs) {
// |     const [time, remainder] = this.timeValue.overflowingAddSigned(rhs);
// |     const remainderDelta = TimeDelta.trySeconds(remainder);
// |     const date = remainderDelta === null ? null : this.dateValue.checkedAddSigned(remainderDelta);
// |     return date === null ? null : new NaiveDateTime({ dateValue: date, timeValue: time });
// |   }
// |
// |   /**
// |    * `checked_sub_signed`.
// |    * @param {TimeDelta} rhs
// |    * @returns {NaiveDateTime | null}
// |    */
// |   checkedSubSigned(rhs) {
// |     const [time, remainder] = this.timeValue.overflowingSubSigned(rhs);
// |     const remainderDelta = TimeDelta.trySeconds(remainder);
// |     const date = remainderDelta === null ? null : this.dateValue.checkedSubSigned(remainderDelta);
// |     return date === null ? null : new NaiveDateTime({ dateValue: date, timeValue: time });
// |   }
// |
// |   /**
// |    * `checked_add_months`.
// |    * @param {Months} rhs
// |    * @returns {NaiveDateTime | null}
// |    */
// |   checkedAddMonths(rhs) {
// |     const date = this.dateValue.checkedAddMonths(rhs);
// |     return date === null ? null : new NaiveDateTime({ dateValue: date, timeValue: this.timeValue });
// |   }
// |
// |   /**
// |    * `checked_sub_months`.
// |    * @param {Months} rhs
// |    * @returns {NaiveDateTime | null}
// |    */
// |   checkedSubMonths(rhs) {
// |     const date = this.dateValue.checkedSubMonths(rhs);
// |     return date === null ? null : new NaiveDateTime({ dateValue: date, timeValue: this.timeValue });
// |   }
// |
// |   /**
// |    * The date after a day change of -1, 0 or 1.
// |    * @param {number} days
// |    * @param {boolean} overflowing fall back to `BEFORE_MIN`/`AFTER_MAX`
// |    * @returns {NaiveDate | null}
// |    */
// |   shiftedDate(days, overflowing) {
// |     if (days === -1) {
// |       return this.dateValue.predOpt() ?? (overflowing ? NaiveDate.BEFORE_MIN : null);
// |     }
// |     if (days === 1) {
// |       return this.dateValue.succOpt() ?? (overflowing ? NaiveDate.AFTER_MAX : null);
// |     }
// |     return this.dateValue;
// |   }
// |
// |   /**
// |    * `checked_add_offset`.
// |    * @param {FixedOffsetLike} rhs
// |    * @returns {NaiveDateTime | null}
// |    */
// |   checkedAddOffset(rhs) {
// |     const [time, days] = this.timeValue.overflowingAddOffset(rhs);
// |     const date = this.shiftedDate(days, false);
// |     return date === null ? null : new NaiveDateTime({ dateValue: date, timeValue: time });
// |   }
// |
// |   /**
// |    * `checked_sub_offset`.
// |    * @param {FixedOffsetLike} rhs
// |    * @returns {NaiveDateTime | null}
// |    */
// |   checkedSubOffset(rhs) {
// |     const [time, days] = this.timeValue.overflowingSubOffset(rhs);
// |     const date = this.shiftedDate(days, false);
// |     return date === null ? null : new NaiveDateTime({ dateValue: date, timeValue: time });
// |   }
// |
// |   /**
// |    * `overflowing_add_offset` (crate-private): may land one day outside the
// |    * range (`BEFORE_MIN`/`AFTER_MAX`).
// |    * @param {FixedOffsetLike} rhs
// |    * @returns {NaiveDateTime}
// |    */
// |   overflowingAddOffset(rhs) {
// |     const [time, days] = this.timeValue.overflowingAddOffset(rhs);
// |     const date = /** @type {NaiveDate} */ (this.shiftedDate(days, true));
// |     return new NaiveDateTime({ dateValue: date, timeValue: time });
// |   }
// |
// |   /**
// |    * `overflowing_sub_offset` (crate-private).
// |    * @param {FixedOffsetLike} rhs
// |    * @returns {NaiveDateTime}
// |    */
// |   overflowingSubOffset(rhs) {
// |     const [time, days] = this.timeValue.overflowingSubOffset(rhs);
// |     const date = /** @type {NaiveDate} */ (this.shiftedDate(days, true));
// |     return new NaiveDateTime({ dateValue: date, timeValue: time });
// |   }
// |
// |   /**
// |    * `signed_duration_since`.
// |    * @param {NaiveDateTime} rhs
// |    * @returns {TimeDelta}
// |    */
// |   signedDurationSince(rhs) {
// |     return expect(
// |       this.dateValue.signedDurationSince(rhs.dateValue)
// |         .checkedAdd(this.timeValue.signedDurationSince(rhs.timeValue)),
// |       'always in range',
// |     );
// |   }
// |
// |   /**
// |    * `NaiveDateTime + TimeDelta` / `+ Months` / `+ FixedOffset`; panics on
// |    * overflow.
// |    * @param {TimeDelta | Months | FixedOffsetLike} rhs
// |    * @returns {NaiveDateTime}
// |    */
// |   add(rhs) {
// |     if (rhs instanceof Months) {
// |       return expect(this.checkedAddMonths(rhs), '`NaiveDateTime + Months` out of range');
// |     }
// |     if (rhs instanceof TimeDelta) {
// |       return expect(this.checkedAddSigned(rhs), '`NaiveDateTime + TimeDelta` overflowed');
// |     }
// |     return expect(this.checkedAddOffset(rhs), '`NaiveDateTime + FixedOffset` out of range');
// |   }
// |
// |   /**
// |    * `NaiveDateTime - TimeDelta` / `- Months` / `- FixedOffset` (panics on
// |    * overflow) or `NaiveDateTime - NaiveDateTime` (a `TimeDelta`).
// |    * @template {TimeDelta | Months | FixedOffsetLike | NaiveDateTime} T
// |    * @param {T} rhs
// |    * @returns {T extends NaiveDateTime ? TimeDelta : NaiveDateTime}
// |    */
// |   sub(rhs) {
// |     /** @type {any} */
// |     let result;
// |     if (rhs instanceof NaiveDateTime) {
// |       result = this.signedDurationSince(rhs);
// |     } else if (rhs instanceof Months) {
// |       result = expect(this.checkedSubMonths(rhs), '`NaiveDateTime - Months` out of range');
// |     } else if (rhs instanceof TimeDelta) {
// |       result = expect(this.checkedSubSigned(rhs), '`NaiveDateTime - TimeDelta` overflowed');
// |     } else {
// |       result = expect(this.checkedSubOffset(/** @type {FixedOffsetLike} */ (rhs)),
// |         '`NaiveDateTime - FixedOffset` out of range');
// |     }
// |     return result;
// |   }
// |
// |   /**
// |    * `and_local_timezone`: `tz.from_local_datetime(self)`.
// |    * @template R
// |    * @param {{ fromLocalDatetime(local: NaiveDateTime): R }} tz
// |    * @returns {R}
// |    */
// |   andLocalTimezone(tz) {
// |     return tz.fromLocalDatetime(this);
// |   }
// |
// |   /**
// |    * `and_utc`: the `DateTime<Utc>` with this UTC time.
// |    * @returns {DateTime}
// |    */
// |   andUtc() {
// |     return DateTime.fromNaiveUtcAndOffset(this, Utc);
// |   }
// |
// |   /**
// |    * `format(fmt).to_string()`.
// |    * @param {string} fmt
// |    */
// |   format(fmt) {
// |     return formatToString(this.dateValue, this.timeValue, null, fmt);
// |   }
// |
// |   /**
// |    * Derived `PartialEq`.
// |    * @param {NaiveDateTime} other
// |    */
// |   equals(other) {
// |     return other instanceof NaiveDateTime && this.dateValue.equals(other.dateValue)
// |       && this.timeValue.equals(other.timeValue);
// |   }
// |
// |   /**
// |    * Derived `Ord`.
// |    * @param {NaiveDateTime} other
// |    * @returns {-1 | 0 | 1}
// |    */
// |   cmp(other) {
// |     return this.dateValue.cmp(other.dateValue) || this.timeValue.cmp(other.timeValue);
// |   }
// |
// |   /** `Debug`: `2016-07-08T09:10:11`. */
// |   fmtDebug() {
// |     return `${this.dateValue.fmtDebug()}T${this.timeValue.fmtDebug()}`;
// |   }
// |
// |   /** `Display`: `2016-07-08 09:10:11`. */
// |   toString() {
// |     return `${this.dateValue.fmtDebug()} ${this.timeValue.fmtDebug()}`;
// |   }
// |
// |   /** `Serialize`: the `Debug` string. */
// |   toJSON() {
// |     return this.fmtDebug();
// |   }
// | }
