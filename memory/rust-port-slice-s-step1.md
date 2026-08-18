---
name: rust-port-slice-s-step1
description: "Slice S step 1 (rungs 43/45) — the plan's own table lost a row its prose counted, and the two injections that reported 'nothing moved' could not have moved anything"
metadata: 
  node_type: memory
  type: project
  originSessionId: ab3d5fe8-03bc-4efd-b939-bfbd7e23d339
  modified: 2026-08-18T22:00:29.821Z
---

Phase 6 slice S step 1 (rungs 43 + 45, `TwoSpoolFuelTransient` → `rust/src/fuel_transient.rs`,
2 225 lines) shipped 2026-08-19: the port + `oracle/dump_slice_s_smoke.py` +
`tests/slice_s_smoke.rs` + `tests/slice_s_dispatch.rs`. **5 536 values bit-exact against PyPy**;
name diff 713 → 715, 0 removals. Steps 2–5 remain. Detail: `docs/plans/todo-rust-port.md` § 5.16.

**A PLAN'S SUMMARY TABLE CAN LOSE A ROW ITS OWN PROSE COUNTS.** § 5.16's armed-limiter table
listed eight cases while the paragraph beside it said *"six of the nine"*. The missing one was the
composite — the only route through the lagged twin **with both min-select legs armed**, i.e. the
only one exercising that twin's `faded` (which references the SCHEDULED fuel where the bare
marcher's same-named closure references the APPLIED fuel). **When a count in prose and a count in
a table disagree, go back to the probe that produced them** — reconstructing the table from the
prose would have shipped an eight-section dump against a nine-section gate. Related:
[[rust-port-slice-n-step4]], [[rust-port-slice-r-step1]].

**AN INJECTION THAT CANNOT EXPRESS ITSELF REPORTS "NOTHING MOVED".** Two in one gate, both mine.
One perturbed a struct field **nothing downstream reads**, so it moved nothing anywhere. The other
SCALED a residual the solver drives to zero — which leaves the root exactly where it was and moves
only the iteration path, reporting a 1-ULP difference that reads as agreement. The rule the plan
already had ("perturb the LIVE cell first and watch it move") is necessary and **not sufficient**:
each injection also has to be shown to reach a consumer. Same family as [[rust-port-slice-r-step3]]
and [[rust-port-slice-q]].

**AND THE PROBE THAT PASSED WAS ASKING A DIFFERENT QUESTION.** "Swapping the other two hook cells
moves nothing in this slice" is TRUE of the fuel path (they are never *called*) and FALSE of the
object — the ramp builds its endpoints and running line out of the *previous* rung's equilibrium,
which uses both. **A claim about a code path is not a claim about the object that owns it.**

**A PARTITION SUM CANNOT SEE A MISSING ARM.** The closure's three-way high wall was gated by
`literal + map + hi0 == calls`, which passes identically whether the third arm binds or is absent
from the source — and it bound **zero** times across all 33 census sections. It is the port's most
prominent departure from the rung it resembles. Fixed by locating where it binds (a threshold, not
a guess) and adding a section there. **Gate the arms, never their total.** Related:
[[rust-port-slice-m]], [[rust-port-guessed-census-bars]].

**A PLATEAU IS A PROPERTY OF THE GRID.** The argmin tie-break gate reported `tied = 0` on a cheaper
grid — so it could not have fired. On the source gate's own grid the plateau is exactly as
pre-registered. Third time this shape has appeared ([[rust-port-slice-q]],
[[rust-port-slice-r-step1]]): **before writing a gate for a rare event, confirm the event occurs on
the grid the gate runs.**

**FALLIBILITY IS PER CALL SITE — a measurement can expire without anything changing.** An earlier
slice measured a gas inverse firing 0 times and kept its panic on that basis. This rung's closure
scans a wider bracket and reaches it 8 times inside a caught scope, so the twin had to be added.
Its 46 swallowed failures are **38 + 8**, two arms from two different files, gated as two counters:
*a registered SUM is not a gated SPLIT*. Related: [[rust-port-slice-l-step1]],
[[rust-port-oracle-cannot-see-a-missing-gate]].

**Two instrument defects the dump found in itself:** the Rust reset counters one statement too
early and hid 39 shared-counter calls the Python section legitimately carries (**census keys are
sensitive to statement POSITION between emit boundaries**), and a pass counter counted evaluations
where the Python recovery counts completed passes.

**Windows hazard, hit again:** `io.open(path, "w", newline="\n")` with a mis-escaped newline
argument **truncates the file to 0 bytes before raising** — the dump had to be rewritten from
scratch. Validate a patch script's arguments before it opens the target for writing. See
[[windows-tooling-file-hazards]].
