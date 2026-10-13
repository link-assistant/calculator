// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=d0b1c31b62890307892d53a8e0ca02b5ab30e59af8005f4f6776c6233bbe815f bytes=10480

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * chrono 0.4.45 formatting (src/format/formatting.rs): `DelayedFormat`
// |  * without locales, `OffsetFormat` and `write_rfc3339`.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=d576713da7ed19555e80d72f594e9dac2ab963b6efa8b5ca88c57e391055f6b6
// | import { panic } from '../core.js';
use crate::rust::core_::panic;
// meta-language:translated JavaScript import_statement items=1 sha256=860482e701847c72b98b1ed244ed02b720c2c8f9ea10bc6d91ae354229e4fb37
// | import { StrftimeItems } from './strftime.js';
use crate::rust::chrono::strftime::StrftimeItems;
// meta-language:translated JavaScript import_statement items=1 sha256=151e5aed61e83957a340ca4b4816e004e5bc67db9c3ed2bb12164f8f59e0164a
// | import { autoSiFraction } from './util.js';
use crate::rust::chrono::util::autoSiFraction;

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {import('./date.js').NaiveDate} NaiveDate */
// | /** @typedef {import('./time.js').NaiveTime} NaiveTime */
// | /** @typedef {import('./datetime.js').NaiveDateTime} NaiveDateTime */
// | /**
// |  * The `(name, fix)` pair `DelayedFormat::new_with_offset` stores.
// |  * @typedef {{ name: string, fix: number }} OffsetInfo
// |  */

// meta-language:translated JavaScript lexical_declaration items=1 sha256=49e151b202d3d20ccc5a05e9a9c82c5602523e2cc75b8173d3b190ecb2b97a8c
// | const SHORT_MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
pub static SHORT_MONTHS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("Jan"), String::from("Feb"), String::from("Mar"), String::from("Apr"), String::from("May"), String::from("Jun"), String::from("Jul"), String::from("Aug"), String::from("Sep"), String::from("Oct"), String::from("Nov"), String::from("Dec")]);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=6f3784c81ccab96bee48bb02f5b864d30c04c02daccd084c1e64fe90fdcf5965
// | const LONG_MONTHS = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August',
// |   'September', 'October', 'November', 'December'];
pub static LONG_MONTHS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("January"), String::from("February"), String::from("March"), String::from("April"), String::from("May"), String::from("June"), String::from("July"), String::from("August"), String::from("September"), String::from("October"), String::from("November"), String::from("December")]);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=3d14a6a7cee9fe48c9d9d417147f4bb7786084917ec4a8e7d43f94ee00a7dea1
// | const SHORT_WEEKDAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
pub static SHORT_WEEKDAYS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("Sun"), String::from("Mon"), String::from("Tue"), String::from("Wed"), String::from("Thu"), String::from("Fri"), String::from("Sat")]);
// meta-language:translated JavaScript lexical_declaration items=1 sha256=bb6f861a51f0eac7934c3367f279b3947636169b668b0ff4b95a3e714714d5fa
// | const LONG_WEEKDAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
pub static LONG_WEEKDAYS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("Sunday"), String::from("Monday"), String::from("Tuesday"), String::from("Wednesday"), String::from("Thursday"), String::from("Friday"), String::from("Saturday")]);

// meta-language:carried JavaScript export_statement (unsupported)
// | /** `SecondsFormat`. */
// | export const SecondsFormat = Object.freeze({
// |   Secs: 'Secs', Millis: 'Millis', Micros: 'Micros', Nanos: 'Nanos', AutoSi: 'AutoSi',
// | });

// meta-language:carried JavaScript class_declaration (unsupported)
// | /** Thrown internally for `fmt::Error`. */
// | class FmtError {}

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @returns {never} */
// | function fail() {
// |   throw new FmtError();
// | }

