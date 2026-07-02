# Turbojet Cycle Simulator

A station-by-station model of a single-spool turbojet (Brayton cycle). It takes
flight + design conditions and produces the gas state at every station, the
thrust, the efficiencies, and a T–s diagram.

**The deliverable is understanding, not the tool.** The code is the medium that
forces every thermodynamic assumption into the open. Optimize the work for
teaching, not for features or polish.

## The rungs

The model is built in cumulative **rungs** — each adds one physical effect and is
anchored to a published case. All rungs are live; the current scope is
**rung 14**. Each rung's full derivation, assumptions, and verification gates live
in its spec (last column) — this table is the one-line map, not the handout.

| Rung | Adds (one-line hook) | Spec |
|------|----------------------|------|
| 1  | The **ideal** Brayton cycle: frozen, calorically-perfect, lossless. | `SPEC.md` |
| 2  | **Real components** — isentropic `η_c/η_t`, pressure losses `π_d/π_b/π_n`, `η_b`, `η_m`; dual cold/hot gas. Anchored to a textbook case. | `docs/rung2-spec.md` |
| 2b | **Polytropic** `e_c/e_t` as a first-class knob beside the isentropic one (the `η_c < e < η_t` asymmetry). | `docs/rung2b-polytropic.md` |
| 3  | **Thermally-perfect** gas — `cp = cp(T)` via NASA `h`/`pr` gas-table property functions (CPG kept as the closed-form branch). | `docs/rung3-variable-cp.md` |
| 4  | **Reacting products** — hot composition tracks `f` via `(CH₂)ₙ` lean stoichiometry (`cp_t/R_t/γ_t = f(T,f)`); the burner becomes an implicit `f = g(f)` solve. | `docs/rung4-reacting-products.md` |
| 5  | **Fork B** — NASA `a6` restored → absolute enthalpies, so the burner heat release (LHV) is **derived**, not assumed. Provably ≡ rung 4 for complete combustion. | `docs/rung5-fork-b.md` |
| 6  | **Chemical equilibrium** — dissociation (`CO/H₂/OH/O/H`), 5 reactions, `Kp = exp(−ΔG°/RuT)` (adds `a7`). Cycle barely moves; the AFT diagnostic drops ~115 K into the real band. | `docs/rung6-spec.md` |
| 7  | **Thermal NOx** — extended Zeldovich as a kinetically-limited trace diagnostic on the frozen rung-6 pool. **Inverts rung 6**: NO does *not* equilibrate (frozen at a few % of it). | `docs/rung7-spec.md` |
| 8  | **Combustor zoning** — the rung-7 integrator on a two-zone (near-stoich **primary** → **dilution**) combustor. EI_NO lifts from the mixed-out ~zero into the **ICAO band**. | `docs/rung8-spec.md` |
| 9  | **Rich primary / RQL** — primary allowed rich (`φ_p ≤ 2`, the soot bound); the NO-vs-φ **bell** peaks near stoich and collapses rich. Mix-out = the ideal (infinitely-fast) quench. | `docs/rung9-spec.md` |
| 10 | **Finite-rate quench** — a `τ_q` knob resolves the dilution in time: a rich primary's T rises through the stoich peak and **re-makes** NO. Low-NOx *only if the quench is fast*. `τ_q=None` = the exact rung-9 ideal quench. | `docs/rung10-spec.md` |
| 11 | **Physical mixing** — a `JetMixing(J,…)` config **derives** `τ_q = H/(C_e·√J·U_c)` from the jet momentum-flux ratio + a decelerating **entrainment** schedule, retiring rung-10's `τ_q`/linear knobs. EI_NO falls **monotonically** in `J` ("quick quench" = strong jet). **Mean-field** ⇒ no mixing *optimum* (the variance seam, rung 12). `mixing=None` = exact rung 10. | `docs/rung11-spec.md` |
| 12 | **Spatial unmixedness** — an `Unmixedness(S,…)` config (rides on `mixing`) splits the quench into a mean-field **bulk** (`τ_mean∝1/√J`, the still-falling reference) + an under-mixed **core** whose fraction `w(C)` AND dwell `τ_core(C)` both grow off-optimum (kinked in the Holdeman group `C=(S/H)√J`). EI_NO **turns back up** with the minimum pinned **AT `C_opt≈2.5`** — the recovered **Holdeman optimum**, `J_min=J_opt` shifting as `(H/S)²`. `unmixedness=None` = exact rung 11. | `docs/rung12-spec.md` |
| 13 | **Resolved mixing PDF** — a `MixingPDF(S,…)` config (rides on `mixing`, mutually exclusive with `unmixedness`) replaces rung-12's *parameterised* segregation with a continuous mean-preserving **β-PDF** of mixture fraction, width `g(C)` on the same Holdeman kink; `⟨EI⟩=∫EI_bell(φ(ξ))·P_β dξ` over the **ideal** bell. Lesson framed right: NO **peaked at stoich** ⇒ segregation **raises** the mean off-stoich (**reverses** at stoich), *not* generic convexity. Min **pinned AT `C_opt`** (both flanks up), `(H/S)²` shift — but a **mechanism separation**: composition variance pins the *location*; the over-penetration **climb was rung-12's dwell**, absent here (the far flank **descends** — ⟨EI⟩(g) humped/bimodal). `pdf=None` = exact rung 12; `g→0` = well-mixed point value. The PDF-through-quench is rung 15. | `docs/rung13-spec.md` |
| 14 | **Equilibrium-vs-frozen nozzle** — the rung-6 *cycle-side* seam. The production nozzle **freezes** the station-4 mixture; `Gas.nozzle_flow(…)` brackets it against a **shifting-equilibrium** expansion (CO/H₂/OH/O/H **recombine** on cooling → more V9). Frozen = lower bound, equilibrium = upper. **Dormant** at the lean design point (~0.006%), **earns its keep hot** (~0.46% at Tt4=2200 K). Corollary: on the same cooling path equilibrium NO **collapses**, so rung-10's **dropped clamp** finally **fires** (max_a≫1, vs its dormant 0.677). Pure diagnostic ⇒ cycle bit-for-bit rung 6. | `docs/rung14-spec.md` |

