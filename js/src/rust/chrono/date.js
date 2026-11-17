/**
 * chrono 0.4.45 `NaiveDate` (src/naive/date/mod.rs, with the year flags of
 * src/naive/internals.rs and `IsoWeek` of src/naive/isoweek.rs).
 *
 * chrono packs a date into `yof` (year << 13 | ordinal << 4 | flags); the port
 * keeps the year and the ordinal (day of the year, 1-based), which is all the
 * packed form encodes besides the derived weekday flags.
 */

import { expect } from '../core.js';
import { I32_MAX, I32_MIN, Months, TimeDelta } from './delta.js';
import { formatToString } from './formatting.js';
import { Parsed, parse } from './parse.js';
import { StrftimeItems } from './strftime.js';
import { NaiveDateTime } from './datetime.js';
import { NaiveTime } from './time.js';
import { debugYear, divModFloor, inRange, isU32, two } from './util.js';

/** `MIN_YEAR` = `(i32::MIN >> 13) + 1`. */
export const MIN_YEAR = -262_143;
/** `MAX_YEAR` = `(i32::MAX >> 13) - 1`. */
export const MAX_YEAR = 262_142;

/** Days before the first of each month (index 1-12) in a common year. */
const CUMULATIVE_DAYS = [0, 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];

/**
 * `YearFlags::from_year(year).ndays() == 366`.
 * @param {number} year
 */
function isLeapYear(year) {
  return year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
}

/**
 * `YEAR_DELTAS[year_mod_400]`: leap days in the 400-year cycle before the
 * year.
 * @param {number} yearMod400
 */
function yearDelta(yearMod400) {
  return Math.floor((yearMod400 + 3) / 4) - Math.floor((yearMod400 + 99) / 100)
    + Math.floor((yearMod400 + 399) / 400);
}

/**
 * `cycle_to_yo`.
 * @param {number} cycle
 * @returns {[number, number]} `(year_mod_400, ordinal)`
 */
function cycleToYo(cycle) {
  let yearMod400 = Math.floor(cycle / 365);
  let ordinal0 = cycle % 365;
  const delta = yearDelta(yearMod400);
  if (ordinal0 < delta) {
    yearMod400 -= 1;
    ordinal0 += 365 - yearDelta(yearMod400);
  } else {
    ordinal0 -= delta;
  }
  return [yearMod400, ordinal0 + 1];
}

/**
 * `yo_to_cycle`.
 * @param {number} yearMod400
 * @param {number} ordinal
 */
function yoToCycle(yearMod400, ordinal) {
  return yearMod400 * 365 + yearDelta(yearMod400) + ordinal - 1;
}

/**
 * `YearFlags::nisoweeks`: 53 when the year starts on a Thursday, or on a
 * Wednesday in a leap year.
 * @param {number} year
 */
function isoWeeksIn(year) {
  const jan1 = new NaiveDate({ yearValue: year, ordinalValue: 1 }).numDaysFromMonday();
  return jan1 === 3 || (jan1 === 2 && isLeapYear(year)) ? 53 : 52;
}

/** chrono `NaiveDate`: a proleptic Gregorian date in `MIN_YEAR..=MAX_YEAR`. */
export class NaiveDate {
  /**
   * Internal: use `fromYmdOpt` and friends. (The fields are not named `year`
   * and `ordinal` because those are `Datelike` methods.)
   * @param {{ yearValue: number, ordinalValue: number }} fields
   */
  constructor({ yearValue, ordinalValue }) {
    /** @type {number} */
    this.yearValue = yearValue;
    /** @type {number} */
    this.ordinalValue = ordinalValue;
    Object.freeze(this);
  }

  /**
   * `from_ordinal_and_flags`: `None` out of range or for day 366 of a common
   * year.
   * @param {number} year
   * @param {number} ordinal
   * @returns {NaiveDate | null}
   */
  static fromOrdinal(year, ordinal) {
    if (year < MIN_YEAR || year > MAX_YEAR || ordinal < 1 || ordinal > (isLeapYear(year) ? 366 : 365)) {
      return null;
    }
    return new NaiveDate({ yearValue: year, ordinalValue: ordinal });
  }

  /**
   * `NaiveDate::from_ymd_opt`.
   * @param {number} year an `i32`
   * @param {number} month a `u32`
   * @param {number} day a `u32`
   * @returns {NaiveDate | null}
   */
  static fromYmdOpt(year, month, day) {
    if (!inRange(year, I32_MIN, I32_MAX) || !inRange(month, 1, 12) || !isU32(day) || day < 1) {
      return null;
    }
    const leap = isLeapYear(year);
    const length = month === 2 ? (leap ? 29 : 28) : CUMULATIVE_DAYS[month + 1] - CUMULATIVE_DAYS[month];
    if (day > length) {
      return null;
    }
    return NaiveDate.fromOrdinal(year, CUMULATIVE_DAYS[month] + (leap && month > 2 ? 1 : 0) + day);
  }

