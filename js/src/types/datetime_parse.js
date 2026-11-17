/**
 * Parsing helpers for the `DateTime` type (port of `src/types/datetime_parse.rs`).
 *
 * This module contains standalone helper functions for parsing dates, times,
 * timezones, and natural-language date strings.
 */

import { FixedOffset, NaiveDate, NaiveTime } from '../rust/chrono.js';
import { now } from '../clock.js';
import { ok } from '../rust/core.js';
import { I32, U32 } from '../rust/num.js';
import {
  endsWith, find, indexToByte, isAlphabetic, rfind, sliceBytes, split, splitWhitespace, startsWith,
  trim, trimEnd, trimEndMatches, trimStart, trimStartMatches, utf8Len,
} from '../rust/str.js';

/** Rust's Unicode `\w` (the `regex` crate's word class). */
const WORD = '[\\p{Alphabetic}\\p{M}\\p{Nd}\\p{Pc}\\p{Join_Control}]';

/**
 * @param {string} tz
 * @returns {string}
 */
function normalizeTzAbbreviation(tz) {
  const other = trim(tz).toLowerCase();
  switch (other) {
    case 'мск':
      return 'MSK';
    default:
      return other.toUpperCase();
  }
}

/**
 * Translates month names from any supported UI language to English.
 *
 * Supported languages: Russian (ru), German (de), French (fr),
 * Chinese Simplified (zh), Hindi (hi), Arabic (ar).
 * English is already understood by the date parser, so no translation needed.
 *
 * Returns the input unchanged if no translatable month name is found.
 * @param {string} input
 * @returns {string}
 */
