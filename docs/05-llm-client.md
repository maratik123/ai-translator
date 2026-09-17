# 05. Клиент модели (llama-server / Ollama)

## Задачи
- [ ] `reqwest` клиент к `POST {base_url}/v1/chat/completions`, `stream: true`, парсинг SSE-чанков.
- [ ] Настройки: `base_url`, `model`, `temperature`, `max_tokens`, `timeout`, `api_key` (пусто для локального).
- [ ] Структурированный вывод для компактификации: `response_format: {type: json_schema}` (llama-server поддерживает), фолбэк на «только JSON» в промпте + `serde_json` с очисткой ```json.
- [ ] Health-check: `GET /health` (llama-server) раз в 10 с, статус в `LlmStatus`.
- [ ] Подсчет токенов из `usage` в ответе для метрик.
- [ ] Клиент эмбеддингов: `POST {embed_url}/v1/embeddings`, батчи по 32 абзаца, нормализация вектора, размерность из конфига (bge-m3: 1024).
- [ ] Совместимость с Ollama: тот же путь `/v1/chat/completions`; отличия только в имени модели и отсутствии `/health` (использовать `/api/tags`).

## Запуск llama-server (deploy/)
```
llama-server -m /models/Qwen3-30B-A3B-Q5_K_M.gguf \
  --n-gpu-layers 999 --n-cpu-moe 24 \
  -c 16384 -ctk q8_0 -ctv q8_0 -fa on \
  -t 8 -np 1 --port 8080 --host 127.0.0.1
```
- `--n-cpu-moe` подобрать так, чтобы `llama-server` показывал VRAM ~14.5 GB из 16.
- Vulkan-сборка: `cmake -DGGML_VULKAN=ON`, проверить `vulkaninfo` видит RADV.
- Для ночного режима отдельный юнит с Gemma 3 27B Q4 и меньшим контекстом.

## Эмбеддинг-сервер
```
llama-server -m /models/bge-m3-Q8_0.gguf --embedding --pooling cls -c 2048 \
  --n-gpu-layers 999 -np 4 --port 8081 --host 127.0.0.1
```
Выбор модели: bge-m3 (многоязычная, 1024) как основной вариант, nomic-embed-text-v2-moe как альтернатива. Одна и та же модель для индексации и запроса.

## Проверки перед разработкой
- [ ] Замерить ток/с на 3 кандидатах (Qwen3 30B-A3B, Mistral Small 24B, Gemma 3 27B) на одной главе, сравнить качество вслепую.
- [ ] Проверить, что кэш префикса работает: второй запрос с той же головой должен иметь `prompt_eval` заметно меньше.
