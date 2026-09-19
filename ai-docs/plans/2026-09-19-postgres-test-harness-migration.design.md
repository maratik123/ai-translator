# Design: Postgres test harness and the vector-extension migration

**Issue:** #13
**Date:** 2026-09-19

## Approach

### What the tree holds today

The workspace's skeleton members declare no dependency at all: the root manifest's
`[workspace.dependencies]` table carries a comment saying so and nothing else
`[measured 0daa273:Cargo.toml:16-18 · awk 'NR>=16 && NR<=18 {print NR": "$0}' Cargo.toml → "16: [workspace.dependencies]" / "17: # A dependency is added here only once a member's code compiles against it;" / "18: # nothing does yet, so the table is empty."]`,
and each member's `[dependencies]` section is empty
`[measured 0daa273 · for c in shared core cli migrate; do sed -n '/^\[dependencies\]/,$p' crates/$c/Cargo.toml; done → the section header alone in each]`.
`crates/core/src/lib.rs` is a single `//!` line and carries no item
`[measured 0daa273:crates/core/src/lib.rs · cat crates/core/src/lib.rs → "//! The translation engine: segmentation, retrieval, prompting and validation."]`.
There is no `migrations` directory, no test target, and no code that opens a database.

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
and the attribute itself accepts only fixtures and a migrations source — its parsed argument struct
holds `fixtures` and `migrations`, with the migrations option ranging over an inferred path, an
explicit path, an explicit migrator and disabled
`[measured sqlx-macros-core@0.9.0 · awk 'NR>=5 && NR<=26 {print NR": "$0}' sqlx-macros-core-0.9.0/src/test_attr.rs → ':6: struct Args {' with the fields fixtures and migrations, and ':20: enum MigrationsOpt {' with the variants InferredPath, ExplicitPath, ExplicitMigrator and Disabled]`.
The `sqlx.toml` setting that renames the variable is consumed by the compile-time query macros and by
`sqlx-cli`, never by the test runtime
`[measured sqlx-core@0.9.0 · grep -rn 'database_url_var' sqlx-core-0.9.0/src sqlx-macros-core-0.9.0/src → the declaration and accessor in sqlx-core-0.9.0/src/config/common.rs, whose accessor defaults to "DATABASE_URL", and two call sites, both in sqlx-macros-core-0.9.0/src/query/metadata.rs]`,
and the upstream change that would add a `var` argument to the attribute is open and unmerged
`[measured launchbadge/sqlx#4050 · gh pr view 4050 --repo launchbadge/sqlx --json state,mergedAt → {"mergedAt":null,"state":"OPEN"}]`.
So `#[sqlx::test]` against a container means putting the container's DSN into the process's own
`DATABASE_URL` before the first test connects — which `AGENTS.md` § *Build & Test* forbids in the
plainest terms it uses anywhere, and which the owner reaffirmed by striking the spec rows that merely
restated it (`prior_qa` round 2: «три критерия повторяют AGENTS.md § Build & Test — убрать их»).

**testcontainers-rs carries no Ryuk.** A case-insensitive sweep of the whole published crate for
either name comes back empty, and the same pattern matches a constructed control line
`[measured testcontainers@0.28.0 · grep -rni 'ryuk\|reaper' testcontainers-0.28.0/ → no output; the control printf 'RYUK_CONTAINER_IMAGE\nryuk\n' | grep -ni 'ryuk\|reaper' → both lines matched, so the pattern ran]`.
What it has instead is an optional `watchdog` feature that stops and removes registered containers on
`SIGTERM`, `SIGINT` or `SIGQUIT`
`[measured testcontainers@0.28.0 · awk 'NR>=1 && NR<=4 {print NR": "$0}' testcontainers-0.28.0/src/watchdog.rs → ':1: //! Watchdog that stops and removes containers on SIGTERM, SIGINT, or SIGQUIT' and ':3: //! By default, the watchdog is disabled. To enable it, enable the "watchdog" feature.']`,
which covers an interrupted run and says nothing about a normal exit. The only removal path on a
normal exit is the container handle's own destructor, or the explicit `rm`
`[measured testcontainers@0.28.0 · grep -n 'client.rm(&id)\|env::Command::Remove\|pub async fn rm' testcontainers-0.28.0/src/core/containers/async_container.rs → ':205: pub async fn rm(mut self) -> Result<()> {', ':276: env::Command::Remove => {' and ':277: if let Err(e) = client.rm(&id).await {', the last two inside the Drop impl that opens at ':244']`.

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
migrator against it. The vehicle is the only thing that changes, and because the row that names the
vehicle lives in the DECISIONS corpus, § Open questions asks the owner to confirm before `docs/03` is
amended.

