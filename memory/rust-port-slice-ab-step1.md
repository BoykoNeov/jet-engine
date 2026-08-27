---
name: rust-port-slice-ab-step1
description: "Slice AB (rung 69) step 1 — two gates that could not fail, and TWO counts typed instead of added up, one of them read off a runner that reported exit 0 with empty output"
metadata: 
  node_type: memory
  type: project
  originSessionId: d998bdc2-4ce6-41ce-a984-cf6a71985004
  modified: 2026-08-27T16:49:25.842Z
---

Slice AB of the Rust port (rung 69, `StatorIncidenceLimiter` + `ReferenceSplitTransient`) shipped
step 1: `src/reference_split.rs`, one added cell (`with_ref`) on `TripleHooks`, nine swapped cells
opened as named panics, one field each on three structs, and `tests/slice_ab_cells.rs` — 13 gates.
Plan record at § 5.26.1. Bodies land at step 2.

**A SWAP-DOMINATED SLICE INVERTS STEP 1's USUAL GATE, and the design follows from that.** A
forgotten swap is not a missing function — it is the parent's, which compiles and runs. So the
rung-69 table is spelled with **no `..R68_TRIPLE` spread** (the one inherited cell is a decision on
the page), and the nine unported bodies are **named panics rather than `todo!()`**, because
`todo!()` and the parent's body look the same to a reader and a per-cell message makes the slots
addressable by a gate. **How to apply:** when the risk is "silently the parent's", make the parent
unreachable by construction rather than trusting a later value key.

**TWO OF MY OWN GATES COULD NOT FAIL, AND THE FIX FOR BOTH WAS TO STOP PREDICTING.**
`assert!(!msg.contains(": _triple_laws"))` — nothing in the crate emits that string, so it was a
tautology on every input (third instance in this phase; see [[rust-port-slice-v-step2]]). The
repair was to *run the closure and read the message*: it is empty, and that is now pinned
positively. And `assert_eq!(got.len(), UNPORTED_AT_STEP1.len())` compared nine hand-written
closures against nine hand-written names — blind to a cell forgotten in both
([[rust-port-documented-gate-that-doesnt-exist]]). Replaced by an exhaustive struct literal with no
`..`, so the width is a compiler error. **How to apply:** before writing an assertion about a
string, grep for the string; before comparing two counts, ask whether both are typed.

**TWO COUNTS TYPED INSTEAD OF ADDED UP, IN THE SLICE WHOSE PRE-FLIGHT NAMES THAT DEFECT.**
(1) I justified `StatorIncidenceLimiter`'s missing third assert with *"a margin is signed, so
`m_lim` is negative"*, asserted it, and the gate went red — `T_c = 1/phi_surge` exactly, so the
shipped floor's margin is POSITIVE. The real witness is the boundary: at `sm = 0` the reciprocals
cancel and `m_lim` is exactly zero, which `from_margin` admits and a copied-over `> 0` would
refuse. (2) I wrote "six literals moved, none in `tests/`" — the measured answer is nine, 8 in
`src/` and 1 in `tests/`. See [[rust-port-slice-ab-preflight]]: inherit the MEASUREMENT, never the
sentence.

**AND THE `tests/` ZERO CAME FROM A BACKGROUNDED `cargo build --tests` THAT REPORTED EXIT 0 WITH
EMPTY OUTPUT.** The real run does not compile until `slice_v_dispatch.rs` gains the new field. I
had the tool's own output (empty — a build of ~100 test binaries prints plenty) sitting beside the
runner's exit code and believed the exit code. [[windows-tooling-file-hazards]] records exactly
this. **How to apply:** an exit code with no output is not a result; re-run in the foreground, or
treat the empty output as the finding.

**One found-not-fixed, from hoisting rung 69's `lp_disabled` guard above the build:** rung 68's
sibling guard in `three_loop.rs` sits after a build that already refuses `lp_disabled` with a
rung-57 message, so it is provably dead — a defence with no reader inherited from slice AA. Booked
at § 5.26.1 (e) rather than repaired, because it is not this slice's.

Related: [[rust-port-slice-aa-step1]], [[rust-port-guessed-census-bars]],
[[rust-port-slice-y-step3]].
