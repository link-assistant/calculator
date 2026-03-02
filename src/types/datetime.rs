//! `DateTime` type for date and time calculations.

use chrono::{
    DateTime as ChronoDateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveDateTime,
    NaiveTime, TimeZone, Utc,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::CalculatorError;

/// A `DateTime` value that can represent dates, times, or both.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateTime {
    /// The underlying datetime (always stored in UTC internally).
    inner: ChronoDateTime<Utc>,
    /// The original timezone offset in seconds for display purposes.
    offset_seconds: Option<i32>,
    /// Whether this includes a time component.
    has_time: bool,
    /// Whether this includes a date component.
    has_date: bool,
    /// Optional label for named time expressions (e.g., "current UTC time").
    /// When set, the display format includes the label and timezone info.
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    /// Optional timezone abbreviation for display (e.g., "UTC", "EST", "GMT").
    /// Used to show the named timezone in the value portion of the display.
    #[serde(skip_serializing_if = "Option::is_none")]
    tz_abbrev: Option<String>,
}

impl PartialEq for DateTime {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for DateTime {}

impl PartialOrd for DateTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DateTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl DateTime {
    /// Creates a `DateTime` representing the current UTC time.
    #[must_use]
    pub fn now() -> Self {
        Self {
            inner: Utc::now(),
            offset_seconds: Some(0),
            has_time: true,
            has_date: true,
            label: None,
            tz_abbrev: None,
        }
    }

    /// Creates a `DateTime` representing the current time with a descriptive label.
    /// The label is shown in the display format, e.g., `('current UTC time': 2026-03-02 20:40:13 UTC (+00:00))`.
    #[must_use]
    pub fn now_with_label(
        label: impl Into<String>,
        offset_seconds: Option<i32>,
        tz_abbrev: Option<String>,
    ) -> Self {
        Self {
            inner: Utc::now(),
            offset_seconds,
            has_time: true,
            has_date: true,
            label: Some(label.into()),
            tz_abbrev,
        }
    }

    /// Returns whether this datetime has a label (i.e., represents a named live time expression).
    #[must_use]
    pub fn is_live_time(&self) -> bool {
        self.label.is_some()
    }

    /// Creates a new `DateTime` from a chrono `DateTime`.
    #[must_use]
    pub fn from_utc(dt: ChronoDateTime<Utc>, has_date: bool, has_time: bool) -> Self {
        Self {
            inner: dt,
            offset_seconds: None,
            has_time,
            has_date,
            label: None,
            tz_abbrev: None,
        }
    }

    /// Creates a new DateTime from a date.
    #[must_use]
    pub fn from_date(date: NaiveDate) -> Self {
        let dt = date.and_hms_opt(0, 0, 0).expect("valid time").and_utc();
        Self {
            inner: dt,
            offset_seconds: None,
            has_time: false,
            has_date: true,
            label: None,
            tz_abbrev: None,
        }
    }

    /// Creates a new DateTime from a time (today's date is used).
    #[must_use]
    pub fn from_time(time: NaiveTime) -> Self {
        let today = Utc::now().date_naive();
        let dt = today.and_time(time).and_utc();
        Self {
            inner: dt,
            offset_seconds: None,
            has_time: true,
            has_date: false,
            label: None,
            tz_abbrev: None,
        }
    }

    /// Gets the fixed offset from the stored seconds.
    fn get_offset(&self) -> Option<FixedOffset> {
        self.offset_seconds.and_then(FixedOffset::east_opt)
    }

    /// Sets the offset from a FixedOffset.
    fn set_offset(&mut self, offset: Option<FixedOffset>) {
        self.offset_seconds = offset.map(|o| o.local_minus_utc());
    }

