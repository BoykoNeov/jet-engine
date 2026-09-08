---
name: rust-port-slice-ah-step2
description: "Slice AH step 2 — a doc comment's ONLY is a claim about a SET while the census behind it reported a TOTAL, and step 1 shipped the very defect that sentence said could not exist"
metadata: 
  node_type: memory
  type: project
  originSessionId: 645bda70-c0ca-4809-a9ff-362152fd53ed
  modified: 2026-09-08T07:51:19.101Z
---

Slice AH (rungs 77/78) step 2 SHIPPED: rung 77's reader layer — `_slope_at`, `_residuals`,
`_legs`, `_ledger_march`, § 1 `leg_slopes`, § 2 `set_point_gains` (six of nine methods).
`rust/src/stiffness_ledger.rs` 166 → **846**, module total **1 370** (P1's band is 2 800–3 150);
`rust/tests/slice_ah_laws.rs`, **7 gates green on the first run**; **zero new fields**,
so P6's 0 ADD holds a second step. **SEVEN of the Python suite's own bars are reproduced with NO golden read anywhere** — four
tolerances (`c_err < 3e-9`, `sep > 1e-2`, `ift_err < 3e-8`, `n >= 5`) and three magnitudes,
of which the accel column's stiffness against rung 76 § 3's `1.22799 … 1.24573` is the only
**ABSOLUTE** bar available before the oracle — every other gate here survives a uniform scale
error, and it passed first time. Plan § 5.32.2.

**THE LESSON — a doc comment's "this is the only one" is a claim about a SET, and the census
behind it reported a TOTAL.** `sensed_cap::c_at` says its `max(w, 1e-9)` is *"the single
expression-first `1e-9` fold in the whole package"* — the spelling matters because Python's `max`
keeps argument 0 on a NaN and `f64::max` does not. Rung 77's `_slope_at` is another one, so the
census was re-run: it **reproduces slice AG step 1's 103-literal-first-of-268 exactly** and finds
**FOUR** expression-first `1e-9` folds, not one (`engine.py:19308`, `:19708`, `:20221`, `:21029`
— the last booked to slice AI). Every one of those lines was already in `engine.py` when the
census ran, so the claim was **FALSE AT BIRTH** and not decayed — [[rust-port-slice-ag-step5]]'s
class. The measurement the sentence rested on counted *how many folds put a literal first*; the
sentence asserted *which folds are expression-first with this literal*. **A total cannot answer a
set question, and nothing in the census output looks wrong when it is asked to.**

**AND IT WAS NOT INERT: step 1 of this same slice shipped the defect.**
`residual_gauge::gauge_root` spelled `:20221` as `w.abs().max(1e-9)` — the form the sentence one
module over says is never needed. Repaired to the explicit fold. Whether a NaN `w` can reach that
line is recorded as **unmeasured**, not asserted unreachable.

**A GATE PROVED LOAD-BEARING BY BUILDING ITS DEFECT — the demonstration step 1 could NOT do for
its own.** Rung 77's Python records that § 2 once built the residual closures inside a
`_b_state = q` block and evaluated them after the `finally`, so both `q ± dq` readings landed on
the same closed-valve plant. A Rust closure over `&FuelTransientCore` inherits that exactly (the
frozen-state carriers are `Cell`s on the core it borrows). The gate file constructs it — one `}`
earlier than the shipped body — and measures `G_q` **exactly 0.0** and `err` **exactly 1.0**
against the shipped body's non-zero and `< 3e-8` at the same point. So `ift_err < 3e-8` is not a
bar with no demonstrated way to fail. Step 1's identity gate could not do this because the builder
it needed is `pub(crate)`; that is a property of THAT defect, not of the method.

**AND A PROMISE NAMED A STEP THAT COULD NOT KEEP IT.** Two sentences (plan § 5.32.1 and
`residual_gauge.rs`'s header) said the owed RAII panic message *"lands at step 2"* — but `_phi_at`
is a rung-78 leaf and step 2 is rung 77's layer. Re-aimed at the BODY rather than a step number,
which is the only form that cannot decay; a fourth [[rust-port-documented-gate-that-doesnt-exist]]
authored one step after logging the third.

**AND STEP 1 LEFT `tests/test_rust_line_citations.py` RED.** Running it at this step: **three of
its five gates failing, seventeen citations never blessed, and TEN of them step 1's** — the two
refusals and six nest sites `residual_gauge.rs` cites, plus `stiffness_ledger.rs`'s two. Step 1
ran `cargo test` and not `pytest`, and the guard lives in `pytest`. Re-blessed to 33 files / 181
sites / 112 lines after checking all seventeen by hand — which incidentally re-verified § (a)'s
census through a second instrument that had no idea what it was confirming. **Plan § 5.32 (iii)
had pointed at the guard's blind spot (it does not scan `docs/`) while the directory it DOES
watch was the one in arrears.**

**How to apply:** when a comment says *only / single / the one*, ask what the measurement behind it
COUNTED — a total, a maximum and a set are three different questions and only one of them answers
"only". And prefer proving a gate load-bearing by BUILDING its defect over asserting that it would
catch one; when that is impossible, say so in the file (step 1) rather than leaving the absence.
And **a Rust-only step still owes the PYTHON gate**: the port’s own citation guard is a `pytest`
test, so `cargo test` green says nothing about it.

See [[rust-port-status]], [[rust-port-slice-ah-step1]], [[rust-port-slice-ag-step5]].
