# Progress: Workspace and crate skeletons — ACTIVE
_Updated: 2026-09-18 18:12_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-09-18-workspace-crate-skeletons
**base_commit:** 5fd1a041c804f2d658cd7832b4656438cfa4b617
**Last build:** green — `cargo build --workspace --all-targets` at d8cacd8
**Issue:** #10
**Spec:** ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md
**current_step:** Step 8 — Group A complete (subtask 1 of 4 done); handoff into Group B pending
**last_passed_gate:** `make cover-ratchet` (check-only) at d8cacd8 — "0.00% holds against 0.00% (tolerance 0.00 pp)"
**entry_args:** 10

## Next action

**Do this immediately:** spawn `/context-reset` handoff into Group B (subtasks 2-4) per the design's `## Handoff plan`.

## Subtasks

- [x] 1. The workspace manifest and its skeleton members (Group A, code, code-writer/sonnet) — commit d8cacd8
- [ ] 2. Retire the empty-workspace guard and every live statement of it  ← CURRENT (Group B)
- [ ] 3. The cargo ecosystem returns to Dependabot (Group B)
- [ ] 4. Record the crate-naming rule (D1) as a project key decision (Group B)

## Decisions log

Append-only, one line per non-trivial decision. Each line is prefixed with the step or phase that made it. Never edit or remove prior entries.

- **Step 7**: design-review reached GO at round 3 of the cap of 3; rounds 1 and 2 were ITERATE, and the cap is now fully spent.
- **Step 7**: the owner routed the round-1 `SPEC-REMIT` against AC4 to option (1), so the spec was amended and the Step 6 to Step 7 cycle re-run on the amended pair.
- **Step 8**: `.gitignore` carries no entry for `ai-docs/plans/.task-inflight`, for `*.progress.md` or for `ai-docs/plans/ignored/`, so this file is staged with a plain `git add` and the in-flight marker stays untracked; cleanliness between group handoffs is read with `git diff --quiet && git diff --cached --quiet`, which is blind to untracked paths, rather than with an empty `git status --porcelain`.
- **Step 8 (subtask 1, code-writer)**: crate-level `//!` comments were written as plain remit prose with no crate-qualified symbol, no `core::`-shaped path and no repository/markdown/URL reference, so the own-crate-exemption asymmetry recorded in the design's § *What the gates will read afterwards* is not exercised by this commit; `check-comment-refs.sh` was run by explicit path before `git add` and again `--staged` after, both green, matching § Risks' mitigation for the whole-tree-lexical-gate risk.
- **Step 8 (subtask 1, code-writer)**: `cargo metadata --format-version 1 --no-deps` was read back after the build and the two binary targets are named `reader-cli` and `reader-migrate`, matching the spec-fixed executable names (AC2), with no `[[bin]]` block in either manifest (D3).
- **Step 8 (subtask 1, code-writer)**: the pre-commit hook found no `ai-docs/coverage-ratchet.txt` and initialised it at `0.00` in the same commit (d8cacd8); the subsequent `make cover-ratchet` check-only run reported a comparison ("0.00% holds against 0.00%"), not a missing-file block and not a no-executable-lines skip, confirming the two binary `main` bodies put lines into the summary per D8.

## GO notes

| # | round | note | kind | route | resolution |
|---|-------|------|------|-------|------------|
| G1 | 1 | Spec AC4's trailing clause restates a standing rule | spec-amending (c) | owner (1) spec amended | answer 2.3; closed by design-review round 3 GO |
| G2 | 3 | Decomposition row 4 fixes the key-decisions row's source field as "the owner's round-2 answer, not this design" | design-internal | folded | design subtask 4 + Test Design subtask 4 @ 5fd1a04 |
| G3 | 3 | Scenario 1 is titled "The configuration is well formed" but the only check it names is a read of GitHub's options reference | design-internal | folded | design Test Design subtask 3 scenarios 1-2 @ 5fd1a04 |
| G4 | 3 | Subtask 2's file set carries the two Python gate scripts but its own stay/go rule appears to exempt them | design-internal | folded | design subtask 2 (c) and (d); both files dropped from the file set @ 5fd1a04 |
| G5 | 3 | D9's clause "those scripts are driven over scratch trees by their own suites" carries no claim tag | design-internal | folded | design D9 tag @ 5fd1a04 |
| G6 | 3 | The ratchet-coordinate tag quotes a clause that begins at line 21 | design-internal | folded | refused and re-resolved: the clause begins at 22, the tag was right; the tag is now self-locating via `grep -n` @ 5fd1a04 |
| G7 | 3 | Record the independently corroborated AC4 enumeration in the design's Risks | design-internal | folded | design Risks, fourth vocabulary run independently @ 5fd1a04 |
| G8 | 3 | AC3's red control compares a pattern against the source while the live check reads the run's stderr | design-internal | folded | design Test Design, capture-vs-capture control @ 5fd1a04 |
| G9 | 3 | Subtask 1's gate runs must go by explicit path before `git add` and again through the staged set | design-internal | folded | already binding in design Risks and Test Design subtask 1; no edit needed |

## Key discoveries (don't re-investigate)

- The AC4 live-site set is settled: four vocabularies over three rounds returned one set, corroborated independently by design-review. Subtask 2 re-runs the widened patterns to confirm its own edits landed, not to search for more sites.
- A member packaged as `core` shadows the standard library's `core` in every dependent (`error[E0433]`), which is why D1 prefixes the package names; the directories keep the corpus's short names by the owner's decision.
- `own_crate()` keys on the directory name, so a package-qualified symbol is not exempt inside its own crate, and `core::` is unwritable in any gated comment outside `crates/core`.
- The ratchet initialises at `0.00`: the binaries' `main` lines are counted but never executed, so the summary carries lines and the "no executable lines" skip is not taken.
- `.gitignore` carries no `ai-docs/plans/**` entry of any kind — neither the progress file, nor the in-flight marker, nor `ignored/`.

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

- `Cargo.toml` (new — virtual workspace manifest)
- `Cargo.lock` (new — generated by the build, committed per D11)
- `crates/shared/Cargo.toml`, `crates/shared/src/lib.rs` (new)
- `crates/core/Cargo.toml`, `crates/core/src/lib.rs` (new)
- `crates/cli/Cargo.toml`, `crates/cli/src/main.rs` (new)
- `crates/migrate/Cargo.toml`, `crates/migrate/src/main.rs` (new)
- `ai-docs/coverage-ratchet.txt` (new — written and staged by the pre-commit hook)