  /**
   * `NaiveDate::from_yo_opt`.
   * @param {number} year an `i32`
   * @param {number} ordinal a `u32`
   * @returns {NaiveDate | null}
   */
  static fromYoOpt(year, ordinal) {
    if (!inRange(year, I32_MIN, I32_MAX) || !isU32(ordinal)) {
      return null;
    }
    return NaiveDate.fromOrdinal(year, ordinal);
  }

  /**
   * `NaiveDate::from_num_days_from_ce_opt` (day 1 is 0001-01-01).
   * @param {number} days an `i32`
   * @returns {NaiveDate | null}
   */
  static fromNumDaysFromCeOpt(days) {
    if (!inRange(days, I32_MIN, I32_MAX - 365)) {
      return null;
    }
    const [yearDiv400, cycle] = divModFloor(days + 365, 146_097);
    const [yearMod400, ordinal] = cycleToYo(cycle);
    return NaiveDate.fromOrdinal(yearDiv400 * 400 + yearMod400, ordinal);
  }

  /**
   * `NaiveDate::parse_from_str`; throws `Err(ParseError)`.
   * @param {string} s
   * @param {string} fmt
   * @returns {NaiveDate}
   */
  static parseFromStr(s, fmt) {
    const parsed = new Parsed();
    parse(parsed, s, new StrftimeItems(fmt));
    return parsed.toNaiveDate();
  }

  /** `NaiveDate::MIN` (-262143-01-01). */
  static get MIN() {
    return new NaiveDate({ yearValue: MIN_YEAR, ordinalValue: 1 });
  }

  /** `NaiveDate::MAX` (+262142-12-31). */
  static get MAX() {
    return new NaiveDate({ yearValue: MAX_YEAR, ordinalValue: 365 });
  }

  /** `NaiveDate::BEFORE_MIN` (crate-private): the day before `MIN`. */
  static get BEFORE_MIN() {
    return new NaiveDate({ yearValue: MIN_YEAR - 1, ordinalValue: 366 });
  }

  /** `NaiveDate::AFTER_MAX` (crate-private): the day after `MAX`. */
  static get AFTER_MAX() {
    return new NaiveDate({ yearValue: MAX_YEAR + 1, ordinalValue: 1 });
  }

  /** `Default`: 1970-01-01. */
  static default() {
    return expect(NaiveDate.fromYmdOpt(1970, 1, 1), 'called `Option::unwrap()` on a `None` value');
  }

  /** `year`. */
  year() {
    return this.yearValue;
  }

  /** `ordinal` (1-based day of the year). */
  ordinal() {
    return this.ordinalValue;
  }

  /** `ordinal0`. */
  ordinal0() {
    return this.ordinalValue - 1;
  }

  /** `leap_year`. */
  leapYear() {
    return isLeapYear(this.yearValue);
  }

  /**
   * The month and the day of the month (`mdf`).
   * @returns {[number, number]}
   */
  monthDay() {
    const leapDay = this.leapYear() ? 1 : 0;
    for (let month = 12; month > 1; month -= 1) {
      const start = CUMULATIVE_DAYS[month] + (month > 2 ? leapDay : 0);
      if (this.ordinalValue > start) {
        return [month, this.ordinalValue - start];
      }
    }
    return [1, this.ordinalValue];
  }

  /** `month` (1-12). */
  month() {
    return this.monthDay()[0];
  }

  /** `month0` (0-11). */
  month0() {
    return this.monthDay()[0] - 1;
  }

  /** `day` (1-31). */
  day() {
    return this.monthDay()[1];
  }

  /** `day0`. */
  day0() {
    return this.monthDay()[1] - 1;
  }

  /** `weekday().num_days_from_monday()` (0 = Monday). */
  numDaysFromMonday() {
    return (((this.numDaysFromCe() - 1) % 7) + 7) % 7;
  }

  /**
   * `weeks_from(day)`: `day` as days from Monday.
   * @param {number} day
   */
  weeksFrom(day) {
    const since = (this.numDaysFromMonday() - day + 7) % 7;
    return Math.trunc((this.ordinalValue - since + 6) / 7);
  }

  /**
   * `iso_week()`: the ISO year and week.
   * @returns {[number, number]}
   */
  isoWeek() {
    const year = this.yearValue;
    const raw = Math.floor((this.ordinalValue - (this.numDaysFromMonday() + 1) + 10) / 7);
    if (raw < 1) {
      return [year - 1, isoWeeksIn(year - 1)];
    }
    return raw > isoWeeksIn(year) ? [year + 1, 1] : [year, raw];
  }

