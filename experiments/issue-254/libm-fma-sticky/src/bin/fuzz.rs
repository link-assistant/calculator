//! Compares `libm::fma` with `f64::mul_add` (glibc's correctly rounded `fma`
//! on x86_64 Linux) on random inputs.
//!
//! Random dense inputs never expose the bug: a lost sticky bit only matters
//! when the rest of the sum is an exact tie. The tie family puts
//! `x * y = 1 + 2^-3a` (with `x = 1 + 2^-a`, `y = 1 - 2^-a + 2^-2a`) on the
//! tie of `z = ±(2^53 + 2k)` (whose ulp is 2), scaled by `2^s`, so only the
//! `2^-3a` term (the sticky bit) decides the rounding.
//!
//!   node scripts/bounded-cargo.mjs run --release --bin fuzz \
//!     --manifest-path experiments/issue-254/libm-fma-sticky/Cargo.toml

/// xorshift64*: deterministic, so every run checks the same inputs.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: u64) -> i32 {
        i32::try_from(self.next() % n).expect("small")
    }

    /// A random sign and 52-bit mantissa with the given unbiased exponent.
    fn float(&mut self, exponent: i32) -> f64 {
        let bits = self.next();
        let biased = u64::try_from(exponent + 1023).expect("normal exponent");
        f64::from_bits(
            (bits & 0x8000_0000_0000_0000) | (biased << 52) | (bits & 0x000f_ffff_ffff_ffff),
        )
    }
}

/// Counts and prints (the first few) inputs where the two results differ.
fn compare(name: &str, samples: u32, mut input: impl FnMut() -> (f64, f64, f64)) {
    let mut mismatches = 0u32;
    for _ in 0..samples {
        let (x, y, z) = input();
        let generic = libm::fma(x, y, z);
        let std = x.mul_add(y, z);
        if generic.to_bits() != std.to_bits() {
            if mismatches < 3 {
                println!(
                    "  fma({:#018x}, {:#018x}, {:#018x}): libm {:#018x}, std {:#018x}",
                    x.to_bits(),
                    y.to_bits(),
                    z.to_bits(),
                    generic.to_bits(),
                    std.to_bits()
                );
            }
            mismatches += 1;
        }
    }
    println!("{name}: {mismatches} of {samples} results differ");
}

fn main() {
    const SAMPLES: u32 = 1_000_000;
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    compare("dense, z 2^54..2^127 above x*y", SAMPLES, || {
        let gap = 54 + rng.below(74);
        (rng.float(0), rng.float(0), rng.float(gap))
    });
    compare("tie family", SAMPLES, || {
        let a = 18 + rng.below(9);
        let scale = f64::powi(2.0, rng.below(1000) - 500);
        let x = (1.0 + f64::powi(2.0, -a)) * scale;
        let y = 1.0 - f64::powi(2.0, -a) + f64::powi(2.0, -2 * a);
        let k = rng.next() >> 13;
        #[allow(clippy::cast_precision_loss)] // k < 2^51: exact
        let z = (f64::powi(2.0, 53) + 2.0 * k as f64) * scale;
        match rng.below(4) {
            0 => (x, y, z),
            1 => (-x, y, -z),
            2 => (x, y, -z),
            _ => (-x, y, z),
        }
    });
}
