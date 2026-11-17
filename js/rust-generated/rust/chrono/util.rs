// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=7737cddd65648e941d28a3365386b7bb971efa0eff57e644eab4782f466f3062 bytes=1749

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Small helpers shared by the chrono modules.
// |  */

// meta-language:carried JavaScript export_statement (unsupported)
// | /** `u32::MAX`. */
// | export const U32_MAX = 2 ** 32 - 1;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Whether `value` is an integer in `min..=max`.
// |  * @param {unknown} value
// |  * @param {number} min
// |  * @param {number} max
// |  * @returns {value is number}
// |  */
// | export function inRange(value, min, max) {
// |   return typeof value === 'number' && Number.isInteger(value) && value >= min && value <= max;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Whether `value` is a `u32`.
// |  * @param {unknown} value
// |  * @returns {value is number}
// |  */
// | export function isU32(value) {
// |   return inRange(value, 0, U32_MAX);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `div_mod_floor` (`div_euclid`, `rem_euclid`) for a positive divisor.
// |  * @param {number} value
// |  * @param {number} divisor
// |  * @returns {[number, number]}
// |  */
// | export function divModFloor(value, divisor) {
// |   const div = Math.floor(value / divisor);
// |   return [div, value - div * divisor];
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Two zero-padded digits (`write_hundreds`).
// |  * @param {number} value
// |  */
// | export function two(value) {
// |   return String(value).padStart(2, '0');
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * The fractional part `Debug` prints: nothing, `.mmm`, `.uuuuuu` or `.nnnnnnnnn`.
// |  * @param {number} nano
// |  */
// | export function autoSiFraction(nano) {
// |   if (nano === 0) {
// |     return '';
// |   }
// |   if (nano % 1_000_000 === 0) {
// |     return `.${String(nano / 1_000_000).padStart(3, '0')}`;
// |   }
// |   if (nano % 1000 === 0) {
// |     return `.${String(nano / 1000).padStart(6, '0')}`;
// |   }
// |   return `.${String(nano).padStart(9, '0')}`;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * A year as `Debug` prints it: four digits in `0..=9999`, else `{:+05}`.
// |  * @param {number} year
// |  */
// | export function debugYear(year) {
// |   if (year >= 0 && year <= 9999) {
// |     return String(year).padStart(4, '0');
// |   }
// |   return `${year < 0 ? '-' : '+'}${String(Math.abs(year)).padStart(4, '0')}`;
// | }