**D2 — The database-backed target owns its `main`: `harness = false` with `libtest-mimic`.** The
argument is § *The shape this design chooses* in full. The crate is established and current
`[measured libtest-mimic@0.8.2 · cargo info libtest-mimic → "version: 0.8.2", "repository: https://github.com/LukasKalbertodt/libtest-mimic"]`,
and it is a dev-dependency, so nothing it brings reaches a shipped binary. The unit tests of `src/`
are untouched by this: they keep the default harness, and only the database-backed integration target
declares its own.

**D3 — The container is removed by an explicit call inside the async runtime, never by letting the
handle drop outside one.** The destructor's helper resolves the current runtime handle before doing
anything else
`[measured testcontainers@0.28.0 · grep -n 'Handle::current\|pub(crate) fn async_drop' testcontainers-0.28.0/src/core/async_drop.rs → ':16: pub(crate) fn async_drop(future: …)' and ':17: let handle = tokio::runtime::Handle::current();']`,
and `main` is not inside a runtime, so a handle dropped there would resolve a handle that does not
exist. The harness therefore exposes a shutdown that `main` drives through the runtime it owns,
calling the crate's own consuming removal
`[measured testcontainers@0.28.0 · grep -n 'pub async fn rm' testcontainers-0.28.0/src/core/containers/async_container.rs → ':205: pub async fn rm(mut self) -> Result<()> {']`.
Its result is reported, not discarded: a container that could not be removed is a message on the way
out, because `AGENTS.md` § *Code Style* forbids discarding a `Result`.

**D4 — The image is the floating tag the task names, reached by overriding the module image's name and
tag.** The spec's own key-decision row fixes the floating tag («Как есть»), and the tag resolves today
`[measured hub.docker.com · curl -s https://hub.docker.com/v2/repositories/pgvector/pgvector/tags/pg18 → {"name":"pg18","tag_status":"active","last_updated":"2026-08-13T21:29:05.149249Z"}]`.
The Postgres module of `testcontainers-modules` hard-codes a different image, so its name and tag are
overridden rather than its environment or its readiness conditions
`[measured testcontainers-modules@0.15.0 · awk 'NR>=5 && NR<=6 {print NR": "$0}' testcontainers-modules-0.15.0/src/postgres/mod.rs → ':5: const NAME: &str = "postgres";' / ':6: const TAG: &str = "11-alpine";']`,
using the builder the container crate provides for exactly that
`[measured testcontainers@0.28.0 · grep -n 'fn with_name\|fn with_tag' testcontainers-0.28.0/src/core/image/image_ext.rs → the trait declarations at ':60: fn with_name(self, name: impl Into<String>) -> ContainerRequest<I>;' and ':66: fn with_tag(...)', with their impls at :312 and :320]`.
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

