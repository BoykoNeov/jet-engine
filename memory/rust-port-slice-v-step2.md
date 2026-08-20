---
name: rust-port-slice-v-step2
description: "Slice V step 2 (rungs 57-60 port) — both gates written to CLOSE the step could not fail; check falsifiability, never greenness"
metadata: 
  node_type: memory
  type: project
  originSessionId: 7a4b055f-9f59-4394-b6bc-77210c27d4ad
  modified: 2026-08-20T15:26:27.693Z
---

Slice V step 2 ported rungs 57–60 (`ScheduledStatorTransient`, ~2 500 lines of Rust) plus a smoke
test against a PyPy dump: 1 986 keys bit-exact on the first run, whole crate green. **Both of the
gates I wrote to close the step were unfalsifiable, and both were green.**

**Gate 1 — a `fn`-pointer comparison that was a compile-time tautology.** To show rung 57 inherits
two of rung 40's three hook cells I asserted `R57_TWO.try_instant_tail == R40.try_instant_tail`.
But `R57_TWO` is built with `..R40`, so no struct literal spelled that way can make it fail; the
inequality assertions beside it are tautologies too (distinct `fn` items always have distinct
addresses). The write-up had already called it MEASURED. The advisor blocked on it. Replaced with
one that turns the same claim into a number — march an armed machine, call both cells against the
stale map and the design map, assert bit-identical — plus an anti-vacuity clause asserting the two
maps differ *observably* first, since otherwise a zero reads the same whether the cells are
invariant or the maps are equal.

**Gate 2 — a golden dump whose cells never reached the interesting branches.** The floor I picked
never bound, so the regime flag was constant and the rung's whole derived claim was `NaN` in every
cell; two identity booleans were `false` everywhere because the source only claims them at zero
setting. Caught by **reading the discrete keys the dump had just emitted** and asking which were
constant across all cells — not by running anything.

**Why:** a gate written at the end of a step is written to confirm what you just built, and that
is exactly the frame in which "it passed" and "it cannot fail" are indistinguishable. Greenness is
evidence about the run, never about the gate.

**How to apply:** before recording any gate as evidence, do two things. (1) **Manufacture the bug
it names and confirm it fails** — and record the injection SIZE, because detectors have floors:
here `+vsv*1e-9` failed the pointwise gate and `+vsv*1e-15` did not (below an ULP), while the
marched dump caught even the 1e-15 one. A pointwise bit gate bottoms out at an ULP; a marched
trajectory amplifies; neither subsumes the other. (2) **List every boolean and tag key** a dump
emits and check none is constant across all cells. Related: [[rust-port-slice-u-step1]],
[[rung83-corrector-law]] (an identity round-trip sold as verification).

Two more from the same step, both worth keeping:

- **A carrier's LEVEL is set by the shallowest hook receiver, not by where the data lives.**
  `bleed`/`stack_lp` put descendant state on the shared core, and I read that as "the deepest core
  holding the data". Wrong: rung 57's arming had to sit on `TwoSpoolTransientCore` because
  `try_close` is a rung-40 cell — and the deeper core is shared with the steady ladder, so a
  transient-only field there is wider than the precedent. See [[rust-port-slice-n-step1]].
- **A guessed bar failed on the clean tree, again.** I asserted a quantity the source calls
  "clock-free" was bit-equal across a rate ladder; it moves 0.73 %, because the source's own
  headline says ~1 point over a 20× range. Re-set on the measured contrast (credit 0.73 % vs
  excursion 66.3 %, 91×). Same shape as [[rust-port-slice-v]] step 1b's guessed HP bar.
