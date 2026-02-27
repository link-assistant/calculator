use link_calculator::Calculator;

fn main() {
    let mut calc = Calculator::new();

    // Test cases from issue #84
    let test_cases = vec![
        "integrate(x^2, x, 0, 3)",
        "sin(0)",
        "cos(0)",
        "sqrt(16)",
        "pow(2, 3)",
        "abs(-5)",
        "ln(1)",
        "log(100)",
        "min(5, 3)",
        "max(5, 3)",
        "2 + 3",
        "2^3",
        "integrate x^2 dx",
        "integrate sin(x)/x dx",
        "sqrt(9 + 7)",
        "sqrt(abs(-16))",
        "factorial(5)",
        "pi()",
        "e()",
        // Expressions with multiple interpretations
        "2 + 3 * 4",
        "2 * 3 + 4",
        "1 + 2 + 3 * 4",
    ];

    for expr in &test_cases {
        let result = calc.calculate_internal(expr);
        println!("Input:  {expr}");
        println!("Lino:   {}", result.lino_interpretation);
        if let Some(alts) = &result.alternative_lino {
            println!("Alternatives:");
            for (i, alt) in alts.iter().enumerate() {
                println!("  [{}]: {alt}", i + 1);
            }
        }
        println!("Result: {}", result.result);
        println!("---");
    }
}
