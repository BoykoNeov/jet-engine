---
name: rung66-two-lag-cascade
description: "Rung 66 — two loops on one variable are ONE loop with the rates ADDED; R_q·C_g ≡ 1 is an identity, and my own pre-registered derivation had the right answer for the wrong reason"
metadata: 
  node_type: memory
  type: project
  originSessionId: ad0a599a-f9bb-41cd-b8d4-3fcd57dac948
  modified: 2026-07-30T17:48:19.049Z
---

Rung 66 (shipped 2026-07-30) — `TwoLagCascadeTransient`: rung 52's lagged FUEL leg beside rung
65's lagged bleed VALVE, both watching `φ_lp`. Four states, two clocks.

**The finding.** Two control laws holding the same variable to the same set point are implicit
functions of the *same* constraint `φ(w,b) = φ_lim`, so `R_q = φ_b/φ_w` and `C_g = φ_w/φ_b` are
**reciprocals by construction** — `R_q·C_g ≡ 1`, `det J ≡ 0`, eigenvalues exactly
`{0, −(1/τ_g + 1/τ_v)}`. The zero is rung 65's degeneracy, now provably unremovable; the other
is the two clocks, which **ADD**. So a second limiter buys BANDWIDTH, not AUTHORITY: measured,
it adds 1.59 % of protection credit where it delivers 60.46 % alone — **38× erosion**.

**Why this matters for how I work, beyond the physics:**

1. **A pre-registered derivation can be right in its conclusion and wrong in its reasoning, and
   that gap is where the rung lives.** My anchor called `R_q C_g = 1` a *locus* the two clocks
   could not move (true) and predicted the frozen state would persist (false). It is an
   IDENTITY, not a locus — and once that is seen, the observable flips: a zero eigenvalue means
   no restoring force *along* a direction, not a state that sits still. Rung 65's freeze
   belonged to the MANIFOLD (its instantaneous fuel leg pinned the state there), not to the
   mode. Score the reasoning, not just the verdict. See [[rung65-lagged-valve]].

2. **My own derived stability floor was wrong in the UNSAFE direction, and I shipped it before
   catching it.** I asserted `ds/min(τ) ≤ 2` by transferring rung 65's single-state bound. The
   rates ADD (`det J = 0` ⇒ `|λ| = 1/τ_g + 1/τ_v`), so the true bound is
   `ds·(1/τ_g + 1/τ_v) ≤ 2` — optimistic by up to 2×. The rung's own identity paid for its own
   guard. Lesson: when a rung adds a state, re-derive the RK4 floor from the coupled block; do
   not scale the previous rung's constant.

3. **Two comparison controls were invalid and a third currency was clamped — all caught by
   looking at WHERE the extremum sat.** (a) Pairing a lagged loop against an *instantaneous*
   one is not a control but a different plant, and any headline resting on it collapses to "the
   instantaneous loop holds the set point". (b) `min φ_lp` on the fuel-alone control has its
   argmin at `s = 0.0025` — it was the running line the march starts on, not a protected
   minimum. Switching to the violation integral `∫max(0, φ_lim − φ)ds` over the ramp fixed both
   the clamping and a late march truncation. **Check where an extremum sits before quoting it.**

4. **"Riding" means the LAW is active, not that the STATE is nonzero.** A lagged clip decays
   but never reaches zero, so `mf < mf_sched` is true forever after first engagement — it
   flagged 340 of 341 points. The correct test is `required > 0` (and `0 < b_cmd < b_max` for
   the valve). Sampling gains on the wrong set would have measured points where `R_q ≡ 0`.

**Scope, and it bounds the headline:** the identity needs one SET POINT, not merely one
variable. Offsetting the valve's `φ_lim` by −2.5 % moves the product to 0.951. Cascade A (a
`Tt4` governor beside the valve) therefore lies outside it — different variables, opposite
cross-gain signs, and an oscillatory actuator mode B provably cannot have. It is rung 66's
named next seam, asserted against in `integrate_fuel`.

Related: [[rung64-phi-bleed-limiter]] (a limiter's LAW cannot buy PROTECTION, only its PRICE —
rung 66 extends that from a law to a whole second limiter), [[rung62-bleed-schedule]] (the
`_b_state` boundary is the `_powers` trap; the merge preserved it, verified by rung 52's
pre-crossing bit-identity surviving as a free bug detector).
