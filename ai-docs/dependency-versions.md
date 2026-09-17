# Dependency versions — live-lookup reference

Recipes behind the AXIOM in `AGENTS.md` § *Dependency Versions*. **Every claim about live state is queried, never remembered** — and the command must reach the category you are asking about, or its exit 0 answers a different question than yours.

## Lookup table

| Category | Question | Command |
|---|---|---|
| Crate versions | "What is the current version of X?" | `cargo search <crate> --limit 1`, or `cargo add <crate> --dry-run` to see what would be selected |
| Resolved version | "Which version does this workspace actually build?" | `cargo tree -p <crate>` — the lockfile decides, not the manifest's range |
| Dep-graph membership | "Is X a dependency here?" | `grep -r '<crate>' --include='Cargo.toml' .` for a **declared** dependency **AND** `cargo tree -i <crate>` for transitive reach |
| Why a crate is here | "Who pulls X in?" | `cargo tree -i <crate>` prints the inverted tree — every path that reaches it |
| Dev versus shipped | "Does the binary link X?" | `cargo tree -e no-dev -p <binary>`; `make import-guard` decides the rule table's cases |
| Feature reality | "Is feature F on?" | `cargo tree -f '{p} {f}'` — a feature enabled by another crate is invisible in your own manifest |
| Tool behaviour | "Does `<tool>` support `--flag`?" | `<tool> --help` or run it. **Never** from memory |
| Toolchain | "Which compiler is this?" | `cargo --version`, `rustc --version`; a `rust-toolchain.toml` in the repository overrides whatever is on PATH |
| VCS state | "Is this file tracked / ignored / committed?" | `git ls-files --error-unmatch <path>` (tracked), `git check-ignore -v <path>` (ignored), `git log -1 -- <path>` (committed). `git status` is **blind to ignored files** — empty output is never proof of absence |
| Upstream issue | "Is this bug fixed?" | `gh issue view <N> --repo <owner>/<repo> --json state,comments` — the body is frozen at filing time; the **closing comment** carries the resolution |
| Model server | "Does the server accept this parameter?" | Ask the running server, or read the version's own documentation. A parameter that decides output quality is verified against the server that will serve it, never assumed from another build |

## Changing a dependency

```bash
cargo add <crate>@<version>        # or: cargo add <crate> --features …
cargo build --workspace --all-targets
make lock-check                    # the manifest and the lockfile agree
git diff Cargo.toml Cargo.lock     # confirm the delta is only the intended edges
```

- **Never hand-edit a version in `Cargo.lock`.** The tool keeps it consistent; a hand edit does not.
- `cargo add` and `cargo update` also move transitive lines. Read the diff before staging — an unrelated bump riding along is exactly what the review is for.
- A range in a manifest is not a pin: the lockfile is what pins. A claim about "the version we use" is read from the lockfile or from `cargo tree`.

## Failure mode — a filter that prints a non-answer and exits 0

A `jq` filter over an error body prints `null` and exits **0**. A `grep` over the wrong file exits 1 and *looks* like "not a dependency". Both are fact-shaped non-answers.

> **If a lookup returns nothing, re-run it without the filter and read the raw output before concluding anything.** Absence of evidence from a command that never reached the category is not evidence of absence — the same principle `.claude/rules/ast-index.md` states for code search.

## Standard library first

A new dependency needs a stated reason in the design document — **and so does hand-rolling in place of one.** `AGENTS.md` § Dependency Versions makes established crates and the standard library the default, refuses *"it's only 10–20 lines"* and *"better than pulling in an established dependency"* as arguments for writing your own, and sets the bar an argued wheel meets. The load-bearing choices are fixed in [`key-decisions.md`](key-decisions.md) and in the corpus under `docs/`: the model is reached over its OpenAI-compatible HTTP API, storage is PostgreSQL with pgvector, the test database arrives through testcontainers, and the protocol types are generated from one crate rather than written twice.
