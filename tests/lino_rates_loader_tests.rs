//! Integration test covering the consolidated-LINO rate loader on
//! [`link_calculator::Calculator`].
//!
//! Previously lived as an inline `#[cfg(test)] mod tests` inside `src/lib.rs`,
//! but every API it touches is public, so it was extracted to keep `lib.rs`
//! under the 1000-line repository limit.

use link_calculator::types::DateTime;
use link_calculator::Calculator;

#[test]
fn lino_rates_used_in_historical_conversion() {
    let mut calc = Calculator::new();
    let content = "conversion:
  from USD
  to RUB
  source 'cbr.ru (Central Bank of Russia)'
  rates:
    2021-02-08 74.2602
    2021-02-09 74.1192";

    let loaded = calc.load_rates_from_consolidated_lino(content);
    assert!(loaded > 0);

    let date = DateTime::from_date(chrono::NaiveDate::from_ymd_opt(2021, 2, 8).unwrap());
    let rate = calc
        .parser()
        .currency_db()
        .get_historical_rate("USD", "RUB", &date);
    assert!(rate.is_some());
    let rate_value = rate.unwrap();
    assert!((rate_value - 74.2602).abs() < 0.001);
}
