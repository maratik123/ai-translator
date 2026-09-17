# Context status

One entry per completed task, appended by `/task` Step 9.5 and given its pull-request locator at Step 12. The entry says what the task changed and what a later reader needs to know that the diff does not show; the plan documents themselves are retired to `ai-docs/plans/done/`.

An entry's heading carries the task's name and, until the pull request exists, the placeholder `/task` Step 9.5 writes there. Step 12 replaces it with the real number once `gh pr create` returns, and CI refuses a tree where any placeholder survives — so the shape is not spelled out on this page, which CI reads with a plain grep.

This log starts empty.
