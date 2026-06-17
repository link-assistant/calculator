//! Local-timezone handling for [`ExpressionParser`].
//!
//! When the user's local timezone offset is configured, `now` and bare
//! (timezone-less) times such as `12:30` are interpreted in that timezone
//! instead of UTC. Inputs with an explicit timezone (e.g. `12:30 UTC`) are
//! always honored regardless of this setting.
//!
//! This is a child module of `expression_parser`, so these `impl` blocks can
//! access `ExpressionParser`'s private `local_offset_seconds` field.

use super::ExpressionParser;
use crate::types::DateTime;

impl ExpressionParser {
    /// Sets the user's local timezone offset in seconds east of UTC.
    ///
    /// Pass `None` to fall back to the default UTC interpretation.
    pub fn set_local_offset_seconds(&mut self, offset_seconds: Option<i32>) {
        self.local_offset_seconds = offset_seconds;
    }

    /// Returns the configured local timezone offset in seconds east of UTC, if any.
    #[must_use]
    pub fn local_offset_seconds(&self) -> Option<i32> {
        self.local_offset_seconds
    }

    /// Returns a `DateTime` representing the current instant, honoring the
    /// configured local timezone offset when one is set.
    pub(super) fn current_now(&self) -> DateTime {
        match self.local_offset_seconds {
            Some(offset) => DateTime::now_local(offset),
            None => DateTime::now_with_label("current UTC time", Some(0), Some("UTC".to_string())),
        }
    }
}
