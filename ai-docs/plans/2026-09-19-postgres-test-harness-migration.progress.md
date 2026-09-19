# Progress: Postgres test harness and the vector-extension migration — ACTIVE
_Updated: 2026-09-19 10:22_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-09-19-postgres-test-harness-migration
**base_commit:** 1b0b287092e010b5a8794a28b2936190546fe1da
**Last build:** not run
**Issue:** #13
**Spec:** ai-docs/plans/2026-09-19-postgres-test-harness-migration.spec.md
**current_step:** Step 8 — subtask 1 of 5 complete
**last_passed_gate:** make verify components run individually for subtask 1 (build, test, fmt --check, clippy, doc-check, lock-check, import-guard, panic-calls, comment-refs, file-limits) — all GREEN; cover-ratchet raised 0.00% -> 89.47% at commit 0d40e6a
**entry_args:** 13

## Next action

**Do this immediately:** hand off Group A (subtasks 1–2) to `code-writer` per the design's `## Handoff plan`, via `/context-reset`.

## Subtasks

- [x] 1. Dependency set, first migration, embedded migrator — `Cargo.toml`, `Cargo.lock`, `crates/core/Cargo.toml`, `crates/core/migrations/0001_vector_extension.sql`, `crates/core/src/lib.rs` — commit 0d40e6a
- [ ] 2. Container harness and the database-backed test target — `crates/core/Cargo.toml`, `crates/core/tests/support/mod.rs`, `crates/core/tests/database.rs`  ← CURRENT
- [ ] 3. Correct the statements this diff falsifies (D12)
- [ ] 4. Record the harness decision where it will be looked for
- [ ] 5. Amend the corpus row and tick what this task closes in full (D13)

Groups per the design's `## Handoff plan`: **Group A** = 1–2 (code, `code-writer`, sonnet/medium pinned in frontmatter); **Group B** = 3–5 (instructions/harness, `general-purpose`, inherit).

## Decisions log

Append-only, one line per non-trivial decision. Each line is prefixed with the step or phase that made it. Never edit or remove prior entries.

- **Step 1-5**: the interview ran three rounds; the owner settled CI reach («И в CI»), image version («Как есть») and struck AC5–AC7 with their Scope twins under a strict reading of spec-writer Rule 1.
- **Step 6**: the corpus row `docs/03-storage.md:7` names three mechanisms that cannot hold together — verified independently by the orchestrator against downloaded crate sources, not taken from the delegate's report.
- **Step 6**: the owner authorised the substitute mechanism AND the corpus amendment in this same pull request (answer 4.1); it is design work and originates no spec row.
- **Step 7**: design-review round 1 returned GO with four issues and three recommendations; all seven are design-internal, so `design-writer` folded them in and design-review did not run again.
- **Step 8 (subtask 1)**: `testcontainers` pinned to `0.27.3`, not the `0.28.0` the design measured — `testcontainers-modules` 0.15.0's own published manifest requires `testcontainers = "0.27.0"` (checked directly in its downloaded `Cargo.toml`, `[dependencies.testcontainers] version = "0.27.0"`), so `0.28.0` and `testcontainers-modules 0.15.0` cannot resolve together (`cargo add` reported the bollard-stubs conflict verbatim). No newer `testcontainers-modules` exists on crates.io (0.15.0 is still the newest). Re-verified against the resolved 0.27.3 source that every API surface the design's D3/D4/D5 cite still exists there: `ImageExt::with_name`/`with_tag` (`image_ext.rs:60,66`), `async_container::rm` at `:205` and the `Drop` removal branch, `async_drop.rs:17` `Handle::current`, no `ryuk`/`reaper` match anywhere in the crate, and `testcontainers-modules`' `postgres::mod.rs` still carries `with_host_auth` and the dual-stream `ready_conditions`. `libtest-mimic` (0.8.2) and `tokio` (1.53.1) resolved as the design named without conflict.
- **Step 8 (subtask 1)**: the Test Design's stated migration description `vector_extension` does not match sqlx's actual behaviour — `sqlx-core-0.9.0/src/migrate/source.rs:220-222` replaces `_` with a space when deriving a migration's description from its file name, so `0001_vector_extension.sql` yields the description `"vector extension"` (observed directly by running the unit test red, then green after correcting the literal). The unit test asserts the observed value with a comment naming why, rather than weakening the assertion or renaming the file.

## GO notes

