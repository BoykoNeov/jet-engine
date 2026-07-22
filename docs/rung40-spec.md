# Rung 40 — The two-shaft transient: the LP map opens a COMPLEX mode

**Status:** shipped. `TwoSpoolTransient` in `turbojet/engine.py`, gates in
`tests/test_rung40.py`, anchor data in `docs/plans/rung40-anchor-two-shaft-transient.md`.

Rung 39 closed by naming this seam and calling it *newly well-posed*: "the two-shaft
transient (now well-posed — rung 38 could supply no `N` at all; rung 39 supplies two)."
Rung 40 builds it.

---

## Honest accounting up front: what is inherited and what is new

A large fraction of this rung is **rung 39 restated through a time derivative**, and the
spec says so rather than dressing it as discovery:

| Result | Status |
|---|---|
| The turbine work split `Pt_L/Pt_H` is `Tt4`-invariant to ~1e-15 on CPG | **INHERITED** — rung 39 B1's `(1+f)`/`Tt4` cancellation |
| `sigma_crit == 1` on flat maps + CPG | **INHERITED** — reduces to rung 39's slip, which B1 pins at 1 |
| Two channels break it (`cp(T)` curve, the map), map dominant | **INHERITED shape** — rung 39 B2 |
| The map's shift *direction* is shape-dependent | rung-32/39 methodology |
| The 2-D well-posed transient itself (states, closure, reduce) | **NEW** — the deliverable rung 39 promised |
| **The clock ratio `rho` cannot destabilize the pair** | **NEW** (the sign conditions are `rho`-free) |
| **A COMPLEX inter-spool mode exists, and the LP map creates it** | **NEW — the positive finding** |
| `sigma_crit`'s authority is first-instant only | **NEW, and a negative** (scope limit) |

---

## The structure

Two states — both shaft speeds — under two shaft-inertia ODEs:

```
I_L * w_L * dw_L/dt = eta_m*P_LPT - P_LPC        I_H * w_H * dw_H/dt = eta_m*P_HPT - P_HPC
```

Nondimensionalize on the HP spool's clock `tau_H = I_H*w_H,d^2/P_ref,H`, `s = t/tau_H`:

```
    dnu_H/ds = Phi_H                dnu_L/ds = Phi_L / rho ,      rho = tau_L / tau_H
```

so exactly **one** clock parameter survives — and it is a **RATIO**, not a scale.

**This is the resolution of rung 34's own tautology.** Rung 34 found that `I` merely set the
clock and that a **second** clock (`tau_fuel`) had to be *imposed* before inertia became
load-bearing. A two-shaft engine has that second clock built in: **each spool is the other's
clock.** But "`rho` survives nondimensionalization" is itself dimensional analysis — the same
vacuous statement rung 34 rejected. What follows below is the non-vacuous part: *what `rho`
can and cannot do*, which turns out to be a clean split.

### The closure (rung 34's move, on two shafts)

Given `(nu_L, nu_H, Tt4)`, close the flow with **NO shaft balance** — that residual is the
whole point. It is a **1-D root in the LP corrected flow `m_L`**, because the chain is causal:

```
  m_L -> [LPC map FORWARD] tau_LPC, pi_LPC, Tt25 -> n_H = nu_H*sqrt(Tt25_d/Tt25)
      -> m_H = m_L*(mcorr_lp_d/mcorr_hp_d)*sqrt(Tt25/Tt2)/pi_LPC
      -> [HPC map FORWARD] tau_HPC, pi_HPC, Tt3 -> pt4 -> f -> HPT-NGV choke imposes mdot
```

Both turbines then follow from rung 38's `(*)` geometry alone (`A4/A45` and `A45/A8`), so the
two power residuals are **OUTPUTS**, not constraints.

**Rung 39's triangular cascade does not arise here at all.** The transient closure reads
`eta_LPC`/`eta_HPC` **forward** off each map at the trial point — a direct read, no secant. The
one-way HP->LP arrow and the closed leaf were artifacts of solving the *steady* problem with
`eta` unknown; the rung does not build on them.

---

## `sigma_crit` — the lead threshold (the transient<->steady link)

Which spool **leads** an acceleration is decided by whose **fractional** speed rate is larger,
so the threshold is a ratio of speed-normalized sensitivities, on the running line:

```
  (dagger)   sigma_crit = [ (dPhi_L/dTt4)/nu_L ] / [ (dPhi_H/dTt4)/nu_H ]
             HP leads  <=>  rho > sigma_crit
```

