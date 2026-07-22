# Turbojet Cycle Simulator

A station-by-station model of a single-spool turbojet (Brayton cycle). It takes
flight + design conditions and produces the gas state at every station, the
thrust, the efficiencies, and a T–s diagram.

**The deliverable is understanding, not the tool.** The code is the medium that
forces every thermodynamic assumption into the open. Optimize the work for
teaching, not for features or polish.

## The rungs

The model is built in cumulative **rungs** — each adds one physical effect and is
anchored to a published case. All rungs are live; the current scope is **rung 42**.

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

| 35 | **Fuel is the control — `Tt4` becomes an OUTPUT** — `SpoolTransient.equilibrium_fuel/integrate_fuel(…)` / `_close_compressor_fuel(…)` / `_tt4_from_f(…)`: rung 34 commanded `Tt4(t)` by fiat — its one filed concession. Rung 35 meters **FUEL** (`ṁ_fuel`) instead, and `Tt4` falls out of the burner balance against the airflow the spool can **currently** pump. **The make-or-break**: command the fuel *mass flow*, not the ratio `f` — if you command `f` then `Tt4=burner(Tt3,f)` and it's a re-labeling; the physics is `f=ṁ_fuel/ṁ_air` **spiking because `ṁ_air` LAGS**. The structural novelty is the burner running **FORWARD** (`_tt4_from_f`, the exact inverse of the shipped `f`-solve): the trial corrected flow `m` fixes `ṁ_air`, so `f` and `Tt4` are OUTPUTS, and the NGV-choke consistency `g(m)=0` closes it (Tt4 floating — no shaft balance, rung 34's move). **THE RUNG is a cross-rung CORRECTION of rung 34** (the rung-28/29/32 move): at a frozen spool a fuel step **starves the airflow** (the hot NGV passes less corrected mass as `Tt4` rises, `(1+f)` rises), so `Tt4` **OVERSHOOTS** its steady endpoint — a **turbine-inlet-temperature (TIT) excursion**, a *second* acceleration limit commanding `Tt4` structurally HID — **and that over-temperature amplifies the airflow deficit**, so it also **ENLARGES** rung 34's surge excursion. **The two acceleration limits (surge + TIT) are COUPLED, not independent**: `E_surge(fuel) > E_Tt4` at every `r=τ_fuel/τ_spool`, gap **MAX at `r→0`** (4.77% vs 5.39%→10.16%) and **VANISHING as `r→∞`** (0.11% at r=3) — rung 34 **under-counted** the surge excursion a fuel-metered engine sees. Sign **shape-robust** across 3 surge maps (magnitude **disclaimed**, rung-32 methodology). The **new axis** `E_temp` (TIT overshoot, monotone in `r`, `r→0`=algebraic map property) is on these maps **larger** than the surge excursion — the accel is TIT-limited before surge-limited (*why fuel schedules are temperature-limited too*). **Reduce — CONTROL-INVARIANCE (non-tautological)**: a steady point is the same however named, so commanding `ṁ_fuel=f_eq·ṁ_air,eq` of a Tt4-point reproduces it (`ν,π_c,τ_t,ṁ_air` machine-zero at design, `Tt4_out==Tt4`) via the **forward-burner closure** — a genuinely different code path than the pinned-`Tt4` one; plus **`r→∞` convergence** (the dynamical reduce), **Tt4-control UNTOUCHED** ⇒ rung 34 bit-for-bit, and the **instant-level inverse** (`Tt4(f)` inverts the burner `f`-solve — the fuel↔`Tt4` analogue of rung 34 gate 6). **Separate entry point** (subclasses `SpoolTransient`; default `run(…)` untouched ⇒ cycle **bit-for-bit rung 6**). Disclaimed: **reacting-gas fuel control deferred** (the forward burner is built for the non-equilibrium gas; the finding is gas-independent, the reacting reduce is the Tt4-control path); no surge line / no TIT-redline number (which limit binds first is map-dependent); `ṁ_fuel(t)` metering-unit schedule with both ends free is a further seam; combustor volume-filling / heat-soak / two-spool inherited from rung 34. | `docs/rung35-spec.md` |
| 36 | **The surge line — the excursion gets a boundary to be measured against** — surge methods on `SpoolTransient` (`surge_margin(…)` / `surge_margin_schedule(…)` / `acceleration_binding(…)` / `_pi_c_map(…)`) + `ComponentMap.with_phi_surge(…)`: rungs 32/34/35 all reported the transient excursion as a distance **above the running line** and all filed the same concession — **no surge line** (a representative efficiency island is not a stability boundary; any margin number rides on where you draw it). Rung 36 draws it and turns it into a **cross-rung CONFIRMATION + SHARPENING** (the rung-28 / rung-29-margin move): the excursion was **half a surge statement** — surge risk is the excursion measured against the **surge margin**, and the margin is **not constant along the running line**. **The honest problem, faced head-on**: the **zero-new-constant** hope (stall at the map's loading-law peak `φ=1−l/(2σ)`) is **DEAD** — it lands at `φ<0` for all three surge shapes (`ψ` is monotone-rising across `φ>0`, no in-range stall point to inherit), so a stall flow coefficient **`φ_surge` must be IMPOSED** as a disclosed constant — exactly the free parameter rung 32 objected to. **So the rung lives or dies on one question: does a shape-robust SIGN survive it?** It does, for a structural reason: `SM(Tt4)` = `π_c(φ_surge)/π_c(φ_op(Tt4))−1` on one speed line — the imposed floor sets the **level**, but the **trend** rides on the running-line flow coefficient **`φ_op(Tt4)`, which the choked hardware DETERMINES** (rung 31/32), not the floor. **THE RUNG (the headline): surge margin is THIN AT LOW POWER** (`φ_op` walks 1.00→0.81 down toward the fixed stall floor as throttled — **CRS Ch. 9**: the equilibrium running line approaches the surge line at low corrected speed), a genuinely new surge-margin object whose **sign is robust to the imposed floor**, across **3 shapes × imposed `φ_surge` × an `n`-slope on the floor** (incl. `k<0`); plus the **currency equivalence** below. **The COMPOUNDING — confirmation + sharpening, NOT relocation**: the rung-34 constant-speed excursion `E0` and `SM_N` share a **currency** (both `π_c` ratios at frozen `n0` to the same denominator), so a step reaches surge **iff `E0≥SM_N` ⟺ `φ_step≤φ_surge`** (airtight, gated). `E0` **rises** AND `SM_N` **falls** as start power drops (BOTH point low, **reinforcing**), so `E0/SM_N` rises **monotonically** toward the low-power end — the low-power burst is most surge-critical on **BOTH** axes. This **confirms and sharpens** rung 34's implicit worst case rather than moving it: rung 34's `E0` (`constant_speed_excursion`) is **already** largest for the low-power burst (`argmax` unchanged — **no relocation**; relocation would need the schedules to point *opposite* ways, which a steeply speed-dependent real surge line *could* do). The surge line's **unique** contribution is `SM_N` — new information varying independently of the ramp ratio `r`, turning "excursion above the running line" into "fraction of the stability margin consumed" (**not** a rescale of `E`). **The anti-overclaim discipline (rung 32's warning, ENFORCED)**: `E0` is floor-**independent**, so the **crossing** into surge slides with the disclaimed `φ_surge` (`Tt4_lo=700`: `E0=0.098` fixed, `SM_N=0.109`@`φ_s=0.55` **no surge** vs `0.073`@`0.65` **surge**) — the crossing is **deliberately NOT gated**; only the **monotone rise** of `E0/SM_N` and the **sign** of the `SM` schedule are load-bearing (magnitudes disclaimed, rungs 12–24/32 methodology). **Pure diagnostic** (like rungs 7–30): the surge line **never touches** the running line or the transient (`E`, `ν(s)` unchanged — it only *measures*), `_pi_c_map(n,φ_op)` reproduces the shipped `π_c` **bit-for-bit** (two paths, one `π_c`), and **surge off (`φ_surge=0`) ⇒ rung 34/35 bit-for-bit** (the field is read only by the surge methods; the rung 31–35 suites pass unchanged). Separate entry point; default `run(…)` untouched ⇒ cycle **bit-for-bit rung 6**. Disclaimed: every margin **magnitude** and the **crossing** (ride on `φ_surge`); constant `φ_surge` in `n` (sign robust to a mild slope); constant-speed is primary (the transient's currency; constant-flow only for sign-robustness); **no bleed valve / variable stator** (the devices that raise `φ_surge` at low speed — rung 36 exhibits the margin they protect); choked branch only (subsonic surge margin out of scope); isentropic/NGV-choke/single-spool inherited. | `docs/rung36-spec.md` |
| 37 | **The two internal clocks — volume-filling CONFIRMS, heat-soak CORRECTS** — `CombustorTransient(…)` (subclasses `SpoolTransient`): the plenum (`plenum_ratio`/`_compressor_from_backpressure`/`_plenum_state`/`equilibrium_plenum`/`plenum_frozen_peak`) + the metal (`soak_gain`,`soak_ratio`/`_close_compressor_fuel_soak`/`_instant_soak`/`equilibrium_soak`/`soak_excursion`). Rungs 34–36 made the **shaft the only dynamic state**; rung 34 bundled the omitted internal clocks into ONE concession ("no combustor volume-filling, no heat soak … faster clocks below `τ_spool`, they do not change the `r` framing"). Rung 37 tests both claims and they **SPLIT** — the two clocks fall on opposite sides of `τ_spool`, so **one CONFIRMS and one CORRECTS** (the rung-28/29/32 cross-rung move, done twice in one contrast). **VOLUME-FILLING** (a combustor plenum, `τ_fill≈ms ≪ τ_spool`) **CONFIRMS**: at `r→0` the plenum fills to its full quasi-steady `pt4` **before `ν` can move**, so the peak surge excursion lands on rung-35's `E0` **to machine zero, INDEPENDENT of the fill clock `r_v`** — a genuine fast clock, the peak unmoved. Its content is **STRUCTURAL, not the clock** (the anti-tautology hook — "fast relaxes fast" is vacuous): it is the **FIRST rung where compressor mass flow ≠ NGV mass flow** (`ṁ_c≠ṁ_NGV` by **~22%** during the fill — the plenum stores the difference; rung 34 tied them rigidly via `pt4=π_b·π_c·pt2`). The one genuinely-new closure is the compressor run from **BACK-PRESSURE** (invert `π_c(m)` for `m` on the STABLE branch `φ≥φ_floor`, past the `η`-island peak — a THIRD use of the map, neither forward-rung-34 nor inverted-for-`n`-rung-32). **HEAT-SOAK** (a metal state `Tm`, `τ_soak≈s ~ τ_spool`) **CORRECTS**: `τ_soak` is NOT a fast clock, so a **second STATE carries thermal memory** ⇒ `E = E(r, θ₀)` — history-dependent, **NOT a function of `r` alone** (rung 34's blanket claim refuted). Surge is **PROTECTED** (**`cold < hot-reslam < adiabatic`** across `G∈{0.05..0.2} × r_m∈{1..10} ×` 3 shapes — the cold metal depresses `Tt4,turb` → colder NGV passes MORE corrected flow → higher `φ` → AWAY from surge; **channel a beats** the rung-36 counter-channel "slower spool parks at thin low-`ν` margin", which does NOT flip the early peak): rung-34/35's **adiabatic combustor is the conservative WORST case** (a CONFIRMATION of the bound with the mechanism named). The **cost** is the **accel-time LAG** (the primary CRS/Walsh-Fletcher effect — cold accel ~**2.5–3× slower**, `→∞` as `τ_soak` grows) and the **HISTORY-dependence** (a hot reslam is the *least-protected* case — just below the no-soak ceiling — never a hazard ABOVE it). **Honest scope (advisor):** the modeled combustor gas-path sink only ever *helps* surge, so it does **NOT** reproduce the operational **bodie/reslam** surge hazard, which is the **OPPOSITE sign** (heat soakage moving the working line *toward* surge) and lives in an **unmodeled compressor-side channel** (tip-clearance / compressor soak) — the overlap with the real bodie is the *history-dependence*, not the sign. Heat-soak equilibrium **== rung 35** because `Tm=Tt4,burner ⇒ Q=0` at the fixed point (**transient-only — never moves the running line**). Modeled **SEPARATELY** (each with the other off — the contrast IS the rung; the combined 3-state is a further seam). **Reduce — EXACT DISPATCH, not a stiff limit** (the advisor's requirement): `plenum_ratio=0`/`soak_gain=0` never build the extra state ⇒ the inherited `equilibrium_fuel`/`integrate_fuel` are **literally rung 34/35** (the rung 31–36 suites pass **unchanged**); the two equilibria reproduce rung 35 via the **independent** back-pressure / `Q=0` closures (`|Δπ_c|/π_c ≤ 1.5e-11`, mass balance `≤2e-14`). Separate entry point; default `run(…)` untouched ⇒ cycle **bit-for-bit rung 6**. Disclaimed: `r_v`,`r_m`,`G` are disclaimed clocks/gain (`I`/`L`/`τ_res`, twice) — only the peak=`E0` identity, the `ṁ_c≠ṁ_NGV` existence, and the two SIGNS (`cold<hot<adiabatic`, accel-lag) are load-bearing; mass-storage-only plenum (`Tt4` quasi-steady in the volume), constant lumped `hA`, no combined 3-state, no tip-clearance / two-spool (further seams); fuel control / NGV-choke / isentropic inherited. | `docs/rung37-spec.md` |

