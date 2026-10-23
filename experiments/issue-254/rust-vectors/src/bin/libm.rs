//! Vectors for js/src/rust/libm.js (issue #254): the results of the `libm`
//! 0.2.16 functions the calculator calls, next to the results of the std `f64`
//! methods (glibc on Linux), for special values and deterministic
//! pseudo-random inputs.
//!
//! Every number is the hex bit pattern of an `f64`, so -0.0, NaN payloads and
//! subnormals survive JSON. Each vector is `[args..., libm, std]`.
//!
//!   node scripts/bounded-cargo.mjs run --release \
//!     --manifest-path experiments/issue-254/rust-vectors/Cargo.toml \
//!     --bin libm > js/test/vectors/libm.json

// The thresholds in the inputs are the decimal literals of the libm sources,
// kept digit for digit so they can be matched against them.
#![allow(clippy::excessive_precision)]

use std::f64::consts::{E, FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, FRAC_PI_6, LN_2, PI, SQRT_2, TAU};
use std::fmt::Write as _;
use std::hint::black_box;

/// SplitMix64: a tiny deterministic generator, so the vectors never change.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Uniform in [lo, hi).
    fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit()
    }

    /// Integer in [lo, hi].
    fn int(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next() % ((hi - lo + 1) as u64)) as i64
    }

    /// Random significand with a binary exponent in [lo, hi] (log-uniform
    /// magnitude); exponents below -1022 give subnormals.
    fn exp_range(&mut self, lo: i32, hi: i32) -> f64 {
        let e = self.int(i64::from(lo), i64::from(hi)) as i32;
        let mant = self.next() & ((1u64 << 52) - 1);
        if e < -1022 {
            // Subnormal: keep the top bits below the implicit position.
            let shift = (-1022 - e).min(52) as u32;
            return f64::from_bits((mant | (1u64 << 52)) >> shift);
        }
        f64::from_bits((((e + 1023) as u64) << 52) | mant)
    }

    /// `exp_range` with a random sign.
    fn signed_exp_range(&mut self, lo: i32, hi: i32) -> f64 {
        let x = self.exp_range(lo, hi);
        if self.next() & 1 == 1 {
            -x
        } else {
            x
        }
    }

    /// Any finite or non-finite bit pattern.
    fn any_bits(&mut self) -> f64 {
        f64::from_bits(self.next())
    }
}

fn hex(x: f64) -> String {
    format!("\"{:016x}\"", x.to_bits())
}

fn hexf(bits: u64) -> f64 {
    f64::from_bits(bits)
}

/// Values every function sees.
fn common_specials() -> Vec<f64> {
    let mut v = vec![
        0.0,
        -0.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        -f64::NAN,
        hexf(0x7ff0_0000_0000_0001), // signalling NaN
        hexf(0x7ff8_0000_dead_beef), // quiet NaN with a payload
        hexf(1),                     // smallest subnormal
        -hexf(1),
        hexf(0x000f_ffff_ffff_ffff), // largest subnormal
        -hexf(0x000f_ffff_ffff_ffff),
        hexf(0x0008_0000_0000_0000),
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
        f64::MAX,
        f64::MIN,
        f64::EPSILON,
        1.0,
        -1.0,
        0.5,
        -0.5,
        2.0,
        -2.0,
        3.0,
        10.0,
        100.0,
        1e-10,
        1e-300,
        1e300,
        1.0 + f64::EPSILON,
        1.0 - f64::EPSILON / 2.0,
        0.1,
        0.2,
        0.3,
        E,
        LN_2,
        SQRT_2,
        SQRT_2 / 2.0,
    ];
    for x in [
        PI,
        FRAC_PI_2,
        FRAC_PI_3,
        FRAC_PI_4,
        FRAC_PI_6,
        TAU,
        3.0 * FRAC_PI_2,
    ] {
        v.push(x);
        v.push(-x);
        v.push(f64::from_bits(x.to_bits() + 1));
        v.push(f64::from_bits(x.to_bits() - 1));
    }
    v
}

