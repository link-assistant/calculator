"""Compare coverage status transitions without rerunning or changing the corpus."""

import argparse
import json
from collections import Counter
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("before", type=Path)
parser.add_argument("after", type=Path)
args = parser.parse_args()
before = json.loads(args.before.read_text(encoding="utf-8"))
after = json.loads(args.after.read_text(encoding="utf-8"))


def key(row):
    return (row["source"], row["lang"], row["expression"])


old = {key(row): row["status"] for row in before["results"]}
current = {key(row): row["status"] for row in after["results"]}
ranks = {status: rank for rank, status in enumerate(
    ["supported", "different", "unsupported", "timeout", "missing"]
)}
transitions = Counter()
regressions = []
for identity in old.keys() | current.keys():
    previous = old.get(identity, "missing")
    status = current.get(identity, "missing")
    if previous != status:
        transitions[f"{previous} -> {status}"] += 1
        if ranks[status] > ranks[previous]:
            regressions.append((identity, previous, status))
print("Before:", before["total"])
print("After:", after["total"])
print("Transitions:", dict(sorted(transitions.items())))
for identity, previous, status in sorted(regressions):
    print("Regression:", *identity, previous, "->", status, sep="\t")
