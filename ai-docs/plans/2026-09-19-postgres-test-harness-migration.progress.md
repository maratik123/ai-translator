# Progress: Postgres test harness and the vector-extension migration — ACTIVE
_Updated: 2026-09-19 12:01_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-09-19-postgres-test-harness-migration
**base_commit:** 1b0b287092e010b5a8794a28b2936190546fe1da
**Last build:** PASS
**Issue:** #13
**Spec:** ai-docs/plans/2026-09-19-postgres-test-harness-migration.spec.md
**current_step:** Step 10 — self-review APPROVE (Round 2)
**last_passed_gate:** make verify (full) + per-AC sweep | 2026-09-19T12:06:09Z | 14b60ac
**entry_args:** 13

## Next action

**Do this immediately:** Step 12 — finalise INDEX.md, move the plans to `done/`, telemetry, commit, retire the state files, open the pull request.

## Subtasks

- [x] 1. Dependency set, first migration, embedded migrator — `Cargo.toml`, `Cargo.lock`, `crates/core/Cargo.toml`, `crates/core/migrations/0001_vector_extension.sql`, `crates/core/src/lib.rs` — commit 0d40e6a
- [x] 2. Container harness and the database-backed test target — `crates/core/Cargo.toml`, `crates/core/tests/support/mod.rs`, `crates/core/tests/database.rs` — commit df64b2a
- [x] 3. Correct the statements this diff falsifies (D12) — `AGENTS.md`, `.githooks/coverage-ratchet.sh`, `Makefile`, `ai-docs/code-style.md` — commit 27ba952
- [x] 4. Record the harness decision where it will be looked for — `ai-docs/key-decisions.md` (KD-20) — commit 89b5773
- [x] 5. Amend the corpus row and tick what this task closes in full (D13) — `docs/03-storage.md` — commit 6fb4505

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

- **Step 8 (subtask 3)**: D12's site list was re-measured rather than taken on the design's word, each sweep with a constructed control line — `carry no test` returns `AGENTS.md:187` and `.githooks/coverage-ratchet.sh:25`, `crates it holds` returns `Makefile:31` and `ai-docs/code-style.md:90`, and no other live site in either case. A wider sweep for `skeleton` over the same corpus adds only `ai-docs/context-status.md:9,13,33`, which is the per-task history log writing in the past tense about the previous task; it is a history surface and was left untouched. The four rewritten sentences replace the *reason* only: the tolerance constant, the two band constants and every conditional branch of the ratchet script are byte-identical, and the three gate-script sentences D12 excludes — including the ratchet's own no-executable-lines branch comment, which shares the file — were not touched.
- **Step 8 (subtask 3)**: the comment-reference gate was seen RED in both gated files before the commit, not merely green afterwards. A markdown path planted inside the build entry point's file-size comment gave `Makefile:29: markdown-path: ai-docs/code-style.md`, exit 2; one planted in the ratchet script's tolerance header gave `.githooks/coverage-ratchet.sh:23: markdown-path: (AGENTS.md`, exit 2. Each was reverted from a `tmp/` copy, the executable bit re-checked on the script, and the gate then reported `no comment in the gated set points outward` against the committed tree. That the gate reaches both files by its own language map is therefore observed rather than assumed.
- **Step 8 (subtask 4)**: the design's stated consequence was executed against the shipped code before it was copied onto the key-decisions page, rather than transcribed. A trial body appended to the test target and left out of `main`'s trial vector made `cargo clippy --workspace --all-targets -- -D warnings` exit non-zero with `error: function unregistered_trial_probe is never used`, and the target failed to compile; the probe was reverted from a `tmp/` copy, `git diff --name-only` came back empty, and the same command then exited 0 with zero error and warning lines. So "an unregistered trial is a denied lint rather than a silent pass" is an observation in both directions, not an inference from `harness = false`.
- **Step 8 (subtask 4)**: the row's *source* field is backticked prose naming the design at its post-retirement path, per the design's own instruction, and that is not a broken link waiting to happen. CI's markdown link check parses the bracket-then-parenthesis inline-link form only, so a backticked path is outside what it resolves — the same program run locally over every tracked markdown file exited 0 — while a real markdown link to a `done/` path that does not exist yet would have failed it. The citation guard passed on the same tree. **That link check then found a real defect of its own:** a decisions-log line written two lines above quoted the inline-link form literally, and the checker's regex reaches inside backticks, so it reported `progress.md -> target` and exited 1. The line was reworded and the checker exited 0 — one RED on a genuine defect and one GREEN, both observed on the same instrument.
- **Step 8 (subtask 5)**: the corpus row's five clauses were changed exactly as D13's table prescribes and no further. Unchanged: the image, the socket and the version parenthetical, byte-identical in the diff. Replaced, each with its one-clause reason in the same sentence: `OnceCell` (a value in a static is never dropped), ryuk (the crate ships none), `#[sqlx::test]` (it takes its connection only from the variable the suite may not read). Kept as properties: one container per test binary, and a database per test with the migrations applied. The two rows this task closes in full are ticked, and nothing else moved — every other checkbox of the storage page is still `[ ]`, and the build-and-deploy page is not in the diff at all, so the Podman-socket row D10 declines is untouched and unticked. The edit leaves no relative link, which the markdown link checker confirms by resolving every one in the tree.
- **Step 8 (Group B close)**: `make verify` exited 0 at `6fb4505` with every sub-target executed — format, build, clippy, doc, test, lockfile, file limits, actionlint, shellcheck, comment references, the panic gate and the dependency-direction gate — no error and no warning line, `test result: ok` on all seven binaries, 1 unit test and 5 database trials. The coverage ratchet did not run and could not have: Group B staged no `.rs`, `.sql`, manifest or lockfile, which is the hook's silent-skip condition, and the recorded high-water mark is therefore unchanged by this group.