/// Arguments for sin, cos and tan.
fn trig_inputs(rng: &mut Rng) -> Vec<f64> {
    let mut v = common_specials();
    for k in -40..=40 {
        v.push(f64::from(k) * FRAC_PI_4);
        v.push(f64::from(k) * FRAC_PI_2);
    }
    for k in [100.0, 1000.0, 1e5, 1e6, 1e7, 1e8] {
        v.push(k * PI);
        v.push(k * FRAC_PI_2);
    }
    v.extend([
        hexf(0x4009_21fb_000f_d5dd), // libm test_near_pi
        3.141592025756836,
        1e5,
        1e6,
        1e10,
        1e15,
        1e16,
        1e17,
        1e22,
        -1e22,
        1e100,
        1e200,
        1e300,
        -1e300,
        1e308,
        6381956970095103.0 * 2f64.powi(797), // hardest argument reduction
        hexf(0x4139_21fb_0000_0000),         // ~2^20 * pi/2, medium/large boundary
        hexf(0x4139_21fa_ffff_ffff),
        (1u64 << 52) as f64,
        (1u64 << 53) as f64,
        2f64.powi(60),
        2f64.powi(1000),
        2f64.powi(1023),
        hexf(0x3e50_0000_0000_0000), // 2^-26
        hexf(0x3e4f_ffff_ffff_ffff),
        hexf(0x3e46_a09e_0000_0000),
        hexf(0x3e40_0000_0000_0000),
        hexf(0x3fe5_9428_0000_0000), // k_tan "big" threshold
        hexf(0x3fe5_9427_ffff_ffff),
    ]);
    for _ in 0..500 {
        v.push(rng.uniform(-10.0, 10.0));
    }
    for _ in 0..300 {
        v.push(rng.uniform(-1000.0, 1000.0));
    }
    for _ in 0..300 {
        v.push(rng.signed_exp_range(-40, 0));
    }
    for _ in 0..300 {
        v.push(rng.signed_exp_range(10, 30));
    }
    for _ in 0..400 {
        v.push(rng.signed_exp_range(20, 1023));
    }
    v
}

fn asin_acos_inputs(rng: &mut Rng) -> Vec<f64> {
    let mut v = common_specials();
    for k in 1..=60 {
        let d = 2f64.powi(-k);
        v.push(1.0 - d);
        v.push(-1.0 + d);
        v.push(d);
        v.push(-d);
    }
    v.extend([
        0.975,
        0.9749999999999999,
        hexf(0x3fef_3333_0000_0000),
        1.0000000000000002,
        1.5,
        -1.5,
    ]);
    for _ in 0..800 {
        v.push(rng.uniform(-1.0, 1.0));
    }
    for _ in 0..300 {
        v.push(rng.uniform(0.5, 1.0));
    }
    for _ in 0..300 {
        v.push(rng.uniform(-1.0, -0.5));
    }
    for _ in 0..300 {
        v.push(rng.signed_exp_range(-60, -1));
    }
    for _ in 0..100 {
        v.push(rng.signed_exp_range(0, 10));
    }
    v
}

fn atan_inputs(rng: &mut Rng) -> Vec<f64> {
    let mut v = common_specials();
    for b in [
        0x3fdc_0000u64,
        0x3fe6_0000,
        0x3ff3_0000,
        0x4003_8000,
        0x4410_0000,
        0x3e40_0000,
    ] {
        v.push(hexf(b << 32));
        v.push(hexf((b << 32) - 1));
        v.push(-hexf(b << 32));
    }
    v.extend([1e20, 2f64.powi(66), 1e30, -1e30, 1e-20]);
    for _ in 0..800 {
        v.push(rng.uniform(-3.0, 3.0));
    }
    for _ in 0..600 {
        v.push(rng.signed_exp_range(-40, 70));
    }
    for _ in 0..200 {
        v.push(rng.signed_exp_range(-1074, 1023));
    }
    v
}

