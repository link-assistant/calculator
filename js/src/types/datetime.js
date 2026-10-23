/**
 * `DateTime` type for date and time calculations (port of
 * `src/types/datetime.rs`).
 *
 * The private fields `offset_seconds`, `has_time` and `has_date` share their
 * names with the accessor methods `offset_seconds()`, `has_time()` and
 * `has_date()`; in JavaScript a field would hide the method, so these fields
 * are stored as `offsetSecondsValue`, `hasTimeValue` and `hasDateValue` (the
 * convention of `rust/chrono/`). `toJSON()` keeps the serde names.
 */

import {
  DateTime as ChronoDateTime, Duration, FixedOffset, Months, NaiveDate, NaiveDateTime, NaiveTime,
  StdDuration, Utc,
} from '../rust/chrono.js';
import { now as clockNow } from '../clock.js';
import { err, expect, ok } from '../rust/core.js';
import { splitWhitespace, stripPrefix, stripSuffix, trim, trimEndMatches } from '../rust/str.js';
import { CalculatorError } from '../error.js';
import {
  extractTimezone, normalizeMonthName, parse12hTime, parsePartialDate,
  parseTzAbbreviation, preprocessNaturalDate, translateMonthNames,
} from './datetime_parse.js';

/**
 * The struct literal of {@link DateTime}.
 * @typedef {object} DateTimeFields
 * @property {ChronoDateTime} inner
 * @property {number | null} offsetSeconds
 * @property {boolean} hasTime
 * @property {boolean} hasDate
 * @property {string | null} label
 * @property {string | null} tzAbbrev
 */

/** A `DateTime` value that can represent dates, times, or both. */
export class DateTime {
  /** @param {DateTimeFields} fields */
  constructor({ inner, offsetSeconds, hasTime, hasDate, label, tzAbbrev }) {
    /** @type {ChronoDateTime} The underlying datetime (always stored in UTC internally). */
    this.inner = inner;
    /** @type {number | null} The original timezone offset in seconds for display purposes. */
    this.offsetSecondsValue = offsetSeconds;
    /** @type {boolean} Whether this includes a time component. */
    this.hasTimeValue = hasTime;
    /** @type {boolean} Whether this includes a date component. */
    this.hasDateValue = hasDate;
    /** @type {string | null} Optional label for named time expressions (e.g., "current UTC time"). When set, the display format includes the label and timezone info. */
    this.label = label;
    /** @type {string | null} Optional timezone abbreviation for display (e.g., "UTC", "EST", "GMT"). Used to show the named timezone in the value portion of the display. */
    this.tzAbbrev = tzAbbrev;
  }

  /**
   * `PartialEq`: compares the instants.
   * @param {DateTime} other
   */
  equals(other) {
    return this.inner.equals(other.inner);
  }

  /**
   * `Ord`: compares the instants.
   * @param {DateTime} other
   * @returns {number}
   */
  cmp(other) {
    return this.inner.cmp(other.inner);
  }

  /**
   * `PartialOrd`.
   * @param {DateTime} other
   * @returns {number}
   */
  partialCmp(other) {
    return this.cmp(other);
  }

  /** Creates a `DateTime` representing the current UTC time. */
  static now() {
    return new DateTime({
      inner: clockNow(),
      offsetSeconds: 0,
      hasTime: true,
      hasDate: true,
      label: null,
      tzAbbrev: null,
    });
  }

  /**
   * Creates a `DateTime` representing the current time with a descriptive label.
   * The label is shown in the display format, e.g., `('current UTC time': 2026-03-02 20:40:13 UTC (+00:00))`.
   * @param {string} label
   * @param {number | null} offsetSeconds
   * @param {string | null} tzAbbrev
   */
  static nowWithLabel(label, offsetSeconds, tzAbbrev) {
    return new DateTime({
      inner: clockNow(),
      offsetSeconds,
      hasTime: true,
      hasDate: true,
      label,
      tzAbbrev,
    });
  }

