# Progress: Workspace and crate skeletons — ACTIVE
_Updated: 2026-09-18 20:30_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-09-18-workspace-crate-skeletons
**base_commit:** 5fd1a041c804f2d658cd7832b4656438cfa4b617
**Last build:** PASS — `make verify` green at 0421983
**Issue:** #10
**Spec:** ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md
**current_step:** Step 9.5 — docs updated
**last_passed_gate:** `make verify` | 2026-09-18T18:35:13Z | 0421983
**entry_args:** 10

## Next action

**Do this immediately:** Step 8 is complete. Push the branch, then run Step 9 (verify) and Step 10 (self-review) before Step 12 opens the pull request.

## Subtasks

- [x] 1. The workspace manifest and its skeleton members (Group A, code, code-writer/sonnet) — commit d8cacd8
- [x] 2. Retire the empty-workspace guard and every live statement of it (Group B)
- [x] 3. The cargo ecosystem returns to Dependabot (Group B)
- [x] 4. Record the crate-naming rule (D1) as a project key decision (Group B)

## Decisions log

Append-only, one line per non-trivial decision. Each line is prefixed with the step or phase that made it. Never edit or remove prior entries.

- **Step 7**: design-review reached GO at round 3 of the cap of 3; rounds 1 and 2 were ITERATE, and the cap is now fully spent.
- **Step 7**: the owner routed the round-1 `SPEC-REMIT` against AC4 to option (1), so the spec was amended and the Step 6 to Step 7 cycle re-run on the amended pair.
- **Step 8**: `.gitignore` carries no entry for `ai-docs/plans/.task-inflight`, for `*.progress.md` or for `ai-docs/plans/ignored/`, so this file is staged with a plain `git add` and the in-flight marker stays untracked; cleanliness between group handoffs is read with `git diff --quiet && git diff --cached --quiet`, which is blind to untracked paths, rather than with an empty `git status --porcelain`.
- **Step 8 (subtask 1, code-writer)**: crate-level `//!` comments were written as plain remit prose with no crate-qualified symbol, no `core::`-shaped path and no repository/markdown/URL reference, so the own-crate-exemption asymmetry recorded in the design's § *What the gates will read afterwards* is not exercised by this commit; `check-comment-refs.sh` was run by explicit path before `git add` and again `--staged` after, both green, matching § Risks' mitigation for the whole-tree-lexical-gate risk.
- **Step 8 (subtask 1, code-writer)**: `cargo metadata --format-version 1 --no-deps` was read back after the build and the two binary targets are named `reader-cli` and `reader-migrate`, matching the spec-fixed executable names (AC2), with no `[[bin]]` block in either manifest (D3).
- **Step 8 (subtask 1, code-writer)**: the pre-commit hook found no `ai-docs/coverage-ratchet.txt` and initialised it at `0.00` in the same commit (d8cacd8); the subsequent `make cover-ratchet` check-only run reported a comparison ("0.00% holds against 0.00%"), not a missing-file block and not a no-executable-lines skip, confirming the two binary `main` bodies put lines into the summary per D8.
- **Step 8**: Group A returned and the branch was pushed to `origin` as the binding visibility point; the orchestrator re-validated branch, base_commit and tracked-tree cleanliness, and re-ran `cargo build --workspace --all-targets` itself before handing off Group B.
- **Step 8 (subtask 2)**: the AC3 red control was run before the Makefile was edited — `git show HEAD:Makefile` into a manifest-less scratch directory, then `make -C` that directory — and its capture carried "no Cargo.toml at the repository root; build skipped"; the post-edit `make verify` capture carried no such line, and each recipe's command was echoed in it, so the aggregate's silence is evidence about the tree and not about the instrument.
- **Step 8 (subtask 2)**: with the guard variable gone, the six cargo recipes were also stripped of their `@` prefix rather than left as `@cargo …` — "the bare gate command" of the design's subtask 2, and the form every non-cargo recipe in the file already used; the echoed command lines are what the `make verify` capture was read for.
- **Step 8 (subtask 2)**: the widened prose sweep (the union of the four vocabularies § Risks names) was run against a constructed control file first — all twenty patterns matched there — then over the tree before and after the edits; the residue was read line by line and every remaining hit is either a conditional branch/state-table row the design's stay rule keeps, `.claude/**` rule text, or subtask 3's own two sites. A Russian-vocabulary pass and a cardinality control confirming the sweep corpus reaches `docs/` and `README.md` were run as well, and the Russian pass returned no era statement.
- **Step 8 (subtask 2)**: the comment-reference gate was run by explicit path over the two gated files this subtask edits and again over the whole tracked set, both green, and its red direction was confirmed on a constructed scratch script carrying a markdown path, which the gate reported.
- **Step 8 (subtask 3)**: the YAML parser was checked in its red direction first — a scratch copy of the configuration with the ecosystem entry indented one column short raised `yaml.scanner.ScannerError` — and only then was the edited file's clean parse read as a pass.
- **Step 8 (subtask 3)**: each field of the cargo entry was read back against a source the rewrite cannot move — the cadence and the open-pull-request limit against the sibling `github-actions` entry in the parsed file itself (both equal), and the `build` commit prefix against the pre-change tool-inventory section taken from the merge base `ae76456`. The ecosystem identifier `cargo` and the four option keys were confirmed against GitHub's own options reference in this session rather than from the design's tag.
- **Step 8 (subtask 3)**: after the propagation row was removed, the three sites that pointed at it — the configuration comment, the skill preamble and the tool-inventory section, all three inside this subtask's own file set — return no hit, with the sweep's pattern matched against a constructed control line first. No gate script names either the configuration file or the propagation page: a sweep of the 47 shell, Python and workflow files under the harness directories returns nothing for either name.
- **Step 8 (subtask 4)**: KD-19 was appended under § Repository and process after KD-18, in the page's own decision/why/consequence/source shape; its consequence field carries both measured comment-gate constraints the directory/package divergence creates. The row's source is backticked prose and holds no markdown link, no live `ai-docs/plans/` path and no interview-state-file path — it names the post-retirement design path — which was asserted mechanically rather than read by eye.
- **Step 8 (subtask 4)**: the relative-link check was re-run after the edit over 84 documents and 207 relative links with none broken; the corpus size was printed before the verdict so that the clean result is about the documents and not about an enumeration that reached nothing.
- **Step 8 (Group B close)**: the widened sweep was run a third time after all three subtasks. No live document or script still states the empty-workspace era: every residual hit is a conditional printf, a state-table row, a docstring governed by its own `While`/`Once`, `.claude/**` rule text, or a rewritten sentence matched on a surviving pattern word. AC4 is discharged.
- **Step 8 (subtask 3)**: the CI paths filter needs no edit for this change — its `commentrefs` key already lists `**/*.yml`, so the configuration file reaches the comment-reference job, and the two documents are under the harness key's `.claude/**` and `ai-docs/**`.
- **Step 9**: every acceptance criterion was re-verified by the orchestrator's own command rather than from a delegate's summary; the AC4 sweep ran over 150 tracked live files with each of its twelve patterns matched against a constructed control first, and its three surviving hits are the conditional statements the design rules stay. No panic-index row was added and no domain invariant is touched — the diff's only domain word is prose inside a remit comment.
- **Step 9.5**: `ai-docs/context.md` is left unedited on purpose — its § Status defers the crate layout to the workspace manifest and states that the page fixes the shape rather than the progress, so recording this task there would contradict the page's own contract and go stale; `README.md` makes no claim about cargo, crates or the workspace, so nothing there is falsified.

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
| AC1 | PASS |
| AC2 | PASS |
| AC3 | PASS — `make verify` ran every gate and its capture carries no skip line, while the pre-change entry point's capture in a manifest-less scratch directory does |
| AC4 | PASS — the widened four-vocabulary sweep, re-run after every subtask, leaves no live document or script asserting the era; each residual hit was read and classified |
| AC5 | PASS |

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
- `Makefile` (guard block deleted, cargo recipes bared, file-size band comment rewritten)
- `AGENTS.md` (§ Build & Test era blockquote and the ratchet tolerance paragraph)
- `ai-docs/code-style.md` (§ File size — the band comment's twin)
- `ai-docs/claude-tools-hierarchy.md` (§ CI only)
- `.githooks/coverage-ratchet.sh` (tolerance header, no-executable-lines branch comment)
- `.github/dependabot.yml` (cargo entry added, absence comment dropped)
- `.claude/skills/dependabot-pr/SKILL.md` (§ preamble)
- `ai-docs/claude-tools-hierarchy.md` (§ Dependabot — a second, separate edit from subtask 2's § CI)
- `ai-docs/propagation-groups.md` (the discharged workspace/Dependabot row removed)
- `ai-docs/key-decisions.md` (KD-19 — the crate-naming rule)