**D6 — The first migration is a sequential `0001`, and its file is immutable once committed.**
Sequential four-digit versions make AC2's second half — that no migration precedes it — readable by
eye rather than by arithmetic on timestamps, and every later migration continues the series. The file
is written once and never edited: the migrator records a checksum per applied version
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
crate resolves its host from `DOCKER_HOST` before any fallback, and falls back to the platform default
socket when the variable is unset
`[measured testcontainers@0.28.0 · awk 'NR>=44 && NR<=54 {print NR": "$0}' testcontainers-0.28.0/src/lib.rs → ':44: ##### The host is resolved in the following order:' over a list whose second entry is ':47: 2. "DOCKER_HOST" environment variable.' and whose fourth is ':49: 4. Read the default Docker socket path'; and grep -n 'pub const DEFAULT_DOCKER_HOST' testcontainers-0.28.0/src/core/env/config.rs → ':34: pub const DEFAULT_DOCKER_HOST: &str = "unix:///var/run/docker.sock";']`.
The developer machine exports the variable already
`[measured podman@5.8.2 · printf '%s\n' "$DOCKER_HOST" in a shell initialised from the user's profile → unix:///run/user/1000/podman/podman.sock, and ls -la /run/user/1000/podman/ → a socket named podman.sock]`,
and the CI runner image ships a daemon on the fallback path
`[measured https://raw.githubusercontent.com/actions/runner-images/main/images/ubuntu/Ubuntu2404-Readme.md · WebFetch → the installed-software listing carrying "Docker Client 28.0.4", "Docker Server 28.0.4" and "Docker Compose 2.38.2"]`,
so both ends of AC5 are served without a recipe that hard-codes either. A recipe that exported the
developer machine's path would be wrong in CI, and one that guessed would hide the very failure
`AGENTS.md` § *Build & Test* wants loud. A machine with neither is told so by the failing test, which
is the stated behaviour rather than a regrettable one.

**D11 — AC4 is asserted from the server's side, not only from the harness's own bookkeeping.** A
counter the harness increments would test the harness. Two databases handed out separately are on the
same server if and only if the server reports the same postmaster start instant, which is the
database's own evidence and not ours; the harness's own start counter is asserted beside it as the
cheap cross-check, and the two disagreeing is itself informative. § Test Design specifies both.

**D12 — The statements this diff falsifies are corrected in the same pull request.** They fall into a
definitive class and a judgement class, separated here so the judgement is visible rather than
smuggled. Definitive: the root manifest's comment that the dependency table is empty because nothing
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

### What the gates will read afterwards

The comment-reference ban reaches SQL by file name, so the migration's own comments are gated exactly
as a Rust comment is
`[measured 0daa273:ai-docs/scripts/comment_refs.py:42 · sed -n '42p' ai-docs/scripts/comment_refs.py → 'GATED_SOURCE_EXTS = (".rs", ".sh", ".sql", ".yml", ".yaml")']`.
Nothing this task writes — in the migration, in `crates/core/src/lib.rs`, in the support module or in
the test target — may carry a markdown path, an acceptance-criterion id, a decision anchor, an issue
number outside `TODO(#…)`, a repository path or a URL. Two consequences are specific enough to be
worth naming: the migration file states *why* the extension is created, if it states anything, without
pointing at the page that decided it; and KD-19's ruling binds every doc comment this task writes —
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

