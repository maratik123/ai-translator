# Learning log

Corrections and validations, append-only, newest last. `AGENTS.md` § *Learning Log* carries the rules: what an entry is for, when one is written, and the two boundary rules. The canonical skeleton is [`templates/learnings-entry.md`](templates/learnings-entry.md) — consult it rather than copying the shape off a neighbouring entry. Carve-outs and the field glossary: [`corrections-log.md`](corrections-log.md).

An entry is a conduct correction or a validation of this project's own runs; a diagnosis about the harness belongs in [`harness-gaps.md`](harness-gaps.md) instead.

### 2026-09-18 — search — set-difference verdict read before its inputs' cardinality and collation
**What happened:** Verifying that every newly created gh issue had a parent, I ran `comm -23` over two lists ordered with `sort -n`. `comm` requires lexicographic collation: it printed `данные файла 1 не отсортированы` on stderr and its stdout named all 56 issues as parentless. The verdict was false, and it would have been reported as fact had that stderr line not landed in the same output block I was reading.
**Rule:** For any check shaped as intersect-two-sets / diff-against-a-baseline, read both inputs' cardinality AND confirm the collation the tool requires BEFORE reading the verdict; pair the check with a control whose expected result is non-empty, so a clean answer is distinguishable from an instrument that cannot produce a dirty one. A degradation notice on stderr voids a probe rather than decorating it.
**at:** `master` @ e8be924
**Kind:** correction
**Escalated?** AGENTS.md, rules:ast-index

### 2026-09-18 — testing — control probe read before confirming the probe's own process had started
**What happened:** Checking whether `llama-server` really needs an explicit chat template, I stopped the first server by the PID the shell reported for the `nohup` job, started a control instance without the template on the same port, and read its answer as the control result. The kill had missed the real process: the control server died with `couldn't bind HTTP server socket`, and the reply came from the still-running templated server. I had already begun treating "the control passed" as a fact about llama.cpp before checking `pgrep` and the control's own log.
**Rule:** Before reading any probe's result, assert that the probe LANDED — that the process under test is the one answering (`pgrep`/`ss` for the listener, the probe's own log for a startup error), and prefer a fresh port or an explicit readiness check over reusing the one the previous run held. A server that failed to start and a server that disagrees with the hypothesis return indistinguishable evidence to the caller.
**at:** `master` @ 811ccef
**Kind:** correction
**Escalated?** AGENTS.md, rules:ast-index

### 2026-09-18 — testing — a token count documented as a request's cost, measured with a call that omits BOS
**What happened:** Writing the provenance README for the miniature models, I documented the prompt budget of a chat request from `/tokenize` output. That endpoint defaults to `add_special=false`, so the figures dropped the BOS token every served request carries: the table read 16 and 56 where `/v1/chat/completions` reports 17 and 57 in `usage.prompt_tokens`. Self-review caught it; the argument the numbers supported (the 40-token gap) was unaffected, the absolute budget against a 128-token slot was not.
**Rule:** When a number is documented as the cost of an operation, measure it on the operation itself — the served endpoint's own accounting — not on a helper endpoint that answers an adjacent question with different defaults. A proxy instrument needs its defaults read before its output is quoted as the subject's figure.
**at:** `chore/testdata-models` @ b65356c
**Kind:** correction
**Escalated?** AGENTS.md

### 2026-09-18 — search — a line number attributed by position in a probe's output instead of read from it
**What happened:** Verifying a design-review finding that a `[measured …:22-24]` tag was off by one, I ran `sed -n '21,24p' .githooks/coverage-ratchet.sh`, saw the quoted clause among the four printed lines, and told the owner the finding held and the clause begins at line 21. It begins at 22; line 21 is a bare `#`. `sed -n` prints no line numbers, so the attribution was inferred from the position of the line in the output rather than read from the instrument — and the range I was handed by the finding was exactly the one that makes the inference come out wrong. The `design-writer` delegate refused the fold-in, re-resolved with `grep -n 'THE TOLERANCE IS ZERO'` → `22:`, and was right; I had reported the reviewer's error onward as independently confirmed.
**Rule:** When the claim under test **is** a coordinate, the probe must print the coordinate — `grep -n` for the clause's own words, or `awk 'NR>=a && NR<=b {printf "%d|%s\n", NR, $0}'` — never a range printer whose output the reader numbers by counting. Reaching for the range from the finding also anchors the probe to the claim it is supposed to test independently; derive the range from a search for the content instead. This is the ast-index rule *assert that the probe LANDED where the instrument looks, and report where that is*, in the case where where-it-landed is the whole question.
**at:** e1f8a4d
**Kind:** correction
**Escalated?** AGENTS.md, rules:ast-index

