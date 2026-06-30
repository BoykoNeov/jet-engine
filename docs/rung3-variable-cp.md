# Rung 3 — Variable `cp(T)`: the Thermally-Perfect Gas

Rungs 1–2b modeled the working fluid as **calorically perfect**: `cp`, `γ`, `R`
constant within a section (rung 2 split it into a *cold* and a *hot* section, but
each was still a single constant triple). Rung 3 lets **`cp` vary with
temperature** — a **thermally-perfect gas** (still an ideal gas, `p = ρRT`, with
`R` per section constant, but `cp = cp(T)` and therefore `γ = γ(T)`). This is the
first rung where the simple isentropic power law `Tt/Tt = (pt/pt)^g` **stops being
exact**, so it forces the move from the algebraic `cp·T` / `π^g` forms to the
**property-function** forms the gas-table / pyCycle / NPSS lineage actually uses.

> **Read `docs/rung2-spec.md` and `docs/rung2b-polytropic.md` first.** This file
> states only what *changes*. Station order, totals-vs-static convention, the two
> efficiency kinds, the dual-section split, the "derive before you code" and
> conservation-assert contracts all carry over unchanged.

---

## What rung 3 adds (and what it deliberately does not)

**Adds:**

- **Temperature-dependent `cp(T)` per section.** Cold section = **air** modeled
  with **NASA 7-coefficient species polynomials** (a fixed N₂/O₂/Ar mixture); hot
  section = combustion products as a **single representative `cp(T)`** at a
  fixed composition. The rung-2 dual-section structure is preserved exactly — each
  section's *constant* `cp` is simply promoted to a *function* `cp(T)`.
- **A property interface on `Gas`,** four functions per section:
  `h(T)` (enthalpy), `pr(T)` (reduced pressure `= exp(φ(T)/R)`, where
  `φ(T) = ∫ cp(T)/T dT` is the entropy function), and the two **inverses**
  `T_from_h(h)` and `T_from_pr(pr)`. Every component talks to these instead of to
  `cp` and `g` directly.
- **Components reworked in `h` / `pr` form.** `cp·T` → `h(T)`; `π^g` → ratios of
  `pr`. This is where the change lives — the SPEC's hope that "components are
  untouched" does **not** hold (see § The honest scope note).
- **`γ` now varies** — the nozzle's static drop and speed of sound use `γ(T)`
  evaluated at the *local* temperature, not one cycle-wide `γ`.

**Deliberately deferred (seams kept — see `CLAUDE.md`):**

- **Composition tracking / reacting gas.** The hot section uses one representative
  `cp(T)`; it does *not* recompute product composition (CO₂/H₂O/excess-O₂) as a
  function of fuel-air ratio, nor high-temperature dissociation. That is the *next*
  rung after this one (SPEC § A, "reacting gas"). Keeping a fixed-composition
  `cp(T)` is the rung-2 spirit (one hot triple) upgraded to one hot *function*.
- **Off-design / component maps, the choked convergent nozzle, the afterburner** —
  all still out of scope.

---

## The gas model (thermally perfect, dual section)

Each section answers the same four questions; the *cold* section uses air, the
*hot* section uses products:

```
h(T)        = h_ref + ∫_{Tref}^{T} cp(T') dT'           # enthalpy,            J/kg
φ(T)        =          ∫_{Tref}^{T} cp(T')/T' dT'        # entropy function,   J/(kg·K)
pr(T)       = exp(φ(T)/R)                                # reduced pressure,   dimensionless
T_from_h(h) : invert h(·)        (Newton + bisection fallback; h monotone↑)
T_from_pr(p): invert pr(·)       (Newton + bisection fallback; pr monotone↑)
```

`cp(T) > 0` makes both `h` and `pr` strictly increasing, so the inverses are
well-posed and a guarded Newton (bisection fallback) converges from any bracket.

**Why `pr` is the right primitive.** For *any* process between states 1 and 2 of a
thermally-perfect gas, the entropy change is
`Δs = φ(T2) − φ(T1) − R·ln(p2/p1)`. Set `Δs = 0` (isentropic) and it collapses to
`p2/p1 = pr(T2)/pr(T1)` — the gas-table relation. So **every** isentropic
pressure↔temperature step in the cycle is one `pr` ratio, and both efficiency
knobs ride on it:

