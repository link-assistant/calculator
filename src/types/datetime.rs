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
    /// Creates a new `DateTime` from a chrono `DateTime`.
    #[must_use]
    pub const fn from_utc(dt: ChronoDateTime<Utc>, has_date: bool, has_time: bool) -> Self {
        Self {
            inner: dt,
            offset_seconds: None,
            has_time,
            has_date,
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

        // Try various date formats
        if let Some(dt) = Self::try_parse_date_formats(input) {
            return Ok(dt);
        }

        // Try various time formats
        if let Some(dt) = Self::try_parse_time_formats(input) {
            return Ok(dt);
        }

        // Try datetime formats
        if let Some(dt) = Self::try_parse_datetime_formats(input) {
            return Ok(dt);
        }

        Err(CalculatorError::InvalidDateTime(format!(
            "Could not parse '{input}' as a date or time"
        )))
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
                    });
                }
            }
        }

        // ISO 8601 format
        if let Ok(dt) = ChronoDateTime::parse_from_rfc3339(input) {
            return Some(Self {
                inner: dt.with_timezone(&Utc),
                offset_seconds: Some(dt.timezone().local_minus_utc()),
                has_time: true,
                has_date: true,
            });
        }

        // Common datetime formats
        if let Ok(dt) = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S") {
            return Some(Self {
                inner: dt.and_utc(),
                offset_seconds: None,
                has_time: true,
                has_date: true,
            });
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

    /// Subtracts another DateTime, returning a Duration.
    pub fn subtract(&self, other: &Self) -> std::time::Duration {
        let diff = self.inner.signed_duration_since(other.inner);
        diff.to_std().unwrap_or_default()
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
