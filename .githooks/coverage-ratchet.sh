#!/usr/bin/env bash
# The coverage ratchet: line coverage may rise freely and may not fall.
#
# ONE IMPLEMENTATION, THREE CALLERS. The pre-commit hook runs it in raise
# mode, `make cover-ratchet` and CI run it with --check. A second spelling of
# the measurement would let a local run and a CI run disagree about what the
# number is, which is the whole reason the project's own build entry point
# exists.
#
# WHAT IS MEASURED. `cargo llvm-cov --workspace --summary-only --json` over the
# whole workspace, and the LINE total of its summary — not regions and not
# functions. Lines are the metric a reader can check by eye against the source,
# and the one whose number means the same thing after a refactor that splits an
# expression. The three totals disagree by several points on the same tree, so
# the choice is recorded here rather than left to whoever reads the JSON next.
#
# ROUNDING. Half-up to hundredths of a percentage point, computed as
# int(p*100 + 0.5)/100 rather than printf "%.2f" — printf rounds half-to-even
# in most awks, and a ratchet should not depend on which awk is installed.
# Float arithmetic throughout; the tolerance absorbs the epsilon.
#
# THE TOLERANCE IS ZERO, and that is a starting value, not a measurement. A
# tolerance is the width of the suite's own run-to-run drift, and no drift
# series has been run here yet: the crates the workspace holds are skeletons
# and carry no test. The first drift this project actually observes is what
# sets it — derived from a series, never from a single blocked commit:
#
#   for i in 1 2 3; do
#     cargo llvm-cov --workspace --summary-only --json > "tmp/cov$i.json"
#   done
#   # the spread between the three line totals is the drift
#
# Re-measure in BOTH a local and a CI environment before moving the constant,
# and say in the commit what the series showed. A tolerance raised from one
# blocked commit hides exactly the drop it was raised for.
#
# WHAT A TOLERANCE COSTS, bounded: the recorded value never decreases, so the
# total that can be lost silently is one tolerance below the all-time high —
# ONCE, not per commit.
#
# Exit 0 = coverage holds (or the ratchet was initialised, or the run was
#          skipped for a named reason printed to stderr).
# Exit 1 = coverage fell past the tolerance, or it could not be measured.

set -uo pipefail

usage() {
  cat <<'USAGE'
Usage:
  coverage-ratchet.sh            raise mode: check, and record a new high
  coverage-ratchet.sh --check    check only: never writes, never stages
USAGE
}

TOLERANCE_PP=0.00
RATCHET_FILE=ai-docs/coverage-ratchet.txt
PROFILE=tmp/coverage.json

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

mode=${1:-raise}
case "$mode" in
  raise|--check) ;;
  *) printf 'coverage-ratchet: unknown argument %s\n' "$mode" >&2; exit 1 ;;
esac

root=$(git rev-parse --show-toplevel 2>/dev/null) || {
  printf 'coverage-ratchet: not a git work tree; skipped\n' >&2; exit 0; }
cd "$root" || exit 1

# --- Fail directions, each named ---------------------------------------------
# Every dependency below fails OPEN, loudly: a machine that cannot measure Rust
# coverage cannot be asked for a Rust coverage number, and refusing every commit
# there would be the expensive error. Silence is what is forbidden, not the skip.
[ -f Cargo.toml ] || {
  printf 'coverage-ratchet: no Cargo.toml at the repository root; ratchet skipped\n' >&2; exit 0; }

command -v cargo >/dev/null 2>&1 || {
  printf 'coverage-ratchet: cargo not on PATH; ratchet skipped\n' >&2; exit 0; }

cargo llvm-cov --version >/dev/null 2>&1 || {
  printf 'coverage-ratchet: cargo-llvm-cov not installed; ratchet skipped\n' >&2
  printf '  install it with: cargo install cargo-llvm-cov\n' >&2
  exit 0; }

command -v jq >/dev/null 2>&1 || {
  printf 'coverage-ratchet: jq not on PATH; ratchet skipped (the summary is JSON)\n' >&2; exit 0; }

if [ "$mode" = raise ]; then
  # Nothing that can move coverage is staged: SKIP, silently. This is what
  # keeps the hook free on the docs, plan and harness commits that make up most
  # of a /task run.
  staged=$(git diff --cached --name-only --diff-filter=ACMR \
    -- '*.rs' '*.sql' Cargo.toml Cargo.lock '**/Cargo.toml' 2>/dev/null)
  [ -n "$staged" ] || exit 0

  # Unstaged edits to files that move coverage: BLOCK, loud. The measurement is
  # taken on the working tree, so with such edits present it describes neither
  # the commit nor the tree. Silently measuring the wrong thing is how a ratchet
  # becomes decoration.
  dirty=$(git diff --name-only -- '*.rs' '*.sql' Cargo.toml Cargo.lock '**/Cargo.toml' 2>/dev/null)
  if [ -n "$dirty" ]; then
    printf 'coverage-ratchet: BLOCKED — unstaged changes to files that move coverage:\n' >&2
    printf '%s\n' "$dirty" | sed 's/^/  /' >&2
    printf '\nThe ratchet measures the working tree, so with these present the number\n' >&2
    printf 'describes neither the commit nor the tree. Stage them or stash them:\n' >&2
    printf '  git add -- %s\n' "$(printf '%s' "$dirty" | tr '\n' ' ')" >&2
    printf '  git stash push --keep-index\n' >&2
    exit 1
  fi
