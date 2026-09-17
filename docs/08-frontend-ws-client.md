# 08. Фронт: WS-клиент и стейт

## Задачи
- [ ] `api/ws.ts`: класс с `connect()`, `request<T>(msg): Promise<T>` (request_id, таймаут 30 с), `on(event, handler)`.
- [ ] Реконнект с backoff 1→2→4→…→30 с, после реконнекта повторное `OpenBook` + `LoadChapter` из стора.
- [ ] Очередь исходящих на время реконнекта: `SetPosition` схлопывается до последнего значения.
- [ ] `store/reader.ts`: `book`, `chapters`, `chapterIndex`, `paragraphs`, `translations: Map`, `positionId`, `contextVersion`.
- [ ] Обработчики событий: `ParagraphTranslated` → `translations.set`, `TranslationFailed` → статус в мапе, `LlmStatus` → индикатор, `BookProgress` → библиотека.
- [ ] `api/http.ts`: upload с прогрессом (`XMLHttpRequest` или `fetch` + ReadableStream), хелпер URL картинок.
- [ ] Обработка `Error{request_id}`: реджект промиса, тост.

## Тесты
- Юнит-тесты стора на редьюсеры событий.
- Мок WS-сервера (`mock-socket`) для реконнекта и схлопывания позиции.
