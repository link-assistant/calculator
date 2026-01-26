# Case Study: Issue #30 - Extra Parentheses in Links Notation for DateTime Subtraction

## Summary

**Issue**: The calculator was generating Links notation with three sets of outer parentheses instead of two when displaying datetime subtraction results.

**Input**: `(Jan 27, 8:59am UTC) - (Jan 26, 10:20am UTC)`

**Incorrect Output**: `(((Jan 27, 8:59am UTC) - (Jan 26, 10:20am UTC)))`

**Expected Output**: `((Jan 27, 8:59am UTC) - (Jan 26, 10:20am UTC))`

## Timeline of Events

1. **2026-01-26T10:21:28.603Z**: User encountered the bug while using the calculator at https://link-assistant.github.io/calculator/
2. **2026-01-26T10:23:22Z**: Issue #30 reported by @konard with detailed environment information and reproduction steps
3. **2026-01-26**: Fix implemented, tested, and merged

## Root Cause Analysis

### Location of Bug

File: `src/grammar/datetime_grammar.rs`
Line: 188 (in `try_parse_datetime_subtraction` method)

### Problematic Code

```rust
let lino = format!("((({}) - ({})))", first_dt_str.trim(), second_dt_str.trim());
```

### Analysis

The `try_parse_datetime_subtraction` function in `DateTimeGrammar` handles the special case of datetime subtraction expressions like `(datetime) - (datetime)`. When generating the Links notation representation, the code was adding **three** sets of outer parentheses:

1. One set around each datetime value: `({datetime})`
2. One set around the entire binary expression: `((...) - (...))`
3. **An extra, unnecessary set** around the whole expression: `(((..)))`

### Links Notation Convention

In Links notation, parentheses serve to:
1. Group operands in binary expressions
2. Preserve the explicit structure from user input

For a datetime subtraction:
- The user input `(Jan 27, 8:59am UTC) - (Jan 26, 10:20am UTC)` has parentheses around each datetime
- The Links notation should represent the binary operation with one additional set of parentheses: `((datetime1) - (datetime2))`

The triple parentheses were semantically incorrect and violated the principle of minimal parentheses in Links notation.

## Solution

### Code Change

```rust
// Before (Bug):
let lino = format!("((({}) - ({})))", first_dt_str.trim(), second_dt_str.trim());

// After (Fix):
let lino = format!("(({}) - ({}))", first_dt_str.trim(), second_dt_str.trim());
```

### Test Added

Added a specific regression test in `tests/integration_test.rs`:

```rust
#[test]
fn test_datetime_subtraction_lino_notation_issue_30() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("(Jan 27, 8:59am UTC) - (Jan 26, 10:20am UTC)");
    assert!(result.success);

    let lino = &result.lino_interpretation;

    // Count leading and trailing parentheses
    let leading_parens = lino.chars().take_while(|&c| c == '(').count();
    let trailing_parens = lino.chars().rev().take_while(|&c| c == ')').count();

    assert_eq!(leading_parens, 2, "Should have exactly 2 leading parentheses");
    assert_eq!(trailing_parens, 2, "Should have exactly 2 trailing parentheses");
}
```

## Impact Assessment

### Affected Functionality
- DateTime subtraction display in Links notation
- No impact on calculation correctness (result was always correct)
- No impact on other expression types

### Regression Risk
- Low: The change is isolated to a single format string
- Test coverage ensures the fix doesn't introduce regressions

## Related Research

### Links Notation Background
Links notation is a semantic representation format used in the Link Calculator project. It aims to provide:
- Unambiguous expression representation
- Minimal but sufficient parentheses
- Human-readable format that can be parsed back

### Similar Issues in Other Projects
Expression pretty-printing with correct parentheses is a common challenge in:
- Compilers and interpreters
- Mathematical notation systems
- Code formatters

Libraries that handle similar problems:
- **pretty-print** crates in Rust ecosystem
- S-expression formatters (LISP-like syntax)
- LaTeX expression generators

## Lessons Learned

1. **Test edge cases in output formatting**: The datetime subtraction path was a special case that didn't go through the normal `Expression::to_lino()` method, which already had correct parentheses handling.

2. **Consistency in code paths**: When adding special-case handling (like `try_parse_datetime_subtraction`), ensure the output format is consistent with the general case.

3. **Value of specific test cases**: The issue was clearly reproducible with a specific test case, making it easy to verify the fix.

## Files Modified

1. `src/grammar/datetime_grammar.rs` - Fixed the parentheses count in Links notation generation
2. `tests/integration_test.rs` - Added regression test for issue #30
3. `docs/case-studies/issue-30/` - Added case study documentation

## References

- Issue: https://github.com/link-assistant/calculator/issues/30
- Pull Request: https://github.com/link-assistant/calculator/pull/31
- Calculator URL with bug reproduction: https://link-assistant.github.io/calculator/?q=KGV4cHJlc3Npb24lMjAlMjIoSmFuJTIwMjclMkMlMjA4JTNBNTlhbSUyMFVUQyklMjAtJTIwKEphbiUyMDI2JTJDJTIwMTAlM0EyMGFtJTIwVVRDKSUyMik%3D
