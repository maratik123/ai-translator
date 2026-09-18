# Learning log

Corrections and validations, append-only, newest last. `AGENTS.md` § *Learning Log* carries the rules: what an entry is for, when one is written, and the two boundary rules. The canonical skeleton is [`templates/learnings-entry.md`](templates/learnings-entry.md) — consult it rather than copying the shape off a neighbouring entry. Carve-outs and the field glossary: [`corrections-log.md`](corrections-log.md).

An entry is a conduct correction or a validation of this project's own runs; a diagnosis about the harness belongs in [`harness-gaps.md`](harness-gaps.md) instead.

### 2026-09-18 — search — set-difference verdict read before its inputs' cardinality and collation
**What happened:** Verifying that every newly created gh issue had a parent, I ran `comm -23` over two lists ordered with `sort -n`. `comm` requires lexicographic collation: it printed `данные файла 1 не отсортированы` on stderr and its stdout named all 56 issues as parentless. The verdict was false, and it would have been reported as fact had that stderr line not landed in the same output block I was reading.
**Rule:** For any check shaped as intersect-two-sets / diff-against-a-baseline, read both inputs' cardinality AND confirm the collation the tool requires BEFORE reading the verdict; pair the check with a control whose expected result is non-empty, so a clean answer is distinguishable from an instrument that cannot produce a dirty one. A degradation notice on stderr voids a probe rather than decorating it.
**at:** `master` @ e8be924
**Kind:** correction
**Escalated?** no

### 2026-09-18 — testing — control probe read before confirming the probe's own process had started
**What happened:** Checking whether `llama-server` really needs an explicit chat template, I stopped the first server by the PID the shell reported for the `nohup` job, started a control instance without the template on the same port, and read its answer as the control result. The kill had missed the real process: the control server died with `couldn't bind HTTP server socket`, and the reply came from the still-running templated server. I had already begun treating "the control passed" as a fact about llama.cpp before checking `pgrep` and the control's own log.
**Rule:** Before reading any probe's result, assert that the probe LANDED — that the process under test is the one answering (`pgrep`/`ss` for the listener, the probe's own log for a startup error), and prefer a fresh port or an explicit readiness check over reusing the one the previous run held. A server that failed to start and a server that disagrees with the hypothesis return indistinguishable evidence to the caller.
**at:** `master` @ 811ccef
**Kind:** correction
**Escalated?** no

### 2026-09-18 — testing — a token count documented as a request's cost, measured with a call that omits BOS
**What happened:** Writing the provenance README for the miniature models, I documented the prompt budget of a chat request from `/tokenize` output. That endpoint defaults to `add_special=false`, so the figures dropped the BOS token every served request carries: the table read 16 and 56 where `/v1/chat/completions` reports 17 and 57 in `usage.prompt_tokens`. Self-review caught it; the argument the numbers supported (the 40-token gap) was unaffected, the absolute budget against a 128-token slot was not.
**Rule:** When a number is documented as the cost of an operation, measure it on the operation itself — the served endpoint's own accounting — not on a helper endpoint that answers an adjacent question with different defaults. A proxy instrument needs its defaults read before its output is quoted as the subject's figure.
**at:** `chore/testdata-models` @ b65356c
**Kind:** correction
**Escalated?** no
