# Progress: Workspace and crate skeletons — ACTIVE
_Updated: 2026-09-18 20:30_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-09-18-workspace-crate-skeletons
**base_commit:** 5fd1a041c804f2d658cd7832b4656438cfa4b617
**Last build:** PASS — `make verify` green at 0421983
**Issue:** #10
**Spec:** ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md
**current_step:** Step 10 — self-review APPROVE (Round 1)
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
- **Step 10**: self-review returned APPROVE on round 1 of the cap of 3; no blocker or major row was opened, and the ten minor observations were each ruled not-a-defect and recorded as `accepted@1` register rows rather than as findings to fix.

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
| R1-1 | round 1 | minor | accepted@1 — design-sanctioned remit prose: § Risks fixes each crate comment as "states its own remit and stops there", and a skeleton with no items can carry no other kind of `//!`; DOC-5 is a convention id, not an AC/D/gate id, so the row is below the severity floor | `sed -n '1p' crates/shared/src/lib.rs` |
| R1-2 | round 1 | minor | accepted@1 — same class as R1-1: "the only binary that does" is true of the corpus's design (the repository-structure block fixes the migration crate as the one place migrations are applied) though not yet of the code | `sed -n '1p' crates/migrate/src/main.rs` |
| R1-3 | round 1 | nit | accepted@1 — DOC-2's "and what it deliberately does not" half is unstated in all four crate comments; the design scoped them to remit-only to keep them clear of the reference ban, and the boundary a skeleton would state is the next task's | `head -1 crates/*/src/*.rs` |
| R1-4 | round 1 | minor | accepted@1 — "Every cargo gate runs for real" overreaches past its own second clause: `make cover-ratchet` and `make import-guard` delegate to scripts that still carry named skips. The sentence scopes itself to the *recipes* (verified: all six cargo recipes are bare), and the ratchet state table 25 lines below enumerates those skips, so no reader is misled | `grep -n 'ratchet skipped' .githooks/coverage-ratchet.sh` |
| R1-5 | round 1 | nit | accepted@1 — bookkeeping only: `## Files touched` omits `ai-docs/context-status.md`, `ai-docs/harness-gaps.md`, `ai-docs/learnings.md` and this progress file, all of which are inside the review window | `git diff --name-only 5fd1a04..HEAD` |
| R1-6 | round 1 | nit | accepted@1 — the `pkill -f` learnings entry carries an exit status and no `**at:**`; the template requires the field for a count, a ratio, a byte size or a percentage, and an exit status is none of those | `grep -n -A5 'pkill -f matched' ai-docs/learnings.md` |
| R1-7 | round 1 | minor | accepted@1 — `#TBD-at-Step-12` would fail CI's context-status locator step, but CI triggers only on `pull_request` and on push to `master`, and `/task` Step 12 replaces the placeholder once `gh pr create` returns; the page documents that lifecycle in its own preamble | `grep -n 'TBD-at-Step-12' ai-docs/context-status.md` |
| R1-8 | round 1 | nit | accepted@1 — KD-19's `*Source:*` names the post-Step-12 path, which does not resolve today by design (subtask 4); the field is backticked prose, so the relative-link gate never reads it — re-measured clean over 98 documents and 262 links | `grep -n 'Source:' ai-docs/key-decisions.md` |
| R1-9 | round 1 | nit | accepted@1 — the build-and-deploy workspace row stays unticked although four of its five crates now exist; the design rules that ticking it would assert the server member and the frontend workspace, which this task does not deliver | `sed -n '4p' docs/09-build-and-deploy.md` |
| R1-10 | round 1 | nit | accepted@1 — the `[workspace.dependencies]` comment asserts a present state ("nothing does yet"); it is true today, it is not an AC4 site (AC4 governs gate-skipping), the file is outside the comment-reference gated set, and the table is edited exactly when the sentence would go stale | `sed -n '16,18p' Cargo.toml` |

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

## Self-Review (Round 1)

**Verdict:** APPROVE

No `blocker` or `major` row is open. Ten `minor`/`nit` observations were examined and ruled
not-a-defect; each is a register row below rather than a table row, per the severity floor.

**What was checked — every claim below is this reviewer's own command, not a delegate's summary.**

*Acceptance criteria, re-derived independently:*

