---
title: "Date and time parity: sign of clock differences, natural display, IANA/city zones, relative dates, workdays, timecode"
---
Part of #227.

## Evidence

| Expression | Source | Expected | Actual |
| --- | --- | --- | --- |
| `5pm - 7pm` | Soulver | 2 hours | **-2 hours** |
| `3am - 4pm` | Soulver | 13 hours | **-13 hours** |
| `days between 3 March and 30 May` | Soulver | 88 days | **-88 days** |
| `January 10 - February 5` | Soulver | 3 weeks 5 days | -26 days |
| `10 June + 3 weeks` | Soulver | 1 July | 2026-07-01 |
| `Feb 28 + 1 month` | Soulver | 28 March | 2026-03-28 |
| `2am PST to GMT` | Soulver | 10:00 am | 10:00:00 GMT |
| `00:12:05 − 00:04:09` | Soulver | 00:07:56 | 7 minutes, 56 seconds |
| `30 fps × 3 minutes` | Soulver | 5,400 frames | **90 FPS** |
| `yesterday - 8 weeks 3 days`, `Yesterday + 1 week` | Soulver | | `Unexpected identifier: yesterday` |
| `time in Paris`, `time in Tokyo`, `5pm ldn in sf` | Soulver/Raycast | | `Could not parse 'time'` |
| `monday in 3 weeks`, `next friday`, `workdays until 25 December` | Raycast/Soulver | | unsupported (`workdays` 2 rows) |
| `Invalid datetime format` | | | 24 rows |

The sign difference is the one that matters: Soulver, Numi and Raycast report the absolute span for "between"/"from-to" style phrases while keeping the sign for explicit subtraction of later from earlier dates; this project negates both.

## Acceptance criteria

- `between`, `from … to …`, `until`, and `a - b` with clock times follow the competitor convention documented in the Soulver dates page; a `--iso` flag or result metadata keeps the machine-readable form.
- Date results display in the input style (`1 July`, `12 February 2020`) with ISO available in the result object.
- IANA zones and city names (`Europe/London`, `Paris`, `Tokyo`, `ldn`, `sf`) via `chrono-tz` 0.10.4 plus a city table; `time in <city>`.
- Relative dates with a caller-supplied reference instant: `yesterday`, `tomorrow`, `next monday`, `monday in 3 weeks`, `3 weeks ago`, `in 2 months`; `workdays`/`weekdays` arithmetic with a configurable weekend.
- Lap times `00:12:05`, timecode `01:00:00:00 @ 25 fps`, `fps × duration = frames`.

## Solution options

1. Extend `src/grammar/datetime_grammar.rs` with a relative-date layer and display mode; add `chrono-tz` behind a feature flag to limit WASM size. Recommended.
2. `rust_dateparser` / `dateparser` crates. Reference only (see issue-220 audit): they do not preserve the AST.

## Blockers

- Blocked by: #229.
