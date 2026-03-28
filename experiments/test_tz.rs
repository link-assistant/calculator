fn main() {}

#[cfg(test)]
mod tests {
    use link_calculator::Calculator;
    
    #[test]
    fn test_timezone_expressions() {
        let mut calc = Calculator::new();
        
        let expressions = vec![
            "6 PM GMT",
            "6 PM EST", 
            "6 PM GTM",
            "6 PM MSK",
            "6 PM GMT as MSK",
            "6 PM GTM as MSK",
        ];
        
        for expr in expressions {
            let result = calc.calculate_internal(expr);
            println!("Expression: {:?} => success={}, result={:?}, error={:?}", 
                expr, result.success, result.result, result.error);
        }
    }
}
