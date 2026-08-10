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
> **Write each new rung row in ≤350 bytes** — name, HEADLINE, cross-rung verdict.
> No measured numbers, no mechanism, no class names (those live in the spec and § Layout).

## The rungs

The model is built in cumulative **rungs** — each adds one physical effect and is anchored to a
published case. All rungs are live; the current scope is **rung 79**. Read a rung's spec (last
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
| 49 | **The φ / surge-margin FEEDBACK limiter** — watches the **PROTECTED variable**; first window with BOTH edges inside the ramp. **HEADLINE: a limiter acts on a spool through BOTH edges, on DIFFERENT clocks** — engagement credits, release **DEBITS**, so an LP floor debits the HP. Inverts 41/44/45. | `docs/rung49-spec.md` |
| 50 | **The release edge, ISOLATED** (`s_off` — forced, stateless) — the closing edge **RELOCATES BOTH SPOOLS' MINIMA TO ITSELF**, under a two-branch `min` law. **THE SEAM CLOSES**: rung 48's immunity is **TIMING, not clip SHAPE**. | `docs/rung50-spec.md` |
| 51 | **The release RATE** (`τ_rel`, a stateless fade) — **the debit is NOT a functional of the applied-fuel trajectory**: the two HARD releases at a fade's own ends bracket it pointwise, yet the faded run lands **OUTSIDE** both. It answers to the **RATE**. | `docs/rung51-spec.md` |
| 52 | **The asymmetric fast-attack / slow-release LAG** — the **realisable** limiter 50/51 forced; `τ_rel` is never *read* pre-crossing, so the leg **PINS ITS OWN TRIGGER**. **HEADLINE: a self-releasing limiter CANNOT DEBIT THE SPOOL IT WATCHES** ⇒ rung 50's watched-side debit is an **ARTIFACT OF FORCING**. | `docs/rung52-spec.md` |
| 53 | **The VARIABLE STATOR** — the **first lever that moves the surge FLOOR**; both channels DERIVED, **zero new constants**. **HEADLINE: a margin is a DISTANCE, so a floor-moving lever makes it COORDINATE-DEPENDENT** — which **BOUNDS** rungs 36–52's `φ`-currency. Thrust-neutral, paid in **SHAFT SPEED**. | `docs/rung53-spec.md` |
| 54 | **The stator-row THROAT** — rung 53's refused half; shape DERIVED, LEVEL disclosed, every verdict a **threshold ON** it. **HEADLINE: so is a CONSTRAINT'S SEVERITY** — an upstream throat can **BIND, NEVER RELIEVE**, so capacity cannot buy back rung 53's overspeed. **CORRECTS** its turning-point concession. | `docs/rung54-spec.md` |
| 55 | **The STAGE STACK** — the compressor stops being ONE block; kinematics DERIVED (rungs 36–53 read the FRONT stage all along — **BOUNDED**), `K` a resolution. **HEADLINE: a POSITIONAL lever buys relief from the part it does not move**; and the row count has an **INTERIOR optimum**. Discharges rung 54's seam. | `docs/rung55-spec.md` |
| 56 | **PER-ROW CAPACITY** — rung 54's throat per ROW; the profile FIGHTS the loading ⇒ the binding row **MIGRATES** with power. **HEADLINE: a resolved machine's two binding rows are DIFFERENT rows — different END *and* SPOOL — and the one lever reaches only the wrong one.** **CORRECTS rung 54's** pure-LP exposure. | `docs/rung56-spec.md` |
| 57 | **The STATOR SCHEDULE on the TRANSIENT plant** — rung 53's lever inside an accel. **HEADLINE: a wall-moving lever has NO CLOCK** — its design-point law still predicts it off design, so rungs 46–52's engagement-timing law is a **POINT**-mover property: their CLOCK **BOUNDED**. **CORRECTS rung 53's P5.** | `docs/rung57-spec.md` |
| 58 | **The COMPOSITE min-select** — the stator schedule BESIDE a fuel-side leg. **HEADLINE: two levers DO NOT SUPERPOSE**, and a pair composes only in coordinates neither moves — a `φ`-referenced leg **cannot compose at all**. **CONFIRMS rung 57** on the delivered credit; **EXTENDS rung 53** to composability. | `docs/rung58-spec.md` |
| 59 | **The MATCHED schedule** — the fuel leg re-derived on the plant it runs on. **HEADLINE: a schedule's ORDINATE cannot see a stator, only its INDEX can**, so matching is **pure RE-INDEXING** — a no-op when the lever leaves the abscissa alone. **Discharges rung 58's concession as VACUOUS.** | `docs/rung59-spec.md` |
| 60 | **The MATCHED `φ` FLOOR** — rung 58's refused repair. **HEADLINE: a floor PINS its own coordinate**, so re-referencing to the wall a stator cannot move buys ADMISSIBILITY but leaves a TAUTOLOGY (`=v` / `=0`, exact). A leg that SETS a minimum cannot compose, in ANY coordinate. **CORRECTS rung 58's diagnosis.** | `docs/rung60-spec.md` |
| 61 | **Stator + BLEED together** — rungs 36/41's concession, both halves. **HEADLINE: a compensating lever buys back the COORDINATE, not the BILL** — the `φ`-debit goes exactly, 73–102% of the overspeed stays (it was a loading REBATE). "Takeover" **REFUTED**; **CORRECTS rung 53's** per-spool cleanliness. | `docs/rung61-spec.md` |
| 62 | **The BLEED SCHEDULE beside the stator schedule** — rung 61's seam. **HEADLINE: a state-fed schedule's LOOP has a SIGN** (`dn/d·`), so the bleed **AMPLIFIES** where rung 57's stator self-cancels; two loops through ONE state don't compose (**ONE-WAY**). **CORRECTS rung 61's** superposition. | `docs/rung62-spec.md` |
| 63 | **FUEL + BLEED on one plant** — rung 62's seam. **HEADLINE: a fuel schedule's TABLE has exactly two guards, and only a MASS-extracting lever reaches them** — so rung 59's invariance is about the core-mass identity, not about schedules; the re-timing that follows is real but TRAJECTORY-bounded. **BOUNDS 58/59; EXTENDS 60** (a floor DISARMED). | `docs/rung63-spec.md` |
| 64 | **The φ-REFERENCED BLEED LIMITER** — rung 63's seam; the first CLOSED loop on an airflow lever. **HEADLINE: a limiter's LAW cannot buy PROTECTION, only its PRICE** — the ceiling is the lever's AUTHORITY, so feedback's whole content is the BILL. **INVERTS 61; BOUNDS 46–52's CEILING** as 53 did their currency. | `docs/rung64-spec.md` |
| 65 | **The LAGGED VALVE** — rung 64's seam; the position a THIRD STATE. **HEADLINE: a lag repairs the SOLVE without removing the DEGENERACY** — two loops on one variable stay redundant, the mode just moves into the STATE. Bandwidth is a 2nd hardware axis and PURE LOSS. **SHARPENS 64 §3; LOCALISES its §4 bound.** | `docs/rung65-spec.md` |
| 66 | **The TWO-LAG CASCADE** — rung 65's seam. **HEADLINE: two loops on one variable are ONE loop with the RATES ADDED** — `R_q·C_g ≡ 1` is an IDENTITY, so `det J ≡ 0`: a 2nd limiter buys BANDWIDTH, not AUTHORITY (38x credit erosion). **CORRECTS 65** — the freeze was the MANIFOLD; **EXTENDS 64**. | `docs/rung66-spec.md` |
| 67 | **CASCADE A — two loops on TWO variables** — rung 66's seam. **HEADLINE: ONE SCALAR sets both faces**: `P=R_q·C_g<0` ends the degeneracy (so the pair buys AUTHORITY — erosion ~1× vs 38×) and opens a ringing window in `τ_v/τ_g`, but the same `P` damps it — **admissible, unobservable**. **SPLITS rung 66 § 8.** | `docs/rung67-spec.md` |
| 68 | **THREE loops on ONE variable** — rung 66's seam; a φ STATOR limiter beside the valve and the fuel leg. **HEADLINE: `n` loops on one variable are ONE loop with ALL `n` RATES ADDED** — rank ONE at every `n`, so only the CYCLIC product tests it. **EXTENDS rung 64**: authority is inert in company, binding alone. | `docs/rung68-spec.md` |
| 69 | **THE REFERENCE SPLIT** — rung 68's stator, referenced to **INCIDENCE**. **HEADLINE: a loop's COORDINATE decides whether it adds a ZERO or a RANK** — `zeros = n − m`, `m` = CONSTRAINT count. `det J` is BLIND to it; `c1` is not. Freed pair RINGS iff the lever fights itself. **CORRECTS rung 68's** reading. | `docs/rung69-spec.md` |
| 70 | **THE GENERIC SPLIT** — rung 47's `Tt4` GOVERNOR as the odd loop; rungs 68/69's seam from BOTH sides. **HEADLINE: the split buys the RANK, but the RING needs the odd constraint to be a SECOND WALL ON THE SAME LEVER** — the floor is rung 67's `ζ`. **RETIRES rung 68's** cyclic summary. | `docs/rung70-spec.md` |
| 71 | **THE FULL SPLIT** — rung 69's INCIDENCE stator beside the governor: `n`=`m`=3, **ZERO zeros**, the last cell. **HEADLINE: a constraint can be INDEPENDENT in RANK and REDUNDANT on the BAND**, so `zeros`=`n−m` counts GRADIENTS, not LIVE loops; `det J` finally ≠0 and FACTORS. **CORRECTS rung 70's** erosion law. | `docs/rung71-spec.md` |
| 73 | **THE APPLIED REFERENCE** — rung 72's sharpest seam: a leg reads the fuel actually burnt. **HEADLINE: `F_r≠0` HOLDS yet the BLOCK SURVIVES — triangularity is MIN-SELECT's, not the reference's**; the masked pole hits the ORIGIN, so `zeros` gains `n_masked` and `det J` DIES. **CORRECTS rung 72's** ledger 110×. | `docs/rung73-spec.md` |
| 72 | **TWO LOOPS ON ONE ACTUATOR** — rung 52's fuel leg beside the governor; `n`=4, rungs 70/71's seam. **HEADLINE: a shared actuator adds a SWITCH BETWEEN PLANTS, not a loop** — min-select MASKS a leg, so ONE plant IS rung 68/69/70/71 by AUTHORITY and `zeros`=`n_live`−`m_live`. **CLOSES `n`=4 by REFUTING it.** | `docs/rung72-spec.md` |
| 74 | **THE DEMAND COORDINATE** — rung 73's sharpest seam: a leg lags the fuel it ALLOWS. **HEADLINE: a STATE's coordinate is PURE BILL — no rank, and the cut moves by the SCHEDULE'S SLOPE**; the redline break is the COORDINATE. **CORRECTS 47 and 73 §0.2.** | `docs/rung74-spec.md` |
| 75 | **THE ANTI-WINDUP DEVICE, DECLARED** — rung 74's accident given a RATE. **HEADLINE: a device is decisive on the SPECTRUM and inert on the RANK** — the masked pole leaves the ORIGIN and `det J` REVIVES, yet the term is in the masked ROW, so `n_live`≤3 a FOURTH time. **CORRECTS rung 74's** residual. | `docs/rung75-spec.md` |
| 76 | **THE FUEL-DEPENDENT CAP** — rung 73 § 11's seam, deferred by 73/74/75; zero new constants. **HEADLINE: a device in a leg's LAW reaches only the MASKED leg, one in the PLANT THE LEGS READ only the AUTHORITATIVE one** — `min` masks a law, not a plant; `n_live`≤3 a FIFTH time, so the obstruction is the COMPOSITION. **BOUNDS rung 48.** | `docs/rung76-spec.md` |
| 77 | **THE STIFFNESS LEDGER** — rung 76 § 8's seam, REFUTED; no knob, constant or plant code. **HEADLINE: a solve's sensitivity is a FORCING OVER A SLOPE, so `1/(1−c)` is the SLOPE HALF of ONE leg** — the others' set points are CONSTANTS: a stiffness, never a gain. | `docs/rung77-spec.md` |
| 78 | **THE RESIDUAL GAUGE** — rung 77 § 9's seam, CLOSED by REFUTING it; one swept knob, zero constants. **HEADLINE: a residual's SLOPE is a GAUGE, its root's UNIQUENESS is not** — `1/(1−c)` reaches ∞ without moving `dw*/dq`, but a SECOND root COLLIDES at `k·c`=1. **CORRECTS rung 77 § 3; BOUNDS rung 76 § 3.** | `docs/rung78-spec.md` |
| 79 | **THE STATE COORDINATE** — rung 78 § 9's 4th seam: rung 60's incidence for rung 49's `φ`, zero constants (`T_c`/`v` CANCEL). **HEADLINE: a coordinate is a GAUGE the PLANT cannot REACH** — the branch making a leg AUTHORITATIVE restores the original one, so the two sets are DISJOINT. **BOUNDS rung 78.** | `docs/rung79-spec.md` |

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

