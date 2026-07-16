# Turbojet Cycle Simulator

A station-by-station model of a single-spool turbojet (Brayton cycle). It takes
flight + design conditions and produces the gas state at every station, the
thrust, the efficiencies, and a T–s diagram.

**The deliverable is understanding, not the tool.** The code is the medium that
forces every thermodynamic assumption into the open. Optimize the work for
teaching, not for features or polish.

## The rungs

The model is built in cumulative **rungs** — each adds one physical effect and is
anchored to a published case. All rungs are live; the current scope is **rung 24**.

**This table is the one-line map, not the handout.** Each rung's derivation,
assumptions, honest concessions, reduce-to-prior contract and verification gates
live in its spec (last column) — read the spec before touching a rung.

| Rung | Adds (one-line hook) | Spec |
|------|----------------------|------|
| 1  | The **ideal** Brayton cycle: frozen, calorically-perfect, lossless. | `SPEC.md` |
| 2  | **Real components** — isentropic `η_c/η_t`, pressure losses `π_d/π_b/π_n`, `η_b`, `η_m`; dual cold/hot gas. | `docs/rung2-spec.md` |
| 2b | **Polytropic** `e_c/e_t` as a first-class knob beside the isentropic one (the `η_c < e < η_t` asymmetry). | `docs/rung2b-polytropic.md` |
| 3  | **Thermally-perfect** gas — `cp = cp(T)` via NASA gas tables (CPG kept as the closed-form branch). | `docs/rung3-variable-cp.md` |
| 4  | **Reacting products** — composition tracks `f`; the burner becomes an implicit `f = g(f)` solve. | `docs/rung4-reacting-products.md` |
| 5  | **Fork B** — NASA `a6` restored ⇒ the burner heat release (LHV) is **derived**, not assumed. Provably ≡ rung 4. | `docs/rung5-fork-b.md` |
| 6  | **Chemical equilibrium** — dissociation (`CO/H₂/OH/O/H`), `Kp = exp(−ΔG°/RuT)`. Cycle barely moves; AFT drops ~115 K into the real band. | `docs/rung6-spec.md` |
| 7  | **Thermal NOx** — extended Zeldovich as a kinetic trace diagnostic. **Inverts rung 6**: NO does *not* equilibrate. | `docs/rung7-spec.md` |
| 8  | **Combustor zoning** — two zones (near-stoich **primary** → **dilution**). EI_NO lifts from the mixed-out ~zero into the **ICAO band**. | `docs/rung8-spec.md` |
| 9  | **Rich primary / RQL** — primary allowed rich (`φ_p ≤ 2`); the NO-vs-φ **bell** peaks near stoich and collapses rich. | `docs/rung9-spec.md` |
| 10 | **Finite-rate quench** — a `τ_q` knob resolves the dilution in time: a rich primary's T rises through the stoich peak and **re-makes** NO. | `docs/rung10-spec.md` |
| 11 | **Physical mixing** — `JetMixing(J,…)` **derives** `τ_q` from the jet momentum-flux ratio. EI_NO falls **monotonically** in `J` — **mean-field ⇒ no mixing optimum**. | `docs/rung11-spec.md` |
| 12 | **Spatial unmixedness** — `Unmixedness(S,…)` splits the quench into bulk + an under-mixed core. EI_NO **turns back up**, min pinned **AT `C_opt`** — the recovered **Holdeman optimum**. | `docs/rung12-spec.md` |
| 13 | **Resolved mixing PDF** — `MixingPDF(S,…)`: a mean-preserving **β-PDF** over the ideal bell. **Mechanism separation** — composition variance pins the *location*; the over-penetration *climb* was rung-12's dwell. | `docs/rung13-spec.md` |
| 14 | **Equilibrium-vs-frozen nozzle** — `Gas.nozzle_flow(…)` brackets the frozen production nozzle against a shifting-equilibrium expansion. **Dormant** lean, **earns its keep hot**. | `docs/rung14-spec.md` |
| 15 | **PDF through the finite quench** — `QuenchPDF(S,…)` carries rung-13's β-PDF through the dwell chain, so both mixing mechanisms **combine**: a **finite floor**, and the far flank **climbs** again. | `docs/rung15-spec.md` |
| 16 | **PDF through the quench, PER POCKET** — `PocketQuenchPDF(S,…)` carries **each** pocket through its OWN quench. A lingering pocket **cools** ⇒ **sublinear** dwell, **eroding** rung-15's far flank. Global-min location **not claimed**. | `docs/rung16-spec.md` |
| 17 | **Exhaust-NO clamp ladder** — `Gas.exhaust_no_clamp(…)` runs three mixing-fidelity models through the rung-14 nozzle: mixed-out **HIDES** super-eq NO (dormant); bulk + per-pocket **FIRE**. The **ordering** is certified; the firing is in-band, not universal. | `docs/rung17-spec.md` |
| 18 | **Transported-variance closure** — `TransportedPDF(S,…)`: a variance-decay ODE from a **derived** ceiling. **NEGATIVE result**: a 0-D transport **cannot derive** `C_opt` — the optimum needs the spatial spacing `S`. Buys the ceiling, the residual floor, and kink-non-genericity. | `docs/rung18-spec.md` |
| 19 | **Super-equilibrium O & prompt NO** — lifts the eq-O **lower bound** every NO number since rung 7 carried. Both refute "the rich primary explodes": super-eq O is **T-driven, not rich**; prompt **survives where thermal dies**. | `docs/rung19-spec.md` |
| 20 | **Super-eq O *through the quench*** — threads the rung-19 lift into `_quench_no`. **Inverts** the intuition: the lift is **modest & peak-concentrated** (the re-making peaks where `m(T)` is minimal). The rung-17 margins **rise**; clamp still dormant at station 4. | `docs/rung20-spec.md` |
| 21 | **Super-eq O through the IDEAL-BELL PDF integrals** — discharges the **last eq-O seam**; rung-20's hybrid dissolves and its forbid guard is removed. A **shape-preserving consistency lift** (location/shift/sign-reversal unmoved). | `docs/rung21-spec.md` |
| 22 | **Resolved cross-plane / spatial PDF** — `SpatialPDF(S,k_p,…)`: the **INVERSION of rung 18**. Resolving the y-z cross-plane makes `C_opt` **EMERGE as an OUTPUT** (**no `C_opt` knob**). Certified: the `g_min` **collapse** + the `(H/S)²` shift. Uniformity, not emissions, is the headline. | `docs/rung22-spec.md` |
| 23 | **Derived dwell spectrum** — `SpatialDwellPDF(S,k_p,…)` develops that cross-plane in **TIME**, so each pocket carries its OWN `τ(ξ)` (**no `C_opt`/`τ_res`/`b_u`**). The positive: the **ξ–τ correlation** rich-pockets-dwell-longest **ADDS NO** — physics rung-16's scalar `τ_core` structurally cannot express. | `docs/rung23-spec.md` |
| 24 | **Locally-resolved mixing time** — `SpatialLocalPDF(S,k_p,…)`: each cell its OWN rate `ω=D_t\|∇ξ\|²/var` (**no new constant**). `τ_mix` **cancels** ⇒ `⟨τ⟩=τ_mix(J)·F(C)` **exactly**. **SPLIT answer**: `F(C)` is U-shaped, min **AT `C_opt`** — rung-16's imposed dwell growth **DERIVED** (kill-tested: `⟨\|∇ξ\|²⟩`, which carries no `g`, is maximal there). But **~40% vs a ~20× scale** ⇒ `⟨EI⟩` **stays monotone**: the emissions pin is **still not recovered**. **Localizes the RATE, not the SCALE.** | `docs/rung24-spec.md` |

