# Turbojet Cycle Simulator

A station-by-station model of a single-spool turbojet (Brayton cycle). It takes
flight + design conditions and produces the gas state at every station, the
thrust, the efficiencies, and a T–s diagram.

**The deliverable is understanding, not the tool.** The code is the medium that
forces every thermodynamic assumption into the open. Optimize the work for
teaching, not for features or polish.

> **⚠ This file is a REFERENCE / index — not a handout.** Keep it compact.
> The rung table is **one line per rung**; each rung's derivation, assumptions,
> honest concessions, reduce-to-prior contract and verification gates live in its
> **spec** (`docs/rungN-spec.md`), not here. "Deferred seams" is a **one-line-per-entry
> status map** (`BUILT BY RUNG N` | `NEGATIVE → doc` | `OPEN`), never an essay.
> A guard test (`tests/test_claude_md_reference.py`) fails if this file exceeds its
> size budget: **if it trips, move detail into a spec — do not raise the budget.**

## The rungs

The model is built in cumulative **rungs** — each adds one physical effect and is
anchored to a published case. All rungs are live; the current scope is **rung 43**.

**This table is the one-line map, not the handout.** Read a rung's spec (last
column) before touching it — that is where the real content lives.

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
| 12 | **Spatial unmixedness** — `Unmixedness(S,…)` splits the quench into bulk + under-mixed core. EI_NO **turns back up**, min pinned **AT `C_opt`** — the recovered **Holdeman optimum**. | `docs/rung12-spec.md` |
| 13 | **Resolved mixing PDF** — `MixingPDF(S,…)`: a mean-preserving **β-PDF** over the ideal bell. **Mechanism separation** — variance pins the *location*; the *climb* was rung-12's dwell. | `docs/rung13-spec.md` |
| 14 | **Equilibrium-vs-frozen nozzle** — `Gas.nozzle_flow(…)` brackets the frozen production nozzle against a shifting-equilibrium expansion. **Dormant** lean, **earns its keep hot**. | `docs/rung14-spec.md` |
| 15 | **PDF through the finite quench** — `QuenchPDF(S,…)` carries rung-13's β-PDF through the dwell chain; both mixing mechanisms **combine**: a **finite floor**, far flank **climbs** again. | `docs/rung15-spec.md` |
| 16 | **PDF through the quench, PER POCKET** — `PocketQuenchPDF(S,…)`: each pocket through its OWN quench. A lingering pocket **cools** ⇒ **sublinear** dwell, **erodes** rung-15's far flank. | `docs/rung16-spec.md` |
| 17 | **Exhaust-NO clamp ladder** — `Gas.exhaust_no_clamp(…)`: three mixing-fidelity models through the rung-14 nozzle. Mixed-out **HIDES** super-eq NO; bulk + per-pocket **FIRE**. Ordering certified. | `docs/rung17-spec.md` |
| 18 | **Transported-variance closure** — `TransportedPDF(S,…)`: a variance-decay ODE. **NEGATIVE**: a 0-D transport **cannot derive** `C_opt` — the optimum needs the spatial spacing `S`. | `docs/rung18-spec.md` |
| 19 | **Super-equilibrium O & prompt NO** — lifts the eq-O lower bound every NO number since rung 7 carried. Both refute "rich primary explodes": super-eq O is **T-driven**; prompt survives where thermal dies. | `docs/rung19-spec.md` |
| 20 | **Super-eq O *through the quench*** — threads the rung-19 lift into `_quench_no`. **Inverts** intuition: the lift is **modest & peak-concentrated**. Clamp still dormant at station 4. | `docs/rung20-spec.md` |
| 21 | **Super-eq O through the IDEAL-BELL PDF integrals** — discharges the last eq-O seam; rung-20's hybrid dissolves. A **shape-preserving consistency lift** (location/shift/sign unmoved). | `docs/rung21-spec.md` |
| 22 | **Resolved cross-plane / spatial PDF** — `SpatialPDF(S,k_p,…)`: the **INVERSION of rung 18**. Resolving the y-z cross-plane makes `C_opt` **EMERGE as an OUTPUT**. Uniformity, not emissions, is the headline. | `docs/rung22-spec.md` |
| 23 | **Derived dwell spectrum** — `SpatialDwellPDF(S,k_p,…)`: develops the cross-plane in **TIME**, each pocket its OWN `τ(ξ)`. The **ξ–τ correlation** (rich-pockets-dwell-longest) **ADDS NO**. | `docs/rung23-spec.md` |
| 24 | **Locally-resolved mixing time** — `SpatialLocalPDF(S,k_p,…)`: each cell its OWN rate `ω`. `τ_mix` cancels. **SPLIT**: `F(C)` U-shaped (min AT `C_opt`, derived) but `⟨EI⟩` stays monotone — localizes the RATE, not the SCALE. | `docs/rung24-spec.md` |
| 25 | **Finite-rate nozzle chemistry** — `FiniteRate(Da,…)` / `finite_rate_nozzle`: Damköhler flow between rung-14's bounds. **THREE-state** picture (F frozen / I irreversible-fast ceiling / R unreachable reversible); reduces to FROZEN, not equilibrium. | `docs/rung25-spec.md` |
| 26 | **Freeze-out** — `FreezeOut(L,…)` / `freeze_out_nozzle`: a local **anchored** `Da(T,p)` clock (GRI-Mech, zero new constants). The freeze point **MOVES with `Tt4`**; density-driven (refutes rung-25's own framing). | `docs/rung26-spec.md` |
| 27 | **NO freeze-out** — `NOFreezeOut(L,…)` / `no_freeze_out_nozzle`: the frozen-NO assumption every NO number carried since rung 7 is **EARNED** (`Da_NO≪1` from entry). Kill test **INVERTS** rung 26 (both terms agree). | `docs/rung27-spec.md` |
| 28 | **The rung-26-coupled NO march** — `CoupledNOFreezeOut(L,…)`: rung-27's clock on rung-26's relaxing pool. Verdict **CONFIRMED**, both reasons **CORRECTED** (heat-release channel + the β<1 repair). | `docs/rung28-spec.md` |
| 29 | **The shifting turbine** — `shifting_turbine(…)` / `_work_limited_expand`: is FREEZING the turbine earned? **EARNED at design**, bites hot — rate-independently. The **RATIO ≠ ENERGY** cross-rung correction. | `docs/rung29-spec.md` |
| 30 | **The choked convergent nozzle** — `Nozzle(convergent=True)` / `_sonic_throat`: is FULL EXPANSION earned? **Not at design** for a convergent engine; the **pressure term rescues 87%** of the momentum deficit. | `docs/rung30-spec.md` |
| 31 | **Off-design matching** — `OffDesignMatcher`: the **first STRUCTURAL rung** — `π_c` becomes an **OUTPUT** on a fixed-hardware running line (no compressor map). `τ_t` **drifts** on the real gas, const on CPG. | `docs/rung31-spec.md` |
| 32 | **Component-map matching** — `MapMatcher` + `ComponentMap`: CORRECTS rung 31's "hardware IS the map" — the work is map-free but `π_c`/`ṁ`/`N` need the map; `N` enters via speed lines. | `docs/rung32-spec.md` |
| 33 | **Subsonic-nozzle matching branch** — `OffDesignMatcher._match_subsonic`: the second matching mode below unchoke. **INVERTS rung 31** — `τ_t` **varies even on CPG** (structural coupling through `π_c`). | `docs/rung33-spec.md` |
| 34 | **The spool transient** — `SpoolTransient`: the **first DYNAMIC rung** — `N` a **STATE** under the shaft-inertia ODE. The finding is the ratio `r=τ_fuel/τ_spool`, not the tautological "`I`-independent shape". | `docs/rung34-spec.md` |
| 35 | **Fuel metering — `Tt4` an OUTPUT** — `SpoolTransient.equilibrium_fuel/integrate_fuel`: meters **fuel**, `Tt4` floats. CORRECTS rung 34 — a fuel step → **TIT overshoot** and an **enlarged** surge excursion (the two limits coupled). | `docs/rung35-spec.md` |
| 36 | **The surge line** — surge methods on `SpoolTransient` + `ComponentMap.with_phi_surge`: pure diagnostic. `φ_surge` **imposed** but the **sign survives** — surge margin **thin at LOW power**; confirms + sharpens rung 34. | `docs/rung36-spec.md` |
| 37 | **The two internal clocks** — `CombustorTransient`: plenum (volume-filling) **CONFIRMS** rung 34's concession; metal (heat-soak) **CORRECTS** it (history-dependent, surge-protected, accel-lag). | `docs/rung37-spec.md` |
| 38 | **Two-spool matching** — `build_two_spool_turbojet` / `TwoSpoolMatcher`: the **first TWO-SHAFT** rung. A **THIRD choked throat** (`A45`) chains rung-31 `(★)` twice; compressor-η is a **terminal leaf** (no 2×2 solve) — a no-map artifact. | `docs/rung38-spec.md` |
| 39 | **Two-spool + component maps** — `TwoSpoolMapMatcher`: **refutes** rung 38's prediction, **confirms** its verdict — the map opens **ONE arrow HP→LP** (`π_LPC` cancels). Two speeds ⇒ **slip `N_L/N_H`**, inverting rung 32. | `docs/rung39-spec.md` |
| 40 | **The two-shaft transient** — `TwoSpoolTransient`: both speeds **STATES**, one parameter `ρ=τ_L/τ_H`. `ρ`'s power **SPLITS** — powerless over stability, decisive over a **MAP-created complex** inter-spool mode. | `docs/rung40-spec.md` |
| 41 | **The two-spool surge line** — surge methods on `TwoSpoolMapMatcher`: the exposure **SPLITS onto the LP spool**. A **LIVE** zero-new-constant `π*=γc^(γc/(γc−1))`; corrects rung 36's mechanism, its verdict survives. | `docs/rung41-spec.md` |
| 42 | **Interstage bleed** — `TwoSpoolBleedMatcher`: the project's **first STEADY mass extraction**. Bleed is a **new DoF on the LP spool and NOT the HP**; the "penalises HP" hypothesis is **refuted**. | `docs/rung42-spec.md` |
| 43 | **Two-shaft fuel metering** — `TwoSpoolFuelTransient`: rung-35 control on rung-40's plant. The two spools sit at **DIFFERENT points in ONE overshoot loop**, so **NEITHER clock governs it**; the **currency-circularity** trap. | `docs/rung43-spec.md` |

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

**Current scope (rung 43).** The **cycle solve** is a thermally-perfect, reacting,
dissociation-equilibrium gas (`Gas.reacting_equilibrium()`) through ideal + real
components (isentropic `η_c/η_t` **or** polytropic `e_c/e_t`, mutually exclusive;
`π_d/π_b/π_n`, `η_b`, `η_m`; dual cold/hot gas; specified exit pressure). The burner
root-finds `f` over the scale-B absolute balance (re-solving equilibrium each trial),
then freezes the station-4 mixture through turbine + nozzle. Fork A/B and
frozen-products gases are kept alongside. **Everything from rung 7 up is a diagnostic
*beside* the cycle**, reached through **separate entry points** that leave the default
`build_turbojet(…).run(…)` design run **bit-for-bit rung 6**. Rungs **31–43** are the
STRUCTURAL / DYNAMIC rungs (a new off-design or transient operating point — the
single-spool ladder `OffDesignMatcher → MapMatcher → SpoolTransient → CombustorTransient`
and the two-spool ladder `TwoSpoolMatcher → TwoSpoolMapMatcher → TwoSpoolTransient →
TwoSpoolBleedMatcher → TwoSpoolFuelTransient`); rungs **7–30, 36, 41** are pure
diagnostics that only *read* the design-point state. Each rung reduces to its
predecessor exactly and by test — the gates are named in its spec.

## Deferred seams — status map
One line per seam: `BUILT BY RUNG N` (detail in its spec) · `NEGATIVE → doc` (investigated,
not shipped, not a rung) · `OPEN` (not yet built). This list is the live map of what is
closed vs open — keep it one line per entry.

**Built (each seam → the rung that closed it; detail in the spec):**
- Finite-rate nozzle chemistry → **rung 25**; freeze-out → **rung 26**; NO freeze-out → **rung 27**; coupled NO march → **rung 28**; the shifting turbine → **rung 29**.
- The choked convergent nozzle → **rung 30**; off-design matching → **rung 31**; component-map matching → **rung 32**; the subsonic-nozzle branch → **rung 33**.
- The spool transient → **rung 34**; fuel metering (`Tt4` output) → **rung 35**; the surge line → **rung 36**; the two combustor internal clocks → **rung 37**.
- Two-spool matching → **rung 38**; two-spool + maps → **rung 39**; the two-shaft transient → **rung 40**; the two-spool surge line → **rung 41**; the bleed valve → **rung 42**; two-shaft fuel metering → **rung 43**.

**Investigated, NEGATIVE — not shipped, not a rung (these facts live only here + the doc):**
- Resolved `τ_res` from the nozzle area-schedule (rung 26's seam a) — `docs/tau-res-negative.md` (shape moot; needs an entry Mach). Confirms rung 26.
- Finite-rate turbine march (rung 29's seam a) — `docs/turbine-march-negative.md` (`I_turb ≡ S`, entry at equilibrium; two un-anchored knobs).
- Locally-resolved mixing **SCALE** — `docs/mixing-scale-negative.md` (the turn rides on the unanchored penetration exponent `p`).
- Anchored `δ(J)` law via a JICF trajectory — `docs/mixing-jicf-anchor-negative.md` (confirms rung 22; emissions optimum rides on a SECOND unanchored exponent, spread).

**Checked, CONFIRMATION / CORRECTION — not a rung (the rung-29/28 margin sweeps):**
- "Earned at design" over `π_c` — `docs/rung29-pi-c-margin.md` (verdict holds ~9.4×; `π_c` NOT protective; `ENERGY = INVENTORY × COMPLETION`).
- "Earned at design" over flight `M0` — `docs/rung29-M0-margin.md` (holds ~8.8×; monotone-protective; the `delta_h`-swing correction).
- `β<1` over `π_c` / hotter cycles — `docs/rung28-beta-margin.md` (β pressure-invariant; higher `π_c` protective).

**Still OPEN — not yet built (the live to-build list):**
- **The real spatial / transported-CFD PDF** — the standing mixing ceiling (rungs 22–24 remain a Gaussian-plume cartoon; `C_opt`/dwell still ride on `k_p`/`τ_mix`; needs an anchored SCALE + spread law, or a real CFD cross-plane).
- **A per-pocket clamp that fires AT THE BURNER** (`max_a>1` at station 4) — lever is a slow-enough freeze on a cooling pocket (rungs 20/21 confirm it is not a hotter `Tt4`).
- **Detailed Fenimore** (`CH+N₂→HCN`) and **super-eq-O radical-decay history** — need new species / a relaxing pocket a 0-D pool cannot derive.
- **Reacting-gas fuel control** (rungs 35/43 defer — the forward burner asserts against an equilibrium gas; the finding is gas-independent).
- **The transient two-spool surge line** (rung 40's complex mode measured against rung 41's boundary — rung 41 is steady-only) ⇒ no surge-**survival** claim yet.
- **The subsonic / unchoked LP branch** in the two-spool solves (rung 38 flags, does not solve) and its **transient**.
- **The variable stator** (moves `φ_surge` itself — rung 42 did the bleed half); a **bleed schedule** `b(n_L)`; **fuel + bleed together**; a **TIT redline**.
- **Rung 37's internal clocks on two shafts** and the combined 3-state; **customer/cooling bleed** at station 3.
- **Afterburner**; a **real hardware/CFD map + surge line** (rung 32's standing concession, now doubled across two spools).
- **Feeding any shifted/marched state into the production cycle** — a re-foundation (re-anchors every rung's numbers), not a rung.

## Open engineering tasks (not rungs, not seams)
- **Audit the iterative solvers for absolute-tolerance-below-noise-floor** — **CLOSED, NEGATIVE**
  (`docs/plans/todo-solver-tolerance-audit.md`). Rung 43 fixed a real rung-40 `_EQ_TOL` hole
  (absolute 1e-12 below the reacting gas's ~1e-10 residual floor); this audit checked the six
  `_ETA_TOL`=1e-11 efficiency secants and found they do **not** share it (their residual has an
  exact float64 root). No code change; the fix shape stays on file for a future steeper/CFD map.

## Conventions
- **SI units throughout** (K, Pa, kg/s, m/s, J/kg). Convert kPa → Pa internally.
- The cycle runs in **total (stagnation)** quantities `Tt, pt`; convert to
  static only at the nozzle exit (station 9) for exhaust velocity.

## Layout
A compact map — the per-rung method/finding detail lives in `docs/rungN-spec.md`, not here.
- `turbojet/gas.py` — **the core.** `FlowState`; the dual-section `Gas` (cold/hot, `unified()`)
  with the CPG closed-form / TPG NASA-integral property interface (hot methods carry `far`); the
  gas factories (`thermally_perfect` / `reacting` / `reacting_forkb` / `reacting_equilibrium`); the
  `_equil_solve` Newton + frozen `_EquilibriumSection`; and **every rung-7+ diagnostic** on `Gas`
  (`thermal_nox`, `zoned_nox`, `nozzle_flow`, `exhaust_no_clamp`, `finite_rate_nozzle`,
  `freeze_out_nozzle`, `no_freeze_out_nozzle`, `coupled_no_freeze_out_nozzle`, `shifting_turbine`)
  with their configs (the mutually-exclusive mixing closures `JetMixing…SpatialLocalPDF`;
  `FiniteRate`/`FreezeOut`/`NOFreezeOut`/`CoupledNOFreezeOut`) and helpers (`_quench_no`,
  `_pdf_mean_ei`, `_finite_rate_expand`, `_freeze_out_expand`, `_work_limited_expand`, …).
- `turbojet/components.py` — `Inlet, Compressor, Burner, Turbine, Nozzle` as pure `apply(state, gas)`
  in `h`/`pr` form (+ loss params, `ram_recovery(M0)`, the polytropic knob). The Nozzle branches
  CPG/TPG and carries rung-30's `convergent=True` choke via the module-level `_sonic_throat`; rung-31's
  `choked_mfp` (the `pt`-independent sonic mass-flow parameter) lives here too. The `Burner` runs the
  implicit `f = g(f)` fixed point, or `_solve_equilibrium` for an equilibrium gas.
- `turbojet/engine.py` — chains the components, solves the `Δh` + `η_m` shaft balance, scores
  performance (`_score`). Home to the **off-design / transient matcher ladders**, each a **separate
  entry point** (the design `run` is untouched): single-spool `OffDesignMatcher` (31) → `MapMatcher` +
  `ComponentMap` (32) → `._match_subsonic` (33) → `SpoolTransient` (34, + fuel 35, + surge 36) →
  `CombustorTransient` (37); two-spool `build_two_spool_turbojet` / `TwoSpoolMatcher` (38) →
  `TwoSpoolMapMatcher` (39) → `TwoSpoolTransient` (40, + surge methods 41) → `TwoSpoolBleedMatcher` (42)
  → `TwoSpoolFuelTransient` (43). Each reduces to its predecessor (exact dispatch or the forward
  closure); the method names + reduce contracts are in each rung's spec.
- `main.py` — the design-point run: ideal-vs-real tables, the overlaid T–s diagram, and **one panel
  per rung** (each demonstrates that rung's load-bearing claim and states its honest scope).
- `tests/` — per-rung `test_rungN.py` (N = 1…43; plus the rung-1/2b/3/4/5 files). Every rung file
  carries that rung's **reduce-to-prior** gate plus its load-bearing claims — the gates are named in
  the spec. `test_claude_md_reference.py` is the size guard on this file.
- `docs/rungN-spec.md` — the derivation, assumptions, concessions and gates for rung N.
  `docs/plans/rungN-anchor-*.md` — that rung's verified anchor data. `docs/plans/` also holds the
  living plan/tasks.

## Commands
- Run the model:  `python main.py`
- Run tests (fast, routine):  `pytest` — the FAST subset (~2.5 min). Inherently-expensive FINDING /
  robustness gates are tagged `slow` and deselected, **but the bit-for-bit reduce spine
  (`test_reduce_*`, `test_cycle_untouched_*`, `*_bit_for_bit`) is always kept.**
- Run tests (full, every gate):  `pytest --runslow` — all tests (~10–15 min). **Use this at commit /
  session-end / CI** — the fast subset is for iteration, not for signing off a rung.
- Only the slow gates:  `pytest -m slow`   ·   One rung by hand:  `python tests/test_rung2.py`
- Install deps:   `pip install -r requirements.txt`  (matplotlib + pytest + pytest-xdist)

The speed policy (fast-by-default via a learned duration cache, longest-first scheduling, the
never-slow-tagged reduce spine) lives in `conftest.py` + `pytest.ini` — no test file is edited, so
the derive/reduce spine stays pristine.

## Stack
Python (standard library) + matplotlib for the plot. No other dependencies.
