#!/usr/bin/env bash
# Regression suite for check-spec-shape.sh.
#
# The must-hit rows are verbatim lines from the round-1 spec of 2026-09-08
# (recovered from the spec-writer's transcript; the file itself was untracked
# and is gone): the five bare `file:line` references in `## Source conflicts`
# that the owner rejected on sight, the `### Sizing` heading, and a `[source:`
# annotation whose command counts. The must-pass rows are the forms the guard
# exists to leave alone: a pinned coordinate, a section-or-symbol locator, a
# URL, a clock time, a code fence, and the visible `spec-shape: example`
# exemption. Both directions are asserted -- a regex that matches everything
# satisfies the first half alone.
#
# Exit 0 = every fixture behaves as specified. Exit 1 = regression.

set -uo pipefail

usage() {
  cat <<'USAGE'
Usage:
  test-spec-shape.sh    run the whole suite; it takes no arguments
USAGE
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root" || exit 1
guard=ai-docs/scripts/check-spec-shape.sh
[ -f "$guard" ] || { echo "FAIL: $guard missing"; exit 1; }

scratch=$(mktemp -d "${TMPDIR:-/tmp}/spec-shape.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
failures=0

# check <HIT|PASS> <one spec line>
check() {
  local want=$1 line=$2 f="$scratch/one.spec.md" rc got
  printf '%s\n' "$line" > "$f"
  bash "$guard" "$f" >/dev/null 2>&1; rc=$?
  if [ "$rc" -eq 1 ]; then got=HIT; else got=PASS; fi
  if [ "$got" != "$want" ]; then
    printf 'FAIL: expected %s, got %s (exit %s), for: %s\n' "$want" "$got" "$rc" "$line"
    failures=$((failures + 1))
  fi
}

while IFS=$'\t' read -r want text; do
  case "${want:-}" in ''|'#'*) continue ;; esac
  check "$want" "$text"
done <<'FIXTURES'
# --- must hit: the five Source-conflicts lines of the rejected round-1 spec ---
HIT	2. **`ai-docs/doc-convention.md` DOC-4 vs. the ban.** `ai-docs/doc-convention.md:41` —
HIT	   `ai-docs/rust-api-naming.md:7` — *"its doc comment MUST state (a) the precondition, and (b)
HIT	   — **Wrong** — the comment is the contract"*. `ai-docs/doc-convention.md:35` — *"A function
HIT	   returning a sentinel error names it"*. `ai-docs/doc-convention.md:55` — *"No `TODO`
HIT	   `ai-docs/rust-api-naming.md:13` — the AXIOM row
# --- must hit: the stored-tally shapes ---
HIT	### Sizing
HIT	## Sizing
HIT	[source: f5b236a · `git ls-files '*.rs' | xargs grep -hE '^[[:space:]]*//' | grep -cE 'AC[0-9]'`]
HIT	Thirteen scripts [source: a386fe7 · `git ls-files '*.sh' | xargs grep -l Usage | wc -l`].
HIT	shapes [source: 76a7b41 · `grep -oE "//[a-z ]*" Cargo.toml | sort | uniq -c`]
# --- must hit: bare references in other extensions and with a range ---
HIT	see `crates/core/src/limit.rs:190` for the ramp
HIT	(`ai-docs/rust-test-conventions.md:42-44`), in a container
HIT	the hook in .claude/settings.json:37 blocks it
FIXTURES

while IFS=$'\t' read -r want text; do
  case "${want:-}" in ''|'#'*) continue ;; esac
  check "$want" "$text"
done <<'FIXTURES'
# --- must pass: the forms the guard leaves alone ---
PASS	[source: f5b236a:clippy.toml § lints · `sed -n "/\[lints\]/,/^\[/p" clippy.toml`]
PASS	[source: f5b236a:crates/core/src/config.rs § ErrMissingToken · `ast-index symbol ErrMissingToken`]
PASS	measured at f5b236a:crates/core/src/limit.rs:190 (coordinates as of that commit)
PASS	[source: pgvector@0.8.6/README.md:40-48 · `sed -n '40,48p' $(cargo pkgid pgvector | sed 's|.*#||')/README.md`]
PASS	the convention lives in `ai-docs/learnings.md` (search it for "names a **symbol**")
PASS	see https://docs.github.com/en/rest/issues.md:1 for the shape
PASS	the run started at 12:30 and the file is ai-docs/x.md
PASS	| A code-line reference | `crates/migrate/src/lib.rs:16` | <!-- spec-shape: example -->
PASS	- **`.env.example`** loses its header; `config/balance.yaml` gains prose
PASS	AC1 | No comment line in a gated file carries a reference of a banned class.
PASS	[source: 76a7b41 · `git ls-files '*.sh' | xargs grep -lE "^#.*Usage:"`]
FIXTURES

# --- fenced code is skipped even when it carries a bare reference ---
f="$scratch/fence.spec.md"
printf '%s\n' '```' 'crates/core/src/limit.rs:190: undefined: x' '```' > "$f"
bash "$guard" "$f" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 0 ] || { echo "FAIL: fenced code block should be skipped (exit $rc)"; failures=$((failures + 1)); }

# --- default mode: only lines this branch added are judged ---
# Run in a scratch repo whose master carries an old-convention spec; the branch
# adds a clean line to it. Old bare refs must not fire; a new one must.
repo="$scratch/repo"; mkdir -p "$repo"
git -C "$repo" init -q -b master . && git -C "$repo" config user.email t@t && git -C "$repo" config user.name t
mkdir -p "$repo/ai-docs/plans/done"
cat > "$repo/ai-docs/plans/done/old.spec.md" <<'OLD'
# old
legacy `AGENTS.md:92` reference
OLD
git -C "$repo" add -A && git -C "$repo" commit -q -m base
git -C "$repo" checkout -q -b feat
cat >> "$repo/ai-docs/plans/done/old.spec.md" <<'NEW'
new clean line [source: 1234567:AGENTS.md § Workflow · `grep -n Workflow AGENTS.md`]
NEW
git -C "$repo" commit -q -am add-clean
cp "$guard" "$repo/guard.sh"
(cd "$repo" && bash guard.sh) >/dev/null 2>&1; rc=$?
[ "$rc" -eq 0 ] || { echo "FAIL: default mode fired on a legacy line nobody touched (exit $rc)"; failures=$((failures + 1)); }
cat >> "$repo/ai-docs/plans/done/old.spec.md" <<'NEW'
new bare `crates/core/src/x.rs:5` reference
NEW
git -C "$repo" commit -q -am add-bare
(cd "$repo" && bash guard.sh) >/dev/null 2>&1; rc=$?
[ "$rc" -eq 1 ] || { echo "FAIL: default mode missed a bare reference this branch added (exit $rc)"; failures=$((failures + 1)); }

if [ "$failures" -eq 0 ]; then
  echo "spec-shape guard: all fixtures behave as specified"
  exit 0
fi
printf 'spec-shape guard: %d check(s) failed\n' "$failures"
exit 1