```
isentropic, ratio π:        pr(T2) = pr(T1)·π                  → T2 = T_from_pr(pr(T1)·π)
polytropic compressor, e:   pr(T2) = pr(T1)·π^(1/e)            (derivation below)
polytropic turbine, e:      p2/p1  = (pr(T2)/pr(T1))^(1/e)     (T2 known from the shaft)
```

This generalizes the calorically-perfect forms exactly: when `cp` is constant,
`pr(T) = T^(cp/R)` and `pr(T1)·π^(1/e)` reproduces `T1·π^(g/e)` (since `g = R/cp`
in that limit) — see the trap below for the one subtlety.

### Air via NASA 7-coefficient polynomials

```
Cp_molar,i(T)/Ru = a1 + a2·T + a3·T² + a4·T³ + a5·T⁴        # per species i, Ru = 8.3145 J/(mol·K)
cp_air(T) = (Σ_i x_i·Cp_molar,i(T)) / M_air                 # mass-specific, mole fractions x_i
M_air     = Σ_i x_i·M_i  ≈ 28.965 g/mol  →  R_c = Ru/M_air ≈ 287.0 J/(kg·K)
```

Dry-air mole fractions: N₂ 0.7808, O₂ 0.2095, Ar 0.0093, CO₂ trace (Ar is
monatomic, `Cp/R = 2.5` constant). The N₂/O₂ coefficients are standard
NASA/GRI-Mech values; **they are certified by the air-table gate below** (the model
must reproduce Keenan-Kaye `Pr` ratios to ~0.1%), so a transcription slip cannot
pass silently. `R_c ≈ 287` falls out of the air molar mass, consistent with the
`R_c = 287` of rungs 1–2.

### Hot products

A single representative `cp_prod(T)` polynomial at fixed (stoichiometric-ish)
composition, with `R_t` per its molar mass. (The rung-2 anchor used `cp_t = 1239`,
`R_t = 285.9`; the rung-3 hot polynomial is chosen to pass through that
neighborhood.) **Honest scope:** the external anchors (Çengel, Mattingly Ex 2.7/2.8)
are all single-gas **air**, so they pin the *air* `cp(T)` end-to-end (up to 1240 K)
but pin **nothing** in the products section — the **products coefficients float**.
For a teaching tool that is acceptable, but it is stated plainly rather than implied
to be anchored. (Anchoring products to the digit would need a reacting-gas source —
that is the next rung.)

---

## The trap that fixes the architecture — keep the CPG branch closed-form

This is the load-bearing design decision, and it is flagged in `gas.py`'s own
docstring. Rungs 1–2 reproduce their validation tables **to the digit** using
`g = (γ−1)/γ` with `γ = 1.4`, i.e. exponent `0.28571`, and a *rounded* `R = 287`
that is ~0.05% off the relation `R = (γ−1)/γ·cp`. But the thermally-perfect
reduced pressure uses `pr = exp(φ/R)`, whose constant-`cp` limit is `pr = T^(cp/R)`
with exponent `R/cp = 287/1004 = 0.28586`. Those two exponents **differ by that
same 0.05%**:

```
cold:  (γ−1)/γ = 0.28571      vs      R/cp = 287/1004 = 0.28586
```

**Consequence:** route a *constant-cp* gas through the integral/`pr` path and it
lands ~3e-4 off the γ-based answer (`Tt3` 552.4 → ~552.6 K). That survives the
loose 1e-3 table checks but **violates "to the digit," threatens the rung-2
Mattingly 5e-4 margin, and breaks any tight (1e-9) reduce-to-ideal gate.** So:

- **A calorically-perfect section keeps the rung-1/2 closed forms exactly**
  (`h = cp·T`, `pr = T^(1/g)` with `g = (γ−1)/γ`, closed-form inverses). γ-based,
  bit-for-bit the existing behavior.
- **A thermally-perfect section integrates** (`h`, `φ` analytic from the
  polynomial; inverses numerical). `R/cp(T)`-based, as physics dictates.

The branch hides **inside `Gas`** (a section is CPG if it carries a constant
triple, TPG if it carries a `cp(T)`); components call the one interface and never
see which. All existing anchors hold because reduce-to-ideal selects the CPG
branch.

