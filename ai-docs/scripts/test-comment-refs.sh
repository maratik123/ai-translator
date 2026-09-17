#!/usr/bin/env bash
# Regression suite for the comment-reference gate.
#
# WHAT IT PINS. Two halves, and the second is the one a gate loses silently:
# that the banned classes are still decided (RED cases), and that the scanner
# still knows a comment from a string literal (GREEN cases). A gate that has
# quietly stopped seeing comments reports a clean tree, which is exactly what a
# clean tree reports.
#
# The fixtures live in quoted here-documents, so the payloads are content rather
# than source: the gate scans this suite like any other tracked script, and a
# banned shape written outside a here-document here would be a finding against
# the suite itself.
#
# Exit 0 = every fixture is decided as specified. Exit 1 = regression.

set -uo pipefail

usage() {
  cat <<'USAGE'
Usage:
  test-comment-refs.sh    run the whole suite; it takes no arguments
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

gate() { python3 "$repo_root/ai-docs/scripts/comment_refs.py" "$@" 2>&1; }

# expect_red <file> <class> — the fixture is refused, and for the named class.
expect_red() {
  local f=$1 class=$2 out
  if out=$(gate "$f"); then
    printf 'FAIL [%s]: expected a finding of class %s, the gate reported a clean file\n' "$f" "$class"
    failures=$((failures + 1))
    return
  fi
  printf '%s' "$out" | grep -q ": $class: " || {
    printf 'FAIL [%s]: refused, but not for class %s:\n%s\n' "$f" "$class" "$out"
    failures=$((failures + 1))
  }
}

# expect_green <file> <why> — the fixture is clean, and the reason it is clean
# is what a regression would break.
expect_green() {
  local f=$1 why=$2 out
  if ! out=$(gate "$f"); then
    printf 'FAIL [%s]: expected a clean file (%s), the gate reported:\n%s\n' "$f" "$why" "$out"
    failures=$((failures + 1))
  fi
}

cd "$sandbox" || exit 1

# --- RED: one fixture per class ----------------------------------------------
cat > locator.rs <<'FIXTURE'
// The caller is at crates/core/src/lib.rs:42
fn f() {}
FIXTURE
expect_red locator.rs locator

cat > markdown.rs <<'FIXTURE'
// Rationale in docs/ARCHITECTURE.md
fn f() {}
FIXTURE
expect_red markdown.rs markdown-path

cat > ids.rs <<'FIXTURE'
// Satisfies AC3, decided as KD-7, raised as R2-1
fn f() {}
FIXTURE
expect_red ids.rs ac-id
expect_red ids.rs decision-anchor
expect_red ids.rs review-register-id

cat > section.sh <<'FIXTURE'
#!/usr/bin/env bash
# The shape is fixed in § 4 of the protocol
echo x
FIXTURE
expect_red section.sh section

cat > issue.sql <<'FIXTURE'
-- Added for #431
SELECT 1;
FIXTURE
expect_red issue.sql issue

cat > repopath.yml <<'FIXTURE'
# Mirrors the filter in .github/workflows/ci.yml
key: value
FIXTURE
expect_red repopath.yml repo-path

cat > url.rs <<'FIXTURE'
// Upstream discussion: https://example.invalid/thread
fn f() {}
FIXTURE
expect_red url.rs url

# --- GREEN: what the scanner must NOT mistake for a comment ------------------
cat > strings.rs <<'FIXTURE'
fn f() -> (&'static str, &'static str, char) {
    let a = "http://127.0.0.1:8080/docs/ARCHITECTURE.md";
    let b = r#"a raw string with // and docs/x.md inside"#;
    let c = '"';
    (a, b, c)
}
FIXTURE
expect_green strings.rs "a string literal is not a comment"

cat > heredoc.sh <<'FIXTURE'
#!/usr/bin/env bash
cat <<'EOF'
# docs/ARCHITECTURE.md inside a here-document body
EOF
n=$#
echo "$n"
FIXTURE
expect_green heredoc.sh "a here-document body and a parameter expansion are not comments"

cat > block.yml <<'FIXTURE'
jobs:
  step:
    run: |
      # docs/ARCHITECTURE.md inside a literal block scalar
      echo hi
FIXTURE
expect_green block.yml "a block scalar's body is content, not a comment"

cat > directive.sh <<'FIXTURE'
#!/usr/bin/env bash
# shellcheck disable=SC2016
echo 'x'
FIXTURE
expect_green directive.sh "a machine-read directive is exempt in full"

cat > todo.rs <<'FIXTURE'
// TODO(#12) covered by the tracking issue form
fn f() {}
FIXTURE
expect_green todo.rs "the tracking-issue form is the one permitted issue reference"

# --- The instrument check ----------------------------------------------------
# Every green case above also passes when the scanner has stopped finding
# comments at all. Drive one file that is nothing BUT a banned comment: if that
# reads clean, none of the green cases measured anything.
cat > instrument.rs <<'FIXTURE'
// docs/ARCHITECTURE.md
fn f() {}
FIXTURE
if gate instrument.rs >/dev/null 2>&1; then
  printf 'FAIL [instrument]: a file whose only content is a banned comment read clean — the scanner is not finding comments, so the green cases above prove nothing\n'
  failures=$((failures + 1))
fi

cd "$repo_root" || exit 1
if [ "$failures" -eq 0 ]; then
  echo "comment-refs: every fixture is decided as specified"
  exit 0
fi
printf 'comment-refs: %d check(s) failed\n' "$failures"
exit 1