**The identity (INHERITED).** On the running line `Phi=0`, so `Pt = Pc` on each shaft and the
`dmdot` term drops out of `dPhi/dTt4`. With **flat** maps `psi==1`, so `tau_c` depends on `n`
alone and both compressor *specific* works are frozen under a `Tt4` step; on **CPG** both
turbine works carry the same `(1+f)*cp_t*Tt4*[geometry]` factor. What survives is

```
  sigma_crit = (Pc_L/P_ref_L)/(Pc_H/P_ref_H) * (nu_H/nu_L) = (nu_L^2/nu_H^2)*(nu_H/nu_L) = slip
```

and **rung 39 B1 pins `slip == 1` exactly** on CPG + flat maps. So `sigma_crit == 1` is rung
39's *steady* identity restated for the *transient* — a derived inheritance, not a new
coincidence, and it is this rung's **reduce spine, not its finding**. Measured: `|sigma_crit-1|`
≤ 9e-14 at every throttle.

**The two channels (INHERITED shape — rung 39 B2).** Measured identically at `Tt4=1100`:

| configuration | `sigma_crit - 1` |
|---|---|
| CPG + flat maps | `-4.1e-15` (the identity) |
| `thermally_perfect` + flat maps | `+4.31e-2` (the `cp(T)` curve alone) |
| `reacting_equilibrium` + flat maps | `+5.12e-2` |
| CPG + shaped maps | `+2.49e-1` (the map alone) |

The map channel is ~5.8x the gas channel — dominant but **not sole**, exactly rung 39 B2's
finding, and the rung-31-gate-5 mirror once more.

**The direction of the map's shift is SHAPE-DEPENDENT — a refuted hypothesis, kept visible.**
"The map favours the LP spool" is **false**: shaping the LP map alone drives `sigma_crit` *below*
1 (0.73–0.95), shaping the HP map alone drives it *above* (1.22–1.28). Both signs are reachable;
only the *existence* of a material shift is claimed.

### Scope limit (a NEGATIVE, stated plainly)

**`sigma_crit`'s authority is FIRST-INSTANT only.** Two candidate dynamic claims were probed
and both are withdrawn:

- *"`sigma_crit` predicts the marched crossover"* — **tautological**. From the running line
  `Phi=0`, so `Phi(Tt4+dT) ~ dT * dPhi/dTt4` and the crossover condition collapses to
  `rho = sigma_crit` **by definition**.
- *"`sigma_crit` is the amplitude->0 limit of the marched threshold `rho*`"* — **refuted by
  measurement**: `rho*/sigma_crit` converges to ~0.60 (flow/press) and ~1.40 (hp-only), not 1.

The reason is structural: referenced to the running line, the finite-ramp slip excursion
`slip - slip_ss(Tt4)` is dominated by **`slip_ss` moving with `Tt4`** while the speeds lag, so
it is **schedule-slaved**, not lead-governed (it is negative at the first step for *every*
`rho`, including `0.70*sigma_crit`). Both routes to finite-amplitude authority fail — one by
collapsing into a definition, one on the data. `sigma_crit` is a legitimate computed object and
the transient<->steady link; it is **not** a predictor of the observable ramp excursion.

---

## THE FINDING: `rho` cannot destabilize the pair — but the LP map opens a COMPLEX mode

Write the Jacobian at `rho=1` as `(a,b,c,d) = d(Phi_L,Phi_H)/d(nu_L,nu_H)`. At clock ratio
`rho` the LP row carries `1/rho`:

```
    J(rho) = [[a/rho, b/rho],
              [c,     d    ]]

    tr   = a/rho + d          det  = (a*d - b*c)/rho
    disc = tr^2 - 4*det = (a/rho - d)^2 + 4*b*c/rho
```

### (i) STABILITY is `rho`-free

`tr<0` and `det>0` hold for **every** `rho>0` as soon as `a<0`, `d<0` and `a*d>b*c` — three
conditions that **contain no `rho`**.

**Precision (per the advisor):** those three signs are **MEASURED**, not derived — 252
`(shape, Tt4, rho, gas)` points, 7 shape pairs x 3 throttles x `rho` over `[0.05, 100]`
(a 2000x range) x 2 gases, **zero violations**, worst eigenvalue real part `-0.011`. What is
**derived** is that, *given* those signs, `rho` cannot destabilize the pair. The gate asserts
the empirical part; the `rho`-freeness is presented as algebra on top of it.

