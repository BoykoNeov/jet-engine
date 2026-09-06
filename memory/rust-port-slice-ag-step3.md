---
name: rust-port-slice-ag-step3
description: Slice AG step 3 (rung 75's six readers) — an enumeration of who reads a field expires at the rung that adds a reader, so a write I proved inert by rung 74's evidence panicked
metadata:
  type: project
---

Slice AG step 3, 2026-09-07 (§ 5.31.3). Rung 75's six readers ported into
`M:\claud_projects\jet engine\rust\src\anti_windup.rs` (601 → 1 531 lines): `rhs_gains_at`,
`windup_rows`, `windup_gains`, `contraction_law`, `device_control`, `windup_bill`, plus
`IcCapScope` and `try_windup_march`. No gate file — slice AF step 4's precedent, a readers step
proves itself by DRIVING. **1 304 keys, `Rust == PyPy` bit for bit on the first run that compiled,
both key sets equal, no port fix.** Sweep 15 injections, 12 killed, 3 survived, ONE misprediction.

**THE LESSON: an enumeration of a field's READERS has an expiry date, and it expires at the rung
that adds one.** `_windup_rows` writes `m._lag_coord = "demand"` by plain assignment. I
pre-registered `SURVIVED` for deleting it, with what looked like proof: rung 74's own measurement
says `clip` and `demand` are indistinguishable in `demand_target` BY CONSTRUCTION, and
`demand_target` was the tag's only reader. **Measured KILLED, by a panic** — `r75_windup_tau`, the
cell this whole slice exists for, REFUSES `track` outside the demand coordinate. The march runs
`clip × none` legally because the device is disarmed, and the flip is the `track` scope's admission
ticket. The reader was born at the rung being ported, so rung 74's enumeration was stale in exactly
the place the port was being written.

**Why:** this is [[rust-port-slice-ag-step1]]'s *a line citation has an expiry date* on a different
object, and it is the mirror of [[rust-port-slice-af-step6]]: that step found four sites that
DISPATCHED a write Python makes plainly; this one found a plain write whose liveness comes from a
reader nobody had re-counted. Both are answered by the same question, asked at the right rung —
[[rust-port-slice-aa-steps2345]]'s *ask what reads a thing, never wait for a failure*.

**How to apply:** before predicting that a carrier write is inert, grep the CURRENT rung's class for
readers of that field, not the parent's. A prediction whose warrant is a measurement taken one rung
down is a prediction about the wrong machine. The corollary for the port: a plain assignment is not
automatically a no-op just because the previous rung measured its two values indistinguishable.

**SECOND: a reader that drops its predecessor's skip census makes its own coverage unaskable.**
Rung 74's `demand_gains` — the direct parent of the reader being ported — counts BOTH of its drop
branches into a `skipped` pair, with Python's comment *a dropped point is a coverage claim*. Rung
75's `_windup_rows` has the same two branches and counts neither, so `windup_gains` reports
`n = 7, n_riding = 56` and 49 discarded points are unaccounted.
The drive's declared-extra section E measured the same march unfiltered: **18 of 22 sampled points
rejected**, in two populations (14 with all sixteen labels off-regime, 3 with eight, 1 near-switch).
A gate written against `windup_gains` cannot tell a correct filter from one that drops all but
seven — proved rather than asserted, by an injection that short-circuits the regime scan and is
invisible to every key outside E. (A grep for the literal `skipped=` also finds none between
`engine.py:18311` and `21767`, but that is ONE SPELLING and not a swept set: two of rungs 76–79 are
reader-only with no Jacobian, so the question may not arise there. The parent-child comparison is
the finding; the span is context — [[rust-port-slice-ab-step5]]'s over-wide exemption otherwise.)

**THIRD: four of the rung's six readings are EXACTLY 0.0, and one of them cannot be scored at all.**
`auth_diag_moved`, `track_leak`, `mask_leak`, `mask_leak0` are the bit pattern zero at all four
cells. Widening `mask_leak`'s fold to include the diagonal kills on 60 keys, so that zero is live;
deleting one of `track_leak`'s three terms SURVIVES, because all three are identically zero on the
leg that holds — [[rust-port-slice-t-step1]] in a new place, and [[rust-port-slice-ag-step2]]'s
N-site lesson one level down. Two further vacuity facts for step 6: `handover_monotone` is
vacuously true when at most one row hands over and Python carries no key for the count, and
`device_control`'s dormant-set asymmetry is live at `tau_t = 0.05` and INERT at `0.0125`, so a gate
driven at the fast cell alone scores the symmetrised bug as correct.

Also measured: `contraction_law`'s six predictions (185, 98, 54, 32, 20, 14) derived by hand from
`ceil(ln(tol/res0)/ln σ)` before any run, all four shipped ones exact; its typed `res0 = 2.898e-3`
is a ROUNDED four-figure version of `device_control`'s measured `0.0028982406470635016` on the same
plant — step 1 § (g)'s shape a second time in one slice. And `_rk4_floor_shared` admits equality, so
both spellings of the grid floor sit exactly ON the boundary at `ds*rate == 2.0`.
