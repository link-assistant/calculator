# Case Study: Issue #55 - Units Conversion Support (KB, KiB, MB, MiB)

## Summary

**Issue**: The calculator does not support data size units (bytes, kilobytes, megabytes, etc.) and conversions between them. Users want to convert between decimal (SI) and binary (IEC) data size units.

**Requested examples**:
- `741 KB as mebibytes` → convert 741 kilobytes (SI) to mebibytes (IEC)
- `741 KB as MB` → convert 741 kilobytes to megabytes (same SI system)
- `741 KiB as MiB` → convert 741 kibibytes to mebibytes (same IEC system)

## Background: The Two Standards Problem

### SI (Decimal) Standard — Powers of 1,000

The International System of Units (SI) defines data size prefixes as powers of 10:

| Unit | Symbol | Value (bytes) | Value (bits) |
|------|--------|---------------|--------------|
| byte | B | 1 | 8 |
| kilobyte | KB | 1,000 | 8,000 |
| megabyte | MB | 1,000,000 | 8,000,000 |
| gigabyte | GB | 1,000,000,000 | 8,000,000,000 |
| terabyte | TB | 1,000,000,000,000 | 8,000,000,000,000 |
| petabyte | PB | 1,000,000,000,000,000 | 8,000,000,000,000,000 |

### IEC (Binary) Standard — Powers of 1,024

The IEC 80000-13 standard (originally Amendment 2 to IEC 60027-2, published **January 29, 1999**) defines binary prefixes unambiguously:

| Unit | Symbol | Power | Value (bytes) |
|------|--------|-------|---------------|
| byte | B | 2^0 | 1 |
| kibibyte | KiB | 2^10 | 1,024 |
| mebibyte | MiB | 2^20 | 1,048,576 |
| gibibyte | GiB | 2^30 | 1,073,741,824 |
| tebibyte | TiB | 2^40 | 1,099,511,627,776 |
| pebibyte | PiB | 2^50 | 1,125,899,906,842,624 |

### Historical Context

The ambiguity started in the early days of computing:
- **1978**: Early floppy disks used "KB" to mean 1,024 bytes (informal usage)
- **1980s**: Commodore 64/128 computer naming used the binary convention
- **January 29, 1999**: IEC published the official binary prefix standard (KiB, MiB, GiB, etc.)
- **2005**: Standard extended with ZiB (zebibyte) and YiB (yobibyte) prefixes
- **Present**: Confusion still widespread in software, operating systems, and everyday use

### Divergence Grows with Scale

The practical difference between SI and IEC units grows at each magnitude:

| Scale | Binary unit | Decimal unit | Difference |
|-------|-------------|--------------|------------|
| ~1K | 1 KiB = 1,024 B | 1 KB = 1,000 B | +2.4% |
| ~1M | 1 MiB = 1,048,576 B | 1 MB = 1,000,000 B | +4.86% |
| ~1G | 1 GiB = 1,073,741,824 B | 1 GB = 1,000,000,000 B | +7.37% |
| ~1T | 1 TiB = 1,099,511,627,776 B | 1 TB = 1,000,000,000,000 B | +9.95% |
| ~1P | 1 PiB = 1,125,899,906,842,624 B | 1 PB = 1,000,000,000,000,000 B | +12.59% |

**Real-world example**: A "1 TB" HDD contains exactly 1,000,000,000,000 bytes (SI). When shown in Windows (which uses binary GiB but incorrectly labels them "GB"), it shows as ~931 GB. The actual size is 931 GiB ≈ 0.909 TiB.

## Conversion Formulas

### Within same system (simple division/multiplication)
```
Within SI:     KB → MB = divide by 1,000
Within IEC:   KiB → MiB = divide by 1,024
```

### Cross-system (convert through bytes)
```
KB → MiB:  (KB × 1,000) / 1,048,576
KiB → MB:  (KiB × 1,024) / 1,000,000
```

### Worked Examples: 741 Units

| Input | Process | Output |
|-------|---------|--------|
| 741 KB → MB | 741 × 1,000 / 1,000,000 | **0.741 MB** |
| 741 KB → MiB | 741 × 1,000 / 1,048,576 | **0.70686... MiB** |
| 741 KiB → MiB | 741 / 1,024 | **0.72363... MiB** |

## Common Usage Patterns

### Operating Systems
- **Linux `df -h`**: Uses IEC binary units (KiB, MiB, GiB) — correct
- **Linux `df -H`** (`--si`): Uses SI decimal units (KB, MB, GB)
- **Windows Explorer**: Displays "KB/MB/GB" labels but computes in binary (incorrect labeling)
- **macOS Finder**: Switched to SI decimal (powers of 1,000) to match drive manufacturer labels

