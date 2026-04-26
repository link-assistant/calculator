# Issue 140 Case Study

## Summary

Issue #140 reported that this input failed:

```text
22822 рублей в рупиях на 11 апреля 2026
```

The parser recognized the expression, but historical currency conversion required an exact
RUB/INR rate for the requested date. When a rate file omits a non-business date, the calculator
returned `No exchange rate available for RUB/INR on 2026-04-11` instead of using the latest prior
available rate.

## Collected Artifacts

- `issue.json`: issue title, body, timestamps, and inline GitHub issue metadata.
- `issue-comments.json`: issue comments fetched via the GitHub API.
- `pr-141.json`: existing PR metadata before the fix.
- `ci-runs.json`: recent CI runs for `issue-140-4a0789606d78`.
- `cbr-xml-api.html`: CBR XML API documentation snapshot.
- `cbr-xml-daily-2026-04-11.xml`: CBR XML response for `date_req=11/04/2026`.

## Timeline

- 2026-04-26 17:58 UTC: Issue #140 was opened with the failing Russian RUB to INR expression.
- 2026-04-26 17:59 UTC: Draft PR #141 was created for branch `issue-140-4a0789606d78`.
- Investigation found existing issue #138 coverage for exact dated CBR rates, but no regression
  test for missing weekend or holiday rows.

## Requirements

- Preserve historical-date conversion for Russian currency expressions.
- Use the requested date when an exact historical rate exists.
- If an exact historical rate is missing, use the latest available rate before the requested date.
- Do not use a future rate for dates before the first known historical rate.
- Do not silently fall back to current/default rates for dated conversions.
- Keep rate metadata in calculation steps so users can see which effective date was used.

## Root Cause

`CurrencyDatabase::convert_at_date` only looked up `(from, to, exact_date)` in
`historical_rates`. If that key was missing, it returned `NoHistoricalRate`.

This is too strict for exchange-rate datasets where official APIs publish rates only for business
days or effective dates. A dated conversion on a weekend or holiday should use the latest prior
available rate for that currency pair.

## External Data Notes

The CBR XML response saved in `cbr-xml-daily-2026-04-11.xml` contains an INR row for
`11.04.2026` with `VunitRate` `0,830794`. That means the current CBR endpoint can serve this exact
date. The code fix still needs prior-date fallback because locally stored `.lino` datasets and
other providers may omit weekend or holiday rows, and the issue explicitly requires previous-rate
fallback rather than inventing unavailable rates.

## Fix

The currency database now centralizes historical rate lookup:

- Try the exact requested date first.
- Otherwise scan rates for the same currency pair.
- Select the maximum rate date that is less than or equal to the requested date.
- Return no rate if every available rate is after the requested date.

Regression tests cover both the weekend fallback case and the future-rate guard.