**Discriminating check (build for it):** feed a *constant-cp polynomial* (a TPG
section whose `cp(T)` happens to be flat) through the integral path. It must
reproduce rung-1 `Tt3` only to ~3e-4, **not** 1e-9 — that ~3e-4 residual is the
proof you are exercising the integral path and that the exact reduce-to-ideal is
correctly routed through the *closed-form* CPG branch instead. (If a constant-cp
polynomial reproduced 1e-9, the integral path would secretly be using `(γ−1)/γ`,
i.e. it is not really the `pr = exp(φ/R)` machinery.)

---

## Station equations (what changed from rung 2)

Below, `h_c/pr_c` are cold-section property functions, `h_t/pr_t` hot. Each reduces
to its rung-2 form on a CPG section.

### 0 — Freestream (cold): ram totals from `h`, not `T·(1+…M²)`
```
V0  = M0·a0,   a0 = √(γ_c(T0)·R_c·T0)            # γ now T-dependent (≈1.40 cold, but evaluated)
Tt0 = T_from_h_c( h_c(T0) + V0²/2 )               # stagnation enthalpy → total temperature
pt0 = p0 · pr_c(Tt0)/pr_c(T0)                      # isentropic ram: pr ratio
```
*Why:* the rung-1 `Tt0 = T0(1+(γ−1)/2·M0²)` is a *calorically-perfect* identity.
The general statement is that bringing the flow isentropically to rest conserves
*stagnation enthalpy* (`h(Tt0) = h(T0) + V0²/2`) and entropy (`pr` ratio sets
`pt0`). At constant `cp` both collapse back. `a0` uses `γ(T0)`.

### 2 — Inlet: unchanged in form
```
Tt2 = Tt0,    pt2 = π_d · pt0
```
Adiabatic work-free duct conserves `Tt` (hence stagnation enthalpy); `π_d` is the
specified recovery. No property change.

### 3 — Compressor: `pr` substate, η on **enthalpy**, polytropic via `pr^(1/e)`
```
pt3   = πc · pt2
Tt3s  = T_from_pr_c( pr_c(Tt2) · πc )              # IDEAL substate at pt3 (pr ratio)
ISENTROPIC knob:
  h3   = h_c(Tt2) + (h_c(Tt3s) − h_c(Tt2))/η_c     # η defined on ENTHALPY, not ΔT
  Tt3  = T_from_h_c(h3)
POLYTROPIC knob:
  Tt3  = T_from_pr_c( pr_c(Tt2) · πc^(1/e_c) )      # native: pr exponent carries the loss
```
*Why (polytropic, derive):* each infinitesimal stage is isentropic-efficient `e_c`,
so `e_c·dh = T·ds_ideal`; with `dh = cp dT` and the ideal `R·dp/p` this integrates
to `e_c·[φ(T2) − φ(T1)] = R·ln(πc)`, i.e. `pr(Tt3) = pr(Tt2)·πc^(1/e_c)`. At
constant `cp` this is `Tt2·πc^(g/e_c)` — the rung-2b form. *Why (isentropic):*
the η definition is fundamentally an **enthalpy** ratio (ideal work / actual work);
rung 2's `Tt3 = Tt2 + (Tt3s−Tt2)/η_c` was the constant-`cp` shadow of
`h3 = h2 + (h(Tt3s)−h2)/η_c`.

### 4 — Burner: enthalpy energy balance
```
pt4 = π_b · pt3
f   = ( h_t(Tt4) − h_c(Tt3) ) / ( η_b·hPR − h_t(Tt4) )
```
*Why:* the steady-flow balance `ṁa·h_c(Tt3) + η_b·ṁf·hPR = (ṁa+ṁf)·h_t(Tt4)`,
divided by `ṁa`. The `cpt·Tt4`, `cpc·Tt3` of rung 2 become `h_t(Tt4)`, `h_c(Tt3)`.
Reduces exactly when `h = cp·T`.

