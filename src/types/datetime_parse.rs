//! Parsing helpers for the `DateTime` type.
//!
//! This module contains standalone helper functions for parsing dates, times,
//! timezones, and natural-language date strings.

use chrono::Datelike;
use chrono::{FixedOffset, NaiveDate, NaiveTime, Utc};
use regex;

/// Translates Russian month names (in any grammatical case) to their English equivalents.
///
/// Russian month names decline in all 6 grammatical cases. This function handles
/// the most common forms used in date expressions:
/// - Nominative: январь, февраль, март, апрель, май, июнь, июль, август, сентябрь, октябрь, ноябрь, декабрь
/// - Genitive (most common in dates like "17 февраля 2027"): января, февраля, марта, апреля, мая, июня, июля, августа, сентября, октября, ноября, декабря
///
/// Returns the input unchanged if no Russian month names are found.
pub(super) fn translate_russian_months(input: &str) -> String {
    let lower = input.to_lowercase();

    // Russian month name → English month name mapping.
    // Ordered from longest to shortest to prevent partial matches.
    let translations: &[(&str, &str)] = &[
        // Genitive/prepositional forms (most common in dates like "17 февраля")
        ("января", "January"),
        ("февраля", "February"),
        ("марта", "March"),
        ("апреля", "April"),
        ("июня", "June"),
        ("июля", "July"),
        ("августа", "August"),
        ("сентября", "September"),
        ("октября", "October"),
        ("ноября", "November"),
        ("декабря", "December"),
        // Nominative forms
        ("январь", "January"),
        ("февраль", "February"),
        ("март", "March"),
        ("апрель", "April"),
        ("май", "May"),
        ("июнь", "June"),
        ("июль", "July"),
        ("август", "August"),
        ("сентябрь", "September"),
        ("октябрь", "October"),
        ("ноябрь", "November"),
        ("декабрь", "December"),
        // Abbreviated forms (common in informal usage)
        ("янв", "Jan"),
        ("фев", "Feb"),
        ("мар", "Mar"),
        ("апр", "Apr"),
        ("авг", "Aug"),
        ("сен", "Sep"),
        ("окт", "Oct"),
        ("ноя", "Nov"),
        ("дек", "Dec"),
    ];

    let mut result = input.to_string();
    for (russian, english) in translations {
        if let Some(pos) = lower.find(russian) {
            result = format!(
                "{}{}{}",
                &result[..pos],
                english,
                &result[pos + russian.len()..]
            );
            return result; // Only translate the first found month name
        }
    }
    result
}

/// Pre-processes natural date strings by removing day names, ordinal suffixes,
/// and the "on" preposition.
pub(super) fn preprocess_natural_date(input: &str) -> String {
    let mut result = input.to_string();

    // Remove day names (case-insensitive)
    let day_names = [
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
        "mon",
        "tue",
        "wed",
        "thu",
        "fri",
        "sat",
        "sun",
    ];
    let lower = result.to_lowercase();
    for day in &day_names {
        if let Some(pos) = lower.find(day) {
            let end = pos + day.len();
            // Remove the day name and any trailing comma/space
            let prefix = result[..pos].trim_end().to_string();
            let suffix_raw = result[end..].trim_start().to_string();
            let suffix = suffix_raw
                .strip_prefix(',')
                .unwrap_or(&suffix_raw)
                .trim_start()
                .to_string();
            result = if prefix.is_empty() {
                suffix
            } else {
                format!("{prefix} {suffix}")
            };
            result = result.trim().to_string();
            break;
        }
    }

    // Remove "on " preposition (often between time and date)
    result = result.replace(" on ", " ");

    // Remove ordinal suffixes from day numbers (1st, 2nd, 3rd, 4th-31st)
    let re_result = regex::Regex::new(r"(\d{1,2})(st|nd|rd|th)\b");
    if let Ok(re) = re_result {
        result = re.replace_all(&result, "$1").to_string();
    }

    result
}

