---
name: rung77-stiffness-ledger
description: "Rung 77 — a set-point solve's sensitivity is a FORCING OVER A SLOPE, so 1/(1−c) is the slope half of ONE leg; the units problem was the discriminator, and a closure that outlived its state block returned a perfect-looking 1.000e+00"
metadata: 
  node_type: memory
  type: project
  originSessionId: ef40f100-c153-42c3-8dbe-2ac9e55deed8
  modified: 2026-08-09T21:08:25.139Z
---

Rung 77 (shipped 2026-08-09/10) reads the residual slope `G_w` of all three set-point solves
this family runs — rung 48's accel leg, rung 46's governor, rung 49's φ leg — and **refutes rung
76 § 8's wording**: the other two legs do not have a `1/(1−c)` and structurally cannot.

**Why:** `dw*/dq = −G_q/G_w` for any `G(w,q)=0`. `1/(1−c)` is just `1/G_w` for the ONE leg whose
set point is a formula for its own actuator (`G_a = w − cap(w)`). `Tt4_max` and `φ_lim` are
CONSTANTS, so those residuals have no `1` to subtract from — a stiffness, never a gain. Rung 76's
own sentence *a floor on a STATE is not a formula for a FUEL* predicts it, read one step further.

**How to apply — the four transferable lessons:**

1. **The units check was the discriminator, and I nearly shipped without it.** The three `G_w`
   carry different units (—, K·s/kg, φ·s/kg), so a raw "stiffness ordering" is not a legal
   comparison. The fix was not a normalisation but a *change of currency*: `dw*/dq` is kg/s per
   valve unit for all three legs and needs no normalisation at all. **When comparing quantities
   across sub-systems, check the units BEFORE choosing a normalisation — if a normalisation is
   needed, the comparison is probably in the wrong currency.** The normalised table survived only
   because it reproduces rung 76 § 3's gain digit for digit on its own column.
2. **A closure that outlives its state block returns a perfect-looking number.** `G_q` came back
   `0.0` because the residual closures were built inside a `_b_state = q±dq` block and evaluated
   after the `finally` restored it — so both readings ran on the same closed-valve plant. The tell
   was a relative error of **exactly `1.000e+00`, with no noise in it**. Fixed structurally
   (`_residuals` exists so a closure cannot outlive its block), not by moving a line. Same family
   as rung 62's `_powers` trap. See [[rung62-bleed-schedule]].
3. **Gate above the differencing floor, not at the measurement.** The anchor's `< 3e−9` for the
   IFT check was optimistic; measured `7.15e−9`. Sweeping `dq` over three decades gave a textbook
   central-difference V (`9.6e−8 / 7.15e−9 / 1.05e−8 / 3.87e−7`), which *proves* the residual is
   arithmetic rather than a gap. Same for P6: its `1e−7` held but sits only ~5× above its own
   roundoff floor, so the shipped gate is `1e−6`. Rung 76's P2 lesson applied forward instead of
   scored afterwards. See [[rung76-fuel-dependent-cap]].
4. **The pre-registered invariance was REFUTED raw and HELD guarded — and finding out why
   recovered rung 76 § 1.3's guard where nothing had asked for it.** 3 of 24 cells inverted, all
   at `margin = 0.40`; a margin scan showed the governor's and φ leg's gains *freeze identically*
   above `margin ≈ 0.15`, because the accel leg has gone **dormant**. The raw ledger was ordering
   a leg that is not acting against two that are. Second time in two rungs that this family's
   `min`-select produced that comparison.

**Also:** the trap did NOT bite for the second time in sixteen rungs, but only because it was
looked for in a new place — all five knobs travel through `at_lever`; what would have leaked is
the **CLASS** (`_shared_rig` builds through `at_lever`, which names its class literally).

Two results worth keeping: rung 64's degeneracy, which that rung marked *"DERIVED, not measured"*,
is now **measured** (`G_s′` `9.97` open → `1.7e−08` closed; `φ_lp` bit-identical over ±10 % fuel),
and the valve's sign **SPLITS** — it tightens both fuel-side caps and loosens the φ leg, which is
rung 61's *buys the COORDINATE, not the BILL* with a sign on it. `c ≤ 0.2234` over 24 cells
**BOUNDS** rung 76's `c → 1` seam before it is built. See [[rung64-phi-bleed-limiter]].
