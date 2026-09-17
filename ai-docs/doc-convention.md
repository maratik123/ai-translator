# Documentation conventions — ai-translator

Rules for doc comments, and — from DOC-4 onward — for every comment in a file the
comment-reference gate covers. `make doc-check` enforces the mechanical half of the doc comment —
rustdoc runs with warnings denied, so a broken intra-doc link or a malformed attribute is a failing
gate; `make comment-refs` enforces the mechanical half of the reference ban; the rest is review.

## Scope

DOC-1, DOC-2, DOC-3 and DOC-6 apply to every `.rs` file under a crate's `src/`. Test code
documents only what is non-obvious about the fixture.

DOC-4 and DOC-5 apply to every comment in the **gated set**: `*.rs`, `*.sh`, `*.sql`, `*.yml`,
`*.yaml`, `.gitignore`, `Makefile`, and everything under `.githooks/`. A comment is
a comment wherever its marker sits on the line — a trailing comment is in scope exactly as a
line-leading one is.

Markdown is outside the gated set entirely. Prose in `docs/`, `ai-docs/`, `.claude/` and `AGENTS.md`
obeys § *Durable references* below instead, which is a different rule and must not be conflated
with the ban below: markdown keeps naming paths and sections.

## DOC-1 — Summary sentence

A doc comment opens with one sentence about the item, in the third person:

```rust
/// Writes one paragraph's translation under the cache key derived from its text.
pub async fn store_translation(&self, p: &Paragraph, text: &str) -> Result<(), StoreError>
```

Not `/// This function writes…`, and not a bare imperative. The sentence says what the item does, not
how it does it, and it ends with a full stop.

## DOC-2 — Crate and module comment

Every crate and every module carries exactly one `//!` comment, at the top of its root file. It states
what the module owns and what it deliberately does not:

```rust
//! The translation cache: the key derivation, the read that takes the newest
//! context version, and the write. It owns every read and write of the
//! translations table — no other module touches it.
```

## DOC-3 — Contract sections

Document the contract, not the implementation. Where relevant, in this order: what it does · what it assumes (preconditions) · what it returns · which errors it can return and what they mean · concurrency safety.

- A fallible function carries an `# Errors` section naming each variant it can return and what it means.
- A function that can panic carries a `# Panics` section; an `unsafe fn` carries a `# Safety` section stating what the caller must uphold.
- An `*_unchecked` variant states its precondition and who guarantees it (see [`rust-api-naming.md`](rust-api-naming.md)).
- A type meant to be shared across tasks says what it locks, what is cheap to clone, and what must not be held across an await point; the default assumption is that it is not shareable.

Those sections name symbols on purpose, and DOC-4 keeps them: a symbol that **is** the contract, in
the comment's own crate, is not an outward reference.

## DOC-4 — No outward references

A comment says what the thing is, states its call contract, and carries what the linter requires. It
names nothing outside itself that moves when this tree moves. The reason is rot: the thing pointed at
is edited, and the comment becomes a claim nothing checks. A reference is not relocated to a design
document or to `ai-docs/` — outside a configuration file's own documentation the fact it carried is simply lost.
Where a sentence exists only to carry the reference, the sentence goes with it.

This inverts what this section used to say. Citing a design section was the previous rule, on the
argument that a section number survives what a line number does not; the ban is that argument carried
to its end, since a section is renumbered and a document is renamed.

### Banned classes

| Banned class | What it looks like |
|---|---|
| A reference to a place in a file that carries no revision | a bare `<file>:<line>` locator, with or without its path |
| A path to a markdown file | a token ending in `.md` |
| A spec or design acceptance-criterion id | `AC` followed by a number |
| A key-decision anchor | `KD-` or `D` followed by a number |
| A review-register finding id | `R` or `SR` followed by a round number, a hyphen and a finding number |
| A design-section number written without its file | the `§` sign |
| An issue or pull-request number outside the `TODO(#…)` form | `#` followed by a number |
| A repository path carrying neither a line number nor a revision | a token whose first segment is a top-level directory here, a token ending in a source extension of the gated set, or a repository-root file name |
| A URL | an `http` or `https` scheme marker |
| A crate-qualified symbol of **this** workspace named outside the comment's own crate | `<crate>::<Symbol>`, where `<crate>` names a crate of this workspace |

### Exemptions

- **The comment's own subject.** DOC-1 mandates a summary sentence about the item; a comment naming
  what it documents is not pointing outward.
- **A same-crate symbol that is the contract** (DOC-3) — an error variant a function returns, the guarantor
  of a precondition.
- **A qualified symbol from outside this workspace.** `std::time::Duration`, `Result`, a crate's own
  public type stay as written: the ban's reason is rot, and a name that does not move
  when this tree is edited does not rot.
- **A machine-read directive** in full — an attribute the compiler reads, a lint suppression,
  `shellcheck disable=`, the shebang. The human **reason text** a directive carries obeys the rule
  like any other prose, because it rots like any other prose.
- **The `TODO(#<issue>)` form** (DOC-5), in that form only. A bare issue number elsewhere in the
  same comment is still banned.
