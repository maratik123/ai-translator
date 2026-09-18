# Key decisions

Decisions with the reasoning that produced them, so a later reader does not re-litigate a settled trade-off. The full argument for each lives in the corpus under [`docs/`](../docs/ARCHITECTURE.md), which is where a decision is **made**; this page is the index a task reads first, and every row points at its source.

Format: **KD-N — decision** · *why* · *consequence* · *source*.

A decision reached during a task is recorded here in the same pull request that takes it, with the same shape. A row whose source page later changes its mind is rewritten, not left standing beside it.

## Stack

**KD-1 — Rust for the engine, with the engine as a library and every client on top of it.** The translation pipeline — import, context, translation, validation — is the `core` crate; the first client is a command-line tool for batch conversion and quality measurement, the second is the server with the reader. *Consequence:* a feature is designed in the engine and exposed by a client, never implemented inside a client. *Source:* `docs/ARCHITECTURE.md` § Принцип: сначала движок.

**KD-2 — `llama-server` from llama.cpp directly, not Ollama.** The project needs fine control over expert offload, prefix-cache behaviour, slots and metrics. Ollama runs the same engine underneath but does not expose the offload controls; it stays a fallback reachable over the same OpenAI-compatible API. *Consequence:* the model process is started and configured outside the application, and the client speaks plain HTTP to it. *Source:* `docs/05-llm-client.md`, `docs/ARCHITECTURE.md` § Ключевые решения.

**KD-3 — ROCm for the main model, with Vulkan built alongside.** The decision is capacity, not speed: ROCm leaves enough VRAM for the model to fit whole, and the offload to host memory disappears with it. Under Vulkan the offload stays and the wall-clock time swings with the state of RAM. *Consequence:* `-dev` is always explicit, because a build with both backends shows one card twice and llama.cpp would otherwise split the model across "two" devices. *Source:* `docs/05-llm-client.md`, `AGENTS.md` § Запуск моделей.

**KD-4 — PostgreSQL with pgvector as the one store.** Books, paragraphs, translations, context versions and embeddings live in one database; it is already on the machine, it holds the glossary as `jsonb`, and it keeps the backend's state outside the process. *Consequence:* a restart of the backend loses nothing but in-flight work, and a test that asserts a database-enforced invariant runs against a real server. *Source:* `docs/03-storage.md`.

**KD-5 — One WebSocket with typed messages, HTTP only for bulk payloads.** The reader's position flows one way and translations the other, so a bidirectional stream with a request id for the few call-response cases is the whole transport; uploads, images and static assets stay on HTTP, where large binary payloads belong. *Consequence:* there is no REST surface to keep in sync, and the protocol types are generated from one crate. *Source:* `docs/06-websocket-server.md`, `docs/01-shared-protocol.md`.

**KD-6 — The contract is a crate, and the frontend's types are generated from it.** Domain and protocol types live in `shared`, and the TypeScript side is derived from them rather than written twice. *Consequence:* a drifted data-transfer object is a build failure, not a runtime surprise. *Source:* `docs/01-shared-protocol.md`.

**KD-7 — The unit of translation is a paragraph with a stable id.** Alignment is by paragraph, not by page. *Consequence:* every cached translation, embedding and annotation hangs off that id, and a re-import must preserve it. *Source:* `docs/02-book-import.md`, `docs/ARCHITECTURE.md` § Ключевые решения.

## The pipeline

**KD-8 — The context version is stored beside a translation and is not part of its cache key.** The first shape folded it in, which meant that after every compaction the key of an already-translated paragraph stopped matching and the paragraph was translated again, although a good translation was sitting in the database. *Consequence:* the key is derived from the model, the prompt version and the paragraph's text; a read takes the newest context version available for that key. *Source:* `docs/03-storage.md` § Решение: `context_version` вне ключа кэша.

**KD-9 — Retrieval takes the top `k`; a distance is only the floor that says "nothing similar exists".** A hard similarity threshold was calibrated against a real book and would have discarded correct neighbours. *Consequence:* the soft cut-off that remains is a property of the embedding model **and of its input prefix** — it is recomputed by the measurement harness when either changes, and never carried across models. *Source:* `docs/04-translation-pipeline.md`.

