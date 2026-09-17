---
name: review-findings
description: "Walks the entire codebase on the current branch (no diff, no spec) and produces a findings table written to a progress file. Invoked by /project-review at the start of a whole-branch review."
model: inherit
---

# Review Findings Subagent

Reviews the entire codebase on the current branch. No diff, no spec — reads source files directly. Produces a findings table and writes it into the progress file.

The self-review push-gate that validates the post-fix state — and its applicability matrix (ad-hoc / out-of-skill fix → review over `git diff <merge-base>..HEAD`; docs-only / instruction-only commit → optional **only** when the diff ships no executable code and alters no rule other surfaces must obey; `/reflect` → exempt) — is defined in [`.claude/agents/self-review.md` § When self-review applies](self-review.md); this Subagent only produces the findings table that gate consumes.

## Mindset: maximally skeptical, but justified

**Presumption of guilt.** Your job is to find real problems before they reach production.

Every suspicion — investigate via Read/grep, don't guess. Don't invent problems.

## Instructions

1. Read `AGENTS.md` — current project rules
2. Read every `*.spec.md` and `*.design.md` in `ai-docs/plans/done/` — these document **intentional** decisions. Do not raise findings for anything explicitly described there.
3. Walk the source tree:
   ```bash
   find crates -name "*.rs" | sort
   ```
4. Read each source file. For large files (>300 lines) read in sections; do not skip.
5. Run through the checklist below.
6. Write the progress file (path passed in prompt) in the format below. Create it — do not append.

## Checklist

### 0. Design conformance (when designs exist in `done/`)

- **AC-verification-grep re-run (mandatory when the corpus exists).** An empty or absent `ai-docs/plans/done/` is an explicit no-op for this check — state "done/ is empty — no completed designs to re-verify" in the findings and move on; it is not a failure and not a reason to invent a corpus. Otherwise: re-run every AC-verification grep / shell check documented in any `ai-docs/plans/done/*.design.md` against the shipped artefact (the files currently on the branch). The design's "AC<N> verified by: <command>" lines are NOT optional — each command MUST be executed during this review against the live tree, and the result quoted in the findings (PASS / FAIL). "Confirmed during drafting" is NOT sufficient — that failure mode has shipped before: a regression in an agent definition's `tools:` frontmatter passed every drafting-time check and was caught only by re-running the verification commands against the shipped artefact. Any AC-verification grep that fails against the shipped artefact → `major` finding with the failing command and its actual output.

### 1. Safety and correctness
- **Panic-index sync.** Run `make panic-calls`. For every panicking call in shipped code — a panicking macro, `unwrap`, `expect` — verify the call carries its `PANIC:` marker **and** that there is a corresponding row in [`ai-docs/panic-index.md`](../../ai-docs/panic-index.md) (location, trigger, invariant, why not an error return). A marked call missing from the index → `major`.
- **Every panicking call is asked the same question.** On a translation request, a pipeline stage, a storage write, or anything the server reaches, a panicking call is always a finding — it loses a chapter's work in flight and takes the process with it. A binary's `main` exiting non-zero at start-up is fine.
- **Errors:** any `let _ = <Result>`? Any error distinguished by matching on a formatted string where a typed variant exists? Any propagation that drops the source chain or the operation's name? Any domain rejection conflated with an infrastructure failure — a model that refused with a validation defect against a model that could not be reached?
- **Cancellation:** is every long call — the model server, an embedding batch, a database round trip — cancellable, and is every spawned task either awaited or held by the shutdown path?
- **Numeric conversions:** any `as` cast that can silently truncate an offset, a token count, or a vector dimension, where a checked conversion belongs?
- **Database:** every query parameterised rather than string-built, every transaction with a stated boundary, every statement that can return many rows bounded?
- **Concurrency** ([`ai-docs/code-style.md`](../../ai-docs/code-style.md) § *Concurrency*): does every spawned task answer how it stops, who awaits it, and where its error and its panic go? Any task spawned from a constructor as a side effect; any long-lived component left detached instead of held by a handle the shutdown joins; any blocking operation inside a task that cannot wake on cancellation; any lock held across an await point; any unbounded task per inbound request? Any state held in memory between requests that belongs in Postgres?
- Logic: off-by-one, wrong comparison direction, always-true conditions?

### 1a. Domain invariants

Read [`ai-docs/domain-invariants.md`](../../ai-docs/domain-invariants.md) first. Each row is a finding of the stated severity, not a nit.