| # | round | note | kind | route | resolution |
|---|-------|------|------|-------|------------|
| G1 | 1 | § *What the gates will read afterwards* allows the migration file to state *why* the extension is created, while § Test Design asserts the embedded migration's trimmed text is exactly the statement | design-internal | folded | D6 decides it in favour of the file — statement plus one newline, no comment at all; § Test Design names both cheap repairs as forfeits @ d29e0e4 |
| G2 | 1 | D2/D3 make the container's removal `main`'s job, but `Trial::test` requires `'static`, so no trial can borrow a harness owned by `main` | design-internal | folded | D2 records `Arc<Harness>`, never leaked, never in a `static`, container in a take-once slot; D3 says why a plain field cannot work @ d29e0e4 |
| G3 | 1 | Trial concurrency is never addressed — `run` executes trials on parallel threads by default | design-internal | folded | D2 keeps the default and fixes names by atomic fetch-and-add; new § Test Design trial `concurrent_requests_get_distinct_databases` drives the shared path @ d29e0e4 |
| G4 | 1 | "Gates are already written … and none of them needs changing" is a universal negative resting on an unrecorded check of the CI paths filter | design-internal | folded | § *What the gates will read afterwards* records `.github/workflows/ci.yml:36,39-49,86-89`; no filter entry added — the check is recorded, not a change @ d29e0e4 |
| G5 | 1 | the `[measured 0daa273 · …]` tag in § *What the tree holds today* carries a commit but no `path:lines` | design-internal | folded | tag now reads `[measured 128c2fb:crates/{shared,core,cli,migrate}/Cargo.toml:8 · …]` @ d29e0e4 |
| G6 | 1 | consider surfacing `docs/09-build-and-deploy.md:12` to the owner separately after this task | design-internal | folded | D10 left exactly as it was, per the recommendation; the row is carried to the owner as a follow-up outside this task's authorised single corpus amendment @ d29e0e4 |
| G7 | 1 | subtask 2's red observation presumes the image coordinates are reachable for a temporary mutation | design-internal | folded | § Test Design says the image is changed by editing the harness's own coordinates in place and reverting, with `git diff --name-only` confirming the revert; no configuration surface grown @ d29e0e4 |

## Key discoveries (don't re-investigate)

- `#[sqlx::test]` has exactly one connection source: `dotenvy::var("DATABASE_URL")` at `sqlx-postgres-0.9.0/src/testing/mod.rs:42` and `:93`, the only two environment reads in that file. The attribute parser accepts `fixtures`, `migrations`, `migrator` and nothing else — catch-all error text at `sqlx-macros-core-0.9.0/src/test_attr.rs:298`.
- `testcontainers` 0.28.0 carries no Ryuk: `ryuk|reaper` is empty across 68 files with a matched control. Only a `watchdog` feature reaping on SIGTERM/SIGINT/SIGQUIT exists.
- A value parked in a `static` is never dropped — executed, both directions: the value owned by `main` printed its `Drop`, the one in a `OnceLock` did not, in two runs.
- `sqlx` keeps a migration file's bytes verbatim: `sqlx-core-0.9.0/src/migrate/source.rs:225` is `fs::read_to_string`, and the only `--` handling at `:234` detects `-- no-transaction` rather than stripping. A comment line lands in both the embedded text and the checksum.
- `libtest_mimic` runs trials in parallel by default — `num_threads` falls through to `available_parallelism()` at `libtest-mimic-0.8.2/src/lib.rs:538`, spawned as scope threads at `:564`.
- `sqlx` and `testcontainers` are not yet dependencies anywhere: absent from every manifest, from `Cargo.lock`, and from the registry cache. Subtask 1 introduces them.

## AC Status

| AC | Status |
|----|--------|
| AC1 | NOT_TESTED |
| AC2 | NOT_TESTED |
| AC3 | NOT_TESTED |
| AC4 | NOT_TESTED |
| AC5 | NOT_TESTED |

## Review register

| id | raised | severity | status | verifying command |
|----|--------|----------|--------|-------------------|

## Files touched

- `Cargo.toml` — populated `[workspace.dependencies]` (sqlx, testcontainers, testcontainers-modules, libtest-mimic, tokio), rewrote the now-false empty-table comment
- `Cargo.lock` — regenerated via `cargo build`/`cargo update -p testcontainers --precise 0.27.3`, never hand-edited
- `crates/core/Cargo.toml` — `sqlx` as a normal dependency; `testcontainers`, `testcontainers-modules`, `libtest-mimic`, `tokio` as dev-dependencies, all `.workspace = true`
- `crates/core/migrations/0001_vector_extension.sql` — new, single statement, no comment
- `crates/core/src/lib.rs` — `pub static MIGRATOR` + `#[cfg(test)]` unit test covering AC2's no-container half
