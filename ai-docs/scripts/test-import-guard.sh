#!/usr/bin/env bash
# Regression suite for the dependency-direction gate.
#
# WHAT IT PINS: that a normal edge to a forbidden crate is still refused, that a
# dev edge to the same crate is still allowed, and that a transitive normal path
# is still found. The middle case is the one a rewrite loses — a gate that
# refuses the dev edge too would make the test database unusable, and the
# obvious repair is to switch the gate off.
#
# The fixtures are real workspaces built out of path dependencies, so the graph
# comes from the toolchain rather than from a parser written twice. They need no
# network: every crate in them is local.
#
# Exit 0 = every fixture is decided as specified. Exit 1 = regression.

set -uo pipefail

usage() {
  cat <<'USAGE'
Usage:
  test-import-guard.sh    run the whole suite; it takes no arguments
USAGE
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root" || exit 1

if ! command -v cargo >/dev/null 2>&1; then
  echo "NOTE: import-guard fixtures SKIPPED — no cargo here, so no graph can be resolved. This is a case that did not run, not a case that passed."
  exit 0
fi

failures=0
sandbox=$(mktemp -d)
trap 'rm -rf "$sandbox"' EXIT

# build_case <dir> <edge-section> [middle]
# Builds a workspace whose binary depends on the forbidden crate through the
# named section, directly or through one hop.
build_case() {
  local dir=$1 section=$2 middle=${3:-}
  mkdir -p "$dir/app/src" "$dir/testcontainers/src"
  cat > "$dir/Cargo.toml" <<TOML
[workspace]
resolver = "2"
members = ["app", "testcontainers"${middle:+, \"mid\"}]
TOML
  cat > "$dir/testcontainers/Cargo.toml" <<'TOML'
[package]
name = "testcontainers"
version = "0.1.0"
edition = "2021"
TOML
  printf 'pub fn start() {}\n' > "$dir/testcontainers/src/lib.rs"
  printf 'fn main() {}\n' > "$dir/app/src/main.rs"
  if [ -n "$middle" ]; then
    mkdir -p "$dir/mid/src"
    cat > "$dir/mid/Cargo.toml" <<'TOML'
[package]
name = "mid"
version = "0.1.0"
edition = "2021"

[dependencies]
testcontainers = { path = "../testcontainers" }
TOML
    printf 'pub fn m() {}\n' > "$dir/mid/src/lib.rs"
    cat > "$dir/app/Cargo.toml" <<TOML
[package]
name = "app"
version = "0.1.0"
edition = "2021"

[$section]
mid = { path = "../mid" }
TOML
  else
    cat > "$dir/app/Cargo.toml" <<TOML
[package]
name = "app"
version = "0.1.0"
edition = "2021"

[$section]
testcontainers = { path = "../testcontainers" }
TOML
  fi
  ( cd "$dir" && cargo generate-lockfile --offline >/dev/null 2>&1 )
}

run_gate() { ( cd "$1" && python3 "$repo_root/ai-docs/scripts/import_guard.py" 2>&1 ); }

# --- RED: a normal edge, direct and transitive -------------------------------
build_case "$sandbox/direct" dependencies
if out=$(run_gate "$sandbox/direct"); then
  printf 'FAIL [direct]: a binary depending on the forbidden crate was not refused\n%s\n' "$out"
  failures=$((failures + 1))
elif ! printf '%s' "$out" | grep -q 'app: reaches testcontainers'; then
  printf 'FAIL [direct]: refused without naming the crate and the chain\n%s\n' "$out"
  failures=$((failures + 1))
fi

build_case "$sandbox/transitive" dependencies mid
if out=$(run_gate "$sandbox/transitive"); then
  printf 'FAIL [transitive]: a path through one hop was not refused\n%s\n' "$out"
  failures=$((failures + 1))
elif ! printf '%s' "$out" | grep -q 'app -> mid -> testcontainers'; then
  printf 'FAIL [transitive]: refused without printing the chain that reaches it\n%s\n' "$out"
  failures=$((failures + 1))
fi

# --- GREEN: the dev edge, which is what a test database is --------------------
build_case "$sandbox/dev" dev-dependencies
if ! out=$(run_gate "$sandbox/dev"); then
  printf 'FAIL [dev]: a dev-dependency on the forbidden crate was refused; that edge is the gate\047s whole point\n%s\n' "$out"
  failures=$((failures + 1))
fi

# --- GREEN: a workspace with no binary has nothing to walk --------------------
mkdir -p "$sandbox/nobin/lib/src"
cat > "$sandbox/nobin/Cargo.toml" <<'TOML'
[workspace]
resolver = "2"
members = ["lib"]
TOML
cat > "$sandbox/nobin/lib/Cargo.toml" <<'TOML'
[package]
name = "onlylib"
version = "0.1.0"
edition = "2021"
TOML
printf 'pub fn f() {}\n' > "$sandbox/nobin/lib/src/lib.rs"
( cd "$sandbox/nobin" && cargo generate-lockfile --offline >/dev/null 2>&1 )
if ! out=$(run_gate "$sandbox/nobin"); then
  printf 'FAIL [nobin]: a workspace with no binary target was refused\n%s\n' "$out"
  failures=$((failures + 1))
elif ! printf '%s' "$out" | grep -q 'no binary target'; then
  printf 'FAIL [nobin]: the empty case passed without naming why\n%s\n' "$out"
  failures=$((failures + 1))
fi

if [ "$failures" -eq 0 ]; then
  echo "import-guard: every fixture is decided as specified"
  exit 0
fi
printf 'import-guard: %d check(s) failed\n' "$failures"
exit 1
