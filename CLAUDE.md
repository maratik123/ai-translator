# ai-translator

Читалка книг с параллельным переводом локальной LLM. Ценность проекта — в конвейере
качества перевода, а не в UI. Полное описание в `docs/`, начинать с `docs/ARCHITECTURE.md`.

**Языки:** en, fr, es, it, de → ru.
**MVP:** библиотека `core` + `reader-cli` (`docs/11-core-api-and-cli.md`), шаги 1-4 из
«Порядок реализации» в `docs/09-build-and-deploy.md`. Сервер и фронт — после.

## Статус
Окружение собрано и проверено замерами. Кода ещё нет: `crates/` пустой.
Следующий шаг — каркас workspace, схема БД, импорт epub до работающего `reader-cli import`.

## Локальное окружение (проверено)

| Компонент | Значение |
|---|---|
| Rust | 1.98.1 |
| llama.cpp | `sci-misc/llama-cpp` b10927 из guru, USE `vulkan rocm wmma curl openmp` |
| GPU | RX 9070 XT (gfx1201), 16 ГБ; ROCm 7.2 видит карту нативно |
| CPU | Ryzen 7 5800X, Zen 3, AVX2, без AVX512; DDR4 |
| Postgres | 18.6 + pgvector 0.8.6 (локальный ebuild в оверлее `local-syt`) |
| БД | `postgres://reader:reader@127.0.0.1/reader`, расширение `vector` создано |
| Тесты | testcontainers через Podman socket, образ `pgvector/pgvector:pg18` |

`models/` и `examples/` в `.gitignore` — файлы только локальные.

## Запуск моделей

Перевод (97 с на книгу из 92 абзацев):
```bash
llama-server \
  -m models/ggml-org/gemma-4-26B-A4B-it-GGUF/gemma-4-26B-A4B-it-Q4_0.gguf \
  -md models/ggml-org/gemma-4-26B-A4B-it-GGUF/mtp-gemma-4-26B-A4B-it-Q4_0.gguf \
  --spec-type draft-mtp --spec-draft-n-max 2 \
  -dev Vulkan0 -c 16384 -ngl 31 \
  -ot "blk\.28\.ffn_(gate|gate_up|down).*=CPU,blk\.29\.ffn_(up|down|gate_up|gate)_(ch|)exps=CPU,blk\.30\.ffn_(up|down|gate_up|gate)_(ch|)exps=CPU" \
  -ctk q8_0 -ctv q8_0 -fa on -t 8 -np 1 --load-mode none \
  --metrics --host 127.0.0.1 --port 8080
```

Эмбеддинги (на CPU, 10 с на книгу, нулевая VRAM):
```bash
llama-server -m models/Qwen/Qwen3-Embedding-0.6B-GGUF/Qwen3-Embedding-0.6B-Q8_0.gguf \
  --embedding --pooling last -ub 512 -c 2048 -np 1 \
  -dev none -ngl 0 -t 8 --host 127.0.0.1 --port 8081
```

Гасить: `pkill -x llama-server`. **Не** `pkill -f "llama-server -m models"` — шаблон
совпадёт с собственной командной строкой и убьёт вызывающий процесс.

Раскладку под новую модель считать не руками, а `llama-fit-params` с теми же
аргументами, что у сервера: он печатает готовые `-ngl` и `-ot`.

## Что легко сделать неправильно

- **`-dev Vulkan0` обязателен.** Сборка с двумя бэкендами показывает одну карту дважды
  (`ROCm0` и `Vulkan0`); без `-dev` llama.cpp разложит модель по «двум» устройствам.
- **`chat_template_kwargs: {"enable_thinking": false}` в каждом запросе.** Иначе Gemma 4
  уходит в `reasoning_content`, `content` остаётся пустым, упирается в `max_tokens`.
- **`repeat_penalty` ставить 1.0 явно.** Штраф за повтор бьёт по глоссарию: имена
  персонажей и термины обязаны повторяться.
- **`--pooling last` для эмбеддингов**, не `cls`. Векторы уже нормализованы.
- **`eval`/`compare` только на `temperature 0` и с выключенным MTP.** Сервер
  недетерминирован при сэмплировании даже с фиксированным seed (расхождение ~12%),
  MTP добавляет ещё ~24%. Иначе измеряется шум, а не эффект промпта.

## Принятые решения с обоснованием в доках

- `llama-server` напрямую, не Ollama (`docs/05`): Ollama не отдаёт управление MoE-офлоадом.
- Vulkan основной, ROCm собран рядом для сравнения (`docs/05`).
- Модель `ggml-org/gemma-4-26B-A4B-it` Q4_0; расцензуренная сборка проверена и отклонена —
  базовая не отказывает, даёт тот же регистр, меньше ошибок и быстрее (`docs/05`).
- `context_version` не входит в `cache_key` (`docs/03`).
- `retrieve` берёт top-k, а не порог по расстоянию — порог откалиброван и отбрасывал
  верные соседи (`docs/04`).
