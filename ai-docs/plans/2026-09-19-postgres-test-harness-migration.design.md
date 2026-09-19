# Design: Postgres test harness and the vector-extension migration

**Issue:** #13
**Date:** 2026-09-19

## Approach

### What the tree holds today

The workspace's skeleton members declare no dependency at all: the root manifest's
`[workspace.dependencies]` table carries a comment saying so and nothing else
`[measured 0daa273:Cargo.toml:16-18 · awk 'NR>=16 && NR<=18 {print NR": "$0}' Cargo.toml → "16: [workspace.dependencies]" / "17: # A dependency is added here only once a member's code compiles against it;" / "18: # nothing does yet, so the table is empty."]`,
and each member's `[dependencies]` section is empty
`[measured 128c2fb:crates/{shared,core,cli,migrate}/Cargo.toml:8 · for c in shared core cli migrate; do awk '/^\[dependencies\]/{print FILENAME":"NR": "$0; f=1; next} f{print FILENAME":"NR": "$0}' crates/$c/Cargo.toml; done → ':8: [dependencies]' in each of the four and no line after it]`.
`crates/core/src/lib.rs` is a single `//!` line and carries no item
`[measured 0daa273:crates/core/src/lib.rs · cat crates/core/src/lib.rs → "//! The translation engine: segmentation, retrieval, prompting and validation."]`.
At that commit there is no `migrations` directory, no test target and no code that opens a database
`[measured 0daa273 · git ls-tree -r --name-only 0daa273 -- crates/ → each member's manifest and its single root source and nothing else; the same listing filtered for "migrations/" or "tests/" returns no path. The commit is read with ls-tree rather than ls-files on purpose: ls-files answers for the index, which now holds what the subtasks below added]`.
This section describes the tree the design was written against, not the tree a later reader finds:
the subtasks below are what change it.

Gates are already written for the world this task creates, and none of them needs changing. The
dependency-direction gate's forbidden table already names both container crates, with the reason each
is test-only
`[measured 0daa273:ai-docs/scripts/import_guard.py:32-35 · awk 'NR>=32 && NR<=35 {print NR": "$0}' ai-docs/scripts/import_guard.py → the FORBIDDEN dict with the "testcontainers" and "testcontainers-modules" rows and their one-line reasons]`,
and its walk skips an edge whose kinds are a subset of `{dev}`
`[measured 0daa273:ai-docs/scripts/import_guard.py:129-134 · awk 'NR>=129 && NR<=135 {print NR": "$0}' ai-docs/scripts/import_guard.py → ':130: kinds = {k.get("kind") for k in dep.get("dep_kinds", [{"kind": None}])}' and ':133: if kinds and kinds <= {"dev"}:' / ':134: continue']`,
so a dev-dependency on either crate is exactly what the gate is built to permit. The build entry
point's `test` recipe already describes the suite as container-backed
`[measured 0daa273:Makefile:57-62 · awk 'NR>=57 && NR<=62 {print NR": "$0}' Makefile → the comment block "The suite provisions its own database …" above ':61: test:' / ':62: cargo test --workspace']`,
and the coverage ratchet already names the failure direction for a machine with no runtime
`[measured 0daa273:.githooks/coverage-ratchet.sh:116-127 · awk 'NR>=116 && NR<=127 {print NR": "$0}' .githooks/coverage-ratchet.sh → the comment "The suite provisions its own database through testcontainers …" and the BLOCKED branch whose message ends "If no container runtime is reachable, start the Podman socket the suite connects to and re-run"]`.

### The corpus names a mechanism that the shipped crates cannot provide

`docs/03-storage.md` § Задачи fixes the storage decisions this task implements from: `sqlx` with the
`postgres` feature, migrations under `crates/core/migrations/`, the `vector` extension in the first
migration, and a test harness built from `testcontainers` with the `pgvector/pgvector:pg18` image over
the Podman socket, ryuk on, one container per test binary held in a `OnceCell`, with `#[sqlx::test]`
creating a database per test and applying the migrations itself. That page is DECISIONS
(`AGENTS.md` § Project), so this design implements from it rather than re-litigating it.

The mechanisms that row names for the harness's lifecycle are not available in the crates that would
have to provide them, and each gap below was measured rather than recalled.

**`#[sqlx::test]` has exactly one source for its connection, and it is the variable the suite is
forbidden to read.** The Postgres test driver reads it in both of its entry points, each time with a
panicking expectation
`[measured sqlx-postgres@0.9.0 · grep -n 'DATABASE_URL' sqlx-postgres-0.9.0/src/testing/mod.rs → ':42: let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");' and ':93:' the same line inside cleanup_test_dbs]`,
and the attribute accepts no argument that could carry one. Its parser recognises the keys
`fixtures`, `migrations` and `migrator`, and rejects everything else with a message that enumerates
them exhaustively
`[measured sqlx-macros-core@0.9.0 · grep -n 'is_ident(' sqlx-macros-core-0.9.0/src/test_attr.rs → ':205: is_ident("fixtures")', ':242: is_ident("migrations")' and ':278: is_ident("migrator")'; and awk 'NR>=295 && NR<=300 {print NR": "$0}' on the same file → the catch-all arm at ':295' whose error text is: expected fixtures("<filename>", ...) or migrations = "<path>" | false or migrator = "<rust path>"]`.
The last two are two spellings of one migrations source — `migrator` parses into the same option
`migrations` does — and none of the three names a connection.
The `sqlx.toml` setting that renames the variable is consumed by the compile-time query macros and by
`sqlx-cli`, never by the test runtime
`[measured sqlx-core@0.9.0 · grep -rn 'database_url_var' sqlx-core-0.9.0/src sqlx-macros-core-0.9.0/src → the declaration and accessor in sqlx-core-0.9.0/src/config/common.rs, whose accessor defaults to "DATABASE_URL", and its call sites, every one of them in sqlx-macros-core-0.9.0/src/query/metadata.rs and none in a testing module]`,
and the upstream work that would let a test name its own environment variable is open and unmerged
`[measured launchbadge/sqlx#4050 · gh pr view 4050 --repo launchbadge/sqlx --json title,state,mergedAt → title "feat: Support for specifying multiple databases in test macros.", state "OPEN", mergedAt null; and the same pull request --json body --jq .body | grep -n var → ':3: Adds grouping by "env" and specification of environment variable names by "var" as macro arguments.' — the title names multiple databases and the body names the argument]`.
So `#[sqlx::test]` against a container means putting the container's DSN into the process's own
`DATABASE_URL` before the first test connects — which `AGENTS.md` § *Build & Test* forbids in the
plainest terms it uses anywhere, and which the owner reaffirmed by striking the spec rows that merely
restated it (`prior_qa` round 2: «три критерия повторяют AGENTS.md § Build & Test — убрать их»).

**testcontainers-rs carries no Ryuk.** A case-insensitive sweep of the whole published crate for
either name comes back empty, and the same pattern matches a constructed control line
`[measured testcontainers@0.27.3 · grep -rni 'ryuk\|reaper' testcontainers-0.27.3/ → no output; and the same pattern over a constructed control file holding the lines RYUK_CONTAINER_IMAGE, ryuk and Reaper → all three matched, so the pattern ran and the absence is the crate's]`.
What it has instead is an optional `watchdog` feature that stops and removes registered containers on
`SIGTERM`, `SIGINT` or `SIGQUIT`
`[measured testcontainers@0.27.3 · awk 'NR>=1 && NR<=4 {print NR": "$0}' testcontainers-0.27.3/src/watchdog.rs → ':1: //! Watchdog that stops and removes containers on SIGTERM, SIGINT, or SIGQUIT' and ':3: //! By default, the watchdog is disabled. To enable it, enable the "watchdog" feature.']`,
which covers an interrupted run and says nothing about a normal exit. The only removal path on a
normal exit is the container handle's own destructor, or the explicit `rm`
`[measured testcontainers@0.27.3 · grep -n 'client.rm(&id)\|env::Command::Remove\|pub async fn rm' testcontainers-0.27.3/src/core/containers/async_container.rs → ':205: pub async fn rm(mut self) -> Result<()> {', ':276: env::Command::Remove => {' and ':277: if let Err(e) = client.rm(&id).await {', the last two inside the Drop impl that opens at ':244']`.

**A container parked in a `static` is therefore never removed, because a `static` is never dropped.**
This is the load-bearing claim of the design, so it was executed rather than reasoned about: a probe
holding one guard in a `static OnceLock` and one guard owned by `main` printed the second guard's
destructor and never the first — and because the pattern did match the line it was meant to match, the
absence is the subject's and not the instrument's
`[measured rustc@1.98.1 · a probe under tmp/ whose main owns Guard("owned-by-main") and whose first trial sets a static OnceLock<Guard>("parked-in-static"), run with cargo test → the log carries "DROP-RAN owned-by-main" after the result line and carries no "DROP-RAN parked-in-static" in any run]`.
So the corpus's own pairing — one container per binary held in a `OnceCell`, and the container
cleaning up after itself — cannot both hold under the default test harness. `AGENTS.md`
§ *Build & Test* states the cleanup half as a standing rule, `ai-docs/rust-test-conventions.md`
§ *Postgres is tested against Postgres* repeats it, and `ai-docs/key-decisions.md` KD-16 repeats it
again
`[measured 0daa273 · grep -rn 'drops it on cleanup' AGENTS.md ai-docs/ → AGENTS.md:145, ai-docs/key-decisions.md:45 and ai-docs/rust-test-conventions.md:58]`,
so the half that gives way cannot be that one.

