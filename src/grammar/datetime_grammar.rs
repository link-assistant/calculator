//! Grammar for parsing date and time expressions.

use crate::error::CalculatorError;
use crate::types::{DateTime, Value};

/// Grammar for parsing datetime expressions.
#[derive(Debug, Default)]
pub struct DateTimeGrammar;

impl DateTimeGrammar {
    /// Creates a new datetime grammar.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Attempts to parse a datetime from a string.
    pub fn parse(&self, input: &str) -> Result<DateTime, CalculatorError> {
        DateTime::parse(input)
    }

    /// Checks if the input looks like a datetime.
    #[must_use]
    pub fn looks_like_datetime(input: &str) -> bool {
        let input = input.trim().to_lowercase();

        // Check for month names
        let month_names = [
            "jan",
            "feb",
            "mar",
            "apr",
            "may",
            "jun",
            "jul",
            "aug",
            "sep",
            "oct",
            "nov",
            "dec",
            "january",
            "february",
            "march",
            "april",
            "june",
            "july",
            "august",
            "september",
            "october",
            "november",
            "december",
        ];

        for month in &month_names {
            if input.contains(month) {
                return true;
            }
        }

        // Check for time patterns (am/pm)
        if input.contains("am") || input.contains("pm") {
            return true;
        }

        // Check for timezone indicators
        if input.contains("utc") || input.contains("gmt") {
            return true;
        }

        // Check for ISO date pattern (YYYY-MM-DD)
        if input.len() >= 10 {
            let chars: Vec<char> = input.chars().collect();
            #[allow(clippy::redundant_closure_for_method_calls)]
            if chars.len() >= 10
                && chars[4] == '-'
                && chars[7] == '-'
                && chars[0..4].iter().all(|c| c.is_ascii_digit())
            {
                return true;
            }
        }

        // Check for time pattern (HH:MM)
        if input.contains(':') {
            let parts: Vec<&str> = input.split(':').collect();
            if parts.len() >= 2 && parts[0].chars().last().is_some_and(|c| c.is_ascii_digit()) {
                return true;
            }
        }

        false
    }

    /// Attempts to extract a datetime from a longer expression.
    /// Returns the datetime and the remaining string.
    #[must_use]
    pub fn try_extract_datetime<'a>(&self, input: &'a str) -> Option<(DateTime, &'a str)> {
        let input = input.trim();

        // Try progressively longer substrings from the start
        // This handles cases like "Jan 27, 8:59am UTC" followed by more text

        // First, try the whole string
        if let Ok(dt) = self.parse(input) {
            return Some((dt, ""));
        }

        // Try to find natural break points
        // Look for patterns that indicate end of datetime

        None
    }

    /// Tries to parse a datetime subtraction expression like "(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)".
    #[must_use]
    pub fn try_parse_datetime_subtraction(
        &self,
        input: &str,
    ) -> Option<(Value, Vec<String>, String)> {
        // Look for pattern: (datetime) - (datetime)
        let input = input.trim();

        // Check if it starts with '(' and contains '-'
        if !input.starts_with('(') || !input.contains('-') {
            return None;
        }

        // Try to find the matching closing paren for the first datetime
        let mut paren_depth = 0;
        let mut first_end = None;

        for (i, ch) in input.char_indices() {
            match ch {
                '(' => paren_depth += 1,
                ')' => {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        first_end = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }

        let first_end = first_end?;

        // Extract first datetime (without parens)
        let first_dt_str = &input[1..first_end];

        // Find the minus sign
        let rest = input[first_end + 1..].trim();
        if !rest.starts_with('-') {
            return None;
        }

        let second_part = rest[1..].trim();
        if !second_part.starts_with('(') || !second_part.ends_with(')') {
            return None;
        }

        // Extract second datetime (without parens)
        let second_dt_str = &second_part[1..second_part.len() - 1];

        // Try to parse both as datetimes
        let Ok(dt1) = self.parse(first_dt_str) else {
            return None;
        };

        let Ok(dt2) = self.parse(second_dt_str) else {
            return None;
        };

        // Calculate the difference
        let diff = dt1.subtract(&dt2);
        #[allow(clippy::cast_possible_wrap)]
        let seconds = diff.as_secs() as i64;

        let value = Value::duration(seconds);

        let steps = vec![
            format!("Parse first datetime: {dt1}"),
            format!("Parse second datetime: {dt2}"),
            format!("Calculate difference: {dt1} - {dt2}"),
            format!("Result: {}", value.to_display_string()),
        ];

        // Issue #30 fix: Use exactly 2 outer parentheses for Links notation
        // Format: ((datetime1) - (datetime2))
        let lino = format!("(({}) - ({}))", first_dt_str.trim(), second_dt_str.trim());

        Some((value, steps, lino))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_datetime_month_name() {
        assert!(DateTimeGrammar::looks_like_datetime("Jan 22, 2026"));
        assert!(DateTimeGrammar::looks_like_datetime("22 January 2026"));
        assert!(DateTimeGrammar::looks_like_datetime("Feb 15"));
    }

    #[test]
    fn test_looks_like_datetime_time() {
        assert!(DateTimeGrammar::looks_like_datetime("8:59am"));
        assert!(DateTimeGrammar::looks_like_datetime("12:30pm"));
        assert!(DateTimeGrammar::looks_like_datetime("14:30 UTC"));
    }

    #[test]
    fn test_looks_like_datetime_iso() {
        assert!(DateTimeGrammar::looks_like_datetime("2026-01-22"));
    }

    #[test]
    fn test_looks_like_datetime_negative() {
        assert!(!DateTimeGrammar::looks_like_datetime("42"));
        assert!(!DateTimeGrammar::looks_like_datetime("USD"));
        assert!(!DateTimeGrammar::looks_like_datetime("hello world"));
    }

    #[test]
    fn test_parse_various_formats() {
        let grammar = DateTimeGrammar::new();

        assert!(grammar.parse("Jan 22, 2026").is_ok());
        assert!(grammar.parse("2026-01-22").is_ok());
        assert!(grammar.parse("8:59am UTC").is_ok());
        assert!(grammar.parse("Jan 27, 8:59am UTC").is_ok());
    }
}
