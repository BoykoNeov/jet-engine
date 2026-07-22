---
name: rung40-two-shaft-transient
description: "SHIPPED rung 40 = the two-shaft transient; the LP map opens a COMPLEX inter-spool mode; rho splits (powerless over stability, decisive over oscillation); advisor caught me shipping a tautological gate AND then over-correcting into a thin negative-result rung"
metadata: 
  node_type: memory
  type: project
  originSessionId: d3d7071a-b4a7-40a7-851b-e1a05e0090ed
  modified: 2026-07-22T13:01:54.097Z
---

**Rung 40 (shipped 2026-07-22)** — `TwoSpoolTransient` in `turbojet/engine.py`, spec
`docs/rung40-spec.md`, anchor `docs/plans/rung40-anchor-two-shaft-transient.md`, gates
`tests/test_rung40.py` (8 gates / 9 tests). Builds the seam
[[rung39-two-spool-maps]] declared **well-posed** (rung 38 could supply no `N`; rung 39
supplies two). Both shaft speeds become STATES; the closure is [[rung34-spool-transient]]'s
forward move on two shafts (a 1-D root in `m_L`, no shaft balance).

**THE FINDING: `ρ = τ_L/τ_H`'s power SPLITS.** With `J(ρ)=[[a/ρ, b/ρ],[c,d]]`, stability needs
`a<0, d<0, ad>bc` — three conditions **containing no `ρ`**, so the clock ratio can *never*
destabilize the pair (measured 252 points, zero violations). But `disc=(a/ρ−d)²+4bc/ρ` vanishes
at `ρ=a/d`, so **`bc<0` ⇒ a COMPLEX inter-spool mode exists** — and `bc<0` **iff the LP
compressor map is SHAPED**, with `hp-only` (HP shaped, LP flat, no band) the discriminator. The
mode is **MAP-CREATED** — [[rung39-two-spool-maps]]'s slip pattern a third time.

**Why:** three process lessons, all about the advisor catching failure modes at *opposite* ends.

1. **It caught me about to ship a TAUTOLOGICAL gate.** I had "σ_crit predicts the marched
   crossover" measured to 0.9998 and was going to make it the non-tautological anchor. On the
   running line `Φ=0`, so `Φ(Tt4+dT)≈dT·∂Φ/∂Tt4` and the condition collapses to `ρ=σ_crit` **by
   definition** — the convergence was finite-difference self-consistency. My own anti-tautology
   bar (enforced since rung 29) would have rejected it; I didn't apply it to my own gate.
2. **Then it caught me OVER-correcting.** When the dynamic finding evaporated I was ready to
   headline "treating the two shafts as independent is EARNED" + `|Im/Re|≤0.25` as a verdict.
   That is a *negative*, and combined with inherited identities it is a doc, not a rung. The fix:
   `≤0.25` is a **DISCLAIMED MAGNITUDE** by my own rung-32/36/39 methodology (gate existence +
   sign + mechanism, disclaim the number) — the complex mode and its LP-map mechanism ARE the
   positive finding. **Caution can overclaim too**: "it doesn't ring on these representative
   maps" does not generalize any more than a magnitude does.
3. **It made me label the INHERITED parts as inherited.** Much of rung 40 is rung 39 B1/B2
   restated through a time derivative (turbine-split invariance; `σ_crit≡1`, which on the running
   line literally *reduces to* the steady slip; the two breaking channels). The spec now opens
   with an explicit inherited-vs-new table rather than dressing inheritance as discovery.

**How to apply:** apply the anti-tautology bar to my OWN gates, not just to the rung's claims —
"two computations agree" is worthless if one is the other's definition. And when a hoped-for
finding dies, don't reflexively headline the negative: check whether the *data already contains*
a positive one being disclaimed away. Probe discipline held again — five probes, and they
refuted three of my hypotheses (map-favours-LP; σ_crit governs the ramp; the pair is
non-oscillatory), same pattern as [[rung37-combustor-dynamics]] and [[rung39-two-spool-maps]].

**What rung 40 leaves open:** a two-spool surge line (rung 36's is single-spool — now with *two*
compressors and a complex mode to measure against it); the subsonic/unchoked LP branch in
transient; fuel metering ([[rung35-fuel-metering]]) on two shafts; rung 37's internal clocks on
two shafts — plus the audit of *its* shaft+metal Jacobian for complex modes, which rung 40
deliberately scoped around (claim is INTER-spool only).