### The shape this design chooses

The database-backed tests live in one integration-test target of `reader-core` that **owns its own
`main`** — `harness = false`, with `libtest-mimic` supplying the libtest-compatible runner. `main`
starts exactly one container, builds one admin pool against it, hands every trial a database of its
own with the repository's migrations applied to it, and removes the container before it returns. Every
requirement then holds by construction rather than by luck:

| Requirement | How this shape satisfies it |
|---|---|
| AC4 — one container per test binary | `main` starts it, once, before any trial runs `[derived → AC4, and the trials § Test Design names for it]` |
| One container, torn down (`AGENTS.md` § Build & Test) | `main` removes it after `run` returns, on the success path and the failure path alike `[derived → the § Test Design red observation "the container is removed after a failing trial"]` |
| AC3 — a database per test, migrations applied | each trial asks the harness for a fresh database, created and migrated before the trial body `[derived → AC3]` |
| The suite never reads `DATABASE_URL` | no environment variable is read; the DSN is built from the container's own host and mapped port `[derived → D5 and the support module subtask 2 writes]` |
| AC1 — the image really carries pgvector | a trial casts a vector literal on a migrated database `[derived → AC1]` |
| AC5 — the same in CI | the runner image ships a Docker daemon, and testcontainers falls back to its socket `[derived → AC5; the two halves are measured in D10]` |

The teardown ordering was executed, not assumed: the guard owned by `main` is dropped after the result
line, **including on a run where a trial failed**, and the process still exits non-zero
`[measured rustc@1.98.1, libtest-mimic@0.8.2 · the same probe with a deliberately failing trial, run with cargo test → "test result: FAILED. 2 passed; 1 failed" followed by "DROP-RAN owned-by-main", and the run's exit status non-zero]`.
A filter reaches the runner unchanged, and the teardown still runs
`[measured libtest-mimic@0.8.2 · cargo test beta_case in the same probe → "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out" followed by "DROP-RAN owned-by-main"]`.

**Rejected alternatives**, each on a measured ground rather than a preference:

- *`#[sqlx::test]` with the container's DSN written into the process environment.* Blocked twice over.
  The write would have to happen before the first trial connects, and the attribute expands to an
  ordinary libtest function whose body builds the test arguments and runs the body, so the connection
  happens inside the test thread and the only hook earlier than it is a life-before-main constructor
  `[measured sqlx-macros-core@0.9.0 · grep -n 'prelude::v1::test' sqlx-macros-core-0.9.0/src/test_attr.rs → ':70:' in the simple expansion and ':171:' in the one that manages databases, the latter followed by ':172: fn #name() #ret {' / ':173: async fn #name(#inputs) #ret {' and a body that constructs ::sqlx::testing::TestArgs]`.
  The write itself is refused by the compiler in this edition, so it would have to be an `unsafe`
  block whose documented safety condition — no other thread — is exactly what a test harness cannot
  promise
  `[measured rustc@1.98.1 · a probe under tmp/ on edition 2024 calling std::env::set_var outside an unsafe block, built with cargo build → "error[E0133]: call to unsafe function `set_var` is unsafe and requires unsafe block"]`.
  It also puts the developer's own `DATABASE_URL` one failed container start away from being the
  connection the suite uses, which is the accident the standing rule exists to prevent.
- *A `OnceCell` static plus the accepted leak.* This is the corpus's literal shape, and it is the one
  that contradicts the standing cleanup rule — measured above. A design does not knowingly break a
  standing rule, so it is offered to the owner in § Open questions rather than taken.
- *A `Weak` slot that the last concurrent test releases.* Cleanup becomes deterministic and AC4 stops
  being: with `--test-threads=1`, or simply with a gap between two slow tests, the container is
  removed and restarted, which is precisely the "one per test" AC4 names as the failure.
- *A life-before-main / at-exit hook (`ctor` / `dtor`).* Keeps `#[tokio::test]` ergonomics at the cost
  of a blocking container teardown running after `main` has returned, in a process whose runtime is
  gone, ordered against every other registered handler. A harness whose whole purpose is reliability
  should not be the first user of that mechanism here.
- *One `#[tokio::test]` owning the container and driving every case from a table.* Satisfies the
  letter of both rules and empties AC4 of content — the criterion contrasts one container per binary
  with one per test, which presupposes more than one test.
- *A hand-written `main` that runs the test functions itself.* Refused by `AGENTS.md`
  § *Dependency Versions*: a test runner is what `libtest-mimic` already is.

The cost of the chosen shape is that a trial is registered in `main` rather than discovered by an
attribute. That cost is bounded by the lint gate: a test function nobody registers is a function
nobody calls, and the compiler says so
`[measured rustc@1.98.1 · the same probe carrying an unregistered never_registered_case function, run with cargo test → "warning: function never_registered_case is never used", with the note that dead_code is part of the unused lint group and on by default]`,
which this project's lint gate runs with warnings denied
`[measured 0daa273:Makefile:48-49 · awk 'NR>=48 && NR<=49 {print NR": "$0}' Makefile → ':48: clippy:' / ':49: cargo clippy --workspace --all-targets -- -D warnings']`.
So the drift this shape could have introduced — a test written and never run — is a build failure
rather than a silent pass.

### Key decisions

**D1 — The suite reads no environment variable for its connection, so `#[sqlx::test]` is not the
vehicle; the outcome that corpus row states is delivered without it.** The measurements are in
§ *The corpus names a mechanism…* above: one hard-coded variable name, no attribute argument and no
configuration key that reaches the test runtime, and an upstream change still open. What
`docs/03-storage.md` decides in that clause is a *property* — a database per test, with the
repository's migrations applied to it by the suite itself — and that property is what AC3 states,
mechanism-free. The harness delivers it by creating the database and running `reader-core`'s embedded
migrator against it. The vehicle is the only thing that changes — and because the row that names the
vehicle lives in the DECISIONS corpus, the owner was shown the three measurements and ruled on both
halves at once: `[answer 4.1: "Подмена + правка docs. Принять свой main и в этом же PR поправить docs/03-storage.md:7, чтобы корпус описывал то, что действительно работает. Стандартные правила (уборка контейнера, запрет на DATABASE_URL в тестах) остаются нетронуты."]`.
So the substitute is accepted, the corpus row is amended in this same pull request (**D13**), and the
two standing rules that answer names — the container cleaning up after itself, and the ban on the
suite reading `DATABASE_URL` — are left exactly as they are. Nothing in this design edits them:
D12's `AGENTS.md` edit is confined to the coverage tolerance paragraph, and `rust-test-conventions.md`
and KD-16 are not in any subtask's file set.

**D2 — The database-backed target owns its `main`: `harness = false` with `libtest-mimic`.** The
argument is § *The shape this design chooses* in full. The crate is established and current
`[measured libtest-mimic@0.8.2 · cargo info libtest-mimic → "version: 0.8.2", "repository: https://github.com/LukasKalbertodt/libtest-mimic"]`,
and it is a dev-dependency, so nothing it brings reaches a shipped binary. The unit tests of `src/`
are untouched by this: they keep the default harness, and only the database-backed integration target
declares its own.

**How a trial reaches a harness that `main` owns.** A trial runner is `FnOnce() -> Result<(), Failed>
+ Send + 'static`
`[measured libtest-mimic@0.8.2 · awk 'NR>=135 && NR<=137 {print NR": "$0}' libtest-mimic-0.8.2/src/lib.rs → ':135: pub fn test<R>(name: impl Into<String>, runner: R) -> Self' / ':137: R: FnOnce() -> Result<(), Failed> + Send + 'static,']`,
so no closure may *borrow* the harness. The ergonomic reach from there is a leak or a `static`, either
of which silently reinstates the never-dropped container this decision exists to rule out — so the
model is written down rather than left to the implementor:

- The harness is shared by **`Arc<Harness>`**, one clone moved into each trial closure. It is never
  leaked and never parked in a `static`; those two words are the whole point of the paragraph.
- The container sits behind a **take-once slot inside the harness** — an `Option` under a lock —
  because the crate's removal consumes the handle (D3). `shutdown` takes `&self`, empties the slot and
  removes what it found, so it works through an `Arc` and depends on no reference count; a second call
  finds the slot empty and is a no-op rather than an error.
- Sole ownership comes back to `main` because `libtest_mimic::run` **takes the trial vector by value
  and drains it**
  `[measured libtest-mimic@0.8.2 · grep -n 'pub fn run' libtest-mimic-0.8.2/src/lib.rs → ':491: pub fn run(args: &Arguments, mut tests: Vec<Trial>) -> Conclusion {'; and awk 'NR>=560 && NR<=561 {print NR": "$0}' → ':560: let iter = Mutex::new(tests.into_iter());']`,
  so when it returns, every closure and every clone it held is gone. `main` then calls `shutdown`.

