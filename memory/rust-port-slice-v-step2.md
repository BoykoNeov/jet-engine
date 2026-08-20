---
name: rust-port-slice-v-step2
description: Slice V step 2 (rungs 57-60 port) — a golden dump can be bit-exact and still never run its own interesting branches; check the discrete flags it emitted
metadata: 
  node_type: memory
  type: project
  originSessionId: 7a4b055f-9f59-4394-b6bc-77210c27d4ad
  modified: 2026-08-20T15:10:02.348Z
---

Slice V step 2 ported rungs 57–60 (`ScheduledStatorTransient`, 2 510 lines of Rust) plus a
smoke test against a PyPy dump. The first cut was **1 742 keys bit-exact on the first run and
green** — and blind to three of its own branches, because the *cells I picked* never reached
them:

- the floor I chose never bound, so `floor_composite` sat in the `armed_clears` regime with both
  cells dormant and rung 60's whole derived claim (`pinned_prediction` = exactly `v`, or exactly
  `0`) was `NaN` everywhere;
- `schedule_invariance`'s two identity booleans were `false` in every cell, because the source
  claims `==` only at zero setting — so the branch that makes them `true` never ran;
- the two `pin_audit` flags downstream of the first were `false` everywhere.

**Why:** bit-exactness compares the values a run produced; it says nothing about which arm
produced them. A dump whose cells were chosen for "does it run" reaches the ordinary branch every
time. The only instrument that saw this was **reading the discrete keys the dump had already
emitted** — regimes, flags, counts — and asking which ones were constant across every cell.
Same shape as [[rust-port-slice-u-step1]] (bit-exact + green says nothing about GATE POWER), one
level out: here it is the *oracle's own cell choice*, not the suite's.

**How to apply:** after a golden dump goes green, list every BOOLEAN and every TAG key it emits
and check whether any is constant across all cells. A flag that is `false` (or a regime that is
one value) everywhere is an unexercised branch, not a passing gate — add a cell that flips it
before writing the step up. Two cheaper mistakes in the same pass (a hand-typed state that does
not bracket, a setting that goes off-map) were caught by Python raising, i.e. for free; the branch
blindness was the one that would have shipped.

Second lesson from the same step, on where descendant state lives: `bleed`/`stack_lp` set the
precedent of putting a descendant's state on the shared core, and I took that to mean the
DEEPEST core that holds the data. The level is actually set by **the shallowest hook receiver** —
here `try_close` is a rung-40 cell, so rung 57's arming had to sit on `TwoSpoolTransientCore`, not
on `TwoSpoolMapCore` (which is also shared with the steady ladder, so a transient-only field there
would be wider than the precedent). See [[rust-port-slice-n-step1]] for the mirror-image burn.

Third: `IncidenceLimiter.phi_lim_at`'s assert had to become an `Abort`, not a panic — it is
reachable from inside the RK4 derivative, where Python swallows the `AssertionError` and
truncates. It fires only outside the admissible band, so **no bit-exact dump over working cells
could have caught it**; it was found by tracing the call path before writing the type.