fn hyperbolic_inputs(rng: &mut Rng) -> Vec<f64> {
    let mut v = common_specials();
    v.extend([
        709.782712893384,
        709.7827128933841,
        710.0,
        710.4758600739439,
        710.475860073944,
        711.0,
        -710.4758600739439,
        -711.0,
        745.0,
        22.0,
        20.0,
        20.000000000000004,
        -20.0,
        0.5493061443340548, // log(3)/2
        0.2554128118829953, // log(5/3)/2
        hexf(0x3fe6_2e42_0000_0000),
        hexf(0x3ff0_0000_0000_0000 - (26u64 << 52)),
        hexf(0x3c90_0000_0000_0000),
        18.021826694558577,
        36.7368005696771,
    ]);
    for _ in 0..600 {
        v.push(rng.uniform(-25.0, 25.0));
    }
    for _ in 0..300 {
        v.push(rng.uniform(-720.0, 720.0));
    }
    for _ in 0..300 {
        v.push(rng.uniform(-1.0, 1.0));
    }
    for _ in 0..400 {
        v.push(rng.signed_exp_range(-40, 10));
    }
    v
}

fn exp_inputs(rng: &mut Rng) -> Vec<f64> {
    let mut v = common_specials();
    v.extend([
        709.782712893383973096,
        709.7827128933841,
        709.78,
        710.0,
        -708.39641853226410622,
        -708.3964185322642,
        -745.13321910194110842,
        -745.1332191019412,
        -744.0,
        -740.0,
        -1000.0,
        1000.0,
        0.34657359027997264,
        hexf(0x3fd6_2e42_0000_0000),
        hexf(0x3fd6_2e43_0000_0000),
        hexf(0x3ff0_a2b2_0000_0000),
        hexf(0x3ff0_a2b1_ffff_ffff),
        hexf(0x3e30_0000_0000_0000),
        hexf(0x3e30_0000_0000_0001),
        1e-20,
        -1e-20,
    ]);
    for k in -30..=30 {
        v.push(f64::from(k) * LN_2);
    }
    for _ in 0..600 {
        v.push(rng.uniform(-746.0, 710.0));
    }
    for _ in 0..500 {
        v.push(rng.uniform(-5.0, 5.0));
    }
    for _ in 0..500 {
        v.push(rng.signed_exp_range(-60, 9));
    }
    v
}

fn log_inputs(rng: &mut Rng) -> Vec<f64> {
    let mut v = common_specials();
    for k in -1074..=1023 {
        if k % 7 == 0 || (-5..=5).contains(&k) || k == -1074 || k == 1023 {
            v.push(2f64.powi(k));
        }
    }
    for k in -20..=22 {
        v.push(10f64.powi(k));
    }
    for k in 1..=60 {
        let d = 2f64.powi(-k);
        v.push(1.0 + d);
        v.push(1.0 - d);
    }
    v.extend([
        hexf(0x3fe6_a09e_0000_0000),
        hexf(0x3ff6_a09e_667f_3bcd),
        1000.0,
        1e15,
        0.001,
        -0.001,
    ]);
    for k in 1..=100 {
        v.push(f64::from(k));
    }
    for _ in 0..600 {
        v.push(rng.exp_range(-1074, 1023));
    }
    for _ in 0..400 {
        v.push(rng.uniform(0.5, 2.0));
    }
    for _ in 0..300 {
        v.push(1.0 + rng.signed_exp_range(-52, -10));
    }
    for _ in 0..300 {
        v.push(rng.uniform(0.0, 1000.0));
    }
    for _ in 0..50 {
        v.push(-rng.exp_range(-30, 30));
    }
    v
}

