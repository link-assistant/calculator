use link_calculator::Calculator;
fn main() {
    let mut calc = Calculator::new();

    let cases = [
        // Using parentheses to force date parsing
        "(2025-03-15) - 2 years",
        "(2024-02-17) + 1 year",
        "(2024-02-17) + 1 yr",
        "(2024-02-17) + 1 y",
        "(2025-03-15) - 2 y",
        "(2027-02-17) - 6 months",
        "(2027-02-17) - 6 mo",
        "(2027-02-17) - 6 month",
        // Russian format (natural date parsing)
        "17 февраля 2027 - 6 месяцев",
        "15 марта 2025 - 2 года",
        "17 февраля 2024 + 1 год",
        // English formats without parens
        "Feb 17 2027 - 6 months",
        "17 February 2027 - 6 months",
    ];

    for expr in &cases {
        let r = calc.calculate_internal(expr);
        println!("{} => {} (success={})", expr, r.result, r.success);
    }
}
