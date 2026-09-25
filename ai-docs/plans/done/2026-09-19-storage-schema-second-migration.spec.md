# Storage schema: the second migration

**Source:** issue #11
**Date:** 2026-09-19
**Tracked in:** #11

The milestone's second database task. It delivers the storage schema itself — every table the
storage document describes, created by one migration. The schema is live data from the day it
lands: a later column rename or enum renumbering is a migration and not an edit, so what the
database accepts and what it refuses is what this task settles now.

## Scope
1. A migration creates the whole storage schema the storage document describes — the book, chapter, paragraph, translation, context-snapshot, paragraph-embedding, paragraph-annotation, position and settings tables. [task: "**Миграция 2** создаёт схему целиком, ровно как она описана в доке"]
2. A translation is identified by the paragraph, the cache key and the context version together, with the context version an attribute of the row rather than a component of the key. [task: "`translations` с первичным ключом `(paragraph_id, cache_key, context_version)`; `context_version` — атрибут строки, **не** часть `cache_key`"]
3. The schema carries the three lookups the task names: paragraphs within a chapter in reading order, translations of a paragraph, and a book's context snapshot up to a given paragraph. [task: "Индексы: `paragraphs(chapter_id, idx)`, `translations(paragraph_id)`, `context_snapshots(book_id, upto_paragraph_id)`"]
4. The paragraph-embedding table's embedding column holds a 1024-dimension vector. [task: "`paragraph_embeddings.embedding` типа `vector(1024)`"]
5. The tables later tasks will fill — paragraph embeddings and paragraph annotations — are created by this migration all the same, because a schema arrives by forward migration. [task: "Таблицы `paragraph_embeddings` и `paragraph_annotations` создаются, хотя наполняются позже"]
6. The migration applies to the database the suite creates for each test inside its container. [task: "Миграция накатывается на базу, которую обвязка #13 создаёт **на каждый тест внутри контейнера**"]
7. The database keeps a book's structure sound on its own: a position is taken once within its parent, and a row missing a value it would be meaningless without does not reach storage. It judges nothing about the content of a value beyond that. [answer 2.1: "Структурная"]
8. A paragraph reaches its book through a chapter, and a work with no divisions of its own is stored as a single untitled chapter rather than as paragraphs hanging off the book. [answer 2.2: "Одна глава"]

## Out of scope
- The HNSW index over the embedding column: a single book's worth of rows does not need it, and the storage document already records when it is added.
- The repositories that read and write these tables, and the queries the storage document sketches over them — a separate row of that document's task list.
- The schema-version check an engine performs against the applied migrations when it opens a database.
- Applying the migration to the owner's own `reader` database: that stays a manual step outside the test loop.
- The glossary search index the storage document leaves conditional on a term search through the UI.

## Deferred
- HNSW index over the embedding column | a library of one book does not need it; the storage document defers it until the library grows | separate issue needed? yes, when the library grows
- GIN index over the context snapshot's glossary | the storage document makes it conditional on term search through the UI | separate issue needed? yes, together with that search

## Key decisions
| Question | Decision |
|---|---|
| How much integrity does the schema enforce beyond the columns, keys and types the storage document sketches? | Structural integrity only: a position is unique within its parent, and a value a row would be meaningless without is required. The content of a value is not judged, so no check rejects an empty text or a negative position. [answer 2.1: "Структурная"] |
| What becomes of a book's chapters, paragraphs, translations, snapshots, embeddings, annotations and reading position when the book row is deleted? | Nothing goes with it. The database refuses the deletion while anything still references the book, and a caller that wants the book gone removes its content itself. [answer 1.2: "Запрет"] |
| Do the categorical columns — a paragraph's kind, a book's format, its source and target languages — refuse a value outside the set the project supports? | No. They hold free text, the supported set lives in the code, and narrowing the columns later is a migration rather than an edit. [answer 1.3: "Свободный текст"] |
| Does a work with no chapter division still reach its paragraphs through a chapter row? | Yes. Such a work carries one chapter with no title of its own, and a paragraph never belongs to a book directly — the reading path stays single. [answer 2.2: "Одна глава"] |

## Acceptance Criteria
| # | Criterion |
|---|-----------|
| AC1 | After the repository's migrations are applied, the database holds the nine tables the task names, each with the columns and types the storage document's schema section lists. [task: "**Миграция 2** создаёт схему целиком, ровно как она описана в доке: `books`, `chapters`, `paragraphs`, `translations`, `context_snapshots`, `paragraph_embeddings`, `paragraph_annotations`, `positions`, `settings`"] [source: c572a26:docs/03-storage.md § Схема · git show c572a26:docs/03-storage.md] |
| AC2 | Two translation rows of one paragraph sharing a cache key and differing in context version coexist, and a row repeating an already-stored paragraph, cache key and context version together is refused. [task: "`translations` с первичным ключом `(paragraph_id, cache_key, context_version)`"] |
| AC3 | A lookup of a translation by paragraph and cache key alone, naming no context version, finds a stored row. [task: "`context_version` — атрибут строки, **не** часть `cache_key`"] |
| AC4 | Paragraphs within a chapter in reading order, translations of a paragraph, and a book's context snapshot up to a given paragraph are each served by an index of their own. [task: "Индексы: `paragraphs(chapter_id, idx)`, `translations(paragraph_id)`, `context_snapshots(book_id, upto_paragraph_id)`"] |
| AC5 | The embedding column accepts a 1024-dimension vector and refuses a vector of any other dimension. [task: "`paragraph_embeddings.embedding` типа `vector(1024)`"] |
| AC6 | No HNSW index exists over the embedding column. [task: "HNSW-индекс пока не добавляем"] |
| AC7 | The paragraph-embedding and paragraph-annotation tables exist once the migration has been applied, with no row written to either. [task: "Таблицы `paragraph_embeddings` и `paragraph_annotations` создаются, хотя наполняются позже"] |
| AC8 | A second chapter at a position already taken within its book, and a second paragraph at a position already taken within its chapter, are each refused. [answer 2.1: "Структурная"] |
| AC9 | A row lacking a value it would be meaningless without — a paragraph with no text, a child row with no reference to its parent, a translation with no cache key — is refused. [answer 2.1: "Структурная"] |
| AC10 | The database judges whether a value is present and whether a position is already taken, never what a value contains: a paragraph whose text is the empty string is stored as given. [answer 2.1: "Структурная"] |
| AC11 | Deleting a book row is refused while a chapter, paragraph, translation, context snapshot, embedding, annotation or reading position still references it, and no such row disappears as a side effect of a deletion anywhere in the schema. [answer 1.2: "Запрет"] |
| AC12 | A paragraph kind, a book format, and a source or target language outside the set the project supports are stored as given: none of those four columns is restricted to a fixed set of values. [answer 1.3: "Свободный текст"] |
| AC13 | Every paragraph belongs to a chapter and a chapter's own title is optional, so a work with no divisions is stored complete as a single untitled chapter, while a paragraph attached to no chapter is refused. [answer 2.2: "Одна глава"] |

## Open questions
None.
