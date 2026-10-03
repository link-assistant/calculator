//! Probe: which formal-ai fallback expressions does `Calculator` handle? (issue #222)
use link_calculator::Calculator;

fn main() {
    let exprs: Vec<String> = std::io::stdin()
        .lines()
        .map_while(Result::ok)
        .filter(|l| !l.trim().is_empty())
        .collect();
    for e in exprs {
        let mut c = Calculator::new();
        let r = c.calculate_internal(&e);
        let out = if r.success {
            r.result.clone()
        } else {
            format!("ERR {}", r.error.unwrap_or_default())
        };
        let out: String = out.chars().take(120).collect();
        println!("{e:<40} => {out}");
    }
}
