---
title: "Finance, statistics and rate phrases: VAT, interest, mortgages, mean/median, per-unit rates"
---
Part of #227. Soulver (money-and-finance pages), Numi, Calca and Raycast document these; fend ships dice statistics.

## Evidence

| Expression | Source | Expected | Actual |
| --- | --- | --- | --- |
| `VAT on $300`, `VAT off $300`, `VAT in $300` | Soulver | $45.00, $260.87, $39.13 | `Unexpected identifier: VAT` (7 rows) |
| `$100k at 5% interest for 3 years`, `monthly payment on $500k at 6% over 30 years` | Soulver | | `Unexpected identifier` (`interest` 2, `monthly` 3, `daily` 2) |
| `what will $1M be worth in 2040 assuming 10% inflation` | Soulver | | `Unexpected identifier: what` |
| `mean d6`, `median of …`, `random number between 1 and 10` | fend/Soulver | | `Unexpected identifier` (`mean` 3, `random` 2) |
| `$5 per hour × 40 hours`, `60 km per hour for 2.5 hours`, `30 fps × 3 minutes` | Soulver rates | $200, 150 km, 5400 frames | unsupported / `90 FPS` |
| `income tax on $85,000`, `midpoint of 10 and 20` | Soulver | | `Unexpected identifier` (`midpoint` 2) |

## Acceptance criteria

- Rate values (`per`, `/`, `a` as in `$5 an hour`) multiply and divide against durations and counts through the dimensional model.
- Tax phrases with a configurable rate (`VAT`, `GST`, `sales tax`, `income tax` with a bracket table), compound interest, loan/mortgage payments, future/present value, inflation with a CPI table (rows depending on live data keep an empty expectation).
- Statistics: `mean`, `median`, `mode`, `stddev`, `variance`, `sum`, `product` over lists; dice `d6`, `mean d6`; `random number between a and b`.

## Solution options

1. Implement as functions over the dimensional/percent value kinds with phrase aliases in LINO data. Recommended.
2. Reuse the finance formulas from Hurmet (MIT) as a reference implementation.

## Blockers

- Blocked by: #234 (percent kind), #235 (rates need dimensions), #237 (phrase grammar).
