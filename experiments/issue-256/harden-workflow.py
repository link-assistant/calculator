"""Mechanical GitHub Actions hardening used for issue #256.

Usage: python3 harden-workflow.py <workflow.yml> [--writer-job name ...]

* ubuntu-latest -> ubuntu-24.04 (ubuntu-latest moves to Ubuntu 26 on 2026-10-19)
* bump first-party action majors to the current release
* hash-pin dtolnay/rust-toolchain and peter-evans/create-pull-request
* add `persist-credentials: false` to every checkout outside writer jobs
* `always() && !cancelled()` -> `!cancelled()`, `always() &&` -> `!cancelled() &&`
"""
import re
import sys

RUST = ('dtolnay/rust-toolchain@e2a55d2ffb04f378e9626c28d38b36d230d1e12f'
        ' # master branch @ 2026-10-09')
CPR = ('peter-evans/create-pull-request@5f6978faf089d4d20b00c7766989d076bb2fc7f1'
       ' # v8.1.1')
BUMPS = {
    'actions/checkout@v6': 'actions/checkout@v7',
    'actions/setup-node@v6': 'actions/setup-node@v7',
    'actions/cache@v5': 'actions/cache@v6',
    'actions/download-artifact@v7': 'actions/download-artifact@v8',
    'actions/setup-python@v6': 'actions/setup-python@v7',
    'peter-evans/create-pull-request@v8': CPR,
}

path = sys.argv[1]
writers = set(sys.argv[sys.argv.index('--writer-job') + 1:]) if '--writer-job' in sys.argv else set()
lines = open(path, encoding='utf-8').read().split('\n')
out = []
job = None
i = 0
while i < len(lines):
    line = lines[i]
    m = re.match(r'^  ([A-Za-z0-9_-]+):\s*$', line)
    if m:
        job = m.group(1)
    line = line.replace('runs-on: ubuntu-latest', 'runs-on: ubuntu-24.04')
    line = line.replace('[ubuntu-latest]', '[ubuntu-24.04]')
    for old, new in BUMPS.items():
        line = line.replace(old, new)
    line = line.replace('always() && !cancelled()', '!cancelled()')
    line = re.sub(r'\balways\(\) && ', '!cancelled() && ', line)
    rust = re.match(r'^(\s*)(- )?uses: dtolnay/rust-toolchain@stable\s*$', line)
    if rust:
        indent = rust.group(1) + ('  ' if rust.group(2) else '')
        out.append(line.replace('dtolnay/rust-toolchain@stable', RUST))
        # Merge with an existing `with:` block when present.
        if i + 1 < len(lines) and lines[i + 1].strip() == 'with:':
            out.append(lines[i + 1])
            out.append(f'{indent}  toolchain: stable')
            i += 2
            continue
        out.append(f'{indent}with:')
        out.append(f'{indent}  toolchain: stable')
        i += 1
        continue
    co = re.match(r'^(\s*)(- )?uses: actions/checkout@v7\s*$', line)
    if co and job not in writers:
        indent = co.group(1) + ('  ' if co.group(2) else '')
        out.append(line)
        if i + 1 < len(lines) and lines[i + 1].strip() == 'with:':
            out.append(lines[i + 1])
            out.append(f'{indent}  persist-credentials: false')
            i += 2
            continue
        out.append(f'{indent}with:')
        out.append(f'{indent}  persist-credentials: false')
        i += 1
        continue
    out.append(line)
    i += 1
open(path, 'w', encoding='utf-8').write('\n'.join(out))
