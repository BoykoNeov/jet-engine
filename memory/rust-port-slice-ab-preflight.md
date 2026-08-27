---
name: rust-port-slice-ab-preflight
description: "Slice AB's pre-flight — a shipped section's EXPLANATION was too strong and my replacement was refuted by the same probe; and a swap whose only content is a panic string"
metadata: 
  node_type: memory
  type: project
  originSessionId: 91ad0bf3-c7c6-4a43-aec1-3223504de8a7
  modified: 2026-08-27T15:18:07.196Z
---

Slice AB (rung 69) pre-registered 2026-08-27, eleven probes, § 5.26 of
`docs/plans/todo-rust-port.md`.

**The lesson that generalises: a shipped section can carry a MECHANISM beside its
measurement, and the mechanism is not measured.** Slice AA § (i) explained its one CPython
`sum()` exemption by LENGTH — "the compensation only has somewhere to accumulate when the
list is long". Rung 69 has a THREE-element sum that diverges on 23 of 256 instances. My own
replacement explanation (catastrophic cancellation) was refuted by the same probe: the
diverging instances' cancellation ratios sit *inside* the agreeing ones' range, and the three
worst-cancelling instances agree bit-for-bit. **How to apply:** when inheriting a prior
slice's finding, inherit the MEASUREMENT and re-run the instrument; never inherit the
sentence that explained it. See [[rust-port-slice-aa-steps2345]], [[rust-port-slice-z-step4]].

**A SWAP-dominated slice inverts the usual risk.** AA added nine cells; AB adds ONE and
swaps TEN. "A slice that forgets a cell fails at its own first gate" is a statement about
ADDED cells, so it buys nothing here — the failure mode is a swap whose Rust body is still
effectively the parent's. **How to apply:** in a swap-dominated slice, run step 5's
observability question in the PRE-FLIGHT — at every call site the suite reaches, would the
parent's body have returned something different? It cost one probe and reshaped the steps.

**One swap's entire content was a panic string.** `_rk4_floor`'s condition is
character-identical to rung 68's; only the assert MESSAGE changed. 0 disagreements in 77
calls — but a ported test does `pytest.raises(match="rank TWO")`, so it IS gated. **How to
apply:** "no value key moves" is not "unobservable"; check what reads the MESSAGE before
writing a cell off, or it ships ungated. Four more cells here are observable only BY PANIC
(the parent dereferences a `None`), which is a dispatch-gate shape the phase has not used.

**Three of my own instruments printed numbers that would have been carried** (§ (xi)): a
column headed `ref=` that was reading `tau_rel`; a "100 % pass-through" line that answered
the wrong question about a clamp an inversion WOULD have been caught by; and sizing bars
taken by `grep -c` instead of a pytest collection ([[rust-port-guessed-census-bars]] again).
The advisor caught the last one before it entered the write-up.

Related: [[rust-port-slice-aa-step1]], [[rust-port-measure-before-registering]],
[[rust-port-oracle-cannot-see-a-missing-gate]].