### 5 — Turbine: shaft balance in **Δh**, then `pr` (signature change)
```
Δh   = ( h_c(Tt3) − h_c(Tt2) ) / ( η_m·(1 + f) )    # SHAFT BALANCE in enthalpy (engine-owned)
Tt5  = T_from_h_t( h_t(Tt4) − Δh )                  # actual exit (shaft-set, knob-free)
ISENTROPIC knob:
  h5s  = h_t(Tt4) − Δh/η_t                           # ideal-work enthalpy
  Tt5s = T_from_h_t(h5s);   pt5 = pt4·pr_t(Tt5s)/pr_t(Tt4)
POLYTROPIC knob:
  pt5  = pt4·( pr_t(Tt5)/pr_t(Tt4) )^(1/e_t)         # native: Tt5 known → pt5 direct
  Tt5s = T_from_pr_t( pr_t(Tt4)·(pt5/pt4) )          # diagnostic substate
```
*Why (shaft):* the rung-2 balance `η_m(1+f)cpt(Tt4−Tt5) = cpc(Tt3−Tt2)` is an
*enthalpy* balance in disguise: `η_m(1+f)·[h_t(Tt4)−h_t(Tt5)] = h_c(Tt3)−h_c(Tt2)`.
So the engine now hands the turbine a **`Δh`, not a `ΔTt`** — `Turbine.apply`'s
signature changes from `(state, gas, delta_Tt)` to `(state, gas, delta_h)`. `Tt5`
follows by inverting `h_t`. (At constant `cp`, `Δh = cpt·ΔTt` and the rung-2
`ΔTt = cpc(Tt3−Tt2)/(η_m(1+f)cpt)` is recovered.) Polytropic stays the natural
turbine knob: `Tt5` is fixed by the shaft before any efficiency, so `pt5` is one
`pr` line.

### 9 — Nozzle: `pr` static drop, `V9` from `h`, `a9` from `γ(T9)`
```
Tt9 = Tt5,   pt9 = π_n · pt5,   p9 : given (default p0)
T9  = T_from_pr_t( pr_t(Tt9) · (p9/pt9) )            # isentropic total→static: pr ratio
V9  = √( 2·( h_t(Tt9) − h_t(T9) ) )                  # stagnation-enthalpy split
a9  = √( γ_t(T9)·R_t·T9 ),    M9 = V9/a9             # γ at the LOCAL exit temperature
```
*Why:* the rung-2 `M9 = √(((pt9/p9)^gt−1)/((γt−1)/2))` and `V9 = M9√(γt Rt T9)` bake
in one cycle-wide `γt`. With `γ = γ(T)`, the honest statements are: the
total→static expansion is isentropic (`pr` ratio gives `T9`), the energy that
appears as kinetic is the *enthalpy* drop (`V9² = 2(h(Tt9)−h(T9))`), and the Mach
number needs the *local* sound speed `a9 = √(γ(T9)·Rt·T9)`. The energy-split assert
`h(Tt9) = h(T9) + V9²/2` is now **exact by construction** (it was loose in rung 2
because `cp·T` carried the rounded-constant residual).

---

## The shaft balance (enthalpy form)

Rung 2's keystone, with `cp·ΔT` promoted to `Δh`:
```
η_m·(1 + f)·[ h_t(Tt4) − h_t(Tt5) ]  =  h_c(Tt3) − h_c(Tt2)
```
Left = useful turbine power to the shaft (hot, heavy, minus friction); right =
compressor power demanded (cold). The engine owns this and the closure assert (it
alone holds the compressor states and `f`), exactly as in rung 2 — only the
quantity passed in changes from `ΔTt` to `Δh`.

---

## Performance — mostly carried over, two notes

`F/ṁ`, TSFC, `eta_thermal`, `eta_propulsive`, `eta_overall`, and the cascade
closure `eta_o == eta_thermal·eta_p` are unchanged in *form* (they are written in
`V9`, `V0`, `f`, `hPR`, `Rt`, `T9` — no `cp·T`), so they need no rework. Two
caveats:

- **`eta_brayton = 1 − Tt2/Tt3` is no longer `1 − 1/πc^gc`.** That equality was a
  calorically-perfect identity. With variable `cp` the isentropic `Tt3` no longer
  obeys the power law, so the rung-1 **primary hand-check fires only on a CPG
  section** (reduce-to-ideal). On a TPG run `eta_brayton` is still *reported* (a
  Brayton-style diagnostic) but is no longer pinned to the closed form — say so.
