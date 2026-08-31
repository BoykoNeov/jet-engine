---
name: rust-port-status
description: "Where the Rust port stands — phases done, the slice currently in flight, and the numbers each closed slice landed on"
metadata: 
  node_type: memory
  type: project
  originSessionId: 57f6e146-1b56-46c8-a924-d490be2f24f2
  modified: 2026-08-28T09:14:08.040Z
---

The living status line for the Rust port. **This file is the only place the running tally lives**;
`MEMORY.md` carries a one-line pointer to it, because the tally grows every step and an index line
that grows is not an index line. Update THIS file at each step, not the index.

**Decided 2026-08-12.** Plan: `docs/plans/todo-rust-port.md`. A new PHASE needs authorisation;
slices inside an authorised phase are free. See [[rust-port-decided]].

## Phases

* **0–5 DONE.**
* **PHASE 6** authorised 2026-08-17, **COMPLETE 2026-08-20** with slice U's five steps.
* **PHASE 7** authorised 2026-08-20. Slices V, W, X, Y, Z, AA, **AB complete**; **AC (rungs 70+71) IN FLIGHT** (§ 5.27, fourteen probes) — **steps 1–6 of 7 done**.

## Phase-7 slices, as they closed

| slice | rungs | landed |
|---|---|---|
| V–Y | 57–65 | slice Y closed 2026-08-27, all five steps |
| Z | 66+67 | 2026-08-27 — both marches + 20 method bodies, **38 ported gates** green first run at **20.0×** PyPy, **35 335 oracle keys** bit-exact on both interpreters (CPython exempt on 8 named keys), **8 dispatch gates** closing the four measured blind spots |
| AA | 68 | 2026-08-27 — `three_loop.rs` 2 302 lines, **47 gates**, **12 084 oracle keys** bit-exact vs PyPy (4 named CPython exemptions), full gate **124 blocks / 1 199 passed / 0 failed** |
| AB | 69 | **2026-08-28, all five steps** — `reference_split.rs` 1 686 lines, 25 ported gates, **15 957 oracle keys** bit-exact vs PyPy (194 named CPython exemptions), and **14 dispatch gates** over TEN cells, every one observable; full gate **129 binaries / 1 258 passed / 0 failed** |

## Slice AC — CLOSED, all seven steps done

Rungs 70 + 71, `docs/plans/todo-rust-port.md` § 5.27. **1 605 Python lines (2.27x slice AB),
57 collected tests (22 slow), ZERO cells added, 5 swaps over two rungs = 5 distinct function
pointers and NO new table field, 6 reduce arms all bit-for-bit by dispatch.** Rust estimated
3 400-3 800 lines. **SEVEN steps, pre-registered as P7** — the port and the gates each split by
rung, because five steps on a 2.27x slice would be habit rather than a measurement. Nine
predictions P1-P9.

**The plan's own column said 1 cell (`split_gains`) and it was wrong**: rung 80's same-named
method has an incompatible signature, so it is a NAME REUSED, not an override. The phase-wide
sweep of all 358 override pairs found one more — `_legs` (63 -> 77), a SHIPPED slice-W cell,
booked to slice AH. See [[rust-port-slice-ac-preflight]].

## Slice AC, step by step

1. **Step 1** — `src/cross_split.rs` + `src/full_split.rs`, `_gov_max`'s carrier + `GovScope`,
   **ten** `R70*`/`R71*` tables (the pre-flight's step list said nine; its own census enumerates
   ten), five swapped cells as named panics, `tests/slice_ac_cells.rs` — **10 gates**, nine
   deliberate mutations all caught. `build_reference_split_cascade` split into a
   table-parameterised body: neither rung defines `__init__`, so both inherit rung 69's eleven
   guards. See [[rust-port-slice-ac-step1]].
