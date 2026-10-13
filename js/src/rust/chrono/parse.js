/**
 * chrono 0.4.45 parsing: `ParseError` (src/format/mod.rs), the scanners of
 * src/format/scan.rs, `Parsed` (src/format/parsed.rs) and `parse` /
 * `parse_rfc3339` (src/format/parse.rs).
 *
 * Inputs are JavaScript strings; wherever chrono compares a byte length
 * (`s.len()`), the port compares the UTF-8 length, so the errors match.
 *
 * Supported by `parse`: literals, spaces, `%Y %C %y %m %b %B %h %d %e %j %q
 * %a %A %u %w %H %k %I %l %p %P %M %S %f %.f %.3f %.6f %.9f %3f %6f %9f %Z %z
 * %:z %::z %:::z %#z` and the expansions `%D %F %R %T %X %x %c %r %v`. The
 * week-number, ISO-week and timestamp fields (`%U %W %V %G %g %s`), `%+` and
 * RFC 2822 items panic: the crate never parses them.
 */

import { err, panic } from '../core.js';
import { isWhitespace, trimStart, utf8Len } from '../str.js';
import { FixedOffset } from './offset.js';
import { NaiveDate, NaiveTime } from './naive.js';

const I32_MIN = -(2 ** 31);
const I32_MAX = 2 ** 31 - 1;

/** chrono `ParseErrorKind` names mapped to their `Display` text. */
const MESSAGES = Object.freeze({
  OutOfRange: 'input is out of range',
  Impossible: 'no possible date and time matching input',
  NotEnough: 'input is not enough for unique date and time',
  Invalid: 'input contains invalid characters',
  TooShort: 'premature end of input',
  TooLong: 'trailing input',
  BadFormat: 'bad or unsupported format string',
});

/** @typedef {keyof typeof MESSAGES} ParseErrorKind */

/** chrono `ParseError(ParseErrorKind)`. */
export class ParseError {
  /** @param {ParseErrorKind} kind */
  constructor(kind) {
    /** @type {ParseErrorKind} */
    this._0 = kind;
    Object.freeze(this);
  }

  /** `kind`. */
  kind() {
    return this._0;
  }

  /**
   * Derived `PartialEq`.
   * @param {unknown} other
   */
  equals(other) {
    return other instanceof ParseError && other._0 === this._0;
  }

  /** `Display`. */
  toString() {
    return MESSAGES[this._0];
  }

  /** Derived `Debug`: `ParseError(Invalid)`. */
  fmtDebug() {
    return `ParseError(${this._0})`;
  }
}

export const OUT_OF_RANGE = new ParseError('OutOfRange');
export const IMPOSSIBLE = new ParseError('Impossible');
export const NOT_ENOUGH = new ParseError('NotEnough');
export const INVALID = new ParseError('Invalid');
export const TOO_SHORT = new ParseError('TooShort');
export const TOO_LONG = new ParseError('TooLong');
export const BAD_FORMAT = new ParseError('BadFormat');

/**
 * Whether `c` (a UTF-16 code unit, or NaN past the end) is an ASCII digit.
 * @param {number} c
 */
function isDigitCode(c) {
  return c >= 0x30 && c <= 0x39;
}

/**
 * `scan::number`: at least `min` and at most `max` ASCII digits.
 * @param {string} s
 * @param {number} min
 * @param {number} max
 * @returns {[string, number]}
 */
export function scanNumber(s, min, max) {
  if (utf8Len(s) < min) {
    return err(TOO_SHORT);
  }
  let n = 0;
  const limit = Math.min(max, s.length);
  for (let i = 0; i < limit; i += 1) {
    const c = s.charCodeAt(i);
    if (!isDigitCode(c)) {
      return i < min ? err(INVALID) : [s.slice(i), n];
    }
    n = n * 10 + (c - 0x30);
    if (n > Number.MAX_SAFE_INTEGER) {
      // i64 overflow, or a value no setter but the unsupported timestamp accepts
      return err(OUT_OF_RANGE);
    }
  }
  return [s.slice(limit), n];
}