  /**
   * Creates a `DateTime` for the current instant, displayed in the user's
   * local timezone given by `offset_seconds` (seconds east of UTC).
   *
   * The internal instant is still the true current UTC time; only the display
   * offset and label differ from [`Self::now`]. Used when the calculator is
   * told the user's local timezone so that `now` reflects their wall clock
   * instead of UTC.
   * @param {number} offsetSeconds
   */
  static nowLocal(offsetSeconds) {
    return new DateTime({
      inner: clockNow(),
      offsetSeconds,
      hasTime: true,
      hasDate: true,
      label: 'current local time',
      tzAbbrev: null,
    });
  }

  /**
   * Creates a date-only value for today's calendar date in the timezone
   * represented by `offset_seconds` (seconds east of UTC).
   * @param {number} offsetSeconds
   */
  static today(offsetSeconds) {
    const localNow = clockNow().add(Duration.seconds(offsetSeconds));
    return DateTime.fromDate(localNow.dateNaive());
  }

  /**
   * Re-anchors a timezone-less ("naive") time or datetime to a local timezone.
   *
   * Bare times like `12:30` are parsed with their wall-clock reading stored as
   * if it were UTC. When the user's local timezone is known, the same
   * wall-clock reading should instead be interpreted as local time. This shifts
   * the internal UTC instant back by `offset_seconds` so the displayed wall
   * clock is preserved while the underlying instant becomes correct for the
   * local timezone.
   *
   * Values that already carry an explicit timezone (e.g. `12:30 UTC`) or that
   * have no time component (plain dates) are returned unchanged.
   * @param {number} offsetSeconds
   * @returns {DateTime}
   */
  reinterpretNaiveAsLocal(offsetSeconds) {
    if (this.offsetSecondsValue !== null || !this.hasTimeValue) {
      return this.clone();
    }
    const adjusted = this.inner.sub(Duration.seconds(offsetSeconds));
    return new DateTime({
      inner: adjusted,
      offsetSeconds,
      hasTime: this.hasTimeValue,
      hasDate: this.hasDateValue,
      label: this.label,
      tzAbbrev: null,
    });
  }

  /**
   * `#[derive(Clone)]`.
   * @returns {DateTime}
   */
  clone() {
    return new DateTime({
      inner: this.inner,
      offsetSeconds: this.offsetSecondsValue,
      hasTime: this.hasTimeValue,
      hasDate: this.hasDateValue,
      label: this.label,
      tzAbbrev: this.tzAbbrev,
    });
  }

  /** Returns whether this datetime has a label (i.e., represents a named live time expression). */
  isLiveTime() {
    return this.label !== null;
  }

  /**
   * Creates a new `DateTime` from a chrono `DateTime`.
   * @param {ChronoDateTime} dt
   * @param {boolean} hasDate
   * @param {boolean} hasTime
   */
  static fromUtc(dt, hasDate, hasTime) {
    return new DateTime({
      inner: dt,
      offsetSeconds: null,
      hasTime,
      hasDate,
      label: null,
      tzAbbrev: null,
    });
  }

  /**
   * Creates a new DateTime from a date.
   * @param {NaiveDate} date
   */
  static fromDate(date) {
    const dt = expect(date.andHmsOpt(0, 0, 0), 'valid time').andUtc();
    return new DateTime({
      inner: dt,
      offsetSeconds: null,
      hasTime: false,
      hasDate: true,
      label: null,
      tzAbbrev: null,
    });
  }

  /**
   * Creates a date-only value for January 1 of a calendar year.
   * @param {number} year
   * @returns {DateTime | null}
   */
  static fromYear(year) {
    const date = NaiveDate.fromYmdOpt(year, 1, 1);
    return date === null ? null : DateTime.fromDate(date);
  }

  /**
   * Creates a new DateTime from a time (today's date is used).
   * @param {NaiveTime} time
   */
  static fromTime(time) {
    const today = clockNow().dateNaive();
    const dt = today.andTime(time).andUtc();
    return new DateTime({
      inner: dt,
      offsetSeconds: null,
      hasTime: true,
      hasDate: false,
      label: null,
      tzAbbrev: null,
    });
  }

  /**
   * Gets the fixed offset from the stored seconds.
   * @returns {FixedOffset | null}
   */
  getOffset() {
    return this.offsetSecondsValue === null ? null : FixedOffset.eastOpt(this.offsetSecondsValue);
  }

