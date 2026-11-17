// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=cf120d42784e3627af76c2bb57a2a0196ad7ee8c1b1bb1dceb9748345c89b378 bytes=7546

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Rust language semantics the port relies on everywhere: `Result` errors,
// |  * panics, `Option` unwrapping, derived equality, ordering and cloning.
// |  *
// |  * See js/PORTING.md for how Rust constructs map onto these helpers.
// |  */

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * A Rust `Err(error)` travelling up the stack as an exception. `?` becomes a
// |  * plain call: the `Err` propagates by itself.
// |  *
// |  * It deliberately does not extend `Error`: throwing it captures no stack
// |  * trace, so parsers that try alternatives stay as cheap as in Rust.
// |  */
// | export class Err {
// |   /** @param {unknown} error the Rust error value */
// |   constructor(error) {
// |     this.error = error;
// |   }
// |
// |   toString() {
// |     return `Err(${String(this.error)})`;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** A Rust panic. Unlike `Err`, it is a bug, so it keeps its stack trace. */
// | export class Panic extends Error {
// |   /** @param {string} message */
// |   constructor(message) {
// |     super(message);
// |     this.name = 'Panic';
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `return Err(error)`.
// |  * @param {unknown} error
// |  * @returns {never}
// |  */
// | export function err(error) {
// |   throw new Err(error);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `panic!(message)`.
// |  * @param {string} message
// |  * @returns {never}
// |  */
// | export function panic(message) {
// |   throw new Panic(message);
// | }

