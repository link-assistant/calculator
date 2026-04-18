# Case Study: Issue #132 — Unrecognized input: 63!

## Summary

**Issue URL:** https://github.com/link-assistant/calculator/issues/132  
**Version:** 0.13.2  
**Status:** Fixed in PR #133  
**Labels:** bug  
**Reported by:** konard (Konstantin Diachenko)  
**Opened:** 2026-04-12

The user typed `63!` expecting factorial computation. Instead, the calculator returned:

```
Parse error: Unexpected character '!' at position 2
```

The expected behavior is for the calculator to evaluate `63!` as **26,012,963,668,938,446,981,945,847,715,207,989,052,594,335,780,736,960,000,000,000,000** (the factorial of 63).

---

## Input and Error

```
63!
```

```
Parse error: Unexpected character '!' at position 2
```

---

## Timeline / Sequence of Events

1. User types `63!` in the calculator input.
2. The `Lexer` in `src/grammar/lexer.rs` calls `next_token()`.
3. The lexer reads `6`, `3` as a `Number("63")` token — OK.
4. The lexer then reads `!`, hits the catch-all `_` branch:
   ```rust
   _ => {
       return Err(CalculatorError::parse(format!(
           "Unexpected character '{ch}' at position {start}"
       )));
   }
   ```
5. The error `Unexpected character '!' at position 2` is returned to the user.

The `!` character was never added to the lexer or parser, even though a `factorial(n)` function already exists in `src/grammar/math_functions.rs` and is fully functional.

---

## Requirements Identified

| # | Requirement | Source |
|---|-------------|--------|
| R1 | Support postfix `!` factorial notation (e.g., `63!` → 26012963668938446981945847715207989052594335780736960000000000000) | Issue description |
| R2 | Support chained/complex factorial expressions (e.g., `(5+3)!`, `5! + 3!`, `n! / k!`) | Standard math notation |
| R3 | Error message must be clear if factorial input is invalid (e.g., negative number, non-integer) | Existing `factorial()` error handling |
| R4 | LaTeX and LINO representations must display factorial correctly | Existing pattern for other operations |

---

## Root Cause Analysis

### Root Cause 1: Lexer does not recognize `!`

**Location:** `src/grammar/lexer.rs`, method `next_token()`, line ~297

The `!` character is not listed in any match arm of the `next_token()` method. The catch-all `_` branch produces the error.

```rust
// BEFORE (broken — no '!' case):
_ => {
    return Err(CalculatorError::parse(format!(
        "Unexpected character '{ch}' at position {start}"
    )));
}
```

**Fix needed:** Add `Bang` token kind and handle `!` in the lexer.

### Root Cause 2: Parser does not handle `!` as a postfix operator

**Location:** `src/grammar/token_parser.rs`, method `parse_unary()`, ~line 102

Even if the lexer produced a `Bang` token, the parser would not know to interpret `n!` as `factorial(n)`. The postfix handling pattern already exists for `%`:

```rust
// In parse_unary:
// Handle postfix percent operator: expr% → expr / 100
if matches!(self.current_kind(), Some(TokenKind::Percent)) {
    self.advance();
    return Ok(Expression::binary(
        expr,
        BinaryOp::Divide,
        Expression::number(Decimal::new(100)),
    ));
}
```

**Fix needed:** Add analogous handling: `n!` → `factorial(n)` function call.

### Root Cause 3: `factorial` function already exists but is unreachable via `!` syntax

**Location:** `src/grammar/math_functions.rs`, line 272

```rust
"factorial" => {
    check_arg_count(&name_lower, args, 1)?;
    let n = args[0].to_f64();
    if n < 0.0 || n != n.floor() {
        return Err(CalculatorError::domain(
            "factorial argument must be a non-negative integer",
        ));
    }
    let n_int = n as u64;
    if n_int > 170 {
        return Err(CalculatorError::Overflow);
    }
    let result = factorial(n_int);
    Ok(Decimal::from_f64(result))
}
```

The function is complete and already handles edge cases. It is already listed in `is_math_function()`. The only missing piece is supporting the `n!` postfix syntax that routes to this function.

