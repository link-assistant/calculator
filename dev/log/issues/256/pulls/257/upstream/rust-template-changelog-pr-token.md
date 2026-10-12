## Problem

The `changelog-pr` job in `release.yml` opens the manual release PR with the built-in token:

```yaml
      - name: Create Pull Request
        uses: peter-evans/create-pull-request@5f6978faf089d4d20b00c7766989d076bb2fc7f1 # v8.1.1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}   # release.yml:1180-1182 at 265219d
```

GitHub documents that a PR created or updated with `GITHUB_TOKEN` gets its `pull_request` runs in an **approval-required** state ([docs](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow#triggering-a-workflow-from-a-workflow)). `release.yml` has no `paths:` filter on `pull_request`, so the release PR always gets such a run. It shows zero jobs until a maintainer clicks "Approve workflows to run". If the PR is merged without approval, the run is concluded as `failure` with no jobs at all.

Observed downstream: link-assistant/calculator PR #255 was opened by a workflow with `GITHUB_TOKEN` at 11:47:36Z. Its run [38136861895](https://github.com/link-assistant/calculator/actions/runs/38136861895) (`event: pull_request`, actor `github-actions[bot]`) never started a job. It was marked `failure` at 12:21:35Z, two seconds after the PR was merged untested.

js template issue #192 hit the same root cause (generated PRs stuck in `action_required`). The js template now requires a dedicated `RELEASE_PR_TOKEN` and fails early when it is missing (`release.yml`, around line 970).

## Reproduction

1. Run `release.yml` with `workflow_dispatch`, `release_mode: changelog-pr`, without any PAT configured.
2. Open the created PR: the merge box shows "Approve workflows to run" and the `pull_request` run has no jobs.
3. Merge without approving: the run turns into a zero-job `failure`.

## Fix

Either:

- Use a dedicated token, as the js template does: `token: ${{ secrets.RELEASE_PR_TOKEN }}` with a preflight step that fails when it is empty. Or
- Fall back to `GITHUB_TOKEN` and dispatch the checks on the PR branch. `workflow_dispatch` runs always start, and their check runs attach to the PR head commit:

```yaml
      - name: Run CI checks on the release PR
        if: steps.cpr.outputs.pull-request-operation == 'created' || steps.cpr.outputs.pull-request-operation == 'updated'
        env:
          GH_TOKEN: ${{ github.token }}   # job needs `actions: write`
          PR_BRANCH: ${{ steps.cpr.outputs.pull-request-branch }}
        run: gh workflow run release.yml --repo "$GITHUB_REPOSITORY" --ref "$PR_BRANCH" -f release_mode=checks
```

  This needs a `checks` value for `release_mode` that runs the test jobs without releasing.

## Workaround

Set a PAT or GitHub App token and pass it to `create-pull-request`. Downstream implementation of the dispatch fallback: link-assistant/calculator#257 (`refresh-competitor-corpus.yml`, commit 880d8ed4).

Found while working on link-assistant/calculator#256.
