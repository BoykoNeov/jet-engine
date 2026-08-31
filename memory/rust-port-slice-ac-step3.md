---
name: rust-port-slice-ac-step3
description: "Slice AC (rungs 70/71) step 3 — the shipped docstring published a number its own reader does not return, and only driving the reader on its OWN defaults could find it"
metadata: 
  node_type: memory
  type: project
  originSessionId: ce0f95cd-5a5e-4cb4-bcb0-2f1752971a37
  modified: 2026-08-31T08:35:04.931Z
---

Slice AC step 3 ported rung 71's bodies: `src/full_split.rs` 171 → **1 536 lines**, all eleven
methods (five non-readers + six readers), `Census71`, `round10`. Plan record at § 5.27.3.

**THE FINDING WAS NOT IN THE PORT — IT WAS IN THE PROSE THE PORT WAS CHECKED AGAINST.**
`FullSplitTransient`'s class docstring says the ring floor fails *"at matched clocks
`zeta = 0.5895` against rung 69's `0.5974`"*. `full_modes` on **its own default grid** returns
`0.588974` / `0.596811` — and `docs/rung71-spec.md` § 5 has carried exactly that pair all along.
Python and Rust agree to the last bit, so it is not a port defect; it is a published number that
57 collected tests do not gate, and every reading of that sentence since it was written has been a
reading rather than a run. Corrected with the finding, on [[rust-port-slice-ac-step1]]'s
precedent. **A drive-once check is worth what its ARGUMENTS are worth: driving a reader on the
CALLER's grid reproduces the caller; driving it on its OWN defaults is the only thing that can
audit a number the prose publishes with no caller.**

**FOUR PLACES WHERE THE PARENT RUNG'S TEMPLATE COMPILES, RUNS, AND MEANS SOMETHING ELSE** — all
registered before the bodies were written, because [[rust-port-slice-ac-step2]] had just paid for
one of these by accident:

* `full_modes`' `complex_pair` is `zeta is not None` — **ANY** root, where rung 70's row asks it of
  the DOMINANT one. Two aggregate keys read the difference.
* `_zeta_ring` does **not sort**: it takes the FIRST complex root in the solver's own order. The
  rung's own docstring already measured the two readers disagreeing on 4 of 12 arms.
* `full_gains` evaluates the parent-plant control **below** the `continue`; rung 70's `split_gains`
  deliberately evaluates both arms **above** it. A skipped sample costs one gains call here and two
  there — invisible to every float, visible to a counter.
* `full_bill`'s `v_hi` is `p.get("v", 0.0)` and **the fallback is REACHABLE** (4 of 8 ledger cells
  march with no stator). The family's other non-`Triple` reader has an UNREACHABLE fallback and
  panics — the opposite conclusion from a line that looks the same.

**THE ONE NEW NUMERIC PRIMITIVE WAS SETTLED BEFORE THE FILE WAS WRITTEN, NOT AFTER A GATE FAILED.**
`ic_contraction`'s `members` is a SET SIZE over `round(x, 10)` triples and IS the rung's headline
(1 on the full rig, 4 on the control). `round10` is format-and-parse with the signed-zero
normalisation carried over from rung 68's `ic_family` — unreachable on the shipped grid, and
therefore exactly the latent off-by-one no value gate could witness.

**AND STEP 2's OWN LESSON WAS APPLIED TO STEP 2's OWN REPAIR.** Its Gate-1 fix (a per-line
`pub struct …Hooks` filter) was authored while `full_split.rs` held 171 lines and zero reader
structs; this step adds **17 `pub struct`s**. The repaired filter reads 0 and is indifferent to
them; the proxy it replaced would have failed for the second time in one slice. The `UNPORTED`
scaffolding const was deleted and **no gate read it** — verified by grep, which is what step 1 had
bought by refusing slice AB's read-the-panic-messages gate.

**Why:** three of these are the same failure — a property asserted somewhere other than where it is
enforced (in a docstring, in a parent's row spelling, in a call ORDER) — and the fourth is a gate
whose scope was fixed by the file's size on the day it was written.

**How to apply:** before porting a reader, diff its row/aggregate spellings against the parent's
line by line, and treat *the parent does this* as a hypothesis. Drive every reader once at its OWN
defaults, not the caller's, and diff against the source language before writing a single gate. And
when a step's predecessor repairs a gate, re-run that gate's question against what THIS step adds.

Related: [[rust-port-slice-ac-step2]], [[rust-port-slice-ac-step1]], [[rust-port-slice-ac-preflight]],
[[rust-port-power-spelling]], [[rust-port-shape-keys]], [[rust-port-copy-vs-rederivation]].