### (ii) OSCILLATION is NOT `rho`-free — and the LP map creates it

`disc` *does* carry `rho`. The term `(a/rho - d)^2` vanishes at `rho = a/d` (positive, both
being negative), leaving `disc = 4*b*c/rho` there. Hence, exactly:

- `b*c < 0`  =>  a **complex pair exists**, in a band around `rho = a/d`;
- `b*c >= 0` =>  the approach is **monotone at every `rho`**.

**The mechanism, measured and sign-robust: `b*c < 0` iff the LP compressor map is SHAPED.**

| shape pair | `b` | `b*c` | complex band? |
|---|---|---|---|
| flat / flat | `-0.011` | `+1e-3` | none |
| **hp-only** (HP shaped, **LP flat**) | `-0.006` | `+3e-4` | **none** |
| flow/press | `+0.574` | `-2.7e-2` | yes |
| press/flow | `+0.800` | `-3.1e-2` | yes |
| tilted | `+0.676` | `-3.0e-2` | yes |
| steep | `+0.815` | `-4.2e-2` | yes |
| lp-only (LP shaped, HP flat) | `+0.843` | `-2.9e-3` | yes |

**`hp-only` is the discriminator**: the HP map is shaped there and no band appears, so it is the
**LP** map specifically — not shaping in general. Shaping the LP map **flips the sign of
`b = dPhi_L/dnu_H`** from small-negative to large-positive: with a flat LP map, speeding the HP
spool slightly *hurts* the LP power balance; with a shaped one it strongly *helps*. Combined
with `c = dPhi_H/dnu_L < 0` (always), that antisymmetric cross-coupling is what makes the pair
complex. The mode is **MAP-CREATED** — the same pattern as rung 39's slip, a third instance.

Verified on the shipped solver: flow/press at `Tt4=1200` predicts a band `rho in [1.233, 2.082]`
centred on `a/d = 1.602`, and the Jacobian is complex inside it and real outside; `hp-only`
grazes `disc = +7e-4` at `rho = a/d` and never crosses.

### The magnitude is DISCLAIMED

At `rho = a/d` the mode's strength has a closed form:

```
    |Im/Re|_max = sqrt( -b*c / (a*d) )
```

which in the sampled maps is **<= 0.25** (0.13 flow/press, 0.20 press/flow, 0.25 steep) — under
a quarter cycle before e-folding, so the marched trajectory shows no visible ringing here.

**That number is a disclaimed magnitude, not a verdict.** Per the rung-32/36/39 methodology this
rung gates **existence + sign + mechanism** and discloses the magnitude without claiming it —
exactly as rung 39 shipped its 1.5–5% slip. "It does not ring *on these representative maps*"
does not generalize, and real twin-spools do exhibit inter-spool coupling oscillation. The
honest statement is *disclaimed*, not *negligible*.

### Novelty scope: INTER-SPOOL

A complex mode requires >= 2 states, so rung 34's scalar shaft ODE structurally cannot
oscillate. **But rung 37 already runs multi-state single-spool dynamics** (shaft + metal at
`tau_soak ~ tau_spool`) and its Jacobian was **not** audited here. The claim is therefore scoped
to the **inter-spool** mode — *not* "the first oscillatory mode in the project."

### The `rho` split (the rung-34 inversion, precisely delimited)

Rung 34: `I` is only a clock. Rung 40: the clock **ratio** is
**powerless over stability** (the sign conditions are `rho`-free) yet **decisive over the
MANNER of approach** (whether the mode is real or complex) and over **which spool leads**
(`sigma_crit`). That delimitation — not "`rho` matters" — is the content.

---

## Reduce-to-prior contract

