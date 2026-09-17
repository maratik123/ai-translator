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