const SCALE = [0, 100_000_000, 10_000_000, 1_000_000, 100_000, 10_000, 1000, 100, 10, 1];

/**
 * `scan::nanosecond`: 1 to 9 significant digits, further digits skipped.
 * @param {string} s
 * @returns {[string, number]}
 */
export function scanNanosecond(s) {
  const [rest, value] = scanNumber(s, 1, 9);
  const consumed = s.length - rest.length;
  let index = 0;
  while (isDigitCode(rest.charCodeAt(index))) {
    index += 1;
  }
  return [rest.slice(index), value * SCALE[consumed]];
}

/**
 * `scan::nanosecond_fixed`: exactly `digits` digits.
 * @param {string} s
 * @param {number} digits
 * @returns {[string, number]}
 */
function scanNanosecondFixed(s, digits) {
  const [rest, value] = scanNumber(s, digits, digits);
  return [rest, value * SCALE[digits]];
}

/**
 * The lowercased ASCII letters of the first `count` code units of `s`.
 * @param {string} s
 * @param {number} count
 */
function lowerPrefix(s, count) {
  let out = '';
  for (let i = 0; i < count; i += 1) {
    const c = s.charCodeAt(i);
    out += String.fromCharCode(c >= 0x41 && c <= 0x5a ? c | 32 : c);
  }
  return out;
}

const SHORT_MONTHS = ['jan', 'feb', 'mar', 'apr', 'may', 'jun', 'jul', 'aug', 'sep', 'oct', 'nov', 'dec'];
const LONG_MONTH_SUFFIXES = ['uary', 'ruary', 'ch', 'il', '', 'e', 'y', 'ust', 'tember', 'ober', 'ember', 'ember'];
const SHORT_WEEKDAYS = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'];
const LONG_WEEKDAY_SUFFIXES = ['day', 'sday', 'nesday', 'rsday', 'day', 'urday', 'day'];

/**
 * A three-letter ASCII case-insensitive prefix from `names`.
 * @param {string} s
 * @param {string[]} names
 * @returns {[string, number]}
 */
function scanShortName(s, names) {
  if (utf8Len(s) < 3) {
    return err(TOO_SHORT);
  }
  const index = names.indexOf(lowerPrefix(s, 3));
  return index < 0 ? err(INVALID) : [s.slice(3), index];
}

/**
 * A short name optionally followed by the rest of the long name.
 * @param {string} s
 * @param {string[]} names
 * @param {string[]} suffixes
 * @returns {[string, number]}
 */
function scanShortOrLongName(s, names, suffixes) {
  const [rest, index] = scanShortName(s, names);
  const suffix = suffixes[index];
  if (rest.length >= suffix.length && lowerPrefix(rest, suffix.length) === suffix) {
    return [rest.slice(suffix.length), index];
  }
  return [rest, index];
}

/**
 * `scan::short_month0`.
 * @param {string} s
 */
export function scanShortMonth0(s) {
  return scanShortName(s, SHORT_MONTHS);
}

/**
 * `scan::short_or_long_month0` (`"Sept"` reads as September plus `t`).
 * @param {string} s
 */
export function scanShortOrLongMonth0(s) {
  return scanShortOrLongName(s, SHORT_MONTHS, LONG_MONTH_SUFFIXES);
}

/**
 * `scan::char`.
 * @param {string} s
 * @param {string} c an ASCII character
 */
function scanChar(s, c) {
  if (s.length === 0) {
    return err(TOO_SHORT);
  }
  return s[0] === c ? s.slice(1) : err(INVALID);
}

/**
 * `scan::colon_or_space`: skips any run of colons and whitespace.
 * @param {string} s
 */
function colonOrSpace(s) {
  let index = 0;
  for (const c of s) {
    if (c !== ':' && !isWhitespace(c)) {
      break;
    }
    index += c.length;
  }
  return s.slice(index);
}