The coverage ratchet acquires a new precondition that is worth stating out loud rather than
discovering: it refuses a commit when the suite is not green, and from this task on the suite needs a
container runtime, so **every commit that stages a `.rs`, a `.sql` or a manifest now needs a reachable
socket**
`[measured 0daa273:.githooks/coverage-ratchet.sh:92-96,119-126 · awk 'NR>=92 && NR<=96 {print NR": "$0}' and awk 'NR>=116 && NR<=127 {print NR": "$0}' .githooks/coverage-ratchet.sh → ':95: staged=$(git diff --cached --name-only --diff-filter=ACMR \' over '*.rs', '*.sql' and the manifests, and ':119: if ! cargo llvm-cov …; then' whose branch prints 'BLOCKED — the test suite is not green' and ':123: If no container runtime is reachable, start the Podman socket the suite connects to']`.
A commit that stages only documents is unaffected, which is most of a run.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | **The dependency set, the first migration, and the embedded migrator.** Add the workspace dependency entries with `cargo add` (never a hand-edited lockfile) and rewrite the root manifest's now-false comment about the empty table (D12). Declare `sqlx` as a normal dependency of `reader-core` with default features off and the set D7 fixes, and the dev-dependencies the next subtask needs — the container crate, its Postgres module, the runner and the async runtime — so the manifest is written once. Add `crates/core/migrations/0001_vector_extension.sql` carrying the single `CREATE EXTENSION IF NOT EXISTS vector` statement (D6), and expose the embedded migrator from `crates/core/src/lib.rs` with a doc comment that obeys the reference ban and KD-19 (§ *What the gates will read afterwards*). Add the `#[cfg(test)]` module beside it asserting the embedded set against AC2 — the lowest-versioned migration is version 1, described `vector_extension`, no migration carries a lower version, and its statement is exactly the one above. That test needs no container and is the half of AC2 that a machine without a runtime can still check. | `Cargo.toml`, `Cargo.lock`, `crates/core/Cargo.toml`, `crates/core/migrations/0001_vector_extension.sql`, `crates/core/src/lib.rs` | — |
| 2 | **The container harness and the database-backed test target.** Declare the target with its own `main` in `crates/core/Cargo.toml` (`harness = false`). Write the support module under `crates/core/tests/support/`: start one container from the overridden image (D4), build one admin pool over connect options assembled field by field with TLS disabled and no password (D5), hand out a freshly created and migrated database per caller, and expose the shutdown `main` drives through its runtime, whose result is reported rather than discarded (D3). Write the test target: `main` builds the runtime, starts the harness, registers the trials § Test Design names, runs them, shuts the harness down, and propagates the runner's verdict as the process's exit status. | `crates/core/Cargo.toml`, `crates/core/tests/support/mod.rs`, `crates/core/tests/database.rs` | 1 |
| 3 | **Correct the statements this diff falsifies (D12).** Rewrite the tolerance paragraph's reason clause and its twin in the ratchet script's header so both say what is now true — a crate carries a test, and the tolerance still has no drift series behind it — leaving the tolerance value and the script's conditional branches untouched. Rewrite the file-size bands' justification in the build entry point's comment and in its twin in the code-style reference to the same judgement: the crates carry their first code, and the bands still wait for a real distribution, so they do not move. Leave the condition-governed gate-script sentences § D12 enumerates alone; they are absent from this file set on purpose, because an untouched listed file reads as a missed site. The build entry point is in the comment-reference gated set, so a rewritten comment there obeys the same ban as a Rust one. | `AGENTS.md`, `.githooks/coverage-ratchet.sh`, `Makefile`, `ai-docs/code-style.md` | 2 |
| 4 | **Record the harness decision where it will be looked for.** Add a key-decision row, in the page's own shape — decision, why, consequence, source — numbered after the last row the page carries, stating that the database-backed target owns its `main` so that one container serves the binary *and* is removed when the run ends, and that `#[sqlx::test]` is not the vehicle because its only connection source is the variable the suite is forbidden to read. The *consequence* field carries what a later test author inherits: a trial is registered in `main`, an unregistered one is a denied lint rather than a silent pass, and the lift threshold D9 fixes. The *source* field is backticked prose, not a markdown link, and names this design at the path it carries after Step 12 — `ai-docs/plans/done/2026-09-19-postgres-test-harness-migration.design.md` § D1–D3 — because the pre-retirement path is stale before the pull request opens. | `ai-docs/key-decisions.md` | 2 |

## Handoff plan

`M = 4`. Two groups, homogeneous by change-type and minimised: the two code subtasks are consecutive
and both harness subtasks depend on them and on neither each other, so no dependency chain forces an
interleave. Two groups is within the default maximum of four, so no user approval is needed.

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
  `model=` — subtasks 3, 4 (instructions/harness change-type: `AGENTS.md`, `Makefile`, `.githooks/**`,
  `ai-docs/**`). Terminal group (2 subtasks; within the `1..=10` range).