Rungs 7–13 are **pure diagnostics** — NO/N never enter the cycle solve, so the cycle
stays **bit-for-bit rung 6**. Rung 14 is *also* a pure diagnostic (`Gas.nozzle_flow` only reads the
run's state; the production nozzle stays frozen), so the cycle is still bit-for-bit rung 6. Each
rung's verified anchor data (textbook / formation / CEA-equilibrium / Zeldovich-kinetics / ICAO-
zoning / rich-RQL / finite-quench / jet-mixing / unmixedness / mixing-PDF / frozen-vs-equilibrium-
nozzle) lives in `docs/plans/rungN-anchor-*.md`; `docs/plans/` also holds the living plan/tasks
(rungs 1–3).

## Working contract (from SPEC.md — these override convenience)
- **Derive before you code.** For each station, write the governing equation and
  a one-line physical justification (why it holds) *before* implementing it.
- **Show the work.** Every run prints the full station table (Tt, pt, …) so the
  numbers can be watched propagating.
- **Pure components.** Each component is `apply(state, gas) -> state` with no
  hidden state (Turbine and Nozzle diverge their signatures by design).
- **Conservation checks are assertions**, run on every execution (not as
  separate tests). See SPEC.md / docs/rung2-spec.md § Conservation checks.
- **Current scope (rung 14):** all rungs above are cumulative and live (see § The
  rungs). The **cycle solve** is a thermally-perfect, reacting, dissociation-
  equilibrium gas (`Gas.reacting_equilibrium()`) run through ideal + real components
  (isentropic `η_c/η_t` **or** polytropic `e_c/e_t`, mutually exclusive; `π_d/π_b/π_n`,
  `η_b`, `η_m`; dual cold/hot gas; specified exit pressure). The burner is a root-find
  on `f` over the scale-B absolute balance (equilibrium composition re-solved each trial,
  then frozen through turbine + nozzle). Rungs 7–13 add the **NOx diagnostics**
  (`Gas.thermal_nox` / `Gas.zoned_nox`) *beside* the cycle, never inside it; rung 14 adds the
  **nozzle-flow diagnostic** (`Gas.nozzle_flow`) — also beside the cycle (the production nozzle
  stays frozen) — hence bit-for-bit rung 6. Fork A/B (`Gas.reacting()` / `reacting_forkb()`) and the
  frozen-products `Gas.thermally_perfect()` are kept alongside. **Deferred seams** (kept open on
  purpose): the **PDF through the finite quench** — rung 13 resolves the mixture-fraction
  distribution (a β-PDF) but on the **ideal** bell, isolating the *composition* mechanism from the
  rung-12 *dwell*; carrying the PDF through the `_quench_no` trajectory (so the two combine and the
  ≈0 optimum floor becomes finite bulk NO) is the immediate **rung-15 seam**; **finite-rate nozzle
  chemistry** — rung 14 gives the frozen↔equilibrium *bracket*, not the real Damköhler-number flow
  *between* the bounds (nor a **shifting turbine**); a **transported/CFD PDF** (predict the β width
  from a mixing equation rather than modeling `g(C)`); super-equilibrium `O` / prompt (Fenimore) NO
  (matters most in the rich primary, the stoich crossing, the rung-12 under-mixed core, the near-
  stoich pockets the rung-13 PDF resolves, *and* the rung-14 exhaust-NO clamp corollary — all
  equilibrium-O lower bounds); off-design / component maps, a *choked* convergent nozzle, afterburner.
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
  CO/H₂-major pool to the same integrator. Rung 10 adds `_quench_trajectory` (the τ_q-independent
  fast-chemistry dilution path — `_mixed_out_T` at partial air over β∈[0,1]) and `_quench_no` (a
  **separate clamp-free** RK4 NO integrator over it, extensive NO, K-check/trace bound at every β;
  returns `max_a` for the clamp-dormancy guard), extends `ZonedNOxState` (`tau_q`/`ei_no_quenched`/
  `x_no_quenched`/`T_peak`/`max_a_quench`, all `None` for the ideal quench) and gives `zoned_nox` a
  `tau_q=None` param that **short-circuits to the exact rung-9 path**; `_thermal_no` is byte-
  identical (its reduce gates need the exact capped trajectory). Still a pure diagnostic. The
  bisection AFT helpers (`_primary_aft`/`_mixed_out_T`) early-break at 1e-6 K and guard against
  bracket-edge pinning post-loop (the equilibrium solver diverges at the cold edge, so the guard
  can't probe endpoints). Rung 11 adds the `JetMixing` config (momentum-flux ratio `J` + geometry;
  a **derived** `tau_q = H/(C_e·√J·U_c)` property and a decelerating `schedule(β)=1−(1−t/τ_q)^n`,
  with `shape_n==1` returning the identity exactly), **generalizes `_quench_no` with an optional
  `schedule`** (β decoupled from time — `schedule=None` ⇒ byte-identical rung 10), and gives
  `zoned_nox` a `mixing=` param **mutually exclusive with `tau_q`** that derives the quench from
  the jet (`mixing=None` ⇒ exact rung-9/10 path). `ZonedNOxState` records the `mixing` config used.
  Mean-field ⇒ the `J`-sweep is monotone (no mixing optimum — the rung-12 variance seam). Rung 12
  adds the `Unmixedness` config (jet spacing `S` + `C_opt`/`tau_res`/`k_u`/`b_u`/`w_max`; a Holdeman-
  group `C(mixing)=(S/H)√J`, a KINKED `_u(C)=|ln(C/C_opt)|`, a `core_fraction(C)=min(w_max,k_u·_u)`
  and a growing `core_dwell(C)=tau_res·(1+b_u·_u)`, both 0-penalty at `C_opt`) and gives `zoned_nox`
  an `unmixedness=` param (**requires `mixing`**) that splits the quench into a mean-field **bulk**
  (`_quench_no` at the derived `τ_mean` — the monotone reference) + an under-mixed **core** (a *second*
  `_quench_no` on the SAME shared trajectory at the **absolute** `core_dwell(C)`), mass-weighted by
  `w(C)` → `ei_no_unmixed` **turns back up** in `J` with the minimum pinned **AT `C_opt`** (the kink;
  `J_min=J_opt`, shifting as `(H/S)²`). `ei_no_quenched` still holds the mean-field bulk.
  `unmixedness=None` ⇒ exact rung 11; `k_u=0` ⇒ bit-for-bit the bulk. `ZonedNOxState` records
  `unmixedness`/`C_holdeman`/`w_core`/`ei_no_unmixed`/`ei_no_core`; the `max_a<1` dormancy gate now
  spans both streams. No new chemistry/integrator — only a second stream. Rung 13 adds the
  `MixingPDF` config (jet spacing `S` + `C_opt`/`k_g`/`g_max` + grid sizes; `C(mixing)=(S/H)√J`, a
  KINKED `segregation(C)=min(g_max,k_g·|ln(C/C_opt)|)`), the module helpers `_ideal_bell_ei` (the
  rung-9 ideal bell at a local `far`), `_bell_interpolator` (builds `EI(ξ)` ONCE on a fixed fine
  ξ-grid, interpolates — the equilibrium-heavy bell is J-independent), `_beta_pdf_nodes_weights` (a
  **regime-aware, mean-preserving** β-PDF quadrature — `u=ξ^a` for the lean-mean `a<1` singularity,
  windowed uniform for `a≥1`, **asserting `⟨ξ⟩≈ξ̄`** + variance every call), and `_pdf_mean_ei`
  (`⟨EI⟩=∫EI_bell·P_β`). `zoned_nox` gains a `pdf=` param (**requires `mixing`**, **mutually
  exclusive with `unmixedness`**) → `ei_no_pdf` = the β-PDF integral over the ideal bell, a sharp
  minimum **pinned AT `C_opt`** (both flanks up), `J_min=J_opt` shifting as `(H/S)²`. A **mechanism
  separation**: composition variance pins the *location*; the over-penetration *climb* was rung-12's
  dwell (absent here — the far flank **descends**, ⟨EI⟩(g) humped/bimodal). `pdf=None` ⇒ exact rung
  12; `g→0` ⇒ well-mixed point value. `ZonedNOxState` records `pdf`/`g_seg`/`ei_no_pdf`
  (`C_holdeman` reused). No new chemistry/integrator — only the PDF quadrature over the existing bell.
  Rung 14 adds the **cycle-side** nozzle-flow diagnostic: `_mix_entropy_molar` (absolute mixture
  entropy per mol air, `Σ nᵢ[s0ᵢ(T)−Ru·ln(xᵢ·p/p0)]`, on the rung-6 `a7` `_s_molar` + the
  partial-pressure/mixing term — the one new thermodynamic quantity), `_mix_mass_per_air`
  (recombination-invariant), `_expand_nozzle` (ONE reversible-adiabatic expansion used both ways —
  frozen `shifting=False` == the production nozzle exactly, equilibrium `shifting=True` re-equilibrates
  each `T`; bisects `T9` on `S_mix(exit)=S_mix(entry)` from a COMMON physical entry, then
  `V9=√(2ΔH/m)` on **absolute** `_h_molar_B` so recombination energy appears), and `_nozzle_clamp_diag`
  (the equilibrium-NO **collapse ratio** + `max_a=x_NO_frozen/x_NO_e(T9)`). `Gas.nozzle_flow(far,Tt4,
  pt4,Tt9,pt9,p9,x_no_frozen=None)` → a `NozzleFlowState` (the `[V9_frozen,V9_equilibrium]` thrust
  bracket + `dV9`/`co_fraction_entry` + the clamp fields). Pure diagnostic: it only READS the run's
  state, so the cycle stays bit-for-bit rung 6. No new chemistry — reversible-adiabatic bookkeeping
  over the existing equilibrium machinery, exercised at nozzle-exit `T` (`_T_EXIT_FLOOR=500 K` guard).