/**
 * The two leading bytes when they are both ASCII digits.
 * @param {string} s
 */
function twoDigits(s) {
  const a = s.charCodeAt(0);
  const b = s.charCodeAt(1);
  return isDigitCode(a) && isDigitCode(b) ? (a - 0x30) * 10 + (b - 0x30) : null;
}

/**
 * `scan::timezone_offset`: `[+-−]HH[:]MM` (or `Z`) in seconds.
 * @param {string} input
 * @param {(s: string) => string} consumeColon
 * @param {boolean} allowZulu
 * @param {boolean} allowMissingMinutes
 * @param {boolean} allowTzMinusSign
 * @returns {[string, number]}
 */
export function scanTimezoneOffset(input, consumeColon, allowZulu, allowMissingMinutes, allowTzMinusSign) {
  let s = input;
  if (allowZulu && (s[0] === 'Z' || s[0] === 'z')) {
    return [s.slice(1), 0];
  }
  let negative;
  if (s.length === 0) {
    return err(TOO_SHORT);
  } else if (s[0] === '+' || s[0] === '-') {
    negative = s[0] === '-';
    s = s.slice(1);
  } else if (s[0] === '−') {
    if (!allowTzMinusSign) {
      return err(INVALID);
    }
    negative = true;
    s = s.slice(1);
  } else {
    return err(INVALID);
  }
  if (utf8Len(s) < 2) {
    return err(TOO_SHORT);
  }
  const hours = twoDigits(s);
  if (hours === null) {
    return err(INVALID);
  }
  s = consumeColon(s.slice(2));
  let minutes;
  const length = utf8Len(s);
  if (length >= 2) {
    const m1 = s.charCodeAt(0);
    if (m1 >= 0x30 && m1 <= 0x35 && isDigitCode(s.charCodeAt(1))) {
      minutes = (m1 - 0x30) * 10 + (s.charCodeAt(1) - 0x30);
    } else if (m1 >= 0x36 && m1 <= 0x39 && isDigitCode(s.charCodeAt(1))) {
      return err(OUT_OF_RANGE);
    } else {
      return err(INVALID);
    }
    s = s.slice(2);
  } else if (allowMissingMinutes) {
    minutes = 0;
    if (length === 1) {
      return err(TOO_SHORT);
    }
  } else {
    return err(TOO_SHORT);
  }
  const seconds = hours * 3600 + minutes * 60;
  return [s, negative ? -seconds : seconds];
}

/**
 * `set_if_consistent`.
 * @param {Parsed} parsed
 * @param {'year' | 'yearDiv100' | 'yearMod100' | 'quarter' | 'month' | 'weekday' | 'ordinal'
 *   | 'day' | 'hourDiv12' | 'hourMod12' | 'minute' | 'second' | 'nanosecond' | 'offset'} field
 * @param {number} value
 */
function setIfConsistent(parsed, field, value) {
  const old = parsed[field];
  if (old !== null && old !== value) {
    err(IMPOSSIBLE);
  }
  parsed[field] = value;
}

/**
 * Throws `OUT_OF_RANGE` unless `value` is in `min..=max`.
 * @param {number} value
 * @param {number} min
 * @param {number} max
 */
function checkRange(value, min, max) {
  if (!(value >= min && value <= max)) {
    err(OUT_OF_RANGE);
  }
}

/**
 * chrono `Parsed`: the fields a format string has read so far. Weekdays are
 * stored as days from Monday (0-6). The week-number, ISO-week and timestamp
 * fields are absent because their items are not supported.
 */
