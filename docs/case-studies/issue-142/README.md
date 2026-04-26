# Issue 142 Case Study

## Summary

Issue #142 asks for calculator's issue-report URL generation to become a reusable API for
`link-assistant/meta-expression` and future browser tools. The existing helper generated useful
calculator reports, but the public call shape was tied to calculator page state, a translation
function, and the hardcoded `link-assistant/calculator` repository.

The related meta-expression issue #7 shows the target use case: a browser tool needs a "Report
Issue" button that opens a GitHub issue with the current statement, result, reasoning diagnostics,
Links Notation payload, and environment already filled in.

## Collected Artifacts

- `issue.json`: issue #142 title, body, timestamps, and GitHub metadata.
- `issue-comments.json`: all issue #142 comments fetched via the GitHub API.
- `pr-143.json`: draft PR metadata before this implementation.
- `ci-runs.json`: recent CI runs for branch `issue-142-efe8b49ab23b`.
- `meta-expression-issue-7.json`: related issue that needs the reusable behavior.
- `meta-expression-issue-7-screenshot.png`: screenshot from meta-expression issue #7. The
  environment lacked the `file` utility, so the PNG signature was verified with `od` before visual
  inspection.

## Related Evidence

The screenshot in meta-expression issue #7 shows:

- Input statement: `moon orbits the Sun`.
- A "Report Issue" button in the target app.
- A generated Links Notation-style diagnostic panel.
- Multiple interpretations and a result estimate.

This confirms that the reusable API must not assume calculator-only data. It needs to accept the
target repository, input text, successful or failed result data, Links Notation diagnostics,
alternative interpretations, steps, and environment details from the consumer.

## Online Research

- GitHub supports prefilled issue creation through `/issues/new` query parameters, including
  `title`, `body`, and `labels`. GitHub also notes permission and URL-length constraints for these
  query parameters: https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-an-issue#creating-an-issue-from-a-url-query
- `URLSearchParams.toString()` returns a query string without the leading question mark and applies
  form-style percent encoding, including `+` for spaces. This is appropriate for prefilled issue
  URLs: https://developer.mozilla.org/en-US/docs/Web/API/URLSearchParams/toString
- The Rust `github-issue-url` crate demonstrates the same pattern for Rust applications, but it is
  not a good fit for the existing React/TypeScript frontend helper without adding a cross-language
  packaging step: https://docs.rs/github-issue-url/latest/github_issue_url/

## Requirements

- Extract report URL generation into a reusable API or package surface.
- Let consumers choose the target GitHub repository.
- Keep the reusable API independent from React and i18n; labels and state come from the caller.
- Include environment, input, result, reasoning steps, and Links Notation diagnostics in the body.
- Support successful result reports.
- Support failed result reports with error diagnostics.
- Support alternative Links Notation interpretations.
- Support reproduction steps.
- Keep calculator's existing "Report Issue" behavior working.
- Add tests that assert the generated issue URL contains title, body, and labels.
- Compile issue data and analysis under `docs/case-studies/issue-142`.
- Review existing components or libraries that solve the same problem.

## Solution Options

### Option 1: Copy calculator helper into meta-expression

This is the fastest local fix, but it preserves the duplication called out in the issue. It also
keeps repository, translation, and page-state assumptions scattered across apps.

Plan:

1. Copy `web/src/utils/reportIssue.ts` into meta-expression.
2. Replace hardcoded repository and labels manually.
3. Repeat future fixes in every consumer.

### Option 2: Pure TypeScript builder inside calculator web utilities

This keeps the implementation small and matches the current consumer language. The reusable surface
accepts `{ repository, input, result, linksNotation, steps, environment }`, while calculator keeps a
compatibility path from its current `PageState`.

Plan:

1. Replace hardcoded report generation with a pure options-based builder.
2. Keep translation labels optional and caller-supplied.
3. Keep `URLSearchParams` as the encoding mechanism.
4. Add tests for custom repository, labels, successful payloads, failed payloads, alternatives, and
   reproduction steps.

This is the chosen solution for this PR.

### Option 3: Publish a separate npm package

This is the cleanest long-term sharing model once multiple apps consume the helper. It needs package
metadata, versioning, release automation, and integration work in meta-expression.

Plan:

1. Move the pure builder into a package directory.
2. Add package publishing workflow and version policy.
3. Replace local imports in calculator and meta-expression with package imports.

### Option 4: Use or adapt an existing Rust crate

Existing Rust libraries demonstrate the concept, but the immediate consumer is a browser
TypeScript frontend. Using a Rust crate would require WASM or a generated binding for a small URL
builder that TypeScript can already express directly.

Plan:

1. Evaluate `github-issue-url` for field coverage.
2. Add WASM bindings or duplicate its behavior in TypeScript.
3. Use only if the Rust codebase needs the same issue URL builder directly.

## Implemented Plan

- Added a pure `generateIssueUrl(options)` path with caller-supplied repository, issue labels,
  environment, input, result, Links Notation diagnostics, alternatives, steps, and reproduction
  steps.
- Kept calculator compatibility by continuing to support `generateIssueUrl(pageState, t)`.
- Removed the direct `i18next` type dependency from the reusable utility.
- Added `web/src/utils/index.ts` so the report builder is available as a small utility surface.
- Added focused Vitest coverage for custom-repository URL generation and failed-result diagnostics.

## Residual Follow-Up

The issue comment also mentions exporting "other useful features" of calculator. That is broader
than the issue-report helper and should be split into separate API design issues so each export has
clear consumers, tests, and versioning expectations.
