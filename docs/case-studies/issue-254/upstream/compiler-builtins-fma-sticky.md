# libm: generic `fma` drops the sticky bit after shifting `x * y`, so `f64::mul_add` on wasm32 is not correctly rounded

Filed as https://github.com/rust-lang/compiler-builtins/issues/1353 on 2026-10-13.

## Summary

`libm::generic::fma_round` (behind `libm::fma`, `libm::fmaf128`, and the `fma`
symbol compiler-builtins provides on targets without a system libm, such as
`wasm32-unknown-unknown`) can return a result one ulp off. The bug appears when
`z` is so much larger than `x * y` that `z` fills the high word and `x * y` is
shifted right by `0 < d < 64` bits. The sticky bit is computed from `rlo`
*after* `rlo` has been overwritten, so the bits shifted out of the old `rlo`
are lost. If the remaining sum is an exact tie, it is then rounded to even
instead of away from the tie.

Found while porting libm 0.2.16 to JavaScript for
[link-assistant/calculator#254](https://github.com/link-assistant/calculator/issues/254).
The JavaScript port follows musl (which is correct here) and agrees with
glibc's `fma` on every recorded vector. libm does not.

## Reproduction

```toml
[dependencies]
# `arch` off: the generic implementation on every target.
libm = { version = "=0.2.16", default-features = false }
```

```rust
fn main() {
    let x = 1.0 + f64::powi(2.0, -26); // 0x1.0000004p+0
    let y = 1.0 - f64::powi(2.0, -26) + f64::EPSILON; // 0x1.ffffff8000002p-1
    let z = f64::powi(2.0, 53); // 0x1p+53
    // x * y is exactly 1 + 2^-78: just above the tie between 2^53 and 2^53 + 2.
    assert_eq!(x.mul_add(y, z), z + 2.0); // glibc / hardware FMA: ok
    assert_eq!(libm::fma(x, y, z), z + 2.0); // fails: 9007199254740992
}
```

Output on x86_64 Linux (rustc 1.99.0):

```
libm::fma (generic)  = 9007199254740992
f64::mul_add (std)   = 9007199254740994
```

The same inputs through std's `f64::mul_add` compiled for
`wasm32-unknown-unknown` (run in Node.js 26) also return `9007199254740992`,
because that target's `fma` comes from this code.

A fuzz test that puts `x * y = 1 + 2^-3a` (`x = 1 + 2^-a`,
`y = 1 - 2^-a + 2^-2a`, `a` in 18..=26) on a rounding tie of
`z = ±(2^53 + 2k)`, scaled by random powers of two, gets 277,634 of 1,000,000
results different from glibc. A million random dense inputs with `z` 2^54 to
2^127 above `x * y` show no difference, because a lost sticky bit only matters
on a tie, which is why random testing has not caught this.

Reproduction crates (generic `fma`, the fuzz test, and the wasm32 `mul_add`
check):
https://github.com/link-assistant/calculator/tree/ae4874347d5df91a42d2a4d97b042fcf2520a69c/experiments/issue-254/libm-fma-sticky
and
https://github.com/link-assistant/calculator/tree/ae4874347d5df91a42d2a4d97b042fcf2520a69c/experiments/issue-254/fma-wasm

## Cause

[`libm/src/math/generic/fma.rs`](https://github.com/rust-lang/compiler-builtins/blob/master/libm/src/math/generic/fma.rs),
unchanged on `master`:

```rust
} else if d < sbits {
    // Remaining shift fits within `sbits`. Leave `z` in place, shift `x * y`
    rlo = (rhi << (sbits - d)) | (rlo >> d);
    // Set the sticky bit
    rlo |= IntTy::<F>::from((rlo << (sbits - d)) != zero);
    rhi = rhi >> d;
}
```

The second statement reads the new `rlo`. Its low `sbits - d` bits are the
high bits of the old `rlo`, not the `d` bits that were shifted out. musl's
`fma.c`, which this was ported from, does it in one expression with the old
value:

```c
rlo = rhi<<(64-d) | rlo>>d | !!(rlo<<(64-d));
```

## Suggested fix

```diff
             } else if d < sbits {
                 // Remaining shift fits within `sbits`. Leave `z` in place, shift `x * y`
-                rlo = (rhi << (sbits - d)) | (rlo >> d);
-                // Set the sticky bit
-                rlo |= IntTy::<F>::from((rlo << (sbits - d)) != zero);
+                // Set the sticky bit from the bits of `rlo` shifted out
+                let sticky = IntTy::<F>::from((rlo << (sbits - d)) != zero);
+                rlo = (rhi << (sbits - d)) | (rlo >> d) | sticky;
                 rhi = rhi >> d;
```

With this change, the reproduction returns `2^53 + 2`, the fuzz test finds 0
differences in 2,000,000 inputs, and the existing `fma` unit tests of libm
0.2.16 still pass. A regression test for `src/math/fma.rs` that fails before
the change and passes after it:

```rust
#[test]
fn fma_sticky_after_shift() {
    // `x * y` is exactly 1 + 2^-78, just above the tie between 2^53 and 2^53 + 2.
    let x = hf64!("0x1.0000004p+0");
    let y = hf64!("0x1.ffffff8000002p-1");
    let z = hf64!("0x1p+53");
    let expected = hf64!("0x1.0000000000001p+53");
    assert_biteq!(generic::fma_round(x, y, z, Round::Nearest).val, expected);
    assert_biteq!(fma(x, y, z), expected);
}
```

`fmaf128` uses the same generic function, so it is probably affected too. I
have not tested it.

## Workaround

- Call `f64::mul_add` on targets whose system libm is correctly rounded
  (glibc, macOS, Windows), or where LLVM emits a hardware FMA instruction.
- On x86_64 and aarch64, keep libm's default `arch` feature. It uses the
  hardware instruction when the CPU has one (always on aarch64 with NEON; on
  x86_64, detected at run time), so only x86_64 CPUs without FMA or FMA4 reach
  the generic code.
- Downstream, link-assistant/calculator keeps `f64::mul_add` and does not call
  `libm::fma`.

## Duplicate check

I searched the compiler-builtins issues and pull requests for `fma`. #1262 and
#1270 cover f32 subnormal results from the widening `fmaf`, and the old
`rust-lang/libm` reports #200, #242 and #263 are closed and about other
inputs. I found nothing about this branch.
