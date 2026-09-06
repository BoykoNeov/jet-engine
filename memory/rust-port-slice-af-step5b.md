---
name: rust-port-slice-af-step5b
description: "Rust port slice AF step 5(b) — a pre-registered CPython exemption falsified in both clauses, because a sum() census counted call sites and half of them sum a literal 1"
metadata: 
  node_type: memory
  type: project
  originSessionId: 8d42ecda-e715-4ce4-b3ba-8f5955f7ae10
  modified: 2026-09-06T05:24:57.154Z
---

Slice AF step 5(b) (rung 74, § 5.30.5 (b)): `rust/oracle/dump_slice_af.py` (sections A–H + Z) and
two goldens at **20 643 keys each**; `rust/tests/slice_af_oracle.rs`, **4 gates, green on the first
run that ever compiled, no port fix**. `Rust == PyPy` on all 20 643; `Rust == CPython` on all but a
measured 49. Success line states **0 keys read as declared inputs** — this slice replays no
captured argument, so the term AE step 4 had been miscounting is named as zero rather than omitted.

**THE LESSON: a `sum()` census is not an ATTRIBUTION until the summand's TYPE and CONDITIONING are
in it.** § 5.30 (iii) located rung 74's four `sum()` calls by line and called that *"attributed
rather than counted"*. **P2 then named `forcing_openloop` in advance** — the reader owning two of
the four, the largest share — and was **falsified in both clauses**:

* the three pre-registered keys are **bit-identical** (asserted equal, not merely absent from the
  diff — *not in the diff* is also what a key the dumper forgot to emit looks like);
* the 49 that differ are all **`demand_gains`'s** `poly_gap` / `poly_scale`;
* and **none of rung 74's four `sum()` calls is in the causal path.** Two of them are
  `sum(1 for ...)` over a generator of ones (`engine.py:18303`, `:18459`) — INTEGER counts no
  compensated summation can move, so they were never candidates. The two real float folds are
  `forcing_openloop`'s and they drift on nothing. The origin is **`_charpoly4`'s float `sum()` —
  rung 72's, INHERITED** — which AD step 5 and AE step 4 each measured independently, and which a
  census of rung-74 bodies could not see.

**AND THE EXEMPTION'S OWN NUMBERS SETTLE IT, ON THE SAME ROWS.** The 49 keys are two readings of
one coefficient vector: `poly_scale` (`max|x|`) drifts **1.19e-16 … 4.78e-16**, one to four ULPs;
`poly_gap` (`max|x−y|`, a difference of two nearly-equal polynomials) drifts **4.5e-07 … 2.48e-04**.
Row 26: `1.196e-16` against `2.478e-04` — **the same perturbation amplified 2.1e12 by the
subtraction.** This is [[golden-fingerprint-gate]]'s *drift follows CONDITIONING* in its sharpest
form: every earlier instance compared two different quantities, this one holds the input fixed and
reads it through two functions in the same row.

Shape notes worth keeping: **section G (the plant) is 16 572 of the 20 643 keys** — A–F are folds
and the suite's reduce spine compares 9 of the march's 35 fields, so the aggregate sections cannot
see a per-point drift. `D/cell1/why` is the ONLY key witnessing a refusal's text and it agrees;
`Z/n_none = 8` (all `first_gov` on arrested arms) and `Z/n_pos_zero = 1 710` are computed by the
PORT's own emitters and compared, never read back out of the golden.

See [[rust-port-slice-af-step5a]], [[rust-port-slice-ad-step5]], [[rust-port-slice-ae-step4]],
[[rust-port-status]].