  /**
   * Sets the offset from a FixedOffset.
   * @param {FixedOffset | null} offset
   */
  setOffset(offset) {
    this.offsetSecondsValue = offset === null ? null : offset.localMinusUtc();
  }

  /**
   * Returns a new `DateTime` displaying the same instant in a different timezone.
   *
   * The internal UTC time does not change — only the display offset and abbreviation.
   * @param {FixedOffset} offset
   * @param {string} tzAbbrev
   */
  withTimezoneOffset(offset, tzAbbrev) {
    return new DateTime({
      inner: this.inner,
      offsetSeconds: offset.localMinusUtc(),
      hasTime: this.hasTimeValue,
      hasDate: this.hasDateValue,
      label: null,
      tzAbbrev: tzAbbrev.toUpperCase(),
    });
  }

  /**
   * Parses a datetime from a string, trying multiple formats; throws
   * `Err(CalculatorError)`.
   * @param {string} input
   * @returns {DateTime}
   */
  static parse(input) {
    input = trim(input);

    // Check for "now" keyword variants
    let dt = DateTime.tryParseNow(input);
    if (dt !== null) {
      return dt;
    }

    // Check for "UTC time" / "time UTC" / "current time" variants
    dt = DateTime.tryParseCurrentTimePhrase(input);
    if (dt !== null) {
      return dt;
    }

    // Pre-process: translate non-English month names to English (all supported UI languages)
    const translated = translateMonthNames(input);
    input = translated !== input ? translated : input;

    // Pre-process: strip day names and ordinal suffixes
    const cleaned = preprocessNaturalDate(input);
    const inputToParse = cleaned !== input ? cleaned : input;

    /** @param {NaiveDate | null} date */
    const fromDate = (date) => (date === null ? null : DateTime.fromDate(date));
    dt = (
      // Try various date formats
      DateTime.tryParseDateFormats(inputToParse)
      ?? fromDate(parsePartialDate(inputToParse))
      // Try various time formats
      ?? DateTime.tryParseTimeFormats(inputToParse)
      // Try datetime formats
      ?? DateTime.tryParseDatetimeFormats(inputToParse)
    );
    if (dt !== null) {
      return dt;
    }

    // If preprocessing changed the input, also try with original
    if (cleaned !== input) {
      dt = DateTime.tryParseDateFormats(input)
        ?? fromDate(parsePartialDate(input))
        ?? DateTime.tryParseTimeFormats(input)
        ?? DateTime.tryParseDatetimeFormats(input);
      if (dt !== null) {
        return dt;
      }
    }

    return err(CalculatorError.InvalidDateTime(`Could not parse '${input}' as a date or time`));
  }

  /**
   * Checks if input represents "now" (current time).
   * @param {string} input
   * @returns {DateTime | null}
   */
  static tryParseNow(input) {
    const lower = input.toLowerCase();
    const trimmed = trim(lower);

    // Exact "now" or "now" with timezone
    switch (trimmed) {
      case 'now':
        return DateTime.nowWithLabel('current UTC time', 0, 'UTC');
      case 'now utc':
      case 'utc now':
      case 'now gmt':
      case 'gmt now':
        return DateTime.nowWithLabel('current UTC time', 0, 'UTC');
      default:
    }

    // "now <timezone>" pattern, then "<timezone> now" pattern
    for (let rest of [stripPrefix(trimmed, 'now '), stripSuffix(trimmed, ' now')]) {
      if (rest !== null) {
        rest = trim(rest);
        const offset = parseTzAbbreviation(rest);
        if (offset !== null) {
          const tzUpper = rest.toUpperCase();
          const label = `current ${tzUpper} time`;
          const offsetSecs = offset.localMinusUtc();
          return DateTime.nowWithLabel(label, offsetSecs, tzUpper);
        }
      }
    }

    return null;
  }