export function translateMonthNames(input) {
  const lower = input.toLowerCase();

  // All month name → English month name mappings.
  // Ordered from longest to shortest within each language to prevent partial matches.
  /** @type {[string, string][]} */
  const translations = [
    // ── Russian (ru) ─────────────────────────────────────────────────────
    // Genitive/prepositional forms (most common in dates like "17 февраля")
    ['января', 'January'],
    ['февраля', 'February'],
    ['марта', 'March'],
    ['апреля', 'April'],
    ['мая', 'May'],
    ['июня', 'June'],
    ['июля', 'July'],
    ['августа', 'August'],
    ['сентября', 'September'],
    ['октября', 'October'],
    ['ноября', 'November'],
    ['декабря', 'December'],
    // Nominative forms
    ['январь', 'January'],
    ['февраль', 'February'],
    ['март', 'March'],
    ['апрель', 'April'],
    ['май', 'May'],
    ['июнь', 'June'],
    ['июль', 'July'],
    ['август', 'August'],
    ['сентябрь', 'September'],
    ['октябрь', 'October'],
    ['ноябрь', 'November'],
    ['декабрь', 'December'],
    // Russian abbreviated forms
    ['янв', 'Jan'],
    ['фев', 'Feb'],
    ['мар', 'Mar'],
    ['апр', 'Apr'],
    ['июн', 'Jun'],
    ['июл', 'Jul'],
    ['авг', 'Aug'],
    ['сент', 'Sep'],
    ['сен', 'Sep'],
    ['окт', 'Oct'],
    ['ноя', 'Nov'],
    ['дек', 'Dec'],
    // ── German (de) ──────────────────────────────────────────────────────
    // German month names (same spelling in nominative/genitive)
    ['januar', 'January'],
    ['februar', 'February'],
    ['märz', 'March'],
    ['april', 'April'],
    ['juni', 'June'],
    ['juli', 'July'],
    ['august', 'August'],
    ['september', 'September'],
    ['oktober', 'October'],
    ['november', 'November'],
    ['dezember', 'December'],
    // German: "Mai" collides with English "May" (already handled); skip
    // ── French (fr) ──────────────────────────────────────────────────────
    // French month names (lowercase; French capitalizes only at sentence start)
    ['janvier', 'January'],
    ['février', 'February'],
    ['fevrier', 'February'], // without accent, common input variant
    ['mars', 'March'],
    // "avril" handled below after longer strings
    ['mai', 'May'],
    ['juin', 'June'],
    ['juillet', 'July'],
    ['août', 'August'],
    ['aout', 'August'], // without accent
    ['septembre', 'September'],
    ['octobre', 'October'],
    ['novembre', 'November'],
    ['décembre', 'December'],
    ['decembre', 'December'], // without accent
    ['avril', 'April'],
    // ── Chinese Simplified (zh) ───────────────────────────────────────────
    // Chinese uses numeric months but also有 full-word forms like "一月" (January)
    ['一月', 'January'],
    ['二月', 'February'],
    ['三月', 'March'],
    ['四月', 'April'],
    ['五月', 'May'],
    ['六月', 'June'],
    ['七月', 'July'],
    ['八月', 'August'],
    ['九月', 'September'],
    ['十月', 'October'],
    ['十一月', 'November'],
    ['十二月', 'December'],
    // ── Hindi (hi) ───────────────────────────────────────────────────────
    // Hindi month names (Devanagari script)
    ['जनवरी', 'January'],
    ['फरवरी', 'February'],
    ['मार्च', 'March'],
    ['अप्रैल', 'April'],
    ['मई', 'May'],
    ['जून', 'June'],
    ['जुलाई', 'July'],
    ['अगस्त', 'August'],
    ['सितंबर', 'September'],
    ['सितम्बर', 'September'], // alternate spelling
    ['अक्टूबर', 'October'],
    ['नवंबर', 'November'],
    ['नवम्बर', 'November'], // alternate spelling
    ['दिसंबर', 'December'],
    ['दिसम्बर', 'December'], // alternate spelling
    // ── Arabic (ar) ──────────────────────────────────────────────────────
    // Arabic month names (used in modern Arab countries; MSA/Levantine)
    ['يناير', 'January'],
    ['فبراير', 'February'],
    ['مارس', 'March'],
    ['أبريل', 'April'],
    ['ابريل', 'April'], // without hamza
    ['مايو', 'May'],
    ['يونيو', 'June'],
    ['يوليو', 'July'],
    ['أغسطس', 'August'],
    ['اغسطس', 'August'], // without hamza
    ['سبتمبر', 'September'],
    ['أكتوبر', 'October'],
    ['اكتوبر', 'October'], // without hamza
    ['نوفمبر', 'November'],
    ['ديسمبر', 'December'],
  ];

  let result = input;
  for (const [native, english] of translations) {
    // All patterns must match as whole words to avoid partial matches.
    // For example, German "januar" must not match inside English "january".
    const index = find(lower, native);
    if (index !== null) {
      // Byte offsets into `lower`, which Rust also uses to slice `result`.
      const pos = indexToByte(lower, index);
      const end = pos + utf8Len(native);
      // Check word boundaries: the character before and after must not be alphabetic.
      const beforeOk = pos === 0 || (() => {
        const ch = [...sliceBytes(lower, 0, pos)].pop();
        return ch === undefined || !isAlphabetic(ch);
      })();
      const afterOk = end >= utf8Len(lower) || (() => {
        const ch = [...sliceBytes(lower, end)][0];
        return ch === undefined || !isAlphabetic(ch);
      })();
      if (beforeOk && afterOk) {
        result = `${sliceBytes(result, 0, pos)}${english}${sliceBytes(result, end)}`;
        return result; // Only translate the first found month name
      }
    }
  }
  return result;
}

/** `(?i)(\d{4})\s*(года|году|годом|год|гг?\.?)\s*$` */
const YEAR_MARKER = /(\p{Nd}{4})\p{White_Space}*(года|году|годом|год|гг?\.?)\p{White_Space}*$/iu;

/**
 * Removes a trailing year marker word that follows a four-digit year.
 *
 * Russian dates are commonly written as "17 августа 2026 года" (or the
 * abbreviated "17 августа 2026 г."), where "года"/"г." only marks the
 * preceding number as a year and carries no additional information.
 * @param {string} input
 * @returns {string}
 */
