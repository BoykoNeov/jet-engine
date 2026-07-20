---
name: turbine-march-negative
description: "Rung 29's finite-rate-turbine-march seam INVESTIGATED, NEGATIVE, NOT a rung; I_turb≡S because turbine entry is at equilibrium (rung-25 dodge cannot repeat)"
metadata: 
  node_type: memory
  type: project
  originSessionId: 67d28380-f468-4afb-b06d-d260d0de4d06
  modified: 2026-07-20T18:12:55.972Z
---

The **finite-rate turbine march** (rung 29's open item (a)) was investigated and is a
**NEGATIVE result — NOT a rung**. Durable record: `docs/turbine-march-negative.md`
(tracked). Same family as [[tau-res-negative]] and [[mixing-scale-negative]].

**The structural finding (deeper than the un-anchored τ_res rung 29 cited):** the rung-25
dodge does NOT transfer. Rung 25 ([[rung25-finite-rate-nozzle]]) became a rung because its
`Da→∞` irreversible-fast ceiling (I) is a rate-independent closed form *strictly below* the
reversible bound — the super-eq nozzle entry relaxes irreversibly even at infinite rate. The
turbine entry is station 4 (fresh burner exit), **at equilibrium by construction**
(`comp_entry = _equilibrium_composition(far,Tt4,pt4)`, gas.py:4830), so `Da→∞` stays on the
equilibrium manifold and lands *exactly* on rung 29's `S` — **`I_turb ≡ S`, no third state,
no (R−I) gap**. Verified: `dS(I_turb)=dS(S)`=machine-zero (~1e-12) at every Tt4 vs the nozzle
I's physical `+4.3e-4→+4.1e-2`, all from EXACT shipped closed forms.

**Two un-anchored knobs** (advisor sharpened this): (1) `τ_res` (Da_turb=0.05–8.8), and
(2) — worse than the nozzle — an **ambiguous progress coordinate** (work-limited ⇒ p5 unknown
⇒ no natural schedule to march on). Only the endpoints (F, S) are certifiable, and they are
already rung 29.

**Positive by-products:** rung 25's (R−I) gap is **manufactured by the freeze, not intrinsic
to expansion** (localized to the one turbine→nozzle handoff); rung 29's `S` is **attainable**
(genuine `Da→∞` limit), not an unreachable ceiling like rung 25's R; design-point-robust
corollary — since F≈S at design, the whole [F,S] band is negligible regardless of either
knob, so the march **cannot overturn "freeze earned at design"** (rung 29's statement).

**Method notes:** probes in `M:\claud_projects\temp\rung29-turbine-march\` (outside git).
Hand-rolled turbine march has a SPURIOUS dS floor ~1.1e-2 (one-step composition-lag artifact);
the doc's entropy contrast uses exact closed forms only, never the march dS. No shipped code
changed. **Do NOT re-run** the geometric-p scalar-Da march with hand-picked p_floor — a real
attempt needs turbine passage A(x) geometry + the choked-flow seam.
