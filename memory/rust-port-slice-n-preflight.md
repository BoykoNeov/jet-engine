---
name: rust-port-slice-n-preflight
description: "Slice N's probes refuted the PREVIOUS slice's pre-registered assumption — reading a method body cannot see what its state's CARRIER costs"
metadata: 
  node_type: memory
  type: project
  originSessionId: e1601c0d-fd3e-4bbc-a442-265d8f4fcc2c
  modified: 2026-08-17T08:16:05.848Z
---

Slice N (rungs 55/56) was pre-registered on 2026-08-17 in `docs/plans/todo-rust-port.md` § 5.10,
after three probes. The probes' most valuable output was not about slice N — it was that
**slice M's own § 5.9 (c) was wrong**, in the half nobody checks.

**Why:** (c) decided ahead of time that slices N and O would "add a VARIANT and a TABLE ENTRY and
change no signature", and it verified this by *reading rung 55's and rung 61's method bodies* to
see which fields they touch. That reading was correct and the conclusion was still wrong twice:
the state rung 55 needs is not the scalars the bodies name, it is the two BUILT stack objects
(runtime-length ladders ⇒ the descendant enum loses `Copy`), and rung 55's overrides live on a
*different* hook table than the one (c) was reasoning about, which rung 53's constructor
hardcodes. **A body-read tells you what state a method READS; it cannot tell you what the state's
CARRIER costs.** Same shape as [[rust-port-slice-k]]'s *a scope list is only as good as an
enumeration over that set* — a claim about a set, verified on a proxy for the set.

**How to apply:** When a pre-registration decides a shape for a FUTURE slice, the check is not
"which fields does the body read" but "can the current carrier HOLD what the body reconstructs,
and does the override land on the table I am holding". Write the future slice's constructor call
in your head, not just its body. And expect the pre-registration to be the thing the probes
refute — that is the pre-registration earning its keep, not failing.

Two more from the same probes, both worth reusing:
* **The opposite sign exists too.** Slice M's step-1 refactor had already made the two efficiency
  loops fallible, so slice N's new raising site needed no signature change — the first time a
  predecessor's refactor PREVENTED churn rather than a zero-firing verdict expiring under a new
  caller ([[rust-port-slice-l-step1]], [[rust-port-slice-j]]).
* **A shared counter hides a dead guard.** Two floors add into ONE `clamped` count, and both
  readers read only the sum — so no existing gate could distinguish the guard firing 3 204 times
  from the one firing 0. Measure per guard, gate per guard ([[rust-port-documented-gate-that-doesnt-exist]]).
* **An argmin can be decided by the last bit.** At the design throttle the per-row margins agree to
  1–2 ULP and some are bit-identical, so `binds`/`inc_worst` are a TIE-BREAK there, not physics —
  and a bit-exact value oracle is blind to it because only the INDEX flips
  ([[rust-port-location-keys-refute]]).