2. **Step 2** — the rung-70 bodies: `src/cross_split.rs` 290 -> **1 779 lines**, all nine remaining
   methods + the **seven readers**, `Census70`, and five complex operations beside `C64`
   (Smith's-algorithm division is the one that costs: 13/18 against a schoolbook spelling).
   A step-1 gate FAILED — not the one step 1 protected, but a `pub struct` count whose own doc line
   named a narrower property. All seven readers driven once and landing on the pre-flight's own
   numbers, `rung67_control` reproducing Python's `n = 7, ratio = 0.921` end-to-end. Full gate
   **130 targets / 1 268 passed / 0 failed** — unchanged from step 1, since the ported gates are
   steps 4–5. See [[rust-port-slice-ac-step2]].

3. **Step 3** — the rung-71 bodies: `src/full_split.rs` 171 -> **1 536 lines**, all ELEVEN methods
   (five non-readers + the six readers), `Census71`, `round10`, and no swapped cell left panicking.
   581 Python method lines -> 1 365 Rust, **2.35x**. All six readers driven at their SHIPPED
   defaults and diffed against PyPy: **every printed value identical, digit for digit** — the
   stator window 27 of 341 (7.92 %), the joint 7 (2.05 %), `members` 1 vs 4, matched-clock
   `zeta = 0.588974`. **The finding was in the PROSE**: the class docstring's `0.5895 / 0.5974` is
   not what `full_modes` returns on its own grid, and `docs/rung71-spec.md` § 5 had the right pair
   all along — corrected in the same pass. Full gate **130 targets / 1 268 passed / 0 failed**,
   unchanged, since the ported gates are steps 4-5. See [[rust-port-slice-ac-step3]].

4. **Step 4** — the rung-70 gates: `tests/rung70.rs` **754 lines, 27 gates**, green first run in
   5.75 s. The Python↔Rust mapping is **1:1 IN ORDER** (0 added, 0 collapsed, 1 body substituted —
   `at_lever_returns_this_class`), reconciled by NAME after a `grep` said 28 and `cargo` ran 27
   (the 28th sits inside a doc comment). Ten injections into `src/cross_split.rs`, **two** binaries
   each: **8 caught, 2 missed** — and both misses are missed by **Python's own 27 gates too**
   (27 passed under each), so they are INHERITED, not introduced. Both proven able to move: the
   clock-grid swap shifts 25 of 38 printed lines; the widened joint window goes 61 → 341 points and
   `joint_fraction` 0.179 → exactly 1.0. The gates survive because they pin SHAPE, not LOCATION,
   and their bars are one-sided; both are booked to step 6's oracle as named value keys. The file
   header's *"15 carry `slow`"* is corrected to the **measured 11** (22 of 57 over the slice). Full
   gate **131 targets / 1 295 passed / 0 failed**, predicted before the run and held.
   **Close-out finding:** `cargo clippy --all-targets` aborts on the lib's deliberate `eq_op`
   error and so had **never linted any `tests/*.rs`**; with `-A clippy::eq_op` it reaches 24 test
   targets and 48 warnings. The one in `rung70.rs` is fixed, the other 47 disclosed and booked.
   See [[rust-port-slice-ac-step4]].

5. **Step 5** — the rung-71 gates: `tests/rung71.rs` **1 016 lines, 30 gates** (11 `slow` in
   Python, MEASURED not typed). **TWO failed on the first run and neither was a transcription
   slip.** (i) Step 3's `assert!(p.im == 0.0)` inside rung 70's `zeta_pair` — a condition § 5.27
   (iv) measured over the READERS (18/18) — is falsified by rung 71's own damping gate, which
   drives a CONSTRUCTED spectrum where `p = 4462 + 4947i`; the port had already published the
   resulting `1.279` in `zeta_ring`'s doc comment at the same step. Replaced by `csqrt`, CPython's
   full `c_sqrt`, plus `porting_rules.rs` **RULE 4**. **And the `assert!` caught what the gate could
   not**: the wrong value is 1.624 vs Python's 1.278 and the gate's one-sided `> 0.5` bar passes on
   both — the mirror of step 4. (ii) My own gate asserted `Census70::triple_laws_gov > 0` on a
   march; that cell is READER-side, so all six counters read 0. Rewritten with its own control.
   Signed-zero census: **90 of 96** shipped `cmath.sqrt(p)` calls carry `im == -0.0` (I had it
   backwards), `p.re < 0` on **0** of them, `sqrt` bits differ on 91, the returned value on 1.
   Ten injections, both binaries: **7 caught, 3 missed**; every miss shown able to move something
   except j10, whose `round10` is a **defence with no reader** (`members` identical under both
   keys). The two analogues step 4 handed forward SPLIT: the clock reorder misses in both languages
   at both rungs (INHERITED family property), the `joint` widening is CAUGHT at rung 71 in both
   because its bar is two-sided where rung 70's were one-sided — partly discharging step 4's oracle
   booking. **Instrument defect:** the sweep labelled a lock-contention failure "did not build";
   re-run by hand, j06 compiles and is caught. Full gate **132 targets / 1 326
   passed / 0 failed**, predicted before the run. See [[rust-port-slice-ac-step5]].

6. **Step 6** — the oracle: `oracle/dump_slice_ac.py` (14 sections, **5 351 keys**) +
   `tests/slice_ac_oracle.rs` (5 gates). **`Rust ≡ PyPy` on all 5 351 keys, green on the FIRST
   run and with no port fix** — the first phase-7 oracle to find no defect, because steps 2/3 had
   already driven every reader and diffed the printed values. CPython arm exempt on **234 named
   keys, THREE causes (119 / 91 / 24)**. **Two findings, both about numbers already written
   down.** (i) § 5.27 (ii)'s shipped row was measured at **`every = 40`** while the fixture passes
   `every = 10`: `len(rows)` is **7**, not 2, and all five stride-dependent numbers reproduce at
   40 — the only column that agreed with the suite (`n_riding`) is computed BEFORE the stride, so
   it could not have disagreed. Step 2 had already published `n = 7`. (ii) **P8 is falsified from
   both ends**: the `cross_identity` subtree contributes ZERO names, and **119 of the 234 are the
   PLANT** — a sixteen-arm probe found the MARCH diverging on exactly three, always first in the
   stator state `v`, by 10–11 ULPs out of a solve whose inputs are bit-identical, decaying to
   bit-equality by the end of the ramp. P6 settled by a DECLARED EXTRA GRID, because the dump
   measured **0 of 38** intercepted `p` complex. Step 5's booked item discharged and corrected:
   **two**, not three, of `zeta_ring`'s quoted pairs are off the shipped grid.
   See [[rust-port-slice-ac-step6]].

7. **Step 7 — SLICE AC CLOSED** — `tests/slice_ac_dispatch.rs`, **9 gates**, five function pointers
   over five swaps, all green first run and **6 of 6 mutations of this file's own gates killed**.
   **Every reader rebuilds its machine through `at_lever`, so four of the five injections are
   LAUNDERED before any value is read** — the pre-flight's "seen by 1 of 6 readers" is a Python
   number that does not transfer, and `triple_laws` needs a DECLARED carrier (with its own control)
   to be observable. § (ii)'s break reproduced at the fixture's own stride, **7 rows -> 0**, both
   endpoints `assert_eq!`. And **four doc comments claimed an alias is louder than a `..` spread**:
   measured over all five hook structs, only `TripleHooks` (5 of 5 consts) is loud. Corrected and
   pinned by a tripwire that can fail. See [[rust-port-slice-ac-step7]].

## Slice AD (rung 72, `SharedActuatorTransient`) — pre-flight + step 1 of 6

**Pre-flight** (§ 5.28, twelve probes): the cell column measures **3** — the first back-half row
where the hand-written number is right. Four findings, all vacuity: **`shared_modes` does not
exist** (3 phantom `Usage:` methods across rungs 65/66/72, none inherited by the port); the
quartic's **three risky roots are dead** (`|a3|` wins `scale`'s max on 1068 of 1068 calls, over
**375 distinct coefficient vectors**); **`_authority`'s 1e-12 tolerance never does any work** (0 of
25 702 calls in the open interval); and **the floor's shipped needle discriminates nothing**
(`"FOUR actuator states"` is in rungs 72/73/74's messages, whose conditions are identical).
Six steps priced from sizing (1 177 source / 502 test lines). See
[[rust-port-slice-ad-preflight]].

1. **Step 1** — `src/shared_actuator.rs`, the three cells in `TripleHooks` (10 → 13) with their
   refusals, `ShareScope`, two carriers, five `R72*` tables, and `tests/slice_ad_cells.rs`
   (**12 gates**, green first run, **9 of 9 mutations killed**). **P1 was falsified by count and
   confirmed in mechanism**: it predicted 5 `E0063` sites and the landed edit needed **7**, because
   `cargo check` stops at the lib and never compiled the two test targets carrying the width
   tripwires. The wrong number AGREED with the prediction, which is why it nearly stood. See
   [[rust-port-slice-ad-step1]].

## Slice AB, step by step

1. **Step 1** — `src/reference_split.rs` 519 lines, 13 cell gates; nine swapped bodies panic.
2. **Step 2** — the nine bodies + six readers, 1 641 Rust lines from 713 Python (2.30×).
3. **Step 3** — the **25 ported gates** of `tests/rung69.rs` (851 lines, green first run); full gate
   **127 binaries / 1 241 passed / 0 failed**. A ten-injection sweep caught 9; the one hole, the
   root finder's Newton budget, was settleable only by step 4's oracle.
4. **Step 4** — the oracle: `dump_slice_ab.py` + `slice_ab_oracle.rs`, **15 957 keys per arm**,
   `Rust ≡ PyPy` on all of them after ONE fix (`py_half` — Python's `0.5 * z` is a complex product,
   not a scaling). CPython arm exempt on **194 named keys, two causes**. See
   [[rust-port-slice-ab-step4]].
5. **Step 5** — `tests/slice_ab_dispatch.rs`, **725 lines, 14 gates**: each of the ten cells swapped
   for its PARENT's function pointer (P2's letter), plus one declared counterfeit for `with_ref`.
   All ten observable, the count EMITTED. **Two of the pre-flight's four "breaks by PANIC" break by
   VALUE** — `triple_rig` and `manifold_v`, because rung 68's bodies for those two read no field at
   all. See [[rust-port-slice-ab-step5]].
