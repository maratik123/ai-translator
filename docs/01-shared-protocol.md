# 01. Shared crate и протокол WS

## Задачи
- [ ] Создать `crates/shared` с `serde`, `ts-rs`, feature `ts` для экспорта.
- [ ] Описать доменные DTO: `BookSummary`, `ChapterSummary`, `ParagraphDto`, `TranslationDto`, `ContextSnapshotDto`, `LlmStatusDto`.
- [ ] Описать `ClientMsg` и `ServerMsg` как `#[serde(tag = "type", content = "payload")]` enum'ы.
- [ ] RPC-обертка: `Envelope { request_id: Option<u64>, msg }`; ответы несут тот же `request_id`, события без него.
- [ ] Тест, генерирующий TS в `frontend/src/generated/` (`cargo test --features ts`), и проверка в CI, что файлы закоммичены без диффа.

## Сообщения
```rust
enum ClientMsg {
    ListBooks,
    OpenBook { book_id },
    LoadChapter { book_id, chapter_index },
    SetPosition { book_id, paragraph_id },
    TranslateBook { book_id },            // полный перевод
    RetryParagraph { paragraph_id },
    GetSettings, UpdateSettings { settings },
}
enum ServerMsg {
    Books { books },
    BookOpened { book, chapters, position },
    Chapter { chapter, paragraphs, translations },
    ParagraphTranslated { paragraph_id, translation },
    TranslationFailed { paragraph_id, error },
    ContextCompacted { book_id, version, upto_paragraph },
    BookProgress { book_id, translated, total },
    LlmStatus { online, model, tokens_per_sec },
    Settings { settings },
    Error { request_id, message },
}
```

## Правила
- Идентификаторы: `u64`/`i64` в БД, в JSON как числа (осторожно с > 2^53, для этого проекта не актуально).
- Все `Option` в TS становятся `T | null`, не `undefined` (настройка ts-rs).
- Версионирование: поле `protocol_version` в первом сообщении сессии, несовпадение → фронт просит перезагрузить страницу.
