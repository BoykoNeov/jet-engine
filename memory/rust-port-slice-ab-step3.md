---
name: rust-port-slice-ab-step3
description: "Slice AB step 3 — my injection sweep ran 2 of the slice's 3 test binaries and printed MISS, which reads as \"no gate sees this\"; and the one real hole was found only by dumping the numbers the injection moved"
metadata: 
  node_type: memory
  type: project
  originSessionId: 726592b0-ff21-4462-94f0-a344c53a6a1c
  modified: 2026-08-28T08:28:13.698Z
---

Slice AB step 3 shipped `rust/tests/rung69.rs` — 25 ported gates, compiled clean first try, all 25
green on the first run in 4.87 s. Full Rust gate 127 binaries / 1 241 passed / 0 failed.

**Two process lessons, both about my own instrument rather than the port.**

**1. An injection sweep's SCOPE is part of its summary column.** I injected ten plausible port
defects into `src/reference_split.rs` and ran each against `rung69` and `slice_ab_smoke`. Two came
back `MISS`. But `MISS` was printed under a heading that reads *"no gate sees this"*, and the sweep
had never run `slice_ab_cells.rs` — the slice's third binary, and the one that owns CELLS. One of
the two misses (`_with_ref` returning `None` instead of the value it displaced) fails **two** gates
there, both written at step 1 before a body existed. So the true score was 9/10, not 8/10, and the
one I nearly recorded as an ungated cell was gated in exactly the right file.

**Why:** a coverage number is only as wide as the set you ran, and a summary column that says
"MISS" without naming that set is claiming something the run cannot support. This is the same shape
as this slice's own pre-flight defect (probe 3's column headed `ref=` printing floats) — an
instrument whose heading answers a wider question than its code.

**How to apply:** before believing an injection's `MISS`, enumerate every test binary that could
plausibly see it and run all of them. When writing the result up, quote the SCOPE beside the score
("invisible to the five binaries run, of 127"), never a bare "nothing sees this".

**2. A `MISS` needs a did-it-move column before it can be called a hole.**
[[rust-port-slice-s-step2]] is the precedent. The one genuine hole — cutting `_cubic_roots_c`'s
Newton budget from 80 steps to 20 — is invisible to every binary run, and I only earned the right
to call that a hole by dumping the roots bit-for-bit under both budgets: **56 of 243 root
components moved, 24 of 81 `worst_zero` values moved, and 0 of 81 `n_zero` counts moved.** `n_zero`
is the only derived key any gate reads, and the pre-flight had already measured why it cannot move
(its threshold is 3.5 decades away). So the exhausted Newton arm is reproducible-by-contract and
gated-by-nothing, and step 4's oracle is the only instrument that can settle the slice's own P4 —
a pre-registered prediction whose status is therefore UNSETTLEABLE until that dump exists.

Two smaller things worth keeping: `reference_modes` takes its clock grid as `(tau_v, tau_att,
tau_s)` and keys each arm as `(tau_att, tau_v, tau_s)`, so two gates look the grid up under a
SWAPPED tuple — the lookup panics when no arm matches rather than unwrapping into the wrong one.
And Python's `type(s) is ReferenceSplitTransient` has no Rust counterpart: comparing the table's
address is [[rust-port-slice-aa-step1]]'s `ptr::eq`-on-a-`const` defect, so the sibling is instead
made to RUN a cell only rung 69's table has.

Related: [[rust-port-slice-ab-step1]], [[rust-port-slice-ab-step2]], [[rust-port-slice-w-step3]].