export class Parsed {
  constructor() {
    /** @type {number | null} */
    this.year = null;
    /** @type {number | null} */
    this.yearDiv100 = null;
    /** @type {number | null} */
    this.yearMod100 = null;
    /** @type {number | null} */
    this.quarter = null;
    /** @type {number | null} */
    this.month = null;
    /** @type {number | null} */
    this.weekday = null;
    /** @type {number | null} */
    this.ordinal = null;
    /** @type {number | null} */
    this.day = null;
    /** @type {number | null} */
    this.hourDiv12 = null;
    /** @type {number | null} */
    this.hourMod12 = null;
    /** @type {number | null} */
    this.minute = null;
    /** @type {number | null} */
    this.second = null;
    /** @type {number | null} */
    this.nanosecond = null;
    /** @type {number | null} */
    this.offset = null;
  }

  /** @param {number} value */
  setYear(value) {
    checkRange(value, I32_MIN, I32_MAX);
    setIfConsistent(this, 'year', value);
  }

  /** @param {number} value */
  setYearDiv100(value) {
    checkRange(value, 0, I32_MAX);
    setIfConsistent(this, 'yearDiv100', value);
  }

  /** @param {number} value */
  setYearMod100(value) {
    checkRange(value, 0, 99);
    setIfConsistent(this, 'yearMod100', value);
  }

  /** @param {number} value */
  setQuarter(value) {
    checkRange(value, 1, 4);
    setIfConsistent(this, 'quarter', value);
  }

  /** @param {number} value */
  setMonth(value) {
    checkRange(value, 1, 12);
    setIfConsistent(this, 'month', value);
  }

  /** @param {number} value days from Monday */
  setWeekday(value) {
    setIfConsistent(this, 'weekday', value);
  }

  /** @param {number} value */
  setOrdinal(value) {
    checkRange(value, 1, 366);
    setIfConsistent(this, 'ordinal', value);
  }

  /** @param {number} value */
  setDay(value) {
    checkRange(value, 1, 31);
    setIfConsistent(this, 'day', value);
  }

  /** @param {boolean} value */
  setAmpm(value) {
    setIfConsistent(this, 'hourDiv12', value ? 1 : 0);
  }

  /** @param {number} value */
  setHour12(value) {
    checkRange(value, 1, 12);
    setIfConsistent(this, 'hourMod12', value === 12 ? 0 : value);
  }

  /** @param {number} value */
  setHour(value) {
    checkRange(value, 0, 23);
    setIfConsistent(this, 'hourDiv12', value < 12 ? 0 : 1);
    setIfConsistent(this, 'hourMod12', value % 12);
  }

  /** @param {number} value */
  setMinute(value) {
    checkRange(value, 0, 59);
    setIfConsistent(this, 'minute', value);
  }

  /** @param {number} value */
  setSecond(value) {
    checkRange(value, 0, 60);
    setIfConsistent(this, 'second', value);
  }

  /** @param {number} value */
  setNanosecond(value) {
    checkRange(value, 0, 999_999_999);
    setIfConsistent(this, 'nanosecond', value);
  }

  /** @param {number} value */
  setOffset(value) {
    checkRange(value, I32_MIN, I32_MAX);
    setIfConsistent(this, 'offset', value);
  }

  /**
   * `resolve_year` for the calendar year.
   * @returns {number | null}
   */
  resolveYear() {
    const y = this.year;
    const q = this.yearDiv100;
    const r = this.yearMod100;
    if (q === null && r === null) {
      return y;
    }
    if (y !== null && (r === null || (r >= 0 && r <= 99))) {
      if (y < 0) {
        return err(IMPOSSIBLE);
      }
      const q2 = Math.trunc(y / 100);
      const r2 = y % 100;
      return (q ?? q2) === q2 && (r ?? r2) === r2 ? y : err(IMPOSSIBLE);
    }
    if (y === null && q !== null && r !== null && r >= 0 && r <= 99) {
      if (q < 0) {
        return err(IMPOSSIBLE);
      }
      const year = q * 100 + r;
      return year > I32_MAX ? err(OUT_OF_RANGE) : year;
    }
    if (y === null && q === null && r !== null && r >= 0 && r <= 99) {
      return r + (r < 70 ? 2000 : 1900);
    }
    return err(r === null ? NOT_ENOUGH : OUT_OF_RANGE);
  }

