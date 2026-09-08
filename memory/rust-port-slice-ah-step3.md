---
name: rust-port-slice-ah-step3
description: "Slice AH step 3 — a doc comment's universally-quantified sentence is a claim about code that does not exist yet, and the property it forbids is the next section's only instrument"
metadata: 
  node_type: memory
  type: project
  originSessionId: f3c55711-1d71-43f4-9e46-664d7977acbe
  modified: 2026-09-08T10:40:28.486Z
---

Rung 77's last two readers (`singular_limit` § 3, `stiffness_ledger` § 4) ported;
`stiffness_ledger.rs` 846 → 1 208, `tests/slice_ah_ledger.rs` 428 lines / 9 gates, 7 of 9 green
on the first run. Rung 77 closed at nine of nine methods. Plan § 5.32.3.

**THE LESSON: a universally-quantified sentence in a doc comment is a claim about code that has
not been written yet — and the property it forbids can turn out to be the next section's only
instrument.** `_residuals`' Python docstring closes *"so a residual can only ever be evaluated on
the plant it was built for."* `singular_limit`, 195 lines further down the SAME class, builds its
residuals inside the frozen block, lets the block close, re-freezes only the stator, and
re-evaluates the same closures — and that second reading IS the rung-64 measurement the section
exists for. So the split is not structural; it is per-call-site discipline, and § 2 is a defect if
a residual outlives its block while § 3 is dead if one does not.

**This is the SECOND instance in two steps, in the same two files.** Step 2 found `c_at`'s *"the
single expression-first `1e-9` fold in the whole package"* false at birth
([[rust-port-slice-ah-step2]]). So the rule is not *audit the "only" sentences*: it is that any
sentence quantified over a set — *only*, *every*, *can never*, *always* — is a promise about
members not yet added, and the cheap defence is to scope it to the body it is attached to. Repair
it WHERE IT STANDS; a correcting note one screen down leaves two contradictory sentences in one
file.

**How it was verified rather than asserted:** one closure, one set point, three calls differing
only in what the two frozen-state `Cell`s hold at call time. Both frozen → the open slope (≈10);
stator only → the closed slope (≈1e-8), which is § 3's whole measurement; the valve guard RE-ARMED
beside the stator's — the natural "tidy the two asymmetric blocks into one" edit — → the OPEN
number `to_bits`-equal. And the two `Cell`s are read AT the instant of the call, because a port
that got the block wrong would still produce entirely plausible § 3 numbers, just the open loop's.
Same shape as [[instrument-fed-by-what-it-certifies]] from the other side: do not infer the state
from the output when you can read the state.

**AND TWO BARS TYPED FROM THE NARRATIVE WERE BOTH FALSE.** I asserted `order_stable` and
`guarded_stable` true over the 24-cell sweep; the suite RETURNS both and GATES NEITHER, and both
are false. The repair was not deletion — the structure was measured and asserted instead (every
unstable cell sits at `margin = 0.40`, the dormancy corner § 4's own narrative names). Beside them,
the suite docstring's discrete count *"3 of 24 cells invert, every one at margin = 0.40"* ported and
the port reproduces exactly 3 — the only absolute cross-language agreement available at a step with
no oracle. **Read the asserts, not the prose, for what to gate; read the prose for what to
cross-check.**

**The citation guard ran INSIDE the step**, per [[rust-port-slice-ah-step2]]'s lesson: two new
anchors, re-blessed 33/181/112 → 34/183/114, zero arrears. The answer to *a guard that watches the
right directory still needs someone to run it* is not a note, it is running it in the step that
creates the citations.
