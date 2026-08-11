---
name: golden-gate-slice6
description: "Rung 81 pinned as TWO arms — a band must clear the smallest LIVE value of everything it covers, and bit-equality can be predicted from the arithmetic instead of merely measured"
metadata: 
  node_type: memory
  type: project
  originSessionId: d43a0418-629a-4d67-9ddf-0ee4110a48ad
  modified: 2026-08-11T12:59:23.838Z
---

Slice 6 of the absolute-number regression gate (`tests/test_numeric_fingerprint.py`), shipped
2026-08-11, paying the debt `docs/rung81-spec.md` § 8 booked. See [[golden-fingerprint-gate]] and
[[golden-gate-slice5]] for the slices before it.

**The debt named the wrong half.** § 8 predicted the arm would need "slice 4's two-sided
reasoning, not a relative leg" — for the whole arm. Measured, rung 81's two readers land in
*different* numerical regimes, so it shipped as **two** arms: `r81` (the clock grid + criterion,
11 462 values) is **bit-exact** across CPython/PyPy, and `r81m` (the mirror mask, 784 values)
carries **all 155** differences.

**Why:** the split is forced, not tidy. One tolerance pair cannot serve both. The mask's
characteristic-polynomial coefficients need an absolute band of `1e-9`; the *smallest live value*
in the clock arm (`lag_gap` = 9.42e-9) is **below that band**. Merged, one of the two terms the
rung's own criterion is an inequality *between* would have sat inside a band worth 10.6 % of
itself — the finding banded away by the gate meant to protect it.

**How to apply:**
- **Size a tolerance against the smallest LIVE value it will cover, not only against the drift it
  was fitted to.** The drift sets the floor; the smallest covered value sets the ceiling. If they
  cross, the arm must split — that check is cheap and it is not the same as the module's existing
  "≥4× above drift AND decades below the live scale" rule, which asks about *one* family.
- **Bit-equality can be PREDICTED, which is stronger than measured.** `r81`'s exactness follows
  from the arithmetic: every operation in the criterion reader is a *single* IEEE op on a
  bit-identical input (two subtractions and a divide, a multiply, a `max`, a compare) — no naive
  `sum()` to reassociate, no transcendental — over a march four earlier arms already pin exact.
  Predict-then-confirm beats measure-then-assert; it also tells you where drift *will* live
  (here: the Jacobian → charpoly path, which sums products — rung 80's mechanism one rung along).
- **Justify a band from the arm's OWN quantities.** Slice 3's imported "alive at 2.9e-5" would
  have read as only 4 decades of headroom, because `r81m`'s own `c0` max is 1.14e-5. In-arm: `c0`
  is a dead determinant (one pole at the origin), `c1` is the surviving triple product
  (4.5e1–5.1e3), so a live fourth pole at the rig's 20 s⁻¹ puts `c0` at ~9e2–1e5 — twelve decades
  above the band, and no imported number needed.
- **Splitting buys separability, not just tolerance** (slice 2's `prop` argument): `r81` red means
  the plant or the criterion moved; `r81m` red means the polynomial arithmetic did.
- **Check order-independence when one rung becomes two kernels.** They may run in either order, in
  either worker, with the design memoised across them. Running mask-first reproduced all 12 246
  values bit-for-bit — the carried-knob trap this ladder has hit eighteen times, checked instead
  of assumed.
- No cut was taken: the rung's six-column table *is* its finding, so unlike [[golden-gate-slice5]]
  the sweep list stays whole and the cost (104.7 s PyPy, the module's heaviest arm) is disclosed.