**The thread model is the runner's default, kept, and the harness is built for it.** The runner splits
trials across threads by default
`[measured libtest-mimic@0.8.2 · awk 'NR>=535 && NR<=539 {print NR": "$0}' libtest-mimic-0.8.2/src/lib.rs → num_threads falling through to ':538: .or_else(|| std::thread::available_parallelism().ok().map(Into::into))'; and awk 'NR>=561 && NR<=564 {print NR": "$0}' → ':561: thread::scope(|scope| {' / ':564: scope.spawn(|| {']`,
so trials run in parallel unless the operator passes a thread count. That default is kept — a harness
that needed serialising would be a harness with a shared-state defect — and three things follow:

- **Database names are unique by construction, not by hope.** The harness holds an atomic counter;
  each request takes the next value with a fetch-and-add and builds the name from a fixed prefix and
  that number's decimal digits. Nothing else reaches the statement, so the name is unique under any
  thread count and no caller-supplied text is ever interpolated into `CREATE DATABASE`.
- **The runtime handle travels with the harness.** `main` builds a multi-threaded runtime and moves a
  clone of its handle into each trial beside the `Arc`; the trial body blocks on that handle. Blocking
  on a handle is legal from a thread that is not a runtime worker and refused from one that is, and
  the runner's threads are plain scope threads — so the trials qualify, and so does `main`, which
  drives the shutdown the same way. The distinction is the same one D3 turns on from the other side:
  a thread with no runtime in reach at all is where the container's destructor fails, which is why
  the removal is an explicit call and not a drop.
- **The shared path is exercised rather than assumed** (`AGENTS.md` § *Test Conventions*): § Test
  Design adds a trial that asks for several databases concurrently and asserts they are all distinct
  and all migrated. The harness spawns no task and owns no channel, so there is no cancellation path
  to drive; what it shares is the counter, the admin pool and the take-once slot, and that trial —
  with the parallel run behind it — is what drives them.

**D3 — The container is removed by an explicit call inside the async runtime, never by letting the
handle drop outside one.** The destructor's helper resolves the current runtime handle before doing
anything else
`[measured testcontainers@0.27.3 · grep -n 'Handle::current\|pub(crate) fn async_drop' testcontainers-0.27.3/src/core/async_drop.rs → ':16: pub(crate) fn async_drop(future: …)' and ':17: let handle = tokio::runtime::Handle::current();']`,
and `main` is not inside a runtime, so a handle dropped there would resolve a handle that does not
exist. The harness therefore exposes a shutdown that `main` drives through the runtime it owns,
calling the crate's own consuming removal
`[measured testcontainers@0.27.3 · grep -n 'pub async fn rm' testcontainers-0.27.3/src/core/containers/async_container.rs → ':205: pub async fn rm(mut self) -> Result<()> {']`.
Because that call consumes the handle, the handle cannot simply live in the harness as a field every
trial can see: it lives in the take-once slot D2 specifies, which `shutdown` empties. Its result is
reported, not discarded: a container that could not be removed is a message on the way out, because
`AGENTS.md` § *Code Style* forbids discarding a `Result`. The admin pool is closed before the removal,
so the teardown does not race a live connection against a disappearing server.

**D4 — The image is the floating tag the task names, reached by overriding the module image's name and
tag.** The spec's own key-decision row fixes the floating tag («Как есть»), and the tag resolves today
`[measured hub.docker.com · curl -s https://hub.docker.com/v2/repositories/pgvector/pgvector/tags/pg18 → {"name":"pg18","tag_status":"active","last_updated":"2026-08-13T21:29:05.149249Z"}]`.
The Postgres module of `testcontainers-modules` hard-codes a different image, so its name and tag are
overridden rather than its environment or its readiness conditions
`[measured testcontainers-modules@0.15.0 · awk 'NR>=5 && NR<=6 {print NR": "$0}' testcontainers-modules-0.15.0/src/postgres/mod.rs → ':5: const NAME: &str = "postgres";' / ':6: const TAG: &str = "11-alpine";']`,
using the builder the container crate provides for exactly that
`[measured testcontainers@0.27.3 · grep -n 'fn with_name\|fn with_tag' testcontainers-0.27.3/src/core/image/image_ext.rs → the trait declarations at ':60: fn with_name(self, name: impl Into<String>) -> ContainerRequest<I>;' and ':66: fn with_tag(...)', with their impls at :312 and :320]`.
Keeping the module's readiness conditions is the point of overriding rather than rebuilding: it waits
for the ready message on **both** streams
`[measured testcontainers-modules@0.15.0 · awk 'NR>=126 && NR<=131 {print NR": "$0}' testcontainers-modules-0.15.0/src/postgres/mod.rs → ':126: fn ready_conditions(&self) -> Vec<WaitFor> {' over a vec holding WaitFor::message_on_stderr and WaitFor::message_on_stdout, both carrying "database system is ready to accept connections"]`,
which is what distinguishes the initdb server from the real one in that image family.

**D5 — No connection string is ever written as a literal; the connect options are built field by
field, and the container runs with host-trust authentication so there is no password to write.** A
`PostToolUse` hook refuses a file carrying a driver URL with a live-looking password, and it judges
the password by a fixed vocabulary of obviously-fake words that the image's own default is not among
`[measured 0daa273:.claude/settings.json · jq -r '.hooks.PostToolUse[] | select(.matcher=="Write|Edit") | .hooks[].command' .claude/settings.json → a body that greps the written file for "(postgres(ql)?|redis|amqp|mysql)://[^:@/[:space:]]+:[^@/[:space:]]+@" and subtracts a second grep for ":[^@/[:space:]]*(password|passwd|secret|changeme|example|placeholder|xxxx|<[^>]+>|\$\{[^}]+\})[^@/[:space:]]*@", exiting 2 on what is left]`.
Building `PgConnectOptions` from host, port, user and database avoids the shape entirely, and the
module's own switch removes the password from the picture
`[measured testcontainers-modules@0.15.0 · awk 'NR>=38 && NR<=40 {print NR": "$0}' testcontainers-modules-0.15.0/src/postgres/mod.rs → ':38: /// Enables the Postgres instance to be used without authentication on host.' above ':40: pub fn with_host_auth(mut self) -> Self {', whose body sets POSTGRES_HOST_AUTH_METHOD to trust]`.
TLS is disabled explicitly in the options rather than left to the default negotiation, because the
crate is built without a TLS backend (D7) and a server on a mapped loopback port has nothing to
protect.

**D6 — The first migration is a sequential `0001`; its file carries the statement and nothing else;
and it is immutable once committed.** Sequential four-digit versions make AC2's second half — that no
migration precedes it — readable by eye rather than by arithmetic on timestamps, and every later
migration continues the series.

**No comment line, and the reason is mechanical rather than stylistic.** The migration loader reads
the file whole and hands the bytes through unchanged, and the only `--` it understands is the
transaction opt-out, which it *detects* rather than strips
`[measured sqlx-core@0.9.0 · awk 'NR>=225 && NR<=236 {print NR": "$0}' sqlx-core-0.9.0/src/migrate/source.rs → ':225: let sql = fs::read_to_string(&entry_path)', ':234: let no_tx = sql.starts_with("-- no-transaction");' and ':236: let checksum = checksum_with(&sql, &config.ignored_chars);'; and grep -n '"--' on the same file returns only ':234', so nothing removes a comment]`.
So a single `--` line would land inside the embedded statement text *and* inside the checksum, and the
exact-text assertion § Test Design specifies for AC2 would go red — whose cheap repair is to weaken it
to a substring, which is the failure that section warns against in the same breath. The file therefore
holds one statement and one newline. The rationale lives in this design, which is where a reader who
needs it can be pointed; the file itself says nothing. **What a later migration may carry is not
decided here** — only this one is constrained, and it is constrained because it is the one an exact
assertion is pinned to.

The file is written once and never edited: the migrator records a checksum per applied version
`[measured sqlx-core@0.9.0 · grep -n 'pub [a-z_]*:' sqlx-core-0.9.0/src/migrate/migration.rs → the Migration fields version, description, migration_type, sql, checksum, no_tx and the AppliedMigration fields version and checksum]`,
so editing an applied migration's text is the redefinition `AGENTS.md` § *API Stability* carve-out and
INV-14 forbid. There is nothing to roll back here and the design says so plainly: the statement is
`CREATE EXTENSION IF NOT EXISTS vector`, it is idempotent by its own `IF NOT EXISTS`, it writes no row
of application data, and a database that has run it and a database that has not differ only in whether
the type exists. The forward-migration paragraph INV-14 requires is therefore short by nature, not by
omission — no rows exist that this migration could strand.

