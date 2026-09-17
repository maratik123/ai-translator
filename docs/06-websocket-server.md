# 06. WS-сервер и HTTP

## Задачи
- [ ] axum роутер: `GET /ws`, `POST /api/books`, `GET /api/books/{id}/assets/*path`, статика + SPA fallback.
- [ ] `rust-embed` для `frontend/dist`, в dev-режиме `ServeDir` + Vite proxy.
- [ ] Сессия на соединение: `tokio::select!` над входящими сообщениями и `broadcast`-каналом событий; фильтрация событий по открытой книге.
- [ ] RPC: `ClientMsg` с `request_id` → обработчик возвращает `ServerMsg`, ошибки в `Error{request_id}`.
- [ ] Несколько клиентов одновременно (комп + планшет): позиция общая на книгу, последняя запись побеждает.
- [ ] Graceful shutdown: дождаться текущей задачи перевода, закрыть БД.
- [ ] Bind на `0.0.0.0` за флагом, по умолчанию `127.0.0.1`; без аутентификации, доступ ограничивается сетью.

## Конфиг (`config.toml`)
```toml
[server] host = "127.0.0.1"; port = 3000
[storage] database_url = "postgres://reader@localhost/reader"; assets_dir = "~/.local/share/reader/assets"
[llm] base_url = "http://127.0.0.1:8080"; model = "gemma-4-26b-a4b-it"
[embeddings] base_url = "http://127.0.0.1:8081"; model = "bge-m3"; dim = 1024
[translate] prefetch_window = 40; compact_every = 60; lang_dst = "ru"
```