    /// Parses a datetime from a string, trying multiple formats.
    pub fn parse(input: &str) -> Result<Self, CalculatorError> {
        let input = input.trim();

        // Check for "now" keyword variants
        if let Some(dt) = Self::try_parse_now(input) {
            return Ok(dt);
        }

        // Check for "UTC time" / "time UTC" / "current time" variants
        if let Some(dt) = Self::try_parse_current_time_phrase(input) {
            return Ok(dt);
        }

        // Pre-process: strip day names and ordinal suffixes
        let cleaned = Self::preprocess_natural_date(input);
        let input_to_parse = if cleaned != input { &cleaned } else { input };

        // Try various date formats
        if let Some(dt) = Self::try_parse_date_formats(input_to_parse) {
            return Ok(dt);
        }

        // Try various time formats
        if let Some(dt) = Self::try_parse_time_formats(input_to_parse) {
            return Ok(dt);
        }

        // Try datetime formats
        if let Some(dt) = Self::try_parse_datetime_formats(input_to_parse) {
            return Ok(dt);
        }

        // If preprocessing changed the input, also try with original
        if cleaned != input {
            if let Some(dt) = Self::try_parse_date_formats(input) {
                return Ok(dt);
            }
            if let Some(dt) = Self::try_parse_time_formats(input) {
                return Ok(dt);
            }
            if let Some(dt) = Self::try_parse_datetime_formats(input) {
                return Ok(dt);
            }
        }

        Err(CalculatorError::InvalidDateTime(format!(
            "Could not parse '{input}' as a date or time"
        )))
    }

    /// Checks if input represents "now" (current time).
    fn try_parse_now(input: &str) -> Option<Self> {
        let lower = input.to_lowercase();
        let trimmed = lower.trim();

        // Exact "now" or "now" with timezone
        match trimmed {
            "now" => {
                return Some(Self::now_with_label(
                    "current UTC time",
                    Some(0),
                    Some("UTC".to_string()),
                ))
            }
            "now utc" | "utc now" | "now gmt" | "gmt now" => {
                return Some(Self::now_with_label(
                    "current UTC time",
                    Some(0),
                    Some("UTC".to_string()),
                ))
            }
            _ => {}
        }

        // "now <timezone>" pattern
        if let Some(rest) = trimmed.strip_prefix("now ") {
            let rest = rest.trim();
            if let Some(offset) = Self::parse_tz_abbreviation(rest) {
                let tz_upper = rest.to_uppercase();
                let label = format!("current {tz_upper} time");
                let offset_secs = offset.local_minus_utc();
                return Some(Self::now_with_label(label, Some(offset_secs), Some(tz_upper)));
            }
        }

        // "<timezone> now" pattern
        if let Some(rest) = trimmed.strip_suffix(" now") {
            let rest = rest.trim();
            if let Some(offset) = Self::parse_tz_abbreviation(rest) {
                let tz_upper = rest.to_uppercase();
                let label = format!("current {tz_upper} time");
                let offset_secs = offset.local_minus_utc();
                return Some(Self::now_with_label(label, Some(offset_secs), Some(tz_upper)));
            }
        }

        None
    }

    /// Formats the timezone offset for display, e.g., `UTC (+00:00)` or `EST (-05:00)`.
    /// When `tz_name` is provided, it is prepended to the numeric offset.
    fn format_tz_for_display(offset_seconds: i32, tz_name: Option<&str>) -> String {
        let sign = if offset_seconds >= 0 { "+" } else { "-" };
        let abs_secs = offset_seconds.abs();
        let hours = abs_secs / 3600;
        let minutes = (abs_secs % 3600) / 60;
        let offset_str = format!("({sign}{hours:02}:{minutes:02})");
        if let Some(name) = tz_name {
            format!("{name} {offset_str}")
        } else {
            offset_str
        }
    }

