## Problem

Checked at 265219d (2026-10-10):

1. **`dtolnay/rust-toolchain@7e38f4b43b4db5c8dd498af069a4f6196df1d067` (16 uses) predates the action's input-injection fix.** dtolnay/rust-toolchain#187 ("Safely handle action inputs", merge commit `e2a55d2ffb04f378e9626c28d38b36d230d1e12f`, 2026-10-09) changed how the action passes its `toolchain`, `targets` and `components` inputs to its shell. The template pin is the 2026-10-01 merge of #186, so it does not include that hardening.
2. **First-party actions are a major behind**, and two of them mix majors in the same repository:

| Action | Template | Latest |
| --- | --- | --- |
| actions/checkout | v6 (33 uses) | v7.0.1 |
| actions/cache | v5 (7) | v6.1.0 |
| actions/upload-artifact | v6 (1) and v7 (2) | v7.0.2 |
| actions/download-artifact | v7 (1) and v8 (2) | v8.0.2 |

checkout v7.0.0 (2026-06-18) also refuses to check out fork PR code under `pull_request_target` and `workflow_run` (actions/checkout#2454), a safe default worth having.

## Reproduction

```sh
grep -rhoE 'uses: (actions/[a-z-]+|dtolnay/rust-toolchain)@[^ ]+' .github | sort | uniq -c
gh api repos/actions/checkout/releases/latest --jq .tag_name
gh api 'repos/dtolnay/rust-toolchain/commits?sha=master&per_page=4' --jq '.[]|.sha[0:8]+" "+.commit.message'
```

## Fix

- Pin `dtolnay/rust-toolchain@e2a55d2ffb04f378e9626c28d38b36d230d1e12f`.
- Bump checkout to v7 and cache to v6, and move upload-artifact to v7 and download-artifact to v8 everywhere.
- A Dependabot `github-actions` ecosystem entry, or a test that every `uses: actions/<name>@` in the tree has the same major, would keep these in step.

Downstream: link-assistant/calculator#257 uses these versions with zizmor and actionlint clean.

Found while working on link-assistant/calculator#256.
