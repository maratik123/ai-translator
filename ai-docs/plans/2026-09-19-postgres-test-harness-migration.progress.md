# Progress: Postgres test harness and the vector-extension migration — ACTIVE
_Updated: 2026-09-19 10:22_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-09-19-postgres-test-harness-migration
**base_commit:** 1b0b287092e010b5a8794a28b2936190546fe1da
**Last build:** PASS
**Issue:** #13
**Spec:** ai-docs/plans/2026-09-19-postgres-test-harness-migration.spec.md
**current_step:** Step 8 — Group A complete and reconciled with the design; Group B (subtasks 3-5) next
**last_passed_gate:** make verify (full) | 2026-09-19T11:52:19Z | f1630d3
**entry_args:** 13

## Next action

**Do this immediately:** hand off Group B (subtasks 3-5) to `general-purpose` per the design's `## Handoff plan`, via `/context-reset`.

## Subtasks

- [x] 1. Dependency set, first migration, embedded migrator — `Cargo.toml`, `Cargo.lock`, `crates/core/Cargo.toml`, `crates/core/migrations/0001_vector_extension.sql`, `crates/core/src/lib.rs` — commit 0d40e6a
- [x] 2. Container harness and the database-backed test target — `crates/core/Cargo.toml`, `crates/core/tests/support/mod.rs`, `crates/core/tests/database.rs` — commit df64b2a
- [ ] 3. Correct the statements this diff falsifies (D12)  ← CURRENT (Group B)
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
- **Step 8 (subtask 2)**: sqlx 0.9.0's new `SqlSafeStr` audit rejected a dynamic `&String` in `sqlx::query`; resolved with `sqlx::AssertSqlSafe` (justified inline: the interpolated name is never caller-supplied — it is the harness's own fixed-prefix-plus-counter text) for `CREATE DATABASE`, and with a bound `$1` parameter instead of string interpolation for the vector-literal round-trip query, avoiding the audit entirely there.
- **Step 8 (subtask 2)**: podman's own `podman pull docker.io/pgvector/pgvector:pg18` failed once against this session's live environment with `storage-untar: error while loading shared libraries: libsubid.so.5` (the host ships `libsubid.so.6`, not `.5`) while pulling a layer that needs subordinate-ID mapping; a plain `podman pull docker.io/library/alpine:3.19` succeeded in the same session. Running `podman pull` for the exact pgvector coordinates directly (outside testcontainers) succeeded and populated the local image cache; every subsequent `cargo test --workspace --test database` run in this session passed cleanly against the cached image. Recorded here rather than in a corpus or rule file — no repository text was touched to work around it, and no environment file outside the project root was edited.
- **Step 8 (subtask 2)**: `make verify`'s pre-commit hook enforces the comment-reference gate on `tests/`, and the first commit attempt was rejected for 13 outward references — bare `(D1)`/`(D2)`/`(D3)`/`(D11)` decision anchors, bare `AC1`/`AC3`/`AC4` acceptance-criterion ids, and one `AGENTS.md` § *Code Style* markdown-path-plus-section citation, all in doc comments this subtask wrote. Rewritten to state the same content — what the trial checks, why the teardown is driven from inside the runtime, why a result is reported rather than discarded — without a pointer outside the comment; re-run of `make comment-refs` came back clean before the successful commit at `df64b2a`.
- **Step 8 (Group A red observations, required before the last commit, AGENTS.md § Patterns 2)**: all four performed and reverted, `git diff --name-only` confirming each revert landed cleanly before the commit that followed.
  1. Image coordinates temporarily swapped to `postgres:18` (plain, no pgvector) — `vector_type_is_usable` failed at the migration step itself (`error returned from database: extension "vector" is not available`), exit 101. Confirms the check is sensitive to the image actually carrying the extension, not merely to the query executing.
  2. `DOCKER_HOST` pointed at a nonexistent socket path for one run — `main` panicked with `Client(Init(SocketNotFoundError(...)))`, exit 101; no test was skipped or silently reported as zero.
  3. `create_database`'s counter temporarily short-circuited to always return id `0` — `each_trial_gets_its_own_database` failed with `database "reader_core_test_0" already exists`, exit 101.
  4. A temporary always-failing trial appended to the target's trial list, run as the built binary directly (`target/debug/deps/database-<hash>`) rather than through `cargo test` (so the process's own exit status is observable) — exit 101, and `podman ps -a` showed no `pgvector/pgvector` container present after the process returned, confirming the container is removed on the failing path.

- **Step 8 (orchestrator)**: re-ran the gates rather than accepting the delegate's green — `cargo build --workspace --all-targets` exit 0 with 0 errors, and `make verify` exit 0 with 0 errors and every `test result` line ok (5 database trials, 1 unit test). Compiler diagnostics for `SqlSafeStr`/`sqlx::query` seen before the re-run were stale intermediate state, not a defect in the committed tree.
- **Step 8 (orchestrator)**: branch pushed to `origin` on the first group return, per the visibility rule; no pull request exists yet.
- **Step 8 (Design Amendment)**: the `testcontainers` 0.27.3 deviation is real and verified independently against the live registry — `testcontainers-modules` 0.15.0 declares `testcontainers ^0.27.0` for both normal and dev kinds, and 0.15.0 is the newest published. The owner chose amendment WITH a re-run of design-review (answer 5.1), declining the per-instance exemption.

