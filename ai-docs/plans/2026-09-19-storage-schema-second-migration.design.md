# Design: Storage schema — the second migration

**Issue:** #11
**Date:** 2026-09-22

## Approach

### What the tree holds today

`reader-core` owns a migrations directory holding one migration, and that migration is the
extension statement
`[measured 183da0c:crates/core/migrations · ls crates/core/migrations → "0001_vector_extension.sql"; cat crates/core/migrations/0001_vector_extension.sql → "CREATE EXTENSION IF NOT EXISTS vector"]`.
The crate exposes that set as an embedded migrator and asserts its lowest-versioned member beside it
`[measured 183da0c:crates/core/src/lib.rs · cat crates/core/src/lib.rs → the public MIGRATOR static over sqlx::migrate!() and a #[cfg(test)] module whose case asserts that the lowest-versioned migration is version 1, described "vector extension", with the extension statement as its exact trimmed text]`.
A database-backed integration target owns its own `main`, starts one container for the whole binary
and hands each trial a freshly created database with that migrator already applied
`[measured 183da0c:crates/core/tests/support/mod.rs · cat crates/core/tests/support/mod.rs → Harness::start over the pgvector image, Harness::create_database which creates a uniquely named database, opens a pool against it and awaits reader_core::MIGRATOR.run(&pool), and Harness::shutdown]`
`[measured 183da0c:crates/core/tests/database.rs · cat crates/core/tests/database.rs → a main that builds a multi-threaded runtime, starts the harness, registers trials through a local make_trial helper, runs them and shuts the harness down]`.
The crate compiles into a library target and that one test target, while the support module inside
`tests/` is no target of its own
`[measured 183da0c · cargo metadata --no-deps --format-version 1 > tmp/design-probe/meta.json; jq -r '.packages[] | select(.name=="reader-core") | .targets[] | "\(.kind[0]) \(.name) \(.src_path)"' tmp/design-probe/meta.json → "lib reader_core …/crates/core/src/lib.rs" and "test database …/crates/core/tests/database.rs", and nothing for tests/support/mod.rs]`.

So everything this task needs in order to *run* already exists: a per-test database, a migrator that
applies whatever the directory holds, and a place to register a trial. What is missing is the schema
itself — no application table exists at this commit, which is also why this migration strands no row
(§ D13).

### What the corpus fixes, and what it leaves to this design

The schema's tables, their columns, the primary keys and the lookups scope row 3 names are **decided** in
`docs/03-storage.md` §§ Схема, Индексы, and this design implements them rather than re-deriving them
`[measured 183da0c:docs/03-storage.md § Схема · sed -n '/^## Схема/,/^## Индексы/p' docs/03-storage.md → the SQL block naming books, chapters, paragraphs, translations, context_snapshots, paragraph_embeddings, paragraph_annotations, positions and settings with their columns, the translations primary key over paragraph_id, cache_key and context_version, and the comment that context_version is not part of cache_key]`.
The cache-key rule behind that primary key is INV-1 and KD-8, and this task changes nothing about
what the key is derived from: the key is an attribute written by a later task, the schema only
declines to fold the context version into it.

Three things the corpus does **not** state, and which are therefore this design's to decide:

- **Nullability.** § Схема writes column names and types and no `NOT NULL` anywhere. The spec's own
  key decision delegates exactly this: structural integrity, so a value a row would be meaningless
  without is required, and no check judges what a value contains (**D2**).
- **What a delete does.** The corpus writes `FK` and no delete rule; the spec's key decision is
  refusal (**D3**).
- **Index names, and the identity of the object that carries positional uniqueness** (**D5**).

One more property is decided by the corpus by *omission* and is honoured as written: § Схема marks
`FK` per column, and the two reference-shaped columns it leaves unmarked — `context_snapshots.upto_paragraph_id`
and `positions.paragraph_id` — stay unmarked here (**D4**).

### The shape this design chooses

One migration file, `crates/core/migrations/0002_storage_schema.sql`, creating every table and every
index in one transaction; the embedded migrator picks it up with no code change, and the existing
harness applies it to every trial's database without being touched. The trials that hold the schema
to the acceptance criteria go into a module of their own under `crates/core/tests/schema/`, reached
by a `mod` declaration from the existing target, because a file placed directly under `tests/`
becomes a test target of its own and a second target means a second container (**D9**).

The schema as this migration writes it — the columns and types are the corpus's, the `Null` column
and the delete rule are this design's:

