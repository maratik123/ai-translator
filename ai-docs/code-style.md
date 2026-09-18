# Code style — ai-translator

The canonical, growing reference behind `AGENTS.md` § *Code Style*. Rules land here through the learning loop (`/improve`); the file is expected to start thin and to grow from measured defects rather than from taste.

**Language.** Code, identifiers and comments are English. The project's own design corpus under `docs/`, the README and commit messages are Russian, following the repository's existing history (`AGENTS.md` § *Language*).

## Source files

Rust, under `crates/<crate>/src/` for code and `crates/<crate>/tests/` for integration tests. A crate is added to the workspace manifest in the same commit that creates it, or no gate sees it. Nothing is published, so the default visibility is private; `pub(crate)` is the next step up and `pub` is reserved for a crate's real surface.

## Linter posture

`cargo clippy --workspace --all-targets -- -D warnings`, strict: the gate denies warnings, so a lint the compiler merely mentions is a hard failure here. A finding is fixed, not silenced.

- `#[allow(<lint>, reason = "…")]` — both parts mandatory: the specific lint and why this site is the exception. A bare `#[allow(...)]` with no reason is a review finding.
- A lint that keeps firing on correct code is a configuration question: change `clippy.toml` in a reviewed diff rather than scattering suppressions.
- The lint set beyond the default is recorded in `clippy.toml` when it exists, with a reason per group. Until the workspace has one, the default set plus denied warnings is the whole posture, and that is a decision rather than an oversight.

## Formatting

`cargo fmt --all` decides every formatting question; `cargo fmt --all --check` is the gate. An editor hook formats a Rust file the moment it is written, so a formatting diff never reaches a review. Formatting is never argued about in a review.

## Comments

A comment says what the thing is and states its contract. It names nothing outside itself that moves when this tree moves — no markdown path, no repository path, no issue number outside the tracking form, no design-section number, no URL, no crate-qualified symbol of this workspace outside the comment's own crate. The rule, its classes and its exemptions are in [`doc-convention.md`](doc-convention.md) § DOC-4, and `make comment-refs` decides the lexical half over `*.rs`, `*.sh`, `*.sql`, `*.yml`, `*.yaml`, `Makefile`, `.gitignore` and `.githooks/**`.

A doc comment opens with a summary sentence about the item, in the third person. A fallible function carries `# Errors`; one that can panic carries `# Panics`; anything `unsafe` carries `# Safety`. `make doc-check` denies rustdoc warnings, so a broken intra-doc link is a failing gate, not a cosmetic one.

## Errors

- A typed error per failing operation, with the source preserved rather than flattened into a string. A caller that must distinguish two failures matches a variant, never a message.
- An error's message names the operation, not the error: "reading the chapter's paragraphs", not "database error".
- A failure that belongs to the domain — a translation the validator refused — is a distinct value from a failure of infrastructure — the model server was unreachable. Conflating them makes a retry policy impossible to write.
- Nothing is discarded silently: `let _ = <Result>` is a finding unless the line above says why the outcome cannot matter.

## Panicking calls

Shipped code returns errors. A panicking macro, `unwrap` or `expect` on a translation request, a pipeline stage, a storage write or anything the server reaches loses a chapter's work in flight and takes the process with it.

A call that genuinely cannot fire carries `PANIC:` and its reason on its own line or the line above — the justification in prose, never a pointer at the index, which would be the outward reference the comment ban forbids — and gets its row in [`panic-index.md`](panic-index.md) in the same commit. `make panic-calls` decides the marker; the index is kept by review. A binary's `main` may exit non-zero on a start-up failure and needs no row.

## Concurrency

Every spawned task answers four questions, and the answers are visible at the spawn site:

1. **How does it stop?** A cancellation token, a closed channel, or the completion of the work it was given.
2. **Who awaits it?** A handle the shutdown path joins — not a detached task nobody owns.
3. **Where does its error go?** Into a channel, a log with the operation named, or a status the engine's event stream carries. Never dropped.
4. **Where does its panic go?** A task that panics must not silently remove a stage from the pipeline; the owner observes the join result.

Beyond that:

- No lock is held across an await point.
- Blocking work — file parsing, a synchronous library — goes through the runtime's blocking escape rather than stalling an async worker.
- A long call is cancellable: the model server, an embedding batch and a database round trip all take the cancellation signal.
- Shared state that must survive a restart is a database row, not a field.

## Database access

- Every query is parameterised. A query built by string concatenation is a finding whatever its inputs.
- A transaction's boundary is stated where it opens: what it covers and what happens on retry.
- A statement that can return many rows is bounded, and the bound is a named constant or a configuration key.
- Migrations are forward-only and belong to the crate that owns them ([`domain-invariants.md`](domain-invariants.md) INV-14).

## Magic numbers and tuning values

A semantic numeric literal gets a named constant whose name describes its **role**. Exempt: `0`, `1`, `-1`, `2`, loop indices and test fixtures.

A **tuning value** is not covered by that rule and naming it does not discharge the obligation: a context budget, a batch size, a top-`k`, a temperature, a timeout, a retry count and a distance floor are configuration keys, and the design says so ([`domain-invariants.md`](domain-invariants.md) INV-5, INV-7).

## Determinism

Segmentation, alignment, prompt rendering and cache-key derivation read no clock, draw from no unseeded random source, and never let the iteration order of an unordered map reach their output. These are the functions whose tests assert exact output.

## Naming and shape

Naming rules, including the `*_unchecked` contract, live in [`rust-api-naming.md`](rust-api-naming.md). Test conventions live in [`rust-test-conventions.md`](rust-test-conventions.md).

## File size

A four-band ladder, counted as raw lines with comments and blanks included:

| Band | Meaning |
|---|---|
| ~500 | reasonable |
| ~800 | plan the split |
| **1200** | **hard limit for a file under a crate's `src/`** |
| **1500** | **hard limit for a file under `tests/`** |

`make file-limits` gates both hard bands. The `src/` band is the wider of the two because a unit test lives in the file it tests, inside its `#[cfg(test)]` module, so one file holds code and its unit tests together. Neither number has been measured against this tree: the crates it holds are skeletons, and a skeleton is not the real distribution the bands wait for. Re-set them from that distribution once the crates carry code, in a commit that says what it measured.

A cohesive medium-sized file is not a defect; one type per file is not a Rust idiom. A split is by responsibility, never by line count.
