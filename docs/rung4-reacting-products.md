# Rung 4 — Reacting Products: Composition that Tracks the Fuel/Air Ratio

Rung 3 let `cp` vary with temperature but kept the **composition frozen**: the hot
section was one fixed lean-products mixture (`_PRODUCTS` at `f ≈ 0.030`), so `cp_t`,
`R_t`, `γ_t` were functions of `T` alone. Rung 4 makes the **product composition — and
therefore `cp_t`, `R_t`, `γ_t` — a function of the fuel/air ratio `f`**. Burn more fuel
and the burned gas the turbine and nozzle see genuinely changes: more CO₂ and H₂O, less
excess O₂, a heavier heat capacity. This is the seam rung 3 promised — *"the next gas
upgrade is just a new `Gas` behind the same four property functions."*

> **Read `docs/rung3-variable-cp.md` first.** This file states only what *changes*. The
> `h`/`pr` property interface, the `pr = exp(φ/R)` machinery, the CPG-closed-form /
> TPG-integral branch and its "trap," the dual-section split, the enthalpy datum
> `h(0)=0`, the two efficiency kinds, "derive before you code," and the conservation-
> assert contract all carry over unchanged.

---

## What rung 4 adds (and what it deliberately does not)

**Adds:**

- **Explicit lean-complete-combustion stoichiometry.** For a hydrocarbon fuel `(CH₂)ₙ`
  (Jet-A ≈ C₁₂H₂₃ ≈ (CH₂)ₙ), `CH₂ + 1.5 O₂ → CO₂ + H₂O`, burned lean (`f < f_stoich`)
  in dry air. The product **mole numbers** (CO₂, H₂O, excess O₂, N₂, Ar) are computed
  deterministically from `f` — the literal "CO₂/H₂O vs `f`." (`f_stoich = 0.0676` for
  `(CH₂)ₙ`, reproduced by the model to 0.15% — see `docs/plans/rung4-anchor-mattingly.md`.)
- **`f`-parameterized hot-section properties.** `cp_t(T,f)`, `h_t(T,f)`, `pr_t(T,f)`,
  `R_t(f)`, `γ_t(T,f)` and the inverses, built by mole-weighting the rung-3 NASA-7
  species over the `f`-dependent composition. Same integral `pr = exp(φ/R)` machinery —
  only the mixture coefficients now depend on `f`.
- **The implicit burner** (rung 4's load-bearing new mechanic — see § The implicit solve).
  Because `h_t` depends on `f`, the burner fuel balance becomes `f = g(f)`, solved by
  fixed-point iteration. This is **Mattingly's own** structure (Eq. 6.36 + his "iterative"
  note; `docs/plans/rung4-anchor-mattingly.md`).
- **A `Gas.reacting(f_design=…)` factory,** added *alongside* `Gas()` (CPG) and
  `Gas.thermally_perfect()` (frozen NASA-7). The frozen paths are untouched, so
  reduce-to-ideal stays free.

**Deliberately deferred (seams kept — `CLAUDE.md`):**

- **Rich combustion (`f > f_stoich`: CO, H₂) and high-temperature dissociation**
  (CO₂⇌CO, H₂O⇌OH, …). Both need a `T`-coupled equilibrium (Gibbs/`Kp`) solve. That is
  **rung 5**, on this same explicit-composition machinery.
- **Formation-enthalpy bookkeeping (Fork B).** Rung 4 stays **Fork A**: fixed `hPR`,
  `h(0)=0` sensible-style datum — composition(`f`) changes only the `cp`-shape, *not* the
  datum. Absolute/formation enthalpies (heat release derived, not assumed; adiabatic-
  flame-temperature anchored) pair naturally with dissociation → also **rung 5**. Rung 4
  keeps a **datum-agnostic seam** so Fork B is *additive* (add per-species `h̄_f°`), not
  a rewrite — see § The Fork-B seam.
- Off-design / component maps, the choked convergent nozzle, the afterburner — still out.

---

## The reacting-gas model (explicit composition, dual section)

The **cold** section is unchanged (air, `f = 0`, rung-3 NASA-7). The **hot** section
composition is computed from `f`:

```
# per 1 mol of dry air (mole fractions N₂ 0.7808, O₂ 0.2095, Ar 0.0093):
n_fuel = f · M_air / M_CH₂                      # mol (CH₂) burned per mol air
CH₂ + 1.5 O₂ → CO₂ + H₂O   (per mol fuel)
products:  N₂ : 0.7808
           Ar : 0.0093
           CO₂: n_fuel
           H₂O: n_fuel
           O₂ : 0.2095 − 1.5·n_fuel             # excess (lean ⇒ > 0; assert it)
```

From that composition (mole-weighting the rung-3 NASA-7 species polynomials, exactly as
`Gas.thermally_perfect()` does for the frozen mixture):

```
R_t(f)      = R_u / M_mix(f)
cp_t(T,f)   = R_t(f) · Σ_k A_k(f) · T^k          # A_k(f) = mole-weighted species coeffs
h_t(T,f)    = R_t(f) · ∫₀ᵀ Σ A_k(f) T'^k dT'      # datum h(0)=0 (shared cold/hot, rung 3)
φ_t(T,f)    = ∫ cp_t/T dT                         # analytic, piecewise across 1000 K
pr_t(T,f)   = exp(φ_t(T,f)/R_t(f))
T_from_h_t(h,f), T_from_pr_t(pr,f)               # guarded Newton, per rung 3
γ_t(T,f)    = cp_t / (cp_t − R_t(f))
```

*Why explicit composition (not a lumped `f`-blend correlation):* the rung the user picked
is *"variable-composition products (CO₂/H₂O/dissociation vs `f`)."* Computing real mole
numbers is the plain reading, keeps the rung ladder honest (composition now, equilibrium/
dissociation next on the same moles), and reuses rung 3's NASA-7 species with no second
air correlation. It reproduces Mattingly's own products example (Ex 6.3, `f=0.0338`) to
~0.02% on every datum-independent quantity (`ηt`, `Tt5`, `πt`) —
`docs/plans/rung4-anchor-mattingly.md`.

### The property interface gains an `f` argument (hot section only)
Downstream of the burner, `f` is fixed and **carried in `FlowState.far`** (rung 3 already
put it there). So the hot-section property calls thread it:
`gas.h_t(state.Tt, state.far)`, `gas.pr_t(...)`, `gas.T_from_h_t(...)`, etc. Non-reacting
sections (CPG, frozen-TPG) **ignore** the `far` argument — so the signature change is
additive and the reduce-to-ideal paths are untouched. Composition and its mole-weighted
coefficients are **memoized per `f`** (the burner solve calls `h_t(Tt4, f)` a few times
at nearby `f`; the cycle downstream calls at one fixed `f`).

---

## The implicit solve — rung 4's load-bearing new mechanic

Rung 3's burner is **explicit**: `h_t(Tt4)` is a fixed number (frozen composition), so
```
f = ( h_t(Tt4) − h_c(Tt3) ) / ( η_b·hPR − h_t(Tt4) )          # rung 3: one shot
```
solves directly. In rung 4 `h_t(Tt4, f)` **depends on `f`** (composition), so `f` appears
on both sides:
```
f = g(f) ≡ ( h_t(Tt4, f) − h_c(Tt3) ) / ( η_b·hPR − h_t(Tt4, f) )     # rung 4: implicit
```
*Why it converges (derive):* over the lean range the products `cp` differs from air `cp`
by only a few percent, and it enters `h_t` through the mass-blend weight `~f/(1+f)`, so
`∂h_t/∂f` is small and `|g'(f)| ≪ 1` — `g` is a contraction. **Fixed-point iteration**
`f_{k+1} = g(f_k)`, seeded from the rung-3 frozen-composition estimate `f₀`, converges
**linearly with contraction factor ≈ 0.1** — measured ~5–6 steps to a 1e-6 relative
tolerance, ~11 to 1e-12 from a cold `f₀=0.02` (the frozen seed cuts it further). Each
step is a handful of enthalpy evaluations, so the cost is trivial; a Newton step (using
`dh_t/dT = cp_t`) would give quadratic convergence if ever wanted, but is unnecessary.
*(Measured — the burner de-risk in `docs/plans/rung4-anchor-mattingly.md` § (3).)*