  /**
   * Formats the timezone offset for display, e.g., `UTC (+00:00)` or `EST (-05:00)`.
   * When `tz_name` is provided, it is prepended to the numeric offset.
   * @param {number} offsetSeconds
   * @param {string | null} tzName
   * @returns {string}
   */
  static formatTzForDisplay(offsetSeconds, tzName) {
    const sign = offsetSeconds >= 0 ? '+' : '-';
    const absSecs = Math.abs(offsetSeconds);
    const hours = Math.trunc(absSecs / 3600);
    const minutes = Math.trunc((absSecs % 3600) / 60);
    const offsetStr = `(${sign}${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')})`;
    if (tzName !== null) {
      return `${tzName} ${offsetStr}`;
    }
    return offsetStr;
  }

  /**
   * Checks if input represents a "current time" phrase.
   * @param {string} input
   * @returns {DateTime | null}
   */
  static tryParseCurrentTimePhrase(input) {
    const lower = input.toLowerCase();
    const trimmed = trim(lower);

    // Phrases that mean "current UTC time" — map phrase to (label, offset_seconds, tz_abbrev)
    /** @type {[string, string, number | null, string][]} */
    const currentTimePhrases = [
      ['utc time', 'current UTC time', 0, 'UTC'],
      ['time utc', 'current UTC time', 0, 'UTC'],
      ['current time', 'current UTC time', 0, 'UTC'],
      ['current utc time', 'current UTC time', 0, 'UTC'],
      ['gmt time', 'current GMT time', 0, 'GMT'],
      ['time gmt', 'current GMT time', 0, 'GMT'],
    ];

    for (const [phrase, label, offsetSecs, tzAbbrev] of currentTimePhrases) {
      if (trimmed === phrase) {
        return DateTime.nowWithLabel(label, offsetSecs, tzAbbrev);
      }
    }

    // "current <TZ> time" or "<TZ> time" patterns
    for (const prefix of ['current ', '']) {
      const rest = stripPrefix(trimmed, prefix);
      if (rest !== null) {
        for (let tzPart of [stripSuffix(rest, ' time'), stripPrefix(rest, 'time ')]) {
          if (tzPart !== null) {
            tzPart = trim(tzPart);
            const offset = parseTzAbbreviation(tzPart);
            if (offset !== null) {
              const tzUpper = tzPart.toUpperCase();
              const label = `current ${tzUpper} time`;
              const offsetSecs = offset.localMinusUtc();
              return DateTime.nowWithLabel(label, offsetSecs, tzUpper);
            }
          }
        }
      }
    }

    return null;
  }

  /**
   * @param {string} input
   * @returns {DateTime | null}
   */
  static tryParseDateFormats(input) {
    /** `if let Ok(date) = NaiveDate::parse_from_str(s, fmt) { return Some(Self::from_date(date)); }` */
    const tryDate = (/** @type {string} */ s, /** @type {string} */ fmt) => {
      const date = ok(() => NaiveDate.parseFromStr(s, fmt));
      return date === null ? null : DateTime.fromDate(date);
    };

    // Month name formats below parse `normalized`.
    const normalized = normalizeMonthName(input);
    return (
      // ISO format: 2026-01-22
      tryDate(input, '%Y-%m-%d')
      // US format: 01/22/2026 or 1/22/2026
      ?? tryDate(input, '%m/%d/%Y')
      // European format: 22/01/2026
      ?? tryDate(input, '%d/%m/%Y')
      // Dot-separated European/German format: 15.10.2025 (DD.MM.YYYY)
      // Dots are the conventional date separator in German, Russian and many
      // other locales, so day-first is tried before month-first.
      ?? tryDate(input, '%d.%m.%Y')
      // Dot-separated US-style fallback: 10.15.2025 (MM.DD.YYYY)
      ?? tryDate(input, '%m.%d.%Y')
      // Dot-separated ISO-like format: 2025.10.15 (YYYY.MM.DD)
      ?? tryDate(input, '%Y.%m.%d')
      // Dash-separated, non-ISO orderings: 15-10-2025 (DD-MM-YYYY) and the
      // US-style 10-15-2025 (MM-DD-YYYY). ISO `%Y-%m-%d` is tried first above,
      // so these only match when the year is in the trailing position. Day-first
      // is preferred, mirroring the dot-separated convention.
      ?? tryDate(input, '%d-%m-%Y')
      ?? tryDate(input, '%m-%d-%Y')
      // Month name formats: Jan 22, 2026 or January 22, 2026
      ?? tryDate(normalized, '%b %d, %Y')
      ?? tryDate(normalized, '%B %d, %Y')
      // 22 Jan 2026 format (without comma)
      ?? tryDate(normalized, '%d %b %Y')
      ?? tryDate(normalized, '%d %B %Y')
      // Jan 22 2026 format (without comma) — e.g., "Feb 17 2027"
      ?? tryDate(normalized, '%b %d %Y')
      ?? tryDate(normalized, '%B %d %Y')
      // None
      ?? null
    );
  }


