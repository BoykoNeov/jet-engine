---
name: rust-port-slice-z-step4
description: "Slice Z step 4 — a pre-registered exemption of TWO keys measured EIGHT, because the pre-registration counted quantities and the dump emits names"
metadata:
  node_type: memory
  type: project
---

Slice Z step 4 (2026-08-27) shipped `oracle/dump_slice_z.py` + `tests/slice_z_oracle.rs`:
**35 335 keys, bit-exact vs PyPy on the first run**, and vs CPython 3.14 with one declared
exemption. Rust arm 16.85 s; dump 70 s per interpreter. Nothing coarsened — including the two
places the suites NARROW a default, because an oracle mirrors GATES and a default the suites never
use is a grid they do not have.

**The leading lesson: a pre-registered exemption was a count of QUANTITIES and the dump emits
NAMES.** § 5.24 (i) named `P_mid` and the `T_over_tau` it feeds — two keys. Measured: **eight key
names**, because `P_mid` is re-published under four further names (as `oscillation_window`'s `P`,
again inside its `window` sub-dict, once more where a leaf section re-evaluates the window at it,
and a fifth time on a second grid). An exempt list transcribed from the pre-registration would
have carried two entries and the oracle would have failed on six more.

**Why:** it is the same shape as the slice's own leading finding, one level out — *a number
written down once reads like a measurement*.

**How to apply:**
- **Derive an exemption list by DIFFING the two committed dumps**, never by transcribing a probe's
  table. The probe measured a quantity; the dump emits every name that quantity reaches.
- **Check the list in BOTH directions.** Assert the exempted set EQUALS the expected names: a
  ninth key failing is a defect, and one of the eight *ceasing* to drift is a change too.
- **Emit the count a stride DELIVERS, not the one requested.** `ride[::max(1, len(ride)//n)]`
  delivered 9 for a requested 8 and 7 for a requested 6, at every grid. Two rows of this dump are
  the SAME clock on the SAME trajectory (`n_ride = 97`) sampled 9 wide and 7 wide — **the 9-wide
  one diverges from CPython and the 7-wide one does not.**
- **Emit the zeros nobody asked for.** `n_skipped` and `n_failed` are both 0 here, and
  `all_converged` is a Python `all(...)` that is vacuously true over an all-failed row set.
- **Re-run the step's injections with the oracle as a target.** It closed two of step 3's four
  blind spots — and one of those **retired a prediction made one step earlier** (P12, booked to
  step 5 on the grounds that no oracle could reach it; a synthetic-ramp section reaches it).
  Record the retirement rather than dropping it.

Related: [[rust-port-slice-z-step3]], [[rust-port-slice-w-step4]], [[rust-port-guessed-census-bars]],
[[rust-port-oracle-cannot-see-a-missing-gate]].
