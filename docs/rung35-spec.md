# Rung 35 — Fuel is the control; `Tt4` is an OUTPUT (the fuel-metering picture)

Rung 34 made the shaft speed `N` a state and marched it under a **commanded** turbine-inlet
temperature `Tt4(t)`. That was the one thing rung 34 explicitly filed as a concession:

> **Control = `Tt4(t)`.** The fuel schedule is expressed as a burner-exit-temperature schedule
> (matches the shipped `Burner`; `f` is the output). A true `ṁ_fuel(t)` schedule with `Tt4` as an
> output — the real fuel-metering picture — is a further seam.

Rung 35 closes that seam. A real engine does not command `Tt4`; it meters **fuel**. The
turbine-inlet temperature is then an **output** of the burner energy balance against the airflow the
spool can *currently* pump — and during a transient that airflow **lags** the fuel.

---

## Why this is not a re-labeling (the make-or-break point)

If you command the fuel-air *ratio* `f` directly, then `Tt4 = burner(Tt3, f)` and nothing new
happens — the airflow drops out and it is rung 34 with a renamed knob. The physics lives entirely in
commanding the fuel **mass flow** `ṁ_fuel` and letting `f = ṁ_fuel/ṁ_air` float, because the
compressor sets `ṁ_air` and the spool sets the compressor.

At a **frozen spool** (the instant a fuel step lands, before `N` can move):

1. The fuel mass flow jumps to its new value; `ṁ_air` is still at the old spool speed.
2. `f = ṁ_fuel/ṁ_air` **spikes** — more fuel per unit air.
3. The burner balance turns that into a higher `Tt4`, which **overshoots** the intended steady
   endpoint.
4. The NGV is a hot sonic throat: its corrected mass flow `MFP*/√Tt4` **falls** as `Tt4` rises, and
   `(1+f)` rises, so the throat now demands **less** airflow — tightening the deficit further.

So a fuel step produces a **turbine-inlet-temperature excursion** (over-temperature), and that
over-temperature **feeds back** to make the airflow deficit worse — which pushes the compressor
operating point **further above the running line** (lower `φ`, higher `ψ` on the surge side, higher
`π_c`) than rung 34's commanded-`Tt4` acceleration did. The two classic acceleration limits — **surge
margin** and **turbine life (TIT)** — are therefore **coupled**, not independent.

---

## The closure: the burner runs FORWARD, the airflow is solved self-consistently

Everything downstream of the compressor is rung-34 machinery, reused unchanged (the forward map, the
NGV choke, the rung-31 turbine geometry / rung-33 subsonic dispatch, the power imbalance `Φ`). Only
the **compressor+burner closure** inverts.

**Forward burner** (`_tt4_from_f`) — the exact inverse of the shipped burner `f`-solve. The same
enthalpy balance the `Burner` closes for `f`, solved instead for `Tt4`:

```
h4·(1 + f) = h_c(Tt3) + f·η_b·hPR     ⇒     Tt4 = T_from_h_t(h4, f)
```

(Solving `f` from a `Tt4` produced this way recovers `f` to machine zero — gate 4.)

**Fuel-control compressor closure** (`_close_compressor_fuel`) — mirrors rung 34's
`_close_compressor` but with `Tt4` floating. For a trial corrected flow `m` at corrected speed `n`:

```
φ = m/n ,   τ_c = 1 + (τ_c,d−1)·ψ(φ)·n² ,   π_c via the enthalpy/pr inverse ,   pt4 = π_b·π_c·pt2
ṁ_air = m·ṁ_corr,d·pt2/√Tt2            # m fixes the compressor-face airflow directly
f     = ṁ_fuel / ṁ_air                 # FUEL imposed ⇒ f is an OUTPUT
Tt4   = _tt4_from_f(Tt3, f)            # ⇒ Tt4 is an OUTPUT
ṁ4    = A4·pt4·MFP*(Tt4,f)/√Tt4        # the NGV then IMPLIES an airflow
g(m)  = m − (ṁ4/(1+f))·√Tt2/(pt2·ṁ_corr,d)      # trial m vs NGV-implied m
```

Root-find `g(m)=0`. `g` is monotone-increasing (higher `m` → more airflow → lower `f`/`Tt4` **and**
lower `π_c` → lower `pt4` → lower NGV-implied airflow), so it brackets and bisects; the search floor
caps `f ≤ 0.05` to keep the forward burner and the gas in range (the operating `f ~ 0.02–0.03` sits
well above it). The instant then flows into rung 34's shared `_instant_tail` — so the shaft ODE, the
choked/subsonic dispatch, the thrust are all identical arithmetic on either control.

**`Tt4` (reacting gas) is deferred.** The forward burner is built for the **non-equilibrium** gas —
the finding is gas-independent and runs on the fast `thermally_perfect` gas (matching rungs 32–34),
and the reduce to rung 34 on the reacting gas is the **Tt4-control** path, untouched. A reacting-gas
fuel control would root-find `Tt4` on the rung-6 scale-B balance; it does not change the `r` framing.