**D7 — `reader-core` embeds the migrator, with the feature set cut to what that needs.** The migrator
is `sqlx::migrate!()` over the crate's own `migrations` directory, which is where
`docs/03-storage.md` § Задачи puts it and what makes `reader-migrate` able to apply the same set later
without a path at run time. The macro needs **both** feature flags, which is easy to get wrong from
memory: it is declared inside a module compiled only under `macros`
`[measured sqlx@0.9.0 · awk 'NR>=97 && NR<=98 {print NR": "$0}' sqlx-0.9.0/src/lib.rs → ':97: #[cfg(feature = "macros")]' / ':98: mod macros;']`
and is itself gated on `migrate`
`[measured sqlx@0.9.0 · sed -n '855,870p' sqlx-0.9.0/src/macros/mod.rs with grep -n 'macro_rules! migrate' → ':861: #[cfg(feature = "migrate")]', ':862: #[macro_export]', ':863: macro_rules! migrate {' whose arms expand to $crate::sqlx_macros::migrate!]`.
So the dependency is declared with default features off and `runtime-tokio`, `tls-none`, `postgres`,
`migrate` and `macros` on
`[measured sqlx@0.9.0 · sed -n '/^\[features\]/,/^\[\[/p' sqlx-0.9.0/Cargo.toml → the feature table carrying runtime-tokio, tls-none, postgres, migrate and macros among its entries]`.
Turning defaults off drops the multi-backend `any` driver and the JSON types, none of which this task
uses. The version is added with `cargo add`, never by hand-editing the lockfile, and `make lock-check`
is run before staging, per `AGENTS.md` § *Dependency Versions*.

**D8 — No compile-time-checked query and no offline query cache in this task.** `docs/03-storage.md`
§ Задачи asks for `SQLX_OFFLINE=true`, a committed `.sqlx/` and `cargo sqlx prepare --check` in CI.
Every one of those exists to serve the `query!` family, and this task writes none: the harness's few
statements are runtime queries, whose text the compiler never inspects. Introducing the cache now
would put an artefact in the tree that nothing verifies and that the first schema task would
immediately rewrite; more concretely, it would give CI's Build job a reason to want a database, which
it has none of today. The offline machinery arrives with the first `query!`, in the task that adds
one.

**D9 — The harness lives in the test target's own support module, and the trade-off is recorded so the
lift is not re-argued.** `ai-docs/rust-test-conventions.md` § *Where a test lives* names
`crates/<crate>/tests/support/` as the home of a fixture shared between integration tests
`[measured 0daa273:ai-docs/rust-test-conventions.md:9 · sed -n '9p' ai-docs/rust-test-conventions.md → "A shared fixture two integration tests need goes in a module under "crates/<crate>/tests/support/" rather than being copied; three copies of a fixture is the duplication rule's trigger like any other code."]`,
and that is where it goes. The duplication rule's trigger is a *second consumer crate*, not a second
test file: another target inside `crates/core` reaches the same module with a `mod` declaration, so
the copy the rule warns about cannot happen there. It becomes possible the moment a different crate —
`reader-migrate`, whose own database-backed test belongs to the task that gives it behaviour — needs
the same harness. **That is the lift threshold this design records:** the second *crate* to need the
harness moves it into a workspace member of its own, and because that member would carry the container
crates as normal dependencies, the dependency-direction gate keeps it away from the binaries by the
transitive walk it already performs `[measured 0daa273:ai-docs/scripts/import_guard.py:120-143 · grep -n 'def reaches\|return None' ai-docs/scripts/import_guard.py → ':120: def reaches(start: str, forbidden: str, nodes: dict, packages: dict) -> list[str] | None:' closing at ':143: return None', a depth-first walk that follows every non-dev edge and returns the chain]`.
Lifting now would create a member with one consumer, which is the opposite error.

**D10 — The socket is the environment's to name; the build entry point is not touched.** The container
crate resolves its host from `DOCKER_HOST` ahead of every socket fallback — a `tc.host` entry in the
user's `~/.testcontainers.properties` is the only source it consults first — and falls back to the
platform default socket when the variable is unset
`[measured testcontainers@0.27.3 · awk 'NR>=44 && NR<=54 {print NR": "$0}' testcontainers-0.27.3/src/lib.rs → ':44: ##### The host is resolved in the following order:' over a list whose second entry is ':47: 2. "DOCKER_HOST" environment variable.' and whose fourth is ':49: 4. Read the default Docker socket path'; and grep -n 'pub const DEFAULT_DOCKER_HOST' testcontainers-0.27.3/src/core/env/config.rs → ':34: pub const DEFAULT_DOCKER_HOST: &str = "unix:///var/run/docker.sock";']`.
The developer machine exports the variable already
`[measured podman@5.8.2 · printf '%s\n' "$DOCKER_HOST" in a shell initialised from the user's profile → unix:///run/user/1000/podman/podman.sock, and ls -la /run/user/1000/podman/ → a socket named podman.sock]`,
and the CI runner image ships a daemon on the fallback path
`[measured https://raw.githubusercontent.com/actions/runner-images/main/images/ubuntu/Ubuntu2404-Readme.md · WebFetch → the installed-software listing carrying "Docker Client 28.0.4", "Docker Server 28.0.4" and "Docker Compose 2.38.2"]`,
so both ends of AC5 are served without a recipe that hard-codes either. A recipe that exported the
developer machine's path would be wrong in CI, and one that guessed would hide the very failure
`AGENTS.md` § *Build & Test* wants loud. A machine with neither is told so by the failing test, which
is the stated behaviour rather than a regrettable one.

**A consequence for D13's ticking, stated here so it is not read as an oversight:** the corpus row
that pairs the Podman socket with a runner setting the variable
`[measured 435e649:docs/09-build-and-deploy.md:12 · awk 'NR==12 {print NR": "$0}' docs/09-build-and-deploy.md → the row ending "«just test» выставляет DOCKER_HOST на этот путь", with the runner named there being one this repository never adopted — its own first task row offers "just/Makefile" as alternatives]`
is **not** closed by this task, because this design deliberately declines that clause. It therefore
stays unticked under `[answer 4.3]`, which ticks only what is closed in full. Amending it is outside
the single corpus amendment the owner authorised, so it is left alone and recorded here.

**D11 — AC4's primary evidence is the count of container starts; the server's own instant
corroborates it.** This decision is restated rather than patched, because its earlier wording had the
reasoning backwards and licensed a check that cannot fail.

The old reading was that "a counter the harness increments would test the harness". That is the wrong
frame: **AC4 is a claim about how many containers a run starts**, so the harness is the subject of the
measurement, not a mock standing in for one. The count is therefore the primary evidence — on one
condition, which is the whole of the correction: it must be incremented **at the site that awaits the
container start**, so that a second start anywhere in the binary is observable. A counter set to its
expected value in a constructor and never touched asserts nothing at all.

Two consequences, both of which the code must honour:

- **The counter is a process-wide `static AtomicU32`, not a field of a harness instance.** A
  per-instance counter would see a start moved into the per-database path, but not a second `Harness`
  constructed somewhere; and "one container for the whole test binary" is a claim about the process,
  so the counter that answers it has to be one too. **This does not reopen D2**: what D2 forbids in a
  `static` is the *container handle*, whose destructor never running is the entire problem. An atomic
  integer owns no resource and has no destructor.
- **The server's `pg_postmaster_start_time()` stays, as corroboration, and its blind spot is named.**
  Both databases in that case are taken inside one trial, so a harness that started one container *per
  trial* would satisfy it. That is exactly the shape AC4 calls the failure — "not one per test" — and
  only the start count sees it. Neither check subsumes the other, which is why both are specified.

*What the shipped harness does instead.* The counter is a harness field initialised to its expected
value and never incremented, so the trial compares that value with itself
`[measured 333d443:crates/core/tests/support/mod.rs:45,74,80-81 · grep -n 'container_starts' crates/core/tests/support/mod.rs crates/core/tests/database.rs → ':45: container_starts: AtomicU32,', ':74: container_starts: AtomicU32::new(1),', ':81: self.container_starts.load(Ordering::SeqCst)' and database.rs ':195: let starts = harness.container_starts();'; and a sweep for any mutation, grep -rnE 'container_starts *(\.fetch_add|\.store|\.swap|\+=)' crates/ → no output, over the 6 .rs files find crates -name '*.rs' reports, with a constructed control file carrying a fetch_add, a store and a += line matched by the same pattern]`,
which makes the comparison at `crates/core/tests/database.rs:196` a branch with no reachable failure
mode. The correction this decision requires is named in the dash that follows — a process-wide static,
zero at rest, incremented at the start site — plus the red observation § Test Design now demands. It
lands inside subtask 2's file set; **no new subtask is created for it.**