Subtask 3 edits `AGENTS.md` and the ratchet script as a pair and the build entry point and the
code-style reference as a second pair; subtask 4 touches neither file set. They are sequential commits
inside one group, so the second reads the first's result.

## Risks

- **The migration's checksum is immutable the moment it is applied anywhere, so a "small fix" to the
  file after this pull request merges is a data-corruption bug wearing a refactor's clothes.**
  Mitigation: subtask 1's unit test pins the statement's exact text, so an edit to the file fails the
  suite instead of passing quietly, and D6 states the rule in the design where a later author will
  look — `[measured sqlx-core@0.9.0 · grep -n 'pub [a-z_]*:' sqlx-core-0.9.0/src/migrate/migration.rs → the Migration and AppliedMigration checksum fields]`, `[derived → the § Test Design case "the embedded migration is exactly the vector-extension statement"]`.
- **A green harness is a claim about the harness until it has been seen red.** A container that never
  started, an image without the extension and a test that asserts something true of every Postgres all
  look identical from a passing run. Mitigation: § Test Design makes three red observations a required
  part of subtask 2 rather than an optional courtesy, each recorded in the progress file with the
  output that was seen — `[derived → the § Test Design section "Red observations required before the group's last commit"]`.
- **Docker Hub meters anonymous pulls per address, and CI runners share addresses.** The documented
  allowance is 100 pulls per six hours for an unauthenticated puller, counted per IPv4 address or IPv6
  /64 subnet
  `[measured https://docs.docker.com/docker-hub/usage/ · WebFetch → "100 per IPv4 address or IPv6 /64 subnet" within a six-hour window for unauthenticated users, against 200 per six hours for an authenticated free account]`,
  so AC5 can go red for a reason that has nothing to do with this repository. Mitigation: nothing is
  built for it now — the escape hatches are a registry login step or a mirrored image, and both are
  scope this task has not been given. If CI reports a pull limit, that is the trigger to ask, and
  § Open questions carries the question so the answer is not invented under time pressure.
- **From this task on, a commit that stages Rust, SQL or a manifest needs a reachable container
  socket**, because the ratchet refuses a commit whose suite is not green and the suite now provisions
  a database — `[measured 0daa273:.githooks/coverage-ratchet.sh:92-96,119-126 · the two awk reads cited in § What the gates will read afterwards → the staged-set skip over '*.rs', '*.sql' and the manifests, and the BLOCKED branch whose message names the Podman socket]`.
  Mitigation: the ratchet already prints the socket instruction on that branch, so the failure is
  self-explaining; subtask 3 leaves that branch's comment alone for the reason D12 gives.
- **The container crate's destructor resolves a runtime handle, so a handle dropped on a path that
  bypasses the explicit shutdown panics instead of cleaning up** — `[measured testcontainers@0.28.0 · grep -n 'Handle::current' testcontainers-0.28.0/src/core/async_drop.rs → ':17: let handle = tokio::runtime::Handle::current();']`.
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
  `1`, the description `vector_extension`, and a statement whose trimmed text is exactly
  `CREATE EXTENSION IF NOT EXISTS vector`; and no migration in the set carries a version below it. The
  assertion is on the exact text, not on a substring: a substring assertion passes for a file that
  grew a second statement, which is the half of AC2 that matters
  `[derived → AC2]`.
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
- **`one_container_serves_the_whole_binary`** — the two databases of the previous case report the
  identical postmaster start instant, which is the server's own evidence that they are one server
  (D11); the harness's own container-start count is asserted to be one beside it, as the cheap
  cross-check whose disagreement with the first half would itself be the finding `[derived → AC4]`.

### Red observations required before the group's last commit