- **Step 9**: every AC verified by the orchestrator's own command over the AC's own scope, not by reading a test's verdict. AC2: one migration file, zero semicolons, zero comment lines, shown with `cat -A`. AC4: zero pgvector containers before the run and zero after. AC1/AC3: the trial bodies read — a bound-parameter vector round-trip, `current_database()` asked of the server, cross-database isolation checked by a table visible in one and not the other, and applied versions compared against the embedded migrator.
- **Step 9**: domain-invariant sweep — the only invariant this diff can reach is the forward-migration rule, and it holds by construction: one `.sql` file added, none modified. The other four subjects (cache keys, retrieval, request parameters, eval conditions) have no code in the workspace yet.
- **Step 9**: a harness gap was filed rather than worked around — CI's relative-link step scans raw text, so a link form quoted inside backticks or inside a fenced block is flagged as a broken link. Reproduced on four constructed cases with a control; the step already carries a hard-coded escape for one placeholder file name.

- **Step 9.5**: `ai-docs/context.md` was deliberately NOT edited. Its § Status says the page fixes the shape and that progress is read from the repository, so it carries no progress bullet to bump; and its architecture row summarising `reader-migrate` as the one place migrations are applied is not falsified by this diff — the corpus row it summarises is untouched and still unticked, and the corpus has named a test mechanism that migrates its own databases since before this task.
- **Step 9.5**: the removed-name sweep found the mechanisms surviving on exactly two live surfaces, both deliberate — the new key decision, which records the rejection of one of them, and the amended corpus row, which names each as the reason it was replaced. No stale prescription survives.

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
| G8 | 4 | § Test Design prescribes the concurrency trial asserts every reported current-database name is distinct and each carries the migrations, while the shipped trial compared harness-assigned names and checked `pg_extension` | design-internal | folded | design sentence kept as written; the CODE moved to match it — `current_database()` asked of the server and applied versions compared against the embedded migrator @ 9dec5b1 |
| G9 | 4 | the `one_container_serves_the_whole_binary` bullet says "the two databases of the previous case", which the design's own ban on cross-trial dependence forbids | design-internal | folded | § Test Design now reads "two databases this trial takes for itself" @ 040dcdb |
| G10 | 4 | subtask 3's prose says the gate-script sentences D12 enumerates are absent from its file set, but the ratchet's no-executable-lines comment shares a file subtask 3 does list | design-internal | folded | the items are named rather than counted, and the gated-set sentence extended to the ratchet script @ 040dcdb |
| G11 | 4 | two claim tags take a file this task rewrites as their subject, and one tag states a row count | design-internal | folded | the version is D14's decision sourced from the registry bound; lockfile and `cargo tree` demoted to corroboration; the count deleted rather than re-measured @ 040dcdb |
| G12 | 4 | the design's prescribed red observation for the start count was recorded as performed when a substituted mutant had been run instead | design-internal | folded | the substitution was recorded with what it is equivalent to and what it is weaker in; then the PRESCRIBED mutation was run by the orchestrator, observed RED, and the second container was seen left behind and removed @ 040dcdb |
| G13 | 4 | the AC5 row attributed the criterion to subtask 3, which owns none of it | design-internal | folded | the AC Status row now attributes AC5 to the CI run on the pull request @ 2b597a9 |
| G14 | 4 | when Group B rewrites the ratchet header and the build entry point, both sit inside the comment-reference gated set | design-internal | folded | subtask 3's contract now says so; the delegate then observed the gate RED on a planted markdown path in each of the two files @ 040dcdb |
| G15 | 4 | D3 leaves to the implementor whether the trial verdict survives a teardown failure | design-internal | folded | D3 decides it: the runner's code is taken before teardown, a failure raises a green run and never replaces an earned failing status, and no panicking call carries the result out @ 040dcdb |

