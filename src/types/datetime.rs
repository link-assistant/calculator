//! `DateTime` type for date and time calculations.

use chrono::{
    DateTime as ChronoDateTime, Duration, FixedOffset, Months, NaiveDate, NaiveDateTime, NaiveTime,
    TimeZone, Utc,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::CalculatorError;

#[path = "datetime_parse.rs"]
mod parse;
use parse::{
    extract_timezone, normalize_month_name, parse_12h_time, parse_partial_date,
    parse_tz_abbreviation, preprocess_natural_date, translate_month_names,
};

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

/// Browser-friendly metadata for displaying timezone conversions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateTimeResult {
    /// The calculated value in its source timezone.
    pub source: String,
    /// The same instant formatted in UTC.
    pub utc: String,
    /// UTC timestamp in milliseconds since the Unix epoch.
    pub epoch_milliseconds: i64,
    /// Whether the original expression includes a date component.
    pub has_date: bool,
    /// Whether the original expression includes a time component.
    pub has_time: bool,
    /// Source timezone abbreviation, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// Source timezone offset in seconds, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_seconds: Option<i32>,
}

impl DateTimeResult {
    /// Creates conversion display metadata for a timezone-aware datetime value.
    #[must_use]
    pub fn from_datetime(dt: &DateTime) -> Option<Self> {
        Some(Self {
            source: dt.to_string(),
            utc: dt.utc_equivalent_display()?,
            epoch_milliseconds: dt.timestamp_millis(),
            has_date: dt.has_date(),
            has_time: dt.has_time(),
            timezone: dt.timezone_abbreviation().map(ToString::to_string),
            offset_seconds: dt.offset_seconds(),
        })
    }
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

    /// Creates a `DateTime` for the current instant, displayed in the user's
    /// local timezone given by `offset_seconds` (seconds east of UTC).
    ///
    /// The internal instant is still the true current UTC time; only the display
    /// offset and label differ from [`Self::now`]. Used when the calculator is
    /// told the user's local timezone so that `now` reflects their wall clock
    /// instead of UTC.
    #[must_use]
    pub fn now_local(offset_seconds: i32) -> Self {
        Self {
            inner: Utc::now(),
            offset_seconds: Some(offset_seconds),
            has_time: true,
            has_date: true,
            label: Some("current local time".to_string()),
            tz_abbrev: None,
        }
    }

    /// Creates a date-only value for today's calendar date in the timezone
    /// represented by `offset_seconds` (seconds east of UTC).
    #[must_use]
    pub fn today(offset_seconds: i32) -> Self {
        let local_now = Utc::now() + Duration::seconds(i64::from(offset_seconds));
        Self::from_date(local_now.date_naive())
    }

