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
The same era is asserted in prose elsewhere in the tree — in live documents and in the gate scripts'
own docstrings — and this diff falsifies each of those sentences
`[measured baabbb6 · a multi-pattern case-insensitive sweep, for p in 'first crate' 'while the workspace' 'no workspace' 'nothing is in it' 'crate lands' 'no crate'; do grep -rn -i -- "$p" . ; done → the sites subtasks 2 and 3 name, and no live site outside them]`.

A second class of sentence says the same thing in different words and was missed by that pattern
set: the **file-size bands** are documented as unmeasured *because the tree holds no crate*, in the
build entry point's own comment and in its twin in the code-style reference. Widening the sweep with
the bands' own wording reaches both and nothing else live
`[measured 0b2bc15 · for p in 'measured against this tree' 'real distribution' 'once crates exist' 'nothing is in it'; do grep -rn -i -- "$p" . ; done, with .git, tmp and ai-docs/plans excluded → Makefile:39-41 and ai-docs/code-style.md:90, and no other live site; the constructed control string "measured against this tree" written to a scratch file was matched, so the pattern ran]`.
Both sentences are subtask 2's, and the judgement they now carry is the same one: crates exist after
this task, so the trigger they name has fired, but a skeleton is **not** the real distribution the
bands wait for, so the re-set stays deliberately deferred to the task that has code to measure
`[derived → AC1]`. The bands themselves do not move here.

