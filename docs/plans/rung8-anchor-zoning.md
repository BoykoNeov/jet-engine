# Rung-8 external anchors — combustor zoning, the primary-zone NOx effect

Verified numbers + a machine-checked worked example that anchor **combustor zoning** (rung 8):
the two-zone (rich-primary → dilution) picture that explains *why* the rung-7 mixed-out
station-4 EI_NO is ~6 orders of magnitude below a real engine, and how resolving a hot,
near-stoichiometric **primary zone** lifts it into the measured **ICAO databank band**. Every
number was re-derived **before any production code** (project discipline); the worked example
runs on the *existing* rung-6/7 primitives (`_equilibrium_composition`, `_h_molar_A`,
`_thermal_no`) in `M:\claud_projects\temp\rung8\proto_zoning.py`.

> **Read `docs/rung7-spec.md` first.** Rung 8 is built *on* rung 7: it runs the **same**
> extended-Zeldovich integrator on the **same** frozen-equilibrium pool — only the `(T, p, far)`
> it is handed changes, from the mixed-out station-4 state to a **hot primary-zone** state.
> Nothing in the thermochemistry, the `Kp`/`_equil_solve` machinery, or the `a6`/`a7` substrate
> changes. NO stays a **trace, decoupled diagnostic**, so the cycle is still **bit-for-bit rung 6**.