fn cbrt_inputs(rng: &mut Rng) -> Vec<f64> {
    let mut v = common_specials();
    for k in 1..=200 {
        let c = f64::from(k * k * k);
        v.push(c);
        v.push(-c);
    }
    v.extend([
        hexf(0x4009_b782_23aa_307c), // 0x1.9b78223aa307cp+1, a hard case
        hexf(0x401a_202b_fc89_ddff), // 0x1.a202bfc89ddffp+2
        -hexf(0x4009_b782_23aa_307c),
        0.001,
        1e-300,
        1e300,
        27.000000000000004,
        26.999999999999996,
    ]);
    for k in -1074..=1023 {
        if k % 5 == 0 {
            v.push(2f64.powi(k));
        }
    }
    for _ in 0..900 {
        v.push(rng.signed_exp_range(-1074, 1023));
    }
    for _ in 0..400 {
        v.push(rng.uniform(-100.0, 100.0));
    }
    for _ in 0..200 {
        v.push(rng.any_bits());
    }
    v
}

fn sqrt_inputs(rng: &mut Rng) -> Vec<f64> {
    let mut v = common_specials();
    for k in 1..=100 {
        v.push(f64::from(k * k));
    }
    for _ in 0..300 {
        v.push(rng.exp_range(-1074, 1023));
    }
    for _ in 0..100 {
        v.push(rng.uniform(0.0, 10.0));
    }
    v
}