**The invariant that spans rungs 7–24: they are all pure diagnostics.** NO/N never
enter `_equil_solve` and the production nozzle stays frozen, so **the cycle is
bit-for-bit rung 6** — every rung above 6 only *reads* the run's state. Each rung's
verified anchor data lives in `docs/plans/rungN-anchor-*.md`; `docs/plans/` also holds
the living plan/tasks (rungs 1–3).

## Working contract (from SPEC.md — these override convenience)
- **Derive before you code.** For each station, write the governing equation and
  a one-line physical justification (why it holds) *before* implementing it.
- **Show the work.** Every run prints the full station table (Tt, pt, …) so the
  numbers can be watched propagating.
- **Pure components.** Each component is `apply(state, gas) -> state` with no
  hidden state (Turbine and Nozzle diverge their signatures by design).
- **Conservation checks are assertions**, run on every execution (not as
  separate tests). See SPEC.md / docs/rung2-spec.md § Conservation checks.
- **Stop and explain surprises.** If a number looks off, reason about the
  physics rather than silently moving on.
- **Every new rung reduces to its predecessor**, exactly and by test (`X=None` ⇒
  the prior code path). This is the project's spine — see any `docs/rungN-spec.md`.

**Current scope (rung 24).** The **cycle solve** is a thermally-perfect, reacting,
dissociation-equilibrium gas (`Gas.reacting_equilibrium()`) through ideal + real
components (isentropic `η_c/η_t` **or** polytropic `e_c/e_t`, mutually exclusive;
`π_d/π_b/π_n`, `η_b`, `η_m`; dual cold/hot gas; specified exit pressure). The burner
root-finds `f` over the scale-B absolute balance, re-solving equilibrium each trial,
then freezes the station-4 mixture through turbine + nozzle. Fork A/B
(`Gas.reacting()` / `reacting_forkb()`) and frozen-products `Gas.thermally_perfect()`
are kept alongside. Everything from rung 7 up is a diagnostic *beside* the cycle.

