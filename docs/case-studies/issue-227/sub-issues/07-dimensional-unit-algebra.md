---
title: "General dimensional unit algebra: length, temperature, electrical and compound units"
---
Part of #227. Largest single gap: `Unit` in `src/types/unit.rs` is a closed enum with only `Currency`, `Duration`, `DataSize`, `Mass`, `Timezone` and `Custom`. Every length, area, volume, temperature, electrical, energy or speed unit is either an undefined variable, a custom unit that silently drops the physics, or a clash with the `m` milli prefix / minute abbreviation.

## Evidence

| Expression | Source | Expected | Actual |
| --- | --- | --- | --- |
| `10m × 10m` | Soulver | 100 m² | **0.0001** |
| `4m + 0` | fend | 4 m | **0.004** |
| `2m == 200cm` | fend | true | **false** |
| `(5 volts) / (2 ohms)` | fend | 2.5 amperes | **2.5 VOLTS** |
| `2 amperes times 5 ohms` | theorem in words | 10 V | **10 amperes** |
| `1 mph * 1 hour` | Numbat | 1 mi | **1 MPH** |
| `1/0.5 kg` | fend | 2 kg | **2** |
| `2 kg^2` | fend | 2 kg^2 | **4** |
| `sqrt(2m)` | Rink | error: non-integer dimension | **0.0447** |
| `128 bit` | Rink | 16 byte | **128 b** |
| `20 celsius` | Numbat | 293.15 K | `20 celsius` (custom unit) |
| `25 °C in °F` | Numi | 77 °F | `Unexpected character '°'` |
| `5 µm in nm` | paper phrasing | 5000 nm | `Cannot convert µm to NM` |
| `1 ft to cm` | fend | 30.48 cm | `No exchange rate available` |
| `5 feet 10 inches` | Soulver | 177.8 cm | `Unexpected trailing input '10'` |
| `1 light year in miles` | Google | 5.879e12 mi | `Unexpected trailing input 'year'` |
| `299792458 m⋅s−1` | Wikipedia | 299792458 m/s | `undefined variable: m` |
| `1 J - 1 kg m^2 s^-2 + 1 kg / (m^-2 s^2)` | fend | 1 J | `Unexpected trailing input 'm'` |

Counts from the report: 30 rows fail with `undefined variable: m`, 62 with unit/conversion errors, 88 with "Expected a unit name after as/in/to", and all 210 Rink rows (0% coverage) and most of the 581 unsupported Qalculate! `units.batch` rows need this.

## Acceptance criteria

- A value carries a dimension vector (length, mass, time, current, temperature, amount, luminosity, information, angle, currency) and a scale; multiplication, division, powers and roots combine dimensions; addition requires equal dimensions.
- Affine units (°C, °F) convert correctly and reject `°C * 2` ambiguity the way Numbat does (document the choice).
- Unit database: at least the SI base and derived units with all SI prefixes (including `µ`/`μ` and `u`), imperial/US customary length, area, volume, mass, speed, pressure, energy, power, electrical units, information units with binary prefixes, angle units, and common natural aliases (`feet`, `foot`, `ft`, `metres`, `meters`, `м`, `км/ч`). Compound inputs `5 feet 10 inches`, `1 meter 20 cm`, `6'2"` are accepted.
- `m` after a number is metre; `min` is minute; `5m` as duration is only kept in an explicit time context (document and test the rule; fend and Numbat both pick metre).
- Unicode forms used in papers parse: `m⋅s−1`, `m·s⁻¹`, `m s^-1`, `km/h`, `kg·m²/s²`, `m²`, `m³`.
- Existing currency, duration, data-size and mass behaviour keeps every baseline row.

## Solution options

1. Replace the closed enum with a `Dimension` (array of eight rational exponents) plus a `UnitDef { name, aliases, dimension, factor, offset }` table loaded from a data file in `data/units/`. Follow Numbat's data model (its `modules/units/*.nbt` defines about 200 units with dimension types) and fend's `builtin.rs` list (about 1000 unit definitions) as references; GNU Units `definitions.units` (7431 lines in Rink's MPL-2.0 copy) is the most complete database but needs a license review before any data is generated from it. Recommended.
2. Depend on `uom` 0.38 (compile-time typed quantities). Rejected: dimensions must be dynamic at runtime for a calculator.
3. Embed `fend-core` 1.5.8 or `rink-core` 0.9 as a unit oracle. Useful as a test oracle in `experiments/`, but a second parser would bypass LINO output and the datetime grammar.

## Blockers

- Blocked by: #229 (gate), #233 (number lexing must be finished first because unit suffixes and thousands separators share the lexer rule for `1,000 km`).