Each of these is performed, its output pasted into the progress file, and the mutation reverted. A
green suite that has never been seen red is a claim about the suite (`AGENTS.md` § Patterns 2).

- **The extension check really checks the extension.** Point the harness at the plain `postgres` image
  of the same major version and confirm `vector_type_is_usable` fails with the server's own
  unknown-type error. This is the control that separates "the image carries pgvector" from "the query
  ran" `[derived → AC1]`.
- **A machine with no runtime is told loudly.** Run the suite with the socket variable pointing at a
  path that does not exist and confirm the target fails with a connection error rather than skipping,
  reporting zero tests, or passing. Confirm the mutated value is what the process actually saw before
  reading the verdict `[derived → the standing rule in "AGENTS.md" § Build & Test]`.
- **The isolation trial really tests isolation.** Make the harness hand out the same database name
  twice and confirm `each_trial_gets_its_own_database` fails. A fixture that never reaches the clause
  makes a green assertion meaningless `[derived → AC3]`.
- **The container is removed after a failing trial.** With one trial forced to fail, confirm the
  container is gone from the runtime's container list after the process exits, and that the process
  exit status is non-zero. The failing path is where an early return would hide the teardown
  `[derived → the standing cleanup rule in "AGENTS.md" § Build & Test]`.

### Gates

`make verify` is run before each of the group's commits, and `make cover-ratchet` is what the
pre-commit hook runs on the coverage-moving ones. Both now need a reachable socket, per § Risks.

## Open questions

- **`docs/03-storage.md` § Задачи names mechanisms this design does not use, and that page is
  DECISIONS.** The measurements are in § Approach: `#[sqlx::test]` reads a hard-coded `DATABASE_URL`
  and offers no override, the container crate ships no ryuk, and a container held in a `OnceCell`
  static is never dropped, so the row's own pairing of "one container per binary" with cleanup cannot
  both hold. This design keeps every *property* the row decides — a database per test, migrations
  applied by the suite, one container per binary, the container removed — and changes only the
  vehicles. **Two questions, and the second is the one that binds:** (a) does the owner accept the
  substitute, and (b) may `docs/03-storage.md`'s test row be amended in this pull request to describe
  the mechanism that exists? Until (b) is answered the corpus row stands as written and this design's
  § D1–D3 are where the divergence is recorded. If the owner would rather keep the corpus's literal
  shape, the alternative is the `OnceCell` static with an accepted leak — the cost is a container that
  outlives the run, removed only by the next run or by a signal, and the standing cleanup rule in
  `AGENTS.md` § *Build & Test*, in `ai-docs/rust-test-conventions.md` and in KD-16 would have to be
  relaxed in the same breath.
- **Does the owner want the `docs/` checkboxes this task discharges ticked?** `docs/03-storage.md`
  § Задачи's test row and `docs/09-build-and-deploy.md` § Задачи's Podman-socket row are each partly
  satisfied by this pull request. Editing `docs/` is the owner's call, so nothing is ticked without
  one.
- **If CI goes red on a Docker Hub pull limit, which escape hatch does the owner want?** A registry
  login step needs a token in the repository's secrets; a mirrored image needs a push to a registry
  the project controls. Neither is built now (§ Risks), and neither should be chosen the first time it
  is needed at speed.
- **No spec row was found to prescribe a mechanism this design would otherwise have chosen
  differently, so no `SPEC-REMIT` tag is raised.** Scope rows 1–4 and AC2 each name a mechanism —
  testcontainers, the Podman socket, the image tag, one container per binary, `#[sqlx::test]`, a single
  `CREATE EXTENSION` statement — and each is the owner's own wording in the issue body as the interview
  state file persists it, anchored as such in the spec, rather than a spec-side choice of how. The
  criteria themselves are stated as outcomes: AC3 says a database of its own with migrations applied,
  and names no vehicle, which is why the substitute in D1 satisfies it rather than evading it.
