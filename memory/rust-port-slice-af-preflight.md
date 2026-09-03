---
name: rust-port-slice-af-preflight
description: "Slice AF pre-flight — an inherited booking named the wrong reader and the wrong mechanism, and the zero it left behind was arithmetic rather than blindness; only a positive control plus a swept coordinate could tell those apart"
metadata:
  node_type: memory
  type: project
---

Slice AF pre-flight (rung 74, `DemandCoordinateTransient`). Plan § 5.30, nine probes. See
[[rust-port-status]], [[rust-port-slice-ae-step5]], [[instrument-fed-by-what-it-certifies]].

**THE LESSON: an inherited IOU is a claim, and it can be wrong in the reader it names, the
mechanism it blames, and the fix it prescribes — all three at once.** Slice AE booked one thing
here: `_with_coord`'s drive test, withdrawn as undriven because *"the reader pins its own
coordinate before the scope is entered"*, with `_cap_march` named as AF's candidate reader.

* **The named reader was not a reader.** `_cap_march` is rung 76's method and rung 79's call site.
  One grep of the class it actually belongs to settles that, and it was never run.
* **The mechanism was wrong.** The coordinate IS read — by exactly one method, on exactly one
  line, 16 times per gains reading. It is a three-valued tag read by a two-valued test, so two of
  its three values are indistinguishable *by construction* and the third is distinguishable only
  where one inequality flips.
* **So the prescribed fix — write the drive test — cannot be carried out**, and that is the
  finding rather than a failure to deliver.

**Why to apply it:** a booking written at the end of a tiring slice is prose, and the next slice
inherits it as a specification. Re-deriving it from the source cost three short probes and
produced a sharper result than the booking predicted; taking it on trust would have produced a
gate aimed at a method that cannot be reached from the rung it was booked to.

**How to apply it:** when a prior slice hands you a named candidate, **verify the name owns what
the sentence says it owns before building on it** — which class defines it, which rungs call it.
Treat the booking's *reason* as a hypothesis too, not just its task.

## The zero that looked exactly like the blind one

The probe reproduced AE's number — 0 of N keys moving — and the first draft wrote it up as a
verdict. Two checks separated the cases, and both were needed:

* **A positive control.** A mutated reader moves 4 of 20 float keys, on every arm. So the
  instrument can see, and the zero is arithmetic: the branch is `min(a, b)` and `a <= b`
  everywhere this call site looks, at 1 040 of 1 040 and 624 of 624 calls.
* **A swept coordinate.** The advisor's blocker on the draft: the identity had been measured at
  ONE setting, while another probe had already shown the inequality reachable on the same plant.
  **So the zero could have been a property of where the filter admits points rather than of the
  reader** — and "cannot be written" is not earned until every arm the reader admits is swept.
  Two arms came back zero; the third refuses through a shipped guard, which is an answer, not a
  gap.

**A zero that reproduces a prior zero is the most dangerous number to inherit**, because the
agreement feels like confirmation when it is the same defect twice.

## Two corrections that fell out of measuring rather than reading

* **A count already corrected elsewhere was still stale where it is read.** The phase table said
  3 cells; the census measures 4, and the missing name had ALREADY been published as missing by
  an earlier slice's phase-wide sweep. Nobody propagated it to the row. **Check the row, not just
  whether the correction exists somewhere.**
* **An inherited "no new solver, so the exemption carries" did not carry.** This rung adds a
  bracket walk into the very root finder a prior slice measured diverging by one iteration
  between interpreters. Measured on both: 139 of 2 732 calls actually solve, and all 2 732
  returned values are bit-identical. **The exemption is not needed — which is a measurement, and
  falsifiable, where "inherited" is neither.**
