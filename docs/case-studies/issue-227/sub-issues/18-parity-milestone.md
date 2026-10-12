---
title: "Milestone: every competitor and scientific-quote corpus row is supported or has a documented difference"
---
Part of #227. Closing criterion for the parity vision.

## Definition of done

- `scripts/competitor-coverage.mjs` reports 0 `unsupported` and 0 `timeout` rows across all corpus files.
- Every remaining `different` row has a justification line in `docs/case-studies/issue-227/results/accepted-differences.tsv` (for example `log` base convention, `mod` sign convention, display precision) and the runner treats listed rows as accepted.
- The per-source table shows 100% supported-or-accepted for each of: fend, Qalculate!, Numbat, Rink, math.js, Insect, kalker, Hurmet, SpeedCrunch, Soulver, Numi, Calca, Parsify, Raycast, Wolfram|Alpha (documented examples), Google, Apple Math Notes, Frink; and for every language in the quotes corpus.
- The CI gate from #229 is tightened from "no regressions" to "no unsupported rows".

## Blockers

- Blocked by: #230, #231, #232, #233, #234, #235, #236, #237, #238, #239, #240, #241, #242, #243, #244, #245.