## Deferred seams (kept open on purpose)
- **Finite-rate nozzle chemistry** — rung 14 gives the frozen↔equilibrium *bracket*,
  not the real Damköhler flow *between* the bounds (nor a **shifting turbine**).
- **A real spatial / transported-CFD PDF** — the standing ceiling. Rungs 22–24 took the
  first steps (deriving the width `g(C)`, the dwell spectrum `τ(ξ)`, and each cell's
  mixing *rate* from a resolved cross-plane), but they remain a Gaussian-plume **cartoon**
  feeding the β-PDF closure: the field's **PATTERN** is still the cartoon and the time
  **SCALE** is still one global `τ_mix` — only the *relaxation* is locally resolved. Hence
  `C_opt≈2.5` and the dwell magnitude still ride on `k_p`/`τ_mix`.
  **CORRECTED BY RUNG 24** — this seam used to say a locally-resolved mixing time was what
  a non-circular emissions optimum would need, and what would let rung 17 claim a firing
  *magnitude*. Rung 24 **built it, and NEITHER followed.** It does derive an off-optimum
  dwell **growth** (`F(C)`, min AT `C_opt`, gradient-located and kill-tested) — but at
  **~40% against `τ_mix`'s ~20× swing**, so `⟨EI⟩` stays monotone and the emissions optimum
  is **still not pinned**. Rung 17 gains a sharper **direction**, not a magnitude:
  **magnitude rides on the SCALE, and localizing the RATE does not touch it.** What the seam
  actually needs is a locally-resolved *SCALE* + the full cross-plane pattern.
  **INVESTIGATED BY "RUNG 25" — negative, NOT shipped** (`docs/rung25-investigation.md`). A
  locally-resolved SCALE (a penetration-growing plume `δ∝J^p`, `σ∝f(J)`, + rung-16's finite-`τ_res`
  dwell cap) **does** finally turn `⟨EI⟩(J)` off monotone — the first time in the project — and the
  field even carries a real over-penetration penalty (`g` is U-shaped in `J`). **But** the turn's
  location, depth, and even its *existence* ride on the **unanchored penetration exponent `p`**:
  a clean interior min only at the hand-picked `p=1/4`; at the more standard `p≈1/2` `⟨EI⟩` is
  monotone-down with no turn. So the SCALE alone still does **not** pin the emissions optimum — and
  the real missing piece is an **anchored `δ(J)` law**, not a missing penalty. Do NOT re-run the
  growing-σ-at-hand-picked-`p` construction; a new attempt is only worthwhile with an *anchored*
  exponent or a real transported/CFD cross-plane field.
