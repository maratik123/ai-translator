# Postgres test harness and the vector-extension migration

**Source:** issue #13
**Date:** 2026-09-19
**Tracked in:** #13

The first task of the milestone that touches the database. It carries no schema: what it
delivers is a live connection the suite provisions for itself, plus one migration small
enough that a single question to the database proves the container really carries the
extension.

## Scope
1. The test suite provisions a Postgres database carrying the `vector` extension for itself, in a container it starts over the Podman socket. [task: "`testcontainers` с образом `pgvector/pgvector:pg18` через Podman socket"]
2. A run of the suite starts one container for the whole test binary rather than one per test. [task: "контейнер один на тестовый бинарь"]
3. Every database-backed test runs against a database of its own, with the repository's migrations applied to it. [task: "`#[sqlx::test]` создаёт базу на тест и накатывает миграции сам"]
4. The repository carries a first migration that creates the `vector` extension and does nothing else. [task: "ровно одна инструкция: `CREATE EXTENSION IF NOT EXISTS vector`"]
5. The suite's connection comes from the database it provisioned, never from `DATABASE_URL`. [task: "Набор тестов не читает `DATABASE_URL`"]
6. A machine that cannot reach a container runtime is told so by a failing test, and that direction is observed rather than assumed. [task: "а не из тихого пропуска. Это надо увидеть красным, а не предположить"]
7. Nothing the suite started outlives the run. [task: "Контейнер убирается за собой."]

## Out of scope
- The storage schema: the tables, the indexes and the repositories `docs/03-storage.md` describes. This task asks the database one question and delivers no table.
- Applying migrations to a database outside the test suite, and the schema-version check a database open performs — each a separate row of `docs/03-storage.md` § Задачи and its own task.
- The application's own database: the role, the database and the connection the application reads at run time.

## Deferred
- None yet; the round-1 answers may add one.

## Key decisions
| Question | Decision |
|---|---|
| Why the first migration belongs to this task rather than to the schema task | Otherwise the two lock each other: a migration test needs the harness, and the harness's definition of done needs a migration. The minimal first migration breaks the cycle and checks the most fragile point of the configuration at the same time. [task: "Минимальная первая миграция разрывает его и заодно проверяет самое хрупкое место в конфигурации"] |
| How far does the harness have to reach — the developer machine, or the repository's CI run as well? | TBD — round-1 question |
| Does the image the suite starts follow the floating tag the task names, or name the extension version the developer machine runs? | TBD — round-1 question |

## Acceptance Criteria
| # | Criterion |
|---|-----------|
| AC1 | The database the suite provisions accepts a value of the `vector` type, so the image it started carries the extension rather than only its name. [task: "`SELECT 'x'::vector` (или эквивалент) проходит — то есть образ действительно несёт pgvector, а не только называется так"] |
| AC2 | The repository's first migration is a single statement creating the `vector` extension, and no other migration precedes it. [task: "ровно одна инструкция: `CREATE EXTENSION IF NOT EXISTS vector`. Дока требует расширение именно первой миграцией."] |
| AC3 | Every database-backed test of the suite runs against a database of its own, with the repository's migrations applied to it. [task: "`#[sqlx::test]` создаёт базу на тест и накатывает миграции сам"] |
| AC4 | A run of the suite starts one container for the whole test binary, not one per test. [task: "контейнер один на тестовый бинарь"] |
| AC5 | No test in the suite takes its database connection from `DATABASE_URL`. [task: "Набор тестов не читает `DATABASE_URL`"] |
| AC6 | On a machine where no container runtime is reachable, the run reports that condition as a failing test rather than as a pass or a skip, and the direction has been observed on such a run rather than inferred. [task: "а не из тихого пропуска. Это надо увидеть красным, а не предположить"] |
| AC7 | No container the suite started is left running or left behind once the run ends. [task: "Контейнер убирается за собой."] |

## Open questions
None beyond the two this round puts to the owner.