/// `(x, y)` pairs for pow: the C standard's special cases, negative bases,
/// overflow and underflow boundaries and random inputs.
fn pow_inputs(rng: &mut Rng) -> Vec<(f64, f64)> {
    let inf = f64::INFINITY;
    let nan = f64::NAN;
    let mut v = Vec::new();
    let xs = [
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.5,
        -0.5,
        2.0,
        -2.0,
        3.0,
        -3.0,
        10.0,
        -10.0,
        inf,
        -inf,
        nan,
        1e-310,
        -1e-310,
        f64::MAX,
        f64::MIN,
        f64::MIN_POSITIVE,
        1.0000000000000002,
        0.9999999999999999,
        -0.9999999999999999,
    ];
    let ys = [
        0.0,
        -0.0,
        1.0,
        -1.0,
        2.0,
        -2.0,
        3.0,
        -3.0,
        0.5,
        -0.5,
        1.5,
        -1.5,
        1.0 / 3.0,
        inf,
        -inf,
        nan,
        1e10,
        -1e10,
        2f64.powi(31) + 1.0,
        2f64.powi(52) + 1.0,
        2f64.powi(53) - 1.0,
        2f64.powi(53),
        2f64.powi(53) + 2.0,
        2f64.powi(64),
        2f64.powi(65),
        1e300,
        -1e300,
        1e-300,
        0.1,
        1023.0,
        1024.0,
        -1074.0,
        -1075.0,
    ];
    for &x in &xs {
        for &y in &ys {
            v.push((x, y));
        }
    }
    for (x, y) in [
        (2.0, 1023.0),
        (2.0, 1024.0),
        (2.0, -1022.0),
        (2.0, -1074.0),
        (2.0, -1075.0),
        (2.0, 1023.9999999999999),
        (2.0, -1074.5),
        (10.0, 308.0),
        (10.0, 309.0),
        (10.0, -307.0),
        (10.0, -323.0),
        (10.0, -324.0),
        (10.0, 22.0),
        (10.0, 23.0),
        (10.0, -5.0),
        (1.0000001, 1e10),
        (0.9999999, 1e10),
        (1.0 + f64::EPSILON, 2f64.powi(62)),
        (1.0 - f64::EPSILON, 2f64.powi(62)),
        (1.0 + f64::EPSILON, 2f64.powi(40)),
        (1.0 - f64::EPSILON / 2.0, 2f64.powi(40)),
        (1.0 - f64::EPSILON / 2.0, -2f64.powi(40)),
        (-2.0, 3.0),
        (-2.0, 4.0),
        (-2.0, -3.0),
        (-2.0, -4.0),
        (-1.5, 2.5),
        (-8.0, 1.0 / 3.0),
        (-27.0, 1.0 / 3.0),
        (-2.0, 1e20),
        (-2.0, 2f64.powi(53) + 2.0),
        (-1.0000001, 2f64.powi(31) + 1.0),
        (-0.9999999, 2f64.powi(31) + 1.0),
        (-3.0, 2f64.powi(52) + 1.0),
        (-0.5, 1075.0),
        (-0.5, 1074.0),
        (-0.5, -1023.0),
        (-0.5, -1025.0),
        (1e-310, 0.5),
        (1e-310, 2.0),
        (1e-310, -0.5),
        (5e-324, -1.0),
        (f64::MAX, 0.5),
        (f64::MAX, -1.0),
        (4.0, 0.5),
        (-4.0, 0.5),
        (-0.0, 0.5),
        (2.0, 0.5),
        (1.0 / 3.0, 3.0),
        (125.0, 1.0 / 3.0),
        (1.0e3, 1.0 / 3.0),
        (2.0, 1.0 / 12.0),
        (1.5, 1e3),
        (1.5, -1e3),
        (7.0, 0.5 + f64::EPSILON),
        (0.0, 5e-324),
        (-0.0, -5e-324),
        (-inf, 5e-324),
        (-inf, -5e-324),
    ] {
        v.push((x, y));
    }
    // pow(integer, integer) must be exact when representable.
    for _ in 0..500 {
        let x = rng.int(-100, 100) as f64;
        let y = rng.int(-30, 30) as f64;
        v.push((x, y));
    }
    for _ in 0..600 {
        v.push((rng.exp_range(-20, 20), rng.uniform(-50.0, 50.0)));
    }
    for _ in 0..400 {
        v.push((rng.uniform(0.0, 2.0), rng.signed_exp_range(-10, 40)));
    }
    for _ in 0..300 {
        v.push((rng.exp_range(-1074, 1023), rng.uniform(-3.0, 3.0)));
    }
    for _ in 0..300 {
        let y = rng.int(-60, 60) as f64;
        v.push((-rng.exp_range(-10, 10), y));
    }
    for _ in 0..200 {
        // Close to 1 with huge exponents: the |y| > 2^31 branch.
        let x = 1.0 + rng.signed_exp_range(-52, -20);
        v.push((x, rng.signed_exp_range(31, 70)));
    }
    for _ in 0..300 {
        // Near the overflow and underflow thresholds.
        let x = rng.exp_range(1, 10);
        let target = rng.uniform(-1080.0, 1030.0);
        v.push((x, target / x.log2()));
    }
    for _ in 0..200 {
        // Integer-valued `y` of every size (the yisint test).
        let y = (rng.signed_exp_range(0, 60)).trunc();
        v.push((-rng.uniform(0.5, 1.5), y));
    }
    v
}

fn atan2_inputs(rng: &mut Rng) -> Vec<(f64, f64)> {
    let inf = f64::INFINITY;
    let vals = [
        0.0,
        -0.0,
        1.0,
        -1.0,
        2.0,
        -0.5,
        inf,
        -inf,
        f64::NAN,
        5e-324,
        -5e-324,
        1e300,
        -1e300,
        f64::MAX,
        1e-300,
        3.0,
    ];
    let mut v = Vec::new();
    for &y in &vals {
        for &x in &vals {
            v.push((y, x));
        }
    }
    for (y, x) in [
        (1.0, 2f64.powi(64)),
        (1.0, 2f64.powi(65)),
        (2f64.powi(64), 1.0),
        (2f64.powi(65), 1.0),
        (1.0, -2f64.powi(64)),
        (1.0, -2f64.powi(65)),
        (-1.0, -2f64.powi(70)),
        (1e-300, -1e300),
        (1e300, 1e-300),
    ] {
        v.push((y, x));
    }
    for _ in 0..800 {
        v.push((rng.uniform(-10.0, 10.0), rng.uniform(-10.0, 10.0)));
    }
    for _ in 0..600 {
        v.push((rng.signed_exp_range(-60, 60), rng.signed_exp_range(-60, 60)));
    }
    for _ in 0..300 {
        v.push((
            rng.signed_exp_range(-1074, 1023),
            rng.signed_exp_range(-1074, 1023),
        ));
    }
    for _ in 0..200 {
        let x = rng.signed_exp_range(-5, 5);
        v.push((x * rng.uniform(-1.0, 1.0) * 1e-17, x));
    }
    v
}

