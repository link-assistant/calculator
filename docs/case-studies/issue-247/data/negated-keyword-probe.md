# Negated closing keyword probe (2026-10-13)

Draft PR #261's description was temporarily extended with:

> Probe for #247: this pull request does not close #258.

`gh pr view 261 --json closingIssuesReferences` then returned `[258, 260]`;
after restoring the original description it returned `[260]`. GitHub ignores
the negation and links "does not close #258" as a closing reference, so the
issue would have been closed on merge. GitLab documents the same behaviour:
https://gitlab.com/gitlab-org/gitlab-foss/-/work_items/55970.
