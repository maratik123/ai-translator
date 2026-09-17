# 09. Сборка, деплой, dev-окружение

## Задачи
- [ ] Workspace `Cargo.toml` с `shared`, `core`, `cli`, `server`, `migrate`; `frontend` с `pnpm`.
- [ ] `just`/Makefile: `gen-types` (cargo test в shared), `dev` (backend с `ServeDir` + `vite dev` с proxy на `/ws` и `/api`), `build` (vite build → rust-embed → cargo build --release).
- [ ] CI (GitHub Actions): clippy, тесты, проверка что `gen-types` не дает диффа, сборка фронта.
- [ ] `deploy/reader.service` (systemd user): `ExecStart=%h/.local/bin/reader --config %h/.config/reader/config.toml`, `After=llama-server.service`.
- [ ] `deploy/llama-server.service` и `deploy/llama-embed.service` с командными строками из `05-llm-client.md`, `Restart=on-failure`. Устройство прибивать явно (`-dev Vulkan0`): видимых Vulkan-устройств два, второе — софтверный lavapipe.
- [ ] Postgres: роль `reader`, БД `reader`, `CREATE EXTENSION vector` от суперпользователя (или разрешить роли). Миграции через `reader-migrate` (отдельный бинарь, `ExecStartPre` в юните сервера), сервер и CLI только проверяют версию схемы.
- [ ] Тесты требуют Podman socket (`systemctl --user enable --now podman.socket`); `just test` выставляет `DOCKER_HOST`.
- [ ] Логи через `journalctl --user -u reader`.
- [ ] Доступ с планшета: `host = "0.0.0.0"`, адрес компа в локальной сети, при необходимости `avahi` для `sytmaratik.local`.

## Окружение (Gentoo, RX 9070 XT + Ryzen 7 5800X + 32 ГБ DDR4)

Установлено и проверено:
- Rust 1.98.1; PostgreSQL 18.6 с server-заголовками (`/usr/include/postgresql-18/server`), кластер живой;
- ROCm 7.2 (`dev-util/hip`, `sci-libs/hipBLAS`, `sci-libs/rocWMMA`), `AMDGPU_TARGETS="gfx1201"` в make.conf, `rocminfo` видит gfx1201 нативно — `HSA_OVERRIDE_GFX_VERSION` не нужен;
- Vulkan: RADV на gfx1201, все зависимости сборки (`vulkan-headers`, `spirv-headers`, `shaderc`, `vulkan-loader`, `openmp`);
- podman 5.8.2 с сокетом для testcontainers;
- CPU: Zen 3, 8 ядер, AVX2/FMA/F16C/BMI2, **без AVX512** — соответствующие `GGML_*` выставляются из `CPU_FLAGS_X86` автоматически.

Пакеты, которых в дереве нет или которые требуют настройки (**всё выполнено**):
- [x] `sci-misc/llama-cpp` — оверлей **guru**, нужен `~amd64`; USE `vulkan rocm wmma curl openmp`. Собран с обоими бэкендами, `--list-devices` показывает `ROCm0` и `Vulkan0`.
- [x] `dev-db/pgvector-0.8.6` — **нет ни в одном репозитории**, лежит локальный ebuild в оверлее `local-syt` (собран на `postgres-multi.eclass`, как штатные `dev-db/pgtap`/`postgis`).
- [x] `POSTGRES_TARGETS="postgres18"` в make.conf: профильный дефолт `postgres17`, а установлен слот 18 — иначе сборка расширений падает.
- [x] Роль и БД `reader`, `CREATE EXTENSION vector` от суперпользователя. Коннект по TCP на `127.0.0.1` с паролем.

Проверено функционально: `vector(1024)`, косинусный поиск `<=>` и индекс `USING hnsw (embedding vector_cosine_ops)` на 2000 строк — всё работает на Postgres 18.6 с pgvector 0.8.6.

## Порядок реализации (MVP)
1. `01` shared + `03` storage + `02` импорт epub (без fb2) → `reader-cli import`, `psql` показывает абзацы.
2. `05` клиент + `04` воркер без планировщика + `11` `translate`/`export` → двуязычный epub одной главы, слепое сравнение 3 моделей через `compare`.
3. `11` `eval` и набор ловушек из `10` → метрики рода и полноты; дальше все изменения промпта проверяются им.
4. Компактификация контекста, таблица персонажей и рассказчик (`10`, шаги 1-2), ночной режим целиком через CLI.
5. `06` WS + `08` клиент + `07` читалка в минимальном виде → чтение с префетчем поверх того же движка.
6. Эмбеддинги и RAG-хвост; аннотации референций; fb2; библиотека и настройки в UI.
