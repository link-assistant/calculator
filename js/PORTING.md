# Porting Conventions

`js/` is a native JavaScript implementation of the calculator engine in
`src/` (issue #254). It runs on Node 20+, Bun and browsers without WASM, and
must return the same results as the Rust engine for every input:
`parity/goldens/*.json` records the Rust results of `parity/corpus.json`, and
`npm run parity` (in `js/`) compares the JavaScript engine against them.

The port is a **line-by-line translation**, not a rewrite. Every rule below is
mechanical, so the JavaScript name and shape of any Rust item can be predicted
from the Rust source without looking at its port. Keep the Rust structure,
order of definitions, comments and doc comments.

## Files

| Rust | JavaScript |
| --- | --- |
| `src/a/b.rs` | `js/src/a/b.js` |
| `src/a/mod.rs` | `js/src/a/index.js` |
| `#[path = "x.rs"] mod y;` | `js/src/<dir>/x.js` (file name wins) |
| `src/types/value/tests.rs`, `*_tests.rs`, `#[cfg(test)] mod tests` | `js/test/<same path>.test.js` (vitest) |
| `include_str!("../../data/x/y.lino")` | `js/src/generated/y-lino.js`, written by `js/scripts/generate-data.mjs` |
| `build.rs` (bundled rates) | `js/src/generated/bundled-rates.js`, same script |

- ES modules with explicit `.js` extensions in relative imports.
- `pub use` re-exports in `mod.rs` become `export { X } from './x.js'` in
  `index.js`. Import from the defining module where Rust uses a full path.
- Private items stay unexported, except where a test needs them; then export
  them and mark them `/** @internal */`.
- Rust emulation helpers live in `js/src/rust/`; never re-implement them inline.

## Names

- Types (structs, enums, traits) keep their Rust names: `Token`, `TokenKind`.
- Functions, methods, fields, locals: `snake_case` -> `camelCase`
  (`to_lino` -> `toLino`, `required_sources` -> `requiredSources`).
- Constants and statics keep `SCREAMING_CASE`.
- Enum variants keep their Rust names exactly (`BinaryOp.Add`, `TokenKind.Eof`).
- Associated functions become static methods with the same mechanical name,
  including `new`: `Token::new(a, b)` -> `Token.new(a, b)`.
- JSON (serde) field names are not renamed: `toJSON()` uses the Rust or
  `#[serde(rename...)]` names.

## Structs

```js
// pub struct Token { pub kind: TokenKind, pub start: usize, pub end: usize, pub text: String }
export class Token {
  /** @param {{ kind: TokenKind, start: number, end: number, text: string }} fields */
  constructor({ kind, start, end, text }) {
    this.kind = kind;
    this.start = start;
    this.end = end;
    this.text = text;
  }
}
// Token { kind, start, end, text }  ->  new Token({ kind, start, end, text })
```

- The constructor takes one object with every field (the struct literal), and
  assigns the fields in declaration order (this order is the JSON key order).
- Tuple structs take positional arguments stored as `_0`, `_1`, ...:
  `struct Decimal(RustDecimal)` -> `new Decimal(inner)`, `this._0`.
- Unit structs: a class with no fields.

## Enums

Every enum is one class; each value has a `tag` that is the exact variant name.

```js
export class TokenKind {
  /** @param {string} tag @param {object} [fields] */
  constructor(tag, fields) {
    this.tag = tag;
    Object.assign(this, fields);
    Object.freeze(this);
  }

  // Unit variant: a frozen singleton (compare with === or by tag).
  static Eof = new TokenKind('Eof');

  // Tuple variant: positional arguments -> _0, _1, ...
  static Number(value) { return new TokenKind('Number', { _0: value }); }

  // Struct variant: one object argument, field names in camelCase.
  static DateLiteral({ text, kind }) { return new TokenKind('DateLiteral', { text, kind }); }
}

switch (kind.tag) {
  case 'Number': {
    const value = kind._0;
    // ...
    break;
  }
  case 'Eof':
    // ...
    break;
  default:
    // the `_ =>` arm
}
```

- Freeze enum values and immutable structs (copy, do not mutate). Structs
  that Rust mutates through `&mut self` stay mutable.
- `matches!(x, A | B)` -> `x.tag === 'A' || x.tag === 'B'`.
- Methods from `impl Enum { ... }` are instance methods of the class.
- `#[derive(PartialEq)]` -> use `eq(a, b)` from `rust/core.js` (structural).
  Write an `equals(other)` method only for a manual `impl PartialEq`.
- `#[derive(Clone)]` + `.clone()` -> `clone(x)` from `rust/core.js` when the
  copy is mutated afterwards; immutable (frozen) values can be shared.
- `#[derive(Debug)]` -> a `fmtDebug()` method only when a value is formatted
  with `{:?}` in non-test code (error messages show it); it must print what
  Rust prints, e.g. `Token { kind: Eof, start: 5, end: 5, text: "" }`. Use
  the helpers of `rust/fmt.js`.
- `impl Display` -> `toString()`. `impl FromStr` -> `static fromStr(s)`.
  `impl Default` -> `static default()`. `impl From<T>` -> `static fromT(...)`
  named after `T` (`From<i64>` -> `fromI64`, `From<&str>` -> `fromStr`).
- Operator traits: `Add` -> `add(other)`, `Sub` -> `sub`, `Mul` -> `mul`,
  `Div` -> `div`, `Rem` -> `rem`, `Neg` -> `neg()`. `Ord`/`PartialOrd` ->
  `cmp(other)` returning -1, 0 or 1 (`partialCmp` returning `null` when
  unordered).
- `impl Serialize` (only where the JSON reaches the output) -> `toJSON()`
  returning a plain object with serde's field names and order; honour
  `skip_serializing_if` and `rename_all`.

## Option and Result

- `Option<T>` -> `T | null`. `None` is always `null`, never `undefined`.
  `Option<Option<T>>` is rare: use `{ some: value }` and document it.
- `Result<T, E>` -> return `T` or `throw new Err(e)` (`err(e)` from
  `rust/core.js`). `?` needs no code: the `Err` propagates.
- Handling a result: `tryResult(() => f())` gives `{ ok, value, error }`;
  `ok(() => f())` is `f().ok()`; `isOk(...)` is `f().is_ok()`.
- Never catch everything: catch only `Err` (`tryResult`/`ok` rethrow anything
  else), so JavaScript bugs and panics are not swallowed.
- `panic!`, `unwrap()` on `None`/`Err`, `expect`, `unreachable!` ->
  `panic(...)`, `unwrap(...)`, `expect(...)`, `unwrapResult(...)`,
  `unreachable(...)` from `rust/core.js`, with Rust's messages.
- Arithmetic that panics in Rust (`Decimal` overflow, integer division by
  zero) panics the same way in the emulation; do not add checks Rust lacks.

## Numbers

- `f64` -> `number`. Use `rust/num.js` for every `f64` method whose JavaScript
  counterpart differs: `round` (half away from zero), `powi`, `powf`,
  `rem_euclid`, `signum`, `clamp`, `x as i64` (saturating, NaN -> 0),
  `to_string` / `{}` / `{:?}` (`fmtF64`, `debugF64` in `rust/fmt.js`),
  `s.parse::<f64>()` (`parseF64`, Rust grammar, throws `Err`).
  `Math.pow` is not `powi`.
- Integers that cannot leave the safe range (indexes, lengths, small counts,
  `i32`, `u32`, `usize`) -> `number`; `i64`/`u64`/`i128`/`u128` whose values can
  exceed 2^53 or whose overflow is checked (`checked_*`, `wrapping_*`) ->
  `bigint`, or the helpers in `rust/num.js`. Integer `/` truncates:
  `Math.trunc(a / b)` for numbers.
- `rust_decimal::Decimal` -> `RustDecimal` (`rust/decimal.js`), an exact
  emulation of rust_decimal 1.43.0 (96-bit mantissa, scale <= 28, its rounding
  and its `Display`).
- `num_bigint::BigInt` -> `bigint`. `num_rational::Ratio<BigInt>` ->
  `Ratio` (`rust/ratio.js`), an emulation of num-rational 0.4.2.
- `chrono` -> `rust/chrono.js`, an emulation of the chrono 0.4.45 subset the
  crate uses (`NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime` with `Utc`
  or `FixedOffset`, `Duration` (`TimeDelta`), `Months`, `parse_from_str`,
  `format`, `to_rfc3339`).

## Strings

- `&str`/`String` -> `string`; `char` -> a one-code-point `string`.
- `.chars()` -> `[...s]` (code points, not UTF-16 units); `.chars().count()`
  -> `[...s].length`; `.len()` is the UTF-8 byte length (`utf8Len`).
- Byte offsets: Rust indexes strings by UTF-8 byte. Any position that reaches
  the output (token `start`/`end`, "at position N") must be the number Rust
  prints. Convert with `rust/str.js` (`utf8Len`, `byteToIndex`,
  `indexToByte`, `charIndices`). Positions used only to slice the same string
  may stay UTF-16 indexes, but never mix the two.
- Use `rust/str.js` instead of the JavaScript methods whose semantics differ:
  `trim`, `trimStart`, `trimEnd`, `splitWhitespace`, `isWhitespace`,
  `isAlphabetic`, `isNumeric`, `isAlphanumeric`, `isUppercase`,
  `isLowercase` (Rust uses the Unicode `White_Space`, `Alphabetic`, `N*`
  properties; JavaScript's `trim` also strips U+FEFF and keeps U+0085).
  `to_lowercase`/`to_uppercase` -> `toLowerCase()`/`toUpperCase()` (both are
  locale-independent full mappings). `to_ascii_lowercase` -> `toAsciiLowercase`.
- Regular expressions (`regex` crate): use the `u` flag and spell out the
  Unicode classes Rust implies: `\d` -> `\p{Nd}`, `\s` -> `\p{White_Space}`,
  `\w` -> `[\p{Alphabetic}\p{M}\p{Nd}\p{Pc}\p{Join_Control}]`, and `\b` is
  Unicode-aware in Rust (`(?<![\w])`/`(?![\w])` with that class).
- `format!` -> template literals. `{}` of an `f64` is `fmtF64(x)`, not
  `${x}`; `{:?}` of a `&str` is `debugStr(s)`; `{x:02}` is
  `String(x).padStart(2, '0')`; `{:02X}` is `x.toString(16).toUpperCase().padStart(2, '0')`.
- String ordering (`<` on `&str`, `sort()` of strings) is by code point:
  `compareStrings` from `rust/core.js`.

## Collections

- `Vec<T>` -> `Array`; `HashMap<K, V>`/`HashSet<K>` -> `Map`/`Set` (keys must be
  strings or numbers; build a string key for composite keys);
  `BTreeMap`/`BTreeSet` -> `Map`/`Set`, sorted with `cmp` when iterated.
- Iterator chains -> array methods. Do not use APIs newer than Node 20:
  iterator helpers (`.values().map(...)`), `Set` algebra
  (`union`, `intersection`), `Object.groupBy`, `Promise.withResolvers`,
  `Array.fromAsync`, `RegExp.escape`.
- `lazy_static!`/`OnceLock`/`thread_local!` -> a module-level variable
  initialised on first use.
- `Box<T>`, `Rc<T>`, `&T`, `&mut T` -> the value itself. When Rust moves or
  clones before mutating, copy (`clone`) before mutating in JavaScript.

## Types and Style

- JSDoc types on every exported function, method and constructor; the
  `npm run typecheck` (`tsc` with `checkJs`) must pass. Use
  `@param {import('../rust/decimal.js').RustDecimal} x` style imports.
- `npm run lint` (ESLint) must pass with no warnings.
- No dependencies at run time. Everything runs in Node 20+, Bun and browsers:
  no `node:` imports in `js/src/` (use `globalThis.fetch` for HTTP).
- Keep each file under 1000 lines, like the Rust sources (split along the
  same lines Rust does).

## Tests

- Port the Rust unit tests of a module to `js/test/<path>.test.js` with the
  same test names (`test_decimal_creation` -> `test('decimal creation', ...)`
  or the same name) and the same assertions.
- `js/test/vectors/*.json` hold values computed by the Rust crates the engine
  links (`experiments/issue-254/rust-vectors`); tests of `js/src/rust/` check
  every vector.
- `npm run parity` compares every case of `parity/goldens/` with the
  JavaScript engine. `PARITY_VERBOSE=1` prints every case,
  `PARITY_FILTER=<substring>` limits the inputs.