  /**
   * @param {string} input
   * @returns {DateTime | null}
   */
  static tryParseTimeFormats(input) {
    input = trim(input);

    // Parse time with optional timezone
    const [timePart, tzOffset, tzAbbrev] = extractTimezone(input);

    // 12-hour format: 8:59am, 12:51pm, 8:59 am, 8:59AM
    let time = parse12hTime(timePart);
    if (time !== null) {
      const dt = DateTime.fromTime(time);
      if (tzOffset !== null) {
        dt.setOffset(tzOffset);
        dt.tzAbbrev = tzAbbrev;
        // Adjust internal time to UTC
        const local = dt.inner.naiveUtc();
        const adjusted = tzOffset.fromLocalDatetime(local).single();
        if (adjusted !== null) {
          dt.inner = adjusted.withTimezone(Utc);
        }
      }
      return dt;
    }

    // 24-hour format: 14:30, 14:30:00
    for (const fmt of ['%H:%M', '%H:%M:%S']) {
      time = ok(() => NaiveTime.parseFromStr(timePart, fmt));
      if (time !== null) {
        const dt = DateTime.fromTime(time);
        dt.setOffset(tzOffset);
        dt.tzAbbrev = tzAbbrev;
        if (tzOffset !== null) {
          const local = dt.inner.naiveUtc();
          const adj = tzOffset.fromLocalDatetime(local).single();
          if (adj !== null) {
            dt.inner = adj.withTimezone(Utc);
          }
        }
        return dt;
      }
    }

    return null;
  }

  /**
   * @param {string} input
   * @returns {DateTime | null}
   */
  static tryParseDatetimeFormats(input) {
    // Try comma separation: "Jan 27, 8:59am UTC"
    const commaIndex = input.indexOf(',');
    if (commaIndex !== -1) {
      const datePart = trim(input.slice(0, commaIndex));
      const timePart = trim(input.slice(commaIndex + 1));

      // Try to parse partial date (no year - assume current year)
      const date = parsePartialDate(datePart);
      if (date !== null) {
        const timeDt = DateTime.tryParseTimeFormats(timePart);
        if (timeDt !== null) {
          return DateTime.combineDateAndTime(date, timeDt);
        }
      }
    }

    // Try "time <date>" pattern: "11:59pm EST January 26"
    let dt = DateTime.tryParseTimeThenDate(input);
    if (dt !== null) {
      return dt;
    }

    // Try "<date> <time>" pattern without a comma: "08 Aug 2026 22:35"
    dt = DateTime.tryParseDateThenTime(input);
    if (dt !== null) {
      return dt;
    }

    // ISO 8601 format
    const rfc = ok(() => ChronoDateTime.parseFromRfc3339(input));
    if (rfc !== null) {
      return new DateTime({
        inner: rfc.withTimezone(Utc),
        offsetSeconds: rfc.timezone().localMinusUtc(),
        hasTime: true,
        hasDate: true,
        label: null,
        tzAbbrev: null,
      });
    }

    // Common datetime formats
    const naive = ok(() => NaiveDateTime.parseFromStr(input, '%Y-%m-%d %H:%M:%S'));
    if (naive !== null) {
      return new DateTime({
        inner: naive.andUtc(),
        offsetSeconds: null,
        hasTime: true,
        hasDate: true,
        label: null,
        tzAbbrev: null,
      });
    }

    return null;
  }

