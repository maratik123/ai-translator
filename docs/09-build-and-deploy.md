# 09. Сборка, деплой, dev-окружение

## Задачи
- [ ] Workspace `Cargo.toml` с `shared`, `core`, `cli`, `server`, `migrate`; `frontend` с `pnpm`.
- [ ] `just`/Makefile: `gen-types` (cargo test в shared), `dev` (backend с `ServeDir` + `vite dev` с proxy на `/ws` и `/api`), `build` (vite build → rust-embed → cargo build --release).
- [ ] CI (GitHub Actions): clippy, тесты, проверка что `gen-types` не дает диффа, сборка фронта.
- [ ] `deploy/reader.service` (systemd user): `ExecStart=%h/.local/bin/reader --config %h/.config/reader/config.toml`, `After=llama-server.service`.
- [ ] `deploy/llama-server.service` и `deploy/llama-embed.service` с командными строками из `05-llm-client.md`, `Restart=on-failure`.
- [ ] Postgres: роль `reader`, БД `reader`, `CREATE EXTENSION vector` от суперпользователя (или разрешить роли). Миграции через `reader-migrate` (отдельный бинарь, `ExecStartPre` в юните сервера), сервер и CLI только проверяют версию схемы.
- [ ] Тесты требуют Podman socket (`systemctl --user enable --now podman.socket`); `just test` выставляет `DOCKER_HOST`.
- [ ] Логи через `journalctl --user -u reader`.
- [ ] Доступ с планшета: `host = "0.0.0.0"`, адрес компа в локальной сети, при необходимости `avahi` для `sytmaratik.local`.

## Порядок реализации (MVP)
1. `01` shared + `03` storage + `02` импорт epub (без fb2) → `reader-cli import`, `psql` показывает абзацы.
2. `05` клиент + `04` воркер без планировщика + `11` `translate`/`export` → двуязычный epub одной главы, слепое сравнение 3 моделей через `compare`.
3. `11` `eval` и набор ловушек из `10` → метрики рода и полноты; дальше все изменения промпта проверяются им.
4. Компактификация контекста, таблица персонажей и рассказчик (`10`, шаги 1-2), ночной режим целиком через CLI.
5. `06` WS + `08` клиент + `07` читалка в минимальном виде → чтение с префетчем поверх того же движка.
6. Эмбеддинги и RAG-хвост; аннотации референций; fb2; библиотека и настройки в UI.
