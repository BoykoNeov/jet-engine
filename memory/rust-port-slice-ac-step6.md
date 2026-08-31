---
name: rust-port-slice-ac-step6
description: "A pre-flight table quoted one stride-independent count beside five stride-dependent ones, so the only column that agreed with the suite was the only one that could not disagree"
metadata: 
  node_type: memory
  type: project
  originSessionId: f0d438fb-b4ba-467d-b0e5-f1af8995bcd2
  modified: 2026-08-31T16:53:12.060Z
---

Slice AC step 6 (rungs 70/71 Rust port) shipped `rust/oracle/dump_slice_ac.py` (14 sections,
**5 351 keys**) and `rust/tests/slice_ac_oracle.rs` (5 gates). **`Rust ≡ PyPy` on all 5 351 keys,
green on the first run** — no port fix needed, because steps 2/3 had already driven every reader
and diffed the printed values.

**THE LESSON: when a table mixes a count taken BEFORE a sampling stride with readings taken AFTER
it, the column that agrees with the suite is the one that cannot disagree.** The pre-flight's
§ (ii) — the headline that a swap *empties the sample* — printed `n_riding = 61` (the suite's),
`len(rows) = 2`, `max_pair_gap = 1.163`, and a two-element `pair_RC`. The shipped fixture passes
`every = 10` and returns **7 rows**, `max_pair_gap = 1.1766`, seven `pair_RC` values. All five
stride-dependent numbers reproduce exactly at **`every = 40`**. `n_riding` counts riding points
before the `[::every]` slice, so it agreed no matter what stride was used. **Step 2 had already
published `n = 7` from `rung67_control` — two numbers for the same quantity, one page apart in the
same document, and nobody diffed them.**

**AND THE PREDICTION ABOUT THE CROSS-INTERPRETER EXEMPTION WAS WRONG IN BOTH DIRECTIONS.** P8
named two subtrees, both reader-side. Measured over 234 exempt names:

* the `cross_identity` subtree contributes **ZERO** — its only reader is section B and not one B
  key drifts, even though a probe had measured that site diverging 1 of 1. *A site diverging and
  a returned value diverging are different questions;* name the KEYS a dump emits, not the SITES
  a probe finds. (Slice AB's P3 named a subtree that did not appear either.)
* **119 of the 234 are the PLANT.** A probe over sixteen arms under both interpreters found the
  MARCH itself diverging on exactly three, always **first in the stator state `v`**, by 10–11
  ULPs — with every other state at that index and the whole preceding point bit-identical. Inputs
  equal and output 10 ULPs apart is a **solve terminating differently**, not a formula rounding
  differently. It decays back to bit-equality by the end of the ramp, and which arms it hits is
  not monotone in any clock.

**A GATED CONDITION MUST BE RE-READ ON A POPULATION THAT CAN REACH IT.** The plan said step 6
should re-read P6 (`p` is real inside `zeta_pair`) *from the dump*. The dump's own coverage line
measured **0 of 38** intercepted `p` complex — the readers cannot reach the branch at all, so
doing that would have re-published the measurement step 5 had already falsified. A DECLARED EXTRA
GRID (the constructed spectra the shipped damping gate uses) covers it with value keys instead.
Same shape as [[rust-port-slice-ac-step5]], one step on: *readers and the shipped suite are
different populations.*

**Three of my own instruments printed success having measured nothing**, all in one hour:
a patch script that called `str.replace` and printed `patched` unconditionally (it had matched
nothing, and the harvest that followed reported 12 drifting keys where there were 234);
`cmd 2>&1 > file`, which sends stdout to the file and stderr to the console, so a failing test's
panic text vanished and the file held 16 bytes; and `GOLDEN_KEYS` typed as `39_099` from a guess
scaled off a neighbouring slice, measured at **5 351**. The repair for the first is `assert old in
s` before the replace — *the assert is what makes the success message a measurement*.

**How to apply:** before quoting a table of readings, ask which of its columns are computed before
the sampling and which after — and treat a column that agrees across two grids as evidence of
nothing until you know it *could* have disagreed. And when a prediction names a subtree, check the
keys that subtree actually reaches, in both directions: what it contributes, and what contributes
outside it.