  /**
   * Try to parse "date time" patterns like "08 Aug 2026 22:35" or
   * "2026-08-08 22:35 UTC", where the date comes first and no comma
   * separates the two parts.
   * @param {string} input
   * @returns {DateTime | null}
   */
  static tryParseDateThenTime(input) {
    input = trim(input);

    const words = splitWhitespace(input);
    if (words.length < 2) {
      return null;
    }

    // Try progressively shorter date prefixes so the longest date wins
    // (e.g. "08 Aug 2026" before "08 Aug").
    for (let splitAt = words.length - 1; splitAt >= 1; splitAt -= 1) {
      const dateCandidate = words.slice(0, splitAt).join(' ');
      const timeCandidate = words.slice(splitAt).join(' ');

      const timeDt = DateTime.tryParseTimeFormats(timeCandidate);
      if (timeDt === null) {
        continue;
      }

      const date = DateTime.tryParseDateFormats(dateCandidate)?.inner.dateNaive()
        ?? parsePartialDate(trimEndMatches(dateCandidate, ','));
      if (date === null) {
        continue;
      }
      return DateTime.combineDateAndTime(date, timeDt);
    }

    return null;
  }

  /**
   * Try to parse "time date" patterns like "11:59pm EST January 26" or
   * "00:35 16 September 2026 IST".
   * @param {string} input
   * @returns {DateTime | null}
   */
  static tryParseTimeThenDate(input) {
    input = trim(input);

    const words = splitWhitespace(input);
    if (words.length < 2) {
      return null;
    }

    // A timezone may follow either the time or the entire expression. If it
    // is trailing, move it beside the time while testing split points.
    const last = words[words.length - 1];
    const hasTrailingTimezone = parseTzAbbreviation(last) !== null || (() => {
      const [remaining, offset] = extractTimezone(last);
      return remaining === '' && offset !== null;
    })();
    const dateEnd = words.length - (hasTrailingTimezone ? 1 : 0);
    if (dateEnd < 2) {
      return null;
    }

    for (let splitAt = 1; splitAt < dateEnd; splitAt += 1) {
      let timeCandidate = words.slice(0, splitAt).join(' ');
      if (hasTrailingTimezone) {
        timeCandidate += ' ';
        timeCandidate += words[words.length - 1];
      }
      const dateCandidate = words.slice(splitAt, dateEnd).join(' ');

      const timeDt = DateTime.tryParseTimeFormats(timeCandidate);
      if (timeDt === null) {
        continue;
      }
      const date = DateTime.tryParseDateFormats(dateCandidate)?.inner.dateNaive()
        ?? parsePartialDate(trimEndMatches(dateCandidate, ','));
      if (date === null) {
        continue;
      }
      return DateTime.combineDateAndTime(date, timeDt);
    }

    return null;
  }

  /**
   * Combines a parsed date with a parsed time without applying its timezone
   * twice. `try_parse_time_formats` stores the instant in UTC, so recover the
   * source wall-clock time before attaching the requested calendar date.
   * @param {NaiveDate} date
   * @param {DateTime} timeDt
   * @returns {DateTime}
   */
  static combineDateAndTime(date, timeDt) {
    const offset = timeDt.offsetSecondsValue === null ? null : FixedOffset.eastOpt(timeDt.offsetSecondsValue);
    const wallTime = offset === null ? timeDt.inner.time() : timeDt.inner.withTimezone(offset).time();
    const naive = date.andTime(wallTime);
    const adjusted = offset === null ? null : offset.fromLocalDatetime(naive).single();
    const inner = adjusted === null ? naive.andUtc() : adjusted.withTimezone(Utc);

    return new DateTime({
      inner,
      offsetSeconds: timeDt.offsetSecondsValue,
      hasTime: true,
      hasDate: true,
      label: null,
      tzAbbrev: timeDt.tzAbbrev,
    });
  }

  /** Returns the underlying chrono DateTime. */
  asChrono() {
    return this.inner;
  }

  /**
   * Subtracts another DateTime, returning a Duration (absolute value).
   * @param {DateTime} other
   * @returns {StdDuration}
   */
  subtract(other) {
    const diff = this.inner.signedDurationSince(other.inner);
    return ok(() => diff.toStd()) ?? StdDuration.default();
  }

