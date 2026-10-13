/**
 * Loads the JSON vectors that experiments/issue-254/rust-vectors prints, and
 * converts `f64` values to and from the bit patterns the vectors record.
 */

import { readFileSync } from 'node:fs';

/**
 * The parsed `js/test/vectors/<name>.json`.
 * @param {string} name
 * @returns {any}
 */
export function loadVectors(name) {
  return JSON.parse(readFileSync(new URL(`vectors/${name}.json`, import.meta.url), 'utf8'));
}

const view = new DataView(new ArrayBuffer(8));

/**
 * The 16 lowercase hex digits of the bits of `x` (`format!("{:016x}", x.to_bits())`).
 * @param {number} x
 */
export function f64Bits(x) {
  view.setFloat64(0, x);
  return view.getBigUint64(0).toString(16).padStart(16, '0');
}

/**
 * `f64::from_bits`.
 * @param {string} hex
 */
export function f64FromBits(hex) {
  view.setBigUint64(0, BigInt(`0x${hex}`));
  return view.getFloat64(0);
}

/**
 * Whether `actual` has the bits `expected`; every NaN matches every NaN,
 * since engines need not keep NaN payloads.
 * @param {number} actual
 * @param {string} expected
 */
export function sameF64(actual, expected) {
  const wanted = f64FromBits(expected);
  return Number.isNaN(wanted) ? Number.isNaN(actual) : f64Bits(actual) === expected;
}