**D12 — The statements this diff falsifies are corrected in the same pull request.** They fall into a
definitive class and a judgement class, separated here so the judgement is visible rather than
smuggled. **The `AGENTS.md` edit below reaches the coverage tolerance paragraph and nothing else** —
in particular neither the container-cleanup sentence nor the `DATABASE_URL` sentence of
§ *Build & Test*, which `[answer 4.1]` leaves untouched and which this design satisfies rather than
relaxes. Definitive: the root manifest's comment that the dependency table is empty because nothing
compiles against anything
`[measured 0daa273:Cargo.toml:16-18 · the awk read cited in § What the tree holds today]`, and the clause "the crates the
workspace holds are skeletons and carry no test", which appears in the build-and-test tolerance
paragraph and in the ratchet script's own header — a pair that is rewritten together or diverges
`[measured 0daa273 · grep -rn -i 'carry no test' --exclude-dir=.git --exclude-dir=tmp --exclude-dir=target --exclude-dir=plans . → AGENTS.md:187 and .githooks/coverage-ratchet.sh:25, and no other live site]`.
After this task a crate carries a test, so both halves are false and the *reason* they give for the
tolerance is what is rewritten; the tolerance itself does not move, because a drift series still has
not been run. Judgement: the file-size bands' justification, which says the bands are unmeasured
because the crates are skeletons, in the build entry point's comment and in its twin in the code-style
reference — the same pair the previous task rewrote together
`[measured 0daa273 · grep -rn -i 'crates it holds' --exclude-dir=.git --exclude-dir=tmp --exclude-dir=target --exclude-dir=plans . → Makefile:31 and ai-docs/code-style.md:90, and no other live site]`.
A crate now carries its first code and its first test, so "the crates it holds are skeletons" is no
longer true of the set; the bands still do not move, because a migration and a harness are not the
distribution those numbers wait for. That deferral is kept explicitly, and the sentence is corrected
to say what is actually true. The gate-script sentences that look like the same class are
**not** touched, for the reason the previous task recorded: each is governed by its own condition and
describes what the script does rather than what this repository is — the dependency-direction gate's
`WHILE THE WORKSPACE IS EMPTY` closer, its no-binary branch message, and the ratchet's
no-executable-lines branch comment.

**D13 — The corpus row is amended in this pull request, and the checkboxes this task closes in full
are ticked; everything else in `docs/` is left alone.** Both halves are the owner's, given after the
measurements in § *The corpus names a mechanism…* were put to them:
`[answer 4.1: "Подмена + правка docs. Принять свой main и в этом же PR поправить docs/03-storage.md:7, чтобы корпус описывал то, что действительно работает. Стандартные правила (уборка контейнера, запрет на DATABASE_URL в тестах) остаются нетронуты."]`
and
`[answer 4.3: "Отметить закрытое. Отметить те пункты, которые эта задача закрывает целиком, остальные оставить."]`.
`docs/` is Russian (`AGENTS.md` CRITICALLY 1), so the amended row is written in Russian like the rest
of the page.

*What the amended row must assert*, clause by clause — the first is unchanged, the next three replace
a mechanism that does not exist, and the last is a deliberate non-change:

| Clause | After the amendment |
|---|---|
| The image and the socket | Unchanged: `testcontainers` with `pgvector/pgvector:pg18` over the Podman socket, `DOCKER_HOST` naming it |
| One container per test binary | Kept as the property; the vehicle becomes the test binary's **own `main`** (`harness = false`), because a value parked in a `OnceCell` static is never dropped — so `OnceCell` goes |
| Cleanup | The container is removed by an explicit call from that `main` when the run ends, because the crate ships no ryuk — so "ryuk включен" goes, and the standing cleanup rule it was serving is what the replacement keeps |
| A database per test | Created and migrated by the harness itself, because `#[sqlx::test]` takes its connection only from `DATABASE_URL` and the suite may not read it — so the attribute goes, and the property it was there for stays |
| The version parenthetical | **Left exactly as it is.** The spec's own key decision fixes the floating tag «Как есть»; that clause describes the developer machine at the time of writing rather than a requirement the harness enforces, and rewriting it would be an amendment the owner did not authorise |

The row should carry its *reason* in the same sentence, so the next reader does not re-propose the
mechanism that was measured away. `docs/` pages cite each other in prose and are outside the
comment-reference gate, which reaches source files by extension and not markdown (§ *What the gates
will read afterwards*), so a short inline reason is allowed there in a way it is not in a migration's
SQL comment.

*Which boxes are ticked.* Only rows this task closes **in full**, which is what the answer says:

| Row | Ticked? | Why |
|---|---|---|
| `docs/03-storage.md:7` — the test harness | **yes**, once amended | the harness the amended row describes is exactly what this task delivers |
| `docs/03-storage.md:8` — the `vector` extension in the first migration | **yes** | D6 delivers that statement as the first migration and nothing else |
| `docs/03-storage.md:4` — `sqlx`, the migrations directory, the pool, the offline query cache | no | D8 defers `SQLX_OFFLINE`, the committed query cache and `cargo sqlx prepare --check`, and no application pool is built here; the row is part-done, so it stays open |
| `docs/09-build-and-deploy.md:12` — the Podman socket | no | D10 declines the clause about a runner exporting the variable, so the row is not closed — see the note in D10 |
| every other row of either page | no | untouched by this task |

`[measured 435e649:docs/03-storage.md:4-8 · awk 'NR>=4 && NR<=8 {print NR": "$0}' docs/03-storage.md → the five unticked task rows of § Задачи, ':7:' the testcontainers row naming ryuk, OnceCell and the sqlx test attribute, and ':8:' the vector-extension row]`

**D14 — The container crate sits at 0.27.3, and that is a coupled constraint rather than a stale
pin.** The design's coordinates were first measured against 0.28.0; the implementation resolved
0.27.3, and the owner sent the divergence back through the normal path rather than letting either
side drift: `[answer 5.1: "Правка + ревью. Штатный путь: design-writer приводит координаты D3/D4/D5 к 0.27.3, затем design-review прогоняется заново."]`.
The version the lockfile holds
`[measured 0f1322b:Cargo.lock · awk '/^name = "testcontainers/{n=$3} /^version/{if(n){print n, $3; n=""}}' Cargo.lock → "testcontainers" "0.27.3" and "testcontainers-modules" "0.15.0"]`.

*Why it is not simply raised.* The Postgres module is what selects the line: its published manifest
and the registry index both put its requirement on the 0.27 series, for the normal and the dev
dependency kind alike, and no newer module crate exists to lift it
`[measured testcontainers-modules@0.15.0 · the sparse index at index.crates.io/te/st/testcontainers-modules → the newest non-yanked version is 0.15.0, whose deps list carries testcontainers "^0.27.0" twice, once with kind normal and once with kind dev; and grep -n -A2 on the published Cargo.toml → ':209: [dependencies.testcontainers]' / ':210: version = "0.27.0"' and ':357: [dev-dependencies.testcontainers]' / ':358: version = "0.27.0"']`,
while the container crate's own newest release is 0.28.0
`[measured testcontainers · the sparse index at index.crates.io/te/st/testcontainers → the newest non-yanked version is 0.28.0, with 0.27.3 present]`.

*And the refusal is one level deeper than the bound, which is the part worth writing down.* Cargo will
happily hold two semver-incompatible lines of the same crate, so "add 0.28.0 beside it" looks
reasonable and is the thing a later reader will try. It was tried here, and the resolver refused —
not on `testcontainers` at all, but on a transitive package the two lines pin to exact and mutually
exclusive versions
`[measured cargo@1.98.1 · a scratch package under tmp/ requiring testcontainers "0.28.0" and testcontainers-modules "0.15.0" with the postgres feature, run with cargo generate-lockfile → exit non-zero and "error: failed to select a version for bollard-stubs", whose two chains read "... required by package bollard v0.20.0 ... which satisfies dependency testcontainers = ^0.27.0 of package testcontainers-modules v0.15.0" against the previously selected "bollard-stubs v1.53.1-rc.29.3.1 ... of package bollard v0.21.0 ... which satisfies dependency testcontainers = ^0.28.0", ending "failed to select a version for bollard-stubs which could resolve this conflict"]`.
So raising the container crate is a **coupled move, not a version bump**: it needs a module release
whose bound admits it. The other escape is to stop using the module and hand-build the image, which
forfeits the readiness conditions D4 keeps on purpose — that is a design change, and it belongs in a
task that argues for it.

*What the move cost this design: nothing, and that was checked rather than assumed.* A diff of the
two `src` trees shows the releases differing in nothing this design's mechanism reaches: a bollard type rename on the mount conversion, which is dead code
here because the harness mounts nothing, and the ssh sidecar's image tag, which belongs to a feature
this design does not enable. Those are the whole of the difference between the two `src` trees; the
published manifests differ too, and in the bollard requirement this decision has already been through
`[measured testcontainers@0.27.3 vs @0.28.0 · diff -rq testcontainers-0.27.3/src testcontainers-0.28.0/src → only src/core/containers/host.rs and src/runners/async_runner.rs differ; diff -u on each → MountTypeEnum renamed to MountType in the From<&Mount> impl, and ssh_tag moved from "1.3.0" to "1.4.0" behind the host-port-exposure feature]`.
In particular the ryuk argument, which D2 leans on hardest, was re-measured on 0.27.3 rather than
carried over — see § *The corpus names a mechanism…*, whose sweep and control were both re-run there.

### What the gates will read afterwards

