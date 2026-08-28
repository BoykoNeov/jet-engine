---
name: rust-port-slice-ac-step2
description: "Slice AC (rungs 70/71) step 2 — a gate whose CODE and whose COMMENT described different properties, and the step-1 lesson only asked about deletion"
metadata: 
  node_type: memory
  type: project
  originSessionId: 1499d79a-9e7d-4273-b5b9-31344173c7de
  modified: 2026-08-28T19:24:08.082Z
---

Slice AC step 2 ported rung 70's bodies: `src/cross_split.rs` 290 → 1 779 lines, all nine
remaining methods plus the seven readers, five complex operations added beside `C64`. Plan record
at § 5.27.2.

**A STEP-1 GATE WENT VACUOUS, BUT NOT THE ONE STEP 1 SPENT ITS EFFORT PROTECTING.** Step 1
deliberately refused slice AB's read-the-panic-messages gate and wrote pointer inequality instead,
asking *does the next step DELETE the thing my gate reads?* — and that repair held, every swap gate
survived the bodies landing. What broke was `assert_eq!(cross.matches("\npub struct ").count() +
… , 1, "the only new struct is GovScope — no table type is added")`. The **doc line directly above
it** already named the real property (*no `pub struct .*Hooks`*); the code was a PROXY that
coincided while the files held one struct. Step 2 added eighteen reader RETURN types and the gate
failed on a change P1 is indifferent to. **A gate whose code and whose comment describe different
properties is already broken, and only one of the two will notice.** The other half of step 1's
question is *does the next step ADD something my gate counts?* Repaired to a per-line
`pub struct …Hooks` filter, zero in each file, plus a positive control over `three_loop.rs` so the
detector proves it can SEE.

**THE HIGHEST-CONSEQUENCE LINE WAS THE ONE WHERE THE TOOL I HAD JUST BUILT WAS THE WRONG TOOL.**
`_gov_max` is written two ways — a bare permanent set on a fresh machine (`_split_rig`) and a
save/restore guard (`_with_gov`). Step 1 shipped the guard, so the guard is what is in hand; using
it in `_split_rig` restores on drop, `_triple_laws` takes its reduce arm, and every reader measures
rung 68 — which returns SUCCESSFULLY WITH ZERO ROWS, so a value-diff gate passes on two empty
tables. `split_gains` uses both spellings in one body and swapping them also diffs clean.

**THREE SEPARATE WAYS TO LOSE ONE PRE-REGISTERED PREDICTION.** `_rk4_floor_split` is not the
`rk4_floor` cell (3 args vs 4), must keep its call site inside the march (rung 71's floor shadows
it — hoisting leaves rung 71 bit-identical and deletes rung 70's only guard), and must use Python's
summation order rather than the parent's Rust template.

**A DOC COMMENT WENT FALSE BY ADDITION AND WAS CORRECTED IN THE SAME PASS.** `C64` said three
operations were needed and *"a fuller one would invite a reader to compose operations whose Python
counterpart was never called"*. Rung 70 calls a complex sum, product, sqrt and division. The rule
was kept, the census widened, and the corrected sentence is *a census is only as wide as the rungs
that have been read*.

**AND THE DRIVE-ONCE CHECK BEAT ANY GATE I COULD HAVE WRITTEN**, because one inherited reader
reproduced a number the pre-flight had quoted from Python end-to-end: `rung67_control` returning
`n = 7, ratio = 0.921` exercises `split_gains`, the bare set, `at_lever`'s freshly-unarmed sibling,
the rung-70 march and rung 67's `cross_identity` **on its own defaults** (not the caller's grid) —
in one number.

**Why:** every one of these is a property asserted somewhere other than where it is enforced —
in a comment, in a tool's shape, in a call site, in a summation order.

**How to apply:** when a step's gate fails, ask whether the gate or the property was wrong; diff a
gate's comment against its code before trusting either. Ask what the next step ADDS as well as what
it deletes. And before declaring a port step done, drive it once against a number the source
already published.

Related: [[rust-port-slice-ac-step1]], [[rust-port-slice-ac-preflight]], [[rust-port-slice-ab-step2]],
[[rust-port-slice-w-step3]], [[rust-port-power-spelling]].
