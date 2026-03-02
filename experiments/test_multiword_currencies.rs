use link_calculator::Calculator;

fn main() {
    let mut calc = Calculator::new();
    
    let tests = vec![
        "1000 USD in british pound",
        "1000 USD in british pounds",
        "1000 USD in pound",
        "1000 USD in pounds",
        "1000 USD in swiss franc",
        "1000 USD in swiss francs",
        "1000 USD in franc",
        "1000 USD in francs",
        "1000 USD in livre sterling",
        "1000 USD en franc suisse",
        "1000 USD en livre sterling",
        "1000 USD en livre",
        "1000 USD en livres",
    ];
    
    for expr in &tests {
        let result = calc.calculate_internal(expr);
        println!("{}: success={}, result={}", expr, result.success, result.result);
        if let Some(err) = &result.error {
            println!("  error: {}", err);
        }
    }
}
