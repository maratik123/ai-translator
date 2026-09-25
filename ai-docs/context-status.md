# Context status

One entry per completed task, appended by `/task` Step 9.5 and given its pull-request locator at Step 12. The entry says what the task changed and what a later reader needs to know that the diff does not show; the plan documents themselves are retired to `ai-docs/plans/done/`.

An entry's heading carries the task's name and, until the pull request exists, the placeholder `/task` Step 9.5 writes there. Step 12 replaces it with the real number once `gh pr create` returns, and CI refuses a tree where any placeholder survives — so the shape is not spelled out on this page, which CI reads with a plain grep.

This log starts empty.

## Workspace and crate skeletons — the cargo gates now bind (#70, 2026-09-18)

The repository had no Cargo manifest at its root, so every cargo gate detected that and exited 0 with a
named skip. A green aggregate run was therefore evidence about nothing. This task laid down the virtual
workspace and the skeleton members, deleted the guard that produced those skips, and retired every live
statement that described the era.

What a later reader needs that the diff does not show:

- **The packages carry a `reader-` prefix; the directories do not.** A member packaged as `core` shadows
  the standard library's `core` in every dependent, so a third-party derive macro emitting `::core::…`
  fails to resolve. The directories keep the corpus's short names by the owner's decision, and the
  divergence has a cost recorded as a project key decision: the comment-reference gate's own-crate
  exemption keys on the **directory** name, so a package-qualified symbol is not exempt inside its own
  crate, and `core::` is unwritable in any gated comment outside the core crate's directory.
- **The executable names are not configured anywhere.** They fall out of the package names, so neither
  binary member carries a `[[bin]]` block. Renaming a package renames its executable.
- **The coverage ratchet starts at zero, and that is a measurement.** The binaries' `main` bodies put
  lines into the coverage summary, so the ratchet takes its comparison branch rather than its
  "no executable lines" skip; the lines are counted and never executed, which is why the figure is what
  it is. No placeholder test was added on purpose: one would move the workspace total off zero and
  record a high-water mark set by scaffolding, which the next task with real code would inherit.
- **The server crate is deliberately absent.** The repository-structure documents list it as a workspace
  member; the owner scoped this task to the member set the issue's own text names, and the server
  skeleton comes with its own task.
- **Dependabot's cargo ecosystem returned with the workspace**, and the propagation-table row that bound
  the two was retired as discharged. Nothing in the tree reads that configuration's values, so its
  fields were each read back against a source the change itself does not rewrite.

## Postgres test harness and the vector-extension migration — the suite provisions its own database (#76, 2026-09-19)

The milestone's first database task. It carries no schema: what it delivers is a live connection the
suite starts for itself, plus one migration small enough that a single question to the server proves the
container really carries the extension. The corpus prescribed a harness built from `testcontainers`,
ryuk, a container parked in a `OnceCell` and `#[sqlx::test]`; three of those four could not hold, the
owner authorised the substitution and the corpus row was amended in the same pull request.

What a later reader needs that the diff does not show:

- **Why the corpus row changed, and what each clause was replaced with.** `#[sqlx::test]` reads its
  connection from `DATABASE_URL` and from nothing else — two environment reads exist in its whole
  source file and both are that variable — while the standing rule forbids the suite that variable,
  which is the application's. The container crate ships no ryuk at all; what it has is a watchdog that
  reaps on a termination signal. And a value parked in a `static` is never dropped, so the corpus's own
  pairing of "one container per binary in a `OnceCell`" with "the container cleans up after itself"
  cannot both hold. The replacement is a test target that owns its own `main`: it starts the container,
  hands each trial a freshly created and migrated database, and removes the container before returning.
- **The container crate is held on the 0.27 series deliberately, and raising it is a coupled move
  rather than a version bump.** The Postgres module crate's published requirement is what decides it.
  The obvious workaround — adding the newer container crate beside the older one, which cargo normally
  permits for semver-incompatible lines — fails a level deeper: the two series pull different `bollard`
  lines, and those pin mutually exclusive `bollard-stubs` versions exactly, so the resolver refuses.
  Anyone raising the pin needs a module release whose bound admits it, not a newer container crate.
- **The first migration file carries no comment, and that is load-bearing rather than tidy.** The
  loader keeps a migration file's bytes verbatim — it detects one leading directive and strips nothing —
  so a comment line would land inside both the embedded statement text and the recorded checksum. The
  unit test pinned to that statement is exact for the same reason. If it ever goes red, the repair is
  the file and never the assertion — except for the version and description assertions, which derive
  from the file's *name*: repairing those by renaming would move an applied migration's version,
  description and checksum at once, which is the redefinition the forward-migration invariant forbids.
- **The migration's description is not its file name.** The loader derives it from the name by replacing
  underscores with spaces, so `0001_vector_extension.sql` is described `vector extension`. The unit test
  asserts the derived value, not the file name.
- **The start counter is process-wide, not a field of the harness.** "One container for the whole test
  binary" is a claim about the process, so a per-instance counter would miss a second harness built
  anywhere in the same binary. It is incremented at the site that awaits the container start rather than
  in a constructor — a counter set to its expected value and never touched passes for every possible
  program. This does not contradict the rule against parking things in a `static`: what that forbids is
  the container *handle*, whose destructor never running is the entire problem, and an atomic integer
  owns no resource.
