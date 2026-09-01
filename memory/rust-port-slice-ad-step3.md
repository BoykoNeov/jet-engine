---
name: rust-port-slice-ad-step3
description: "A cell census whose caller filter is written in call syntax cannot see a method that is only ever PASSED, and CPython's compensated sum() reaches complex too"
metadata:
  node_type: memory
  type: project
---

Slice AD step 3 (rung 72 Rust port) shipped the quartic chain (`_jac4`, `_charpoly4`,
`_quartic_roots_c`, `_parent_quartic`, `charpoly_selftest`), `_quad_laws`, `_quad_gains_at`,
`_riding4`, `_shared_march`, `_assert_fuel_boundary` and **all five public readers**, plus four new
`C64` operations and `py_max5`. Plan § 5.28.3.

**THE LESSON: a census predicate written in CALL SYNTAX is blind to a method that is only ever
PASSED.** The pre-flight measured rung 72's cell column at **3** and called it the first back-half
row where the hand-written number was right. It measures **4**. `_quad_gains_at` has two definers
(rungs 72 and 73), an identical signature and a behavioural override — but **zero `.name(` call
sites anywhere**, because it is handed to `_with_share` / `_with_ref` as a bound method at eleven
lines across six rungs. The probe's "a caller exists" filter therefore scored it 0 and dropped it
silently.

**AND THE PROBE HAD THE ANSWER HARD-CODED.** `probe_ad_a.py` opens with
`TARGETS = ['_reference', '_rk4_floor_shared', '_shared_rig']` — it VERIFIED a list the phase table
had already handed it. Its one sweeping half counts a name as read only at `self.NAME` / `cls.NAME`,
and **rung 72's readers dispatch on a local `m`, never on `self`**, so every one of that rung's own
dispatches was invisible to it: 54 names narrow, **61** with the attribute base widened. Run over
the whole ladder rather than the row that refuted it, seven names were invisible and
`_quad_gains_at` is the only one with **zero call syntax of any kind**.

**Booked to slice AE rather than installed**, and that is measured too: all five rung-72 readers are
redefined by nobody among the twelve descendants, and no rung-73+ code calls one — so the dispatch
is unreachable today and a `TripleHooks` field would be a pointer nothing could select differently
(see [[rust-port-slice-aa-steps2345]]'s *defence with no reader*). Same treatment `_legs` got.

**THE SECOND FINDING IS AN INTERPRETER FACT THE REPO HAS BEEN RECORDING ONE CLASS TOO NARROW.**
`charpoly_selftest` was ported FIRST — no march, no rig, pure arithmetic — and is **bit-exact vs
PyPy on run one**, settling P2 at the earliest point it could be settled. CPython 3.14 differs on
**4 of 10** keys, and substituting a naive fold into each `sum()` in turn attributes them exactly:
the inner matrix product is **INERT**, the float traces move two keys, and **`sum(roots)` — a
COMPLEX sum — moves a third**. Five shipped comments in this repo state the divergence as *`sum()`
for floats*; measured, **CPython 3.12+ compensates complex too**. Swept for consequences: every
other `sum()` over roots in the ladder is `sum(1 for z in …)`, an integer count, so the exposure is
exactly one site and it is in this slice.

**THE PORT IS BIT-EXACT vs PyPy ON 3 216 KEYS, FIRST RUN** — all five readers at their shipped
defaults, every gain row and all 32 ledger cells, 29 s against PyPy's minutes; § 2's four cells land
with the predicted zero counts 2/1/1/0 and `worst_v_gap` exactly 0.

**AND A DOC COMMENT I HAD JUST WRITTEN MEASURED FALSE, WHICH IS WHY THE DUMP EARNED ITS KEEP
TWICE.** `skipped.switch` is 0 on all six sampling arms: the reader's one genuinely NEW filter never
fires. My comment said dropping the `share_law == "max"` half of its guard *"would silently thin the
arm carrying § 3's discriminator"* — deleting that half moves **0 of 3 216 keys**. Replaced by the
MARGIN, which is the part worth keeping: the closest shipped point sits **4.08x** the bar, so
`switch_guard` ~16.4 rather than 4.0 would start dropping points — dead by 0.6 of a decade, not rung
69's 3.5. [[rust-port-slice-ac-step7]] recorded exactly this shape one slice earlier and I
reproduced it anyway.

**Two more, both about my own instruments:**
- The step-3 row of the plan enumerated **9 of 15** methods — the five readers and
  `_assert_fuel_boundary`, the 481 lines the pre-flight itself attributes to readers, appear in NO
  step. That is [[rust-port-slice-ac-step1]]'s lesson one slice on: **a step row is a list of names,
  and a list of names is not a partition until something adds it up.**
- The step-2 gate baseline died twice before it was taken: once to `cargo test 2>&1 > f` (redirect
  order sends stdout to the file, and the pipeline's exit status is not cargo's), once to
  `LNK1104: cannot open file …exe` — a linker lock from a run I launched over, **not** a test
  failure. Both caught by reading the file's CONTENT for `test result:` and `error[E`.

**How to apply:** when a census filters on "is it called", ask how the language can reach a function
WITHOUT calling it — a bound method, a decorator, a table entry, a `getattr`. And never let a probe
verify a list it was handed; make it derive the list, then diff the two.
