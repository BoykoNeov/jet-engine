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
* **PHASE 7** authorised 2026-08-20. Slices V, W, X, Y, Z, AA, AB, AC, **AD complete** (rung 72, § 5.28, twelve probes, all six steps). **Slice AE (rung 73) PRE-FLIGHT DONE 2026-09-01 (§ 5.29, ten probes); no step shipped yet.**

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

## Slice AD (rung 72, `SharedActuatorTransient`) — **CLOSED, all six steps**

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

2. **Step 2** — the six-state march, `PointExtra::Shared` (30 keys), the `Authority` label,
   nineteen widened reader sites, and `tests/slice_ad_march.rs` (**13 gates**, **10 of 10 mutations
   killed on the SECOND run**). **The crate's "the next variant breaks the build" convention holds
   at 7 of 20 sites** — 13 wildcards compiled silently, 3 of them a `false` inside a FILTER, which
   drops a rung-72 point and reports perfect tracking over an empty set. And **my own authority gate
   compared the recorded label against the function that produced it**, so an inverted comparison
   passed; the mutation sweep found it, not review. See [[rust-port-slice-ad-step2]].

3. **Step 3** — the quartic chain (`_jac4`, `_charpoly4`, `_quartic_roots_c`, `_parent_quartic`,
   `charpoly_selftest`), `_quad_laws`, `_quad_gains_at`, `_riding4`, `_shared_march`,
   `_assert_fuel_boundary` and **all five public readers**; `shared_actuator.rs` 813 -> **2 659
   lines**, four new `C64` operations and `py_max5`. No gate file — the ported gates are step 4, and
   slice AC's steps 2/3 set the precedent that a body step proves itself by DRIVING and diffing.
   **3 216 keys, `Rust == PyPy` bit-for-bit on the FIRST run**, at 29 s against PyPy's minutes; § 2's
   four cells land with the predicted zero counts 2/1/1/0 and `worst_v_gap` exactly 0. **THE CELL
   COLUMN MEASURES 4, NOT 3**: `_quad_gains_at` has two definers and an identical signature but
   **zero call sites of any kind** — it is PASSED as a bound method at eleven lines over six rungs,
   so the pre-flight's caller filter scored it 0 and dropped it. Booked to slice AE (measured
   unreachable today). Also: `charpoly_selftest` splits **4 of 10** keys on CPython 3.14, and one
   cause is a **COMPLEX** `sum()`, which five shipped comments in this repo call float-only; and a
   doc comment written in this same step measured FALSE — deleting the guard it defended moves 0 of
   3 216 keys. Closes with three disclosures a bit-exact dump cannot reach (`manifold=false` dead;
   `riding4`'s foreign-point arm made a REFUSAL, matching Python's bare index; and the deferred
   cell's MRO trap for rungs 74/80/81) and **P7, the CPython exemption PRE-REGISTERED** so step 5
   cannot produce it post-hoc. See [[rust-port-slice-ad-step3]].

4. **Step 4** — the 28 ported gates: `tests/rung72.rs` **949 lines, 28 gates**, green first run in
   8.63 s; the Python↔Rust map 1:1 IN ORDER, **0 added / 0 collapsed / 1 body substituted / 3 split
   by parameter**, reconciled by a machine-checked **BIJECTION** (0 unmapped, 0 extra, 0 collisions)
   after `grep` said 30 where `cargo` ran 28 — the two extras inside this file's own sentence
   *documenting that trap*, its third instance in the phase. Ten injections plus one declared
   control, **three binaries each**: **7 caught, 3 missed**, and every miss re-scored BY KEY against
   step 3's preserved 3 216-key dump rather than left as a word. **The headline is j05**: deleting
   the `|a3|` term that wins the root finder's start scale on 1 068 of 1 068 calls moves **26 keys —
   8 of `charpoly_selftest`'s own 10** — and **all 28 gates pass in Rust AND in Python**, so the
   hole is INHERITED; the two unmoved keys are the only two computed from a coefficient rather than
   a root, which turns the reading into a mechanism. j06 moves **0 of 3 216** (unobservable, margin
   3.5e−2 against a 0.1-wide interval, 165 of 165 points strictly interior) and j09 moves **3** that
   no gate in either language reads. j05 and j09 booked to step 5's oracle; P4's control `c11`
   missed as pre-registered, but on the GATE seat only, so P4 is **corroborated, not settled**.
   Full gate **137 targets / 1 393 passed / 0 failed / 0 ignored**, predicted before the run and
   held — and the number was nearly taken off a log that was still being written (125 blocks /
   1 335 passed, both plausible), so the bar is now structural: `Running` lines + `Doc-tests` must
   equal the result blocks. See [[rust-port-slice-ad-step4]].

5. **Step 5** - the oracle: `oracle/dump_slice_ad.py` (nine sections - A-F the five readers plus
   `charpoly_selftest` at their OWN defaults, G/H/J three declared extra grids) + two goldens at
   **54 116 keys each**, the phase's largest (1.5x slice Z's 35 335), and `tests/slice_ad_oracle.rs`
   (**6 gates**). **`Rust == PyPy` on all 54 116, green on the FIRST run, no port fix**; CPython arm
   exempt on **180 names, two causes (174 / 6)**. **THE FINDING: the two goldens differ on 5 022
   keys and the port drifts on 180**, because section H reads its 374 coefficient vectors as INPUTS
   - fed CPython's own coefficients the Rust reproduces CPython's roots bit-for-bit, so all 4 842
   of the solver section's differences are UPSTREAM, in the `sum()`-built polynomial. Read off the
   golden diff instead, the step ships a 5 022-name exemption blaming the root finder. **P7's two
   checkable clauses CONFIRMED exactly (4 keys, by name; 0 of the three non-drifters) and its
   headline FALSIFIED by 6** - march values at 2 points of 1 302, 1-4 ULPs, no `sum()` in reach:
   AC step 6's plant drift, which P7's own clause (ii) predicts and its headline forbids. All three
   bookings discharged by key: **j05 CAUGHT at 2 937** (A/D/E share = step 4's 26 exactly, the two
   unmoved keys the same two BY NAME, and `n_complex` - a DISCRETE key - flips on 163 of 374),
   **j09 CAUGHT at 3**, **j06 still 0** in a 17x larger space, **c11 0 => P4 SETTLED** with the
   margin emitted (`n_open` 0 over 12 676 calls; smallest non-zero gap 2.74e-07 = **273 641x** the
   tolerance). P3 re-measured on this grid - **417 calls / 374 vectors / 69 near-double**, not the
   suite-wide 1 068 / 375 / 167 - with a tripwire against re-transcribing the old pair. And my own
   sweep's per-section histogram **summed to exactly 400, its own print cap**; section H's true
   share is 2 911. **Close-out re-measured every testable sentence in the two new headers: the
   stride backstop's "a hidden-point defect still has to move a key" is FALSE (index 137 moves 0
   of 54 116, control at 135 moves 10), the three-way solver claim had been two PAIRWISE runs on
   disjoint inputs (cross-feed then run: PyPy on CPython's 374 vectors, 0 of 4 488 root keys), and
   "1 294 emitted points" is 1 302 in six places.** Two unreported positives: section G was green
   first run though its 21 reduce-spine-invisible fields had never been diffed, and the plant
   carries no state between reader calls (full-trajectory equality on 4 repeated signatures).
   See [[rust-port-slice-ad-step5]].

6. **Step 6 - the 3 dispatch gates, and THE SLICE CLOSES.** `tests/slice_ad_dispatch.rs`, **three
   injections across TEN tests**, no source file touched, all ten green first run.
   **THE FINDING: a FIRST DEFINER still has a PARENT POINTER, because the parent slot carries a
   REFUSAL.** The pre-flight reasoned from "rung 72 defines all three cells first" to "there is no
   parent function to install", and concluded the gates had to be hand-written sentinels (AB's
   declared exception). Measured: `R71_TRIPLE` holds a pointer in all three slots - the shared
   refusal, the **same address** in `NO_TRIPLE`, `R68_TRIPLE` and `R71_TRIPLE`. So all three gates
   are plain parent-pointer injections, i.e. **AB's RULE where the pre-flight cited AB's
   EXCEPTION**; a counterfeit's observability is a property of the body I wrote, a shipped
   constant's is not. **The SEAT MATRIX run whole is 3 cells x 6 seats = 18 readings, 7 panics and
   11 silences, where § 5.28 (vii)'s table names 3** - and the eleven silences are TWO mechanisms
   that read identically (ten laundered by the rig's `at_lever` rebuild, the eleventh a path that
   never calls the cell), separable only by the OTHER seat. `shared_rig` scored as a **census over
   all five readers**, not one. **And my own new header typed NINE tests above the TEN that
   disprove it** - the same nine pre-registered as the run's count - caught by the runner, not by
   re-reading; the count is now **read off the file's own source** and pinned. Re-measuring the
   header's other countable claims found one more wrong (AC's laundering gate asserts a row count
   as well as the identity). **P6 settled clause by clause (i CONFIRMED / ii FALSIFIED / iii
   carried to AE); P5 CLOSED UNSETTLED** with reason and destination - its subjects are branches
   inside one body, which no pointer-level instrument reaches; **the six-step count CONFIRMED**,
   and AC's own seven-step precedent noted as having held and never been marked. Full gate
   **139 blocks (138 Running + 1 Doc-tests) / 1 409 passed / 0 failed / 0 ignored**, every row
   predicted before the run and held, plus a re-run of the target on the final tree after two
   post-launch comment-level edits. See [[rust-port-slice-ad-step6]].

