# ai-translator — Agent Rules

**CRITICALLY**
1) **Language is split by surface, and the split is deliberate.** Russian: this file's project sections, `docs/`, `README.md`, commit messages and pull-request bodies — the repository's own history is Russian and stays that way. English: the harness (`.claude/**`, `ai-docs/**`), code, identifiers, comments, and the specs and designs under `ai-docs/plans/`. Conversation with the owner is Russian.

Читалка книг с параллельным переводом локальной LLM. Ценность проекта — в конвейере
качества перевода, а не в UI. Полное описание в `docs/`, начинать с `docs/ARCHITECTURE.md`.

**Языки:** en, fr, es, it, de → ru.
**MVP:** библиотека `core` + `reader-cli` (`docs/11-core-api-and-cli.md`), шаги 1-4 из
«Порядок реализации» в `docs/09-build-and-deploy.md`. Сервер и фронт — после.

## Работа с репозиторием
`master` защищён, прямой push запрещён — в том числе владельцу. Любое изменение идёт веткой:
коммит → push ветки → PR → merge. Апрувы не требуются, но PR обязателен. Force-push и удаление
ветки запрещены. Слияние только merge-коммитом (squash и rebase отключены), ветка после merge удаляется сама.

## Локальное окружение (проверено)

| Компонент | Значение |
|---|---|
| Rust | 1.98.1 |
| llama.cpp | b10927, собран с обоими бэкендами (ROCm и Vulkan) |
| GPU | RX 9070 XT (gfx1201), 16 ГБ; ROCm 7.2 видит карту нативно |
| CPU | Ryzen 7 5800X, Zen 3, AVX2, без AVX512; DDR4 |
| Postgres | 18.6 + pgvector 0.8.6 |
| БД | локальный Postgres, база `reader`, расширение `vector` создано; DSN в `DATABASE_URL` |
| Тесты | testcontainers через Podman socket, образы `pgvector/pgvector:pg18` и `ghcr.io/ggml-org/llama.cpp:server` |

`models/` и `examples/` в `.gitignore` — файлы только локальные. Мини-модели для тестов — другое: они лежат в `testdata/models/` и **закоммичены**, вместе с лицензиями и описанием происхождения.

## Запуск моделей

Перевод — **ROCm, без офлоада**, 65 с на книгу из 92 абзацев, разброс ±1 с:
```bash
llama-server \
  -m models/ggml-org/gemma-4-26B-A4B-it-GGUF/gemma-4-26B-A4B-it-Q4_0.gguf \
  -md models/ggml-org/gemma-4-26B-A4B-it-GGUF/mtp-gemma-4-26B-A4B-it-Q4_0.gguf \
  --spec-type draft-mtp --spec-draft-n-max 2 \
  -dev ROCm0 -c 16384 -ngl 99 \
  -ctk q8_0 -ctv q8_0 -fa on -t 8 -np 1 --load-mode none \
  --metrics --host 127.0.0.1 --port 8080
```
`-ot` не нужен: под ROCm модель влезает в VRAM целиком (14.34 ГБ, с MTP 14.71 — голова стоит 382 МБ). Под Vulkan модель целиком не помещается, остаётся офлоад на CPU — и время
начинает плавать на 20% в зависимости от состояния RAM.

Эмбеддинги — **bge-m3** (на CPU, ~6 с на книгу, нулевая VRAM):
```bash
llama-server -m models/ggml-org/bge-m3-Q8_0-GGUF/bge-m3-q8_0.gguf \
  --embedding -ub 2048 -c 2048 -np 1 \
  -dev none -ngl 0 -t 8 --host 127.0.0.1 --port 8081
```
`--pooling` не задавать — читается из метаданных GGUF. `-ub` должен вмещать самый
длинный отдельный абзац, иначе `input (N tokens) is too large to process`.

Гасить: `pkill -x llama-server`. **Не** `pkill -f "llama-server -m models"` — шаблон
совпадёт с собственной командной строкой и убьёт вызывающий процесс.

Раскладку под новую модель считать не руками, а `llama-fit-params` с теми же
аргументами, что у сервера: он печатает готовые `-ngl` и `-ot`.

## Что легко сделать неправильно

- **`-dev` обязателен.** Сборка с двумя бэкендами показывает одну карту дважды
  (`ROCm0` и `Vulkan0`); без `-dev` llama.cpp разложит модель по «двум» устройствам.
- **`chat_template_kwargs: {"enable_thinking": false}` в каждом запросе.** Иначе Gemma 4
  уходит в `reasoning_content`, `content` остаётся пустым, упирается в `max_tokens`.
- **`repeat_penalty` ставить 1.0 явно.** Штраф за повтор бьёт по глоссарию: имена
  персонажей и термины обязаны повторяться.
- **`--pooling` для эмбеддингов не задавать** — берётся из метаданных GGUF. Векторы уже нормализованы.
- **Префикс входа эмбеддинг-модели.** Берётся из `config_sentence_transformers.json` модели и для
  нашей симметричной задачи ставится на обе стороны: `Document: ` у `jina-v5-*-text-matching`,
  `task: sentence similarity | query: ` у EmbeddingGemma, пусто у bge-m3. Без него jina-v5 выглядит
  сломанной (кросс-язык 54 вместо 98), и её легко отбраковать по ошибке. Это не то же самое, что
  инструкция `Instruct: …\nQuery:` на одной стороне — та, наоборот, ухудшает (`docs/05`).
- **`eval`/`compare` только на `temperature 0`, с выключенным MTP и без офлоада на CPU.**
  Сервер недетерминирован при сэмплировании даже с фиксированным seed (~12%), MTP
  добавляет ещё ~24%, а офлоад экспертов в host-память — ещё 20% от состояния RAM/swap.