  /** `num_days_from_ce` (crate-private): 0001-01-01 is day 1. */
  numDaysFromCe() {
    let year = this.yearValue - 1;
    let ndays = 0;
    if (year < 0) {
      const excess = 1 + Math.trunc(-year / 400);
      year += excess * 400;
      ndays -= excess * 146_097;
    }
    const div100 = Math.trunc(year / 100);
    ndays += Math.floor((year * 1461) / 4) - div100 + Math.floor(div100 / 4);
    return ndays + this.ordinalValue;
  }

  /**
   * `add_days` (crate-private).
   * @param {number} days an `i32`
   * @returns {NaiveDate | null}
   */
  addDays(days) {
    const ordinal = this.ordinalValue + days;
    if (ordinal > 0 && ordinal <= (this.leapYear() ? 366 : 365)) {
      return new NaiveDate({ yearValue: this.yearValue, ordinalValue: ordinal });
    }
    const [yearDiv400, yearMod400] = divModFloor(this.yearValue, 400);
    const cycle = yoToCycle(yearMod400, this.ordinalValue) + days;
    if (cycle < I32_MIN || cycle > I32_MAX) {
      return null;
    }
    const [cycleDiv400y, cycleRem] = divModFloor(cycle, 146_097);
    const [newYearMod400, newOrdinal] = cycleToYo(cycleRem);
    return NaiveDate.fromOrdinal((yearDiv400 + cycleDiv400y) * 400 + newYearMod400, newOrdinal);
  }

  /**
   * `checked_add_signed`: whole days of `rhs` only.
   * @param {TimeDelta} rhs
   * @returns {NaiveDate | null}
   */
  checkedAddSigned(rhs) {
    const days = rhs.numDays();
    if (days < BigInt(I32_MIN) || days > BigInt(I32_MAX)) {
      return null;
    }
    return this.addDays(Number(days));
  }

  /**
   * `checked_sub_signed`.
   * @param {TimeDelta} rhs
   * @returns {NaiveDate | null}
   */
  checkedSubSigned(rhs) {
    const days = -rhs.numDays();
    if (days < BigInt(I32_MIN) || days > BigInt(I32_MAX)) {
      return null;
    }
    return this.addDays(Number(days));
  }

  /**
   * `diff_months`: clamps the day to the target month.
   * @param {number} months an `i32`
   * @returns {NaiveDate | null}
   */
  diffMonths(months) {
    const total = this.yearValue * 12 + this.month() - 1 + months;
    if (total < I32_MIN || total > I32_MAX) {
      return null;
    }
    const [year, month0] = divModFloor(total, 12);
    const month = month0 + 1;
    const dayMax = month === 2
      ? (isLeapYear(year) ? 29 : 28)
      : CUMULATIVE_DAYS[month + 1] - CUMULATIVE_DAYS[month];
    return NaiveDate.fromYmdOpt(year, month, Math.min(this.day(), dayMax));
  }

  /**
   * `checked_add_months`.
   * @param {Months} months
   * @returns {NaiveDate | null}
   */
  checkedAddMonths(months) {
    if (months._0 === 0) {
      return this;
    }
    return months._0 <= I32_MAX ? this.diffMonths(months._0) : null;
  }

  /**
   * `checked_sub_months`.
   * @param {Months} months
   * @returns {NaiveDate | null}
   */
  checkedSubMonths(months) {
    if (months._0 === 0) {
      return this;
    }
    return months._0 <= I32_MAX ? this.diffMonths(-months._0) : null;
  }

  /** `succ_opt`. @returns {NaiveDate | null} */
  succOpt() {
    if (this.ordinalValue < (this.leapYear() ? 366 : 365)) {
      return new NaiveDate({ yearValue: this.yearValue, ordinalValue: this.ordinalValue + 1 });
    }
    return NaiveDate.fromYoOpt(this.yearValue + 1, 1);
  }

  /** `pred_opt`. @returns {NaiveDate | null} */
  predOpt() {
    if (this.ordinalValue > 1) {
      return new NaiveDate({ yearValue: this.yearValue, ordinalValue: this.ordinalValue - 1 });
    }
    return NaiveDate.fromYmdOpt(this.yearValue - 1, 12, 31);
  }

  /**
   * `signed_duration_since`: whole days.
   * @param {NaiveDate} rhs
   * @returns {TimeDelta}
   */
  signedDurationSince(rhs) {
    const [year1Div400, year1Mod400] = divModFloor(this.yearValue, 400);
    const [year2Div400, year2Mod400] = divModFloor(rhs.yearValue, 400);
    const cycle1 = yoToCycle(year1Mod400, this.ordinalValue);
    const cycle2 = yoToCycle(year2Mod400, rhs.ordinalValue);
    const days = (year1Div400 - year2Div400) * 146_097 + (cycle1 - cycle2);
    return expect(TimeDelta.tryDays(days), 'always in range');
  }

