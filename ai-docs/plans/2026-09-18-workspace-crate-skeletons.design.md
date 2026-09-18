# Design: Workspace and crate skeletons

**Issue:** #10
**Date:** 2026-09-18

## Approach

### What the tree holds today

The repository carries no Cargo manifest at any depth
`[measured baabbb6 · find . -path ./.git -prune -o -path ./tmp -prune -o -name '*.toml' -print → no path printed]`,
so every cargo recipe of the project's build entry point takes a guard branch that prints a named
skip and exits 0
`[measured baabbb6:Makefile:25-31 · sed -n '25,31p' Makefile → the comment block headed "WHILE THE WORKSPACE IS EMPTY every cargo gate below skips itself" and the CARGO_GUARD definition that prints "no Cargo.toml at the repository root; <target> skipped"]`.
The same era is asserted in prose elsewhere in the tree — in live documents and in two gate scripts'
own docstrings — and this diff falsifies each of those sentences
`[measured baabbb6 · a multi-pattern case-insensitive sweep, for p in 'first crate' 'while the workspace' 'no workspace' 'nothing is in it' 'crate lands' 'no crate'; do grep -rn -i -- "$p" . ; done → the sites subtasks 2 and 3 name, and no live site outside them]`.
They are enumerated in subtasks 2 and 3 below.

This task does two things: it lays the workspace and its skeleton members down so the cargo gates
bind on real code, and it retires the empty-workspace era wherever the repository still states it.

### The workspace

A **virtual** manifest at the repository root — no `[package]` of its own — whose `members` are the
crate directories the spec names, each a minimal crate that compiles and delivers nothing
`[derived → AC1]`. The library members carry a crate-level `//!` comment and no items; the binary
members carry a crate-level `//!` comment and an empty `main` `[derived → AC1, AC2]`.

`docs/ARCHITECTURE.md` already binds those directories to the executable names this task
fixes
`[measured baabbb6:docs/ARCHITECTURE.md § Структура репозитория · git show baabbb6:docs/ARCHITECTURE.md | sed -n '/^## Структура репозитория/,/^## Документы/p' → the block lists "/crates/cli  reader-cli" and "/crates/migrate  reader-migrate: единственное место, где накатываются миграции"]`,
so the directory layout below is the corpus's, not a new one. The corpus's own structure stands: the
spec records the owner's choice to deliver the member set the task text names and to leave the server
crate to its own task
`[measured baabbb6:ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md:27-42 · sed -n '27,42p' ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md → § Deferred routes the server crate to a separate issue and § Source conflicts resolves the member set to the task text's, by the owner's round-1 answer "Четыре"]`,
and the build-and-deploy page keeps its unticked workspace row, which also covers the server member
and the frontend workspace
`[measured baabbb6:docs/09-build-and-deploy.md:4 · sed -n '4p' docs/09-build-and-deploy.md → "- [ ] Workspace `Cargo.toml` с `shared`, `core`, `cli`, `server`, `migrate`; `frontend` с `pnpm`."]` —
ticking it here would assert work this task does not do.

### Key decisions

**D1 — Every package name carries the `reader-` prefix; the directories keep the corpus's short
role names.** A workspace member packaged as `core` is put in the extern prelude of every crate that
depends on it and shadows the standard library's `core` there: an absolute `::core::fmt::Debug` path
— the exact shape a third-party derive macro emits — stops resolving
`[measured rustc@1.98.1 · a dependent of a workspace member packaged as `core`, holding `let x: &dyn ::core::fmt::Debug = &1u8;` → error[E0433]: cannot find `fmt` in `core`]`.
The built-in derives survive it, so the failure is latent until the first proc-macro dependency
arrives
`[measured rustc@1.98.1 · the same dependent carrying #[derive(Debug, Clone, PartialEq)] and no written ::core:: path → compiles]`.
Prefixing uniformly removes the whole class rather than special-casing one member, and it makes the
binary names fall out of the package names with no target override (D3). The corpus names the crates
by role in prose (`core`, `shared`), which the directories keep; what the corpus does not state is
the package name, so this is surfaced for confirmation in § Open questions rather than decided
silently.