- **AC1** PASS — `cargo metadata --format-version 1 --no-deps`: four workspace members
  (`reader-shared`, `reader-core`, `reader-cli`, `reader-migrate`), all edition 2024,
  licence `Apache-2.0`, `publish = []`. `make lock-check` (`cargo metadata --locked`) green, so the
  manifest and the committed lockfile agree (D11). `[workspace.dependencies]` present and empty (D5);
  no inter-member dependency edge (D6); no `rust-toolchain.toml`, no `[workspace.lints]`, no
  `rust-version` (D7). No `resolver` warning anywhere in the aggregate capture (D2).
- **AC2** PASS — the same metadata read: the only `bin` targets are `reader-cli` and `reader-migrate`,
  each named by its package with no `[[bin]]` block in either member manifest (D3).
- **AC3** PASS — `make verify` green; its capture carries no hit for
  `no Cargo.toml|skipped|skips itself|workspace is empty|empty workspace`, and twelve recipe command
  lines are echoed in it, so the silence is about the tree and not about an aggregate that printed
  nothing. **The instrument was seen RED first, as a capture and not as a source:**
  `git show 5fd1a04:Makefile` into a manifest-less scratch directory, `make -C` that directory,
  and the identical pattern matched `make: no Cargo.toml at the repository root; build skipped`
  in that capture. `grep -n 'CARGO_GUARD\|if \[ ! -f Cargo.toml' Makefile` returns nothing.
- **AC4** PASS — the widened four-vocabulary sweep (the guard/era wording, the file-size bands', the
  ratchet's and round 3's skip wording — twenty patterns), each pattern matched against a constructed
  control line first and all twenty seen to match, then run over the 152 tracked live files (the
  tracked set minus the plan documents). Twelve patterns returned hits; every hit was read and
  classified, and each is one the design's stay rule keeps: a conditional printf inside a gate script,
  a docstring governed by its own `While`/`Once`, a state-table row describing a branch, `.claude/**`
  rule text, or a lexical coincidence. A separate Russian-vocabulary pass over the same corpus
  (nine patterns, all controlled) returned no era statement, and `README.md` makes no cargo, crate or
  workspace claim at all.
- **AC5** PASS — `make cover-ratchet` prints `coverage-ratchet: 0.00% holds against 0.00%
  (tolerance 0.00 pp)`: a comparison, not a missing-file block and not a no-executable-lines skip.
  `ai-docs/coverage-ratchet.txt` is part of commit `d8cacd8`, the same commit that created the
  workspace. The figure was re-derived directly —
  `cargo llvm-cov --workspace --summary-only --json` reports `lines.count = 2`, `covered = 0`, with
  the only two file entries `crates/cli/src/main.rs` and `crates/migrate/src/main.rs`, which confirms
  D8's measurement that the binaries' `main` bodies keep the summary from reporting a workspace with
  no executable lines.

*Design conformance:*

- Every file of all four decomposition rows is present and changed; `ai-docs/scripts/import_guard.py`
  and `ai-docs/scripts/comment_refs.py` are correctly **absent** from the diff, per subtask 2 (c) and (d).
- `AGENTS.md` line 168 — the no-executable-lines state-table row the design rules unedited — is
  untouched; the two `AGENTS.md` hunks are the § Build & Test blockquote and the tolerance paragraph only.
- All three twin pairs read side by side and agreeing: the Makefile band comment and
  `ai-docs/code-style.md` § File size are word-for-word the same judgement, with 1200/1500 unmoved in
  both; the ratchet header and the `AGENTS.md` tolerance paragraph give the same reason, with
  `TOLERANCE_PP=0.00` matching the documented 0.00 pp; the rewritten no-executable-lines comment still
  says what the unedited state-table row says.
- **GO notes round-trip:** all nine rows carry a route. G2–G8 are `folded` and resolved `@ 5fd1a04`;
  `git show --stat 5fd1a04` confirms that commit is the design edit itself, so every folded note
  landed in the design *before* the implementation window opened. G9 needed no edit; G1 is the
  owner-routed spec amendment. `git log 5fd1a04..HEAD -- 'ai-docs/plans/*.design.md' '*.spec.md'` is
  empty, so neither document drifted during implementation and no Design/Spec Amendment is triggered.
