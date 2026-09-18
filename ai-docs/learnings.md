# Learning log

Corrections and validations, append-only, newest last. `AGENTS.md` § *Learning Log* carries the rules: what an entry is for, when one is written, and the two boundary rules. The canonical skeleton is [`templates/learnings-entry.md`](templates/learnings-entry.md) — consult it rather than copying the shape off a neighbouring entry. Carve-outs and the field glossary: [`corrections-log.md`](corrections-log.md).

This log starts empty. An entry is a conduct correction or a validation of this project's own runs; a diagnosis about the harness belongs in [`harness-gaps.md`](harness-gaps.md) instead.

### 2026-09-18 — search — set-difference verdict read before its inputs' cardinality and collation
**What happened:** Verifying that every newly created gh issue had a parent, I ran `comm -23` over two lists ordered with `sort -n`. `comm` requires lexicographic collation: it printed `данные файла 1 не отсортированы` on stderr and its stdout named all 56 issues as parentless. The verdict was false, and it would have been reported as fact had that stderr line not landed in the same output block I was reading.
**Rule:** For any check shaped as intersect-two-sets / diff-against-a-baseline, read both inputs' cardinality AND confirm the collation the tool requires BEFORE reading the verdict; pair the check with a control whose expected result is non-empty, so a clean answer is distinguishable from an instrument that cannot produce a dirty one. A degradation notice on stderr voids a probe rather than decorating it.
**at:** `master` @ e8be924
**Kind:** correction
**Escalated?** no