function stripYearMarker(input) {
  return trimEnd(input.replace(YEAR_MARKER, (_m, year) => year));
}

/** `(?i)\b(?:on|of)\b` */
const PREPOSITIONS = new RegExp(`(?<!${WORD})(?:on|of)(?!${WORD})`, 'giu');
/** `(\d{1,2})(st|nd|rd|th)\b` */
const ORDINAL_SUFFIX = new RegExp(`(\\p{Nd}{1,2})(st|nd|rd|th)(?!${WORD})`, 'gu');

/**
 * Pre-processes natural date strings by removing day names, ordinal suffixes,
 * and optional "on"/"of" prepositions.
 * @param {string} input
 * @returns {string}
 */
export function preprocessNaturalDate(input) {
  let result = stripYearMarker(input);

  // Remove day names (case-insensitive)
  const dayNames = [
    'monday',
    'tuesday',
    'wednesday',
    'thursday',
    'friday',
    'saturday',
    'sunday',
    'mon',
    'tue',
    'wed',
    'thu',
    'fri',
    'sat',
    'sun',
  ];
  const lower = result.toLowerCase();
  for (const day of dayNames) {
    const index = find(lower, day);
    if (index !== null) {
      const pos = indexToByte(lower, index);
      const end = pos + utf8Len(day);
      // Remove the day name and any trailing comma/space
      const prefix = trimEnd(sliceBytes(result, 0, pos));
      const suffixRaw = trimStart(sliceBytes(result, end));
      const suffix = trimStart(suffixRaw.startsWith(',') ? suffixRaw.slice(1) : suffixRaw);
      result = prefix === '' ? suffix : `${prefix} ${suffix}`;
      result = trim(result);
      break;
    }
  }

  // Remove optional prepositions used in natural date phrases such as
  // "on January 26th" and "8th of August".
  result = result.replace(PREPOSITIONS, '');
  result = splitWhitespace(result).join(' ');

  // Remove ordinal suffixes from day numbers (1st, 2nd, 3rd, 4th-31st)
  result = result.replace(ORDINAL_SUFFIX, (_m, day) => day);

  return result;
}

/**
 * Parses common timezone abbreviations to `FixedOffset`.
 *
 * Returns `None` if the abbreviation is not recognized.
 * Supports half-hour and 45-minute offsets (e.g., IST +5:30, NPT +5:45).
 * @param {string} tz
 * @returns {FixedOffset | null}
 */
