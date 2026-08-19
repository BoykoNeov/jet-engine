---
name: rust-port-slice-t-step1
description: "Slice T step 1 — an EXACT ZERO makes its own gate blind to the sign, and the only real defect came from a cell no suite has"
metadata: 
  node_type: memory
  type: project
  originSessionId: 387b9e3e-994c-4785-963d-3fb30836e567
  modified: 2026-08-19T10:00:23.693Z
---

Slice T (rungs 46/47/48, gates only) step 1: the four reader methods plus rung 46's six gates.
All six passed first run; everything the step actually learned came from elsewhere.

**The measurement that makes a finding SHARP can be what makes its gate BLIND.** Rung 46's
headline is that the temperature governor rebates surge margin on one shaft and gives the other
*exactly zero* — and the port reproduces that zero bit-for-bit. But the gate for it asserts
`|relief_lp| < 1e-9`, so flipping the sign of `relief_lp` in the shipped reader **passed**: a sign
flip on an exact zero is invisible. The same flip applied to the other shaft, whose values are
2.7e-3 to 3.6e-3, was caught by two gates. So the blindness is not the suite's, it is the
exact-zero half's specifically, and the exact zero *is* the mechanism. Sibling of
[[rust-port-slice-s-step2]] (a non-strict ordering gate satisfied by deleting its own variable).

**How to apply:** when a gate's bar is an absolute tolerance around zero, ask what else besides
the truth satisfies it — sign, and anything else that also lands inside the bar. Inject and
measure rather than reasoning about it; three of four injections here were caught by exactly ONE
gate each, which is also how you learn the gates are not redundant.

**The one real defect came from a cell no suite has, and it was in phase-2 code.** Python's
`_sonic_throat` bracket check is an `AssertionError` that every marcher's `except AssertionError:
break` catches; Rust ported it as `assert!`, which panics past the whole `Result<_, Abort>` chain
built to model exactly that. Python returns an empty trajectory, Rust crashes. Found by running
the neighbouring cell Python's own gate leaves open, not by any ported gate — the shape of
[[rust-port-oracle-cannot-see-a-missing-gate]], and of slice S's own added-cell finding.

**Booked, not fixed, and the reason is worth keeping:** a PARTIAL conversion of the 28 call sites
would leave some paths refusing and some panicking with no principle separating them — harder to
reason about than today's uniform "asserts panic". Disclosed with a gate on *both* branches
(panic-today, empty-trajectory-after-repair), so the test keeps passing when the repair lands
instead of being a tripwire someone repairs by deleting the assert.

**Two process slips.** (i) I wrote "the crate is green" into the plan before the run finished —
the same class as the `#[ignore]` range that excluded its own most recent measurement. (ii) I
twice invalidated a ~19-minute background crate run by editing sources while it was in flight.
Finish the edits, *then* start the long gate.

**A count from a name grep is a floor, not a census.** "10 of 28 call sites are in fallible
chains" came from matching a `try_` prefix, which is wrong in both directions. Recorded as a lower
bound with its method named — see [[rust-port-guessed-census-bars]].
