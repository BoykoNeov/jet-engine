---
name: rust-port-slice-j
description: "Slice J (rung 32, the component map) — the port's habitual currency was a perfect non-discriminator, and a qualifier with no assertion behind it is free"
metadata: 
  node_type: memory
  type: project
  originSessionId: cd5969d1-a53f-40aa-b7d9-921d25427fef
  modified: 2026-08-14T07:39:45.795Z
---

Phase 5 slice J of the Rust port (rung 32, `ComponentMap` + `MapMatcher`) shipped 2026-08-14,
7 252/7 252 bit-exact vs PyPy, 69.2 % vs CPython. Five process lessons, in the order they bit.

**1. A HABITUAL CURRENCY IS NOT A DISCRIMINATOR UNTIL YOU MEASURE THAT IT DISCRIMINATES.** The
port states "constant vs varies" as a count of distinct bit patterns — it has worked since
[[rust-port-shape-keys]]. Here it was worthless: the quantity claimed map-free moved in the last
bits in **every single cell**, exactly as the quantity claimed map-dependent did. The claim was a
MAGNITUDE claim all along (the source's own bar is `1e-4`, not zero), because the quantity is
reached through a fixed point whose *other* variables do move, and a converged iterate carries its
history in the last bits. Measure the discriminator on both arms before building a gate on it.

**Why:** structural independence ("no map coefficient enters the equation that sets it") does not
imply arithmetic independence when the value comes out of an iteration. Those are different claims
and the port kept conflating them — the same split as
[[rust-port-inside-outside-exactness]], one level up.

**How to apply:** before writing a count-based gate, compute the count for BOTH the quantity that
should collapse and the one that should not. If both spread, the claim is about magnitude; gate the
relative spread instead — and assert the RATIO, not the direction. `a < b` still passes with the
whole effect gone (the [[rung83-corrector-law]] lesson: a bar naming a DIRECTION instead of a POINT
dies).

**2. A CONDITIONAL CLAIM NEEDS ITS CONDITION ASSERTED, OR THE QUALIFIER IS FREE.** The reduce is
bit-exact "on the choked branch only". The first draft asserted the choked half (28/28) and merely
*dumped* the other. Nothing tested that the excluded cells actually FAIL to reduce — so a cell that
happened to agree would have read as support. Added `assert!(n_sub_bitequal < n_sub)`; measured
0 of 4. Same shape as [[rung79-state-coordinate]]: registering the vacuity condition beat
registering the result.

**3. REGISTER THE NUMBER THE INSTRUMENT READS, NOT THE ONE THE ALGORITHM SUGGESTS.** Pre-registered
"48 residual evaluations" from `ceil(log2(bracket/tol))`. Measured 50 — the two bracket-endpoint
evaluations that decide the assert are residual evaluations too. Same loop, same zero-spread claim,
wrong number to compare against.

**4. A LOCATION CLAIM MUST BE SWEPT WIDER THAN THE POINTS THAT SUGGESTED IT.** Predicted a quantity
would be least interpreter-stable AT THE DESIGN POINT, from three probe throttles. Refuted twice
over on the full grid: it ranks 21st of 38 quantities, and its worst cells sit at a throttle the
probe never sampled. Exactly [[rust-port-location-keys-refute]], re-learned.

**5. `debug_assert!` IS COMPILED OUT OF THE PROFILE THE GATE RUNS IN.** A name list justified in a
comment as keeping three arms in step was read only by a `debug_assert_eq!` under `--release`. It
guarded nothing — [[rust-port-documented-gate-that-doesnt-exist]] in the file I had just written.

**The advisor caught 2, 3-as-coverage, 5, and the ratio-vs-direction half of 1**, on a first-pass
gate that was already green at 100 %. A green bit-exact gate says nothing about whether the gate
covers the claim — [[rust-port-oracle-cannot-see-a-missing-gate]].

Also settled here, without drama: the second live site of slice I's virtual hook
([[rust-port-ladder-architecture]]) — one the phase-5 census structurally could not name, because
it enumerated method triples rather than call sites inside a class that did not exist in Rust yet.
Found by grepping the phase-6 subclass before writing the function, not after gating it.