---

## The finding

Measured on **one** fuel-acceleration trajectory (`Tt4_lo → Tt4_hi`, the fuel levels pinned to the
two steady endpoints so the comparison to rung 34 is apples-to-apples — **no new knob**):

```
E_surge = max_t [ π_c(t) / π_c,runningline(ν(t)) − 1 ]      # the surge axis (rung 34's E)
E_temp  = max_t [ Tt4(t) / Tt4,runningline(ν(t)) − 1 ]      # the NEW TIT axis (E_surge analogue)
Tt4_peak = max_t Tt4(t)                                     # the ABSOLUTE peak (redline number)
```

Both `E`'s are referenced to the **running line at the current speed** `ν(t)`, so `E_temp` is the
exact analogue of `E_surge` — *not* an overshoot above the target endpoint. The peak occurs early
(`ν ≈ ν₀`, where the running-line temperature is `Tt4_lo`), so `E_temp` measures how far `Tt4` climbs
**above the running-line temperature at the frozen speed**. Because a TIT limit is **absolute**, the
load-bearing number is the peak temperature itself: `Tt4_peak`.

**(a) The CORRECTION — fuel control *enlarges* the surge excursion.** `E_surge(fuel) > E_Tt4` at
every `r = τ_fuel/τ_spool`, with the gap **maximal at `r→0`** and **vanishing as `r→∞`**. Numbers
(fast gas, `surge_flow` map, `Tt4` 1100→1400):

| `r` | `E_surge` (Tt4-control, rung 34) | `E_surge` (fuel-control) | gap | `E_temp` | `Tt4_peak` |
|-----|------|------|------|------|------|
| 0*  | 5.39% | 10.16% | **4.77%** | 59.5% | **1754 K** |
| 0.3 | 4.71% | 7.69%  | 2.98% | 37.9% | 1629 K |
| 1.0 | 3.47% | 4.26%  | 0.79% | 18.0% | 1507 K |
| 3.0 | 1.67% | 1.78%  | 0.11% | 7.6%  | 1434 K |

(*`r→0` algebraic, no integration.*) So **rung 34 under-counted the surge excursion** because
commanding `Tt4` suppressed the over-temperature that amplifies the airflow deficit. This is a
genuine cross-rung correction (the rung-28/29/32 move): rung 34's E(r) numbers stand as the *surge
axis under Tt4-control*, but they are **not** the surge excursion a fuel-metered engine actually
sees. The sign is **shape-robust** across ≥3 surge maps (gap +3.5% to +4.1% at `r→0`); the magnitude
rides on the map and is **disclaimed** (rung-32 methodology). **The magnitude claim rests on the
`r→0` step** — there both control modes are steps to the same endpoint, so the gap is *unconfounded*.
At finite `r` a linear-`Tt4` ramp and a linear-`ṁ_fuel` ramp are **different forcings**, so the
finite-`r` gap mixes the airflow-lag coupling with a forcing-shape difference; the `r→∞` vanishing
(gap → 0) is the clean *trend* claim.

**(b) The NEW axis — the TIT excursion.** `Tt4` climbs **above the running-line temperature at the
current speed** by `E_temp` — a second acceleration limit (turbine life) that commanding `Tt4`
structurally cannot show. In absolute terms the `r→0` step for 1100→1400 drives `Tt4` to a peak
**1754 K** — **+25% over the 1400 K target** (and +59.5% over the frozen-speed running-line value,
1100 K, which is what `E_temp` reports). A real engine reads that absolute peak against a **TIT
redline**, which is why acceleration fuel schedules are **temperature-limited as well as
surge-limited**. `E_temp` (and `Tt4_peak`) is **monotone-decreasing** in `r`, its `r→0` limit an
**algebraic** map property (the constant-`N` displacement); on these maps it binds *before* the surge
excursion. Same two-clock story as rung 34: slow the fuel (raise `r`) and both excursions collapse
toward the running line.

---

## Reduce-to-prior contract (the spine)

- **CONTROL-INVARIANCE (the non-tautological reduce).** A steady point does not care whether it is
  named by its `Tt4` or by its fuel flow. Commanding `ṁ_fuel = f_eq·ṁ_air,eq` of a Tt4-control
  running-line point, `equilibrium_fuel` returns the **same** `(ν, π_c, τ_t, ṁ_air)` and `Tt4` comes
  back out at the commanded value — through the **forward-burner + fuel closure**, a genuinely
  different code path than the pinned-`Tt4` closure. Two closures, one operating point (machine-zero
  at design). This is the tightest, tautology-free anchor.
- **`r→∞` convergence (the dynamical reduce).** As the fuel ramp slows, the airflow keeps up, `f`
  never spikes, and the fuel-control trajectory converges onto the Tt4-control one — the excursion gap
  → 0 (0.11% at `r=3`).
