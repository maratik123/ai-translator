# Мини-модели для тестов

Две крошечные модели, на которых идут тесты конформности клиента `llama-server`.
Они закоммичены, поэтому тесты офлайновые и воспроизводимые — и локально, и в CI.
Проверяется **протокол, а не качество перевода**, поэтому 260K параметров хватает.

Раскладка повторяет `models/`, то есть как на Hugging Face:
`<organization>/<model_name>/<файлы>`. Правило `/models` из `.gitignore` сюда не
распространяется — директория названа иначе намеренно, эти файлы отслеживаются.

## Что здесь лежит

| Файл | Байт | sha256 | Происхождение | Лицензия |
|---|---|---|---|---|
| `karpathy/tinyllamas/stories260K.gguf` | 1 185 376 | `047bf46455a544931cff6fef14d7910154c56afbc23ab1c5e56a72e69912c04b` | сконвертирован нами из `karpathy/tinyllamas` | MIT, © 2023 Andrej — [LICENSE](karpathy/tinyllamas/LICENSE) |
| `ggml-org/gte-small-Q8_0-GGUF/gte-small-q8_0.gguf` | 36 685 056 | `945a2da32e7bfb2cdb3a3fde50bc40c5d103db84fedc078adf415f3bb10f2ed0` | взят без изменений из `ggml-org/gte-small-Q8_0-GGUF` | MIT, GTE от Alibaba DAMO Academy — [LICENSE](ggml-org/gte-small-Q8_0-GGUF/LICENSE) |
| `karpathy/tinyllamas/stories260K.chat-template.jinja` | 180 | — | наш файл, не из источника | лицензия репозитория |

Правообладатели у двух моделей разные, поэтому и файла лицензии два — общего «MIT»
на директорию не существует.

## LLM: почему собираем сами

У `ggml-org/models` лежит готовый `tinyllamas/stories260K.gguf` — ровно тот файл, на
котором llama.cpp гоняет собственные серверные тесты. Но на странице `ggml-org/models`
лицензия не указана вообще, а файл без заявленной лицензии в публичный репозиторий
класть нельзя. Первоисточник `karpathy/tinyllamas` — MIT, поэтому конверсия своими
руками даёт чистый провенанс. У `gte-small-Q8_0-GGUF` MIT проставлена в карточке, его
можно брать как есть.

## Как сконвертирован stories260K.gguf

Проверено 2026-09-18 на llama.cpp b10927.

1. Скачать из `karpathy/tinyllamas` (подкаталог `stories260K/`) два файла — веса и
   собственный токенизатор модели на 512 токенов:

   ```bash
   curl -LO https://huggingface.co/karpathy/tinyllamas/resolve/main/stories260K/stories260K.bin
   curl -LO https://huggingface.co/karpathy/tinyllamas/resolve/main/stories260K/tok512.bin
   ```

   Ожидаемые sha256 входов:

   ```
   b0a507e7ad0f626624f17112325e66691f9076d622e1d3274d103d00299f2696  stories260K.bin
   037cb335abb25d1fa9e8ecae30ed2a3a8ace9302862ebcdc05d51a6bbb10c312  tok512.bin
   ```

2. Собрать конвертер. Это пример из дерева llama.cpp
   (`examples/convert-llama2c-to-ggml`), и в установленном на машине разработки пакете
   его нет — нужны исходники той же версии, что и `llama-server`:

   ```bash
   tar -xzf llama-cpp-0.4.0_p10927.tar.gz     # исходники b10927
   cd llama.cpp-b10927
   cmake -B build -DCMAKE_BUILD_TYPE=Release -DLLAMA_BUILD_EXAMPLES=ON \
         -DLLAMA_BUILD_TESTS=OFF -DLLAMA_BUILD_SERVER=OFF -DLLAMA_BUILD_TOOLS=OFF \
         -DLLAMA_CURL=OFF
   cmake --build build --target llama-convert-llama2c-to-ggml -j 8
   ```

3. Сконвертировать. Вокабуляр берётся из `tok512.bin`, а не из какой-нибудь модели
   Llama: у `stories260K` свой словарь на 512 токенов.

   ```bash
   ./build/bin/llama-convert-llama2c-to-ggml \
     --copy-vocab-from-model tok512.bin \
     --llama2c-model stories260K.bin \
     --llama2c-output-model stories260K.gguf
   ```

Конверсия побайтово детерминирована: два прогона подряд дали один и тот же файл,
sha256 совпал с закоммиченным. Проверять так:

```bash
sha256sum karpathy/tinyllamas/stories260K.gguf
```

Оригинальные `stories260K.bin` и `tok512.bin` в репозиторий не кладём: они нужны
только для повторной конверсии и тянутся по ссылкам выше.

### Что важно знать о полученном файле

Конвертер записывает в метаданные длину контекста **128** токенов, хотя обучалась
модель с `max_seq_len=512`. `llama-server` это видит и обрезает слот до 128, сообщая
в лог `exceeds the training context of the model (128) - capping` (число слева от
этой фразы — запрошенный `-c`, поэтому у разных команд оно разное). Значит запрос в
тестах должен укладываться в 128 токенов вместе с ответом.

## Чат-шаблон

`stories260K` — базовая модель на TinyStories, встроенного чат-шаблона у неё нет.
Рядом лежит `stories260K.chat-template.jinja`: он просто склеивает содержимое
сообщений через пробел, без разметки ролей, — так вход остаётся внутри словаря
модели.

```bash
llama-server -m stories260K.gguf --jinja --chat-template-file stories260K.chat-template.jinja
```

**Уточнение против исходной постановки задачи:** `/v1/chat/completions` обслуживается
и без шаблона — llama.cpp b10927 подставляет встроенный ChatML. Шаблон нужен не для
того, чтобы endpoint отвечал, а чтобы промпт не раздувался: разметка ChatML в словаре
из 512 токенов раскладывается побайтово. Замерено на одном и том же сообщении
(`Once upon a time, there was a little girl named Lily.`):

| Шаблон | Токенов в промпте |
|---|---|
| наш `stories260K.chat-template.jinja` | 17 |
| встроенный ChatML (без флага) | 57 |

Это `usage.prompt_tokens` реального ответа `/v1/chat/completions`, то есть вместе с
BOS; `/tokenize` без `add_special` даёт на единицу меньше. При контексте в 128 токенов
разница в 40 токенов — это разница между тестом и его отсутствием.

## Эмбеддер

`gte-small-q8_0.gguf` взят без изменений; sha256 совпадает с тем, что Hugging Face
публикует для LFS-объекта. Размерность — 384, векторы уже нормированы (проверено:
норма 1.0). `--pooling` не задавать, читается из метаданных GGUF. Длина контекста
модели — 512 токенов.

```bash
llama-server -m gte-small-q8_0.gguf --embedding -c 512 -ub 512
```

## Как их запускает набор тестов

Только на CPU и только в контейнере `ghcr.io/ggml-org/llama.cpp:server`. К серверу,
который обслуживает приложение, тесты не обращаются никогда: он держит GPU, и
конкуренция за неё обесценивает и результат теста, и пропускную способность сервера.
Поэтому в командной строке теста — `-dev none -ngl 0` и собственный порт.
