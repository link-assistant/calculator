#!/usr/bin/env bash
# Bound each documentation probe, including deliberately extreme numeric literals.
set -eu
ulimit -v 524288
ulimit -s 8192
task_root="${BASH_SOURCE[0]%/*}/../.."
exec "$task_root/target/release/link-calculator" "$@"