| Table | Column | Type | Null | Notes |
|---|---|---|---|---|
| `books` | `id` | `bigserial` | no | primary key |
| | `title` | `text` | no | |
| | `author` | `text` | **yes** | an anonymous work has none |
| | `lang_src` | `text` | no | free text (AC12) |
| | `lang_dst` | `text` | no | free text (AC12) |
| | `format` | `text` | no | free text (AC12) |
| | `cover_path` | `text` | **yes** | a book may carry no cover |
| | `added_at` | `timestamptz` | no | written by the caller, no default (**D2**) |
| `chapters` | `id` | `bigserial` | no | primary key |
| | `book_id` | `bigint` | no | references `books (id)` on delete restrict |
| | `idx` | `int` | no | position within the book |
| | `title` | `text` | **yes** | the untitled chapter of an undivided work (AC13) |
| | `css` | `text` | **yes** | a chapter may carry no stylesheet |
| `paragraphs` | `id` | `bigserial` | no | primary key |
| | `chapter_id` | `bigint` | no | references `chapters (id)` on delete restrict (AC13) |
| | `idx` | `int` | no | position within the chapter |
| | `kind` | `text` | no | free text (AC12) |
| | `html` | `text` | no | the segmenter writes both forms for every block |
| | `text` | `text` | no | the payload; the empty string is a value (AC10) |
| | `stable_hash` | `bytea` | no | the re-import identity (INV-13) |
| `translations` | `paragraph_id` | `bigint` | no | references `paragraphs (id)` on delete restrict |
| | `cache_key` | `bytea` | no | no length check — that would judge content |
| | `model` | `text` | no | |
| | `prompt_version` | `int` | no | |
| | `context_version` | `int` | no | an attribute, and part of the key's third column |
| | `text` | `text` | no | |
| | `created_at` | `timestamptz` | no | |
| | — | — | — | primary key `(paragraph_id, cache_key, context_version)` |
| `context_snapshots` | `book_id` | `bigint` | no | references `books (id)` on delete restrict |
| | `version` | `int` | no | |
| | `upto_paragraph_id` | `bigint` | no | a pointer, not a reference (**D4**) |
| | `summary` | `text` | no | |
| | `glossary` | `jsonb` | no | |
| | `model` | `text` | no | |
| | `created_at` | `timestamptz` | no | |
| | — | — | — | primary key `(book_id, version)` |
| `paragraph_embeddings` | `paragraph_id` | `bigint` | no | primary key; references `paragraphs (id)` on delete restrict |
| | `model` | `text` | no | |
| | `embedding` | `vector(1024)` | no | the dimension is schema, not tuning (**D7**) |
| `paragraph_annotations` | `paragraph_id` | `bigint` | no | references `paragraphs (id)` on delete restrict |
| | `context_version` | `int` | no | |
| | `annotated_text` | `text` | no | |
| | `uncertain` | `boolean` | no | no default — the writer states it |
| | — | — | — | primary key `(paragraph_id, context_version)` |
| `positions` | `book_id` | `bigint` | no | primary key; references `books (id)` on delete restrict |
| | `paragraph_id` | `bigint` | no | a pointer, not a reference (**D4**) |
| | `updated_at` | `timestamptz` | no | |
| `settings` | `key` | `text` | no | primary key |
| | `value` | `jsonb` | no | |

And the indexes this migration creates beyond the primary keys' own:

| Name | Table | Key columns | Unique | Serves |
|---|---|---|---|---|
| `chapters_book_id_idx_uniq` | `chapters` | `(book_id, idx)` | yes | AC8's first half |
| `paragraphs_chapter_id_idx_uniq` | `paragraphs` | `(chapter_id, idx)` | yes | AC4's reading order **and** AC8's second half (**D5**) |
| `translations_paragraph_id_idx` | `translations` | `(paragraph_id)` | no | AC4, named by the corpus (**D6**) |
| `context_snapshots_book_id_upto_paragraph_id_idx` | `context_snapshots` | `(book_id, upto_paragraph_id)` | no | AC4's snapshot lookup |

Nothing is created over the embedding column (AC6, **D8**).

### Key decisions

**D1 — One migration file, `0002_storage_schema.sql`, carrying every statement, applied in one
transaction, immutable from the moment it is applied anywhere.** The version and the description are
derived from the file name by the loader, which strips the suffix and turns underscores into spaces,
so this file is version `2` described `storage schema` and the description is never the file name's
own spelling
`[measured sqlx-core@0.9.0 · awk 'NR>=219 && NR<=223 {print NR": "$0}' src/migrate/source.rs → ':219: // remove the `.sql` and replace `_` with ` `' / ':220: let description = parts[1]' / ':221: .trim_end_matches(migration_type.suffix())' / ':222: .replace('_', " ")']`.
A migration that does not open with the transaction opt-out runs inside a transaction together with
its own bookkeeping row, so a schema that fails halfway leaves no half-built database
`[measured sqlx-postgres@0.9.0 · awk 'NR>=231 && NR<=242 {print NR": "$0}' src/migrate.rs → ':231: if migration.no_tx {' … ':239: let mut tx = self.begin().await?;' / ':240: execute_migration(&mut tx, table_name, migration).await?;' / ':241: tx.commit().await?;']`,
and this file does not carry that opt-out.

The checksum is taken over the **whole file text**, and the migrator refuses a version whose recorded
checksum no longer matches
`[measured sqlx-core@0.9.0 · grep -n 'let checksum = checksum_with' src/migrate/source.rs → ':236: let checksum = checksum_with(&sql, &config.ignored_chars);'; awk 'NR>=274 && NR<=275 {print NR": "$0}' src/migrate/migrator.rs → ':274: if migration.checksum != applied_migration.checksum {' / ':275: return Err(MigrateError::VersionMismatch(migration.version));']`.
So an edit to this file after it has been applied anywhere — including a comment tidy — is the
redefinition INV-14 and the `AGENTS.md` § *API Stability* carve-out forbid. **Comments are permitted
in this file** (unlike migration 1, whose exact text an assertion pins), and every one of them is in
the comment-reference gated set by file name (§ *What the gates will read afterwards*).

**D2 — Nullability is the structural rule, and the nullable set is closed and enumerated.** A column
is `NOT NULL` unless absence is a real state of the domain, and the whole nullable set is
`books.author`, `books.cover_path`, `chapters.title` and `chapters.css` — each a thing a real book
may genuinely lack. Everything else is required, which is what AC9 asks for and what the owner's own
framing named when the decision was taken
`[answer 2.1: "Структурная"]`.
The direction is what makes this cheap: the owner's round-2 framing states that tightening later is
possible only if the data already fits, while loosening is always possible — so requiring a column
now is the **reversible** choice and leaving it optional is the irreversible one
`[measured 183da0c:ai-docs/plans/2026-09-19-storage-schema-second-migration.spec.md.state.md · sed -n '/Цена: ужесточить/p' ai-docs/plans/2026-09-19-storage-schema-second-migration.spec.md.state.md → the round-2 question line, carrying "Цена: ужесточить позже можно, только если данные уже подходят, ослабить — всегда. Название главы под NOT NULL не попадает ни в одном варианте."]`.
Two consequences are spelled out because they are where this rule is most likely to be misread:

- **No `CHECK` anywhere, and no length or range constraint.** An empty paragraph text, an empty cache
  key and a negative position are all stored as given (AC10); the validator that refuses an empty
  translation is INV-8's work in code, not the database's.
- **No `DEFAULT` anywhere**, beyond the sequence default that the corpus's own `bigserial` spelling
  carries. The timestamps are written by the caller: a `now()` default would put a clock inside the
  schema, where a repository's own tests could not pin it, and the corpus writes no default.