- **Step 10 (R1-4 closed)**: the design's AC3 red observation was re-run in the sharper form the reviewer named — `.database(database)` dropped from the connect options, so every pool reaches the default database while the harness-assigned names stay distinct. Both isolation trials then failed at their own assertions rather than at `CREATE DATABASE`: `both databases reported the same name "postgres"`, and the concurrency trial listing eight identical reported names. Reverted from a `tmp/` copy, `git diff --name-only` empty, five trials green again at exit 0.
- **Step 10 (R1-6)**: filed as a harness gap with a correction. The delegate's symptom held — `grep` here is ugrep 7.8.4 behind a shell function, shadowing GNU grep 3.12 at `/bin/grep`. Its diagnosed cause did NOT reproduce: the four-way alternation it blamed matched a constructed control under ugrep, under the per-scheme form, and under the GNU binary. The entry records the verified half and says plainly that the causal half is unconfirmed.

- **Step 11 (R1-1 closed)**: the design prescription was struck and the dead `assert!` removed. Removing it dropped workspace coverage, because the assertion's lines were covered while verifying nothing — the ratchet rewarding the exact shape a harness-gap entry written an hour earlier had predicted it rewards.
- **Step 11**: that removal then exposed a live defect in the ratchet itself. It compares a full-precision figure and records a half-up-rounded one, so at a measured 84.61538 it wrote 84.62 and blocked its own next evaluation, printing both sides as the same rounded number. Repaired by recording 84.61; filed, with the observation that the next commit staging a Rust file will re-raise and re-block until the script is fixed.

- **Step 10 (round 2)**: APPROVE. The round's load-bearing result is that R1-1's removal left no hole — a migration planted at version 0 made the surviving version assertion fail with `left: 0`. The reviewer's FIRST run of that probe came back green and it read that green correctly, as a fact about the build rather than the guard: `sqlx::migrate!()` registers no rebuild dependency on the migrations directory, so the binary still held the old embedded set until the source was touched.
- **Step 10 (round 2)**: `make cover-ratchet` is NOT part of `make verify` — CI runs it as its own job, so it has to be run separately before the pull request. Noted because the ratchet defect filed this run makes that separate run the one that would catch a re-raise.

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
| AC1 | PASS — bound-parameter vector round-trip in `vector_type_is_usable`; the red direction was observed with a plain `postgres` image, which fails at the migration step |
| AC2 | PASS — `crates/core/migrations/` holds one file; `cat -A` shows `CREATE EXTENSION IF NOT EXISTS vector` plus one newline, zero semicolons, zero comment lines; the unit test asserts version 1 and the description the loader derives |
| AC3 | PASS — `each_trial_gets_its_own_database` asks the server `current_database()` for both and checks a table made in one is invisible from the other; `migrations_are_applied_to_every_database` compares applied versions against the embedded migrator; `concurrent_requests_get_distinct_databases` does both under contention |
| AC4 | PASS — process-wide start counter asserted first, postmaster instant corroborating; zero pgvector containers before the run and zero after; the assertion was observed RED under the design's prescribed mutation |
| AC5 | PENDING CI — discharged by the run on the pull request, which does not exist yet. The Rust jobs are reached by the `**/*.rs`, `**/*.sql` and manifest path filters, and the runner image ships a Docker daemon |

## Review register