- **Спекулятивка: только `draft-mtp` с головой Q4_0.** Проверено и отвергнуто: голова
  Q8_0 (приём падает, драфтер должен совпадать с целью по кванту), четыре режима
  `ngram-*` (дают ноль: они копируют из контекста, а при переводе вход и выход на
  разных языках), `draft-dflash` (не влезает в 16 ГБ). Комбинировать типы через
  запятую бесполезно — это цепочка фолбэков, а не объединение. Подробности в `docs/05`.
- **Замеры производительности повторять и перемежать.** 31 ГБ RAM при 32 ГБ моделей:
  последовательный прогон нескольких конфигураций даёт порядок-зависимую систематику.

## Принятые решения с обоснованием в доках

- `llama-server` напрямую, не Ollama (`docs/05`): Ollama не отдаёт управление MoE-офлоадом.
- ROCm для основной модели: вмещает её целиком, офлоад исчезает (`docs/05`).
- Модель `ggml-org/gemma-4-26B-A4B-it` Q4_0; расцензуренная сборка проверена и отклонена —
  базовая не отказывает, даёт тот же регистр, меньше ошибок и быстрее (`docs/05`).
- `context_version` не входит в `cache_key` (`docs/03`).
- `retrieve` берёт top-k, а не порог по расстоянию — порог откалиброван и отбрасывал
  верные соседи (`docs/04`).

Полный список с обоснованиями и ссылками на источник — [`ai-docs/key-decisions.md`](ai-docs/key-decisions.md).

> **AXIOM — `docs/` is DECISIONS.** The corpus under `docs/` is the finalized design: the shared protocol, import, storage, the translation pipeline, the model client, the server, the reader, build and deploy, gender and coreference, the core API and the command line. **Implement from it. Never redesign it** without an explicit request from the owner. A question it leaves open is not a licence to decide it silently: surface it, and tuning values go to configuration, never into the source.
>
> Read [`ai-docs/context.md`](ai-docs/context.md) for orientation and the standing decisions — on demand. Key decisions with their rationale: [`ai-docs/key-decisions.md`](ai-docs/key-decisions.md). Invariants that outrank convenience: [`ai-docs/domain-invariants.md`](ai-docs/domain-invariants.md).


## Permissions

Machine-enforced rules live in `.claude/settings.json` (allow/deny entries, hooks). Read that file for the authoritative list — duplicating it here lets the two sources drift.

> **`origin` enforces only what a ruleset requires — read it, never assume it.** Server-side protection of `master` is decided by `gh api repos/maratik123/ai-translator/rulesets`, not by this file, and not by `gh api repos/maratik123/ai-translator/branches/master/protection` — that legacy endpoint answers `404: Branch not protected` even while a ruleset is active, so reading only it reports an unprotected branch that is in fact protected. At the harness transplant the ruleset `master` was **active** with `deletion`, `non_fast_forward` and a `pull_request` rule whose `allowed_merge_methods` is `["merge"]` and whose required approvals are 0 — and with **no required status checks**, so CI reports but blocks nothing yet. Re-read the ruleset rather than trusting this prose: enforcement is a live setting, and `bypass_actors` decides who may merge past a red check, which a required-check list never tells you.

> **Bypassing or changing server-side protection is the owner's action, never a flow's.** A session acts with the owner's credentials, so GitHub cannot tell an agent's `gh pr merge --admin` or ruleset write from the owner's own. A `PreToolUse` hook refuses `gh pr merge … --admin`, a `gh api` write to `…/rulesets` or `…/branches/<branch>/protection`, and a GraphQL mutation of a ruleset or a branch protection rule; reading them passes (suite `ai-docs/scripts/test-bypass-guard.sh`). A required check that blocks a merge is reported to the owner, never routed around.

Honour-system rules (no machine check; still binding):

- **DENY:** `git push --force` to feature branches — prefer `--force-with-lease`, and only after explicit approval. Never force-push `master`.
- **DENY:** `git push` to `master`. Every change reaches `master` through a merged pull request.
- **DENY:** files outside the project root.
- **DENY:** a real credential — a connection string with a password, an API key, a token — in any tracked file, including a test fixture, an example, a commit message and a pull-request body. The database connection lives in the environment variable the process reads; a document names the variable, never a value. A leaked credential is rotated, not edited out of history.
- **ASK:** any tool not allow-listed in `settings.json`; if denied, suggest an alternative.

On session start: read `.gitignore`, treat matched paths as a read blacklist. Model weights and book texts are local and ignored — never read them into a durable artefact.

## Build & Test

```bash
make verify                                             # every gate below, in one run
cargo build --workspace --all-targets                   # whole workspace
cargo test --workspace                                  # all tests
cargo test --workspace <test name>                      # filter
cargo fmt --all                                         # apply the formatter
cargo fmt --all --check                                 # format check — non-zero exit = dirty
cargo clippy --workspace --all-targets -- -D warnings   # strict lint gate
make doc-check                                          # rustdoc with warnings denied
make lock-check                                         # the manifest and the lockfile agree
make file-limits                                        # the hard line bands
make comment-refs                                       # the comment-reference ban over the gated set
make panic-calls                                        # every panicking call in shipped code is marked
make import-guard                                       # no binary reaches a test-only crate
make cover-ratchet                                      # the coverage ratchet, check-only
actionlint .github/workflows/<file>.yml                 # required gate for any new/modified workflow file
shellcheck <script>.sh                                  # required gate for any new/modified shell script
```