**D2 — Edition 2024, with `resolver = "3"` written explicitly on the virtual manifest.** A virtual
manifest whose members are on edition 2024 and which states no resolver emits a warning on every
build
`[measured cargo@1.98.1 · cargo build --workspace --all-targets against such a workspace → "warning: virtual workspace defaulting to `resolver = "1"` despite one or more workspace members being on edition 2024 which implies `resolver = "3"`"]`.
Cargo's warning is not caught by the linter's `-D warnings`, so it would be permanent noise that no
gate removes; stating the resolver is the fix, not suppressing it.

**D3 — The fields every member manifest would otherwise repeat live in `[workspace.package]`, and
each member inherits them with `.workspace = true`.** The fields are the version, the edition, the
licence and the publish flag; each would be copied into every member manifest, which is past the two
sites the duplication rule calls borderline and squarely inside the "lift it" case. The binary names
are **not** among them: each binary member is packaged under the name its executable must carry, so
the executable name follows from the package name with no `[[bin]]` block
`[derived → AC2]`. The rejected alternative — packages named after the directories plus a `[[bin]]`
block renaming the target — costs a second name per binary member to keep in sync and buys nothing
that D1 does not already give.

**D4 — `publish = false` in `[workspace.package]`.** This is an application and nothing outside the
workspace depends on it
`[measured baabbb6:AGENTS.md § API Stability · grep -n 'nothing outside this workspace depends on it' AGENTS.md → 183: "ai-translator is an application, not a library — nothing outside this workspace depends on it."]`;
the flag makes an accidental publish a refusal rather than an upload. `license` is set from the
repository's own licence
`[measured baabbb6:LICENSE:1-2 · head -2 LICENSE → "Apache License / Version 2.0, January 2004"]`,
which the README states in the same terms
`[measured baabbb6:README.md:37-39 · sed -n '37,39p' README.md → the § Лицензия section names Apache License 2.0 and links the repository's licence file]`.

**D5 — `[workspace.dependencies]` is present and empty, carrying a comment that states the rule.**
The mechanism is the owner's own words in the issue; the width of the set is the owner's round-1
answer
`[measured baabbb6:ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md.state.md:43 · grep -n 'Минимум' ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md.state.md → 43: answer: "Минимум"]`,
which the spec carries in both its scope row and its key-decisions row
`[measured baabbb6:ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md:14,33 · grep -n 'Минимум' ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md → the scope row "the set reaches no further than what the skeletons compile with" and the decision row "a dependency nothing yet compiles against is not declared in advance"]` —
nothing is declared for later use. Nothing the skeletons compile with is a third-party crate, so the table has
no rows yet, and the empty table is what the next task adds its first row to. An empty table is
accepted by the toolchain with no diagnostic `[measured cargo@1.98.1 · cargo build --workspace --all-targets and cargo metadata --locked --format-version 1 against a workspace carrying an empty [workspace.dependencies] → both succeed, no warning line]`.
The rejected alternative — omitting the table until something needs it — drops the mechanism the
issue names.

**D6 — No dependency edge between members.** Nothing in a skeleton compiles against another
skeleton, so the engine crate does not depend on the protocol crate and neither binary depends on
the engine. An edge added now would be a dependency declared for later use, which the spec refuses
`[measured baabbb6:ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md:24 · sed -n '24p' ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md → "Declaring a dependency the skeletons do not compile with, for later use."]`;
the task that writes the first call adds it `[derived → AC1]`.

**D7 — No `[workspace.lints]`, no `rust-toolchain.toml`, no `rust-version`.** The linter posture is
already a command line that the build entry point and CI both run with warnings denied
`[measured baabbb6:Makefile:55-56 · sed -n '55,56p' Makefile → the clippy recipe is cargo clippy --workspace --all-targets -- -D warnings]`,
so a lint table would be a second, divergable statement of it; an MSRV field has no consumer while
nothing outside the workspace depends on it; a toolchain file would pin what CI deliberately reads as
`stable`
`[measured baabbb6:.github/workflows/ci.yml:96-164 · grep -n 'rust-toolchain@stable' .github/workflows/ci.yml → every Rust job in the range uses dtolnay/rust-toolchain@stable]`.
Each is a policy this task was not asked for, and each remains available to the task that needs it.

