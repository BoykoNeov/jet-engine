---
name: rust-port-slice-u-step2
description: "Slice U step 2 — the one gate that reads the release edge sweeps none of the cells where the edge is at risk, so a whole-cell defect passes all 15"
metadata: 
  node_type: memory
  type: project
  originSessionId: 3688e397-f7fc-4957-8677-b3e66aa2c742
  modified: 2026-08-20T11:37:03.613Z
---

Slice U step 2 (rung 50's forced release: `release_relief` + `release_sweep` + 15 gates) shipped
2026-08-20. 1 323 keys over 49 cells bit-exact on the first run; 229 lines added, ZERO deleted.
Three process lessons.

**A DEFENDER AND AN EXPOSURE CAN SIT ON DISJOINT CELLS, AND NEITHER A BAR-MARGIN TABLE NOR A
READER CENSUS CAN SEE THAT.** The forced release compares the ACCUMULATED march coordinate against
`s_off`, and every `s_off` the suite passes sits ON the grid — so a "cleaner" `k*ds` spelling moves
the last armed index by a WHOLE CELL at two of them. Measured *before* writing the gates, and the
prediction that followed ("a gate must catch this") was WRONG. Gate 10 is the file's only reader
of the release edge's LOCATION and it does catch the same one-cell shift written directly at the
comparison site — but gate 10 sweeps `0.30/0.40/0.44` and `1.10/1.56`, none of them a knife-edge
cell, while the two gates that DO sweep those cells read only a sign. **Ask which cells a reader
reads, not just which keys** — both standing instruments are blind to that axis.

**A DUPLICATED COMPUTATION IS HELD AGAINST ITS DUPLICATE, NEVER AGAINST THE TRUTH.** `fuel_removed`
doubling IS caught — by the reduce gate, which compares `release_relief(s_off=None)` against
`surge_relief` field for field. So the reduce spine is a VALUE gate wherever a rung
re-implements its predecessor's quantity, which is a real second job it does. But break all THREE
identical trapezoids in the module and rungs 48/49/50 go green again: the protection is exactly
"the two copies agree". [[rust-port-slice-t-step2]]'s scale-invariance finding stands, sharpened.
Its twin: `nu_hp_end` off the wrong march is caught and `nu_hp_end_bare` is not, and the only
difference is which of ten key names Python's contract happened to list.

**WRITE THE PREDICTIONS DOWN BEFORE THE HARNESS RUNS, BECAUSE THE WRONG ONES ARE THE FINDING.**
Three of twelve were wrong, and each wrong one taught something a right one would not have. Also:
16 of the 27 keys have no reader at all — including `deficit_at_release`, which is the subject of
the gate that reads `fuel_removed` as its proxy. **The key carrying a rung's own concept can be
the one nothing reads.**

See `docs/plans/todo-rust-port.md` § 5.18 step 2. Related: [[rust-port-slice-u-step1]],
[[rust-port-guessed-census-bars]].
