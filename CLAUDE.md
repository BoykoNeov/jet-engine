# Turbojet Cycle Simulator

A station-by-station model of a single-spool turbojet (Brayton cycle). It takes
flight + design conditions and produces the gas state at every station, the
thrust, the efficiencies, and a T–s diagram.

**The deliverable is understanding, not the tool.** The code is the medium that
forces every thermodynamic assumption into the open. Optimize the work for
teaching, not for features or polish.

## The rungs

The model is built in cumulative **rungs** — each adds one physical effect and is
anchored to a published case. All rungs are live; the current scope is **rung 34**.

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
| 25 | **Finite-rate nozzle chemistry** — `FiniteRate(Da,…)` / `Gas.finite_rate_nozzle(…)`: the Damköhler flow BETWEEN rung-14's bounds, on the exact `dh=v·dp` spine (species-vector relaxation). **INVERTS the seam into a THREE-state picture**: the frozen-in station-4 mixture arrives **super-equilibrium**, so a real (irreversible) flow re-equilibrates the entry **irreversibly even at `Da→∞`**. **(F)** frozen (`Da→0`, the exact reduce); **(I)** irreversible-fast (`Da→∞`, the ATTAINABLE ceiling, closed form); **(R)** rung-14 reversible (a STRICT **UNREACHABLE** ceiling above I). **Reduces to FROZEN, deliberately NOT to equilibrium** — the **(R−I)** gap quantifies rung-14's named "sliver of entry irreversibility" (dormant lean, ~7% of the bracket hot). Keystone certified: integrator `Da→∞` ⇒ closed-form (I). | `docs/rung25-spec.md` |
| 26 | **Freeze-out** — `FreezeOut(L,…)` / `Gas.freeze_out_nozzle(…)`: rung-25's scalar `Da` promoted to a **local** `Da(T,p)=τ_res/τ_chem(T,p)` from an **ANCHORED** GRI-Mech 3.0 clock (`H+OH+M`, `Ea=0`, `n=−2` — **zero new constants**), so the relaxation **shuts off partway down the nozzle** and the freeze point **MOVES with `Tt4`** (frozen-from-entry ≤1650 K → `s_freeze` walks 0.12→0.29→0.38 hot). **Refutes rung-25's own seam framing on BOTH counts**: the rate is anchored (not the "unanchored-Arrhenius trap"), and freeze-out is **density-driven** (`c_tot²∝(p/T)²`) **against an opposing `T` effect** (`k` *accelerates* on cooling — kill-tested, opposite sign to Arrhenius). Adds **no new bound** (lands inside rung-25's `[F,I]`); the **moving freeze point is the finding**, `s_freeze`/location disclaimed (rides on the one geometric knob `L`). Reduce: constant `Da_local` ⇒ rung-25 `_finite_rate_expand` **bit-for-bit**. | `docs/rung26-spec.md` |
| 27 | **NO freeze-out** — `NOFreezeOut(L,…)` / `Gas.no_freeze_out_nozzle(…)`: rung-26's anchored-clock/local-`Da` machinery applied to **exhaust NO** via a `_tau_no_destroy` clock from **rung 7's OWN Zeldovich reverse rates** (`NO+O`, `NO+H` — **zero new constants**, already K-checked), asking whether the **frozen-NO assumption every NO number has carried since rung 7** (and the rung-14/17 clamp reads OFF) is EARNED. **It is: `Da_NO≪1` from ENTRY at every `Tt4`** (3–9 orders clear — frozen-from-entry *everywhere*, unlike rung-26's major pool), on an **upper bound** (radical-rich frozen pool = fastest possible relaxation), robust to the NO level (the super-eq clock is `[NO]_e`- and `a`-independent). **The kill test INVERTS rung 26's**: this clock is **Arrhenius** (`θ≈20820/24560 K` ⇒ `k` *craters* on cooling) AND **bimolecular** (`c¹`), so its two factors **AGREE — both DRIVE** (vs rung-26's density-DESPITE-temperature). A **CONFIRMATION** that retires the clamp corollary's last premise; **no moving freeze point** (rung-26's headline has no analogue — `s_freeze_NO≡0`), the honest trend is the Da_NO-vs-Da_recomb **separation narrowing** with `Tt4` (3.7e7→2.2e3, no crossing). Reduce: `Da_NO≡0` ⇒ rung-14/17 clamp `max_a` **bit-for-bit**. | `docs/rung27-spec.md` |
| 28 | **The rung-26-coupled NO march** — `CoupledNOFreezeOut(L,…)` / `Gas.coupled_no_freeze_out_nozzle(…)`: rung-27's NO clock read on rung-26's **relaxing** pool (one-way, pool→NO). Rung 27's **verdict is CONFIRMED and BOTH its reasons are CORRECTED**. (a) "**can ONLY slow NO further**" is one-sided — coupling to rung 26 couples to its **exothermic heat release**, which lifts `T` and (this clock being **Arrhenius**) **SPEEDS** NO destruction: **two OPPOSING channels**, decomposed by running one clock on two **hybrid trajectories**. `net<1` everywhere (conclusion holds) but `ch2>1` always, `|ln ch2/ln ch1|` rising **monotonically 0.003→0.48** — the opposing channel cancels **~half** the depletion hot. The win is **STRUCTURAL**: depletion **UNBOUNDED** (`ch1→0`) vs heat release **SATURATING** (capped by finite frozen-in chemical enthalpy) — certified over 6 orders of `pool_rate_scale`. (b) The **β repair** — rung 27 justified its `a≫1` clock with "NO arrives super-equilibrium", **false at the ENTRY** (`a`=0.31–0.61 hot; NO arrives **SUB**-eq and tries to FORM) where freeze-from-entry is decided. What holds is **`β=R1/(R2+R3)<1`** ⇒ `τ_ex/τ_surr=(1+u)²/[(1+u)²−(1−β²)]>1` for **all `a`** ⇒ an upper bound on the rate in **BOTH** regimes. **Rung 27's numbers are unaffected**; only its reasoning is repaired. Headline **structurally unreachable** (entry `Da_NO` bit-for-bit rung 27's — path-independent). Disclaimed: the net **turnaround location** (rides on `L`), **β<1 as a theorem** (margin ~0.51 hot — a factor 2, the honest weak point). Reduce: `couple=False` ⇒ rung 27 **bit-for-bit** (structural, via `_frozen_no_trajectory`). | `docs/rung28-spec.md` |

| 29 | **The shifting turbine** — `Gas.shifting_turbine(…)` / `_work_limited_expand(…)`: the question every rung since 6 skipped — is **FREEZING the turbine** earned? Brackets the turbine the way rung 14 bracketed the nozzle (frozen vs fully-shifting) but on a **WORK-limited** endpoint, the one structural novelty: the shaft fixes `delta_h` (compressor + `f` only), so a shifting turbine reopens **NO shaft fixed point** — it moves where the flow *ends up*. Two unknowns `(T5,p5)`, two equations (work-limited `H_abs` drop + reversible `S`), on **absolute** enthalpy (composition changes ⇒ formation enthalpy no longer cancels). **Zero knobs, no rate ⇒ the verdict is RATE-INDEPENDENT.** Verdict: **EARNED at design** (`ΔT5/T5`=0.011% at `Tt4`=1500 — an order below the cycle's own `η_t`/`π_b` precision) and **BITES HOT** (1.86% / `Δp5`=0.47% at 2400, a **174×** growth) ⇒ the freeze is a **design-point fact, not a structural one**. The rung is the **inversion: RATIO ≠ ENERGY** — rungs 25–28 justify the super-eq entry with a *ratio* (`x_frozen/x_eq`, 10–100×), correct for **kinetic** distance but **not** a proxy for exploitable **enthalpy**, which scales with the absolute radical **INVENTORY**; across the band ratio **÷33** while inventory **×121** and shift **×174** — the ratio is **loudest exactly where the shift is most negligible** (109× of ~3e-5 is still nothing). A **cross-rung correction**. Disclaimed: `(R−I)→0` on a shifted entry is **STRUCTURAL, not a finding** (an entry at equilibrium has no super-eq left to relax — a tautology); `η_t=1` **by nature** (reversible ⇒ isentropic, same concession rung 14 makes); the rate **deferred** (turbine `τ_res` un-anchored — `Da_turb`=0.05–8.8, a *supporting sketch* only, and notably **not fast despite high `p`**). Reduce: frozen branch **delegates** to `Turbine.apply` at `η_t=1` ⇒ **bit-for-bit by construction**. | `docs/rung29-spec.md` |

| 30 | **The choked convergent nozzle** — `Nozzle(convergent=True)` / `_sonic_throat(…)`: is **FULL EXPANSION** — assumed since rung 2 — EARNED? Every thrust number has expanded fully to `p0`, which at design means `M9`=**1.86 (SUPERSONIC)** — silently a **C-D** nozzle. A fixed **convergent** nozzle (the subsonic-engine choice, and the fixed throat **rung 31 needs**) can reach only `M9`=1 and **chokes**. Brackets the two like rung 29 bracketed the turbine; **zero new knobs, no rate**. The one novelty: a convergent nozzle lets the **FLOW decide `p9`** (not a told back-pressure) — a **choke test + branch**. The **TPG sonic throat** root-finds `h_t(Tt9)−h_t(T*)=½γ_t(T*)R_tT*` (the velocity↔enthalpy trap again); `p*=pt9·pr_t(T*)/pr_t(Tt9)`; CHOKED ⇔ `p*>p0`. **Verdict: NOT earned at design** — `pt9/p0`=6.29 (crit ~1.85), chokes hard (`p*`=170.85 kPa, **underexpanded 3.4×**), `V9` drops 38% and momentum thrust **51%**, yet specific thrust falls only **6.6%** (`798→746`, TSFC +7.1%). **The finding: the pressure term rescues 87%** of the momentum deficit (`+356` N·s/kg direct pressure thrust = **48%** of the choked total) — the gap between "51% loss" and "6.6% loss" is why high-PR engines fit C-D/variable nozzles, and it is the pressure-thrust term the cycle has carried honestly since rung 2. **Diagnostic beside the cycle** (default `convergent=False` = ideal expansion ⇒ cycle **bit-for-bit rung 6**). Reduce: subcritical convergent ⇒ full expansion `p9=p0`, `M9<1` **bit-for-bit**; sonic solver ⇒ CPG closed-form critical ratio (`p*/pt`=0.5283@γ=1.4) to machine precision on a self-consistent gas (gate 2, non-tautological). Disclaimed: **fixed throat AREA deferred to rung 31** (rung 30 supplies the choke physics, not the `A9` that pins off-design); convergent-only (no C-D/variable modelled — the shipped path *is* the C-D reference). | `docs/rung30-spec.md` |

| 31 | **Off-design matching — the operating point becomes an OUTPUT** — `OffDesignMatcher.match(…)` / `choked_mfp(…)` / `_score(…)`: every rung so far SPECIFIED `π_c`+`Tt4`; real hardware is FIXED. The **first STRUCTURAL rung** — `π_c` is no longer a knob but the **OUTPUT of a matching solve**. With the **turbine NGV + the rung-30 convergent nozzle both CHOKED**, two mass-flow-parameter constraints (`ṁ√Tt/(A·pt)=MFP*`, computed EXACTLY from rung-30's `_sonic_throat`, `pt`-independent) pin the turbine: `π_t/√τ_t=A4·MFP4/(A8·π_n·MFP9)` (★). **The inversion**: at design the shaft SETS turbine work (`π_c` in); off-design `τ_t` is pinned by the choke and the shaft instead hands back the **compressor** (`τ_c-1 ∝ Tt4/(τ_r·T0)`, `π_c=[1+η_c(τ_c-1)]^(γc/(γc-1))` OUT). Zero new knobs, no rate. **Verdict: the choked hardware STRIPS the compressor of freedom** — `π_c`, `ṁ`, thrust ride ONE fixed **running line** (pumping characteristics **WITHOUT a compressor map** — the choked downstream hardware *is* the map: `π_c` 10→4.0 as `Tt4` 1500→800). **The finding: `τ_t` DRIFTS** — the textbook says a choked-turbine/choked-nozzle pair holds `τ_t` EXACTLY constant, but that is a **CPG** statement; on the real gas the two sonic `MFP*` sit at different `T` so `τ_t` drifts **2.8%** over a 2:1 throttle (0.16% on the `M0` axis). **Kill-tested by a 3-gas ladder**: CPG 0.000% → `thermally_perfect` (var `cp(T)`, FROZEN comp) **+2.30%** → reacting +2.84%, so the **`γ_t(T)` CURVE drives 81%** (composition the minority ~19%) — clean because within a point both throats share the frozen comp so `R` cancels in `MFP4/MFP9`. Same species as rung 30's "0.03% is the physics, not error". **Reduce-BY-CONSTRUCTION** (the rung-29 move): matching at design returns `π_c`=10 (5e-10), all stations/`ṁ`/thrust — the design reference IS the rung-30 choked-convergent point (specific thrust 745.7); A4/A8 captured from it, the compressor inverse is the exact inverse of `Compressor.apply`. **Gate 2 (non-tautological)**: on a self-consistent CPG gas the sonic-throat matching reproduces Mattingly Ch-8 closed-form referencing (`τ_t`,`π_t` const to **machine zero**, the `Tt4/(τ_r T0)` slaving factor const, `π_c` closed form to 1e-14) — two code paths onto the same operating point. **Choked envelope**: throttle down → `pt9/p0` falls → nozzle **UNCHOKES near `Tt4`≈600** (`pt9/p0`<~1.85), the pin lost — the matcher **flags** `nozzle_choked=False` rather than lying; the subsonic-nozzle matching mode deferred. **Separate entry point** — the default `build_turbojet(…).run(…)` design path is untouched, so the production cycle stays **bit-for-bit rung 6**. Disclaimed: `η_c/η_t` held at design (the map curvature is **rung 32**); NGV choke ASSUMED (not an NGV passage); isentropic knobs only; the M0>1 inlet folds in `ram_recovery`. | `docs/rung31-spec.md` |

| 32 | **Component-map matching — the map re-labels the choke-pinned work** — `MapMatcher.match(…)` / `ComponentMap` / `_operating_point(…)`: rung 31 closed with "a pumping characteristic **WITHOUT a compressor map**". Rung 32 puts a **representative compressor + turbine map** on the matcher (`η_c/η_t = f(mdot_corr, N_corr)`) and the result is a **cross-rung CORRECTION** (the rung-29/28 move): **rung 31 over-claimed by holding `η` at design.** The choked hardware sets the **work schedule `τ_c(Tt4)` map-free** (that part of rung 31 SURVIVES — `τ_c` matches to ~1e-4), but converting the work into `π_c`, `ṁ` and a **shaft speed `N`** needs the real map — and rung 31 never even computed `N`. **The structural novelty: `N` enters** (a compressor map is indexed by corrected speed; rung 31 traced the whole running line without it). **The finding: `π_c`/`ṁ` DROOP** — a peaked (peak-η-near-design) compressor map droops `η_c` off-design (throttle walks off the efficiency island), so `π_c`/`ṁ` fall **below** rung 31's constant-η line, **SAME SIGN across 3 map shapes**, gap growing with throttle (~−2% at `Tt4`=900; **magnitude shape-dependent, DISCLAIMED**). **Sub-finding (sharper than "turbine maps are flat"): the turbine barely moves for a STRUCTURAL reason** — on a single spool `nu_t=N/√Tt4` stays within **~1%** of design (N and √Tt4 fall together), so `|Δη_t|`~2e-5 **even for a 25×-steeper turbine map** vs `|Δη_c|`~1e-2 — **the compressor is where the map bites**, and rung 31's "hold `η_t` const" was nearly exact *because the turbine is pinned in corrected speed*. `N/N_d` falls monotonically (1→0.69) and its schedule is robust across the speed-line curvature `σ` (~few-% spread) — genuinely `σ`-dependent (NOT the tautological `√(τ_c−1)`), but **absolute rpm disclaimed** (needs blade geometry). **Representative-map closure** (rungs 12–24 methodology: shapes disclosed, load-bearing claims shape-robust, magnitudes disclaimed); **no surge line ⇒ no surge-margin claim** (the CRS payoff deliberately NOT made). The `η_c` feedback is POSITIVE (lower `η_c`→lower `π_c`→lower `φ,n`→lower `η_c`) ⇒ solved by a **secant** on `η_c` (a non-convergence assert guards the deep-throttle edge). **Reduce**: the **FLAT map** `{a=b=c=σ=a_t=0}` ⇒ `MapMatcher.match` == rung-31 `OffDesignMatcher.match` **bit-for-bit** (machine-zero at design, ≤1e-9 sweep; two code paths, one operating point) — `N` a passive diagnostic. **Separate entry point** (subclasses rung 31; default `run(…)` untouched ⇒ cycle **bit-for-bit rung 6**). Disclaimed: droop magnitude & absolute `N(Tt4)` (ride on the map); no surge line; `η_b/π_b/π_n` held at design; isentropic + choked-nozzle-branch only (inherited from rung 31). | `docs/rung32-spec.md` |

| 33 | **The subsonic-nozzle matching branch — the decoupling BREAKS** — `OffDesignMatcher._match_subsonic(…)` / the auto-dispatch in `.match(…)`: rung 31 flagged the nozzle-unchoke boundary (`Tt4≈600` at design) and **deferred** the second matching mode; rung 33 builds it. Below unchoke the nozzle is **SUBSONIC** (expands fully to `p0`, `M9<1`), so only the **NGV stays choked** — rung 31's two-choke pin `(★) π_t/√τ_t=A4·MFP4/(A8·π_n·MFP9)` (pure geometry ⇒ `τ_t,π_t` const) is **void**. The nozzle now passes the compressible-flow `MFP(M9)` with `M9=M9(pt9/p0)`, and `pt9/p0` moves with `π_c`, so `π_t` becomes the **equilibrating unknown** that matches NGV-choked supply to subsonic-nozzle demand: `resid(π_t)=ṁ_NGV−ṁ_noz=0` (★★), a 1-D root-find with the rung-31 `(f,pt4)` fixed point nested inside. Zero new knobs, no rate. **THE RUNG — the INVERSION of rung 31**: on the choked branch the coupling ran through the **`γ_t(T)` curve** (var-`cp`, 2nd order) so `τ_t` drifted on the reacting gas but was **machine-constant on CPG** (rung 31 gate 2); on the subsonic branch the coupling runs through **`π_c`** (structural, geometric, 1st order) so `τ_t` **VARIES even on a CPG gas** (~1.2% across the window, rising toward 1). The effect that **died on CPG** for the choked branch is **first-order and alive** on the subsonic branch — rung 31's "the turbine does not know the operating condition changed" holds **only while both throats choke**. **Framing correction** (advisor): the coupling is to the pressure **RATIO `π_c`** via `pt9/p0`, **NOT** ambient `p0` — the cycle is pressure-homogeneous (ratios `p0`-invariant to machine zero, gate 6). **Envelope — TWO boundaries**: bounded **ABOVE** by nozzle-unchoke and **BELOW** by **thrust-neutral idle** (as `Tt4` falls `π_c→1`, `(1+f)V9→V0`, net thrust→0 near `Tt4≈440`; below it the engine windmills — reported SUB-IDLE, not a drag point, and NOT left to trip the shared `_score` cascade). Window **widens at low ram** (CPG: unchokes at `Tt4≈820` at `M0≈0.10`) — the idle-descent regime. **Reduce**: the choked path is **left LITERALLY unchanged** (dispatch only fires when the rebuilt nozzle is subsonic) ⇒ all choked points **bit-for-bit rung 31** (the 31/32 suites pass unchanged, 14/14); at the boundary `M9→1` continuously. **Non-tautological gate 4** (the advisor's load-bearing catch — gate 1 is a *choked* point that returns before dispatch, so the subsonic solve has NO reduce anchor of its own): an **independent CPG closed-form solve of `(★★)`** (pure algebra — `τ_t=1−η_t(1−π_t^((γ−1)/γ))` → shaft → `π_c` → `M9` from `pt9/p0` → `MFP` closed forms → root-find `π_t`; **no `_sonic_throat`, no `Nozzle.apply`**) reproduces the shipped solver's `(π_t,π_c,τ_t,M9)` to **machine zero** (`Δπ_t=0`) — two paths, one point; catches a 1% `π_c` corruption that gates 1/2 miss. **Separate entry point** (default `run(…)` untouched ⇒ cycle **bit-for-bit rung 6**). Disclaimed: `η_c/η_t` held at design (inherited from rung 31); **subsonic + component map OUT OF SCOPE** (`MapMatcher` overrides `match`, stays choked-only); NGV choke assumed; thrust-neutral is the modeled lower bound (spool-down/windmilling dynamics a separate time-dependent seam). | `docs/rung33-spec.md` |

| 34 | **The spool transient — `N` becomes a STATE, not an output** — `SpoolTransient.equilibrium/integrate(…)` / `_instant(…)` / `_close_compressor(…)`: rungs 31–33 solved **steady** points, each closed by the shaft **power balance** `η_m·P_t=P_c`. Rung 34 unbalances it — a real spool has inertia `I`, so a fuel change drives a net torque and `N` accelerates. **The shaft balance becomes an ODE** (`I·ω·dω/dt=η_m·P_t−P_c`) and `N` — which rungs 31–33 *computed* — becomes the **STATE** carrying the engine's memory. **The first DYNAMIC rung** (all prior were fixed points). **The structural novelty: the compressor map runs FORWARD** (rungs 31–32 ran it backward): given corrected speed `n(N,Tt2)` and trial flow `m`, the Euler speed line gives `τ_c=1+(τ_c,d−1)ψ(φ)n²` **directly**, and — the key simplification — the **NGV choke closes `m` on EITHER branch with NO shaft balance** (`pt4=π_b·π_c·pt2` doesn't involve the turbine, so mass continuity `ṁ(1+f)=A4·pt4·MFP*/√Tt4` is one equation in `m`); the turbine expansion is then rung-31 geometry `(★)` when choked, nozzle-continuity when subsonic (rung 33 dispatch, reused). The leftover power drives `dν/ds=Φ(ν,Tt4)` in **nondimensional time** `s=t/τ_spool`, `τ_spool=I·ω_d²/P_ref`. **THE FINDING is a CORRECTION of the obvious framing** (advisor): "the trajectory shape is `I`-independent, `I` only sets the clock" is a **TAUTOLOGY** in a 1-state model (dimensional analysis — the project's rung-29-gate-2 / rung-33-gate-4 anti-tautology bar rejects it). `I` is load-bearing **only when a SECOND clock competes**: ramp `Tt4` over a finite `τ_fuel` and the peak **excursion above the running line** (toward lower surge margin) is `E(r)`, `r=τ_fuel/τ_spool` — **max at `r→0`** (the constant-`N` displacement, an **algebraic map property**, `+5.4%`), **vanishing as `r→∞`** (stays on the line), knee at `r≈1`. *That* is why real engines **schedule fuel ramps**, and it is the honest home for `I`. **The map needed a fix** (a genuine sub-finding): rung 32's loading law `ψ=1−σ(φ−1)²` **peaks** at design, giving the **wrong speed-line slope** on the surge side; rung 34 adds a **linear slope `l`** (`dψ/dφ|_1=−l`, default 0 ⇒ rung 32 **bit-for-bit**) so `π_c` **rises toward low flow** and the accel excursion is physical. Direction **shape-robust** (accel `+`/decel `−` across 3 surge maps; magnitude **disclaimed**, **no surge line drawn** — inherited rung-32 concession). **Spool-down/windmilling** (the rung-33 handshake): cut fuel ⇒ `dν/ds<0`, `N` coasts down, the nozzle **unchokes**, the branch flips **choked→subsonic** at `M9≈1` (continuous), and the trajectory approaches rung-33's **thrust-neutral idle** (a too-fast chop instead hits the **flameout boundary** — the integrator stops, the decel analogue of accel-toward-surge). **Reduce**: `dν/ds=0` ⇒ the equilibrium reproduces `OffDesignMatcher.match` (flat map, rung 31) / `MapMatcher.match` (shaped, rung 32) — via the **forward closure ONLY** (never calls the matchers ⇒ non-circular), machine-zero at design, ≤1e-8 on the sweep incl. a subsonic point; a genuinely different closure onto one point. **Separate entry point** (subclasses `MapMatcher`; default `run(…)` untouched ⇒ cycle **bit-for-bit rung 6**). Disclaimed: `I`+`ω_d` = **one disclaimed clock group** (only `ν(s)` and `r` claimed, wall-clock illustrative — the `L`/`τ_res` concession); quasi-steady components (no combustor volume-filling / heat-soak — faster clocks *below* `τ_spool`, a further seam); `Tt4(t)` control (a true `ṁ_fuel(t)` schedule with `Tt4` an output is a further seam); isentropic knobs / NGV-choke / single-spool (inherited rungs 31–33). | `docs/rung34-spec.md` |

**The invariant that spans rungs 7–30: they are all pure diagnostics** (rungs 31–34 are the
**STRUCTURAL rungs** — they compute a *new* off-design operating point: rung 32 with the component
map, rung 33 on the **subsonic-nozzle branch** below unchoke, rung 34 the **dynamic** point where
`N` is a *state* not an output — but through **separate entry points**
(`OffDesignMatcher`, `MapMatcher`, `SpoolTransient`) that leave the default path untouched). NO/N
never enter `_equil_solve`, the production nozzle stays frozen AND ideally-expanded
(`convergent=False`), and the default `build_turbojet(…).run(…)` design run is unchanged, so
**the cycle is bit-for-bit rung 6** — every rung above 6 only *reads* the run's design-point
state (rungs 31–34 match a new operating point *beside* it — rung 33 the subsonic-nozzle
branch). Each rung's
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

**Current scope (rung 34).** The **cycle solve** is a thermally-perfect, reacting,
dissociation-equilibrium gas (`Gas.reacting_equilibrium()`) through ideal + real
components (isentropic `η_c/η_t` **or** polytropic `e_c/e_t`, mutually exclusive;
`π_d/π_b/π_n`, `η_b`, `η_m`; dual cold/hot gas; specified exit pressure). The burner
root-finds `f` over the scale-B absolute balance, re-solving equilibrium each trial,
then freezes the station-4 mixture through turbine + nozzle. Fork A/B
(`Gas.reacting()` / `reacting_forkb()`) and frozen-products `Gas.thermally_perfect()`
are kept alongside. Everything from rung 7 up is a diagnostic *beside* the cycle —
including rung 30's **choked convergent nozzle** (`Nozzle(convergent=True)`), offered as
an alternative to the default ideally-expanded nozzle so the cycle stays rung-6 exact.
Rung 31's **off-design matching** (`OffDesignMatcher`) is the first STRUCTURAL rung — it
solves a *new* operating point (`π_c` becomes an OUTPUT) against the fixed rung-30 choked
hardware — but on a **separate entry point**, so the default design run is still rung-6 exact.
Rung 32's **component-map matching** (`MapMatcher` + `ComponentMap`) subclasses it: representative
`η_c/η_t` maps + speed lines droop `π_c`/`ṁ` off-design and attach the shaft speed `N`, while the
choke-pinned work `τ_c` stays map-free — the flat map reduces to rung 31 bit-for-bit, and the
default run is untouched. Rung 33's **subsonic-nozzle matching branch** (`OffDesignMatcher._match_subsonic`,
auto-dispatched from `.match`) solves the *second* matching mode below the nozzle-unchoke boundary:
only the NGV chokes, so `π_t` re-couples to `π_c` and the running line's `τ_t` **varies even on a CPG
gas** (the inversion of rung 31's CPG-constant `τ_t`) — the choked path is left literally unchanged, so
choked points stay bit-for-bit rung 31. Rung 34's **spool transient** (`SpoolTransient`) is the first
**DYNAMIC** rung: it makes `N` a *state* under the shaft-inertia ODE (`I·ω·dω/dt=η_m·P_t−P_c`), running
the compressor map **forward** + NGV-choke to close the flow with **no shaft balance**, and marching
`dν/ds` in nondimensional time. Its equilibrium reduces to the rung 31/32 running line via that forward
closure; the finding is the two-timescale ratio `τ_fuel/τ_spool` (not the tautological "`I`-independent
shape"), and it hands off to rung 33's subsonic branch on spool-down. Separate entry point; default run
still rung-6 exact.

## Deferred seams (kept open on purpose)
- **Finite-rate nozzle chemistry** — **BUILT BY RUNG 25** (`docs/rung25-spec.md`,
  `Gas.finite_rate_nozzle`). Rung 14 gave the frozen↔equilibrium *bracket*; rung 25 resolved
  the Damköhler flow *between* on the `dh=v·dp` spine — and **inverted** it into a THREE-state
  picture (the super-equilibrium frozen entry makes `Da→∞` land at an **irreversible-fast**
  ceiling *strictly below* the reversible bound).
- **Freeze-out** — **BUILT BY RUNG 26** (`docs/rung26-spec.md`, `Gas.freeze_out_nozzle`,
  `docs/plans/rung26-anchor-freeze-out.md`). Rung 25 named this seam and **mis-framed it** as needing
  "a `T`-dependent Arrhenius `τ_chem(T)` … the unanchored-Arrhenius trap." Rung 26 built it and
  **refuted the framing on both counts**: `τ_chem` is **anchored** to GRI-Mech 3.0's `H+OH+M` sink
  (`Ea=0`, `n=−2` — zero new constants), and freeze-out is **density-driven** (`c_tot²∝(p/T)²`)
  **against** an opposing `T` effect (`k` accelerates on cooling). The local `Da(T,p)=τ_res/τ_chem`
  shuts the relaxation off partway down the nozzle and the freeze point **MOVES with `Tt4`** (the
  finding); it adds no new bound (lands inside rung-25's `[F,I]`), reduces to rung-25 `_finite_rate_expand`
  bit-for-bit at constant `Da_local`. **What rung 26 leaves open:** (a) a **resolved `τ_res`** from the
  nozzle area-schedule (retire the last geometric knob `L`, *pin* the location) — **INVESTIGATED,
  NEGATIVE on BOTH counts, NOT shipped and NOT a rung** (`docs/tau-res-negative.md`). The conical
  `dt=dx/V` reshape does give a normalized shape `ĝ(s)∝|da/ds|/(√a·V)` in which `ṁ` and `tanθ`
  cancel — but the shape is **moot**: the march starts from *stagnation*, so `ĝ∝s^{-7/4}` and the
  normalization **does not converge** without an **entry Mach `M_e`**. So deriving the distribution
  **ADDS** a geometric knob rather than retiring `L`, and the answer is **more** sensitive to `M_e`
  than rung 26 ever was to `L` (at `Tt4=1500`, `s_freeze≈s_e` at both cutoffs — the location is
  *slaved to the cutoff*). **The positive by-product: rung 26 is CONFIRMED** (freeze motion exists
  and rises with `Tt4` under both cutoffs; lean still frozen-from-entry) with its **disclaimed**
  magnitude refined ~3× (span 0.378→≈0.12, grid-converged, ~11% cross-cutoff) — and the hot case,
  which the uniform cartoon pushed *past* its throat, now freezes upstream of it like the others.
  Do NOT re-run the reshape-on-the-`ln p`-frame construction; a new attempt needs a **real `A(x)`
  geometry** (physical entry plane + throat area, hence `ṁ` threaded in and the *choked* nozzle
  seam); (b) a **`T`-dependent
  freeze-out of exhaust NO** — **BUILT BY RUNG 27** (below); (c) a **shifting turbine** — **BUILT BY
  RUNG 29** (below).
- **NO freeze-out** — **BUILT BY RUNG 27** (`docs/rung27-spec.md`, `Gas.no_freeze_out_nozzle`,
  `docs/plans/rung27-anchor-no-freeze-out.md`). Rung 26 named this seam ("Zeldovich is slow, may freeze
  earlier than the recombination clock"). Rung 27 applied rung-26's anchored-clock/local-`Da` machinery
  to a `_tau_no_destroy` clock from **rung 7's OWN Zeldovich reverse rates** (zero new constants) and
  found the frozen-NO assumption every NO number has carried since rung 7 is **EARNED**: `Da_NO≪1` from
  entry at every `Tt4` (3–9 orders — frozen-from-entry *everywhere*, unlike rung-26's major pool), on an
  **upper bound**. The kill test **INVERTS rung 26's** (Arrhenius + bimolecular ⇒ the two terms AGREE,
  both DRIVE, vs rung-26's density-DESPITE-temperature). A **confirmation** that retires the rung-14/17
  clamp corollary's last premise; **no moving freeze point** (`s_freeze_NO≡0`), the honest trend is the
  separation narrowing with `Tt4`. Reduce: `Da_NO≡0` ⇒ the rung-14/17 clamp `max_a` bit-for-bit.
- **The rung-26-coupled NO march** — **BUILT BY RUNG 28** (`docs/rung28-spec.md`,
  `Gas.coupled_no_freeze_out_nozzle`, `docs/plans/rung28-anchor-coupled-no-march.md`). Rung 27 named this
  seam and predicted it "can **only** slow NO further … a secondary refinement". Rung 28 built it and
  **confirmed the verdict while correcting BOTH stated reasons**. (a) "Only" is wrong: rung-26
  recombination is **exothermic**, so coupling also **lifts `T`**, and this clock being **Arrhenius**
  that **SPEEDS** NO destruction — **two opposing channels**. The conclusion survives (`net<1` at every
  in-band `Tt4`) but for a different reason: depletion is **UNBOUNDED**, heat release **SATURATES**. The
  opposing channel is **not** negligible (cancels ~half the depletion hot; makes the net trend
  non-monotone). (b) The **β repair**: rung 27's "NO arrives super-equilibrium" premise is **false at the
  entry** (`a`=0.31–0.61) — exactly where freeze-from-entry is decided; the bound holds because **β<1**,
  which covers **both** regimes. Rung 27's *numbers* are untouched. **What rung 28 leaves open:** a
  **coupled formation clock** (the `a≫1` surrogate *bounds* the sub-eq entry but does not *describe* it;
  moot while `Da_NO≪1`).
- **The shifting turbine** — **BUILT BY RUNG 29** (`docs/rung29-spec.md`, `Gas.shifting_turbine`,
  `docs/plans/rung29-anchor-shifting-turbine.md`). Rungs 14/25/26 all named this seam and all predicted the
  same thing: a less-super-equilibrium entry *shrinks* rung-25's (R−I) gap and *moves* the freeze point.
  Rung 29 built it as a zero-knob **bracket** (the turbine analogue of rung 14) and the prediction is
  **technically right but the wrong headline**: `(R−I)→0` on a shifted entry is **STRUCTURAL, a tautology**
  (an entry pinned at equilibrium has nothing left to relax irreversibly), not a measurement. What the
  bracket actually establishes is that **freezing the turbine is EARNED at the design point** (`ΔT5/T5`
  =0.011%, an order below the cycle's own `η_t`/`π_b` precision) and **NOT hot** (1.86% at `Tt4`=2400) —
  **rate-independently**, since the bound is the instant-chemistry reversible limit. And the reason all
  three rungs expected a bigger effect is the rung: **RATIO ≠ ENERGY** — `x_frozen/x_eq` measures *kinetic*
  distance from equilibrium (correct, and what rungs 25–28 need for **rate** questions) but is **not** a
  proxy for exploitable **enthalpy**, which scales with the absolute radical **INVENTORY**; the two
  **anti-correlate** across the band (ratio ÷33, inventory ×121, shift ×174), so the ratio is loudest
  exactly where the shift is most negligible. **What rung 29 leaves open:** (a) ~~a **finite-rate turbine
  march**~~ — **CLOSED, NEGATIVE, NOT a rung** (`docs/turbine-march-negative.md`). Built (probe): march the
  Damköhler flow between `F` and `S` as rung 25 did for the nozzle. Negative for a reason **deeper than the
  un-anchored `τ_res`** — the rung-25 dodge (a rate-independent `Da→∞` ceiling *strictly below* the
  reversible bound) **cannot repeat**: the turbine entry (station 4, fresh burner exit) is **at equilibrium
  by construction**, so `Da→∞` stays on the equilibrium manifold and lands *exactly* on `S` — **`I_turb ≡ S`,
  no third state, no `(R−I)` gap** (`dS(I_turb)`=machine-zero vs the nozzle's `+4.3e-4→+4.1e-2`, exact closed
  forms). The march is a plain `F→S` interpolation whose only new DoF rides on **two** un-anchored knobs:
  `τ_res` (`Da_turb`=0.05–8.8, transitional, NOT fast despite the high `p`) **and** — worse than the nozzle —
  an **ambiguous progress coordinate** (work-limited ⇒ `p5` unknown ⇒ no natural schedule). By-products:
  rung 25's `(R−I)` is **manufactured by the freeze, not intrinsic to expansion** (one turbine→nozzle
  handoff); rung 29's `S` is **attainable** (the genuine `Da→∞` limit), not an unreachable ceiling; and
  since `F≈S` at design the whole `[F,S]` band is negligible **regardless of either knob**, so the march
  **cannot overturn "freeze earned at design."** A real rung needs turbine passage geometry (blade-row count
  + the choked-flow seam `docs/tau-res-negative.md` named) to anchor both knobs. **Do NOT re-run** the
  geometric-`p` scalar-`Da` march with a hand-picked `p_floor`. (b) ~~the **`π_c` axis**~~ — **CLOSED** (`docs/rung29-pi-c-margin.md`);
  ~~the **`M0` / flight axis**~~ — **CLOSED** (`docs/rung29-M0-margin.md`, below); both the
  CONFIRMATION+SHARPENING/CORRECTION checks, NOT rungs. (c) **feeding the shifted station 5 into the
  production cycle** — a **re-foundation** (it re-anchors every rung's numbers), **not a rung**.
- **"Earned at design" at higher/lower `π_c`** — **CHECKED, CONFIRMATION + SHARPENING, NOT a rung**
  (`docs/rung29-pi-c-margin.md`). Rung 29 shipped its verdict from one `π_c`. Re-measured over `π_c` 2→80:
  the verdict **holds everywhere** — the design-point bound never exceeds **0.0107%** (**9.4×** under the
  threshold) and the earned/not-earned boundary `Tt4*` stays above **1846 K** (never within 346 K of design).
  But **unlike rung 28's β the worry did NOT invert**: `π_c` is **weak, non-monotone and double-edged** —
  `Tt4*` is **bowl-shaped** with an **interior** worst case near `π_c`≈15, and the runnable ceiling rises
  faster than `Tt4*` moves, so the not-earned band **widens 2.7×**. Rung 29 did **not** sample a favourable
  `π_c`: its `π_c`=10 sits essentially **at** the design-point maximum (0.010668% vs 0.010672% at ≈10.5,
  resolved to 8 digits — the solver bisects to 1e-13, gate 2's 1e-6 is a loose assert, not its accuracy).
  **The substantive result is the SHARPENING of rung 29's own finding**: `RATIO ≠ ENERGY` replaced the
  super-eq ratio with the radical **inventory** — but the inventory is itself **incomplete**. Along `π_c`
  inventory **falls** 3.4× (pressure suppresses dissociation) while the shift **rises**: the *same* failure,
  committed by the replacement. The complete currency is **`ENERGY = INVENTORY × COMPLETION`** — the
  **recombined** inventory, completion climbing 36.5%→99.995% as a larger `delta_h` runs deeper and colder.
  The two channels are **comparable and opposed** on `π_c` (3.4× vs 2.7×) hence the interior turnover, and
  **the same** on `Tt4` where inventory swings two orders and dominates — which is why rung 29's `Tt4`-axis
  claim **stands untouched**. Disclaimed: the product law is quoted **only at the cool design point**
  (`x_O+x_H+x_OH` omits `CO→CO₂` — flat to ±4% at `Tt4`=1500, varies 2× at 2100); the `π_c`≈10.5 peak
  *location* is not claimed as physical; **`M0` now CHECKED below**.
- **"Earned at design" at higher/lower flight Mach `M0`** — **CHECKED, CONFIRMATION + CORRECTION, NOT a
  rung** (`docs/rung29-M0-margin.md`). Rung 29's LAST "one design point" concession (after `π_c`). Over
  `M0` 0.3→3.0 (fixed ambient, `π_c`=10): the verdict **holds with an 8.8× margin** (design bound ≤0.0113%),
  and — the clean **opposite** of `π_c` — the shift is **monotone-protective**, no turnover (the bracket's
  `β`-like axis), worst case low-`M0` **takeoff** not cruise. Same `INVENTORY × COMPLETION` currency, read
  where it is **LOPSIDED**: `M0` suppresses inventory ×4.7 (ram `pt4`) but completion is **near-saturated**
  (86→100%, ×1.16), so inventory dominates monotonically. **The result CORRECTS the `π_c` doc's unification**:
  the turnover discriminator is **NOT completion "headroom" but the `delta_h` SWING** that drives it — `π_c`
  swings `delta_h` ×11 (compressor `τ_c`, a work climb), `M0` only ×2–3 (a ram-temperature *datum* shift),
  so completion can never outpace inventory on `M0`. Proven by a **`π_c`=2 control**: with completion
  headroom restored (33→61%) the `M0` sweep is **still monotone**. The flight axis is **double-edged** in a
  way `π_c` is not: protective per point, yet ram heating lifts the burner-squeeze **floor** faster than the
  boundary, shrinking the earned **operating band ×2.1** while the not-earned band widens ×1.7 (earned
  fraction 69%→39%). Disclaimed: **fixed ambient** — this is the `M0` axis, **not a flight envelope** (real
  high-Mach flies thinner, lower `p0`); "supersonic cruise is safe" NOT claimed. `CO` caveat **worse** here
  (`pt4`≈17 MPa at `M0`=3 ⇒ +7.3% currency-law drift vs `π_c`'s ±4%). `η_t`=1 and no rate, unchanged.
- **β at higher `π_c` / hotter cycles** — **CHECKED, CONFIRMATION, NOT a rung** (`docs/rung28-beta-margin.md`).
  Rung 28 filed its `β<1` bound as its one factor-not-orders margin. Re-measured on the axis it named, and
  the worry **INVERTS**: β is **exactly pressure-invariant** (`c_tot²` cancels — every `R` is a product of
  *two* concentrations, so `β = k1f·x_O·x_N2/(x_NOe·(k2r·x_O+k3r·x_H))`, flat to 8 digits over 160× in `p`),
  so π_c has **no direct channel at all**, and both its indirect channels (lower `far`, lower `Tt9`) push β
  **DOWN** — `0.512→0.278` over π_c 10→80. **Higher π_c is PROTECTIVE**, and entry `Da_NO` falls with it too
  (rung 27 hardens on the same axis). The shipped `0.512→0.513` flatness is **NOT** a plateau (β climbs
  monotonically in T and crosses 1 near ~3200 K) — but the crossing is a **temperature**, sitting **1.6–1.9×
  above** the hottest reachable nozzle entry, and the cycle stops solving (`Tt4≥2450–2500`) long before.
  Whole-plane max **0.5444** at `Tt4=2300/π_c=8` — an **INTERIOR** max on a flat diagonal ridge (β **turns
  over** below `π_c≈8`, where the two channels compete with opposite signs), and slightly **above** the
  0.513 rung 28 quoted, so the correction is not purely favourable. Still **empirical, not a theorem**;
  what is now excluded is specifically the pressure route.
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
  **INVESTIGATED (locally-resolved SCALE) — negative, NOT shipped, NOT a rung**
  (`docs/mixing-scale-negative.md`; originally filed as "rung 25" while that slot was empty — the
  shipped rung 25 is the *unrelated* finite-rate nozzle). A
  locally-resolved SCALE (a penetration-growing plume `δ∝J^p`, `σ∝f(J)`, + rung-16's finite-`τ_res`
  dwell cap) **does** finally turn `⟨EI⟩(J)` off monotone — the first time in the project — and the
  field even carries a real over-penetration penalty (`g` is U-shaped in `J`). **But** the turn's
  location, depth, and even its *existence* ride on the **unanchored penetration exponent `p`**:
  a clean interior min only at the hand-picked `p=1/4`; at the more standard `p≈1/2` `⟨EI⟩` is
  monotone-down with no turn. So the SCALE alone still does **not** pin the emissions optimum — and
  the real missing piece is an **anchored `δ(J)` law**, not a missing penalty. Do NOT re-run the
  growing-σ-at-hand-picked-`p` construction; a new attempt is only worthwhile with an *anchored*
  exponent or a real transported/CFD cross-plane field.
  **INVESTIGATED (the anchored `δ(J)` law, via a jet-in-crossflow trajectory) — negative, NOT shipped,
  NOT a rung** (`docs/mixing-jicf-anchor-negative.md`). Anchor the penetration exponent with the published
  JICF trajectory `y/rd=A(x/rd)^m` (`m≈0.28` Pratte-Baines / `0.33` Hasselbrink-Mungal). **Two findings.**
  (1) **PENETRATION axis — CONFIRMS rung 22.** Any *bent*-trajectory exponent (`m>0`) **breaks** the
  measured Holdeman `(S/H)√J` collapse (the g-optimum drifts 27–30% per 2× geometry); only `δ∝rd` (the
  momentum-**depth** scaling, `m=0` — rung-22's own law) is consistent. **Deflated honestly:** `g` depends
  on `δ/H` **by construction**, so "collapse ⟺ `p=1/4`" is *algebra*, not data pulled out — the correct
  claim is a **ruling-out** of the bent forms, not "data anchors `m=0`" (and it is a *depth* scaling, not a
  near-field claim — the near field is `m≈1/2`). This defuses the SCALE-negative's `p≈1/2` pessimism, but
  only on the penetration axis. (2) **EMISSIONS axis — still NOT pinned.** Holding penetration at the
  collapse-consistent `p=1/4`, the emissions turn's *location* is penetration-anchored and grid-robustly
  **stable at `C≈3.12`** across the cap (×0.4–2) and `c_D` (×0.5–2), BUT its *existence* rides on a
  **SECOND** un-anchored mixing exponent — the **spread/entrainment** growth `p_σ` (`p_σ=0` flat, `0.25`
  turns, `0.5` erases it), which JICF **trajectory** scaling does not supply; and the **global** min sits at
  a max-segregation **endpoint** in 6/7 configs (rung-22's concession — the SCALE-negative's "clean U" only
  looked clean because its `J`-grid excluded that endpoint). **Anchoring penetration MOVES the free
  parameter from penetration to spread; it does not eliminate it.** So the seam needs **BOTH** exponents
  anchored (or the full CFD **pattern**). Do NOT re-run the JICF-penetration + growing-σ-at-hand-picked-`p_σ`
  construction; a new attempt needs an **anchored spread/entrainment law** (murkier than the trajectory for
  a *confined* jet) or a real transported/CFD cross-plane pattern.
- **A per-pocket clamp that fires AT THE BURNER** (`max_a>1` at station 4, not just in
  the rung-14/17 nozzle). The lever is a **slow-enough freeze on a cooling pocket**,
  *not* a hotter `Tt4` (which raises the terminal `[NO]_e` and *lowers* the ratio).
  Rungs 20 and 21 both confirm they are **not** this lever.
- **Detailed Fenimore** (`CH+N₂→HCN→…`) and **super-eq-O radical-decay history** — both
  need new species / a relaxing pocket, which a 0-D pool cannot derive (hence rung 19's
  imposed prompt magnitude and semi-empirical super-eq ratio).
- **The choked convergent nozzle** — **BUILT BY RUNG 30** (`docs/rung30-spec.md`,
  `Nozzle(convergent=True)`, `_sonic_throat`, `docs/plans/rung30-anchor-choked-nozzle.md`). Named as
  deferred since rung 2. The shipped nozzle expands fully to `p0` (`M9`=1.86 supersonic — silently a
  C-D nozzle); a fixed convergent nozzle **chokes** at `M9`=1 (`pt9/p0`=6.29 ≫ crit 1.85), underexpanded
  3.4×. **Full expansion is NOT earned at design** — specific thrust falls 6.6%, but the finding is that
  the **pressure term rescues 87%** of the momentum deficit (raw `V9` drop would imply 51%). A **diagnostic
  beside the cycle** (default off ⇒ rung-6 exact); reduce: subcritical convergent ⇒ full expansion
  bit-for-bit, sonic solver ⇒ CPG closed-form critical ratio to machine precision.
- **Off-design matching** — **BUILT BY RUNG 31** (`docs/rung31-spec.md`, `OffDesignMatcher`,
  `docs/plans/rung31-anchor-offdesign.md`). The analytic (Mattingly Ch-8 / *AEDsys*) performance-analysis
  route, done on the project's own TPG machinery (not CPG referencing — chosen so the reduce is bit-for-bit,
  the rung-29 delegation move): with the **choked turbine NGV + the rung-30 convergent nozzle** both choked,
  two mass-flow-parameter constraints (`ṁ√Tt/(A·pt)=MFP*`, from `_sonic_throat`) pin the turbine and the
  shaft balance **inverts** to hand back the compressor — `π_c` becomes an **OUTPUT** and the engine runs a
  unique **running line** set by the choked hardware, **without a compressor map** (the choked downstream
  hardware *is* the map), reducing to the design point when operated there. The **first STRUCTURAL** rung.
  **The finding: `τ_t` DRIFTS** — the textbook "choked turbine ⇒ `τ_t` exactly constant" is a **CPG**
  statement; on the real variable-`cp`/reacting gas the two sonic `MFP*` shift with `γ_t(T)` so `τ_t` drifts
  ~2.8%/2:1-throttle (CPG holds it to machine zero — gate 2 reproduces Mattingly's closed-form referencing).
  **Choked envelope**: throttling back **unchokes the nozzle near `Tt4`≈600** (`pt9/p0`<~1.85) — the pin is
  lost and the matcher **flags** it rather than lying; the **subsonic-nozzle matching mode** past unchoke is
  the natural extension (Mattingly's dual mode). **What rung 31 leaves open:** (a) ~~**component-map
  matching**~~ — **BUILT BY RUNG 32** (`docs/rung32-spec.md`, `MapMatcher` + `ComponentMap`); (b) ~~the
  **subsonic-nozzle branch** past unchoke~~ — **BUILT BY RUNG 33** (`docs/rung33-spec.md`, below); (c)
  ~~**feeding the matched operating point into a transient/spool-dynamics** model (`N` from `τ_c`,
  acceleration)~~ — **BUILT BY RUNG 34** (`docs/rung34-spec.md`, `SpoolTransient`, below).
  Afterburner is a further seam still.
- **Component-map matching** — **BUILT BY RUNG 32** (`docs/rung32-spec.md`, `MapMatcher` + `ComponentMap`,
  `docs/plans/rung32-anchor-component-maps.md`). Rung 31 named this seam ("earns the η curvature rung 31
  holds constant along the running line"). Rung 32 built it and turned it into a **cross-rung CORRECTION**:
  rung 31's "pumping characteristic WITHOUT a compressor map" **over-claimed by holding η at design**. The
  choke sets the **work `τ_c(Tt4)` map-free** (rung 31 survives), but a representative peaked compressor map
  droops `η_c` off-design ⇒ `π_c`/`ṁ` fall **below** rung 31's line (same sign, 3 shapes; magnitude
  disclaimed), and the **shaft speed `N`** — which rung 31 never computed — is attached from the compressor
  **speed lines** (the structural novelty: a map is indexed by corrected speed). **Sub-finding**: the turbine
  barely moves for a **structural** reason (single-spool `nu_t=N/√Tt4` pinned within ~1% ⇒ `|Δη_t|`~2e-5 even
  for a steep turbine map) — the compressor is where the map bites. **Representative-map closure** (rungs
  12–24 methodology, shapes disclosed / claims shape-robust / magnitudes disclaimed); **no surge line** ⇒ the
  CRS surge-margin payoff deliberately NOT made. Reduce: the **flat map** ⇒ `MapMatcher` == rung-31
  `OffDesignMatcher` **bit-for-bit** (machine-zero at design). **What rung 32 leaves open:** a **real
  hardware/CFD map** with a surge line (would earn the surge-margin claim); the transient/spool-dynamics seam
  (c above, now that `N` exists). (The subsonic-nozzle branch is **BUILT BY RUNG 33**, below — but subsonic
  + component map stays OUT OF SCOPE: `MapMatcher` overrides `match` and stays choked-only.)
- **The subsonic-nozzle matching branch** — **BUILT BY RUNG 33** (`docs/rung33-spec.md`,
  `OffDesignMatcher._match_subsonic`, `docs/plans/rung33-anchor-subsonic.md`). Rung 31 named this seam (it
  flagged the nozzle-unchoke boundary `Tt4≈600` and deferred "Mattingly's dual mode"). Rung 33 built it: below
  unchoke only the **NGV chokes**, so rung 31's two-choke pin `(★)` is void; the nozzle passes the
  compressible-flow `MFP(M9)` (`M9=M9(pt9/p0)`), `pt9/p0` moves with `π_c`, and `π_t` becomes the
  **equilibrating unknown** matching NGV-choked supply to subsonic-nozzle demand `(★★)`. **The rung is the
  INVERSION of rung 31**: the choked branch's `τ_t` coupling ran through the `γ_t(T)` curve (2nd order, died on
  CPG); the subsonic branch's runs through `π_c` (structural, 1st order) so `τ_t` **VARIES even on a CPG gas**
  — "the turbine does not know the operating condition changed" holds only while **both** throats choke.
  Framing (advisor): the coupling is to the RATIO `π_c` via `pt9/p0`, **not** ambient `p0` (pressure-homogeneous,
  gate 6). **Envelope — two boundaries**: nozzle-unchoke above, **thrust-neutral idle** below (`Tt4≈440`; below
  it the engine windmills → SUB-IDLE). Reduce: choked path left literally unchanged ⇒ choked points bit-for-bit
  rung 31 (31/32 suites pass unchanged); gate 4 (non-tautological): the matched subsonic point satisfies the
  textbook `MFP(M9)`/isentropic relations to <1e-9 on a CPG gas. **What rung 33 leaves open:** subsonic +
  component map (rung 32's `MapMatcher` stays choked-only); a subsonic-NGV mode (the NGV is assumed choked
  throughout); ~~spool-down/windmilling **transient dynamics** below thrust-neutral idle~~ — **BUILT BY
  RUNG 34** (`docs/rung34-spec.md`, `SpoolTransient`, below — the shaft-inertia march reaches thrust-neutral
  idle on the subsonic branch; a too-fast fuel chop hits the flameout boundary).
- **The spool transient** — **BUILT BY RUNG 34** (`docs/rung34-spec.md`, `SpoolTransient`,
  `docs/plans/rung34-anchor-spool-transient.md`). Rungs 31 and 33 both named this seam (the transient/spool
  dynamics that `N` makes possible). Rung 34 built it as the first **DYNAMIC** rung: `N` becomes a *state*
  under the shaft-inertia ODE, the compressor map runs **forward** + NGV-choke closes the flow with **no
  shaft balance**, and `dν/ds` marches in nondimensional time. The finding **corrects the obvious framing**
  (the advisor's catch): "shape is `I`-independent, `I` is only the clock" is a **tautology** in a 1-state
  model; the load-bearing result is the two-timescale ratio `r=τ_fuel/τ_spool` — the accel excursion above
  the running line is `E(r)`, max at `r→0` (an algebraic map property), vanishing as `r→∞` (why fuel ramps
  are scheduled). Reduce: the equilibrium reproduces rung 31 (flat map) / rung 32 (shaped) via the forward
  closure only (non-circular). **What rung 34 leaves open:** (a) **combustor volume-filling / heat-soak
  dynamics** — faster clocks *below* `τ_spool` (the shaft is the only dynamic element here); (b) a true
  **`ṁ_fuel(t)` fuel-metering schedule** with `Tt4` an output (rung 34 controls `Tt4(t)` directly); (c) a
  **surge line** (rung 32's standing concession — would turn the excursion into a surge-margin number);
  (d) **two-spool / multi-shaft** dynamics; (e) feeding the marched `N(t)` into the production cycle — a
  re-foundation, not a rung.

## Conventions
- **SI units throughout** (K, Pa, kg/s, m/s, J/kg). Convert kPa → Pa internally.
- The cycle runs in **total (stagnation)** quantities `Tt, pt`; convert to
  static only at the nozzle exit (station 9) for exhaust velocity.

## Layout
- `turbojet/gas.py` — the core. `FlowState`; dual-section `Gas` (cold/hot, `unified()`);
  the CPG closed-form / TPG NASA-integral property interface (hot methods carry `far`);
  the gas factories (`thermally_perfect` / `reacting` / `reacting_forkb` /
  `reacting_equilibrium`); the `_equil_solve` Newton solver + frozen `_EquilibriumSection`;
  and **every diagnostic** — `thermal_nox`, `zoned_nox`, `nozzle_flow`, `exhaust_no_clamp`,
  `finite_rate_nozzle`, `freeze_out_nozzle`, `no_freeze_out_nozzle`, `coupled_no_freeze_out_nozzle`,
  `shifting_turbine` —
  plus their configs (`JetMixing`,
  `Unmixedness`, `MixingPDF`, `QuenchPDF`, `PocketQuenchPDF`, `TransportedPDF`, `PromptNO`, `SpatialPDF`,
  `SpatialDwellPDF`, `SpatialLocalPDF`; the eight mixing closures are mutually exclusive — plus
  `FiniteRate`, the rung-25 nozzle knob, `FreezeOut`, the rung-26 freeze-out knob, `NOFreezeOut`,
  the rung-27 NO-freeze-out knob, and `CoupledNOFreezeOut`, the rung-28 coupled-march knob) and helpers
  (`_quench_no`, `_pdf_mean_ei`, `_pocket_quench_mean_ei`, `_spatial_segregation`,
  `_spatial_dwell_field`, `_spatial_local_field`, the rung-25 `_finite_rate_expand` /
  `_irreversible_fast_expand` / `_equilibrate_hp`, the rung-26 `_tau_chem_recomb` /
  `_freeze_out_expand` (which takes the rung-28 pure-observer `record=`), the rung-27 `_tau_no_destroy` /
  `_no_freeze_out_expand`, the rung-28 `_tau_no_exact` / `_frozen_no_trajectory` /
  `_coupled_no_march`, and the rung-29 `_work_limited_expand`, …). (Rung 30's choked nozzle lives on the
  `Nozzle` component, not here — its `_sonic_throat` helper is in `components.py`.)
- `turbojet/components.py` — `Inlet, Compressor, Burner, Turbine, Nozzle` as pure
  `apply(state, gas)` in `h`/`pr` form (+ loss params, `ram_recovery(M0)`, the polytropic
  knob; the Nozzle branches CPG/TPG — the velocity↔enthalpy trap — and carries rung 30's
  `convergent=True` choke mode via the module-level `_sonic_throat` M=1 solver; rung 31's
  `choked_mfp` — the `pt`-independent sonic mass-flow parameter `ṁ√Tt/(A·pt)` — lives here too,
  built on `_sonic_throat`). The `Burner` runs the implicit `f = g(f)` fixed point, or
  `_solve_equilibrium` for an equilibrium gas.
- `turbojet/engine.py` — chains the components, solves the `Δh` + `η_m` shaft balance,
  scores performance (`_score`, two thermal efficiencies + cascade check). Also home to rung
  31's **`OffDesignMatcher`** — captures the fixed throat areas `A4/A8` from a design run, then
  `match(flight, Tt4)` solves the off-design operating point (`π_c` is an OUTPUT) via the
  two-choke MFP match + the compressor inverse; a **separate entry point**, the design `run`
  is untouched. And rung 32's **`MapMatcher`** (subclasses `OffDesignMatcher`) + **`ComponentMap`**
  (the representative compressor/turbine map — an efficiency island, Euler-work speed lines, and a
  near-fixed turbine): `match(flight, Tt4, comp_map)` reads `η_c/η_t` off the map at the operating
  point (a **secant** on `η_c` for the positive feedback) and attaches the shaft speed `N` via the
  speed-line inversion `_operating_point`; the flat map reduces it to `OffDesignMatcher` bit-for-bit.
  `_solve_turbine` gained an optional `eta_t=` (default = design, so rung 31 is untouched) so the
  map can pass a per-trial turbine efficiency. And rung 33's **subsonic-nozzle branch**: `match`
  auto-dispatches to **`_match_subsonic`** when the choked-branch rebuild leaves the nozzle unchoked —
  a 1-D root-find (`_subsonic_operating`) on the turbine `π_t` matching NGV-choked supply to the
  subsonic-nozzle demand (`OffDesignResult.branch` = `"choked"`/`"subsonic"`); the choked path is left
  literally unchanged (dispatch only fires on unchoke), and `MapMatcher` — which overrides `match` —
  does NOT inherit it (subsonic + map out of scope). And rung 34's **`SpoolTransient`** (subclasses
  `MapMatcher`): the shaft-inertia transient — `_instant(ν,Tt4)` runs the compressor map FORWARD
  (`_tau_c_forward` = the exact inverse of `ComponentMap.solve_n`) + `_close_compressor` (NGV-choke
  closes the flow with no shaft balance) + the rung-33 choked/subsonic turbine dispatch, returning the
  power residual `Φ=dν/ds`; `equilibrium(…)` root-finds `Φ=0` (reduces to rung 31/32), `integrate(…)`
  RK4-marches `ν(s)`, and `ramp_excursion`/`constant_speed_excursion` compute the finding `E(r)`.
  `ComponentMap` gained a **linear loading slope `l`** (default 0 ⇒ rung 32 bit-for-bit) so the forward
  speed line has the physical surge-side slope; `SpoolTransient` overrides `_solve_turbine` with an
  Illinois version (same root, faster — a marched trajectory calls it thousands of times). The
  module-level `_illinois` is the shared fast bracketed root-finder.
- `main.py` — the design-point run: ideal-vs-real tables, the overlaid T–s diagram, and
  **one panel per rung** (each panel demonstrates that rung's load-bearing claim and
  states its honest scope).
- `tests/` — `test_stations.py` / `test_validation.py` (rung 1), `test_rung2.py`,
  `test_polytropic.py` (2b), `test_variable_cp.py` (3), `test_reacting.py` (4),
  `test_forkb.py` (5), then **`test_rungN.py` for N = 6…34**. Every rung file carries that
  rung's **reduce-to-prior** gate plus its load-bearing claims; the gates are named in the
  rung's spec. Rungs 16, 23 and 24 **deliberately assert no emissions global-min location**;
  rung 25 **reduces to rung-14 FROZEN but deliberately NOT to equilibrium** (the (R−I) gap is
  the finding); rung 26 **reduces to rung-25 `_finite_rate_expand` bit-for-bit at constant
  `Da_local`** and **deliberately asserts no freeze LOCATION** — only its *existence*, its *absence
  lean*, and its *motion with `Tt4`* (the moving freeze point is the finding); rung 27 **reduces to
  the rung-14/17 clamp `max_a` bit-for-bit at `Da_NO≡0`** and **deliberately asserts no moving freeze
  point** — only that NO is *frozen from entry at every `Tt4`* and the kill-test *inversion* of rung 26;
  rung 28 **reduces to rung 27 bit-for-bit at `couple=False`** and **deliberately asserts no net-turnaround
  location** (it rides on `L`) — only the *monotone* channel ratio, the *unbounded-vs-saturating*
  asymmetry, and `β<1` (whose ~0.51 hot margin it asserts as a disclosed weak point, not a comfort);
  rung 29 **reduces to `Turbine.apply` at `η_t=1` bit-for-bit (`==`) BY CONSTRUCTION** (the frozen branch
  *delegates* rather than re-solving) and carries a **second** gate that the independent work-limited
  solver reproduces that closed form — **without which the reduce gate is a tautology**; it
  **deliberately asserts no rate and no freeze location** (the turbine `τ_res` is un-anchored) — only the
  *rate-independent* bound, the earned/not-earned split across the band, and the **ratio-vs-inventory
  anti-correlation** (the rung). `(R−I)→0` is **NOT** gated: it is structural. `test_rung29.py` also
  carries the **`π_c`-margin** gates (`docs/rung29-pi-c-margin.md`): `π_c`-robustness of the design-point
  verdict + the `1800 < Tt4* < 2200` boundary bracket, the **two opposed channels** (inventory ↓,
  completion ↑), a **forbid** on the β-style "higher `π_c` is protective" reading, and the sharpening —
  **entry inventory alone FAILS on the `π_c` axis**. And the **`M0`-margin** gates
  (`docs/rung29-M0-margin.md`): `M0`-robustness of the verdict (8.8× margin), the **monotone-protective**
  differentiator (sign-flip of `π_c`, asserted hot at `Tt4`=2100), the **lopsided** channels (completion
  near-saturated), the **`delta_h`-swing-not-headroom** correction (the `π_c`=2 control stays monotone),
  and the **envelope band squeeze** (floor/`Tt4*`/ceiling all rise, earned operating band shrinks).
  Rung 30 **reduces two ways**: `convergent=False` (or a subcritical convergent PR) ⇒ the shipped
  ideally-expanded nozzle **bit-for-bit**, AND — the **non-tautological** gate — the `_sonic_throat`
  bisection reproduces the **CPG closed-form critical ratio** to 1e-9 on a *self-consistent* gas (the
  rounded-R trap is why it must be self-consistent). It **deliberately carries no throat area** (the
  fixed-`A9` off-design pin is rung 31's) — only the *choke verdict* (full expansion NOT earned at
  design), the *pressure-term-rescues-87%* finding, and the *cycle-untouched* invariant.
  Rung 31 **reduces BY CONSTRUCTION** (not `==` — the operating point is a root-find): matching at the
  design flight + `Tt4` returns `π_c`, all stations, `ṁ`, thrust to ≤1e-9, because A4/A8 are captured
  from that design run and the compressor inverse is the exact inverse of `Compressor.apply`. Its
  **non-tautological gate 2** is the CPG closed-form referencing (`τ_t`/`π_t` const to machine zero, the
  `Tt4/(τ_r T0)` slaving factor const, `π_c` closed form to 1e-14) — the sonic-throat matching *is*
  Mattingly Ch-8 on the gas he assumes. It **deliberately holds `η_c/η_t` at design** (the map curvature
  is rung 32) and **flags the nozzle-unchoke boundary** rather than quoting the invalid choked-branch match;
  the finding gated is the **`τ_t` drift** (reacting ≠ constant, CPG == constant).
  Rung 32 **reduces to rung 31 bit-for-bit** at the **FLAT map** (`ComponentMap.flat()`) — machine-zero at
  design, ≤1e-9 on the throttle sweep, on the **reacting** gas (the non-tautological gate: `MapMatcher` vs
  `OffDesignMatcher`, two code paths, one operating point). Its finding gates (on the fast `thermally_perfect`
  gas — gas-independent physics): `π_c`/`ṁ` **droop** below rung 31, **same sign across 3 map shapes**, gap
  growing with throttle (the **magnitude is DISCLAIMED**, only the sign/existence gated); `τ_c` **map-free**
  to ~1e-4 (the work is choke-pinned — isolates *what* the map moves); the turbine **pinned in corrected
  speed** (`nu_t` within ~1%, `|Δη_t| ≪ |Δη_c|` even for a steep turbine map); `N` attaches, is monotone, and
  its schedule is robust across the speed-line `σ` (bounded spread — genuinely `σ`-dependent, **not** the
  tautological `√(τ_c−1)`). It **deliberately makes no surge-margin claim** (no surge line modeled).
  Rung 33 **reduces by leaving the choked path LITERALLY unchanged** (dispatch only fires when the rebuilt
  nozzle is subsonic) ⇒ all choked points bit-for-bit rung 31 (the **rung-31/32 suites pass unchanged**, the
  bit-for-bit witness) + design reduces to `π_c`=10 branch=`"choked"`. Its non-tautological **gate 4**: on a
  self-consistent CPG gas an **independent closed-form solve of `(★★)`** (no `_sonic_throat`/`Nozzle`)
  reproduces the shipped solver's `(π_t,π_c,τ_t,M9)` to machine zero — the load-bearing anchor, since gate 1
  (a choked point) returns before the subsonic dispatch and never exercises `_match_subsonic` (a 1% `π_c`
  corruption is caught here, not by gates 1/2). Finding gates (fast **CPG** gas — gas-independent structural physics):
  the subsonic **`τ_t` VARIES** with throttle (>1e-3 spread, monotone) while the CHOKED `τ_t` on the SAME CPG
  gas is machine-constant (the **inversion** of rung 31 gate 2 asserted side-by-side); **boundary continuity**
  (`M9→1`, no `π_c`/`τ_t` jump across the switch); the **envelope** (monotone; SUB-IDLE raised below
  thrust-neutral idle); **homogeneity** (`π_c`/`τ_t`/`M9` `p0`-invariant to machine zero — the coupling is to
  `π_c`, not `p0`). It **deliberately asserts no subsonic map** (`MapMatcher` stays choked-only) and reports
  thrust-neutral idle as the modeled lower bound.
  Rung 34 **reduces** by the equilibrium (`Φ=0`) solve reproducing `OffDesignMatcher.match` (FLAT map, rung
  31) and `MapMatcher.match` (SHAPED map, rung 32) across a throttle sweep incl. a subsonic point — via the
  **forward closure only** (never calling those matchers ⇒ non-circular), machine-zero at design, ≤1e-8 on
  the sweep, on the fast gas + one reacting design point. Its finding gates (fast gas — gas-independent
  dynamics): the **excursion `E(r)` is MONOTONE-decreasing** in `r=τ_fuel/τ_spool` with the `r→0` limit
  equal to the **algebraic** constant-`N` displacement (the step excursion is a MAP property, the dynamical
  content the ratio); **direction shape-robust** (accel above / decel below the running line across 3 surge
  maps, magnitude **disclaimed**); the running line is a **stable attractor** (`Φ` decreasing through zero,
  an off-equilibrium `N` relaxes back); **`I` is only the clock** (the anti-tautology witness — `ν(s)` is
  `I`-free, physical time scales with `I`); the **forward/backward map inverse** to machine zero; the
  **spool-down** crosses `choked→subsonic` at `M9≈1` toward thrust-neutral idle. It **deliberately makes no
  surge-margin claim** (no surge line — inherited rung 32) and quotes `I`/`ω_d`/`τ_spool` only as one
  disclaimed clock group.
- `docs/rungN-spec.md` — the derivation, assumptions, concessions and gates for rung N.
  `docs/plans/rungN-anchor-*.md` — that rung's verified anchor data.

## Commands
- Run the model:  `python main.py`
- Run tests:      `pytest`  (or `python tests/test_rung2.py`, etc.)
- Install deps:   `pip install -r requirements.txt`  (matplotlib only)

## Stack
Python (standard library) + matplotlib for the plot. No other dependencies.
