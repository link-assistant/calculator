---
bump: patch
---

### Fixed
- Fixed parsing of time expressions that start with a number followed by a colon, such as `11:59pm EST on Monday, January 26th`. Previously these returned just the number (e.g. `11`); now they are correctly parsed as datetime values. Fixes #23.
