---
name: rust-port-copy-vs-rederivation
description: "An 'exactly' claim survives a COPY and dies on a REDERIVATION — the discriminator to apply before measuring the next one"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 519adbe9-fbbb-49f5-9712-29d4dd7fed36
  modified: 2026-08-12T17:30:21.599Z
---

Slices C, D and E each CORRECTED a source claim of the form "reduces to X **exactly** / to the
ULP" — three for three, so slice F pre-registered rung 26's version as the likely fourth
correction. It **survived**, 40/40 bit-exact, and the reason is structural rather than lucky.

**The discriminator: what is being compared.**

* **A COPY survives.** Rung 26's march is rung 25's loop duplicated line for line with one scalar
  promoted to a function. A constant rate collapses it back to the *identical instruction
  sequence*, so "to the ULP" is a statement about two runs of the same arithmetic.
* **A REDERIVATION does not.** Slice E's corrected claim compared two ROUTES to the same physical
  number (a molar-entropy bisection against the production nozzle's safeguarded Newton). Different
  arithmetic reaching the same value is exact in algebra, never in floating point — which is also
  the [[rust-port-inside-outside-exactness]] lesson from the other side.

**Why:** this turns "the source says exactly" from a uniform prior into a prediction with a
reason, so the probe can be registered as a *prediction* rather than a bare measurement — and a
refutation then carries real information instead of just adding to a tally.

**How to apply:** before measuring the next exactness claim, ask whether the two sides are the
same instruction sequence or two derivations of the same quantity. Register the prediction that
follows. **And when the answer is COPY, do not factor the duplication away in the port** — the
Python duplicates rung 25's loop deliberately, and merging the two Rust copies into one generic
body would compile, pass the oracle, and silently turn the reduce into a self-comparison. That is
[[rust-port-ported-test-vacuity]] again, reached from the opposite direction: there a better
factorisation dissolved a test, here it would dissolve the thing the test checks. Related:
[[rust-port-measure-before-registering]].
