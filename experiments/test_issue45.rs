// Experiment: Test current behavior for issue #45
// Check links notation for standalone datetime and is_live_time flag

use link_calculator::Calculator;

fn main() {
    let mut calc = Calculator::new();

    // Test standalone datetime
    let result = calc.calculate_internal("Jan 27, 9:33am UTC");
    println!("=== Input: Jan 27, 9:33am UTC ===");
    println!("Result: {}", result.result);
    println!("Links Notation: {}", result.lino_interpretation);
    println!("is_live_time: {:?}", result.is_live_time);
    println!("Steps:");
    for step in &result.steps {
        println!("  - {step}");
    }
    println!();

    // Test "now" expression for comparison
    let result2 = calc.calculate_internal("now");
    println!("=== Input: now ===");
    println!("Result: {}", result2.result);
    println!("Links Notation: {}", result2.lino_interpretation);
    println!("is_live_time: {:?}", result2.is_live_time);
    println!();

    // Test standalone date without time
    let result3 = calc.calculate_internal("Jan 27, 2026");
    println!("=== Input: Jan 27, 2026 ===");
    println!("Result: {}", result3.result);
    println!("Links Notation: {}", result3.lino_interpretation);
    println!("is_live_time: {:?}", result3.is_live_time);
    println!("Steps:");
    for step in &result3.steps {
        println!("  - {step}");
    }
    println!();

    // Test datetime subtraction
    let result4 = calc.calculate_internal("(Jan 27, 9:33am UTC) - (Jan 25, 12:51pm UTC)");
    println!("=== Input: (Jan 27, 9:33am UTC) - (Jan 25, 12:51pm UTC) ===");
    println!("Result: {}", result4.result);
    println!("Links Notation: {}", result4.lino_interpretation);
    println!("is_live_time: {:?}", result4.is_live_time);
    println!();

    // Future date
    let result5 = calc.calculate_internal("Dec 31, 2026");
    println!("=== Input: Dec 31, 2026 ===");
    println!("Result: {}", result5.result);
    println!("Links Notation: {}", result5.lino_interpretation);
    println!("is_live_time: {:?}", result5.is_live_time);
    println!("Steps:");
    for step in &result5.steps {
        println!("  - {step}");
    }
}