/// Parses common timezone abbreviations to `FixedOffset`.
///
/// Returns `None` if the abbreviation is not recognized.
/// Supports half-hour and 45-minute offsets (e.g., IST +5:30, NPT +5:45).
pub(super) fn parse_tz_abbreviation(tz: &str) -> Option<FixedOffset> {
    // Handle half-hour and non-standard offsets first (return early)
    match tz.to_uppercase().as_str() {
        "IST" => return FixedOffset::east_opt(5 * 3600 + 30 * 60), // India +5:30
        "ACST" => return FixedOffset::east_opt(9 * 3600 + 30 * 60), // Australia Central +9:30
        "ACDT" => return FixedOffset::east_opt(10 * 3600 + 30 * 60), // Australia Central DT +10:30
        "NPT" => return FixedOffset::east_opt(5 * 3600 + 45 * 60), // Nepal +5:45
        "CHAST" => return FixedOffset::east_opt(12 * 3600 + 45 * 60), // Chatham Standard +12:45
        "CHADT" => return FixedOffset::east_opt(13 * 3600 + 45 * 60), // Chatham DT +13:45
        "MMT" => return FixedOffset::east_opt(6 * 3600 + 30 * 60), // Myanmar +6:30
        "AFT" => return FixedOffset::east_opt(4 * 3600 + 30 * 60), // Afghanistan +4:30
        "IRDT" => return FixedOffset::east_opt(4 * 3600 + 30 * 60), // Iran DT +4:30
        "IRST" => return FixedOffset::east_opt(3 * 3600 + 30 * 60), // Iran Standard +3:30
        "NST" => return FixedOffset::east_opt(-(3 * 3600 + 30 * 60)), // Newfoundland Standard -3:30
        "NDT" => return FixedOffset::east_opt(-(2 * 3600 + 30 * 60)), // Newfoundland DT -2:30
        _ => {}
    }

    let offset_hours = match tz.to_uppercase().as_str() {
        "UTC" | "GMT" | "GTM" | "Z" => 0, // GTM: common typo for GMT
        // US & Canada timezones
        "EST" => -5,
        "EDT" => -4,
        "CST" => -6,
        "CDT" => -5,
        "MST" => -7,
        "MDT" => -6,
        "PST" => -8,
        "PDT" => -7,
        "AKST" => -9,
        "AKDT" => -8,
        "HST" | "HAST" => -10,
        "AST" => -4, // Atlantic Standard Time
        "ADT" => -3, // Atlantic Daylight Time
        // European timezones
        "CET" => 1,
        "CEST" => 2,
        "EET" => 2,
        "EEST" => 3,
        "WET" => 0,
        "WEST" => 1,
        "GMT+1" | "BST" => 1, // British Summer Time
        "MSK" => 3,           // Moscow Standard Time
        "MSD" => 4,           // Moscow Summer Time (historical)
        "SAMT" => 4,          // Samara Time
        "YEKT" => 5,          // Yekaterinburg Time
        "OMST" => 6,          // Omsk Time
        "KRAT" => 7,          // Krasnoyarsk Time
        "IRKT" => 8,          // Irkutsk Time
        "YAKT" => 9,          // Yakutsk Time
        "VLAT" => 10,         // Vladivostok Time
        "MAGT" => 11,         // Magadan Time
        "PETT" => 12,         // Kamchatka Time
        "TRT" => 3,           // Turkey Time
        // Middle East & Central Asia
        "GST" => 4,  // Gulf Standard Time
        "AZT" => 4,  // Azerbaijan Time
        "GET" => 4,  // Georgia Time
        "AMT" => 4,  // Armenia Time
        "PKT" => 5,  // Pakistan Standard Time
        "UZT" => 5,  // Uzbekistan Time
        "TMT" => 5,  // Turkmenistan Time
        "TJT" => 5,  // Tajikistan Time
        "KGT" => 6,  // Kyrgyzstan Time
        "ALMT" => 6, // Almaty Time (Kazakhstan)
        "QYZT" => 6, // Qyzylorda Time (Kazakhstan)
        // Southeast & East Asian timezones
        "BDT" => 6,                                   // Bangladesh Time
        "ICT" => 7,  // Indochina Time (Thailand, Vietnam, Laos, Cambodia)
        "WIB" => 7,  // Western Indonesia Time
        "WITA" => 8, // Central Indonesia Time
        "WIT" => 9,  // Eastern Indonesia Time
        "CST+8" | "SGT" | "HKT" | "PHT" | "MYT" => 8, // Singapore, Hong Kong, Philippines, Malaysia
        "JST" => 9,  // Japan Standard Time
        "KST" => 9,  // Korea Standard Time
        "TWT" => 8,  // Taiwan Time
        // Australian timezones
        "AEST" => 10,
        "AEDT" => 11,
        "AWST" => 8,
        // New Zealand
        "NZST" => 12,
        "NZDT" => 13,
        // South America
        "ART" | "BRT" | "CLST" | "UYT" | "GFT" | "SRT" => -3,
        "BRST" => -2,
        "CLT" | "VET" | "BOT" | "PYT" => -4,
        "COT" | "PET" | "ECT" => -5,
        // Africa
        "WAT" => 1,  // West Africa Time
        "CAT" => 2,  // Central Africa Time
        "EAT" => 3,  // East Africa Time
        "SAST" => 2, // South Africa Standard Time
        // Atlantic & misc
        "AZOT" => -1, // Azores Time
        "CVT" => -1,  // Cape Verde Time
        _ => return None,
    };
    FixedOffset::east_opt(offset_hours * 3600)
}