**KD-10 — A translation is validated, never merely received.** The response is held against a set of named failure classes — empty output, an echo of the source, a preamble, reasoning instead of a translation, glossary violations, words outside the dictionary, silent softening. *Consequence:* each class is a test case, and the one class no check catches — fluent, grammatical, plausible nonsense — is recorded as an open risk rather than papered over. *Source:* `docs/04-translation-pipeline.md`.

**KD-11 — Gender and coreference get their own pass.** Verb and adjective gender, especially in the first person, is the systematic error of translation into Russian, and each source language brings its own traps. *Consequence:* a character table with gender goes into the prompt, pronouns are marked in a pre-pass, and the target text is checked morphologically afterwards. *Source:* `docs/10-gender-and-coreference.md`.

**KD-12 — The embedding model is `bge-m3`, on the CPU.** It costs no VRAM and a whole book's embeddings take seconds. The comparison that chose it is reproducible and its harness lives in the repository. *Consequence:* the input prefix is whatever that model's own configuration states — for `bge-m3`, none — and the pooling mode is read from the model's metadata rather than set by hand. *Source:* `docs/05-llm-client.md`, `tools/emb-eval`.

**KD-13 — Speculative decoding is the draft head only, and it is off during measurement.** The head buys a measured speed-up at a small VRAM cost, but its output is not identical to a run without it: a different batch shape changes the summation order, and argmax picks differently where probabilities are close. *Consequence:* an eval or a model comparison runs at temperature zero, with the draft head off and without expert offload; a figure from any other run is not comparable. *Source:* `docs/05-llm-client.md`, `docs/11-core-api-and-cli.md`.

## Repository and process

**KD-14 — `master` is protected and every change arrives by pull request.** Approvals are not required, the pull request is. Merge is by merge commit; squash and rebase are disabled for the branch, and a merged branch is deleted automatically. *Consequence:* a flow never pushes to `master`, and the hooks in `.claude/settings.json` refuse a commit made on it. *Source:* `AGENTS.md` § Работа с репозиторием.

**KD-15 — Migrations are forward-only and applied by one crate.** Persisted data outlives every deployment, so a rename or a re-numbered enum is a new migration rather than a redefinition, and one crate is the only place that applies them. *Consequence:* a schema change ships with the paragraph that says what happens to rows written before it. *Source:* `docs/03-storage.md`, `AGENTS.md` § API Stability.

**KD-16 — The test suite provisions its own database.** A database-backed test starts a `pgvector` container through testcontainers over the Podman socket and drops it on cleanup; the application's own connection string is never the suite's. *Consequence:* a test asserting a database-enforced invariant runs against a real server, and a machine without a container runtime is told so loudly rather than passing silently. *Source:* `AGENTS.md` § Локальное окружение, [`rust-test-conventions.md`](rust-test-conventions.md).

**KD-17 — Production model weights and book texts are not in the repository.** They are local files, ignored by git. Miniature models for the test suite are the deliberate exception: they are committed under `testdata/models/` with their licences and provenance, laid out as on Hugging Face, because a suite that downloads its own subject is a suite that needs the network to be honest. *Consequence:* a fixture that needs a production model either ships a small excerpt or is an eval that names what it ran against; a test that needs a live model server gets the committed miniature one. *Source:* `AGENTS.md`, `.gitignore`.

**KD-18 — The model client is tested against a real `llama-server`, not against a mock alone.** Three layers: recorded SSE fixtures for parsing, a stub HTTP server for what the client sends, and a conformance test that starts `llama.cpp:server` on CPU with the committed miniature models. The third one exists because `chat_template_kwargs`, `repeat_penalty`, `min_p` and `top_k` are llama.cpp extensions to the OpenAI schema, so "the server accepts them" is a claim about one build; a mock encodes the belief under test and therefore passes in exactly the runs where that belief is wrong, and a recorded fixture, while honest about what the server once sent, is untouched by an upgrade. *Consequence:* a container runtime is not the database's business alone, and the conformance test asserts shape, status codes and schema — never the content of a reply. What it cannot prove is that a parameter was **applied** rather than merely accepted: whether `enable_thinking: false` suppresses reasoning is unprovable on a model with no reasoning mode and stays with `eval`. *Source:* `docs/05-llm-client.md` § Тестирование клиента.