> **While the workspace is empty, every cargo gate skips itself loudly and exits 0** — there is no `Cargo.toml` at the root until the first crate lands, and a gate that cannot run says so on stderr rather than passing in silence. The skips disappear with the commit that creates the workspace; nothing has to be switched on by hand.

> **The test suite provisions its own database.** A database-backed test starts a `pgvector` container through testcontainers over the Podman socket and drops it on cleanup. `DATABASE_URL` is the application's connection, never the suite's — a test that reads it runs against the developer's own data. A machine with no container runtime is told so by the failing test, not by a silent skip. **The model client's conformance tests are built the same way** — they start `llama.cpp:server` on CPU with the committed miniature models under `testdata/models/`, so a container runtime is not the database's business alone.

> **AXIOM — `actionlint` MUST pass before `git add` on any modified `.github/workflows/*.yml`; `shellcheck` MUST pass before `git add` on any modified `*.sh`.**
> Required gates, **same status as `cargo build` and `cargo clippy`.** Never bypass.
>
> | If you see... | Action |
> |---|---|
> | `M .github/workflows/<name>.yml` in `git status` | Run `actionlint <file>` (pass every changed workflow file in one invocation) **before** `git add` |
> | `M <any>.sh` in `git status` | Run `shellcheck <file>` **before** `git add` |
> | Either tool reports an error | Fix it. **NEVER** bypass. |
>
> What `actionlint` catches that cargo cannot: runner-version mismatches, deprecated action versions, expression-syntax errors, shell-quoting issues. Harness scripts are executable code and get the same treatment as `.rs` files.

> **A zero exit status is evidence about the LAST pipeline stage, not about your question.** Never pipe a gate whose exit code is load-bearing — `cargo test --workspace | tail -6` reports `tail`'s status (always 0), so a RED gate records as green, and `tail -N` can truncate away the failure line you needed. Capture to a file **under `tmp/`** and grep the saved log: `mkdir -p tmp && cargo test --workspace > tmp/gate.log 2>&1 && echo GATE-GREEN || echo GATE-RED`, then `grep -E "^(error|warning|test result|failures)" tmp/gate.log`. `tmp/` is the one ignored scratch directory. A gate's output redirected to a bare filename — which lands in the repository ROOT — is refused by a `PreToolUse` hook; a mutation backup or a throwaway probe written there by any other tool is **not** matched by that hook, goes to `tmp/` all the same, and is yours to keep out. A `PreToolUse` hook blocks the `cargo test … | tail/head` form, and a second blocks the common spelling of a `$?` read after a pipeline, not every spelling; the principle is broader than what either hook matches — the same silent-success shape covers a `jq` filter printing `null` from an error body, and a mutating flag rewriting output while exiting 0. (`set -o pipefail` also works.)

> **AXIOM — line coverage may rise and may not fall.** `.githooks/pre-commit` is a symbolic link to `.githooks/pre-commit.sh`, which runs the comment-reference gate over the staged set and then the ratchet. `.githooks/coverage-ratchet.sh` measures `cargo llvm-cov --workspace --summary-only --json`, compares the line total with the value recorded in `ai-docs/coverage-ratchet.txt`, refuses a drop past the tolerance, and records a new high-water mark in the same commit. `make cover-ratchet` and CI's Coverage-ratchet job run the identical script with `--check` — it never writes there.
>
> | State | What happens |
> |---|---|
> | No `.rs` / `.sql` / manifest / lockfile staged | Skipped, silently — coverage cannot have moved. Most commits in a `/task` run cost nothing. |
> | Unstaged edits to such files | **Blocked.** The measurement is taken on the working tree, so with them present it describes neither the commit nor the tree. |
> | Suite not green | **Blocked** — coverage is not measurable. |
> | No workspace, no `cargo`, no `cargo-llvm-cov`, no `jq` | Skipped, loud. Named fail direction per dependency: a machine that cannot measure cannot be asked for a number. |
> | The workspace has no executable lines yet | Skipped, loud. Once a crate carries code this cannot happen silently. |
> | Coverage fell past the tolerance | **Blocked**, with the least-covered files listed. |
> | Ratchet file absent | Initialised at the measured value and staged. There is no separate setup step. |
>
> **The two legitimate exits from a block are: cover what the change added, or lower the recorded value IN THE SAME COMMIT and say in the message why the drop is correct.** `--no-verify` is not a third one — a `PreToolUse` hook refuses it, because a gate an agent can switch off is not a gate.
>
> The tolerance starts at **0.00 pp**, and that is a starting value rather than a measurement: a tolerance is the width of the suite's own run-to-run drift, and a suite that does not exist has none. The first drift this project actually observes is what sets it, derived from a series of runs and never from a single blocked commit. The script's header carries the recipe.

**CI runs the same gates** (`.github/workflows/ci.yml`, through `make`): Format · Build (build + doc + lockfile + the dependency-direction gate) · Lint (clippy + file limits + the panic gate) · Test · Coverage ratchet · Harness guards (shellcheck on every script and hook body, the citation guard, the guard suites, the link check, and the check that the jobs and the checks the `master` ruleset requires are one set) · Comment references · Actionlint. Each job is paths-filtered, so **a job that did not run is not a passing job** — read the run, not the absence of red.

Search: `ast-index` first (see [`.claude/rules/ast-index.md`](.claude/rules/ast-index.md)); fall back to `rg <pattern> --type rust [-l | -C 3]` when `ast-index` returns empty.

## API Stability