/// Normalizes month names by capitalizing the first letter of each word.
pub(super) fn normalize_month_name(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parses a partial date (month + day, no year) using the current year.
pub(super) fn parse_partial_date(input: &str) -> Option<NaiveDate> {
    let normalized = normalize_month_name(input);
    let current_year = Utc::now().year();

    // "Jan 27" or "January 27"
    if let Ok(md) = NaiveDate::parse_from_str(&format!("{normalized} {current_year}"), "%b %d %Y") {
        return Some(md);
    }
    if let Ok(md) = NaiveDate::parse_from_str(&format!("{normalized} {current_year}"), "%B %d %Y") {
        return Some(md);
    }

    // "27 Jan" format
    if let Ok(md) = NaiveDate::parse_from_str(&format!("{normalized} {current_year}"), "%d %b %Y") {
        return Some(md);
    }

    None
}

/// Parses a 12-hour time string (e.g., "8:59am", "12:51pm").
pub(super) fn parse_12h_time(input: &str) -> Option<NaiveTime> {
    let input = input.trim().to_lowercase();

    // Patterns: 8:59am, 8:59 am, 12:51pm
    let (time_str, is_pm) = if input.ends_with("am") {
        (input.trim_end_matches("am").trim(), false)
    } else if input.ends_with("pm") {
        (input.trim_end_matches("pm").trim(), true)
    } else {
        return None;
    };

    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let hour: u32 = parts[0].parse().ok()?;
    let minute: u32 = parts[1].parse().ok()?;
    let second: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    let hour = if is_pm && hour != 12 {
        hour + 12
    } else if !is_pm && hour == 12 {
        0
    } else {
        hour
    };

    NaiveTime::from_hms_opt(hour, minute, second)
}

/// Extracts timezone information from the end of a time string.
///
/// Returns (time_part, offset, optional_tz_abbreviation).
pub(super) fn extract_timezone(input: &str) -> (&str, Option<FixedOffset>, Option<String>) {
    let input = input.trim();

    // Check for UTC/GMT suffix
    let upper = input.to_uppercase();
    if upper.ends_with("UTC") || upper.ends_with("GMT") || upper.ends_with("GTM") {
        let suffix_len = 3;
        let time_part = input[..input.len() - suffix_len].trim();
        let tz_str = input[input.len() - suffix_len..].to_uppercase();
        // Normalize GTM to GMT
        let tz_abbrev = if tz_str == "GTM" {
            "GMT".to_string()
        } else {
            tz_str
        };
        return (
            time_part,
            Some(FixedOffset::east_opt(0).unwrap()),
            Some(tz_abbrev),
        );
    }

    // Check for common timezone abbreviations as suffix
    // Try to match the last word as a timezone abbreviation
    if let Some(last_space) = input.rfind(' ') {
        let potential_tz = &input[last_space + 1..];
        if let Some(offset) = parse_tz_abbreviation(potential_tz) {
            let time_part = input[..last_space].trim();
            return (time_part, Some(offset), Some(potential_tz.to_uppercase()));
        }
    }

    // Check for explicit offset like +05:00 or -08:00
    if let Some(idx) = input.rfind('+').or_else(|| {
        // Find the last minus that's not at the start
        let last_minus = input.rfind('-')?;
        if last_minus > 0 {
            Some(last_minus)
        } else {
            None
        }
    }) {
        let offset_str = &input[idx..];
        if let Some(offset) = parse_offset(offset_str) {
            return (&input[..idx], Some(offset), None);
        }
    }

    (input, None, None)
}

/// Parses a UTC offset string like "+05:00" or "-08:00".
pub(super) fn parse_offset(offset_str: &str) -> Option<FixedOffset> {
    let sign = if offset_str.starts_with('-') { -1 } else { 1 };
    let offset_str = offset_str.trim_start_matches(['+', '-']);

    let parts: Vec<&str> = offset_str.split(':').collect();
    let hours: i32 = parts.first()?.parse().ok()?;
    let minutes: i32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

    let total_seconds = sign * (hours * 3600 + minutes * 60);
    FixedOffset::east_opt(total_seconds)
}
