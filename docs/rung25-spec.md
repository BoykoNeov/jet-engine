# Rung 25 — Finite-rate nozzle chemistry: the Damköhler flow BETWEEN rung-14's bounds

Rung 14 bracketed the production (frozen) nozzle against a shifting-**equilibrium** expansion and
named the seam it left open: *"No finite-rate nozzle chemistry (the real Damköhler-number flow
between the bounds) … a transported/finite-rate nozzle is a further seam."* **Rung 25 builds it** —
and the build **inverts the seam's own framing.**

**THE HEADLINE IS A THREE-STATE PICTURE, NOT A TWO-BOUND INTERPOLATION.** The plan was "one Damköhler
knob `Da` slides `V9` from the frozen bound (`Da→0`) to the equilibrium bound (`Da→∞`)." **The physics
refused the second half.** The nozzle is fed the **frozen-in station-4 mixture**, which arrives
**super-equilibrium** — equilibrated at the hotter `Tt4`, frozen through the turbine to `Tt9 < Tt4`,
so it carries *more* dissociation than local equilibrium at the entry. A **real (irreversible) flow
must re-equilibrate that entry**, and that relaxation is **irreversible even at infinite rate**. So:

```
  (F) FROZEN              — Da→0.  rung-14 lower bound.       THE REDUCE (exact).
  (I) IRREVERSIBLE-FAST   — Da→∞.  the NEW ATTAINABLE ceiling. finite-rate's true endpoint.
  (R) REVERSIBLE-SHIFT    —        rung-14 upper bound.        a STRICT, UNREACHABLE ceiling above (I).
```

**Rung 25 reduces to rung-14 FROZEN exactly, and DELIBERATELY does NOT reduce to rung-14
equilibrium** — the non-reduction is the finding. The gap **(R−I)** is the *availability lost*
re-equilibrating the super-equilibrium entry — the **"sliver of entry irreversibility"** the rung-14
spec named and set aside (§ "the load-bearing modeling call"). Rung 25 **quantifies it**: a genuine
sliver lean (0.001% at `Tt4=1500`), a real ~7%-of-bracket ceiling hot (`Tt4=2200`). **Even infinitely
fast chemistry cannot reach the reversible bound** — it is a ceiling no real nozzle attains.

> **Read `docs/rung14-spec.md` first** (the frozen↔equilibrium bracket, `_expand_nozzle`, the
> common-physical-entry choice, `_mix_entropy_molar`/`_mix_h_abs_B`/`_mix_mass_per_air`) and
> `docs/plans/rung25-anchor-finite-rate-nozzle.md` (the verified numbers). This file states only what
> *changes*. No new chemistry, no new species — only rate-limited bookkeeping over the rung-6/14
> equilibrium machinery. A **pure diagnostic beside the cycle**: the production nozzle stays frozen,
> so the cycle is **bit-for-bit rung 6.**

---

## The spine — `dh = v·dp`, and why it reduces at (F) for the right reason

The one relation that holds for **any** adiabatic frictionless flow, reversible or not (energy +
momentum: `dh_t = 0` and `V dV = −dp/ρ` combine to `dh = v·dp`), is the integration backbone.
Written per mol dry air (mass `m` is recombination-invariant, atoms conserved):

```
  dH = (n_tot·Ru·T / p)·dp                                          [energy+momentum, ANY chemistry]
  dn_i/ds = Da·(n_{i,eq}(T,p) − n_i)                                [species-vector linear relaxation]
  V9 = √( 2·(H_entry − H_exit)/m )                                  [total-enthalpy conservation]
```

The temperature update carries **both** terms — the pressure work cools, the recombination reheats:
`dT = [ (n_tot·Ru·T/p)·dp − Σ_i h_i·dn_i ] / Σ_i n_i·cp_i`. Dropping the `Σ h_i dn_i` term would
silently give the frozen temperature at every `Da`. **The shipped code realizes this IMPLICITLY** —
it does not evaluate this explicit `dT`; it bisects `T1` on the integrated energy balance
`H_abs(comp1,T1) − H_abs(comp0,T0) = ½(v0+v1)·dp` (the trapezoidal `dh=v·dp`), whose `Σ n_i cp_i`
and `Σ h_i dn_i` are both inside `H_abs`. The implicit form is 2nd-order and stable where the
explicit Euler step is not (the frozen-limit convergence, gate 2, is the proof).

