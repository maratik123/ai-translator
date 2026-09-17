#!/usr/bin/env bash
# The panic gate, over the tracked Rust sources or over the files it is given.
#
# ONE IMPLEMENTATION, TWO CALLERS: the project's own build entry point runs it
# over the tree, and the editor hook runs it over the one file just written, so
# a call refused at commit time is refused the moment it is typed. The rule and
# the escape are documented in the module beside this file.
#
# Exit 0 = every panicking call in shipped code carries its marker.
# Exit 1 = at least one does not.

set -uo pipefail

usage() {
  cat <<'USAGE'
Usage:
  check-panic-calls.sh              every tracked non-test Rust source
  check-panic-calls.sh --staged     the staged Rust sources
  check-panic-calls.sh <path> [...] the named files
USAGE
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

root=$(git rev-parse --show-toplevel 2>/dev/null) || {
  printf 'check-panic-calls: not a git work tree; skipped\n' >&2; exit 0; }
cd "$root" || exit 1

command -v python3 >/dev/null 2>&1 || {
  printf 'check-panic-calls: python3 not on PATH; gate skipped\n' >&2; exit 0; }

exec python3 ai-docs/scripts/panic_calls.py "$@"
