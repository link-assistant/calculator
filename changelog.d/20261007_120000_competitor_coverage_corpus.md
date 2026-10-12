---
bump: patch
---

### Added
- Case study `docs/case-studies/issue-227` for parity with every calculator on natural-language math: requirement inventory, method, initial measurement on 3055 corpus rows, silent-wrong-answer families, error classes, solution options, component evaluation, the 18 sub-issue specifications (#229 to #246) with their blocker graph, and the progress re-check on the 13975-row corpus (#227).
- `experiments/issue-227/create-sub-issues.mjs`: idempotently files the sub-issues, attaches them to #227 and registers their blocked-by dependencies.