**D8 — The skeletons carry no test.** They carry no logic, so the standing substantial-logic
threshold is not reached
`[measured baabbb6:AGENTS.md § Workflow · grep -n 'Any file with ~50+ lines' AGENTS.md → 270: "Any file with ~50+ lines of substantial logic MUST have tests."]`,
and there is nothing a test could assert that the build does not. The stronger reason is the ratchet: a test module's own lines are
counted and reported covered, so one trivial placeholder test moves the workspace total off zero and
records a high-water mark set by scaffolding rather than by covered production code
`[measured cargo-llvm-cov@0.9.1 · cargo llvm-cov --workspace --summary-only --json over a scratch workspace before and after one #[cfg(test)] module with one trivial assertion is added to a library → before, the libraries appear in no file entry and nothing is reported covered; after, that library appears with every line covered and the workspace total is no longer zero]`.
The next task would then inherit a floor it did not earn and would be blocked by it. What the mark
must **not** be is unmeasurable: a library holding only a `//!` comment contributes no line at all,
while a binary's `main` does, so the binary members are what keep the summary from reporting a
workspace with no executable lines
`[measured cargo-llvm-cov@0.9.1 · the same command over a scratch workspace of doc-only libraries and empty-bodied binaries → only the binary sources appear as file entries and the line total is non-zero]`
`[derived → AC5]`.

**D9 — The manifest guard in the build entry point is deleted, not reworded.** Once the manifest
exists the guard can only fire on a broken tree, and what it does there is turn "the gates could not
run" into exit 0 — the silent-success shape the project's own rule refuses
`[measured baabbb6:AGENTS.md § Build & Test · grep -n 'a job that did not run is not a passing job' AGENTS.md → 176: "a job that did not run is not a passing job — read the run, not the absence of red."]`.
Deleting it lets the toolchain fail loudly instead, and it is the only way the file stops stating
the empty-workspace era at all `[derived → AC3, AC4]`. No regression suite drives it
`[measured baabbb6 · grep -rn -iE 'CARGO_GUARD|no Cargo\.toml at the repository root' ai-docs/scripts/ .githooks/ .claude/ → only the gate scripts' own printf lines, no fixture]`.
The fail-open branches **inside** the gate scripts are a different thing and stay: those scripts are
driven over scratch trees by their own suites, so their conditionals remain true statements about
what the script does — only the sentences asserting this repository's present state change.

**D10 — Dependabot's cargo ecosystem is restored in the same pull request as the workspace, not
necessarily in the same commit.** The propagation table's row says "in the same commit"
`[measured baabbb6:ai-docs/propagation-groups.md:41 · sed -n '41p' ai-docs/propagation-groups.md → "The commit that creates the cargo workspace (a root `Cargo.toml`) | `.github/dependabot.yml` … Restore it in the same commit AND …"]`,
while the standing rule it implements is the same **pull request**
`[measured baabbb6:AGENTS.md § Propagation Rule · grep -n 'SAME pull request' AGENTS.md → 295: "AXIOM — Edits to one instruction file MUST propagate to its sync-group siblings in the SAME pull request."]`.
The row's stricter wording buys nothing here: Dependabot resolves its configuration and the manifests
it updates from the default branch only
`[measured docs.github.com@2026-09-18 · WebFetch https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuring-dependabot-version-updates → "When this file is checked in, Dependabot checks the manifest files on the default branch for outdated dependencies"]`,
and `master` is reached by a merge commit only, so the interval between two commits of a feature
branch is not a state Dependabot ever sees. Keeping the restoration in its own commit is what lets
the code group and the harness group stay change-type homogeneous. The wording discrepancy is raised
in § Open questions.

**D11 — The lockfile is generated by the build and committed.** The lockfile gate runs
`cargo metadata --locked`, which fails rather than writing one
`[measured baabbb6:Makefile:74-75 · sed -n '74,75p' Makefile → the lock-check recipe is cargo metadata --locked --format-version 1]`,
so a manifest committed without its lockfile is a red gate on the next machine. It is never
hand-edited
`[measured baabbb6:AGENTS.md § Dependency Versions · grep -n 'never hand-edit a version' AGENTS.md → 243: "never hand-edit a version in `Cargo.lock`"]`
`[derived → AC1]`.

