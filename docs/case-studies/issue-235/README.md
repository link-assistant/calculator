# General dimensional unit algebra (#235)

## Reproduction and cause

Before this change a unit was one of a few closed enums: currency, duration,
data size, mass, timezone or an opaque custom name. There was no notion of
a physical dimension. Unknown unit names became custom labels or currency
codes (`ft` → `FT`), and an adjacent `m` was the SI suffix milli, so `5m` was
`0.005`. The "Before" column below is the CLI output of `main` at `0e388a9d`.

| Input | Before | After |
| --- | --- | --- |
| `10m × 10m` | `0.0001` | `100 m^2` |
| `4m + 0` | `0.004` | `4 m` |
| `2m == 200cm` | `false` | `true` |
| `(5 volts) / (2 ohms)` | `2.5 VOLTS` | `2.5 A` |
| `2 amperes times 5 ohms` | `10 amperes` | `10 V` |
| `1 mph * 1 hour` | `1 MPH` | `1 mi` |
| `1/0.5 kg` | `2` | `2 kg` |
| `2 kg^2` | `4` | `2 kg^2` |
| `sqrt(2m)` | `0.04472135954999579` | Error: non-integer dimension |
| `128 bit` | `128 b` | `128 bit` (`in byte` → `16 B`) |
| `20 celsius` | `20 celsius` | `293.15 K` |
| `25 °C in °F` | Parse error at `°` | `77 °F` |
| `5 µm in nm` | Cannot convert µm to NM | `5000 nm` |
| `1 ft to cm` | No exchange rate FT → CM | `30.48 cm` |
| `5 feet 10 inches` | Parse error | `5.83333333333333 ft` (`in cm` → `177.8 cm`) |
| `1 light year in miles` | Parse error | `5878625373183.61 mi` |
| `299792458 m⋅s−1` | Undefined variable `m` | `299792458 m/s` |
| `1 J - 1 kg m^2 s^-2 + 1 kg / (m^-2 s^2)` | Parse error | `1 J` |

Every row is asserted in
[`issue_235_dimensional_units_tests.rs`](../../../tests/issue_235_dimensional_units_tests.rs).
[`rows.sh`](../../../experiments/issue-235/rows.sh) prints them from the CLI,
and [`dimensional_units.rs`](../../../examples/dimensional_units.rs) runs them
through the library API.

## Design

- **`Dimension`** ([`dimension.rs`](../../../src/types/dimension.rs)) is an
  integer exponent vector over ten base dimensions: length, mass, time,
  current, temperature, amount, luminosity, information, angle and currency.
  Information and angle are base dimensions, so `bit` and `rad` cannot be
  silently cancelled the way they are in pure SI.
- **`UnitDef`** ([`unit_db.rs`](../../../src/types/unit_db.rs)) is loaded at
  startup from [`data/units/units.lino`](../../../data/units/units.lino).
  Each unit has a dimension, an exact rational factor to the coherent SI unit,
  an optional affine offset, its aliases and its allowed prefixes (`si`,
  `si-binary` or an explicit list). Adding a unit is a one-line data change.
- **`PhysicalUnit`** ([`physical.rs`](../../../src/types/physical.rs)) is a
  product of `(symbol, power)` terms, displayed as `kg m^2/s^2`. Simplifying a
  product does three things: it merges equal dimensions (`cm·m` → `cm^2`);
  it expands compound units when that shortens the product (`mph·h` → `mi`);
  and it collapses coherent or SI-prefixed products into a named derived unit
  (`V/Ω` → `A`, `mA·kΩ` → `V`).
- Arithmetic lives in
  [`value/physical.rs`](../../../src/types/value/physical.rs):
  - `*` and `/` add or subtract dimension vectors.
  - Integer powers multiply them.
  - Roots divide them, and must give integer exponents, so `sqrt(2m)` is an
    error.
  - `+`, `-` and comparisons require equal dimensions and convert exactly
    through the rational factors.
  - An exact zero has every dimension (`4m + 0`).
- The existing duration, data-size and mass variants remain unchanged. They
  take part in the algebra through `PhysicalUnit::from_unit`, and single-term
  results convert back (`into_unit`). This leaves currency, duration, data-size
  and mass behaviour unchanged.

`uom` was rejected as the issue requires: its dimensions are fixed at compile
time and cannot express data-driven units. No fend or rink code is used; the
expected values in the issue come from those tools and are reproduced by the
tests.

## Grammar

[`quantities.rs`](../../../src/grammar/token_parser/quantities.rs) parses the
unit after a number and the target of `in`/`to`/`as` as a unit expression:

- **Factors** are joined by a space, `*`, `·`, `⋅`, `×`, `/` or `per`. A
  parenthesized group may follow `/`.
- **Powers** are written `^n`, `^-n`, `^(n)`, as superscripts (`²`, `⁻¹`), or
  as an adjacent Unicode minus (`s−1`). They are bounded by ±1000.
- **Only dimensional units take part**, so `5 USD * 2` stays arithmetic.
- **Two-word names** such as `light year` resolve through the database.
- **Compound quantities:** a quantity followed by further quantities of the
  same dimension is their sum: `5 feet 10 inches`, `1 meter 20 cm`, `6'2"`,
  `1 h 30 min`. The sum is displayed in the first unit.
