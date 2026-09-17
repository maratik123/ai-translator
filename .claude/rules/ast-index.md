# ast-index Rules

## Mandatory Search Rules

1. **ALWAYS use ast-index FIRST** for any code search task
2. **NEVER duplicate results** — if ast-index found usages/implementations, that IS the complete answer
3. **DO NOT run grep "for completeness"** after ast-index returns results
4. **Use grep/Search ONLY when:**
   - ast-index returns empty results
   - Searching for regex patterns (ast-index uses literal match)
   - Searching for string literals inside code (`"some text"`)
   - Searching in comments content
   - Searching a surface the index does not cover. **Measured on a probe tree holding one file per extension: `.rs`, `.sql`, `.sh` and `.ts` are indexed; `.md`, `.yml` and `.toml` are not** — so the harness corpus, the workflows and the manifests are grep's, and migrations, scripts and the reader are not. Re-measure the same way rather than trusting this line after a toolchain upgrade.

## Negative results are NOT evidence

A search that comes back **empty or short** for a construct that SHOULD exist is a
**search-method failure first, code-absence second.** Never conclude "X does not
exist" from a miss — re-run with a different method, or read the region.

| Cause of the false negative | Fix |
|---|---|
| Multi-line construct (a `rustfmt`-split signature, a struct literal with tags, a chained builder call) | `rg -U` (multiline), or read the region |
| Hand-rolled identifier class — `[a-z_]*` excludes digits and capitals, and Rust spans three conventions at once: `snake_case` for functions and locals, `CamelCase` for types and traits, `SCREAMING_SNAKE_CASE` for constants (`cache_key`, `ContextVersion`, `MAX_CONTEXT_TOKENS`) | `[A-Za-z0-9_]+`, or `ast-index symbol` / `ast-index outline`, which need no hand-written pattern |
| The symbol lives behind a `#[cfg(...)]`, in generated code, or in a file your pattern's path filter excluded | Read the file list first (`ast-index file`), then the source |
| A macro-generated item — a `derive`, a declarative macro's expansion — exists in no source file at all | Read the macro's definition, or expand the crate; a clean sweep here is about the tree, not about the program |
| Case-sensitive pattern over **prose** — instruction text, comments and headings capitalise mid-sentence words freely, so the emphatic occurrence is the one that escapes | `grep -rni` / `rg -i`; a clean sweep is evidence about your *pattern* until you have varied its case |
| A trait method searched as a declaration — the method's body lives in an `impl` block, and a blanket `impl<T: Bound> Trait for T` names no concrete type at all, so no search for the type finds it | `ast-index implementations "<Trait>"`, or search for the trait name and read its `impl` blocks |

**MUST — a claim that an API, symbol, flag, or precedent does NOT exist requires a
raw read of the source (or `cargo doc`), never a search tool's silence.**
Prescribing a *replacement* off such a negative compounds it by inventing a second
nonexistent symbol.

## Positive results are NOT evidence either

**MUST — no search result is reportable until the same pattern has been run against a constructed
string it MUST match, and seen to match.** One control line per probe, **before** the conclusion —
not after being challenged. This binds a probe you write for yourself exactly as it binds a guard in
the repository.

**MUST — the control string is CONSTRUCTED, never borrowed from the artefact under test**, and least
of all from a line this change edits: a borrowed control fails silently in exactly the runs where the
edit worked, so its empty output is equally consistent with a working instrument. **NEVER** put
`2>/dev/null` on a grep whose emptiness is the verdict — a tool refusing to run and a genuine
absence produce the same empty stdout, and the redirect is what makes them indistinguishable. Prefer
several simple patterns over one long alternation: a regex engine has complexity limits, and its
failure mode is an error you have arranged not to see. An enumeration in prose has no canonical
order, so a literal phrase carrying one is a pattern for one variant, never for the claim.

**MUST — before reading a probe's result, assert that the probe LANDED where the instrument looks,
and report where that is.** A control proves the pattern RUNS; it never proves the pattern REACHED
the subject. So the probe reports two things or it reports nothing: the region the instrument
actually judges — read from its section parser, its glob, or its index walk, never assumed — and the
evidence that the mutation or the pattern is inside that region (print the mutated line, or diff it).
Never chain a probe behind its own mutation with `&&`, which converts a failed mutation into a
skipped verification that looks like nothing happened. A probe that relocates or reshapes the
artefact tests the harness, not the subject, and a degradation notice on stderr **voids** the probe
rather than decorating it. An instrument's silence becomes evidence only after one probe has been
seen to FAIL **and** one genuine case has been seen to PASS — the second half is the one skipped once
the first probe finally fails.

**MUST — a hit proves a STRING occurs; it NEVER proves a BEHAVIOUR exists.** That a script *handles*
a flag, that a gate *fires*, that a function *does* X — each is established by running the thing,
never by matching its name.