The comment-reference ban reaches SQL by file name, so the migration's own comments are gated exactly
as a Rust comment is
`[measured 0daa273:ai-docs/scripts/comment_refs.py:42 · sed -n '42p' ai-docs/scripts/comment_refs.py → 'GATED_SOURCE_EXTS = (".rs", ".sh", ".sql", ".yml", ".yaml")']`.
Nothing this task writes — in the migration, in `crates/core/src/lib.rs`, in the support module or in
the test target — may carry a markdown path, an acceptance-criterion id, a decision anchor, an issue
number outside `TODO(#…)`, a repository path or a URL. Two consequences are specific enough to be
worth naming: **the migration file carries the statement and no comment at all** (D6 decides it and
gives the reason, which is stronger than the reference ban), so the ban has nothing to bite on there
and binds the migrations that come later instead; and KD-19's ruling binds every doc comment this task writes —
inside `crates/core` the crate's own symbols are written in the directory-name form, and a `core::`
path is unwritable in any gated comment outside that directory
`[measured 0daa273:ai-docs/key-decisions.md:51 · grep -n 'KD-19' ai-docs/key-decisions.md → ':51:' carrying the KD-19 row whose consequence fixes the directory-name form for a crate's own contract symbol and states that a "core::" path is unwritable in a gated comment outside "crates/core"]`.

The documentation gate denies rustdoc warnings
`[measured 0daa273:Makefile:51-55 · awk 'NR>=51 && NR<=55 {print NR": "$0}' Makefile → the comment naming rustdoc-with-warnings-denied the doc-comment gate, then ':54: doc-check:' / ':55: RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps']`,
so the public migrator constant carries a doc comment and any intra-doc link in it resolves
`[derived → the documentation gate on the subtask-1 commit]`. The panic
gate reads shipped code only and takes test code out of scope by position
`[measured 0daa273:ai-docs/panic-index.md · sed -n '/Test code is out of scope/p' ai-docs/panic-index.md → "Test code is out of scope by position: a "#[cfg(test)]" module, and everything under "tests/", "benches/" and "examples/"."]`,
so the harness may `unwrap` where a panic is the intended failing-test outcome, while `crates/core/src`
gains no panicking call and the index stays empty
`[derived → the panic gate on the subtask-1 and subtask-2 commits]`. The dependency-direction gate
passes because both container crates and the runner are declared as dev-dependencies, on edges it
skips by the rule measured at the top of § Approach
`[derived → the dependency-direction gate on the subtask-1 commit]`.

**The workflow's own paths filter is checked here because the file asks for it in writing**, and a
universal "no gate needs changing" that skipped it would rest on the one place the repository names as
mandatory. Its comment reads that any future Rust or harness artefact must be added in the same pull
request that introduces it, or its gate silently stops running. The artefact classes this task
introduces are Rust sources, a SQL migration and the cargo manifests, and each is already named in
both filters that would have to reach them
`[measured 128c2fb:.github/workflows/ci.yml:36,39-49,86-89 · grep -n "rust:\|commentrefs:\|'\*\*/\*\.rs'\|'\*\*/\*\.sql'\|'\*\*/Cargo.toml'\|'Cargo.lock'\|Any future Rust or harness artefact" .github/workflows/ci.yml → ':36:' the standing instruction, ':39: rust:' over ':40: - "**/*.rs"', ':41: - "**/*.sql"', ':48: - "**/Cargo.toml"', ':49: - "Cargo.lock"', and ':86: commentrefs:' over ':87: - "**/*.rs"' and ':89: - "**/*.sql"']`,
so **no filter entry is added by this task** — the check is recorded, not a change. One neighbouring
fact is recorded with it rather than acted on: no filter names `docs/**`, so a commit touching only
the corpus page reaches no job
`[measured 128c2fb:.github/workflows/ci.yml · grep -n 'docs/' .github/workflows/ci.yml → only ai-docs paths, ':55: - "ai-docs/**"' among them, and no bare docs entry; a constructed control line carrying "docs/**" was matched by the same pattern, so it ran]`.
That is a property `docs/` already had before this task and not one subtask 5 introduces, and this
pull request reaches the harness job through its other paths regardless, so nothing is proposed for
it here.

