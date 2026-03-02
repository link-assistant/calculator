---
bump: patch
---

### Fixed
- Fixed datetime arithmetic with duration-unit expressions (e.g., `now - 10 days`, `now + 2 hours`) that previously resulted in "Cannot subtract number from datetime" errors