### Storage Hardware
- **HDDs / SSDs / SD cards**: Always SI decimal (powers of 1,000) — manufacturer marketing convention
- **RAM / DRAM**: Binary IEC (powers of 1,024) — memory addressing is inherently binary
- **Optical media (DVD, Blu-ray)**: SI decimal

### Network Speeds
- Always SI decimal and always in bits (not bytes)
- 100 Mbps = 100,000,000 bits/s = 12.5 MB/s = ~11.92 MiB/s

## Existing Libraries and Tools

### Rust Crates
- `bytesize` (crates.io): Provides ByteSize type with IEC and SI display modes
- `humansize` (crates.io): Formats bytes into human-readable strings

### Online Calculators
- [gbmb.org](https://www.gbmb.org/kb-to-mib) - KB to MiB Conversion Calculator
- [convertlive.com](https://www.convertlive.com/u/convert/kilobytes/to/mebibytes)
- [dr-lex.be](https://www.dr-lex.be/info-stuff/bytecalc.html) - kB/MB/GB to KiB/MiB/GiB calculator

## Proposed Solution

### Data Size Unit Type

Add a new `DataSizeUnit` enum variant to the `Unit` enum in `src/types/unit.rs`:

```rust
/// Data size units (both decimal SI and binary IEC).
DataSize(DataSizeUnit)
```

With variants for all common units:
- **Binary (IEC)**: `Bit`, `Byte`, `Kibibyte`, `Mebibyte`, `Gibibyte`, `Tebibyte`, `Pebibyte`
- **Decimal (SI)**: `Kilobyte`, `Megabyte`, `Gigabyte`, `Terabyte`, `Petabyte`

### Conversion Logic

Convert through bytes (as canonical unit):
```
value_in_bytes = value * source_unit_in_bytes
result = value_in_bytes / target_unit_in_bytes
```

### `as` Keyword Syntax

Add support for the `as` keyword in expressions:
```
741 KB as MB           → 0.741 MB
741 KB as mebibytes    → 0.7069 MiB
741 KiB as MiB         → 0.7236 MiB
```

The `as` keyword requires a unit on the right-hand side (not a full expression) and performs unit conversion.

### Unit Recognition

Support both abbreviated and full-name forms:
- `B`, `bytes`, `byte`
- `KB`, `kilobytes`, `kilobyte`, `kB`
- `MB`, `megabytes`, `megabyte`
- `GB`, `gigabytes`, `gigabyte`
- `TB`, `terabytes`, `terabyte`
- `PB`, `petabytes`, `petabyte`
- `KiB`, `kibibytes`, `kibibyte`
- `MiB`, `mebibytes`, `mebibyte`
- `GiB`, `gibibytes`, `gibibyte`
- `TiB`, `tebibytes`, `tebibyte`
- `PiB`, `pebibytes`, `pebibyte`
- `b`, `bits`, `bit`
- `Kb`, `kilobits`, `kilobit`
- `Mb`, `megabits`, `megabit`
- `Gb`, `gigabits`, `gigabit`
- `Tb`, `terabits`, `terabit`
- `Pb`, `petabits`, `petabit`
- `Kib`, `kibibits`, `kibibit`
- `Mib`, `mebibits`, `mebibit`
- `Gib`, `gibibits`, `gibibit`
- `Tib`, `tebibits`, `tebibit`
- `Pib`, `pebibits`, `pebibit`

## Impact Assessment

### Affected Files
- `src/types/unit.rs` — Add `DataSizeUnit` enum
- `src/types/value.rs` — Add conversion logic for data size units
- `src/grammar/lexer.rs` — Add `As` token type
- `src/grammar/number_grammar.rs` — Recognize data size unit strings
- `src/grammar/token_parser.rs` — Handle `as` keyword for conversion
- `tests/integration_test.rs` — Add integration tests

### Risks
- Low: New feature, no changes to existing functionality
- The `as` keyword must not conflict with Rust identifier `as` (lexer handles this)
- Must handle case-sensitivity carefully (KB vs kb vs Kb)

## References

- [IEC 80000-13 Standard](https://www.iec.ch/prefixes-binary-multiples)
- [NIST Binary Prefix Definitions](https://physics.nist.gov/cuu/Units/binary.html)
- [Binary prefix - Wikipedia](https://en.wikipedia.org/wiki/Binary_prefix)
- [Kilobyte ambiguity - Wikipedia](https://en.wikipedia.org/wiki/Kilobyte)
- [Data-rate units - Wikipedia](https://en.wikipedia.org/wiki/Data-rate_units)