| id | raised | severity | status | verifying command |
|----|--------|----------|--------|-------------------|
| R1-1 | 1 | minor |  fixed@e93c275 | `sed -n '16,42p' crates/core/src/lib.rs` — the predicate at `:36-42` is `all(\|m\| m.version >= lowest.version)` with `lowest = min_by_key(\|m\| m.version)`, i.e. the definition of `min`. Probed over `[1]`, `[5,2,9]`, `[0,1,2]`, `[-7,1,3]`, `[9,4,1]`: all true. Falsifiable control `v >= 1` over the same five: false on `[0,1,2]` and `[-7,1,3]`, so the probe can report false. AC2 stays covered by `assert_eq!(lowest.version, 1)` at `:22-25`, so no hole — a redundant line that cannot fail. Amending means the `design-writer` Subagent edits § Test Design and Decomposition subtask 1 |
| R1-2 | 1 | minor |  fixed@54debb3 | `grep -n -A4 'fn exit_code' ~/.cargo/registry/src/*/libtest-mimic-0.8.2/src/lib.rs` → `:384-386` returns `ExitCode::from(101)` on failure; `sed -n '64,79p' crates/core/tests/database.rs` → the `Err` arm returns `ExitCode::FAILURE` (1) unconditionally. So when the trials fail **and** the teardown fails, 101 is replaced by 1 — narrowly contradicting the comment at `:67-72` ("keeps failing rather than having its status replaced") and D3's "never *replaces* the status of a run the trials already failed". No behavioural hole: every failing case still exits non-zero and the trial verdict still reaches the reader on stdout, which is D3's stated purpose |
| R1-3 | 1 | minor |  fixed@abefca8 | `awk '/^## GO notes/,/^## Key discoveries/' <this file> \| grep -E '^\\\| G' \| awk -F'\|' '{print $3}'` → seven rows, every one round 1; a constructed round-4 row is matched by the same pattern, so the absence is the table's. The Decisions log records that design-review round 4 returned GO and that "folding the GO notes surfaced a code non-compliance", so round 4's GO notes exist and have no row. The fold itself is traceable via `040dcdb` and `f1630d3`, so this is a traceability gap in the state file, not a stale design |
| R1-4 | 1 | minor |  fixed@54debb3 | Decisions log Step 8 red observation 3 vs `crates/core/tests/database.rs:157-165`. The recorded mutation (counter short-circuited to id `0`) made the trial fail at the harness's `CREATE DATABASE` (`database "reader_core_test_0" already exists`), not at the trial's own isolation assertions, so those assertions have not been shown to have a reachable failure mode. A sharper mutation reaches them: drop `.database(database)` from `connect_options` at `crates/core/tests/support/mod.rs:150-157`, which makes both pools report the same `current_database()`. The trial did fail, so nothing is cosmetic-and-green; the record is weaker than it reads |
| R1-5 | 1 | — | accepted@1 — examined, not a defect | `sed -n '226,266p' crates/core/tests/database.rs` — the early return at `:255-263` drops the remaining `JoinHandle`s, detaching rather than cancelling those tasks. Test-position code on an already-failing path; every task is awaited on the success path, its error propagated with `??` and its panic surfaced through `JoinError`. All four concurrency-ownership questions answered |
| R1-6 | 1 | — | accepted@1 — out of scope for this diff; a tooling hazard for agent-authored probes, hand to `ai-docs/harness-gaps.md` | `command -v grep; grep --version \| head -1` → **ugrep 7.8.4**, shadowing GNU grep 3.12 at `/bin/grep`. Against a constructed driver-URL line, a single-scheme pattern of the credential shape matches and a **four-way alternation** of the same shape returns *nothing* — no error, no diagnostic, just empty — while `/bin/grep` matches it. The failure is silent, so any sweep an agent writes in that alternation shape reports the clean answer for every possible tree. **The `PostToolUse` secret-leak hook is NOT affected and was wrongly implicated in this row's first draft: it fired on the draft itself and refused the write, which is a live demonstration that its own runner resolves a `grep` that handles the pattern.** The lesson is the probe AXIOM's, not the hook's — a `grep` result in the agent's Bash shell is evidence about that shell until a control has been seen to match, which is what caught this review's own credential sweep before its clean output was believed |
| R2-1 | 2 | minor | accepted@2 — below severity floor | `/bin/grep -n 'fn embedded_migrations_satisfy_ac2' crates/core/src/lib.rs` → `:15`. The name carries an acceptance-criterion id instead of the behaviour, against `AGENTS.md` § *Test Conventions* ("Test names describe behaviour"). The referent retires to `ai-docs/plans/done/` at Step 12, leaving a name pointing at nothing. **Not a Design Amendment trigger:** `/bin/grep -n 'satisfy_ac2' <design>` and the same for `embedded_migrations` both exit 1, so the design prescribes no name and the repair is code-only. Not gated either — `make comment-refs` is green on this file and the token is an identifier, not a comment. A behaviour-describing replacement: `first_migration_is_the_vector_extension` |
| R2-2 | 2 | minor | accepted@2 — below severity floor | `printf 'SELECT 1\n' > crates/core/migrations/0000_earlier.sql; cargo test -p reader-core --lib` → **exit 0**, `test result: ok. 1 passed`, against a migrations directory that now carries a version `0`. Then `touch crates/core/src/lib.rs; cargo test -p reader-core --lib` → **exit 101**, `assertion left == right failed: the lowest-versioned migration is version 1` with `left: 0`, at `crates/core/src/lib.rs:22`. So `sqlx::migrate!()` registers no rebuild dependency on `crates/core/migrations/` on this toolchain: a migration added without touching a `.rs` file is invisible to the AC2 guard in an incremental build, and the first run above is an instrument failure rather than a green subject. No CI hole — every CI job builds from a clean checkout. Probe file removed, `git status --porcelain` empty, `git diff --name-only HEAD --` empty, re-run exit 0. **The same probe establishes the positive half:** with the recompile forced, `assert_eq!(lowest.version, 1)` at `:22-25` goes RED on a preceding migration, so R1-1's removal of the dead predicate left AC2's "nothing precedes it" half genuinely covered, exactly as the amended design claims |
| R2-3 | 2 | minor | accepted@2 — out of this spec's scope; the script repair is the owner's call and the defect is already filed | `make cover-ratchet` → **exit 0**, `coverage-ratchet: 84.62% >= 84.61% (a rise the pre-commit hook would have recorded)`; `jq -r '.data[0].totals.lines' tmp/coverage.json` → count 13, covered 11, percent 84.61538461538461. CI's Coverage-ratchet job is therefore green on this branch, and the recorded `84.61` is a value the measurement satisfies. The shipped state is still primed to self-block: `.githooks/coverage-ratchet.sh:170-175` compares `cur` at full precision while `:197` writes `rounded`, so the next commit staging a `.rs`, `.sql`, manifest or lockfile raises the file to `84.62` in raise mode and the commit after that is refused. Filed in `ai-docs/harness-gaps.md` with the correct repair (round down, or record `current`) |
| R2-4 | 2 | — | accepted@2 — examined, not a defect | `crates/core/tests/support/mod.rs:133-147` — `shutdown` can panic at the `.expect()` on `:137` and carries no `# Panics` section. `awk 'NR==10' ai-docs/doc-convention.md` → "DOC-1, DOC-2, DOC-3 and DOC-6 apply to every `.rs` file under a crate's `src/`", and `# Panics` lives in DOC-3, so a file under `tests/` is outside that scope. The panic gate agrees by position rather than by luck: `/bin/grep -n 'EXCLUDED_DIRS' ai-docs/scripts/panic_calls.py` → `:46` excluding `tests/`, `benches/` and `examples/`, and `:67` matching every `#[cfg(test)]` span — so the `.expect()` at `crates/core/src/lib.rs:20` owes no `ai-docs/panic-index.md` row either, and `make panic-calls` exits 0 |
| R2-5 | 2 | — | accepted@2 — examined, not a defect | `crates/core/tests/database.rs:77-81` — the `conclusion.has_failed()` branch R1-2's fix added is exercised by no test; reaching it needs a failing trial **and** a failing container removal in one run, inside a `harness = false` target's own `main`. Semantics re-read rather than assumed: `/bin/grep -n -A4 'fn exit_code' ~/.cargo/registry/src/*/libtest-mimic-0.8.2/src/lib.rs` → `:384-390`, `exit_code` returns `ExitCode::from(101)` exactly when `has_failed()`, and `:393-395`, `has_failed` is `num_failed > 0`. The branch therefore yields 101 when the trials already failed and 1 otherwise, which is what D3 decides and what the comment above it now says |
| R2-6 | 2 | — | accepted@2 — examined, not a defect | `crates/core/tests/support/mod.rs:18-22` — `Error` is a boxed trait object rather than a typed error per failing operation. Test-position code, and the deviation is stated in the item's own doc comment rather than left implicit; `ai-docs/doc-convention.md:10-11` puts test code under "documents only what is non-obvious about the fixture", and no error of this type crosses a crate boundary |