  /**
   * `to_naive_date`; throws `Err(ParseError)`.
   * @returns {NaiveDate}
   */
  toNaiveDate() {
    const givenYear = this.resolveYear();
    const verifyYmd = (/** @type {NaiveDate} */ date) => {
      const year = date.year();
      const div100 = year >= 0 ? Math.trunc(year / 100) : null;
      const mod100 = year >= 0 ? year % 100 : null;
      return (this.year ?? year) === year
        && (this.yearDiv100 ?? div100) === div100
        && (this.yearMod100 ?? mod100) === mod100
        && (this.month ?? date.month()) === date.month()
        && (this.day ?? date.day()) === date.day();
    };
    // Of the ISO week date only the weekday can be parsed.
    const verifyIsoweekdate = (/** @type {NaiveDate} */ date) => (
      this.weekday ?? date.numDaysFromMonday()) === date.numDaysFromMonday();
    // Of the ordinal and week dates only the ordinal can be parsed.
    const verifyOrdinal = (/** @type {NaiveDate} */ date) => (
      this.ordinal ?? date.ordinal()) === date.ordinal();
    let date;
    let verified;
    if (givenYear !== null && this.month !== null && this.day !== null) {
      date = NaiveDate.fromYmdOpt(givenYear, this.month, this.day) ?? err(OUT_OF_RANGE);
      verified = verifyIsoweekdate(date) && verifyOrdinal(date);
    } else if (givenYear !== null && this.ordinal !== null) {
      date = NaiveDate.fromYoOpt(givenYear, this.ordinal) ?? err(OUT_OF_RANGE);
      verified = verifyYmd(date) && verifyIsoweekdate(date) && verifyOrdinal(date);
    } else {
      return err(NOT_ENOUGH);
    }
    if (!verified || (this.quarter !== null && this.quarter !== Math.floor(date.month0() / 3) + 1)) {
      return err(IMPOSSIBLE);
    }
    return date;
  }

  /**
   * `to_naive_time`; throws `Err(ParseError)`.
   * @returns {NaiveTime}
   */
  toNaiveTime() {
    const hourDiv12 = this.hourDiv12 ?? err(NOT_ENOUGH);
    checkRange(hourDiv12, 0, 1);
    const hourMod12 = this.hourMod12 ?? err(NOT_ENOUGH);
    checkRange(hourMod12, 0, 11);
    const minute = this.minute ?? err(NOT_ENOUGH);
    checkRange(minute, 0, 59);
    let second = this.second ?? 0;
    let nano = 0;
    if (second === 60) {
      second = 59;
      nano = 1_000_000_000;
    } else {
      checkRange(second, 0, 59);
    }
    if (this.nanosecond !== null) {
      checkRange(this.nanosecond, 0, 999_999_999);
      if (this.second === null) {
        return err(NOT_ENOUGH);
      }
      nano += this.nanosecond;
    }
    return NaiveTime.fromHmsNanoOpt(hourDiv12 * 12 + hourMod12, minute, second, nano) ?? err(OUT_OF_RANGE);
  }

  /**
   * `to_naive_datetime_with_offset` (without a timestamp field the offset
   * plays no role); the date error wins over the time error.
   * @param {number} _offset
   */
  toNaiveDatetimeWithOffset(_offset) {
    const date = this.toNaiveDate();
    return date.andTime(this.toNaiveTime());
  }
}

/**
 * Panics for an item `parse` does not support.
 * @param {string} name
 * @returns {never}
 */
function unsupported(name) {
  return panic(`chrono port: parsing ${name} is not supported`);
}

/** Widths of the numeric fields `parse` supports, and whether they take a sign. */
const NUMERIC = new Map([
  ['Year', 4], ['YearDiv100', 2], ['YearMod100', 2], ['Quarter', 1], ['Month', 2], ['Day', 2],
  ['NumDaysFromSun', 1], ['WeekdayFromMon', 1], ['Ordinal', 3], ['Hour', 2], ['Hour12', 2],
  ['Minute', 2], ['Second', 2], ['Nanosecond', 9],
]);

