---
name: cutaway-binding-priority
description: "User 2026-11-20 — PRIORITY for a future session: bind the cutaway to the model permanently (roadmap A9); ask before/after rung 110"
metadata:
  node_type: memory
  type: project
  originSessionId: 765d0165-d7cd-430a-aa2f-7daab84dc6af
  modified: 2026-11-20T09:59:56.906Z
---

On 2026-11-20 the user asked, "with priority" for a future session, to "make stronger connections between the
computation and what the cutaway shows and make that connection permanent". Written up as **roadmap A9** in
`docs/plans/roadmap.md` (what the cutaway computes vs draws, the stale Python names in its footer, the open
choice: separate page with computed constants vs a live sandbox view).

**Why:** the cutaway draws one baked design point; much of what moves on it (flame temperature 2150 K, parcel
speeds, stage counts, shaft speed) is hand-drawn, and it does not follow the sandbox knobs.

**How to apply:** at the next session start, raise A9 as the priority, but ASK whether it goes before or after
finishing the paused rung 110 ([[rung110-wip-handoff]]). Ask the user's choice between the two designs before
building. Cutaway link: [[cutaway-artifact]]. "Permanent" = a gate that fails when a drawn constant creeps back
or a computed number stops matching the model.
