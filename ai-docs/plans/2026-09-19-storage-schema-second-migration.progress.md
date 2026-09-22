# Progress: Storage schema — the second migration — ACTIVE
_Updated: 2026-09-22 08:41_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-09-19-storage-schema-second-migration
**base_commit:** 1492d7b6e3c62cc631ca1efa7bedcaf211b873fc
**Last build:** PASS
**Issue:** #11
**Spec:** ai-docs/plans/2026-09-19-storage-schema-second-migration.spec.md
**current_step:** Step 8 — Implementation start
**last_passed_gate:** cargo build --workspace --all-targets | 2026-09-22T08:41:38Z | 1492d7b
**entry_args:** 11

## Next action

**Do this immediately:** hand off Group A (subtasks 1 and 2) to `code-writer` through `/context-reset`, per the design's `## Handoff plan`.

## Subtasks

- [ ] 1. The migration `crates/core/migrations/0002_storage_schema.sql`, and the embedded-set assertion that proves the build saw it  ← CURRENT
- [ ] 2. The schema trials under `crates/core/tests/schema/`, with the red observations before the group's last commit
- [ ] 3. The key-decision row recording the schema's integrity posture

## Decisions log

- **Step 7**: design-review reached GO on round 3; rounds 1 and 2 returned ITERATE, and the cap of 3 was not raised.
- **Step 8**: Group A routes to `code-writer` (frontmatter-pinned sonnet/medium, no inline override) and Group B to `general-purpose` (inherit), as the design's `## Handoff plan` marks them.

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

## AC Status

| AC | Status |
|----|--------|
| AC1 | NOT_TESTED |
| AC2 | NOT_TESTED |
| AC3 | NOT_TESTED |
| AC4 | NOT_TESTED |
| AC5 | NOT_TESTED |
| AC6 | NOT_TESTED |
| AC7 | NOT_TESTED |
| AC8 | NOT_TESTED |
| AC9 | NOT_TESTED |
| AC10 | NOT_TESTED |
| AC11 | NOT_TESTED |
| AC12 | NOT_TESTED |
| AC13 | NOT_TESTED |

## Review register

| id | raised | severity | status | verifying command |
|----|--------|----------|--------|-------------------|

## Files touched