## Files touched

- `Cargo.toml` — populated `[workspace.dependencies]` (sqlx, testcontainers, testcontainers-modules, libtest-mimic, tokio), rewrote the now-false empty-table comment
- `Cargo.lock` — regenerated via `cargo build`/`cargo update -p testcontainers --precise 0.27.3`, never hand-edited
- `crates/core/Cargo.toml` — `sqlx` as a normal dependency; `testcontainers`, `testcontainers-modules`, `libtest-mimic`, `tokio` as dev-dependencies, all `.workspace = true`
- `crates/core/migrations/0001_vector_extension.sql` — new, single statement, no comment
- `crates/core/src/lib.rs` — `pub static MIGRATOR` + `#[cfg(test)]` unit test covering AC2's no-container half
- `crates/core/Cargo.toml` — added `[[test]] name = "database" path = "tests/database.rs" harness = false`
- `crates/core/tests/support/mod.rs` — new, the `Harness` (container start/shutdown, admin pool, per-call database creation and migration, take-once container slot behind `Arc`)
- `crates/core/tests/database.rs` — new, `main` (multi-threaded runtime, trial registration, shutdown, exit code) and five trials covering AC1, AC3 (×3) and AC4
- `AGENTS.md` — the coverage-tolerance paragraph's reason clause only; the tolerance value and every other sentence unchanged
- `.githooks/coverage-ratchet.sh` — the tolerance header's reason clause only; the constant and every conditional branch unchanged
- `Makefile` — the file-size bands' justification; both band constants unchanged
- `ai-docs/code-style.md` — the same justification, in the twin sentence the bands' table carries
- `ai-docs/key-decisions.md` — KD-20 appended to § Repository and process; no existing row edited, KD-16 left as it stands
- `docs/03-storage.md` — the test row amended to the harness that exists, and it plus the vector-extension row ticked; every other row of the page untouched
- `ai-docs/plans/2026-09-19-postgres-test-harness-migration.spec.md` — the approved spec, written through the interview and amended by the owner's strike
- `ai-docs/plans/2026-09-19-postgres-test-harness-migration.design.md` — the design, across eight writing rounds and four review rounds
- `ai-docs/plans/2026-09-19-postgres-test-harness-migration.spec.md.state.md` — the interview state, carrying the owner's answers verbatim
- `ai-docs/plans/2026-09-19-postgres-test-harness-migration.progress.md` — the run's own record
- `ai-docs/context-status.md` — the per-task entry appended at Step 9.5, carrying its placeholder pull-request locator until Step 12 fills it
- `ai-docs/harness-gaps.md` — the diagnoses filed during the run: the edit guard's missing intended-shrink path, the relative-link check that cannot tell a link from a quoted example, the shell's `grep` resolving to ugrep, the unnamed third test-instrument failure mode, and the coverage ratchet recording a rounded-up value it cannot satisfy
- `ai-docs/learnings.md` — the conduct entries: a probe that left files in the repository root, and a claim about an upstream pull request made from a query that did not fetch its body
- `ai-docs/coverage-ratchet.txt` — lowered with its reason when a dead assertion's removal took covered lines with it, then repaired after the script raised it past its own measurement

