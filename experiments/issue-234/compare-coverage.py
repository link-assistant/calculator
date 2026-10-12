"""Compare coverage status transitions without rerunning or changing the corpus."""

import argparse
import json
from collections import Counter, defaultdict
from itertools import zip_longest
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("before", type=Path)
parser.add_argument("after", type=Path)
args = parser.parse_args()
before = json.loads(args.before.read_text(encoding="utf-8"))
after = json.loads(args.after.read_text(encoding="utf-8"))


def key(row):
    return (row["source"], row["lang"], row["expression"])


old = defaultdict(list)
current = defaultdict(list)
for row in before["results"]:
    old[key(row)].append(row["status"])
for row in after["results"]:
    current[key(row)].append(row["status"])
ranks = {status: rank for rank, status in enumerate(
    ["supported", "different", "unsupported", "timeout", "missing"]
)}
transitions = Counter()
regressions = []
for identity in old.keys() | current.keys():
    for previous, status in zip_longest(old[identity], current[identity], fillvalue="missing"):
        if previous != status:
            transitions[f"{previous} -> {status}"] += 1
            if ranks[status] > ranks[previous]:
                regressions.append((identity, previous, status))
print("Before:", before["total"])
print("After:", after["total"])
print("Transitions:", dict(sorted(transitions.items())))
for identity, previous, status in sorted(regressions):
    print("Regression:", *identity, previous, "->", status, sep="\t")
