# API naming

Rules behind `AGENTS.md` § *API Naming*. The Rust API guidelines are the baseline; what follows is what this project adds or pins.

## The `_unchecked` AXIOM

> **A function that skips a validity check its sibling performs MUST carry the `_unchecked` suffix, and its doc comment MUST state (a) the precondition, and (b) who guarantees it.**
>
> | Shape | Verdict |
> |---|---|
> | `fn translate_unchecked(&self, p: &Paragraph) -> String` with a doc comment naming the validation that already ran | Correct |
> | `fn translate(&self, …)` that silently assumes the caller validated | **Wrong** — rename or check |
> | `*_unchecked` whose doc comment does not name the guarantor | **Wrong** — the comment is the contract |
>
> The suffix is a warning to the reader, not an optimisation badge. If no measurement showed the check mattered, delete the unchecked variant and keep one honest function.
>
> **Naming the guarantor under the comment-reference ban.** The guarantor is named the way the ban allows: a symbol in the comment's own crate is written as the symbol, because a symbol that *is* the contract is not an outward reference. A guarantor in **another crate of this workspace** is described rather than written as a path — "the caller that has already validated the response", not the qualified name. The AXIOM is unchanged by this: the comment still states the precondition and still says who guarantees it. See [`doc-convention.md`](doc-convention.md) § DOC-4.

An `unsafe fn` carries a `# Safety` section stating what the caller must uphold; `_unchecked` and `unsafe` are different obligations and a function can carry both.

## Naming rules

- **No stutter.** `cache::Key`, not `cache::CacheKey`. `pipeline::Stage`, not `pipeline::PipelineStage`. The module is part of the path at the call site.
- **Constructors** are `new` for the obvious one, `with_<thing>` for a variant, `from_<source>` for a conversion that cannot fail, and `try_from` / `try_new` for one that can.
- **Accessors carry no `get_`**: `paragraph.id()`, not `paragraph.get_id()`. A setter is `set_<field>`.
- **Conversions follow the standard conventions**: `as_` borrows, `to_` allocates, `into_` consumes.
- **Errors are enums with one variant per failing operation**, named for what failed rather than for the type that failed. A variant preserves its source; a caller that must distinguish two failures matches a variant, never a message.
- **Traits are named for behaviour** (`Translator`, `Embedder`, `Store`) and **declared where they are consumed**, in the crate that calls them, when the point of the trait is that the caller varies. A trait shipped beside its single implementation is usually a struct wanting to be a struct.
- **Abbreviations are lowercase in identifiers**: `http_client`, `book_id`, `ws_message` — following the standard library rather than inventing a second convention.
- **A future or a stream returned from a public API is named for what it yields**, and the doc comment says whether it is cancel-safe. That sentence is part of the contract, not decoration.
- **Domain vocabulary is the corpus's vocabulary.** The corpus under `docs/` is Russian and the code is English; the mapping is fixed once, in the crate that owns the concept — `book`, `chapter`, `paragraph`, `translation`, `glossary`, `context version`, `compaction`, `prefetch window`, `alignment`. Do not invent a second English word for a concept that already has one in the code.
