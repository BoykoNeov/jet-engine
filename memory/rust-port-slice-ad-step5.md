---
name: rust-port-slice-ad-step5
description: "Slice AD step 5 (rung 72's oracle) — an input-fed section measures the FUNCTION where a self-feeding one measures the PLANT, so 5 022 golden differences became a 180-name exemption; and a per-section histogram that summed to its own print cap"
metadata: 
  node_type: memory
  type: project
  originSessionId: 83cf0ea9-5027-4b74-bb06-67cbdc8ab3b3
  modified: 2026-09-01T12:48:04.237Z
---

Slice AD step 5 shipped `rust/oracle/dump_slice_ad.py` and `rust/tests/slice_ad_oracle.rs` —
**54 116 keys**, the phase's largest oracle, `Rust ≡ PyPy` bit-for-bit on the first run with no
port fix. See [[rust-port-status]] for the tally; details in `docs/plans/todo-rust-port.md`
§ 5.28.5.

**The lesson: where a section gets its ARGUMENTS decides what it is measuring.** The quartic
section reads its 374 coefficient vectors from the golden as INPUTS and replays only the solver.
On the CPython arm that means the Rust runs Durand–Kerner on *CPython's own* coefficients — and
reproduces CPython's roots bit-for-bit on every vector. So of the **5 022 keys the two goldens
differ on, the port drifts on 180, and the solver section contributes ZERO**; all 4 842 of its
differences are upstream, in the `sum()`-built polynomial. **Read off the golden diff instead, the
step would have shipped a 5 022-name exemption naming the root finder as a cause it has nothing to
do with, and every gate would have passed.** A section that recomputes its own arguments measures
the plant and the function composed, and can only ever report their sum.

**Why:** the instinct when building an oracle is to make each section self-contained — call the
reader, emit what it returns. That is right for the readers and wrong for a leaf function you are
trying to localise a divergence *to*. Feeding the leaf its inputs is what makes "the solver agrees"
and "the plant agrees" separable claims at all.

**How to apply:**
- **Intercept a leaf function's ARGUMENTS, not just its call site**, and emit them as inputs the
  other arm replays on. Slice Z's *intercept, never reconstruct* one level deeper. The payoff is
  not tidiness — it is that the cross-interpreter exemption becomes a statement about a named
  function instead of a blob.
- **Score a pre-registered prediction CLAUSE BY CLAUSE and let the headline fall.** P7 read
  "exempt on keys downstream of a `sum()`, and on nothing else", with clause (ii) predicting the
  plant's own drift — which an earlier slice had already measured as *not* a `sum()`. The two
  halves cannot both hold. Measured: (i) exactly 4 keys and the right 4 BY NAME, (iii) exactly 0,
  (ii) confirmed — and the headline falsified by **6 keys**, march values at 2 points of 1 294,
  1–4 ULPs, no `sum()` within reach. Do not reinterpret a clause to rescue a headline; that is
  what writing the prediction down beforehand is for. Fourth phase-7 exemption falsified this way.
- **Re-score a booked injection in the SHIPPED key space, never by carrying the old count.** j05
  moved 26 keys in step 3's 3 216; here it moves **2 937** and is CAUGHT. The part that matters is
  that its A/D/E share is step 4's 26 exactly and the two keys that do NOT move are the same two
  **by name** — an independent measurement in a different key space landing on the same names is
  what turns a reading into a mechanism.
- **A discrete key is the strongest thing a bit-exact oracle can offer.** j05 moves `n_complex` on
  **163 of 374** vectors — the *number* of complex roots changes. No tolerance gate can absorb an
  integer, which is the case that the bit-exact seat was necessary and not merely convenient.
- **Make a "nothing changed" result non-vacuous by emitting the grid's own census.** P4 predicted a
  tolerance change moves no key; it moves 0 of 54 116. That is worth nothing unless the grid
  reaches the function, so the call count (12 676), the open-interval count (0) and **the margin**
  (2.74e−07, or **273 641× the tolerance**) are all keys. A margin, never the word "unreachable".
  Likewise: the `tie` branch is reached **0** times on this grid where the whole suite reaches it
  once — written as a GATE, because a silent absence reads as coverage.
- **A count quoted from a bigger population is a number about a different function.** The
  pre-flight's "375 distinct vectors / 167 near-double" was the whole suite (1 068 calls); this
  grid is 417 calls, **374** vectors, **69** near-double. Re-measured and emitted, with a tripwire
  that fails if the suite-wide pair is ever transcribed back in.
- **MEASURE before writing a section, even when the gap looks obvious.** I asserted the six-state
  march was uncovered; it is not — every reader reaches it. What is uncovered is the per-point
  FIELDS, and perturbing each in turn showed several moving **0** of the 3 216 reader keys. The
  advisor's demand for that measurement changed the section's justification, not just its size.
- **And my own histogram summed to my own print cap.** The first j05 split read `A 8, D 11, E 7,
  H 374` — total exactly **400**, the `.take(400)` I had installed on the panic message. The count
  was never capped; the NAMES were. Section H's true share is **2 911**. A total that equals a
  round number the instrument itself chose is the tell — re-take it with the cap lifted AND an
  assertion that the printed names equal the reported count.

Related: [[rust-port-slice-ad-step4]] (the booking this step discharges), [[rust-port-slice-ac-step6]]
(the plant drift that falsifies P7's headline, and the `every = 40`-vs-`10` defect),
[[rust-port-slice-z-step4]] (a pre-registered exemption that counted quantities where a dump emits
names), [[rust-port-slice-ad-preflight]] (a confident number from a run that reached nothing).
