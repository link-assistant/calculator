---
bump: minor
---

### Added
- General dimensional unit algebra: values carry a dimension vector (length, mass, time, current, temperature, amount, luminosity, information, angle, currency) and an exact scale; `*`, `/`, powers and roots combine dimensions, addition and comparison require equal dimensions (`10m × 10m` = `100 m^2`, `(5 volts) / (2 ohms)` = `2.5 A`, `1 mph * 1 hour` = `1 mi`, `sqrt(2m)` is an error).
- A data-driven unit database in `data/units/` with SI base and derived units, all SI prefixes (including `µ`/`μ`/`u`), binary prefixes, Imperial/US, electrical, information, angle and temperature units, plus English and Russian aliases (`feet`, `metres`, `м`, `км`, `ч`).
- Unit expressions such as `m/s`, `m⋅s−1`, `m·s⁻¹`, `m s^-1`, `km per hour`, `kg·m²/s²` and `kg / (m^-2 s^2)`, as quantities and as conversion targets.
- Compound quantities `5 feet 10 inches`, `1 meter 20 cm` and `6'2"`.
- Affine temperatures: `20 celsius` is `293.15 K`, `25 °C in °F` is `77 °F`, and arithmetic on `°C`/`°F` is rejected as in Numbat.

### Changed
- `m` after a number is the metre and `min` the minute; `5m` reads as minutes only in an explicit time context (`5m in s`, `1h + 5m`, `now + 5m`).
- Products of compound units that were previously rejected (`5 kg * 9.8 m/s^2`) now evaluate, and `meters` is displayed as `m`.