---

## Related Components and Libraries Investigated

### kalkulator (Rust crate)
- The [`kalkulator`](https://docs.rs/kalkulator/latest/kalkulator/) Rust crate supports factorial via postfix `!`, e.g. `4!/(2+3)` evaluates to 24.
- Approach: `!` is tokenized as `FactorialToken` and parsed as a postfix unary operator during parsing.

### Shunting-yard algorithm ([Wikipedia](https://en.wikipedia.org/wiki/Shunting-yard_algorithm))
- Classic algorithm for parsing infix expressions with correct precedence.
- Postfix unary operators (like `!`) can be handled as having the highest precedence and immediately popped off the operator stack when encountered after an operand.

### Christian Kramp notation (1808)
- The `!` factorial notation was introduced by Christian Kramp in 1808.
- It is universally recognized in scientific calculators, programming languages (Wolfram Language: `n!`), and math software.

### Python and C++ standard libraries
- Python `math.factorial(n)`, Wolfram Language `n!`, Haskell `product [1..n]`
- All major calculators (TI, Casio, HP, Wolfram Alpha) support `n!` notation.

---

## Proposed Solutions

### Solution A (Recommended): Add `!` as a postfix lexer token, handle in parser

**Steps:**
1. Add `Bang` variant to `TokenKind` enum in `src/grammar/lexer.rs`
2. Add `'!'` to the match arm in `next_token()` in `src/grammar/lexer.rs`
3. In `parse_unary()` in `src/grammar/token_parser.rs`, after parsing a primary expression, check for `Bang` token and convert `expr!` → `Expression::function_call("factorial", vec![expr])`

**Pros:**
- Minimal change, follows existing `%` postfix pattern
- Reuses fully-implemented `factorial()` function
- Correctly handles complex expressions: `(5+3)!`, `5! + 3!`, `5! * 2`
- LaTeX output `n!` can be special-cased in `to_latex()` for proper display

**Cons:**
- Need to handle multiple postfix `!` e.g. `(3!)!` — but this is naturally handled since `parse_unary` loops/recurses

### Solution B: Transform `n!` at the expression parser level without new token

- In `ExpressionParser`, pre-process the token stream to replace `number !` with `factorial(number)`
- **Cons:** More complex, harder to maintain

### Solution C: New `Factorial` `Expression` variant

- Add a dedicated `Expression::Factorial(Box<Expression>)` AST node
- **Pros:** Cleaner AST representation
- **Cons:** Requires changes throughout (evaluate, to_lino, to_latex, collect_currencies), more code for the same result
- Not needed since `FunctionCall { name: "factorial", args: [expr] }` is already well-handled

---

## Chosen Solution: Solution A

**Implementation plan:**

1. `src/grammar/lexer.rs`:
   - Add `Bang` to `TokenKind` enum
   - Add `'!' => Token::new(TokenKind::Bang, ...)` to `next_token()`

2. `src/grammar/token_parser.rs`:
   - In `parse_unary()`, after primary parse, check for `Bang`:
     ```rust
     if matches!(self.current_kind(), Some(TokenKind::Bang)) {
         self.advance();
         return Ok(Expression::function_call("factorial", vec![expr]));
     }
     ```

3. `src/types/expression.rs` `to_latex()`:
   - In the `"factorial"` case, output `n!` LaTeX notation:
     ```rust
     "factorial" if args.len() == 1 => {
         format!("{}!", args[0].to_latex())
     }
     ```

4. `tests/issue_132_factorial_tests.rs`:
   - Add test cases covering:
     - `63!` → large integer result
     - `5!` → 120
     - `0!` → 1
     - `(3+2)!` → 120
     - `5! + 3!` → 126
     - `factorial(5)` still works
     - `(-1)!` → error
     - `3.5!` → error

5. `docs/case-studies/issue-132/README.md` — this file

---

## Verification

After implementing Solution A:

- `cargo test` must pass all existing and new tests
- `63!` should evaluate to 26012963668938446981945847715207989052594335780736960000000000000
- `5!` should evaluate to 120
- `factorial(5)` should still work unchanged
