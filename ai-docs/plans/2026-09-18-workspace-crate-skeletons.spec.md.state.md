# Interview state — workspace and crate skeletons

Handoff between rounds, and the re-entry point for every later return to `spec-writer`. Kept on `ready`.

```yaml
schema_version: 1
spec_path: ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md
issue_ref: "#10"
gh_issue:
  title: "Воркспейс и скелеты крейтов shared / core / cli / migrate"
  state: open
  labels: ["infra"]
  body: |
    Первый крейт в дереве. До него каждый cargo-гейт пропускает себя и выходит с нулём, то есть зелёный `make verify` ничего не проверяет.

    ## Объём
    - Корневой `Cargo.toml` с воркспейсом: `crates/shared`, `crates/core`, `crates/cli`, `crates/migrate`.
    - Скелеты крейтов: `//!`-документация модуля, `lib.rs` / `main.rs`, общие зависимости через `[workspace.dependencies]`.
    - Имена бинарей: `reader-cli`, `reader-migrate`.

    ## DoD
    - `cargo build --workspace --all-targets` собирает всё.
    - `make verify` не печатает ни одной строки о пустом воркспейсе — гейты работают по-настоящему.
    - `make lock-check` проходит: манифест и локфайл согласованы.
    - Ратчет покрытия инициализирован измеренным значением.

    ## Источник
    `docs/ARCHITECTURE.md` § Структура репозитория, `docs/09-build-and-deploy.md` § Задачи.
  comments: []
  linked_issues: []
  linked_prs: []
  issue_body_status: current
round_cap: 4
questions_per_round_cap: 3
round: 1
agent_id: null
prior_qa: []
```
