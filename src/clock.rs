//! The calculator's clock.
//!
//! Every reading of the current time goes through [`now`], so fixing the
//! instant with [`set_fixed_now`] makes `now`, `today`, dates without a year
//! and rate timestamps reproducible. The JavaScript engine has the same
//! override (`js/src/clock.js`); the parity fixtures in `parity/goldens/` are
//! computed at a fixed instant with both.

use chrono::{DateTime, Utc};
use std::cell::Cell;

thread_local! {
    static FIXED_NOW: Cell<Option<DateTime<Utc>>> = const { Cell::new(None) };
}

/// Returns the current instant, or the instant fixed by [`set_fixed_now`].
#[must_use]
pub fn now() -> DateTime<Utc> {
    FIXED_NOW.with(Cell::get).unwrap_or_else(Utc::now)
}

/// Fixes the instant [`now`] returns on the current thread; `None` restores
/// the system clock.
pub fn set_fixed_now(instant: Option<DateTime<Utc>>) {
    FIXED_NOW.with(|cell| cell.set(instant));
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn fixed_instant_is_returned_until_cleared() {
        let instant = Utc.with_ymd_and_hms(2026, 3, 15, 12, 34, 56).unwrap();
        set_fixed_now(Some(instant));
        assert_eq!(now(), instant);
        set_fixed_now(None);
        assert_ne!(now(), instant);
    }
}
