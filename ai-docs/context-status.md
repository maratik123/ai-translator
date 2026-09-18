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