**Why the limits reduce (thermodynamics, not luck):**
- **(F) `Da→0`, fixed composition:** `T ds = dh − v dp = 0` — **isentropic**, matches rung-14 frozen.
- **`Da→∞`, local equilibrium:** the Gibbs relation `dh = T ds + v dp + Σμ_i dn_i` has `Σμ_i dn_i = 0`
  at equilibrium, so again `ds = 0` — **but only from an ALREADY-equilibrated entry.** From the
  super-equilibrium frozen entry, the flow first relaxes an out-of-equilibrium state (`Σμ_i dn_i < 0`
  ⇒ `T ds > 0`), and **that entry entropy is produced no matter how fast** — the (I)≠(R) gap.
- **Finite `Da`:** `Σμ_i dn_i < 0` throughout ⇒ `T ds > 0` — the irreversibility appears
  **automatically** through the composition path. Isentropy is never imposed; it falls out at the
  reversible limits and fails, correctly, in between.

**Species-vector, not a scalar progress variable** (advisor call): relaxing the whole vector conserves
every element **exactly** — each atom count `A_e = Σ ν_{e,i} n_i` is linear and shared by `n` and
`n_eq`, so `dA_e/ds = Da·(A_{e,eq} − A_e) = 0`. A free conservation assert (measured `|Δ|=1.7e-16`).

## (I) The irreversible-fast ceiling — a rate-law-INDEPENDENT closed form

The `Da→∞` limit is not marched (the marching integrator is stiff-unstable as `relax→1`, and would
in any case only *approach* it). It is computed **directly**, and it is the **keystone match** that
certifies the whole picture:

```
  1. equilibrate the frozen entry at constant (H, pt9):  find (comp*, T*) with comp*=eq(T*,pt9),
     H_abs(comp*,T*) = H_abs(comp_entry, Tt9).            [adiabatic + isobaric ⇒ enthalpy conserved;
                                                           recombination reheats, T* > Tt9]
  2. reversible shifting from (comp*, T*, pt9) to p9:     rung-14 _expand_nozzle(comp*, T*, …, shifting=True).
```

This is **independent of the relaxation law** — it is a pure thermodynamic state calculation — which
is *why* the (R−I) gap's **existence and sign are robust** while its magnitude and the interior curve
are model-dependent. **The keystone:** the marching integrator, pushed to large `Da`, converges to
this closed form (`Tt4=2200`: `Da=1000` gives `V9=1412.50` vs closed-form `1412.58`, residual
truncation). That match is the proof that the ~7%-short asymptote is **physics, not a numerical
failure to reach `R`.**

## What rung 25 adds (and what it deliberately does not)

**Adds** (all in `turbojet/gas.py`, all *decoupled* from the cycle):

- `FiniteRate(Da, nstep)` — the config: the Damköhler knob `Da` and the marching resolution.
- `_finite_rate_expand(comp_entry, far, Tt9, pt9, p9, Da, nstep)` — the interior integrator: exact
  exponential composition relaxation + implicit-trapezoid `dh=v·dp` energy on the geometric pressure
  schedule `p(s)=pt9·(p9/pt9)^s`. Returns `(T9, V9, comp9, dS)`.
- `_equilibrate_hp(far, H_target, p, …)` + `_irreversible_fast_expand(…)` — the closed-form (I).
- `FiniteRateNozzleState` + `Gas.finite_rate_nozzle(far, Tt4, pt4, Tt9, pt9, p9, finite_rate)` — the
  public diagnostic. Composes rung-14's `_expand_nozzle` for (F) and (R) and the above for (I) and
  the interior `V9(Da)`. **Only reads** the handed-in state; touches no cycle path.

**Does NOT add / deliberately out of scope:**

- **`nozzle_flow` is literally untouched** — rung 14's method and tests are bit-for-bit unchanged
  (rung 25 is a *new* method beside it, per the one-diagnostic-per-rung pattern).
- **No `Da` dispatch to (R).** `Da=∞` returns the closed-form **(I)**, not rung-14's reversible bound
  — dispatching to (R) would paste a discontinuity onto the integrator's own curve.
- **No anchored chemical time / no freeze-out.** `Da` is a **normalized-schedule cartoon** (a single
  constant Damköhler), NOT an Arrhenius `τ_chem(T)`. A constant `Da` interpolates the bracket but
  **cannot show freeze-out** — the point where `τ_chem` overtakes `τ_flow` as the gas cools and
  recombination *stops*. That is the pedagogical heart of nozzle non-equilibrium and the **honest
  next seam**; a `T`-dependent `τ_chem` would capture it but reintroduces an unanchored Arrhenius
  constant (the exact trap `docs/mixing-scale-negative.md` recorded). **Ship constant-`Da`, defer
  freezing.**
