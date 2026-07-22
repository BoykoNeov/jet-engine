---
name: rung42-interstage-bleed
description: "SHIPPED rung 42 = interstage bleed; the valve is a degree of freedom on ONE spool (LP yes, HP no); my \"penalizes HP\" hypothesis REFUTED by the probe; advisor made me probe-before-headline AND caught the confounded currency"
metadata: 
  node_type: memory
  type: project
  originSessionId: f9d11eca-fdd5-434a-a997-0f554c97a7a1
  modified: 2026-07-22T16:54:39.504Z
---

**Rung 42 (shipped 2026-07-22) = the interstage bleed valve** — `TwoSpoolBleedMatcher` in
`turbojet/engine.py`, `docs/rung42-spec.md`,
`docs/plans/rung42-anchor-interstage-bleed.md`, `tests/test_rung42.py` (12 gates, all green).
Closes the bleed-valve half of the seam rungs 36 **and** 41 both filed, on the spool
[[rung41-two-spool-surge-line]] located as exposed.

**The finding: bleed is a NEW degree of freedom on the LP spool and NOT on the HP spool.**
`x_L = Tt4/Tt2` is built from two *inputs*, so it is **exactly** bleed-invariant ⇒ all of
`Δφ_L` (+8–12%) is displacement **off** the LP running line. The HP stays on **one curve**
(`φ_H(x_H)` invariant to 0.01–0.016%, ~1000× contrast) because rung 39's `(†)` is core flow
on both sides and carries no `b` — so `_hp_eta_loop` is reused **verbatim** and that reuse
*is* the structural claim.

**Three lessons worth carrying forward:**

1. **My stated hypothesis was wrong, and the probe caught it before the spec.** I predicted
   the textbook trade — "bleed protects LP *at the HP's expense*". False: above `π*` the HP
   is *helped* too (10–100× less); below it, hurt by ~1e-4. Kept visible in spec, tests and
   panel (the [[rung40-two-shaft-transient]] convention). **Probing before writing the
   headline is now twice-vindicated** (rung 37's initial signs were both wrong too).
2. **The advisor caught a confounded currency, again the [[rung41-two-spool-surge-line]]
   lesson.** I was about to rest "self-targeting" on the *relative* `SM_L` gain growing
   (+23%→+53%) — but the **absolute** `ΔSM_L` **shrinks**; only the collapsing base makes the
   ratio grow. The defensible statement is in **φ-space**: `Δφ_L` is near-**constant** while
   `Δφ_H` collapses ×8, so the fraction of the shrinking `(φ_op−φ_surge)` gap closed rises
   17%→46% (LP) and falls 1.8%→0.25% (HP). *A denominator that is itself collapsing is not a
   controlled comparison* — this is the same trap in a new costume.
3. **Partition NEW vs INHERITED explicitly** (the rung-40 register). Most of the HP story
   *is* rung 41's closed-form `s_H` and its `π*` turn. What is genuinely new is
   **perturbation-independence**: valve-derived `s_H` == throttle-derived `s_H` to ≤0.004,
   which **could have failed** (on the real gas the HP loop reads `Tt4, Tt25, f` separately;
   only CPG at frozen `f` makes it one-parameter in `x_H`). `π*` surfaced a **third** time —
   the bleed sign-reversal brackets it at **+0.40%**, the *same* fuel-fraction residual rung
   41's own kill test isolated (+0.44%).

Reduce = **exact dispatch** ([[rung38-two-spool-matching]]'s contract, fourth use):
`bleed == 0.0` forwards `match` to rung 39's verbatim ⇒ bit-for-bit on the fast **and**
reacting gas; rung 31–41 suites pass unchanged (84/84).

**Two post-commit corrections the advisor caught (fixed in a follow-up commit) — both worth
generalizing:**

4. **A "first X" claim must be stated in the dimension that is actually new.** I shipped "the
   first shaft whose compressor and turbine pass **different air**" — **false**: `(1+f)` has
   made the LPC pass `ṁ₂` and the LPT `ṁ₂(1+f)` since rung 38. The true novelty is that mass
   **LEAVES** the flowpath (so the two *compressors* differ), not that a flow *changes* along
   it. This project holds "first X" claims to a high bar (rung 37 scoped its own carefully to
   the *transient* excess over `(1+f)`), so imprecision here reads as a lapse. **When writing
   "the first rung where X", name the exact axis and check the nearest prior thing that
   already looks like X.**
5. **When a rung breaks a deliberate methodological streak, say so in the spec.** Rungs
   38/39/40 each ship an independent bare-math CPG cascade *because* their reduce never enters
   the new code. Rung 42's reduce dispatches away too — but it ships without one. That is
   defensible (the HP side is anchored transitively; the LP magnitudes are disclaimed and its
   load-bearing claim is a shape plus an input identity) — but it was defensible **by
   accident** until written down. **Silent streak-breaks are the thing to catch.**
