/**
 * Grammar for parsing date and time expressions.
 */

import { DateTime, Value } from '../types/index.js';
import { ok } from '../rust/core.js';
import {
  charIndices, endsWith, eqIgnoreAsciiCase, isAsciiDigit, sliceBytes, split, splitOnce,
  startsWith, trim, utf8Len,
} from '../rust/str.js';

/**
 * Grammar for parsing datetime expressions.
 */
export class DateTimeGrammar {
  /**
   * Creates a new datetime grammar.
   * @returns {DateTimeGrammar}
   */
  static new() {
    return new DateTimeGrammar();
  }

  /** @returns {DateTimeGrammar} */
  static default() {
    return new DateTimeGrammar();
  }

  /**
   * Attempts to parse a datetime from a string. Throws `Err(CalculatorError)`.
   * @param {string} input
   * @returns {DateTime}
   */
  parse(input) {
    return DateTime.parse(input);
  }

  /**
   * Checks if the input looks like a datetime.
   * @param {string} input
   * @returns {boolean}
   */
  static looksLikeDatetime(input) {
    input = trim(input).toLowerCase();

    // Check for "now" keyword
    if (input === 'now' || input.startsWith('now ') || input.endsWith(' now')) {
      return true;
    }

    // Check for "time" keyword combined with timezone
    if (input === 'time' || input.startsWith('time ') || input.endsWith(' time')) {
      return true;
    }

    // Check for "current time" phrase
    if (input.includes('current')) {
      return true;
    }

    // Check for month names (all supported UI languages)
    const monthNames = [
      // English abbreviated
      'jan',
      'feb',
      'mar',
      'apr',
      'may',
      'jun',
      'jul',
      'aug',
      'sep',
      'oct',
      'nov',
      'dec',
      // English full
      'january',
      'february',
      'march',
      'april',
      'june',
      'july',
      'august',
      'september',
      'october',
      'november',
      'december',
      // Russian (ru) nominative forms
      'январь',
      'февраль',
      'март',
      'апрель',
      'май',
      'июнь',
      'июль',
      'август',
      'сентябрь',
      'октябрь',
      'ноябрь',
      'декабрь',
      // Russian (ru) genitive forms (most common in dates like '17 февраля 2027')
      'января',
      'февраля',
      'марта',
      'апреля',
      'мая',
      'июня',
      'июля',
      'августа',
      'сентября',
      'октября',
      'ноября',
      'декабря',
      // Russian (ru) abbreviated forms
      'янв',
      'фев',
      'мар',
      'апр',
      'июн',
      'июл',
      'авг',
      'сент',
      'сен',
      'окт',
      'ноя',
      'дек',
      // German (de)
      'januar',
      'februar',
      'märz',
      'juni',
      'juli',
      'oktober',
      'dezember',
      // French (fr)
      'janvier',
      'février',
      'fevrier',
      'mars',
      'avril',
      'juin',
      'juillet',
      'août',
      'aout',
      'septembre',
      'octobre',
      'novembre',
      'décembre',
      'decembre',
      // Chinese Simplified (zh) — numeric month names like 一月, 十二月
      '一月',
      '二月',
      '三月',
      '四月',
      '五月',
      '六月',
      '七月',
      '八月',
      '九月',
      '十月',
      '十一月',
      '十二月',
      // Hindi (hi)
      'जनवरी',
      'फरवरी',
      'मार्च',
      'अप्रैल',
      'मई',
      'जून',
      'जुलाई',
      'अगस्त',
      'सितंबर',
      'सितम्बर',
      'अक्टूबर',
      'नवंबर',
      'नवम्बर',
      'दिसंबर',
      'दिसम्बर',
      // Arabic (ar)
      'يناير',
      'فبراير',
      'مارس',
      'أبريل',
      'ابريل',
      'مايو',
      'يونيو',
      'يوليو',
      'أغسطس',
      'اغسطس',
      'سبتمبر',
      'أكتوبر',
      'اكتوبر',
      'نوفمبر',
      'ديسمبر',
            ];

    for (const month of monthNames) {
      if (input.includes(month)) {
        return true;
      }
    }

    // Check for day names (indicates a date expression)
    const dayNames = [
      'monday',
      'tuesday',
      'wednesday',
      'thursday',
      'friday',
      'saturday',
      'sunday',
    ];
    for (const day of dayNames) {
      if (input.includes(day)) {
        return true;
      }
    }

    // Check for time patterns (am/pm)
    if (input.includes('am') || input.includes('pm')) {
      return true;
    }

    // Check for timezone indicators
    if (input.includes('utc') || input.includes('gmt') || input.includes('gtm')) {
      return true;
    }

    // Check for common timezone abbreviations
    const tzAbbrevs = [
      // US & Canada
      'est', 'edt', 'cst', 'cdt', 'mst', 'mdt', 'pst', 'pdt', 'akst', 'akdt', 'hst', 'ast',
      'adt', 'nst', 'ndt', // European
      'cet', 'cest', 'eet', 'eest', 'wet', 'west', 'bst', 'msk', 'msd', // Asian
      'ist', 'jst', 'kst', 'sgt', 'hkt', 'pht', 'ict', 'myt', 'twt', 'wib', 'wita', 'wit',
      'pkt', 'bdt', // Russian regional
      'samt', 'yekt', 'omst', 'krat', 'irkt', 'yakt', 'vlat', 'magt', 'pett',
      // Middle East
      'gst', 'trt', 'irst', 'irdt', 'aft', // Australian & Pacific
      'aest', 'aedt', 'acst', 'acdt', 'awst', 'nzst', 'nzdt', // South American
      'art', 'brt', 'brst', 'clt', 'clst', // African
      'wat', 'cat', 'eat', 'sast',
            ];
    for (const tz of tzAbbrevs) {
      // Match as whole word (not substring)
      if (
        input === tz
        || input.startsWith(`${tz} `)
        || input.endsWith(` ${tz}`)
        || input.includes(` ${tz} `)
      ) {
        return true;
      }
    }

    // Check for ISO date pattern (YYYY-MM-DD)
    if (utf8Len(input) >= 10) {
      const chars = [...input];
      if (
        chars.length >= 10
        && chars[4] === '-'
        && chars[7] === '-'
        && chars.slice(0, 4).every((c) => isAsciiDigit(c))
      ) {
        return true;
      }
    }

    // Check for time pattern (HH:MM)
    if (input.includes(':')) {
      const parts = split(input, ':');
      if (parts.length >= 2) {
        const last = [...parts[0]].pop();
        if (last !== undefined && isAsciiDigit(last)) {
          return true;
        }
      }
    }

    return false;
  }