// meta-language:carried JavaScript export_statement (syntax)
// | /**
// |  * `unreachable!()`.
// |  * @param {string} [message]
// |  * @returns {never}
// |  */
// | export function unreachable(message) {
// |   throw new Panic(message === undefined
// |     ? 'internal error: entered unreachable code'
// |     : `internal error: entered unreachable code: ${message}`);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `match f() { Ok(value) => .., Err(error) => .. }`: runs `fn` and returns
// |  * `{ ok: true, value }` or `{ ok: false, error }`. Anything thrown that is not
// |  * an `Err` (a panic, a JavaScript bug) is rethrown.
// |  * @template T
// |  * @param {() => T} fn
// |  * @returns {{ ok: true, value: T, error?: undefined } | { ok: false, error: any, value?: undefined }}
// |  */
// | export function tryResult(fn) {
// |   try {
// |     return { ok: true, value: fn() };
// |   } catch (thrown) {
// |     if (thrown instanceof Err) {
// |       return { ok: false, error: thrown.error };
// |     }
// |     throw thrown;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `f().ok()`: the value, or `null` when `fn` returns `Err`.
// |  * @template T
// |  * @param {() => T} fn
// |  * @returns {T | null}
// |  */
// | export function ok(fn) {
// |   try {
// |     return fn();
// |   } catch (thrown) {
// |     if (thrown instanceof Err) {
// |       return null;
// |     }
// |     throw thrown;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `f().is_ok()`.
// |  * @param {() => unknown} fn
// |  */
// | export function isOk(fn) {
// |   return tryResult(fn).ok;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `option.unwrap()`.
// |  * @template T
// |  * @param {T | null | undefined} value
// |  * @returns {T}
// |  */
// | export function unwrap(value) {
// |   if (value === null || value === undefined) {
// |     throw new Panic('called `Option::unwrap()` on a `None` value');
// |   }
// |   return value;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `option.expect(message)`.
// |  * @template T
// |  * @param {T | null | undefined} value
// |  * @param {string} message
// |  * @returns {T}
// |  */
// | export function expect(value, message) {
// |   if (value === null || value === undefined) {
// |     throw new Panic(message);
// |   }
// |   return value;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `{:?}` of an error value: its `fmtDebug()`, or its string form for errors
// |  * that have none.
// |  * @param {unknown} error
// |  */
// | function debugError(error) {
// |   const value = /** @type {{ fmtDebug?: () => string }} */ (error);
// |   return typeof value?.fmtDebug === 'function' ? value.fmtDebug() : String(error);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `result.unwrap()` for a function returning `Result`.
// |  * @template T
// |  * @param {() => T} fn
// |  * @returns {T}
// |  */
// | export function unwrapResult(fn) {
// |   return expectResult(fn, 'called `Result::unwrap()` on an `Err` value');
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `result.expect(message)` for a function returning `Result`: panics with
// |  * `message: {error:?}`.
// |  * @template T
// |  * @param {() => T} fn
// |  * @param {string} message
// |  * @returns {T}
// |  */
// | export function expectResult(fn, message) {
// |   try {
// |     return fn();
// |   } catch (thrown) {
// |     if (thrown instanceof Err) {
// |       throw new Panic(`${message}: ${debugError(thrown.error)}`);
// |     }
// |     throw thrown;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Derived `PartialEq`: structural equality. Objects with an `equals` method
// |  * decide themselves; arrays, maps and plain fields compare element-wise.
// |  * `NaN != NaN` as in Rust.
// |  * @param {unknown} a
// |  * @param {unknown} b
// |  * @returns {boolean}
// |  */
// | export function eq(a, b) {
// |   if (a === b) {
// |     return typeof a !== 'number' || !Number.isNaN(a);
// |   }
// |   if (a === null || b === null || typeof a !== 'object' || typeof b !== 'object') {
// |     return false;
// |   }
// |   if (typeof (/** @type {any} */ (a)).equals === 'function') {
// |     return (/** @type {any} */ (a)).equals(b);
// |   }
// |   if (Object.getPrototypeOf(a) !== Object.getPrototypeOf(b)) {
// |     return false;
// |   }
// |   if (Array.isArray(a)) {
// |     const other = /** @type {unknown[]} */ (b);
// |     return a.length === other.length && a.every((item, index) => eq(item, other[index]));
// |   }
// |   if (a instanceof Map) {
// |     const other = /** @type {Map<unknown, unknown>} */ (b);
// |     if (a.size !== other.size) {
// |       return false;
// |     }
// |     for (const [key, value] of a) {
// |       if (!other.has(key) || !eq(value, other.get(key))) {
// |         return false;
// |       }
// |     }
// |     return true;
// |   }
// |   if (a instanceof Set) {
// |     const other = /** @type {Set<unknown>} */ (b);
// |     return a.size === other.size && [...a].every((item) => other.has(item));
// |   }
// |   const keys = Object.keys(a);
// |   if (keys.length !== Object.keys(b).length) {
// |     return false;
// |   }
// |   return keys.every((key) => Object.hasOwn(b, key) && eq(a[key], b[key]));
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Derived `Clone`: a deep copy of mutable containers. Class instances keep
// |  * their prototype; frozen objects (unit enum variants, immutable numbers such
// |  * as `RustDecimal`) are shared, as they cannot change.
// |  * @template T
// |  * @param {T} value
// |  * @returns {T}
// |  */
// | export function clone(value) {
// |   if (value === null || typeof value !== 'object' || Object.isFrozen(value)) {
// |     return value;
// |   }
// |   if (typeof (/** @type {any} */ (value)).clone === 'function') {
// |     return (/** @type {any} */ (value)).clone();
// |   }
// |   if (Array.isArray(value)) {
// |     return /** @type {T} */ (/** @type {unknown} */ (value.map(clone)));
// |   }
// |   if (value instanceof Map) {
// |     return /** @type {T} */ (/** @type {unknown} */ (new Map([...value].map(([k, v]) => [k, clone(v)]))));
// |   }
// |   if (value instanceof Set) {
// |     return /** @type {T} */ (/** @type {unknown} */ (new Set(value)));
// |   }
// |   const copy = Object.create(Object.getPrototypeOf(value));
// |   for (const key of Object.keys(value)) {
// |     copy[key] = clone(value[key]);
// |   }
// |   return copy;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Ord::cmp` for numbers, bigints and strings (strings compare by UTF-8
// |  * bytes, which equals code point order), or for objects with a `cmp` method.
// |  * Returns -1, 0 or 1.
// |  * @param {any} a
// |  * @param {any} b
// |  * @returns {-1 | 0 | 1}
// |  */
// | export function cmp(a, b) {
// |   if (a !== null && typeof a === 'object' && typeof a.cmp === 'function') {
// |     return a.cmp(b);
// |   }
// |   if (typeof a === 'string') {
// |     return compareStrings(a, b);
// |   }
// |   if (a < b) {
// |     return -1;
// |   }
// |   return a > b ? 1 : 0;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Rust `str` ordering (by UTF-8 bytes, i.e. by code point). JavaScript's `<`
// |  * compares UTF-16 code units, which differs for astral characters.
// |  * @param {string} a
// |  * @param {string} b
// |  * @returns {-1 | 0 | 1}
// |  */
// | export function compareStrings(a, b) {
// |   const length = Math.min(a.length, b.length);
// |   for (let index = 0; index < length; index += 1) {
// |     const x = a.codePointAt(index);
// |     const y = b.codePointAt(index);
// |     if (x !== y) {
// |       return x < y ? -1 : 1;
// |     }
// |     if (x > 0xffff) {
// |       index += 1;
// |     }
// |   }
// |   if (a.length === b.length) {
// |     return 0;
// |   }
// |   return a.length < b.length ? -1 : 1;
// | }