    /// Checks if input represents a "current time" phrase.
    fn try_parse_current_time_phrase(input: &str) -> Option<Self> {
        let lower = input.to_lowercase();
        let trimmed = lower.trim();

        // Phrases that mean "current UTC time" — map phrase to (label, offset_seconds, tz_abbrev)
        let current_time_phrases: &[(&str, &str, Option<i32>, &str)] = &[
            ("utc time", "current UTC time", Some(0), "UTC"),
            ("time utc", "current UTC time", Some(0), "UTC"),
            ("current time", "current UTC time", Some(0), "UTC"),
            ("current time utc", "current UTC time", Some(0), "UTC"),
            ("utc current time", "current UTC time", Some(0), "UTC"),
            ("current utc time", "current UTC time", Some(0), "UTC"),
            ("gmt time", "current GMT time", Some(0), "GMT"),
            ("time gmt", "current GMT time", Some(0), "GMT"),
        ];

        for (phrase, label, offset, tz_abbrev) in current_time_phrases {
            if trimmed == *phrase {
                return Some(Self::now_with_label(
                    *label,
                    *offset,
                    Some((*tz_abbrev).to_string()),
                ));
            }
        }

        // "<timezone> time" and "time <timezone>" patterns for known timezones
        // e.g., "EST time", "time PST"
        let (tz_part, prefix) = if let Some(rest) = trimmed.strip_suffix(" time") {
            (rest.trim(), true)
        } else if let Some(rest) = trimmed.strip_prefix("time ") {
            (rest.trim(), true)
        } else {
            ("", false)
        };

        if prefix && !tz_part.is_empty() {
            if let Some(offset) = Self::parse_tz_abbreviation(tz_part) {
                let tz_upper = tz_part.to_uppercase();
                let label = format!("current {tz_upper} time");
                let offset_secs = offset.local_minus_utc();
                return Some(Self::now_with_label(
                    label,
                    Some(offset_secs),
                    Some(tz_upper),
                ));
            }
        }

        None
    }

    /// Pre-processes natural date strings by removing day names, ordinal suffixes,
    /// and the "on" preposition.
    fn preprocess_natural_date(input: &str) -> String {
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
    fn parse_tz_abbreviation(tz: &str) -> Option<FixedOffset> {
        let offset_hours = match tz.to_uppercase().as_str() {
            "UTC" | "GMT" | "Z" => 0,
            // US timezones
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
            // European timezones
            "CET" => 1,
            "CEST" => 2,
            "EET" => 2,
            "EEST" => 3,
            "WET" => 0,
            "WEST" => 1,
            "GMT+1" | "BST" => 1,
            // Asian timezones
            "IST" => 5, // India Standard Time (5:30 handled separately)
            "JST" => 9,
            "KST" => 9,
            "CST+8" | "SGT" | "HKT" | "PHT" => 8,
            // Australian timezones
            "AEST" => 10,
            "AEDT" => 11,
            "ACST" => 9, // Actually +9:30
            "AWST" => 8,
            _ => return None,
        };
        FixedOffset::east_opt(offset_hours * 3600)
    }

    fn try_parse_date_formats(input: &str) -> Option<Self> {
        // ISO format: 2026-01-22
        if let Ok(date) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
            return Some(Self::from_date(date));
        }

        // US format: 01/22/2026 or 1/22/2026
        if let Ok(date) = NaiveDate::parse_from_str(input, "%m/%d/%Y") {
            return Some(Self::from_date(date));
        }

        // European format: 22/01/2026
        if let Ok(date) = NaiveDate::parse_from_str(input, "%d/%m/%Y") {
            return Some(Self::from_date(date));
        }

        // Month name formats: Jan 22, 2026 or January 22, 2026
        let normalized = Self::normalize_month_name(input);
        if let Ok(date) = NaiveDate::parse_from_str(&normalized, "%b %d, %Y") {
            return Some(Self::from_date(date));
        }
        if let Ok(date) = NaiveDate::parse_from_str(&normalized, "%B %d, %Y") {
            return Some(Self::from_date(date));
        }

        // 22 Jan 2026 format
        if let Ok(date) = NaiveDate::parse_from_str(&normalized, "%d %b %Y") {
            return Some(Self::from_date(date));
        }
        if let Ok(date) = NaiveDate::parse_from_str(&normalized, "%d %B %Y") {
            return Some(Self::from_date(date));
        }

        None
    }

