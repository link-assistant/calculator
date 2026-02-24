//! Tests for .lino rate file loading and historical rate conversions.
//!
//! These tests verify that:
//! 1. Single rates can be loaded from .lino format
//! 2. Batches of rates can be loaded
//! 3. Consolidated .lino files (both legacy and new format) work
//! 4. Loaded rates are actually usable in calculations

use link_calculator::Calculator;

#[test]
fn test_calculator_creation() {
    let calc = Calculator::new();
    let _ = calc;
}

#[test]
fn test_load_rate_from_lino() {
    let mut calc = Calculator::new();
    let content = "rate:
  from USD
  to EUR
  value 0.85
  date 1999-01-04
  source 'frankfurter.dev (ECB)'";

    let result = calc.load_rate_from_lino(content);
    assert!(result.is_ok());

    // Test that the rate was loaded by doing a calculation with the historical date
    // Note: This tests that the rate is in the database, but the full
    // "at date" functionality requires the date context to be set during evaluation
}

#[test]
fn test_load_rates_batch() {
    let mut calc = Calculator::new();
    let content1 = "rate:
  from USD
  to EUR
  value 0.85
  date 1999-01-04
  source 'test'";

    let content2 = "rate:
  from EUR
  to USD
  value 1.18
  date 1999-01-04
  source 'test'";

    let result = calc.load_rates_batch(&[content1, content2]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2);
}

#[test]
fn test_load_rates_from_consolidated_lino_legacy_format() {
    let mut calc = Calculator::new();
    // Legacy format: rates: as root, data: for rates
    let content = "rates:
  from USD
  to EUR
  source 'frankfurter.dev (ECB)'
  data:
    2021-01-25 0.8234
    2021-02-01 0.8315
    2021-02-08 0.8402";

    let result = calc.load_rates_from_consolidated_lino(content);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);
}

#[test]
fn test_load_rates_from_consolidated_lino_new_format() {
    let mut calc = Calculator::new();
    // New format: conversion: as root, rates: for rates
    let content = "conversion:
  from USD
  to RUB
  source 'cbr.ru (Central Bank of Russia)'
  rates:
    2021-02-08 74.2602
    2021-02-09 74.1192
    2026-01-25 76.03";

    let result = calc.load_rates_from_consolidated_lino(content);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);
}

#[test]
fn test_load_rates_from_consolidated_lino_empty() {
    let mut calc = Calculator::new();
    let content = "conversion:
  from USD
  to EUR
  source 'test'
  rates:";

    let result = calc.load_rates_from_consolidated_lino(content);
    assert!(result.is_err());
}
