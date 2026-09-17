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
- [ ] `eval <suite.toml> --models a,b --prompt-versions 1,2` → таблица: доля верного рода, расхождение по числу предложений, средняя длина, ток/с. Набор из `10-gender-and-coreference.md`.
- [ ] `compare <book> --chapter 3 --models a,b` → markdown с абзацами в три колонки для слепой оценки.
- [ ] `status` → очередь, модель, версия контекста.

## Взаимодействие CLI и читалки
Общая БД: книга, импортированная и переведенная через CLI ночью, открывается в читалке уже готовой; читалка при чтении досчитывает то, чего нет. Ключ кэша один и тот же.