Two required columns are worth naming because they could be mistaken for optional ones.
`paragraphs.html` is written for every block beside the plain text extracted from it
`[measured 183da0c:docs/02-book-import.md § Задачи · sed -n '/Сегментация/p' docs/02-book-import.md → "Блочные элементы (`p`, `h1..h6`, `li`, `blockquote`, `pre`) становятся абзацами, inline-разметка (`em`, `strong`, `a`) сохраняется в `html`, чистый текст кладется в `text`."]`,
and `paragraphs.stable_hash` is computed for every paragraph — it is what INV-13's stable identity
across a re-import rests on
`[measured 183da0c:docs/02-book-import.md § Задачи · sed -n '/Стабильные id/p' docs/02-book-import.md → "Стабильные id абзацев: `hash(book_id, chapter_index, paragraph_index, text)`; при повторном импорте той же книги переводы переиспользуются."]`.
The edge case both survive is the paragraph that carries only a picture: the corpus gives it a kind of
its own and no text
`[measured 183da0c:docs/02-book-import.md § Крайние случаи · sed -n '/Абзацы без текста/p' docs/02-book-import.md → "Абзацы без текста (только картинка): `kind = image`, перевод не запрашивается."]`,
and AC10 makes the empty string a value the schema stores rather than an absence it refuses.

**D3 — Every reference spells `ON DELETE RESTRICT`; the update rule is left at the default.** The
spec's key decision is refusal
`[answer 1.2: "Запрет"]`,
and refusal has two spellings that are not equivalent. Left at the default, a reference is `NO
ACTION`, which is checked at the end of the statement and can be deferred: measured, a parent row was
deleted and the transaction committed because the children were deleted later in the same
transaction, while the same schema written with `RESTRICT` refused the parent delete on the spot
`[measured pgvector/pgvector@pg18 (PostgreSQL 18.6, Debian build) · psql -U postgres over two table pairs, one referencing with the default rule and one with ON DELETE RESTRICT, both DEFERRABLE INITIALLY DEFERRED → the default pair committed "DELETE 1"/"DELETE 1" and left parents_left = 0, while the RESTRICT pair answered "ERROR: 23001: update or delete on table \"parent_r\" violates RESTRICT setting of foreign key constraint \"child_r_parent_id_fkey\"" and left parents_left = 1]`.
AC11 says the deletion is refused *while* anything still references the book, so the immediate form
is the one that states it. **The refusal's SQLSTATE is `23001`, not `23503`** — a test written
against the foreign-key-violation code would be green for the wrong schema, which is why § Test
Design names the code.

No reference carries a cascading or nulling rule, and the catalogue records the rule per reference as
`confdeltype`, which is what makes AC11's second half checkable schema-wide rather than table by
table
`[measured pgvector/pgvector@pg18 (PostgreSQL 18.6) · psql -U postgres -c "SELECT conrelid::regclass, conname, contype, confdeltype FROM pg_constraint WHERE connamespace = 'public'::regnamespace" → the row "chapters | chapters_book_id_fkey | f | r" for a reference written ON DELETE RESTRICT]`.

**D4 — The two reference-shaped columns the corpus leaves unmarked stay plain `bigint`, and they are
still `NOT NULL`.** § Схема marks `FK` column by column, and writes `upto_paragraph_id bigint` and
`positions(… paragraph_id bigint …)` without it — the same shape in both places, which reads as a
decision rather than an oversight. Making either a reference would extend the refusal net to
*paragraph* deletion, a rule the spec settles only for the book row, and it would do so in a schema
that outlives every deploy. Both columns are required all the same: a snapshot with no window bound
and a reading position with no paragraph are meaningless (**D2**). Neither is a parent link, so AC9's
"a child row with no reference to its parent" is discharged by the `NOT NULL` on the true parent
columns — `context_snapshots.book_id` and `positions.book_id`, both references. Adding a reference
later is a forward migration, and it is the direction that stays open: the data would have to fit,
which it will as long as nothing writes a pointer to a paragraph that does not exist.

**D5 — Positional uniqueness is carried by a unique index, and that same object is AC4's reading-order
index.** A unique btree index over `(chapter_id, idx)` refuses the second paragraph at a taken
position *and* serves "the paragraphs of a chapter in reading order" — measured, the planner reads
that index in order and adds no sort step
`[measured pgvector/pgvector@pg18 (PostgreSQL 18.6) · psql -U postgres, "SET enable_seqscan = off; EXPLAIN (COSTS OFF) SELECT id FROM paragraphs WHERE chapter_id = 1 ORDER BY idx;" against a table carrying a unique index on (chapter_id, idx) → "Index Scan using paragraphs_chapter_idx on paragraphs" / "Index Cond: (chapter_id = 1)" and no Sort node; the same query before the index existed → "Sort" over a disabled "Seq Scan"]`.
One object, named once, is preferred to a constraint plus a second index; the corpus's § Индексы
names this lookup as an index, which is the form chosen. The index names are fixed in § *The shape
this design chooses* because they are persisted identifiers: renaming one later is a migration.

**D6 — `translations_paragraph_id_idx` is created although the primary key's own index already serves
that lookup, and the redundancy is recorded rather than removed.** Measured, the primary key over
`(paragraph_id, cache_key, context_version)` serves both a lookup by paragraph alone and AC3's
lookup by paragraph and cache key with no context version named
`[measured pgvector/pgvector@pg18 (PostgreSQL 18.6) · psql -U postgres, "SET enable_seqscan = off; EXPLAIN (COSTS OFF) …" → "Bitmap Index Scan on translations_pkey / Index Cond: (paragraph_id = 1)" for the paragraph-only lookup, and "Index Only Scan using translations_pkey … Index Cond: ((paragraph_id = 1) AND (cache_key = '\x01'::bytea))" for the cache pre-check]`.
The corpus names the separate index and so does the task, and `docs/` is decisions — so it is
created. It is not pure duplication either: it is the narrow index, without the key's `bytea` column,
which is the one a cache pre-check scans most often. This paragraph exists so that a later reader
meets the fact and the reason together instead of re-opening it.

