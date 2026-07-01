# Rung-9 external anchors — rich primary / RQL, the rich flank of the NOx bell

Verified numbers + a machine-checked worked example that anchor the **rich primary** (rung 9):
the rich side of the NO-vs-φ bell that explains *why* a Rich-burn / Quick-Quench / Lean-burn
(RQL) combustor burns its primary rich. Every number was re-derived **before any production
code** (project discipline); the worked example runs on the *existing* rung-6/7/8 primitives
(`_equil_solve`, `_equilibrium_composition`, `_h_molar_A`, `_thermal_no`) plus the one new thing
— a rich-branched Newton seed — in `M:\claud_projects\temp\rung9\proto_rich.py`.

> **Read `docs/rung8-spec.md` first.** Rung 9 is built *on* rung 8: it runs the **same**
> two-zone `zoned_nox` with the **same** extended-Zeldovich integrator on the **same** frozen-
> equilibrium pool — only now the primary may run **rich**, so the pool carries major CO/H₂.
> Nothing in the thermochemistry, the `Kp`/`_equil_solve` reaction machinery, or the `a6`/`a7`
> substrate changes. The **only** new code is a seed branch. NO stays a **trace, decoupled
> diagnostic**, so the cycle is still **bit-for-bit rung 6**.

**The lesson (it completes rung 7's inversion on the rich side).** Rung 8 climbed the *lean*
flank of the bell (φ_p 0.7 → 1.0, EI_NO rising into the ICAO band). Rung 9 shows the **whole
bell**: EI_NO peaks near stoichiometric, then **collapses** as the primary goes rich — because
thermal NO needs high T *and* free O/O₂, and going rich rolls the AFT over *and* starves the O
pool. A rich primary is therefore a **low-NOx regime**. That is the entire design premise of RQL:
burn rich (low NO), then **quick-quench past** the stoichiometric NO peak to a lean burnout. The
model now contains both flanks, so it explains not just *that* zoning makes NO, but *how you move
the zone off the peak*.

---

## 1. Sourced numbers (numbers before code)

### 1a. Primary-zone equivalence ratio & the RQL architecture
Real high-power combustor primaries run **slightly rich to rich, φ ≈ 1.0–1.5** (~90 % of the fuel
burnt there). NO-formation rate peaks sharply at φ ≈ 1; going rich cuts it because peak flame
temperature falls past φ≈1.05 *and* the O-atom pool the Zeldovich initiation `O + N₂ → NO + N`
needs is depleted. **RQL** (Rich-burn / Quick-Quench / Lean-burn) exploits exactly this: a rich
primary (φ ≈ 1.2–1.8, low NO because O-starved and cooler than stoich), a **fast** dilution that
quenches **through** the stoichiometric NO peak before much NO can form, then a lean burnout. The
"quick" in quick-quench is the whole game — a slow quench that lets the gas *dwell* at stoich
re-makes the NO the rich primary avoided.
*(Lefebvre & Ballal, *Gas Turbine Combustion*, ch. on low-emissions combustors; NASA
TR 19730017094 for the near-stoichiometric NO peak.)*

### 1b. The rich upper bound (soot / no-C(s) basis limit)
The model's 5 dissociation reactions produce only gas-phase CO/H₂/CO₂/H₂O/O₂/OH/O/H — **no solid
carbon C(s), no higher hydrocarbons, no soot**. Equilibrium graphite appears at the atomic
**C/O = 1** line; for the (CH₂)ₙ fuel in air, O per fuel unit = 3/φ and C = 1, so C/O = φ/3 ⇒
graphite onset at **φ = 3** (where the proto's Newton solve correctly *fails to converge*, the
system going singular). Practical flame **soot** appears kinetically much lower, φ ~ 1.8–2. So
the hard scope guard is **φ_p ≤ 2.0**: it covers the real RQL primary range (φ 1.2–1.8) with
headroom, sits at the practical soot limit, stays well below the φ=3 graphite singularity, and
the branched-seed Newton converges there with margin (~100 of a 200-step cap). Above the bound
the model is **silently wrong** (it would happily return a soot-free composition that does not
exist), so it is a **hard assert**, not a caveat.

### 1c. The CEA rich-methane anchor (the equilibrium physics)
Methane-air adiabatic flame temperature (chemical equilibrium, NASA-CEA / GRI-Mech): **peaks
slightly rich (φ ≈ 1.05) at ~2231 K at 1 atm**, ~2224 K at stoichiometric, and *falls on both
flanks*. Our 5-reaction solver reproduces this: peak at φ ≈ 1.02–1.05, ~2238 K (≈7 K high — the
same NO/N-deferred, 5-species offset rung 6 noted at stoich), stoich 2231.6 K, and the rich
rollover (φ=1.3 → 2058 K < stoich). The rich pool also satisfies the **water-gas shift**
CO+H₂O⇌CO₂+H₂ (Δν=0) to ~1e-6 — a pure thermodynamic self-check (the shift Kp is `K₁/K₂` of the
two CO₂/H₂O dissociation reactions the solver already enforces), confirming the branched rich
seed lands on the *real* equilibrium, not just an atom-balanced point.
*(Marzouk 2024, *Eng. Tech. Appl. Sci. Res.* / arXiv 2503.11826 — CEARUN & Cantera methane-air
AFT; Turns, *An Introduction to Combustion* — the NO-vs-φ bell.)*

### 1d. Carried over from rungs 7–8 (unchanged)
- **Residence time** `τ ≈ 3 ms`; the extended-Zeldovich rate constants + the thermo-kinetic
  **`K`-check** (asserted every run — now also at the *lower* rich primary T, still in band).
- The EI_NO **ICAO band** (18–64 g/kg take-off, § rung-8 anchor 1b) — an order-of-magnitude
  landing zone, not a book digit.

---

## 2. The rich two-zone worked example

**Model (rung-8 flow, now with a rich primary):** split the air (α to the primary with all the
fuel, α = far/far_p < 1 for rich), burn the primary adiabatically from Tt3 on scale A (rich
equilibrium products, CO/H₂ major), run the rung-7 `_thermal_no` there, then add the rest of the
air, re-equilibrate the majors (T_mix → Tt4), freeze the NO moles.

**Worked example** (`main.py` design point: **Tt3 = 584 K, Tt4 = 1500 K, far = 0.0272,
p = 7.5 bar, τ = 3 ms**; φ_overall = 0.40) — the full bell:

| primary φ_p | AFT (K) | xCO (%) | xH₂ (%) | NO_eq (ppm) | NO_kin (ppm) | **EI_NO (g/kg)** | T_mix (K) |
|---|---|---|---|---|---|---|---|
| 0.80 | 2225 | 0.14 | 0.03 | 5922 | 236.2 | **4.78** | 1517 |
| 0.90 | 2356 | 0.46 | 0.08 | 5680 | 881.2 | **15.99** | 1517 |
| **1.00** | 2442 | 1.35 | 0.24 | 3803 | 1254.6 | **20.74** ← peak | 1517 |
| 1.05 | 2452 | 2.21 | 0.42 | 2436 | 880.2 | **13.98** | 1517 |
| 1.10 | 2435 | 3.38 | 0.70 | 1319 | 419.3 | **6.42** | 1517 |
| 1.30 | 2276 | 8.27 | 2.62 | 105 | 7.1 | **0.096** | 1517 |
| 1.50 | 2110 | 11.96 | 5.36 | 11 | 0.1 | **0.001** | 1517 |
| 1.80 | 1879 | 15.86 | 10.06 | ~0 | ~0 | **~0** | 1517 |
| 2.00 | 1736 | 17.86 | 13.20 | ~0 | ~0 | **~0** | 1517 |

The bell peaks at φ_p ≈ 1.0 (**EI_NO ≈ 21 g/kg**, in the ICAO take-off band) and **collapses on
the rich flank**: a rich primary at φ_p = 1.4 makes ~0.01 g/kg — **~1800× less NO** — even though
it burns *all* the same fuel. CO/H₂ climb into the double digits (unoxidized fuel), the AFT rolls
over past φ≈1.05, and [O]/[OH] crash — the two effects that starve the Zeldovich rate. `T_mix` is
**split-independent** (1517 K for *every* φ_p) and returns to ≈ Tt4: adding the rest of the air
re-equilibrates the rich CO/H₂ back to lean products, releasing their oxidation energy so the
mixed-out state lands exactly where the cycle's station 4 already is — no matter how rich the
primary was.

---

## 3. The reduce-to-rung-8 gate (bit-for-bit by construction)

At φ_p ≤ 1 the primary far ≤ f_stoich, so the O-balance is lean (`bO ≥ 2bC + bH/2`) and
`_equil_solve` takes the **lean branch** — the *byte-identical* rung-6 seed expression. Every
rung-1..8 code path is lean (the cycle burner runs at φ ≈ 0.40), so **none of them ever reach the
rich branch**, and their damped-Newton trajectory is identical to before. The reduce is therefore
**provable, not empirical**: the whole rung 1–8 suite is bit-for-bit (verified: 58 tests green,
untouched), and rung 8's exact same-`T_p` identity still holds. The one rung-8 test that changed
is the *scope guard* (`test_phi_primary_guard`): its φ_p ≤ 1 bound was the rung-8 contract; rung 9
widens the physical scope to φ_p ≤ 2, so the guard now rejects only above the soot bound. This is
not "editing a test to pass" — it is the scope legitimately expanding, and it is called out at the
test.

**Why the seed had to branch (not just lift the guard).** The lean seed's
`O2 = (bO − 2bC − bH/2)/2` goes *negative* when rich (floored to 1e-8) and grossly violates the O
balance, so damped Newton is fragile from it — the stock solver has real convergence gaps at the
O-balance-violating stoich/low-T corner (proto `proto_stockrobust.py`). Branching on the O-sign
gives the rich regime an O-limited (atom-conserving) seed that converges cleanly to the soot
bound, while leaving the lean expression *literally untouched* so the reduce stays bit-for-bit.

---

## 4. The invariants (discriminating checks, not book digits)

- **The bell peaks near stoich and falls on both flanks.** Lean flank (rung 8) rises; rich flank
  (rung 9) collapses. Peak at φ_p ≈ 0.95–1.0. This is the classic Turns/Heywood NO-vs-φ curve —
  the *shape* is the anchor, not a single digit.
- **Water-gas-shift self-check.** The rich pool satisfies (x_CO₂·x_H₂)/(x_CO·x_H₂O) = Kp_WGS(T)
  to ~1e-6, with Kp_WGS from the *same* `g0(T)` the reaction Kp's use — the rich analogue of
  rung 7's thermo-kinetic K-check.
- **Split-independent T_mix → Tt4, spanning CO/H₂ oxidation.** `T_mix` is identical across the
  rich sweep and returns to Tt4 — now because the majors re-equilibrate *and* the CO/H₂ oxidize
  on dilution, releasing their chemical energy. Freezing the rich composition would trap far more
  energy than rung 8's dissociation-only case and miss Tt4 by more — a sharper discriminator.
- **NO-mole conservation through dilution** (carried from rung 8): the mole fraction falls, EI
  (per kg fuel) is set in the primary.
- **`K`-check + trace guard bind at the rich primary T** (~1715–2450 K, inside the rung-7 band);
  rich NO_eq is *lower*, so the trace assumption is safer, not weaker.

---

## 5. What stays UN-anchored (state it plainly)

- **Finite-rate quench (RQL's defining hazard).** Our mix-out is the **ideal, infinitely-fast**
  quench: NO frozen at the primary value. The real RQL failure mode — NO spiking as the gas
  **dwells at stoich** while the quench air mixes in — needs a secondary-zone Zeldovich (finite
  mixing rate). It is the **explicit next seam**; rung 9 is "rich primary + ideal quench," and a
  flat "RQL solved" would overclaim.
- **Super-equilibrium O / prompt (Fenimore) NO** (rung-7 seam) is still deferred — and it matters
  *most* in the rich primary (the flame front has super-equilibrium radicals and the prompt path
  is a rich-zone effect). So even the rich primary under-counts the true rate; our rich flank is
  the *equilibrium-O* lower bound.
- **α (air split) is a knob**, like φ_p and τ — swept to show the mechanism, not derived from a
  specific combustor's airflow schedule.
- **The soot bound is a scope guard, not a soot model.** Above φ_p ~2 real mixtures soot; we do
  not model soot or C(s) — we *stop* (hard assert). The graphite-onset φ=3 is where the math goes
  singular; the practical guard at 2.0 keeps us in the physically-meaningful, converged range.
- The EI_NO **band** (§1d) is an order-of-magnitude landing zone, not a to-the-digit anchor (the
  absolute Zeldovich rate is un-pinned, per rung 7). What is certified is the **shape** of the
  bell and that the rich flank collapses for the right physical reasons (AFT rollover + O-starve).

---

## Sources
- A. H. Lefebvre & D. R. Ballal, *Gas Turbine Combustion* — primary-zone equivalence ratio, RQL
  (rich-burn / quick-quench / lean-burn) low-NOx architecture.
- NASA TR 19730017094 — *Parameters Controlling Nitric Oxide Emissions from Gas Turbine
  Combustors* (near-stoichiometric NO peak): https://ntrs.nasa.gov/api/citations/19730017094/downloads/19730017094.pdf
- S. R. Turns, *An Introduction to Combustion* — the NO-vs-φ bell, thermal-NO / Zeldovich, EI_NO.
- O. Marzouk (2024), *Eng. Tech. Appl. Sci. Res.* — CEARUN / GRI-Mech / Cantera methane-air
  adiabatic flame temperatures (peak slightly rich, ~2231 K @ φ=1.05, 1 atm):
  https://arxiv.org/pdf/2503.11826
- ICAO Aircraft Engine Emissions Databank (EASA):
  https://www.easa.europa.eu/en/domains/environment/icao-aircraft-engine-emissions-databank