> **AXIOM — No Rust API-stability contract. Clean breaks, always. No compat shims.**
> ai-translator is an **application**, not a library — nothing outside this workspace depends on it. A public item may be freely renamed, removed or restructured at any time without deprecation layers or aliases.
>
> | If you're tempted to... | Do this instead |
> |---|---|
> | Keep `old_name(...)` delegating to `new_name` "for compat" | **DELETE** it — call sites update directly |
> | Keep both old and new APIs side by side temporarily | Pick one — old is gone |
> | Add a trait solely to preserve an old signature | Remove the old signature |

> **CARVE-OUT — data contracts are the opposite, and the asymmetry is the point.** The Postgres schema, the stored translations and their cache keys, the context versions, the stored embeddings and their dimension, and any persisted enum value are **live data that outlives every deployment**. They change by **forward migration**, never by redefinition: a translation written last month must still be readable and must still be found by the key its text derives. A renamed state string, a re-numbered enum or a repurposed column is a data-corruption bug wearing a refactor's clothes ([`ai-docs/domain-invariants.md`](ai-docs/domain-invariants.md) INV-14).

## API Naming

Full rules — including the **`_unchecked` AXIOM** (a function skipping a precondition check carries the suffix, and its doc comment names both the precondition and the caller that guarantees it) — live in [`ai-docs/rust-api-naming.md`](ai-docs/rust-api-naming.md). The three that get violated most: no stutter (`cache::Key`, not `cache::CacheKey`); traits named for behaviour and declared **by the consumer** where the caller is what varies; a `# Errors` section on every fallible public function.

## Code Style

Thin by design — this project grows its own style rules through the learning loop (`/improve`). Start with:

- **Source files:** Rust under `crates/<crate>/src/` and `crates/<crate>/tests/`; format via `cargo fmt --all`, never by hand.
- **Linter posture:** `cargo clippy --workspace --all-targets -- -D warnings`; no `#[allow(...)]` without a specific lint and a stated reason.
- **Errors:** a typed error per failing operation, the source chain preserved, the message naming the operation. Never `let _ = <Result>`. Never an unmarked panicking call in shipped code — see § *Test Conventions*.
- **Magic numbers:** a literal with semantic meaning becomes a named constant. **Tuning values are different and stronger: they belong in configuration, not in source** — a context budget, a batch size, a top-`k`, a temperature, a timeout, a retry count, a distance floor. A tuning value hard-coded in a `.rs` file is a defect even when it is named.
- **Determinism:** segmentation, alignment, prompt rendering and cache-key derivation are pure functions of their inputs. No clock, no unseeded randomness, and no unordered-map iteration whose order reaches the output.
- **Documentation:** every public item carries a doc comment; every crate and module carries its own `//!`. See [`ai-docs/doc-convention.md`](ai-docs/doc-convention.md).
- **Comments point at nothing outside themselves.** A comment says what the thing is, states its call contract, and carries what the toolchain requires; it carries no markdown path, design-section number, acceptance-criterion id, decision anchor, review-register finding id, issue number outside `TODO(#…)`, repository path, URL, or crate-qualified symbol of this workspace named outside its own crate. The reason is rot: the thing pointed at is edited and the comment becomes a claim nothing checks. `make comment-refs` gates the lexical half over `*.rs`, `*.sh`, `*.sql`, `*.yml`, `*.yaml`, `.gitignore`, `Makefile` and `.githooks/**`; narration and the bare unqualified name are review-judged. Full rule and exemptions: [`ai-docs/doc-convention.md`](ai-docs/doc-convention.md) § DOC-4.
- **File size:** soft 500/800; hard 1200 under a crate's `src/` and 1500 under `tests/` — both gated; the wider src band exists because a unit test lives in the file it tests.

See [`ai-docs/code-style.md`](ai-docs/code-style.md) for the canonical (growing) reference.

## Domain Rules

Project invariants that outrank convenience. Full detail, one numbered rule each: [`ai-docs/domain-invariants.md`](ai-docs/domain-invariants.md).

> **AXIOM — A translation's cache key is derived from the model, the prompt version and the paragraph's text, and from nothing else.** The context version is stored beside the translation, not inside the key; a read takes the newest context version available. Folding it into the key re-translates the whole book after every compaction, discarding good translations already in the database. A change to what the key is derived from is a migration, not an edit.

> **AXIOM — A response from the model is validated before it is stored.** Empty output, an echo of the source, a preamble, reasoning in place of a translation, glossary violations, words outside the dictionary, silent softening — each is a named class and each is a test case. The class that defeats all of them, fluent and plausible nonsense, is an open risk recorded in `docs/04-translation-pipeline.md`, and no check may claim to cover it.

Three more, each with its mechanics on that page: **retrieval takes the top `k`**, and a distance floor is a guard against "nothing similar exists" that belongs to one embedding model and one input prefix; **the request parameters that decide output quality are set explicitly**, because the defaults produce reasoning instead of content and penalise the repetition a glossary requires; **a measurement means something only under the eval conditions** — temperature zero, draft head off, no expert offload — and a figure from any other run is not a figure.

## Dependency Versions

