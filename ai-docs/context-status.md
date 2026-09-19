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
