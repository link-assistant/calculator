---
bump: minor
---

### Added
- Cache expression results in `localStorage` for instant page loads. Cached results are keyed by expression and app version, so stale entries are automatically invalidated after an upgrade.
- LRU-style eviction caps the cache at 50 entries, keeping `localStorage` usage bounded.