**D12 — The crate-naming rule is recorded as a project key decision.** `ai-docs/key-decisions.md`
states its own standing rule that a decision reached during a task is recorded there in the same
pull request that takes it
`[measured baabbb6:ai-docs/key-decisions.md:7 · sed -n '7p' ai-docs/key-decisions.md → "A decision reached during a task is recorded here in the same pull request that takes it, with the same shape."]`.
D1 binds every crate this repository will ever add, including the deferred server member, so it is
the one decision here that outlives the task.

### What the gates will read afterwards

The CI paths filter already routes the new artefact classes: its Rust filter lists the Rust sources,
every manifest and the lockfile
`[measured baabbb6:.github/workflows/ci.yml:39-52 · sed -n '39,52p' .github/workflows/ci.yml → the rust filter lists '**/*.rs', '**/Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'Makefile' and '.github/workflows/**']`,
so no filter edit is needed and no gate silently stops running `[derived → AC3]`.

The comment-reference gate changes behaviour once crate directories exist, and it was checked before
specifying anything: its repository-path class already lists the crate
directory among the top-level names, and its root-name set is a fixed literal that the root manifest's
file name does not join
`[measured baabbb6:ai-docs/scripts/comment_refs.py:42-54 · sed -n '42,54p' ai-docs/scripts/comment_refs.py → GATED_ROOT_NAMES is the literal set {"Makefile", ".gitignore"}, and REPO_TOP_DIRS already lists "crates"]`;
its crate-symbol class becomes live, reading both each member's package name and each crate
directory's own name
`[measured baabbb6:ai-docs/scripts/comment_refs.py:98-126 · sed -n '98,126p' ai-docs/scripts/comment_refs.py → workspace_crates adds the manifest's name field and the directory entry name, and own_crate exempts a comment inside the crate it names]`.
No comment anywhere in the gated set trips that class under the crate-name set this task creates
`[measured baabbb6 · python3 tmp/probe-crate-symbol.py, which classifies every tracked gated comment against that name set → no comment reported; the constructed control "core::Engine" was reported as crate-symbol outside the crate and reported nothing inside it, and a constructed "Cargo.toml" string was reported as nothing]`.
The consequence for the new sources is a constraint, not a finding: a `//!` comment may not name a
repository path, a markdown file, a section sign, an issue number, a URL or another member's
crate-qualified symbol
`[measured baabbb6:ai-docs/doc-convention.md § DOC-4 · sed -n '57,90p' ai-docs/doc-convention.md → the banned-class table lists each of them, with the own-crate contract symbol exempted]`,
so each crate comment states its own remit and stops there `[derived → AC1]`.

The panic gate reads the new binary sources — a binary's `main` is not excluded by position
`[measured baabbb6:ai-docs/scripts/panic_calls.py:58-61 · sed -n '58,61p' ai-docs/scripts/panic_calls.py → is_excluded covers build.rs and the tests, benches and examples directories only]` —
and the index it backs is empty and is to stay empty
`[measured baabbb6:ai-docs/panic-index.md · cat ai-docs/panic-index.md → the table carries a single placeholder row of em dashes]`.
Nothing in a skeleton panics `[derived → AC3]`.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | The workspace manifest and its skeleton members: virtual root manifest (members, `resolver`, `[workspace.package]`, empty `[workspace.dependencies]`), one manifest and one root source per member, each binary member packaged under the executable name the spec fixes for it, the lockfile generated by the build. The commit's own pre-commit hook measures coverage, finds no ratchet file and initialises it at the measured value, staging it into this same commit. | `Cargo.toml`, `Cargo.lock`, `crates/shared/Cargo.toml`, `crates/shared/src/lib.rs`, `crates/core/Cargo.toml`, `crates/core/src/lib.rs`, `crates/cli/Cargo.toml`, `crates/cli/src/main.rs`, `crates/migrate/Cargo.toml`, `crates/migrate/src/main.rs`, `ai-docs/coverage-ratchet.txt` (written and staged by the hook, not by hand) | — |
| 2 | Retire the empty-workspace guard and every live statement of it: delete the manifest guard and its comment block from the build entry point and leave each recipe as the bare gate command; correct the same file's file-size comment, which asserts the tree is empty (the bands themselves are not re-set — a skeleton is not the real distribution they wait for); replace the build-and-test blockquote and the tool-inventory sentence that assert the era; correct the gate-script docstrings and the script comment that assert this repository's present state. **What stays:** a conditional branch inside a gate script, and a state-table row that describes one, remains a true statement about what the script does, so neither is edited — only a sentence asserting this repository's present state is. The build entry point is in the comment-reference gated set, so a rewritten comment there may name the manifest's file name (measured safe) but no repository path under a crate directory. | `Makefile`, `AGENTS.md`, `ai-docs/claude-tools-hierarchy.md` (§ CI), `ai-docs/scripts/import_guard.py`, `ai-docs/scripts/comment_refs.py`, `.githooks/coverage-ratchet.sh` | 1 |
| 3 | The cargo ecosystem returns to Dependabot: add the cargo entry at the repository root with the weekly cadence, the open-pull-request limit and the commit prefix the propagation row fixes, and drop the comment that explains its absence; propagate to the triage skill's preamble and the tool inventory's Dependabot section; retire the propagation row itself, whose trigger this pull request discharges and whose claim it falsifies. The configuration file is itself in the comment-reference gated set, so any comment left in it obeys the same reference ban as a Rust one — no markdown path, no section sign, no repository path. | `.github/dependabot.yml`, `.claude/skills/dependabot-pr/SKILL.md` (§ preamble), `ai-docs/claude-tools-hierarchy.md` (§ Dependabot), `ai-docs/propagation-groups.md` | 1 |
| 4 | Record the crate-naming rule (D1) as a project key decision, in the page's own shape — decision, why, consequence, source — under § Repository and process, numbered after the last row the page carries. | `ai-docs/key-decisions.md` | 1 |