- **No shifting turbine** — the entry irreversibility is a *direct consequence* of the **frozen
  turbine** (it delivers the super-equilibrium entry). The deferred shifting-turbine seam would
  partially equilibrate the gas upstream and **shrink the (R−I) gap**; rung 25 sharpens that seam.

## The reduce contract — CONVERGENT at (F), NON-reduction at (R), stated loudly

The same discipline rung 14 used for "convergent, not bit-for-bit":

- **`finite_rate=None` ⇒ rung-14 untouched** (trivially exact — a separate method).
- **`Da=0` ⇒ (F) is the DISPATCHED rung-14 frozen value** (exact). The integrator is not run at 0.
- **The integrator `Da→0` CONVERGES to (F)** at 2nd order in `1/nstep` (`|ΔV9|`: 3.0e-2 → 2.2e-3 →
  4.3e-4 at nstep 100→400→1600) — a convergent reduce with a stated tolerance, NOT identity.
- **`Da→∞` ⇒ (I)**, the closed form — and the rung **does NOT reduce to (R)**, on purpose. The (R−I)
  gap is the finding, not a bug to be closed.

**Certify the robust, not the number:** the (R−I) gap's **existence and sign** are thermodynamically
robust (an availability loss, relaxation-law-independent). Its **magnitude** (0.031% at `Tt4=2200`)
and the interior `V9(Da)` shape ride on the model (relaxation closure + frozen-entry choice) — **not
certified.**

## Verification gates (priority order)

1. **REDUCE — `finite_rate=None` untouched** (LOAD-BEARING): `nozzle_flow` and the whole rung-1..24
   suite are bit-for-bit unchanged. The frozen dispatch `V9_frozen` equals rung-14's exactly.
2. **REDUCE — integrator `Da→0` → (F)**, convergent 2nd-order in `1/nstep` (§ above; `<3e-3` at
   nstep=400).
3. **THE KEYSTONE — integrator at large `Da` (composition pinned) asymptotes to closed-form (I)**
   (`<0.15 m/s` at `Tt4=2200`, `nstep≥1200`; residual is trapezoid truncation). This certifies (I)
   as the true finite-rate endpoint.
4. **THE THREE-STATE ORDERING** — `V9_F ≤ V9_I ≤ V9_R` with **both** gaps `> 0` hot, and the interior
   `V9(Da)` **monotone-increasing**, strictly inside `[V9_F, V9_I]`.
5. **DORMANT LEAN, EARNS ITS KEEP HOT** — both gaps → 0 at `Tt4=1500` (`<0.01%`) and grow monotonically
   with `Tt4` (rung-14's arc; `(I−F)` ≈ 0.43%, `(R−I)` ≈ 0.031% at `Tt4=2200`).
6. **2nd LAW** — `dS ≥ 0` for all `Da`, → 0 as `Da→0`, peaking at intermediate `Da`.
7. **ATOM CONSERVATION** — the vector relaxation conserves C, H, O exactly (`|Δ| < 1e-12`).
8. **CYCLE UNTOUCHED** — a `finite_rate_nozzle` call does not perturb station 4 or the cycle `V9`
   (pure diagnostic; NO/dissociation never enter the cycle solve).
9. **GUARDS** — requires the equilibrium (rung-6) gas; rejects `p9 > pt9`; the exit-`T` bisection and
   the const-`(H,p)` entry equilibration guard against the cold `_equilibrium_composition` floor.

## Done when

`Gas.finite_rate_nozzle` returns the three-state ceiling `[V9_F, V9_I, V9_R]` (dormant lean, ~7%
unreachable-gap hot) plus the interior `V9(Da)` curve; `main.py` prints the rung-25 panel (the
three-state picture + a `Da` sweep filling the attainable bracket at `Tt4=2200`); `tests/test_rung25.py`
is green on gates 1–9; the whole prior suite is untouched.

## The rung-26+ seam (keep it additive)

- **Freeze-out** — a `T`-dependent chemical time `τ_chem(T)` (Arrhenius) resolving *where* the
  recombination quenches on the cooling path. Needs an anchored rate constant (the deferred trap).
- **A shifting turbine** — extend the equilibrium expansion upstream of the nozzle; shrinks the (R−I)
  entry-irreversibility gap by delivering a less-super-equilibrium entry (reopens the shaft balance).
- **Finite-rate exhaust NO** — the clamp corollary (rungs 14/17) is still an equilibrium-O lower bound
  frozen through the nozzle; a real NO relaxation would ride this same `dh=v·dp` chain.
