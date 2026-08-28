---
name: rust-port-slice-ac-preflight
description: "Slice AC's pre-flight — my headline was an ambiguity in functions my own census had measured as unreachable, while the real defect (a swap that breaks by emptying the sample) sat buried in a sub-table"
metadata: 
  node_type: memory
  type: project
  originSessionId: 67ab7e3b-ddec-4b0d-822d-77d5d6d5435f
  modified: 2026-08-28T16:50:47.033Z
---

Slice AC (rungs 70/71, `CrossSplitTransient` + `FullSplitTransient`) was pre-registered on
2026-08-28 off thirteen probes, written to `docs/plans/todo-rust-port.md` § 5.27.

**MY FIRST DRAFT'S LEADING FINDING WAS A HYPOTHETICAL THAT MY OWN CENSUS HAD ALREADY RULED OUT.**
I had measured that the three RK4 floors across rungs 69/70/71 assert a character-identical
condition and differ only in their message, and that six of the eight shipped gate strings match
more than one of the three — including the two Rust gates whose entire justification is that the
floor has no other observable. I wrote that up as *"AB's gates stop discriminating the moment AC
lands"*. **They do not**: probe 1 had measured, three sections earlier in the same document, that
the floors are **not cells** — each defined exactly once, under a distinct name — so no function
pointer exists for a floor and nothing in the dispatch harness can put the wrong body in the slot.
The ambiguity is real; the *defect* needs a mechanism, and there isn't one. The advisor blocked it,
and it demoted cleanly to a one-line tightening.

**AND THE REAL FINDING WAS ALREADY IN MY OWN TABLE, ONE ROW DOWN.** Swapping rung 68's
`_triple_laws` into rung 70's slot removes the governor, every sampled point goes off-regime, and
the reader returns **successfully** with `rows = []` and every aggregate `None`. No value differs
because there are no values. A dispatch gate of the shape every previous slice wrote — march both,
diff the keys — compares two empty tables and passes; the only shipped detector is
`assert gains["rows"]`, and the two value assertions beside it would raise `TypeError`. *A cell
whose output is a SAMPLE can break by changing the sample's SIZE rather than its values.*

**Why:** a pre-flight's headline sets what the slice's gates are built to catch, so an unreachable
one costs the slice its focus and buries the reachable one. The discriminating question is not *is
this true* but *what mechanism the port actually ships could exercise it* — and the answer was
sitting in a census I had run myself.

**How to apply:** before promoting anything to a leading finding, name the mechanism that reaches
it and check that mechanism against the census. If the answer is "a future hand-edit", it is a
note, not a headline. Three more things this pre-flight wants carried:

* **An impossible pair of numbers in one row is the cheapest self-check there is, and it only
  works if both are printed.** Probe 12 emitted `MAX NESTING DEPTH 37` beside `OVERWRITE 0`, which
  cannot both be true of one carrier — a global depth counter summing thirty machines. That is the
  *same* artifact `probe_ab11` recorded at slice AA. Per instance it is 1.
* **A docstring saying "single process" is not `-n 0`.** Probe 4's first run printed fifteen zeros
  because `pytest.ini` carries `-n auto`; the probe's own docstring named the blindness mode it
  then hit. The same run also failed a shipped gate — `inspect.getsource` on a method the probe had
  wrapped without `functools.wraps` — so an unlucky reading would have booked the probe's own
  damage as a finding.
* **When you trade a full-suite instrument for a cheaper one, disclose what the cheap one cannot
  see.** The reader-driven capture runs four grid-walkers at two arms instead of 4/9/6/9, so its
  site list is a LOWER BOUND and the exemption prediction is registered against the reduced grid.

Related: [[rust-port-slice-ab-step5]], [[rust-port-ported-test-vacuity]],
[[rust-port-guessed-census-bars]], [[rust-port-oracle-cannot-see-a-missing-gate]],
[[run-tests-below-normal]].