The coverage ratchet acquires a new precondition that is worth stating out loud rather than
discovering: it refuses a commit when the suite is not green, and from this task on the suite needs a
container runtime, so **every commit that stages a `.rs`, a `.sql` or a manifest now needs a reachable
socket**
`[measured 0daa273:.githooks/coverage-ratchet.sh:92-96,119-126 · awk 'NR>=92 && NR<=96 {print NR": "$0}' and awk 'NR>=116 && NR<=127 {print NR": "$0}' .githooks/coverage-ratchet.sh → ':95: staged=$(git diff --cached --name-only --diff-filter=ACMR \' over '*.rs', '*.sql' and the manifests, and ':119: if ! cargo llvm-cov …; then' whose branch prints 'BLOCKED — the test suite is not green' and ':123: If no container runtime is reachable, start the Podman socket the suite connects to']`.
A commit that stages only documents is unaffected, which is most of a run.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | **The dependency set, the first migration, and the embedded migrator.** Add the workspace dependency entries with `cargo add`, never a hand-edited lockfile, taking the container crate from the 0.27 series rather than the newest release (the Postgres module's bound decides it and the resolver refuses the alternative — **D14**). Rewrite the root manifest's now-false comment about the empty table (D12). Declare `sqlx` as a normal dependency of `reader-core` with default features off and the set D7 fixes, and the dev-dependencies the next subtask needs — the container crate, its Postgres module, the runner and the async runtime — so the manifest is written once. Add `crates/core/migrations/0001_vector_extension.sql` carrying the single `CREATE EXTENSION IF NOT EXISTS vector` statement and **no comment line** (D6 — the loader keeps the file's bytes verbatim, so a comment would land inside both the embedded text and the checksum), and expose the embedded migrator from `crates/core/src/lib.rs` with a doc comment that obeys the reference ban and KD-19 (§ *What the gates will read afterwards*). Add the `#[cfg(test)]` module beside it asserting the embedded set against AC2 — the lowest-versioned migration is version 1, described **`vector extension`** (the loader replaces the file name's underscores with spaces, so the description is never the file name's spelling — § Test Design carries the measurement), no migration carries a lower version, and its statement is exactly the one above. That test needs no container and is the half of AC2 that a machine without a runtime can still check. | `Cargo.toml`, `Cargo.lock`, `crates/core/Cargo.toml`, `crates/core/migrations/0001_vector_extension.sql`, `crates/core/src/lib.rs` | — |
| 2 | **The container harness and the database-backed test target.** Declare the target with its own `main` in `crates/core/Cargo.toml` (`harness = false`). Write the support module under `crates/core/tests/support/`: start one container from the overridden image (D4), build one admin pool over connect options assembled field by field with TLS disabled and no password (D5), hand out a freshly created and migrated database per caller, and expose the shutdown `main` drives through its runtime, whose result is reported rather than discarded (D3). Hold the container in a take-once slot and hand the harness out as an `Arc` — never a leak, never a `static` — with the per-database name built from a fixed prefix and an atomic counter so it is unique under the runner's default parallelism (D2). Write the test target: `main` builds a multi-threaded runtime, starts the harness, registers the trials § Test Design names — each closure taking an `Arc` clone and a runtime-handle clone — runs them, shuts the harness down after `run` returns, and propagates the runner's verdict as the process's exit status. | `crates/core/Cargo.toml`, `crates/core/tests/support/mod.rs`, `crates/core/tests/database.rs` | 1 |
| 3 | **Correct the statements this diff falsifies (D12).** Rewrite the tolerance paragraph's reason clause and its twin in the ratchet script's header so both say what is now true — a crate carries a test, and the tolerance still has no drift series behind it — leaving the tolerance value and the script's conditional branches untouched. Rewrite the file-size bands' justification in the build entry point's comment and in its twin in the code-style reference to the same judgement: the crates carry their first code, and the bands still wait for a real distribution, so they do not move. Leave the condition-governed gate-script sentences § D12 enumerates alone; they are absent from this file set on purpose, because an untouched listed file reads as a missed site. The build entry point is in the comment-reference gated set, so a rewritten comment there obeys the same ban as a Rust one. | `AGENTS.md`, `.githooks/coverage-ratchet.sh`, `Makefile`, `ai-docs/code-style.md` | 2 |
| 4 | **Record the harness decision where it will be looked for.** Add a key-decision row, in the page's own shape — decision, why, consequence, source — numbered after the last row the page carries, stating that the database-backed target owns its `main` so that one container serves the binary *and* is removed when the run ends, and that `#[sqlx::test]` is not the vehicle because its only connection source is the variable the suite is forbidden to read. The *consequence* field carries what a later test author inherits: a trial is registered in `main`, an unregistered one is a denied lint rather than a silent pass, and the lift threshold D9 fixes. The row also records that the corpus row naming the old mechanism was amended in the same pull request on the owner's authorisation, so a later reader meets the amendment and its reason together. The *source* field is backticked prose, not a markdown link, and names this design at the path it carries after Step 12 — `ai-docs/plans/done/2026-09-19-postgres-test-harness-migration.design.md` § D1–D3 and § D13 — because the pre-retirement path is stale before the pull request opens. | `ai-docs/key-decisions.md` | 2 |
| 5 | **Amend the corpus row and tick what this task closes in full (D13).** Rewrite `docs/03-storage.md`'s test row so the corpus describes the harness that exists: the image and the socket unchanged, one container per test binary owned by the test binary's own `main`, the container removed by an explicit call from that `main`, and a database per test created and migrated by the harness itself — each replacement carrying its one-clause reason, and the version parenthetical left exactly as it stands. Written in Russian, like the page. Then tick the rows this task closes in full — the amended test row and the vector-extension row — and leave every other checkbox on that page and on `docs/09-build-and-deploy.md` unticked, including the Podman-socket row D10 declines. **Touch no other rule text:** the container-cleanup and `DATABASE_URL` sentences of `AGENTS.md` § *Build & Test*, the matching paragraph of `ai-docs/rust-test-conventions.md` and KD-16 stay as they are, which is what the owner's answer requires. Trace any relative link this edit leaves with `realpath` before committing, because the harness job's link check sweeps every tracked markdown file. | `docs/03-storage.md` | 2 |

## Handoff plan

`M = 5`. Two groups, homogeneous by change-type and minimised: the code subtasks are consecutive, and
every document subtask depends on them and on none of the others, so no dependency chain forces an
interleave and they cluster into one group. Two groups is within the default maximum of four, so no
user approval is needed. The owner's round-4 answers added subtask 5 without moving a boundary: it is
a markdown edit, so it joins the group that already holds the document subtasks. The review round
after it moved no boundary either — every item it raised lands inside a subtask that already exists
(the migration's contents and the exact assertion in 1, the sharing and thread models in 2, the
workflow's paths filter recorded rather than changed), so `M`, the grouping and the change-type
homogeneity are all unchanged and re-checked rather than merely restated. The version amendment that
followed moved nothing either: it rewrites coordinates and adds **D14**, touching no file set and
creating no subtask. Nor does the review round after it: correcting the migration's description and
making AC4's start count falsifiable (**D11**) both land inside file sets subtasks 1 and 2 already
own, so `M`, the grouping and the change-type homogeneity are unchanged again. The code correction
D11 requires is a change to files Group A has already committed, and the orchestrator routes it — this
design decides what the correct fix is, and creates no subtask to carry it.

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md`
  § Compaction recovery (re-entry). The first group takes a handoff exactly as every later one does.
- **Group A** — model `sonnet`, effort `medium` (pinned in the `code-writer` frontmatter; no inline
  `model=` and no effort override, because there is no per-invocation effort parameter), 1M-token
  window, via `subagent_type="code-writer"` — subtasks 1, 2 (code change-type: `*.rs`, the migration
  `*.sql`, and the cargo manifests and lockfile that ship with them). The ratchet file this group's
  commits may carry is written and staged by the pre-commit hook rather than authored, so the group
  stays homogeneous.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md`
  § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B** — model `inherit` (the orchestrator's), effort inherited from the orchestrator
  (typically xHigh, not pinned), 1M-token window, via `subagent_type="general-purpose"` with no inline
  `model=` — subtasks 3, 4, 5 (instructions/harness change-type: `AGENTS.md`, `Makefile`,
  `.githooks/**`, `ai-docs/**`, `docs/**`). Terminal group (3 subtasks; within the `1..=10` range).

The three share a group and no file: subtask 3 edits `AGENTS.md` and the ratchet script as a pair and
the build entry point and the code-style reference as a second pair, subtask 4 writes only to the
key-decisions page, and subtask 5 only to the corpus page. They are sequential commits inside one
group, so each reads the previous one's result.

## Risks

- **The migration's checksum is immutable the moment it is applied anywhere, so a "small fix" to the
  file after this pull request merges is a data-corruption bug wearing a refactor's clothes.**
  Mitigation: subtask 1's unit test pins the statement's exact text, so an edit to the file fails the
  suite instead of passing quietly, and D6 states the rule in the design where a later author will
  look — `[measured sqlx-core@0.9.0 · grep -n 'pub [a-z_]*:' sqlx-core-0.9.0/src/migrate/migration.rs → the Migration and AppliedMigration checksum fields]`, `[derived → the § Test Design case "the embedded migration is exactly the vector-extension statement"]`.
- **A green harness is a claim about the harness until it has been seen red.** A container that never
  started, an image without the extension and a test that asserts something true of every Postgres all
  look identical from a passing run. Mitigation: § Test Design makes its red observations a required
  part of subtask 2 rather than an optional courtesy, each recorded in the progress file with the
  output that was seen — `[derived → the § Test Design section "Red observations required before the group's last commit"]`.
- **Docker Hub meters anonymous pulls per address, and CI runners share addresses.** The documented
  allowance is 100 pulls per six hours for an unauthenticated puller, counted per IPv4 address or IPv6
  /64 subnet
  `[measured https://docs.docker.com/docker-hub/usage/ · WebFetch → "100 per IPv4 address or IPv6 /64 subnet" within a six-hour window for unauthenticated users, against 200 per six hours for an authenticated free account]`,
  so AC5 can go red for a reason that has nothing to do with this repository. Mitigation: **none is
  built, by the owner's decision** — `[answer 4.2: "Ничего сейчас. Не усложнять задачу; если CI упрётся в лимит — разбираться отдельной задачей по факту красного прогона."]`.
  The row stays here as a recorded risk and nothing more: no registry login step, no mirrored image,
  no retry wrapper is designed for or built. A red run against the limit is the trigger for a separate
  task, not for work inside this one.
- **From this task on, a commit that stages Rust, SQL or a manifest needs a reachable container
  socket**, because the ratchet refuses a commit whose suite is not green and the suite now provisions
  a database — `[measured 0daa273:.githooks/coverage-ratchet.sh:92-96,119-126 · the two awk reads cited in § What the gates will read afterwards → the staged-set skip over '*.rs', '*.sql' and the manifests, and the BLOCKED branch whose message names the Podman socket]`.
  Mitigation: the ratchet already prints the socket instruction on that branch, so the failure is
  self-explaining; subtask 3 leaves that branch's comment alone for the reason D12 gives.
- **The container crate's destructor resolves a runtime handle, so a handle dropped on a path that
  bypasses the explicit shutdown panics instead of cleaning up** — `[measured testcontainers@0.27.3 · grep -n 'Handle::current' testcontainers-0.27.3/src/core/async_drop.rs → ':17: let handle = tokio::runtime::Handle::current();']`.
  Mitigation: D3 puts the shutdown inside the runtime `main` owns, and § Test Design requires the
  teardown to be observed on a failing run as well as a passing one, which is the path where an early
  return would hide — `[derived → the § Test Design case "the container is removed after a failing trial"]`.
- **The image tag floats, so a registry push can change the server under the suite without a commit
  here.** That is the spec's own key decision («Как есть»), not a defect, but it means a suite that
  goes red on a tag move is red for an external reason. Mitigation: the trials assert the extension's
  *usability* rather than a version string, so a compatible rebuild passes and an incompatible one
  fails loudly with the database's own error — `[derived → AC1's trial, which casts a vector literal rather than reading a version]`.
- **`reader-core` gains its first dependency, so the workspace's resolved graph becomes something the
  lockfile gate can disagree about.** Mitigation: `cargo add` for every entry, then a build, then
  `make lock-check` before staging, and `git diff Cargo.toml Cargo.lock` read before the commit, per
  `AGENTS.md` § *Dependency Versions* — `[measured 0daa273:Makefile:64-68 · awk 'NR>=64 && NR<=68 {print NR": "$0}' Makefile → the comment explaining that --locked fails rather than rewriting the lockfile, then ':67: lock-check:' / ':68: cargo metadata --locked --format-version 1 >/dev/null']`.

## Test Design

Everything this section specifies is an artefact the task creates, so its claims carry
`[derived → …]`. The one fact it leans on from outside the tree — the extension's literal syntax —
carries its own measurement where it is used.

### Beside the code — `crates/core/src/lib.rs`, `#[cfg(test)] mod tests`

- **Entry point:** the embedded migrator constant.
- **Case — the embedded migration set satisfies AC2.** The lowest-versioned migration carries version
  `1`, the description **`vector extension`** — the loader derives a description from the file name by
  stripping the type suffix and replacing underscores with spaces, so the file `0001_vector_extension.sql`
  yields a description with a space and never the file name's spelling
  `[measured sqlx-core@0.9.0 · awk 'NR>=217 && NR<=223 {print NR": "$0}' sqlx-core-0.9.0/src/migrate/source.rs → ':220: let description = parts[1]' / ':221: .trim_end_matches(migration_type.suffix())' / ':222: .replace('_', " ")'; and the suffix for a simple migration is ".sql" per MigrationType::suffix in sqlx-core-0.9.0/src/migrate/migration_type.rs:61]` —
  and a statement whose trimmed text is exactly `CREATE EXTENSION IF NOT EXISTS vector`; and no
  migration in the set carries a version below it. The statement assertion is on the exact text, not
  on a substring: a substring assertion passes for a file that grew a second statement, which is the
  half of AC2 that matters. `trim` is there for the file's trailing newline and for nothing else — the
  loader keeps every other byte, comments included (D6), so **if the statement assertion goes red the
  repair is the migration file, never the assertion.** Weakening it to a substring, or teaching it to
  strip comment lines, hands back exactly the property it exists to hold.

  **That prescription is scoped to the statement text and to nothing else, and the scope is load-bearing.**
  The version and description assertions are derived from the file's *name*, so "repair the file" would
  there mean renaming an applied migration — which moves its version, its description and its recorded
  checksum at once, and is precisely the redefinition INV-14 and the `AGENTS.md` § *API Stability*
  carve-out forbid, and which § Risks names as this task's first risk. If a version or description
  assertion disagrees with the file name, the correct move is to find out which of the two is wrong and
  say so, never to rename a migration that has been applied anywhere `[derived → AC2]`.
- **Why it lives here and not in the container target:** it asserts what the *repository* embeds, not
  what a database did with it, so it must stay runnable on a machine with no container runtime
  `[derived → AC2]`.
- **Fixtures:** none. The migrator is a compile-time constant.

### The database-backed target — `crates/core/tests/database.rs`, with `crates/core/tests/support/`

- **Location:** an integration target, because the behaviour is only observable across the crate
  boundary and needs a real server (`ai-docs/rust-test-conventions.md` § *Where a test lives*).
- **Harness:** `main` owns the runtime, the container and the admin pool; each trial receives a handle
  to the shared harness and asks it for a database of its own.
- **Fixtures:** none recorded on disk. The subject is the server, and the only inputs are SQL literals
  written in the trials. No golden is minted by this task, so the golden rule has nothing to bind
  here.

Trials:

- **`vector_type_is_usable`** — on a freshly migrated database, cast a bracketed vector literal and
  assert the round-tripped text is exactly that literal. The task text's shorthand `SELECT 'x'::vector`
  is not a parseable value of the type — the extension's literal form is the bracketed list
  `[measured https://raw.githubusercontent.com/pgvector/pgvector/master/README.md · WebFetch → every vector value in the examples written in bracketed form, '[1,2,3]' and '[3,1,2]' among them, and no unbracketed form shown]` —
  so the trial uses the real literal, which is the "или эквивалент" the task allows. Asserting the
  exact round-trip rather than "no error" is what makes the trial fail on a server that accepted the
  cast and stored something else `[derived → AC1]`.
- **`migrations_are_applied_to_every_database`** — on a freshly created database, assert the extension
  is present in the catalogue *and* that the applied-migration table records exactly the versions the
  embedded set carries. Both halves are needed: the first without the second would pass on an image
  that ships the extension pre-created, which is precisely the confusion AC1 and AC2 are separate
  criteria for `[derived → AC3]`.
- **`each_trial_gets_its_own_database`** — one trial takes two fresh databases, asserts their reported
  current-database names differ, creates a table in the first, and asserts the second does not have
  it. Written inside one trial rather than across two, because cross-trial ordering is not something
  the runner promises and a test that depends on it is a flake `[derived → AC3]`.
- **`one_container_serves_the_whole_binary`** — the process-wide start count, incremented at the site
  that awaits the container start, is asserted to be one; and the two databases of the previous case
  report the identical postmaster start instant, the server's own evidence that one server backs both.
  The count is the primary evidence and the instant corroborates it, in that order and for the reason
  D11 gives: the instant cannot see a container started once per trial, and the count can. Neither is
  decoration, and the count's failure mode is exercised below rather than asserted to exist
  `[derived → AC4]`.
- **`concurrent_requests_get_distinct_databases`** — the shared path the runner's default parallelism
  puts the harness on is driven rather than assumed (`AGENTS.md` § *Test Conventions*): one trial asks
  for several databases from concurrent tasks, joins them, and asserts every reported current-database
  name is distinct and every one of them carries the migrations. It is the counter, the admin pool and
  the creation path under contention that this drives — the parallel *run* exercises them too, but
  only incidentally and only when the operator has not passed a thread count, which is not a property
  a test may rest on. The harness spawns no task and owns no channel, so this diff opens no
  cancellation path and none is asserted `[derived → AC3 and AC4]`.

### Red observations required before the group's last commit

Each of these is performed, its output pasted into the progress file, and the mutation reverted. A
green suite that has never been seen red is a claim about the suite (`AGENTS.md` § Patterns 2).

- **The extension check really checks the extension.** Point the harness at the plain `postgres` image
  of the same major version and confirm `vector_type_is_usable` fails with the server's own
  unknown-type error. This is the control that separates "the image carries pgvector" from "the query
  ran". **The image is changed by editing the harness's own image coordinates in place and reverting
  the edit afterwards** — the harness grows no environment variable, no feature flag and no
  configuration surface for it, because a switch that exists only to be flipped during one verification
  is a permanent surface bought for a single use, and `git diff --name-only` before the commit is what
  confirms the revert landed `[derived → AC1]`.
- **A machine with no runtime is told loudly.** Run the suite with the socket variable pointing at a
  path that does not exist and confirm the target fails with a connection error rather than skipping,
  reporting zero tests, or passing. Confirm the mutated value is what the process actually saw before
  reading the verdict `[derived → the standing rule in "AGENTS.md" § Build & Test]`.
- **The isolation trial really tests isolation.** Make the harness hand out the same database name
  twice and confirm `each_trial_gets_its_own_database` fails. A fixture that never reaches the clause
  makes a green assertion meaningless `[derived → AC3]`.
- **The start count can actually go red.** Add a second harness start to `main`, or move the start
  into the per-database path, and confirm `one_container_serves_the_whole_binary` fails naming the
  count it observed; then revert and confirm the revert with `git diff --name-only`. A count
  initialised to its expected value and never incremented passes for every possible program, and that
  is the shape this observation exists to rule out — it is also why D11 puts the increment at the start
  site rather than in a constructor `[derived → AC4]`.
- **The container is removed after a failing trial.** With one trial forced to fail, confirm the
  container is gone from the runtime's container list after the process exits, and that the process
  exit status is non-zero. The failing path is where an early return would hide the teardown
  `[derived → the standing cleanup rule in "AGENTS.md" § Build & Test]`.

### Gates

`make verify` is run before each of the group's commits, and `make cover-ratchet` is what the
pre-commit hook runs on the coverage-moving ones. Both now need a reachable socket, per § Risks.

## Open questions

- **The corpus row that names mechanisms this design does not use — CLOSED.** The measurements are in
  § Approach: `#[sqlx::test]` takes its connection only from `DATABASE_URL` and offers no override,
  the container crate ships no ryuk, and a container held in a `OnceCell` static is never dropped, so
  the row's own pairing of "one container per binary" with cleanup cannot both hold. The owner
  answered
  `[answer 4.1: "Подмена + правка docs. Принять свой main и в этом же PR поправить docs/03-storage.md:7, чтобы корпус описывал то, что действительно работает. Стандартные правила (уборка контейнера, запрет на DATABASE_URL в тестах) остаются нетронуты."]`.
  Both halves are folded in: the substitute stands (**D1**, **D2**, **D3**), and the corpus row is
  amended in this same pull request as **subtask 5**, whose clause-by-clause contract is **D13**. The
  two standing rules the answer protects are edited nowhere — D12 states that the `AGENTS.md` edit
  reaches only the coverage tolerance paragraph. **No spec row is created or changed by this**: the
  authorisation is design work, recorded here with the owner's words, and the spec stays as approved.
- **Ticking the `docs/` checkboxes — CLOSED.** The owner answered
  `[answer 4.3: "Отметить закрытое. Отметить те пункты, которые эта задача закрывает целиком, остальные оставить."]`.
  Folded into **D13**, which names the rows ticked — the amended test row and the vector-extension row
  of `docs/03-storage.md` — and the rows left open with the reason each is only part-done, the
  Podman-socket row of `docs/09-build-and-deploy.md` among them (**D10**).
- **A Docker Hub pull limit in CI — CLOSED.** The owner answered
  `[answer 4.2: "Ничего сейчас. Не усложнять задачу; если CI упрётся в лимит — разбираться отдельной задачей по факту красного прогона."]`.
  Folded into § Risks, which keeps the measured allowance as a **recorded risk and nothing more**: no
  registry login, no mirrored image, no retry wrapper is designed for or built, and a red run against
  the limit is the trigger for a separate task.
- **No spec row was found to prescribe a mechanism this design would otherwise have chosen
  differently, so no `SPEC-REMIT` tag is raised.** Scope rows 1–4 and AC2 each name a mechanism —
  testcontainers, the Podman socket, the image tag, one container per binary, `#[sqlx::test]`, a single
  `CREATE EXTENSION` statement — and each is the owner's own wording in the issue body as the interview
  state file persists it, anchored as such in the spec, rather than a spec-side choice of how. The
  criteria themselves are stated as outcomes: AC3 says a database of its own with migrations applied,
  and names no vehicle, which is why the substitute in D1 satisfies it rather than evading it.