export function parseTzAbbreviation(tz) {
  const normalizedTz = normalizeTzAbbreviation(tz);

  // Handle half-hour and non-standard offsets first (return early)
  switch (normalizedTz) {
    case 'IST':
      return FixedOffset.eastOpt(5 * 3600 + 30 * 60); // India +5:30
    case 'ACST':
      return FixedOffset.eastOpt(9 * 3600 + 30 * 60); // Australia Central +9:30
    case 'ACDT':
      return FixedOffset.eastOpt(10 * 3600 + 30 * 60); // Australia Central DT +10:30
    case 'NPT':
      return FixedOffset.eastOpt(5 * 3600 + 45 * 60); // Nepal +5:45
    case 'CHAST':
      return FixedOffset.eastOpt(12 * 3600 + 45 * 60); // Chatham Standard +12:45
    case 'CHADT':
      return FixedOffset.eastOpt(13 * 3600 + 45 * 60); // Chatham DT +13:45
    case 'MMT':
      return FixedOffset.eastOpt(6 * 3600 + 30 * 60); // Myanmar +6:30
    case 'AFT':
      return FixedOffset.eastOpt(4 * 3600 + 30 * 60); // Afghanistan +4:30
    case 'IRDT':
      return FixedOffset.eastOpt(4 * 3600 + 30 * 60); // Iran DT +4:30
    case 'IRST':
      return FixedOffset.eastOpt(3 * 3600 + 30 * 60); // Iran Standard +3:30
    case 'NST':
      return FixedOffset.eastOpt(-(3 * 3600 + 30 * 60)); // Newfoundland Standard -3:30
    case 'NDT':
      return FixedOffset.eastOpt(-(2 * 3600 + 30 * 60)); // Newfoundland DT -2:30
    default:
  }

  let offsetHours;
  switch (normalizedTz) {
    case 'UTC': case 'GMT': case 'GTM': case 'Z':
      offsetHours = 0; // GTM: common typo for GMT
      break;
    // US & Canada timezones
    case 'EST':
      offsetHours = -5;
      break;
    case 'EDT':
      offsetHours = -4;
      break;
    case 'CST':
      offsetHours = -6;
      break;
    case 'CDT':
      offsetHours = -5;
      break;
    case 'MST':
      offsetHours = -7;
      break;
    case 'MDT':
      offsetHours = -6;
      break;
    case 'PST':
      offsetHours = -8;
      break;
    case 'PDT':
      offsetHours = -7;
      break;
    case 'AKST':
      offsetHours = -9;
      break;
    case 'AKDT':
      offsetHours = -8;
      break;
    case 'HST': case 'HAST':
      offsetHours = -10;
      break;
    case 'AST':
      offsetHours = -4; // Atlantic Standard Time
      break;
    case 'ADT':
      offsetHours = -3; // Atlantic Daylight Time
      break;
    // European timezones
    case 'CET':
      offsetHours = 1;
      break;
    case 'CEST':
      offsetHours = 2;
      break;
    case 'EET':
      offsetHours = 2;
      break;
    case 'EEST':
      offsetHours = 3;
      break;
    case 'WET':
      offsetHours = 0;
      break;
    case 'WEST':
      offsetHours = 1;
      break;
    case 'GMT+1': case 'BST':
      offsetHours = 1; // British Summer Time
      break;
    case 'MSK':
      offsetHours = 3; // Moscow Standard Time
      break;
    case 'MSD':
      offsetHours = 4; // Moscow Summer Time (historical)
      break;
    case 'SAMT':
      offsetHours = 4; // Samara Time
      break;
    case 'YEKT':
      offsetHours = 5; // Yekaterinburg Time
      break;
    case 'OMST':
      offsetHours = 6; // Omsk Time
      break;
    case 'KRAT':
      offsetHours = 7; // Krasnoyarsk Time
      break;
    case 'IRKT':
      offsetHours = 8; // Irkutsk Time
      break;
    case 'YAKT':
      offsetHours = 9; // Yakutsk Time
      break;
    case 'VLAT':
      offsetHours = 10; // Vladivostok Time
      break;
    case 'MAGT':
      offsetHours = 11; // Magadan Time
      break;
    case 'PETT':
      offsetHours = 12; // Kamchatka Time
      break;
    case 'TRT':
      offsetHours = 3; // Turkey Time
      break;
    // Middle East & Central Asia
    case 'GST':
      offsetHours = 4; // Gulf Standard Time
      break;
    case 'AZT':
      offsetHours = 4; // Azerbaijan Time
      break;
    case 'GET':
      offsetHours = 4; // Georgia Time
      break;
    case 'AMT':
      offsetHours = 4; // Armenia Time
      break;
    case 'PKT':
      offsetHours = 5; // Pakistan Standard Time
      break;
    case 'UZT':
      offsetHours = 5; // Uzbekistan Time
      break;
    case 'TMT':
      offsetHours = 5; // Turkmenistan Time
      break;
    case 'TJT':
      offsetHours = 5; // Tajikistan Time
      break;
    case 'KGT':
      offsetHours = 6; // Kyrgyzstan Time
      break;
    case 'ALMT':
      offsetHours = 6; // Almaty Time (Kazakhstan)
      break;
    case 'QYZT':
      offsetHours = 6; // Qyzylorda Time (Kazakhstan)
      break;
    // Southeast & East Asian timezones
    case 'BDT':
      offsetHours = 6; // Bangladesh Time
      break;
    case 'ICT':
      offsetHours = 7; // Indochina Time (Thailand, Vietnam, Laos, Cambodia)
      break;
    case 'WIB':
      offsetHours = 7; // Western Indonesia Time
      break;
    case 'WITA':
      offsetHours = 8; // Central Indonesia Time
      break;
    case 'WIT':
      offsetHours = 9; // Eastern Indonesia Time
      break;
    case 'CST+8': case 'SGT': case 'HKT': case 'PHT': case 'MYT':
      offsetHours = 8; // Singapore, Hong Kong, Philippines, Malaysia
      break;
    case 'JST':
      offsetHours = 9; // Japan Standard Time
      break;
    case 'KST':
      offsetHours = 9; // Korea Standard Time
      break;
    case 'TWT':
      offsetHours = 8; // Taiwan Time
      break;
    // Australian timezones
    case 'AEST':
      offsetHours = 10;
      break;
    case 'AEDT':
      offsetHours = 11;
      break;
    case 'AWST':
      offsetHours = 8;
      break;
    // New Zealand
    case 'NZST':
      offsetHours = 12;
      break;
    case 'NZDT':
      offsetHours = 13;
      break;
    // South America
    case 'ART': case 'BRT': case 'CLST': case 'UYT': case 'GFT': case 'SRT':
      offsetHours = -3;
      break;
    case 'BRST':
      offsetHours = -2;
      break;
    case 'CLT': case 'VET': case 'BOT': case 'PYT':
      offsetHours = -4;
      break;
    case 'COT': case 'PET': case 'ECT':
      offsetHours = -5;
      break;
    // Africa
    case 'WAT':
      offsetHours = 1; // West Africa Time
      break;
    case 'CAT':
      offsetHours = 2; // Central Africa Time
      break;
    case 'EAT':
      offsetHours = 3; // East Africa Time
      break;
    case 'SAST':
      offsetHours = 2; // South Africa Standard Time
      break;
    // Atlantic & misc
    case 'AZOT':
      offsetHours = -1; // Azores Time
      break;
    case 'CVT':
      offsetHours = -1; // Cape Verde Time
      break;
    default:
      return null;
  }
  return FixedOffset.eastOpt(offsetHours * 3600);
}

