// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=1e7bb4eef731f4e1391242db1df2d21463fa0970976049d00c7955438faae699 bytes=870

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * The calculator's clock (src/clock.rs).
// |  *
// |  * Every reading of the current time goes through `now`, so fixing the
// |  * instant with `setFixedNow` makes `now`, `today`, dates without a year and
// |  * rate timestamps reproducible. The system clock has millisecond precision
// |  * (`Date.now()`), exactly like the wasm build, whose `Utc::now` reads
// |  * `js_sys::Date`.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=d77deb5240a5050012d2c0c9e19539da7c4a00652fb58ac29e71b17246476e17
// | import { Utc } from './rust/chrono.js';
use crate::rust::chrono::Utc;

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {import('./rust/chrono.js').DateTime} DateTime */

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** @type {DateTime | null} */
// | let fixedNow = null;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * The current instant (`DateTime<Utc>`), or the one fixed by `setFixedNow`.
// |  * @returns {DateTime}
// |  */
// | export function now() {
// |   return fixedNow ?? Utc.now();
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Fixes the instant `now` returns; `null` restores the system clock.
// |  * @param {DateTime | null} instant
// |  */
// | export function setFixedNow(instant) {
// |   fixedNow = instant;
// | }
