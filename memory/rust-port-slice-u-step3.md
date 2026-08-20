---
name: rust-port-slice-u-step3
description: "Slice U step 3 — a function's one job can be exercised only on cells chosen for inertness, so breaking it moves nothing but the echoed argument"
metadata: 
  node_type: memory
  type: project
  originSessionId: 3688e397-f7fc-4957-8677-b3e66aa2c742
  modified: 2026-08-20T11:37:16.774Z
---

Slice U step 3 (rung 51's release RATE: `rate_sweep` + `deficit_curve` + 16 gates) shipped
2026-08-20. 972 keys over 36 cells bit-exact on the first run; 65 lines added, ZERO deleted, and
that diff IS the discharge of the pre-registered prediction that rung 51 adds no new logic.

**A FUNCTION'S ONE JOB CAN BE EXERCISED ONLY ON CELLS CHOSEN FOR INERTNESS.** `rate_sweep` exists
to forward the rate. Delete the forwarding entirely and **2 of 972 keys move, both of them the
record echoing its own argument back** — every physical quantity was already bit-identical. The
reason is structural: of the four `rate_sweep` rows the whole suite produces, exactly two carry a
live rate and both belong to the one contract whose CLAIM is that the rate is inert there. Every
gate that reads a genuinely faded march bypasses the function. **"Two keys moved" is a different
finding from "no gate caught it", and only measuring WHICH keys separates them** —
[[rust-port-slice-u-step1]]'s did-it-move-a-value column needing a second column.

**READ THE CELL LIST OFF THE SUITE, NEVER ENUMERATE IT.** The audit probe runs all sixteen gates
and then dumps the suite's own memo dict. A hand-written cell list is this port's standing way of
under-covering a suite ([[rust-port-slice-s-step4]]); reading the memo cannot miss a cell a gate
visits nor invent one it does not. It also surfaced something the gates never say: **all 29
memoised cells run at one ramp rate**, because the helper's defaults are the deep-dive cell where
the neighbouring rung's default is the other one. A gate that reads as regime-agnostic was not.

Also: a prediction registered as "caught by gate A and not gate B" came back caught by BOTH —
because the second was a plant REFUSAL firing, not a value comparison. **"Two gates caught it"
reads as two independent checks and can be one refusal reaching two call sites.**

See `docs/plans/todo-rust-port.md` § 5.18 step 3. Related: [[rust-port-slice-u-step2]],
[[rust-port-slice-t-step3]].
