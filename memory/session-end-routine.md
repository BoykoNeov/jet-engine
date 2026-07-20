---
name: session-end-routine
description: "Wrap-up routine to run at end of a work batch, end of planning, or when the user says \"session end\""
metadata: 
  node_type: memory
  type: feedback
  originSessionId: d6ed48ca-d67d-48f6-b698-b30b3ba4c4b9
---

**Standing authorization (2026-06-29):** the user said "always commit and push —
work batches, plans, or memory updates." So commit AND push proactively at every
such boundary without asking each time — this routine is the blanket
authorization, overriding the repo's "commit only when asked" default for these
boundaries.

At the **end of a work batch**, at the **end of planning**, after any **memory
update**, or whenever the user says **"session end"**, always do all four, in order:
1. **Update memory** — write/refresh any relevant memories in this store.
2. **Update docs** — refresh the living docs (`docs/plans/*`, checklists,
   `NOTES.md`) so they reflect what just landed.
3. **Commit** — stage and commit with a descriptive message in the repo's style
   (`type: summary`, e.g. `feat: implement station N (...)`), including the doc
   updates.
4. **Push to main** — push the `main` branch to `origin`.

**Why:** the user wants a consistent wrap-up so memory, docs, and git stay in
sync and nothing is lost between sessions. The action needs judgment (what to
record, what message to write), so it lives as this preference, not a
settings.json hook (a hook can only fire a canned command on every Stop, which
can't tell when a batch actually ended).

**How to apply:** detect the trigger (a coherent chunk of work finished, a plan
finalized, or the literal phrase "session end"), then run steps 1–4. Git target
is settled in [[git-remote-setup]] — branch `main`, remote `origin` on GitHub.
Commit/push only as part of this routine or when the user explicitly asks
(repo's standing "commit only when asked" rule); this routine *is* that standing
authorization for batch boundaries.
