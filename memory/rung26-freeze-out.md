---
name: rung26-freeze-out
description: "SHIPPED rung 26 = freeze-out; anchored local Da(T,p) clock, moving freeze point, refuted rung-25's own seam framing"
metadata: 
  node_type: memory
  type: project
  originSessionId: a7dc75ec-21c3-4dad-a722-ee9e3eb923a0
---

Rung 26 (SHIPPED) = **freeze-out**: rung-25's scalar `Da` promoted to a **local**
`Da(T,p)=τ_res/τ_chem(T,p)` from an **anchored** GRI-Mech 3.0 clock (`H+OH+M`, `Ea=0`,
`n=−2`, `_K_HOHM_A=2.2e22`/`_N_HOHM=-2.0` — **zero new constants**). The relaxation shuts
off partway down the nozzle and the **freeze point MOVES with `Tt4`** (the finding):
frozen-from-entry ≤1650 K, then `s_freeze` walks 0.118→0.288→0.378 hot (real
self-quenching integrator, `L=0.5 m`).

**Refuted rung-25's own seam framing on BOTH counts** (rung-25 spec called this "the
unanchored-Arrhenius trap"): the rate is **anchored** (same mechanism `gas.py:94` cites),
and freeze-out is **density-driven** (`c_tot²∝(p/T)²`) **against** an opposing `T` effect
(`k` accelerates on cooling) — kill test fires with the OPPOSITE sign to Arrhenius. This is
the counter-example to [[mixing-scale-negative]]: here an anchored rate genuinely exists,
unlike that seam's unanchored penetration exponent.

Adds **no new bound** (lands inside rung-25's `[F,I]` — see [[rung25-finite-rate-nozzle]]);
the moving freeze point is the finding, `s_freeze`/location **disclaimed** (rides on the one
geometric knob `L`). Reduce: constant `Da_local` ⇒ rung-25 `_finite_rate_expand`
**bit-for-bit** (identity, via a `da_local_fn` callable seam).

Code (all in `turbojet/gas.py`, beside rung 25): `_tau_chem_recomb`, `_freeze_out_expand`
(rung-25 loop duplicated verbatim, scalar Da → per-step `Da_local`), `FreezeOut(L, nstep,
rate_scale)`, `FreezeOutNozzleState`, `Gas.freeze_out_nozzle`. Spec `docs/rung26-spec.md`,
anchor `docs/plans/rung26-anchor-freeze-out.md`, tests `tests/test_rung26.py` (10 gates),
`main.py` panel `print_freeze_out_nozzle_table`. Open seams: resolved `τ_res` from area
schedule (pin location), `T`-dependent freeze-out of exhaust NO, shifting turbine.
