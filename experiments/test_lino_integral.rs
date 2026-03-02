// Quick experiment to test current lino output for indefinite integrals
// Run with: cargo test --test integration_test
// or compile as a bin

fn main() {
    // Show the expected transformation from issue #89:
    // Input: integrate cos(x) dx
    // Current lino: (integrate (cos (x)) dx)
    // Expected lino: (integrate ((cos (x)) * (differential of (x))))
    
    println!("Current: (integrate (cos (x)) dx)");
    println!("Expected: (integrate ((cos (x)) * (differential of (x))))");
    
    // Other test cases:
    // integrate x^2 dx
    // Current: (integrate (x ^ 2) dx)
    // Expected: (integrate ((x ^ 2) * (differential of (x))))
    
    // integrate sin(x)/x dx
    // Current: (integrate ((sin (x)) / x) dx)
    // Expected: (integrate (((sin (x)) / x) * (differential of (x))))
}
