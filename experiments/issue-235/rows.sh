#!/usr/bin/env bash
# Prints the calculator result for every issue #235 evidence row.
BIN=${BIN:-target/debug/link-calculator}
while IFS= read -r expr; do
  [ -z "$expr" ] && continue
  printf '%-45s => %s\n' "$expr" "$("$BIN" "$expr" 2>&1 | head -1)"
done <<'ROWS'
10m × 10m
4m + 0
2m == 200cm
(5 volts) / (2 ohms)
2 amperes times 5 ohms
1 mph * 1 hour
1/0.5 kg
2 kg^2
sqrt(2m)
128 bit
128 bit in byte
20 celsius
25 °C in °F
°C * 2
25 °C * 2
5 µm in nm
5 um in nm
5 μm in nm
1 ft to cm
5 feet 10 inches
5 feet 10 inches in cm
1 meter 20 cm
6'2"
6'2" in cm
1 light year in miles
299792458 m⋅s−1
299792458 m·s⁻¹
299792458 m s^-1
100 km/h
1 kg·m²/s²
5 m²
2 m³
1 J - 1 kg m^2 s^-2 + 1 kg / (m^-2 s^2)
5m
5 min
5m in s
2in
2 in to cm
5 км
1 ч
60 km per hour
sin(30°)
180 deg in rad
ROWS