/**
 * Stores a numeric field.
 * @param {Parsed} parsed
 * @param {string} numeric
 * @param {number} v
 */
function setNumeric(parsed, numeric, v) {
  switch (numeric) {
    case 'Year': return parsed.setYear(v);
    case 'YearDiv100': return parsed.setYearDiv100(v);
    case 'YearMod100': return parsed.setYearMod100(v);
    case 'Quarter': return parsed.setQuarter(v);
    case 'Month': return parsed.setMonth(v);
    case 'Day': return parsed.setDay(v);
    case 'NumDaysFromSun':
      checkRange(v, 0, 6);
      return parsed.setWeekday((v + 6) % 7);
    case 'WeekdayFromMon':
      checkRange(v, 1, 7);
      return parsed.setWeekday(v - 1);
    case 'Ordinal': return parsed.setOrdinal(v);
    case 'Hour': return parsed.setHour(v);
    case 'Hour12': return parsed.setHour12(v);
    case 'Minute': return parsed.setMinute(v);
    case 'Second': return parsed.setSecond(v);
    default: return parsed.setNanosecond(v);
  }
}

/**
 * Parses a `Fixed` item at the start of `s`.
 * @param {Parsed} parsed
 * @param {string} name
 * @param {string} s
 * @returns {string} the remainder
 */
function parseFixed(parsed, name, s) {
  /** @type {[string, number]} */
  let scanned;
  switch (name) {
    case 'ShortMonthName':
    case 'LongMonthName':
      scanned = name === 'ShortMonthName' ? scanShortMonth0(s) : scanShortOrLongMonth0(s);
      parsed.setMonth(scanned[1] + 1);
      return scanned[0];
    case 'ShortWeekdayName':
    case 'LongWeekdayName':
      scanned = name === 'ShortWeekdayName'
        ? scanShortName(s, SHORT_WEEKDAYS)
        : scanShortOrLongName(s, SHORT_WEEKDAYS, LONG_WEEKDAY_SUFFIXES);
      parsed.setWeekday(scanned[1]);
      return scanned[0];
    case 'LowerAmPm':
    case 'UpperAmPm': {
      if (utf8Len(s) < 2) {
        return err(TOO_SHORT);
      }
      const ampm = lowerPrefix(s, 2);
      if (ampm !== 'am' && ampm !== 'pm') {
        return err(INVALID);
      }
      parsed.setAmpm(ampm === 'pm');
      return s.slice(2);
    }
    case 'Nanosecond':
    case 'Nanosecond3':
    case 'Nanosecond6':
    case 'Nanosecond9':
      if (!s.startsWith('.')) {
        return s;
      }
      scanned = name === 'Nanosecond' ? scanNanosecond(s.slice(1)) : scanNanosecondFixed(s.slice(1), Number(name[10]));
      parsed.setNanosecond(scanned[1]);
      return scanned[0];
    case 'Nanosecond3NoDot':
    case 'Nanosecond6NoDot':
    case 'Nanosecond9NoDot': {
      const digits = Number(name[10]);
      if (utf8Len(s) < digits) {
        return err(TOO_SHORT);
      }
      scanned = scanNanosecondFixed(s, digits);
      parsed.setNanosecond(scanned[1]);
      return scanned[0];
    }
    case 'TimezoneName': {
      let index = 0;
      for (const c of s) {
        if (isWhitespace(c)) {
          break;
        }
        index += c.length;
      }
      return s.slice(index);
    }
    case 'TimezoneOffset':
    case 'TimezoneOffsetColon':
    case 'TimezoneOffsetDoubleColon':
    case 'TimezoneOffsetTripleColon':
    case 'TimezoneOffsetPermissive': {
      const permissive = name === 'TimezoneOffsetPermissive';
      scanned = scanTimezoneOffset(trimStart(s), colonOrSpace, permissive, permissive, true);
      parsed.setOffset(scanned[1]);
      return scanned[0];
    }
    default:
      return unsupported(name);
  }
}