## Self-Review (Round 1)

**Verdict:** APPROVE

**What was checked.** All five ACs against the shipped tree, not against the delegates' reports.
AC1 — `vector_type_is_usable` at `crates/core/tests/database.rs:98-115` binds the literal and compares
the round-trip exactly. AC2 — mutated `crates/core/migrations/0001_vector_extension.sql` to carry a
second statement and ran `cargo test -p reader-core --lib`: exit 101, `left: "CREATE EXTENSION IF NOT
EXISTS vector;\nCREATE TABLE sneaky (id integer);"` against `right: "CREATE EXTENSION IF NOT EXISTS
vector"`; restored from a `tmp/` copy, `git diff --name-only` empty, re-run exit 0. So the exact-text
assertion is load-bearing in both directions. AC3 — the two isolation trials and the concurrent trial
read `current_database()` from the server rather than trusting the harness's own name. AC4 — the
process-wide `CONTAINER_STARTS` at `crates/core/tests/support/mod.rs:37` is zero at rest and
incremented at `:69`, the site that awaits the container start. AC5 — re-measured the CI paths filter
against HEAD, not against the design's pin: `.github/workflows/ci.yml` carries `**/*.rs` (:40),
`**/*.sql` (:41), `**/Cargo.toml` (:48), `Cargo.lock` (:49) under `rust:` and `**/*.rs` / `**/*.sql`
under `commentrefs:` (:87,:89), pattern confirmed against a constructed control, so no filter entry
was owed and none was added.

Gates re-run against the shipped tree, each exit code read apart from stdout: `make panic-calls` 0,
`make comment-refs` 0, `make file-limits` 0, `make lock-check` 0, `make import-guard` 0
(`2 binary target(s), no forbidden path`), `make doc-check` 0, `cargo clippy --workspace --all-targets
-- -D warnings` 0 — zero `error`/`warning` lines in either captured log. The design carries no
`AC<N> verified by:` lines (pattern confirmed against a control), so § 2's re-run obligation was
discharged over the design's `[measured …]` claims that the shipped code rests on.

Also checked: no `let _ = <Result>` and no `.unwrap()` anywhere in the new Rust; the four `.expect()`
calls are all test-position code and the panic gate is green. No `#[allow(…)]` added. Domain
invariants — the only one this diff can reach is the forward-migration rule, and it holds by
construction (one `.sql` added, none modified); the credential sweep ran per-scheme over every file
the diff touches, each pattern first shown alive against its own constructed control, and returned
nothing. Scope — `coverage-ratchet.txt`, `learnings.md`, `harness-gaps.md` and `context-status.md` are
each mandated by a standing rule or written by the pre-commit hook, not scope creep;
`rust-test-conventions.md` and KD-16 are absent from the diff and the `AGENTS.md` edit is confined to
the coverage-tolerance paragraph, as D12 and answer 4.1 require. `docs/03-storage.md` has exactly rows
7 and 8 ticked and every other box unticked, and `docs/09-build-and-deploy.md` is not in the diff.
GO-notes round trip — the seven round-1 rows all resolve at `d29e0e4`, which precedes the first
implementation commit `0d40e6a`.