- **The server's own start instant corroborates the count and does not replace it.** Both databases in
  that trial are taken inside one trial, so a harness that started a container per trial would still
  report identical instants. Only the count sees that failure; neither check subsumes the other.
- **A teardown failure raises the run's verdict but never replaces it.** The runner's exit code is taken
  before the shutdown runs. A failed removal goes to the error channel and turns a green run red — a
  leaked container must not exit zero — while a run the trials already failed keeps the status they
  earned. No panicking call carries the shutdown's result out of `main`, because a panic there
  substitutes its own status for the verdict at the moment the verdict matters most.
- **An unregistered trial is a denied lint, not a silent skip.** The target sets its own harness off, so
  a trial body nobody registers is dead code, and the strict lint gate fails on it. That was executed
  against the shipped target rather than inferred from the manifest key.
- **The CI reach is discharged by the run on the pull request, not by any subtask.** The Rust jobs are
  reached by the source, SQL and manifest path filters, and the runner image ships the daemon the
  container crate falls back to when no socket variable points elsewhere. Nothing in the harness names
  the developer machine's socket path.
- **Two live statements this diff falsified were retired** — the coverage tolerance's reason clause and
  the file-size bands' justification both described a workspace whose crates carried no test. Their
  constants are untouched; only the reasons moved.
- **One corpus row that this work makes stale was deliberately left alone.** The build-and-deploy page
  still describes a runner exporting the container socket variable, a mechanism this harness declines.
  Amending it was outside the single corpus amendment the owner authorised, so it is recorded here and
  belongs to its own task.

## Storage schema: the second migration — the database now refuses what the owner said it must (#77, 2026-09-22)

- **The owner settled four questions the corpus leaves open, and each is a data contract from the day it
  lands.** The schema enforces structure and judges no content: a position is taken once within its
  parent and a row lacking a value it would be meaningless without is refused, while an empty paragraph
  text is stored as given. Deleting a book is refused while anything still references it — nothing
  cascades anywhere in the schema. The categorical columns hold free text, with the supported set in the
  code. A work with no divisions of its own is stored as one untitled chapter, so a paragraph never
  belongs to a book directly and the reading path stays single.
- **A delete refused by a restrict rule raises SQLSTATE `23001`, not `23503`.** A trial written against
  the foreign-key code would be green for a schema with the wrong rule — the discriminator is the whole
  reason the references spell the rule explicitly rather than relying on the default.
- **`information_schema` loses a vector column's dimension**, reporting the type as user-defined;
  `format_type` over the catalogue is what reports it in full. The golden that pins the schema's shape
  reads the catalogue for that reason, and its type strings are the formatter's own spellings rather
  than the ones the migration file writes.
- **The golden's domain excludes the migrator's own bookkeeping table by name, never by a pattern.** A
  pattern would also swallow a wrongly-created table of a similar name, which is the failure the golden
  exists to catch; pinning that table's shape would couple the trial to the migration library's version.
- **PostgreSQL 18 records `NOT NULL` as constraint rows of the `n` kind**, so the assertion that the
  schema carries no check constraint answers clean for every possible schema if it is scoped to the
  wrong kind or the wrong namespace. It is scoped twice, and the mutation that proves it is paired with
  the unmutated schema's green beside it.
- **Postgres forces `NOT NULL` on every primary-key column regardless of the column's own
  declaration.** A red observation that drops the marker from a key column therefore mutates nothing and
  stays green; the observation has to pick a column outside every key.
- **Dropping a table's primary key removes that table from the key query's answer rather than leaving an
  empty key list**, so the assertion that every table carries the key the corpus fixes compares as a set
  in both directions. Written the easy way — every row that came back matches — it is green on precisely
  the schema it exists to catch.
- **No assertion is made about the query plan, deliberately.** The reading-order plan carries a sort over
  a bitmap scan until the table has been analysed, so an assertion about it would be red from the first
  run and repairable only by analysing a table and pinning cost-model switches — which would make it a
  test about the planner's cost model. The ordered key-column list in the catalogue already catches the
  wrong-column-order failure the plan assertion was there for.
- **The two reference-shaped columns the corpus leaves unmarked stay unmarked**, and the consequence is
  live rather than urgent: a paragraph may be deleted while a context snapshot or a reading position
  still points at it. Nothing in this milestone deletes a paragraph. Making them references later is a
  forward migration in the direction that stays open.
- **No corpus row is ticked by this work.** The storage page's task list is about the sqlx setup, the
  migrate binary, the schema check an engine performs when it opens a database, the repositories, and
  the role and database on the local Postgres — none of which this task delivers. The schema now exists
  in the repository and in every test database, and nothing shipped applies it anywhere else.
- **The coverage ratchet blocked this branch on a value it had written itself.** It compares the
  measurement at full precision and records it rounded half-up, so the recorded mark sat strictly above
  the measurement that produced it. The recorded value was lowered to the round-down in its own
  document-only commit, which keeps the hook's raise branch skipped; the script is untouched and its
  diagnosis remains an open harness-gaps entry.
