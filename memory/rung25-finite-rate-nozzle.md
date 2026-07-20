---
name: rung25-finite-rate-nozzle
description: "Shipped rung 25 = finite-rate nozzle chemistry (Da flow between rung-14's bounds); INVERTED into a THREE-state picture — Da→∞ lands at an irreversible-fast ceiling STRICTLY BELOW the reversible bound."
metadata: 
  node_type: memory
  type: project
  originSessionId: 0cd88142-c9ee-4518-ac52-d99c40b02663
---

**Shipped rung 25 (2026-07-16) = finite-rate nozzle chemistry** — the Damköhler flow BETWEEN
rung-14's frozen and reversible-equilibrium bounds, on the exact `dh=v·dp` spine (species-vector
linear relaxation to local equilibrium; one cartoon knob `Da`). `FiniteRate(Da, nstep)` +
`Gas.finite_rate_nozzle(...)` + `FiniteRateNozzleState`; helpers `_finite_rate_expand`,
`_irreversible_fast_expand`, `_equilibrate_hp`, `_cp_molar`. Spec `docs/rung25-spec.md`, anchor
`docs/plans/rung25-anchor-finite-rate-nozzle.md`, tests `tests/test_rung25.py` (9 gates, green).

**The finding (a surprise that reframed the rung).** The plan was "Da interpolates rung-14's two
bounds." The physics REFUSED it: the nozzle is fed the frozen-in station-4 mixture, which arrives
**super-equilibrium**, so a real irreversible flow must re-equilibrate the entry **irreversibly even
at Da→∞**. So a THREE-state picture: (F) frozen = Da→0 (the EXACT reduce); (I) irreversible-fast =
Da→∞, the ATTAINABLE ceiling (closed form: const-(H,pt9) entry re-equilibration → reversible
shifting); (R) rung-14 reversible = a STRICT UNREACHABLE ceiling above (I). **Reduces to FROZEN,
deliberately NOT to equilibrium** — the (R−I) gap quantifies rung-14's own named "sliver of entry
irreversibility." Dormant lean (both gaps →0 at Tt4=1500), earns its keep hot (Tt4=2200: attainable
(I−F)≈0.43%, unreachable (R−I)≈0.031% of V9). KEYSTONE certified: the marching integrator's Da→∞
asymptote == the closed-form (I) (rate-law-independent).

**Why THIS rung 25 shipped where the OTHER "rung 25" did not** (see [[mixing-scale-negative]], the
locally-resolved-SCALE negative that once held the slot): the (R−I) gap's existence/sign are
thermodynamically ROBUST (an availability loss, relaxation-law-independent); only the magnitude and
the interior V9(Da) curve ride on the cartoon Da — no knob-dependent optimum whose location/sign
flips. Numbering was resolved (user): finite-rate IS rung 25; the SCALE negative was renamed
`docs/mixing-scale-negative.md`.

**Deferred (rung 26 seams):** freeze-out (a T-dependent Arrhenius `τ_chem(T)` — constant-Da cannot
show it; the unanchored-Arrhenius trap); a shifting turbine (delivers a less-super-eq entry, shrinks
the (R−I) gap; reopens the shaft balance).

Numerical note: the interior integrator is exponential-relaxation composition + implicit-trapezoid
`dh=v·dp` energy on a geometric pressure schedule. Forward-Euler (1st-order, too slow), RK4
(stiff-unstable at large Da), and operator splitting (mis-allocates recombination energy, plateaus
short of (I)) were all tried and REJECTED — recorded in the anchor doc so they aren't re-tried.