**D7 — `vector(1024)` is a persisted schema dimension, not a tuning value, and the type enforces it
by itself.** The tuning-value rule sends a batch size, a top-`k` or a temperature to configuration;
a vector dimension is none of those — it is the width of stored data, fixed by the embedding model
the project chose (KD-12, `bge-m3`), and INV-14 names a changed vector dimension as a migration
outright. The column type is the whole enforcement: a value of another dimension is refused by the
server with its own error
`[measured pgvector/pgvector@pg18 (PostgreSQL 18.6, vector 0.8.6) · psql -U postgres against a column declared vector(1024): a 1024-element literal stored and read back with vector_dims = 1024, and "INSERT … VALUES (2, '[1,2,3]')" → "ERROR: 22000: expected 1024 dimensions, not 3", after which the table still held the single row]`.
No `CHECK` is added beside it; that would be the content judgement AC10 rules out, and it would
duplicate the type.

**D8 — No index of any kind is created over the embedding column, and the assertion that says so
reads the access method rather than a name.** AC6 is a claim that nothing of a kind exists, so the
check has to be able to see one: measured, the catalogue reports an index's access method as `hnsw`
or `btree` through `pg_am`, and the row disappears when the index is dropped
`[measured pgvector/pgvector@pg18 (PostgreSQL 18.6, vector 0.8.6) · psql -U postgres, "SELECT c.relname, am.amname FROM pg_index i JOIN pg_class c ON c.oid = i.indexrelid JOIN pg_am am ON am.oid = c.relam WHERE i.indrelid = 'e'::regclass" → "e_embedding_hnsw | hnsw" and "e_pkey | btree" while the hnsw index existed, and only the btree row after DROP INDEX]`.
The corpus and the spec both defer the index until the library grows; this task writes nothing that
would have to be dropped later.

**D9 — The schema trials live in `crates/core/tests/schema/`, reached by a `mod` declaration from the
existing target, and `make_trial` moves into the support module.** Two constraints decide the
placement. A `.rs` file placed directly under `tests/` is discovered as a test target of its own —
**even though this manifest already declares one explicitly**, which is the half that is easy to get
wrong from memory — and a second target with its own `main` would start a second container for the
same run, while a file inside a subdirectory of `tests/` is no target, which the existing support
module demonstrates at this commit (§ *What the tree holds today*)
`[measured cargo@1.98.1 · an empty crates/core/tests/zz_autodiscovery_probe.rs was written, cargo metadata --no-deps --format-version 1 read (it compiles nothing) and the file removed in the same command, with git status --porcelain afterwards showing only this design file → the target list gained "test zz_autodiscovery_probe …/crates/core/tests/zz_autodiscovery_probe.rs" beside the declared "test database", while no target ever appears for the support module inside the subdirectory]`. The second constraint is the file bands, which
count a file whole: the trial set is split by what the trials interrogate so that neither file
carries all of it — `shape.rs` reads the catalogue, `behaviour.rs` drives the database
`[derived → the file-size gate on each of the group's commits]`.
`mod.rs` holds the fixtures and the function that builds this module's trials for `main` to register.
The trial-wrapping helper currently local to the target is needed by both files, so it moves to
`crates/core/tests/support/mod.rs` — harness plumbing in the harness module — rather than being
copied. This is the only change this task makes to an existing test file besides registering the new
trials, and it starts no new crate: KD-20 puts that threshold at a second *crate* needing the
harness, which is not this.

