/**
 * The calculator's clock (src/clock.rs).
 *
 * Every reading of the current time goes through `now`, so fixing the
 * instant with `setFixedNow` makes `now`, `today`, dates without a year and
 * rate timestamps reproducible. The system clock has millisecond precision
 * (`Date.now()`), exactly like the wasm build, whose `Utc::now` reads
 * `js_sys::Date`.
 */

import { Utc } from './rust/chrono.js';

/** @typedef {import('./rust/chrono.js').DateTime} DateTime */

/** @type {DateTime | null} */
let fixedNow = null;

/**
 * The current instant (`DateTime<Utc>`), or the one fixed by `setFixedNow`.
 * @returns {DateTime}
 */
export function now() {
  return fixedNow ?? Utc.now();
}

/**
 * Fixes the instant `now` returns; `null` restores the system clock.
 * @param {DateTime | null} instant
 */
export function setFixedNow(instant) {
  fixedNow = instant;
}
