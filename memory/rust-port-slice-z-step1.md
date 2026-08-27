---
name: rust-port-slice-z-step1
description: "Slice Z step 1 — a shipped refusal that cannot see the carrier the rung below installs, and a count bar that had already been WRITTEN DOWN once, which reads like a measurement"
metadata: 
  node_type: memory
  type: project
  originSessionId: fb6a156a-cddd-48e3-ad03-77bfd1953bac
  modified: 2026-08-27T07:56:15.816Z
---

Rungs 66/67's plumbing: `MarchScope` grows `lag` + `tau_gov`, two restore-previous carriers,
the same three cell swaps twice (`two_lag.rs`, `cross_loop.rs`), `_RINGS`. Both marches stubbed.
`slice_z_smoke.rs` 9 green; whole Rust suite exit 0.

**A TYPED COUNT THAT SURVIVES A SLICE READS LIKE A MEASUREMENT.** [[rust-port-guessed-census-bars]]
is five bars typed wrong; this is the harder version. P1 said "all **55** shipped call sites stay
as they are" — a number slice Y typed into `MarchScope`'s doc comment, which P1 then quoted, which
I was about to quote a third time. The advisor asked whether I had *run* it. Measured: **82**
un-scoped and **16** scoped. The verdict was unchanged, so nothing would ever have caught it.
**When a prediction quotes a count, re-run the count — inheriting one across a slice launders a
guess into a fact.**

**A MARCH SCOPE CONSUMES ITS OWN FIELD AND DROPS THE RUNGS ABOVE IT, SO A REFUSAL CAN BE BLIND TO
THE CARRIER THE RUNG BELOW INSTALLS.** Rung 67's `assert lag is None` reads the ARGUMENT; the fuel
lag arrives on rung 66's *carrier*; and rung 67's armed branch returns before `super()`. So on a
rung-67 machine, arming the lag through `_stator_march` is **silently discarded** — measured
(`probe_z10.py`): no raise, `self._lag` holds the lag inside the armed branch, trajectory
bit-identical 171/171 to one with no lag. The same probe one rung down returns `False`, which is
what makes the zero the grid's and not the instrument's ([[rust-port-slice-w-step3]]). Found by
writing gates: the first draft armed the refusals through the march and could not reach one.

**A ONE-TOKEN DIFFERENCE NO VALUE KEY CAN SEE NEEDS AN OWNER, NOT A PARAGRAPH.** Reproducing that
discard is `lim.lag` vs `ft.inner.lag.get()`. Booked as **P11** — a step-5 manufactured gate —
rather than left as an observation, because an observation with no owner is how it gets lost.

**GROWTH IS FREE FROM THE SECOND TIME ON.** Adding a field to a `Copy` struct is a compile error
at every EXHAUSTIVE literal — nine here, **eight of them in test files**, invisible to a `src/`
grep. `..MarchScope::DEFAULT` absorbs slice AA's next two for free. P1 falsified at its letter,
held at its intent, booked as a verdict rather than patched quietly.

**AND LANDING THE DISPATCH LIVE WITH THE MARCH STUBBED TURNED STEP 1 INTO A GATE.** Reduce arms,
refusals and both carriers are provable before a line of march exists. Cost, booked forward: four
of the nine gates assert on the stub's panic string and step 2 must REWRITE them.

**Why:** three of the four things worth knowing came from being asked what I had actually run —
the count, the body read, the alias's guarantee.

**How to apply:** re-run any count a prediction quotes, even one already written down. Ask of a
`try/finally`-carried parameter which frame READS it, then measure. And when a doc comment claims
a guarantee (`R66_TWO`'s "makes the next addition loud"), check the code provides it — the alias
did not, and the faithful fix was to correct the comment, since propagating IS what inheritance
does.

Related: [[rust-port-slice-z-preflight]], [[rust-port-slice-y-step4]], [[rust-port-copy-vs-rederivation]],
[[rust-port-measure-before-registering]], [[rust-port-slice-v-step4]].
