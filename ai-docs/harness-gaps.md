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
