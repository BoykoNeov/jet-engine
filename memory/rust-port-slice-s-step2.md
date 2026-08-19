---
name: rust-port-slice-s-step2
description: "Slice S step 2 (rung 43's 11 gates) — a NON-STRICT ordering assertion is satisfied by the variable going inert, and only ONE of eleven gates asserts a number"
metadata: 
  node_type: memory
  type: project
  originSessionId: 202a37ef-75c9-4f71-a577-0906d599f34b
  modified: 2026-08-19T05:51:49.998Z
---

Slice S step 2 of 5 shipped 2026-08-19: `rust/tests/rung43.rs`, 11 `#[test]`, 726 total names
(715 → 726, 0 removals), plus one additive `src` method
(`TwoSpoolFuelTransient::equilibrium_fuel_lp_disabled`, which step 1 had booked to step 3 — gate 2
needed it a step early).

**The lesson: a NON-STRICT ordering assertion is satisfied by the quantity going INERT.** Rung
43's gate 5 closes with `ratios == sorted(ratios, reverse=True)` on a share that is supposed to
TRADE with the mass-ratio knob. Delete the knob from the plant and the three ratios come back
bit-identical — and that assertion passes. Measured, not reasoned. The Rust gate now asserts the
strict `>` beside it, with the measured 28 % tightest adjacent drop written in place as the licence.

**Why:** the assertion whose whole subject is that something MOVES is exactly the one that must not
be spelled `>=`. This generalises past this gate: any ported ordering claim on a *response* wants
the strict form, and the way to find out is to make the driver inert and re-run.

**How to apply:** when porting a sign/ordering gate, ask what a CONSTANT would do to it. If constant
passes, either strengthen and record the margin, or record that the gate cannot see inertness.

**Three more from the same step, each measured:**

- **A suite of SIGN claims needs one gate that asserts a NUMBER.** Feeding rung 43's LP shaft the
  HP power residual — a gross defect — fires **1 of 11** gates: the dynamical reduce, which demands
  the march land on an independently solved equilibrium. Every finding gate (sign, ordering,
  monotonicity) is blind, because a wrong-but-similar derivative still satisfies them.
- **A line number stops being a target once the file underneath it moved.** Three injections were
  `sed '<n>s|…|…|'` written before a +21-line `src` addition; the re-run landed on an unrelated
  statement, changed nothing, and reported all-green — which reads as "the new gate does not fire".
  Caught by echoing the patched line, not by the result. Every injection after it is a text
  substitution asserting `count(old) == 1`. Second instance of [[rust-port-slice-r-step3]] inside
  one slice.
- **An axis can be deleted by a SIGNATURE, in an earlier slice, unrecorded.** Python builds the
  degenerate object with BOTH maps and its constructor picks one; every `lp_disabled` constructor
  in the port takes only that one, so "picked the wrong map" is unrepresentable — in `rung40.rs` as
  much as here. What survives was measured rather than assumed: the caller's choice is live, but
  fails by 1 ULP at the design point. [[rust-port-ported-test-vacuity]]'s trap, arriving through the
  Rust signature instead of through a factorisation.

The advisor caught the last two of those and the gate-3 clone note (Python shares ONE design object
so a mutation would surface; a clone severs that channel — say "thinner than Python", not "pure
function of its arguments"). See [[rust-port-slice-s-step1]] and § 5.16 of
`docs/plans/todo-rust-port.md`. Steps 3–5 remain: `rung45.rs`, the oracle, the docs.
