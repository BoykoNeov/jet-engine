# Turbojet Cycle Simulator

A station-by-station model of a single-spool turbojet (Brayton cycle). It takes
flight + design conditions and produces the gas state at every station, the
thrust, the efficiencies, and a T–s diagram. **Rung 1** is the ideal cycle;
**rung 2** adds real components (isentropic efficiencies + pressure losses) and a
dual-section (cold/hot) gas, anchored to a published textbook case; **rung 2b**
adds polytropic efficiency `e_c/e_t` as a first-class knob beside the isentropic one;
**rung 3** upgrades the working fluid from calorically perfect (constant `cp`) to
thermally perfect (`cp = cp(T)`, gas-table `h`/`pr` property functions); **rung 4**
makes the hot-section **composition track the fuel/air ratio** `f` (explicit `(CH₂)ₙ`
lean-combustion stoichiometry → `cp_t/R_t/γ_t = f(T,f)`), which turns the burner into
an implicit `f = g(f)` fixed-point solve; **rung 5** is **Fork B — formation-enthalpy
bookkeeping**: restores the NASA `a6` constant so enthalpies are absolute and the
burner's heat release is **derived** (`hPR`/LHV falls out of the chemistry) instead of
assumed — provably identical to rung 4 for complete combustion, but on the absolute
scale rung-6 dissociation needs; **rung 6** adds **high-temperature dissociation +
chemical equilibrium**: the products (`CO₂`/`H₂O`) partly split into `CO/H₂/OH/O/H`, set
by a `Kp(T) = exp(−ΔG°/RuT)` solve (adds the absolute-entropy constant `a7`, derived like
`a6`). The cycle barely moves (a steeply Tt4-dependent delta on `f`: +0.02 % at the 1500 K
`main.py` panel, +0.15 % at the 1800 K anchor — station 4 is lean, high-pressure,
metallurgically capped), but the adiabatic-flame-temperature diagnostic finally falls
~115 K (2375→2259 K) into the real band; **rung 7** adds **thermal NOx**: the extended
Zeldovich mechanism (`O+N₂⇌NO+N`, …) as a **kinetically-limited** trace diagnostic on the
frozen rung-6 pool — the lesson **inverts rung 6** (major species reach equilibrium; NO does
*not* — at a ~3 ms residence it is frozen at a few % of equilibrium, and its rate is
exponentially T-sensitive, ~500× over 2000→2400 K). NO is trace, so the cycle stays
**bit-for-bit rung 6**; NOx rides as a diagnostic (a thermo-kinetic `K`-check binds the rate
constants to the `a6`/`a7` substrate); **rung 8** adds **combustor zoning**: the same rung-7
Zeldovich integrator run on a **two-zone** (near-stoichiometric **primary** → **dilution**)
combustor instead of the mixed-out station 4. NO is set in a hot primary (AFT ≈ 2000–2450 K,
from Tt3, scale A) and **frozen** through the dilution that cools the mixed-out gas back to Tt4;
EI_NO climbs from the rung-7 mixed-out ~zero (8e-6 g/kg) into the measured **ICAO band**
(~16–21 g/kg at φ_p ≈ 0.9–1.0) — a ~6-order lift purely from *where* the chemistry is evaluated,
**completing rung 7's inversion**. Still bit-for-bit rung 6 (a pure diagnostic); the from-Tt3
flame temperature is the first quantity to expose the ~8 K scale-A/scale-B datum offset (noted,
not chased — the cycle's sensible differences cancel it); **rung 9** adds a **rich primary /
RQL**: the same two-zone diagnostic with the primary allowed to run **rich** (φ_p up to 2.0),
so the equilibrium pool carries major `CO`/`H₂` (a **branched Newton seed** in `_equil_solve`;
lean branch byte-identical → reduce-to-rung-8 is provable). The payoff is the **rich flank of the
NO-vs-φ bell**: EI_NO peaks near stoich (~21 g/kg, ICAO band) and **collapses rich** (~1800× lower
at φ_p=1.4 — the AFT rolls over and the O-starved pool crashes `[O]`), which is *why* real
low-NOx combustors burn rich then quick-quench past the peak. Mix-out is the **ideal
(infinitely-fast) quench** (NO frozen); still bit-for-bit rung 6 (a pure diagnostic). Deferred:
the **finite-rate quench** (dwell-at-stoich NO spike) is the next seam — this is "rich primary +
ideal quench," not "RQL done."

**The deliverable is understanding, not the tool.** The code is the medium that
forces every thermodynamic assumption into the open. Optimize the work for
teaching, not for features or polish.

## Read this first
- **`SPEC.md`** — rung-1 contract + thermodynamics handout: station equations,
  shaft balance, the frozen ideal assumptions, the validation case.
- **`docs/rung2-spec.md`** — rung-2 contract + handout: efficiencies, dual gas,
  the reworked asserts, the two thermal efficiencies, the verification gates.
- **`docs/rung2b-polytropic.md`** — rung-2b contract + handout: polytropic `e_c/e_t`
  as a first-class knob, the `η_c < e < η_t` asymmetry, the equivalence gate.
- **`docs/rung3-variable-cp.md`** — rung-3 contract + handout: the `h`/`pr` property
  interface, the closed-form-CPG-branch trap, station equations in enthalpy form.
- **`docs/rung4-reacting-products.md`** — rung-4 contract + handout: `(CH₂)ₙ`
  stoichiometry, the `f`-parameterized properties, the implicit burner, the
  Fork-A/Fork-B datum split.
- **`docs/rung5-fork-b.md`** — rung-5 contract + handout: the `a6` restoration, the
  derived-heat-release burner, the exact reduce-to-rung-4 theorem, the datum invariant,
  the rung-6 (`a7` + `Kp`) seam.
- **`docs/rung6-spec.md`** — rung-6 contract + handout: the five dissociation species +
  reactions, the `Kp` standard-state `(p/p°)^Δν` factor, the `a7` derivation, the
  one-energy-datum split (composition on scale A, burner energy on scale B), the nested
  root-find burner, frozen-downstream, the verification gates.
- **`docs/rung7-spec.md`** — rung-7 contract + handout: the two new species (`NO`, `N`), the
  extended Zeldovich mechanism, the thermo-kinetic `K`-check, the superimposed equilibrium-NO
  layer, the one-equation kinetic integrator + residence-time knob `τ`, the decoupled-trace
  diagnostic, the lesson inversion, the gates.
- **`docs/rung8-spec.md`** — rung-8 contract + handout: the two-zone (primary → dilution)
  diagnostic, the primary-AFT-from-Tt3 solve (scale A), the re-equilibrating mix-out, the
  frozen-NO freeze, the two-part reduce-to-rung-7 gate (exact same-`T_p` + physical), the
  φ_p ≤ 1 scope, the split-independent-`T_mix`-→-Tt4 + NO-mole-conservation gates.
- **`docs/rung9-spec.md`** — rung-9 contract + handout: the rich primary (φ_p ≤ 2), the
  branched `_equil_solve` seed (lean byte-identical → provable reduce-to-rung-8), the rich CO/H₂
  equilibrium, the EI_NO bell (peaks near stoich, collapses rich), the ideal-quench framing +
  finite-quench seam, the soot-bound guard, the CEA-rich-methane + WGS self-check gates.
- `docs/plans/` — living plan/tasks (rungs 1–3), plus `rung2-anchor-mattingly.md`,
  `rung3-anchor-cengel.md`, `rung4-anchor-mattingly.md`, `rung5-anchor-formation.md`,
  `rung6-anchor-equilibrium.md`, `rung7-anchor-nox.md`, `rung8-anchor-zoning.md`, and
  `rung9-anchor-rql.md` (the verified textbook / formation / CEA-equilibrium / Zeldovich-kinetics
  / ICAO-zoning / rich-RQL anchor data).

## Working contract (from SPEC.md — these override convenience)
- **Derive before you code.** For each station, write the governing equation and
  a one-line physical justification (why it holds) *before* implementing it.
- **Show the work.** Every run prints the full station table (Tt, pt, …) so the
  numbers can be watched propagating.
- **Pure components.** Each component is `apply(state, gas) -> state` with no
  hidden state (Turbine and Nozzle diverge their signatures by design).
- **Conservation checks are assertions**, run on every execution (not as
  separate tests). See SPEC.md / docs/rung2-spec.md § Conservation checks.
- **Current scope (rung 9):** ideal + real components (isentropic `η_c/η_t` **or**
  polytropic `e_c/e_t`, mutually exclusive; pressure ratios `π_d/π_b/π_n`, `η_b`,
  `η_m`, dual cold/hot gas, specified exit pressure) on a **thermally-perfect** gas
  (`cp = cp(T)`; calorically-perfect sections kept as the closed-form branch) — with
  the hot section a **reacting** gas whose composition (and `cp_t/R_t/γ_t`) tracks
  `f` via explicit `(CH₂)ₙ` lean-complete-combustion stoichiometry, solved through an
  implicit `f = g(f)` burner. **Fork B** (rung 5): enthalpies carry each species'
  **formation** enthalpy (NASA `a6`, absolute datum) and the burner derives its heat
  release. Now **rung 6 — chemical equilibrium** (`Gas.reacting_equilibrium()`): the hot
  composition at the burner is the **dissociation-equilibrium** mixture (add `CO/H₂/OH/O/H`;
  5 reactions; `Kp = exp(−ΔG°/RuT)` with the `(p/p°)^Δν` factor, on `a7` derived like `a6`),
  **frozen** through turbine + nozzle; the burner is a **root-find on `f`** over the absolute
  balance (composition re-solved each trial). `Kp` uses the formation scale (scale A, required),
  the burner energy the production scale (scale B, reduces to Fork B); only the datum-free
  composition crosses between. Fork A/B (`Gas.reacting()`/`reacting_forkb()`) kept beside it.
  Now **rung 7 — thermal NOx** (`Gas.thermal_nox(far, T, p, τ)`): a **decoupled trace
  diagnostic** on the frozen rung-6 pool — the extended **Zeldovich** mechanism (add `NO`/`N`;
  Hanson–Salimian rate constants), a **superimposed equilibrium-NO** layer (`½N₂+½O₂⇌NO`,
  `Δν=0` ⇒ no `(p/p°)` factor, on the existing `a6`/`a7` `_g_molar`), and a one-equation
  **kinetic integrator** (QSS-`N`, reverse-rate form) over a **residence-time knob `τ`**. NO is
  trace, so the **cycle is bit-for-bit rung 6** (NO/N never enter `_equil_solve`); a
  thermo-kinetic **`K`-check** binds the rate constants to the thermochemistry. The lesson
  **inverts rung 6** (major species reach equilibrium; NO does not — frozen far below it).
  Now **rung 8 — combustor zoning** (`Gas.zoned_nox(far, Tt3, Tt4, p, φ_p, τ)`): the same rung-7
  `_thermal_no` run on a **two-zone** combustor — a near-stoichiometric **primary** (`φ_p ≤ 1`;
  adiabatic flame temp solved from Tt3 on scale A via `_primary_aft`; equilibrium products) where
  NO forms, then a **dilution / mix-out** (`_mixed_out_T`) that adds the rest of the air, **re-
  equilibrates the majors** (→ `T_mix` ≈ Tt4), and **freezes the NO moles**. A pure diagnostic
  (NO/N still never touch the cycle), so still **bit-for-bit rung 6**; EI_NO lifts from the
  mixed-out ~zero into the **ICAO band**, completing rung 7's inversion. Now **rung 9 — rich
  primary / RQL**: the same `zoned_nox` with the primary allowed to run **rich** (`φ_p ≤ 2.0`, the
  soot bound), so the equilibrium pool carries major `CO`/`H₂`. The 8-species `_equil_solve` was
  *already* complete rich (CO/H₂ are unknowns; reactions 1+2 span the water-gas shift) — the only
  change is a **branched Newton seed** (lean branch **byte-identical** → reduce-to-rung-8 is
  provable; rich branch = an O-limited seed). The payoff is the **rich flank of the NO-vs-φ bell**:
  EI_NO peaks near stoich and **collapses rich** (~1800× lower at φ_p=1.4 — AFT rollover + O-starve),
  which is *why* RQL burns rich then quick-quenches past the peak. Mix-out is the **ideal
  (infinitely-fast) quench** (NO frozen); still a pure diagnostic, **bit-for-bit rung 6**. Still
  deferred — keep the seams: the **finite-rate quench** (a secondary-zone Zeldovich — the
  dwell-at-stoich NO spike, *the* next seam), **super-equilibrium `O` / prompt NO (Fenimore)**
  (matters most in the rich primary), and the rung-6 **equilibrium-vs-frozen nozzle flow**;
  off-design / component maps, a *choked* convergent nozzle, afterburner.
- **Stop and explain surprises.** If a number looks off, reason about the
  physics rather than silently moving on.

## Conventions
- **SI units throughout** (K, Pa, kg/s, m/s, J/kg). Convert kPa → Pa internally.
- The cycle runs in **total (stagnation)** quantities `Tt, pt`; convert to
  static only at the nozzle exit (station 9) for exhaust velocity.

## Layout
- `turbojet/gas.py` — `FlowState`, dual-section `Gas` (cold/hot, `unified()`); the
  CPG closed-form / TPG NASA-integral property interface (`h/pr/T_from_*/cp_*_at`,
  hot methods carry `far`) and the `Gas.thermally_perfect()` (frozen products),
  `Gas.reacting()` (composition-tracks-`f`, Fork A), `Gas.reacting_forkb()` (Fork B,
  formation enthalpies) and `Gas.reacting_equilibrium()` (rung 6, dissociation) factories;
  `_products_composition(f)` + `_ReactingSection` (memoized per-`f` `_TPGSection`). Fork B
  adds `_HF298`/`_formation_products_mass(f)`/`_lhv_from_fuel` and the burner-only absolute
  interface (`a6` cancels in every difference → only the burner uses it). Rung 6 adds
  `_S298`/`_a7_of` (absolute entropy), `_g_molar`/`_lnKp` + the `_equil_solve`/
  `_equilibrium_composition` Newton solver, `_EquilibriumSection` (freezes the station-4
  mixture, keyed on `far` with a burn-config guard), and the burner interface
  `equilibrium_composition/h_air_abs_B/h_products_abs_B/freeze_equilibrium` (scale-B energy;
  scale-A `_h_molar_A`/`_g_molar` for `Kp`/AFT only). Rung 7 adds `NO`/`N` to the data dicts
  (inert to rungs 1–6), the `_ZELDOVICH` rate constants + `_k_zeldovich`, `_kp_no`/
  `_equilibrium_no_fraction` (superimposed NO), the `_kcheck_ratio` self-check, the `NOxState`
  dataclass + `_thermal_no` kinetic integrator, and `Gas.thermal_nox(far, T, p, τ)` — a
  decoupled diagnostic, so no cycle path is touched. Rung 8 adds `_h_air_molar_A` (scale-A air
  enthalpy), `_primary_aft` (from-Tt3 AFT), `_mixed_out_T` (re-equilibrating dilution), the
  `ZonedNOxState` dataclass, and `Gas.zoned_nox(far, Tt3, Tt4, p, φ_p, τ)` — all on the rung-6/7
  primitives, still a pure diagnostic (no cycle path touched). Rung 9 **branches the `_equil_solve`
  seed** on the O-balance sign (lean = byte-identical rung-6 expression → provable reduce; rich =
  an O-limited CO+H₂O seed), lifts the `zoned_nox` guard to `φ_p ≤ 2.0` (soot bound), and
  otherwise reuses every rung-6/7/8 primitive unchanged — the rich primary just hands a
  CO/H₂-major pool to the same integrator.
- `turbojet/components.py` — `Inlet, Compressor, Burner, Turbine, Nozzle` in `h`/`pr`
  form (+ loss params, `ram_recovery(M0)`, the polytropic `e_c/e_t` knob; the Nozzle
  branches CPG/TPG — the velocity↔enthalpy trap). The `Burner` runs the implicit
  `f = g(f)` fixed point (Fork B: `hPR` := derived LHV, plus a standing absolute-enthalpy
  balance assert), OR — for an equilibrium gas — `_solve_equilibrium` (a root-find on `f`
  over the scale-B absolute balance, equilibrium composition per trial, then freezes the
  station-4 mixture); `Turbine`/`Nozzle` hot-section calls thread `far` (sensible `h`, so
  bit-for-bit rung 4 — the `a6` offset cancels in their differences).
- `turbojet/engine.py` — chains components, solves the `Δh` + `η_m` shaft balance,
  scores performance (two thermal efficiencies + cascade check); freestream branches
  CPG/TPG.
- `tests/test_stations.py` — per-station rung-1 checks (turbine takes `delta_h`).
- `tests/test_validation.py` — rung-1 spec table (doubles as reduce-to-ideal).
- `tests/test_rung2.py` — reduce-to-ideal, directional, Mattingly Ex 7.1 anchor.
- `tests/test_polytropic.py` — rung-2b: reduce-to-ideal, polytropic⇄isentropic
  equivalence (1e-9), polytropic-native anchor, the `η_c<e<η_t` asymmetry.
- `tests/test_variable_cp.py` — rung-3: round-trip inverses, dual-section
  discriminating check, air-table + Çengel/Mattingly machinery anchors, gas-table effect.
- `tests/test_reacting.py` — rung-4: stoichiometry hand-check, implicit-solve
  direction + cross-datum burner, Mattingly Ex 6.3 products anchor, test-only McKinney
  cross-check, `f`-sweep directional; the implicit burner is a no-op on frozen gas.
- `tests/test_forkb.py` — rung-5: exact reduce-to-rung-4 (Fork B ≡ Fork A to machine
  precision), derived LHV = Mattingly `hPR`, formation self-check, absolute-balance
  closure + fuel-enthalpy live-knob, test-only adiabatic-flame-temperature plausibility.
- `tests/test_rung6.py` — rung-6: anti-seam reduce-to-rung-5 (cold-`Tt4` limit, `f` == Fork B
  to ~1e-6), CEA methane-AFT equilibrium anchor + pressure-suppression, formation/entropy
  self-checks, the equilibrium-AFT drop (test-only, scale A), station-4 delta bounded, the
  burn-config guard.
- `tests/test_rung7.py` — rung-7: reduce-to-rung-6 (NO/N never enter `_equil_solve`, cycle
  unchanged), the thermo-kinetic `K`-check, the `τ→∞` equilibrium asymptote, NO/N
  formation/entropy self-checks + GRI cross-check, magnitude + kinetic freezing (`τ_NO ≫`
  residence), T-sensitivity, equilibrium-NO pressure-independence.
- `tests/test_rung8.py` — rung-8: two-part reduce-to-rung-7 (exact zoned == `thermal_nox` at the
  same `T_p`; physical `T_p ≈ Tt4` + O(1) mixed-out factor), cycle-untouched, EI_NO in the ICAO
  band vs mixed-out ~zero, split-independent `T_mix` → Tt4 + the frozen-majors discriminator,
  NO-mole conservation through dilution, φ_p T-sensitivity, φ_p ≤ 1 guard, primary-T `K`-check.
- `tests/test_rung9.py` — rung-9: reduce-to-rung-8 (lean branch byte-identical, rung-8 same-`T_p`
  identity, cycle-untouched by rich zoning), CEA rich-methane AFT anchor + AFT rollover, rich
  CO/H₂-major + water-gas-shift self-check, the EI_NO bell (peaks near stoich, collapses on the
  rich flank), split-independent rich `T_mix` → Tt4, soot-bound guard (φ_p ≤ 2), K-check/trace at
  the rich primary T.
- `main.py` — runs ideal vs real at one design point: tables + overlaid T–s diagram,
  plus the rung-2-frozen-`cp` vs rung-3-`cp(T)` table, the rung-4 frozen-vs-reacting
  + `f`-sweep table, the rung-5 Fork-A-vs-Fork-B (derived-`hPR`) panel, the rung-6
  Fork-B-vs-equilibrium panel (AFT drop + dissociation-vs-pressure), the rung-7 thermal-NOx
  panel (flame-T sweep: equilibrium vs kinetic vs `τ`, the ~500× T-sensitivity, the near-zero
  station-4 number, the pressure-independence contrast), the rung-8 zoning panel (φ_p sweep:
  primary AFT, EI_NO into the ICAO band vs the mixed-out ~zero, `T_mix` → Tt4, the dilution
  NO-fraction drop at conserved EI_NO), and the rung-9 RQL panel (φ_p sweep across the bell: rich
  CO/H₂, the AFT rollover, EI_NO peaking near stoich then collapsing on the rich flank, `T_mix` →
  Tt4).

## Commands
- Run the model:  `python main.py`
- Run tests:      `pytest`  (or `python tests/test_rung2.py`, etc.)
- Install deps:   `pip install -r requirements.txt`  (matplotlib only)

## Stack
Python (standard library) + matplotlib for the plot. No other dependencies.