- **A reference from one key of a config file to a sibling key of the same file.** That is part of
  the key's contract and is not outward.

### What the gate decides, and what review decides

`make comment-refs` decides the classes above and refuses a commit that stages a violation. Two
halves of the rule are **review-judged against the diff instead**, and neither is a gap to be closed
by a cleverer pattern:

- **Narration.** A comment does not walk through what the code does step by step or how it is
  implemented. The owner ruled out a machine check for this; it is a stated condition over the tree
  that a reviewer judges.
- **A bare unqualified name used as a pointer** — "see such-and-such", or a naked `Type.Field`. No
  lexical rule separates it from ordinary prose, or from the same-crate contract symbol the
  exemption above keeps. It is banned, and it is refused in review rather than by the gate.

### Shell that does not live in a `.sh` file

`run:` blocks in the workflow files and hook bodies in `.claude/settings.json` are shell, and they
obey this rule in full. They are **not** machine-gated: the gate reads YAML-level comments, not the
inside of a block scalar, and it reads no JSON at all. A code writer and a reviewer look for them by
hand.

### Configuration files gain prose rather than lose it

Every key of every comment-bearing file under `config/` and its nested directories carries
self-contained English prose describing what the key controls and what changing it does, at the
completeness a configuration file's documentation reaches per key. Stripping a pointer from a config key means writing
the prose that replaces it, in the same edit. This requirement is checked in review; no gate asserts
it.

### Usage prose belongs behind `--help`

A shell script does not document its own invocation grammar in a comment — a comment naming the
script's own path is a banned reference, and it lies the moment the script is renamed. A script whose
comments carried usage prose answers `-h` / `--help` by printing that grammar and exiting 0, before
it does anything else. A script with no invocation grammar gains no flag.

## DOC-5 — What not to write

- No comment that restates the code (`// increment i`).
- No commented-out code — the history holds it.
- No `TODO` without an issue reference: `// TODO(#<issue>): …`. A `TODO` without an owner is a lie about future work.
- No stale comment: changing behaviour without updating the comment above it is the same defect class as a broken test.
- **No unchecked behavioural claim.** A comment asserting a bound, a cost or complexity, a
  reachability, a "never", a "least", or that some other item uses or shares this one, is a claim
  about the code — and it is checked against the code or its tests **before the commit**, the same as
  a design claim. Nothing else checks it: a false comment compiles, the linter is silent, a stale
  `t.Errorf` string never prints on a green run, and `make comment-refs` decides references, not
  truth. A comment that paraphrases a design's proof in **stronger** words than the proof establishes
  is the likeliest false one. A claim that X does **not** use Y is checked through the whole call
  chain, not from direct call sites.

## DOC-6 — Examples

A doc test — a fenced example inside the item's own doc comment — is the preferred documentation for anything with a non-obvious call sequence: it compiles, `cargo test` runs it, and it cannot rot silently.


## Durable references — a place named to be read later names a SYMBOL

The rule § *Scope* points at, and the whole of it. It governs markdown prose in `docs/`, `ai-docs/`,
`.claude/` and `AGENTS.md` — **not** comments, which DOC-4 governs far more strictly (a comment names
no place at all, symbols included).

**MUST — a reference written into a durable file to be read later names a symbol: module plus
function, a type, or a named document section.** Resolve it before writing it — `ast-index symbol` /
`ast-index outline` for code, the heading text for a document — and write the NAME the tool
confirmed, never the coordinate it printed.

**MUST — a `file:line`, a line range, or a coverage-profile coordinate appears only as a historical
measurement, and only carrying the commit it was taken at.** A bare coordinate is a defect wherever
it sits.

**Why a name and not a coordinate.** A coordinate is invalidated by any edit above it, and these
files are read on a different tree. The observed symptom is not "slightly imprecise" — it is a
**false finding**: two coordinate lists taken at different commits, compared against each other,
produce a discrepancy that does not exist.

**The spec surface is stricter, mechanised, and defined ONCE — elsewhere.** A spec's quoted value
carries a pinned annotation whose exact form, its required commit, and the reviewer's obligation to
raise a violation are stated in `.claude/agents/spec-writer.md` Rule 8. That text is authoritative
for specs and is deliberately **not** restated here: two copies of a format rule drift, and the copy
nobody gates is the one that goes stale. This section is the general case; Rule 8 is its spec-scoped
instance, and where they meet Rule 8 governs.

**Verifying a citation has two halves, and the second is the one that gets skipped:** does the source
say what is claimed, **and** is the reference spelled in the form this project allows. The quote
being right is exactly what stops you looking at the pointer.

**Where two numbering spaces exist** — a project-wide registry and a document's own local table — a
**bare** identifier resolves against the project-wide one. Write the qualified form, and resolve
**every** such reference on a touched line, not only the one you came for.

**Mechanical half, and its exact scope.** `ai-docs/scripts/check-spec-shape.sh` refuses a bare
`path:line`, but only over `*.spec.md`, and only from CI. Every other durable surface is
review-judged.