- **Step 7 (round 2)**: design-review returned ITERATE — two major, two minor, four recommendations. Both majors re-verified by the orchestrator against the shipped code before routing: the design's `vector_extension` description contradicts `crates/core/src/lib.rs:27` (`"vector extension"`), and `container_starts` is initialised to 1 at `crates/core/tests/support/mod.rs:74` and never mutated anywhere under `crates/` (grep empty against a matching constructed control over 6 files), so `crates/core/tests/database.rs:196` asserts a condition with no reachable failure mode.
- **Step 7 (round 2)**: the dead-assertion finding implies a `.rs` fix to files Group A already committed. Routed design-first, because the design's choice between incrementing at the start site and striking the bookkeeping half is what decides which code fix is correct.

- **Step 7 (fix round)**: the dead start-count assertion is fixed per amended D11 — a process-wide `static CONTAINER_STARTS`, zero at rest, incremented at the site that awaits the container start; the harness field is gone. Authored by a Mode B delegate, diff read and committed by the orchestrator.
- **Step 7 (fix round)**: the assertion was shown RED by the orchestrator independently, not on the delegate's word — a duplicated `fetch_add` made the trial fail with `the harness recorded 2 container starts, expected 1`, exit 101; reverted from a `tmp/` copy, and the unmutated five trials then passed, exit 0. Both directions observed, which is what the instrument rule requires. **The mutant was a substitution, recorded as such:** the design prescribes adding a second harness start to `main` or moving the start into the per-database path, and what was run instead was a duplicated `fetch_add` at the start site. It is equivalent for what the observation exists to establish — the assertion reads a live counter and fails when the count is wrong — and slightly stronger in one respect, since the observed value was 2 rather than 0, which also shows the increment site executes exactly once per start. It is weaker in one respect: it does not exercise a second real container start, so it says nothing about teardown under that condition. Anyone reading this row should not take the prescribed observation as having been performed verbatim.

- **Step 7 (round 3)**: design-review returned ITERATE at round 3 of 3 — one major (D11's closing paragraph is present-tense false of HEAD after the counter fix landed), three minor, one note. All re-verified by the orchestrator against the files before routing. Cap exhausted, so the owner was asked rather than a bypass invented; the owner raised it — **cap: 4 (was 3)**.

- **Step 7 (round 4, GO)**: the design's PRESCRIBED red observation was performed by the orchestrator, closing the substitution gap recorded earlier. A second `Harness::start()` added to `main` gave exit 101 and `the harness recorded 2 container starts, expected 1`; reverted from a `tmp/` copy, `git diff --name-only` then listed no `.rs` file, and the five trials passed again at exit 0.
- **Step 7 (round 4, GO)**: the half the substituted mutant could not reach was observed too, and it is the design's own reasoning made visible — the second container, never entered into the take-once slot, was NOT removed, and its handle's destructor panicked at `testcontainers-0.27.3/src/core/async_drop.rs:17` with `there is no reactor running`. Two orphaned containers (one from this probe, one from an earlier delegate's) were found running and removed by explicit id; the developer's own unrelated container was left alone.

- **Step 7 (round 4, GO)**: folding the GO notes surfaced a code non-compliance nobody had asked about — `main` carried the shutdown's `Result` out through a panicking call, which substitutes its own process status for the runner's verdict. The panic gate cannot see it (everything under `tests/` is out of its scope by position), so only the amended D3 catches it. Fixed: the runner's code is captured before teardown, a removal failure goes to the error channel and raises a green run to a failing status, and never replaces a status the trials already earned.

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
| AC1 | PASS — `vector_type_is_usable` (commit df64b2a), verified with the red observation that a plain `postgres` image fails it |
| AC2 | PASS — `embedded_migrations_satisfy_ac2` unit test (commit 0d40e6a), no container needed |
| AC3 | PASS — `migrations_are_applied_to_every_database`, `each_trial_gets_its_own_database`, `concurrent_requests_get_distinct_databases` (commit df64b2a) |
| AC4 | PASS — `one_container_serves_the_whole_binary` (commit df64b2a), verified with the red observation that a forced trial failure still ends with the container removed and a non-zero exit |
| AC5 | NOT_TESTED — discharged by the CI run on the pull request itself, not by any subtask. The Rust jobs are reached by the `**/*.rs`, `**/*.sql` and manifest path filters, and the runner image ships the Docker daemon the container crate falls back to. Recorded against subtask 3 earlier, which was wrong: subtask 3 rewrites coverage-tolerance prose and owns none of AC5. |

## Review register

| id | raised | severity | status | verifying command |
|----|--------|----------|--------|-------------------|

## Files touched

- `Cargo.toml` — populated `[workspace.dependencies]` (sqlx, testcontainers, testcontainers-modules, libtest-mimic, tokio), rewrote the now-false empty-table comment
- `Cargo.lock` — regenerated via `cargo build`/`cargo update -p testcontainers --precise 0.27.3`, never hand-edited
- `crates/core/Cargo.toml` — `sqlx` as a normal dependency; `testcontainers`, `testcontainers-modules`, `libtest-mimic`, `tokio` as dev-dependencies, all `.workspace = true`
- `crates/core/migrations/0001_vector_extension.sql` — new, single statement, no comment
- `crates/core/src/lib.rs` — `pub static MIGRATOR` + `#[cfg(test)]` unit test covering AC2's no-container half
- `crates/core/Cargo.toml` — added `[[test]] name = "database" path = "tests/database.rs" harness = false`
- `crates/core/tests/support/mod.rs` — new, the `Harness` (container start/shutdown, admin pool, per-call database creation and migration, take-once container slot behind `Arc`)
- `crates/core/tests/database.rs` — new, `main` (multi-threaded runtime, trial registration, shutdown, exit code) and five trials covering AC1, AC3 (×3) and AC4
