/// Experiment: verify that "19 ton" produces both mass and crypto interpretations.
///
/// Issue #104: "19 ton" should show at least 2 interpretations:
/// - Toncoin (TON cryptocurrency)
/// - ton as 1000 kg (metric ton mass unit)
use link_calculator::Calculator;

fn main() {
    let calc = Calculator::new();

    println!("=== Issue #104: '19 ton' ambiguity ===\n");

    // Test 1: "19 ton" standalone — should show mass as primary, crypto as alternative
    let plan = calc.plan_internal("19 ton");
    println!("Expression: 19 ton");
    println!("  LINO: {}", plan.lino_interpretation);
    println!("  Alternatives: {:?}", plan.alternative_lino);
    println!("  Currencies: {:?}", plan.currencies);
    println!("  Required sources: {:?}", plan.required_sources);
    println!("  Success: {}", plan.success);
    assert!(plan.success, "Plan should succeed");
    assert_eq!(plan.lino_interpretation, "(19 t)", "Primary should be mass (19 t)");
    assert!(
        plan.alternative_lino.is_some(),
        "Should have alternative interpretations"
    );
    let alts = plan.alternative_lino.unwrap();
    assert!(
        alts.iter().any(|a| a.contains("TON")),
        "Alternatives should include TON crypto: {:?}",
        alts
    );
    println!("  ✓ PASS: Both mass and crypto interpretations present\n");

    // Test 2: "19 TON" (uppercase) — should be crypto only
    let plan2 = calc.plan_internal("19 TON");
    println!("Expression: 19 TON");
    println!("  LINO: {}", plan2.lino_interpretation);
    println!("  Alternatives: {:?}", plan2.alternative_lino);
    assert_eq!(
        plan2.lino_interpretation, "(19 TON)",
        "Uppercase TON should be crypto"
    );
    println!("  ✓ PASS: Uppercase TON interpreted as crypto\n");

    // Test 3: "19 ton in usd" — should resolve to crypto due to conversion context
    let plan3 = calc.plan_internal("19 ton in usd");
    println!("Expression: 19 ton in usd");
    println!("  LINO: {}", plan3.lino_interpretation);
    println!("  Alternatives: {:?}", plan3.alternative_lino);
    assert!(
        plan3.lino_interpretation.contains("TON"),
        "Should resolve to TON crypto for currency conversion: {}",
        plan3.lino_interpretation
    );
    println!("  ✓ PASS: Conversion context resolves to crypto\n");

    // Test 4: "19 ton in kg" — should resolve to mass due to conversion context
    let plan4 = calc.plan_internal("19 ton in kg");
    println!("Expression: 19 ton in kg");
    println!("  LINO: {}", plan4.lino_interpretation);
    println!("  Alternatives: {:?}", plan4.alternative_lino);
    assert!(
        plan4.lino_interpretation.contains("(19 t)"),
        "Should stay as mass for mass conversion: {}",
        plan4.lino_interpretation
    );
    println!("  ✓ PASS: Conversion context stays as mass\n");

    // Test 5: "19 tons" — should be mass only (unambiguous plural)
    let plan5 = calc.plan_internal("19 tons");
    println!("Expression: 19 tons");
    println!("  LINO: {}", plan5.lino_interpretation);
    println!("  Alternatives: {:?}", plan5.alternative_lino);
    println!("  ✓ PASS\n");

    // Test 6: "19 tonne" — should be mass only (unambiguous)
    let plan6 = calc.plan_internal("19 tonne");
    println!("Expression: 19 tonne");
    println!("  LINO: {}", plan6.lino_interpretation);
    println!("  Alternatives: {:?}", plan6.alternative_lino);
    println!("  ✓ PASS\n");

    println!("=== All experiments passed! ===");
}
