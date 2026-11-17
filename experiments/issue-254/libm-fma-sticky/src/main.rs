//! `libm::fma(1 + 2^-26, 1 - 2^-26 + 2^-52, 2^53)`.
//!
//! `x * y` is exactly 1 + 2^-78, so the sum is 2^53 + 1 + 2^-78: just above
//! the tie between 2^53 and 2^53 + 2, so the correctly rounded result is
//! 2^53 + 2. Only the sticky bit records the 2^-78.

fn main() {
    let x = 1.0 + f64::powi(2.0, -26);
    let y = 1.0 - f64::powi(2.0, -26) + f64::EPSILON;
    let z = f64::powi(2.0, 53);
    let expected = z + 2.0;
    let generic = libm::fma(x, y, z);
    let std = x.mul_add(y, z);
    println!("x = {x:e} ({:#018x})", x.to_bits());
    println!("y = {y:e} ({:#018x})", y.to_bits());
    println!("z = {z:e} ({:#018x})", z.to_bits());
    println!("expected (2^53 + 2) = {expected}");
    println!("libm::fma (generic)  = {generic}");
    println!("f64::mul_add (std)   = {std}");
    if generic == expected {
        println!("OK: libm::fma is correctly rounded");
    } else {
        println!("BUG: libm::fma lost the sticky bit");
    }
}
