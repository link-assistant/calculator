"""List percentage corpus rows and their recorded coverage status.

Run from the repository root. --baseline can select a saved pre-fix baseline
to inspect changes without modifying upstream expectations.
"""

import argparse
import json
import re
from collections import Counter
from pathlib import Path

base = Path("docs/case-studies/issue-227")
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--baseline", type=Path, default=base / "results/coverage-baseline.json")
args = parser.parse_args()
statuses = {
    (row["source"], row["lang"], row["expression"]): row["status"]
    for row in json.loads(args.baseline.read_text(encoding="utf-8"))["results"]
}
counts = Counter()
for path in sorted((base / "corpus").glob("*.tsv")):
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        row = line.split("\t")
        text = row[2].lower()
        if "percentile" in text or not any(
            word in text for word in ["%", "percent", "процент", "百分", "パーセント"]
        ):
            continue
        expression = re.sub(
            r"\\(t|n|\\)",
            lambda match: {"t": "\t", "n": "\n", "\\": "\\"}[match[1]],
            row[2],
        )
        status = statuses.get((row[0].strip(), row[1].strip(), expression), "missing")
        counts[status] += 1
        print("\t".join([status, *row[:4]]))
print("Counts:", dict(counts))
