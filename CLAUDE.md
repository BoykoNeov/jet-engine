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
anchored to a published case. All rungs are live; the current scope is **rung 54**.

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
| 44 | **The transient two-spool surge line** — the accel drives BOTH spools toward surge, the **LP eats ~1.6–2.2×** (rung 41 survives dynamically), but the excursion is **SCHEDULE-slaved**: `ρ`-invariant, ramp-rate-driven, **mode-independent**. | `docs/rung44-spec.md` |
| 45 | **The transient surge line on the FUEL path** — rung 43's `Tt4`-overshoot is `ρ`-MONOTONE yet **never reaches the reference-free surge object** (raw min `φ` `ρ`-invariant <2%): a **currency trap on the surge axis**. Fuel ENLARGES the approach, compresses the LP-eats-more ratio. | `docs/rung45-spec.md` |
| 46 | **The TIT topping governor** — the **first fuel-side FEEDBACK**. Holding `Tt4≤Tt4_max` rebates surge on the **LATE non-binding HP** spool but is **MACHINE-ZERO on the EARLY binding LP** one: a relief **SPLIT**, the debit paid upstream of the governor's window. INVERTS rung 35. | `docs/rung46-spec.md` |
| 47 | **The lagged topping governor** (`τ_gov`; clip AMOUNT a 3rd state) — a first-order lag is a **TRAILING-edge** tool, so it cannot reach the EARLY LP min: `relief_lp=0` EXACTLY at moderate `r`, ERODED at fast `r`. **Cost of realism**: it breaks the redline hold (~55→190 K). | `docs/rung47-spec.md` |
| 48 | **The `Wf/pt3` acceleration schedule** — the **FEEDFORWARD** min-select leg. **UNIFIES 46/47**: a fuel-side limiter rebates a spool **IFF it engages UPSTREAM of THAT spool's OWN surge minimum**; `m` is an ENGAGEMENT-TIME dial, NOT rung 44's ramp-rate lever. | `docs/rung48-spec.md` |
| 49 | **The φ / surge-margin FEEDBACK limiter** — watches the **PROTECTED variable**; a clean sliding mode, first window with BOTH edges inside the ramp. **HEADLINE: a limiter acts on a spool through BOTH edges, on DIFFERENT clocks** — engagement credits (48), release **DEBITS**, so an LP floor debits the HP. Exposed spool = the **LATE** one (inverts 41/44/45). | `docs/rung49-spec.md` |
| 50 | **The release edge, ISOLATED** (`s_off` — forced, stateless, two-sided) — the closing edge **RELOCATES BOTH SPOOLS' MINIMA TO ITSELF**, under the two-branch law `min(rung-48 truncation, the dive bottoming at s_rel)`. **THE SEAM CLOSES**: rung 48's immunity is **TIMING, not clip SHAPE**. | `docs/rung50-spec.md` |
| 51 | **The release RATE** (`τ_rel`, a stateless fade) — **the debit is NOT a functional of the applied-fuel trajectory**: the two HARD releases at a fade's own ends bracket it pointwise, yet the faded run lands **OUTSIDE** both. The debit answers to the **RATE**. Scope gated as an explicit negative. | `docs/rung51-spec.md` |
| 52 | **The asymmetric fast-attack / slow-release LAG** — the **realisable** limiter 50/51 forced. **REFUTES rung 51's deferral reason**: `τ_rel` is never *read* pre-crossing, so the leg **PINS ITS OWN TRIGGER**. **A self-releasing limiter CANNOT DEBIT THE SPOOL IT WATCHES** ⇒ rung 50's watched-side debit is an **ARTIFACT OF FORCING**; `τ_att` owns the credit, the debit **JOINT**. | `docs/rung52-spec.md` |
| 53 | **The VARIABLE STATOR** — the **first lever that moves the surge FLOOR** (rung 42's named other half); both channels DERIVED from the map's own `l` and rungs 36/41's own `φ_surge` — **zero new constants**. **HEADLINE: a margin is a DISTANCE, and when the lever moves the boundary that distance is COORDINATE-DEPENDENT** — the `φ`-margin SHRINKS on closing while the incidence margin GROWS; only the coordinate whose boundary is FIXED measures a margin. Rungs 36–52's `φ`-currency **BOUNDED** (a floor-fixed lever *provably* cannot split them). Thrust-neutral, paid in **SHAFT SPEED**. | `docs/rung53-spec.md` |
| 54 | **The stator-row THROAT** — rung 53's refused half. The cosine rule DERIVES `A_th ∝ cos α₁` off rung 53's OWN `v`; only the LEVEL needs one disclosed `C` (a design throat Mach), every verdict a **threshold ON** it. **HEADLINE: rung 53 — a MARGIN is coordinate-dependent; so is a CONSTRAINT'S SEVERITY** (the throat cuts the SETTING 30 %, the MARGIN 4 %). An upstream throat can **BIND, NEVER RELIEVE** (reduce = invariance over `C`) ⇒ capacity **cannot** buy back rung 53's overspeed. **CORRECTS rung 53**: the turning point IS reached (3/5 shapes) — the ceiling is the incidence PEAK, not an artifact. | `docs/rung54-spec.md` |

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

**Current scope (rung 54).** The **cycle solve** is a thermally-perfect, reacting,
dissociation-equilibrium gas (`Gas.reacting_equilibrium()`) through ideal + real
components (isentropic `η_c/η_t` **or** polytropic `e_c/e_t`, mutually exclusive;
`π_d/π_b/π_n`, `η_b`, `η_m`; dual cold/hot gas; specified exit pressure). The burner
root-finds `f` over the scale-B absolute balance (re-solving equilibrium each trial),
then freezes the station-4 mixture through turbine + nozzle. Fork A/B and
frozen-products gases are kept alongside. **Everything from rung 7 up is a diagnostic
*beside* the cycle**, reached through **separate entry points** that leave the default
`build_turbojet(…).run(…)` design run **bit-for-bit rung 6**. Rungs **31–53** are the
STRUCTURAL / DYNAMIC rungs (a new off-design or transient operating point — the
single-spool ladder `OffDesignMatcher → MapMatcher → SpoolTransient → CombustorTransient`
and the two-spool ladder `TwoSpoolMatcher → TwoSpoolMapMatcher → TwoSpoolTransient →
TwoSpoolBleedMatcher → TwoSpoolFuelTransient`, with rung 44's transient-surge methods on
`TwoSpoolTransient`, rung 45's on `TwoSpoolFuelTransient`, rung 46's **fuel-side FEEDBACK**
topping governor on it, rung 47's **response lag** `τ_gov` on that governor, rung 48's **feedforward** `Wf/pt3` leg
beside it, rung 49's **φ-FEEDBACK** floor beside that, rung 50's **forced release** `s_off` across the
last two, rung 51's **release RATE** `τ_rel` on that, and rung 52's **realisable** asymmetric
lag beside the whole forced family; and rung 53's **variable stator** `VariableStatorMatcher`
beside the *steady* two-spool matcher — the first **floor-moving** lever, off the fuel path
entirely — with rung 54's **THROAT** a post-hoc read on it); rungs **7–30, 36, 41, 44, 45** are pure
diagnostics that only *read* the design-point / running-line state. Each rung reduces to its
predecessor exactly and by test — the gates are named in its spec.

## Deferred seams — status map
One line per seam: `BUILT BY RUNG N` (detail in its spec) · `NEGATIVE → doc` (investigated,
not shipped, not a rung) · `OPEN` (not yet built). This list is the live map of what is
closed vs open — keep it one line per entry.

**Built — each seam → the rung that closed it. The rung table above is the one-line hook for
each; the derivation is in its spec. Listed here only so a seam is never re-opened by mistake:**
- 25 finite-rate nozzle · 26 freeze-out · 27 NO freeze-out · 28 coupled NO march · 29 shifting turbine · 30 choked convergent nozzle.
- 31 off-design matching · 32 component maps · 33 subsonic branch · 34 spool transient · 35 fuel metering (`Tt4` an output) · 36 surge line · 37 combustor internal clocks.
- 38 two-spool matching · 39 two-spool + maps · 40 two-shaft transient · 41 two-spool surge line · 42 bleed valve · 43 two-shaft fuel metering · 44 transient surge · 45 transient surge on the FUEL path.
- 46 TIT topping governor · 47 its lag `τ_gov` · 48 the `Wf/pt3` feedforward leg · 49 the **φ-FEEDBACK** limiter (the door the `pt3`-filter negative left open) · 50 the release edge ISOLATED (*why rung 48 escapes*) · 51 the release **RATE** `τ_rel` (deficit vs deficit-RATE) · 52 the asymmetric fast-attack/slow-release **LAG** — rung 51's own named seam, *refuting its deferral reason*: a realisable leg **pins its own trigger**.
- 53 the **VARIABLE STATOR** — rung 42's named other half, and the **first floor-moving lever**: it **BOUNDS the `φ`-margin currency** every rung 36–52 scored surge in. · 54 the stator **THROAT** — rung 53's own refused half: **BIND, NEVER RELIEVE**; CORRECTS its turning-point concession.

**Investigated, NEGATIVE — not shipped, not a rung (these facts live only here + the doc):**
- Resolved `τ_res` from the nozzle area-schedule (rung 26's seam a) — `docs/tau-res-negative.md` (shape moot; needs an entry Mach). Confirms rung 26.
- Finite-rate turbine march (rung 29's seam a) — `docs/turbine-march-negative.md` (`I_turb ≡ S`, entry at equilibrium; two un-anchored knobs).
- Locally-resolved mixing **SCALE** — `docs/mixing-scale-negative.md` (the turn rides on the unanchored penetration exponent `p`).
- Anchored `δ(J)` law via a JICF trajectory — `docs/mixing-jicf-anchor-negative.md` (confirms rung 22; emissions optimum rides on a SECOND unanchored exponent, spread).
- Lagged/filtered `pt3` sensor on rung 48's leg — `docs/pt3-sensor-lag-negative.md` (an effective-margin reparameterisation; release edge structurally post-ramp. CONFIRMS rung 48, corrects its seam's SIGN).
- A limiter with BOTH edges inside the ramp (rate-limited / washout-filtered `pt3`) — `docs/both-edges-limiter-negative.md`. Closes the WHOLE `pt3`-filter family: **the ramp is the only clock**. Fuel-command rate limit = rung 44's lever BY IDENTITY. **By-product UPGRADES rung 48's law**: a clip ARRESTS the φ descent ⇒ min sits AT `s_eng` ⇒ `relief = min_{s≤s_eng} φ_bare − min φ_bare` (DERIVES the exact-zero) ⇒ EDGE not window.

**Checked, CONFIRMATION / CORRECTION — not a rung (the rung-29/28 margin sweeps):**
- "Earned at design" over `π_c` — `docs/rung29-pi-c-margin.md` (verdict holds ~9.4×; `π_c` NOT protective; `ENERGY = INVENTORY × COMPLETION`).
- "Earned at design" over flight `M0` — `docs/rung29-M0-margin.md` (holds ~8.8×; monotone-protective; the `delta_h`-swing correction).
- `β<1` over `π_c` / hotter cycles — `docs/rung28-beta-margin.md` (β pressure-invariant; higher `π_c` protective).

**Still OPEN — not yet built (the live to-build list):**
- **The real spatial / transported-CFD PDF** — the standing mixing ceiling (rungs 22–24 remain a Gaussian-plume cartoon; `C_opt`/dwell still ride on `k_p`/`τ_mix`; needs an anchored SCALE + spread law, or a real CFD cross-plane).
- **A per-pocket clamp that fires AT THE BURNER** (`max_a>1` at station 4) — lever is a slow-enough freeze on a cooling pocket (rungs 20/21 confirm it is not a hotter `Tt4`).
- **Detailed Fenimore** (`CH+N₂→HCN`) and **super-eq-O radical-decay history** — need new species / a relaxing pocket a 0-D pool cannot derive.
- **Reacting-gas fuel control** (rungs 35/43 defer — the forward burner asserts against an equilibrium gas).
- **The subsonic / unchoked LP branch** in the two-spool solves (rung 38 flags, does not solve) and its **transient**.
- **STAGE REMATCHING — the stage stack** — rung 54's named seam: it *refutes* capacity as the reason a real engine escapes rung 53's overspeed, and rematching is what actually does it (`K` stage blocks sharing `τ_c`, `K` a disclosed integer; the "front stalls / rear chokes" mechanism a lumped block cannot have). Then a **stator schedule `v(n)` on the TRANSIENT plant** (the first lever that could move the wall *during* an accel), and **stator + bleed together** (rung 53's saturation says the bleed takes over where the stator's authority runs out).
- A **bleed schedule** `b(n_L)`; **fuel + bleed together**.
- **The lag's SHAPE and the two-lag CASCADE** — rung 52's named seam: a second-order / rate-limited *attack* (the valve, not the loop), and `tau_gov` + `lag` together (redline lag + surge lag on one plant — what a real FADEC runs), which rung 52 refuses as a different instrument. Its §3 non-factorization says the cascade should not be additive.
- **Rung 37's internal clocks on two shafts** and the combined 3-state; **customer/cooling bleed** at station 3.
- **Afterburner**; a **real hardware/CFD map + surge line** (rung 32's standing concession, now doubled across two spools).
- **Feeding any shifted/marched state into the production cycle** — a re-foundation (re-anchors every rung's numbers), not a rung.

## Open engineering tasks (not rungs, not seams)
- **Audit the iterative solvers for absolute-tolerance-below-noise-floor** — **CLOSED, NEGATIVE**
  (`docs/plans/todo-solver-tolerance-audit.md`). Rung 43 fixed a real rung-40 `_EQ_TOL` hole; the
  six `_ETA_TOL`=1e-11 efficiency secants do **not** share it (exact float64 root). No code change.

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
  `TwoSpoolMapMatcher` (39) → `TwoSpoolTransient` (40, + surge methods 41, + transient-surge methods 44)
  → `TwoSpoolBleedMatcher` (42) → `TwoSpoolFuelTransient` (43, + transient-surge methods 45, + the
  fuel-side-FEEDBACK TIT topping governor `integrate_fuel(Tt4_max=…)`/`topping_relief` 46, + its
  **response lag** `integrate_fuel(…,tau_gov=…)`/`_integrate_fuel_lagged`/`topping_command_trace` 47,
  + the **feedforward** `Wf/pt3` leg `AccelSchedule`/`accel_schedule`/`_sched_fuel`/`integrate_fuel(…,accel=…)`/
  `schedule_relief`/`engagement_sweep` 48, + the **φ-FEEDBACK** floor `SurgeLimiter`/`_surge_fuel`/
  `integrate_fuel(…,surge=…)`/`surge_relief`/`floor_sweep` 49, + the **forced release**
  `integrate_fuel(…,s_off=…)`/`release_relief`/`release_sweep` 50, + its **RATE**
  `integrate_fuel(…,tau_rel=…)`/`_release_weight`/`rate_sweep`/`deficit_curve` 51, + the
  **realisable** asymmetric lag `AsymmetricLag`/`integrate_fuel(…,lag=…)`/`_integrate_fuel_asym`/
  `lag_relief`/`lag_sweep`/`factorization_grid` 52). Off the fuel path, on the **steady** two-spool
  matcher: `VariableStatorMatcher`/`at_setting`/`stator_margin`/`stator_sweep`/`currency_split`/
  `throttle_currency`/`incidence_schedule` (53), with its two derived channels on `ComponentMap`
  (`with_vsv`/`psi`'s swirl term/`phi_surge_at`/`tan_beta1`/`tan_beta1_crit`); and rung 54's THIRD
  channel — `with_capacity`/`throat_ratio`/`throat_loading`/`capacity_margin` on the map,
  `throat_margin`/`throat_sweep`/`authority_ceiling`/`schedule_throat` on the matcher (a pure
  post-hoc read).
  Each reduces to its predecessor (exact dispatch, an inherited identity, or the forward
  closure); the method names + reduce contracts are in each rung's spec.
- `main.py` — the design-point run: ideal-vs-real tables, the overlaid T–s diagram, and **one panel
  per rung** (each demonstrates that rung's load-bearing claim and states its honest scope).
- `tests/` — per-rung `test_rungN.py` (N = 1…54; plus the rung-1/2b/3/4/5 files). Every rung file
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