```
f ← f₀ (rung-3 frozen estimate)
repeat:
    h4 ← h_t(Tt4, f)
    f_new ← (h4 − h_c(Tt3)) / (η_b·hPR − h4)
    if |f_new − f| ≤ tol·f: break
    f ← f_new
assert converged within N_max   # STANDING conservation assert (rung-4 gate 3)
```

This is **Mattingly's Eq. 6.36 verbatim**, including his Note "*the value of `h_t4` is a
function of the fuel/air ratio `f`, and thus the solution … is iterative*" — so the
mechanic is anchored to the source book, not invented. It is the engine's job (the engine
already owns the shaft balance and holds the compressor states and `f`), exactly as the
rung-3 burner was. The converged `f` is written into `FlowState.far` and carried
downstream to every hot-section property call.

---

## Station equations — what changed from rung 3

**Only the burner changes in *form*.** Every other hot-section equation is unchanged;
it merely evaluates `h_t`/`pr_t`/`γ_t`/`R_t` at the carried `f` instead of at frozen
composition. Cold section (0→3) is entirely unchanged (`f = 0`, air).

### 4 — Burner: implicit fuel balance (the one new solve)
```
pt4 = π_b · pt3
f   : solve  f = ( h_t(Tt4, f) − h_c(Tt3) ) / ( η_b·hPR − h_t(Tt4, f) )   (fixed point)
```
Reduces to the rung-3 explicit form the instant `h_t` stops depending on `f` (frozen
composition) — i.e. one iteration converges. `hPR` is fixed (Fork A).

### 5 — Turbine: shaft balance in Δh, evaluated at the burned-gas `f`
```
Δh   = ( h_c(Tt3) − h_c(Tt2) ) / ( η_m·(1 + f) )
Tt5  = T_from_h_t( h_t(Tt4, f) − Δh,  f )
… η_c/e_c and η_t/e_t knobs exactly as rung 3, with every h_t/pr_t call carrying f …
```
Unchanged in form; `f` is now the *converged* burner value.

### 9 — Nozzle: `pr`/`γ`/`R` at `f`
```
T9  = T_from_pr_t( pr_t(Tt9, f) · (p9/pt9),  f )
V9  = √( 2·( h_t(Tt9, f) − h_t(T9, f) ) )
a9  = √( γ_t(T9, f)·R_t(f)·T9 ),   M9 = V9/a9
```
Unchanged in form; `R_t` and `γ_t` now depend on `f`.

The **shaft balance** (engine-owned keystone) is rung-3's, with the hot enthalpies at `f`:
```
η_m·(1 + f)·[ h_t(Tt4, f) − h_t(Tt5, f) ] = h_c(Tt3) − h_c(Tt2)
```

---

## The Fork-B seam (keep rung 5 additive)

Rung 4 is **Fork A**: `hPR` fixed, `h(0)=0` sensible-style datum — the composition(`f`)
only reshapes `cp_t`. Keep the property layer **datum-agnostic** so rung 5's Fork B is a
*local* addition, not a rewrite:

- Enthalpies are `h_species(T) = ∫₀ᵀ cp dT` (sensible, datum 0). Fork B adds a per-species
  **formation enthalpy** `h̄_f°`: `h_species(T) = h̄_f° + ∫ cp dT`. Everything else — the
  mole-weighting, `pr`, the inverses, the components — is untouched; only the burner swaps
  from `f = g(f)` with fixed `hPR` to the `Σ N·h̄ = Σ N·h̄` balance (heat release derived).
- The composition solver (rung 4, deterministic complete combustion) becomes the *seed*
  for rung 5's `T`-coupled equilibrium (dissociation) solve — same mole-number machinery.

So rung 5 = Fork B **+** equilibrium/dissociation, both landing on rung 4's explicit
composition. Rung 4 writes the substrate once; rung 5 adds the two ideas that share it.

---

## Verification gates (priority order)

1. **Reduce-to-ideal (load-bearing, unchanged).** `Gas()` (CPG) and dual-CPG reproduce
   the rung-1/2/2b tables **to the digit**; the existing suites (`test_validation`,
   `test_rung2`, `test_polytropic`, `test_variable_cp`) **stay green and untouched**.
   Guaranteed because the reacting gas is a *separate* `Gas.reacting(f)` path.