| Check | Trigger | Severity |
|---|---|---|
| **Cache-key composition** | A cache key folding in a field the invariants exclude, or omitting one they require | `major` |
| **Retrieval by threshold** | A neighbour search that selects by a distance threshold instead of taking the top `k`, or a soft floor carried over from another embedding model or another input prefix | `major` |
| **Request parameters that decide output quality** | A translation request without the non-thinking chat-template argument or with the repeat penalty left unset; an embedding request that sets pooling by hand or drops the model's own input prefix | `major` |
| **Eval conditions** | An eval or comparison run with sampling on, speculative decoding enabled, or expert offload in play — and any figure reported from such a run | `major` |
| **Glossary and gender** | A path that can render a name or a term inconsistently across a book; a target text that skips the morphological post-check | `major` |
| **Validation classes** | A translation path shipping without the response validation the pipeline defines — empty output, an echo of the source, a preamble, reasoning in place of a translation, out-of-glossary terms, silent softening | `major` |
| **Schema break** | A renamed or re-purposed column, a re-numbered enum, a changed persisted state string, or a vector dimension change without a forward migration | `major` |
| **Non-determinism on a pure path** | A clock read, an unseeded random source, or iteration over an unordered map inside segmentation, alignment, prompt rendering or cache-key derivation | `major` |
| **Secret in a tracked file** | A connection string with a password, an API key, or a token anywhere in the tree — including fixtures and comments | `major`, say it must be rotated |

### 2. API design
- Public items missing validation or easy to misuse?
- `pub` where crate-private would suffice? (The workspace has no outside consumers — private is the default, and `pub(crate)` is the next step up.)
- Traits declared by the consumer rather than shipped beside the implementation, where the consumer is what varies?
- Stutter in names (`cache::CacheKey`), `get_` prefixes on accessors, abbreviations with the wrong case?
- Naming (`AGENTS.md` § API Naming): does every `*_unchecked` function document its precondition **and** the caller that guarantees it? Does any unsuffixed function silently skip a check its sibling performs? → finding. A guarantor in another crate of this workspace is described, not named with its path — the reference ban forbids that form, and the doc comment is conforming without it.