> **AXIOM — Query live state BEFORE asserting any claim about an external dependency, this workspace's own dependency graph, an external tool's flags or behaviour, this repository's VCS state, or an upstream issue's status. Memory is stale — and so is any tool blind to the category you are asking about.**
>
> **Each category has its own recipe, and the command must reach the CATEGORY or its exit 0 is about a different question than yours:** a crate's published versions (`cargo search`, `cargo add --dry-run`); whether `X` is a dependency here (`grep` the manifests **AND** `cargo tree -i <crate>` for transitive reach); which version the build actually resolves (the lockfile, or `cargo tree`); an external tool's flag (`<tool> --help` or run it — **never** from memory); a file's tracked / ignored / on-disk status (a category-matched `git` command — `git status` is **blind to ignored files**, so empty output is never proof of absence); an upstream issue's state (`gh issue view <N> --repo <owner>/<repo> --json state,comments` — the body is frozen, the **closing comment** carries the resolution).
>
> **Full per-category recipes: [`ai-docs/dependency-versions.md`](ai-docs/dependency-versions.md).**
>
> If your draft contains substrings like *"would add"*, *"introduce X as a dep"*, *"pull in X"*, *"avoid X as a dep"*, *"X is not currently a dependency"*, *"supports `--flag`"*, *"takes `--flag`"*, *"is committed"*, *"is tracked"*, *"is gitignored"*, *"there are no"*, *"still affects"*, *"is unfixed"* — **STOP**, run the relevant check, and either rewrite with the verified fact or drop the claim.

> **AXIOM — Established crates and the standard library first. Hand-rolling is a decision that must be ARGUED, and two arguments are refused outright.**
>
> Fewer dependencies is better, and that cuts against writing your own just as hard: code you hand-rolled is a dependency this project owns, tests and carries forever, while a settled crate is one the ecosystem already tests. **"Prefer the standard library" means the standard library over a third-party crate — never *the standard library plus your own implementation* over an established one.**
>
> | Argument offered for hand-rolling | Standing |
> |---|---|
> | *"It's only 10–20 lines — cheaper than writing the import"* | **REFUSED.** Line count is not the cost. Ownership is: the edge cases not hit yet, plus every test and review round each of them buys. |
> | *"Better to write our own than to pull in an established dependency"* | **REFUSED.** Dependency aversion is not a reason by itself. A maintained, widely-used crate is the default, not the concession. |
> | The crate is unmaintained or abandoned, or its API cannot express the requirement | **A real argument** — make it in the design document, naming the crate and the specific mismatch. |
> | A rejected-alternatives comparison: what was evaluated, why each lost, what the escape hatch is | **The standard an argued wheel meets.** |

When changing dependencies: **never hand-edit a version in `Cargo.lock`** — `cargo add <crate>@<version>`, then build, then `make lock-check`, then read `git diff Cargo.toml Cargo.lock` **before staging**. A new dependency needs a stated reason in the design document, and so does hand-rolling in place of one.

## Workflow

> **AXIOM 1 — NEVER edit on local `master` when work is intended for a pull request.**
> Create a feature branch (`git checkout -b feat/...` or `chore/...`) **before** any file edit — not before commit, **before edit**.
>
> | If `git branch --show-current` returns... | Action |
> |---|---|
> | `master` AND you're about to make an edit meant for a pull request | **STOP**. Run `git checkout -b <prefix>/<descriptive-name>` first. Only then edit. |
> | A feature branch | Proceed with edits |
> | `master` AND you've already made commits (recovery) | `git stash` → `git checkout -b <feature>` → `git checkout master && git reset --soft origin/master && git restore --staged .` → push the feature branch → open the pull request. Pop the stash on the feature branch if needed. |
>
> The first action of any skill or workflow that produces commits is `git branch --show-current`; if `master`, switch **before** any `Edit`/`Write`. Before any `git push`, confirm again. A `PreToolUse` hook refuses a commit made on `master`, and that hook plus this rule are what you actually follow — a ruleset is a live setting that can be relaxed without this file changing.

