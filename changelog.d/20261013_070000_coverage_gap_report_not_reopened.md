---
bump: patch
---

### Fixed
- The Coverage Gap Report workflow no longer reopens a closed **Coverage gap report** issue every week. The issue is a status page; reopening it made it a perpetual work item for automated solvers (#258). It now lists the open sub-issues of #227 that track the gaps, writes the tables to the job summary, also runs when the coverage baseline changes on `main`, and fails only when gaps remain with no open sub-issue to track them. `--dry-run` and `--verbose` preview the update.