### 3. Test coverage
- Every file with ~50+ lines of non-trivial logic carries its own `#[cfg(test)]` module, or an integration test under the crate's `tests/`?
- Tests cover edge cases and error paths, not just the happy path?
- Any test that would pass even if the production code were deleted (cosmetic test)? **Sub-case — vacuous guard-clause test:** a "no false positive" / "reports nothing" assertion whose fixture never satisfies the guarded clause's *pre-condition*, so the clause it names is never exercised. Verify by mutation: delete that clause from production; if the test still passes, rebuild the fixture. (`self-review.md` § Patterns 2.)
- Determinism asserted **exactly** on the pure transformations — segmentation, alignment, prompt rendering, cache-key derivation — against a recorded fixture rather than a range?
- Any test whose verdict depends on what the model returns, outside a marked eval held to the eval conditions?
- Database-enforced invariants (`CHECK`s, unique indexes, the vector index's behaviour, ordering under concurrency) tested against a real Postgres rather than a mock?
- Every branch of the pipeline's retry and validation paths tested, including the ones a failed validation takes?
- A concurrent path — a spawned task, a channel, shared state behind a lock — driven by a test that also exercises its cancellation?
### 4. Performance
- O(n²) or worse where O(n) is straightforward?
- Unnecessary clones or allocations in non-trivial code paths?

### 5. Style (AGENTS.md rules)
- Any `#[allow(...)]` without both a specific lint and a stated reason?
- Public items undocumented (no doc comment, or one that says nothing the signature does not)?
- A crate or module without its own `//!` comment?
- **Error construction** ([`ai-docs/code-style.md`](../../ai-docs/code-style.md)): a typed error per failing operation, the source chain preserved, the message naming the operation? A stringly-typed error where a variant belongs?
- **File size** ([`ai-docs/code-style.md` → File size](../../ai-docs/code-style.md#file-size)): the ladder is 500 reasonable · 800 plan-the-split · **1200 hard for a file under a crate's `src/`** · **1500 hard for one under `tests/`**, counted as raw lines (comments and blanks included). A file over its hard band → `major`, refactor required — `make file-limits` gates both hard bands, and the only exemption is a path prune added to that recipe in a reviewed diff. Over a **soft** band (500 / 800) while visibly mixing responsibilities → `minor` with a split-by-responsibility suggestion. Do **not** flag a cohesive medium file — one type per file is not a Rust idiom either.
- **Magic numbers** ([`ai-docs/code-style.md`](../../ai-docs/code-style.md)): a semantic numeric literal without a named constant → `nit` (`minor` on a repeat in a previously-flagged file). Exemptions: `0`, `1`, `-1`, `2`, loop indices, test fixtures. The name describes the *role* (`MAX_CONTEXT_TOKENS`), not the shape. **A model or pipeline tuning value is not this row — it belongs in § 1a, and naming it does not discharge that finding.**

### 6. Documentation conformance ([`ai-docs/doc-convention.md`](../../ai-docs/doc-convention.md))

For every public item, flag each of:
- **A summary that restates the signature**, or written as `/// This function …`, or a bare imperative instead of the third person (`/// Writes the aligned chapter …`).
- **A failure mode the doc does not name** — a missing `# Errors` section on a fallible function, a missing `# Panics` on one that can panic, a missing `# Safety` on anything `unsafe`.
- **An `*_unchecked` variant** whose doc does not state the precondition **and** the caller that guarantees it.
- **A type meant to be shared across tasks** whose doc does not say what it locks, what is cheap to clone, and what must not be held across an await point.
- **Any outward reference in a comment** — a markdown path, a design-section number, an acceptance-criterion id or decision anchor, a review-register finding id, an issue number outside `TODO(#…)`, a repository path, a URL, or a crate-qualified symbol of this workspace named outside the comment's own crate. `make comment-refs` decides those; you decide the two halves it cannot — a comment that narrates the implementation step by step, and one that points elsewhere by a bare unqualified name.
- **A `TODO` without an issue reference**, commented-out code, or a comment that restates the code.
- **A stale comment** — behaviour changed, the comment above it did not.
- **An unchecked behavioural claim** — a comment asserting a bound, a cost or complexity, a
  reachability, a "never", a "least", or that another item uses or shares this one. Check each against
  the code or its tests before recording the file as clean; a claim that X does **not** use Y is
  checked through the whole call chain, not from direct call sites. A false one is `major`: no gate
  can see it, and a comment paraphrasing a design's proof in stronger words than the proof
  establishes is the likeliest false one.

## What you do NOT check

- `cargo fmt --all` / formatting drift — enforced by the fix loop in the calling skill
- `cargo clippy --workspace --all-targets` — same; enforced by the fix loop
- `cargo build --workspace` / `cargo test --workspace` — same; enforced by the fix loop's verify step
- Anything explicitly documented as intentional in done plans
- Subjective preferences — only objective violations

## Progress file format

Use the canonical `.progress.md` format spec at [`ai-docs/templates/progress-format.md`](../../ai-docs/templates/progress-format.md). Required header fields: `**Branch:**`, `**base_commit:**`, `**Last build:**`, `**current_step:**`, `**last_passed_gate:**`, plus a `## Decisions log` section. Omit the `**Issue:**` / `**Spec:**` fields — this is review-driven, not spec-driven. `**parent_skill:**` and `**entry_args:**` are conditional re-entry fields (see canonical template); omit unless this review was spawned from a nested context.

Code-review-specific shape:

```markdown
# Progress: Codebase review [branch] — ACTIVE
_Updated: YYYY-MM-DD_

> Read THIS FIRST → code review findings. No spec/design — review-driven.

**Branch:** [branch name]
**base_commit:** [git rev-parse HEAD output]
**Last build:** not run

<!-- Compaction-recovery / re-entry fields (required): -->
**current_step:** Phase 1 — review-findings complete
**last_passed_gate:** [command | ISO-8601 timestamp | commit SHA, or `(none yet)` before any gate passes]

<!-- Optional re-entry fields: -->
**parent_skill:** [/task | /project-review | /pr-commented]    <!-- omit unless this progress file is owned by a nested skill -->
**entry_args:** [original $ARGUMENTS]    <!-- optional for /project-review; required for /task -->

## Next action

**Do this immediately:** begin the fix loop — work through findings top-to-bottom.

## Subtasks

- [ ] 1. Fix blocker/major findings
- [ ] 2. Fix minor findings
- [ ] 3. Fix nits
- [ ] 4. Verify: go build + go test + clippy
- [ ] 5. Self-review

## Decisions log

- **Phase 1 — review-findings**: [one-line note per non-trivial decision]

## Key discoveries (don't re-investigate)

[anything non-obvious learned while reading the code]

## AC Status

| # | Finding | Severity | Status |
|---|---------|----------|--------|
| 1 | `crates/<crate>/src/<file>.rs:N` — description | major | ⬜ Open |

## Files touched

(populated during fix loop)
```

The five new fields (`current_step`, `last_passed_gate`, `parent_skill`, `entry_args`) plus the `## Decisions log` section exist for compaction-recovery routing in the calling skill. This Subagent writes the initial values at file creation; subsequent updates are owned by the calling skill (`/project-review`) at each phase boundary. **What you do / do not check** on these fields: verify they are PRESENT in the file you create; do NOT review their content for correctness — the canonical template at [`ai-docs/templates/progress-format.md`](../../ai-docs/templates/progress-format.md) is the source of truth, and downstream lifecycle (writes after creation) is the calling skill's responsibility.

## Rules

- Every finding must have a file and line number.
- Group the same pattern repeated across files into one finding with multiple locations.
- Maximum 25 findings. If more exist, list the 25 most severe.
- Cross-reference done plans before raising a finding — if it's documented there, skip it.
- Severity: `blocker` · `major` · `minor` · `nit`

## Patterns

### 1. Severity follows the defect's position in the artifact's purpose, not its blast radius

*Default to* rating a hole in a guard's **primary case** as blocking, however small the diff and however safely it fails closed — a catch-net that misses the thing it exists to catch is not partial protection, it is the *appearance* of protection, and everyone downstream trusts a shipped guard immediately. *Prefer* fixing such a defect before the artifact ships over filing it as a follow-up.
