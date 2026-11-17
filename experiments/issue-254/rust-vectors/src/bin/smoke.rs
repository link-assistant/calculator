//! Proves the generator builds: prints one JSON line.
fn main() {
    let d: rust_decimal::Decimal = "1.50".parse().expect("decimal");
    println!("{}", serde_json::json!({ "display": d.to_string() }));
}