  /**
   * Attempts to extract a datetime from a longer expression.
   * Returns the datetime and the remaining string.
   * @param {string} input
   * @returns {[DateTime, string] | null}
   */
  tryExtractDatetime(input) {
    input = trim(input);

    // Try progressively longer substrings from the start
    // This handles cases like "Jan 27, 8:59am UTC" followed by more text

    // First, try the whole string
    const dt = ok(() => this.parse(input));
    if (dt !== null) {
      return [dt, ''];
    }

    // Try to find natural break points
    // Look for patterns that indicate end of datetime

    return null;
  }

  /**
   * Localizes a parsed datetime to the user's local timezone, when one is known.
   *
   * A bare `now` becomes the current local time; bare (timezone-less) times are
   * re-anchored to the local offset. Inputs with an explicit timezone, and the
   * explicit `now utc` / `now <tz>` forms, are returned unchanged.
   * @param {DateTime} dt
   * @param {string} raw
   * @param {number | null} offset
   * @returns {DateTime}
   */
  static localize(dt, raw, offset) {
    if (offset === null) {
      return dt;
    }
    if (eqIgnoreAsciiCase(trim(raw), 'now')) {
      return DateTime.nowLocal(offset);
    }
    return dt.reinterpretNaiveAsLocal(offset);
  }