  /**
   * Subtracts another DateTime, returning signed seconds (positive if self > other).
   * @param {DateTime} other
   * @returns {bigint}
   */
  signedSubtractSeconds(other) {
    return this.inner.signedDurationSince(other.inner).numSeconds();
  }

  /** Returns the inner chrono DateTime (for comparisons, etc.). */
  innerUtc() {
    return this.inner;
  }

  /** Returns the UTC timestamp in milliseconds since the Unix epoch. */
  timestampMillis() {
    return this.inner.timestampMillis();
  }

  /** Returns whether this value has a date component. */
  hasDate() {
    return this.hasDateValue;
  }

  /** Returns whether this value has a time component. */
  hasTime() {
    return this.hasTimeValue;
  }

  /**
   * Returns the source timezone offset in seconds, if known.
   * @returns {number | null}
   */
  offsetSeconds() {
    return this.offsetSecondsValue;
  }

  /**
   * Returns the source timezone abbreviation, if available.
   * @returns {string | null}
   */
  timezoneAbbreviation() {
    return this.tzAbbrev;
  }

  /** Returns true when this value has enough timezone context to show conversions. */
  shouldShowTimezoneConversions() {
    return this.hasTimeValue && this.offsetSecondsValue !== null;
  }

  /**
   * Formats the same instant in UTC for conversion summaries.
   * @returns {string | null}
   */
  utcEquivalentDisplay() {
    if (!this.shouldShowTimezoneConversions()) {
      return null;
    }

    let formatted;
    if (this.hasDateValue && this.hasTimeValue) {
      formatted = this.inner.format('%Y-%m-%d %H:%M:%S UTC');
    } else if (this.hasDateValue) {
      formatted = this.inner.format('%Y-%m-%d');
    } else if (this.hasTimeValue) {
      formatted = this.inner.format('%H:%M:%S UTC');
    } else {
      formatted = this.inner.toRfc3339();
    }

    return formatted;
  }

  /**
   * Adds a duration to this DateTime.
   * @param {number | bigint} seconds an `i64`
   */
  addDuration(seconds) {
    const duration = Duration.seconds(seconds);
    return new DateTime({
      inner: this.inner.add(duration),
      offsetSeconds: this.offsetSecondsValue,
      hasTime: this.hasTimeValue,
      hasDate: this.hasDateValue,
      label: null,
      tzAbbrev: null,
    });
  }

  /**
   * Adds (or subtracts when negative) a number of calendar months to this DateTime.
   *
   * Unlike `add_duration`, this performs true calendar arithmetic: the day-of-month
   * is preserved whenever possible, and clamped to the last day of the target month
   * when the original day does not exist (e.g. 31 Jan + 1 month → 28/29 Feb).
   * @param {number} months an `i32`
   */
  addCalendarMonths(months) {
    const naive = this.inner.naiveUtc();
    let newNaive;
    if (months >= 0) {
      const m = Months.new(months);
      newNaive = naive.checkedAddMonths(m) ?? naive;
    } else {
      const m = Months.new(-months);
      newNaive = naive.checkedSubMonths(m) ?? naive;
    }
    const newInner = newNaive.andUtc();
    return new DateTime({
      inner: newInner,
      offsetSeconds: this.offsetSecondsValue,
      hasTime: this.hasTimeValue,
      hasDate: this.hasDateValue,
      label: null,
      tzAbbrev: null,
    });
  }

  /** Returns the year. */
  year() {
    return this.inner.year();
  }

  /**
   * Parses common timezone abbreviations to `FixedOffset`.
   *
   * Returns `None` if the abbreviation is not recognized.
   * @param {string} tz
   * @returns {FixedOffset | null}
   */
  static parseTzAbbreviation(tz) {
    return parseTzAbbreviation(tz);
  }