**Which criterion owns which twin is worth stating, because they differ.** The build entry point's
band comment sits in the same file as the manifest guard and asserts the same empty tree, so
correcting it is part of the AC4 edit `[derived → AC4]`. The code-style reference's twin asserts
nothing about the cargo gates, so it is **not** AC4 work on its own — it is the propagation that the
AC4 edit triggers, and leaving it standing while its sibling is rewritten is exactly the divergence
the propagation rule's *every live document must agree* step exists to prevent
`[measured 0b2bc15:AGENTS.md:310 · grep -n 'Every LIVE document must agree' AGENTS.md → 310: § Propagation Rule step 4, "Every LIVE document must agree; history surfaces (`ai-docs/learnings.md`, `ai-docs/plans/done/**`) are left untouched."]`.

All of these sites are enumerated in subtasks 2 and 3 below.

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
`[measured 0b2bc15:ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md:26-42 · sed -n '26,42p' ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md → § Deferred routes the server crate to a separate issue and § Source conflicts resolves the member set to the task text's, by the owner's round-1 answer "Четыре"]`,
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
binary names fall out of the package names with no target override (D3).

**The corpus names the crates by role and never by package, at every site that names them, and the
owner ruled on the silence.** The live corpus sites that name a crate are the repository-structure
block, which binds each directory to its role and fixes the executable names
`[measured baabbb6:docs/ARCHITECTURE.md § Структура репозитория · git show baabbb6:docs/ARCHITECTURE.md | sed -n '/^## Структура репозитория/,/^## Документы/p' → "/crates/shared … /crates/core движок (см. 11) … /crates/cli reader-cli … /crates/migrate reader-migrate: единственное место, где накатываются миграции"]`;
the backend module map, which assigns the modules to crates by role
`[measured 0b2bc15:docs/backend/ARCHITECTURE.md:3 · sed -n '3p' docs/backend/ARCHITECTURE.md → "Модули ниже живут в крейте `core`, кроме `http/` и `ws/`, которые составляют крейт `server`."]`;
the build-and-deploy task row, which is the row describing the workspace manifest's own contents
`[measured 0b2bc15:docs/09-build-and-deploy.md:4 · sed -n '4p' docs/09-build-and-deploy.md → "- [ ] Workspace `Cargo.toml` с `shared`, `core`, `cli`, `server`, `migrate`; `frontend` с `pnpm`."]`;
and the core-API page's opening row
`[measured 0b2bc15:docs/11-core-api-and-cli.md:7 · sed -n '7p' docs/11-core-api-and-cli.md → "- [ ] Крейт `core` с публичным `Engine`:"]`.
Every one of them writes the short role name and none of them writes a package name — including the
structure block and the build-and-deploy row, which are precisely the rows a reader would consult for
the manifest's contents. So the silence is the corpus's across the whole set the decision rules over,
not a gap in one page. What the corpus *does* write with a `reader-` prefix is only ever an
**executable** name, at every occurrence
`[measured 0b2bc15 · grep -rn 'reader-' docs/ → only `reader-cli` and `reader-migrate`, each as a binary that is listed in the structure block or invoked in a task row; no `reader-shared` and no `reader-core` anywhere; a constructed control string "reader-x" written to a scratch file was matched, so the pattern ran]`.
Under D3 the executable name falls out of the package name, so for the binary members D1 adopts a
name the corpus already fixed; the genuinely new names it introduces are the library members'
`[derived → AC1, AC2]`.

Since the corpus is DECISIONS and this is a question it leaves open, it was put to the owner rather
than decided silently. The owner's answer, verbatim, is **"reader-\* у всех"**, with the directories
fixed in the same exchange: "каталоги в любом случае остаются shared/core/cli/migrate"
`[measured 0b2bc15:ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md.state.md:44-46 · sed -n '44,46p' ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md.state.md → the round-2 prior_qa entry whose question names the package-name silence and whose answer is "reader-* у всех"]`.
So the packages are `reader-shared`, `reader-core`, `reader-cli` and `reader-migrate`, and the
directories stay `crates/shared`, `crates/core`, `crates/cli` and `crates/migrate`. This is a design
decision carrying the owner's words; it is **not** a spec row and none is added for it. The
divergence it creates between directory name and package name has a measured consequence inside the
comment-reference gate — see § *What the gates will read afterwards*, where it is stated as a
standing constraint rather than as a finding.

**D2 — Edition 2024, with `resolver = "3"` written explicitly on the virtual manifest.** A virtual
manifest whose members are on edition 2024 and which states no resolver emits a warning on every
build
`[measured cargo@1.98.1 · cargo build --workspace --all-targets against such a workspace → "warning: virtual workspace defaulting to `resolver = "1"` despite one or more workspace members being on edition 2024 which implies `resolver = "3"`"]`.
Cargo's warning is not caught by the linter's `-D warnings`, so it would be permanent noise that no
gate removes
`[measured cargo@1.98.1 · cargo clippy --workspace --all-targets -- -D warnings against that same workspace → exit 0 with the resolver warning printed; -D warnings promotes the compiler's lints and leaves cargo's own diagnostics alone]`;
stating the resolver is the fix, not suppressing it.

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
`[measured 0b2bc15:LICENSE:1-3 · head -3 LICENSE → a blank line, then "Apache License", then "Version 2.0, January 2004"]`,
which the README states in the same terms
`[measured baabbb6:README.md:37-39 · sed -n '37,39p' README.md → the § Лицензия section names Apache License 2.0 and links the repository's licence file]`.

**D5 — `[workspace.dependencies]` is present and empty, carrying a comment that states the rule.**
The mechanism is the owner's own words in the issue; the width of the set is the owner's round-1
answer
`[measured 0b2bc15:ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md.state.md:43 · grep -n 'Минимум' ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md.state.md → 43: answer: "Минимум"]`,
which the spec carries in both its scope row and its key-decisions row
`[measured 0b2bc15:ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md:14,33 · grep -n 'Минимум' ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md → the scope row "the set reaches no further than what the skeletons compile with" and the decision row "a dependency nothing yet compiles against is not declared in advance"]` —
nothing is declared for later use. Nothing the skeletons compile with is a third-party crate, so the table has
no rows yet, and the empty table is what the next task adds its first row to. An empty table is
accepted by the toolchain with no diagnostic `[measured cargo@1.98.1 · cargo build --workspace --all-targets and cargo metadata --locked --format-version 1 against a workspace carrying an empty [workspace.dependencies] → both succeed, no warning line]`.
The rejected alternative — omitting the table until something needs it — drops the mechanism the
issue names.

**D6 — No dependency edge between members.** Nothing in a skeleton compiles against another
skeleton, so the engine crate does not depend on the protocol crate and neither binary depends on
the engine. An edge added now would be a dependency declared for later use, which the spec refuses
`[measured 0b2bc15:ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md:24 · sed -n '24p' ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md → "Declaring a dependency the skeletons do not compile with, for later use."]`;
the task that writes the first call adds it `[derived → AC1]`.

**D7 — No `[workspace.lints]`, no `rust-toolchain.toml`, no `rust-version`.** The linter posture is
already a command line that the build entry point and CI both run with warnings denied
`[measured baabbb6:Makefile:55-56 · sed -n '55,56p' Makefile → the clippy recipe is cargo clippy --workspace --all-targets -- -D warnings]`,
so a lint table would be a second, divergable statement of it; an MSRV field has no consumer while
nothing outside the workspace depends on it; a toolchain file would pin what CI deliberately reads as
`stable`
`[measured 20aa023:.github/workflows/ci.yml · grep -n 'dtolnay/rust-toolchain' .github/workflows/ci.yml → every occurrence in the file is dtolnay/rust-toolchain@stable, the Rust jobs' and the one in the harness-guards job below them alike; no job pins a version]`.
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
driven over scratch trees by their own suites
`[measured e1f8a4d:ai-docs/scripts/test-import-guard.sh, ai-docs/scripts/test-comment-refs.sh, ai-docs/scripts/test-panic-calls.sh · for f in test-import-guard test-comment-refs test-panic-calls; do grep -n 'mktemp -d' "ai-docs/scripts/$f.sh"; done → each suite builds its own sandbox before driving its gate]`,
so their conditionals remain true statements about what the script does — only the sentences
asserting this repository's present state change. Which sentences those are, per script, is
enumerated in subtask 2.

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
the code group and the harness group stay change-type homogeneous.

The discrepancy was put to the owner rather than resolved by this design. The owner's answer,
verbatim, is **"Снять строку"** — the project follows the AXIOM and retires the row as discharged
`[measured 0b2bc15:ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md.state.md:47-49 · sed -n '47,49p' ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md.state.md → the round-2 prior_qa entry whose question names the same-commit / same-pull-request discrepancy and whose answer is "Снять строку"]`.
Subtask 3 removes it. This is a design decision carrying the owner's words, not a spec row, and the
owner wanted no `harness-gaps.md` entry for it.

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
the one decision here that outlives the task. Because it outlives the task, its *consequence* field
carries the measured comment-gate constraints the directory/package divergence creates — see
§ *What the gates will read afterwards*. Recording the decision without them would hand a later crate
author a rule whose cost is invisible until a commit-blocking gate refuses their doc comment.

**D13 — Of the cargo entry's settable fields, the commit prefix is fixed by the corpus and the
cadence and the open-pull-request limit are copied from the sibling entry.** The prefix is `build`,
and the one live document that fixes it is the tool inventory's Dependabot section — the very section
subtask 3 rewrites
`[measured 20aa023:ai-docs/claude-tools-hierarchy.md:147 · sed -n '147p' ai-docs/claude-tools-hierarchy.md → the section opens by stating the file's cadence, describes the single github-actions entry's commit prefix and its open-pull-request limit, and then says of the cargo ecosystem "It returns with the commit that creates the workspace, carrying the `build` prefix, and the propagation table binds the two."]`.
So the rewrite carries the prefix forward rather than restating it from recall, and § Test Design
reads the written value back against the **pre-change** text of that section rather than against the
rewrite. The propagation row fixes none of these fields — its whole obligation is *where* to
propagate, never *what* to write
`[measured 20aa023:ai-docs/propagation-groups.md:41 · sed -n '41p' ai-docs/propagation-groups.md → the row says the cargo ecosystem is left out while no manifest exists, then "Restore it in the same commit AND `.claude/skills/dependabot-pr/SKILL.md` § preamble AND `ai-docs/claude-tools-hierarchy.md` § Dependabot"; no cadence, no limit, no prefix]`.

The cadence and the open-pull-request limit are fixed by **no** live document: the only such limit
written anywhere in the tree belongs to the sibling `github-actions` entry
`[measured 20aa023 · grep -rn -i 'open-pull-requests-limit' . with .git, tmp and ai-docs/plans excluded → the only site in the tree is .github/dependabot.yml:13, inside the github-actions entry, and there is no other; a constructed control line carrying the same string was matched, so the pattern ran]`.
Both are therefore **copied from that sibling entry** — same cadence, same limit — because a
divergence between two entries of one file would be a policy nobody decided, and because the
section's own opening sentence already states the cadence as a property of the file rather than of an
entry. Neither value is written as a number here: the implementor reads the sibling entry and copies
it, and the owner may move either later. This matters more than it looks, because **nothing
downstream would catch a wrong value** — no gate reads the file's values at all
`[measured 20aa023 · grep -rn -i 'dependabot' Makefile ai-docs/scripts/ .githooks/ .github/workflows/ → no hit, with a constructed control line matched so the pattern ran; and sed -n '81,82p' Makefile → the workflow-linter recipe is `actionlint .github/workflows/*.yml`, a glob the configuration file's path does not match]`.
The comment-reference gate reads its comment **lines** and nothing of its semantics, which is the
constraint subtask 3 already carries.

### What the gates will read afterwards

The CI paths filter already routes the new artefact classes: its Rust filter lists the Rust sources,
every manifest and the lockfile
`[measured 20aa023:.github/workflows/ci.yml:39-52 · sed -n '39,52p' .github/workflows/ci.yml → the rust filter lists '**/*.rs', '**/*.sql', '**/*.golden' (the last carrying its own comment on why a golden is a Rust path), '**/Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'Makefile' and '.github/workflows/**']`,
so no filter edit is needed and no gate silently stops running `[derived → AC3]`.

The comment-reference gate changes behaviour once crate directories exist, and it was checked before
specifying anything: its repository-path class already lists the crate
directory among the top-level names, and its root-name set is a fixed literal that the root manifest's
file name does not join
`[measured baabbb6:ai-docs/scripts/comment_refs.py:42-54 · sed -n '42,54p' ai-docs/scripts/comment_refs.py → GATED_ROOT_NAMES is the literal set {"Makefile", ".gitignore"}, and REPO_TOP_DIRS already lists "crates"]`;
its crate-symbol class becomes live, and **under D1 the name set it reads is not what a reader would
guess from the directory layout.** The gate's own functions were executed against a D1-shaped tree
rather than read: `workspace_crates` adds the manifest's `name` field *and* the directory entry name,
unconditionally and with hyphens folded to underscores, while `own_crate` returns the **directory**
segment alone
`[measured 0b2bc15:ai-docs/scripts/comment_refs.py:98-126 · PYTHONPATH=ai-docs/scripts python3 -c "import comment_refs as cr; print(sorted(cr.workspace_crates('tmp/wsprobe')))" over a scratch tree whose crates/core and crates/cli manifests are named reader-core and reader-cli → ['cli', 'core', 'reader_cli', 'reader_core']]`.

The consequences below follow, and each was run through `classify` rather than derived from the source
`[measured 0b2bc15:ai-docs/scripts/comment_refs.py:122-158 · python3 tmp/probe-own-crate.py, which calls cr.own_crate and cr.classify on constructed comments against that name set → the instrument was seen in both directions: "//! core::Engine" inside crates/core/src/lib.rs returned no finding (own-crate exemption fires) while "//! reader_shared::Protocol" inside crates/cli/src/main.rs returned crate-symbol (the class is live)]`:

- **The own-crate exemption keys on the directory name, so the package name is NOT exempt inside its
  own crate.** A comment in `crates/core/src/lib.rs` writing the package-qualified form is reported
  `[measured 0b2bc15:ai-docs/scripts/comment_refs.py:122-158 · the same probe, case "//! reader_core::Engine" in crates/core/src/lib.rs → [('crate-symbol', 'reader_core::Engine')]]`.
  A crate's own contract symbol is therefore written in its **directory-name** form inside its own
  crate, never in its package-name form.
- **The bare directory token `core` is in the name set unconditionally, so `core::` is not writable
  in any gated comment outside `crates/core`** — a standard-library path such as `core::fmt::Debug`
  in a comment in another member, in a shell script or in a workflow is reported
  `[measured 0b2bc15:ai-docs/scripts/comment_refs.py:122-158 · the same probe, case "//! core::fmt::Debug" in crates/shared/src/lib.rs and case "# core::fmt::Debug" in a gated script → [('crate-symbol', 'core::fmt')] for both; the same string inside crates/core/src/lib.rs → no finding]`.
  This revives, inside a commit-blocking gate, exactly the `core`-shadowing class D1 removes from the
  compiler — with the difference that the gate's version is a lexical false positive on a comment,
  not a broken build.

Neither consequence bites this task: the skeleton comments carry no qualified symbol at all, and no
comment already in the tracked gated corpus trips the class under the confirmed D1 name set
`[measured 0b2bc15:ai-docs/scripts/comment_refs.py:122-158 · python3 tmp/probe-gated-corpus.py, which enumerates the tracked gated corpus through git ls-files and cr.GATED_SOURCE_EXTS / cr.GATED_ROOT_NAMES and runs cr.classify over every line of it against the reader-prefixed name set → no crate-symbol finding; the enumeration was asserted non-empty before the verdict was read, and the instrument was seen in both directions on a constructed control — "core::fmt::Debug" with no own crate returned crate-symbol, the same string with own="core" returned nothing]`.
But D12 escalates D1 to a standing rule binding every crate this repository will ever add, so the
constraints are recorded here as **standing constraints on future comments**, inherited by every
later task, and D12's key-decisions row carries them as the decision's consequence. Whether
`comment_refs.py` should bridge directory name to package name — so that the own-crate exemption
covers both forms and the bare role token leaves the set — is a follow-up this task does not take;
it is routed in § Open questions.

Beyond the crate-symbol class the new sources are under the ordinary reference ban: a `//!` comment
may not name a repository path, a markdown file, a section sign, an issue number, a URL or another
member's crate-qualified symbol
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
| 2 | Retire the empty-workspace guard and every live statement of it: delete the manifest guard and its comment block from the build entry point and leave each recipe as the bare gate command; replace the build-and-test blockquote and the tool-inventory sentence that assert the era; correct the ratchet script's two comments that assert this repository's present state, and leave the two Python gate scripts' docstrings standing for the reason spelled out in (c) and (d). **The file-size band comment and its twin are corrected together:** the build entry point's band comment and the code-style reference's § File size paragraph both say the bands are unmeasured *because the tree holds no crate*, and both are rewritten to the same judgement — crates exist after this task, so the trigger they name has fired, but a skeleton is not the real distribution they wait for, so the re-set stays deliberately deferred to the task that has code to measure. The bands themselves do not move. Rewriting one and leaving the other is the live-document divergence § Approach names. **The ratchet script's two present-state sentences are enumerated the same way, and each has its twin in the build-and-test section.** *(a) The tolerance header.* Its clause "In this project nothing has been measured yet … a suite that does not exist has none" `[measured e1f8a4d:.githooks/coverage-ratchet.sh:22-24 · grep -n 'THE TOLERANCE IS ZERO' .githooks/coverage-ratchet.sh → 22, the clause's own first line, with the line above it a bare comment marker; then sed -n '22,24p' .githooks/coverage-ratchet.sh → "THE TOLERANCE IS ZERO, and that is a starting value, not a measurement. In this project nothing has been measured yet: a tolerance is the width of the suite's own run-to-run drift, and a suite that does not exist has none."]` is a **present-state assertion and goes**: subtask 1's commit records a measured value and leaves a workspace behind it. The judgement does not change — the tolerance stays where it is and the drift is still unobserved, because the skeletons carry no test (D8) — so what is rewritten is the *reason*, from "no suite exists" to "no drift series has been run". Its twin is the build-and-test section's tolerance paragraph, which repeats the same clause and then names this header as where the recipe lives `[measured 20aa023:AGENTS.md:174 · sed -n '174p' AGENTS.md → "The tolerance starts at **0.00 pp** … a tolerance is the width of the suite's own run-to-run drift, and a suite that does not exist has none. … The script's header carries the recipe."]`, so the two are rewritten to the same reason or they diverge exactly as the file-size pair would have. *(b) The no-executable-lines branch comment.* Its opening clause "A workspace with no executable lines yet is the state this repository starts in" `[measured 20aa023:.githooks/coverage-ratchet.sh:139-142 · sed -n '139,142p' .githooks/coverage-ratchet.sh → that clause, then "so it is a named skip and not a block. Once a crate carries code, a zero here means the measurement broke, and the suite's own failure would have blocked above."]` is a **present-state assertion and goes** — D8 measures that the binary members' `main` bodies put lines into the summary — while the sentence after it is the **conditional branch and stays**, and after this task its antecedent holds, so it becomes that branch's whole justification. Its twin is the build-and-test state table's row for the same branch, which states what the script does rather than what this repository is `[measured 20aa023:AGENTS.md:168 · sed -n '168p' AGENTS.md → the state-table row whose state reads "The workspace has no executable lines yet" and whose outcome reads "Skipped, loud. Once a crate carries code this cannot happen silently."]` — so by the rule below **that row is not edited**, and the rewritten comment must still say what the row says. ***(c) The dependency-direction gate's docstring closer.*** Its sentence "WHILE THE WORKSPACE IS EMPTY the gate reports a named skip and exits 0. Once a workspace exists, every rule is enforced." `[measured e1f8a4d:ai-docs/scripts/import_guard.py:19-20 · sed -n '19,20p' ai-docs/scripts/import_guard.py → that sentence, wrapping between "Once a" and "workspace exists"]` is **governed end to end by its own `WHILE` / `Once` pair and therefore STAYS**: both halves describe what the gate does under a condition, the skip branch they describe is one D9 keeps, and the second half is the half that now holds. It is not rewritten, and because nothing else in that file asserts this repository's present state the file is **absent from this subtask's file set** — an untouched listed file would read as a missed site. ***(d) The comment-reference gate's `workspace_crates` docstring.*** Its sentence "While no crate exists the set is empty and the crate-symbol class is inert, which is the honest state: there is no symbol of this workspace to point at." `[measured e1f8a4d:ai-docs/scripts/comment_refs.py:101-102 · sed -n '101,102p' ai-docs/scripts/comment_refs.py → that sentence, wrapping between "inert," and "which is the honest state"]` **STAYS for the same reason, and the trailer is part of the same period**: the "which is the honest state" clause justifies the behaviour *under* the leading `While`, it does not assert that this repository is in that state, and the branch it describes — the one that returns an empty set when the crate directory is absent — survives this task untouched. So this file too is **absent from the file set**. Note the wrap in both quotes: a pattern written against either sentence's natural reading spans a line break and matches nothing, which is the encoding failure § Risks records. **What stays, as a rule:** a conditional branch inside a gate script, a docstring sentence governed by its own condition, and a state-table row that describes one, each remains a true statement about what the script does, so none is edited — only a sentence asserting this repository's present state is. The same ruling covers the tool inventory's § Shell guards row for the dependency-direction gate, which ends "A workspace with no binary, or no workspace at all, is a named skip" `[measured e1f8a4d:ai-docs/claude-tools-hierarchy.md:94 · sed -n '94p' ai-docs/claude-tools-hierarchy.md → that row, in § Shell guards and not in § CI]` — that row states the gate's conditional behaviour, so the edit to § CI in the same file stops at § CI. The build entry point is in the comment-reference gated set, so a rewritten comment there may name the manifest's file name (measured safe) but no repository path under a crate directory, and — per § *What the gates will read afterwards* — no `core::` path either. | `Makefile` (guard block and § file-size band comment), `ai-docs/code-style.md` (§ File size), `AGENTS.md`, `ai-docs/claude-tools-hierarchy.md` (§ CI only), `.githooks/coverage-ratchet.sh` | 1 |
| 3 | The cargo ecosystem returns to Dependabot: add the cargo entry at the repository root — its cadence and its open-pull-request limit copied from the sibling `github-actions` entry, its commit prefix the `build` one the tool inventory's Dependabot section fixes, per **D13**, which also records that no gate reads this file's values — and drop the comment that explains the ecosystem's absence. Propagate to the **`/dependabot-pr`** skill's preamble (a skill of its own, not `/triage`) and to the tool inventory's Dependabot section; retire the propagation row itself, whose trigger this pull request discharges and whose claim it falsifies. **Every live sentence that points at that row is inside this subtask's own file set**, so the removal leaves nothing pointing at a deleted row `[measured 20aa023 · grep -rn -i 'propagation table' . with .git, tmp and ai-docs/plans excluded → the configuration file's own comment, the `/dependabot-pr` preamble and the tool inventory's Dependabot section, and no other live site; a constructed control line carrying the same phrase was matched, so the pattern ran]`: the configuration file's comment goes with the comment; the preamble's clause "the `cargo` ecosystem returns with the commit that creates the workspace — see the propagation table" and the tool inventory's "It returns with the commit that creates the workspace, carrying the `build` prefix, and the propagation table binds the two" each describe a restoration **this** pull request performs, so each is rewritten to describe the configuration as it then stands — the cargo and `github-actions` ecosystems both present, each with its own commit prefix — and neither acquires a replacement pointer. The tool inventory's section additionally opens by describing a single configured ecosystem, so that opening is rewritten in the same edit rather than left contradicting the entry added below it. The configuration file is itself in the comment-reference gated set, so any comment left in it obeys the same reference ban as a Rust one — no markdown path, no section sign, no repository path. | `.github/dependabot.yml`, `.claude/skills/dependabot-pr/SKILL.md` (§ preamble), `ai-docs/claude-tools-hierarchy.md` (§ Dependabot), `ai-docs/propagation-groups.md` | 1 |
| 4 | Record the crate-naming rule (D1) as a project key decision, in the page's own shape — decision, why, consequence, source — under § Repository and process, numbered after the last row the page carries. The *consequence* field carries the measured comment-gate constraints the directory/package divergence creates (§ *What the gates will read afterwards*), since they are what a later crate author inherits and what the page exists to stop them re-litigating. **The *source* field is written as backticked prose and never as a markdown link**, which is how the page's rows write a source; where a row does carry a markdown link beside the prose, its target is a page sitting beside `key-decisions.md` in the same directory — which this row's target is not `[measured e1f8a4d:ai-docs/key-decisions.md:45 · grep -n '\*Source:\*.*](' ai-docs/key-decisions.md → the KD-16 row alone, whose linked target is a page in `ai-docs/` beside `key-decisions.md` itself]`. It names the owner's round-2 answer as the authority, and for the durable location of that verbatim answer it names this design **at the path it carries after Step 12** — `ai-docs/plans/done/2026-09-18-workspace-crate-skeletons.design.md` § D1. It writes **no** `ai-docs/plans/<name>.*` path and, above all, not the interview state file's: the spec and the design both `git mv` into `ai-docs/plans/done/` at Step 12 and the state file is moved under `ai-docs/plans/ignored/` and committed as a deletion in the same step, so every pre-retirement path is stale before the pull request opens and resolves in neither the diff nor CI's checkout. Backticked prose is also what keeps the row out of the relative-link gate's reach, which matters because that gate runs only after the retirement — see § Test Design. | `ai-docs/key-decisions.md` | 1 |

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
  encoding — round 1 proved it by missing a site.** The enumeration was built from several simple
  case-insensitive patterns rather than one alternation, across English and Russian and across the
  assertion and the printf spellings
  `[measured baabbb6 · for p in 'first crate' 'while the workspace' 'no workspace' 'nothing is in it' …; do grep -rn -i -- "$p" . ; done → the sites subtasks 2 and 3 name, and no other outside ai-docs/plans and tmp]`,
  and that set still missed the file-size bands, which say *the tree holds no crate* in the bands'
  own vocabulary and in none of those patterns' words. Round 2 found a third vocabulary the same way:
  the **ratchet script's** two present-state sentences say it in the ratchet's words — a tolerance
  with no suite to measure, a workspace with no executable lines — and are reached by none of the
  earlier patterns. **The pattern set subtask 2 re-runs is therefore the union of three vocabularies,
  the guard/era one, the bands' and the ratchet's** — the wording above, plus
  `'measured against this tree'`, `'real distribution'`, `'once crates exist'` for the bands, plus
  `'nothing has been measured'`, `'the state this repository'`, `'suite that does not exist'`,
  `'no executable lines'` and `'tolerance starts at'` for the ratchet — which is what reaches every
  twin subtask 2 names and nothing else live
  `[measured 0b2bc15 · the widened per-pattern sweep with .git, tmp and ai-docs/plans excluded → Makefile:39-41 and ai-docs/code-style.md:90 for the bands' vocabulary, and no other live site; the constructed control string was matched, so the pattern ran]`
  `[measured 20aa023 · for p in 'nothing has been measured' 'the state this repository' 'suite that does not exist' 'no executable lines' 'tolerance starts at'; do grep -rn -i -- "$p" . ; done with the same exclusions → .githooks/coverage-ratchet.sh:23, :24 and :139, and AGENTS.md:168 and :174, and no other live site; a constructed control line was matched for every one of those patterns, so each of them ran]`.
  **And the encoding that hid this pair is a line wrap, not a word choice** — the phrase a reader
  would reach for spans the comment's line break, so it matches nothing in the tree while matching
  its own control, and only the truncated form reaches the site
  `[measured 20aa023 · grep -rn -i 'state this repository starts in' . with the same exclusions → no hit, while the identical pattern matched a constructed control line; the comment breaks between "starts" and "in", and 'the state this repository' is what reaches .githooks/coverage-ratchet.sh:139]`.
  So the re-run's patterns stay short enough to survive a wrap, or run multiline-aware.
  **Round 3 closed the sweep rather than widening it again.** A third independent pass added a
  **fourth** vocabulary this design does not use — the guard's own printf wording and the skip
  vocabulary, rather than the era, the bands or the ratchet — and it returned no live site the
  enumeration was missing: every additional hit is either a site subtasks 2 and 3 already name, or
  text the stay rule and the `.claude/**` carve-out already dispose of, the tool inventory's
  § Shell guards row among them
  `[measured e1f8a4d · for p in 'workspace is empty' 'empty workspace' 'no Cargo.toml' 'skips itself' 'gate skipped' 'until the first'; do grep -rn -i --exclude-dir=.git --exclude-dir=tmp --exclude-dir=plans -- "$p" . ; done, each pattern run against a constructed control file first and matched there → the Makefile guard block, AGENTS.md's build-and-test blockquote and its two ratchet rows, the tool inventory's § CI and § Dependabot paragraphs, the configuration file's comment, the ratchet script's tolerance header and no-executable-lines comment, the two Python docstrings subtask 2 rules on, and otherwise only conditional skip printfs, gate-table rows and instruction-file rule text]`.
  Four vocabularies over three rounds returning one site set is the point at which a fifth pass buys
  nothing, so **subtask 2 re-runs the widened set to confirm its own edits landed, not to search for
  more sites**; a later round that reopens the sweep is repeating work this tag records.
  Mitigation: subtask 2 re-runs the **widened** set after its edits and reads the residue rather than
  its emptiness. A residual hit is correct and stays where it is a hit in a history surface, which
  the propagation rule leaves untouched by design
  `[measured baabbb6:AGENTS.md § Propagation Rule · grep -n 'When the change propagates a' AGENTS.md → 310: step 4 sweeps the user-facing documents and leaves the history surfaces untouched]`,
  and a hit in `.claude/**` that is *rule text about* truncating or skipping gates rather than a
  statement of this repository's workspace state — that corpus stays **in the sweep's scope and out
  of its finding set**, and a hit there is read before it is judged `[derived → AC4]`.
- **Deleting the manifest guard makes a broken tree loud rather than green**, which is the point, but
  it also means a machine without the toolchain now fails the aggregate instead of skipping it. That
  is the intended direction and matches the project's own rule that a gate which could not run is not
  a green tree; no mitigation is wanted, and the change is recorded in the build-and-test section that
  subtask 2 rewrites `[derived → AC3]`.
- **A doc comment is the one place in the new sources that can fail a gate**, through the reference
  ban or through rustdoc's denied warnings. Mitigation: each crate comment states its own remit and
  names no path, no section, no issue number, no URL and no other member's qualified symbol — and,
  per the measured constraints of § *What the gates will read afterwards*, no `core::` path and no
  package-qualified form of its own crate's symbol either; the documentation gate is part of the
  aggregate run that subtask 1 must pass `[derived → AC1]`.
- **The executable names are a spec-fixed contract that a manifest rename would break silently.**
  The corpus writes them in prose — the structure block, the storage page's migration rule, the
  command-line page's task heading, the build-and-deploy rows
  `[measured 0b2bc15 · grep -rn 'reader-' docs/ → `reader-cli` and `reader-migrate` in docs/ARCHITECTURE.md, docs/03-storage.md, docs/09-build-and-deploy.md and docs/11-core-api-and-cli.md; a constructed control string "reader-x" written to a scratch file was matched, so the pattern ran]` —
  but prose is not a gate, and after this task the names live in a manifest field that a refactor can
  rename without any check objecting. Mitigation: § Test Design names the command that reads the built
  target names back out of the workspace metadata, and it is run as part of subtask 1's verification
  `[derived → AC2]`.

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
    printed. **The instrument is checked in its red direction first, and the control is a capture,
    not a source.** Matching the pattern against the pre-change file's text would compare a pattern
    with the *source* while the live check reads a *run's stderr*; the two share a substring, so it
    would pass, but it never exercises the channel under test. The control instead runs the
    pre-change build entry point and reads its output: write it into a scratch directory that holds
    no manifest, invoke one cargo-guarded target there, capture the run, and the skip sentence must
    appear in **that capture**. Only then is the real run's silence evidence about the tree
    `[measured e1f8a4d:Makefile:31 · git show HEAD:Makefile > tmp/redctl/Makefile && make -C tmp/redctl build > tmp/redctl/capture.log 2>&1 → exit 0 and the capture carries "make: no Cargo.toml at the repository root; build skipped", so the control is runnable and the guard fires on a manifest-less directory rather than on the repository root]`
    `[derived → AC3]`. Running it from a scratch directory is what keeps the control honest after
    subtask 1: at the repository root the manifest then exists, so the pre-change entry point would
    not skip there either and the control would come back clean for the wrong reason.
  - *No live statement of the era survives* — re-run the **widened** multi-pattern sweep of § Risks
    (the union of the vocabularies that section names — the guard/era one, the file-size bands', the
    ratchet's and round 3's skip wording), in both languages, and read
    every residual hit rather than its emptiness: a hit in a history surface is correct and stays; a
    hit in `.claude/**` that is rule text about truncating or skipping gates is correct and stays; a
    hit in any other live document or script is a finding `[derived → AC4]`.
  - *The two file-size band sentences agree* — after the edits, read the build entry point's band
    comment and the code-style reference's § File size paragraph **side by side** and confirm they
    state the same judgement about the same trigger, and that neither band value moved. This is the
    scenario the round-1 design had no test for, which is why the divergence was only caught in
    review. The build entry point's half is AC4's; the code-style half is the propagation AC4's edit
    triggers, so the scenario's verdict is *agreement between the two*, not either one alone
    `[derived → AC4]`.
  - *The ratchet script and the build-and-test section agree, in both pairs* — the same shape again,
    for the two sentences subtask 2 enumerates. Read the rewritten tolerance header **side by side**
    with the build-and-test tolerance paragraph and confirm both give the same reason for the
    tolerance staying where it is, and that the recorded tolerance itself did not move; then read the
    rewritten no-executable-lines comment side by side with the state-table row for that branch and
    confirm the comment still says what the row says — the row being deliberately unedited, so a
    disagreement here is the comment's. The verdict of each half is *agreement between the two*,
    never either one alone; what it catches is a rewritten script header whose instruction-file twin
    still carries the retired reason `[derived → AC4]`.
  - *The edited scripts still run* — the shell gate over the changed script, and the regression
    suites that drive the edited modules, since a docstring edit that breaks a module breaks its gate
    `[derived → AC3]`.
- Fixtures: the pre-change build entry point from `git show`, written into a manifest-less scratch
  directory and **run** there, as the red control above; and the widened pattern set of § Risks with
  its constructed control line.

**Subtask 3 — Dependabot.**

- Entry point: the configuration file, and the documents that describe it.
- Scenarios:
  - *The file parses as YAML — a step of its own, run before anything is read off it.* Nothing in
    the tree parses this file: D13 measures that no script names it, and the workflow linter's recipe
    globs the workflow directory, which this path does not join. So a syntax error introduced by the
    edit reaches `master` unchallenged and surfaces as the failed-updater red run the entry's
    original absence existed to prevent — the one failure mode this subtask can ship. The parser is
    run explicitly: `python3 -c 'import yaml; yaml.safe_load(open(".github/dependabot.yml"))'`
    `[measured pyyaml@6.0.3 · python3 -c 'import yaml; print(yaml.__version__)' → 6.0.3, and the same safe_load over the pre-change configuration file → no exception]`.
    **The instrument is checked in its red direction first**, as everywhere else in this design: a
    deliberately mis-indented copy of the entry is written under the scratch directory and the same
    parser run against it, and it must raise before the real file's silence is read as a pass
    `[measured pyyaml@6.0.3 · the same safe_load over a scratch copy whose ecosystem entry is indented one column short → yaml.scanner.ScannerError, "expected <block end>, but found '<block mapping start>'"]`
    `[derived → AC4]`.
  - *The entry names the right ecosystem and only valid keys* — the identifier for Rust is
    `cargo`, and both the commit-prefix and the open-pull-request-limit keys are valid for it
    `[measured docs.github.com@2026-09-18 · WebFetch https://docs.github.com/en/code-security/dependabot/working-with-dependabot/dependabot-options-reference → the Rust identifier is "cargo"; commit-message.prefix and open-pull-requests-limit are listed as valid keys]`.
    With no gate reading this file's values, this half of the check is a read of the options
    reference against the written entry, and it is only meaningful once the parse step above is
    green `[derived → AC4]`.
  - *Each field of the new entry is read back against the source that fixes it — and for the prefix
    that source must not be the rewrite.* The only live document fixing `build` is the tool
    inventory's § Dependabot, which this same subtask rewrites, so comparing the entry against the
    rewritten section compares the edit with itself and passes however wrong the value is. Read the
    written `commit-message.prefix` back against the **pre-change** § Dependabot, taken from git
    (`git show <merge-base>:ai-docs/claude-tools-hierarchy.md`), which the rewrite cannot move; read
    the cadence and the open-pull-request limit back against the sibling `github-actions` entry in
    the configuration file itself, which D13 makes their source, rather than against any prose. A
    mismatch in either direction is a finding — nothing downstream would catch one `[derived → AC4]`.
  - *Both documents describe the configuration as it now stands* — read the **`/dependabot-pr`**
    skill's preamble and the tool inventory's Dependabot section back against the file, and check in
    particular that neither still promises a future restoration and that the inventory's opening no
    longer describes a single configured ecosystem `[derived → AC4]`.
  - *The discharged propagation row is gone and nothing still points at it* — sweep the harness corpus
    for references to that row **after** removing it as well as before, and read the residue rather
    than its emptiness; the sweep's own pattern is checked against a constructed control line first,
    since an empty result is otherwise equally consistent with a pattern that never ran
    `[derived → AC4]`.
- Fixtures: the pre-change tool-inventory § Dependabot from `git show <merge-base>:…`, used as the
  prefix check's independent source; and a constructed control line for the propagation-row sweep.

**Subtask 4 — the key decision.**

- Entry point: the key-decisions page.
- Scenarios:
  - *The row is in the page's own shape* — decision, why, consequence, source, under § Repository and
    process, numbered after the last row the page carries `[derived → AC4]`.
  - *The source field resolves, and it is checked at the moment it can resolve.* The field is
    backticked prose, which CI's relative-link step does not sweep at all: that step matches only the
    bracket-then-parenthesis markdown-link form, never a path inside a code span
    `[measured e1f8a4d:.github/workflows/ci.yml:261-277 · sed -n '261,277p' .github/workflows/ci.yml → the step walks every *.md in the checkout and tests only the targets of a markdown-link regex, skipping http, https, mailto and angle-bracketed placeholders]`,
    so a backticked path is invisible to it and the row's own check is a **read**: the path written
    is the post-Step-12 one, and it names neither the live `ai-docs/plans/` layout nor the interview
    state file. **If a markdown link is added to this row despite the above, its resolution is checked
    after Step 12 has moved the plan documents into `ai-docs/plans/done/` — never inside this
    subtask**, where the target does not exist yet and a check would fail on a correct row. That
    ordering is safe because CI first sees this branch only on the pull request, which opens after the
    retirement
    `[measured e1f8a4d:.github/workflows/ci.yml:3-6 · sed -n '3,6p' .github/workflows/ci.yml → the workflow triggers on push to master and on pull_request to master, and on nothing else, so a feature-branch push with no pull request runs no job]`
    `[derived → AC4]`.
- Fixtures: none.

## Open questions

**Nothing is open. Every question round 1 raised was put to the owner and answered in round 2, and
each is now a design decision carrying the owner's words rather than a question.** They are recorded
below with their disposition so a later reader does not reopen them.

- **Package names — CLOSED.** The corpus names the crates by role at every site that names them and
  never by package; D1 proposed the `reader-` prefix on the measured ground that a member packaged as
  `core` shadows the standard library's `core` in every dependent. The owner answered **"reader-\* у
  всех"**, with the directories fixed in the same exchange. Folded into D1, which now cites the whole
  corpus set the decision rules over — the repository-structure block, the backend module map, the
  build-and-deploy task row and the core-API page's opening row — rather than the structure block
  alone. No spec row is added for it.
- **The propagation table's "same commit" versus the standing rule's "same pull request" — CLOSED.**
  The owner answered **"Снять строку"**: the project follows the AXIOM and the row is retired as
  discharged. Folded into D10; subtask 3 removes the row. The owner wanted no `harness-gaps.md`
  entry, so none is written.
- **No spec row was found to prescribe a mechanism this design would otherwise choose differently.**
  Scope 3 names `[workspace.dependencies]`, but that is the owner's own wording in the issue body as
  the interview state file persists it, anchored as such in the spec — not a spec-side choice of how,
  so no `SPEC-REMIT` tag is raised. AC4's round-1 trailer, which restated a standing rule, was struck
  by the spec amendment and the anchor moved to the owner's answer, so that `SPEC-REMIT` is
  discharged too; the work AC4 implies is unchanged, because the Propagation Rule binds either way.

**One follow-up is routed, not taken.** `comment_refs.py`'s crate-symbol class keys its own-crate
exemption on the **directory** name while putting both the directory token and the package token in
the name set — so under D1 a crate's own package-qualified symbol is reported inside its own crate,
and `core::` is unwritable in any gated comment outside `crates/core` (both measured, § *What the
gates will read afterwards*). Whether the gate should bridge directory name to package name is a
harness change with its own blast radius across every gated comment in the tree; it is out of this
task's contract, it blocks nothing here, and it is the orchestrator's to route — the constraints
themselves are recorded in D12's key-decisions row so no later crate author meets them unwarned.
