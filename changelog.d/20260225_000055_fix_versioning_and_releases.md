---
bump: minor
---

### Fixed

- Fixed auto-release pipeline not bumping version when a git tag already existed from a previous partial release (version-and-commit.mjs now uses Cargo.toml as source of truth instead of git tags)
- Fixed changelog fragments not being deleted after collection in auto-release, causing duplicate release attempts on every push
- Fixed tag force-update to allow retrying a failed release without manual tag deletion
- Fixed changelog fragment check not requiring fragments for web/ frontend changes
- Fixed code change detection not recognizing TypeScript (.ts, .tsx), CSS, and HTML files as code changes requiring changelog entries
