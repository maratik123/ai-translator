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
round: 2
agent_id: a45a288bba5f92528
prior_qa:
  - round: 1
    question: "Входит ли пятый крейт `crates/server` в воркспейс этой задачей? Текст задачи перечисляет четыре члена, а оба документа, на которые она ссылается, — пять."
    answer: "Четыре"
  - round: 1
    question: "Насколько широкий набор общих зависимостей объявляет воркспейс в этой задаче?"
    answer: "Минимум"
  - round: 2
    question: "Имена пакетов — единственное молчание корпуса: `docs/` фиксирует роли и имена бинарей, но имён пакетов не называет. Какие берём? (→ design amendment via design-writer; каталоги в любом случае остаются shared/core/cli/migrate)"
    answer: "reader-* у всех"
  - round: 2
    question: "`ai-docs/propagation-groups.md` требует пропагацию Dependabot-строки «в том же коммите», а AGENTS.md § Propagation Rule — «в том же PR». Проект идёт за AXIOM и снимает строку как отработанную. (→ design amendment via design-writer)"
    answer: "Снять строку"
```
