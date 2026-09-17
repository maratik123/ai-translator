# Panic index

Every intentional panicking call — a panicking macro, `unwrap`, `expect` — in **shipped** code, each with a one-line justification that it is genuinely unreachable or unrecoverable. Kept in sync by review and by the `panic-gate` hook in `.claude/settings.json`; `make panic-calls` decides the marker half.

**The project targets zero unmarked panicking calls in shipped code and currently holds it** — the table below is empty. A binary's `main` may exit non-zero on a start-up failure; that is not a panic and needs no row here. Treat any new panicking call on a translation request, a pipeline stage, a storage write, or anything the server reaches as a red flag: those paths must degrade into an error the caller can render or retry, because a panic there loses a chapter's work in flight and takes the process with it.

A surviving call carries **both** halves: the `PANIC:` marker with its reason on its own line or the line above — the justification in prose, never a pointer at this index, which would be the outward reference the comment ban forbids — and a row here, added in the same commit.

Test code is out of scope by position: a `#[cfg(test)]` module, and everything under `tests/`, `benches/` and `examples/`.

| File:line | Call | Why it cannot fire (or is unrecoverable) |
|---|---|---|
| — | — | — |
