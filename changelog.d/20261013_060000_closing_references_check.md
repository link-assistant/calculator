---
bump: patch
---

### Added
- Pull Request Description workflow (`scripts/check-closing-references.mjs`): fails when a description's closing keywords close an issue it says stays open, when it says "closes only #N" but closes more, when `Fixes #1, #2` closes only #1, or when a negated keyword ("does not close #N") would still close the issue (#247).
