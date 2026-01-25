# Case Study: Issue #3 - Unrecognized input: integrate sin(x)/x dx

## Summary

The user tried to use natural mathematical notation `integrate sin(x)/x dx` to compute the indefinite integral of sin(x)/x, expecting behavior similar to Wolfram Alpha. The calculator returned a parse error instead.

## Timeline of Events

1. **User Input**: `integrate sin(x)/x dx`
2. **Error Received**: `Parse error: Unexpected identifier: integrate`
3. **Expected Behavior**: Recognition as indefinite integral, formula rendering, plot visualization (as shown in Wolfram Alpha screenshot)

## Root Cause Analysis

### The Actual Error

The error message is **misleading**. The actual failure occurs at parsing `dx`, not `integrate`. Here's what happens:

1. **Lexer tokenizes correctly**: Input becomes `[Identifier("integrate"), Identifier("sin"), LeftParen, Identifier("x"), RightParen, Slash, Identifier("x"), Identifier("dx")]`

2. **Parser reaches `integrate`**: Since no `(` follows, it's NOT treated as a function call

3. **Math function check passes**: `integrate` is in the `is_math_function()` list, so it's parsed as a zero-argument function call

4. **Parsing continues**: `sin(x)` is parsed as a function call, `/x` is parsed correctly

5. **`dx` fails parsing**:
   - NOT followed by `(` (not a function call)
   - NOT a single letter (not a variable)
   - NOT a registered math function
   - Triggers "Unexpected identifier" error

### Why the Error Message Shows "integrate"

The error messaging system reports the first non-standard identifier encountered, which may not be the actual failure point in the token stream.

### Root Cause Summary

The calculator currently only supports **explicit function call syntax** for integration:
```
integrate(expr, var, lower, upper)
```

It does NOT support:
1. Natural mathematical notation: `integrate sin(x)/x dx`
2. Indefinite integrals (no bounds specified)
3. The `dx` suffix notation for specifying the variable of integration

## What the Calculator Currently Supports

### Definite Integration (Working)
```
integrate(x^2, x, 0, 3)     → 9
integrate(sin(x), x, 0, pi()) → 2
```

### What's Missing
1. **Indefinite integral notation**: `integrate sin(x)/x dx`
2. **Symbolic result**: Si(x) + constant
3. **Math formula rendering**: LaTeX-style visualization
4. **Plot rendering**: Graph of the function/integral

## Proposed Solutions

### Solution 1: Parse Natural Integration Notation

Modify the parser to recognize patterns like:
- `integrate <expr> d<var>` → indefinite integral
- `integrate <expr> d<var> from <a> to <b>` → definite integral

### Solution 2: Add Indefinite Integral Support

For known integrals, return symbolic results:
- sin(x)/x → Si(x) + constant (Sine integral)

For others, explain that only definite integrals with bounds are supported.

### Solution 3: Add Math Rendering (Frontend)

Add KaTeX or MathJax to render formulas beautifully:
- Input: `integrate sin(x)/x dx`
- Rendered: ∫ sin(x)/x dx = Si(x) + C

### Solution 4: Add Plotting (Frontend)

Add function-plot or Plotly.js to visualize:
- The integrand function
- The antiderivative (if known)

## Recommended Libraries

### Math Formula Rendering
- **KaTeX** (https://katex.org/): Fast, lightweight, good for real-time rendering
- **MathJax** (https://www.mathjax.org/): More complete LaTeX support, broader compatibility

### Plotting
- **function-plot** (https://mauriciopoppe.github.io/function-plot/): Specifically designed for mathematical functions
- **Plotly.js** (https://plotly.com/javascript/): Feature-rich, interactive, widely used

### Special Functions
- **@stdlib/math-base-special-sici**: JavaScript implementation of Si(x) and Ci(x)

## Artifacts

- `expected-behavior-wolframalpha.png`: Screenshot of Wolfram Alpha handling the same query
- `current-behavior.png`: Screenshot of Link Calculator's error response

## References

- [GitHub Issue #3](https://github.com/link-assistant/calculator/issues/3)
- [Wolfram Alpha Sine Integral](https://mathworld.wolfram.com/SineIntegral.html)
- [stdlib-js sici](https://github.com/stdlib-js/math-base-special-sici)
