---
name: rust-port-location-keys-refute
description: "A location key earns its cost by REFUTING the claim it was dumped to confirm; and a dimensionless group is earned by the factor that appears twice, not the one that appears once"
metadata: 
  node_type: memory
  type: project
  originSessionId: b6852aeb-8129-4b34-b69f-40824de686a5
  modified: 2026-08-12T09:29:35.390Z
---

Phase 3 slice B of the Rust port (rungs 10/11/12/20, 2026-08-12) landed at 100% bit-equality
like every slice before it — so the bit-count was not where the value was.

**A location key pays for itself when it DISAGREES.** [[rust-port-shape-keys]] established
dumping an extremum's argmax beside its value, and in slice A that key *confirmed*: the two
interpreters disagreed on a peak's value and agreed on its position. In slice B the same
technique *refuted* — rung 12's argmin moved off the optimum its spec said it pinned to "for all
S". The refutation is what was worth having. A confirming location key costs almost nothing;
budget for them precisely because the one that fires is unpredictable.

**Why the shipped test could not have found it:** rung 12's own gate only ever sampled two
values of the knob, both inside the valid band. A gate that samples where the claim is true
cannot discover where it stops being true — so when porting, sweep WIDER than the source's own
gate, not equal to it.

**Then: a dimensionless group is earned by the factor that appears TWICE.** I first offered a
sweep of one constant as evidence that `S_x = tau_res*C_e*C_opt*U_c` was the right group. The
advisor killed it: two of those four factors reach the model only through one intermediate
quantity, so they are one lever wearing two names and sweeping either proves nothing. The
discriminator is the factor that appears in the group AND somewhere else independently — if its
appearance in the group were doing double duty, the ratio would drift. It did not, across 4x.
**Before claiming a collapse, ask which factor could have exposed it as a coincidence, and sweep
that one.**

Two smaller traps, both self-inflicted and both caught within the hour:

- **An invented bar failed immediately.** I set a tolerance class to 1e-15 by analogy with a
  neighbouring class instead of reading the measured spread, and it missed on one key at
  1.36e-15. The prior slice had it at 1e-12. Read the measurement; the comparison table is
  printed right there. See [[golden-fingerprint-gate]] for the same lesson from the other side.
- **A "the guard is load-bearing" assertion failed because the guard is DORMANT.** Rung 20's
  flame-band floor never binds at the shipped design point — it clears it by 17 K. Asserting a
  guard at the design point alone can be a gate on a branch nothing takes; the fix was to add a
  second, cooler point where the clip actually fires. Same family as [[rung78-residual-gauge]]'s
  vacuity traps.

Related: [[rust-port-decided]], [[rust-port-arithmetic-is-pypy]], [[rust-port-power-spelling]]
(whose "always use the pow helper" rule INVERTS here — `math.sqrt` is the sqrt instruction and
must not go through it).