/**
 * Normalizes month names by capitalizing the first letter of each word.
 * @param {string} input
 * @returns {string}
 */
export function normalizeMonthName(input) {
  return splitWhitespace(input)
    .map((word) => {
      const [c, ...chars] = [...word];
      if (c === undefined) {
        return '';
      }
      return c.toUpperCase() + chars.join('').toLowerCase();
    })
    .join(' ');
}

/**
 * Parses a partial date (month + day, no year) using the current year.
 * @param {string} input
 * @returns {NaiveDate | null}
 */
export function parsePartialDate(input) {
  const normalized = normalizeMonthName(input);
  const currentYear = now().year();

  // "Jan 27" or "January 27"
  let md = ok(() => NaiveDate.parseFromStr(`${normalized} ${currentYear}`, '%b %d %Y'));
  if (md !== null) {
    return md;
  }
  md = ok(() => NaiveDate.parseFromStr(`${normalized} ${currentYear}`, '%B %d %Y'));
  if (md !== null) {
    return md;
  }

  // "27 Jan" format
  md = ok(() => NaiveDate.parseFromStr(`${normalized} ${currentYear}`, '%d %b %Y'));
  if (md !== null) {
    return md;
  }
  md = ok(() => NaiveDate.parseFromStr(`${normalized} ${currentYear}`, '%d %B %Y'));
  if (md !== null) {
    return md;
  }

  return null;
}

/**
 * Parses a 12-hour time string (e.g., "8:59am", "12:51pm").
 * @param {string} input
 * @returns {NaiveTime | null}
 */