**Current scope (rung 79).** The **cycle solve** is a thermally-perfect, reacting,
dissociation-equilibrium gas (`Gas.reacting_equilibrium()`) through ideal + real components
(isentropic `η_c/η_t` **or** polytropic `e_c/e_t`, mutually exclusive; `π_d/π_b/π_n`, `η_b`,
`η_m`; dual cold/hot gas; specified exit pressure). The burner root-finds `f` over the scale-B
absolute balance, then freezes the station-4 mixture through turbine + nozzle; Fork A/B and
frozen-products gases are kept alongside. **Everything from rung 7 up is a diagnostic *beside*
the cycle**, reached through **separate entry points** that leave the default
`build_turbojet(…).run(…)` design run **bit-for-bit rung 6**. Rungs **31–79** are the
STRUCTURAL / DYNAMIC ones — a new off-design or transient operating point, through the two
matcher ladders mapped in § Layout; rungs **7–30, 36, 41, 44, 45** are pure diagnostics that
only *read* the design-point / running-line state. Each rung reduces to its predecessor
exactly and by test — the gates are named in its spec.

## Deferred seams — status map
One line per seam — `BUILT` (detail in its rung's spec) · `NEGATIVE → doc` · `OPEN`.

**BUILT — every seam numbered 25–79 was closed by the SAME-NUMBERED rung**, whose one-line hook is
the table above and whose derivation is in its spec. A numbered seam is closed and must never be
re-opened by mistake. (This entry named all of 61–79 until rung 79; that enumeration duplicated the
table and grew per rung, which the size guard says to DELETE rather than pay for.)

**Investigated, NEGATIVE — not shipped, not a rung (these facts live only here + the doc):**
- Resolved `τ_res` from the nozzle area-schedule (rung 26's seam a) — `docs/tau-res-negative.md`.
- Finite-rate turbine march (rung 29's seam a) — `docs/turbine-march-negative.md` (`I_turb ≡ S`).
- Locally-resolved mixing **SCALE** — `docs/mixing-scale-negative.md` (unanchored exponent `p`).
- Anchored `δ(J)` law via a JICF trajectory — `docs/mixing-jicf-anchor-negative.md` (a SECOND unanchored exponent).
- Lagged/filtered `pt3` sensor on rung 48's leg — `docs/pt3-sensor-lag-negative.md` (CONFIRMS 48, corrects its SIGN).
- A limiter with BOTH edges inside the ramp — `docs/both-edges-limiter-negative.md`. Closes the WHOLE `pt3`-filter family; **UPGRADES rung 48's law**.
- **PER-ROW BLADING** (rung 56's seam) — `docs/per-row-blading-negative.md`. **OVER-DETERMINED** ⇒ capacity inert. **CORRECTS 55.**
- The **φ-RATE limiter** (rung 60's seam) — `docs/phi-rate-limiter-negative.md`. **Fuel's authority over `φ` INVERTS between LEVEL and DERIVATIVE**. BOUNDS rung 49; the one negative with a gate.

**Checked, CONFIRMATION / CORRECTION — not a rung (the rung-29/28 margin sweeps):**
- "Earned at design" over `π_c` — `docs/rung29-pi-c-margin.md` (holds ~9.4×; `π_c` NOT protective; `ENERGY = INVENTORY × COMPLETION`).
- "Earned at design" over flight `M0` — `docs/rung29-M0-margin.md` (holds ~8.8×; monotone-protective; the `delta_h`-swing correction).
- `β<1` over `π_c` / hotter cycles — `docs/rung28-beta-margin.md` (β pressure-invariant; higher `π_c` protective).

**Still OPEN — not yet built (the live to-build list):**
- **The real spatial / transported-CFD PDF** — the standing mixing ceiling (rungs 22–24 are a Gaussian-plume cartoon; needs an anchored SCALE + spread law).
- **A per-pocket clamp that fires AT THE BURNER** (`max_a>1` at station 4) — a slow-enough freeze on a cooling pocket (20/21).
- **Detailed Fenimore** (`CH+N₂→HCN`) and **super-eq-O radical decay** — need new species / a relaxing pocket.
- **Reacting-gas fuel control** (35/43 defer — the forward burner asserts against an equilibrium gas).
- **The subsonic / unchoked LP branch** in the two-spool solves (rung 38 flags, does not solve) and its **transient**.
- An **ANCHOR for the blading**: a stress / tip-Mach limit pinning `U` from outside the stack, or an annulus law `Vx(k)`.
- **Fuel + bleed + STATOR**, all three on one plant (rung 63's seam, untouched by 64/65).
- **Rungs 69–79's seams — in each rung's spec, not here.** Shortlist: THREE legs on one actuator; the DEMAND/ANTI-WINDUP pair for the valve/stator. `n`=4 needs rung 69's 4th LP lever or a non-`min` composition — every route TRIED (72–76) is CLOSED, and 76 names the COMPOSITION as the obstruction; **79 adds the SOLVER's short-circuit as a second one.**
- **Rung 37's internal clocks on two shafts** and the combined 3-state; **customer/cooling bleed** at station 3.
- **Afterburner**; a **real hardware/CFD map + surge line** (rung 32's standing concession, now doubled across two spools).
- **Feeding a shifted/marched state into the production cycle** — a re-foundation, not a rung.

## Open engineering tasks (not rungs, not seams)
All three CLOSED, detail in the linked file: the per-rung `main.py` panel (no test covers it —
check on every ship); the solver-tolerance audit (NEGATIVE, `docs/plans/todo-solver-tolerance-audit.md`);
the fingerprint gate's rung 67–79 arms (that module's docstring §§ SLICE 3–4).

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
  (`thermal_nox`, `zoned_nox`, `nozzle_flow`, `exhaust_no_clamp`, and the nozzle/turbine marches
  `finite_rate_` / `freeze_out_` / `no_freeze_out_` / `coupled_no_freeze_out_nozzle`,
  `shifting_turbine`) with their configs — the mutually-exclusive mixing closures
  `JetMixing…SpatialLocalPDF`, and `FiniteRate`/`FreezeOut`/`NOFreezeOut`/`CoupledNOFreezeOut`.
- `turbojet/components.py` — `Inlet, Compressor, Burner, Turbine, Nozzle` as pure `apply(state, gas)`
  in `h`/`pr` form (+ loss params, `ram_recovery(M0)`, the polytropic knob). The Nozzle branches
  CPG/TPG and carries rung-30's `convergent=True` choke via the module-level `_sonic_throat`; rung-31's
  `choked_mfp` (the `pt`-independent sonic mass-flow parameter) lives here too. The `Burner` runs the
  implicit `f = g(f)` fixed point, or `_solve_equilibrium` for an equilibrium gas.
- `turbojet/engine.py` — chains the components, solves the `Δh` + `η_m` shaft balance, scores
  performance (`_score`). Home to the **off-design / transient matcher ladders**, each a **separate
  entry point** (the design `run` is untouched). Single-spool: `OffDesignMatcher` (31) → `MapMatcher`
  + `ComponentMap` (32) → `._match_subsonic` (33) → `SpoolTransient` (34, + fuel 35, + surge 36) →
  `CombustorTransient` (37). Two-spool: `build_two_spool_turbojet` / `TwoSpoolMatcher` (38) →
  `TwoSpoolMapMatcher` (39) → `TwoSpoolTransient` (40, + surge 41, + transient surge 44) →
  `TwoSpoolBleedMatcher` (42) → `TwoSpoolFuelTransient` (43, + transient surge 45) — then the whole
  **fuel-side limiter family** as keywords on its `integrate_fuel`: TIT topping governor (46), its
  lag `τ_gov` (47), the feedforward `Wf/pt3` `AccelSchedule` (48), the **φ-FEEDBACK** `SurgeLimiter`
  (49), forced release `s_off` (50), its rate `τ_rel` (51), the realisable `AsymmetricLag` (52).
  Off the fuel path, on the **steady** two-spool matcher: `VariableStatorMatcher` (53) — the first
  **floor-moving** lever, with its derived `ComponentMap` channels (`with_vsv`, `phi_surge_at`,
  `tan_beta1`) plus rung 54's `with_capacity`; then `StageStack` + `StageStackMatcher` (55–56),
  the compressor resolved into `K` stage blocks (it replaces the speed-line inversion ONLY; the
  **transient** ladders never see it), and `StatorBleedMatcher` (61). Back on the **transient**
  ladder, `ScheduledStatorTransient` (57–60: `StatorSchedule`, one fuel leg beside it, matched,
  `IncidenceLimiter`) → `ScheduledBleedTransient` (62–63: `BleedSchedule` threaded through the
  FORWARD closure, then rung 63's READERS beside a fuel leg, built on `at_lever`) →
  `LimitedBleedTransient` (64: the `BleedLimiter` φ FLOOR, an outer root over closures) →
  `LaggedBleedTransient` (65: that limiter's `tau` makes the POSITION a third state) →
  `TwoLagCascadeTransient` (66) → `CrossLoopCascadeTransient` (67) → `ThreeLoopCascadeTransient`
  (68) → `ReferenceSplitTransient` (69) → `CrossSplitTransient` (70) → `FullSplitTransient` (71) →
  `SharedActuatorTransient` (72) → `AppliedReferenceTransient` (73) → `DemandCoordinateTransient`
  (74) → `AntiWindupTransient` (75) → `SensedCapTransient` (76) → `StiffnessLedgerTransient` (77) → `ResidualGaugeTransient` (78) → `StateCoordinateTransient` (79).
  Each adds ONE thing — a state, a clock, a knob, or (77) nothing but a reader — and reduces to its
  predecessor; **what each adds, its method names and its reduce contract are in that rung's
  spec, not here.**
- `main.py` — the design-point run: ideal-vs-real tables, the overlaid T–s diagram, and **one panel
  per rung** (each demonstrates that rung's load-bearing claim and states its honest scope).
- `tests/` — per-rung `test_rungN.py` (N = 1…79; plus the rung-1/2b/3/4/5 files). Every rung file
  carries that rung's **reduce-to-prior** gate plus its load-bearing claims — the gates are named in
  the spec. `test_claude_md_reference.py` is the size guard on this file;
  `test_phi_rate_limiter_negative.py` is the only NEGATIVE carrying a gate (it BOUNDS rung 49's
  bracket, which no per-rung gate watches — it owns no production code); and
  `test_numeric_fingerprint.py` is the only **ABSOLUTE-value** gate — the reduce spine compares
  two quantities from the SAME run, so it is blind to anything that moves both sides together.
  Its goldens are a committed **CPython** anchor; regenerating them needs CPython, never PyPy.
- `docs/rungN-spec.md` — the derivation, assumptions, concessions and gates for rung N.
  `docs/plans/rungN-anchor-*.md` — that rung's verified anchor data. `docs/plans/` also holds the
  living plan/tasks.

## Commands
- Run the model: `python main.py` · Install: see `requirements.txt` (a PyPy venv — § Stack)
- **The gate: `pytest`** — **EVERYTHING**, 1289 tests, **~9:06 at 1287** (PyPy, rung 79, box load
  unknown; a 18:31 at 1289 came off a LOADED box, so it neither differences nor replaces this).
  ONE gate; nothing is ever silently deselected, so no regression can hide. (`main.py` has no test.)
- **Iterate: `pytest -m "not slow"`** — 954 tests, **~1:54 at 890**. The run minus the expensive sweeps.
  `slow` is a LABEL you opt out of by typing, never a default. Only those: `pytest -m slow`.
- **WHEN to run the gate:** at session end (unless run shortly before), and after a code change.
  NOT at session start, NOT on a docs-only change, NOT "just to be sure", and **NEVER to refresh
  a timing** — take that from a run already happening, or leave it stale.
- One rung by hand: `python tests/test_rung2.py`

`conftest.py` holds the policy and why the three-gate tiering was retired; `--runslow` is accepted
and ignored, since everything runs anyway.

## Stack
**PyPy 3.11** in the repo venv `.venv` (`.venv\Scripts\activate`), not CPython — the gate is 6.2×
faster, and `psutil` is REQUIRED (without it `-n auto` means 16 workers). Install + why:
`docs/plans/todo-pypy-switch.md`. Otherwise stdlib + matplotlib for the plot.