- `turbojet/components.py` — `Inlet, Compressor, Burner, Turbine, Nozzle` in `h`/`pr`
  form (+ loss params, `ram_recovery(M0)`, the polytropic `e_c/e_t` knob; the Nozzle
  branches CPG/TPG — the velocity↔enthalpy trap, plus a back-pressure guard `p9 ≤ pt9`). The
  `Burner` runs the implicit `f = g(f)` fixed point (Fork B: `hPR` := derived LHV, plus a
  standing absolute-enthalpy balance assert), OR — for an equilibrium gas — `_solve_equilibrium`
  (a root-find on `f` over the scale-B absolute balance, equilibrium composition per trial, then
  freezes the station-4 mixture); `Turbine`/`Nozzle` hot-section calls thread `far` (sensible `h`,
  so bit-for-bit rung 4 — the `a6` offset cancels in their differences).
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
- `tests/test_rung10.py` — rung-10: reduce-to-rung-9 (exact `τ_q=None` short-circuit; quench fields
  `None`; finite quench additive; cycle-untouched), the smoking-gun T(β) rise through the stoich
  peak (rich) vs monotone fall (lean/stoich), the NO spike monotone in `τ_q`, the re-filled rich
  flank (~φ_p-independent floor), the clamp-dormancy `max_a < 1` guard, K-check along the
  trajectory, the soot-bound + `τ_q > 0` guards. (Cached design point + reusable trajectory to
  keep the equilibrium-heavy sweeps fast.)