- **Typographic marks:** the lexer reads `°C`, `°F`, `℃`, `℉`, `'`, `′`, `"` and
  `″` as unit symbols. An apostrophe still groups digits when exactly three
  digits follow (`1'000`), so `6'2"` is feet and inches.
- **`in` as inch:** `in` is the inch only when the input, a group or a
  conversion follows it (`2in`, `2 in to cm`). Otherwise it remains the
  conversion keyword.
- **Division by a quantity:** `a / b unit` with a plain `a` reads as
  `(a / b) unit`, giving `1/0.5 kg` = `2 kg` as the issue expects.

## Decisions

### Affine temperatures (the Numbat rule)

`°C` and `°F` carry an offset. A temperature result is shown in kelvin
(`20 celsius` → `293.15 K`) unless it was explicitly converted
(`25 °C in °F` → `77 °F`, `0 K in °C` → `-273.15 °C`). Inside an expression a
`°C` value keeps its offset, so multiplication, division, powers, addition and
subtraction are rejected:

```text
25 °C * 2  → Domain error: cannot multiply 25 °C: affine temperature units
             only convert; convert to K first (e.g. `25 °C in K`)
```

This follows Numbat, which disallows arithmetic on affine units because
`2 × 25 °C` has no single physical meaning. Comparisons convert through
kelvin, so `25 °C == 77 °F` is `true`. A bare `°C * 2` without a number is a
parse error.

### `m` versus `min`

`m` after a number is the metre, and `min` is the minute (`5m` → `5 m`,
`5 min` → `5 minutes`). `5m` is a duration only in an explicit time context:

- it is converted to a duration (`5m in s` → `300 seconds`, `5m in minutes`);
- it is added to or subtracted from a duration or a date (`1h + 5m`,
  `now + 5m`).

Without one of these contexts, `2 km + 5m` is `2.005 km` and `5m + 2m` is
`7 m`. Of the SI suffixes only `m` is a unit when written adjacent to a
number, so `5T` is still `5×10¹²` and `500m USD` is `0.5 USD`. `min` and `sec`
are units after a number unless they are called: `2 min(3, 4)` is still `6`.

### Symbol case

Unit symbols are case-sensitive. Uppercase `W`, `H` and `S` are watt, henry
and siemens, not week, hour and second. An all-uppercase well-known currency
code (`PI`, `SEK`) remains a currency, so `2 pi` is still `2π`.

## Intentional compatibility changes and deviations

- Some products of compound units used to be rejected by
  `issue_233_number_separator_tests` and `issue_234_percent_arithmetic_tests`.
  They now evaluate: `5 kg * 9.8 m/s^2` → `49 kg m/s^2`, and
  `20% of (10 km/h)^2` → `20 km^2/h^2`. `sqrt(9 km/h)` is still rejected.
- An adjacent `m` used to be the SI suffix milli (`5m` → `0.005`). As the
  issue requires, it is now the metre. Before another unit it is still milli
  (`500m USD` → `0.5 USD`), and every `issue_162_si_suffix_tests` case passes
  unchanged.
- A plain number added to a quantity takes the quantity's unit
  (`2 m + 3` → `5 m`), as it already does for currencies, durations and mass
  (`5 USD + 3` → `8 USD`). Numbat rejects this. Two quantities of different
  dimensions are always rejected (`1 kg + 1 m`).
- `10 meters + 20%` is displayed as `12 m` instead of the custom-unit spelling
  `12 meters`.
- `128 bit` keeps the unit the user wrote. Rink prints `16 byte`; here
  `128 bit in byte` gives `16 B`.
- `5 feet 10 inches` is shown in the first unit (`5.83333333333333 ft`).
  `5 feet 10 inches in cm` gives the issue's `177.8 cm`.
- `1 light year in miles` is `5878625373183.61 mi`, the exact
  9460730472580800 m divided by 1609.344 m. This is the issue's `5.879e12 mi`
  without the rounding.

## Competitor coverage

The [issue #227 corpus](../issue-227/results/coverage-report.md) went from
1856 to 2010 supported expressions out of 13975. Numbat went from 72 to 102,
fend from 391 to 423 and Insect from 31 to 37. The baseline was regenerated.
Six earlier passes no longer pass:

| Source | Input | Now | Why |
| --- | --- | --- | --- |
| fend | `cbrt (1kg)` | non-integer dimension error | Required: roots must give integer dimensions. fend expects `1 kg^(1/3)`. |
| Rink | `sqrt(2m)` | non-integer dimension error | Rink also rejects it, but the corpus counts every error as unsupported. |
| fend | `4 Metres`, `4 mEtRes` | `4 m` | The unit is recognized and shown by its symbol. fend expects `4 metres`. |
| Hurmet | `2 L = ?`, `L = 3.1 m` | equation error | `L` is now the litre, not a free variable. These rows have no expected value; on `main` they printed the symbolic `? = 2*L` and `L = 3.1*m`. |

## Verification

```bash
cargo test --test issue_235_dimensional_units_tests
cargo run --example dimensional_units
bash experiments/issue-235/rows.sh
cargo build --release
node scripts/competitor-coverage.mjs --binary experiments/issue-227/bounded-calculator.sh \
  --baseline docs/case-studies/issue-227/results/coverage-baseline.json
```
