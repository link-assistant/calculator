// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=0f63939f5285c952686b12417b530c251fc6456f2fbf66bb54acb176777a55fd bytes=4304

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Rust `Display` and `Debug` formatting: `f64` (`{}` and `{:?}` differ from
// |  * JavaScript's `String(x)`), and the shapes `#[derive(Debug)]` prints.
// |  *
// |  * See js/PORTING.md.
// |  */

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * The shortest digits that round-trip to `x` (finite, non-zero) and the
// |  * decimal exponent `exp` such that `x = 0.d1d2... * 10^exp`, as Rust's
// |  * `flt2dec` produces them.
// |  * @param {number} x
// |  * @returns {{ digits: string, exp: number }}
// |  */
// | function shortestDigits(x) {
// |   const text = String(Math.abs(x));
// |   const [mantissa, exponent = '0'] = text.split('e');
// |   const [whole, fraction = ''] = mantissa.split('.');
// |   let digits = whole + fraction;
// |   let exp = whole.length + Number(exponent);
// |   const leading = /^0*/.exec(digits)[0].length;
// |   digits = digits.slice(leading);
// |   exp -= leading;
// |   return { digits: digits.replace(/0+$/, ''), exp };
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `flt2dec::digits_to_dec_str`.
// |  * @param {string} digits
// |  * @param {number} exp
// |  * @param {number} fracDigits the minimum number of fractional digits
// |  */
// | function digitsToDecStr(digits, exp, fracDigits) {
// |   if (exp <= 0) {
// |     const fraction = '0'.repeat(-exp) + digits;
// |     return `0.${fraction.padEnd(fracDigits, '0')}`;
// |   }
// |   if (exp < digits.length) {
// |     const fraction = digits.slice(exp);
// |     return `${digits.slice(0, exp)}.${fraction.padEnd(fracDigits, '0')}`;
// |   }
// |   const whole = digits + '0'.repeat(exp - digits.length);
// |   return fracDigits > 0 ? `${whole}.${'0'.repeat(fracDigits)}` : whole;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `flt2dec::digits_to_exp_str` with no minimum digit count, lowercase `e`.
// |  * @param {string} digits
// |  * @param {number} exp
// |  */
// | function digitsToExpStr(digits, exp) {
// |   const mantissa = digits.length > 1 ? `${digits[0]}.${digits.slice(1)}` : digits;
// |   return `${mantissa}e${exp - 1}`;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * The common part of `{}` and `{:?}`: sign, NaN and infinities.
// |  * @param {number} x
// |  * @param {(digits: string, exp: number) => string} finite
// |  * @param {string} zero
// |  */
// | function formatF64(x, finite, zero) {
// |   if (Number.isNaN(x)) {
// |     return 'NaN';
// |   }
// |   const sign = x < 0 || Object.is(x, -0) ? '-' : '';
// |   if (!Number.isFinite(x)) {
// |     return `${sign}inf`;
// |   }
// |   if (x === 0) {
// |     return sign + zero;
// |   }
// |   const { digits, exp } = shortestDigits(x);
// |   return sign + finite(digits, exp);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `{}` / `to_string()` of an `f64`: the shortest round-trip digits without an
// |  * exponent (`1e21` prints all its zeros, `1.0` prints `1`, `-0.0` prints
// |  * `-0`, infinities print `inf`).
// |  * @param {number} x
// |  */
// | export function fmtF64(x) {
// |   return formatF64(x, (digits, exp) => digitsToDecStr(digits, exp, 0), '0');
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `{:?}` of an `f64`: at least one fractional digit (`1.0`), and the
// |  * exponent form below 1e-4 and from 1e16 on (`1e16`, `1.5e-5`).
// |  * @param {number} x
// |  */
// | export function debugF64(x) {
// |   return formatF64(x, (digits, exp) => {
// |     const abs = Math.abs(x);
// |     if (abs < 1e-4 || abs >= 1e16) {
// |       return digitsToExpStr(digits, exp);
// |     }
// |     return digitsToDecStr(digits, exp, 1);
// |   }, '0.0');
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `{:?}` of a struct: `Name { a: 1, b: "x" }`, or `Name` with no fields.
// |  * The values are already formatted with `{:?}`.
// |  * @param {string} name
// |  * @param {Array<[string, string]>} fields
// |  */
// | export function debugStruct(name, fields) {
// |   if (fields.length === 0) {
// |     return name;
// |   }
// |   return `${name} { ${fields.map(([field, value]) => `${field}: ${value}`).join(', ')} }`;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `{:?}` of a tuple struct or tuple variant: `Name(1, "x")`.
// |  * @param {string} name
// |  * @param {string[]} values already formatted with `{:?}`
// |  */
// | export function debugTuple(name, values) {
// |   return values.length === 0 ? name : `${name}(${values.join(', ')})`;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `{:?}` of a slice or `Vec`: `[1, 2]`.
// |  * @param {string[]} values already formatted with `{:?}`
// |  */
// | export function debugList(values) {
// |   return `[${values.join(', ')}]`;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `{:?}` of an `Option<T>`.
// |  * @template T
// |  * @param {T | null} value
// |  * @param {(value: T) => string} debug the `{:?}` of `T`
// |  */
// | export function debugOption(value, debug) {
// |   return value === null ? 'None' : `Some(${debug(value)})`;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `{:?}` of a map: `{"a": 1}`.
// |  * @param {Array<[string, string]>} entries keys and values already formatted with `{:?}`
// |  */
// | export function debugMap(entries) {
// |   return `{${entries.map(([key, value]) => `${key}: ${value}`).join(', ')}}`;
// | }