**MUST — read the cardinality of every input BEFORE the verdict**, for any check shaped as *intersect
two sets* / *diff against a baseline* / *grep a corpus*. An empty right-hand side makes `comm -12`
and `grep -f` report the clean answer for every possible left-hand side. Likewise an
enumeration that reached nothing reports "no findings": `git ls-files`, `git diff --cached` and any
index walk say nothing about a file not yet in that set, so run such a checker AFTER `git add`, and a
NEW checker against its own new files by explicit path.

**Vary the encoding before believing a clean sweep over PROSE** — digits against spelled numerals,
leading markers against trailing ones, case, and the multi-line form. A pattern is written against
the *typical* form of its target; prose is where one encoding hides the instances.

Each pattern fails on a VARIANT of the thing sought, never on the thing itself: a bracket inside a
bracket class, a trailing marker against a leading one, a spelled numeral against a digit, an
occurrence against a behaviour, a legitimate non-zero exit against an error.

## Why ast-index

ast-index is 17–69× faster than grep (1–10 ms vs 200 ms–3 s) and returns structured, accurate results.

## Command Reference

**`ast-index --help` is the authoritative list, and it grows between releases — this table is the subset the flows lean on, not a picture of the tool.** Every row below was run against a probe tree and behaved as written.

| Task | Command |
|------|---------|
| Open an unfamiliar area | `ast-index explore "<question or bag of names>"` — ranked symbols with their source and tests, in one shot |
| Universal search | `ast-index search "query"` |
| Find struct / enum / trait | `ast-index class "Paragraph"` |
| Find symbol | `ast-index symbol "SymbolName"` |
| Resolve a file's indexed path | `ast-index file "lib.rs"` |
| File outline | `ast-index outline "src/lib.rs"` — **the indexed path, not the base name**: a bare `lib.rs` answers `File not found`, which is why the row above exists |
| Imports of a file | `ast-index imports "src/lib.rs"` |
| Definitions, imports and usages at once | `ast-index refs "SymbolName"` |
| Find usages | `ast-index usages "SymbolName"` |
| Find implementations | `ast-index implementations "Translator"` |
| Call hierarchy | `ast-index call-tree "function" --depth 3` |
| Find callers | `ast-index callers "process_paragraph"` |
| Module deps | `ast-index deps "module-name"` |
| What this branch touched | `ast-index changed` — names the branch it diffs against |
| Where things live | `ast-index map` — one line per directory with its symbol kinds |
| Open markers | `ast-index todo` |
| Candidates for deletion | `ast-index unused-symbols` — **read it as a question, never an answer**: a symbol nothing in the index calls shows up here, and the index does not see a SQL table reached from a string-built query, a shell function called by CI, or a TypeScript export consumed by the bundler |

## Rust-Specific Commands

The indexer reads Rust structurally: a `struct` is a class, an `enum` an enum, a `trait` an interface, an `impl Trait for Type` a class carrying the trait as its parent, a `macro_rules!` a function, a `mod` a package, a `use` an import — and **attributes and each derive are indexed as annotations**, which is what makes the last two rows work.

| Task | Command |
|------|---------|
| Find a struct or an enum | `ast-index class "Paragraph"` |
| Find a trait | `ast-index class "Translator"` |
| Find implementors of a trait | `ast-index implementations "Translator"` |
| Find the methods on a type | `ast-index outline "<file>.rs"` |
| Find impl blocks | `ast-index search "impl"` |
| Find macros | `ast-index search "macro_rules"` |
| Find derives | `ast-index search "#[derive"` |
| Find tests | `ast-index search "#[test]"` |

**What the index cannot show, however the query is spelled:** what a derive or a macro *generates*. The attribute is indexed; the `impl` it expands to exists in no source file, so a clean sweep for that `impl` is a fact about the tree, not about the program. `cargo expand` is what shows it.

**`annotations` is not the command for a derive here.** It answers for annotation styles this project does not use — measured against a probe carrying `#[derive(Debug)]`, `ast-index annotations "derive(Debug)"` returned nothing while `ast-index search "#[derive"` returned both derives as annotation symbols. Use `search`.

## SQL-Specific Commands

Migrations are indexed too — a `CREATE TABLE` is a class, a `CREATE FUNCTION` or `PROCEDURE` a function, a `CREATE INDEX` a property, a `CREATE TYPE` and a `CREATE DOMAIN` a class. A commented-out statement is not indexed, so a hit is a live definition.

| Task | Command |
|------|---------|
| Find a table | `ast-index class "translations"` |
| Find an index | `ast-index symbol "idx_translations_key"` |
| Find a function or a procedure | `ast-index symbol "<name>"` |
| Find where a table is touched | `ast-index usages "translations"` — and read the callers, because a query built as a string reaches no index |

All four rows were measured: a `CREATE TABLE` came back as a class, a `CREATE INDEX` as a property, a `CREATE FUNCTION` as a function, and a commented-out `CREATE TABLE` produced a content match and **no symbol** — so a symbol hit here is a live definition.

## Index Management

- `ast-index rebuild` — Full reindex (run once after clone)
- `ast-index update` — After git pull/merge
- `ast-index stats` — Show index statistics

Both are automated by `SessionStart` and pre-commit hooks in `.claude/settings.json`.