### 2026-09-18 — process — an instruction file's size measured while deciding how to read it
**What happened:** About to read the handoff protocol before spawning the first Step-8 group, I ran `wc -l` on a skill file together with `cat`, to size the read before making it. A `PreToolUse` hook refused the command: the size of `AGENTS.md`, `CLAUDE.md` and `.claude/{skills,agents,rules}/**.md` belongs to `/ai-audit` alone, and every other flow is forbidden to measure it, report it or plan around it — **including for the purpose of deciding how to read a file**, which is exactly the purpose I had. The plain `cat` that followed was always the right call and cost nothing extra.
**Rule:** Never size an instruction file. Read it with `cat`, or navigate it with `sed -n` ranges and `grep -n` for structure. "I only wanted to know how much to read" is the motive the rule names and refuses, not an exemption from it — a habit of sizing a file before opening it is correct for a data file and prohibited for this class.
**at:** 2ddf9de
**Kind:** correction
**Escalated?** no

### 2026-09-18 — tooling — `pkill -f` matched the calling shell's own command line
**What happened:** To stop a slow background sweep I ran `pkill -f` with a pattern naming the script. The pattern matched the very Bash invocation that carried it, so the command killed its own process tree and returned exit 144; the background task was reported as failed rather than stopped.
**Rule:** `pkill -f <pattern>` matches every process whose full command line contains the pattern — including the shell running the `pkill`. The project already records this failure mode for one specific target, but it is a property of `-f`, not of that target: kill by exact process name (`pkill -x <name>`), by recorded PID, or with the flow's own stop control, and never with a pattern that the issuing command line itself contains.
**Kind:** correction
**Escalated?** AGENTS.md

### 2026-09-18 — tooling — read a gate's result through a truncating filter instead of a captured log
**What happened:** Reading the coverage ratchet's result at the end of a subtask, I ran the gate target and filtered its output through a line-truncating pager in the same pipeline. A `PreToolUse` hook refused the command. Had it run, the recorded exit status would have been the filter's — always zero — so a red ratchet would have been recorded as green.
**Rule:** A gate whose exit status is load-bearing is never piped. Redirect it to a file under the scratch directory, branch on the gate's own status, and read the saved log afterwards. This binds a one-line convenience read at the end of a turn exactly as it binds the deliberate gate run at the start of one — the shape is the hazard, not the intent. A second consequence learned in the same turn: the hook matches command TEXT, so a later command that merely quotes such a pipeline is refused too; describe the shape in prose rather than reproducing it.
**Kind:** correction
**Escalated?** AGENTS.md

### 2026-09-19 — process — a sub-floor review item fixed behind its own review round, three times, while the pull request merged without it
**What happened:** `self-review` returned APPROVE on round 1 of a harness branch and recorded two items below the severity floor. Instead of pushing on that APPROVE and folding the two fixes into the same push, I fixed one item, re-ran the reviewer, fixed the next, re-ran it again — three APPROVEs over progressively smaller prose, no finding after round 1. The reviewer diagnosed it before I did: its contract says a sub-floor item "must not be the difference between verdicts", and with no `Progress:` path there was no register, so each cold round re-derived the whole review and surfaced the next-smallest item because nothing recorded that the previous one had been weighed. While those rounds ran, the owner merged the pull request — it carried one commit, and three reviewed commits were left outside it, needing a second pull request to reach `master`.
**Rule:** APPROVE means push. A sub-floor item worth fixing is fixed **before** that push, in the same batch, and the round that produced the APPROVE is the round that covered it — a fix the reviewer itself proposed does not buy a new round, and re-verifying it is not what the round cap is for. When a review runs outside a flow that owns a progress file, pass one anyway or accept that every round starts cold: without the register the loop has no memory, and a loop with no memory converges on churn rather than on a verdict. A branch left unpushed through extra rounds is also a branch the owner can merge underneath, which turns one pull request into two.
**at:** a46eea6
**Kind:** correction
**Escalated?** no