## Handoff plan

`M = 4`. Two groups, homogeneous by change-type, minimised: every harness subtask depends on the one
code subtask and none of them depends on another, so the harness subtasks cluster into a single
group with no dependency-forced interleaving. Two groups is within the default maximum of four, so
no user approval is needed.

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md`
  § Compaction recovery (re-entry). The first group takes a handoff exactly as every later one does.
- **Group A** — model `sonnet`, effort `medium` (pinned in the `code-writer` frontmatter; no inline
  `model=` or effort override), 1M-token window, via `subagent_type="code-writer"` — subtask 1 (code
  change-type: `*.rs` and the cargo manifests and lockfile that ship with them). The ratchet file
  this commit carries is written and staged by the pre-commit hook, not authored — it is a gate
  artefact of the same commit, so the group stays homogeneous by change-type.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md`
  § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B** — model `inherit` (the orchestrator's), effort inherited from the orchestrator
  (typically xHigh, not pinned), 1M-token window, via `subagent_type="general-purpose"` with no
  inline `model=` — subtasks 2, 3, 4 (instructions/harness change-type: `Makefile`, `AGENTS.md`,
  `ai-docs/**`, `.claude/**`, `.github/dependabot.yml`, `.githooks/**`). Terminal group (3 subtasks;
  within the `1..=10` range).

`ai-docs/claude-tools-hierarchy.md` is touched by subtasks 2 and 3 of Group B, in different sections —
§ CI in the first, § Dependabot in the second. They are sequential commits inside one group, so the
later one reads the earlier one's result; neither section is the other's.

## Risks

- **The ratchet's raise mode blocks on unstaged coverage-moving files, so a partial `git add` turns
  subtask 1's commit into a refusal.** Mitigation: stage every new manifest, source and the lockfile
  before committing; the hook's own message names the offending paths and the legitimate ways out —
  `[measured baabbb6:.githooks/coverage-ratchet.sh:99-112 · sed -n '99,112p' .githooks/coverage-ratchet.sh → the BLOCKED branch lists the dirty paths and prints the git add / git stash recipe]`.
- **The whole-tree lexical gates read the git index, not the working tree, so they report a
  clean sweep over sources that are not staged yet.** Both enumerate through `git ls-files`
  `[measured baabbb6:ai-docs/scripts/comment_refs.py:162-176 · sed -n '162,176p' ai-docs/scripts/comment_refs.py → git_files falls back to git ls-files]`
  `[measured baabbb6:ai-docs/scripts/panic_calls.py:102-116 · sed -n '102,116p' ai-docs/scripts/panic_calls.py → the same fallback over git ls-files '*.rs']`,
  so a green run of either before `git add` is evidence about an enumeration that reached nothing.
  Mitigation: run both against the new paths **by explicit path** first, then again after staging.
- **The coverage job is the one CI gate that cannot run before the ratchet file exists**, since its
  check-only mode treats a missing file as a block rather than an initialisation
  `[measured baabbb6:.githooks/coverage-ratchet.sh:156-167 · sed -n '156,167p' .githooks/coverage-ratchet.sh → the --check branch prints BLOCKED and the measured value when the file is missing or unparseable]`.
  Mitigation: the file is created inside subtask 1's own commit by the hook, so no commit of this
  branch ever reaches CI without it `[derived → AC5]`.
- **A gate that stops at the first failing crate hides the sites behind it**, so an enumeration read
  off one red run is a floor. Mitigation: after subtask 1 is green, re-run the aggregate and read its
  tail rather than trusting the first list; any newly revealed class outside this design's contract
  is surfaced to the orchestrator rather than absorbed.
- **The sweep for statements of the empty-workspace era is a prose sweep, and prose hides an
  encoding.** The enumeration in subtasks 2 and 3 was built from several simple case-insensitive
  patterns rather than one alternation, across English and Russian and across the assertion and the
  printf spellings
  `[measured baabbb6 · for p in 'first crate' 'while the workspace' 'no workspace' 'nothing is in it' …; do grep -rn -i -- "$p" . ; done → the sites subtasks 2 and 3 name, and no other outside ai-docs/plans and tmp]`.
  Mitigation: subtask 2 re-runs the same sweep after its edits and reads the residue; a hit left in a
  history surface is left alone by design
  `[measured baabbb6:AGENTS.md § Propagation Rule · grep -n 'When the change propagates a' AGENTS.md → 310: step 4 sweeps the user-facing documents and leaves the history surfaces untouched]`
  `[derived → AC4]`.
- **Deleting the manifest guard makes a broken tree loud rather than green**, which is the point, but
  it also means a machine without the toolchain now fails the aggregate instead of skipping it. That
  is the intended direction and matches the project's own rule that a gate which could not run is not
  a green tree; no mitigation is wanted, and the change is recorded in the build-and-test section that
  subtask 2 rewrites `[derived → AC3]`.
- **A doc comment is the one place in the new sources that can fail a gate**, through the reference
  ban or through rustdoc's denied warnings. Mitigation: each crate comment states its own remit and
  names no path, no section, no issue number, no URL and no other member's qualified symbol; the
  documentation gate is part of the aggregate run that subtask 1 must pass `[derived → AC1]`.
- **The executable names are a spec-fixed contract that a manifest rename would break silently** —
  nothing in the tree asserts them today. Mitigation: § Test Design names the command that reads the
  built target names back out of the workspace metadata, and it is run as part of subtask 1's
  verification `[derived → AC2]`.

## Test Design

The skeletons carry no logic, so this task adds no Rust test (D8). What it adds instead is a
verification pass whose subject is the gates themselves — the thing the issue asks for is that they
bind. Nearly every claim below is about an artefact this task creates and therefore carries
`[derived → …]`; the one exception is the external documentation the Dependabot entry is written
against.

**Subtask 1 — the workspace.**

- Entry point: the project's aggregate build entry point, and the lockfile gate inside it.
- Scenarios:
  - *The workspace compiles and lints clean* — the aggregate run passes with no cargo diagnostic of
    its own (the resolver warning of D2 is the one to watch for, since no `-D warnings` catches it)
    `[derived → AC1]`.
  - *The executables carry the fixed names* — read the binary target names out of the workspace
    metadata (`cargo metadata --format-version 1 --no-deps`, the `targets` entries whose kind is
    `bin`) and compare them against the names the spec fixes. Reading metadata rather than
    listing a build directory keeps the check independent of the profile and of any stale artefact
    `[derived → AC2]`.
  - *The manifest and the lockfile agree* — the lockfile gate, which refuses to write one
    `[derived → AC1]`.
  - *The two lexical gates see the new sources* — run the comment-reference gate and the panic gate
    against the new paths by explicit path before staging, then again through the staged set, and
    read both results (§ Risks: an enumeration that reached nothing reports clean) `[derived → AC1]`.
  - *The ratchet initialises* — after the commit, confirm the ratchet file is part of that commit and
    that the check-only mode reports a comparison rather than a missing file or a summary with no
    lines `[derived → AC5]`.
- Fixtures: none. The suite provisions nothing here — no database-backed or model-backed test exists
  yet, so the aggregate run needs no container runtime for this subtask; the coverage measurement
  runs the same empty suite.

**Subtask 2 — the retired era.**

- Entry point: the aggregate run's stderr, and the prose sweep.
- Scenarios:
  - *No gate reports itself skipped for want of a manifest* — capture the aggregate run to a file
    under the scratch directory and read the capture for the skip sentence each recipe would have
    printed. The instrument is checked in its red direction first: the same pattern is run against
    the pre-change file (`git show HEAD:Makefile`) and must match there, or the clean result is
    evidence about the pattern `[derived → AC3]`.
  - *No live statement of the era survives* — re-run the multi-pattern sweep of § Risks, in both
    languages, and read every residual hit: a hit in a history surface is correct and stays; a hit in
    a live document or script is a finding `[derived → AC4]`.
  - *The edited scripts still run* — the shell gate over the changed script, and the regression
    suites that drive the edited modules, since a docstring edit that breaks a module breaks its gate
    `[derived → AC3]`.
- Fixtures: the pre-change file content from `git show`, used only as the red control above.

**Subtask 3 — Dependabot.**

- Entry point: the configuration file, and the documents that describe it.
- Scenarios:
  - *The configuration is well formed and names the right ecosystem* — the identifier for Rust is
    `cargo`, and both the commit-prefix and the open-pull-request-limit keys are valid for it
    `[measured docs.github.com@2026-09-18 · WebFetch https://docs.github.com/en/code-security/dependabot/working-with-dependabot/dependabot-options-reference → the Rust identifier is "cargo"; commit-message.prefix and open-pull-requests-limit are listed as valid keys]`.
    This file is not reached by the workflow linter, which reads the workflow directory only, so the
    check is a read of the options reference against the written entry `[derived → AC4]`.
  - *Both documents describe the configuration as it now stands* — read the triage skill's preamble
    and the tool inventory's Dependabot section back against the file `[derived → AC4]`.
  - *The discharged propagation row is gone and nothing still points at it* — sweep the harness corpus
    for references to that row before removing it `[derived → AC4]`.
- Fixtures: none.

**Subtask 4 — the key decision.**

- Entry point: the key-decisions page.
- Scenarios: *the row is in the page's own shape and its source resolves* — decision, why,
  consequence, source, under § Repository and process, numbered after the last row the page carries;
  the relative-link check in CI sweeps every tracked document, so any link the row adds must resolve
  from that page's directory `[derived → AC4]`.
- Fixtures: none.

## Open questions

- **The package names are the corpus's one silence, and D1 fills it — confirm rather than assume.**
  `docs/ARCHITECTURE.md` and `docs/backend/ARCHITECTURE.md` name the crates by role and fix the
  executable names
  `[measured baabbb6:docs/backend/ARCHITECTURE.md:3 · sed -n '3p' docs/backend/ARCHITECTURE.md → "Модули ниже живут в крейте `core`, кроме `http/` и `ws/`, которые составляют крейт `server`."]`;
  neither states a package name. D1 proposes
  `reader-`-prefixed package names on the measured ground that a member packaged as `core` shadows
  the standard library's `core` in every dependent, and the directories keep the corpus's names.
  Since the corpus is DECISIONS and this is a question it leaves open, the owner's confirmation is
  wanted before subtask 1 lands — not a redesign of the corpus, but a naming the corpus did not make.
- **The propagation table's row says "in the same commit" where the standing rule says "in the same
  pull request".** D10 follows the standing rule, on the measured ground that Dependabot only ever
  reads the default branch, so the stricter wording constrains nothing real and would force a
  mixed-change-type group. Subtask 3 retires that row as discharged, so the discrepancy disappears
  with it; if the orchestrator would rather keep the stricter wording as a general pattern for
  future one-shot triggers, that belongs in `ai-docs/harness-gaps.md` and is the orchestrator's to
  route, not this design's to write.
- **No spec row was found to prescribe a mechanism this design would otherwise choose differently.**
  Scope 3 names `[workspace.dependencies]`, but that is the owner's own wording in the issue body as
  the interview state file persists it, anchored as such in the spec — not a spec-side choice of how,
  so no `SPEC-REMIT` tag is raised.
