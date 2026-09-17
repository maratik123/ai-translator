# Project context — ai-translator

## Purpose

A **book reader with a parallel translation produced by a local model**: the original on the left, the translation on the right, aligned paragraph by paragraph. Translation runs ahead of the reader's position in the background, and the model accumulates and periodically compacts the book's context — plot summary, glossary of names and terms, style. Source languages: English, French, Spanish, Italian, German; target: Russian.

The canonical specification is the corpus under [`docs/`](../docs/ARCHITECTURE.md) — **Russian**, and the source of truth for anything non-obvious about import, storage, the pipeline, the model client, the protocol, the reader, or deployment. This page is the short orientation; those documents decide.

## The positioning that decides ties

> **The value is the translation-quality pipeline, not the reader.** Showing a translation next to the original is a solved problem with several existing tools. When two designs compete, the one that makes a translation more consistent across a whole book wins over the one that makes the interface nicer.

## Load-bearing invariants worth knowing before touching a block

- **The unit of translation is a paragraph with a stable id.** Columns are aligned by paragraph, never by page, and every cached translation, embedding and annotation hangs off that id.
- **The cache key does not carry the context version.** A translation is keyed by the model, the prompt version and the paragraph's text; the context version is stored beside it and read as "the newest one available". Folding the context version into the key would re-translate the whole book after every compaction, which is the defect the current shape exists to prevent (`docs/03-storage.md`).
- **Retrieval takes the top `k`, and a distance is only a floor against "nothing similar exists".** A hard similarity threshold was calibrated on a real book and would have discarded correct neighbours; the soft cut-off that remains is a property of the embedding model **and of its input prefix**, recomputed by the measurement harness when either changes, never carried over (`docs/04-translation-pipeline.md`, `docs/05-llm-client.md`).
- **A translation is not accepted because it came back.** The response is held against a set of validation classes — empty output, an echo of the source, a preamble, reasoning in place of a translation, glossary violations, words outside the dictionary, silent softening. One class defeats every one of those checks and is named in the corpus: fluent, grammatical, plausible nonsense (`docs/04-translation-pipeline.md`).
- **Gender and coreference are the systematic error of translation into Russian**, and they are handled by a pre-pass that marks pronouns, a character table with gender in the prompt, and a morphological post-check (`docs/10-gender-and-coreference.md`).
- **A measurement means something only under stated conditions.** Sampling, speculative decoding and expert offload each add their own spread, so an eval or a model comparison runs at temperature zero, with the draft head off and without offload — and a figure from a run outside those conditions is not a figure (`docs/05-llm-client.md`, `docs/11-core-api-and-cli.md`).
- **State lives in Postgres, not in the process.** Books, paragraphs, translations, context versions and embeddings are rows; a restart of the backend loses nothing but in-flight work.

## Architecture

Planned layout, one implementation spec at a time and never by assumption (`docs/ARCHITECTURE.md` § Структура репозитория):

| Crate / component | Responsibility | Doc |
|---|---|---|
| `crates/shared` | Domain and protocol types; the single source of truth for the contract, with TypeScript types generated from it | `docs/01-shared-protocol.md` |
| `crates/core` | The engine: import, storage, translation queue, context, model client, export — a public engine plus an event stream, with no knowledge of transport | `docs/11-core-api-and-cli.md` |
| `crates/cli` | Import, batch translation, bilingual export, the eval set, model comparison | `docs/11-core-api-and-cli.md` |
| `crates/server` | A thin wrapper over the engine: the WebSocket server, HTTP for uploads and images, static assets | `docs/06-websocket-server.md` |
| `crates/migrate` | The one place migrations are applied | `docs/03-storage.md` |
| `frontend` | The library view and the two-column reader | `docs/07-frontend-reader.md`, `docs/08-frontend-ws-client.md` |
| `tools/` | Measurements outside the product — today the embedding-model harness | — |

The model runs as a separate `llama-server` process reached over an OpenAI-compatible HTTP API, with a second instance on the CPU for embeddings; storage is PostgreSQL with pgvector.

## Status

Read it from the repository rather than from this page: the crate layout as the workspace manifest lists it, and the open work as the issue tracker holds it. What this page fixes is the shape, not the progress.

Key decisions with rationale: [`key-decisions.md`](key-decisions.md). Domain invariants that outrank convenience: [`domain-invariants.md`](domain-invariants.md).
