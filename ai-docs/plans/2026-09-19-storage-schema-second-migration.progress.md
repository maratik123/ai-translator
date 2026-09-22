# Progress: Storage schema — the second migration — ACTIVE
_Updated: 2026-09-22 09:13_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-09-19-storage-schema-second-migration
**base_commit:** 1492d7b6e3c62cc631ca1efa7bedcaf211b873fc
**Last build:** PASS
**Issue:** #11
**Spec:** ai-docs/plans/2026-09-19-storage-schema-second-migration.spec.md
**current_step:** Step 8 — subtask 3 of 3 complete (Group B done; all subtasks complete)
**last_passed_gate:** make verify — fmt-check, build, clippy, doc-check, test (25: 2 lib unit + 23 database trials), lock-check, file-limits, actionlint, shellcheck, comment-refs, panic-calls, import-guard | 2026-09-22T09:13Z | 18a2311
**entry_args:** 11

## Next action

**Do this immediately:** Step 8 is complete — both groups have returned and the tree is committed. Push the branch (Step 8 visibility rule) and proceed to Step 9 — Verify.

## Subtasks

- [x] 1. The migration `crates/core/migrations/0002_storage_schema.sql`, and the embedded-set assertion that proves the build saw it
- [x] 2. The schema trials under `crates/core/tests/schema/`, with the red observations before the group's last commit
- [x] 3. The key-decision row recording the schema's integrity posture

## Decisions log

