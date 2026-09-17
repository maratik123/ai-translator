# The single entry point every runner shares.
#
# `make verify` is the local aggregate: it runs every gate this project owns, in
# the order in which a failure is cheapest to read. CI never runs `verify` — it
# invokes the same sub-targets from its paths-filtered jobs, so a local run and a
# CI run cannot disagree about what any gate's command is.
#
# One gate CI reaches by another route, deliberately:
#   * actionlint — CI uses `reviewdog/action-actionlint@v1`, because the binary is
#     not preinstalled on `ubuntu-latest`. `make actionlint` is the local path.
#
# No recipe swallows a failure: SHELL/.SHELLFLAGS below put `pipefail` in force for
# every recipe, and no recipe absorbs a non-zero exit status.

SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c
.NOTPARALLEL:

# `tmp/` is pruned from the two find-based recipes: it is the ignored scratch
# directory every gate log and throwaway probe is written to, so a local
# `make verify` and a CI run — which never sees it — agree. Cargo needs no such
# teaching: a workspace compiles the crates its manifest names, so a stray
# source file under the scratch directory is not part of any build.

# WHILE THE WORKSPACE IS EMPTY every cargo gate below skips itself, loudly, and
# exits 0: there is no `Cargo.toml` at the root until the first crate lands. A
# skip is printed to stderr with the target's own name — never silent, because a
# gate that could not run is not a green tree either. The skips disappear by
# themselves with the commit that creates the workspace; no gate has to be
# switched on by hand.
CARGO_GUARD = if [ ! -f Cargo.toml ]; then printf 'make: no Cargo.toml at the repository root; %s skipped\n' "$@" >&2; exit 0; fi

# Hard file-size limits — raw lines, comments and blanks included.
#
# The src cap sits above the 1000 this ladder was inherited with, for a reason
# that is Rust's and not a preference: a unit test lives in the file it tests,
# inside its `#[cfg(test)]` module, so one cap covers code and its unit tests
# together. Integration tests under `tests/` get the wider one. Neither number
# has been measured against this tree — nothing is in it yet. Re-set them from
# the real distribution once crates exist, in a commit that says what it
# measured.
RS_MAX_LINES ?= 1200
RS_MAX_TEST_LINES ?= 1500

.PHONY: verify fmt-check build clippy doc-check test lock-check file-limits actionlint shellcheck cover-ratchet comment-refs panic-calls import-guard

verify: fmt-check build clippy doc-check test lock-check file-limits actionlint shellcheck comment-refs panic-calls import-guard

fmt-check:
	@$(CARGO_GUARD); cargo fmt --all --check

build:
	@$(CARGO_GUARD); cargo build --workspace --all-targets

clippy:
	@$(CARGO_GUARD); cargo clippy --workspace --all-targets -- -D warnings

# rustdoc with warnings denied is the doc-comment gate: a broken intra-doc link
# and a malformed doc attribute are errors here and invisible to every other
# target.
doc-check:
	@$(CARGO_GUARD); RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# The suite provisions its own database: a database-backed test starts a
# `pgvector/pgvector` container through testcontainers over the Podman socket
# and drops it on cleanup. `DATABASE_URL` is the application's connection, never
# the suite's — a test that reads it would run against the developer's own data.
test:
	@$(CARGO_GUARD); cargo test --workspace

# The lockfile gate: `--locked` fails rather than rewriting `Cargo.lock`, so a
# manifest edit committed without its lockfile is caught here instead of on the
# next machine.
lock-check:
	@$(CARGO_GUARD); cargo metadata --locked --format-version 1 >/dev/null

file-limits:
	find . -path ./.git -prune -o -path ./tmp -prune -o -path ./target -prune -o -name '*.rs' -exec awk \
	  '{n[FILENAME]++} END{rc=0; for (k in n) {lim=(k ~ /(^|\/)tests\//)?$(RS_MAX_TEST_LINES):$(RS_MAX_LINES); if (n[k]>lim) {printf "%s: %d lines exceeds hard limit %d\n", k, n[k], lim; rc=1}} exit rc}' {} +

actionlint:
	actionlint .github/workflows/*.yml

shellcheck:
	find . -path ./.git -prune -o -path ./tmp -prune -o -path ./target -prune -o -name '*.sh' -exec shellcheck -s bash {} +

# Check-only: never writes the ratchet file, never stages. The pre-commit hook
# runs the same script in raise mode. Not part of `verify` — it re-runs the
# whole suite under coverage instrumentation, and `verify` already ran it once.
cover-ratchet:
	.githooks/coverage-ratchet.sh --check

# The comment reference gate, over the whole tracked gated set: no comment in
# a gated file carries an outward reference.
comment-refs:
	bash ai-docs/scripts/check-comment-refs.sh

# Every panicking call in compiled, non-test source carries a row in the panic
# index. The same script backs the `PostToolUse` hook, which judges one file.
panic-calls:
	bash ai-docs/scripts/check-panic-calls.sh

# The dependency-direction gate: a crate named in the rule table never reaches a
# forbidden crate through its non-dev dependency graph — today, that a shipped
# binary never reaches the container-runtime crate a test-only database depends
# on.
import-guard:
	bash ai-docs/scripts/check-import-guard.sh