  /**
   * `years_since`.
   * @param {NaiveDate} base
   * @returns {number | null}
   */
  yearsSince(base) {
    let years = this.yearValue - base.yearValue;
    const [month, day] = this.monthDay();
    const [baseMonth, baseDay] = base.monthDay();
    if (month * 32 + day < baseMonth * 32 + baseDay) {
      years -= 1;
    }
    return years >= 0 ? years : null;
  }

  /**
   * `and_time`.
   * @param {NaiveTime} time
   */
  andTime(time) {
    return new NaiveDateTime({ dateValue: this, timeValue: time });
  }

  /**
   * `and_hms_opt`.
   * @param {number} hour
   * @param {number} min
   * @param {number} sec
   * @returns {NaiveDateTime | null}
   */
  andHmsOpt(hour, min, sec) {
    const time = NaiveTime.fromHmsOpt(hour, min, sec);
    return time === null ? null : this.andTime(time);
  }

  /**
   * `and_hms_milli_opt`.
   * @param {number} hour
   * @param {number} min
   * @param {number} sec
   * @param {number} milli
   * @returns {NaiveDateTime | null}
   */
  andHmsMilliOpt(hour, min, sec, milli) {
    const time = NaiveTime.fromHmsMilliOpt(hour, min, sec, milli);
    return time === null ? null : this.andTime(time);
  }

  /**
   * `and_hms_nano_opt`.
   * @param {number} hour
   * @param {number} min
   * @param {number} sec
   * @param {number} nano
   * @returns {NaiveDateTime | null}
   */
  andHmsNanoOpt(hour, min, sec, nano) {
    const time = NaiveTime.fromHmsNanoOpt(hour, min, sec, nano);
    return time === null ? null : this.andTime(time);
  }

  /**
   * `NaiveDate + TimeDelta` / `NaiveDate + Months`; panics on overflow.
   * @param {TimeDelta | Months} rhs
   * @returns {NaiveDate}
   */
  add(rhs) {
    if (rhs instanceof Months) {
      return expect(this.checkedAddMonths(rhs), '`NaiveDate + Months` out of range');
    }
    return expect(this.checkedAddSigned(rhs), '`NaiveDate + TimeDelta` overflowed');
  }

  /**
   * `NaiveDate - TimeDelta` / `NaiveDate - Months` (a `NaiveDate`, panics on
   * overflow) or `NaiveDate - NaiveDate` (a `TimeDelta`).
   * @template {TimeDelta | Months | NaiveDate} T
   * @param {T} rhs
   * @returns {T extends NaiveDate ? TimeDelta : NaiveDate}
   */
  sub(rhs) {
    if (rhs instanceof NaiveDate) {
      return /** @type {any} */ (this.signedDurationSince(rhs));
    }
    if (rhs instanceof Months) {
      return /** @type {any} */ (expect(this.checkedSubMonths(rhs), '`NaiveDate - Months` out of range'));
    }
    return /** @type {any} */ (expect(this.checkedSubSigned(/** @type {TimeDelta} */ (rhs)),
      '`NaiveDate - TimeDelta` overflowed'));
  }

  /**
   * `format(fmt).to_string()`; panics like `to_string` on a bad format.
   * @param {string} fmt
   */
  format(fmt) {
    return formatToString(this, null, null, fmt);
  }

  /**
   * Derived `PartialEq`.
   * @param {NaiveDate} other
   */
  equals(other) {
    return other instanceof NaiveDate && this.yearValue === other.yearValue
      && this.ordinalValue === other.ordinalValue;
  }

  /**
   * Derived `Ord`.
   * @param {NaiveDate} other
   * @returns {-1 | 0 | 1}
   */
  cmp(other) {
    if (this.yearValue !== other.yearValue) {
      return this.yearValue < other.yearValue ? -1 : 1;
    }
    if (this.ordinalValue !== other.ordinalValue) {
      return this.ordinalValue < other.ordinalValue ? -1 : 1;
    }
    return 0;
  }

  /** `Debug`: `2015-09-05`, `-0001-01-01`, `+10000-01-01`. */
  fmtDebug() {
    const [month, day] = this.monthDay();
    return `${debugYear(this.yearValue)}-${two(month)}-${two(day)}`;
  }

  /** `Display` (same as `Debug`). */
  toString() {
    return this.fmtDebug();
  }

  /** `Serialize`: the `Display` string. */
  toJSON() {
    return this.toString();
  }
}