- **Tt4-control UNTOUCHED ⇒ rung 34 bit-for-bit.** The rung-35 methods are a separate entry point;
  `equilibrium`/`integrate`/`ramp_excursion` are the shared `_instant_tail` on the pinned-`Tt4`
  closure and still reduce to rung 32's `MapMatcher` (and thence rungs 31/6). Adding fuel control
  moves **no** steady number.
- **INSTANT-LEVEL inverse (the fuel↔`Tt4` analogue of rung 34's map-inverse gate 6).** The forward
  burner `Tt4(f)` is the exact inverse of the burner `f`-solve; at a fixed `ν` the fuel closure at a
  Tt4-instant's fuel recovers that instant's `(Tt4, π_c, ṁ_air)` — off the running line too.
- **Cycle untouched ⇒ rung 6 bit-for-bit.** Constructing a `SpoolTransient` and calling the fuel
  methods does not perturb the default `build_turbojet(…).run` design path.

---

## Verification gates (`tests/test_rung35.py`)

1. **REDUCE — CONTROL-INVARIANCE (non-tautological).** `equilibrium_fuel(f_eq·ṁ_air,eq)` reproduces
   the Tt4-control running-line point (`ν, π_c, τ_t, ṁ_air`) across a throttle sweep and returns
   `Tt4_out == Tt4`; machine-zero at design (`ν=1`, `π_c=10`). Two closures, one point.
2. **REDUCE — Tt4-CONTROL UNTOUCHED + CYCLE.** The Tt4-control equilibrium still reduces to rung 32's
   `MapMatcher`; building a `SpoolTransient` leaves the design run bit-for-bit rung 6.
3. **THE FINDING.** (a) fuel control **enlarges** the surge excursion (`E_surge_fuel > E_Tt4`), gap
   max at `r→0` and shrinking to <40% of that by `r=3` (vanishing), **shape-robust in sign** across
   ≥3 surge maps; (b) the TIT overshoot `E_temp > 0`, monotone-decreasing in `r`, its `r→0` limit the
   algebraic map property. Both axes bounded by their `r→0` limits.
4. **INSTANT-LEVEL INVERSE.** The forward burner `Tt4(f)` inverts the burner `f`-solve to machine
   zero; the fuel closure at a Tt4-instant's fuel recovers that instant off the running line.

---

## Concessions

- **Non-equilibrium gas for fuel control.** The forward burner `Tt4(f)` is built for the fast gas;
  the finding is gas-independent (rungs 32–34 precedent) and the reacting-gas reduce is the
  Tt4-control path. A reacting-gas fuel control (root-find `Tt4` on the rung-6 balance) is deferred —
  it does not change the `r` framing.
- **Magnitude disclaimed (inherited rung 32).** Only the **sign** of the correction and the
  **existence** of the TIT overshoot are claimed shape-robustly; both magnitudes ride on the map.
- **No surge line / no TIT redline number.** Like rung 34, `E_surge` is a signed displacement toward
  surge on a representative map, and `E_temp` is a displacement toward the (undrawn) TIT limit — never
  a *crossing*. Which limit binds first is map-dependent and disclaimed.
- **`Tt4_hi` pinned to a steady endpoint.** The ramp endpoints are the fuels whose steady points are
  `Tt4_lo`/`Tt4_hi`, so the two control modes share endpoints and no new knob enters; a true
  `ṁ_fuel(t)` metering *schedule* (with `Tt4` genuinely free at both ends, and a fuel-metering-unit
  model) is a further seam.
- **`f_cap = 0.05` search floor.** The fuel-closure bracket caps the frozen-spool fuel-air ratio at
  `f ≤ 0.05` to keep the forward burner and the gas in range. The shipped acceleration spans peak at
  `f ≈ 0.037`, well under it; a *much* larger step (e.g. a near-idle→max fuel slam) could drive the
  frozen-spool `f` above `0.05` and trip the bracket assert — by design, so it reads as a clear scope
  error, not a silent wrong answer.
- **Quasi-steady components, single spool, isentropic knobs, NGV choke** — all inherited from rungs
  31–34. Combustor volume-filling / heat-soak (faster clocks below `τ_spool`) and two-spool dynamics
  remain the next dynamic seams; they do not change the fuel-vs-`Tt4` framing.
- **Diagnostic beside the cycle.** Separate entry point; the production run stays rung-6 exact.

---

## Anchor

`docs/plans/rung35-anchor-fuel-metering.md`. The **method** is the standard gas-turbine acceleration
picture — a scheduled fuel ramp kept clear of the surge line **and** the turbine-inlet-temperature
limit, with the over-fuelling/TIT overshoot at a lagging spool (Cohen–Rogers–Saravanamuttoo *Gas
Turbine Theory* Ch. 9; Walsh & Fletcher *Gas Turbine Performance*, transient / fuel-schedule
chapters — the acceleration line sits between surge and the TIT/over-temperature limit). The **reduce
gate** (control-invariance: `equilibrium_fuel` == `equilibrium` via the independent forward-burner
closure) is the rigorous, non-tautological anchor — two code paths onto one operating point, exactly
as rungs 31–34 did for their solves.
