## Problem

On `pull_request` runs, `pipeline-status` gets:

```yaml
RUN_SHA: ${{ github.sha }}
BRANCH_REF: ${{ github.ref_name }}
```

On a `pull_request` event `github.sha` is the synthetic merge commit and `github.ref_name` is `<number>/merge`. `scripts/check-pipeline-status.sh` resolves the branch head with `git ls-remote origin refs/heads/<number>/merge`, which never exists, so it prints `Could not resolve the head of <n>/merge; cannot prove this run was superseded.` and fails closed.

The result: every PR run superseded by a new push, with a job that sets `cancel-in-progress: true` cancelled, ends red. That is exactly the case the gate means to excuse.

The js and python templates already pass the PR head. Every gate in this template still uses the merge values: `release.yml`, `links.yml`, `security.yml`, `workflows.yml` and `desktop-release.yml` (checked at 265219d).

## Reproduction

Script: https://github.com/link-assistant/calculator/blob/issue-256-9c021bdb8c9d/experiments/pipeline-status-pr-ref.sh. It builds a throwaway remote whose `feature` branch has moved past the run's commit, then runs the gate with both sets of env.

```console
$ experiments/pipeline-status-pr-ref.sh /path/to/rust-template/scripts/check-pipeline-status.sh
--- template env: github.sha + github.ref_name
Could not resolve the head of 7/merge; cannot prove this run was superseded.
::error title=Pipeline has cancelled jobs::lint. No supersede accounts for these cancellations; ...
exit 1
--- fixed env: pull_request.head.sha + github.head_ref
This run tests 404817d…; feature is at bd8828f….
::warning title=Cancelled jobs in a superseded run::lint. ...
exit 0
```

## Fix

Use the values the js and python templates already use, in every gate:

```yaml
          # On pull_request, github.sha is the synthetic merge commit and ref_name
          # is <n>/merge, neither of which a superseding push moves; compare
          # the PR head instead so superseded PR runs are recognised.
          RUN_SHA: ${{ github.event.pull_request.head.sha || github.sha }}
          BRANCH_REF: ${{ github.head_ref || github.ref_name }}
```

Fork PRs: `refs/heads/<head_ref>` names a branch in the fork, not in this repository. So `ls-remote` either finds nothing (fail closed, the same as today) or finds an unrelated same-named branch. Comparing `github.event.pull_request.head.repo.full_name` with `github.repository` before trusting the lookup would close that gap.

A test like the following would catch a regression: the `env:` block of every `pipeline-status` job must reference `pull_request.head.sha`.

## Workaround

Apply the two env lines above. Downstream fix: link-assistant/calculator#257 (commit a6c7a4a2).

Found while working on link-assistant/calculator#256.