/// `(a, b, c)` triples for fma, including results that differ from the
/// unfused `a * b + c` and results that overflow or underflow.
fn fma_inputs(rng: &mut Rng) -> Vec<(f64, f64, f64)> {
    let inf = f64::INFINITY;
    let nan = f64::NAN;
    let eps = f64::EPSILON;
    let mut v = vec![
        (1.0 + eps, 1.0 - eps, -1.0),
        (0.1, 10.0, -1.0),
        (1.0 / 3.0, 3.0, -1.0),
        (f64::MAX, 2.0, -f64::MAX),
        (f64::MAX, 2.0, f64::MAX),
        (f64::MAX, f64::MAX, -inf),
        (f64::MAX, f64::MAX, inf),
        (f64::MAX, f64::MAX, nan),
        (f64::MAX, 1.0 + eps, 0.0),
        (f64::MAX, 1.0, f64::MAX / 2f64.powi(53)),
        (f64::MAX, 1.0, f64::MAX / 2f64.powi(54)),
        (-(1.0 - eps), f64::MIN, f64::MIN),
        (
            hexf(0x3cc0_0000_0000_0000),
            hexf(0x3cc0_0000_0000_0000),
            hexf(0x3cc0_0000_0000_0000),
        ),
        (
            -0.0000000000000002220446049250313,
            -0.0000000000000002220446049250313,
            -0.0000000000000002220446049250313,
        ),
        (-0.992, -0.992, -0.992),
        (1.1102230246251565e-16, -9.812526705433188e-305, 1.0894e-320),
        (
            2f64.powi(-1070),
            2f64.powi(-1070),
            hexf(0x000f_ffff_ffff_ffff) * 2.0,
        ),
        (2f64.powi(-1070), 2f64.powi(-1070), -f64::MIN_POSITIVE),
        (hexf(1), hexf(1), 0.0),
        (hexf(1), -hexf(1), 0.0),
        (-hexf(1), hexf(1), 0.0),
        (-hexf(1), -hexf(1), 0.0),
        (hexf(1), hexf(1), -0.0),
        (1e-200, 1e-200, 0.0),
        (1e-200, 1e-200, -0.0),
        (-1e-200, 1e-200, 0.0),
        (1e-200, 1e-200, hexf(1)),
        (1e-200, -1e-200, hexf(1)),
        (1e-160, 1e-160, -hexf(3)),
        (f64::MIN_POSITIVE, 0.5, 0.0),
        (f64::MIN_POSITIVE, 0.5, hexf(1)),
        (f64::MIN_POSITIVE, 1.0 - eps, 0.0),
        (f64::MIN_POSITIVE, 1.0 - eps / 2.0, hexf(1)),
        (f64::MIN_POSITIVE, 1.0 - eps / 2.0, -hexf(1)),
        (0.0, inf, 1.0),
        (inf, 0.0, 1.0),
        (inf, 1.0, -inf),
        (inf, -1.0, inf),
        (nan, 1.0, 1.0),
        (1.0, nan, 1.0),
        (1.0, 1.0, nan),
        (0.0, 0.0, 0.0),
        (0.0, 0.0, -0.0),
        (-0.0, 0.0, -0.0),
        (-0.0, 0.0, 0.0),
        (1.0, -1.0, 1.0),
        (2.0, 3.0, 4.0),
        (1.0, 1.0, 2f64.powi(-60)),
        (1.0, 1.0, 2f64.powi(-53)),
        (1.0, 1.0, 2f64.powi(-53) + 2f64.powi(-105)),
        (1.0, 1.0, -2f64.powi(-54)),
        (1.0 + eps, 1.0 + eps, -1.0 - 2.0 * eps),
        (1e308, 10.0, -1e308),
        (1e308, 10.0, -1.7e308),
        (1e-308, 1e-308, -1e-300),
        (3.0, 1e-320, 0.0),
    ];
    // Near ties decided only by bits more than 2^64 below `c`: the sticky bit
    // that generic/fma.rs drops (the FMA instruction and musl keep it).
    for k in [20, 24, 26, 27, 30, 35, 40] {
        let a = 1.0 + 2f64.powi(-k);
        let b = 1.0 - 2f64.powi(-k) + eps;
        for p in [-700, 0, 600] {
            let s = 2f64.powi(p);
            for m in [1.0, 1.0 + eps, 1.5, 2.0 - eps] {
                let c = m * 2f64.powi(53) * s;
                v.push((a * s, b, c));
                v.push((-a * s, b, -c));
                v.push((a * s, -b, 2.0 * c));
            }
        }
    }
    for _ in 0..600 {
        v.push((
            rng.signed_exp_range(-30, 30),
            rng.signed_exp_range(-30, 30),
            rng.signed_exp_range(-60, 60),
        ));
    }
    for _ in 0..600 {
        // Cancellation: c is close to -(a * b), so the fused result differs.
        let a = rng.signed_exp_range(-20, 20);
        let b = rng.signed_exp_range(-20, 20);
        let p = a * b;
        let c = -p * (1.0 + rng.int(-4, 4) as f64 * eps);
        v.push((a, b, c));
    }
    for _ in 0..300 {
        // Simpson-style loops: (i as f64).mul_add(h, a) and k.mul_add(f, sum).
        let i = rng.int(0, 10_000) as f64;
        v.push((i, rng.uniform(-1.0, 1.0) * 1e-3, rng.uniform(-10.0, 10.0)));
        v.push((
            if rng.next() & 1 == 1 { 2.0 } else { 4.0 },
            rng.uniform(-100.0, 100.0),
            rng.uniform(-1e4, 1e4),
        ));
    }
    for _ in 0..300 {
        // Subnormal and underflowing results.
        let a = rng.signed_exp_range(-560, -500);
        let b = rng.signed_exp_range(-560, -500);
        let c = rng.signed_exp_range(-1074, -1000);
        v.push((a, b, c));
    }
    for _ in 0..200 {
        let a = rng.signed_exp_range(-600, -480);
        let b = rng.signed_exp_range(-600, -480);
        let c = -(a * b) * (1.0 + rng.int(-2, 2) as f64 * eps);
        v.push((a, b, c));
    }
    for _ in 0..300 {
        // Overflow and near-overflow results.
        let a = rng.signed_exp_range(500, 520);
        let b = rng.signed_exp_range(495, 515);
        let c = if rng.next() & 1 == 1 {
            -(a * b)
        } else {
            rng.signed_exp_range(1000, 1023)
        };
        v.push((a, b, c));
    }
    for _ in 0..300 {
        v.push((rng.any_bits(), rng.any_bits(), rng.any_bits()));
    }
    v
}

