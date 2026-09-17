# Backend: архитектура (core + server)

Модули ниже живут в крейте `core`, кроме `http/` и `ws/`, которые составляют крейт `server`. Публичный API `core` и CLI описаны в `impl/11-core-api-and-cli.md`.

## Стек
Rust, tokio, axum (HTTP + WS), sqlx (PostgreSQL + pgvector), reqwest (стриминг), serde, rust-embed, tracing.

## Модули (`crates/backend/src`)
```
lib.rs             (core) Engine, EngineConfig, Event
main.rs            (server) запуск, конфиг, сборка роутера, graceful shutdown
export/
  epub.rs          двуязычный и одноязычный epub
  md.rs            markdown для диффов и сравнения моделей
config.rs          конфиг из TOML + env (порт, путь БД, URL модели, промпты, размеры окон)
http/
  static.rs        раздача встроенной статики, SPA fallback
  upload.rs        POST /api/books (multipart)
  assets.rs        GET /api/books/{id}/assets/{path} (картинки, обложки)
ws/
  server.rs        апгрейд, сессия на соединение
  session.rs       состояние сессии: открытая книга, позиция, подписки
  rpc.rs           диспетчеризация ClientMsg → обработчики, request_id → ответ
import/
  epub.rs          парсер epub (ZIP + OPF + XHTML)
  fb2.rs           парсер fb2
  segment.rs       разбиение в абзацы, стабильные id, очистка HTML
storage/
  mod.rs           трейт Store
  pg/              PgStore: пул, миграции, репозитории
  mem.rs           MemStore для тестов и eval
  db.rs            (внутри pg/) пул, миграции
  books.rs         книги, главы, абзацы
  translations.rs  кэш переводов по ключу
  context.rs       версии компактного контекста
  embeddings.rs    запись векторов, поиск похожих абзацев
embed/
  worker.rs        фоновый расчет эмбеддингов новых абзацев (батчами)
translate/
  scheduler.rs     окно префетча, приоритеты, дедупликация задач
  worker.rs        единственный потребитель очереди, вызывает LLM
  prompt.rs        сборка промптов перевода и компактификации
  context.rs       модель компактного контекста (сводка + глоссарий)
  retrieve.rs      выбор похожих ранних абзацев для хвоста промпта
  annotate.rs      препроход: разметка местоимений и реплик референтами
  validate.rs      проверки результата (число предложений, пустой ответ, эхо оригинала)
  validate/gender.rs  сверка рода глаголов и прилагательных с полом рассказчика и персонажей
llm/
  client.rs        OpenAI-совместимый клиент, стриминг SSE
  embeddings.rs    клиент /v1/embeddings
  types.rs         запрос/ответ
```

## Ключевые сущности
- `Book { id, title, author, lang_src, lang_dst, format, added_at }`
- `Chapter { id, book_id, index, title, html_head }` (стили/метаданные главы)
- `Paragraph { id, chapter_id, index, html, text, kind }` (`kind`: text, heading, image, code)
- `Translation { paragraph_id, cache_key, model, prompt_version, context_version, text, created_at }` (`context_version` — атрибут строки, в ключ не входит)
- `ContextSnapshot { book_id, version, upto_paragraph, summary, glossary_json, model }`; `glossary_json` содержит `narrator`, `characters` (с полом), `scene_state`, `terms`, `style`
- `ParagraphAnnotation { paragraph_id, context_version, annotated_text, uncertain }`
- `ReadingPosition { book_id, paragraph_id, updated_at }`

## Очередь перевода
- Один воркер перевода на процесс (модель одна, параллелить не имеет смысла кроме префилла). Очередь в памяти: второй инстанс бэка не нужен, GPU один.
- Отдельный воркер эмбеддингов с низким приоритетом; эмбеддинг-сервер работает **на CPU** и VRAM не занимает — иначе он не помещается рядом с основной моделью (см. `../05-llm-client.md`). Вся книга считается за секунды.
- `scheduler` держит `BinaryHeap<Task>` с приоритетом по расстоянию от позиции читателя; задачи компактификации имеют приоритет выше перевода абзацев, следующих за точкой компактификации, но ниже уже запрошенных.
- Задача перевода перед выполнением проверяет кэш по `cache_key = hash(model, prompt_version, paragraph.text)` — **без `context_version`**, см. решение в `../03-storage.md`. Попадание в кэш → сразу событие, без вызова модели; продвижение контекста само по себе перевод не инвалидирует.
- Отмена: при смене позиции далеко вперед задачи вне окна снимаются из очереди (уже начатый запрос доживает).

## Контекст модели
Промпт перевода состоит из стабильной головы (системная инструкция + текущий `ContextSnapshot`) и короткого хвоста (последние 2-3 абзаца оригинала с переводом, 2-3 похожих ранних абзаца из pgvector с их переводами, текущий абзац). Голова меняется только при новой версии контекста, чтобы работал кэш префикса llama-server. Замерено, что доля попаданий равна отношению «голова / (голова + хвост)», поэтому всё стабильное (глоссарий, таблица персонажей, правила регистра) должно жить в голове.

## Ошибки и деградация
- Postgres недоступен: бэк не стартует, health-check в `LlmStatus` расширить до `BackendStatus`.
- Эмбеддинг-сервер недоступен: перевод идет без RAG-хвоста, эмбеддинги досчитываются позже.
- llama-server недоступен: очередь копится, фронт получает `LlmStatus{offline}`, читатель видит оригинал без перевода.
- Невалидный ответ модели: один retry с указанием причины, затем `TranslationFailed`, абзац помечается для ручного ретрая. Полный список проверок и причин — в `../04-translation-pipeline.md`.

## Наблюдаемость
`tracing` с уровнями, метрики в лог: ток/с, размер очереди, глубина префетча относительно читателя, доля попаданий в кэш. Источник части метрик — `GET /metrics` самого llama-server, см. `../05-llm-client.md`.
