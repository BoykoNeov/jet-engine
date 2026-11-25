---
name: cutaway-binding-priority
description: "User 2026-11-20 priority: bind the cutaway to the model permanently (roadmap A9) — DONE 2026-11-25 as the sandbox's Engine view"
metadata:
  node_type: memory
  type: project
  originSessionId: 19af6582-8bb4-4a85-824c-d17edc6db008
  modified: 2026-11-25T09:11:05.254Z
---

On 2026-11-20 the user asked, "with priority", to "make stronger connections between the computation and what the
cutaway shows and make that connection permanent" (roadmap A9).

**DONE 2026-11-25** — see [[a9-engine-view]]: the user chose the live sandbox view; it shipped as the Engine view,
gated so a drawn constant cannot creep back. The old page is retired ([[cutaway-artifact]]).

**Why it mattered:** the old cutaway drew one baked design with a hand-typed flame temperature, stage count, shaft
speed and parcel speeds, and did not follow the sandbox knobs. "Permanent" = a gate that fails when a drawn constant
creeps back or a computed number stops matching the model.
