#!/usr/bin/env bash
# Repro for issue #256: the pipeline-status gate on a pull_request run.
# A superseded PR run whose jobs were cancelled must be excused. With the
# template's env (RUN_SHA=github.sha, BRANCH_REF=github.ref_name) the gate
# resolves refs/heads/<n>/merge, which never exists, and fails closed.
#
# Usage: experiments/pipeline-status-pr-ref.sh [path/to/check-pipeline-status.sh]
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
gate="${1:-$here/scripts/check-pipeline-status.sh}"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT

# A remote with a PR branch whose head has moved on (superseded run).
git init -q --bare "$tmp/remote.git"
git -C "$tmp" init -q -b main work
git -C "$tmp/work" -c user.name=t -c user.email=t@t commit -q --allow-empty -m old
old="$(git -C "$tmp/work" rev-parse HEAD)"
git -C "$tmp/work" -c user.name=t -c user.email=t@t commit -q --allow-empty -m new
git -C "$tmp/work" push -q "$tmp/remote.git" HEAD:refs/heads/feature
mkdir -p "$tmp/wf/.github/workflows"
printf 'jobs:\n  lint:\n    concurrency:\n      group: lint\n      cancel-in-progress: true\n' > "$tmp/wf/.github/workflows/release.yml"
git -C "$tmp/wf" init -q && git -C "$tmp/wf" remote add origin "$tmp/remote.git"

run() {
  echo "--- $1"
  (cd "$tmp/wf" && env NEEDS_JSON='{"lint":{"result":"cancelled"}}' \
    WORKFLOW_FILE=.github/workflows/release.yml RUN_SHA="$2" BRANCH_REF="$3" \
    bash "$gate"); echo "exit $?"
}
# github.sha on pull_request is the synthetic merge commit; ref_name is 7/merge.
run 'template env: github.sha + github.ref_name' "$(printf 'm%.0s' {1..40})" '7/merge'
run 'fixed env: pull_request.head.sha + github.head_ref' "$old" 'feature'