// meta-language:translated JavaScript function_declaration items=1 sha256=891268ea0fd3ff81452b0f7d7db8e474dc661848b687785bf7fa0de0f6c66517
// | /**
// |  * Wraps a value to `u8` like `as u8`.
// |  * @param {number} v
// |  */
// | function u8(v) {
// |   return ((v % 256) + 256) % 256;
// | }
pub fn u8_(v: f64) -> f64 {
    (((v % 256f64) + 256f64) % 256f64)
}

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `write_hundreds`.
// |  * @param {number} n a `u8`
// |  */
// | function hundreds(n) {
// |   return n >= 100 ? fail() : String(n).padStart(2, '0');
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `write_two`.
// |  * @param {number} value
// |  * @param {string} pad
// |  */
// | function writeTwo(value, pad) {
// |   const v = u8(value);
// |   const tens = Math.floor(v / 10);
// |   let out;
// |   if (tens === 0) {
// |     out = pad === 'None' ? '' : pad === 'Space' ? ' ' : '0';
// |   } else {
// |     out = String.fromCharCode(48 + tens);
// |   }
// |   return out + String.fromCharCode(48 + (v % 10));
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `write_n`.
// |  * @param {number} n
// |  * @param {number} v
// |  * @param {string} pad
// |  * @param {boolean} sign
// |  */
// | function writeN(n, v, pad, sign) {
// |   const digits = String(Math.abs(v));
// |   const signText = v < 0 ? '-' : sign ? '+' : '';
// |   const width = sign ? n + 1 : n;
// |   if (pad === 'Zero') {
// |     return signText + digits.padStart(width - signText.length, '0');
// |   }
// |   if (pad === 'Space') {
// |     return (signText + digits).padStart(width, ' ');
// |   }
// |   return signText + digits;
// | }

