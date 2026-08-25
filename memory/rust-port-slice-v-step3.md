---
name: rust-port-slice-v-step3
description: "Slice V step 3 (rungs 57-60 suites ported, 59/59) — an injection whose only trace is OBJECT STATE is indistinguishable from one that never applied, so a did-it-move probe must carry witness keys no gate reads"
metadata: 
  node_type: memory
  type: project
  originSessionId: 6c80232e-6503-449e-bb22-bbd4cc194a99
  modified: 2026-08-25T18:11:04.792Z
---

Slice V step 3 ported `test_rung{57,58,59,60}.py` to Rust one-to-one — **59 `#[test]` against
Python's 59 collected** (57 `def test_` + 2 from rung 57's two-way `parametrize`s), 0 ignored,
**5.4 s** against Python's 42.4 s. Both counts EMITTED, never typed.

**THE STEP'S RESULT IS THAT THE SLICE'S OWN CARRIER BUG IS INVISIBLE TO ALL 59 GATES, AND THE
FIRST RUN THAT SAID SO WAS VOID.** The probe was built to dump the readings the gates compare
(302 keys). Against the local-armed-core injection — the natural Rust shape § 5.20 (ii) measured
Python moving 15.4 % — it reported **`moved 0/302, caught 0/59`**, and so did an injection that
deleted `arm`'s whole HP branch. **`moved 0` is not a result.** An injection whose only observable
is the OBJECT'S OWN STATE looks exactly like an injection that never compiled in, and nothing in
the harness could separate them. Repaired by adding a **witness section counted separately** — the
live map's `vsv` after a march, `arm`'s dispatch counters on an HP-scheduled machine no suite
builds, and the steady `surge_margin` — after which both injections proved live.

**Why:** [[rust-port-slice-s-step3]]'s lesson was about injections that MISSED; this is the same
zero produced by a probe that could not SEE. Same reading, opposite cause, and the fix is
structural rather than a re-aim.

**How to apply:** a did-it-move probe must read at least one quantity **no gate reads**, chosen
because the injection's mechanism can reach it. Count witness keys separately from gate-visible
ones — a witness moving proves the injection is live and says nothing about whether any gate can
see it.

**AND THE PAYOFF WAS A CROSS-LANGUAGE REPRODUCTION TO THREE FIGURES.** With the witnesses in, the
carrier injection moves the steady LP surge margin **4.631 %** where Python measured **4.632 %**
(and 2.356 vs 2.357, 4.742 vs 4.743), with the scoped values agreeing to ~7 digits — **caught by
0 of 59**. The channel is named rather than inferred: `surge_margin` sits on the map core and runs
a STEADY match, so it never passes through rung 57's `try_close` and nothing re-arms. That makes
step 5's carrier gate *measured*-necessary. **One half did NOT reproduce and is booked as a BOUND,
not an immunity**: Python's `margin_min_lp` moved 15.4 %, the Rust reading did not, because the
Rust call re-marches from `equilibrium` and every close inside it re-arms — a difference in CALL
ORDER. Step 5 owes a gate that reaches that reader in an order preserving the staleness.

Four more from the same step:

- **A SHIPPED DOC COMMENT TRUE OF THE SIGNATURES AND FALSE OF THREE OF THE FOUR SUITES.**
  `Ramp::fine`'s comment (written at step 2) calls `ds = 0.005` *"rungs 58/59/60's default"* —
  correct about the reader METHODS, and **`test_rung58.py` and `test_rung59.py` declare
  `DS = 0.01` and pass it explicitly at every call site**. Porting either through `Ramp::fine`
  because its comment named the rung would have halved the step and moved every number, and the
  gates are relational so none would have said so. FIXED in the source, not footnoted — slice W
  inherits it. [[rust-port-slice-l-step4]] on a comment that was true of what it described and
  false of what a reader would use it for.
- **TABULATE BAR MARGINS, and expect most of them to be loose ON PURPOSE.** 63 inequalities:
  seven within 10 % of their bar, twelve passing at over 5× (up to **127×**). The loose ones are
  not bugs — their Python docstrings say *"gate the claim, not the boundary value"* — but the
  **tightest non-physics bar is a TOLERANCE bar**: rung 59's `d_abscissa < 1e-12` reads 7.76e-13,
  **22 % of headroom**, one solver change from flipping. Name it, do not loosen it.
- **PORT A NON-STRICT ORDERING AS WRITTEN, THEN MEASURE ITS STRICTNESS.** Python's
  `xs == sorted(xs, reverse=True)` is a `>=` an inert sequence satisfies. Both ported ones are in
  fact strict, but `self_cancel`'s smallest adjacent gap is **1.6 % of its range** — its ordering
  assertion is doing almost no work at the slow end. Don't silently tighten a ported contract;
  measure what it actually holds by.
- **A TYPE THAT DELETES A REFUSAL IS NOT THE SAME GATE, AND THE TWO CASES DECIDE DIFFERENTLY.**
  Two of rung 60's sixteen poke a refusal Rust makes unrepresentable (a `Floor` where Python takes
  any leg; a `LadderAxis` where Python takes two exclusive keyword lists). Re-gated as an
  **exhaustive `match`** that stops compiling on a third variant, plus a runtime half asserting
  what the refusal protects. Decided the OPPOSITE way from rung 57's `Shape`, where the bad value
  is a STRING a caller could supply, so the port kept a `try_from_str`. Rule: **a bad VALUE keeps
  a runtime entry point; a bad TYPE becomes an exhaustiveness check** — and both decisions get
  written at their own site.

**AND THE INSTRUMENTS ARE IN THE REPO, NOT IN SCRATCH.** The probe, its harness and both
tables are committed (`rust/oracle/slice_v_probe.rs.keep`, `rust/oracle/inject_slice_v.py`,
`docs/plans/slice-v-step3-evidence.md`) because step 5 is a different session and cites
them. An instrument a later step depends on, left outside git, is the same non-durability
the phase's own step-4 checklist item (b) was written to stop — and it recurred here on my
own tooling one step after being written down.

Related: [[rust-port-slice-v-step2]] (both closing gates unfalsifiable), [[rust-port-slice-t-step2]]
(9/9 green and blind to 24 %), [[rust-port-ported-test-vacuity]],
[[rust-port-oracle-cannot-see-a-missing-gate]].
