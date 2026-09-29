---
name: rust-port-slice-aj-preflight
description: "Slice AJ pre-flight (rungs 81–84) — an inference that a suite is blind, drawn from a grep for what a catcher would SAY, was refuted by RUNNING the mutation: the catcher was an incidental method lookup on the rebuilt object"
metadata:
  node_type: memory
  type: project
  originSessionId: e531caa2-a1d9-43c2-b26a-ac12e7fb2e03
  modified: 2026-09-29T09:27:01.042Z
---

Slice AJ pre-flighted 2026-09-29, plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.34, probes in `W:\temp\claude\slice-aj-preflight\`. 24 methods over 4 classes, all
single-definer: 0 ADD, 0 SWAP, no `R81`…`R84` tables (readers run on an `R80` machine).

**THE LESSON: an inferred blindness is a prediction, and running the mutation is cheaper than
trusting it.** AI step 7 grepped `test_rung80.py` for `at_lever`/`isinstance`/a carry test, found
none, and called the Rust `rung80.rs` "faithfully blind". Run in Python (two full copies of the
tree under `W:\temp\claude`, one mutated), the deletion failed `test_the_knob_is_loud` with an
`AttributeError`: the test calls `rig._walls_of(...)` ON THE REBUILT RIG, and the rung-79 class has
no such method. Rust calls a free function, which accepts any core — so the port is under-ported.

**Why:** a grep searches for what a catcher would SAY; an incidental catcher (a method lookup, an
attribute read) says nothing about what it catches.

**How to apply:** when a claim is "suite X cannot see defect Y", run Y against X before writing it
down; and when porting a Python call made ON an object built elsewhere, ask whether the Python
lookup itself was a structural check the port's free function erases.

Also measured / process:
- A LOAD control (which copy was imported) is not an ENTRY control (was the path run). The advisor
  caught it; a counter on `at_lever` in each kernel showed the rebuild path ran 2–81 times in all six.
- A light-mode probe's zeros are "not instrumented", not measurements.
- Cross-run call COUNTS under xdist differ because module fixtures rebuild per worker — checked per
  worker (1 call per worker needing it), then only SETS compared.
- A census of NEW methods on a rig missed OLD methods called on it (`authority_mask` dispatches
  five rung-72/73 methods on the rig); an AST receiver scan found it.
- Two void strings misname their cause (empty window reported as "kappa impure" / "edge off the
  grid"); the port reproduces them verbatim.
- P1 finally discriminates: AJ has 0.84 dict fields per body line vs ~0.34 for AG–AI; predicted
  2 700–3 700 Rust lines (per-line), above 4 000 = per-field wins.

Related: [[rust-port-slice-ai-step7]], [[rust-port-status]], [[instrument-fed-by-what-it-certifies]].