- **Polytropic implied-η cross-check is CPG-only in closed form.** The rung-2b
  assert `η_c == (πc^gc−1)/(πc^(gc/e_c)−1)` is a constant-`cp` result. On a TPG
  section the implied isentropic efficiency is the **enthalpy ratio**
  `(h(Tt3s)−h(Tt2))/(h(Tt3)−h(Tt2))` with no elementary closed form; assert it lies
  in `(0, 1]` and that the entropy-generation inequality `Tt3 ≥ Tt3s` holds, and
  keep the *exact* closed-form cross-check scoped to CPG sections.

---

## Verification gates (in priority order)

1. **Reduce-to-ideal (load-bearing).** A CPG `Gas()` (constant triple) with all
   efficiencies/ratios = 1 reproduces the **rung-1 table to the digit**, and a dual
   CPG gas reproduces the **rung-2 Mattingly Ex 7.1** to its existing 5e-4 — i.e.
   the existing `tests/test_validation.py`, `test_rung2.py`, `test_polytropic.py`
   **stay green untouched**. This is guaranteed by routing CPG sections through the
   closed-form branch (§ the trap).
2. **Round-trip inverses (new conservation assert).** `T_from_h(h(T)) == T` and
   `T_from_pr(pr(T)) == T` to ~1e-10 relative, across the working range, on every
   run. Plus monotonicity (`pr`, `h` strictly increasing).
3. **Discriminating CPG-vs-integral check — run it DUAL-SECTION.** Feed *constant-`cp`
   polynomials* through the integral path and reproduce the prior-rung result to
   ~3e-4 (not 1e-9) — proving the integral path is genuinely `pr = exp(φ/R)` and that
   exact reduce-to-ideal takes the closed-form branch (§ the trap). Use **two
   distinct flat `cp` polynomials** (a cold and a hot constant) and reproduce the
   **rung-2 dual-cp turbojet** to ~3e-4 — a *single* flat polynomial through a
   unified gas cannot catch **cold/hot section-confusion** (a component calling
   `pr_c` where `pr_t` belongs), and neither can the single-gas Çengel/Mattingly
   anchors. **Residual gap (flag honestly):** what two flat polynomials still can't
   see is a `γ`-evaluated-at-the-wrong-temperature bug (it vanishes when `γ` is
   flat). That only affects the *reported* `M9` (diagnostic), not thrust; a
   directional sanity check on `M9` plus the Mattingly Ex 2.8 nozzle anchor (which
   does vary `γ`) is the cover for it.
4. **Air-table isentropic anchor (sourced).** Isentropic compression of air,
   `πc = 10` from `T1 = 300 K`, must land at the Keenan-Kaye / Wark gas-table value
   **`T2 ≈ 574.1 K`** (vs the calorically-perfect `300·10^0.2857 = 579.2 K`). The
   check uses the datum-independent `pr` ratio (`pr(T2) = 10·pr(300 K)`), so it is
   immune to the table's enthalpy/entropy datum. Spot-check a second point (e.g.
   `T1 = 290 K`, `Pr1 = 1.2311`) for safety. *(Source: standard air gas table after
   Keenan & Kaye / Wark — `Pr(300 K) = 1.3860`, `Pr(580 K) = 14.38`,
   `Pr(570 K) = 13.50`; interpolating `Pr2 = 13.86` gives ≈574.1 K.)*
5. **External machinery anchors — Çengel + Mattingly variable-`cp` examples (SOURCED).**
   See `docs/plans/rung3-anchor-cengel.md`. **Çengel 9-89** (air, Table A-17, η_c=0.83,
   η_t=0.87, π=10): isentropic compression 295 K → `T2s = 564.9 K`, expansion 1240 K →
   `T4s = 689.6 K`, cycle `η_th = 0.301`. **Mattingly Ex 2.7** (compression, `T2 = 627.57 K`)
   and **Ex 2.8** (nozzle, `T2 = 2377.7 °R`, `P2/P1 = 0.3757`). The NASA-7 air model
   reproduces all of them to **≤ 0.11%** (verified before coding) — set the test
   tolerance from that gap (~0.15% / ±1 K), *not* the 1e-9 reduce-to-ideal bound.
   ⚠ **Topology caveat:** the Çengel cases are Brayton *power* cycles (turbine expands
   the full π, `w_T > w_C`), so they **cannot** be matched through `build_turbojet`
   end-to-end — they anchor the *property + process* machinery (the station-3/5/9
   substate math), not the turbojet shaft balance/thrust. The full turbojet topology
   is anchored by reduce-to-ideal (CPG); its TPG path by the dual-section gate 3.