export function parse12hTime(input) {
  input = trim(input).toLowerCase();

  // Patterns: 8:59am, 8:59 am, 12:51pm
  let timeStr;
  let isPm;
  if (endsWith(input, 'am')) {
    [timeStr, isPm] = [trim(trimEndMatches(input, 'am')), false];
  } else if (endsWith(input, 'pm')) {
    [timeStr, isPm] = [trim(trimEndMatches(input, 'pm')), true];
  } else {
    return null;
  }

  const parts = split(timeStr, ':');
  if (parts.length < 2) {
    return null;
  }

  let hour = ok(() => Number(U32.parse(parts[0])));
  if (hour === null) {
    return null;
  }
  const minute = ok(() => Number(U32.parse(parts[1])));
  if (minute === null) {
    return null;
  }
  const second = (parts.length > 2 ? ok(() => Number(U32.parse(parts[2]))) : null) ?? 0;

  if (isPm && hour !== 12) {
    // `hour + 12` wraps in release builds.
    hour = Number(U32.wrap(hour + 12));
  } else if (!isPm && hour === 12) {
    hour = 0;
  }

  return NaiveTime.fromHmsOpt(hour, minute, second);
}

/**
 * Extracts timezone information from the end of a time string.
 *
 * Returns (time_part, offset, optional_tz_abbreviation).
 * @param {string} input
 * @returns {[string, FixedOffset | null, string | null]}
 */
export function extractTimezone(input) {
  input = trim(input);

  // Check for UTC/GMT suffix
  const upper = input.toUpperCase();
  if (upper.endsWith('UTC') || upper.endsWith('GMT') || upper.endsWith('GTM')) {
    const suffixLen = 3;
    const len = utf8Len(input);
    const timePart = trim(sliceBytes(input, 0, len - suffixLen));
    const tzStr = sliceBytes(input, len - suffixLen).toUpperCase();
    // Normalize GTM to GMT
    const tzAbbrev = tzStr === 'GTM' ? 'GMT' : tzStr;
    return [timePart, FixedOffset.eastOpt(0), tzAbbrev];
  }

  // Check for common timezone abbreviations as suffix
  // Try to match the last word as a timezone abbreviation
  const lastSpace = rfind(input, ' ');
  if (lastSpace !== null) {
    const potentialTz = input.slice(lastSpace + 1);
    const offset = parseTzAbbreviation(potentialTz);
    if (offset !== null) {
      const timePart = trim(input.slice(0, lastSpace));
      return [timePart, offset, normalizeTzAbbreviation(potentialTz)];
    }
  }

  // Check for explicit offset like +05:00 or -08:00
  const idx = rfind(input, '+') ?? (() => {
    // Find the last minus that's not at the start
    const lastMinus = rfind(input, '-');
    if (lastMinus === null) {
      return null;
    }
    return lastMinus > 0 ? lastMinus : null;
  })();
  if (idx !== null) {
    const offsetStr = input.slice(idx);
    const offset = parseOffset(offsetStr);
    if (offset !== null) {
      return [input.slice(0, idx), offset, null];
    }
  }

  return [input, null, null];
}

/**
 * Parses a UTC offset string like "+05:00" or "-08:00".
 * @param {string} offsetStr
 * @returns {FixedOffset | null}
 */
export function parseOffset(offsetStr) {
  const sign = startsWith(offsetStr, '-') ? -1 : 1;
  offsetStr = trimStartMatches(offsetStr, ['+', '-']);

  const parts = split(offsetStr, ':');
  const hours = ok(() => Number(I32.parse(parts[0])));
  if (hours === null) {
    return null;
  }
  const minutes = (parts.length > 1 ? ok(() => Number(I32.parse(parts[1]))) : null) ?? 0;

  // The i32 arithmetic wraps in release builds.
  const totalSeconds = Number(I32.wrap(
    BigInt(sign) * I32.wrap(I32.wrap(BigInt(hours) * 3600n) + I32.wrap(BigInt(minutes) * 60n)),
  ));
  return FixedOffset.eastOpt(totalSeconds);
}