**The lesson (it completes rung 7's inversion).** Rung 7 showed NO is kinetically frozen far
*below* equilibrium and is exponentially T-sensitive — then computed it at the **capped, lean,
mixed-out** turbine inlet (Tt4 = 1500 K, φ ≈ 0.41) and got **EI_NO ≈ 8e-6 g/kg**, essentially
zero, ending with *"Real NOx is a hot primary-zone effect."* Rung 8 makes that concrete: real
combustors burn a **near-stoichiometric primary zone** at ~2300–2450 K (where NO forms), then
add **dilution air** to cool the mixed-out gas back to the metallurgical Tt4 — *without
destroying the frozen NO*. Resolve that zone and EI_NO jumps to **~13–18 g/kg**, squarely in the
measured band. The capped mixed-out temperature never made the NO; a hot zone you averaged away
did.

---

## 1. Sourced numbers (numbers before code)

### 1a. Primary-zone equivalence ratio
NO forms in the primary zone, where the burnt-gas temperature is highest and the mixture is
**close to stoichiometric**. At high power a real primary zone runs **slightly rich**, mean
φ ≈ **1.0–1.5** (~90 % of the fuel burnt there); NO-formation rate peaks sharply at **φ = 1**.
Lean-premixed low-NOx designs deliberately hold the primary **φ ≲ 0.6** to cap peak flame
temperature at ~1500–1600 °C and starve thermal NO.
*(NASA TR 19730017094, "Parameters Controlling Nitric Oxide Emissions from Gas Turbine
Combustors"; Lefebvre & Ballal, *Gas Turbine Combustion*.)*

**Scope decision (bounds rung 8).** Our `_products_composition(f)` / `_equilibrium_composition`
stoichiometry is **lean-complete-combustion (φ ≤ 1)** — a *rich* primary needs CO/H₂/unburnt
stoichiometry we do not have. So rung 8 holds the primary **lean-to-stoichiometric (φ ≤ 1)**; a
rich-primary → RQL combustor is the explicit **next** seam. Thermal NO peaks *slightly lean*
anyway, so φ ≤ 1 (default sweep 0.7 → 1.0) carries the full lesson. φ = 0.9–1.0 is the
teaching point; φ > 1 is guarded, not modelled.

### 1b. The real EI_NO band to land in (ICAO databank)
Emission index EI_NO = **g NO(x) per kg fuel** at the ICAO LTO reference conditions. Measured
values across modern high-bypass turbofans:

| condition | EI_NO band (g/kg fuel) | source |
|---|---|---|
| take-off / climb (85–100 % thrust) | **18–64** (typical **20–40**) | ICAO EEDB; Sheikhi 2023 (24 engines) |
| cruise / approach (lower power) | **~5–11** | cruise-condition turbofan studies |
| ICAO-style reference points (100/85/30/7 %) | 52.7 / 36.2 / 7.5 / 4.3 | lissys PUG ch.12 worked datum |

**Target:** the rung-7 near-zero (8e-6 g/kg) must climb into **single-digit-to-tens g/kg** once
the primary zone is resolved — the rung-8 analogue of rung-6's AFT-into-the-real-band moment.

### 1c. Carried over from rung 7 (unchanged)
- **Residence time** `τ ≈ 3 ms` (primary-zone knob; swept, not derived from geometry).
- The extended-Zeldovich rate constants + the thermo-kinetic **`K`-check** (still asserted on
  every run — and now exercised at the *hotter* primary T, ~2300–2450 K, well inside its band).

---

## 2. The two-zone model + worked example

**Model (all on existing primitives, φ ≤ 1 primary):**
1. **Split the air.** Fraction `α` of the air enters the primary with **all** the fuel, so the
   primary fuel/air ratio is `far_p = far_overall / α`. Choose `α` to hit a target primary
   φ_p = far_p / far_stoich (≤ 1).
2. **Primary burn — adiabatic AFT from Tt3.** Solve `T_p` such that the equilibrium products'
   enthalpy equals the reactants' (primary air preheated to **Tt3**, plus fuel), on **scale A**
   (`_h_molar_A`). Preheating from Tt3 — *not* 298 K — is what ties the primary flame to the
   actual cycle and makes the reduce gate (§3) exact.
3. **Zeldovich NO in the primary.** Run the rung-7 `_thermal_no` on the primary equilibrium pool
   at `(T_p, p, τ, far_p)`. This is where essentially all the NO forms.
4. **Dilution / mix-out.** Add the remaining `(1−α)` air at Tt3; **re-equilibrate the major
   species** (so stored dissociation energy releases) and find `T_mix` by enthalpy conservation.
   **Freeze the NO moles** (NO is the trace kinetic species — never in `_equil_solve`); the NO
   *mole fraction* falls by dilution, but NO *moles per kg fuel* (→ EI_NO) are set in step 3.

**Worked example** (equilibrium-engine design point: **Tt3 = 548.6 K, Tt4 = 1500 K,
far = 0.02782, p = 8.03 bar, τ = 3 ms**; φ_overall = 0.411):

| primary φ_p | air frac α | far_p | primary AFT | NO_eq (ppm) | NO_kin (ppm) | **EI_NO (g/kg)** | T_mix | NO_mix (ppm) |
|---|---|---|---|---|---|---|---|---|
| — mixed-out (rung 7) | 1.000 | 0.02782 | 1500 K | 982 | 2.0e-4 | **7.8e-6** | 1500 | 2.0e-4 |
| 0.70 | 0.587 | 0.04737 | 2044 K | 4732 | 21.4 | **0.49** | 1508 | 12.8 |
| 0.80 | 0.514 | 0.05414 | 2203 K | 5641 | 182.5 | **3.69** | 1508 | 96.4 |
| 0.90 | 0.457 | 0.06091 | 2338 K | 5452 | 735.7 | **13.35** | 1508 | 348 |
| 1.00 | 0.411 | 0.06768 | 2427 K | 3578 | 1080.8 | **17.86** | 1508 | 466 |

The primary-zone EI_NO (φ_p = 0.9–1.0) lands at **13–18 g/kg — inside the ICAO take-off band** —
vs the mixed-out **8e-6 g/kg**: a **~6-order-of-magnitude** lift purely from resolving the hot
zone. The T-sensitivity is on display: φ_p 0.7 → 1.0 (AFT 2044 → 2427 K, +383 K) moves EI_NO
**36×** (0.49 → 17.9), the rung-7 exponential-in-T rate showing through.

---

## 3. The reduce-to-rung-7 gate (exact to within η_b)

At `α → 1` (all air in the primary) the primary far = the overall far, so the two-zone
diagnostic must collapse to rung 7's single, mixed-out `thermal_nox(far, Tt4, p)`. The primary
AFT at full air is the load-bearing check:

```
primary AFT at α = 1 : 1508.2 K
burner-set Tt4        : 1500.0 K   → gap 8.2 K = η_b (0.99) effect
```

The gap is **exactly** the combustion-efficiency loss: the cycle burner needs ~1 % *more* fuel
to reach 1500 K (1 % of the heat is "lost"), so a truly **adiabatic** burn at that same far runs
~8 K hotter. Set η_b = 1 and the two coincide to the bisection tolerance. This is the rung-8
analogue of rung-6's cold-Tt4 reduce-to-rung-5 seam: **the model contains its predecessor as a
limit.** (Test gate: at α→1 the zoned EI_NO must equal the rung-7 mixed-out EI_NO to ~1e-6.)

---

## 4. The invariants (discriminating checks, not book digits)

- **Mix-out temperature is split-independent.** `T_mix = 1508 K` for *every* φ_p in the table.
  Enthalpy is conserved and the fuel is fixed, so the mixed-out state does not depend on *how*
  the air was divided — only on total fuel + total air. And `T_mix ≈ Tt4` (within the η_b gap of
  §3): the zoned combustor mixes back to the same station-4 the cycle already computed. This is
  the central conservation gate — it only holds because the majors **re-equilibrate** on mix-out
  (freezing the dissociated primary composition would trap its energy and miss Tt4).
- **NO-mole conservation through dilution.** Dilution drops the NO *mole fraction* (NO_kin →
  NO_mix, e.g. 736 → 348 ppm at φ_p = 0.9) but conserves NO *moles*. Since all fuel and all NO
  are in the primary, **EI_NO (per kg fuel) is set in the primary and is unchanged by dilution**
  — a clean discriminating check that separates "concentration" from "emission index."
- **`K`-check still binds** at the primary T (~2300–2450 K, inside the rung-7 band), and the
  trace guard (`x_NO,e < 0.02`) still holds (peak NO_eq ≈ 5600 ppm = 0.0056).

---

## 5. What stays UN-anchored (state it plainly)

- **Rich primary / RQL.** Real high-power primaries run φ ≈ 1.0–1.5 rich; we cap at φ ≤ 1
  (lean stoichiometry only). The rich-burn → quick-quench → lean-burn combustor — the actual
  low-NOx architecture — is the **explicit next seam** (needs rich CO/H₂ stoichiometry).
- **Super-equilibrium `O` / prompt NO** (rung-7's deferred seam) is still deferred; the primary
  pool is rung-6 *equilibrium* `O`, so even the resolved primary underpredicts the true flame-
  front rate. Our EI_NO landing in-band is therefore partly the *lean-stoich cap* standing in
  for *rich + super-eq O* — stated, not claimed as a first-principles match.
- **Single-step, instantaneous mix-out** (no finite mixing rate, no secondary-zone Zeldovich in
  the cooling gas — NO is simply frozen at the primary value). Dilution-zone NO is second-order.
- **α (air split) is a knob**, like φ_p and τ — swept to show the mechanism, not derived from a
  specific combustor's airflow schedule.
- The EI_NO **band** (§1b) is an order-of-magnitude landing zone, **not** a to-the-digit book
  anchor (the absolute Zeldovich rate is un-pinned, per rung 7); what is certified is that
  resolving the primary lifts EI_NO by the right *orders of magnitude* into the measured range.

---

## Sources
- NASA TR 19730017094 — *Parameters Controlling Nitric Oxide Emissions from Gas Turbine
  Combustors* (primary-zone φ, near-stoichiometric NO peak):
  https://ntrs.nasa.gov/api/citations/19730017094/downloads/19730017094.pdf
- A. H. Lefebvre & D. R. Ballal, *Gas Turbine Combustion* — primary-zone equivalence ratio,
  RQL, NOx control.
- S. R. Turns, *An Introduction to Combustion* — thermal-NO / Zeldovich, EI_NO definition.
- ICAO Aircraft Engine Emissions Databank (EASA):
  https://www.easa.europa.eu/en/domains/environment/icao-aircraft-engine-emissions-databank
- Sheikhi et al. (2023), *Env. Progress & Sustainable Energy* — EI_NOx 18.4–64.4 g/kg across 24
  turbofans at take-off: https://aiche.onlinelibrary.wiley.com/doi/abs/10.1002/ep.13974
- lissys PUG ch.12 (Pollutant Emissions) — ICAO-style reference EI_NOx worked datum
  (52.7/36.2/7.5/4.3 g/kg): https://lissys.uk/pug/c12.html