**D10 — AC1 is checked by a catalogue golden, and the fields it covers are named.** The comparison
reads, for every table in the public schema, the tuple (table name, column name, the catalogue's own
formatted type, whether the column is required, the column's default expression), and holds the whole
sorted set against a table written in the test — both directions, so a missing column and an
unexpected extra column each fail. The formatted type is read with the catalogue's type formatter
rather than from `information_schema`, because the latter reports a vector column as a user-defined
type and loses the dimension entirely
`[measured pgvector/pgvector@pg18 (PostgreSQL 18.6, vector 0.8.6) · psql -U postgres over a table with an embedding column → information_schema.columns gave "embedding | USER-DEFINED | vector", while "SELECT format_type(a.atttypid, a.atttypmod) FROM pg_attribute a …" gave "embedding | vector(1024)"]`.
Column *order* is deliberately outside the comparison (the set is sorted before it is compared),
because a column's ordinal position carries nothing any caller reads. Nothing here is model-produced,
so no model or sampling parameters need pinning. **What a diff in this golden means:** the schema and
this design disagree. If the migration has never been applied outside a throwaway test database, the
repair is the migration; if it has been applied anywhere real, the repair is a new forward migration
and never an edit to `0002` (**D1**).

**D11 — The embedded set gains a unit assertion for the new migration, and that assertion is also
what forces the macro to re-expand.** The migrator macro emits an `include_str!` per migration it
found, which is how the compiler learns to watch those files
`[measured sqlx-macros-core@0.9.0 · awk 'NR>=61 && NR<=62 {print NR": "$0}' src/migrate.rs → ':61: // this tells the compiler to watch this path for changes' / ':62: Ok(quote! { include_str!(#path_str) })']`.
A *newly added* file is watched by no such path, so an incremental build can keep an expansion that
never saw it. The existing trial that compares the applied versions against the embedded set cannot
catch that — both sides of its comparison come from the same stale constant. So the `#[cfg(test)]`
module beside the migrator gains a case asserting that the embedded set carries version `2` described
`storage schema`, and editing that file is itself the change that makes the crate rebuild. The
assertion deliberately does not pin the migration's SQL text: the exact-text assertion that guards
migration 1 exists because that file is a single statement, and pinning a whole schema's text would
be a golden that fails on every reformat while proving nothing the catalogue golden does not prove
better.

**D12 — No dependency is added, and no value reaches the database through a crate that would be one.**
`reader-core` declares `sqlx` and, for tests, the runner, the container crates and the runtime
`[measured 183da0c:crates/core/Cargo.toml · cat crates/core/Cargo.toml → [dependencies] sqlx.workspace = true; [dev-dependencies] libtest-mimic, testcontainers, testcontainers-modules, tokio; and the [[test]] section declaring the database target with harness = false]`.
The trials bind `bytea` as a byte slice and reach `jsonb` and `vector` by casting a text literal in
the statement — the form the existing vector trial already uses — so no JSON crate is needed for a
fixture. The queries are runtime queries, not the compile-time-checked macros, so this task needs no
offline query data and does not touch the corpus row that asks for it.

**D13 — The forward migration, what it strands, and what a rollback is.** Nothing precedes this
schema: at this commit the migrations directory holds only the extension statement (§ *What the tree
holds today*), so no application table exists and no row can have been written under an older shape.
There is therefore no dual-read window and no code that must understand two shapes — the repositories
that will read these tables are a later task and are out of this spec's scope. **Rollback**, for the
test loop, is the database dying with its container; for the owner's own database, it is
`pg_dump`-restore, which is the corpus's stated backup, and after that it is a new forward migration
that drops what this one created. There is no down-step and there will not be one: the migration set
is forward-only (INV-14, KD-15).

**D14 — Applying this migration to the owner's database is out of scope, and the binary that will do
it stays empty.** The spec puts that step outside the test loop, and the binary that owns it is a
stub at this commit
`[measured 183da0c:crates/migrate/src/main.rs · cat crates/migrate/src/main.rs → a module doc line and an empty main]`.
This is recorded so the boundary is visible rather than discovered: after this task the schema exists
in the repository and in every test database, and nothing shipped applies it anywhere else.

### What the gates will read afterwards

The comment-reference gate reaches SQL by file name, so every `--` comment this migration carries is
gated exactly as a Rust comment is
`[measured 183da0c:ai-docs/scripts/comment_refs.py:42 · grep -n 'GATED_SOURCE_EXTS' ai-docs/scripts/comment_refs.py → ':42:GATED_SOURCE_EXTS = (".rs", ".sh", ".sql", ".yml", ".yaml")']`
— no markdown path, no section sign, no acceptance-criterion id, no decision anchor, no issue number
outside the tracking form, no repository path and no URL, in the migration or in the new test module
`[derived → the comment-reference gate on each of the group's commits]`.

The panic gate reads tracked non-test Rust sources only
`[measured 183da0c:ai-docs/scripts/check-panic-calls.sh · sed -n '/^Usage:/,/named files/p' ai-docs/scripts/check-panic-calls.sh → "check-panic-calls.sh  every tracked non-test Rust source"]`,
so the trials may `unwrap` where a panic is the intended failing-test outcome, and the index stays
empty because this task adds no shipped Rust line at all — the only source file it edits outside
`tests/` is the `#[cfg(test)]` module beside the migrator (**D11**)
`[derived → the panic gate on each of the group's commits]`.

CI's paths filters already name SQL in both the jobs this task's artefacts reach, so **no filter
entry is added here**
`[measured 183da0c:.github/workflows/ci.yml:41,89 · grep -n "'\*\*/\*\.sql'" .github/workflows/ci.yml → ':41:' under the rust filter and ':89:' under the comment-references filter]`.
The coverage ratchet's staged-set filter names `.sql` too, so every commit of the code group runs the
suite under instrumentation and therefore needs a reachable container socket
`[measured 183da0c:.githooks/coverage-ratchet.sh:96-97 · awk 'NR>=96 && NR<=97 {print NR": "$0}' .githooks/coverage-ratchet.sh → ':96: staged=$(git diff --cached --name-only --diff-filter=ACMR \' / ':97: -- '"'"'*.rs'"'"' '"'"'*.sql'"'"' Cargo.toml Cargo.lock '"'"'**/Cargo.toml'"'"' 2>/dev/null)']`.

No checkbox in `docs/03-storage.md` is closed in full by this task, so no corpus row is ticked: the
schema section is a specification rather than a task row, and the task rows that neighbour it ask for
things this task does not deliver — offline query data with its CI check, the binary that applies
migrations, the engine's schema check, and the repositories
`[measured 183da0c:docs/03-storage.md § Задачи · sed -n '/^## Задачи/,/^## Схема/p' docs/03-storage.md → the unticked rows naming SQLX_OFFLINE with cargo sqlx prepare --check in CI, reader-migrate as the only applier, Engine::open's version-and-checksum check and the repositories, beside the ticked rows for the test harness and the vector extension]`.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | **The migration, and the embedded-set assertion that proves the build saw it.** Write `crates/core/migrations/0002_storage_schema.sql` creating the tables in dependency order (books, chapters, paragraphs, translations, context snapshots, embeddings, annotations, positions, settings) with the columns, types, nullability, primary keys and `ON DELETE RESTRICT` references § *The shape this design chooses* fixes, then the indexes that section's second table names, with those exact index names. No `CHECK`, no `DEFAULT` beyond the `bigserial` sequences, no transaction opt-out line, and no comment carrying an outward reference (**D1**, **D2**, **D3**, **D5**, **D7**, **D8**). Add the case to the `#[cfg(test)]` module beside the migrator asserting that the embedded set carries version 2 described `storage schema`, which is also the edit that makes the macro re-expand over the new file (**D11**); leave the existing lowest-version case untouched. | `crates/core/migrations/0002_storage_schema.sql`, `crates/core/src/lib.rs` | — |
| 2 | **The schema trials.** Move the trial-wrapping helper from the test target into the support module and have the target use it from there (**D9**). Add `crates/core/tests/schema/` — `mod.rs` with the fixtures and the function that builds this module's trials, `shape.rs` with the catalogue trials and `behaviour.rs` with the database-driving trials § Test Design names — and register them from the existing target's `main`. Perform the red observations § Test Design requires before the group's last commit, each with its output recorded in the progress file and its mutation reverted, confirming the revert with `git diff --name-only`. | `crates/core/tests/support/mod.rs`, `crates/core/tests/database.rs`, `crates/core/tests/schema/mod.rs`, `crates/core/tests/schema/shape.rs`, `crates/core/tests/schema/behaviour.rs` | 1 |
| 3 | **Record the schema's integrity posture where the next task will look for it.** Add a key-decision row in the page's own shape — decision, why, consequence, source — numbered after the last row the page carries, stating that the schema enforces structure and nothing about content: required values and positional uniqueness, no check constraint, no default beyond the identifier sequences, every reference refusing a delete rather than cascading it, and the categorical columns holding free text with the supported set living in the code. The *consequence* field carries what a repository author inherits: the caller writes every timestamp, a delete of a book is refused until its content is removed by the caller, the refusal arrives as the restrict SQLSTATE rather than the foreign-key one, and the embedding dimension is schema rather than configuration. The *source* field is backticked prose naming this design at the path it carries after Step 12 — `ai-docs/plans/done/2026-09-19-storage-schema-second-migration.design.md` § D2–D4 and § D7 — because the pre-retirement path is stale before the pull request opens. | `ai-docs/key-decisions.md` | 2 |

## Handoff plan

`M = 3`. Two groups, homogeneous by change-type and minimised: subtasks 1 and 2 both change code
(a migration and Rust), subtask 3 changes a harness document and depends on both, so no dependency
chain forces an interleave and the clustering is already the fewest groups the dependencies allow.
Two groups is within the default maximum of four, so nothing is surfaced to the user for approval.
Every group is at or under the size cap of ten consecutive subtasks, and the terminal group's size is
inside the `1..=10` range.

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md`
  § Compaction recovery (re-entry). The first group takes a handoff exactly as every later one does.
- **Group A** — model `sonnet`, effort `medium` (pinned in the `code-writer` frontmatter; no inline
  `model=` and no effort override, because there is no per-invocation effort parameter), 1M-token
  window, via `subagent_type="code-writer"` — subtasks 1, 2 (code change-type: the `*.sql` migration
  and `*.rs` sources). The ratchet file these commits may carry is written and staged by the
  pre-commit hook rather than authored, so the group stays homogeneous
  `[measured 183da0c:.githooks/coverage-ratchet.sh:165,198 · awk 'NR==165 || NR==198 {print NR": "$0}' .githooks/coverage-ratchet.sh → ':165: git add -- "$RATCHET_FILE"' / ':198: git add -- "$RATCHET_FILE"', each following the write of the rounded measurement]`.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md`
  § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B** — model `inherit` (the orchestrator's), effort inherited from the orchestrator
  (typically xHigh, not pinned), 1M-token window, via `subagent_type="general-purpose"` with no
  inline `model=` — subtask 3 (instructions/harness change-type: `ai-docs/**`). Terminal group
  (1 subtask; within the `1..=10` range).

## Risks

- **An edit to the migration after it has been applied anywhere is a data-corruption bug wearing a
  refactor's clothes.** Mitigation: **D1** states the rule in the design where a later author looks,
  subtask 1's unit case pins the version and description so a rename fails the suite, and the
  catalogue golden's diff instruction (**D10**) names the forward migration as the repair rather than
  an edit — `[measured sqlx-core@0.9.0 · the checksum and VersionMismatch reads cited in D1 → the checksum is taken over the whole file text and a changed one is refused]`.
- **An incremental build can keep a macro expansion that never saw the new migration**, and the trial
  that compares applied versions against the embedded set would stay green because both sides read
  the same stale constant — `[measured sqlx-macros-core@0.9.0 · the include_str! read cited in D11 → only the files the expansion already found are watched]`.
  Mitigation: subtask 1 edits the file the macro lives in and adds an assertion naming version 2, so
  a stale expansion is a red unit test rather than a silent absence — `[derived → the § Test Design case "the embedded set carries the storage-schema migration"]`.
- **A refusal test written against the wrong SQLSTATE passes for the wrong schema.** A delete refused
  by `ON DELETE RESTRICT` reports `23001`, while the foreign-key-violation code `23503` is what an
  insert against a missing parent reports — `[measured pgvector/pgvector@pg18 (PostgreSQL 18.6) · the psql run cited in D3 → "ERROR: 23001: … violates RESTRICT setting of foreign key constraint"; and psql -U postgres with VERBOSITY verbose, "INSERT INTO chapters (book_id, idx) VALUES (999, 0)" against a table referencing books → "ERROR: 23503: insert or update on table \"chapters\" violates foreign key constraint \"chapters_book_id_fkey\""]`.
  Mitigation: § Test Design names the code each case expects, and the red observation for the delete
  rule flips one reference to a cascade and requires the trial to be seen red.
- **Every "nothing of this kind exists" assertion passes for any schema until it has been seen red** —
  AC6's absent index, AC7's empty tables, AC11's absent cascade and AC12's absent check constraint are
  all of that shape. Mitigation: § *Red observations* makes one mutation per such assertion a required
  part of subtask 2 rather than an optional courtesy — `[derived → the § Test Design section "Red observations required before the group's last commit"]`.
- **A catalogue query can answer for the wrong catalogue.** PostgreSQL 18 records `NOT NULL` as a
  constraint row of its own, so a query that asks "does this table carry any constraint beyond its
  keys" reports rows on a schema with no check constraint at all
  `[measured pgvector/pgvector@pg18 (PostgreSQL 18.6) · psql -U postgres over pg_constraint → rows with contype 'n' named books_title_not_null and chapters_idx_not_null beside the 'p' and 'f' rows, while the same query filtered to contype = 'c' returned no row]`.
  Mitigation: § Test Design pins the check-constraint query to the check contype and says why.
- **From this task on, every commit of the code group needs a reachable container socket**, because
  the ratchet refuses a commit whose suite is not green and the suite provisions a database —
  `[measured 183da0c:.githooks/coverage-ratchet.sh:96-97 · the staged-set read cited in § What the gates will read afterwards → the filter names '*.rs' and '*.sql']`.
  Mitigation: none is built; the ratchet already prints the socket instruction on the branch that
  blocks, so the failure explains itself, and this is the same precondition the harness task
  introduced — `[measured 183da0c:.githooks/coverage-ratchet.sh:124 · grep -n 'socket' .githooks/coverage-ratchet.sh → ":124:  printf '\\nIf no container runtime is reachable, start the Podman socket the suite connects to\\n' >&2"]`.
- **The schema lands with nothing shipped that applies it outside the test loop.** Mitigation: none —
  that is the corpus's own task order and the spec's own scope boundary (**D14**), recorded so it is
  visible rather than discovered.

## Test Design

Everything this section specifies is an artefact this task creates, so its claims carry
`[derived → …]`; the facts about the server that decide *how* a check is written are measured in
§ Approach and cited there. Every trial takes a freshly created and migrated database from the
existing harness, so no trial cleans anything and no trial sees another's rows.

### Beside the code — `crates/core/src/lib.rs`, `#[cfg(test)] mod tests`

- **Location:** the existing `#[cfg(test)]` module, because the claim is about what the repository
  embeds and must stay runnable on a machine with no container runtime.
- **Entry point:** the embedded migrator constant.
- **Case — the embedded set carries the storage-schema migration.** The highest-versioned migration
  is version `2` with the description `storage schema`. The description is derived from the file name
  by the loader (**D1**), so this case fails on a renamed file as well as on a stale expansion — and
  a failure here means one of those two things, never that the assertion should be relaxed
  `[derived → AC1]`.
- **Fixtures:** none. The migrator is a compile-time constant.

### The catalogue trials — `crates/core/tests/schema/shape.rs`

- **Location:** an integration target's module, because the subject is a real server's catalogue
  (`ai-docs/rust-test-conventions.md` § *Where a test lives*, INV-15).
- **Fixtures:** none recorded on disk; the expected sets are case tables written in the test.

- **`schema_matches_the_recorded_shape`** — the golden **D10** specifies: for every table of the
  public schema, the tuple (table, column, formatted type, required, default expression), compared as
  a sorted set against the expected table in both directions. This is the trial that carries AC1's
  "the columns and types the storage document lists", and it is the only one that would notice a
  table created with a right-looking name and a wrong column `[derived → AC1]`.
- **`the_named_lookups_have_an_index_of_their_own`** — for each lookup AC4 names,
  assert that an index exists whose access method is btree and whose **ordered** key-column list is
  the expected one; the order matters, because an index over the same columns in the other order
  exists just as happily and serves neither the reading order nor the snapshot lookup. One
  corroborating assertion is added for the reading-order lookup only, where ordering rather than
  filtering is the claim: with sequential scans disabled in the session, the plan for the chapter's
  paragraphs ordered by position names that index and carries no sort step. A failure of the
  corroborating half means the planner's choice changed and is investigated as such; it never
  licenses weakening the catalogue half `[derived → AC4]`.
- **`the_embedding_column_is_the_declared_dimension`** — the formatted type of the embedding column is
  the 1024-dimension vector type, read with the catalogue's type formatter, because
  `information_schema` cannot see the dimension (**D10**) `[derived → AC5]`.
- **`no_index_over_the_embedding_column_is_hnsw`** — the access methods of every index on the
  embedding table are asserted exactly, and the expected set is the primary key's btree alone. Written
  as an exact set rather than as "no row has amname hnsw", so an index added under another access
  method is caught too `[derived → AC6]`.
- **`the_tables_later_tasks_fill_are_created_and_empty`** — the embedding and annotation tables are
  present in the catalogue and each holds no row after the migration `[derived → AC7]`.
- **`every_reference_refuses_a_delete`** — over every foreign key in the public schema, the recorded
  delete rule is restrict. Schema-wide rather than table by table, so a reference added later by an
  unrelated migration cannot introduce a cascade unnoticed; this is the half of AC11 that says nothing
  disappears as a side effect anywhere `[derived → AC11]`.
- **`the_schema_carries_no_check_constraint`** — the catalogue holds no constraint of the check kind,
  the query filtered to that contype for the reason § Risks gives. This is AC12's and AC10's
  structural half: nothing in the schema judges what a value contains `[derived → AC10 and AC12]`.

### The behaviour trials — `crates/core/tests/schema/behaviour.rs`

- **Fixtures:** helpers in `crates/core/tests/schema/mod.rs` that insert a book, a chapter and a
  paragraph and hand back their identifiers, one that builds a whole book with a row in every
  referencing table, and one that runs a statement expected to fail and returns the server's SQLSTATE
  so a case table can name the code it expects. Values that have no natural Rust binding — the
  glossary object and the embedding vector — are written as text literals cast in the statement
  (**D12**).
- **The codes the cases name**, measured against the image the harness starts rather than recalled:
  a repeated key is `23505`, a missing required value is `23502`, a reference to a row that does not
  exist is `23503`, a delete refused by the restrict rule is `23001`, and a vector of the wrong
  dimension is `22000`
  `[measured pgvector/pgvector@pg18 (PostgreSQL 18.6, vector 0.8.6) · psql -U postgres with \set VERBOSITY verbose over a books/chapters pair → "ERROR: 23503: insert or update on table \"chapters\" violates foreign key constraint", "ERROR: 23502: null value in column \"book_id\" … violates not-null constraint", "ERROR: 23505: duplicate key value violates unique constraint \"chapters_book_id_idx_uniq\""; and the runs cited in D3 and D7 for 23001 and 22000]`.
  A case that asserts only "this failed" would pass on the wrong refusal, which is the failure this
  bullet exists to rule out.

- **`context_versions_of_one_cache_key_coexist`** — two translations of one paragraph sharing a cache
  key and differing in context version are both stored and both readable `[derived → AC2]`.
- **`a_repeated_translation_key_is_refused`** — a third row repeating paragraph, cache key and context
  version is refused with the unique-violation SQLSTATE, and the stored rows are unchanged afterwards
  `[derived → AC2]`.
- **`a_cache_lookup_naming_no_context_version_finds_the_row`** — the corpus's own pre-check shape, a
  lookup by paragraph and cache key with no context version named, returns a row. This is the
  behaviour INV-1 and KD-8 exist for, stated as a test `[derived → AC3]`.
- **`a_vector_of_another_dimension_is_refused`** — a 1024-element vector is stored and reads back at
  that dimension; a vector of another dimension is refused with the data-exception SQLSTATE the server
  reports, and no row lands `[derived → AC5]`.
- **`a_position_is_taken_once_within_its_parent`** — a case table: a second chapter at a position
  already taken in its book, and a second paragraph at a position already taken in its chapter, each
  refused with the unique-violation SQLSTATE; and, as the same table's passing cases, the same
  positions under a *different* parent accepted, which is what makes the constraint "within its
  parent" rather than global `[derived → AC8]`.
- **`a_row_missing_a_required_value_is_refused`** — a case table over the classes AC9 names: a
  paragraph with no text, a chapter with no book, a paragraph with no chapter, a translation with no
  cache key, a snapshot with no window bound, a position with no paragraph — each refused with the
  not-null SQLSTATE. One further case carries the neighbouring failure so the two are not confused: a
  child row naming a parent that does not exist is refused with the foreign-key-violation SQLSTATE
  `[derived → AC9]`.
- **`the_database_judges_presence_not_content`** — a case table of values a content-judging schema
  would reject and this one stores unchanged: a paragraph whose text is the empty string, a negative
  position, an empty cache key, an empty glossary object. Each is read back and compared exactly
  `[derived → AC10]`.
- **`a_value_outside_the_supported_set_is_stored_as_given`** — a case table over the categorical
  columns AC12 names — a paragraph kind, a book format, a source language and a target language — each written
  with a value the project does not support and read back unchanged `[derived → AC12]`.
- **`deleting_a_book_is_refused_while_anything_references_it`** — a whole book is built with a row in
  every referencing table; the delete is refused with the restrict SQLSTATE; every row is then counted
  and found still present, which is the half that says nothing disappeared; and the deletion succeeds
  once the caller has removed the content itself, in dependency order. A case table drives the first
  half so the failure message names *which* referencing row held the book `[derived → AC11]`.
- **`a_work_with_no_divisions_is_stored_as_one_untitled_chapter`** — a book with a single chapter
  carrying no title holds its paragraphs, and the paragraphs are reachable from the book through that
  chapter; and a paragraph attached to no chapter is refused `[derived → AC13]`.

### Red observations required before the group's last commit

Each is performed, its output pasted into the progress file, and the mutation reverted — the revert
confirmed with `git diff --name-only`, never with a re-read. A green suite that has never been seen
red is a claim about the suite (`AGENTS.md` § Patterns 2). Mutating the migration is safe during
these: every trial's database is created fresh inside the run, so a changed checksum meets no
recorded one `[derived → the per-trial database the harness creates]`.

- **The golden fails in both directions.** Add a column to one table in the migration and confirm the
  shape trial fails naming it; then remove a column and confirm it fails again. A set comparison
  written in one direction only is the shape this rules out `[derived → AC1]`.
- **The index assertion sees a wrong column order.** Reverse the key columns of the reading-order
  index and confirm the trial fails, on the catalogue half as well as the planner half `[derived → AC4]`.
- **The dimension assertion sees another dimension.** Declare the embedding column at a different
  dimension and confirm both the shape trial and the refusal trial fail `[derived → AC5]`.
- **The absent-index assertion can see an index.** Create an index over the embedding column with the
  vector access method in the migration and confirm the trial fails `[derived → AC6]`.
- **The empty-table assertion can see a row.** Add an insert into the embedding table to the
  migration and confirm the trial fails `[derived → AC7]`.
- **The delete rule assertion can see a cascade.** Change one reference to cascade on delete and
  confirm both the schema-wide rule trial and the book-deletion trial fail `[derived → AC11]`.
- **The required-value assertion can see an optional column.** Drop one `NOT NULL` from the migration
  and confirm the corresponding case fails — and that the content-blindness trial stays green, which
  is what separates the two rules `[derived → AC9 and AC10]`.
- **The embedded-set assertion can see a stale build.** Rename the migration to another version,
  confirm the unit case fails, and restore the name `[derived → AC1]`.

### Gates

`make verify` runs before each of the group's commits, and the pre-commit hook runs the ratchet on
the commits that stage a migration or a Rust source. Both need a reachable container socket, per
§ Risks.

## Open questions

- **No `SPEC-REMIT` tag is raised, and the check was made row by row rather than assumed.** Every
  spec row that names a mechanism — one migration for the whole schema, the primary key's columns, the
  named indexes, the vector dimension, the absent HNSW index, the tables created but not filled,
  the per-test database — is anchored to the owner's own wording in the issue body or to an interview
  answer, as the state file persists them
  `[measured 183da0c:ai-docs/plans/2026-09-19-storage-schema-second-migration.spec.md.state.md · sed -n '/^  body: |/,/^  comments:/p' … → the issue body carrying each of those clauses verbatim, and prior_qa carrying the answers "Запрет", "Свободный текст", "Структурная" and "Одна глава"]`.
  None of them is a spec-side choice of how, so none is flagged; where the spec is silent — nullability,
  the delete rule's spelling, index names, placement of the trials — this design decides, in **D2**
  through **D5** and **D9**.
- **The nullable set is decided here rather than asked, and the direction is why.** The owner ruled
  on the two rows that were actually contested — the chapter title stays optional, the paragraph text
  does not — and the rest of the set follows the rule **D2** states. Asking about the remainder would
  be asking about the reversible direction: a column made required now can be loosened later, while a
  column left optional can be tightened only if the data already fits, which is the owner's own
  framing of the trade-off. If any member of the nullable set is wrong, the repair is a forward
  migration and not an edit.
- **The two unmarked pointer columns — decided in D4, open to revision by migration.** Whether
  `context_snapshots.upto_paragraph_id` and `positions.paragraph_id` should become references is a
  question the corpus answers by omission and the spec does not raise. This design follows the corpus
  and states the consequence plainly: a paragraph may be deleted while a snapshot or a reading
  position still points at it. Nothing in this milestone deletes a paragraph — the repositories are a
  later task — so the question is live rather than urgent, and it is carried to the owner as a
  follow-up rather than decided differently here.
