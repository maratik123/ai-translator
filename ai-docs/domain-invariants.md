# Domain invariants

The rules that outrank convenience. Each one is enforced by review — the reviewer charters read this page — and several are also checked mechanically; where a gate exists, the row names it. The argument behind a row lives in the corpus under [`docs/`](../docs/ARCHITECTURE.md), which is where it was decided; this page states the rule and what violating it costs.

**How to use this page.** A design touching any of these blocks names the invariant it is working under. A review finds a violation and rejects — it does not weigh it against convenience. A rule that turns out to be wrong is changed here **and** in its source document, in one pull request, never worked around in a call site.

## The cache

**INV-1 — A translation's cache key is derived from the model, the prompt version and the paragraph's text, and from nothing else.** The context version is stored beside the translation, not inside the key. Folding it in re-translates the whole book after every compaction, discarding good translations that are already in the database — which is the defect the current shape exists to prevent (`docs/03-storage.md`).

**INV-2 — A read takes the newest context version available for a key; a write never deletes an older one.** The history of a paragraph's translations under successive contexts is what makes a regression visible at all.

**INV-3 — A change to what the key is derived from is a migration, not an edit.** Every stored row's key was computed by the old rule; the new rule makes them unreachable. A pull request that changes the derivation states what happens to the rows already stored.

## Retrieval and embeddings

**INV-4 — Retrieval takes the top `k`. A distance is a floor against "nothing similar exists", never the selector.** A hard similarity threshold was calibrated against a real book and would have discarded correct neighbours (`docs/04-translation-pipeline.md`).

**INV-5 — The soft distance floor belongs to one embedding model **and** one input prefix.** Changing either shifts the distribution as much as changing the model does, so the floor is recomputed by the measurement harness under `tools/` and never carried across. A constant copied from another model's row is a defect even when it looks conservative.

**INV-6 — The embedding model's input prefix comes from the model's own configuration, and the pooling mode from its metadata.** Setting pooling by hand, or dropping the prefix, makes a good model look broken — one model measured at 54 instead of 98 on cross-language pairs for exactly that reason, and it was nearly rejected on the strength of it (`AGENTS.md` § Что легко сделать неправильно, `docs/05-llm-client.md`).

## The model request

**INV-7 — A translation request sets the parameters that decide output quality explicitly:** the non-thinking chat-template argument, or the model returns reasoning and an empty content field until it hits the token ceiling; and a repeat penalty of exactly 1.0, because penalising repetition attacks the glossary — a character's name and a recurring term are *required* to repeat (`AGENTS.md` § Что легко сделать неправильно).

**INV-8 — A response is validated before it is stored.** The named classes are: empty output, an echo of the source, a preamble such as "Here is the translation", reasoning in place of a translation, glossary violations, words outside the dictionary, and silent softening of the text. Each class is a test case. The class that defeats all of them — fluent, grammatical, plausible nonsense — is an open risk recorded in `docs/04-translation-pipeline.md`, and no check may claim to cover it.

**INV-9 — Gender and coreference are handled by their own pass, not by a prompt sentence.** A character table with gender in the prompt, pronouns marked in a pre-pass, a morphological check over the target text. Dropping any leg of that is a defect against the project's main systematic error (`docs/10-gender-and-coreference.md`).

## Measurement

**INV-10 — An eval or a model comparison runs at temperature zero, with the draft head off and without expert offload.** The server is non-deterministic under sampling even with a fixed seed; the draft head changes the batch shape and with it the summation order; offload to host memory adds a spread that depends on the state of RAM. A figure produced outside those conditions is not a figure, and reporting one is the defect — not the number it produced (`docs/05-llm-client.md`, `docs/11-core-api-and-cli.md`).

**INV-11 — Performance measurements are repeated and interleaved.** A sequential sweep of several configurations on a machine whose memory is comparable to the models' size carries an order-dependent bias; a single run of each is not a comparison (`AGENTS.md` § Что легко сделать неправильно).

**INV-12 — A test's verdict never depends on what the model returns.** Anything model-backed is an eval, marked as one and held to INV-10; a unit test stands on a recorded fixture.

## Storage and schema

**INV-13 — A paragraph id is stable across re-import.** Every translation, embedding and annotation hangs off it, and the reader's alignment is by paragraph (`docs/02-book-import.md`).

**INV-14 — Migrations are forward-only, applied by the one crate that owns them.** A rename, a re-numbered enum, a changed persisted string or a changed vector dimension is a new migration that states what happens to rows written before it — never a redefinition (`docs/03-storage.md`).

**INV-15 — A database-enforced invariant is tested against a real database.** The suite provisions a `pgvector` container through testcontainers; a mock proves the mock.

## Determinism and secrets

**INV-16 — The pure paths stay pure.** Segmentation, alignment, prompt rendering and cache-key derivation read no clock, draw from no unseeded random source, and do not iterate an unordered map where the order reaches the output. These are the functions whose tests assert exact output, and a hidden non-determinism turns that assertion into a flake.

**INV-17 — A credential never enters a tracked file.** The database connection lives in the environment variable the process reads; a document names the variable, never a value. This is enforced by a `PostToolUse` hook over connection strings that carry a password, and the repository has already had one such line removed from a tracked document. If a real credential reaches a commit, it is rotated — editing it out of the working tree does not unpublish it.

**INV-18 — Model weights and book texts stay out of the repository.** They are local, ignored, and named by path rather than committed.
