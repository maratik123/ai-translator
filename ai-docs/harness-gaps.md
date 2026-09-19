# Harness gaps

Diagnoses about the harness itself: what a flow failed to make happen, and where the fix belongs. The second learning log — conduct corrections go to [`learnings.md`](learnings.md), and the boundary between the two is stated in `AGENTS.md` § *Learning Log*.

An entry names the file its fix must land in, so a later reader can route it without re-deriving the diagnosis. The closing fields are written by a forge branch only, and `ai-docs/scripts/check-harness-gaps-forge.sh` enforces that.

The skeleton, fenced so the guards read it as a shape rather than as an entry:

```
### YYYY-MM-DD — one-line title
**target:** `path/to/file.md` § Section
**Observed:** what happened, with the coordinate it was observed at.
**Gap:** what the harness did not make happen.
**Proposed edit:** the change that would have made it happen.
**at:** <commit>
**Forge:** forge-N            (written by a forge branch only)
**Closed by:** #N             (written by a forge branch only)
**Superseded by:** YYYY-MM-DD (when a later entry replaces this one)
```

This log starts empty.

### 2026-09-18 — the interview-window hook decides on an instruction file's name in the COMMAND TEXT, not on the write's target
**target:** `.claude/settings.json` § the `PreToolUse` `Bash` hook that enforces Learning Log Boundary rule 2
**Observed:** during `/task` on issue #10, two Bash commands were refused with `BLOCKED: write into an instruction file while an interview is live`. Neither wrote to an instruction file. The first wrote owner answers into `ai-docs/plans/2026-09-18-workspace-crate-skeletons.spec.md.state.md`; the second, run by the `spec-writer` delegate, replaced the AC4 row inside the spec. The hook's guard is a conjunction of two greps over the command STRING — a path pattern (`\.claude/|AGENTS\.md|CLAUDE\.md|ai-docs/(code-style|doc-convention)\.md`) and a write pattern that includes `open\([^)]*["'][wa]["']`. Both commands satisfied the path half only because the TEXT BEING WRITTEN quoted an instruction file by name: an owner's question in the first case, the AC4 clause being struck in the second. Both satisfied the write half with a Python heredoc. Two independent agents hit it on the same shape within one run.
**Gap:** the hook judges the command's text rather than the file the command writes, so a legitimate write is refused whenever its CONTENT names an instruction file. That content is not rare in this flow — it is the flow's normal subject matter: a `SPEC-REMIT` finding quotes the standing rule it is about, and the answer, the spec row and the design row that resolve it all carry the quotation. The block message lists three legal routes and none of them is the one both agents actually took, so each had to infer that a dedicated edit tool is untouched by a `Bash` matcher.
**Proposed edit:** evaluate the path pattern against the write TARGET rather than the whole command — the redirect operand, the `sed -i` / `tee` operand, the first argument of `open()` — so that quoting a rule is never mistaken for editing it. If that is judged too fragile to parse, add the dedicated-edit-tool route to the block message's list, since the `Edit`/`Write` half of the same rule already keys on `file_path` and refuses the real case correctly.
**at:** 0b2bc15
**Forge:** forge-1

### 2026-09-18 — `/task` asserts a `.gitignore` state this repository does not have, and three steps are written against it
**target:** `.claude/skills/task/SKILL.md` § Step 8 and § Step 12 sub-step 9a (and `ai-docs/templates/progress-format.md` § Lifecycle by field, which repeats the claim)
**Observed:** Step 8 says the in-flight marker `ai-docs/plans/.task-inflight` is "(gitignored; Stop-hook contract)" and that the progress file needs `git add -f` "because the path matches a `.gitignore` glob; once tracked the glob no longer applies". Neither holds here. `.gitignore` has ten lines and no `ai-docs/` entry at all: `git check-ignore` reports `ai-docs/plans/.task-inflight`, `ai-docs/plans/<name>.progress.md` and `ai-docs/plans/ignored/<f>` all **not ignored**, while `tmp/<f>` is ignored. So the marker sits in `git status --porcelain` for the whole run, a plain `git add` tracks the progress file, and `ai-docs/plans/ignored/` becomes an untracked directory the moment Step 12 sub-step 9a moves the state files into it.
**Gap:** three instructions are written against the absent entries. Step 8's per-group cleanliness check offers "an empty `git status --porcelain`" as an equivalent of `git diff --quiet && git diff --cached --quiet`, and the two are not equivalent here — the marker the same step creates makes the first form non-empty for every handoff. Step 12 sub-step 9a then requires `git status --porcelain` empty **after** the move, which the untracked `ignored/` directory and the still-present marker both defeat. Neither failure is a real defect in the work, so the flow would be teaching its operator to read past its own verification.
**Proposed edit:** either add the entries the instructions assume (`ai-docs/plans/.task-inflight`, `ai-docs/plans/*.progress.md`, `ai-docs/plans/ignored/`) and keep the text, or drop the ignore claims and state the checks in the form that is actually sound here — `git diff --quiet && git diff --cached --quiet` for a handoff, and a status check scoped with `--untracked-files=no` or to the paths the step owns. The `git add -f` at Step 8 is harmless either way and can stay.
**at:** 5fd1a04
**Closed by:** #71