  /** `Display`. */
  toString() {
    // When a label is present (e.g., "current UTC time"), use the enhanced format:
    // ('current UTC time': 2026-03-02 20:40:13 UTC (+00:00))
    if (this.label !== null) {
      const label = this.label;
      const offsetSecs = this.offsetSecondsValue ?? 0;
      const tzDisplay = DateTime.formatTzForDisplay(offsetSecs, this.tzAbbrev);
      const offset = this.getOffset();
      if (offset !== null) {
        const local = this.inner.withTimezone(offset);
        return `('${label}': ${local.format('%Y-%m-%d %H:%M:%S')} ${tzDisplay})`;
      }
      return `('${label}': ${this.inner.format('%Y-%m-%d %H:%M:%S')} ${tzDisplay})`;
    }

    if (this.hasDateValue && this.hasTimeValue) {
      const offset = this.getOffset();
      if (offset !== null) {
        const local = this.inner.withTimezone(offset);
        if (this.tzAbbrev !== null) {
          return `${local.format('%Y-%m-%d %H:%M:%S')} ${this.tzAbbrev}`;
        }
        return local.format('%Y-%m-%d %H:%M:%S %:z');
      }
      return this.inner.format('%Y-%m-%d %H:%M:%S UTC');
    } else if (this.hasDateValue) {
      return this.inner.format('%Y-%m-%d');
    } else if (this.hasTimeValue) {
      const offset = this.getOffset();
      if (offset !== null) {
        const local = this.inner.withTimezone(offset);
        if (this.tzAbbrev !== null) {
          return `${local.format('%H:%M:%S')} ${this.tzAbbrev}`;
        }
        return local.format('%H:%M:%S %:z');
      }
      return this.inner.format('%H:%M:%S UTC');
    }
    return this.inner.toString();
  }

  /**
   * `FromStr`; throws `Err(CalculatorError)`.
   * @param {string} s
   */
  static fromStr(s) {
    return DateTime.parse(s);
  }

  /** `Serialize`: serde's field names and order. */
  toJSON() {
    /** @type {Record<string, unknown>} */
    const json = {
      inner: this.inner.toJSON(),
      offset_seconds: this.offsetSecondsValue,
      has_time: this.hasTimeValue,
      has_date: this.hasDateValue,
    };
    if (this.label !== null) {
      json.label = this.label;
    }
    if (this.tzAbbrev !== null) {
      json.tz_abbrev = this.tzAbbrev;
    }
    return json;
  }
}

/** Browser-friendly metadata for displaying timezone conversions. */
export class DateTimeResult {
  /**
   * @param {{ source: string, utc: string, epochMilliseconds: number, hasDate: boolean,
   *   hasTime: boolean, timezone: string | null, offsetSeconds: number | null }} fields
   */
  constructor({ source, utc, epochMilliseconds, hasDate, hasTime, timezone, offsetSeconds }) {
    /** The calculated value in its source timezone. */
    this.source = source;
    /** The same instant formatted in UTC. */
    this.utc = utc;
    /** UTC timestamp in milliseconds since the Unix epoch. */
    this.epochMilliseconds = epochMilliseconds;
    /** Whether the original expression includes a date component. */
    this.hasDate = hasDate;
    /** Whether the original expression includes a time component. */
    this.hasTime = hasTime;
    /** Source timezone abbreviation, when known. */
    this.timezone = timezone;
    /** Source timezone offset in seconds, when known. */
    this.offsetSeconds = offsetSeconds;
  }

  /**
   * Creates conversion display metadata for a timezone-aware datetime value.
   * @param {DateTime} dt
   * @returns {DateTimeResult | null}
   */
  static fromDatetime(dt) {
    const utc = dt.utcEquivalentDisplay();
    if (utc === null) {
      return null;
    }
    return new DateTimeResult({
      source: dt.toString(),
      utc,
      epochMilliseconds: dt.timestampMillis(),
      hasDate: dt.hasDate(),
      hasTime: dt.hasTime(),
      timezone: dt.timezoneAbbreviation(),
      offsetSeconds: dt.offsetSeconds(),
    });
  }

  /** `Serialize`: serde's field names and order. */
  toJSON() {
    /** @type {Record<string, unknown>} */
    const json = {
      source: this.source,
      utc: this.utc,
      epoch_milliseconds: this.epochMilliseconds,
      has_date: this.hasDate,
      has_time: this.hasTime,
    };
    if (this.timezone !== null) {
      json.timezone = this.timezone;
    }
    if (this.offsetSeconds !== null) {
      json.offset_seconds = this.offsetSeconds;
    }
    return json;
  }
}
