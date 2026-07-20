# Turbojet Cycle Simulator

A station-by-station model of a single-spool turbojet (Brayton cycle). It takes
flight + design conditions and produces the gas state at every station, the
thrust, the efficiencies, and a T–s diagram.

**The deliverable is understanding, not the tool.** The code is the medium that
forces every thermodynamic assumption into the open. Optimize the work for
teaching, not for features or polish.

## The rungs

The model is built in cumulative **rungs** — each adds one physical effect and is
anchored to a published case. All rungs are live; the current scope is **rung 29**.

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

**The invariant that spans rungs 7–29: they are all pure diagnostics.** NO/N never
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

**Current scope (rung 29).** The **cycle solve** is a thermally-perfect, reacting,
dissociation-equilibrium gas (`Gas.reacting_equilibrium()`) through ideal + real
components (isentropic `η_c/η_t` **or** polytropic `e_c/e_t`, mutually exclusive;
`π_d/π_b/π_n`, `η_b`, `η_m`; dual cold/hot gas; specified exit pressure). The burner
root-finds `f` over the scale-B absolute balance, re-solving equilibrium each trial,
then freezes the station-4 mixture through turbine + nozzle. Fork A/B
(`Gas.reacting()` / `reacting_forkb()`) and frozen-products `Gas.thermally_perfect()`
are kept alongside. Everything from rung 7 up is a diagnostic *beside* the cycle.

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
  `_coupled_no_march`, and the rung-29 `_work_limited_expand`, …).
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
  `test_forkb.py` (5), then **`test_rungN.py` for N = 6…29**. Every rung file carries that
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
- `docs/rungN-spec.md` — the derivation, assumptions, concessions and gates for rung N.
  `docs/plans/rungN-anchor-*.md` — that rung's verified anchor data.

## Commands
- Run the model:  `python main.py`
- Run tests:      `pytest`  (or `python tests/test_rung2.py`, etc.)
- Install deps:   `pip install -r requirements.txt`  (matplotlib only)

## Stack
Python (standard library) + matplotlib for the plot. No other dependencies.
