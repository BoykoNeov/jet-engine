---
name: rust-port-slice-w
description: "Slice W (rungs 62/63) — a plan's cell column was short by four names, and only re-running the census over the WHOLE phase found them; a smoke section on a path the slice cannot reach is what tells a wrong port from a wrong grid"
metadata: 
  node_type: memory
  type: project
  originSessionId: 8326bca7-b684-4023-9c87-ee3d60d8c006
  modified: 2026-08-26T09:46:52.902Z
---

Slice W of the Rust port (rungs 62/63, the bleed schedule on the transient plant),
pre-registered 2026-08-26 as § 5.21 of `docs/plans/todo-rust-port.md`. Three process lessons,
all of which cost a run or would have.

**A CELL CENSUS SCOPED TO THE SLICE MISSES THE ERRORS THE PLAN ALREADY MADE.** The phase plan
booked slice W as adding four cells and classed `at_lever`/`at_stator` as "pure sibling
constructors" needing none. `at_lever` turned out to be overridden by **17** downstream classes
and called **46** times — the most-dispatched name in the phase. Re-running the same emitter over
**every remaining ladder class**, not just this slice's, found four such names in total and put
the phase's measured cell count at 35 against a hand-written 28. **Why:** the error was in the
plan's method, not in one row, so one row-sized check could never find it. **How to apply:** when
a probe refutes a table entry, run that probe over the whole table before writing the section —
it is one loop and it catches the error class instead of one instance. See
[[rust-port-guessed-census-bars]], of which this is the membership-list form.

**A DEFERRAL JUSTIFIED BY A BODY READ IS NOT A MEASUREMENT.** Slice V booked `at_stator` as inert
"because rungs 57–60 build only rung-57 objects." True of slice V, false of what slice W
inherits: a shipped rung-63 test calls an INHERITED reader on a bleed-armed machine and asserts it
returns the parent rung's exact identity *for free* — a counterfeit the override exists to pin.
Forcing the un-overridden body flipped both identities. **How to apply:** before booking a
deferral, grep the SUITES for the name, not just the source; a gate that reads the thing is worth
more than any argument about who calls it.

**A SMOKE DUMP NEEDS A SECTION ON A PATH THE SLICE CANNOT REACH.** The first smoke run failed 243
of 522 keys by 1–10 ULP, spread across every section — all individually plausible as arithmetic
slips in the new closures. The section that localised it was the REDUCE, on a bare machine with no
valve and no stator, which the new code never enters: a defect reaching there is the GRID's, not
the port's. Cause: the dump re-spelled the gas constant `R_c=(gamma-1)/gamma*cp` as its
arithmetic value `0.4/1.4*cp`, and `1.4 - 1.0` is `0.3999999999999999` in IEEE-754, so the whole
file ran on a gas one ULP away. **Why:** without a control section, the only way to tell "my port
is wrong" from "my instrument is wrong" is to re-derive the port line by line. **How to apply:**
copy a suite's constants character for character rather than evaluating them, and always carry one
smoke section whose path the slice cannot touch. [[rust-port-copy-vs-rederivation]] pointed at the
instrument instead of the port.

After the grid was fixed the port needed **no numerical correction**: 522/522 bit-exact, first
run, zero tolerance tiers. Remaining: step 3 (88 ported gates + injections), step 4 (the oracle),
step 5 (the dispatch and manufactured-bug gates). See [[rust-port-decided]],
[[rust-port-phase7-preflight]], [[rust-port-slice-v-step5]].