1. **The 2-D equilibrium -> rung 39.** `equilibrium(flight, Tt4)` solves
   `Phi_L = Phi_H = 0` in `(nu_L, nu_H)` by damped Newton from the design start, and reproduces
   `TwoSpoolMapMatcher.match` at the same point — `nu_L`, `nu_H`, `pi_lpc`, `pi_hpc`, `mdot`
   to `<= 1e-12` on CPG **and** the reacting gas. Through the **forward closure only**: it never
   calls the matcher, so the reduce is non-circular (rung 34's discipline). Rung 34's was a 1-D
   bracket; this is a genuine 2-D root.
2. **`lp_disabled=True` -> rung 34, EXACT DISPATCH.** No two-shaft state is built at all:
   `__init__` constructs and holds a plain `SpoolTransient` and every call forwards to it, so
   the fields compare `==` (the rung 38/39 contract, one rung on).
3. **Rung 39 is left LITERALLY unchanged.** `match`/`_cascade_map` are untouched (the rung-33
   discipline), so the rung-39 suite still witnesses them bit-for-bit.
4. **The default `run(...)` is untouched** => the production cycle is **bit-for-bit rung 6**.

---

## Verification gates

1. **REDUCE — the 2-D equilibrium == rung 39** across a throttle sweep, CPG + reacting.
2. **REDUCE — `lp_disabled` dispatch == rung 34 `SpoolTransient`**, bit-for-bit (`==`).
3. **NON-TAUTOLOGICAL — an INDEPENDENT bare-math CPG two-shaft closure** (no `Gas` /
   `Component` / `ComponentMap` / `TwoSpoolTransient` calls; own closed-form CPG
   thermodynamics, own choke bisection, own forward speed lines, own 2-D equilibrium)
   reproduces the shipped `(nu_L, nu_H, pi_lpc, pi_hpc)` **and `sigma_crit` on SHAPED maps**
   across a throttle sweep. Reproducing `sigma_crit == 1` on flat+CPG would only re-check the
   reduce; the shaped value (~1.20, ~0.92) is what ties the object down (mirrors rung 39 gate 3).
4. **The `sigma_crit` IDENTITY and its two channels** — `== 1` to ~1e-13 on flat+CPG at every
   throttle (labelled INHERITED from rung 39 B1); broken on the variable-`cp` gases and by the
   map, with the map channel the larger. Plus the **refutation**: the shift direction is
   shape-dependent (`lp-only` below 1, `hp-only` above).
5. **FINDING (i) — STABILITY.** `a<0`, `d<0`, `a*d>b*c` at every sampled point (>= 5 shape pairs
   x 3 throttles x 2 gases), hence both eigenvalues negative at every `rho` in `[0.05, 100]`.
   The empirical signs are asserted; `rho`-freeness follows.
6. **FINDING (ii) — THE COMPLEX MODE.** `b*c < 0` for **every** shaped-LP pair and `b*c >= 0`
   for **every** flat-LP pair (incl. `hp-only`, the discriminator), so `oscillatory_band` returns
   a band in the first case and `None` in the second; and the returned band genuinely brackets
   the sign change of the discriminant. **Existence + sign + mechanism only** — the band
   location and `|Im/Re|` are NOT gated.
7. **SCOPE — `sigma_crit` is first-instant only.** The marched threshold `rho*` does **not**
   converge to `sigma_crit` (asserted as a *deliberate* non-convergence, so the withdrawn claim
   cannot silently creep back), and the finite-ramp excursion is schedule-slaved.
8. **CYCLE UNTOUCHED** — the default design run is bit-for-bit rung 6.

---

## Concessions

- **Much of this rung is inherited** (see the table at the top). The genuinely new content is
  the 2-D transient itself, the `rho`-split, and the LP-map-created complex mode.
- **`rho` is a DISCLAIMED clock group, doubled** — `I_L`, `I_H`, `w_L,d`, `w_H,d` are not
  modelled; only the ratio enters, and no wall-clock time is claimed (rung 34's concession).
- **Every magnitude rides on the representative maps** — the band location, `|Im/Re| <= 0.25`,
  the `sigma_crit` shift size. Rung-32 methodology: shapes disclosed, claims shape-robust,
  magnitudes disclaimed.
- **The oscillation claim is scoped to INTER-SPOOL** — rung 37's shaft+metal Jacobian is not
  audited.
- **Fully-choked branch only; both NGVs assumed choked** — inherited from rungs 38/39
  unchanged. The LP nozzle-unchoke branch remains the rung-33-shaped follow-on.
- **One `eta_m` for both shafts; no bypass/bleed/interstage loss/reheat; isentropic knobs
  only** — inherited from rung 38.
- **No surge line on either spool** — rung 36's machinery is single-spool. Whether the complex
  mode or the lead threshold has surge consequences is exactly the claim this rung declines.
- **`Tt4` is the control** — rung 35's fuel metering is not carried onto the two-shaft path.

---

## Anchor

`docs/plans/rung40-anchor-two-shaft-transient.md`. The **method** is rung 34's forward-closure
transient applied to rung 39's two-spool hardware; the **non-tautological gate** (an independent
bare-math CPG two-shaft closure reproducing `sigma_crit` on shaped maps) is the rigorous anchor,
exactly as for rungs 31/33/38/39.
