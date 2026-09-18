# Rust test conventions — detail

The detail behind `AGENTS.md` § *Test Conventions*. The reviewer charters cite this page; a rule here is a rule a review applies.

## Where a test lives

- **Unit tests** live in the file they test, inside its `#[cfg(test)] mod tests`. They may reach private items, which is the point.
- **Integration tests** live in `crates/<crate>/tests/<name>.rs` and see only the crate's public surface. Use one when the behaviour is only observable across the crate boundary, or when the test needs a real database.
- A shared fixture two integration tests need goes in a module under `crates/<crate>/tests/support/` rather than being copied; three copies of a fixture is the duplication rule's trigger like any other code.

The file-size ladder counts a file whole, unit tests included ([`code-style.md`](code-style.md) § File size), so a `#[cfg(test)]` module that outgrows its file is the signal to move those cases into an integration test, not to raise the limit.

## Case tables are the default

More than two cases of the same shape go in a table, not in copy-pasted functions:

```rust
#[test]
fn rejects_the_named_failure_classes() {
    struct Case {
        name: &'static str,
        response: &'static str,
        want: Verdict,
    }

    for case in [
        Case { name: "empty", response: "", want: Verdict::Rejected },
        Case { name: "echo of the source", response: SOURCE, want: Verdict::Rejected },
        Case { name: "preamble", response: "Here is the translation: …", want: Verdict::Rejected },
    ] {
        assert_eq!(validate(case.response), case.want, "case: {}", case.name);
    }
}
```

- Every case carries a **name**, and the name describes the behaviour: `rejects_empty_translation`, never `test2`.
- The failure message names the case. A table whose failure says only `assertion failed` costs a debugging session per failure.
- A case that needs a different setup is a different test, not a flag inside the table.

## Assert exactly what the function promises

- A pure transformation — segmentation, alignment, prompt rendering, cache-key derivation — is asserted against its **exact** expected output. A range, "not empty", or "no error" silently forfeits the property the pure function was written for ([`domain-invariants.md`](domain-invariants.md) INV-16).
- An assertion that would pass for every plausible output is cosmetic. The mental test: comment out the production change — does the test fail? If not, it tests nothing.
- A guard-clause test — "reports nothing", "no false positive" — is verified by mutation: delete the clause from production and confirm the test fails. A fixture that never reaches the clause makes a green assertion meaningless.

## The model is never the oracle

A test's verdict never depends on what the model returns ([`domain-invariants.md`](domain-invariants.md) INV-12). A unit test stands on a **recorded fixture**: a captured response, stored beside the test, with the model and the parameters that produced it named in the fixture itself.

Anything whose verdict rests on the **quality** of what the model returned is an **eval**, not a test: it is marked as one, it runs under the eval conditions (temperature zero, draft head off, no expert offload), and it reports a measurement rather than passing or failing a build.

The conformance test is not an exception to that rule but an illustration of it. It does call a model, and it still asserts nothing about what came back — only the shape, the status code and the schema. What decides the category is what the verdict rests on, never whether a model was involved.

The suite reaches the CPU-only miniature models under `testdata/models/` and nothing else. It never talks to a server that holds the GPU: the production model occupies nearly all of the VRAM, and a test contending for it makes its own result and the server's throughput both mean less than they claim. A check the miniature models cannot support is **raised with the owner**, never redirected at a real model.

## Postgres is tested against Postgres

A test that asserts a database-enforced invariant — a `CHECK`, a unique index, the vector index's behaviour, ordering under concurrency — runs against a real server. The suite provisions a `pgvector` container through testcontainers over the Podman socket and drops it on cleanup; the application's own connection string is never the suite's, or the test runs against the developer's data.

- A mock proves the mock. Use one for the model client, never for the schema.
- Each test gets its own schema or its own database, so tests do not see each other's rows.
- A machine with no container runtime is told so loudly by the failing test, never passed silently.

## Concurrency is exercised, not assumed

Rust's type system rules out a data race; it rules out neither a lost cancellation nor a task nobody awaits, and those are the ones that surface as a stalled translation. A diff that spawns a task, adds a channel or shares state behind a lock carries a test that drives the concurrent path **and** its cancellation: the work stops, the handle is joined, and the error the caller sees names what happened.

Timing is asserted through the runtime's own control — a paused clock, a deterministic scheduler — never through a sleep long enough to "probably" be enough. A test that sleeps to synchronise is a flake with a timer.

## Panicking calls in tests

A test may `unwrap` freely: a panic there is a failing test, which is the intended outcome. The panic gate reads only shipped code — a `#[cfg(test)]` module and everything under `tests/`, `benches/` and `examples/` is out of its scope by position ([`panic-index.md`](panic-index.md)).

## What not to do

- **No test that only exercises the mock.** If replacing the production function with `todo!()` leaves the test green, delete the test.
- **No assertion on a formatted error string** where a typed variant exists.
- **No sleep-based synchronisation.**
- **No test that depends on another test's rows, files or order.**
- **No fixture pulled from a book text or a production model weight** — neither is tracked ([`domain-invariants.md`](domain-invariants.md) INV-18); a fixture ships a short excerpt it owns. The miniature models under `testdata/models/` are the exception the same invariant carves out: they are tracked precisely so the conformance suite can run against them offline.
