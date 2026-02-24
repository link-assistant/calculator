---
bump: minor
---

### Added
- Data size unit conversions support: KB, MB, GB, TB, PB (SI decimal) and KiB, MiB, GiB, TiB, PiB (IEC binary), plus bit variants (b, Kb, Mb, etc.)
- `as` keyword for unit conversion syntax: `741 KB as MB`, `741 KB as mebibytes`, `741 KiB as MiB`
- Arithmetic with data size units: `(500 KB + 241 KB) as MB`
- Cross-standard conversions between SI and IEC systems (e.g., `1 GiB as GB`)
- Full-name unit support: `kilobytes`, `mebibytes`, `gibibytes`, etc.
- Case study for issue #55 at `docs/case-studies/issue-55/case-study.md`
