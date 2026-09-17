#!/usr/bin/env bash
# Regression suite for the panic gate.
#
# WHAT IT PINS: that a panicking call in shipped code is still refused, that the
# marker still exempts one, that test code is still out of scope, and that a
# panicking name inside a string or a comment is not a call. The last two are
# the halves that decay into a gate which refuses honest code, and a gate people
# route around is worse than no gate.
#
# The fixtures live in quoted here-documents, so their payloads are content
# rather than source.
#
# Exit 0 = every fixture is decided as specified. Exit 1 = regression.

set -uo pipefail

usage() {
  cat <<'USAGE'
Usage:
  test-panic-calls.sh    run the whole suite; it takes no arguments
USAGE
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root" || exit 1

failures=0
sandbox=$(mktemp -d)
trap 'rm -rf "$sandbox"' EXIT

gate() { python3 "$repo_root/ai-docs/scripts/panic_calls.py" "$@" 2>&1; }

expect_red() {
  local f=$1 want=$2 out
  if out=$(gate "$f"); then
    printf 'FAIL [%s]: expected %s to be refused, the gate reported a clean file\n' "$f" "$want"
    failures=$((failures + 1))
    return
  fi
  printf '%s' "$out" | grep -qF "$want" || {
    printf 'FAIL [%s]: refused, but not for %s:\n%s\n' "$f" "$want" "$out"
    failures=$((failures + 1))
  }
}

expect_green() {
  local f=$1 why=$2 out
  if ! out=$(gate "$f"); then
    printf 'FAIL [%s]: expected a clean file (%s), the gate reported:\n%s\n' "$f" "$why" "$out"
    failures=$((failures + 1))
  fi
}

cd "$sandbox" || exit 1

cat > shipped.rs <<'FIXTURE'
fn f(v: Option<u8>) -> u8 {
    v.unwrap()
}
FIXTURE
expect_red shipped.rs ".unwrap()"

cat > macros.rs <<'FIXTURE'
fn f() {
    panic!("no");
    todo!();
    unreachable!();
}
FIXTURE
expect_red macros.rs "panic!"

cat > marked.rs <<'FIXTURE'
fn f(v: Option<u8>) -> u8 {
    // PANIC: the caller validated the option one line above and holds the lock
    v.unwrap()
}
FIXTURE
expect_green marked.rs "a marked call carries its justification in prose"

cat > marked_same_line.rs <<'FIXTURE'
fn f(v: Option<u8>) -> u8 {
    v.unwrap() // PANIC: the constant is checked at start-up
}
FIXTURE
expect_green marked_same_line.rs "the marker may sit on the call's own line"

cat > unit_test.rs <<'FIXTURE'
fn f(v: Option<u8>) -> u8 {
    v.unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::f;

    #[test]
    fn t() {
        assert_eq!(f(Some(1)), 1);
        let _ = "1".parse::<u8>().unwrap();
    }
}
FIXTURE
expect_green unit_test.rs "a module behind the test attribute is out of scope"

cat > not_a_call.rs <<'FIXTURE'
fn f() -> &'static str {
    // The word unwrap() in a comment is prose, not a call
    "a string mentioning .unwrap() and panic!()"
}
FIXTURE
expect_green not_a_call.rs "a panicking name inside a comment or a string is not a call"

# The instrument check: a file that is one bare panicking call must be refused.
# If it is not, every green case above passed on a gate that sees nothing.
cat > instrument.rs <<'FIXTURE'
fn f() { panic!("x"); }
FIXTURE
if gate instrument.rs >/dev/null 2>&1; then
  printf 'FAIL [instrument]: a bare panicking call read clean — the gate is not scanning, so the green cases above prove nothing\n'
  failures=$((failures + 1))
fi

cd "$repo_root" || exit 1
if [ "$failures" -eq 0 ]; then
  echo "panic gate: every fixture is decided as specified"
  exit 0
fi
printf 'panic gate: %d check(s) failed\n' "$failures"
exit 1