fi

mkdir -p tmp

# The suite provisions its own database through testcontainers, so this runs the
# same way as `make test` does and needs nothing set up around it.
if ! cargo llvm-cov --workspace --summary-only --json > "$PROFILE" 2> tmp/coverage-run.log; then
  printf 'coverage-ratchet: BLOCKED — the test suite is not green, so coverage is not measurable.\n' >&2
  printf 'Log: tmp/coverage-run.log\n' >&2
  grep -E '^(error|test result|failures:)' tmp/coverage-run.log >&2
  printf '\nIf no container runtime is reachable, start the Podman socket the suite connects to\n' >&2
  printf 'and re-run; a database-backed test cannot be measured without one.\n' >&2
  exit 1
fi

read -r current rounded lines <<EOF
$(jq -r '.data[0].totals.lines | "\(.percent) \(.count)"' "$PROFILE" 2>/dev/null | awk '
  {
    if ($2 == 0 || $1 == "null") { print "NaN NaN 0"; exit }
    p = $1 + 0
    printf "%.6f %.2f %d\n", p, int(p * 100 + 0.5) / 100, $2
  }
  END { if (NR == 0) print "NaN NaN 0" }')
EOF

if [ "$current" = NaN ]; then
  # A workspace with no executable lines is a named skip and not a block. Once a
  # crate carries code, a zero here means the measurement broke, and the suite's
  # own failure would have blocked above — so this branch cannot be reached
  # silently.
  printf 'coverage-ratchet: the summary carries no lines (%s); nothing measured, ratchet skipped\n' "$PROFILE" >&2
  exit 0
fi

# Ratchet file absent or unparseable: INITIALISE and let the commit through.
# The first coverage-moving commit is what installs the ratchet, rather than a
# separate ceremony nobody remembers to perform.
recorded=""
[ -r "$RATCHET_FILE" ] && recorded=$(tr -d '[:space:]' < "$RATCHET_FILE")
case "$recorded" in
  ''|*[!0-9.]*) recorded="" ;;
esac

if [ -z "$recorded" ]; then
  if [ "$mode" = --check ]; then
    printf 'coverage-ratchet: BLOCKED — %s is missing or unparseable.\n' "$RATCHET_FILE" >&2
    printf 'Measured now: %s%% of %s lines. Commit that value to initialise the ratchet.\n' "$rounded" "$lines" >&2
    exit 1
  fi
  mkdir -p "$(dirname "$RATCHET_FILE")"
  printf '%s\n' "$rounded" > "$RATCHET_FILE"
  git add -- "$RATCHET_FILE"
  printf 'coverage-ratchet: initialised at %s%% (%s)\n' "$rounded" "$RATCHET_FILE" >&2
  exit 0
fi

verdict=$(awk -v cur="$current" -v rec="$recorded" -v rnd="$rounded" -v tol="$TOLERANCE_PP" '
  BEGIN {
    if (cur + tol < rec) { print "FELL"; exit }
    if (rnd > rec)       { print "ROSE"; exit }
    print "HELD"
  }')

case "$verdict" in
  FELL)
    printf 'coverage-ratchet: BLOCKED — line coverage fell past the tolerance.\n\n' >&2
    printf '  recorded: %s%%   (%s)\n' "$recorded" "$RATCHET_FILE" >&2
    printf '  measured: %s%%\n' "$rounded" >&2
    printf '  tolerance: %s pp\n\n' "$TOLERANCE_PP" >&2
    printf 'Cover what this change added, or say in the commit why the drop is correct\n' >&2
    printf 'and lower the recorded value in the same commit — never with --no-verify.\n' >&2
    printf 'Uncovered files, worst first:\n' >&2
    jq -r '.data[0].files[] | select(.summary.lines.percent < 100)
             | "\(.summary.lines.percent | . * 100 | round / 100)%  \(.filename)"' \
      "$PROFILE" 2>/dev/null | sort -n | head -20 >&2
    exit 1
    ;;
  ROSE)
    if [ "$mode" = --check ]; then
      printf 'coverage-ratchet: %s%% >= %s%% (a rise the pre-commit hook would have recorded)\n' \
        "$rounded" "$recorded" >&2
      exit 0
    fi
    printf '%s\n' "$rounded" > "$RATCHET_FILE"
    git add -- "$RATCHET_FILE"
    printf 'coverage-ratchet: raised %s%% -> %s%%\n' "$recorded" "$rounded" >&2
    exit 0
    ;;
  *)
    printf 'coverage-ratchet: %s%% holds against %s%% (tolerance %s pp)\n' \
      "$rounded" "$recorded" "$TOLERANCE_PP" >&2
    exit 0
    ;;
esac