- **Step 7**: design-review reached GO on round 3; rounds 1 and 2 returned ITERATE, and the cap of 3 was not raised.
- **Step 8**: Group A routes to `code-writer` (frontmatter-pinned sonnet/medium, no inline override) and Group B to `general-purpose` (inherit), as the design's `## Handoff plan` marks them.
- **Step 8, subtask 1**: wrote `crates/core/migrations/0002_storage_schema.sql` per design § *The shape this design chooses* — all nine tables in dependency order, the four named indexes, no `CHECK`, no `DEFAULT` beyond the `bigserial` sequences, no transaction opt-out, no comment. Added `embedded_set_carries_the_storage_schema_migration` beside the existing `#[cfg(test)]` case in `crates/core/src/lib.rs`, asserting the highest-versioned migration is version 2 described `storage schema` (D11); the existing lowest-version case is untouched. `cargo build --workspace --all-targets`, `cargo test --workspace --lib -p reader-core`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and the whole-workspace `cargo test --workspace` (database-backed, container socket reachable) all ran green before this commit.
- **Step 8, subtask 2**: moved `make_trial` from `crates/core/tests/database.rs` into `crates/core/tests/support/mod.rs` (D9) and had `database.rs`'s `main` use it from there. Added `crates/core/tests/schema/{mod.rs,shape.rs,behaviour.rs}` under exactly those file names, registered from `database.rs`'s `main` via `schema::trials(&harness, &handle)`. `mod.rs` holds the fixtures (`insert_book`, `insert_chapter`, `insert_paragraph`, `insert_book_chapter_paragraph`, `insert_whole_book`, `zero_vector_literal`, `assert_sqlstate`/`sqlstate_of`); `shape.rs` carries the 8 catalogue trials § Test Design names; `behaviour.rs` carries the 10 database-driving trials. All 23 trials (2 lib unit + 8 shape + 10 behaviour + the 5 pre-existing `database.rs` trials → 23 total in the `database` target, 2 in the `reader_core` lib) passed on the whole-workspace `cargo test --workspace` before this commit, alongside `cargo build --workspace --all-targets`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `make file-limits`, `make comment-refs` and `make panic-calls`, all green.

  **Red observations performed before this commit, each mutation reverted and the revert confirmed with `git diff --name-only crates/core/migrations/0002_storage_schema.sql`** (empty output after every revert):
  1. *Golden sees an added column* — added `settings.extra_column text`; `schema_matches_the_recorded_shape` failed: `missing: [], extra: [("settings", "extra_column", "text", false, None)]`. Reverted.
  2. *Golden sees a removed column* — dropped `books.cover_path`; same trial failed: `missing: [("books", "cover_path", "text", false, None)], extra: []`. Reverted.
  3. *Key assertion sees a missing key* — dropped `settings`'s `PRIMARY KEY`; `every_table_carries_the_key_the_corpus_fixes` failed: `missing: [("settings", ["key"])], extra: []` — confirming the dropped key produces no row rather than an empty list. Reverted.
  4. *Index assertion sees a wrong column order* — reversed `paragraphs_chapter_id_idx_uniq` to `(idx, chapter_id)`; `the_named_lookups_have_an_index_of_their_own` failed: `index paragraphs_chapter_id_idx_uniq has key columns ["idx", "chapter_id"], expected ["chapter_id", "idx"]`. Reverted.
  5. *Dimension assertion sees another dimension* — declared `embedding vector(768)`; `the_embedding_column_is_the_declared_dimension` failed (`"vector(768)"` vs expected `"vector(1024)"`) and `a_vector_of_another_dimension_is_refused` failed (`error returned from database: expected 768 dimensions, not 1024`). Reverted.
  6. *Absent-index assertion sees an index* — added `CREATE INDEX … USING hnsw (embedding vector_l2_ops)`; `no_index_over_the_embedding_column_is_hnsw` failed: `missing: [], extra: ["hnsw"]`. Reverted.
  7. *Empty-table assertion sees a row* — inserted a book/chapter/paragraph/embedding chain into the migration; `the_tables_later_tasks_fill_are_created_and_empty` failed: `table paragraph_embeddings holds 1 row(s), expected none`. Reverted.
  8. *Delete-rule assertion sees a cascade* — changed `chapters.book_id`'s reference to `ON DELETE CASCADE`; `every_reference_refuses_a_delete` failed: `[("chapters_book_id_fkey", "c")]`, and `deleting_a_book_is_refused_while_anything_references_it` failed: `chapter as the sole reference: expected SQLSTATE 23001, the statement succeeded`. Reverted.
  9. *Absent-check assertion sees a check constraint* — added `CHECK (text <> '')` to `paragraphs.text`; `the_schema_carries_no_check_constraint` failed: `["paragraphs_text_check"]`. Reverted. (The unmutated schema's green on this same trial was already observed in the whole-suite run preceding these mutations.)
  10. *Required-value assertion sees an optional column* — first tried `translations.cache_key` (stayed green: PostgreSQL implicitly forces `NOT NULL` on every primary-key column, and `cache_key` is part of `translations`'s composite key — a discovery worth recording below), then dropped `NOT NULL` from `paragraphs.text` instead; `a_row_missing_a_required_value_is_refused` failed (`paragraph with no text: expected SQLSTATE 23502, the statement succeeded`) while `the_database_judges_presence_not_content` stayed green — separating the two rules as design requires. Reverted.
  11. *Embedded-set assertion sees a stale build* — `git mv` the migration to `0003_storage_schema.sql`; `embedded_set_carries_the_storage_schema_migration` failed (`left: 3, right: 2`). `git mv` back to `0002_storage_schema.sql`; `git status --short` confirmed no diff against the committed tree.

- **Step 8, subtask 2, pre-commit ratchet block**: `git commit` was BLOCKED by `coverage-ratchet: recorded 90.91%, measured 90.91%`, despite subtask 2 adding no shipped `.rs` line (only `tests/**`). `cargo llvm-cov --workspace --summary-only --json` gave the exact figure `20/22 = 90.9090909090909...%`, identical to what subtask 1's commit measured; the recorded `90.91` is that same figure rounded **up** by the hook's write path, so the raw comparison `90.9090909090909 < 90.91` blocks every future commit touching a gated file even with coverage unchanged. This is the exact self-inconsistency `ai-docs/harness-gaps.md`'s 2026-09-19 entry "the coverage ratchet records a rounded-up value it then cannot satisfy, and locks itself" already diagnoses (open, no `Closed by:`); its proposed remedy is to round down or record full precision. Applied the round-down repair to the recorded value only (`90.91` → `90.90`, i.e. `int(90.9090909090909 * 100) / 100`), staged alongside this commit; `bash .githooks/coverage-ratchet.sh --check` then reports `90.91% >= 90.90% (a rise …)`. No hook script edit made — out of this subtask's scope; the existing harness-gaps entry already carries the diagnosis and remedy.

- **Step 8, subtask 3**: added `KD-21 — The storage schema enforces structure and judges no content.` to `ai-docs/key-decisions.md`, appended after `KD-20` at the end of § *Repository and process* — the page numbers monotonically across its three sections, so "numbered after the last row the page carries" and the page's own ordering agree only at the end. The row keeps the page's shape (bold decision, why-prose, `*Consequence:*`, `*Source:*`) and carries all five posture clauses the design's subtask row names (required values and positional uniqueness, no `CHECK`, no `DEFAULT` beyond the identifier sequences, `ON DELETE RESTRICT` on every reference, categorical columns as free text with the supported set in the code) and all four inherited consequences (the caller writes every timestamp, a book delete is refused until the caller removes its content, the refusal is SQLSTATE `23001` rather than `23503`, the embedding dimension is schema rather than configuration). `*Source:*` is backticked prose naming the post-Step-12 path `ai-docs/plans/done/2026-09-19-storage-schema-second-migration.design.md` § D2–D4 and § D7 — verified against `.claude/skills/task/SKILL.md`, whose Step 12 `git mv`s the spec and design into `ai-docs/plans/done/`, and matching the form `KD-19` and `KD-20` already use. No `INV-` citation, because the page carries none (`grep -c "INV-" ai-docs/key-decisions.md` → `0`).

  **Propagation check (AGENTS.md § *Propagation Rule*, steps 1 and 5):** swept `.claude/ AGENTS.md ai-docs/ README.md docs/` for `structural integrity|integrity posture|структурн` and for `on delete restrict|23001|cascad`, excluding this task's own plan files — both empty, with the instrument confirmed live by a control (`pgvector` over the same corpus → 42 hits) and by two constructed strings it was seen to match. Re-swept after the edit for `on delete restrict|judges no content|free text|check constraint|23001`: the only match carrying this claim is the new row itself. No other live document states the schema's integrity posture, so nothing else needed changing. `AGENTS.md` § *Принятые решения с обоснованием в доках* is a curated subset, not a mirror — `KD-16` through `KD-20` have no bullet there either — so no bullet was added, and the subtask's file list is one file.

  **Gates:** `make verify` green in full (`tmp/gate-subtask3.log`; `test result: ok` on every target, 2 lib unit cases and 23 database trials, no `error`/`warning` line). The CI markdown-link check was run locally over all 110 tracked `*.md` — exit 0; the new row adds no markdown link, and the checker was confirmed to include `ai-docs/key-decisions.md` and to resolve its two existing links. The citation guard (`.claude/skills/ai-audit/scripts/check-citations.sh`) exits 0 with `PASS: every citation resolves for its reader.` The pre-commit ratchet is skipped for this commit by design — no `.rs`, `.sql`, manifest or lockfile is staged.

## GO notes

| # | round | note | kind | route | resolution |
|---|-------|------|------|-------|------------|
| G1 | 3 | The AC1 golden covers (table, column, formatted type, required, default) and **no key** | design-internal | folded | design § Test Design (`every_table_carries_the_key_the_corpus_fixes`) + § Red observations @ 1492d7b |
| G2 | 3 | D9 states "a file inside a subdirectory of `tests/` is no target" | design-internal | folded | design § D9 + § Decomposition subtask 2 @ 1492d7b |
| G3 | 3 | D10: "the two differ for **three** of the types this migration writes" | design-internal | folded | design § D10 @ 1492d7b |
| G4 | 3 | Keep D5's refusal of a plan assertion exactly as written | design-internal | folded | design § D5, left unchanged and verified unchanged @ 1492d7b |
| G5 | 3 | keep the exclusion of `_sqlx_migrations` a single named table in the golden and **not** in the new key assertion's domain | design-internal | folded | design § D10 @ 1492d7b |
| G6 | 3 | Round-trip required: before Step 8, update the design doc to incorporate each note/recommendation above | design-internal | folded | the round-trip commit @ 1492d7b |

## Key discoveries (don't re-investigate)

- A delete refused by `ON DELETE RESTRICT` raises SQLSTATE `23001`, not `23503`: a trial written against the foreign-key code would be green for the wrong schema.
- `information_schema` reports the embedding column as `USER-DEFINED`/`vector` and loses the dimension; `format_type(atttypid, atttypmod)` reports it in full.
- PostgreSQL 18 records `NOT NULL` as `pg_constraint` rows of contype `n`, so a check-constraint query scoped to the wrong kind or the wrong namespace answers clean for every schema.
- Dropping a table's primary key removes that table from the key query's answer rather than leaving an empty list, so the key assertion compares as a set in both directions.
- The reading-order plan carries a `Sort` over a bitmap scan until the table is analysed; the design therefore asserts the catalogue and not the plan.
- PostgreSQL implicitly forces `NOT NULL` on every column that is part of a primary key, regardless of the column's own declaration — declaring `translations.cache_key` without `NOT NULL` left the row still refused on a NULL insert, because it is one of the three primary-key columns. A red observation for "a required-value assertion can see an optional column" needs a column outside the table's key.

## AC Status

| AC | Status |
|----|--------|
| AC1 | PASS (`schema_matches_the_recorded_shape`, `every_table_carries_the_key_the_corpus_fixes`, `embedded_set_carries_the_storage_schema_migration`) |
| AC2 | PASS (`context_versions_of_one_cache_key_coexist`, `a_repeated_translation_key_is_refused`) |
| AC3 | PASS (`a_cache_lookup_naming_no_context_version_finds_the_row`) |
| AC4 | PASS (`the_named_lookups_have_an_index_of_their_own`) |
| AC5 | PASS (`the_embedding_column_is_the_declared_dimension`, `a_vector_of_another_dimension_is_refused`) |
| AC6 | PASS (`no_index_over_the_embedding_column_is_hnsw`) |
| AC7 | PASS (`the_tables_later_tasks_fill_are_created_and_empty`) |
| AC8 | PASS (`a_position_is_taken_once_within_its_parent`) |
| AC9 | PASS (`a_row_missing_a_required_value_is_refused`) |
| AC10 | PASS (`the_database_judges_presence_not_content`, `the_schema_carries_no_check_constraint`) |
| AC11 | PASS (`deleting_a_book_is_refused_while_anything_references_it`, `every_reference_refuses_a_delete`) |
| AC12 | PASS (`a_value_outside_the_supported_set_is_stored_as_given`, `the_schema_carries_no_check_constraint`) |
| AC13 | PASS (`a_work_with_no_divisions_is_stored_as_one_untitled_chapter`) |

## Review register

| id | raised | severity | status | verifying command |
|----|--------|----------|--------|-------------------|

## Files touched

- `crates/core/migrations/0002_storage_schema.sql` (new, subtask 1)
- `crates/core/src/lib.rs` (subtask 1)
- `crates/core/tests/support/mod.rs` (subtask 2 — `make_trial` moved in)
- `crates/core/tests/database.rs` (subtask 2 — uses `support::make_trial`, registers `schema::trials`)
- `crates/core/tests/schema/mod.rs` (new, subtask 2)
- `crates/core/tests/schema/shape.rs` (new, subtask 2)
- `crates/core/tests/schema/behaviour.rs` (new, subtask 2)
- `ai-docs/key-decisions.md` (subtask 3 — `KD-21` appended)