### 2026-09-19 — `/improve`'s clean-context eval cannot reach the probe-discipline or command-text classes
**target:** `ai-docs/improve-eval-contract.md` § The RED baseline
**Observed:** an `/improve` pass over eight correction entries escalated six of them as two proposals, assembled four reproducers and dispatched eight clean-context agents. All eight passed — four pre-change baselines, two post-change verifications and two load-bearing variants — so neither proposal ever obtained the FAIL-before the gate requires. Meanwhile the property under test recurred three times inside the same run: both agents dispatched on the command-text reproducer reported their naive first spelling matching their own shell and reporting a running process against an empty machine, and the orchestrating thread reproduced it when a control string it wrote put the target literal back into the command line a bracket class was protecting.
**Gap:** the instrument cannot separate *the rule is unnecessary* from *the rule's failure mode does not occur while the rule is the whole question*. Both classes are caught reliably by an idle agent asked about them directly and missed while doing something else — including under the load-bearing variant, whose primary task evidently did not demand enough attention to displace the clause.
**Proposed edit:** record these two classes as known-unreachable on the contract page, and state what a load-bearing variant must cost the dispatched agent before its PASS counts as evidence of reach rather than evidence of an undemanding primary task.
**at:** 136b290
**Forge:** forge-1

### 2026-09-19 — the documented `comment-refs` gated set is narrower than the coded one, and its `.githooks/**` claim is not implemented
**target:** `AGENTS.md` § Code Style and `ai-docs/doc-convention.md` § DOC-4 — a propagation pair, both carrying the list
**Observed:** both name the gated set as `*.rs`, `*.sh`, `*.sql`, `*.yml`, `*.yaml`, `.gitignore`, `Makefile` and everything under `.githooks/`. Selection is `git ls-files` filtered by `language_of`, which also gates `.bash` and `*.mk`, and which selects under `.githooks/` by extension alone — so the extensionless, symlinked `.githooks/pre-commit` is never scanned. The `.githooks` string near the top of the gate's module belongs to the repository-top-directory set the *token* classifier consumes, not to file selection, which is what a plain grep for it suggests.
**Gap:** latent today, since no tracked `.bash` or `.mk` file exists — but an extensionless hook added under `.githooks/`, which is git's ordinary spelling, would be silently ungated while both documents state it is covered.
**Proposed edit:** bring the two prose lists into agreement with the scanner's own language map, and replace the `.githooks/**` directory claim with the extension rule the code implements.
**at:** 136b290
**Forge:** forge-1

### 2026-09-19 — `/improve` Step 5 prescribes an English commit subject, which the language split forbids
**target:** `.claude/agents/self-improve.md` § Step 5 item 2b
**Observed:** the Commit B template is given literally as `chore(learnings): backfill Escalated? / Superseded by: for entries <date1>, <date2>, ...`. A run that followed it produced an English commit subject whose own body was Russian; self-review raised it as a major against `AGENTS.md` CRITICALLY 1 — *commit messages and pull-request bodies* sit on the Russian surface — and the subject was amended before the push.
**Gap:** the contract instructs the violation, so a run following Step 5 faithfully reproduces it. Nothing catches it except a review standing between the commit and the push, and after a push the only repair is a force-push, which § *Permissions* denies.
**Proposed edit:** give the Commit B template in Russian, or drop the literal subject and name the fields the message must carry plus a pointer to the language split.
**at:** 53adf8c
**Forge:** forge-1