- `tests/test_rung11.py` — rung-11: two-level reduce-to-rung-10 (`mixing=None` exact; `shape_n=1`
  == rung-10 linear `_quench_no` at the derived `τ_q`, bit-for-bit), the monotone `J`-sweep
  (EI_NO falls as `J` rises — no optimum), `τ_q ∝ 1/√J` in the RQL band, the schedule-shape
  discriminator (decelerating `shape_n>1` re-makes less than linear at fixed `J`, stoich crossing
  at low β), cycle-untouched, clamp dormancy, `mixing`/`tau_q` mutual exclusivity + `JetMixing`
  positivity guards, K-check along the trajectory. (Reuses the cached-design-point + trajectory.)
- `tests/test_rung12.py` — rung-12: two-level reduce-to-rung-11 (`unmixedness=None` exact; `k_u=0`
  == the mean-field bulk bit-for-bit at every `J`; `C=C_opt` ⇒ total = bulk) + a helper-vs-
  production `zoned_nox` pin, the **turn-up** (non-monotone `J`-sweep: falls then rises, interior
  min — the bulk still falling proves the variance turns it up), the optimum **AT `C_opt`**
  (`J_min==J_opt`, shifting exactly as `(H/S)²`), the core penalty **surviving strong jets +
  growing off-optimum** (min dwell at `C_opt`), the kinked `w(C)` (0 at `C_opt`, both flanks,
  non-zero slope), cycle-untouched, clamp dormancy over both streams, require-`mixing` +
  `Unmixedness` positivity/range guards. (Reuses cached DP + trajectory.)