    /// Re-anchors a timezone-less ("naive") time or datetime to a local timezone.
    ///
    /// Bare times like `12:30` are parsed with their wall-clock reading stored as
    /// if it were UTC. When the user's local timezone is known, the same
    /// wall-clock reading should instead be interpreted as local time. This shifts
    /// the internal UTC instant back by `offset_seconds` so the displayed wall
    /// clock is preserved while the underlying instant becomes correct for the
    /// local timezone.
    ///
    /// Values that already carry an explicit timezone (e.g. `12:30 UTC`) or that
    /// have no time component (plain dates) are returned unchanged.
    #[must_use]
    pub fn reinterpret_naive_as_local(&self, offset_seconds: i32) -> Self {
        if self.offset_seconds.is_some() || !self.has_time {
            return self.clone();
        }
        let adjusted = self.inner - Duration::seconds(i64::from(offset_seconds));
        Self {
            inner: adjusted,
            offset_seconds: Some(offset_seconds),
            has_time: self.has_time,
            has_date: self.has_date,
            label: self.label.clone(),
            tz_abbrev: None,
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

    /// Creates a date-only value for January 1 of a calendar year.
    pub(crate) fn from_year(year: i32) -> Option<Self> {
        NaiveDate::from_ymd_opt(year, 1, 1).map(Self::from_date)
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

    /// Returns a new `DateTime` displaying the same instant in a different timezone.
    ///
    /// The internal UTC time does not change — only the display offset and abbreviation.
    #[must_use]
    pub fn with_timezone_offset(&self, offset: FixedOffset, tz_abbrev: &str) -> Self {
        Self {
            inner: self.inner,
            offset_seconds: Some(offset.local_minus_utc()),
            has_time: self.has_time,
            has_date: self.has_date,
            label: None,
            tz_abbrev: Some(tz_abbrev.to_uppercase()),
        }
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

        // Pre-process: translate non-English month names to English (all supported UI languages)
        let translated = translate_month_names(input);
        let input = if translated != input {
            translated.as_str()
        } else {
            input
        };

        // Pre-process: strip day names and ordinal suffixes
        let cleaned = preprocess_natural_date(input);
        let input_to_parse = if cleaned != input { &cleaned } else { input };

        // Try various date formats
        if let Some(dt) = Self::try_parse_date_formats(input_to_parse) {
            return Ok(dt);
        }
        if let Some(date) = parse_partial_date(input_to_parse) {
            return Ok(Self::from_date(date));
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
            if let Some(date) = parse_partial_date(input) {
                return Ok(Self::from_date(date));
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
            if let Some(offset) = parse_tz_abbreviation(rest) {
                let tz_upper = rest.to_uppercase();
                let label = format!("current {tz_upper} time");
                let offset_secs = offset.local_minus_utc();
                return Some(Self::now_with_label(
                    label,
                    Some(offset_secs),
                    Some(tz_upper),
                ));
            }
        }

        // "<timezone> now" pattern
        if let Some(rest) = trimmed.strip_suffix(" now") {
            let rest = rest.trim();
            if let Some(offset) = parse_tz_abbreviation(rest) {
                let tz_upper = rest.to_uppercase();
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
            ("current utc time", "current UTC time", Some(0), "UTC"),
            ("gmt time", "current GMT time", Some(0), "GMT"),
            ("time gmt", "current GMT time", Some(0), "GMT"),
        ];

        for &(phrase, label, offset_secs, tz_abbrev) in current_time_phrases {
            if trimmed == phrase {
                return Some(Self::now_with_label(
                    label,
                    offset_secs,
                    Some(tz_abbrev.to_string()),
                ));
            }
        }

        // "current <TZ> time" or "<TZ> time" patterns
        for prefix in &["current ", ""] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                if let Some(tz_part) = rest.strip_suffix(" time") {
                    let tz_part = tz_part.trim();
                    if let Some(offset) = parse_tz_abbreviation(tz_part) {
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
                if let Some(tz_part) = rest.strip_prefix("time ") {
                    let tz_part = tz_part.trim();
                    if let Some(offset) = parse_tz_abbreviation(tz_part) {
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
            }
        }

        None
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

        // Dot-separated European/German format: 15.10.2025 (DD.MM.YYYY)
        // Dots are the conventional date separator in German, Russian and many
        // other locales, so day-first is tried before month-first.
        if let Ok(date) = NaiveDate::parse_from_str(input, "%d.%m.%Y") {
            return Some(Self::from_date(date));
        }

        // Dot-separated US-style fallback: 10.15.2025 (MM.DD.YYYY)
        if let Ok(date) = NaiveDate::parse_from_str(input, "%m.%d.%Y") {
            return Some(Self::from_date(date));
        }

        // Dot-separated ISO-like format: 2025.10.15 (YYYY.MM.DD)
        if let Ok(date) = NaiveDate::parse_from_str(input, "%Y.%m.%d") {
            return Some(Self::from_date(date));
        }

        // Dash-separated, non-ISO orderings: 15-10-2025 (DD-MM-YYYY) and the
        // US-style 10-15-2025 (MM-DD-YYYY). ISO `%Y-%m-%d` is tried first above,
        // so these only match when the year is in the trailing position. Day-first
        // is preferred, mirroring the dot-separated convention.
        if let Ok(date) = NaiveDate::parse_from_str(input, "%d-%m-%Y") {
            return Some(Self::from_date(date));
        }
        if let Ok(date) = NaiveDate::parse_from_str(input, "%m-%d-%Y") {
            return Some(Self::from_date(date));
        }

        // Month name formats: Jan 22, 2026 or January 22, 2026
        let normalized = normalize_month_name(input);
        if let Ok(date) = NaiveDate::parse_from_str(&normalized, "%b %d, %Y") {
            return Some(Self::from_date(date));
        }
        if let Ok(date) = NaiveDate::parse_from_str(&normalized, "%B %d, %Y") {
            return Some(Self::from_date(date));
        }

        // 22 Jan 2026 format (without comma)
        if let Ok(date) = NaiveDate::parse_from_str(&normalized, "%d %b %Y") {
            return Some(Self::from_date(date));
        }
        if let Ok(date) = NaiveDate::parse_from_str(&normalized, "%d %B %Y") {
            return Some(Self::from_date(date));
        }

        // Jan 22 2026 format (without comma) — e.g., "Feb 17 2027"
        if let Ok(date) = NaiveDate::parse_from_str(&normalized, "%b %d %Y") {
            return Some(Self::from_date(date));
        }
        if let Ok(date) = NaiveDate::parse_from_str(&normalized, "%B %d %Y") {
            return Some(Self::from_date(date));
        }

        None
    }

    fn try_parse_time_formats(input: &str) -> Option<Self> {
        let input = input.trim();

        // Parse time with optional timezone
        let (time_part, tz_offset, tz_abbrev) = extract_timezone(input);

        // 12-hour format: 8:59am, 12:51pm, 8:59 am, 8:59AM
        if let Some(time) = parse_12h_time(time_part) {
            let mut dt = Self::from_time(time);
            if let Some(offset) = tz_offset {
                dt.set_offset(Some(offset));
                dt.tz_abbrev.clone_from(&tz_abbrev);
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
            dt.tz_abbrev.clone_from(&tz_abbrev);
            if let Some(offset) = tz_offset {
                let local = dt.inner.naive_utc();
                if let Some(adj) = offset.from_local_datetime(&local).single() {
                    dt.inner = adj.with_timezone(&Utc);
                }
            }
            return Some(dt);
        }
        if let Ok(time) = NaiveTime::parse_from_str(time_part, "%H:%M:%S") {
            let mut dt = Self::from_time(time);
            dt.set_offset(tz_offset);
            dt.tz_abbrev = tz_abbrev;
            if let Some(offset) = tz_offset {
                let local = dt.inner.naive_utc();
                if let Some(adj) = offset.from_local_datetime(&local).single() {
                    dt.inner = adj.with_timezone(&Utc);
                }
            }
            return Some(dt);
        }

        None
    }

    fn try_parse_datetime_formats(input: &str) -> Option<Self> {
        // Try comma separation: "Jan 27, 8:59am UTC"
        if let Some((date_part, time_part)) = input.split_once(',') {
            let date_part = date_part.trim();
            let time_part = time_part.trim();

            // Try to parse partial date (no year - assume current year)
            if let Some(date) = parse_partial_date(date_part) {
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
                if let Some(date) = parse_partial_date(&date_candidate) {
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

    /// Returns the UTC timestamp in milliseconds since the Unix epoch.
    #[must_use]
    pub fn timestamp_millis(&self) -> i64 {
        self.inner.timestamp_millis()
    }

    /// Returns whether this value has a date component.
    #[must_use]
    pub fn has_date(&self) -> bool {
        self.has_date
    }

    /// Returns whether this value has a time component.
    #[must_use]
    pub fn has_time(&self) -> bool {
        self.has_time
    }

    /// Returns the source timezone offset in seconds, if known.
    #[must_use]
    pub fn offset_seconds(&self) -> Option<i32> {
        self.offset_seconds
    }

    /// Returns the source timezone abbreviation, if available.
    #[must_use]
    pub fn timezone_abbreviation(&self) -> Option<&str> {
        self.tz_abbrev.as_deref()
    }

    /// Returns true when this value has enough timezone context to show conversions.
    #[must_use]
    pub fn should_show_timezone_conversions(&self) -> bool {
        self.has_time && self.offset_seconds.is_some()
    }

    /// Formats the same instant in UTC for conversion summaries.
    #[must_use]
    pub fn utc_equivalent_display(&self) -> Option<String> {
        if !self.should_show_timezone_conversions() {
            return None;
        }

        let formatted = if self.has_date && self.has_time {
            self.inner.format("%Y-%m-%d %H:%M:%S UTC").to_string()
        } else if self.has_date {
            self.inner.format("%Y-%m-%d").to_string()
        } else if self.has_time {
            self.inner.format("%H:%M:%S UTC").to_string()
        } else {
            self.inner.to_rfc3339()
        };

        Some(formatted)
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

    /// Adds (or subtracts when negative) a number of calendar months to this DateTime.
    ///
    /// Unlike `add_duration`, this performs true calendar arithmetic: the day-of-month
    /// is preserved whenever possible, and clamped to the last day of the target month
    /// when the original day does not exist (e.g. 31 Jan + 1 month → 28/29 Feb).
    #[must_use]
    pub fn add_calendar_months(&self, months: i32) -> Self {
        let naive = self.inner.naive_utc();
        #[allow(clippy::cast_sign_loss)] // sign is checked by the if/else branches
        let new_naive = if months >= 0 {
            let m = Months::new(months as u32);
            naive.checked_add_months(m).unwrap_or(naive)
        } else {
            let m = Months::new((-months) as u32);
            naive.checked_sub_months(m).unwrap_or(naive)
        };
        let new_inner = new_naive.and_utc();
        Self {
            inner: new_inner,
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

    /// Parses common timezone abbreviations to `FixedOffset`.
    ///
    /// Returns `None` if the abbreviation is not recognized.
    pub(crate) fn parse_tz_abbreviation(tz: &str) -> Option<FixedOffset> {
        parse_tz_abbreviation(tz)
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // When a label is present (e.g., "current UTC time"), use the enhanced format:
        // ('current UTC time': 2026-03-02 20:40:13 UTC (+00:00))
        if let Some(ref label) = self.label {
            let offset_secs = self.offset_seconds.unwrap_or(0);
            let tz_display = Self::format_tz_for_display(offset_secs, self.tz_abbrev.as_deref());
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
                if let Some(ref tz) = self.tz_abbrev {
                    write!(f, "{} {tz}", local.format("%Y-%m-%d %H:%M:%S"))
                } else {
                    write!(f, "{}", local.format("%Y-%m-%d %H:%M:%S %:z"))
                }
            } else {
                write!(f, "{}", self.inner.format("%Y-%m-%d %H:%M:%S UTC"))
            }
        } else if self.has_date {
            write!(f, "{}", self.inner.format("%Y-%m-%d"))
        } else if self.has_time {
            if let Some(offset) = self.get_offset() {
                let local = self.inner.with_timezone(&offset);
                if let Some(ref tz) = self.tz_abbrev {
                    write!(f, "{} {tz}", local.format("%H:%M:%S"))
                } else {
                    write!(f, "{}", local.format("%H:%M:%S %:z"))
                }
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
    use chrono::Timelike;

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
    fn test_parse_dot_date_european() {
        // DD.MM.YYYY (German/Russian convention) — issue #166.
        let dt = DateTime::parse("15.10.2025").unwrap();
        assert!(dt.has_date);
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.to_string(), "2025-10-15");
    }

    #[test]
    fn test_parse_dot_date_iso() {
        // YYYY.MM.DD
        let dt = DateTime::parse("2025.10.15").unwrap();
        assert_eq!(dt.to_string(), "2025-10-15");
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
    fn test_today_uses_requested_timezone_date() {
        let now = Utc::now();
        let (offset_seconds, expected_date) = if now.time().hour() < 12 {
            (-12 * 60 * 60, (now - Duration::hours(12)).date_naive())
        } else {
            (14 * 60 * 60, (now + Duration::hours(14)).date_naive())
        };

        let today = DateTime::today(offset_seconds);
        assert_eq!(today.inner.date_naive(), expected_date);
        assert!(today.has_date());
        assert!(!today.has_time());
    }

    #[test]
    fn test_22_jan_2026_format() {
        let dt = DateTime::parse("22 Jan 2026").unwrap();
        assert!(dt.has_date);
        assert_eq!(dt.year(), 2026);
    }
}
