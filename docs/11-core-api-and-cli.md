# 11. Ядро как библиотека и CLI

## Принцип
Конвейер перевода это продукт, читалка это один из клиентов. `core` не знает о транспорте и UI, наружу отдает методы и поток событий. Сервер и CLI одинаково тонкие.

## Задачи: core
- [ ] Крейт `core` с публичным `Engine`:
```rust
pub struct Engine { store: Arc<dyn Store>, llm: LlmClient, embed: Option<EmbedClient>, cfg: EngineConfig }

impl Engine {
    pub async fn import(&self, path: &Path, opts: ImportOpts) -> Result<BookId>;
    pub async fn book(&self, id: BookId) -> Result<BookInfo>;
    pub async fn chapter(&self, id: BookId, idx: u32) -> Result<ChapterView>;   // абзацы + переводы
    pub fn scheduler(&self) -> SchedulerHandle;                                 // set_position, translate_range, translate_book, cancel
    pub fn events(&self) -> broadcast::Receiver<Event>;                         // ParagraphTranslated, ContextCompacted, Progress, LlmStatus, Failed
    pub async fn export(&self, id: BookId, fmt: ExportFormat, out: &Path) -> Result<()>;
    pub async fn context(&self, id: BookId) -> Result<ContextSnapshot>;         // + update_characters(...)
}
```
- [ ] Трейт `Store` (books, chapters, paragraphs, translations, context, embeddings, annotations, positions). Реализации: `PgStore` (основная), `MemStore` (тесты и eval без БД).
- [ ] Воркер перевода запускается внутри `Engine::start()`, живет пока жив `Engine`; один воркер на инстанс.
- [ ] `export`: двуязычный epub (после каждого абзаца оригинала блок перевода с классом `.tr`, CSS в ресурсах), только перевод, markdown для диффов.
- [ ] Ошибки через `thiserror`, без `anyhow` в публичном API.
- [ ] Модули из `backend/ARCHITECTURE.md` (`import`, `storage`, `translate`, `llm`, `embed`) переезжают в `core` как есть; в `server` остаются только `http/` и `ws/`.

## Задачи: CLI (`reader-cli`)
- [ ] `clap`, конфиг тот же TOML, что у сервера, `DATABASE_URL` из env.
- [ ] `import <file>` → id книги.
- [ ] `translate <book> [--chapters 1..3] [--model ...] [--prompt-version N]` с прогресс-баром (`indicatif`), ток/с, ETA; Ctrl-C сохраняет сделанное.
- [ ] `export <book> --bilingual out.epub | --translation-only | --md`.
- [ ] `context <book> [show | set-narrator f | set-character Tom m Том]`.
- [ ] `eval <suite.toml> --models a,b --prompt-versions 1,2 --lang en,fr,es,it,de` → таблица метрик. Наборы ловушек по языкам из `10-gender-and-coreference.md`.
- [ ] `compare <book> --chapter 3 --models a,b [--backend vulkan,rocm] [--mtp on,off]` → markdown с абзацами в колонки для слепой оценки плюс строка производительности на каждую комбинацию.
- [ ] `status` → очередь, модель, версия контекста.

## Условия воспроизводимости
`eval` и `compare` запускаются с `temperature 0` (`top_k 1`) и **выключенным MTP**. Причина в `05-llm-client.md`: при сэмплировании сервер недетерминирован даже с фиксированным seed (расхождение ~12%), а MTP меняет вывод ещё на ~24% из-за другой формы батча. Без этих двух условий измеряется шум, а не эффект промпта.

## Метрики eval
Базовые:
- доля верного рода при первом ответе и после ретрая (отдельно для первого лица, отдельно по языку);
- доля верного «ты/вы» там, где форма извлекается из оригинала (FR/ES/IT/DE);
- расхождение по числу предложений, средняя длина, ток/с.

Добавлены по итогам разбора моделей (см. `05-llm-client.md`):
- **доля schema-valid ответов на компактификации** за N вызовов — здесь всплывает деградация следования инструкциям у расцензуренных сборок;
- **доля ответов с преамбулой** («Here is the translation», рассуждение перед ответом) — модели, дообученные «рассуждать перед ответом», поднимают долю ретраев;
- **дрейф регистра**: доля мест, где грубый оригинал переведён нейтральной лексикой. Считается по словарю пар «источник → ожидаемый регистр» для книги; отдельная метрика от отказов, потому что отказов может не быть вовсе при заметном смягчении;
- **доля слов вне словаря** — опечатки, окказионализмы и искажённые фразы. На первом замере оказалась продуктивнее проверки согласования: 5 настоящих ошибок против 0;
- **согласование внутри именной группы** — доля ошибок вида «мою анус». Контекст не нужен, метрика дешёвая и ловит то, что остальные проверки пропускают;
- **доля смягчений** на трудных абзацах (насилие, секс, брань). Тихое смягчение проходит все текущие проверки `validate`, поэтому нужен отдельный прокси: детектор отказных формул на русском и на языке оригинала плюс обвал отношения длин оригинал/перевод;
- **доля принятых черновых токенов** из `llamacpp:spec_decode_num_accepted_tokens_per_pos_total` — падает, если MTP-голова взята от немодифицированной модели, а целевая дообучена.

Каждый прогон расцензуренной сборки сравнивается с немодифицированным контролем на той же базе (`google/gemma-4-26B-A4B-it-qat-q4_0-gguf`): разница показывает, что именно расцензуривание дало и что забрало.

## Взаимодействие CLI и читалки
Общая БД: книга, импортированная и переведенная через CLI ночью, открывается в читалке уже готовой; читалка при чтении досчитывает то, чего нет. Ключ кэша один и тот же.
