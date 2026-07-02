# Rung-14 external anchors — equilibrium-vs-frozen nozzle flow, and the dropped clamp

Verified numbers + worked checks that anchor the **frozen-vs-equilibrium (shifting) nozzle
expansion** (rung 14): the entropy machinery (inherited `a7`, rung 6), the frozen-reduces-to-
production identity, the recombination direction/magnitude, and the two numbers the rung turns on —
the **thrust bracket** (dormant at the design point, ~0.46% hot) and the **dropped clamp** firing on
the cooling path. Numbers re-derived and certified **before production code** (project discipline),
in `M:\claud_projects\temp\rung14\`.

> **Read `docs/plans/rung6-anchor-equilibrium.md` first.** Rung 14 adds **no chemistry** — it reuses
> rung 6's `_equilibrium_composition` (CEA-anchored: methane-air AFT 2231.7 K vs CEA 2226 K; product
> mole fractions textbook), the scale-B absolute enthalpy `_h_molar_B`, and the absolute entropy
> `_s_molar` (`a7`, GRI cross-checked ≤ 0.3%). The only new thermodynamics is the **mixture entropy
> with its partial-pressure/mixing term** and the reversible-adiabatic expansion built on it.

The mechanism, in one line: **frozen = a lower bound, equilibrium = an upper bound; the real nozzle
sits between**, and the gap is the recombination energy (CO/H₂/OH/O/H → CO₂/H₂O) recovered on
cooling. It is a **hot, dissociated-exhaust** phenomenon — negligible at the cool lean design point,
exactly as rung 6 predicted when it deferred this seam.

---

## 1. The physics anchor — direction, bound, and the published trend

The frozen-vs-shifting nozzle comparison is a classic result (Hill & Peterson *Mechanics and
Thermodynamics of Propulsion* § nozzle flow; Sutton *Rocket Propulsion Elements* § chemical
equilibrium in nozzles). The transferable, **signed** statements we gate against:

| statement | source | ours |
|---|---|---|
| shifting-equilibrium flow gives **higher** exit velocity / Isp than frozen | H&P, Sutton | `V9_equil > V9_frozen` always (gate 3) |
| the exit is **hotter** under shifting flow (recombination reheats) | H&P | `T9_equil > T9_frozen` (gate 3) |
| the gap **grows with chamber temperature / dissociation** | H&P, Sutton | monotone in Tt4 (gate 4) |
| the gap is a **few %** for high-Tc rockets, **< 1%** for modest-Tc air-breathing | Sutton (rockets ~1–4%) | 0.006% (1500 K) → 0.46% (2200 K) — air-breathing, consistent |

The **composition** driving the gap (equilibrium dissociation of CO₂/H₂O) is already the
CEA-anchored rung-6 object; rung 14 adds only reversible-adiabatic bookkeeping, whose correctness is
pinned by the machine-precision reduce below (§3). There is **no bespoke published digit** for this
particular `(CH₂)ₙ`/air, lean, `pt≈7.5 bar`, `Tt4` case (stated plainly in §5); the binding ties are
the inherited composition anchor + the signed trend + the exact reduce.

---

## 2. The two numbers rung 14 turns on

### (a) The thrust bracket — dormant at the design point, earns its keep hot
Real-loss cycle (`π_d=0.97, η_c=0.88, η_b=0.99, π_b=0.96, η_t=0.90, η_m=0.99, π_n=0.98`), fully
expanded (`p9 = p0 = 50 kPa`), `π_c=10`, flight `M0=0.85`:

| Tt4 (K) | entry CO/(CO+CO₂) | V9 frozen (m/s) | V9 equilibrium (m/s) | ΔV9 (m/s) | ΔV9 / V9 |
|---|---|---|---|---|---|
| 1500 (design) | 5.12e−6 | 1039.89 | 1039.96 | +0.071 | **+0.0068 %** |
| 1800 | 2.52e−4 | 1208.61 | 1209.23 | +0.623 | +0.0516 % |
| 2000 | 1.89e−3 | 1310.45 | 1312.51 | +2.054 | +0.157 % |
| 2200 (hot anchor) | 1.10e−2 | 1406.54 | 1413.01 | +6.474 | **+0.460 %** |

At the metallurgically-capped design point dissociation is suppressed twice (lean + high pressure,
rung 6's own lesson), so the bracket is **DORMANT** (~0.006%) — negligible *here*, like the clamp. By
2200 K about **1% of the carbon** is dissociated and recombination in the nozzle buys **~0.46%** more
exhaust velocity. The recovery is **strictly positive** and **monotone** in Tt4. (Ideal-loss cycle
cross-ref: 1500 K → +0.0063%, 2200 K → +0.438% — same story, loss-independent.)

### (b) The dropped clamp fires on the cooling path
Design point, real losses, exhaust cooling `Tt9=1263 K → T9=812 K`:

| quantity | value |
|---|---|
| equilibrium NO at nozzle entry (Tt9) | 253 ppm |
| equilibrium NO at nozzle exit (T9) | 2.11 ppm |
| **collapse ratio** (frozen-NO-independent) | **120×** |
| realistic zoned exhaust NO (φ_p=1, EI_NO≈21 g/kg) | 529 ppm |
| **max_a = frozen NO / eq NO(T9)** | **250** |
| rung-10 combustor-quench max_a (dormant) | 0.677 |

The equilibrium NO **collapses 120×** as the exhaust cools (Kp_NO = exp(−ΔG°/RuT) is steeply
T-dependent). A realistic combustor exhaust carrying **ICAO-band NO** (the rung-8 zoned number) is
frozen through the nozzle, so at the exit it is **250× super-equilibrium**. Rung 7's `cNO≤cNOe` clamp
would **delete that surplus** — a plausible-but-wrong low number with every assert green. **This is
where rung 10's dropped clamp earns its keep:** rung 10 measured `max_a=0.677<1` (dormant) on the
combustor quench and dropped the clamp *on principle*, flagging that it *would* bite in the near-
stoich exhaust cooling. It does — decisively.

**Honest note (the mixed-out artifact):** the rung-7 *mixed-out* combustor NO is ~0 at this lean
point (φ≈0.4 makes essentially no NO), so a naive `a_exit` computed from it is < 1. That is a
modeling artifact of ignoring zoning, not a reprieve for the clamp: the NO that actually leaves a
real engine is the **zoned** ICAO-band number, and *that* is wildly super-equilibrium at the exit.
The certified core is the **frozen-NO-independent collapse ratio** (120×); `max_a` needs a frozen
exhaust NO and we feed it the physically-correct zoned value.

---

## 3. The reduce gates (exact by construction)

- **Frozen ≡ the production nozzle** (machine precision): the frozen branch of `_expand_nozzle` is
  the production pr-ratio expansion re-derived on the molar entropy/enthalpy scale (the fixed-comp
  mixing term cancels, § spec). Measured `|V9_frozen − V9_production| ≈ 5×10⁻¹² m/s` at Tt4 =
  1500/1800/2200 K. Bisection tolerance `1e-13·T` makes this near-machine.
- **Freeze-comp-in-eq-branch ≡ frozen** (bit-for-bit): forcing `_equilibrium_composition` to return
  the station-4 mixture makes the shifting branch equal the frozen branch **exactly**
  (`V9_equil − V9_frozen = 0.0`) — proving the ONLY difference between the two brackets is the
  composition shift, not any entropy/enthalpy bookkeeping asymmetry.
- **Dissociation → 0** (bounded): at Tt4 = 1300 K the entry `CO/(CO+CO2) = 1.4×10⁻⁷` and `ΔV9 =
  0.0098 m/s` (0.001%) — the bracket collapses when there is nothing to recombine.
- **Reduce-to-lower-rung (tight):** `nozzle_flow` is a *separate, opt-in* diagnostic that only reads
  state; the whole rung 1–13 suite stays green untouched and the cycle is bit-for-bit rung 6.

---

## 4. The invariants (discriminating checks, not book digits)

- **Isentropic:** both expansions conserve mixture entropy, `S_mix(exit) = S_mix(entry)` to `< 1e-6`
  relative (the constraint each bisection is built on).
- **Mass invariant:** recombination conserves atoms, so `_mix_mass_per_air` is identical for the
  frozen and equilibrium exits — the two `V9 = √(2ΔH/m)` denominators match (a wrong `m` would break
  the direction gate).
- **Recombination signature:** the equilibrium exit has **less CO** than the frozen entry pool
  (CO → CO₂), and a **higher T9** than frozen despite a *smaller* sensible enthalpy drop — the
  released formation energy shows up because the expansion uses **absolute** (scale-B) enthalpy.
- **Bracket guard:** the exit-`T` bisection never pins at the 500 K floor (real exits > 700 K; the
  equilibrium Newton diverges below ~500 K — measured: OK at 500 K, diverges at 450 K).

---

## 5. What stays UN-anchored / deferred (state it plainly)

- **No bespoke published digit** for this exact `(CH₂)ₙ`/air, lean, `pt≈7.5 bar` frozen-vs-shifting
  case — the anchor is the inherited CEA composition (rung 6) + the signed textbook trend (§1) + the
  machine-precision frozen reduce (§3). A CEA rocket-Isp digit is a *different* gas (stoich H₂/O₂,
  high Tc) and is not transferable to a lean air-breather.
- **Finite-rate nozzle chemistry** — the *real* flow between the frozen and equilibrium bounds (a
  nozzle Damköhler number / transported composition). Rung 14 gives the **bracket**, not the point.
- **Shifting turbine** — the equilibrium expansion is isolated to the nozzle (a full equilibrium hot
  path reopens the shaft balance). The turbine effect is even smaller (smaller ΔT).
- **Super-equilibrium O / prompt (Fenimore) NO** — the clamp corollary uses equilibrium-O NO, so the
  frozen exhaust NO (and hence `max_a`) is an equilibrium-O **lower bound**.
- **Entry irreversibility sliver** — the "infinite-rate-at-the-throat" common-entry model ignores the
  tiny entropy generated by re-equilibrating the frozen entry gas; taking the *fair bracket* over the
  strict-isentropy entry is the deliberate call (§ spec).

## Sources
- Hill, P. & Peterson, C., *Mechanics and Thermodynamics of Propulsion*, 2nd ed. — frozen vs
  equilibrium nozzle flow; recombination and the exit-velocity bound.
- Sutton, G. & Biblarz, O., *Rocket Propulsion Elements* — frozen vs shifting-equilibrium expansion,
  the few-% Isp difference and its growth with chamber temperature.
- Inherited: `docs/plans/rung6-anchor-equilibrium.md` (the CEA-anchored equilibrium composition,
  `a6`/`a7`/`Kp`) and `docs/rung10-spec.md` § "the clamp trap" (the dropped `cNO≤cNOe` cap).
