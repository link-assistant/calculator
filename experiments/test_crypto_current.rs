// Experiment: test current behavior with TON/crypto and unit conversions
fn main() {
    // This is a test script to understand current behavior.
    // Since we can't import the library directly here, we note what we've read:
    // - "100 USD in EUR" is in examples.lino, so "in" must be handled somehow
    // - The lexer only has "at" and "as" keywords
    // - So "in" must be parsed as an identifier
    println!("Checking grammar for 'in' keyword handling...");
    println!("Based on code reading:");
    println!("- Lexer only makes 'at' and 'as' keywords");
    println!("- 'in' would be tokenized as Identifier('in')");
    println!("- The token parser looks for 'as' keyword (check_as)");
    println!("- But 'in' has no special handling");
    println!("- So '100 USD in EUR' probably fails or 'in EUR' is parsed as custom unit");
}
