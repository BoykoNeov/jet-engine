---
name: rust-port-slice-ah-step1
description: "Slice AH step 1 — a booking no gate can CLOSE can still be BLOCKED by one, and three instrument defects found by asking each instrument what it can see"
metadata: 
  node_type: memory
  type: project
  originSessionId: 01018948-48b1-4917-a966-9ac883194361
  modified: 2026-09-07T21:07:41.936Z
---

Slice AH (rungs 77/78) step 1 SHIPPED: `rust/src/stiffness_ledger.rs` (166) +
`rust/src/residual_gauge.rs` (513) = **679 module lines** against AG step 1's 658;
`rust/tests/slice_ah_cells.rs`, **5 gates green on the first run**; four re-aimed pointers, one
new core carrier (`gauge_k`), **zero new table fields**. Plan § 5.32.1.

**THE LESSON — a booking that no gate can CLOSE can still be BLOCKED by one.** The pre-flight
([[rust-port-slice-ah-preflight]]) proposed one repair for all six frozen-state nests: an RAII
guard restoring the PREVIOUS value, and called it *free*. Four of the six nests are inside rung
76's already-shipped `_c_at`, so the repair is a change to two guards' `Drop` across **66 call
sites in 11 modules**. Measured both halves:

* it is **value-invisible below rung 77** — rungs ≤69 were already at zero, **rungs 70–76 had
  never been measured** and hold most of those sites: **0 nests in 5 182 138 + 4 856 210 sets**,
  161 tests, nine per-worker tallies each with a passing positive control. A SECOND counter,
  added this step because an explicit `= None` is a real assignment the nest counter is blind
  to, found **5 018 510 events at 46 sites and every one a `finally` restore, zero guard
  entries**;
* and it is **blocked anyway** — `tests/slice_y_dispatch.rs` MANUFACTURES that exact nest and
  asserts the opposite policy, saying *"Two guards, two policies — do not unify them."*

So the pre-flight looked for a gate that could STATE its claim and found none; **it never looked
for a gate that had already DECIDED it.** The structural claim shrinks from six sites to two —
the ones this slice writes new code for — and the rest are safe by the DEAD-WINDOW criterion,
not by construction.

**THREE MORE, EVERY ONE ABOUT AN INSTRUMENT.** A doc comment cites a manufactured `MarchedStator`
nest that does not exist (the file it names pins two OTHER guards; the real pin is a different
field, file and slice) — third instance of [[rust-port-documented-gate-that-doesnt-exist]]. The
new gate's own header claimed it ran in the dev profile and so was weak against linker folding;
`Cargo.toml` sets `[profile.test] opt-level = 2`, so it runs in the folding-FRIENDLY profile and
is at its STRONGEST — **an error with the sign that flatters the author, which would have excused
a future miss as a profile artefact.** And the gate is **not yet shown load-bearing**: the
AG-style counterfeit-table demonstration needs a builder that is `pub(crate)`, so the file asserts
the swaps and says in its own header that it does not yet demonstrate a dropped one is silent.

**How to apply:** before adopting a port-wide "free" repair, grep the test tree for a gate that
already pins the opposite — a value-invisible choice is exactly the kind an earlier slice
manufactures a case for. And ask every instrument what it CANNOT see: the profile it runs in, the
assignment shape its predicate skips, and whether the demonstration it cites exists.

See [[rust-port-status]], [[rust-port-slice-ah-preflight]], [[instrument-fed-by-what-it-certifies]].
