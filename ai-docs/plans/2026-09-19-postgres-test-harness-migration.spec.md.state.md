# Interview state — Postgres test harness and the vector-extension migration

Handoff between rounds, and the re-entry point for every later return to `spec-writer`. Kept on `ready`.

```yaml
schema_version: 1
spec_path: ai-docs/plans/2026-09-19-postgres-test-harness-migration.spec.md
issue_ref: "#13"
gh_issue:
  title: "Тестовая обвязка Postgres и первая миграция: расширение vector"
  state: open
  labels: ["infra", "storage"]
  issue_body_status: current
  body: |
    Первая задача вехи, которая касается БД. Она намеренно **не знает о схеме**: чтобы доказать себя, обвязке нужно живое соединение и что-нибудь, что можно спросить у базы, — но не девять таблиц.
    
    ## Объём
    - `testcontainers` с образом `pgvector/pgvector:pg18` через Podman socket (`DOCKER_HOST=unix:///run/user/$UID/podman/podman.sock`).
    - Версия образа совпадает с локальной машиной: Postgres 18.6 + pgvector 0.8.6.
    - Ryuk включён; контейнер один на тестовый бинарь (`OnceCell`).
    - **Миграция 1** — ровно одна инструкция: `CREATE EXTENSION IF NOT EXISTS vector`. Дока требует расширение именно первой миграцией.
    - `#[sqlx::test]` создаёт базу на тест и накатывает миграции сам (`docs/03-storage.md` § Задачи); на этой задаче накатывать нечего, кроме миграции 1.
    
    ## DoD
    - Тест поднимает контейнер, получает соединение, и `SELECT 'x'::vector` (или эквивалент) проходит — то есть образ действительно несёт pgvector, а не только называется так.
    - Машина без контейнерного рантайма узнаёт об этом из **падающего теста**, а не из тихого пропуска. Это надо увидеть красным, а не предположить: зелёная обвязка до первого наблюдавшегося падения — утверждение об обвязке, а не о машине.
    - Набор тестов не читает `DATABASE_URL`: это подключение приложения, и тест, который его возьмёт, поедет по данным разработчика.
    - Контейнер убирается за собой.
    
    ## Почему миграция здесь, а не в задаче про схему
    Иначе получается цикл: тест миграции требует обвязки, а DoD обвязки требует миграции. Минимальная первая миграция разрывает его и заодно проверяет самое хрупкое место в конфигурации — что поднятый образ действительно несёт pgvector нужного слота.
    
    ## Источник
    `docs/03-storage.md` § Задачи, `docs/09-build-and-deploy.md` § Задачи, `AGENTS.md` § Build & Test.
    
  comments: []
  linked_issues: []
  linked_prs: []
round_cap: 4
questions_per_round_cap: 3
round: 1
agent_id: a9e90b90a58970a4b
prior_qa: []
```
