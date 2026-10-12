#!/usr/bin/env python3
"""List corpus rows whose expression matches a regex, with the baseline
status, the expected value and the current CLI output (issue #235).

Usage: python3 experiments/issue-235/rows.py REGEX [--status supported,different]
"""
import glob
import json
import re
import subprocess
import sys

ROOT = __file__.rsplit('/experiments/', 1)[0]
pattern = re.compile(sys.argv[1])
statuses = None
if '--status' in sys.argv:
    statuses = set(sys.argv[sys.argv.index('--status') + 1].split(','))
baseline = json.load(open(f'{ROOT}/docs/case-studies/issue-227/results/coverage-baseline.json'))
status_of = {(r['source'], r['expression']): r['status'] for r in baseline['results']}
for path in sorted(glob.glob(f'{ROOT}/docs/case-studies/issue-227/corpus/*.tsv')):
    for line in open(path, encoding='utf-8'):
        if line.startswith('#') or not line.strip():
            continue
        cols = line.rstrip('\n').split('\t')
        if len(cols) < 4:
            continue
        source, _lang, expr, expected = cols[:4]
        expr = expr.replace('\\t', '\t').replace('\\n', '\n')
        if not pattern.search(expr):
            continue
        status = status_of.get((source, expr), '?')
        if statuses and status not in statuses:
            continue
        run = subprocess.run([f'{ROOT}/target/release/link-calculator', expr],
                             capture_output=True, text=True, timeout=10)
        out = next((l for l in run.stdout.splitlines() if l.startswith(('Result: ', 'Error: '))), run.stderr.strip())
        print(f'{status:11} | {source} | {expr} | exp: {expected} | now: {out}')
