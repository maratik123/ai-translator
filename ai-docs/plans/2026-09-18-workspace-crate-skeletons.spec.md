# Workspace and crate skeletons

**Source:** issue #10
**Date:** 2026-09-18
**Tracked in:** #10

There is no Cargo manifest at the repository root, so every cargo gate skips itself and a
green aggregate run is evidence about nothing. This task lays the workspace and the crate
skeletons down so those gates bind.

## Scope
1. A Cargo workspace exists at the repository root, carrying the crates the task lists as its members. [task: "Корневой `Cargo.toml` с воркспейсом: `crates/shared`, `crates/core`, `crates/cli`, `crates/migrate`."]
2. Each member is a skeleton that compiles and delivers no engine behaviour of its own. [task: "Скелеты крейтов: `//!`-документация модуля, `lib.rs` / `main.rs`, общие зависимости через `[workspace.dependencies]`."]
3. A dependency more than one member uses is declared once for the workspace rather than per member, and the set reaches no further than what the skeletons compile with. [task: "общие зависимости через `[workspace.dependencies]`"] [answer 1.2: "Минимум"]
4. The executables the workspace produces carry the names the task fixes. [task: "Имена бинарей: `reader-cli`, `reader-migrate`."]
5. The cargo gates measure real code instead of skipping, and every live statement in the repository that describes the skipping behaviour agrees with the tree afterwards. [task: "не печатает ни одной строки о пустом воркспейсе — гейты работают по-настоящему"]
6. The coverage ratchet starts from a value measured on this workspace. [task: "Ратчет покрытия инициализирован измеренным значением."]

## Out of scope
- A server component: this task delivers no `crates/server` member. The owner chose the member set the task text names, and the server arrives with its own task.
- Engine behaviour of any kind — import, storage, the translation pipeline, the model client, protocol types, migration statements. The crates are skeletons; the numbered documents under `docs/` are each their own task.
- The frontend workspace and the `gen-types` / `dev` / `build` targets that sit beside the workspace row in `docs/09-build-and-deploy.md` § Задачи.
- The `deploy/` examples and the Postgres role and database setup listed in the same section.
- Declaring a dependency the skeletons do not compile with, for later use. The task that needs it declares it, with its reason.

## Deferred
- A `server` crate skeleton | the repository-structure documents list it as a workspace member, and the owner scoped this task to the member set its text names | yes — separate issue.

## Key decisions
| Question | Decision |
|---|---|
| Does this task deliver a fifth member, `crates/server`, so that the workspace matches the repository-structure documents it cites? | No. The workspace carries the members the task text names; the server component comes with its own task. [answer 1.1: "Четыре"] |
| How far does the shared-dependency set reach: only what the skeletons compile with, or the stack the documents plan for? | No further than what the skeletons compile with — a dependency nothing yet compiles against is not declared in advance. [answer 1.2: "Минимум"] |

## Source conflicts
The task text names four workspace members; both documents it names as its source list five, `crates/server` among them.

- Task text, `## Объём` (issue #10, as the interview state file persists it): "Корневой `Cargo.toml` с воркспейсом: `crates/shared`, `crates/core`, `crates/cli`, `crates/migrate`."
- [source: de11627:docs/ARCHITECTURE.md § Структура репозитория · git show de11627:docs/ARCHITECTURE.md | sed -n '/^## Структура репозитория/,/^## Документы/p'] — the block lists `/crates/shared`, `/crates/core`, `/crates/cli`, `/crates/server`, `/crates/migrate`.
- [source: de11627:docs/09-build-and-deploy.md § Задачи · git show de11627:docs/09-build-and-deploy.md | sed -n '/^## Задачи/,/^## Окружение/p'] — "Workspace `Cargo.toml` с `shared`, `core`, `cli`, `server`, `migrate`; `frontend` с `pnpm`."

Resolution: the owner chose the task text's member set in round 1, question 1 — answer "Четыре". The documents keep their five-member structure for the tree that eventually exists; this task delivers the four, and the fifth is the Deferred row above.

## Acceptance Criteria
| # | Criterion |
|---|-----------|
| AC1 | The workspace manifest at the repository root names the crates the task lists among its members. [task: "Корневой `Cargo.toml` с воркспейсом: `crates/shared`, `crates/core`, `crates/cli`, `crates/migrate`."] |
| AC2 | The executable built from the command-line crate is named `reader-cli`, and the one built from the migration crate is named `reader-migrate`. [task: "Имена бинарей: `reader-cli`, `reader-migrate`."] |
| AC3 | No gate of the local aggregate run reports itself skipped for want of a manifest at the repository root. [task: "не печатает ни одной строки о пустом воркспейсе — гейты работают по-настоящему"] |
| AC4 | No live document or script of this repository still claims that the cargo gates skip themselves because the workspace is empty; the class is every site whose claim this diff falsifies, per AGENTS.md § Propagation Rule step 4. [task: "не печатает ни одной строки о пустом воркспейсе — гейты работают по-настоящему"] |
| AC5 | The coverage ratchet holds a line-coverage value its script measured on this workspace, and that script reports neither an absent ratchet file nor a workspace without executable lines. [task: "Ратчет покрытия инициализирован измеренным значением."] |

## Open questions
None.
