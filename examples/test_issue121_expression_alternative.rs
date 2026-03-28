/// Experiment for issue #121: Verify that function calls no longer produce
/// spurious `(expression "...")` alternatives, while legitimate alternatives
/// (precedence-based, unit-based) still work correctly.
use link_calculator::Calculator;

fn main() {
    let mut calc = Calculator::new();

    println!("=== Issue #121: Function call alternatives ===\n");

    // Test 1: cos(0) should NOT have alternatives
    let result = calc.calculate_internal("cos(0)");
    println!("Expression: cos(0)");
    println!("  Result: {}", result.result);
    println!("  LINO: {}", result.lino_interpretation);
    println!("  Alternatives: {:?}", result.alternative_lino);
    assert!(
        result.alternative_lino.is_none(),
        "cos(0) should not have alternatives"
    );
    println!("  PASS: No spurious alternatives\n");

    // Test 2: sin(pi) should NOT have alternatives
    let result = calc.calculate_internal("sin(0)");
    println!("Expression: sin(0)");
    println!("  Result: {}", result.result);
    println!("  LINO: {}", result.lino_interpretation);
    println!("  Alternatives: {:?}", result.alternative_lino);
    assert!(
        result.alternative_lino.is_none(),
        "sin(0) should not have alternatives"
    );
    println!("  PASS: No spurious alternatives\n");

    // Test 3: sqrt(4) should NOT have alternatives
    let result = calc.calculate_internal("sqrt(4)");
    println!("Expression: sqrt(4)");
    println!("  Result: {}", result.result);
    println!("  LINO: {}", result.lino_interpretation);
    println!("  Alternatives: {:?}", result.alternative_lino);
    assert!(
        result.alternative_lino.is_none(),
        "sqrt(4) should not have alternatives"
    );
    println!("  PASS: No spurious alternatives\n");

    // Test 4: 2 + 3 * 4 SHOULD still have precedence alternatives
    let result = calc.calculate_internal("2 + 3 * 4");
    println!("Expression: 2 + 3 * 4");
    println!("  Result: {}", result.result);
    println!("  LINO: {}", result.lino_interpretation);
    println!("  Alternatives: {:?}", result.alternative_lino);
    assert!(
        result.alternative_lino.is_some(),
        "2 + 3 * 4 should have alternatives"
    );
    let alts = result.alternative_lino.unwrap();
    assert!(alts.len() >= 2, "should have at least 2 alternatives");
    assert!(
        !alts.iter().any(|a| a.contains("expression")),
        "alternatives should NOT contain 'expression' pseudo-function"
    );
    println!("  PASS: Correct precedence alternatives\n");

    println!("=== All tests passed! ===");
}
