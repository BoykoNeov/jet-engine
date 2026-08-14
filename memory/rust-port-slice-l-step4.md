---
name: rust-port-slice-l-step4
description: Slice L step 4 (the oracle + both rung suites) — a claim written into the SHIPPED source was false and the gate built from it passed the defect; and a pre-registered bar re-measured on the wrong construction looked like a refutation
metadata: 
  node_type: memory
  type: project
  originSessionId: 91e0aea1-3b04-4389-ade5-c7d45b3d297c
  modified: 2026-08-14T17:01:20.114Z
---

Slice L is COMPLETE: `dump_slice_l.py` (25 458 keys), `slice_l_oracle.rs`, `rung41.rs`,
`rung42.rs`. 479 Rust tests green, PyPy arm 25 458/25 458 bit-identical, all nine predictions
settled. Four process lessons:

**1. A CLAIM IN THE SHIPPED SOURCE CAN BE FALSE, AND THE GATE BUILT FROM IT WILL PASS THE
DEFECT.** `two_spool.rs` asserted the golden section's check-first shape was load-bearing — "a
`do`-while makes the refinement count 34 instead of 33" — so I wrote P5's gate on that claim.
Injected, the `do`-while makes **33**: same result, no bit moved, gate green. The bracket is
always 20 wide on entry, so the stopping rule can't be met before the first pass, which is the
only thing separating the shapes. The claim had been carried across from rung 39's efficiency
loops, where a flat map DOES meet the residual on entry. **How to apply:** a comment in the code
you are porting is a hypothesis, not a fact — inject the defect it names before writing a gate
that cites it. Same family as [[rust-port-documented-gate-that-doesnt-exist]].

**2. RE-MEASURE A PRE-REGISTERED BAR ON THE CONSTRUCTION IT WAS PROBED ON, OR A BOOKKEEPING SLIP
READS AS A REFUTATION.** My first census swept rung 42's grid on FLAT maps and got 68/68/68/67
matched with the UNCHOKED column flat at 23 — i.e. *the valve does not shrink the choked
envelope*, the rung's own gate 6 failing. The probe had used the shaped `mixed` pair; re-swept
there it reproduces the registered table to the cell (67/67/66/65, UNCHOKED 23/23/24/25). The flat
numbers were perfectly correct and answered a different question, which is what makes this
survivable-looking. **How to apply:** when a measured census disagrees with a registered one,
diff the CONSTRUCTION before believing the physics. See [[rust-port-guessed-census-bars]].

**3. A GATE WITH N LEGS ONLY CALIBRATES ITS FIRST.** The dispatch gate (three schedule methods
reaching rung 42's body through the hook) was one test with three legs. Pointing the hook at rung
39's function fires leg 1 and legs 2–3 never run — so "all three witness the dispatch" was
uncalibrated. Split into three tests, each fails alone. The wider finding: with rung 42's physics
replaced wholesale, **14 of the suite's 17 tests still pass**, because every value gate reaches
the bleed body directly and only rung 41's methods use the hook. **How to apply:** one test per
claimed witness, and always ask which tests a deliberate defect leaves green. Extends
[[rust-port-slice-l-step3]]'s "1 of 3 methods" lesson.

**4. MEASURE THE DETECTOR — AND HERE THE VALUE BAR WAS BLIND.** Flipping one `(1-b)` association
moved **254 of 25 458 keys (1.00 %)**, but only ONE exceeded the 1e-8 value bar, and it was an
`n_pass` (12 passes vs 11), not a value: the worst value deviation was 2.05e-9. So the PyPy arm's
BIT-equality is the detector, not belt-and-braces — toleranced it would catch 1 key instead of
254. The worst cell sat at the sweep edge (`M0=1.60`, TPG) exactly as step 3 predicted.

**5. A BAR COPIED FROM THE PREVIOUS SLICE IS NOT A MEASURED BAR, AND THE SECOND ARM IS WHAT SAYS
SO.** The 1e-8 value bar came over wholesale from slice K, where it holds. The CPython arm failed
on 34 of 23 772 value keys — and the 34 were two disjoint populations with nothing between them,
so the fix was two measured classes, not one widened number. (A) **A flat extremum's LOCATION is
10⁴–10⁶ times less determined than its VALUE**: out of one golden section, `phi_star` agrees to
4.07e-11 and `Tt4_star` to 7.39e-6. Zero slope at the maximum means objective noise — the inner
matcher's convergence, not machine epsilon — buys a first-order move in the abscissa and none in
the ordinate; the 1e-5 K stopping rule is finer than the objective's own noise floor, so the last
refinements resolve nothing. This **INVERTS** [[rust-port-shape-keys]] and bounds it to a GRIDDED
argmax. (B) **A pass-count flip costs exactly one decade** (1.55e-8 vs 1.03e-9 in cells that did
not flip) — both populations measured, both asserted, because the SEPARATION is the content.
**How to apply:** when a second arm fails, first ask whether the failing keys form populations; a
split that makes the accounting close exactly is a finding, a widened single bar is a shrug. And
never inherit a tolerance across slices — [[rust-port-measure-before-registering]].

Also: the plan said "the 9 + 10 Python gates"; the files have **12 + 12** functions under **8 + 8**
headings. Enumerated first, rosters shipped as asserted arrays. 10 of rung 41's port (2 defer to
phase 6, 1 splits); all 12 of rung 42's.

Full record: `docs/plans/todo-rust-port.md` § 5.8.4 (a)–(h). Next: slice M (rungs 53–56, 61 — the
airflow levers), which is where `is_flat` and `vsv` finally land. Related:
[[rust-port-slice-l-step1]], [[rust-port-slice-k]], [[rust-port-oracle-cannot-see-a-missing-gate]].