- `tests/test_rung13.py` — rung-13: two reduces (`pdf=None` code-path-identical rung 12; `g→0` == the
  exact well-mixed point value) + a helper-vs-production `zoned_nox` pin, **mean-preservation**
  (`⟨ξ⟩≈ξ̄` + variance across the `g`-range — the `u=ξ^a` transform), the optimum **pinned AT
  `C_opt`** (both flanks lift by orders) with `J_min==J_opt` shifting as `(H/S)²`, the **humped
  ⟨EI⟩(g)** (peak at low `g`, descending — the tested reason the far flank descends), the convexity
  jump + **stoich-mean sign reversal**, the kinked `g(C)`, cycle-untouched, require-`mixing` +
  `pdf`/`unmixedness` mutual-exclusivity + `MixingPDF` positivity/range guards. (Reuses a cached DP +
  a bell built once.)
- `tests/test_rung14.py` — rung-14: the LOAD-BEARING reduce (frozen `_expand_nozzle` == the production
  nozzle `V9`/`T9` to machine precision; freeze-comp-in-eq-branch == frozen bit-for-bit), the
  dissociation→0 collapse (cool `Tt4` ⇒ `ΔV9→0`), direction (equilibrium faster + hotter at exit +
  recombines CO), magnitude/monotone (`ΔV9/V9` grows with `Tt4` — dormant at design, ~0.46% hot), the
  isentropic self-check (both expansions conserve `S`), the clamp corollary (eq-NO collapse ≫1 + the
  zoned-NO `max_a`≫1 past rung-10's dormant 0.677), cycle-untouched, and the requires-equilibrium /
  back-pressure guards.
- `main.py` — runs ideal vs real at one design point: tables + overlaid T–s diagram,
  plus the rung-2-frozen-`cp` vs rung-3-`cp(T)` table, the rung-4 frozen-vs-reacting
  + `f`-sweep table, the rung-5 Fork-A-vs-Fork-B (derived-`hPR`) panel, the rung-6
  Fork-B-vs-equilibrium panel (AFT drop + dissociation-vs-pressure), the rung-7 thermal-NOx
  panel (flame-T sweep: equilibrium vs kinetic vs `τ`, the ~500× T-sensitivity, the near-zero
  station-4 number, the pressure-independence contrast), the rung-8 zoning panel (φ_p sweep:
  primary AFT, EI_NO into the ICAO band vs the mixed-out ~zero, `T_mix` → Tt4, the dilution
  NO-fraction drop at conserved EI_NO), the rung-9 RQL panel (φ_p sweep across the bell: rich
  CO/H₂, the AFT rollover, EI_NO peaking near stoich then collapsing on the rich flank, `T_mix` →
  Tt4), the rung-10 finite-quench panel (a `τ_q` sweep at a rich primary: T rising through the
  stoich peak, the EI_NO spike vs `τ_q`, and the re-filled rich flank vs the rung-9 ideal quench),
  the rung-11 jet-mixing panel (a `J`-sweep: the derived `τ_q` and EI_NO falling monotonically
  as jet momentum rises, plus the schedule-shape contrast — decelerating entrainment vs rung-10's
  linear — at fixed `J`), the rung-12 unmixedness panel (a `J`-sweep: the mean-field bulk still
  falling vs the two-stream total that **turns back up** with its minimum pinned **at `C_opt`** — the
  recovered Holdeman optimum — plus the `(H/S)²` optimum shift when the jet spacing `S` shrinks),
  and the rung-13 mixing-PDF panel (the **peaked×off-mean** mechanism — a lean-mean vs stoich-mean
  column pair showing the **sign flip** — and a `J`-sweep: the sharp notch **pinned AT `C_opt`** with
  both flanks lifting, the far over-penetration flank **descending** (the humped/bimodal ⟨EI⟩(g)) —
  the composition-vs-dwell mechanism separation from rung 12), and the rung-14 nozzle-flow panel (a
  `Tt4` sweep of the frozen↔equilibrium **thrust bracket** — dormant ~0.006% at the design point,
  ~0.46% at 2200 K — and the **dropped-clamp** corollary: equilibrium NO collapsing ~120× on cooling
  so a realistic zoned exhaust is `max_a`≈250 super-equilibrium at the exit, where rung-10's clamp fires).

## Commands
- Run the model:  `python main.py`
- Run tests:      `pytest`  (or `python tests/test_rung2.py`, etc.)
- Install deps:   `pip install -r requirements.txt`  (matplotlib only)

## Stack
Python (standard library) + matplotlib for the plot. No other dependencies.