6. **Directional / asymmetry checks carry over** (losses move thrust/TSFC/η the
   right way; `η_c < e < η_t` at the anchor `πc`), now on the variable-`cp` gas.

### Anchor RESOLVED — no Mattingly variable-`cp` turbojet exists (gate 5)

Reading the local Mattingly *Elements of Propulsion* PDF directly (text layer via
`pypdf`; `Read`'s PDF render needs poppler, absent on Windows) settled the open
question: **Mattingly has no worked variable-`cp` turbojet cycle.** "Variable specific
heat" appears only in the Ch. 2 fundamentals (§2.6.6 Gas Tables) and one Ch. 6
component-performance page — never in the Ch. 5/7/8 cycle analysis, which is
calorically perfect throughout (Example 7.1 = constant dual-`cp`, the rung-2 anchor).
So the variable-`cp` anchor is necessarily *per-process machinery*. Two clean sources
provide it, both validated against the same NASA-air model (so they agree with each
other): **Çengel 9-89/9-88E** (full air Brayton, with/without efficiencies) and
**Mattingly's own Ex 2.7/2.8** (compression, nozzle). Mattingly even uses the
*identical* `Pr = exp(φ/R)` formalism this spec adopts (his Eqs. 2.53–2.58), so the
design is confirmed against the chosen anchor book — just at the process level, not
the cycle level. The original "also match Mattingly" intent is honored by Ex 2.7/2.8.

---

## Conservation asserts (rung-3 deltas)

Carry over rung 2's habits, restated on properties:

- **Round-trip inverses** (gate 2) become standing asserts in `Gas`.
- **Compressor/turbine substate** isentropic check moves onto `pr`:
  `pr(Tt3s)/pr(Tt2) == πc` and `pr(Tt5s)/pr(Tt4) == pt5/pt4` (exact by construction);
  entropy-generation inequalities `Tt3 ≥ Tt3s`, `Tt5 ≥ Tt5s` unchanged.
- **Burner / shaft / nozzle energy balances** asserted in **enthalpy** (the nozzle
  split `h(Tt9) = h(T9) + V9²/2` is now exact, tolerance tightened).
- **Specified ratios** (`π_d, π_b, π_n`) asserted exactly, as before.
- **Cascade closure** `eta_o == eta_thermal·eta_p` unchanged.

---

## API & scope

- `Gas` gains per-section property functions and accepts either a constant triple
  (CPG, default — rungs 1–2 behavior) **or** a `cp(T)` model (TPG). A factory like
  `Gas.thermally_perfect(...)` or a `cp_c=<callable/coeffs>, cp_t=<…>` field; the
  reduce-to-ideal default stays `Gas()`.
- Components and `build_turbojet(...)` keep their signatures **except**
  `Turbine.apply` and the engine's shaft step, which trade `delta_Tt` for `delta_h`.
- **Is not:** composition tracking, dissociation, off-design, choked nozzle,
  afterburner — seams kept.

### The honest scope note
The SPEC's prediction that variable `cp` "swaps the `Gas` model behind a property
interface; components are untouched" is **half right**: the *interface* is the clean
seam, but every `cp·T` and every `π^g` in the components/engine/freestream **is**
rewritten through it. That rewrite is the rung. The payoff is that, once done, the
*next* gas upgrade (reacting/variable-composition products) really is just a new
`Gas` behind the same four functions.

---

## Done when

The reduce-to-ideal gate reproduces the rung-1/2/2b tables **to the digit** (existing
suites untouched and green), the round-trip and dual-section discriminating checks
hold, the air-table isentropic anchor matches (~574 K), and the **Çengel 9-89 +
Mattingly Ex 2.7/2.8** machinery anchors match to **~0.1%** (already verified against
the NASA-air model before coding). `NOTES.md` gains a rung-3 section explaining, in
plain language: why constant `cp` ever worked, what `h(T)` and `pr(T)` buy, why the
isentropic power law had to go, and the closed-form-CPG-branch trap. The T–s diagram
is left alone (variable `cp` is a numbers/curvature point, not a new leg-tilt) unless
a clean way to show the isobar curvature change presents itself.