/**
 * `parse`: fills `parsed` from `s` according to `items`; throws
 * `Err(ParseError)`, `TOO_LONG` for unparsed trailing input.
 * @param {Parsed} parsed
 * @param {string} input
 * @param {Iterable<import('./strftime.js').Item>} items
 */
export function parse(parsed, input, items) {
  let s = input;
  for (const item of items) {
    switch (item.tag) {
      case 'Literal':
        if (utf8Len(s) < utf8Len(item.value)) {
          return err(TOO_SHORT);
        }
        if (!s.startsWith(item.value)) {
          return err(INVALID);
        }
        s = s.slice(item.value.length);
        break;
      case 'Space':
        s = trimStart(s);
        break;
      case 'Numeric': {
        const width = NUMERIC.get(item.numeric);
        if (width === undefined) {
          return unsupported(item.numeric);
        }
        s = trimStart(s);
        let scanned;
        if (item.numeric === 'Year' && (s.startsWith('-') || s.startsWith('+'))) {
          scanned = scanNumber(s.slice(1), 1, Infinity);
          if (s.startsWith('-')) {
            scanned[1] = -scanned[1];
          }
        } else {
          scanned = scanNumber(s, 1, width);
        }
        s = scanned[0];
        setNumeric(parsed, item.numeric, scanned[1]);
        break;
      }
      case 'Fixed':
        s = parseFixed(parsed, item.fixed, s);
        break;
      default:
        return err(BAD_FORMAT);
    }
  }
  if (s.length > 0) {
    err(TOO_LONG);
  }
}

/**
 * `parse_rfc3339`: the strict RFC 3339 parser behind
 * `DateTime::parse_from_rfc3339`; throws `Err(ParseError)`.
 * @param {string} input
 * @returns {import('./offset.js').DateTime}
 */
export function parseRfc3339(input) {
  if (utf8Len(input) < 19) {
    return err(TOO_SHORT);
  }
  const digit = (/** @type {number} */ index) => {
    const c = input.charCodeAt(index);
    return isDigitCode(c) ? c - 0x30 : err(INVALID);
  };
  const separator = (/** @type {number} */ index, /** @type {string} */ chars) => {
    if (!chars.includes(input[index])) {
      err(INVALID);
    }
  };
  // Non-ASCII characters among the first 19 bytes fail as INVALID before
  // their UTF-16 index could differ from the byte index.
  const year = digit(0) * 1000 + digit(1) * 100 + digit(2) * 10 + digit(3);
  separator(4, '-');
  const month = digit(5) * 10 + digit(6);
  separator(7, '-');
  const day = digit(8) * 10 + digit(9);
  const date = NaiveDate.fromYmdOpt(year, month, day) ?? err(OUT_OF_RANGE);
  separator(10, 'tT ');
  const hour = digit(11) * 10 + digit(12);
  separator(13, ':');
  const min = digit(14) * 10 + digit(15);
  separator(16, ':');
  let sec = digit(17) * 10 + digit(18);
  let nano = 0;
  if (sec === 60) {
    sec = 59;
    nano = 1_000_000_000;
  }
  let s;
  if (input[19] === '.') {
    const [rest, nanosecond] = scanNanosecond(input.slice(20));
    s = rest;
    nano += nanosecond;
  } else {
    s = input.slice(19);
  }
  const time = NaiveTime.fromHmsNanoOpt(hour, min, sec, nano) ?? err(OUT_OF_RANGE);
  const [rest, offset] = scanTimezoneOffset(s, (x) => scanChar(x, ':'), true, false, true);
  if (rest.length > 0) {
    return err(TOO_LONG);
  }
  const tz = FixedOffset.eastOpt(offset) ?? err(OUT_OF_RANGE);
  return date.andTime(time).andLocalTimezone(tz).single() ?? panic('internal error: entered unreachable code');
}