- **D13's fields re-read against sources the change cannot move:** the cargo entry's cadence
  (`weekly`) and `open-pull-requests-limit` (5) match the sibling `github-actions` entry in the parsed
  file itself; its `commit-message.prefix` (`build`) matches the **pre-change** tool-inventory section
  at `5fd1a04`. The file parses (`yaml.safe_load`) into exactly two update entries. D13's
  "no gate reads this file" claim re-measured: `grep -rn -i dependabot Makefile ai-docs/scripts/
  .githooks/ .github/workflows/` returns nothing, with a constructed control line matched.
- The discharged propagation row is gone and nothing points at it: the `propagation table` sweep over
  the live corpus returns nothing, its pattern having matched a constructed control first.

*Gates, all run against the shipped tree:*

`make verify` GREEN · `make doc-check` GREEN (rustdoc emitted four crates) · `cargo clippy --workspace
--all-targets -- -D warnings` GREEN · `make panic-calls` GREEN · `make comment-refs` GREEN ·
`make file-limits` GREEN · `make lock-check` GREEN · `make import-guard` GREEN (`2 binary target(s)`) ·
`make cover-ratchet` GREEN · `shellcheck .githooks/coverage-ratchet.sh` GREEN · all twenty guard
regression suites GREEN · `check-ac-shape` / `check-spec-shape` / `check-spec-anchors` /
`check-script-shape` / `check-harness-gaps-forge` / `check-harness-gaps-targets` /
`check-review-register` / `check-ruleset-checks` / `check-citations` all GREEN · the relative-link
check GREEN over 98 documents and 262 links.

*Instrument checks — a green gate is a claim about the gate until it has been seen to go red:*

- The comment-reference gate enumerates 158 files including all eight `crates/**` paths; the panic
  gate enumerates exactly the four new `.rs` files. Neither clean result comes from an enumeration
  that reached nothing.
- The comment gate was driven RED on a constructed `.rs` carrying a markdown path
  (`markdown-path: ai-docs/doc-convention.md`, exit 1) and GREEN by explicit path over the four new
  sources.
- **KD-19's standing constraints were executed, not read.** `comment_refs.workspace_crates('.')`
  returns `{cli, core, migrate, shared, reader_cli, reader_core, reader_migrate, reader_shared}`, and
  `classify` was run in both directions: `core::Engine` inside `crates/core` is exempt while
  `reader_core::Engine` there is reported; `core::fmt::Debug` is reported in `crates/shared`, in a
  gated shell script and in a workflow, and exempt only inside `crates/core`. Every sentence of KD-19's
  *Consequence* field is true of the live tree.
- `git check-ignore` re-run for the new harness-gaps entry's claims: `ai-docs/plans/.task-inflight`,
  `ai-docs/plans/*.progress.md` and `ai-docs/plans/ignored/*` are all **not** ignored while `tmp/*` is,
  and `.gitignore` is ten lines with no `ai-docs/` entry. `/target/` was already ignored, so the new
  build directory needs no `.gitignore` change.
- `ai-docs/context.md` is correctly left unedited: its § Status reads "the crate layout as the
  workspace manifest lists it … What this page fixes is the shape, not the progress", so the Step-9.5
  rationale holds on the page's own words.

*Safety, style and domain:* no panicking call added (`make panic-calls` green, `ai-docs/panic-index.md`
untouched); no fallible call, no error type, no task, no channel and no lock in the diff; no domain
invariant reached — the diff's only domain word is prose inside a remit comment; no secret-shaped
string among the added lines; no file near any size band (largest new source is 3 lines).

*Test coverage:* no Rust test is expected or added, per D8 and AGENTS.md's ~50-lines-of-substantial-logic
threshold — the four sources carry 1–3 lines each and no logic. D8's stronger reason was re-measured
above: a placeholder test would move the workspace total off zero and hand the next task a floor set
by scaffolding.

**Reservations recorded, not raised — 10 items, none at or above the severity floor.**
Files: `crates/shared/src/lib.rs`, `crates/migrate/src/main.rs`, `crates/cli/src/main.rs`,
`crates/core/src/lib.rs`, `Cargo.toml`, `AGENTS.md`, `ai-docs/claude-tools-hierarchy.md`,
`ai-docs/key-decisions.md`, `ai-docs/context-status.md`, `ai-docs/learnings.md`,
`docs/09-build-and-deploy.md`, and this progress file. Each is a register row below.
