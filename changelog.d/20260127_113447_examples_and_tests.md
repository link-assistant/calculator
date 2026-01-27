### Added
- Examples section showing 6 random examples from a centralized examples.lino file
- React unit tests for App.tsx component (23 new tests covering branding, input, settings, examples, and footer)
- USE-CASES.md documentation with screenshots of calculator features
- CI/CD workflow for auto-updating screenshots when web code changes
- New data/examples.lino file containing categorized calculator examples (arithmetic, currency, datetime, functions, integration)
- E2E screenshot generation tests for documentation

### Changed
- Updated E2E tests to use explicit calculation trigger (Enter key or button click) instead of reactive calculation
- Examples are now randomly selected from examples.lino on each page load for variety
