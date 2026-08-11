---
name: golden-gate-slice7
description: "Rung 82 pinned as THREE bit-exact arms — a root-finding arm needs a go/no-go before it is written, and the smallest live value belonged to the SEARCH, not the physics"
metadata: 
  node_type: memory
  type: project
  originSessionId: c3d978cb-c37a-4515-b9c3-2d1053dfa3e9
  modified: 2026-08-11T16:30:01.069Z
---

Slice 7 of the absolute-number regression gate (`tests/test_numeric_fingerprint.py`), shipped
2026-08-11, paying the debt `docs/rung82-spec.md` § 8 booked. See [[golden-fingerprint-gate]] and
[[golden-gate-slice6]] for the slices before it.

**The debt was wrong on every count it made.** § 8 predicted two readers in different numerical
regimes needing different tolerance pairs, and nominated `8.29e-04` as the smallest live value a
band would have to clear. Measured: three readers, **all three arms BIT-EXACT** (0 of 440 values
differ, CPython 3.14.3 vs PyPy 3.11.15), so no band exists to be shared or split — and the
smallest live value on every arm is the bisection **width** `2.8906e-04`, a decade below the
nominated one.

**Why:** rung 82 builds no Jacobian, so `_charpoly4` — the sums-of-products path that carried
*every* difference in slices 5 and 6 — is absent from it. What remains is single-op arithmetic on
a march five earlier arms already pin exact.

**How to apply:**
- **An arm that ROOT-FINDS needs a go/no-go before a line of it is written.** Slices 3–6 read
  values off a *fixed* march, so the module's shared gas (`R_c = 286.9` vs the rung's own 286.857)
  was a benign caveat. A bisection can put the root on the other side of a censoring edge and
  leave the arm pinning `void` strings — and rung 82's whole marchable wall range is 0.01 wide.
  Probe first, check `n_void`/live-row counts, *then* write. Here it passed, and the arms
  reproduce every number the spec publishes.
- **Name the arm's own vacuity condition, not a generic one.** For the § 3a arm it is `crossing`:
  a 5-of-5 sign law where all five references sat on ONE side of the threshold would score
  perfect having tested nothing. Pin the `(reference, side)` pairs, not the boolean.
- **Predict-then-confirm again ([[golden-gate-slice6]]), and it is stronger when the drifting
  mechanism is ABSENT rather than merely elsewhere.** Slice 6 could predict exactness for one half
  and had to name where drift would live; here the named mechanism does not exist in the rung, so
  all three arms follow.
- **Splitting arms does not need a tolerance reason — but then say which reason it is.** These
  three are split for COST (410 s PyPy in one test is a serial tail on one xdist worker vs 191 s
  for the largest of three) and ATTRIBUTION (three different findings get three names). Slice 6's
  split was *forced* by a band; dressing this one up the same way would be false.
- **Extend the vacuity guard when a new reader names its row lists something new.** Rung 82 calls
  them `govs`/`walls`, and the guard knew only `rows`/`cells` — slice 5's lesson recurring. Here
  the strict form is right (neither may be empty): an emptied wall sweep means the marchable band
  moved, which is the regression the slice exists to catch.
- Cost disclosed, not traded: 410.2 s PyPy, 1964.0 s CPython (32.7 min), putting a whole-module
  regeneration at ~65 min. Order-independence checked reader-reversed: 0 of 440 differ.
