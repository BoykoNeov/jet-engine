# Rung-5 external anchors — formation enthalpies, derived heat release (Fork B)

Verified constants + worked checks that anchor the **formation-enthalpy bookkeeping**
of rung 5 (Fork B): absolute (formation + sensible) enthalpies on the standard
reference (elements = 0 at 298.15 K), a **derived** lower heating value, and the
adiabatic energy balance that replaces rung 4's *assumed* `hPR`. Every number was
re-derived and **certified before any production code** (project discipline).

The mechanism (the advisor's framing, confirmed): Fork B **restores the additive
formation constant that rungs 3/4 deliberately dropped.** The NASA-7 species carry a
sixth coefficient `a6` in `H/RuT = a1 + a2 T/2 + … + a5 T⁴/5 + a6/T`; since rung 3's
`_antideriv_h` is exactly the polynomial part *with no constant*, `Ru·a6` is precisely
the additive formation term. It **cancels in every enthalpy difference** (turbine Δh,
nozzle `h(Tt9)−h(T9)`), so **only the burner** — the one cross-section subtraction —
sees it. (`a7`, the absolute-entropy constant, is left for rung 6's `Kp` solve.)

---

## 1. Formation enthalpies — sourced constants (CODATA/JANAF, 298.15 K)
Standard molar enthalpies of formation, J/mol:

| species | ΔH̄f°(298.15) | note |
|---|---|---|
| N₂, O₂, Ar | 0 | elements — the reference datum |
| CO₂ (g) | −393 520 | CODATA |
| H₂O (**gas**) | −241 826 | CODATA — **vapour ⇒ LHV**, not liquid (−285 830 = HHV) |

**Deriving `a6` instead of transcribing it (teaching-honest, and it reuses the
certified rung-3 polynomials).** Rather than copy a sixth NASA column (a fresh
transcription risk), pin each species' formation constant to its ΔH̄f°:

```
H̄(T) = Ru·( antideriv_h(A,T) + a6 )                     [antideriv_h = ∫(Cp/Ru)dT]
require  H̄(298.15) = ΔH̄f°   ⇒   a6 = ΔH̄f°/Ru − antideriv_h(A_low, 298.15)
```

Self-check (by construction, verified numerically): `H̄(298.15)` reproduces ΔH̄f° to
the last digit for all five species; the elements N₂/O₂/Ar land at **h = 0 at
298.15 K** — the absolute datum, cleanly replacing rung 3/4's `h(0) = 0`.

Derived low-range `a6` (units K), for the record:
`N₂ −1021.07, O₂ −1063.94, Ar −745.38, CO₂ −48373.44, H₂O −30293.89`.

---

## 2. Derived LHV = Mattingly's assumed `hPR` (the primary book anchor)
Rung 4 **assumed** `hPR = 42.8 MJ/kg` (Mattingly's 18400 Btu/lbm). Rung 5 **derives**
the heat release from formation enthalpies. For `CH₂ + 1.5 O₂ → CO₂ + H₂O(gas)`:

```
LHV_molar = ΔH̄f°(CH₂) − ΔH̄f°(CO₂) − ΔH̄f°(H₂O,gas)      (reactants − products, O₂=0)
LHV_mass  = LHV_molar / M_CH₂        M_CH₂ = 14.027 g/mol
```

The fuel formation enthalpy is the **one calibration input** (it carries the same
information `hPR` did — Fork B does not conjure heat). Pinning it so the derived LHV
equals Mattingly's value:

| quantity | value |
|---|---|
| target `hPR` (rung 4) | 42.8 MJ/kg = 18400 Btu/lbm |
| ⇒ **ΔH̄f°(CH₂)** | **−34.99 kJ/mol** |
| round-trip LHV | **42.8000 MJ/kg** (exact) |

`ΔH̄f°(CH₂) ≈ −35 kJ/mol` is physically reasonable for a liquid-hydrocarbon `(CH₂)`
unit (Benson group additivity puts a gaseous −CH₂− increment near −21 kJ/mol; liquid
is more negative; Jet-A LHV is quoted at 42.8–43.2 MJ/kg). We do **not** over-claim a
first-principles fuel enthalpy — it is the calibration knob, stated plainly.

---

## 3. Exact reduce-to-rung-4 equivalence (the load-bearing gate)
**Finding — tighter than first expected.** The rung-4 anchor doc §(3) measured a
0.17% burner-`f` gap, but that compared two *different* property correlations (NASA
`h(0)=0` vs Mattingly's McKinney table *with* its `href` offsets). Fork A and Fork B
here use the **same** NASA polynomials, so the datum difference cancels **exactly**.

Proof (verified numerically, gap ~1e-7 relative = floating-point): the released
chemical energy per kg air is *identically* `f·LHV` for all `f` —
```
f·h̄f_fuel/M − (1+f)·h̄f_products(f)  ≡  f · LHV         (only CO₂,H₂O carry formation)
```
because mass conservation gives `(1+f) = Σ n_i M_i / M_air`. Therefore the Fork-B
absolute-enthalpy balance
```
(1+f)·h_t0(Tt4,f)  =  h_c0(Tt3)  +  η_b · [derived release]
```
is **algebraically identical** to rung-4 Fork A with `hPR := LHV`. Measured at
`Tt3=800 K, Tt4=1600 K, η_b=0.99`:

| model | burner `f` |
|---|---|
| Fork A (rung 4, datum-0, hPR=42.8) | 0.024537911518 |
| Fork B (derived LHV from formation) | 0.024537911518 |
| |gap| | **0.0e0 (machine zero)** |

So rung 5's reduce-to-rung-4 gate is **exact**, not approximate — the reacting-Fork-A
and reacting-Fork-B cycles produce bit-identical `f`, `Tt5`, thrust, TSFC. **The value
of Fork B is purely structural:** heat release now lives on the absolute formation
scale, so (a) it varies automatically with product composition and (b) it is the
substrate rung 6's equilibrium/dissociation needs (`Kp = exp(−ΔG°/RuT)` requires
absolute `h̄` and `s̄`). We do not oversell "derived, not assumed": for complete
combustion to fixed products the numbers are identical *by construction*.

---

## 4. Adiabatic flame temperature — physical-plausibility anchor + rung-6 motivation
No local Çengel PDF and no worked AFT example in the Mattingly/Farokhi PDFs, so AFT is
a **physical** anchor, not a book digit. Constant-pressure adiabatic balance
`Σ N h̄(T_react) = Σ N h̄(T_flame)`, reactants at 298.15 K, **no dissociation**:

| `f` | T_flame (K) |
|---|---|
| 0.0200 | 1059 |
| 0.0338 | 1493 |
| 0.0500 | 1942 |
| 0.0676 (≈stoich) | 2375 |

Monotone in `f` and the stoichiometric value (2375 K) sits in the right band. **It is
deliberately HIGH:** real kerosene–air stoichiometric flame temperature is ~2200–2300 K
because CO₂/H₂O **dissociate** (CO₂⇌CO+½O₂, H₂O⇌OH+½H₂, …) — endothermic, capping the
peak. Our complete-combustion model cannot see that. **That gap is exactly rung 6's
job**, and it is why Fork B (absolute enthalpies) has to come first: the equilibrium
that lowers the flame temperature is driven by the formation enthalpies restored here.

---

## Tolerances (from the measured gaps, not guessed)
- **Derived LHV vs Mattingly hPR:** exact by construction (the fuel enthalpy is pinned
  to it); assert `|LHV − 42.8e6| < 1e-6·42.8e6`.
- **Formation self-check** `H̄(298.15)=ΔH̄f°`: to ~1e-6 relative.
- **Reduce-to-rung-4 equivalence:** Fork-B `f` == reacting-Fork-A `f` to ~1e-9
  relative (measured machine-zero) — the load-bearing gate.
- **Reduce-to-ideal (unchanged, tight):** `Gas()` / dual-CPG and the frozen/reacting
  Fork-A paths reproduce rungs 1–4 to the digit — Fork B is a *separate* factory, so
  every existing suite stays green untouched.

## What stays UN-anchored (state it plainly)
The fuel enthalpy `ΔH̄f°(CH₂)` is a **calibration input**, not first-principles (same
status `hPR` had). H₂O is taken as **vapour** (LHV convention). **Complete** combustion
only — no dissociation, no CO/H₂ (rung 6). AFT has no book-digit anchor (physical
plausibility only). `a7`/absolute entropy and the equilibrium `Kp` solve are rung 6.
