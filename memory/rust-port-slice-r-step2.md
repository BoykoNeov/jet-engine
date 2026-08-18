---
name: rust-port-slice-r-step2
description: "Slice R step 2 (rung 40's 9 gates) — the gate that anchors the object was blind at the throttle it starts on, and a registered SUM is not a gated SPLIT"
metadata: 
  node_type: memory
  type: project
  originSessionId: 90ca045f-9f35-4f45-8ace-04b125ccc0a7
  modified: 2026-08-18T12:49:17.714Z
---

Phase 6 slice R step 2 (`tests/test_rung40.py` → `rust/tests/rung40.rs`, 591 lines) shipped
2026-08-18: **9 collected items / 8 test functions, 9 run / 0 failed in 3.0 s**, no `src/` edit —
every field the port needed was already `pub`, so the step stayed test-only as its step table
assumed. Steps 3 (rung 44's 8 gates + `rung41.rs`'s roster discharge) and 4 (the oracle) remain.

**WHICH ASSERTION IS THE DISCRIMINATOR DEPENDS ON WHERE THE SWEEP STARTS.** Gate 3 is the
non-tautological one — a bare-math reference reproducing the solver. Injecting a defect into that
reference (deleting the map's linear loading slope from its loading law) IS caught, but by the
`sigma_crit` bar and not by the four speed / pressure-ratio bars four lines above it. At the FIRST
throttle in the sweep — the design point — the flow coefficient is 1 by construction, so the
injected term vanishes identically and moves the speeds by **4.4e-15** against a `1e-8` bar. One
throttle down it moves them by **2.9e-2**. The bars are live; they are blind at the only point the
sweep reaches before the gate fails. **Before believing a two-path gate covers what its docstring
says, check whether its FIRST cell can move the thing under test** — cell ORDER decides which rule
is actually discriminated, which is [[rust-port-slice-j]] one level up.

**A REGISTERED SUM IS NOT A GATED SPLIT.** The plan pre-registered "245 real / 7 complex" for the
two eigenvalue branches "on gate 5's grid". That is the two gases ADDED. Measured per gas it is
**124/2 and 121/5**, 126 each — and only then does the pair reconcile. The bar ships as four
constants plus an assertion that their total equals the grid size `7×3×6`, so a count is read
against a grid whose own size is asserted. Both halves were re-measured on the PYTHON side (which
has no counter — re-evaluate the shipped discriminant) and agree cell for cell. Related:
[[rust-port-guessed-census-bars]], [[rust-port-slice-n-step4]].

**A DEFAULT ARGUMENT IS A VALUE THE SOURCE STATES ONCE, FAR FROM THE CALL.** Two silent ones here,
and one is non-uniform *inside a single gate*: `lead_threshold`'s `d` is passed `25.0` in two parts
of gate 4 and left at its `5.0` default in the third; `slip_excursion`'s ramp rate is defaulted at
gate 7's call site while its two siblings are named. Rust has no defaults, so both had to be read
off the `def` line — carrying the wrong one changes the physics without failing loudly.

**AND MEASURE THE COUNT AFTER THE LAST EDIT.** This file's own line count was written as 591
un-measured, corrected to the measured 579, then pushed back to 591 by the two doc paragraphs
the writeup describes — so the guess is right by coincidence and the measurement is stale. A
count taken before the final edit describes a file that no longer exists.

Prediction 10's second half (every two-shaft accessor on the degenerate variant panics) had **no
Python gate at all** and was written into gate 2 with `catch_unwind`, plus the mirror, so it is a
discriminator rather than a blanket panic — inside gate 2 deliberately, since a tenth function
would move the collected count [[rust-port-slice-r-step1]] had just finished correcting.
Detail: `docs/plans/todo-rust-port.md` § 5.15 step 2.
