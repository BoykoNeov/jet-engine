---
name: rust-port-slice-z-preflight
description: "Slice Z pre-registration — the leading finding landed on my own probe: it chunked by the n_sample the gate PASSES, not the count the STRIDE delivers, and at the right width the answer inverted"
metadata:
  node_type: memory
  type: project
---

Rungs 66 + 67, nine probes, every table emitted. **1 496 source lines (3.12× slice Y), 38 gates,
0 cells added by either rung** — both swap the same three already-open cells, which the emitter
confirms for the third and fourth time in a row.

**A PROBE THAT READS A GRID PARAMETER INSTEAD OF THE DELIVERED ONE MEASURES A DIFFERENT
FUNCTION.** The one float `sum()` in either rung is `cross_identity`'s `P_mid`, so slice W's
CPython-compensated-vs-PyPy-naive question had to be asked again. The probe chunked the captured
products by `n_sample = 8` — the value the gate passes — and reported **no divergence**. A later
probe printed the reader's own delivered count and it is **9**: the expression is a STRIDE,
`ride[:: max(1, len(ride) // n_sample)]`, so the delivered length is
`len(ride) // (len(ride) // n_sample)`. Re-chunked at 9 the answer **inverted** — CPython differs
on 2 of 4 windows. A "no divergence" that came from summing windows the reader never sums is the
same class as [[rust-port-slice-w-step3]]'s five wrong rows, landing on my own instrument.

**THE AMPLIFICATION QUESTION IS CHEAP AND WAS ANSWERED BY MEASUREMENT.** Pushing both
interpreters' `P_mid` through the reader it feeds took seconds and turned "an ulp somewhere" into
a counted list: **2 of 8 keys move by one ulp and five absorb it exactly**. So the exemption step
4 owes is a named pair, not a tolerance tier. Do this before writing the exemption, never after.

**A DEAD ARM CAN BE DEAD BECAUSE THE RUNG'S OWN HEADLINE IS TRUE.** `_eig`'s complex branch is
**80 of 80 unreached** on rung 66, which is exactly what `det J ≡ 0` means — and it is live one
rung up, where `det J ≠ 0`. The first census conflated the two because rung 67 calls its parent's
`_eig`; splitting by CALLING FUNCTION is what separated them.

**AND THE PREVIOUS SLICE'S HEADLINE IS THE THING A PRE-REGISTRATION IS TEMPTED TO INHERIT.** Slice
Y's finding was two one-armed dispatches. Here all four arms of both dispatches are live, measured.
Registered as a non-recurrence so nobody carries it forward on the strength of the family
resemblance. Same for `py_max3`: rung 66's three-argument `max` is over a quantity that cannot be
NaN, measured, so slice Y's bug does not recur.

**Why:** three of the four things worth knowing about this slice came from asking an instrument
what it actually measured, rather than from reading code.

**How to apply:** when a probe slices a sequence, print the delivered length beside the requested
one — a stride, a `//`, or a `[::k]` means they differ. Split any counter on a method the rung
above also calls by its CALLING FUNCTION. And when a pre-registration reaches for the previous
slice's shape, measure the non-recurrence explicitly instead of omitting it.

Related: [[rust-port-slice-y-step4]], [[rust-port-slice-w-step3]], [[rust-port-slice-w-step4]],
[[rust-port-copy-vs-rederivation]], [[rust-port-guessed-census-bars]].