fn unary(out: &mut String, name: &str, inputs: &[f64], lib: fn(f64) -> f64, std: fn(f64) -> f64) {
    writeln!(out, "    \"{name}\": [").unwrap();
    for (i, &x) in inputs.iter().enumerate() {
        let a = lib(black_box(x));
        let b = std(black_box(x));
        let comma = if i + 1 == inputs.len() { "" } else { "," };
        writeln!(out, "      [{}, {}, {}]{comma}", hex(x), hex(a), hex(b)).unwrap();
    }
    out.push_str("    ],\n");
}

fn binary(
    out: &mut String,
    name: &str,
    inputs: &[(f64, f64)],
    lib: fn(f64, f64) -> f64,
    std: fn(f64, f64) -> f64,
) {
    writeln!(out, "    \"{name}\": [").unwrap();
    for (i, &(x, y)) in inputs.iter().enumerate() {
        let a = lib(black_box(x), black_box(y));
        let b = std(black_box(x), black_box(y));
        let comma = if i + 1 == inputs.len() { "" } else { "," };
        writeln!(
            out,
            "      [{}, {}, {}, {}]{comma}",
            hex(x),
            hex(y),
            hex(a),
            hex(b)
        )
        .unwrap();
    }
    out.push_str("    ],\n");
}

