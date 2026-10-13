# Issue #254 requirement 2: generating Rust from `js/src` with meta-language

This directory records an experiment: translating the native JavaScript engine
(`js/src`) into Rust with
[link-foundation/meta-language](https://github.com/link-foundation/meta-language)
self-translation. It covers how the tool is invoked, what it produces for this
repository, whether the output compiles, and a proposal for the
regenerate-and-diff CI step and the allowlist. It also drafts the upstream
issues for constructs that no open meta-language issue covers. None of these
issues has been filed.

## 1. The tool

| Channel | Version | Has `selfTranslate`? |
|---|---|---|
| npm `meta-language` | 0.46.0 (the only published version, 2026-06-28) | **no** |
| crates.io `meta-language` | 1.0.0 (tag `v1.0.0`, commit `a79782093cae`, 2026-10-08) | yes, but without cross-module imports |
| GitHub `main` | `eb0574d5528babd3b4aadece81590d8e2230b99b` (2026-10-08 22:15), `js/package.json` 1.0.0, unreleased | yes, with `imports` / `moduleDirectory` / `selfTranslationSignatures` |
| GitHub releases | `v1.0.0`, Rust source only, no prebuilt binaries | n/a |

The JavaScript and Rust packages have the same API and CLI (`docs/self-translation.md`):

```js
import { selfTranslate, selfTranslationSignatures } from 'meta-language'; // js/src/self-translation.js
const { code, items } = selfTranslate(source, 'JavaScript', 'Rust', { moduleDirectory, imports, decorators });
// items: [{ term, start, end, status: 'translated'|'carried'|'comment'|..., reason }]
```

```sh
meta-language translate --to rust [--from js] [--items] module.js
```

Output format (one `.rs` per module):

- header `// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=… bytes=…`;
- `// meta-language:prelude begin … end` with `#![allow(...)]` and helper modules (`pub mod ml { … }`, `ml_array`, `ml_math`);
- for each top-level item, either
  `// meta-language:translated JavaScript <term> items=N sha256=…` + the source as `// |` lines + the Rust, or
  `// meta-language:carried JavaScript <term> (<reason>)` + the source as `// |` lines (no Rust).

The carried reason is only the kind: `unsupported`, `type`, `syntax`,
`top-level statement`, `no definition`, `comment the target cannot hold`.
The construct ("bitwise &: bitwise operators are outside the portable core at
34..35") is dropped (meta-language#214). formal-ai, and `census.mjs` here,
recover it by running `checkProgram(parseJavaScript(itemSource))` again on each
carried item.

**How formal-ai wires it into CI** (link-assistant/formal-ai; there is no
`link-foundation/formal-ai`):

- `scripts/translate-js-rust.mjs` has `--verify` (no upstream, seconds),
  `--check` (re-translate, byte-compare, optional `--compile`), `--write`
  (regenerate, refuses a regression) and `--why MODULE`.
- `.github/workflows/layered-ci.yml`, job `js-rust` (`needs: [changes, js]`):
  checks out `link-foundation/meta-language` at the pinned commit into
  `.meta-language`, runs `npm ci --prefix .meta-language/js --omit=dev --ignore-scripts`,
  then `node scripts/check-js-rust-translation.mjs`. No vendoring.
- `data/meta/js-rust-translation.lino`: the pin (`commit eb0574d5…`), totals,
  and one ratchet row per module: `module <path> translated N workaround N inferred N carried N`.
- `data/meta/translation-workarounds.lino`: one record per local workaround,
  `workaround <id>` / `lifts "<construct>"` / `upstream "<issue URL>"` / `scope` / `how`.
- `data/meta/translation-blockers.lino`: a census of the constructs per carried item.

## 2. Commands run

Untrusted sources were kept in `/tmp/ml-research/` and nothing from them was executed except the
library itself, imported by the drivers in this directory.

```sh
gh repo clone link-foundation/meta-language /tmp/ml-research/ml -- --depth 1
git -C /tmp/ml-research/ml fetch --depth 1 origin tag v1.0.0
# released tag and current main, each with runtime deps only and no install scripts
git -C /tmp/ml-research/ml archive v1.0.0 js | tar x -C /tmp/ml-research/pkg
git -C /tmp/ml-research/ml archive HEAD   js | tar x -C /tmp/ml-research/head
(cd /tmp/ml-research/pkg/js  && npm ci --ignore-scripts --omit=dev)
(cd /tmp/ml-research/head/js && npm ci --ignore-scripts --omit=dev)

E=experiments/issue-254/meta-language
node $E/run.mjs /tmp/ml-research/pkg/js/src  js/src $E/v1.0.0            # v1.0.0, no import binding
node $E/run.mjs /tmp/ml-research/head/js/src js/src $E/head --bind       # main, modules in import order,
                                                                         # each bound to its imports' signatures
node $E/census.mjs     /tmp/ml-research/head/js/src $E/head              # re-diagnoses every carried block -> census.json
node $E/categorize.mjs $E/head/census.json 3                             # groups diagnostics, maps to issues
node $E/assemble.mjs $E/head /tmp/ml-research/crate-head                 # one crate, one module per file
node $E/assemble.mjs $E/head /tmp/ml-research/crate-head-wa --workarounds
CALCULATOR_CARGO_TARGET_DIR=/tmp/ml-research/target \
  node scripts/bounded-cargo.mjs check --manifest-path /tmp/ml-research/crate-head/Cargo.toml --message-format short
CALCULATOR_CARGO_TARGET_DIR=/tmp/ml-research/target \
  node scripts/bounded-cargo.mjs check --manifest-path /tmp/ml-research/crate-head-wa/Cargo.toml --message-format short
node $E/repro.mjs  /tmp/ml-research/head/js/src > $E/repro-output.txt    # minimal examples of the drafted issues
node $E/rt-min.mjs /tmp/ml-research/head/js/src                          # round-trip defect
node $E/det.mjs    /tmp/ml-research/head/js/src js/src/rust/libm/log.js  # determinism
```

Toolchain: Node 22, cargo/rustc 1.99.0. The generated files are under
`v1.0.0/`, `head/` (43 modules, calculator commit `5a0b9849`) and `head-all/`
(every module at the commit in `head-all.commit`). The generated `.rs` files are not committed (see `.gitignore`; the commands
above regenerate them). Each has a committed `report.json`
with per-item status and reason; `head/census.json` has the diagnostic of
each carried block.

## 3. Results

### 3.1 Item counts (43 modules, `js/src` at `5a0b9849`)

| Translator | translated | carried | comment | errors |
|---|---:|---:|---:|---:|
| v1.0.0 (crates.io / tag) | 289 | 982 | 128 | 0 |
| main `eb0574d5` with import binding | **407** | **864** | 128 | 0 |

The numbers count items. A JSDoc comment travels with its item, so the 864
carried items make up 500 carried blocks.

Per module (main, translated / carried):

- **Mostly translated**: `rust/libm/pow` 52/2, `rust/libm/log` 30/10, `rust/libm/exp` 35/18,
  `rust/libm/atrig` 33/11, `rust/libm/trig` 32/10, `rust/libm/rem_pio2` 21/9,
  `rust/chrono/date` 20/7, `rust/chrono/datetime` 8/4, `rust/chrono/time` 6/4, and the
  `generated/*` data modules.
- **Mostly carried**: `rust/fmt` 0/23, `rust/core` 0/35, `rust/chrono/util` 0/15,
  `rust/libm/bits` 0/27, `rust/str` 5/116, `rust/decimal` 18/65, `rust/ratio` 18/62,
  `types/*` (classes) 1-7 translated each, `types/value/kind` 0/5.

Timing: 5-104 s per module, about 25 min for 43 modules on one thread
(meta-language#215: two lossless parses at about 5 KB/s). A CI check needs
worker threads or shards and a cache keyed by `(pin, sha256(source))`.

The translation is deterministic: two runs give identical bytes.

### 3.2 What keeps items out (main, 500 carried blocks, `categorize.mjs`)

| count | construct | upstream |
|---:|---|---|
| 91 | exported classes, `new`, custom `Error` subclasses, `throw new Panic(…)` | #210 |
| 78 | cascade: reads or calls a sibling or imported item that is itself carried (`unknown name crate.X`, `function value X`) | none of its own (lifts with the root cause) |
| 68 | `null`, `T \| null`, `undefined` | #204 |
| 47 | free-standing `/** … */` comments (module headers, `@typedef {import(…)}` lines) | **new: draft A** |
| 43 | JSDoc tuple, function, record, `unknown` types; heterogeneous array literals | #205 |
| 31 | string methods (`.padStart`, `.slice`, `.charCodeAt`, `.codePointAt`, `.replace`, `startsWith(s, i)`), string `.length` | #209 |
| 29 | bitwise `& \| ^ ~` and shifts `<< >> >>>` (Number and BigInt) | **new: draft D** |
| 18 | regex literals; lexer misreading `9a`, `9_` in regex bodies | #211 |
| 17 | JSDoc unions of non-null types: `number \| bigint`, `'Hours' \| 'Minutes'`, `-1 \| 0 \| 1`, `@typedef {A \| B} Pattern` | **new: draft B** |
| 16 | re-exports `export { a } from './m.js'`, `export * from` | **new: draft G** |
| 15 | string `switch`, `try`, `let x;`, array destructuring, destructured parameters | #212 |
| 14 | plain object records, frozen constant namespaces | #206 |
| 4 (+37 cascade) | arrow functions / closures as values | #207 |
| 8 | `**` exponentiation (`2n ** 63n - 1n`) | **new: draft E** |
| 7 | array methods (`.map`, `.join`) | #208 |
| 7 | `BigInt(x)`, `Number(x)`, `x.toString(2)` | **new: draft F** |
| 4 | module-level mutable state (`let CACHE = null` assigned in a function) | **new: draft H** |
| 3 | JSDoc `import('./m.js').T` types, `never`, `Iterable<T>` | **new: draft C** |
| 1 | parameter type taken from one body | #213 |

### 3.3 Does the generated Rust compile?

The 43 modules were assembled as one crate, one Rust module per JavaScript
module (`assemble.mjs`), and checked with `bounded-cargo.mjs check`:

- **As generated: 152 errors.** All are path errors, no type errors:
  - 51 `E0433 cannot find ml in the crate root`. Each module puts its helpers in
    its own prelude (`pub mod ml { … }`) but calls them as `crate::ml::Big`,
    `crate::ml_gcd_loop1` (meta-language#219).
  - about 100 `E0432 unresolved import`. Of 245 imported names only 3 are
    defined in the target module. The rest name items that the target carries
    (`use crate::rust::fmt::{debugStruct, debugTuple}`) (meta-language#220).
  - The translator calls `./str.js` and `./core.js` `str_` and `core_`, so the
    assembly has to name those module files the same way. This is a
    convention to follow, not a defect.
- **With the three formal-ai-style workarounds** (`--workarounds`: rename
  `str`/`core`, re-root `crate::ml*` at the module, drop unbound imported
  names): **`cargo check` passes with 0 errors and 0 warnings.** Everything the
  translator does emit type-checks.

### 3.4 Round trip

`docs/self-translation.md` says JavaScript → Rust → JavaScript restores the
source byte for byte. That fails for `rust/libm/log.js`. A comment on the same
line after an item (`const LN2_HI = 6.93e-1; /* 3fe62e42 fee00000 */`) is
moved to its own line before the next item, so the header hash no longer
matches. The minimal case is `const A = 1.5; /* note */\nconst B = 2.5;\n`
(`rt-min.mjs`). See draft I.

## 4. Proposal

### 4.1 `js/scripts/translate-rust.mjs`

Model it on formal-ai's `scripts/translate-js-rust.mjs`:

```
node js/scripts/translate-rust.mjs --write [--meta-language DIR] [MODULE...]
    regenerate rust-generated/<module>.rs for every module of js/src (or the named ones),
    in import order, each bound to its imports' selfTranslationSignatures;
    rewrite rust-generated/translation.lino; refuse a module whose translated count falls
node js/scripts/translate-rust.mjs --check [--meta-language DIR] [--compile]
    re-translate into a temp dir and byte-compare with the committed files and the ledger;
    every carried item must match an allowlist entry, and every allowlist entry must still
    match a carried item (no stale entries); with --compile run
    `node scripts/bounded-cargo.mjs check` on the assembled crate
node js/scripts/translate-rust.mjs --verify
    no upstream, seconds: each committed .rs header's sha256/bytes equals the current
    js/src module, and the CI pin equals the ledger's pin. Runs in the ordinary JS lint stage.
node js/scripts/translate-rust.mjs --why MODULE
    print each carried item with its full diagnostic
```

Design points:

- **Pin by commit.** The npm package is too old, and `main` has the import
  binding that the crate 1.0.0 lacks. Pin
  `link-foundation/meta-language@eb0574d5…` in the ledger. CI does
  `actions/checkout` of that ref into `.meta-language` and runs
  `npm ci --prefix .meta-language/js --omit=dev --ignore-scripts`, as
  formal-ai does. Switch to the npm package once a release has
  `selfTranslationSignatures`.
- **Output location.** Use `rust-generated/` (or `src/generated_from_js/`), not
  the hand-written `src/`. The current Rust in `src/` stays the reference
  implementation, and CI compares behaviour through the existing shared test
  vectors. The workaround layer (re-rooting, import pruning, module renaming)
  lives in `js/scripts/lib/` and applies only to the assembled `--compile`
  crate. The committed files stay byte for byte what upstream emits.
- **Cost.** About 25-40 min single-threaded for 64 modules. Use
  `worker_threads` (formal-ai uses 4) and skip unchanged modules whose
  `(pin, sha256(source), sha256(imported signatures))` matches the ledger row.
  A PR then translates only the modules it touched.
- **CI job.** A `js-rust` job with `needs: [js]`, path-filtered on `js/src/**`,
  the ledger and the script, running `--check --compile`. `--verify` goes in
  the JS stage.

### 4.2 Allowlist file format (`js/translation-allowlist.lino`)

The format is Links Notation, like formal-ai's ledgers. There is one record
per carried item. Each record is keyed by module and item name and names the
construct and the upstream issue. A record without an `upstream` issue is
rejected by `--check`.

```
# Items of js/src that meta-language cannot translate to Rust yet (issue #254 R2).
# Generated skeleton by `translate-rust.mjs --write`; `upstream` and `note` are edited by hand.
# --check fails on a carried item with no entry, an entry whose item now translates (stale),
# or an entry whose construct differs from the current diagnostic.
translation_allowlist
  pin eb0574d5528babd3b4aadece81590d8e2230b99b
item rust/libm/bits.js hf64
  construct "malformed number"
  upstream "https://github.com/link-foundation/meta-language/issues/211"
item rust/decimal.js partialDivide64
  construct "bitwise |"
  upstream "<draft D once filed>"
item types/unit.js massTable
  construct "assignment of MASS_TABLE"
  upstream "<draft H once filed>"
cascade rust/chrono/strftime.js D_FMT
  needs rust/chrono/strftime.js num0
```

`construct` is the diagnostic message with its span and quoted names removed
(the normalisation in `census.mjs`). `cascade` records list an item carried
only because a dependency is carried. They need no issue of their own and
disappear when the dependency translates.

## 5. Upstream issues

### 5.1 Already covered (comment with the calculator counts, do not refile)

| construct | issue | calculator count |
|---|---|---:|
| classes, `new`, Error subclasses, `throw new Custom` | #210 | 91 |
| null / undefined / `T\|null` | #204 | 68 |
| JSDoc tuple/function/record/unknown types, `[opt]` params, heterogeneous arrays | #205 | 43 |
| string methods, string length, `startsWith(s, i)` | #209 | 31 |
| regex literals, lexer `malformed number 9a` | #211 | 18 |
| string switch, try, `let x;`, destructuring, **destructured parameters** (add to #212) | #212 | 15 |
| object records, frozen namespaces with value properties | #206 | 14 |
| arrow functions / closures | #207 | 4 direct |
| array methods | #208 | 7 |
| parameter type from one body | #213 | 1 |
| crate-rooted helper paths (`crate::ml`) | #219 | 51 compile errors |
| imports naming untranslated items | #220 | about 100 compile errors |
| carried reason drops the diagnostic | #214 | all 500 blocks |
| two parses at about 5 KB/s | #215 | about 25 min for 43 modules |
| CI generate-and-diff upstream | #221 | n/a |

### 5.2 Drafted new issues (not filed)

Each draft below was reproduced with `repro.mjs` against main `eb0574d5`. The
output is in `repro-output.txt`.

---

#### Draft A: Self-translation carries free-standing `/** … */` comments into Rust as source, not as comments

**Construct.** A JSDoc block comment that is not directly before an item: a
module header, or a `/** @typedef {import('./m.js').T} T */` line. It is
carried with `comment the target cannot hold`.

```js
/**
 * Utility helpers.
 */

/** @param {number} x @returns {number} */
export function twice(x) { return x * 2; }
```

Items: `comment:carried(comment the target cannot hold) comment:translated export_statement:translated`.
In link-assistant/calculator this is 47 of 500 carried blocks.

**Why.** `/** */` is an outer doc comment in Rust, and an outer doc comment
followed by nothing (or by another comment) does not compile, so the
translator refuses it.

**Workaround.** Write module headers as `//` line comments, or ignore these
entries in the allowlist.

**Suggested fix.** Write a free-standing block comment as a plain block
comment `/* … */` (drop one `*`), or as `//` lines. Write the leading module
header as `//!` inner doc lines. Keep the original text as `// |` provenance
so the round trip still restores `/**`. Pure `@typedef {import(…)}` comments
can be dropped, because the type is bound through `imports`.

---

#### Draft B: Portable core: JSDoc unions of non-null types (`number | bigint`, string- and number-literal unions)

```js
/** @param {number | bigint} x @returns {number} */
export function toNumber(x) { return 0; }
/** @param {'Hours' | 'Minutes'} unit @returns {number} */
export function scale(unit) { return unit === 'Hours' ? 3600 : 60; }
/** @param {number} a @param {number} b @returns {-1 | 0 | 1} */
export function cmp(a, b) { return a < b ? -1 : a > b ? 1 : 0; }
```

Each is refused with `JSDoc type {…}: portable types are number, bigint,
boolean, string, arrays T[] of them and @typedef data types`. A union
`@typedef {string | ((c: string) => boolean) | string[]} Pattern` is refused
at its use as `unknown type crate.Pattern`. There are 17 blocks in the
calculator. #205 lists tuples, records, Map/Set and function types, and #204
lists `T | null`. Neither covers unions of non-null types or literal types.

**Workaround.** Widen to the base type (`{string}`, `{number}`) and validate
by hand. For `number | bigint`, split the function in two.

**Suggested fix.** (1) Map a literal union to its base type (`'a' | 'b'` →
`String` or a generated `enum` with `as_str`, and `-1 | 0 | 1` → `i32` or
`std::cmp::Ordering` when used as a comparison result). (2) Lower a
heterogeneous union to a generated `enum` with one variant per member, and
narrow it with `typeof` checks (overlaps #212's `typeof`).

---

#### Draft C: Portable core: JSDoc `import('./m.js').T` types, `never` and `Iterable<T>`

```js
/** @param {import('./m.js').Thing} t @returns {number} */
export function f(t) { return 1; }
/** @param {string} m @returns {never} */
export function fail(m) { throw new Error('unreachable'); }
```

The first gives `JSDoc type {import('./m.js').Thing}: portable types are …`.
The second gives `unknown type crate.never`. `Iterable<import('./strftime.js').Item>`
is refused the same way.

**Workaround.** Import the typedef with a named import and use the bare name.
Replace `never` with `void`.

**Suggested fix.** Resolve `import('<specifier>').Name` through the same
`imports` signatures as an ES import, which also needs signatures to carry
`@typedef` data types. Map `never` to `!` (or `-> !` for functions whose body
always throws). Map `Iterable<T>` to `impl IntoIterator<Item = T>` or `&[T]`.

---

#### Draft D: Portable core: bitwise and shift operators (`& | ^ ~ << >> >>>`) on numbers and BigInt

```js
/** @param {number} x @returns {number} */
export function low(x) { return x & 0xff; }
/** @param {number} x @returns {number} */
export function hi(x) { return x >>> 16; }
/** @param {bigint} x @returns {bigint} */
export function hi64(x) { return (x >> 32n) & 0xffffffffn; }
```

These give `bitwise &: bitwise operators are outside the portable core`,
`shift >>>: bit shifts are outside the portable core` and `shift >>: …`.
There are 29 blocks in the calculator, all in the `libm` and `decimal` ports.
These are the core of any numeric or binary-format port.

**Workaround.** None in JavaScript that keeps the code idiomatic. Arithmetic
replacements (`Math.floor(x / 65536)`) differ for negative numbers and are
slower.

**Suggested fix.** On `number`, lower with JavaScript's ToInt32/ToUint32
semantics: `((x as i64) as i32) & ((y as i64) as i32)` as f64, `>>>` via
`as u32`, and the shift count masked with `& 31`. Better, infer an integer
type when JSDoc says `@type {number}` and every use is bitwise. On `bigint`
(`crate::ml::Big`), implement two's-complement `& | ^ << >>` (num-bigint has
them). Add parity cases for negative operands and `>>> 0`.

---

#### Draft E: Portable core: the exponentiation operator `**`

```js
export const I64_MAX = 2n ** 63n - 1n;
/** @param {number} x @returns {number} */
export function sq(x) { return x ** 2; }
```

Both give `exponentiation: powers are outside the portable core; use recursion`.
There are 8 blocks in the calculator, and the integer bound constants cascade
into 30+ further carried items through `function value I64_MAX`.

**Workaround.** Write constants as literals (`9223372036854775807n`) and
powers as `Math.pow`, or `x * x`.

**Suggested fix.** `number ** number` → `f64::powf`. Check that JS and Rust
agree on the special cases: `NaN ** 0 = 1`, `1 ** Infinity = NaN` in JS but
`1.0f64.powf(INF) = 1` in Rust, so a helper is needed. `bigint ** bigint` →
`Big::pow(u32)` with a RangeError on a negative exponent. Fold constant
powers at translation time.

---

#### Draft F: Portable core: numeric conversions `BigInt(x)`, `Number(x)` and `x.toString(radix)`

```js
/** @param {number} x @returns {bigint} */
export function big(x) { return BigInt(x); }
/** @param {bigint} x @returns {number} */
export function num(x) { return Number(x); }
/** @param {bigint} x @returns {number} */
export function bits(x) { return x.toString(2).length; }
```

These give `BigInt: the JavaScript standard library is outside the portable core`,
`Number: …` and `toString with a radix: only decimal text is portable`.
There are 7 blocks in the calculator. The portable core has both `number` and
`bigint` but no way to convert between them.

**Workaround.** None inside the portable core.

**Suggested fix.** `BigInt(n)` for an integral f64 → `Big::from_f64_exact`,
throwing RangeError otherwise. `Number(b)` → `Big::to_f64` (round half to
even, as JavaScript does). `toString(r)` for 2 ≤ r ≤ 36 → a `to_str_radix`
helper, on both `bigint` and integral `number`.

---

#### Draft G: Self-translation: re-exports `export { a } from './m.js'` and `export * from './m.js'`

```js
export { twice } from './m.js';
export * from './m.js';
```

These give `export "{": only function and const declarations are exported`
and `export "*": …`. There are 16 blocks in the calculator. Every facade
module (`rust/chrono.js`, `rust/libm.js`) is made of them. Since main binds
`import { a } from './m.mjs'` to `use crate::m::{a}`, the matching re-export
is a small step.

**Workaround.** None. An `import` plus `export const a = …` alias changes the
API.

**Suggested fix.** `export { a, b as c } from './m.js'` → `pub use crate::m::{a, b as c};`
using the same specifier-to-path mapping and signatures as imports. Re-exported
names join the module's own signatures, so importers of the facade bind them.
`export * from` → `pub use crate::m::*;`, with the signatures being the union.

---

#### Draft H: Portable core: module-level mutable state (`let` at top level assigned inside a function)

```js
let cache = 0;
/** @returns {number} */
export function next() { cache = cache + 1; return cache; }
```

The `let` is carried as `top-level statement`, and the function gives
`assignment of cache: only local variables declared with let, and parameters,
are assignable`. There are 4 blocks in the calculator, all lazy tables
(`let MASS_TABLE = null; function massTable() { if (MASS_TABLE === null) MASS_TABLE = build(); return MASS_TABLE; }`)
and a test clock override.

**Workaround.** For a lazy table, compute it eagerly as a `const`. This
becomes a `LazyLock`, which upstream already emits for non-literal constants.

**Suggested fix.** Recognise the lazy-init pattern
(`let X = null; … if (X === null) X = e; return X;`) and lower it to
`static X: LazyLock<T> = LazyLock::new(|| e)`. Lower other top-level `let`s to
`thread_local! { static X: RefCell<T> }`, or refuse them with a specific
`module-level mutable state` diagnostic.

---

#### Draft I: Self-translation round trip moves a same-line trailing comment onto its own line

```js
const A = 1.5; /* note */
const B = 2.5;
```

JavaScript → Rust → JavaScript gives a new header plus
`const A = 1.5;\n/* note */\nconst B = 2.5;\n`, not the source. The same
happens with `// note`. `docs/self-translation.md` promises a byte-for-byte
restore for unedited output. In the calculator, `rust/libm/log.js` (FreeBSD
constant tables with hex comments) does not round-trip.

**Workaround.** Put the comment on the line above the item.

**Suggested fix.** When items are cut (`topLevelItems`), attach a comment that
starts on the same line as the previous item's end to that item: either keep
it in the item's `// |` provenance, or record the separator. Add a corpus case
with trailing `/* */` and `//` comments to `parity/self-translation/cases.lino`.

## 6. Full run over all 64 modules (calculator `44508454`)

`node run.mjs /tmp/ml-research/head/js/src js/src head-all --bind` with main `eb0574d5`:
**531 translated, 1230 carried, 128 comment** across 64 modules (output in `head-all/`).
The census, categorisation and compile check in sections 3.2-3.3 were run on the
43-module set only; run `census.mjs` and `categorize.mjs` on `head-all/` to break down
the 21 newer modules (math, dimensional, integral, datetime, number, equation grammars, lino, plan).
