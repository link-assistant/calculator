//! Tests for issue #37: Recursive Real Arithmetic (RRA) support.
//!
//! Verifies that the calculator can precisely represent computable numbers
//! using arbitrary-precision rational arithmetic (BigInt-based Rational).

use link_calculator::Calculator;

// --- Core requirement: 10^100 must be representable exactly and not equal to 0 ---

#[test]
fn test_10_pow_100_is_exact() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10^100");
    assert!(result.success, "10^100 should succeed");
    // Must be 1 followed by exactly 100 zeros
    assert_eq!(result.result.len(), 101, "10^100 should have 101 digits");
    assert!(result.result.starts_with('1'), "10^100 should start with 1");
    assert!(
        result.result[1..].chars().all(|c| c == '0'),
        "10^100 should be followed by 100 zeros"
    );
}

#[test]
fn test_10_pow_100_not_zero() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10^100");
    assert!(result.success);
    assert_ne!(result.result, "0", "10^100 must not be 0");
}

#[test]
fn test_10_pow_100_not_infinity() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10^100");
    assert!(result.success);
    assert!(
        !result.result.contains("inf"),
        "10^100 must not be infinity"
    );
    assert!(
        !result.result.contains("Inf"),
        "10^100 must not be Infinity"
    );
}

// --- Large exponent arithmetic ---

#[test]
fn test_2_pow_256_exact() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("2^256");
    assert!(result.success);
    // 2^256 = 115792089237316195423570985008687907853269984665640564039457584007913129639936
    assert_eq!(
        result.result,
        "115792089237316195423570985008687907853269984665640564039457584007913129639936"
    );
}

#[test]
fn test_large_power_arithmetic() {
    let mut calc = Calculator::new();
    // 10^100 / 10^100 should be exactly 1
    let result = calc.calculate_internal("10^100 / 10^100");
    assert!(result.success);
    assert_eq!(result.result, "1");
}

#[test]
fn test_large_power_subtraction() {
    let mut calc = Calculator::new();
    // 10^50 * 10^50 - 10^100 should be 0
    let result = calc.calculate_internal("10^50 * 10^50 - 10^100");
    assert!(result.success);
    assert_eq!(result.result, "0");
}

#[test]
fn test_large_power_addition() {
    let mut calc = Calculator::new();
    // 10^100 + 1 should not lose the +1
    let result = calc.calculate_internal("10^100 + 1");
    assert!(result.success);
    let s = &result.result;
    assert_eq!(s.len(), 101);
    // Should end with 1, not 0
    assert!(s.ends_with('1'), "10^100 + 1 should end with 1, got: {s}");
}

// --- Previously working exact arithmetic still works ---

#[test]
fn test_small_powers_still_exact() {
    let mut calc = Calculator::new();

    let result = calc.calculate_internal("2^10");
    assert!(result.success);
    assert_eq!(result.result, "1024");

    let result = calc.calculate_internal("3^5");
    assert!(result.success);
    assert_eq!(result.result, "243");
}

#[test]
fn test_2_pow_64_exact() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("2^64");
    assert!(result.success);
    assert_eq!(result.result, "18446744073709551616");
}

// --- Power edge cases ---

#[test]
fn test_power_of_zero() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("5^0");
    assert!(result.success);
    assert_eq!(result.result, "1");
}

#[test]
fn test_power_of_one() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("999^1");
    assert!(result.success);
    assert_eq!(result.result, "999");
}

#[test]
fn test_negative_exponent() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("2^(-1)");
    assert!(result.success);
    assert_eq!(result.result, "0.5");
}

#[test]
fn test_right_associative_power() {
    let mut calc = Calculator::new();
    // 2^3^2 = 2^(3^2) = 2^9 = 512 (right-associative)
    let result = calc.calculate_internal("2^3^2");
    assert!(result.success);
    assert_eq!(result.result, "512");
}

// --- Exact fractional arithmetic preserved ---

#[test]
fn test_exact_fraction_one_third_times_three() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1/3 * 3");
    assert!(result.success);
    assert_eq!(result.result, "1");
}

#[test]
fn test_exact_fraction_power() {
    let mut calc = Calculator::new();
    // (1/2)^3 = 1/8 = 0.125
    let result = calc.calculate_internal("(1/2)^3");
    assert!(result.success);
    assert_eq!(result.result, "0.125");
}

// --- Googol operations ---

#[test]
fn test_googol_operations() {
    let mut calc = Calculator::new();
    // A googol is 10^100
    let result = calc.calculate_internal("10^100 * 2");
    assert!(result.success);
    assert!(result.result.starts_with('2'));
    assert_eq!(result.result.len(), 101); // 2 followed by 100 zeros
}

#[test]
fn test_beyond_i128_range() {
    let mut calc = Calculator::new();
    // i128 max is ~1.7×10^38, so 10^50 exceeds it
    let result = calc.calculate_internal("10^50");
    assert!(result.success);
    assert_eq!(result.result.len(), 51); // 1 followed by 50 zeros
    assert_ne!(result.result, "0");
}