- **A per-pocket clamp that fires AT THE BURNER** (`max_a>1` at station 4, not just in
  the rung-14/17 nozzle). The lever is a **slow-enough freeze on a cooling pocket**,
  *not* a hotter `Tt4` (which raises the terminal `[NO]_e` and *lowers* the ratio).
  Rungs 20 and 21 both confirm they are **not** this lever.
- **Detailed Fenimore** (`CH+N₂→HCN→…`) and **super-eq-O radical-decay history** — both
  need new species / a relaxing pocket, which a 0-D pool cannot derive (hence rung 19's
  imposed prompt magnitude and semi-empirical super-eq ratio).
- **Off-design / component maps**, a *choked* convergent nozzle, afterburner.

## Conventions
- **SI units throughout** (K, Pa, kg/s, m/s, J/kg). Convert kPa → Pa internally.
- The cycle runs in **total (stagnation)** quantities `Tt, pt`; convert to
  static only at the nozzle exit (station 9) for exhaust velocity.

## Layout
- `turbojet/gas.py` — the core. `FlowState`; dual-section `Gas` (cold/hot, `unified()`);
  the CPG closed-form / TPG NASA-integral property interface (hot methods carry `far`);
  the gas factories (`thermally_perfect` / `reacting` / `reacting_forkb` /
  `reacting_equilibrium`); the `_equil_solve` Newton solver + frozen `_EquilibriumSection`;
  and **every diagnostic** — `thermal_nox`, `zoned_nox`, `nozzle_flow`, `exhaust_no_clamp`
  — plus their configs (`JetMixing`, `Unmixedness`, `MixingPDF`, `QuenchPDF`,
  `PocketQuenchPDF`, `TransportedPDF`, `PromptNO`, `SpatialPDF`, `SpatialDwellPDF`,
  `SpatialLocalPDF`; the eight mixing closures are mutually exclusive) and helpers (`_quench_no`,
  `_pdf_mean_ei`, `_pocket_quench_mean_ei`, `_spatial_segregation`, `_spatial_dwell_field`, `_spatial_local_field`, …).
- `turbojet/components.py` — `Inlet, Compressor, Burner, Turbine, Nozzle` as pure
  `apply(state, gas)` in `h`/`pr` form (+ loss params, `ram_recovery(M0)`, the polytropic
  knob; the Nozzle branches CPG/TPG — the velocity↔enthalpy trap). The `Burner` runs the
  implicit `f = g(f)` fixed point, or `_solve_equilibrium` for an equilibrium gas.
- `turbojet/engine.py` — chains the components, solves the `Δh` + `η_m` shaft balance,
  scores performance (two thermal efficiencies + cascade check).
- `main.py` — the design-point run: ideal-vs-real tables, the overlaid T–s diagram, and
  **one panel per rung** (each panel demonstrates that rung's load-bearing claim and
  states its honest scope).
- `tests/` — `test_stations.py` / `test_validation.py` (rung 1), `test_rung2.py`,
  `test_polytropic.py` (2b), `test_variable_cp.py` (3), `test_reacting.py` (4),
  `test_forkb.py` (5), then **`test_rungN.py` for N = 6…24**. Every rung file carries that
  rung's **reduce-to-prior** gate plus its load-bearing claims; the gates are named in the
  rung's spec. Rungs 16, 23 and 24 **deliberately assert no emissions global-min location**.
- `docs/rungN-spec.md` — the derivation, assumptions, concessions and gates for rung N.
  `docs/plans/rungN-anchor-*.md` — that rung's verified anchor data.

## Commands
- Run the model:  `python main.py`
- Run tests:      `pytest`  (or `python tests/test_rung2.py`, etc.)
- Install deps:   `pip install -r requirements.txt`  (matplotlib only)

## Stack
Python (standard library) + matplotlib for the plot. No other dependencies.
