---
name: rust-port-slice-m
description: "Slice M (rungs 53/54, the variable stator and its throat) — the CPython arm's bar was asserted as bit-equality without ever being measured and a third of the dump refuted it; the two measuring passes that followed found defects in the INSTRUMENT, not the port; and Python's `slow` markers encode a COST that does not survive the port"
metadata: 
  node_type: memory
  type: project
  originSessionId: eb9f04f1-c4db-4dc2-8df1-507e1a32539c
  modified: 2026-08-15T02:56:44.957Z
---

Slice M shipped rungs 53 + 54: `stator.rs` (~1100 lines), `dump_slice_m.py` (10 950 + 840 keys),
`slice_m_oracle.rs`, `rung53.rs` (24 tests), `rung54.rs` (25 tests). Both PyPy arms bit-exact.
Six process lessons, and four of them are about instruments rather than physics.

**1. A BAR THAT WAS NEVER MEASURED IS A GUESS EVEN WHEN IT IS WRITTEN IN A DOC COMMENT.** The
CPython arm shipped saying "held to the SAME bit bar on values". Held to bits, **3290 of 10 950
keys differ** — a third of the dump, every one tolerance noise from a search that terminates on a
tolerance rather than on a fixed point. The doc comment had made the guess look like a decision.
**How to apply:** if a bar is not traceable to a measuring run, it is not a bar. Sixth instance of
[[rust-port-guessed-census-bars]] / [[rust-port-measure-before-registering]].

**2. THE FIRST TWO MEASURING PASSES FOUND DEFECTS IN THE INSTRUMENT, NOT THE PORT.** (a) `n` is
two different quantities in one dump: `scan/*/n` is a step COUNT, `margin|sthroat|sched/*/n` is
the shaft SPEED. A tail-only classifier collided them and held four continuous speeds to a bit
bar, reporting them as branch differences. (b) A converged residual has no relative scale —
6.3e-13 against -3.7e-13 is the same zero read twice, and relative deviation calls it 2.69.
**How to apply:** classify keys by PREFIX not by tail; and make the absolute gate the FIRST of
two, never a special case for an exact zero.

**3. A FINITE DIFFERENCE INHERITS THE DRIFT OF WHAT IT DIFFERENCES, SO ITS BAR IS THREE ORDERS
LOOSER BY ARITHMETIC.** The measured worst was `d_sm_n` at 2.38e-8 relative — the margin under it
drifts ~3e-11 and dividing by `DV = 5e-4` amplifies that 2000×. Confirms
[[golden-gate-slice5]] on a second code base.

**4. WHEN THE ZERO IS THE FINDING, THE BAND AROUND IT BELONGS IN THE RUNG'S OWN SUITE, NOT IN THE
INTERPRETER-COMPARISON ARM.** `d_m = -1.0e-9` and `flow_vs_speed = 1.5e-9` look like a 1.3%
relative failure and are actually rung 53's headline: the stator is thrust-neutral, so the flow
term is structurally zero. Widening the oracle's band to swallow them would have loosened the
CLAIM. They pass the oracle on the absolute gate and are asserted, as a claim, in `rung53.rs`.
Same shape as [[golden-gate-slice4]].

**5. A `slow` MARKER RECORDS A COST, AND THE COST DOES NOT SURVIVE THE PORT.** Python marks 13 of
the 43 gates slow; the ported 49 tests run in 0.82 s + 1.70 s. I first carried the markers over as
`#[ignore]` and that silently deselected 13 real gates to save ~2 s — exactly what the one-gate
policy forbids ([[test-suite-speed-policy]]). The port plan's own § 6 prescribed that mapping and
is now corrected: **port the gate, drop the marker, re-introduce `#[ignore]` only against a
measured cost.**

**6. A SHIPPED FALLIBILITY VERDICT EXPIRES WHEN A NEW CALLER ARRIVES.** `map.rs` recorded
`solve_n`'s bracket as "fires 0 times, so it stays an `assert!`" — measured over slice J's 810
cells, which contained no rung-54 `_scan`. Rung 54's scan walks until it fires: 100 of 100 probe
cells. Verdicts now carry the grid they were measured on and an expiry note (rung 61 / slice O
will do the same to `lp_eta_loop_bleed`). Extends [[rust-port-slice-l-step1]].

Also: three Python assertions are about Python, not physics, and each needed a DIFFERENT
substitute — object identity → bit equality; method inheritance → fn-pointer equality against
`R39` (comparing two `R53` entries would be a self-comparison that passes on any table); the
`lp_disabled` refusal → unrepresentable, because the parameter does not exist, which is strictly
stronger and so owes nothing. Two genuine debts booked: `phi_max` (phase 6) and the `StatorHooks`
dispatch (slice N, arity pinned by an exhaustive `match` so a second variant fails to compile).
The `#[should_panic]` splits inflate the test count above 43 — the coverage check is a
**name → parameter-set diff, never a count**.

One pre-registered census bar was off by one scan step: the plan recorded the sweep's break span
as the FAILING setting, while the code divides by the last SURVIVING one.

Full record: `docs/plans/todo-rust-port.md` § 5.9. Next: slices N (rung 55/56) and O (rung 61).
Related: [[rust-port-slice-l-step4]], [[rust-port-ported-test-vacuity]],
[[rust-port-oracle-cannot-see-a-missing-gate]], [[rung53-variable-stator]], [[rung54-stator-throat]].