- Merge pull requests via merge commit (`gh pr merge --merge`); squash and rebase are disabled for `master`, and the branch is deleted on merge.
- Run `cargo build --workspace --all-targets` before commit; run `make lock-check` whenever dependencies moved.
- Stage explicitly; **never** `git add -A` / `git add .` — the working tree holds ignored local state and gate logs.
- **Never** `git commit --no-verify` (or any hook-skip flag) — fix the hook.
- **`gh … --body` vs the commit-block hook.** The commit-block hook matches `git[[:space:]]+commit`, so a `gh issue create` / `gh pr create` / `gh pr comment` invocation whose `--body` argument *contains* that substring is falsely blocked — use `--body-file <path>` instead of inlining the body.
- **NEVER** batch a `git commit` or a data-dependent `AskUserQuestion` in the same turn as the `Edit` or subagent call producing its inputs; verify with `git diff --cached --stat` first.
- **Before every `git commit` during a task**, stage `ai-docs/learnings.md` with the related change — learnings are part of the deliverable and must be visible in the pull request's diff. **After a push**, a new learning entry gets its own commit.
- **Delegation has FIVE phases, and failures land where attention is thinnest** — outbound (**every load-bearing claim you put INTO the prompt is executed first, whoever authored it**), fit (charter *and* environment), hand-off (leave the index CLEAN, or your staged work lands in the delegate's commit), while-it-runs (a delegate waiting on a long job is waiting, not stuck), and return (**a return summary is a claim, not a record**). Read [`ai-docs/delegation-rules.md`](ai-docs/delegation-rules.md) before any spawn that commits, edits protected files, or runs long.
- **No "too simple" step-skip in `/task`.** Steps 6 / 7 / 10 are MANDATORY; the owner's authorisation is the only bypass.
- **CI-fix commits get self-review too** — `/pr-ci-failed` and `/master-ci-failed` each run it before their push.
- **NEVER** `git reset --hard` — it discards uncommitted work. The same hazard applies to `git checkout -- <file>` and `git restore <file>`: both restore the *whole* working-tree file to HEAD, silently dropping every uncommitted edit to it. To undo a probe edit, use a copy under `tmp/`, then re-verify with `git diff --name-only <base>`.
- Plan first. Tests before production code. Lint changed files.
- Any file with ~50+ lines of substantial logic MUST have tests.
- After generating or moving a markdown file with relative links, trace one link with `realpath` before committing.
- **Pull-request review comment resolution:** resolve only comments fixed by code; objections stay open for the reviewer.

> **AXIOM 2 — Read the pull-request body via `gh pr view <N>` after EVERY `git push` to a feature branch with an open pull request. Unconditional.**
> The READ is mandatory even for a routine typo or format push. The EDIT is conditional — only when the body contradicts the new commits.
>
> | After... | Required action |
> |---|---|
> | `git push` to a feature branch with an open pull request | Run `gh pr view <N> --json title,body` immediately. Read the body. |
> | The body still describes the diff accurately | No `gh pr edit` needed — the read is complete |
> | The body contradicts the new commits (renames, scope drift, criterion flips, cited counts) | Run `gh pr edit` to sync |
> | `gh pr create` immediately preceded the push | **Skip** the read — the body is what you just authored. The rule fires on the **next** push. |

> **AXIOM — Every code-producing commit on a feature branch with an open (or about-to-be-opened) pull request must pass a self-review before `git push`.**
> **Carve-out — Step-8 branch pushes are visibility, not presentation.** During `/task` Step 8 the branch is pushed from the first group return onward with **no pull request existing yet**; self-review gates **pull-request creation** (Step 12, after APPROVE), and CI on the pull request is the FINAL gate — it confirms the locally-green tree is green in the reference environment, it does not hunt defects the loop should have found.
> Named instances: `/task` Step 10, `/bugfix` Step 6, `/project-review` Step 5. The enumeration is a list of *named* instances, **never** the only covered surfaces. Full matrix: [`.claude/agents/self-review.md` § When self-review applies](.claude/agents/self-review.md). An unnamed surface is covered when it **either** ships executable code (a hook body, a script, Rust code) **or** changes an instruction-file rule that other surfaces must obey — "no `.rs` diff" is never the test.
>
> APPROVE = push. REJECT = fix on the same branch and re-run; after 3 REJECTs in a row, surface and stop without pushing.

> **AXIOM — `ai-docs/deferred/_inbox.jsonl` is written ONLY by `/task` Step 12 and `/triage`.**
> A hand-edit hides rows from the parser and collides with future appends; one malformed line breaks the whole read. Row shape: [`ai-docs/templates/inbox-row.md`](ai-docs/templates/inbox-row.md).

## Propagation Rule

> **AXIOM — Edits to one instruction file MUST propagate to its sync-group siblings in the SAME pull request.**
>
> | If you edit... | You MUST also check / update... |
> |---|---|
> | Any `.claude/agents/*.md` or `.claude/skills/**` file in a declared sync group | Apply the same change to its siblings — the group table lives in [`ai-docs/propagation-groups.md`](ai-docs/propagation-groups.md). |
> | `AGENTS.md` (rule add / exemption) | Run `grep -rni "<changed-keyword>" .claude/ AGENTS.md ai-docs/` and apply the same change to every match. |
> | Any edit that changes a Tool/Subagent/Skill/Hook contract | Update [`ai-docs/claude-tools-hierarchy.md`](ai-docs/claude-tools-hierarchy.md) in the same pull request. |
> | `AGENTS.md` § *Learning Log* (boundary rules, entry format, `Kind:` / `Escalated?` semantics) | `.claude/agents/self-improve.md` AND `.claude/agents/learnings-escalation-audit.md` (Learning-Log group) |
> | A hook body in `.claude/settings.json` | Re-verify it fires per [`ai-docs/hook-verification.md`](ai-docs/hook-verification.md), and update the rule text in `AGENTS.md` that the hook backs. |
> | Any other instruction file | Run the same grep — the Procedure below catches lingering references. |

**Procedure:**
1. Before closing the edit, `grep -rni "<changed-keyword>" .claude/ AGENTS.md ai-docs/` for any file referencing the same rule or terminology. **`-i` is not optional** — a sweep over prose is case-insensitive or it under-reports. Corollary: **a file you have already edited is not thereby done** — re-grep it whole, after the edit.
2. Apply the same change (or the corresponding enforcement adjustment) in every match.
3. Rule exemptions must propagate to the checklists that enforce the rule.
4. When the change propagates a **factual or policy claim** (a version, a CI-gate status, a "the repository does X" statement) rather than a rule keyword, the step-1 grep set is necessary but not sufficient — also sweep the user-facing documents (`README.md`, `docs/**`) for the same claim. Every LIVE document must agree; history surfaces (`ai-docs/learnings.md`, `ai-docs/plans/done/**`) are left untouched.
5. **Bind the check to the COMMIT, not to the fix.** Before `git add`, ask one mechanical question of the staged set: *does this diff change a behaviour some durable document states, and is that document in this staged set?* The act that discharges this is **reading a match list**, never recalling where a claim lives — so `grep -rn` the document set for the **key term of the behaviour** being changed, read the matches, and re-run it after the edit: its empty output is the evidence. Then `git show --stat` the commit before considering it done.

Do not refer to a skill as an "agent" or vice versa — the distinction matters for spawning. (`project-review` is a skill; `review-findings` and `self-review` are agents it spawns.)

## Communication

Interpret the owner's phrasing literally and conservatively. When uncertain — ask, don't guess.

- **"Submit / push to PR"** = `git push` the branch so the commits appear in the open pull request. **NOT** `gh pr merge`. Only merge when the owner explicitly says "merge".
- **"wtf?" / "what?" / "huh?"** (or similar surprise) = the previous action was the opposite of what was wanted. **Stop immediately**, do not retry, ask what was wrong before doing anything else.
- **IDE files** (`.idea/`, `.vscode/`, `*.swp`) — never add, remove, modify, stage or ignore them unless the owner explicitly asks.
- **A verbal acknowledgement is not a fix.** When the owner corrects a fact — especially one already written into a file — the correction is a **work item**, not a conversational beat. Reply **and**, in the same turn, `grep` the artefact for the wrong claim and edit it. Tell: any reply containing *"fair"*, *"good point"*, *"you're right"* that is **not accompanied by an `Edit`** to whatever asserts the now-refuted thing.
- **A recorded result is a claim, not a completion.** A sentence asserting your *own* work-state — "verified", "confirmed", "gate passed", "done" — written to any durable surface is a **timestamped claim**, not a standing fact. Re-run the underlying check *after the LAST edit of the turn*, immediately before recording — never record-then-edit.
- **A citation offered as authority is itself a claim — open it.** Before invoking a rule, a table row, a learnings entry, a date or a `file:line` as the reason for an action *or an inaction*, resolve it and confirm it says what you are citing it for. Special force when the cited rule lives in the file you are editing. A reviewer's reading of a rule is an argument, not the rule: verify a **permissive** reading harder than a restrictive one.
- **A bound is not a target; a limit that is not a gate is a limit.** "Never weaker" authorises staying put, not movement. Tightening a rule nobody asked to tighten is unapproved scope exactly as loosening one is.
- **A correction propagates to every delegate that received the original — in the same turn.**
- **Deviating from approved scope requires an ask, not a notification.** *"I also did X — say the word if you'd rather I revert"* puts the burden of catching scope drift on the owner. Ask **before** widening scope, even when the argument is compelling. **An ask is never a route into the spec:** the owner's "yes" to extra work authorises that work in the design, recorded with their words; it becomes no Scope item, Key decision or acceptance criterion.

## Patterns

### 1. Verify an intermediary's claims — findings, retractions, premises, wave-throughs — as skeptically as each other

*Prefer* verifying a reviewer's *retraction*, *salvage suggestion*, and *"leave it / harmless / follow-up"* call with the same command you would run against its original finding. A retraction is an assertion; a proposed fix is a claim that the fix works; a "harmless" ruling is a claim about harm.

**The asymmetry to resist.** A finding feels like a challenge and invites checking, while a withdrawal or a wave-through feels like *relief* and invites acceptance — which is exactly when an unverified claim slips through, because agreeing costs nothing in the moment. *Prefer* overriding a reviewer only in the direction of **more** verification: declining a suggested fix because you tested it and it fails is sound; accepting one because it sounds right is not.

**Not only reviewers — any intermediary.** The same posture applies to a *delegate's* claims. *Default to* verifying a delegate's design-blocking STOP with a command — build a reduced repro, read the cited code — before amending a design. A delegate that **reports its own defect** has earned trust about the *symptom*, never about the *cause*: re-resolve its cited coordinates before authorising the fix, and extend the check to the neighbours it did not mention. And a **design's stated consequence** is a claim of exactly the same kind as a reviewer's finding: execute the path it describes against the shipped code before copying the sentence onto a second live surface.

### 2. A green instrument is a claim about the instrument until you have seen it go red

*Prefer* treating a clean result from any verification apparatus — a control, a negative test, a baseline, a guard suite, a set intersection — as evidence about the **apparatus** first and the **subject** second. A genuinely absent effect and an instrument that cannot detect one look identical from the result alone, and only one of them is worth acting on.

**Three shapes, each cheap to rule out.** *Prefer* running a changed guard's own fixtures against the **pre-change** artefact (`git show HEAD:<file>`) — the set of rows that flip, and only that set, is what distinguishes a load-bearing edit from a tautological test. *Prefer* reading the **cardinality of every input** before the verdict of any check shaped as *intersect two sets* / *diff against a baseline* / *grep a corpus* — an empty right-hand side makes `comm -12` and `grep -f` report the clean answer for every possible left-hand side. And where a control comes back clean across **every** variation tried, spend the next step on the channel rather than the conclusion.

**When the instrument is a TEST.** A test earns its name as a criterion's verifier only after its assertion has been pointed at the broken mechanism and seen RED. Two failure modes recur and both look like coverage — a fixture configured where the two branches coincide, and a test that hand-builds the object under test instead of going through the wiring that constructs it. Confirm the mutant **builds** as a separate step before reading the test result: a mutant that changes the shape of the program tests the compiler, not the suite.

**And in its RED direction.** Treat **any** result the apparatus could have produced regardless of the subject as evidence about the apparatus — a RED whose text describes the *instrument* qualifies exactly as an empty set does. A guard can only report the failure mode it was taught to name, so *"scan clean"* beside a red result is the shape that most invites fixing the wrong thing.

## Agent Docs

Read on nearly every task:

| Path | Purpose |
|------|---------|
| [`ai-docs/context.md`](ai-docs/context.md) | Project orientation |
| [`ai-docs/domain-invariants.md`](ai-docs/domain-invariants.md) | Cache keys, retrieval, request parameters, validation, eval conditions |
| [`ai-docs/code-style.md`](ai-docs/code-style.md) | Rust code-style reference |
| [`ai-docs/rust-test-conventions.md`](ai-docs/rust-test-conventions.md) | Where a test lives, case tables, exact assertions, the real database |
| [`ai-docs/learnings.md`](ai-docs/learnings.md) | Corrections log — feed for `/improve` |

**Every other page — key decisions, naming, doc convention, dependency recipes, delegation, hook verification, writing style, tool inventory, propagation groups, templates, plans, telemetry schema — is indexed in [`ai-docs/agent-docs-index.md`](ai-docs/agent-docs-index.md).**

## Learning Log

On **ANY** instruction violation, write a new entry to `ai-docs/learnings.md` — there is no "obvious", "minor", "trivial", "already-known" or "duplicate" disposition. The history (including recurrences and superseded entries) is the artefact `/improve` audits to decide escalation fan-out. See [`ai-docs/corrections-log.md`](ai-docs/corrections-log.md) for the enumerated skip-reasons that are explicitly disallowed. **Read the two boundary rules below before you write.**

**Two logs, one genre each.** `ai-docs/learnings.md` holds **conduct corrections and validations**; it is `/improve`'s input. A harness diagnosis (a gap or defect in an instruction file) goes to [`ai-docs/harness-gaps.md`](ai-docs/harness-gaps.md) — same skeleton plus a required `target:` field. Two closing fields record how an entry was dealt with, and only a forge branch writes either (CI gates it): `**Forge:**` names the forge that took the entry, `**Closed by:**` the pull request that fixed it in full outside any forge. An entry about **another entry** belongs in neither: use the original's `Superseded by:` field. Misfiled genre = violation.

### Boundary rule 1 — `ai-docs/learnings.md` is APPEND-ONLY

> **NEVER** edit, rewrite, reorder, summarise or delete an existing entry. Only append new entries at the end. This applies even when:
> - a newer correction supersedes an older one — write a NEW entry that says so, leave the old one intact
> - an entry turns out to be wrong, redundant or poorly worded — write a NEW entry that corrects it
> - you are tempted to "tidy up" or "consolidate" the file
>
> **Exception — `Escalated?` and `Superseded by:` fields, subagent-driven only.** Both MAY be updated in place by the `self-improve` subagent (`/improve`) and the `learnings-escalation-audit` subagent (`/ai-audit` Phase 1). All other lines of an entry remain immutable.

### Boundary rule 2 — writing to `learnings.md` triggers NO other rule-file edits in the same turn

> When you write to `ai-docs/learnings.md`, you **MUST NOT** also edit `AGENTS.md`, `CLAUDE.md`, `.claude/**`, `ai-docs/code-style.md` or `ai-docs/doc-convention.md` in the same conversation turn.
>
> Writing a learning entry is **NOT** authorisation to escalate the rule into instruction files. Set `Escalated? no` and stop. Project-level escalation happens only when the owner runs `/improve`, or explicitly asks.
>
> **Carve-out:** appending to `ai-docs/harness-gaps.md` is NOT an instruction-file edit — it is the designated parking surface for harness diagnoses.
>
> **Machine-enforced while an interview is live.** A `PreToolUse` hook blocks any `Edit`/`Write` to the files this rule names, and any `Bash` command that writes into one, while an `ai-docs/plans/*.spec.md.state.md` exists on the branch and `ai-docs/plans/.task-inflight` does not — between round 1 of `/interview` and `/task` Step 8. The owner's *"the instruction should say X"* is a diagnosis for `harness-gaps.md`, not a licence. Steps 8–12 carry the marker and are exempt; `/improve` runs on a branch with no state file and is untouched.

### Entry format

**Copyable skeleton and a filled example: [`ai-docs/templates/learnings-entry.md`](ai-docs/templates/learnings-entry.md) — consult that template to inspect the format, NOT the live log.** An entry is a `### YYYY-MM-DD — [category] — [short description]` heading followed by `**What happened:**`, `**Rule:**`, optional `**Kind:**`, `**Escalated?**`, and optional `**Superseded by:**`.

`Kind:` defaults to `correction` (a violation to stop doing); write `Kind: validation` for a working protocol to keep doing. `Escalated?` records **project-level** persistence only — user-local auto-memory does **not** count → stays `no`.

Categories: `code-style` | `process` | `architecture` | `testing` | `documentation` | `tooling` | `search` | `other`

Run `/improve` when **≥3 unescalated correction entries**, **≥2 unescalated validation entries**, or a stale-validation flag from `/ai-audit` accumulates.

## Test Conventions

- **Unit tests live in the file they test**, inside its `#[cfg(test)] mod tests`; an integration test lives under the crate's `tests/` and sees only the public surface. Test names describe behaviour: `rejects_empty_translation`.
- **A case table is the default shape** when there are more than two cases: every case carries a name, and the failure message names the case.
- **Concurrency is exercised, not assumed.** A diff that spawns a task, adds a channel or shares state behind a lock carries a test that drives the concurrent path **and** its cancellation. Rust rules out the data race; it rules out neither a lost cancellation nor a task nobody awaits.
- **No unmarked panicking call in shipped code.** `make panic-calls` decides the marker; a surviving call carries its reason in prose on its own line or the line above **and** a row in [`ai-docs/panic-index.md`](ai-docs/panic-index.md). The comment states the justification itself and does not point at the index — a comment naming a markdown path is what the reference ban forbids. A binary's `main` may exit non-zero; everything else returns errors.
- **Determinism is testable, so test it exactly.** Segmentation, alignment, prompt rendering and cache-key derivation assert exact outputs against recorded fixtures.
- **The model is never the oracle.** A test whose verdict depends on what the model returns is an eval, marked as one and held to the eval conditions; a unit test stands on a recorded fixture that names the model and the parameters that produced it.
- **Postgres is tested against Postgres**, not a mock: a `CHECK`, a unique index and the vector index's behaviour are database behaviour.
- A search miss on a construct that SHOULD exist is a **search-method failure first** ([`.claude/rules/ast-index.md`](.claude/rules/ast-index.md) → *Negative results are NOT evidence*).

Detail and worked examples: [`ai-docs/rust-test-conventions.md`](ai-docs/rust-test-conventions.md).
