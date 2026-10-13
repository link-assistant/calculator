"""One-off: turn release.yml into the JavaScript entry of the #254 CI split."""
import re
p='.github/workflows/release.yml'
L=open(p).read().split('\n')
assert L[83]=='jobs:', L[83]
fm=next(i for i,l in enumerate(L) if l.startswith('  # === FRESH MERGE'))
head='\n'.join(L[:84])
js_jobs='\n'.join(L[84:fm]).rstrip('\n')+'\n'
cl_start=next(i for i,l in enumerate(L) if l.startswith('  # === MANUAL CHANGELOG PR'))
ps_start=next(i for i,l in enumerate(L) if l.startswith('  # === PIPELINE STATUS'))
changelog_pr='\n'.join(L[cl_start:ps_start])

head=re.sub(r"\n    paths:\n(      - '.*'\n)+", "\n", head)
head=head.replace("""  workflow_dispatch:
    inputs:""","""  # Weekly: dependency freshness, advisories and the coverage gap report
  # depend on upstream state, not on this repository's diff.
  schedule:
    - cron: '0 6 * * 1'
  workflow_dispatch:
    inputs:""")
head=head.replace("""        # it, so its workflow dispatches this mode on the PR branch (issue #256).""","""        # it, so its workflow dispatches this mode on the PR branch (issue #256).
        # `screenshots` and `coverage-gap-report` run those jobs of rust.yml,
        # which replaced the separate workflow triggers (issue #254).""")
head=head.replace("""          - checks
""","""          - checks
          - screenshots
          - coverage-gap-report
""")
head=head.replace("""concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}
""","""# No workflow-level concurrency (issue #254): every JavaScript job cancels
# its superseded run through its own group, while rust-stage queues per branch
# and never cancels, so a push does not kill a Rust build halfway.
""")
head=re.sub(r"  CARGO_TERM_COLOR: always\n  RUSTFLAGS: -Dwarnings\n  CARGO_NET_RETRY: '10'\n  CARGO_HTTP_MULTIPLEXING: 'false'\n  # The crates.io credentials.*?instead\.\n", "", head, flags=re.S)
head=head.replace("name: CI/CD Pipeline\n","""name: CI/CD Pipeline

# JavaScript first (issue #254): every job in this file is JavaScript, Python
# or shell. The Rust jobs live in rust.yml, which rust-stage calls only after
# every JavaScript job passed, so a broken JavaScript check never spends Rust
# build minutes. scripts/check-javascript-first.mjs enforces the split. There
# are no path filters: the parity and translation checks run on every change.
""",1)

def add_conc(text, job):
    i=text.index(f"  {job}:\n")
    j=text.index("    runs-on:", i)
    return text[:j]+f"    concurrency:\n      group: js-${{{{ github.workflow }}}}-${{{{ github.ref }}}}-{job}\n      cancel-in-progress: true\n"+text[j:]
for job in ['detect-changes','changelog','version-check','secrets-scan']:
    js_jobs=add_conc(js_jobs, job)
js_jobs=js_jobs.replace("    if: github.event_name != 'workflow_dispatch'\n","    if: github.event_name == 'push' || github.event_name == 'pull_request'\n",1)

new_jobs=open('experiments/issue-254/release-js-jobs.yml').read()
stage=open('experiments/issue-254/release-stages.yml').read()
changelog_pr=changelog_pr.replace("    if: github.event_name == 'workflow_dispatch' && github.event.inputs.release_mode == 'changelog-pr'\n","    if: github.event_name == 'workflow_dispatch' && github.event.inputs.release_mode == 'changelog-pr'\n    concurrency:\n      group: js-${{ github.workflow }}-${{ github.ref }}-changelog-pr\n      cancel-in-progress: true\n")
out=head+'\n'+js_jobs+new_jobs+'\n'+changelog_pr.rstrip('\n')+'\n'+stage
open(p,'w').write(out)
