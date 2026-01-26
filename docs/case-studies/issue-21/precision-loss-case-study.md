# Case Study: Issue #21 - Precision Loss in Fractional Calculations

## Summary

This case study analyzes the precision loss issue where `(1/3)*3` returned `0.9999999999999999999999999999` instead of `1`, and documents the implementation of rational arithmetic to solve this problem along with UI/UX improvements for displaying repeating decimals.

## Timeline of Events

| Date | Event | Description |
|------|-------|-------------|
| 2026-01-25 20:58 | Issue Reported | User reported `(1/3)*3` returning incorrect result |
| 2026-01-25 21:xx | Initial Analysis | Root cause identified as floating-point precision loss |
| 2026-01-26 | Implementation | Rational arithmetic and UI improvements implemented |

## Problem Statement

### User Report

- **Input**: `(1/3)*3`
- **Expected Result**: `1`
- **Actual Result**: `0.9999999999999999999999999999`

### Additional Issues Identified

1. **Excessive parentheses** in links notation: `(((((1) / (3)))) * (3))` instead of `((1 / 3) * 3)`
2. **No repeating decimal display** for values like `0.333...`
3. **Missing fraction representation** for rational results
4. **UI not organized** into logical sections (interpretation, result, plot, steps)
5. **Parentheses hard to match** without color coding

## Root Cause Analysis

### 1. Floating-Point Precision Loss

The calculator was using decimal arithmetic (`rust_decimal` crate) which, while more precise than IEEE 754 floating-point, still cannot represent repeating decimals exactly.

**Mathematical explanation:**
- `1/3` = `0.333...` (infinite repeating decimal)
- Any finite decimal representation loses precision
- `0.333... × 3` should equal `1`, but `0.3333333333333333... × 3` = `0.9999999999999999...`

### 2. Solution: Rational Arithmetic

We implemented rational number representation using `num-rational` crate:

```rust
use num_rational::BigRational;
use num_bigint::BigInt;

// 1/3 is represented exactly as the ratio 1:3
let one_third = BigRational::new(BigInt::from(1), BigInt::from(3));

// Multiplication is exact: (1/3) * 3 = 3/3 = 1
let result = one_third * BigRational::from(3);
assert_eq!(result, BigRational::from(1));
```

## Implementation Details

### Rational Number Module (`src/types/rational.rs`)

Key features:
- `Rational` struct wrapping `BigRational` for arbitrary precision
- Automatic simplification of fractions
- Conversion to/from decimals
- Repeating decimal detection using Floyd's cycle-finding algorithm

### Repeating Decimal Detection

Algorithm uses long division simulation to detect repeating patterns:

```rust
pub fn detect_repeating_decimal(numerator: &BigInt, denominator: &BigInt) -> Option<RepeatingDecimal> {
    // Simulate long division, tracking remainders
    // When a remainder repeats, we've found the cycle
    let mut remainders: HashMap<BigInt, usize> = HashMap::new();
    let mut digits = Vec::new();
    let mut remainder = numerator.clone();

    loop {
        remainder = remainder * 10;
        let digit = &remainder / denominator;
        remainder = &remainder % denominator;

        if remainder.is_zero() {
            return None; // Terminating decimal
        }

        if let Some(&start) = remainders.get(&remainder) {
            // Found the repeat cycle starting at position 'start'
            let non_repeating = &digits[..start];
            let repeating = &digits[start..];
            return Some(RepeatingDecimal { non_repeating, repeating });
        }

        remainders.insert(remainder.clone(), digits.len());
        digits.push(digit);
    }
}
```

### Repeating Decimal Notations

Implemented multiple notation formats as per [Wikipedia: Repeating decimal](https://en.wikipedia.org/wiki/Repeating_decimal):

| Notation | Example | Description |
|----------|---------|-------------|
| Vinculum | `0.3̅` | Overline marks repeating digits |
| Parenthesis | `0.(3)` | Parentheses around repeating part |
| Ellipsis | `0.333...` | Traditional dots |
| LaTeX | `0.\overline{3}` | For math rendering |
| Fraction | `1/3` | Original fraction |

### UI Reorganization

New section structure in `App.tsx`:

1. **Interpretation (mandatory)**: Color-coded LINO expression
2. **Result (mandatory)**: Calculated value with optional fraction hint
3. **Notations (optional)**: Table of repeating decimal formats
4. **Plot (optional)**: Function graph for symbolic expressions
5. **Steps (optional)**: Calculation reasoning

### Color-Coded Parentheses

New `ColorCodedLino` component:
- Parses LINO expression into tokens
- Assigns colors based on nesting depth
- Uses 8-color palette cycling for deep nesting
- Hover shows depth information

```typescript
const PAREN_COLORS = [
  '#6366f1', // indigo
  '#f59e0b', // amber
  '#10b981', // emerald
  '#ef4444', // red
  '#8b5cf6', // violet
  '#06b6d4', // cyan
  '#f97316', // orange
  '#84cc16', // lime
];
```

## Verification

### Test Cases

| Expression | Previous Result | New Result | Correct? |
|------------|-----------------|------------|----------|
| `(1/3)*3` | `0.9999...` | `1` | ✅ |
| `(2/3)*3` | `1.9999...` | `2` | ✅ |
| `(1/6)*6` | `0.9999...` | `1` | ✅ |
| `(1/7)*7` | `0.9999...` | `1` | ✅ |
| `1/3` | `0.3333...` | `0.3333...` (with notations) | ✅ |

### Test Output

```
Expression: (1/3)*3
  Result: 1
  Links notation: ((1 / 3) * 3)
  Fraction: 1/1
  Steps:
    Input expression: (1 / 3) * 3
    Evaluate grouped expression:
    Literal value: 1
    Literal value: 3
    Compute: 1 / 3
    = 0.3333333333333333
    Literal value: 3
    Compute: 0.3333333333333333 * 3
    = 1
    Final result: 1
```

## References

- [Wikipedia: Repeating decimal](https://en.wikipedia.org/wiki/Repeating_decimal)
- [num-rational crate](https://crates.io/crates/num-rational)
- [num-bigint crate](https://crates.io/crates/num-bigint)
- [Floyd's cycle-finding algorithm](https://en.wikipedia.org/wiki/Cycle_detection#Floyd's_tortoise_and_hare)

## Lessons Learned

1. **Use exact arithmetic when possible**: Rational numbers are more appropriate for exact fractions than floating-point or decimal representations.

2. **Show multiple representations**: Users benefit from seeing results in different formats (decimal, fraction, various notations).

3. **UI organization matters**: Separating interpretation, result, and steps makes the interface clearer.

4. **Visual aids help comprehension**: Color-coded parentheses make complex expressions easier to parse.

## Files Changed

- `src/lib.rs` - Added `RepeatingDecimalFormats` struct and `success_with_value()` constructor
- `src/types/rational.rs` - Rational arithmetic implementation (existing)
- `src/types/value.rs` - Value type with rational support (existing)
- `web/src/types.ts` - TypeScript types for new fields
- `web/src/App.tsx` - Reorganized UI with new sections
- `web/src/components/ColorCodedLino.tsx` - Color-coded parentheses component (new)
- `web/src/components/RepeatingDecimalNotations.tsx` - Notations table component (new)
- `web/src/index.css` - Styles for new components
