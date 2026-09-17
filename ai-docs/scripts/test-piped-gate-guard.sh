#!/usr/bin/env bash
# Regression suite for the PreToolUse piped-gate hook guard.
#
# The guard blocks a gate piped into tail/head without pipefail, because bash
# reports the LAST pipeline stage's exit status and a RED gate then records as
# green. The project rule text advertises that hook as the enforcement for the
# whole class, so a silent narrowing of its alternation is a live
# instruction-file claim going false.
#
# Anti-drift: this suite runs the LIVE hook body, extracted with jq and
# executed as the program it is. There is no copied regex here, so there is
# nothing to drift. An edit that un-blocks a must-block case, or that starts
# blocking a must-allow case, fails here instead of being discovered by a
# blocked agent weeks later.
#
# Verdict convention: the body exits 2 to block a tool call. Any other exit
# status means the call proceeds.
#
# Known false positive, asserted deliberately: `make -n verify` piped into
# head is BLOCKED. `make` is matched as a class, not by target enumeration,
# because an anchored enumeration binds only when a target name follows `make`
# immediately: every shape that puts a flag in between leaks past such an
# enumeration, and bare `make` names no target at all. Every target this project's build entry point offers is a
# gate. A dry run executes nothing, so the cost is a loud refusal rather than a green
# record of a red gate. It is a fixture below so that a later "fix" which
# quietly un-blocks it fails this suite.
#
# Exit 0 = every fixture behaves as specified. Exit 1 = regression.

set -uo pipefail

usage() {
  cat <<'USAGE'
Usage:
  test-piped-gate-guard.sh    run the whole suite; it takes no arguments
USAGE
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root" || exit 1

settings=".claude/settings.json"
# Select the entry by a fixed substring of its message, never by index: an
# index silently re-points the moment another PreToolUse entry is added.
body=$(jq -r '.hooks.PreToolUse[].hooks[].command
  | select(contains("BLOCKED: gate piped through tail/head"))' "$settings")
[ -n "$body" ] || { echo "FAIL: piped-gate guard body not found in $settings"; exit 1; }

failures=0

# Feed one command string to the real body and report BLOCK or ALLOW.
verdict() {
  local payload rc
  payload=$(jq -n --arg c "$1" '{tool_input: {command: $c}}')
  printf '%s' "$payload" | bash -c "$body" >/dev/null 2>&1 && rc=0 || rc=$?
  if [ "$rc" -eq 2 ]; then echo BLOCK; else echo ALLOW; fi
}

check() {
  local got
  got=$(verdict "$2")
  if [ "$got" != "$1" ]; then
    printf 'FAIL: expected %s, got %s, for: %s\n' "$1" "$got" "$2"
    failures=$((failures + 1))
  fi
}

# verdict<TAB>command. BLOCK rows are the must-block class plus the known false
# positive; ALLOW rows are the must-allow class. Both directions are required:
# the block half alone is satisfied by a regex that matches everything.
while IFS=$'\t' read -r want cmd; do
  case "${want:-}" in ''|'#'*) continue ;; esac
  check "$want" "$cmd"
done <<'FIXTURES'
# --- must block: this project's gates ---
BLOCK	cargo fmt --all --check | tail -5
BLOCK	make verify | tail -5
BLOCK	make clippy | head -30
BLOCK	make test | tail -5
BLOCK	make | tail -5
BLOCK	make -s verify | tail -5
BLOCK	make -B clippy | tail -5
BLOCK	make -C . verify | tail -5
BLOCK	make -j4 test | tail -5
BLOCK	make -f Makefile verify | tail -5
# --- must block: the cargo class, each sub-command the gates use ---
BLOCK	cargo test --workspace | tail -5
BLOCK	cargo clippy --workspace --all-targets | tail -5
BLOCK	cargo build --workspace | head -20
BLOCK	cargo doc --workspace --no-deps | tail -3
BLOCK	cargo llvm-cov --workspace --summary-only | tail -3
BLOCK	rustfmt --check src/lib.rs | tail -3
# --- must block: the accepted false positive (see the header) ---
BLOCK	make -n verify | head
# --- must allow ---
ALLOW	set -o pipefail; make verify | tail -5
ALLOW	make --help | head
ALLOW	cargo --version | head -2
ALLOW	cargo tree -e no-dev | head -20
ALLOW	git log --oneline | head -20
ALLOW	cargo test --workspace > tmp/gate.log 2>&1 && echo GATE-RED
ALLOW	grep -E "^(error|test result)" tmp/gate.log | head -5
ALLOW	cmake --build . | tail -5
ALLOW	echo makezero | tail -1
ALLOW	make verify
FIXTURES

# Both carve-outs must survive byte-for-byte.
for carveout in '(^|[[:space:]])(--help|-h)([[:space:]]|$)' \
                '(^|[;&|[:space:]])set[[:space:]]+-o[[:space:]]+pipefail'; do
  grep -qF -- "$carveout" <<<"$body" && continue
  printf 'FAIL: carve-out no longer present verbatim: %s\n' "$carveout"
  failures=$((failures + 1))
done

# ...and no third carve-out has been added. A `-n` carve-out in particular is
# rejected on the record: the carve-outs are whole-command greps, so it would
# exempt `tail -n 5`, the canonical spelling of `tail -5`.
negations=$(grep -o -- '! printf' <<<"$body" | wc -l)
if [ "$negations" -ne 2 ]; then
  printf 'FAIL: expected exactly 2 carve-outs, found %s\n' "$negations"
  failures=$((failures + 1))
fi

if [ "$failures" -eq 0 ]; then
  echo "piped-gate guard: all fixtures behave as specified"
  exit 0
fi
printf 'piped-gate guard: %d check(s) failed\n' "$failures"
exit 1
