#!/usr/bin/env bash
# The dependency-direction gate: no shipped binary reaches a test-only crate
# through its non-dev dependency graph. The rule table and the walk live in the
# Python module beside this file.
#
# Exit 0 = every binary target's graph is clean, or the workspace has none yet.
# Exit 1 = a forbidden path exists, or the graph could not be read.

set -uo pipefail

usage() {
  cat <<'USAGE'
Usage:
  check-import-guard.sh    walk every binary target's non-dev dependency graph
USAGE
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

root=$(git rev-parse --show-toplevel 2>/dev/null) || {
  printf 'check-import-guard: not a git work tree; skipped\n' >&2; exit 0; }
cd "$root" || exit 1

command -v python3 >/dev/null 2>&1 || {
  printf 'check-import-guard: python3 not on PATH; gate skipped\n' >&2; exit 0; }

exec python3 ai-docs/scripts/import_guard.py "$@"