// meta-language:carried JavaScript function_declaration (type)
// | /**
// |  * `write_year`.
// |  * @param {number} year
// |  * @param {string} pad
// |  */
// | function writeYear(year, pad) {
// |   if (year >= 1000 && year <= 9999) {
// |     return String(year);
// |   }
// |   return writeN(4, year, pad, !(year >= 0 && year < 10000));
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `OffsetFormat::format` with `Pad::Zero`.
// |  * @param {number} offset seconds east of UTC
// |  * @param {'Hours' | 'Minutes' | 'Seconds'} precision
// |  * @param {boolean} colons
// |  * @param {boolean} allowZulu
// |  */
// | export function formatOffset(offset, precision, colons, allowZulu) {
// |   if (allowZulu && offset === 0) {
// |     return 'Z';
// |   }
// |   const sign = offset < 0 ? '-' : '+';
// |   const off = Math.abs(offset);
// |   let hours;
// |   let mins = 0;
// |   let secs = 0;
// |   if (precision === 'Hours') {
// |     hours = Math.floor(off / 3600);
// |   } else if (precision === 'Minutes') {
// |     const minutes = Math.floor((off + 30) / 60);
// |     mins = minutes % 60;
// |     hours = Math.floor(minutes / 60);
// |   } else {
// |     const minutes = Math.floor(off / 60);
// |     secs = off % 60;
// |     mins = minutes % 60;
// |     hours = Math.floor(minutes / 60);
// |   }
// |   let out = hours < 10 ? `${sign}0${hours}` : sign + hundreds(hours);
// |   if (precision !== 'Hours') {
// |     out += (colons ? ':' : '') + hundreds(mins);
// |   }
// |   if (precision === 'Seconds') {
// |     out += (colons ? ':' : '') + hundreds(secs);
// |   }
// |   return out;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `write_rfc3339`.
// |  * @param {NaiveDateTime} dt
// |  * @param {number} offset
// |  * @param {string} secform a `SecondsFormat`
// |  * @param {boolean} useZ
// |  */
// | export function writeRfc3339(dt, offset, secform, useZ) {
// |   try {
// |     const date = dt.date();
// |     const year = date.year();
// |     let out = year >= 0 && year <= 9999
// |       ? String(year).padStart(4, '0')
// |       : writeN(4, year, 'Zero', true);
// |     out += `-${hundreds(date.month())}-${hundreds(date.day())}T`;
// |     const time = dt.time();
// |     let sec = time.second();
// |     let nano = time.nanosecond();
// |     if (nano >= 1_000_000_000) {
// |       sec += 1;
// |       nano -= 1_000_000_000;
// |     }
// |     out += `${hundreds(time.hour())}:${hundreds(time.minute())}:${hundreds(sec)}`;
// |     if (secform === 'Millis') {
// |       out += `.${String(Math.floor(nano / 1_000_000)).padStart(3, '0')}`;
// |     } else if (secform === 'Micros') {
// |       out += `.${String(Math.floor(nano / 1000)).padStart(6, '0')}`;
// |     } else if (secform === 'Nanos') {
// |       out += `.${String(nano).padStart(9, '0')}`;
// |     } else if (secform === 'AutoSi') {
// |       out += autoSiFraction(nano);
// |     }
// |     return out + formatOffset(offset, 'Minutes', true, useZ);
// |   } catch (e) {
// |     if (e instanceof FmtError) {
// |       return null;
// |     }
// |     throw e;
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Formats one numeric item.
// |  * @param {string} numeric
// |  * @param {string} pad
// |  * @param {NaiveDate | null} d
// |  * @param {NaiveTime | null} t
// |  * @param {OffsetInfo | null} off
// |  */
// | function formatNumeric(numeric, pad, d, t, off) {
// |   if (d !== null) {
// |     switch (numeric) {
// |       case 'Year': return writeYear(d.year(), pad);
// |       case 'YearDiv100': return writeTwo(Math.floor(d.year() / 100), pad);
// |       case 'YearMod100': return writeTwo(((d.year() % 100) + 100) % 100, pad);
// |       case 'IsoYear': return writeYear(d.isoWeek()[0], pad);
// |       case 'IsoYearDiv100': return writeTwo(Math.floor(d.isoWeek()[0] / 100), pad);
// |       case 'IsoYearMod100': return writeTwo(((d.isoWeek()[0] % 100) + 100) % 100, pad);
// |       case 'Quarter': return String(Math.floor(d.month0() / 3) + 1);
// |       case 'Month': return writeTwo(d.month(), pad);
// |       case 'Day': return writeTwo(d.day(), pad);
// |       case 'WeekFromSun': return writeTwo(d.weeksFrom(6), pad);
// |       case 'WeekFromMon': return writeTwo(d.weeksFrom(0), pad);
// |       case 'IsoWeek': return writeTwo(d.isoWeek()[1], pad);
// |       case 'NumDaysFromSun': return String((d.numDaysFromMonday() + 1) % 7);
// |       case 'WeekdayFromMon': return String(d.numDaysFromMonday() + 1);
// |       case 'Ordinal': return writeN(3, d.ordinal(), pad, false);
// |       default:
// |     }
// |   }
// |   if (t !== null) {
// |     switch (numeric) {
// |       case 'Hour': return writeTwo(t.hour(), pad);
// |       case 'Hour12': return writeTwo(t.hour() % 12 === 0 ? 12 : t.hour() % 12, pad);
// |       case 'Minute': return writeTwo(t.minute(), pad);
// |       case 'Second': return writeTwo(t.second() + Math.floor(t.nanosecond() / 1e9), pad);
// |       case 'Nanosecond': return writeN(9, t.nanosecond() % 1e9, pad, false);
// |       default:
// |     }
// |   }
// |   if (numeric === 'Timestamp' && d !== null && t !== null) {
// |     const days = d.numDaysFromCe() - 719_163;
// |     const ts = days * 86_400 + t.numSecondsFromMidnight() - (off === null ? 0 : off.fix);
// |     return writeN(9, ts, pad, false);
// |   }
// |   return fail();
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Formats one fixed item.
// |  * @param {string} name
// |  * @param {NaiveDate | null} d
// |  * @param {NaiveTime | null} t
// |  * @param {OffsetInfo | null} off
// |  */
// | function formatFixed(name, d, t, off) {
// |   if (d !== null) {
// |     switch (name) {
// |       case 'ShortMonthName': return SHORT_MONTHS[d.month0()];
// |       case 'LongMonthName': return LONG_MONTHS[d.month0()];
// |       case 'ShortWeekdayName': return SHORT_WEEKDAYS[(d.numDaysFromMonday() + 1) % 7];
// |       case 'LongWeekdayName': return LONG_WEEKDAYS[(d.numDaysFromMonday() + 1) % 7];
// |       default:
// |     }
// |   }
// |   if (t !== null) {
// |     const nano = t.nanosecond();
// |     switch (name) {
// |       case 'LowerAmPm': return t.hour() >= 12 ? 'pm' : 'am';
// |       case 'UpperAmPm': return t.hour() >= 12 ? 'PM' : 'AM';
// |       case 'Nanosecond': return autoSiFraction(nano % 1e9);
// |       case 'Nanosecond3': return `.${String(Math.floor(nano / 1e6) % 1000).padStart(3, '0')}`;
// |       case 'Nanosecond6': return `.${String(Math.floor(nano / 1e3) % 1e6).padStart(6, '0')}`;
// |       case 'Nanosecond9': return `.${String(nano % 1e9).padStart(9, '0')}`;
// |       case 'Nanosecond3NoDot': return String(Math.floor(nano / 1e6) % 1000).padStart(3, '0');
// |       case 'Nanosecond6NoDot': return String(Math.floor(nano / 1e3) % 1e6).padStart(6, '0');
// |       case 'Nanosecond9NoDot': return String(nano % 1e9).padStart(9, '0');
// |       default:
// |     }
// |   }
// |   if (off !== null) {
// |     switch (name) {
// |       case 'TimezoneName': return off.name;
// |       case 'TimezoneOffset': return formatOffset(off.fix, 'Minutes', false, false);
// |       case 'TimezoneOffsetZ': return formatOffset(off.fix, 'Minutes', false, true);
// |       case 'TimezoneOffsetColon': return formatOffset(off.fix, 'Minutes', true, false);
// |       case 'TimezoneOffsetColonZ': return formatOffset(off.fix, 'Minutes', true, true);
// |       case 'TimezoneOffsetDoubleColon': return formatOffset(off.fix, 'Seconds', true, false);
// |       case 'TimezoneOffsetTripleColon': return formatOffset(off.fix, 'Hours', false, false);
// |       case 'RFC3339':
// |         if (d !== null && t !== null) {
// |           return writeRfc3339(d.andTime(t), off.fix, 'AutoSi', false) ?? fail();
// |         }
// |         break;
// |       case 'RFC2822':
// |         return panic('chrono port: formatting RFC 2822 is not supported');
// |       default:
// |     }
// |   }
// |   return fail();
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `DelayedFormat::write_to` into a string, or `null` on `fmt::Error`.
// |  * @param {NaiveDate | null} date
// |  * @param {NaiveTime | null} time
// |  * @param {OffsetInfo | null} off
// |  * @param {string} fmt
// |  * @returns {string | null}
// |  */
// | export function tryFormat(date, time, off, fmt) {
// |   let out = '';
// |   try {
// |     for (const item of new StrftimeItems(fmt)) {
// |       switch (item.tag) {
// |         case 'Literal':
// |         case 'Space':
// |           out += item.value;
// |           break;
// |         case 'Numeric':
// |           out += formatNumeric(item.numeric, item.pad, date, time, off);
// |           break;
// |         case 'Fixed':
// |           out += formatFixed(item.fixed, date, time, off);
// |           break;
// |         default:
// |           fail();
// |       }
// |     }
// |   } catch (e) {
// |     if (e instanceof FmtError) {
// |       return null;
// |     }
// |     throw e;
// |   }
// |   return out;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `format(fmt).to_string()`: panics like `ToString` when the format fails.
// |  * @param {NaiveDate | null} date
// |  * @param {NaiveTime | null} time
// |  * @param {OffsetInfo | null} off
// |  * @param {string} fmt
// |  */
// | export function formatToString(date, time, off, fmt) {
// |   return tryFormat(date, time, off, fmt)
// |     ?? panic('a Display implementation returned an error unexpectedly: Error');
// | }
