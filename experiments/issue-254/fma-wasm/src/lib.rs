//! Exports `f64::mul_add` so `run.mjs` can call it from a wasm32 build.

/// `x.mul_add(y, z)` as std computes it on this target.
#[no_mangle]
pub extern "C" fn mul_add(x: f64, y: f64, z: f64) -> f64 {
    x.mul_add(y, z)
}