  /**
   * Tries to parse a datetime subtraction expression like "(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)".
   *
   * `localOffset` is the user's timezone offset in seconds east of UTC, when
   * known; it makes bare `now` and timezone-less times resolve to local time.
   * @param {string} input
   * @param {number | null} localOffset
   * @returns {[Value, string[], string] | null}
   */
  tryParseDatetimeSubtraction(input, localOffset) {
    // Look for pattern: (datetime) - (datetime)
    input = trim(input);

    if (!input.includes('-')) {
      return null;
    }

    const halves = splitOnce(input, ' - ');
    if (halves !== null) {
      const [left, right] = halves;
      const dt1 = ok(() => this.parse(trim(left)));
      const dt2 = ok(() => this.parse(trim(right)));
      if (dt1 !== null && dt2 !== null) {
        const local1 = DateTimeGrammar.localize(dt1, trim(left), localOffset);
        const local2 = DateTimeGrammar.localize(dt2, trim(right), localOffset);
        return DateTimeGrammar.datetimeDifferenceResult(local1, local2, trim(left), trim(right));
      }
    }

    // Check if it starts with '(' for the explicit "(datetime) - (datetime)" form.
    if (!startsWith(input, '(')) {
      return null;
    }

    // Try to find the matching closing paren for the first datetime
    let parenDepth = 0;
    /** @type {number | null} */
    let firstEnd = null;

    for (const [i, ch] of charIndices(input)) {
      if (ch === '(') {
        parenDepth += 1;
      } else if (ch === ')') {
        parenDepth -= 1;
        if (parenDepth === 0) {
          firstEnd = i;
          break;
        }
      }
    }

    if (firstEnd === null) {
      return null;
    }

    // Extract first datetime (without parens)
    const firstDtStr = sliceBytes(input, 1, firstEnd);

    // Find the minus sign
    const rest = trim(sliceBytes(input, firstEnd + 1));
    if (!startsWith(rest, '-')) {
      return null;
    }

    const secondPart = trim(sliceBytes(rest, 1));
    if (!startsWith(secondPart, '(') || !endsWith(secondPart, ')')) {
      return null;
    }

    // Extract second datetime (without parens)
    const secondDtStr = sliceBytes(secondPart, 1, utf8Len(secondPart) - 1);

    // Try to parse both as datetimes
    const dt1 = ok(() => this.parse(firstDtStr));
    if (dt1 === null) {
      return null;
    }

    const dt2 = ok(() => this.parse(secondDtStr));
    if (dt2 === null) {
      return null;
    }

    const local1 = DateTimeGrammar.localize(dt1, trim(firstDtStr), localOffset);
    const local2 = DateTimeGrammar.localize(dt2, trim(secondDtStr), localOffset);

    return DateTimeGrammar.datetimeDifferenceResult(local1, local2, trim(firstDtStr), trim(secondDtStr));
  }

  /**
   * @param {DateTime} dt1
   * @param {DateTime} dt2
   * @param {string} firstDtStr
   * @param {string} secondDtStr
   * @returns {[Value, string[], string]}
   */
  static datetimeDifferenceResult(dt1, dt2, firstDtStr, secondDtStr) {
    // Calculate the signed difference (dt1 - dt2). Using signed seconds keeps
    // the result correct when dt1 is earlier than dt2 (a negative duration),
    // instead of collapsing to zero.
    const seconds = dt1.signedSubtractSeconds(dt2);

    const value = Value.duration(seconds);

    const steps = [
      `Parse first datetime: ${dt1}`,
      `Parse second datetime: ${dt2}`,
      `Calculate difference: ${dt1} - ${dt2}`,
      `Result: ${value.toDisplayString()}`,
    ];

    // Issue #30 fix: Use exactly 2 outer parentheses for Links notation
    // Format: ((datetime1) - (datetime2))
    const lino = `((${firstDtStr}) - (${secondDtStr}))`;

    return [value, steps, lino];
  }
}
