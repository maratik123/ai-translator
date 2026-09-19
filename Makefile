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

# Hard file-size limits — raw lines, comments and blanks included.
#
# The src cap sits above the 1000 this ladder was inherited with, for a reason
# that is Rust's and not a preference: a unit test lives in the file it tests,
# inside its `#[cfg(test)]` module, so one cap covers code and its unit tests
# together. Integration tests under `tests/` get the wider one. Neither number
# has been measured against this tree: the crates carry their first code now,
# and a migration with the harness that exercises it is not the real
# distribution the bands wait for. Re-set them from that distribution once the
# crates carry the code they are for, in a commit that says what it measured.
RS_MAX_LINES ?= 1200
RS_MAX_TEST_LINES ?= 1500

.PHONY: verify fmt-check build clippy doc-check test lock-check file-limits actionlint shellcheck cover-ratchet comment-refs panic-calls import-guard

verify: fmt-check build clippy doc-check test lock-check file-limits actionlint shellcheck comment-refs panic-calls import-guard

fmt-check:
	cargo fmt --all --check

build:
	cargo build --workspace --all-targets

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

# rustdoc with warnings denied is the doc-comment gate: a broken intra-doc link
# and a malformed doc attribute are errors here and invisible to every other
# target.
doc-check:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# The suite provisions its own database: a database-backed test starts a
# `pgvector/pgvector` container through testcontainers over the Podman socket
# and drops it on cleanup. `DATABASE_URL` is the application's connection, never
# the suite's — a test that reads it would run against the developer's own data.
test:
	cargo test --workspace

# The lockfile gate: `--locked` fails rather than rewriting `Cargo.lock`, so a
# manifest edit committed without its lockfile is caught here instead of on the
# next machine.
lock-check:
	cargo metadata --locked --format-version 1 >/dev/null

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
