---
name: rust-port-slice-ag-step5
description: "Slice AG step 5 (rung 76's readers) — a mutation sweep's verdict is a property of the GRID, and a grid copied from the test suite inherits the suite's coverage, not the code's"
metadata: 
  node_type: memory
  type: project
  originSessionId: 61332565-1b3b-4ef7-8512-5709495eae9c
  modified: 2026-09-07T05:46:54.035Z
---

Slice AG step 5, 2026-09-07 (§ 5.31.5). Rung 76's four readers ported into
`M:\claud_projects\jet engine\rust\src\sensed_cap.rs` (578 → 1 319 lines): `cap_rows`,
`cap_gains`, `cap_bill`, `solve_gain`, their row/cell types, the Python-faithful folds
`py_max`/`py_min`, and `REF_SCHED`. No gate file — steps 3/4's precedent. **1 814 keys,
`Rust == PyPy` bit for bit on the first run that ever compiled, both key sets equal, no port fix.**
Sweep: 16 injections, 12 killed, 4 survived, **16 of 16 verdicts right**.

**THE LESSON: a mutation sweep's verdict is a property of the GRID it is scored on, and a grid
copied from the test suite inherits the SUITE's coverage, not the CODE's.** The drive was built
from `tests/test_rung76.py`'s arguments, the way steps 3 and 4 built theirs, and came back exact on
1 266 keys. Reading the golden for what it could SEE found two tells, both visible in the emitted
TSV without running anything: an aggregate (`n_inert`) that was **0 in all eight cells**, and an
optional (`row_err`) that was **absent in all four live ones**. So at the suite's own margin the
three-`_cap_fuel` min-select guard filters nothing and the governor-only branch never runs. Adding
a second margin (0.20) made both live — and then **two injections that score SURVIVED on the
suite's grid score KILLED on the wider one**: `accel_binds` folding `min` for `max` (0 keys → 33)
and `row_err`'s two targets inverted (0 → 4). Both are real defects in load-bearing expressions,
and on the first grid this step would have written them up as *defences with no reader* beside the
four genuine ones.

**Why:** [[rust-port-slice-w-step3]]'s *make the instrument prove it can SEE* has to be asked of
the GOLDEN before the sweep runs, not only of the probe. A suite's grid is chosen to make the
suite's assertions fire; the assertions a suite makes are not the expressions a port has to get
right. This is [[rust-port-slice-ag-step4]] § (b)'s *a drive that picks one cell scores the bug as
correct* with the cell chosen by INHERITANCE rather than by economy.

**How to apply:** before scoring a sweep, grep the golden for aggregates that are exactly 0 and for
`?`-flags that are 0 in every cell. Each one is a branch the sweep cannot score. Then widen the
grid until they move — and if they will not move, say so as an unreachability rather than letting
the sweep report it as a passing spelling. **Booked forward: the ported gates at step 6 inherit
the same grid by construction, so the second margin belongs in step 6's file too.**

**SECOND: the port's line-citation guard caught a citation that was WRONG AT BIRTH, which its own
docstring says it cannot do.** It is documented as detecting DRIFT, not wrongness — *no instrument
can know what a comment MEANT to point at*. But blessing a NEW anchor **prints the line it lands
on**, and `19353: None -> 'g = read("sensed")'` did not match the sentence, which is about the
`accel` argument one line up. So the stated limit binds only for a citation blessed silently; when
the blessing prints, birth defects are caught for free. Now in the guard's own census comment.
Census `10 / 59 / 47` → `10 / 63 / 51`, **no arrears** — step 4's procedural repair held, and this
step ran `pytest` before shipping rather than two steps later.

**THIRD: five defences with no reader in one step, all five pre-registered.** Step 4 shipped one.
A READERS step ships more because an aggregate's edge cases are exactly the states a converged
march does not visit: the `tau_auth`/`tau_masked` distinction (both clocks are `0.05`), the two
scope orders (disjoint fields), `masked_moved`'s conditional (a live 4×4 diagonal is never within
`1e-30` of zero), `s_tail`'s `max(taus)` (four equal clocks), and `py_max`/`py_min`'s NaN
faithfulness (`dS == 0` at 0 of 29 rows). Not a regression — a property of the kind of step.

**FOURTH: an identity that holds EXACTLY makes the obvious gate on it self-certifying.**
`solve_gain`'s `fixed_point` is `+0.0` bit for bit at **8 of 10 rows at the suite's own margin**, so
a reader that wrongly compared the solve with ITSELF returns the same float there; the injection is
caught only by the minority of rows where the last bits differ (17 keys of 1 814, none at those
eight). [[instrument-fed-by-what-it-certifies]] with a twist — the exactness doing the certifying
IS the fact being certified. Booked as a requirement on step 6.

Also measured: dropping the `accel` argument in the READER is **silent** — killed by value at 144
keys with no panic — where step 4 measured the same deletion in the MARCH panicking on
`integrate_fuel`'s third refusal. So that refusal defends the march's callers and **nothing**
defends the readers; three steps, three answers to *what refuses this*, and the third is *nothing*.
`fuel_int`'s fold direction moves **exactly 4 keys**, which is P2 asked of the port. Sizing
**2.48× by region, 2.35× by body**, both inside P1's band, against step 4's 5.09×/3.15×. And the
sweep's own classifier scored two panics as compile errors, because cargo prints `error: test
failed` — no verdict moved, the label was wrong, recorded rather than quietly fixed.