2. **Stoichiometry hand-check.** `f_stoich((CH₂)ₙ) = 0.0676 ± 0.15%`; at `f=0.0338` the
   product mole fractions match the hand-derived values (N₂ 0.7548, O₂ 0.1014, CO₂/H₂O
   0.0674, Ar 0.0090); excess-O₂ `> 0` asserted for every lean `f` (the lean guard).
3. **Implicit-solve convergence + cross-datum de-risk (the new mechanic's gate).**
   `f = g(f)` contracts linearly (factor ≈ 0.1); the residual `|f_new − f|` is asserted
   `≤ tol` within a bounded step count on every run. Direction check: `f` rises with
   `Tt4` and falls with `Tt3`. **Cross-datum check (the burner is the one datum-dependent
   step):** the `h(0)=0` production model reproduces Mattingly's full-datum McKinney
   burner `f` to **0.17%** (href offset worth only 0.19%), so `hPR=42.8 MJ/kg` is
   consistent with no recalibration — measured in `docs/plans/rung4-anchor-mattingly.md`
   § (3), and worth a permanent test since no existing gate exercises the cross-section
   enthalpy subtraction.
4. **Mattingly Ex 6.3 products anchor (primary, sourced).** Explicit `(CH₂)ₙ`
   stoichiometry at `f=0.0338` reproduces the **datum-independent** results — `ηt=0.9057`,
   `Tt5=2677.52 °R`, `πt=0.5650` — to **~0.05%** (measured 0.001–0.02%;
   `docs/plans/rung4-anchor-mattingly.md`). This *exercises the real rung-4 stoichiometry
   + property code*.
5. **McKinney test-only cross-check (exact digit anchor).** A small in-test implementation
   of Mattingly's Table 2.2 `f`-blend (English units, certified coeffs in the anchor doc)
   reproduces Ex 2.7 (`T2=627.57 K`, air) and Ex 6.3 (`h=828.75 Btu/lbm`, products) **to
   the digit**, and its `Pr` to ~0.1%. Confirms the anchor numbers independently of the
   production stoichiometry model. *(Not production code — a validation scaffold only.)*
6. **`f`-sweep directional / physical checks.** As `f` rises (still lean): `Tt4` capacity,
   thrust, and TSFC move the right way; `cp_t`, CO₂/H₂O fractions rise, excess O₂ falls;
   `R_t(f)` **increases** slightly with `f` — each mol fuel swaps 1.5 O₂ for one CO₂ **and
   one light H₂O**, so the mean molar mass *falls* and `R = R_u/M` rises. (This corrects an
   earlier "decreases (heavier products)" reading; the measured `R_t=287.4` at `f=0.0338`
   vs `R_air≈287.1`, and Mattingly's own `R(f)=1.9857/(28.97−0.946·f)`, both rise with `f`
   — see `docs/plans/rung4-anchor-mattingly.md`.) Round-trip inverses and monotonicity
   (rung-3 gate 2) hold at the swept `f`.

## Conservation asserts (rung-4 deltas)
Carry over rung 3's, plus:
- **Fixed-point residual** (gate 3) is a standing assert.
- **Lean guard:** excess `O₂ > 0` (composition valid) — rich `f` is out of scope and must
  trip, not silently produce garbage.
- **Mole/atom conservation** in the stoichiometry (C, H, O balance) asserted when the
  composition is built.
- Burner / shaft / nozzle energy balances in **enthalpy**, now at the carried `f`.

## Done when
Reduce-to-ideal reproduces rung-1/2/2b to the digit (existing suites untouched, green);
the stoichiometry, implicit-convergence, and `f`-sweep asserts hold; the **Mattingly
Ex 6.3** products anchor matches to ~0.05% and the **McKinney** cross-check to the digit
(both verified before coding). `main.py` gains an `f`-sweep / frozen-vs-reacting table
(composition and `cp_t` moving with `f`); `NOTES.md` gains a rung-4 section (why frozen
composition worked, what `composition(f)` buys, why the burner had to go implicit, the
Fork-A/Fork-B datum split). `CLAUDE.md` scope + the deferred-seams list updated (Fork B +
equilibrium/dissociation = rung 5).
