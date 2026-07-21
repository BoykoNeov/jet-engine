---
name: rung35-fuel-metering
description: "SHIPPED rung 35 = fuel is the control, Tt4 becomes an OUTPUT; cross-rung CORRECTION of rung 34 (fuel control ENLARGES the surge excursion + exposes the TIT overshoot — the two accel limits are COUPLED); reduce = control-invariance via the forward-burner closure"
metadata: 
  node_type: memory
  type: project
  originSessionId: b9355d37-5f1b-4a1c-b71a-fd36105417db
  modified: 2026-07-21T12:38:55.239Z
---

SHIPPED rung 35 — **fuel is the control; `Tt4` becomes an OUTPUT** (the fuel-metering picture).
Code on `SpoolTransient` in `turbojet/engine.py`; `docs/rung35-spec.md`,
`docs/plans/rung35-anchor-fuel-metering.md`, `tests/test_rung35.py`, `main.py`
`print_fuel_metering_table`. Closes rung 34's one filed concession (`Tt4(t)`-control).

**The make-or-break (advisor):** command the fuel **mass flow** `ṁ_fuel`, NOT the ratio `f`. If you
command `f` then `Tt4=burner(Tt3,f)` and it's a re-labeling — the whole finding lives in
`f=ṁ_fuel/ṁ_air` **spiking because `ṁ_air` LAGS** the spool.

**The structural move:** the burner runs **FORWARD** (`_tt4_from_f` = exact inverse of the shipped
`f`-solve `_solve_f`: `h4=(h_c(Tt3)+f·η_b·hPR)/(1+f)`, `Tt4=T_from_h_t(h4,f)`). In
`_close_compressor_fuel` the trial corrected flow `m` fixes `ṁ_air` directly (corrected-flow def), so
`f` and `Tt4` are OUTPUTS; NGV-choke consistency `g(m)=m−m_imp=0` closes it (Tt4 floating, **no shaft
balance** — rung 34's move). Refactored rung 34's `_instant` turbine/power tail into shared
`_instant_tail` and the equilibrium-ν bracket into `_find_equilibrium_nu` (both bit-for-bit; rung
31–34 suites stay green). Fuel closure scoped to the **non-equilibrium gas** (`f_cap=0.05` search
floor; peak operating `f≈0.037`).

**THE RUNG = a cross-rung CORRECTION of rung 34** (rung-28/29/32 move). At a frozen spool a fuel step
**starves the airflow** (hot NGV passes less corrected mass as `Tt4` rises, `(1+f)` rises), so `Tt4`
**OVERSHOOTS** — a **TIT excursion**, a *second* accel limit commanding `Tt4` HID — **and** that
over-temperature amplifies the airflow deficit, so it also **ENLARGES** rung 34's surge excursion.
The two accel limits (surge + TIT) are **COUPLED, not independent**: `E_surge(fuel) > E_Tt4` at every
`r=τ_fuel/τ_spool`, gap **MAX at r→0** (4.77%; E_surge 5.39%→10.16% for 1100→1400) **VANISHING as
r→∞** (0.11% at r=3) — rung 34 **under-counted** surge. Sign **shape-robust** across 3 surge maps
(magnitude disclaimed). New axis `E_temp` (TIT), on these maps larger than surge.

**Advisor's reporting fix (applied):** `E_temp` is referenced to the running line at the current
speed (the `E_surge` analogue) — NOT an overshoot above the target. Peak occurs at ν≈ν0 so the
reference is `Tt4_lo`. Since a TIT limit is **absolute**, lead with `Tt4_peak` in Kelvin: the r→0 step
peaks at **1754 K = +25% over the 1400 K target** (the 59.5% is vs the 1100 K frozen-speed
running-line value). Added `Tt4_peak` to `ramp_/constant_speed_excursion_fuel`. Also: magnitude claim
rests on the **r→0 STEP** (both modes are steps — unconfounded); finite-`r` gap mixes the coupling
with a forcing-shape diff (linear-`Tt4` vs linear-`ṁ_fuel`); r→∞ vanishing is the clean trend.

**Reduce = CONTROL-INVARIANCE (non-tautological, the tightest anchor):** `equilibrium_fuel` at
`ṁ_fuel=f_eq·ṁ_air,eq` of a Tt4-point reproduces it (`ν,π_c,τ_t,ṁ_air` machine-zero at design,
`Tt4_out==Tt4`) via the forward-burner closure — a genuinely different code path than pinned-`Tt4`.
Plus `r→∞` convergence (dynamical reduce), Tt4-control UNTOUCHED ⇒ rung 34 bit-for-bit, and the
instant-level inverse (`Tt4(f)` inverts the `f`-solve — fuel↔Tt4 analogue of rung 34 gate 6). Separate
entry point (subclasses `SpoolTransient`); default `run` bit-for-bit rung 6.

Leaves open: reacting-gas fuel control (gas-independent finding), a true `ṁ_fuel(t)` metering-unit
schedule + valve model, and rung 34's remaining seams (surge line, volume-filling/heat-soak,
two-spool). Cross-links: [[rung34-spool-transient]] (the rung it corrects + subclasses),
[[rung33-subsonic-matching]] (the inversion-of-a-prior-rung precedent), [[rung32-component-maps]]
(representative-map methodology, magnitude-disclaimed), [[rung29-shifting-turbine]] (RATIO≠ENERGY-style
cross-rung correction precedent).
