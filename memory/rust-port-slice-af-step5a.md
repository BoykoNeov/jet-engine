---
name: rust-port-slice-af-step5a
description: "Rust port slice AF step 5(a) — a shipped Usage: block whose documented calls raise TypeError, and the phase record that measured the same defect at 3 when it is 10"
metadata: 
  node_type: memory
  type: project
  originSessionId: 8d42ecda-e715-4ce4-b3ba-8f5955f7ae10
  modified: 2026-09-06T04:13:59.126Z
---

Slice AF step 5(a) (rung 74, § 5.30.5 (a)): `rust/tests/rung74.rs`, **17 gates green on the first
run**, 16 ported 1:1 in order + **1 declared ADD**. Counts measured on both sides: `pytest
--collect-only` says 16 collected / 8 slow; `cargo test -- --list` and `grep -c '#\[test\]'`
BOTH say 17, so the four-slice streak of grep over-counting (AC ×2, AD, AE — every one an extra
`#[test]` inside the file's own prose) **did not continue**, and the trap is a property of files
that discuss it in a code fence rather than of ported gate files.

**THE LESSON: *what supplies the value under test?* has a sibling — *what has ever RUN the
sentence?*** `DemandCoordinateTransient`'s shipped `Usage:` block documents four calls and
**three of them raise `TypeError`**: `sm=` was never a parameter of `demand_gains`,
`latch_discriminator` or `flat_schedule_identity` (all three take `phi_lim=`); only `demand_law`
takes `sm`. Found only because the ADDED gate's bars needed a source and that block is the one
thing in the tree that mentions `latch_discriminator` at all.

**Swept over the WHOLE table rather than the row** (slice W's rule), as a static bind of every
written call against `inspect.signature` — no plant built, so a failure is the signature's and
cannot be a runtime error inside a body. **58 classes, 34 `Usage:` blocks, 102 calls, 5 skipped,
87 bind OK, 10 BIND FAILS**, in three causes:

* **NO SUCH METHOD — 3** (rungs 65 `restored_plant`, 66 `cascade_modes`, 72 `shared_modes`).
  **These are exactly [[rust-port-slice-ad-preflight]]'s three, reproduced by a different
  instrument.**
* **missing a required argument — 4** (rung 63 ×3 want `lever`, rung 65's `bandwidth_ceiling`
  wants `phi_lim`). New.
* **unexpected keyword `sm=` — 3** (rung 74's). New, and mine.

AD measured the LOUDEST cause and stopped at 3. The two quieter ones — the name is there and the
ARGUMENTS are wrong — are **7 of the 10**. Rung 74's three fixed in this commit (10 → 7 measured);
the other seven belong to closed slices and are disclosed and booked with the guard test that
would stop the class regrowing.

**The ADDED gate's bars come from a DOC, and that is the point.** `latch_discriminator` has no
caller in the shipped tree, so there is no Python test to mirror; its bars are transcribed from
`docs/rung74-spec.md` § 3's prose (`floor_dTt4 = 65.2 K`, `332 of 341`), **never from step 4's
drive output** — a number the port produced is not a bar on the port. The two integers assert
exactly, `65.2` as its rounding bracket. It also asserts anchor P6's refutation *in the direction
it was refuted*, so a port that made the two arms agree fails rather than looks tidy.

**Both refusal needles are transcribed from `turbojet/engine.py`'s literals**, never from
`demand_coordinate.rs`: step 4 measured the port's refusal text four formatting divergences wide,
so a needle copied from the Rust would certify the divergence instead of catching it.

See [[rust-port-slice-af-step4]], [[rust-port-slice-ad-preflight]], [[rust-port-status]].
