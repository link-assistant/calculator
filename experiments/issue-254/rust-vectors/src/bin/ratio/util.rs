//! Random inputs and the JSON encodings the sections share.

use std::panic::{self, AssertUnwindSafe};

use num_bigint::BigInt;
use num_rational::Ratio;
use num_traits::{One, Pow, Zero};
use serde_json::{json, Value};

pub type R = Ratio<BigInt>;

pub struct Rng(pub u64);

impl Rng {
    /// splitmix64.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
    pub fn coin(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
    /// A bit length in [0, max], skewed towards small values.
    pub fn skewed(&mut self, max: u64) -> u64 {
        let m = self.below(max + 1);
        self.below(m + 1)
    }
    /// A random non-negative integer of exactly `bits` bits.
    pub fn magnitude(&mut self, bits: u64) -> BigInt {
        if bits == 0 {
            return BigInt::zero();
        }
        let words = bits.div_ceil(64);
        let mut n = BigInt::zero();
        for _ in 0..words {
            n = (n << 64usize) + BigInt::from(self.next_u64());
        }
        n >>= (words * 64 - bits) as usize;
        n | p2(bits - 1)
    }
    /// A signed integer of up to `max` bits: random digits, or 2^k +- small.
    pub fn shaped(&mut self, max: u64) -> BigInt {
        let bits = self.skewed(max);
        let m = match self.below(8) {
            0 => p2(bits) - BigInt::from(self.below(3)),
            1 => p2(bits) + BigInt::from(self.below(3)),
            _ => self.magnitude(bits),
        };
        if self.coin() {
            -m
        } else {
            m
        }
    }
    pub fn nonzero(&mut self, max: u64) -> BigInt {
        loop {
            let n = self.shaped(max);
            if !n.is_zero() {
                return n;
            }
        }
    }
    /// A reduced ratio, or (one time in four) a raw one: scaled by a small
    /// factor and/or with a negative denominator.
    pub fn ratio(&mut self, max: u64) -> R {
        let n = self.shaped(max);
        let d = self.nonzero(max);
        if self.below(4) == 0 {
            let k = BigInt::from(self.below(5) + 1);
            let k = if self.coin() { -k } else { k };
            Ratio::new_raw(n * &k, d * &k)
        } else {
            Ratio::new(n, d)
        }
    }
}

pub fn p2(k: u64) -> BigInt {
    BigInt::one() << (k as usize)
}

pub fn p10(k: u32) -> BigInt {
    Pow::pow(&BigInt::from(10), k)
}

pub fn big(n: i128) -> BigInt {
    BigInt::from(n)
}

pub fn raw(n: BigInt, d: BigInt) -> R {
    Ratio::new_raw(n, d)
}

pub fn rawi(n: i128, d: i128) -> R {
    raw(big(n), big(d))
}

pub fn catch<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    panic::catch_unwind(AssertUnwindSafe(f)).map_err(|e| {
        if let Some(s) = e.downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = e.downcast_ref::<String>() {
            s.clone()
        } else {
            "<non-string panic>".to_string()
        }
    })
}

pub fn rs(r: &R) -> String {
    format!("{}/{}", r.numer(), r.denom())
}

pub fn op(f: impl FnOnce() -> R) -> Value {
    match catch(f) {
        Ok(r) => json!(rs(&r)),
        Err(m) => json!(format!("!{m}")),
    }
}

pub fn int_op(f: impl FnOnce() -> BigInt) -> Value {
    match catch(f) {
        Ok(n) => json!(n.to_string()),
        Err(m) => json!(format!("!{m}")),
    }
}

pub fn hex(f: f64) -> String {
    format!("{:016x}", f.to_bits())
}

pub fn opt_hex(f: Option<f64>) -> Value {
    match f {
        Some(f) => json!(hex(f)),
        None => Value::Null,
    }
}

pub fn opt_str<T: ToString>(v: Option<T>) -> Value {
    match v {
        Some(v) => json!(v.to_string()),
        None => Value::Null,
    }
}

pub fn flag(b: bool) -> Value {
    json!(u8::from(b))
}

pub fn ord(o: std::cmp::Ordering) -> Value {
    json!(o as i8)
}