## Slice AE (rung 73, `AppliedReferenceTransient`) — PRE-FLIGHT ONLY

§ 5.29, ten probes. **684 source / 518 test lines, 27 collected (13 slow), 12 methods** — AB's
shape (706/582), so **FIVE steps predicted**, with the step table emitted and checked to be a
PARTITION (12 of 12 placed, 0 missing / extra / duplicated).

**The cell column says 0 and measures 1 ADD + 6 SWAPS.** `_with_ref` (69→73) is a **NAME REUSED**,
not an override — identical arity, disjoint fields (`_ref` / `_ref_law`), and rung 69's inherited
`reference_bill` **raises** on a rung-73 machine (with a passing control). **§ 5.27 (x) saw the
pair and cleared it as a harmless RENAME**; the same sentence clears `_with_coord` (74→79), now
booked to **AF (install) and AI (observe)**, its behavioural verdict left UNDRIVEN because my own
probe was measured blind. The shipped port is NOT broken — it already dispatches the setter
through the cell for this exact reason — so what AE owes is the **REFUSAL**, not the cell.

**AD's two carried items both settled.** P6 (iii) **CONFIRMED**: `_reference`'s three paths are
all live over 260 190 intercepted calls — 41 346 / 109 537 / **109 307 (42.01 %) on the value
break**, none of which return `req` bitwise; absolute gap median 8.7e−03, and 6 380 calls have
`req == 0.0` exactly, where a relative spread is undefined (my first headline was a `1e-300`
guard artifact and is recorded as such). `_quad_gains_at`'s **"unreachable today" booking REFUTED
BY VALUE**: machine held fixed, pointer swapped — **32 keys move and 70 vanish**, `F_r` going
−1.000000000002735 → 0.0.

Eight predictions P1–P8, including the CPython exemption pre-registered as a named set with a
falsifier — and **P1/P7 were jointly impossible as first written** (one claimed a new hook
field, the other that re-aiming the existing one is the defect, but that slot ships since AB).
Settled by measurement: **0 reads of  on a rung-73 machine** against a liveness control
at 1, so the re-aim is RIGHT,  stays at **13**, and the shippable defect is the
**missing refusal**. P7 stands FALSIFIED before step 1 rather than quietly rewritten. See [[rust-port-slice-ae-preflight]].

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