**Findings.** No `blocker` or `major` row clears the severity floor, so none is open and the verdict is
APPROVE. **4 `minor` items**, recorded in the register rather than as table rows, in:
`crates/core/src/lib.rs`, `crates/core/tests/database.rs` (with the design's D3), and the progress
file itself (two).

**One of them is a Design Amendment trigger, surfaced explicitly rather than filed as a code fix
(R1-1).** `crates/core/src/lib.rs:36-42` asserts
`MIGRATOR.migrations.iter().all(|m| m.version >= lowest.version)` where `lowest` is
`min_by_key(|m| m.version)` — the predicate is the definition of `min`, so it is true for every
possible migration set. Probed over five adversarial sets, including one carrying version `0` and one
carrying a negative version: all five true; a falsifiable control predicate (`v >= 1`) over the same
five inputs returns false on exactly those two, so the probe can report false and the all-true result
is a property of the shipped predicate. AC2's "no other migration precedes it" is nonetheless fully
covered by `assert_eq!(lowest.version, 1)` at `:22-25`, so this leaves no hole — it is a redundant
line that cannot fail, which is the shape `AGENTS.md` § *Patterns* 2 names. The design prescribes it
(§ Test Design, "and no migration in the set carries a version below it"; Decomposition subtask 1,
"no migration carries a lower version"), so removing it from the code would make the design stale:
**Design Amendment trigger — spawn the `design-writer` Subagent to amend
`ai-docs/plans/2026-09-19-postgres-test-harness-migration.design.md` § Test Design and Decomposition
subtask 1; recipe at `.claude/skills/task/SKILL.md` Step 11 fail-loud table.** Severity is `minor` by
the mechanical floor: it violates no AC, D or gate id, and no command fails against the shipped tree.

**Out of scope for this diff, handed to the orchestrator rather than filed as a finding.** While
running the credential sweep, the shell's `grep` on this machine resolved to **ugrep 7.8.4**, which
shadows GNU grep 3.12 at `/bin/grep` and **fails to match a four-way alternation** of the
driver-URL-with-password shape — returning empty, with no error and no diagnostic, while a
single-scheme pattern matches the same constructed line and `/bin/grep` matches the four-way form
correctly. A sweep written in that shape therefore reports the clean answer for every possible tree.
This review's own credential sweep was written that way first, and its clean output was caught as
meaningless only because a control was run before the result was believed; it was then redone
per-scheme, each pattern shown alive against its own control.

**A first draft of this paragraph blamed the `PostToolUse` secret-leak hook for the same blind spot,
and that was wrong.** The hook fired on the draft, refused the write, and quoted the offending line
back — a live demonstration that its own runner resolves a `grep` that handles the pattern the
agent's Bash shell does not. The claim was corrected here and in register row R1-6 rather than merely
withdrawn in conversation. The residual item worth recording in `ai-docs/harness-gaps.md` is the
tooling hazard for **agent-authored probes**, not a defect in the hook: nothing in this diff
introduced it and nothing in this diff is affected, since D5 avoids the credential shape entirely.

| # | File:line | Severity | Finding | Status |
|---|-----------|----------|---------|--------|

_No `blocker`/`major` row is open; per the findings-format rule the `minor` items ride along as
register rows rather than table rows._

## Self-Review (Round 2)

**Verdict:** APPROVE

**Spawn prompt.** Within the closed list — the invocation line, `Spec:`, `Design:`, `Progress:` and
the commit range, nothing else. No `PROMPT-CONTAMINATION` finding.

**What was checked.** The whole window `1b0b287..HEAD`, not the post-round-1 commits alone, with the
register's scoping applied on top.

*The four `fixed@` rows, each re-examined over the diff since its own sha.* **R1-1** (`fixed@e93c275`)
— the predicate true by definition is gone from `crates/core/src/lib.rs`, gone from the design
(`763b1b7` rewrites § Test Design and Decomposition subtask 1), and the exemption is real rather than
asserted: the state file's `round: 6` answer is «Править без ревью… точечное освобождение владельца»,
read from `ai-docs/plans/2026-09-19-postgres-test-harness-migration.spec.md.state.md:86-89`. The fix
is also *sufficient*, which is the half a prose read cannot settle — see the mutation below.
**R1-2** (`fixed@54debb3`) — `crates/core/tests/database.rs:77-81` now keeps the runner's 101 when the
trials already failed; `libtest-mimic-0.8.2/src/lib.rs:384-395` re-read to confirm `exit_code` is
`ExitCode::from(101)` exactly when `has_failed()`, so the branch matches D3 and the comment above it.
**R1-3** (`fixed@abefca8`) — rows G8–G15 now carry round 4, and each cited resolution commit exists
and touches the file it claims (`9dec5b1` the trial, `040dcdb` the design, `2b597a9` the AC row).
**R1-4** (`fixed@54debb3`) — the sharper mutation is recorded at Step 10 with both isolation trials
failing at their own assertions; no `.rs` moved since that sha, so the row's subject is closed.
The two `accepted@1` rows were not re-raised: nothing has changed for either.

*The mutation that settles R1-1, performed rather than reasoned about — and its instrument failure
reported, not hidden.* Planting `crates/core/migrations/0000_earlier.sql` and running
`cargo test -p reader-core --lib` gave **exit 0, `test result: ok. 1 passed`** — a green that is
evidence about the build, not about the guard: `sqlx::migrate!()` registers no rebuild dependency on
the migrations directory, so the binary still held the old embedded set. After
`touch crates/core/src/lib.rs` forced the recompile, the same tree gave **exit 101**,
`assertion left == right failed: the lowest-versioned migration is version 1`, `left: 0`, at
`crates/core/src/lib.rs:22`. So the surviving `assert_eq!(lowest.version, 1)` does carry AC2's
"nothing precedes it" half, and the removed predicate left no hole. Probe file deleted,
`git status --porcelain` empty, `git diff --name-only HEAD --` empty, re-run exit 0. The staleness
itself is recorded as R2-2; CI builds clean, so it is not a CI hole.

*Gates re-run against the shipped tree, exit codes read apart from stdout.* `make verify` **exit 0**
— every sub-target, `test result: ok` on all seven binaries, 1 unit test and 5 database trials, zero
`error`/`warning` lines in `tmp/sr2-verify.log`. `make cover-ratchet` **exit 0** (it is *not* part of
`verify` — `Makefile:40` omits it — so it was run separately, since CI runs it as its own job):
`84.62% >= 84.61%`. `shellcheck .githooks/coverage-ratchet.sh` **exit 0**, the one `*.sh` this diff
touches. The design carries no `AC<N> verified by:` line — `/bin/grep -nE 'AC[0-9]+ verified by'`
over it exits 1 while the same pattern matches a constructed control, so § 2's re-run obligation has
no commands to execute and was discharged over the design's `[measured …]` claims instead.

*Design conformance.* The round-1 GO notes G1–G7 all resolve at `d29e0e4`, and
`git merge-base --is-ancestor d29e0e4 0d40e6a` confirms that fold precedes the first implementation
commit. G8–G15 fold after it, which the design's own § Handoff paragraph decides and the owner's
cap-raise authorises; the test is whether the design is stale now, and it is not — G8's prescription
(`current_database()` asked of the server, applied versions compared against the embedded migrator)
is what `crates/core/tests/database.rs:230-283` does. The `763b1b7` amendment and the shipped
`crates/core/src/lib.rs` agree line for line.

*Claims re-derived rather than read* (the diff is predominantly prose). `tmp/coverage.json` totals are
`count 13, covered 11, percent 84.61538461538461`, so the harness-gaps ratchet entry's figure is
exact. `.githooks/coverage-ratchet.sh` does compare `cur` at `:170-175` and write `rounded` at `:197`,
so that entry's diagnosis is the script's, not a guess. `grep` here is a shell function and
`/bin/grep` is GNU 3.12, as the ugrep entry says — every sweep in this round was run through
`/bin/grep` for that reason. `AGENTS.md:356` carries "tautological test" and `:358` the two named
failure modes, so the design's `[measured 425b81b:AGENTS.md:356,358 …]` tag resolves.

*Also checked.* Domain invariants — the only one this diff can reach is the forward-migration rule:
`git diff --name-status` over `*.sql` returns a single `A`, no `M`. Credential sweep over all 1829
added non-lockfile lines, six patterns run one at a time through `/bin/grep`, each first shown alive
against its own constructed control: three hits, all inspected and all false (`subtask-1`/`subtask-2`
against `sk-`, and the harness-gaps entry quoting its own `password=hunter2` control line). No
`let _ = <Result>` and no `.unwrap()` in the new Rust, the pattern shown matching a control. No
`#[allow(…)]`. File sizes 38 / 158 / 285 lines, far inside the bands. No `⚠️ Objected` row exists, so
§ 7 is vacuous. Scope — every path in the diff is either a decomposition file or mandated by a
standing rule; `ai-docs/context-status.md` was confirmed present in the base commit, so it is an
append to a history surface and not a new artefact.

**Findings.** No `blocker` or `major` row clears the severity floor, so none is open and the verdict
is APPROVE. **3 `minor` items**, recorded in the register rather than as table rows, in:
`crates/core/src/lib.rs` (the AC-id test name, R2-1), `crates/core/migrations/` with
`crates/core/src/lib.rs` (the stale-embedded-set hazard, R2-2), and `ai-docs/coverage-ratchet.txt`
with `.githooks/coverage-ratchet.sh` (the primed self-block, R2-3). None is a Design or Spec Amendment
trigger — R2-1's name is prescribed nowhere in the design, and R2-2 and R2-3 are properties of
`sqlx` and of a gate script, not of a recorded decision. Three further items were examined and ruled
not-a-defect (R2-4 to R2-6): the absent `# Panics` on `shutdown` is outside DOC-3's `src/`-only scope,
the new `has_failed()` branch is untestable by construction and matches D3, and the boxed harness
error is test-position and documented as such.

**AC5 remains the one criterion this review cannot discharge**, and that is the design's own routing
rather than a gap: it is answered by the CI run on the pull request that does not exist yet. What
*can* be checked here was: `.github/workflows/ci.yml` reaches the Test and Coverage-ratchet jobs
through the `rust` filter's `**/*.rs`, `**/*.sql`, `**/Cargo.toml` and `Cargo.lock` entries, all four
of which this diff touches, so neither job can silently not-run on this pull request.

| # | File:line | Severity | Finding | Status |
|---|-----------|----------|---------|--------|

_No `blocker`/`major` row is open; per the findings-format rule the `minor` items ride along as
register rows rather than table rows._