    fn try_parse_time_formats(input: &str) -> Option<Self> {
        let input = input.trim();

        // Parse time with optional timezone
        let (time_part, tz_offset) = Self::extract_timezone(input);

        // 12-hour format: 8:59am, 12:51pm, 8:59 am, 8:59AM
        if let Some(time) = Self::parse_12h_time(time_part) {
            let mut dt = Self::from_time(time);
            if let Some(offset) = tz_offset {
                dt.set_offset(Some(offset));
                // Adjust internal time to UTC
                let local = dt.inner.naive_utc();
                let adjusted = offset.from_local_datetime(&local).single();
                if let Some(adj) = adjusted {
                    dt.inner = adj.with_timezone(&Utc);
                }
            }
            return Some(dt);
        }

        // 24-hour format: 14:30, 14:30:00
        if let Ok(time) = NaiveTime::parse_from_str(time_part, "%H:%M") {
            let mut dt = Self::from_time(time);
            dt.set_offset(tz_offset);
            return Some(dt);
        }
        if let Ok(time) = NaiveTime::parse_from_str(time_part, "%H:%M:%S") {
            let mut dt = Self::from_time(time);
            dt.set_offset(tz_offset);
            return Some(dt);
        }

        None
    }

    fn parse_12h_time(input: &str) -> Option<NaiveTime> {
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

    fn extract_timezone(input: &str) -> (&str, Option<FixedOffset>) {
        let input = input.trim();

        // Check for UTC/GMT suffix
        if input.to_uppercase().ends_with("UTC") || input.to_uppercase().ends_with("GMT") {
            let time_part = input[..input.len() - 3].trim();
            return (time_part, Some(FixedOffset::east_opt(0).unwrap()));
        }

        // Check for common timezone abbreviations as suffix
        // Try to match the last word as a timezone abbreviation
        if let Some(last_space) = input.rfind(' ') {
            let potential_tz = &input[last_space + 1..];
            if let Some(offset) = Self::parse_tz_abbreviation(potential_tz) {
                let time_part = input[..last_space].trim();
                return (time_part, Some(offset));
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
            if let Some(offset) = Self::parse_offset(offset_str) {
                return (&input[..idx], Some(offset));
            }
        }

        (input, None)
    }

    fn parse_offset(offset_str: &str) -> Option<FixedOffset> {
        let sign = if offset_str.starts_with('-') { -1 } else { 1 };
        let offset_str = offset_str.trim_start_matches(['+', '-']);

        let parts: Vec<&str> = offset_str.split(':').collect();
        let hours: i32 = parts.first()?.parse().ok()?;
        let minutes: i32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

        let total_seconds = sign * (hours * 3600 + minutes * 60);
        FixedOffset::east_opt(total_seconds)
    }

    fn try_parse_datetime_formats(input: &str) -> Option<Self> {
        // Extract date and time parts
        // Format: "Jan 27, 8:59am UTC" -> date="Jan 27", time="8:59am UTC"
        // We need to handle various separators

        // Try comma separation: "Jan 27, 8:59am UTC"
        if let Some((date_part, time_part)) = input.split_once(',') {
            let date_part = date_part.trim();
            let time_part = time_part.trim();

            // Try to parse partial date (no year - assume current year)
            if let Some(date) = Self::parse_partial_date(date_part) {
                if let Some(time_dt) = Self::try_parse_time_formats(time_part) {
                    let datetime = date.and_time(time_dt.inner.time()).and_utc();
                    return Some(Self {
                        inner: datetime,
                        offset_seconds: time_dt.offset_seconds,
                        has_time: true,
                        has_date: true,
                        label: None,
                        tz_abbrev: None,
                    });
                }
            }
        }

        // Try "time <date>" pattern: "11:59pm EST January 26"
        // Look for a time-like pattern at the start followed by a date
        if let Some(dt) = Self::try_parse_time_then_date(input) {
            return Some(dt);
        }

        // ISO 8601 format
        if let Ok(dt) = ChronoDateTime::parse_from_rfc3339(input) {
            return Some(Self {
                inner: dt.with_timezone(&Utc),
                offset_seconds: Some(dt.timezone().local_minus_utc()),
                has_time: true,
                has_date: true,
                label: None,
                tz_abbrev: None,
            });
        }

        // Common datetime formats
        if let Ok(dt) = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S") {
            return Some(Self {
                inner: dt.and_utc(),
                offset_seconds: None,
                has_time: true,
                has_date: true,
                label: None,
                tz_abbrev: None,
            });
        }

        None
    }

    /// Try to parse "time date" patterns like "11:59pm EST January 26"
    fn try_parse_time_then_date(input: &str) -> Option<Self> {
        let input = input.trim();

        // Look for am/pm pattern or HH:MM pattern at the start
        let words: Vec<&str> = input.split_whitespace().collect();
        if words.len() < 2 {
            return None;
        }

        // Try progressively longer prefixes as the time part
        for split_at in 1..words.len() {
            let time_candidate = words[..split_at].join(" ");
            let date_candidate = words[split_at..].join(" ");

            if let Some(time_dt) = Self::try_parse_time_formats(&time_candidate) {
                // Try the remaining as a date (partial or full)
                if let Some(date) = Self::parse_partial_date(&date_candidate) {
                    let datetime = date.and_time(time_dt.inner.time()).and_utc();
                    let mut result = Self {
                        inner: datetime,
                        offset_seconds: time_dt.offset_seconds,
                        has_time: true,
                        has_date: true,
                        label: None,
                        tz_abbrev: None,
                    };
                    // If time had a timezone, adjust the UTC time accordingly
                    if let Some(offset) = time_dt.offset_seconds.and_then(FixedOffset::east_opt) {
                        let local = result.inner.naive_utc();
                        if let Some(adj) = offset.from_local_datetime(&local).single() {
                            result.inner = adj.with_timezone(&Utc);
                        }
                    }
                    return Some(result);
                }
                if let Some(date_dt) = Self::try_parse_date_formats(&date_candidate) {
                    let date = date_dt.inner.date_naive();
                    let datetime = date.and_time(time_dt.inner.time()).and_utc();
                    let mut result = Self {
                        inner: datetime,
                        offset_seconds: time_dt.offset_seconds,
                        has_time: true,
                        has_date: true,
                        label: None,
                        tz_abbrev: None,
                    };
                    if let Some(offset) = time_dt.offset_seconds.and_then(FixedOffset::east_opt) {
                        let local = result.inner.naive_utc();
                        if let Some(adj) = offset.from_local_datetime(&local).single() {
                            result.inner = adj.with_timezone(&Utc);
                        }
                    }
                    return Some(result);
                }
            }
        }

        None
    }

    fn parse_partial_date(input: &str) -> Option<NaiveDate> {
        let normalized = Self::normalize_month_name(input);
        let current_year = Utc::now().year();

        // "Jan 27" or "January 27"
        if let Ok(md) =
            NaiveDate::parse_from_str(&format!("{normalized} {current_year}"), "%b %d %Y")
        {
            return Some(md);
        }
        if let Ok(md) =
            NaiveDate::parse_from_str(&format!("{normalized} {current_year}"), "%B %d %Y")
        {
            return Some(md);
        }

        // "27 Jan" format
        if let Ok(md) =
            NaiveDate::parse_from_str(&format!("{normalized} {current_year}"), "%d %b %Y")
        {
            return Some(md);
        }

        None
    }

    fn normalize_month_name(input: &str) -> String {
        // Capitalize first letter of each word for month parsing
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

    /// Returns the underlying chrono DateTime.
    #[must_use]
    pub fn as_chrono(&self) -> &ChronoDateTime<Utc> {
        &self.inner
    }

    /// Subtracts another DateTime, returning a Duration (absolute value).
    pub fn subtract(&self, other: &Self) -> std::time::Duration {
        let diff = self.inner.signed_duration_since(other.inner);
        diff.to_std().unwrap_or_default()
    }

    /// Subtracts another DateTime, returning signed seconds (positive if self > other).
    #[must_use]
    pub fn signed_subtract_seconds(&self, other: &Self) -> i64 {
        self.inner.signed_duration_since(other.inner).num_seconds()
    }

    /// Returns the inner chrono DateTime (for comparisons, etc.).
    #[must_use]
    pub fn inner_utc(&self) -> ChronoDateTime<Utc> {
        self.inner
    }

    /// Adds a duration to this DateTime.
    #[must_use]
    pub fn add_duration(&self, seconds: i64) -> Self {
        let duration = Duration::seconds(seconds);
        Self {
            inner: self.inner + duration,
            offset_seconds: self.offset_seconds,
            has_time: self.has_time,
            has_date: self.has_date,
            label: None,
            tz_abbrev: None,
        }
    }

    /// Returns the year.
    #[must_use]
    pub fn year(&self) -> i32 {
        use chrono::Datelike;
        self.inner.year()
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // When a label is present (e.g., "current UTC time"), use the enhanced format:
        // ('current UTC time': 2026-03-02 20:40:13 UTC (+00:00))
        if let Some(ref label) = self.label {
            let offset_secs = self.offset_seconds.unwrap_or(0);
            let tz_display =
                Self::format_tz_for_display(offset_secs, self.tz_abbrev.as_deref());
            if let Some(offset) = self.get_offset() {
                let local = self.inner.with_timezone(&offset);
                return write!(
                    f,
                    "('{label}': {} {tz_display})",
                    local.format("%Y-%m-%d %H:%M:%S")
                );
            }
            return write!(
                f,
                "('{label}': {} {tz_display})",
                self.inner.format("%Y-%m-%d %H:%M:%S")
            );
        }

        if self.has_date && self.has_time {
            if let Some(offset) = self.get_offset() {
                let local = self.inner.with_timezone(&offset);
                write!(f, "{}", local.format("%Y-%m-%d %H:%M:%S %:z"))
            } else {
                write!(f, "{}", self.inner.format("%Y-%m-%d %H:%M:%S UTC"))
            }
        } else if self.has_date {
            write!(f, "{}", self.inner.format("%Y-%m-%d"))
        } else if self.has_time {
            if let Some(offset) = self.get_offset() {
                let local = self.inner.with_timezone(&offset);
                write!(f, "{}", local.format("%H:%M:%S %:z"))
            } else {
                write!(f, "{}", self.inner.format("%H:%M:%S UTC"))
            }
        } else {
            write!(f, "{}", self.inner)
        }
    }
}

impl FromStr for DateTime {
    type Err = CalculatorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_iso_date() {
        let dt = DateTime::parse("2026-01-22").unwrap();
        assert!(dt.has_date);
        assert!(!dt.has_time);
    }

    #[test]
    fn test_parse_us_date() {
        let dt = DateTime::parse("01/22/2026").unwrap();
        assert!(dt.has_date);
    }

    #[test]
    fn test_parse_month_name_date() {
        let dt = DateTime::parse("Jan 22, 2026").unwrap();
        assert!(dt.has_date);
        assert_eq!(dt.year(), 2026);
    }

    #[test]
    fn test_parse_time_12h() {
        let dt = DateTime::parse("8:59am").unwrap();
        assert!(dt.has_time);
    }

    #[test]
    fn test_parse_time_with_utc() {
        let dt = DateTime::parse("8:59am UTC").unwrap();
        assert!(dt.has_time);
        assert!(dt.offset_seconds.is_some());
    }

    #[test]
    fn test_parse_datetime_with_partial_date() {
        let dt = DateTime::parse("Jan 27, 8:59am UTC").unwrap();
        assert!(dt.has_date);
        assert!(dt.has_time);
    }

    #[test]
    fn test_datetime_subtraction() {
        let dt1 = DateTime::parse("Jan 27, 8:59am UTC").unwrap();
        let dt2 = DateTime::parse("Jan 25, 12:51pm UTC").unwrap();
        let diff = dt1.subtract(&dt2);
        // Should be approximately 44 hours and 8 minutes
        let hours = diff.as_secs() / 3600;
        assert!(hours > 40 && hours < 50);
    }

    #[test]
    fn test_22_jan_2026_format() {
        let dt = DateTime::parse("22 Jan 2026").unwrap();
        assert!(dt.has_date);
        assert_eq!(dt.year(), 2026);
    }
}
