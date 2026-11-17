# CI runs per pull request: baseline before the JavaScript-first split

Measured on 2026-10-13 with `node scripts/ci-runs-per-pr.mjs 5` over the last five merged pull requests. Before the split every CI/CD Pipeline run started Cargo. Rerun the script after the next five merged pull requests and append the table below for comparison (issue #254, requirement 4).

| PR | CI runs | Runs that reached Rust |
| --- | ---: | ---: |
| #259 | 5 | 5 |
| #257 | 14 | 14 |
| #255 | 1 | 1 |
| #253 | 6 | 6 |
| #252 | 1 | 1 |
