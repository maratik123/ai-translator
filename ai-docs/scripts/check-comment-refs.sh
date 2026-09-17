#!/usr/bin/env bash
# The comment-reference gate, over the tracked gated set or over one staged
# commit's share of it.
#
# ONE IMPLEMENTATION, THREE CALLERS: this wrapper, the pre-commit hook (which
# passes the staged flag), and the project's own build entry point. The decision
# itself lives in the Python module beside this file, because the classes are
# decided on tokenised source and a scanner that understands string literals is
# not something a line-oriented filter can stand in for.
#
# Exit 0 = no comment in the scanned set points outward.
# Exit 1 = at least one does, or a scanned file could not be read.

set -uo pipefail

usage() {
  cat <<'USAGE'
Usage:
  check-comment-refs.sh              every tracked file of the gated set
  check-comment-refs.sh --staged     the staged files of the gated set
  check-comment-refs.sh <path> [...] the named files
USAGE
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

root=$(git rev-parse --show-toplevel 2>/dev/null) || {
  printf 'check-comment-refs: not a git work tree; skipped\n' >&2; exit 0; }
cd "$root" || exit 1

# Fail direction, named: without the interpreter the gate cannot run, and a
# machine that cannot run it is not a machine whose commits are all wrong. The
# skip is loud; silence is what is forbidden.
command -v python3 >/dev/null 2>&1 || {
  printf 'check-comment-refs: python3 not on PATH; gate skipped\n' >&2; exit 0; }

exec python3 ai-docs/scripts/comment_refs.py "$@"