fn main() {
    let mut rng = Rng(0x2540_be3f_1a2b_3c4d);
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"crate\": \"libm 0.2.16\",\n");
    out.push_str(
        "  \"format\": \"f64 bit patterns in hex; each vector is [args..., libm, std]; std is the Rust f64 method (glibc on x86_64 Linux)\",\n",
    );
    out.push_str("  \"functions\": {\n");

    let trig = trig_inputs(&mut rng);
    unary(&mut out, "sin", &trig, libm::sin, f64::sin);
    unary(&mut out, "cos", &trig, libm::cos, f64::cos);
    unary(&mut out, "tan", &trig, libm::tan, f64::tan);
    let inv = asin_acos_inputs(&mut rng);
    unary(&mut out, "asin", &inv, libm::asin, f64::asin);
    unary(&mut out, "acos", &inv, libm::acos, f64::acos);
    let at = atan_inputs(&mut rng);
    unary(&mut out, "atan", &at, libm::atan, f64::atan);
    binary(
        &mut out,
        "atan2",
        &atan2_inputs(&mut rng),
        libm::atan2,
        f64::atan2,
    );
    let hyp = hyperbolic_inputs(&mut rng);
    unary(&mut out, "sinh", &hyp, libm::sinh, f64::sinh);
    unary(&mut out, "cosh", &hyp, libm::cosh, f64::cosh);
    unary(&mut out, "tanh", &hyp, libm::tanh, f64::tanh);
    unary(&mut out, "exp", &exp_inputs(&mut rng), libm::exp, f64::exp);
    let logs = log_inputs(&mut rng);
    unary(&mut out, "log", &logs, libm::log, f64::ln);
    unary(&mut out, "log10", &logs, libm::log10, f64::log10);
    unary(&mut out, "log2", &logs, libm::log2, f64::log2);
    binary(&mut out, "pow", &pow_inputs(&mut rng), libm::pow, f64::powf);
    unary(
        &mut out,
        "cbrt",
        &cbrt_inputs(&mut rng),
        libm::cbrt,
        f64::cbrt,
    );
    unary(
        &mut out,
        "sqrt",
        &sqrt_inputs(&mut rng),
        libm::sqrt,
        f64::sqrt,
    );

    let fma = fma_inputs(&mut rng);
    out.push_str("    \"fma\": [\n");
    for (i, &(a, b, c)) in fma.iter().enumerate() {
        let r = libm::fma(black_box(a), black_box(b), black_box(c));
        let s = black_box(a).mul_add(black_box(b), black_box(c));
        let comma = if i + 1 == fma.len() { "" } else { "," };
        writeln!(
            out,
            "      [{}, {}, {}, {}, {}]{comma}",
            hex(a),
            hex(b),
            hex(c),
            hex(r),
            hex(s)
        )
        .unwrap();
    }
    out.push_str("    ]\n");
    out.push_str("  }\n}\n");
    print!("{out}");
}