| 38 | **Two-spool matching — the triangular cascade (no simultaneous solve)** — `build_two_spool_turbojet(…)` / `TwoSpoolEngine` / `TwoSpoolMatcher` (a separate class, NOT a subclass of `OffDesignMatcher`): the first **two-shaft** rung — a plain (no-bypass) turbojet, LPC+LPT on one shaft, HPC+HPT on the other, station layout `0→2→25→3→4→45→5→9`. Rungs 31–37 are all single-spool; this rung adds a **THIRD choked throat** that does not exist in any of them — the LP-turbine NGV / inter-turbine duct (station `45`, area `A45`), between the HP turbine's exit and the LP turbine's inlet. With all three throats (`A4`,`A45`,`A8`) choked, rung 31's `(★)` mass-flow-compatibility trick applies **TWICE, chained**: `τ_HPT` is pinned by `(A4,A45)` alone, `τ_LPT` by `(A45,A8)` alone — both **independent of either compressor**, giving `Tt4→Tt45→Tt5` purely from geometry. **THE FINDING (self-corrected mid-rung — an initial over-claim caught and fixed, not just disclosed):** the first framing — "the LP spool solves independent of the HP spool" — is **WRONG**: `η_HPT` (an HP-turbine parameter) demonstrably moves `π_LPC` too, because it shapes the shared `Tt45` that BOTH shaft balances read. What survives, precise and airtight: **each compressor's OWN isentropic efficiency is a terminal leaf** — `η_LPC` enters only the last algebraic step of the LP shaft balance (converting an already energy-fixed `ΔT` into a pressure ratio) and **cannot reach `π_HPC`**; symmetrically `η_HPC` cannot reach `π_LPC`. So the two compressor PRESSURE ratios are **never bound by a joint (2×2) solve** — `π_LPC` solves in full (Steps 1–3: `Tt4→(★-HP)→Tt45→(★-LP)→Tt5→`LP balance) strictly *before* `π_HPC` even begins (Step 4, onto `π_LPC`'s `Tt25` output) — while every turbine/geometry parameter (`A4,A45,A8,π_n,η_HPT,η_LPT`) legitimately reaches BOTH ratios. Verified directly on the exposed `_cascade` method, at a FIXED `(Tt2,Tt4,f)` so the outer `f`-loop's own (separately disclosed) cross-talk cannot confound the reading. **Framed as a NO-COMPRESSOR-MAP model artifact** (the rung-31-before-rung-32 shape, explicitly), not a physical law — a real compressor MAP (flow capacity = f(speed, pressure ratio)) would very likely reintroduce genuine 2×2 coupling, exactly as rung 32 corrected rung 31's own over-claim; "two-spool + maps" is the deferred seam this rung names as its own likely correction. **Scope: the fully-choked branch only** — nozzle-unchoke is flagged (`AssertionError`, "OUT OF SCOPE"), not solved: it would relocate rung 33's inversion one throat upstream onto the LP spool, a genuinely different solve, deliberately deferred to a rung-33-shaped follow-on. **Reduce — EXACT DISPATCH** (the rung-37 pattern, not a knob-to-zero): `lp_disabled=True` never builds an LPC/LPT/`A45` at all — `TwoSpoolMatcher.__init__` constructs a bare `OffDesignMatcher` around the supplied single-spool design and forwards every `.match()` call to it verbatim (bit-for-bit `==`, not a converged limit; a knob-based `π_LPC=1` degenerate would NOT reduce exactly, since `A45`'s geometric pin would generically not land on `τ_LPT=1`). **Non-tautological gate**: an independent bare-math CPG cascade (no `Gas`/`Component`/`TwoSpoolMatcher` calls, its own bisection) reproduces the shipped solver's `(π_LPC,π_HPC,τ_HPT,τ_LPT)` to machine zero across a throttle sweep — the only anchor tying the cascade's numbers down, since the `lp_disabled` reduce never enters the two-spool code path at all. Disclaimed: both NGVs **assumed** choked (rung-31 parity, not derived station-by-station); no compressor/turbine maps; one `η_m` for both shafts; no bypass/bleed/interstage-duct-loss/reheat; **steady only** — the two-shaft transient (two inertias, the natural two-spool rung-34 analogue) needs this matcher first and is deferred. Separate entry point; default `run(…)` untouched ⇒ cycle **bit-for-bit rung 6**. | `docs/rung38-spec.md` |

| 39 | **Two-spool + component maps — the cascade acquires a DIRECTION** — `TwoSpoolMapMatcher` (subclasses rung 38's `TwoSpoolMatcher`) / `_cascade_map(…)` / `_hp_eta_loop(…)` / `_lp_eta_loop(…)`: rung 38 closed by **predicting its own successor would break it** ("a real map … would very likely reintroduce the coupling … the two spools DO need a joint solve"). **The prediction is WRONG, and how it is wrong is the rung** — the **rung-28 shape** (verdict confirmed, stated reason refuted). **THE ALGEBRA**: refer the HPT-NGV choke to the HP compressor face; since `pt4 = π_b·π_HPC·π_LPC·pt2` and `pt25 = π_LPC·pt2`, the ratio `pt4/pt25 = π_b·π_HPC` and **`π_LPC` CANCELS** ⇒ `ṁ_corr,25 = A4·π_b·π_HPC·MFP*(Tt4,f)·√(Tt25/Tt4)/(1+f)` **(†)** — the LPC raises pressure and mass flow **proportionally**, so the HP core sees the same **corrected** flow whatever the LP spool delivers, and no modeled loss between 25 and 4 reintroduces it. `Tt25`/`Tt3` come from rung 38's **energy** cascade (which reads no compressor efficiency), so the HP compressor's whole map coordinate pair is a **closed fixed point in `π_HPC` alone** and cannot see `η_LPC`. The LP face does **not** cancel — `ṁ_corr,2 = A4·π_b·π_HPC·π_LPC·MFP*·√(Tt2/Tt4)/(1+f)` **(‡)** carries `π_HPC`. **THE FINDING: the map opens EXACTLY ONE arrow (HP→LP)** — `η_HPC → π_LPC` is real (**−1.5e-4 … −6.7e-4**, negative at every shape × throttle) while `η_LPC → π_HPC` is **EXACTLY ZERO (bit-for-bit)**, gas-independent (holds on the reacting gas too). So rung 38's **verdict SURVIVES** (the two compressor pressure ratios are still **never** a joint 2×2 solve — the solve stays strictly triangular); it merely **acquires an order** (HP first, LP onto it), the *opposite* of the order rung 38's energy cascade runs in. **Not a tautology**: the gated claim is the **ASYMMETRY** (one specified arrow opens with a measured sign, the other stays shut for a provable algebraic reason), and the closed leaf is a **code-level guarantee** — the solve is built triangular (`_hp_eta_loop` writes **(†)** in closed form and runs *before* `_lp_eta_loop`) precisely so it is exact; a jointly-iterated implementation leaves ~1e-15 residue and could not make the claim at all (measured at the probe stage). The **one channel that DOES re-open it** is a representative **turbine** map (`η_LPC→φ_L→n_L→ν_LPT→η_LPT→Tt5→Tt25→π_HPC`) at **8e-7…2.3e-6** — **119×–548× weaker**, which is **rung 32's own sub-finding** ("the turbine is pinned in corrected speed") transplanted onto the LP spool; sign + order-of-magnitude gated, **ratio disclaimed**. **THE STRUCTURAL NOVELTY: TWO shaft speeds** (rung 32 attached one `N`; rung 38 computes **none**), hence the **SLIP `N_L/N_H`** — the natural two-spool diagnostic. **B1 (structural)**: with both NGVs choked both shaft works are `η_m·(1+f)·cp_t·Tt4·[pure geometry]`, so **`(1+f)` AND `Tt4` both cancel** in `N_L/N_H` ⇒ on a CPG gas with flat maps **slip ≡ 1 EXACTLY at every throttle**, verified `f`-independently (forced `f` at 0.5×/1×/2×/4×). **B2**: exactly two channels break it, separated by the **rung-31-gate-5 mirror** — the `cp(T)` gas curve (**1.5%** at `Tt4`=900, on the *same* flat maps) and the **map** (**5.0%** on the *same* CPG gas), the map **~3.4× the larger** but **not the sole** one (an initial "100% map content" reading was refuted by this table). This **INVERTS rung 32's decomposition — on the CPG gas, and only there**: there the work was choke-pinned and map-free and the map only **re-labelled** it; on CPG the slip-deviation **does not exist without the map** (identically zero on the flat map), so the map is the **sole** channel and genuinely *creates* it. **NOT unconditional**: on the reacting gas the same flat maps already give 0.9835 at `Tt4`=900, so on the real gas the map is the **dominant** channel, not the only one. **B3 (empirical, sign only)**: `N_L/N_H` falls **monotonically** with throttle across all shape pairs (5.1%–7.5% at `Tt4`=900) — **the LP spool falls away from the HP spool**, the textbook twin-spool behaviour (idle runs high `N_H`, much lower `N_L`); **NOT structural** (no cancellation guarantees the sign — it rides on relative map droop), magnitude **disclaimed**. **Reduce — the full LADDER**: FLAT maps ⇒ rung 38 `TwoSpoolMatcher` **bit-for-bit** (`==` on the reacting gas — targeted, not promised, and it landed); `lp_disabled=True` **exact dispatch** ⇒ rung 32 `MapMatcher` (shaped) and rung 31 `OffDesignMatcher` (flat), both bit-for-bit. **Non-tautological gate**: an INDEPENDENT bare-math CPG two-spool **map** cascade (no `Gas`/`Component`/`ComponentMap`/`TwoSpoolMapMatcher`; own bisections, own speed-line inversions, efficiency fixed points by **damped substitution** not the shipped secant) reproduces `(π_LPC, π_HPC, η_LPC, η_HPC, n_L, n_H)` across a throttle sweep — the only anchor tying the *map* cascade down, since the flat reduce holds every `η` at design and the `lp_disabled` reduce never enters the two-spool path. Separate entry point; default `run(…)` untouched ⇒ cycle **bit-for-bit rung 6**. Disclaimed: representative maps (every magnitude rides on the shapes — rung-32 methodology); the slip **direction** is empirical, not structural; the back-arrow **ratio**; fully-choked branch / both NGVs choked / steady only / one `η_m` / no bypass / **no surge line on either spool** (inherited or deferred). | `docs/rung39-spec.md` |

| 40 | **The two-shaft transient — the LP map opens a COMPLEX mode** — `TwoSpoolTransient` (subclasses rung 39's `TwoSpoolMapMatcher`) / `_close(…)` / `_instant(…)` / `equilibrium(…)` / `lead_threshold(…)` / `oscillatory_band(…)`: rung 39 named this seam and called it **newly well-posed** ("rung 38 could supply no `N` at all; rung 39 supplies two"). BOTH shaft speeds become **STATES** under two inertia ODEs; nondimensionalizing on the HP clock `τ_H` leaves exactly **ONE** parameter, the clock **RATIO** `ρ=τ_L/τ_H`. That is the **resolution of rung 34's own tautology** — rung 34 had to *impose* a second clock (`τ_fuel`) before inertia became load-bearing; a two-shaft engine has it built in (**each spool is the other's clock**). The closure is **rung 34's move on two shafts**: a **1-D root in `m_L`** (LPC map forward → `Tt25` → `n_H` → the corrected-flow transfer `m_H` → HPC map forward → `pt4` → `f` → the HPT-NGV choke imposes `ṁ` back), with **NO shaft balance** — so both power residuals are OUTPUTS. Rung 39's triangular η-cascade/one-way-arrow **does not arise** (the transient reads η **forward** off each map; that apparatus was a *steady*-η-fixed-point artifact). **HONEST ACCOUNTING — much of this rung is INHERITED and the spec says so**: the turbine work split `Pt_L/Pt_H` is `Tt4`-invariant to **4.7e-15** (rung 39 B1's `(1+f)`/`Tt4` cancellation); the lead threshold `σ_crit=[(∂Φ_L/∂Tt4)/ν_L]/[(∂Φ_H/∂Tt4)/ν_H]` (**HP leads ⟺ `ρ>σ_crit`**) **≡1 on flat maps + CPG** because on the running line it *reduces to the steady slip*, which B1 pins at 1 — a **derived inheritance** and this rung's **reduce SPINE, not its finding**; and the two channels that break it (`cp(T)` curve **+4.3e-2**, map **+2.5e-1**, map ~5.8× dominant **but not sole**) are rung 39 B2's shape. A **refuted hypothesis kept visible**: "the map favours the LP spool" is **FALSE** — `lp-only` shaping gives `σ_crit`<1 (0.73–0.95), `hp-only` >1 (1.22–1.28); both signs reachable, so only the *existence* of a shift is claimed. **THE FINDING (new, two-spool-specific) — `ρ`'s power SPLITS.** Write `J(ρ)=[[a/ρ, b/ρ],[c,d]]`: **STABILITY is `ρ`-FREE** — `tr<0` and `det>0` hold for **every** `ρ>0` as soon as `a<0, d<0, ad>bc`, three conditions **containing no `ρ`** (the signs are **MEASURED** — 252 `(shape,Tt4,ρ,gas)` points, 7 shapes × 3 throttles × `ρ`∈[0.05,100] × 2 gases, **zero violations**, worst eigenvalue **−0.011**; the `ρ`-freeness is algebra *on top*, so the composite is **not** billed as "provable"). But **OSCILLATION is NOT**: `disc=(a/ρ−d)²+4bc/ρ` kills its first term at `ρ=a/d`, so **`bc<0` ⇒ a COMPLEX inter-spool mode exists** in a band around `a/d`, and `bc≥0` ⇒ monotone at every `ρ`. **THE MECHANISM: `bc<0` iff the LP compressor map is SHAPED** — a shaped LP map flips `b=∂Φ_L/∂ν_H` from small-negative to large-positive (with `c<0` always), and **`hp-only` is the DISCRIMINATOR** (HP shaped, LP **flat** ⇒ `bc=+3e-4`, **no band**), proving it is the **LP map specifically**, not shaping in general. The mode is **MAP-CREATED** — rung 39's slip pattern a **third** time. Verified on the solver: `flow/press` at `Tt4`=1200 predicts `ρ∈[1.233,2.082]` (centre 1.602) and the Jacobian is complex inside / real outside. **Magnitude DISCLAIMED**: `|Im/Re|_max=√(−bc/(ad))` ≤ **0.25** in the sampled maps (no visible ringing) — **reported, not gated**, per rung-32/36/39 methodology; "it does not ring on *these* maps" is **not** "hunting is impossible", and the rung deliberately does **not** make "treating the shafts as independent is EARNED" its headline. **Scope: INTER-SPOOL** (rung 37's shaft+metal Jacobian is not audited, so "first oscillatory mode in the project" is NOT claimed). **A NEGATIVE stated plainly**: `σ_crit`'s authority is **FIRST-INSTANT only** — two dynamic claims were probed and **withdrawn**, "σ_crit predicts the marched crossover" (**tautological**: from `Φ=0`, `Φ(Tt4+dT)≈dT·∂Φ/∂Tt4`, so the condition collapses to `ρ=σ_crit` *by definition*) and "σ_crit is the amplitude→0 limit of the marched `ρ*`" (**refuted**: `ρ*/σ_crit`→0.60 / 1.40, not 1) — because the running-line-referenced ramp excursion is **SCHEDULE-SLAVED** (dominated by `slip_ss(Tt4)` moving while the speeds lag; negative at the first step for *every* `ρ`). **Reduce**: the **2-D** equilibrium (`Φ_L=Φ_H=0`, damped Newton from the design start — rung 34's was a 1-D bracket) reproduces rung 39's `match` to **≤1e-12** on CPG **and** reacting, via the **forward closure only** (never calling the matcher ⇒ non-circular); `lp_disabled=True` **exact dispatch** ⇒ rung 34 `SpoolTransient` **bit-for-bit** (`==`); rung 39's `match`/`_cascade_map` left **literally unchanged** ⇒ the rung-39 suite still witnesses them bit-for-bit. **Non-tautological gate**: an INDEPENDENT bare-math CPG two-shaft closure (no `Gas`/`Component`/`ComponentMap`/`TwoSpoolTransient`; own CPG thermodynamics, own bisections, own forward speed lines, own 2-D Newton) reproduces `(ν_L,ν_H,π_LPC,π_HPC)` **and `σ_crit` ON SHAPED MAPS** (~1.2) — the shaped value is what ties the object down, since reproducing the ≡1 identity would only re-check the reduce. Separate entry point; default `run(…)` untouched ⇒ cycle **bit-for-bit rung 6**. Disclaimed: `ρ` is a **disclaimed clock group, DOUBLED** (`I_L,I_H,ω_L,d,ω_H,d` unmodelled, no wall-clock time); every magnitude rides on the representative maps (band location, `|Im/Re|`, the `σ_crit` shift); fully-choked branch / both NGVs choked / one `η_m` / no bypass / `Tt4` control (not rung-35 fuel) / **no surge line on either spool** — all inherited. | `docs/rung40-spec.md` |

| 41 | **The two-spool surge line — the exposure SPLITS between the spools** — surge methods on `TwoSpoolMapMatcher` (`surge_margin(…)` / `surge_margin_schedule(…)` / `running_line_map(…)` / `flow_coefficient_turn(…)` / `_pi_c_spool(…)` / `critical_flow_turn_pi(…)`) + `SpoolTransient.surge_margin_channels(…)`: rungs 39 **and** 40 both closed by naming this seam in nearly the same words ("rung 36's machinery is single-spool — and now there are **two** compressors"). Rung 41 draws rung 36's line on both. **THE FINDING: the two-spool running line does not HALVE the low-power surge problem — it CONCENTRATES it on the LP compressor.** Over a 2:1 throttle `φ_L` falls **~29%** while `φ_H` falls **~7%** and is *bounded* (it **turns back up**); the excursion ratio is **3.8×–4.3×** across four shape pairs. **THE CAUSE is rung 39's `(†)` cancellation, and it is CLOSED FORM, not a sign**: the HP face sees only its **own** pressure ratio (`pt4/pt25=π_b·π_HPC`), the LP face carries the **PRODUCT** `(‡)`, so writing `φ ∝ Π_face/x_face` with `π=[1+η_c(τ_c−1)]^k` gives `s_H=dlnφ_H/dlnx_H = k(1−π_HPC^(−1/k))−1` — containing **NO LP quantity** — and `s_L = k(1−π_LPC^(−1/k)) + k(1−π_HPC^(−1/k))/τ_LPC − 1`. Both land within **0.013** of the measured sensitivity; **dropping `π_HPC` from `s_L` misses by 0.81–1.00** (60–100× worse) **and gets the SIGN wrong** — the shielding certified quantitatively, not as an observed ordering. **THE COROLLARY — a LIVE zero-new-constant anchor** (rung 36's was **DEAD**: its loading-law peak landed at `φ<0`): `s_H=0` ⇒ **`1+η_c(τ_c−1)=γ_c` ⟺ `π_c*=γ_c^(γ_c/(γ_c−1))`** = 3.2467 at `γ_c`=1.4 — `η_c`, the shaft constant, `cp_t/cp_c`, `τ_HPT` and the design split **all drop out**; **`γ_c` ALONE**. Verified invariant to `η_HPC` (0.80/0.95), `η_HPT`, `η_LPC`, `γ_t`, `cp_t` (the last two **bit-identical** — hot-section knobs cannot enter a cold-section form), three design splits and two flight conditions, **while `Tt4*` moves 666→1171 K (1.76×) and `π*` moves only 1.5%** (3.286→3.337, all of it the fuel fraction): *the closest approach is at a **pressure ratio**, not a throttle setting*. **KILL TEST**: the whole **+0.44%** residual is the **fuel fraction** (`f` enters both `K` and the choked flow ⇒ `(★)` is exact with `f` frozen) — raising `hPR` ×1000 drives `f`→1e-5 and the residual **monotonically to +0.000%**, linearly in `f`. Three regimes: design `π_face` **below** `π*` ⇒ the face walks **AWAY** from surge when throttled (verified at a 6×3 split, `φ_H` 1.000→1.157); **above** ⇒ walks in, bottoms at `π*`, walks back out **if** the choked envelope reaches (at 1.5×12 or `M0`=0.40 it rails out — `flow_coefficient_turn` returns `RAIL` rather than inventing a minimum). **WHAT `(★)` IS NOT — and the payoff of that**: it is the stationary point of the running-line **FLOW COEFFICIENT** (incidence/geometry), **NOT** a margin extremum — `SM_N` keeps falling past it on **both** spools and every sampled shape (gated as a **deliberate divergence** so the tempting reading cannot creep in). Rung 36's currency equivalence `E0≥SM_N ⟺ φ_step≤φ_surge` is a **CONSTANT-SPEED** statement; along a **varying-speed running line** flow-coefficient proximity and pressure-ratio margin are **DIFFERENT SCHEDULES**, and the HP spool is the clean exhibit where they diverge. **THE CROSS-RUNG CORRECTION of rung 36 (the rung-28 shape)**: `(★)` is **SURFACED by, not created by** the two-spool work — the same turn sits **INSIDE rung 36's OWN choked envelope** (`π_c`=10 single spool, `Tt4`≈620, still choked; rung 36 simply never plotted that low). Its **gated verdict SURVIVES** (`SM_N` still monotone-thin at low power past the turn, all three surge shapes — **no rung-36 test changes**) but its stated **MECHANISM** ("the trend is set by `φ_op(Tt4)`") was **SINGLE-CHANNEL**: freezing one coordinate at a time separates the **φ-walk** (~56% of the log-decay) from the **SPEED-LINE FLATTENING** (`τ_c−1 ∝ n²`, ~48%), and **below `π*` the φ channel REVERSES** while the speed-line channel keeps consuming margin — at deep throttle the flattening speed line is the **only** channel still thinning it. Rung 36's *conclusion* is untouched: both channels are choke-determined hence **floor-independent**, so its sign-robustness argument survives. **THE MARGINS**: with **matched** shapes and a **common** floor, `SM_L<SM_H` everywhere and `SM_L/SM_H` falls **monotonically to under a third** (3 shapes × 3 floors); at `φ_surge`=0.70 the LP running line **crosses** a line the HP never approaches. **The gated content is the RATIO's COLLAPSE, and the spec names why**: matching the map *shape* does **not** match the **design split**, so `SM_L<SM_H` **already holds AT DESIGN** (`tilted`: 0.3165 vs 0.5186 at `Tt4`=1500, where `φ_L=φ_H=1` and there is no exposure difference) purely because `π_LPC`=3 < `π_HPC`=6 — that level offset is a **design-split artifact, NOT exposure**, and is not attributed to it (in a rung about fixing rung 36's over-attribution). Only the falling **ratio** is the running-line statement; the absolute **gap** is not even monotone (it peaks near `Tt4`≈1300, both margins tending to zero). The **flow-coefficient** headline is unaffected — `φ_L`/`φ_H` are both normalized to 1 at design. **A framing PROBED AND WITHDRAWN** (written, then removed): "the HP running line collapses across flight conditions and the LP's does not" is **VACUOUS** — `τ_LPC−1=K_L·x_L` and `x_H=x_L/τ_LPC` put `x_L` and `x_H` in **BIJECTION**, so the whole matched state is a **one-parameter family** and *both* collapse on *either* ratio; what separates the spools is *which pressure ratios enter the sensitivity*, hence the gate above. **Reduce**: `phi_surge` is the **rung-36 field reused** (no new knob) and read **only** by the surge methods ⇒ a floor-carrying map leaves rung 39 `match` and rung 40 `equilibrium` **bit-for-bit** (`==`); the rung 31–40 suites pass **unchanged** (72/72). **Non-tautological**: `_pi_c_spool` at the operating `(n,φ)` reproduces the shipped `π` on **each** spool (≤1e-9 — two code paths, one `π`, per spool), and the sensitivity gate above. Separate entry point; default `run(…)` untouched ⇒ cycle **bit-for-bit rung 6**. Disclaimed: **two** imposed `φ_surge` (rung 36's cost doubled) ⇒ every margin **magnitude** and every **crossing**; `(★)` is a **CPG + flat-map** statement (shaped maps shift it −3.0%, the variable-`cp` gas +2.5%) and the *turnaround phenomenon* rides on the analytic speed line (a real front-stage-stalling map may not reproduce it); the turn's `Tt4` **location**; which spool binds at **unmatched** shapes/floors (with `press/flow` at design `SM_L>SM_H`); **"the slip protects the LP spool" is NOT claimed** — that is a rigid-shaft **counterfactual this model does not run**, and `slip` is a *speed ratio*, not a surge-proximity measure (`φ_L` is); steady / fully-choked / both NGVs choked / no bypass / one `η_m` / **no bleed valve or variable stator** — inherited or deferred, as are the **transient** surge line (rung 40's complex mode measured against a boundary) and the **subsonic LP branch**. | `docs/rung41-spec.md` |
| 42 | **Interstage bleed — the valve is a degree of freedom on ONE spool** — `TwoSpoolBleedMatcher` (subclasses rung 39's `TwoSpoolMapMatcher`) / `_cascade_bleed(…)` / `_lp_eta_loop_bleed(…)` / `bleed_trade(…)`: rungs 36 **and** 41 both closed with the SAME standing concession ("no bleed valve / variable stator — this rung exhibits the margin they protect, it does not model them"), and rung 41 additionally **LOCATED** the exposure on the **LP** compressor. Rung 42 fits the device there — a fraction `b` extracted at station 25 and dumped overboard. **The structural novelty: the project's first STEADY mass EXTRACTION** — the first time mass *leaves* the flowpath, so the two **COMPRESSORS** pass different air (`ṁ_LPC=ṁ₂`, `ṁ_HPC=(1−b)ṁ₂`); every prior flow change was fuel **ADDITION** (rung 37's `ṁ_c≠ṁ_NGV` was *transient storage*). Deliberately **not** stated as "the first shaft whose compressor and turbine pass different air" — that is **false**, `(1+f)` has made the LPC and LPT pass different mass since rung 38. `b` enters exactly **THREE** places — (1) the **LP shaft balance** `h_c(Tt25)−h_c(Tt2)=η_m(1−b)(1+f)·Δh_LPT` ⇒ **`Tt25` falls** (the one place it touches the energy cascade); (2) the **LP face referral** `(‡-b)` picks up an explicit **`1/(1−b)`**; (3) the **thrust books** (dumped air keeps full ram drag, returns no exhaust momentum) — **and NOT the fourth**: rung 39's `(†)` refers the HPT-NGV choke through `pt4/pt25=π_b·π_HPC` and **both sides are core flow**, so it carries **no `b`**, and the HP shaft balance cancels `(1−b)`, and both turbine pins `(★-HP)/(★-LP)` are untouched (bleed is upstream of station 4). **The structural claim in rung 39's register**: `b` reaches the HP spool **ONLY through the shared `Tt25`**, never through the HP face's own flow referral — which is why `_hp_eta_loop` is reused **VERBATIM** (its **body** is `b`-free; its **arguments** are not), a **code-level** guarantee. **THE FINDING: bleed is a genuinely NEW degree of freedom on the LP spool and NOT on the HP spool.** `x_L=Tt4/Tt2` is built from two **INPUTS**, so it is **EXACTLY** bleed-invariant (`==`) ⇒ the entire **+8–12%** move in `φ_L` is displacement **OFF** the running line: the LP running line becomes a **FAMILY indexed by `b`**. The HP compressor stays on **ONE curve** — take the bled point's `x_H`, root-find the `b=0` **throttle** setting with the same `x_H`, and `φ_H` matches to **0.01–0.016%** (vs the LP's 8–12% at the same `x_L` — a **~700–1300×** contrast, with `Tt4'` differing ~4% so it is a real statement about the real gas, not CPG algebra). Opening the valve does not give the HP a new freedom; it only **SLIDES** it along the line the throttle slides it along. **HONEST ACCOUNTING — much of the HP story is INHERITED and the spec says so** (the rung-40 register): because the HP only slides along its own curve, its whole response **IS** rung 41's closed form `s_H=k(1−π_HPC^(−1/k))−1` — including the turn at `π*`. **What is NEW is PERTURBATION-INDEPENDENCE, and it could have failed**: "throttle-derived `s_H` == valve-derived `s_H`" says the HP response depends on `x_H` **alone, regardless of how that `x_H` was reached** — algebra only on CPG at frozen `f`, because on the shipped gas the HP loop reads `(Tt4, Tt25, f)` **separately**. Measured: **≤0.004 absolute** across a 2.4:1 throttle (0.4039/0.4023 → −0.1410/−0.1369). **`π*` SURFACES A THIRD TIME** (rung 40's move with the slip pattern): since `s_H=0` at `π*=γ_c^(γ_c/(γ_c−1))`, bleed has **exactly zero** first-order HP effect there and **REVERSES SIGN** below it — bracketed between `π_HPC`=3.26878 (`dlnφ_H`=+7.0e-6) and 3.23391 (−1.98e-5), interpolating to **+0.40%**, the *same* fuel-fraction residual rung 41's own kill test isolated (**+0.44%**). Its **location** is inherited; that a **second, independent perturbation sweeps through it** is new. **A HYPOTHESIS WRITTEN DOWN AND REFUTED** (rung 40's convention — kept visible, not dropped): the rung was proposed as *"bleed protects the LP **at the HP spool's expense**"* — the textbook trade — and it is **FALSE**: above `π*` the HP flow coefficient **RISES too** (just 10–100× less), below it falls by ~1e-4. The growing selectivity ratio (8.4 → 659) is the **HP denominator passing through zero** — `dlnφ_L` is nearly constant (~0.022) throughout — **not** "infinite selectivity". **SELF-TARGETING, stated in φ-SPACE and NOT in relative margin** (the tempting relative-`SM_L` version, +23%→+53%, is **CONFOUNDED**: the **absolute** `ΔSM_L` **SHRINKS** 0.056→0.018 pp and only its collapsing base makes the ratio grow — gating it would repeat this project's own **rung-41 lesson**): in rung 41's surge-proximity currency `Δφ_L` is nearly **CONSTANT** (±1% over a 1.76:1 throttle) while `Δφ_H` **collapses ×8** toward its zero at `π*`, so a fixed absolute increment into a **shrinking** LP gap closes **17%→46%** of `(φ_op−φ_surge)` on the LP spool and **1.8%→0.25%** on the HP — robust across shapes × three imposed floors. **THE TRADE**: thrust **−10.0%→−14.7%** and TSFC **+6.3%→+14.6%** at `b`=0.10 as the throttle comes back — the valve gets **more selective AND more expensive together**, *which is why real bleed is SCHEDULED, not left open*; and bleed lowers `π_LPC` hence `pt4`, so it **SHRINKS the choked envelope** (lowest runnable `Tt4` 605→630 K over `b`=0→0.15) — the inherited guard bites sooner and **flags**, it does not lie. **Reduce — EXACT DISPATCH** (rungs 38/39/40's contract): `bleed==0.0` forwards `match` to rung 39's **verbatim** ⇒ **bit-for-bit** (`==`) on the fast gas **and** the **reacting** gas; rung 39's `_cascade_map`/`_lp_eta_loop` left **LITERALLY unchanged** (the rung 31–41 suites pass **unchanged**, 84/84). **A deliberate break in a streak, recorded**: rungs 38/39/40 each ship an **independent bare-math CPG cascade** because their reduce never enters the new code — and neither does rung 42's, yet it ships **without** one. Stated reason (spec § "Why rung 42 carries no bare-math gate"): the HP side is anchored **transitively twice** (gate 2 lands the bled point on the `b=0` line rung 39's own bare-math gate ties down; gate 3 pins the HP *response* to rung 41's closed form), every LP `b>0` **magnitude** is disclaimed, and the one load-bearing LP claim is a **SHAPE** (`Δφ_L` near-constant) plus an **identity in the inputs** (`x_L` invariance) — both survive a uniform magnitude error. Separate entry point; default `run(…)` untouched ⇒ cycle **bit-for-bit rung 6**. Disclaimed: `b` is an **imposed device setting** (a valve position, not a fudge — but every magnitude rides on it, on the two representative maps and on the two imposed `φ_surge` floors inherited from rungs 36/41); a **fixed `b`, not a schedule** `b(n_L)`; bleed moves `φ_op`, **not `φ_surge`** (the **variable-stator** half of the seam is untouched); **overboard dump with zero recovery** and no bleed-duct loss; **steady only** ⇒ **no surge-SURVIVAL claim** (`E0` vs `SM_N` needs rung 41's deferred transient surge line); **customer/cooling bleed at station 3 NOT modelled**; fully-choked / both NGVs choked / one `η_m` / no bypass / isentropic — inherited. | `docs/rung42-spec.md` |

| 43 | **Two-shaft fuel metering — the two spools sit at DIFFERENT points in ONE overshoot loop** — `TwoSpoolFuelTransient` (subclasses rung 40's `TwoSpoolTransient`) / `_close_fuel(…)` / `_instant_fuel(…)` / `equilibrium_fuel(…)` / `integrate_fuel(…)` / `freeze_channels(…)` / `collapse_exponent(…)`: rung 40 named this seam (“**fuel metering** on two shafts — rung 35's control, not carried over”). Rung 43 carries it over: fuel (`ṁ_fuel`) is metered and `Tt4` becomes an **OUTPUT** floating against the airflow **two lagging spools** can currently pump. **HONEST ACCOUNTING FIRST — rung 35's finding is INHERITED and the spec says so throughout** (the rung-40 register): a fuel step at a frozen spool starves the airflow ⇒ `Tt4` **overshoots** (a TIT excursion) and that over-temperature amplifies the surge excursion — **re-measured unchanged** here (`E_temp0`≈0.35 vs `E_surge0`=0.024–0.079, a **4.4–6.3×** ordering: TIT-limited before surge on these maps). That is **not** this rung's finding. **The two-shaft content is a question ONE shaft structurally cannot ask.** The overshoot loop is `f=ṁ_fuel/ṁ_air` → `Tt4=burner(Tt3,f)` → `ṁ4=A4·pt4·MFP*(Tt4)/√Tt4`, and `ṁ_air` is the **LP-face** corrected flow while the `Tt4` it produces is metered back through the **HP-fed** NGV throat (`pt4=π_b·π_HPC·π_LPC·pt2`): **the two spools sit at DIFFERENT points in the ONE loop**. With one shaft there is one clock and no question; with two there is the clock **RATIO `ρ=τ_L/τ_H`** (rung 40's one surviving parameter) and the question becomes **which spool's lag governs the overshoot?** **THE FINDING: NEITHER — and *why* is the rung.** By **channel isolation** (rung 41's `surge_margin_channels` move, applied to the transient — march with one spool's speed **held**): **freezing EITHER spool makes the overshoot WORSE at every sampled point** (gated 12/12: 2 shapes × `ρ`∈{0.5,1,2} × `r`∈{0.25,1}) — **both** spools' motion *relieves* it, neither is a bystander — and the **SHARE of the relief TRADES with `ρ`** (LP channel +51→+30→+16 K as `ρ` rises, HP channel +35→+38→+39 K). *That* is why no single spool's clock can govern it: the responsibility for quenching the overshoot is **SHARED and `ρ`-dependent**. **Direction only** — `ΔLP`+`ΔHP` sums to nothing, it is not a decomposition. **The positive is BOUNDED, and the bound is STRUCTURAL**: at fixed `r` the overshoot `X=Tt4_peak−Tt4_target` rises **monotonically with `ρ`** (6/6, 3 shapes × 2 durations — a heavier LP spool worsens the TIT excursion, the LP-face lag being what spikes `f`), but `ρ` multiplies **ONLY the LP ODE** (`dν_L/ds=Φ_L/ρ`) so **`ρ→∞` IS the LP-frozen system** — the LP-frozen march is `ρ`-independent **BIT-FOR-BIT** and `X(ρ)` converges upward onto it (188.6→192.1→193.0 vs the ceiling 195.6): **the worst TIT excursion a heavy LP spool can produce is computable WITHOUT marching the LP spool at all.** **THE NEGATIVE, stated plainly — there is NO effective clock ratio `r_eff=r/ρ^q`** (`q=0` ⇒ the HP clock, `q=1` ⇒ “the slow spool rate-limits”), and *why it appeared to exist* is the trap worth recording: **THE CURRENCIES ARE CIRCULAR** — the fitted exponent reads back **whichever spool sits in the excursion's DENOMINATOR** (`E_temp_H`, referenced to the `ν_H` running line: **q\*=0.05 on every shape**; the spool-neutral `X`: **0.35–0.45**; `E_temp_L`: **0.45–0.65** — the HP-referenced currency reading far below the spool-neutral one. **Stated at its true strength**: the ordering is `q*(E_temp_H) < q*(X) ≤ q*(E_temp_L)`, strict on two shapes and a **TIE on `press/flow`** — an earlier draft claimed a strict three-way monotone across all three and **the shipped measurement refuted it**). So `E_temp`'s `q≈0` was **never** evidence that “the HP clock governs”; it was the reference reading itself back. Only `X` is spool-neutral, which is why every magnitude is quoted in it — **the data selected the instrument, not the answer it gave** — and even on `X` there is **no collapse** (~14–15% residual; a real collapse in this project runs 1–2%). **THREE MORE CLAIMS WRITTEN, PROBED AND WITHDRAWN** (rung 40's convention — kept visible): ~~“it rides on the geometric-mean composite clock `√det∝ρ^(−1/2)`”~~ **DROPPED** (`√det·√ρ`=const IS a true rung-40 Jacobian identity but is **not connected** to the overshoot; `q*(X)`=0.35 is almost exactly the **MIDPOINT** of the two circular currencies — an averaging artifact, not evidence for ½); ~~“`q=1` is refuted in every currency”~~ **FALSE** (on `X`, `q=0`→0.72 fits *worse* than `q=1`→0.60 — **nothing** about the exponent is currency-independent); ~~“the overshoot is irreducibly two-dimensional”~~ **OVERCLAIM** (only **power-law** collapses were tested — the honest statement is that **rung 35's single-clock `r` framing does not extend to two shafts via any effective clock ratio**). And a fourth, **killed before any code**: ~~“fuel metering breaks rung 39's `(†)` cancellation”~~ — a **category error** (`(†)` is a *steady* η-fixed-point artifact absent from the transient closure), refuted empirically by gate 1. **The closure** is rung 40's `_close` with the burner run **FORWARD** (rung 35's `_tt4_from_f`): still **ONE root in `m_L`**, still **NO shaft balance**. One consequence of the control change, **not** a rung-40 fix: with `Tt4` floating, far past the root the mixture goes lean and the HP map leaves its physical branch, so `_close_fuel` scans **up from the rich wall and takes the FIRST sign change** (rung 40's `_close` is left **LITERALLY unchanged**). **Reduce**: **CONTROL-INVARIANCE** (the non-tautological gate — feeding `ṁ_fuel=f_eq·ṁ_air,eq` of a rung-40 `Tt4`-point to `equilibrium_fuel` returns that point to **machine zero** via the forward-**burner** closure, a genuinely different code path); `lp_disabled` **exact dispatch** ⇒ rung 35's `SpoolTransient` fuel path **bit-for-bit** (`==`); `Tt4`-control **untouched** ⇒ rung 40 bit-for-bit (the rung 31–42 suites pass **unchanged**). **No independent bare-math gate** — the second deliberate break in the rung-38/39/40 streak, reason on the record (unlike rungs 38–40, rung 43's reduce **does** enter the new code: control-invariance lands every steady fuel point **exactly on rung 40's steady manifold**, which rung 40's own bare-math cascade ties down, so the fuel closure is anchored **transitively**; the genuinely new content is transient **signs**, not magnitudes a bare-math replica would constrain). **A pre-existing rung-40 defect found and fixed here** (its reacting-gas 2-D Newton chased an **absolute** `_EQ_TOL`=1e-12 below that gas's ~1e-10 residual noise floor, raising at `Tt4`=1300/1400 while 1500/1450/1200 squeaked under — **non-monotone in `Tt4`**, a solver artifact): a **best-so-far acceptance AFTER the loop**, reached **only** by inputs that previously *raised*, so rungs 40/41/42 are untouched **by construction**. Separate entry point; default `run(…)` untouched ⇒ cycle **bit-for-bit rung 6**. Disclaimed: **every magnitude** (all ride on `ρ` — a disclaimed clock group, **DOUBLED**, inherited from rung 40 — on the two representative maps, the fuel step and the `Tt4` band); **no exponent / no effective clock / no `√det`**; the freeze is **sign/existence** (no calibrated split); **reacting-gas fuel control DEFERRED** (rung 35's concession **verbatim** — `_tt4_from_f` **asserts** against an equilibrium gas rather than mis-solving; the reacting reduce is the `Tt4`-control path); **no surge line on either spool in transient ⇒ no surge-SURVIVAL claim** (rung 41's transient two-spool surge line still deferred); no TIT **redline**; fully-choked / both NGVs choked / one `η_m` / no bypass / no bleed / isentropic — inherited from rungs 38–42. | `docs/rung43-spec.md` |

**The invariant that spans rungs 7–30 (and now 36 and 41): they are all pure diagnostics** (rungs 31–35 are
the **STRUCTURAL rungs** — they compute a *new* off-design operating point: rung 32 with the component
map, rung 33 on the **subsonic-nozzle branch** below unchoke, rung 34 the **dynamic** point where
`N` is a *state* not an output, rung 35 the same transient with **fuel** as the control and `Tt4` an
**output** — but through **separate entry points**
(`OffDesignMatcher`, `MapMatcher`, `SpoolTransient`) that leave the default path untouched). Rung 36
is a **pure diagnostic again** — a **surge line** that *measures* the rung-34/35 transient against a
stability boundary without ever perturbing it (`E`, `ν(s)`, the running line unchanged; surge off ⇒
rung 34/35 bit-for-bit). Rung 37 is **structural-in-time again** — `CombustorTransient` adds two more
*dynamic states* (the combustor plenum `pt4`, the metal `Tm`) that split rung 34's bundled internal-
clock concession (volume-filling CONFIRMS, heat-soak CORRECTS), but through a **separate entry point**
that reduces to rung 34/35 bit-for-bit when both clocks are off (exact dispatch — the extra state is
never built). Rung 38 is **structural again, on a SECOND shaft**: `TwoSpoolMatcher` solves a new
two-spool operating point (`π_LPC, π_HPC` both OUTPUTS) through its own separate entry point
(`build_two_spool_turbojet`/`TwoSpoolEngine`), reducing to rung 31's `OffDesignMatcher` bit-for-bit
by exact dispatch (`lp_disabled=True`) rather than reading anything from the single-spool design run
at all — it is a parallel structural universe beside the cycle, not a diagnostic reading it. NO/N
never enter `_equil_solve`, the production nozzle stays frozen AND ideally-expanded
(`convergent=False`), and the default `build_turbojet(…).run(…)` design run is unchanged, so
**the cycle is bit-for-bit rung 6** — every rung above 6 only *reads* the run's design-point
state (rungs 31–34/37 match a new operating point *beside* it — rung 33 the subsonic-nozzle
branch; rung 36 reads the rung-34 running line/transient *beside* it; rung 37 marches two extra
internal-clock states *beside* it; rung 38 builds its OWN two-spool design point *beside* it,
via a separate factory). Rung 39 is **structural again, on that same second shaft**:
`TwoSpoolMapMatcher` puts a `ComponentMap` on each spool, so both compressor efficiencies AND
both shaft speeds become OUTPUTS — but through a separate entry point that reduces to rung 38
**bit-for-bit** on flat maps and, via `lp_disabled` exact dispatch, to rungs 32/31 as well.
Rung 40 is **structural-in-time on that second shaft**: `TwoSpoolTransient` promotes BOTH shaft
speeds to *dynamic states* (the two-shaft analogue of rung 34, which rung 39 made well-posed),
marching them beside the cycle through a separate entry point that reduces to rung 39 by a 2-D
equilibrium solve and to rung 34 bit-for-bit by exact dispatch. Rung 41 is a **pure diagnostic
again, on that second shaft** — the **two-spool surge line**: it *measures* rungs 39/40's running
line against a stability boundary on **each** compressor without ever perturbing it (a
`phi_surge`-carrying map leaves rung 39's `match` and rung 40's `equilibrium` bit-for-bit), and it
reaches back to **correct rung 36's stated mechanism** while leaving rung 36's gates untouched.
Rung 42 is **structural again, on that same second shaft** — `TwoSpoolBleedMatcher` fits the
**bleed valve** rungs 36 and 41 both deferred, onto the spool rung 41 showed is exposed, and
solves a *new* operating point beside the cycle (the first **steady mass extraction**: the LP
two **compressors** pass different air). It is structural in the rung-31/32 sense but its
*finding* is about a **degree of freedom** — the valve gives the LP spool one and the HP spool
none — and it reduces to rung 39 **bit-for-bit** by exact dispatch at `bleed=0`.
Rung 43 is **structural-in-time again, on that second shaft** — `TwoSpoolFuelTransient` carries
rung 35's **fuel** control onto rung 40's two-shaft plant, so `Tt4` becomes an **OUTPUT** floating
against the airflow *two* lagging spools can pump. Rung 35's TIT-overshoot finding is **INHERITED**
(re-measured unchanged); the new content is a question one shaft structurally cannot ask — `f` is
set at the **LP face** but the `Tt4` it produces is metered back through the **HP-fed** NGV choke,
so the two spools sit at **different points in the one loop** and **neither** clock governs the
overshoot (freezing *either* spool makes it worse). It marches beside the cycle through a separate
entry point, reducing to rung 35 by exact dispatch and to rung 40 by leaving the `Tt4`-control path
untouched.
Each rung's verified anchor data lives in `docs/plans/rungN-anchor-*.md`; `docs/plans/` also holds
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

**Current scope (rung 43).** The **cycle solve** is a thermally-perfect, reacting,
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
still rung-6 exact. Rung 35 (`SpoolTransient.equilibrium_fuel/integrate_fuel`) closes rung 34's one filed
concession — it meters **fuel** (`ṁ_fuel`) instead of commanding `Tt4`, running the burner **forward**
(`_tt4_from_f`) so `Tt4` is an **output** floating against the lagging airflow. It is a **cross-rung
correction of rung 34**: a fuel step at a frozen spool starves the airflow, so `Tt4` **overshoots** (a TIT
excursion — a second acceleration limit) **and** that over-temperature amplifies the airflow deficit, so
fuel control **enlarges** the surge excursion (`E_surge(fuel) > E_Tt4`, gap max at `r→0`, vanishing as
`r→∞`) — the two limits are **coupled**. Reduce: **control-invariance** (`equilibrium_fuel` of a Tt4-point's
fuel reproduces it via the forward-burner closure, machine-zero at design) + Tt4-control untouched ⇒ rung 34
bit-for-bit. Separate entry point; default run still rung-6 exact. Rung 36 (surge methods on
`SpoolTransient` + `ComponentMap.with_phi_surge`) is a **pure diagnostic again**: it draws the **surge
line** rungs 32/34/35 declined and turns their above-running-line excursion into a real **surge margin**.
The zero-new-constant hope (stall at the loading-law peak) is dead (`φ<0`), so a stall flow coefficient
`φ_surge` is **imposed** — but the load-bearing result is a **sign** immune to it: surge margin is **thin
at low power** (the running-line `φ_op(Tt4)` walks toward the fixed floor — choked-hardware determined,
CRS Ch. 9). Because the rung-34 constant-speed excursion `E0` and the margin `SM_N` share a currency
(`E0≥SM_N ⇔ φ_step≤φ_surge`), the low-power burst is most surge-critical on **both** axes (`E0↑` AND
`SM_N↓`) — a **confirmation + sharpening** of rung 34's implicit worst case (`E0` was already largest
there; **no relocation**), with `SM_N` the genuinely new info (not a rescale of `E`). The **crossing**
into surge is disclaimed (rides on `φ_surge`); only the trend is gated. Surge off (`φ_surge=0`) ⇒ rung 34/35 bit-for-bit;
default run still rung-6 exact. Rung 37 (`CombustorTransient`, subclassing `SpoolTransient`) closes rung
34's **bundled internal-clock concession** — it adds the two omitted dynamic states (the combustor
**plenum** `pt4`, the **metal** `Tm`) and they **split**: **volume-filling** (`τ_fill ≪ τ_spool`)
**CONFIRMS** ("faster clocks below `τ_spool` don't change the `r` framing") — the `r→0` peak surge
excursion lands on rung-35's `E0` to machine zero, independent of the fill clock — while its real content
is **STRUCTURAL** (the first rung where compressor mass flow `≠` NGV mass flow, `~22%`, the plenum stores
the difference; the compressor is run from **back-pressure**, a third use of the map); **heat-soak**
(`τ_soak ~ τ_spool`) **CORRECTS** it — a second STATE makes `E = E(r, θ₀)` **history-dependent**; the
modeled combustor sink is surge-**PROTECTED** (`cold < hot-reslam < adiabatic`; rung 34/35's adiabatic
no-soak case is the **ceiling**, a hot reslam merely the least-protected case), and the cost is the
**accel-time LAG** (~2.5–3× slower). This channel is the **OPPOSITE sign** to the operational bodie/reslam
surge hazard (compressor-side, unmodeled) — so it does NOT reproduce it (advisor). Modeled **separately**
(the contrast is the rung). Reduce — **exact dispatch**: both clocks off ⇒ the
inherited `equilibrium_fuel`/`integrate_fuel` are literally rung 34/35 (the rung 31–36 suites pass
unchanged), and the two equilibria reproduce rung 35 via independent closures (back-pressure; `Q=0` at
steady ⇒ heat-soak never moves the running line). Separate entry point; default run still rung-6 exact.
Rung 38 (`build_two_spool_turbojet`/`TwoSpoolEngine`/`TwoSpoolMatcher`) is the first **two-shaft**
rung: a plain (no-bypass) turbojet with LPC+LPT on one shaft, HPC+HPT on the other. It adds a
**THIRD choked throat** no single-spool rung has — the LP-turbine NGV (station `45`, area `A45`) —
and with all three throats choked, rung 31's `(★)` mass-flow trick chains TWICE: `τ_HPT` from
`(A4,A45)` alone, `τ_LPT` from `(A45,A8)` alone, both independent of either compressor. **A
self-correction mid-rung**: the first framing ("the LP spool solves independent of the HP spool")
is **WRONG** — `η_HPT` demonstrably moves `π_LPC` too, since it shapes the shared `Tt45`. What
survives is narrower and airtight: each compressor's OWN isentropic efficiency is a **terminal
leaf** (`η_LPC` cannot reach `π_HPC`, `η_HPC` cannot reach `π_LPC`), so the two compressor
PRESSURE ratios are never a joint (2×2) solve — a **NO-COMPRESSOR-MAP model artifact** (the
rung-31-before-rung-32 shape), not a physical law. Scope: the fully-choked branch only (nozzle
unchoke is flagged, not solved — a rung-33-shaped follow-on). Reduce — **exact dispatch**:
`lp_disabled=True` never builds an LPC/LPT/`A45` at all; `TwoSpoolMatcher` forwards every
`.match()` call to an internally-held `OffDesignMatcher`, bit-for-bit. Separate entry point;
default run still rung-6 exact. Rung 39 (`TwoSpoolMapMatcher`) builds the seam rung 38 named as
its own likely correction — a `ComponentMap` on **each** spool — and **refutes rung 38's
prediction while confirming its verdict** (the rung-28 shape). `π_LPC` **cancels** out of the HP
compressor's corrected flow (`pt4/pt25 = π_b·π_HPC`: the LPC raises pressure and mass flow
*proportionally*, so the HP core sees the same **corrected** flow whatever the LP spool
delivers), so the HP map coordinates are a closed fixed point in `π_HPC` alone. The map
therefore opens **exactly ONE arrow, HP→LP**: `η_HPC` moves `π_LPC` (−1.5e-4…−6.7e-4) while
`η_LPC` leaves `π_HPC` **bit-for-bit unchanged**. The cascade is **not** dissolved into a 2×2 —
it **acquires a direction** (HP solved first, LP onto it), and the closed leaf is a *code-level*
guarantee because the solve is built triangular. A representative **turbine** map re-opens the
leaf only at ~1e-6 (119×–548× weaker — rung 32's "turbine pinned in corrected speed", on the LP
spool). The **structural novelty is two shaft speeds**, hence the **slip `N_L/N_H`**: exactly 1
on a CPG gas with flat maps (`(1+f)` and `Tt4` both cancel — an identity, verified
`f`-independently), broken ~1.5% by the `cp(T)` curve and ~5.0% by the map, and falling
monotonically with throttle (the LP spool falls away from the HP spool — sign shape-robust,
magnitude disclaimed). That **inverts rung 32**: there the map only *re-labelled* map-free work;
here it is identically zero without the map **on the CPG gas** (so there the map is the **sole**
channel and genuinely *creates* the object) — but **NOT unconditionally**: on the reacting gas
the same flat maps already give 0.9835 at `Tt4`=900, so the map is the **dominant** channel
(~3.4×), not the only one. Reduce: flat
maps ⇒ rung 38 bit-for-bit; `lp_disabled` ⇒ rung 32 (shaped) / rung 31 (flat) by exact dispatch.
Separate entry point; default run still rung-6 exact. Rung 40 (`TwoSpoolTransient`) builds the
seam rung 39 declared well-posed: BOTH shaft speeds become **STATES** under two inertia ODEs,
closed **forward** with no shaft balance (a 1-D root in `m_L`), so nondimensionalizing leaves
exactly one parameter — the clock **RATIO** `ρ=τ_L/τ_H`, the second clock rung 34 had to impose
(**each spool is the other's clock**). **Much of the rung is INHERITED and it says so**: the
turbine-split invariance and the lead threshold's `σ_crit≡1` identity are rung 39 **B1** restated
for the transient (on the running line `σ_crit` *reduces to the steady slip*), and its two
breaking channels (`cp(T)` +4.3e-2, map +2.5e-1) are **B2**'s shape. **The new finding is that
`ρ`'s power SPLITS**: it can **never** destabilize the pair (`a<0, d<0, ad>bc` carry **no `ρ`** —
252 points, zero violations) yet it **decides whether the mode is real or COMPLEX**
(`disc=(a/ρ−d)²+4bc/ρ` vanishes at `ρ=a/d`). **`bc<0` iff the LP compressor map is SHAPED**, with
`hp-only` (HP shaped, LP flat, no band) the **discriminator** — so the complex inter-spool mode is
**MAP-CREATED**, rung 39's slip pattern a third time; `|Im/Re|≤0.25` is **disclaimed, not
"negligible"**. A **negative** is stated plainly: `σ_crit` is **first-instant only** — the
finite-ramp excursion is **schedule-slaved**, and two dynamic claims were probed and withdrawn
(one tautological, one refuted by measurement). Reduce: the **2-D** equilibrium ⇒ rung 39 (≤1e-12,
forward closure only ⇒ non-circular); `lp_disabled` ⇒ rung 34 **bit-for-bit**. Separate entry
point; default run still rung-6 exact. Rung 41 (surge methods on `TwoSpoolMapMatcher` +
`SpoolTransient.surge_margin_channels`) draws rung 36's **surge line on BOTH compressors** and
finds the exposure **SPLITS**: the two-spool running line does not halve the low-power surge
problem, it **concentrates** it on the **LP** compressor (`φ_L` falls ~29% over a 2:1 throttle,
`φ_H` ~7% and **bounded**). The cause is rung 39's `(†)` cancellation, in **closed form**: the HP
face's sensitivity `s_H = k(1−π_HPC^(−1/k))−1` contains **no LP quantity**, the LP's needs the
**product**, and dropping `π_HPC` from `s_L` fails by 0.8–1.0 with the **wrong sign**. Its
corollary is the **live zero-new-constant** anchor rung 36's dead one never got —
**`1+η_c(τ_c−1)=γ_c` ⟺ `π_c*=γ_c^(γ_c/(γ_c−1))`** ≈ 3.2467, invariant to every efficiency, both
hot-section knobs, the split and the flight condition while `Tt4*` moves 1.76×, with the whole
+0.44% residual **killed** by driving `f`→0. `(★)` is an **incidence** fact, **not** a margin
extremum (`SM_N` keeps falling past it — gated as a deliberate divergence), and that divergence
**corrects rung 36** (rung-28 shape): the same turn sits inside rung 36's own choked envelope, its
gated verdict **survives**, but its single-channel mechanism is corrected — the φ-walk (~56%) and
the **speed-line flattening** (~48%) are comparable and below `π*` the φ channel **reverses**.
Reduce: `phi_surge` is the rung-36 field reused, read only by the surge methods ⇒ rung 39/40
bit-for-bit; the rung 31–40 suites pass unchanged. Separate entry point; default run still
rung-6 exact. Rung 42 (`TwoSpoolBleedMatcher`) fits the **bleed valve** rungs 36 and 41 both
deferred, onto the spool rung 41 located: a fraction `b` extracted at station 25 and dumped —
the project's **first steady mass extraction** — the first time mass *leaves* the flowpath, so
the two **compressors** pass different air (`(1+f)` had already made each compressor and its own
turbine differ, since rung 38). `b` enters
exactly **three** places (the LP shaft balance ⇒ `Tt25` falls; the LP face referral ⇒ an explicit
`1/(1−b)`; the thrust books) and **not the fourth** — rung 39's `(†)` is core flow on both sides
and carries no `b`, so `_hp_eta_loop` is reused **verbatim**. **The finding: bleed is a genuinely
NEW degree of freedom on the LP spool and NOT on the HP spool** — `x_L=Tt4/Tt2` is *exactly*
bleed-invariant, so all of `Δφ_L` (+8–12%) is displacement **off** the LP running line (which
becomes a family in `b`), while the HP stays on **one curve** (`φ_H(x_H)` invariant to 0.01–0.016%,
a ~1000× contrast) — the valve only *slides* it where the throttle slides it. **Inherited (and
said so)**: the HP response is therefore rung 41's `s_H`, `π*` sign-reversal included. **New**:
**perturbation-independence** — valve-derived `s_H` == throttle-derived closed form to ≤0.004,
which could have failed (the real-gas HP loop reads `Tt4, Tt25, f` separately). `π*` **surfaces a
third time** at **+0.40%**, the same fuel-fraction residual rung 41's kill test isolated. The
proposed "**bleed protects LP at the HP's expense**" is **refuted and kept visible** (the HP is
*helped* above `π*`, 10–100× less). **Self-targeting stated in φ-space** (the relative-`SM`
version is confounded — absolute `ΔSM_L` shrinks): `Δφ_L` near-constant, `Δφ_H` ×8 down, so the
fraction of the shrinking gap closed rises 17%→46% (LP) and falls 1.8%→0.25% (HP). The trade:
thrust −10.0%→−14.7%, TSFC +6.3%→+14.6%, and the choked envelope shrinks. Reduce: `bleed=0` ⇒
rung 39 **bit-for-bit** by exact dispatch (fast and reacting gas); the rung 31–41 suites pass
unchanged (84/84). Separate entry point; default run still rung-6 exact. Rung 43
(`TwoSpoolFuelTransient`) carries rung 35's **fuel** control onto rung 40's two-shaft plant, closing
the seam rung 40 named: `ṁ_fuel` is metered and `Tt4` **floats** against the airflow *two* lagging
spools can pump (the burner run **FORWARD**, still one root in `m_L`, still no shaft balance).
Rung 35's TIT-overshoot finding **re-measures unchanged** and is labelled **INHERITED** throughout
(`E_temp0`≈0.35 vs `E_surge0`=0.024–0.079 — TIT-limited before surge on these maps). The new content
is a question **one shaft cannot ask**: `f=ṁ_fuel/ṁ_air` is set at the **LP face** but the `Tt4` it
produces is metered back through the **HP-fed** NGV choke, so **the two spools sit at different
points in the ONE overshoot loop**. Asking which spool's lag governs it, the answer is **NEITHER** —
by **channel isolation**, freezing **either** spool makes the overshoot **worse** (12/12) and the
*share* of the relief **trades with `ρ`** (direction only; the two do not sum). The positive is
**bounded and structurally so**: `X` rises monotonically with `ρ`, but `ρ` multiplies **only the LP
ODE**, so **`ρ→∞` IS the LP-frozen march** — `ρ`-independent **bit-for-bit**, and `X(ρ)` converges
up onto it. **The negative is stated plainly**: there is **no** effective clock ratio `r/ρ^q`, and
the reason it *seemed* to exist is that **the referenced currencies are CIRCULAR** — the fitted
exponent reads back whichever spool sits in the denominator (0.05 → 0.35 → 0.65, HP → neutral → LP),
so the data selected the instrument. Three further claims were written, probed and **withdrawn**
(the `√det` composite clock, "`q=1` refuted in every currency", "irreducibly 2-D"), plus a fourth
killed before any code (the `(†)` framing). Reduce: **control-invariance** (machine zero via the
forward-burner closure) + `lp_disabled` ⇒ rung 35 bit-for-bit + `Tt4`-control untouched ⇒ rung 40
bit-for-bit. Separate entry point; default run still rung-6 exact.

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
  hardware/CFD map** with a surge line (would earn a surge-margin *magnitude* — the *representative* surge
  line + margin **sign** is **BUILT BY RUNG 36**, below); the transient/spool-dynamics seam (c above, now
  that `N` exists). (The subsonic-nozzle branch is **BUILT BY RUNG 33**, below — but subsonic + component
  map stays OUT OF SCOPE: `MapMatcher` overrides `match` and stays choked-only.)
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
  closure only (non-circular). **What rung 34 leaves open:** (a) ~~**combustor volume-filling / heat-soak
  dynamics** — faster clocks *below* `τ_spool`~~ — **BUILT BY RUNG 37** (`docs/rung37-spec.md`, below —
  the two clocks SPLIT: volume-filling IS a fast clock below `τ_spool` and CONFIRMS the concession;
  heat-soak is `~τ_spool` and CORRECTS it); (b) ~~a true
  **`ṁ_fuel(t)` fuel-metering schedule** with `Tt4` an output~~ — **BUILT BY RUNG 35** (below); (c) ~~a
  **surge line** (rung 32's standing concession — would turn the excursion into a surge-margin number)~~
  — **BUILT BY RUNG 36** (`docs/rung36-spec.md`, below); (d) ~~two-spool / multi-shaft
  matching~~ — the STEADY case **BUILT BY RUNG 38** (`docs/rung38-spec.md`, below); the
  **transient** (two shaft inertias, the natural two-spool analogue of THIS rung) needs
  rung 38's steady matcher first and remains open;
  (e) feeding the marched `N(t)` into the production cycle — a re-foundation, not a rung.
- **Fuel metering — `Tt4` an OUTPUT** — **BUILT BY RUNG 35** (`docs/rung35-spec.md`,
  `SpoolTransient.equilibrium_fuel/integrate_fuel`, `_close_compressor_fuel`, `_tt4_from_f`,
  `docs/plans/rung35-anchor-fuel-metering.md`). Rung 34 named this seam and filed it as its `Tt4(t)`-control
  concession. Rung 35 meters **fuel** (`ṁ_fuel`, the *mass flow* — commanding the ratio `f` would be a
  re-labeling), runs the burner **forward** so `Tt4` floats against the **lagging** airflow, and closes the
  compressor by NGV-choke consistency with **no shaft balance** (rung 34's move). **A cross-rung CORRECTION
  of rung 34**: a fuel step at a frozen spool starves the airflow, so `Tt4` **overshoots** (a TIT excursion —
  a second acceleration limit rung 34's fiat-`Tt4` HID) **and** the over-temperature amplifies the airflow
  deficit, so fuel control **ENLARGES** the surge excursion (`E_surge(fuel) > E_Tt4`, gap max at `r→0`,
  vanishing as `r→∞`) — the surge and TIT limits are **coupled**, and rung 34 under-counted surge. Sign
  shape-robust; magnitude disclaimed. Reduce: **control-invariance** (the fuel of a Tt4-point reproduces it
  via the forward-burner closure, machine-zero at design) + Tt4-control untouched ⇒ rung 34 bit-for-bit.
  **What rung 35 leaves open:** reacting-gas fuel control (the forward burner is built for the non-equilibrium
  gas — the finding is gas-independent); a true `ṁ_fuel(t)` metering-unit schedule with both ends free and a
  fuel-metering-valve model; and rung 34's remaining seams (~~surge line~~ — **BUILT BY RUNG 36**;
  ~~volume-filling/heat-soak~~ — **BUILT BY RUNG 37**; ~~two-spool matching~~ — **BUILT BY RUNG 38**
  (the transient remains open)).
- **The surge line** — **BUILT BY RUNG 36** (`docs/rung36-spec.md`, surge methods on `SpoolTransient`
  + `ComponentMap.with_phi_surge`, `docs/plans/rung36-anchor-surge-line.md`). Rungs 32, 34 and 35 all named
  this seam and all declined it for the same reason (a representative efficiency island is not a stability
  boundary; the margin rides on where you draw the line). Rung 36 draws it and turns the above-running-line
  excursion into a real **surge margin**. The **zero-new-constant** hope (stall at the loading-law peak
  `φ=1−l/(2σ)`) is **DEAD** — it lands at `φ<0` for all three surge shapes, so a stall flow coefficient
  `φ_surge` is **IMPOSED** (the free parameter rung 32 warned of). The rung survives because a **SIGN**
  survives it: surge margin is **thin at LOW power** (the running-line `φ_op(Tt4)` walks toward the fixed
  floor — choked-hardware determined, not floor-determined; **CRS Ch. 9**), sign-robust across shapes ×
  imposed `φ_surge` × an `n`-slope on the floor, under both margin definitions (constant-flow a weak
  sign-check only — its magnitude extrapolates absurdly). **The compounding (confirmation + sharpening,
  NOT relocation)**: `E0` (rung-34 constant-speed excursion) and `SM_N` share a currency
  (`E0≥SM_N ⇔ φ_step≤φ_surge`), and both point low (`E0↑`, `SM_N↓`, reinforcing), so the low-power burst is
  worst on **both** axes — rung 34's `E0` was **already** largest there (`argmax` unchanged, no relocation),
  and `SM_N` is the new info (not a rescale of `E`). **The crossing into surge is DISCLAIMED** (rides on
  `φ_surge`; `E0` is floor-independent) — only the trend is gated. Pure diagnostic: surge off ⇒ rung 34/35 bit-for-bit;
  `_pi_c_map` == the shipped `π_c`; default run rung-6 exact. **What rung 36 leaves open:** a **bleed valve /
  variable stator** model (the devices that raise `φ_surge` at low speed — rung 36 exhibits the margin they
  protect, does not model them); a **real hardware/CFD surge line** with a measured `(ṁ,π)` shape (rung 32's
  standing "real map" concession — would earn a margin *magnitude*, and a steeply speed-dependent one
  could *oppose* the `E0`/`SM_N` schedules and produce a genuine **relocation** that rung 36's parallel
  representative maps do not); the **subsonic-branch** surge margin (choked branch only); ~~**combustor
  volume-filling / heat-soak**~~ — **BUILT BY RUNG 37**; ~~**two-spool** matching~~ — **BUILT BY
  RUNG 38** (the two-shaft transient, inherited from rung 34, is still open).
- **The two internal clocks (combustor volume-filling + heat-soak)** — **BUILT BY RUNG 37**
  (`docs/rung37-spec.md`, `CombustorTransient`, `docs/plans/rung37-anchor-combustor-dynamics.md`). Rungs 34–36
  all named this seam (rung 34 bundled it: "faster clocks below `τ_spool`, they do not change the `r`
  framing"). Rung 37 built it and the two clocks **SPLIT**. **Volume-filling** (a combustor plenum,
  `τ_fill ≪ τ_spool`) **CONFIRMS** the concession — the `r→0` peak surge excursion is unmoved (== rung-35
  `E0` to machine zero, independent of the fill clock) — while its real content is **STRUCTURAL**: the
  **first rung where compressor mass flow ≠ NGV mass flow** (`~22%`, the plenum stores the difference;
  the compressor is run from **back-pressure**, a third use of the map). **Heat-soak** (a metal state
  `Tm`, `τ_soak ~ τ_spool`) **CORRECTS** it — a second STATE makes `E = E(r, θ₀)` **history-dependent**;
  the modeled combustor gas-path sink is surge-**PROTECTED** (`cold < hot-reslam < adiabatic`; a colder
  NGV passes more corrected flow, `φ` away from surge — rung 34/35's adiabatic no-soak case is the
  **ceiling**), and the cost is the **accel-time LAG** (~2.5–3× slower — the primary CRS/Walsh-Fletcher
  effect). **Honest scope (advisor):** this channel is the **OPPOSITE sign** to the operational
  **bodie/reslam** surge hazard (heat soakage moving the working line *toward* surge — an **unmodeled
  compressor-side** channel), so it does not reproduce it; the overlap with the real bodie is the
  *history-dependence*, not the sign. Reduce — **exact dispatch** (both clocks off ⇒ inherited `equilibrium_fuel`/`integrate_fuel` are
  literally rung 34/35; the rung 31–36 suites pass unchanged) + the two equilibria == rung 35 via
  independent closures (`Q=0` at steady ⇒ heat-soak never moves the running line). **What rung 37 leaves
  open:** the **combined 3-state** (`ν, pt4, Tm` together — the effects are exhibited separately, the
  interaction not claimed); an **energy-storage** plenum (`Tt4` quasi-steady in the volume) and a
  distributed/flow-varying `hA`; **tip-clearance** transients; ~~**two-spool** matching~~ — **BUILT
  BY RUNG 38** (the two-shaft transient remains open); feeding the marched
  internal states into the production cycle (a re-foundation, not a rung).
- **Two-spool matching** — **BUILT BY RUNG 38** (`docs/rung38-spec.md`, `build_two_spool_turbojet`
  / `TwoSpoolEngine` / `TwoSpoolMatcher`, `docs/plans/rung38-anchor-two-spool-matching.md`). Rungs
  31, 34 and 37 all named this seam. Rung 38 built the first two-shaft (LPC+LPT / HPC+HPT, no
  bypass) matcher: a THIRD choked throat (the LP-turbine NGV, `A45`) chains rung 31's `(★)` trick
  twice, pinning both turbine ratios by geometry alone. Self-corrected mid-rung: "the LP spool
  solves independent of the HP spool" is wrong (`η_HPT` moves `π_LPC` too); what survives is that
  each compressor's OWN efficiency is a terminal leaf, so the two compressor PRESSURE ratios are
  never a joint (2×2) solve — a no-compressor-map model artifact, not a physical law. Reduce —
  exact dispatch (`lp_disabled=True` ⇒ `OffDesignMatcher` bit-for-bit). **What rung 38 leaves
  open:** nozzle-unchoke on the LP spool (a rung-33-shaped follow-on, flagged not solved);
  ~~**two-spool + component maps**~~ — **BUILT BY RUNG 39** (below — and it **refuted** rung
  38's own prediction: the coupling does NOT become a 2×2, it becomes one-way); the **two-shaft
  transient** (two inertias, the natural two-spool rung-34 analogue, needs this steady matcher
  first — and now rung 39's two *speeds* to be its states); a fan/bypass split (a different
  engine entirely).
- **Two-spool + component maps** — **BUILT BY RUNG 39** (`docs/rung39-spec.md`,
  `TwoSpoolMapMatcher`, `docs/plans/rung39-anchor-two-spool-maps.md`). Rung 38 named this seam and
  **predicted it would correct it** ("a real compressor MAP would very likely reintroduce genuine
  2×2 coupling, exactly as rung 32 corrected rung 31"). Rung 39 built it and the prediction is
  **WRONG** — the **rung-28 shape**: the *verdict* (no joint 2×2 between the compressor pressure
  ratios) **SURVIVES**, the stated *reason for expecting it to fail* is **refuted**. `π_LPC`
  **cancels** out of the HP compressor's corrected flow (`pt4/pt25 = π_b·π_HPC` — the LPC raises
  pressure and mass flow proportionally), so the HP map coordinates close on `π_HPC` alone and
  the map opens **exactly ONE arrow, HP→LP**: `η_HPC → π_LPC` real and negative, `η_LPC → π_HPC`
  **bit-for-bit zero** (gas-independent). The cascade **acquires a direction** instead of
  dissolving. A representative **turbine** map re-opens the leaf at ~1e-6 only (119×–548× weaker
  — rung 32's "turbine pinned in corrected speed", on the LP spool; ratio disclaimed). The
  **structural novelty** is **two shaft speeds** ⇒ the **slip `N_L/N_H`**: an exact identity
  (=1) on CPG + flat maps (`(1+f)` and `Tt4` both cancel), broken ~1.5% by the `cp(T)` curve and
  ~5.0% by the map, monotone-falling with throttle (LP falls away from HP; sign shape-robust,
  magnitude disclaimed) — **inverting rung 32 ON CPG** (there the map is the *sole* channel and the map
  only *re-labelled* map-free work in rung 32); on the **real** gas the `cp(T)` curve alone already breaks
  the identity, so the map is the **dominant** channel, not the only one.
  Reduce: flat maps ⇒ rung 38 bit-for-bit; `lp_disabled` ⇒ rung 32/31 by exact dispatch.
  **What rung 39 leaves open:** ~~the **two-shaft transient**~~ — **BUILT BY RUNG 40** (below);
  ~~a **two-spool surge line / surge margin**~~ — **BUILT BY RUNG 41** (below; and the
  slip-protects-the-LP question it named is **still declined**, deliberately — rung 41 shows the
  complementary truth, that the LP is the **exposed** spool, and files the rigid-shaft
  counterfactual as not run); the
  **subsonic/unchoked** LP branch (still rung 38's, now with the map on top); a **real
  hardware/CFD map** (rung 32's standing concession, doubled).
- **The two-shaft transient** — **BUILT BY RUNG 40** (`docs/rung40-spec.md`, `TwoSpoolTransient`,
  `docs/plans/rung40-anchor-two-shaft-transient.md`). Rungs 34, 37, 38 and 39 all named this seam;
  rung 39 made it **well-posed** by supplying the two shaft speeds rung 38 could not. Rung 40
  makes both speeds **STATES** under two inertia ODEs, closed **forward** with no shaft balance
  (a 1-D root in `m_L`), leaving exactly one parameter — the clock **RATIO** `ρ=τ_L/τ_H`, the
  second clock rung 34 had to impose. **Honest accounting: much of the rung is INHERITED** —
  the turbine-split invariance and the `σ_crit≡1` identity are rung 39 B1 restated for the
  transient (on the running line `σ_crit` *reduces to the steady slip*), and its two breaking
  channels are B2's shape. **The new finding is that `ρ`'s power SPLITS**: it can **never**
  destabilize the pair (the sign conditions `a<0, d<0, ad>bc` carry **no `ρ`** — measured over
  252 points, zero violations) but it **decides whether the mode is real or COMPLEX**
  (`disc=(a/ρ−d)²+4bc/ρ` vanishes at `ρ=a/d`). **`bc<0` iff the LP compressor map is SHAPED** —
  with `hp-only` (HP shaped, LP flat, **no band**) the discriminator proving it is the **LP map
  specifically**. The mode is **MAP-CREATED**, rung 39's slip pattern a third time; its strength
  (`|Im/Re|≤0.25`) is **disclaimed, not billed as negligible**. A **negative** is stated plainly:
  `σ_crit`'s authority is **first-instant only** (the finite-ramp excursion is schedule-slaved;
  two candidate dynamic claims were probed and withdrawn — one tautological, one refuted).
  Reduce: the 2-D equilibrium ⇒ rung 39 (≤1e-12, forward closure only); `lp_disabled` ⇒ rung 34
  bit-for-bit. **What rung 40 leaves open:** ~~a **two-spool surge line**~~ — **BUILT BY
  RUNG 41** (below) for the STEADY running line; measuring rung 40's **complex inter-spool mode**
  against that boundary is still open (rung 41 is steady-only); the
  **subsonic/unchoked LP branch** in transient; ~~**fuel metering** on two shafts
  (rung 35's control, not carried over)~~ — **BUILT BY RUNG 43** (below); rung 37's
  **internal clocks** on two shafts (and the
  audit of *its* shaft+metal Jacobian for complex modes, which this rung scoped around); a
  **real hardware/CFD map** (rung 32's standing concession, doubled).
- **The two-spool surge line** — **BUILT BY RUNG 41** (`docs/rung41-spec.md`, surge methods on
  `TwoSpoolMapMatcher` + `SpoolTransient.surge_margin_channels`,
  `docs/plans/rung41-anchor-two-spool-surge.md`). Rungs 39 **and** 40 both named this seam in
  nearly the same words. Rung 41 draws rung 36's line on **both** compressors and finds the
  two-spool running line does not *halve* the low-power surge problem — it **CONCENTRATES** it on
  the **LP** compressor (`φ_L` falls ~29% over a 2:1 throttle, `φ_H` ~7% and **bounded**; ratio
  3.8×–4.3× across four shapes). The cause is rung 39's `(†)` cancellation and it is **closed
  form, not a sign**: the HP face's flow-coefficient sensitivity `s_H = k(1−π_HPC^(−1/k))−1`
  contains **no LP quantity**, while the LP's needs the **product** — and dropping `π_HPC` from
  `s_L` misses by 0.8–1.0 **with the wrong sign**. Its corollary is the **LIVE zero-new-constant
  anchor** rung 36's dead one never got: **`1+η_c(τ_c−1)=γ_c` ⟺ `π_c* = γ_c^(γ_c/(γ_c−1))`**
  (≈3.2467), invariant to every efficiency, both hot-section knobs, the design split and the
  flight condition while `Tt4*` moves 1.76× — *the closest approach is at a pressure ratio, not a
  throttle setting* — with the entire +0.44% residual **killed** by driving `f`→0. `(★)` is an
  **incidence** fact, **NOT** a margin extremum (`SM_N` keeps falling past it on both spools —
  gated as a deliberate divergence), and *that* divergence is a **cross-rung CORRECTION of rung
  36** (the rung-28 shape): the same turn sits **inside rung 36's own choked envelope**, its
  gated verdict **survives**, but its stated single-channel mechanism ("the trend is set by
  `φ_op`") is corrected — the **φ-walk** (~56%) and the **speed-line flattening** `τ_c−1∝n²`
  (~48%) are comparable, and below `π*` the φ channel **reverses**. Reduce: `phi_surge` is the
  rung-36 field reused, read only by the surge methods ⇒ rung 39/40 **bit-for-bit**.
  **What rung 41 leaves open:** the **transient** two-spool surge line (rung 40's complex mode
  measured against the boundary); the **subsonic/unchoked LP** branch's margin; ~~a **bleed
  valve**~~ — **BUILT BY RUNG 42** (below; the **variable stator** half is still open); a **real
  hardware/CFD surge line** (rung 32/36's standing concession, doubled); and the **rigid-shaft
  counterfactual** that would settle whether the slip *protects* the LP spool.
- **The bleed valve** — **BUILT BY RUNG 42** (`docs/rung42-spec.md`, `TwoSpoolBleedMatcher`,
  `docs/plans/rung42-anchor-interstage-bleed.md`). Rungs 36 **and** 41 both filed the same
  concession ("no bleed valve / variable stator — this rung exhibits the margin they protect, it
  does not model them"), and rung 41 **located** the exposure on the LP compressor. Rung 42 fits
  the valve there: a fraction `b` extracted at station 25 and dumped — the project's **first
  STEADY mass EXTRACTION** — the first time mass *leaves* the flowpath, so the two
  **compressors** pass different air (`ṁ_LPC=ṁ₂`, `ṁ_HPC=(1−b)ṁ₂`); every prior flow change was
  fuel **addition**, and rung 37's `ṁ_c≠ṁ_NGV` was transient storage. `b` enters
  exactly **three** places — the LP shaft balance (`η_m(1−b)(1+f)`, so `Tt25` falls), the LP face
  referral `(‡-b)` (an explicit `1/(1−b)`), the thrust books — and **NOT the fourth**: rung 39's
  `(†)` is core flow on both sides and carries no `b`, so `_hp_eta_loop` is reused **verbatim**
  (its body is `b`-free, its arguments are not — rung 39's leaf, one rung on).
  **THE FINDING: bleed is a genuinely NEW degree of freedom on the LP spool and NOT on the HP
  spool.** `x_L=Tt4/Tt2` is **exactly** bleed-invariant (both are *inputs*), so the whole
  `+8–12%` in `φ_L` is displacement **OFF** the running line — the LP line becomes a **family**
  indexed by `b`; the HP stays on **one curve** (`φ_H(x_H)` bleed-invariant to **0.01–0.016%**, a
  **~1000×** contrast), so the valve only **slides** it along the line the throttle slides it
  along. **INHERITED and the spec says so** (rung-40 register): the HP response is therefore rung
  41's closed-form `s_H`, sign reversal at `π*` included. **NEW: perturbation-independence** —
  valve-derived `s_H` == throttle-derived closed form to **≤0.004** over a 2.4:1 throttle, which
  **could have failed** (on the real gas the HP loop reads `Tt4, Tt25, f` **separately**; only CPG
  at frozen `f` makes it one-parameter in `x_H`). `π*` **surfaces a THIRD time**: `dφ_H/db`
  crosses zero bracketing `π*` at **+0.40%** — the *same* fuel-fraction residual rung 41's own
  kill test isolated (+0.44%). **A hypothesis REFUTED and kept visible** (rung 40's convention):
  "bleed protects LP **at the HP's expense**" is **FALSE** — above `π*` the HP is *helped* too,
  just 10–100× less; below it, hurt by ~1e-4. **Self-targeting, stated in φ-space** (the
  relative-`SM` version is **confounded** — absolute `ΔSM_L` *shrinks*; this project's own rung-41
  lesson): `Δφ_L` is near-**constant** (±1%) while `Δφ_H` collapses **×8**, so the fraction of the
  shrinking `(φ_op−φ_surge)` gap closed **rises 17%→42%** on LP and **falls 1.8%→0.4%** on HP.
  **The trade**: thrust −10.0%→−14.7%, TSFC +6.3%→+14.6% as throttled — more selective *and* more
  expensive together (*why bleed is scheduled*); and it **shrinks the choked envelope** (605→630 K).
  Reduce — **exact dispatch**: `bleed=0` forwards `match` to rung 39 **bit-for-bit** (`==`, fast
  **and** reacting gas); rung 39's `_cascade_map`/`_lp_eta_loop` left **literally unchanged** (the
  rung 31–41 suites pass unchanged, 84/84). **What rung 42 leaves open:** the **variable stator**
  (it moves `φ_surge` itself — the other half of the seam); a bleed **schedule** `b(n_L)` and
  bleed **during a transient** (the surge-*survival* claim, `E0` vs `SM_N`, needs rung 41's
  deferred transient surge line); **customer/cooling bleed** at station 3 (a different sink);
  bleed-duct losses / partial momentum recovery; and the inherited fully-choked / both-NGVs-choked
  / no-bypass / one-`η_m` scope.

- **Two-shaft fuel metering** — **BUILT BY RUNG 43** (`docs/rung43-spec.md`,
  `TwoSpoolFuelTransient`, `docs/plans/rung43-anchor-two-shaft-fuel.md`). Rung 40 named this
  seam ("**fuel metering** on two shafts — rung 35's control, not carried over"). Rung 43
  carries it over: `ṁ_fuel` is metered and `Tt4` becomes an **OUTPUT** floating against the
  airflow **two lagging spools** can currently pump (the burner run **FORWARD** via rung 35's
  `_tt4_from_f`; still **one root in `m_L`**, still **no shaft balance**).
  **Rung 35's finding is INHERITED and the spec says so throughout** (the rung-40 register) —
  the TIT overshoot and its amplification of the surge excursion **re-measure unchanged**
  (`E_temp0`≈0.35 vs `E_surge0`=0.024–0.079, a **4.4–6.3×** ordering: TIT-limited before
  surge on these maps). **The new content is a question ONE shaft structurally cannot ask**:
  `f=ṁ_fuel/ṁ_air` is set at the **LP face**, but the `Tt4` it produces is metered back
  through the **HP-fed** NGV choke (`pt4=π_b·π_HPC·π_LPC·pt2`) — **the two spools sit at
  DIFFERENT points in the ONE overshoot loop**. With one shaft there is one clock and no
  question; with two there is the ratio `ρ=τ_L/τ_H`, and asking **which spool's lag governs
  the overshoot** the answer is **NEITHER**. By **channel isolation** (rung 41's move applied
  to the transient — march with one spool's speed *held*): **freezing EITHER spool makes the
  overshoot WORSE, 12/12** — both spools' motion *relieves* it, neither is a bystander — and
  the **share of the relief TRADES with `ρ`** (LP +51→+30→+16 K, HP +35→+38→+39 K).
  **Direction only**: the two do not sum, it is not a decomposition. The positive is
  **BOUNDED and structurally so** — `X=Tt4_peak−Tt4_target` rises **monotonically with `ρ`**
  (6/6, 3 shapes × 2 durations), but `ρ` multiplies **only the LP ODE**, so **`ρ→∞` IS the
  LP-frozen march**: `ρ`-independent **bit-for-bit**, with `X(ρ)` converging up onto it — the
  worst excursion a heavy LP spool can produce is computable **without marching it**.
  **THE NEGATIVE, stated plainly**: there is **no** effective clock ratio `r_eff=r/ρ^q`, and
  why it *appeared* to exist is the trap worth recording — **the referenced currencies are
  CIRCULAR**, the fitted exponent reading back whichever spool sits in the **denominator**
  (`E_temp_H` 0.05 on every shape, vs 0.35–0.45 for the spool-neutral `X` and 0.45–0.65
  for `E_temp_L` — strict on two shapes, a TIE on `press/flow`, so only the HP-vs-neutral
  separation is claimed; an earlier draft's strict three-way monotone was refuted by the
  shipped measurement). So `q≈0` was **never** evidence that "the HP clock governs"; **the data
  selected the instrument, not the answer it gave** — and even on `X` there is no collapse
  (a ~14–15% residual — the best exponent cuts the spread ~4.9× but points a real clock
  would place on one curve still differ by a seventh). **Four claims written, probed and
  withdrawn** (rung 40's convention, kept visible): the `√det` composite clock (a true rung-40
  identity, **not connected** to the overshoot — 0.35 is the *midpoint* of the two circular
  currencies); "`q=1` refuted in every currency" (**false** — on `X`, `q=0` fits worse);
  "irreducibly two-dimensional" (**overclaim** — only *power-law* collapses were tested); and,
  killed **before any code**, "fuel metering breaks rung 39's `(†)`" (a **category error** —
  `(†)` is a *steady* η-fixed-point artifact absent from the transient closure, refuted
  empirically by control-invariance). Reduce: **control-invariance** (machine zero via the
  forward-**burner** closure — a genuinely different code path) + `lp_disabled` **exact
  dispatch** ⇒ rung 35 bit-for-bit + `Tt4`-control **untouched** ⇒ rung 40 bit-for-bit.
  Along the way it **found and fixed a pre-existing rung-40 defect**: that class's reacting-gas
  2-D Newton chased an **absolute** `_EQ_TOL`=1e-12 below the reacting gas's own ~1e-10
  residual noise floor, so it raised at `Tt4`=1300/1400 while 1500/1450/1200 squeaked under
  (**non-monotone in `Tt4`** — a solver artifact, not physics); the fix is a **best-so-far
  acceptance AFTER the loop**, reached **only** by inputs that previously *raised*, so rungs
  40/41/42 are untouched **by construction** (44/44 witness it).
  **What rung 43 leaves open:** **reacting-gas** fuel control (rung 35's concession carried
  **verbatim** — the forward burner **asserts** against an equilibrium gas rather than
  mis-solving; the reacting reduce is the `Tt4`-control path); a true **`ṁ_fuel(t)` metering
  unit** with both ends free; **fuel metering + bleed** together (rung 42's valve still does
  not read the transient) and a bleed **schedule** during an accel; the **transient two-spool
  surge line** — so **no surge-survival claim** — and a **TIT redline**, which would turn the
  overshoot into a limit; rung 37's **internal clocks** on two shafts; the **subsonic/unchoked
  LP branch** in transient; a **real hardware/CFD map** (rung 32's standing concession,
  doubled).

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
  module-level `_illinois` is the shared fast bracketed root-finder. And rung 35's **fuel-control**
  methods on `SpoolTransient`: `_tt4_from_f` (the burner run FORWARD — `Tt4` from `f`, the exact inverse
  of `_solve_f`), `_close_compressor_fuel` (mirrors `_close_compressor` but with `ṁ_fuel` imposed and
  `Tt4` floating — the airflow lag), and `_instant_fuel`/`equilibrium_fuel`/`integrate_fuel`/
  `ramp_excursion_fuel`/`constant_speed_excursion_fuel` (the fuel-control instant, running line and the
  finding — `E_surge` vs rung 34's `E`, plus the new `E_temp` TIT-overshoot axis). The turbine/power/thrust
  tail of `_instant` was factored into a shared `_instant_tail` (bit-for-bit — both controls use it), and
  the equilibrium-`ν` bracket into `_find_equilibrium_nu` (shared by both controls, so `equilibrium` stays
  bit-for-bit rung 34). And rung 36's **surge line** — pure-diagnostic methods on `SpoolTransient`
  (`surge_margin`/`surge_margin_schedule` — the steady margin `SM_N` (constant-speed, the transient's
  currency) + `SM_flow` (CRS constant-flow) along the running line; `acceleration_binding` — the finding,
  `E0/SM_N` for a burst; `_pi_c_map` — `π_c` at an arbitrary map point, the same forward arithmetic
  `_close_compressor` uses, reproducing the shipped `π_c` bit-for-bit) reading the stall flow coefficient
  `ComponentMap.phi_surge` (added as a field, default 0 ⇒ off ⇒ rung 34/35 bit-for-bit; set via
  `with_phi_surge`; `is_flat` deliberately ignores it). The surge line never perturbs the running line or
  transient — it only measures. And rung 37's **`CombustorTransient`** (subclasses `SpoolTransient`): the
  two omitted internal-clock states. The **plenum** (volume-filling) — `plenum_ratio` (τ_fill/τ_spool, 0 ⇒
  off), `_compressor_from_backpressure` (invert `π_c(m)` for `m` on the stable branch — the third use of
  the map, via `_pic_of_m`/`_pic_band` and the `_PHI_FLOOR` past the η-island peak), `_plenum_state` (the
  decoupled instant: `ṁ_c ≠ ṁ_NGV`, honest two-mass-flow power), `_plenum_pt4_at`/`equilibrium_plenum` (the
  mass-balance/back-pressure reduce to rung 35), `plenum_frozen_peak` (the finding: peak == `E0`, + the
  split). The **metal** (heat-soak) — `soak_gain` (`G`, 0 ⇒ off), `soak_ratio` (τ_soak/τ_spool),
  `_close_compressor_fuel_soak` (rung 35's fuel closure with `Tt4_turb = Tt4_burner − G·(Tt4_burner−Tm)`),
  `_instant_soak` (adds `dTm/ds`, reuses `_instant_tail`), `equilibrium_soak` (`Q=0` at steady ⇒ rung 35),
  `soak_excursion`/`adiabatic_excursion` (the finding: `cold < hot-reslam < adiabatic` + the accel-lag).
  Both default off ⇒ the inherited `equilibrium_fuel`/`integrate_fuel` never read them (exact dispatch to
  rung 34/35). Diagnostic beside the cycle — the states are marched separately, never in the design run.
  And rung 38's **two-spool matching** — `build_two_spool_turbojet(…)` (factory) + `TwoSpoolEngine`
  (deliberately NOT a subclass of `Engine`; its own `run(flight, mdot)` closes BOTH shaft balances
  explicitly, stations `0→2→25→3→4→45→5→9`) + `TwoSpoolMatcher` (deliberately NOT a subclass of
  `OffDesignMatcher`; captures THREE throat areas `A4, A45, A8` from the design run).
  `_solve_choked_turbine(gas, Tt_in, f, A_in, A_out, pi_loss, eta)` is the shared `(★)` bisection,
  parameterized so ONE method serves both turbine choke-pins (`(★-HP)`: `A_in=A4, A_out=A45,
  pi_loss=1`; `(★-LP)`: `A_in=A45, A_out=A8, pi_loss=π_n`). `_cascade(wgas, Tt2, Tt4, f)` is the
  triangular Steps-1–4 solve, exposed as its own method (not inlined in `match()`'s loop)
  specifically so the finding is directly testable by mutating an instance attribute (e.g.
  `matcher.eta_hpc = X`) and re-calling `_cascade` at the SAME `(Tt2, Tt4, f)` — isolating the
  claim from the outer `(f, pt4)` fixed-point loop's own (separately disclosed) cross-talk.
  `lp_disabled=True` builds no two-spool state at all: `TwoSpoolMatcher.__init__` constructs and
  holds a plain `OffDesignMatcher`, and `.match()` forwards to it — the exact-dispatch reduce.
  And rung 39's **`TwoSpoolMapMatcher`** (subclasses `TwoSpoolMatcher`; rung 38's own
  `match`/`_cascade` are left **literally unchanged**, the rung-33 discipline, so the rung-38
  suite still witnesses them bit-for-bit): a `ComponentMap` per spool (`map_lp` carries the LPC
  island/speed lines **and** the LP turbine's `a_t`; `map_hp` likewise), per-FACE design
  references (`mcorr_lp_d` at station 2, `mcorr_hp_d` at station **25**), and `_cascade_map`
  — the **triangular** map cascade: geometry `(★-HP)/(★-LP)` → ENERGY (`Tt25`, `Tt3`; map-free)
  → **`_hp_eta_loop`** (a secant on `η_HPC` closing on the `π_LPC`-FREE HP-face corrected flow
  `(†)`, so it reads **no** LP quantity — the code-level guarantee behind the bit-for-bit closed
  leaf) → **`_lp_eta_loop`** (a secant on `η_LPC` whose LP-face flow `(‡)` **carries `π_HPC`** —
  the ONE arrow), wrapped in an outer turbine-efficiency loop that is **inert when `a_t == 0`**
  (it returns on its first pass, which is what keeps the leaf exact). `TwoSpoolMapResult` adds
  the four efficiencies, both corrected speeds `n_lp`/`n_hp`, both `N` ratios, and `slip`.
  `lp_disabled=True` dispatches to a `MapMatcher` (rung 32), which itself reduces to rung 31 on
  a flat map — one dispatch completing the whole ladder.
  And rung 40's **`TwoSpoolTransient`** (subclasses `TwoSpoolMapMatcher`; rung 39's own
  `match`/`_cascade_map` are again left **literally unchanged**, so the rung-39 suite still
  witnesses them bit-for-bit): the two-shaft transient. `_close(…)` is the **forward** closure —
  a **1-D root in `m_L`** (LPC map forward → `Tt25` → `n_H` → the corrected-flow transfer `m_H`
  → HPC map forward → `pt4` → `f` → the HPT-NGV choke imposes `ṁ` back) with **no shaft
  balance**, so `_instant(…)` returns BOTH power residuals `Φ_L, Φ_H` as the two ODE right-hand
  sides (`dν_H/ds=Φ_H`, `dν_L/ds=Φ_L/ρ`, `s=t/τ_H`). `equilibrium(…)` is a **2-D** damped Newton
  (rung 34's was a 1-D bracket) reproducing rung 39's `match`; `integrate(…)` RK4-marches the
  2-vector; `lead_threshold(…)` is `σ_crit`; `jacobian`/`eigenvalues`/**`oscillatory_band`**/
  **`damping_ratio_max`** carry the finding (the band is `None` exactly when `b·c≥0`).
  `lp_disabled=True` builds no two-shaft state at all — `__init__` holds a rung-34
  `SpoolTransient` and forwards to it (exact dispatch, the rung 38/39 contract one rung on).
  And rung 41's **two-spool surge line** — pure-diagnostic methods on `TwoSpoolMapMatcher`
  (`surge_margin`/`surge_margin_schedule` — `SM_L` and `SM_H` along the running line, rung 36's
  constant-speed currency **doubled**, each spool reading its own `map_*.phi_surge`;
  `running_line_map` — the two running lines in map coordinates, the object behind the SPLIT;
  `flow_coefficient_turn` — locates the `(★)` stationary point, returning `RAIL` rather than
  inventing a minimum when it lies outside the choked band; `critical_flow_turn_pi` — the closed
  form `γ_c^(γ_c/(γ_c−1))`; `_pi_c_spool` — rung 36's `_pi_c_map` parameterized by spool,
  reproducing the shipped `π` on each) plus `SpoolTransient.surge_margin_channels`, which
  freezes one running-line coordinate at a time to separate the **φ-walk** from the
  **speed-line-flattening** channel — the rung-41 correction of rung 36's stated mechanism,
  added to the rung-36 class without touching anything it reads. `phi_surge` is the **rung-36
  field reused** (no new knob) and is read ONLY by these methods, so rungs 39/40 are bit-for-bit.
  And rung 42's **interstage bleed** — **`TwoSpoolBleedMatcher`** (subclasses rung 39's
  `TwoSpoolMapMatcher`; rung 39's `match`/`_cascade_map`/`_lp_eta_loop` are again left
  **literally unchanged**, so the rung 39/40/41 suites still witness them bit-for-bit) +
  **`TwoSpoolBleedResult`**: a `bleed` fraction extracted at station 25. `_cascade_bleed`
  is rung 39's triangular cascade with exactly two differences — the LP shaft balance carries
  `(1−b)` (so `Tt25` falls) and the LP efficiency loop is `_lp_eta_loop_bleed`, whose `(‡-b)`
  picks up `1/(1−b)`. **`_hp_eta_loop` is called VERBATIM** — `(†)` is core flow on both sides
  and carries no `b`, so its body is `b`-free while its arguments are not; that reuse *is* the
  structural claim, in rung 39's leaf register. The forward rebuild books the extraction
  explicitly (`replace(s25, mdot=(1−b)·mdot)`), so every shipped conservation assert downstream
  still fires — on the core flow, which is what they should see. `bleed_trade(…)` opens the
  valve at a **FIXED `Tt4`** (the controlled comparison: the valve sets `b`, not the throttle)
  and returns both flow coefficients, both margins and the thrust/TSFC trade. `bleed == 0.0`
  forwards `match` to rung 39's verbatim — exact dispatch, bit-for-bit.
  And rung 43's **two-shaft fuel metering** — **`TwoSpoolFuelTransient`** (subclasses rung 40's
  `TwoSpoolTransient`; rung 40's `_close`/`equilibrium`/`integrate` are left **literally
  unchanged**, so the rung 40/41/42 suites still witness them bit-for-bit): rung 35's control on
  rung 40's plant. `_tt4_from_f` is rung 35's **forward burner** (`Tt4` from `f`, the exact
  inverse of the shipped `f`-solve) and **asserts** against an equilibrium gas rather than
  mis-solving; `_close_fuel(…)` is rung 40's `_close` with that burner forward — `f =
  ṁ_fuel/ṁ_air` from the **LP-face** airflow, `Tt4` an OUTPUT, still **one root in `m_L`** and
  still **no shaft balance** (so both power residuals stay the two ODE right-hand sides). Its
  one deliberate difference from rung 40 is the bracket: with `Tt4` floating, far past the root
  the mixture goes lean and the HP map leaves its physical branch, so it scans **up from the rich
  wall and takes the FIRST sign change** rather than trusting rung 40's global high wall (a
  consequence of the control change, **not** a rung-40 fix). `_instant_fuel` reaches rung 40's
  shared `_instant_tail` (factored for this rung exactly as rung 35 factored
  `SpoolTransient._instant_tail`), `equilibrium_fuel` is the 2-D damped Newton carrying the
  control-invariance reduce, `integrate_fuel` RK4-marches the 2-vector and takes a `freeze` in
  `{None,'lp','hp'}` — the **channel isolation** that is this rung's finding, exposed as
  `freeze_channels(…)`. `ramp_excursion_fuel(…)` reports the spool-neutral `X = Tt4_peak −
  Tt4_target` **plus** `E_temp_L`/`E_temp_H` (the running-line-referenced pair) kept **only so
  the currency circularity itself can be gated**, and the static `collapse_exponent(…)` is the
  guard for the withdrawn effective-clock claim. `lp_disabled=True` builds no two-shaft state —
  `__init__` holds a rung-34/35 `SpoolTransient` and forwards, exact dispatch. **Rung 40's
  `equilibrium` gained one thing here** (a fix, not a feature): a **best-so-far acceptance after
  the loop**, because its absolute `_EQ_TOL`=1e-12 sits below the reacting gas's ~1e-10 residual
  noise floor; the branch is reached **only** by inputs that previously raised, so every
  previously-converging point returns at the identical iteration with identical `(ν_L, ν_H)`.
- `main.py` — the design-point run: ideal-vs-real tables, the overlaid T–s diagram, and
  **one panel per rung** (each panel demonstrates that rung's load-bearing claim and
  states its honest scope).
- `tests/` — `test_stations.py` / `test_validation.py` (rung 1), `test_rung2.py`,
  `test_polytropic.py` (2b), `test_variable_cp.py` (3), `test_reacting.py` (4),
  `test_forkb.py` (5), then **`test_rungN.py` for N = 6…43**. Every rung file carries that
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
  Rung 35 **reduces** by **control-invariance** (the non-tautological gate): `equilibrium_fuel` at the fuel
  `ṁ_fuel=f_eq·ṁ_air,eq` of a Tt4-control point reproduces that point (`ν,π_c,τ_t,ṁ_air`; `Tt4_out==Tt4`)
  via the **forward-burner closure** — machine-zero at design, tight on a throttle sweep; two closures onto
  one point. Plus the Tt4-control path reduces to rung 32 **unchanged** (so rung 34 is bit-for-bit) and the
  design run is bit-for-bit rung 6. Its finding gates (fast gas — gas-independent): fuel control **enlarges**
  the surge excursion (`E_surge_fuel > E_Tt4`, gap **max at `r→0`**, **shrinking** toward `r→∞` — the
  correction of rung 34), **shape-robust in sign** across ≥3 surge maps; the **TIT overshoot** `E_temp>0`,
  monotone in `r`, its `r→0` limit the algebraic map property; both axes bounded by their `r→0` limits. And
  the **instant-level inverse** (the fuel↔`Tt4` analogue of rung 34 gate 6): the forward burner `Tt4(f)`
  inverts the burner `f`-solve to machine zero, and the fuel closure recovers a Tt4-instant off the running
  line. It **deliberately claims only the sign** of the correction and the **existence** of the overshoot
  (magnitudes disclaimed, rung-32 methodology), **no surge line / no TIT-redline number**, and **defers
  reacting-gas fuel control** (the finding is gas-independent).
  Rung 36 **reduces** by the surge line being a **pure diagnostic**: with `phi_surge=0` the surge methods
  add no cycle knob and the running line/transient are unperturbed (the rung 31–35 suites pass **unchanged**
  — the bit-for-bit witness), and `_pi_c_map(n,φ_op)` reproduces the shipped `equilibrium` `π_c` to machine
  zero (the non-tautological gate: the margin is measured on the running-line map itself, two code paths /
  one `π_c`). Its finding gates (fast gas — gas-independent): **THE HEADLINE — the `SM` schedule is thin at
  LOW power**, monotone-decreasing as `Tt4` falls, **sign-robust across ≥3 shapes × ≥3 imposed `φ_surge`**
  (and, weak corroboration only, the constant-flow definition — whose *magnitude* extrapolates absurdly and
  is not billed as independent; magnitude **disclaimed**); the **COMPOUNDING (confirmation + sharpening, NOT
  relocation)** — `E0/SM_N` rises monotonically as start power falls (both `E0↑` and `SM_N↓`, reinforcing),
  shape-robust, the low-power burst worst on **both** axes (rung 34's `E0` already largest there ⇒ `argmax`
  unchanged, **no relocation**); the **currency equivalence** `reaches_surge == (φ_step≤φ_surge)` at every
  point (`E0≥SM_N ⇔ φ_step≤φ_surge`, airtight);
  and — the **anti-overclaim gate** — with `E0` fixed, varying `φ_surge` **FLIPS** `reaches_surge`, so the
  test **asserts the crossing is floor-dependent** (rung 36 claims the trend, never the crossing, the exact
  discipline rung 32's warning demands). It **deliberately gates no surge-margin magnitude and no crossing
  location**, holds `φ_surge` constant in `n` (sign robust to a mild slope), and is **choked-branch only**.
  Rung 37 **reduces** by **exact dispatch**: with both clocks off (`plenum_ratio=0`, `soak_gain=0`) the
  inherited `equilibrium_fuel`/`integrate_fuel` never read them, so a `CombustorTransient` IS rung 34/35
  **bit-for-bit** (gate 1, and the rung 31–36 suites pass unchanged); and the two equilibria reproduce
  rung 35 via **independent** closures — `equilibrium_plenum` through the **back-pressure** invert
  (`≤1e-9`, mass balance closed), `equilibrium_soak` through `Q=0` at the fixed point (heat-soak is
  **transient-only** — never moves the running line). Its finding gates (fast gas — gas-independent): the
  **plenum** `r→0` peak **== rung-35 `E0`** to tolerance and **independent of `r_v`** (the CONFIRMATION),
  with the `ṁ_c≠ṁ_NGV` **split real** (>5%); the **heat-soak** ordering **`cold < hot-reslam < adiabatic`**
  shape- AND knob-robust (≥3 shapes × ≥2 `G` × ≥2 `r_m`), plus the **accel-time LAG** (cold slower than
  adiabatic, growing in `G`; hot reslam ≈ adiabatic-fast). It **deliberately claims only the SIGNS**
  (peak=`E0`, `cold<hot<adiabatic`, accel-lag) and the `ṁ_c≠ṁ_NGV` **existence** — every magnitude (the
  path-cushion, the surge protection, the lag) is **disclaimed** (rides on `r_v`/`G`/`r_m`), and the
  effects are exhibited **separately** (no combined 3-state claim).
  Rung 38 **reduces** by **exact dispatch**: `lp_disabled=True` builds a `TwoSpoolMatcher` that
  never constructs an LPC/LPT/`A45` at all — `__init__` holds a plain `OffDesignMatcher` and every
  `.match()` call is forwarded to it verbatim, so the fields compare `==` (not a converged limit;
  the rung 31 suite's own numbers, replayed through a wrapper). Its non-tautological gate (gate 2,
  since the reduce path never enters the two-spool cascade at all) is an INDEPENDENT bare-math CPG
  cascade — no `Gas`/`Component`/`TwoSpoolMatcher` calls, its own bisection — reproducing the shipped
  `(π_LPC, π_HPC, τ_HPT, τ_LPT)` to machine zero across a throttle sweep, plus the structural CPG
  fact that both `τ`'s are themselves `Tt4`-independent (`choked_mfp` being `Tt`-independent for
  CPG, doubled across two chained throat-pairs). Its finding gate (gate 3, on the exposed `_cascade`
  method at a FIXED `(Tt2, Tt4, f)` so the outer loop's own cross-talk cannot confound it): `η_HPC`
  perturbed leaves `π_LPC` **bit-for-bit unchanged** and `η_LPC` perturbed leaves `π_HPC`
  **bit-for-bit unchanged** (each compressor's own efficiency is a terminal leaf), while `η_HPT` and
  `η_LPT` perturbed **move BOTH** ratios (the asserted CONTRAST, so the gate cannot be misread as
  "the spools don't talk"). A scope-guard gate asserts nozzle-unchoke **raises** the documented
  "OUT OF SCOPE" error rather than silently mis-solving. It **deliberately claims only**: no 2×2
  solve between the compressor ratios, the compressor-efficiency-leaf property, and the geometric
  pinning of both turbine ratios — **not** "the two spools are independent" (an initial framing this
  rung's own spec caught and corrected before shipping), **not** a physical twin-spool-engine claim
  (the triangular result is a no-compressor-map artifact, disclosed as such), and **not** any
  nozzle-unchoke behavior (flagged, not solved).
  Rung 39 **reduces three ways**: FLAT maps ⇒ rung 38 `TwoSpoolMatcher` **bit-for-bit** (`==` on
  `π_LPC`, `π_HPC`, `τ_HPT`, `τ_LPT`, `ṁ`, thrust, on the **reacting** gas across a throttle
  sweep — targeted, not promised, and it landed), and `lp_disabled=True` ⇒ rung 32 `MapMatcher`
  (shaped) **and** rung 31 `OffDesignMatcher` (flat) by **exact dispatch**, completing the
  ladder. Its non-tautological gate (gate 3, since the flat reduce holds every `η` at design and
  the `lp_disabled` reduce never enters the two-spool path) is an INDEPENDENT bare-math CPG
  two-spool **MAP** cascade — no `Gas`/`Component`/`ComponentMap`/`TwoSpoolMapMatcher`, its own
  bisections, its own speed-line inversions, and efficiency fixed points by **damped
  substitution** rather than the shipped secant — reproducing `(π_LPC, π_HPC, η_LPC, η_HPC,
  n_L, n_H)` across a throttle sweep. Its finding gates: **gate 4 (THE ASYMMETRY**, on
  `_cascade_map` at a FIXED `(Tt2, pt2, Tt4, f)`, `a_t=0`, ≥3 shape pairs × 3 throttles, CPG
  **and** reacting) — `η_LPC` leaves `π_HPC` **bit-for-bit unchanged** (`==`) while `η_HPC`
  **moves** `π_LPC` with a **negative** sign, plus the asserted CONTRAST that `η_HPT`/`η_LPT`
  move **both** (so it cannot be misread as "the spools don't talk"); **gate 5** — a turbine map
  **does** open the closed leaf but **>50×** weaker (a deliberately loose bound; measured
  119×–548×, **ratio disclaimed**); **gate 6** — `slip ≡ 1` on CPG + flat maps at every throttle
  **and under a forced `f`** (the `(1+f)`-cancellation, so the identity is structural, not a
  design-point coincidence); **gate 7** — the rung-31-gate-5 **mirror** (the same flat maps drift
  on the variable-`cp` gases) asserted beside the **dominance** (on the same CPG gas the map
  channel exceeds the gas channel); **gate 8** — the slip is **monotone-decreasing** across ≥3
  shape pairs (**sign only**; magnitude disclaimed). It **deliberately claims no** magnitude
  (arrow strength, back-arrow ratio, slip depth all ride on the representative shapes), does
  **not** claim the slip DIRECTION is structural (only B1's identity is), and makes **no
  two-spool surge-margin claim**.
  Rung 40 **reduces two ways**: the **2-D** equilibrium (`Φ_L=Φ_H=0`, damped Newton from the
  design start) reproduces rung 39's `TwoSpoolMapMatcher.match` to **≤1e-12** on CPG **and** the
  reacting gas — through the **forward closure only**, never calling that matcher, so the reduce
  is **non-circular** (rung 34's discipline); and `lp_disabled=True` **exact dispatch** ⇒ rung 34
  `SpoolTransient` **bit-for-bit** (`==`), no two-shaft state built. Its **non-tautological gate
  3** is an INDEPENDENT bare-math CPG two-shaft closure (no `Gas`/`Component`/`ComponentMap`/
  `TwoSpoolTransient`; own CPG thermodynamics, own bisections, own forward speed lines, own 2-D
  Newton) reproducing `(ν_L, ν_H, π_LPC, π_HPC)` **and `σ_crit` ON SHAPED MAPS** — the shaped
  value is the load-bearing part, since reproducing the `≡1` identity would only re-check the
  reduce. Finding gates: **gate 4** — the `σ_crit` identity (labelled **INHERITED** from rung 39
  B1) + its two channels with the map the larger + the **REFUTATION** that the shift *direction*
  is shape-dependent (`lp-only`<1<`hp-only`); **gate 5** — STABILITY, the measured signs
  `a<0, d<0, ad>bc` at every sampled point hence both eigenvalues negative at every `ρ`∈[0.05,100]
  (the `ρ`-freeness is asserted as algebra *on top of* the measured signs, so the composite is
  never billed "provable"); **gate 6** — THE COMPLEX MODE, `b·c<0` for **every** shaped-LP pair
  and `b·c≥0` for **every** flat-LP pair (incl. `hp-only`, the discriminator), the returned band
  genuinely bracketing the discriminant's sign change — **existence + sign + mechanism only**, the
  band LOCATION and `|Im/Re|` deliberately **NOT** gated; **gate 7** — SCOPE, asserted as a
  *deliberate NON-convergence* (`|ρ*/σ_crit−1|>0.2`) so the withdrawn "σ_crit predicts the ramp"
  claim cannot silently creep back. It **deliberately claims no** magnitude, does **not** claim
  the two shafts are dynamically independent (that negative is *not* the headline), scopes the
  oscillation claim to **INTER-SPOOL** (rung 37's shaft+metal Jacobian unaudited), and makes **no
  two-spool surge-margin claim**.
  Rung 41 **reduces** by the surge line being a **pure diagnostic on two spools**: `phi_surge`
  (the rung-36 field, reused) is read only by the rung-41 surge methods, so a floor-carrying map
  leaves rung 39's `match` and rung 40's `equilibrium` **bit-for-bit** (`==`, four shape pairs)
  and the rung 31–40 suites pass **unchanged** (72/72); `is_flat` still ignores it. Its
  **non-tautological** gates are (2) `_pi_c_spool` at the operating `(n,φ)` reproducing the
  shipped `π` on **each** spool to ≤1e-9 — the margin is measured on the very forward map that
  sets that spool's running line — and (4) **THE SHIELDING, quantified**: the closed-form
  sensitivities `s_H` (containing **no LP quantity**) and `s_L` (containing the **product**) each
  match the measured value to <0.05 while **dropping `π_HPC` from `s_L`** misses by >0.5 (a >10×
  separation, wrong sign), plus 4b recording the **withdrawn** "HP collapses / LP doesn't"
  framing as its true, weaker statement (flight enters only through `Tt2`; `x_L` and `x_H` are in
  bijection, so *both* collapse). Finding gates: (3) the **SPLIT** — `φ_L` falls >3× as far as
  `φ_H` and `φ_L<φ_H` at every part-power point, four shapes; (5) **`(★)`** within 1% across
  `η_HPC`/`η_HPT`/`η_LPC`/`γ_t`/`cp_t`/three splits/two flight conditions **while `Tt4*` moves
  >1.4×**, tracking `γ_c` over 1.30–1.45, plus the **KILL TEST** (raising `hPR` drives `f`→0 and
  the residual monotonically below 1e-4 — the whole residual is the fuel fraction); (6) the
  **ORDERING** at *matched* shapes + a *common* floor (`SM_L<SM_H` everywhere, `SM_L/SM_H`
  monotone-falling and at least halving, 3 shapes × 3 floors — the **ratio** is the gated
  content, the ordering's *level* being partly the `π_LPC`=3 vs `π_HPC`=6 design split, named
  as such); (7) the **DIVERGENCE** asserted
  deliberately (`φ_H` turns **up** while `SM_H` keeps **falling**, so `(★)` can never be misread
  as a margin extremum) **and** the rung-36 correction (its `SM_N` still monotone past the turn —
  **verdict survives, no rung-36 test changes** — with both channels non-negligible). It
  **deliberately gates no** margin magnitude, no crossing, no `Tt4` turn location, no ordering at
  unmatched shapes/floors, and **no slip-protection claim** (the rigid-shaft counterfactual is
  not run); steady + choked-branch only.
  Rung 42 **reduces** by **exact dispatch**: `bleed=0.0` forwards `match` to rung 39's verbatim,
  so a bleed matcher with the valve shut is `TwoSpoolMapMatcher` **bit-for-bit** (`==` on
  `π_LPC`, `π_HPC`, both `φ`, both `η`, `ṁ`, thrust) across four shape pairs × four throttles on
  the fast gas **and** on the **reacting** gas — plus the rung 31–41 suites passing **unchanged**
  (84/84) as the standing witness that rung 39's cascade was not touched. Its **non-tautological**
  gates are (2) **THE ASYMMETRY** — `x_L` **exactly** bleed-invariant (`==`) with `φ_L` moving
  >5%, against the bled HP point landing on the `b=0` running line at the **same `x_H`** to
  <5e-4 in `φ_H` (a >100× contrast, asserted as a ratio), plus the mass-extraction identity
  `ṁ_core == (1−b)·ṁ_air`; and (3) **PERTURBATION-INDEPENDENCE** — valve-derived `s_H` equals
  rung 41's **throttle**-derived closed form to <0.01 absolute at every point of the CPG+flat
  band, with a **guard against spurious exactness** (`worst > 1e-6`) so the gate cannot pass by
  the two paths being secretly the same. Finding gates: (4) **`π*` A THIRD TIME** — `dφ_H/db`
  **changes sign** and the crossing **brackets** `π*`, asserted beside the CONTRAST that the LP
  response never reverses (existence + sign + bracket only; the exact crossing **disclaimed**);
  (5) **SELF-TARGETING in φ-SPACE** — `Δφ_L` near-constant (spread <10%) while `Δφ_H` falls ≥5×,
  hence the fraction of `(φ_op−φ_surge)` closed **monotone rising** on LP and **monotone falling**
  on HP, 2 shapes × 3 floors — deliberately gated in φ-space, with the confounded relative-`SM`
  version **not** gated (the rung-41 lesson, enforced); (6) the **TRADE + ENVELOPE** (thrust and
  TSFC monotone in `b`, the penalty **growing** with throttle-down, the choked band shrinking);
  (7) **THE REFUTED HYPOTHESIS kept visible** — `dφ_H/db > 0` at design across two gases × four
  shapes, so "bleed penalises the HP spool" is asserted **false** rather than quietly dropped.
  It carries **no independent bare-math gate** — a deliberate break in the rung-38/39/40
  streak, with the reason on the record (the HP side is anchored transitively by gates 2/3;
  the LP magnitudes are disclaimed and its load-bearing claim is a shape plus an input
  identity). It **deliberately gates no** magnitude (all ride on `b`, the maps and the two imposed floors),
  **no bleed schedule**, **no variable stator** (bleed moves `φ_op`, never `φ_surge`), and **no
  surge-SURVIVAL claim** (`E0` vs `SM_N` needs the deferred transient); steady + choked-branch only.
  Rung 43 **reduces three ways**, and the first is also its **non-tautological** gate:
  **CONTROL-INVARIANCE** — feeding `ṁ_fuel = f_eq·ṁ_air,eq` of a rung-40 `Tt4`-control point to
  `equilibrium_fuel` reproduces that point (`ν_L`, `ν_H`, `Tt4`, `π_LPC`, `π_HPC` to **machine
  zero**, measured ≤6e-15, with the residuals themselves asserted <1e-9) through the **forward-
  BURNER** closure, a genuinely different code path that never calls the `Tt4` path. That gate is
  also the **empirical death of the framing this rung was proposed with** ("fuel metering breaks
  rung 39's `(†)`"): two controls landing on one manifold cannot differ in their coupling. Then
  `lp_disabled` **exact dispatch** ⇒ rung 35's `SpoolTransient` fuel path **bit-for-bit** (`==`),
  and the `Tt4`-control path **untouched** ⇒ rung 40 **bit-for-bit** (`==`) — the rung 31–42
  suites passing **unchanged** (44/44 across 40–43) is the standing witness. A fourth reduce is
  **dynamical**: a fuel ramp marched long **settles onto** the target equilibrium. Finding gates
  (fast CPG gas — gas-independent dynamics): **gate 5, THE MECHANISM** — freezing **either** spool
  **worsens** the overshoot at every sampled `(ρ, r)`, asserted **beside the CONTRAST** that the
  *share* trades with `ρ` (the LP channel weakens, the HP strengthens), so it cannot be misread as
  "one spool does the work" — **sign/existence only, no calibrated split** (`ΔLP`+`ΔHP` sums to
  nothing); **gate 6, THE CEILING** — the LP-frozen march is `ρ`-independent **BIT-FOR-BIT** (`==`,
  `ρ` multiplying only the LP ODE) and `X(ρ)` rises **toward** it; **gate 7** — `ρ`-monotonicity of
  `X` across ≥3 shape pairs × 2 ramp durations (**sign only**); **gate 8, INHERITED** — rung 35's
  TIT-limited-before-surge ordering re-measured on two shafts (>4×; the multiple **disclosed**,
  4.4–6.3×, not tuned) together with the `r→0` step being **exactly `ρ`-free** (both spools frozen,
  so no clock can enter); **gate 9, THE WITHDRAWN CLAIMS asserted as such** (rung 40's gate-7 move)
  — the best-fit exponent **differs across currencies** (the HP-referenced one sits far below the
  spool-neutral one, the circularity) **and** no currency collapses below ~10%, so "the overshoot
  collapses on an effective clock" cannot silently creep back. It **deliberately gates no**
  magnitude (all ride on `ρ` — a disclaimed clock group, doubled — the two representative maps and
  the fuel step), **no exponent** beyond `0 < q*(X) < 1`, **no calibrated channel split**, **no
  surge-SURVIVAL claim** and **no TIT redline**; and it carries **no independent bare-math gate** —
  the second deliberate break in the rung-38/39/40 streak, reason on the record (unlike those
  rungs, this reduce **does** enter the new code and lands on rung 40's own bare-math-anchored
  steady manifold, so the closure is tied down **transitively**). Reacting-gas fuel control is
  **refused** by an assert rather than mis-solved (rung 35's concession verbatim), and that refusal
  is itself gated beside a witness that the reacting `Tt4`-control path still works.
- `docs/rungN-spec.md` — the derivation, assumptions, concessions and gates for rung N.
  `docs/plans/rungN-anchor-*.md` — that rung's verified anchor data.

## Commands
- Run the model:  `python main.py`
- Run tests (fast, routine):  `pytest`  — the FAST subset (~2.5 min). The inherently-expensive
  FINDING / robustness gates (the mixing-PDF per-pocket sweeps of rungs 16/20–24, the transient
  marches) are tagged `slow` and **deselected** — BUT the bit-for-bit **reduce spine**
  (`test_reduce_*`, `test_cycle_untouched_*`, `*_bit_for_bit`) is kept in the fast run, so routine
  `pytest` still guards "each rung reduces to its predecessor, exactly and by test."
- Run tests (full, every gate):  `pytest --runslow`  — all 420 tests (~10–15 min). **Use this at
  commit / session-end / CI** — the fast subset is for quick iteration, not for signing off a rung.
- Only the slow gates:  `pytest -m slow`   ·   One rung by hand:  `python tests/test_rung2.py`
- Install deps:   `pip install -r requirements.txt`  (matplotlib + pytest + pytest-xdist)

**Test-suite speed policy** (nothing about the gates changed — the full run is bit-for-bit what it
always was; only *which run when* and *scheduling*): the suite is PARALLEL by default
(`-n auto`, `pytest.ini`) and `conftest.py` (a) tags a test `slow` from its learned per-test
duration (≥ 8 s, seeded so a cold checkout is already fast; `.pytest_cache` refines it) and
deselects those unless `--runslow` — **except the reduce-spine gates, which are never slow-tagged**
(`_is_spine`), so the every-time run always checks the invariant; and (b) reorders collection
LONGEST-FIRST (interleaved so xdist's 2-per-worker seed can't stack the two longest poles) — LPT
scheduling so the full-run wall clock approaches the single longest test (rung 24's ~6-min monotone
scan) instead of a stacked tail. Was 49 min serial → now ~2.5 min routine / ~10–15 min full. The rung
gates themselves are untouched (no test file edited; the derive/reduce spine is pristine).

## Stack
Python (standard library) + matplotlib for the plot. No other dependencies.
