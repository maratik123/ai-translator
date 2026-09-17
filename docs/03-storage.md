# 03. Хранилище (PostgreSQL + pgvector)

## Задачи
- [ ] `sqlx` с фичей `postgres`, миграции в `crates/core/migrations/`, пул через `PgPoolOptions`. Компиляция с `SQLX_OFFLINE=true` и `.sqlx/` в репозитории; `cargo sqlx prepare --check` в CI.
- [ ] Миграции накатывает только отдельный бинарь `reader-migrate` (`sqlx::migrate!()` из `core`). `sqlx-cli` только для разработки (`migrate add`, `prepare`).
- [ ] `Engine::open` проверяет схему по `_sqlx_migrations`: версии и чексуммы должны совпадать со встроенными; не хватает → ошибка «схема старше кода, запустите reader-migrate»; лишние → «схема новее кода». Конвертер и сервер сами не мигрируют.
- [ ] Тесты: `testcontainers` с образом `pgvector/pgvector:pg18` через Podman socket (версия образа должна совпадать с локальной: на машине Postgres 18.6 + pgvector 0.8.6) (`DOCKER_HOST=unix:///run/user/$UID/podman/podman.sock`), ryuk включен, контейнер один на тестовый бинарь (`OnceCell`); `#[sqlx::test]` создает БД на тест и накатывает миграции сам.
- [ ] Расширение `vector` в первой миграции: `CREATE EXTENSION IF NOT EXISTS vector`.
- [ ] Репозитории: `books`, `chapters`, `paragraphs`, `translations`, `context_snapshots`, `positions`, `settings`, `embeddings`.
- [ ] Отдельная роль и БД `reader` на локальном Postgres; connection string в конфиге.
- [ ] Файлы книг (картинки, css, обложки) на диске в `assets_dir`, в БД только пути. Бэкап = `pg_dump` + каталог.

## Схема
```sql
books(id bigserial PK, title, author, lang_src, lang_dst, format, cover_path, added_at timestamptz)
chapters(id bigserial PK, book_id FK, idx int, title, css text)
paragraphs(id bigserial PK, chapter_id FK, idx int, kind text, html text, text text,
  stable_hash bytea)                              -- hash(book_id, chapter_idx, idx, text) для переиспользования при реимпорте
translations(
  paragraph_id FK, cache_key bytea, model text, prompt_version int, context_version int,
  text text, created_at timestamptz, PRIMARY KEY(paragraph_id, cache_key))
context_snapshots(book_id FK, version int, upto_paragraph_id bigint,
  -- glossary: narrator, characters, scene_state, terms, style — см. 10-gender-and-coreference.md
  summary text, glossary jsonb, model text, created_at timestamptz,
  PRIMARY KEY(book_id, version))
paragraph_embeddings(paragraph_id PK FK, model text, embedding vector(1024))
paragraph_annotations(paragraph_id FK, context_version int, annotated_text text, uncertain bool,
  PRIMARY KEY(paragraph_id, context_version))
positions(book_id PK FK, paragraph_id bigint, updated_at timestamptz)
settings(key text PK, value jsonb)
```

## Индексы
- `paragraphs(chapter_id, idx)`, `translations(paragraph_id)`.
- `paragraph_embeddings USING hnsw (embedding vector_cosine_ops)`; для одной книги (≤ 10k строк) достаточно и без индекса, HNSW добавить, когда библиотека вырастет.
- `context_snapshots(book_id, upto_paragraph_id)` для поиска актуальной версии.

## Кэш переводов
- `cache_key = blake3(model || prompt_version || context_version || text)`.
- **Важно:** ключ идентифицирует запись кэша, а не воспроизводимый результат. Повторный перевод того же абзаца с теми же параметрами даёт другой текст (замер в `05-llm-client.md`). Для кэша это безразлично, для сравнения версий промпта — нет: сравнение должно идти на жадном декодировании.
- «Активный» перевод выбирается по текущим настройкам; старые остаются для сравнения моделей.
- Запрос для окна: `SELECT ... WHERE paragraph_id = ANY($1) AND cache_key = ANY($2)`.

## Открытый вопрос: `context_version` в ключе кэша
`cache_key` включает `context_version`. Значит после каждой компактификации ключ уже переведённого абзаца перестаёт совпадать с текущим, и при повторном открытии книги абзац переводится заново, хотя готовый перевод лежит в базе. Варианты:

1. указатель «активный перевод» на абзац (`translations.is_active` или отдельная таблица);
2. `context_version` как метаданные строки, а не часть ключа;
3. выбирать перевод по «той версии контекста, что была актуальна для этого абзаца» — то есть последней с `upto_paragraph_id < paragraph_id`.

Не решено. Решение ложится в миграцию, поэтому закрыть до первого `reader-migrate`.

## Поиск похожих абзацев (RAG по книге)
```sql
SELECT p.id, p.text, t.text AS translation
FROM paragraph_embeddings e
JOIN paragraphs p ON p.id = e.paragraph_id
JOIN chapters c ON c.id = p.chapter_id
JOIN translations t ON t.paragraph_id = p.id AND t.cache_key = ANY($3)   -- только уже переведенные
WHERE c.book_id = $1 AND p.id < $4                                        -- только раньше по тексту
ORDER BY e.embedding <=> $2
LIMIT 3;
```

## Глоссарий
`glossary jsonb` с GIN-индексом, если понадобится поиск по терминам через UI: `CREATE INDEX ON context_snapshots USING gin (glossary)`.
