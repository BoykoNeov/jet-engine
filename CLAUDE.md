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
**rung 23**. Each rung's full derivation, assumptions, and verification gates live
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
| 15 | **PDF through the finite quench** — rung-13's resolved β-PDF **carried through** the rung-10/12 dwell chain, so the two mixing mechanisms **combine**. A `QuenchPDF(S,…)` config (rides on `mixing`, mutually exclusive with `pdf`/`unmixedness`) gives `ei_no_pdf_quench = ei_no_quenched + D(u)·⟨EI_bell⟩(g)`: term 1 the rung-11 mean-field **bulk quench** (the **finite floor**), term 2 the rung-13 β-PDF integral over the ideal bell scaled by an off-optimum-growing dwell `D(u)=τ_res(1+b_u·u)/τ`. The **≈0 rung-13 optimum floor becomes finite bulk NO** (at `C_opt`, `g→0` ⇒ `=ei_no_quenched`), the min stays pinned **AT `C_opt`** (`(H/S)²` shift), and the rung-13 **descending** far flank **CLIMBS** again (dwell restored) — a non-monotone over-flank showing **both** parents. The nonlinear bell keeps the **stoich-mean sign reversal** a dwell-only closure can't. `pdf_quench=None` = exact rung 13. | `docs/rung15-spec.md` |
| 16 | **PDF through the quench, PER POCKET** — retires rung-15's one acknowledged **linearisation** (term 2 = `D(u)·⟨EI_bell⟩`, a **constant-T** bell × a **scalar** dwell ratio). A `PocketQuenchPDF(S,…)` config (rides on `mixing`, ≤1-of-four with `pdf_quench`/`pdf`/`unmixedness`) carries **each rich-of-mean β-PDF pocket through its OWN finite quench** (`_quench_no` at `τ_core(C)`), so the dwell acts **inside** the cooling chemistry: `ei_no_pocket_quench = ei_no_quenched + ⟨EI_pocket_quench(ξ;τ_core)⟩_g`. Because a lingering pocket **cools**, term 2 is **SUBLINEAR** in `τ_core` (far-flank ≈×1.3 vs rung-15's LINEAR ×1.51 = the dwell ratio (grid-dependent)), which **ERODES** rung-15's over-penetration far flank ~18–32% into **near-degeneracy** with the `C_opt` notch (which **survives** — the composition excess still →0 at `C_opt`). The honest headline is the **cooling-limited erosion**, NOT a relocated optimum: the global-min **location is NOT claimed** (it flips sign across the β-PDF quadrature ~5%, the φ>2 tail, and the `C_e` regime 2%→21%). Clamp **dormant** (`max_a<1`) — the difference is cooling, not super-eq rollover. `pocket_quench=None` = exact rung 15. | `docs/rung16-spec.md` |
| 17 | **Exhaust-NO clamp through the mixing-fidelity ladder** — a rung-14 corollary *from the rich side*. `Gas.exhaust_no_clamp(…)` carries the exhaust NO from **three** combustor-mixing-fidelity models through the **same** rung-14 nozzle collapse to `T9` and reads the dropped-clamp margin `a=[NO]/[NO]_e(T9)`: **MIXED-OUT** (rung 8, `x_no_mix`) reads **DORMANT** (`a≈0.02`) at the RICH `φ_p=1.5` primary — mixing-out **HIDES** the super-eq NO; **BULK QUENCH** (rung 11, `x_no_quenched`) **FIRES** (`a≈3.4`, the re-making); **PER-POCKET** (rung 16) **FIRES harder** (`a≈13.6`, segregation lifts the mean). The **load-bearing** claim splits: the **ORDERING** `a_mixed≤a_bulk≤a_pocket` is **STRUCTURAL** (the quench only *adds* NO; the per-pocket excess is additive) and `a_mixed<1` is robust — *these* are certified; the **FIRING** (`a_bulk,a_pocket>1`) holds **in-band** but is **NOT universal** (a fast quench `J→∞` drives `a_bulk→a_mixed<1`, the rung-10 `τ_q→0` reduce). The **identity** `a_pocket/a_bulk ≡` rung-16's station-4 gap is **algebra**, not a test (the nozzle denominator cancels — a *witnessed no-op*, stated not gated). Every firing magnitude *and the gap* ride on un-pinned scales (`C_e`, `τ_res`, `H`, `J`; the gap moves ~23% over `C_e 0.15→0.20`). **Contrast rung 14** (which fires on the `φ_p=1.0` mixed-out number, `a≈250` — the *zoned-vs-unzoned* axis): rung 17 is the *mixing-fidelity* axis, the same dropped-clamp lesson from the rich side where the mixed-out shortcut is **unconservative**. Clamp **dormant at station 4** (`max_a<1` — super-eq is a *nozzle* effect). No new chemistry/config: it composes rung-8/11/16 + rung-14 outputs. | `docs/rung17-spec.md` |
| 18 | **Transported-variance closure** — the deferred "transported PDF" seam, done as its **honest LIMIT**. A `TransportedPDF(S,…)` config (rides on `mixing`, ≤1-of-**five** with `unmixedness`/`pdf`/`pdf_quench`/`pocket_quench`) replaces rung-13's imposed kink `g(C)=k_g·|ln(C/C_opt)|` with the **residual of a variance DECAY ODE** `dg/dt=−C_φ·ω(C)·g` (`_transport_variance`, backward-Euler, `C_φ≈2` anchored) from a **DERIVED two-stream ceiling** `g_ceiling=(ξ_p−ξ̄)/(1−ξ̄)` (`_two_stream_ceiling`, from `φ_p` — NOT a knob), fed through the **rung-13 ideal bell** (`_pdf_mean_ei`). **The load-bearing result is NEGATIVE**: a 0-D transport **CANNOT derive** the `C_opt` optimum — a genuine ODE with any **mean-field** `ω(J)` (or `τ_q(J)∝1/√J`) gives a **monotone/flat** `g(J)`, no interior optimum; the optimum appears **only** once `ω` is given a **SPATIAL** coverage `ω(C=(S/H)√J)` peaked at `C_opt` — i.e. once the jet **spacing `S`** is injected (rung-11's *"mean-field ⇒ no mixing optimum"* made literal). So the coverage `ω(C)` is an **explicitly imposed** spatial closure (the honest successor of the kink), and a **spatial/CFD PDF** stays the deferred ceiling. What transport **legitimately adds** (certified): the **DERIVED ceiling** (exposes `g_max=0.3` as **4.4× too large**), the **RESIDUAL floor** `g(C_opt)=g_ceiling·exp(−Da_opt)>0` (perfect mixing never reached ⇒ the emissions optimum is **elevated** off the well-mixed value, not the kink's touch-the-floor ≈0), and **KINK-is-non-generic** (the imposed corner has one-sided slopes `±k_g/C_opt`; any analytic mixing rate rounds it — the EIbell ratio one step off `J_opt`: kink **×5.8e4** vs transported **×1.05** — the *sharpness* was the artifact, not the location). Reduce: `Da_opt→∞` (perfect mixing) recovers the kinked notch (the well-mixed point value); `g_ceiling→0` = point value. `transported=None` = exact rung 17. Shape-only on `g` (the dwell `τ_core` stays the rung-16 kink — the "also transport the dwell" seam declined). | `docs/rung18-spec.md` |
| 19 | **Super-equilibrium O & prompt NO** — lifting the equilibrium-O **lower bound**. Every NO number since rung 7 read the rung-6 **equilibrium** `[O]` into the Zeldovich rate, so it is a **lower bound**. Two knobs on `thermal_nox`/`zoned_nox` (`super_eq_o=False`, `prompt=None`; both off ⇒ **bit-for-bit** the prior rung) lift it, and the load-bearing result is that **both contradict the naive "the rich primary explodes" intuition**, from opposite directions. **(1) super-eq O** — the Westenberg partial-equilibrium O closure (adds the 3-body `O+O+M⇌O₂+M`) sits **above** equilibrium O; the two share `[O₂]^0.5`, so their ratio is **dimensionless & T-only** `m(T)=(C2/C1)·T·exp((θ1−θ2)/T)∈[1.16,1.50]` (`_super_eq_o_multiplier`). We lift **our own** `comp["O"]` by `m(T)` inside the rung-7 integrator (`_thermal_no(…, o_multiplier=m)`); `m≡1` ⇒ rung 7. The one thing rung 19 **computes** cleanly (only `C2/C1, θ1−θ2` — no absolute-magnitude risk; the equilibrium-O **units gate** cross-validates the pool to ~5%). Lesson: it is **T-driven, not rich-driven** — φ-independent, **weakest** in the O₂-starved rich primary where thermal NO already dies. **(2) prompt NO** — an **imposed** De Soete (1975) φ-bump `EI_prompt=scale·max(f(φ,n),0)·exp(−Ea/RuT)` (`PromptNO` config; the burnt-pool `[O₂]^a·[FUEL]` **dropped** — it double-counts O₂ depletion & flips the shape lean-peaking). The rich-peak lives **only** in De Soete's fitted `f(φ)`. Lesson: prompt **survives where thermal dies** — prompt/thermal grows **monotonically rich** (0.24→455), and prompt is **~27× less T-sensitive** (single vs the double `k1f·[O]_eq` Arrhenius exp). Both stay **trace** (summed guard `x_NO<0.02`) ⇒ cycle bit-for-bit rung 6. **Two honest concessions** (rung-18-flavored, stated loudly): the prompt **magnitude is imposed** (a 0-D pool has no flame structure — only the φ-shape + the directional ratio are certified), and the super-eq **ratio is semi-empirical** (a full-equilibrium pool cannot self-yield super-eq O). φ>1.6 is De Soete extrapolation (clamped, flagged); constants **transcribed** (image-locked sources), not digit-verified. Lifts only the **primary** diagnostic (`ei_no`/`x_no_mix`/`ei_no_total`); threading through the quench is a **deferred seam**. `super_eq_o=False`+`prompt=None` ⇒ exact prior path. | `docs/rung19-spec.md` |
| 20 | **Super-equilibrium O *through the quench*** — threading the rung-19 lower-bound lift **into** the finite-quench re-making, closing the seam where `ei_no_quenched`/`ei_no_pocket_quench` and the rung-17 clamp `a` still rode on **equilibrium O**. A `super_eq_o=` flag on `zoned_nox`/`exhaust_no_clamp`'s quench path lifts `[O]` by `m(T)` **inside** the `_quench_no` cooling chemistry (floored at the flame band, `m(max(T,1500 K))`; `super_eq_o=False` ⇒ **bit-for-bit** prior). The load-bearing result **INVERTS** the naive "strongest on the cooling pocket": the Zeldovich re-making peaks at the **hottest** stoich crossing (`T_peak≈2448 K`) where `m(T)` is at its **minimum** (`≈1.14`), and the cool tail (large `m`) makes negligible NO (and there `a>1` ⇒ destruction), so the effective lift is **modest & peak-concentrated** (`≈m(T_peak)`, **×1.17** on the bulk / ×1.16 per-pocket) — even **smaller** than the rung-19 primary lift (**×1.28**; the quench samples a *hotter* peak than the flame). The **certified spine**: the rung-17 `a` margins **RISE** because the **numerator** (kinetic re-made NO) lifts while the **denominator** `x_no_e(T9)=Kp_NO·√(x_N2·x_O2)` — a **thermodynamic** ceiling untouched by the O-atom closure — stays **bit-identical** (`a_bulk 3.27→3.83`, `a_pocket 11.06→12.87`); the rung-17 ordering + `a_mixed<1` survive. Clamp **still dormant at station 4** (`max_a 0.72→0.81<1`): super-eq O speeds *formation*, not the `[NO]_e` collapse — the burner-clamp lever is a **slow freeze** (a separate seam). **Prompt** rides the quench as an **invariant** per-kg-fuel EI (`ei_no_quenched_total`), kept **out** of `a` (imposed magnitude ⇒ no false precision through destruction chemistry). Scope **narrow, per-field consistent**: only the `_quench_no`-based fields (bulk/core/per-pocket/clamp) lift; the **ideal-bell PDF integrals** (rung 13 `ei_no_pdf` / rung 15 term 2 / rung 18) **deliberately** stay eq-O lower bounds — **forbidden** to combine with `super_eq_o` (`pdf_quench` would else be a half-lifted **hybrid**). Pure diagnostic ⇒ cycle bit-for-bit rung 6. `super_eq_o=False` ⇒ exact prior rung. | `docs/rung20-spec.md` |
| 21 | **Super-equilibrium O through the IDEAL-BELL PDF integrals** — discharging the **last eq-O seam** rung 20 deliberately left open. Rung 20 lifted everything through `_quench_no` but kept the three **ideal-bell composition integrals** (`ei_no_pdf` rung 13, `ei_no_pdf_quench` **term 2** rung 15, `ei_no_transported` rung 18) on **equilibrium O** and **forbade** combining them with `super_eq_o` (because `ei_no_pdf_quench = term1+term2` would be a half-lifted **hybrid** — lifted term1 + eq-O term2). Rung 21 threads the **same** Westenberg `m(T)` through the ideal bell (`_bell_interpolator`/`_pdf_mean_ei` gain `super_eq_o=`; both the built bell AND the `g→0` delta short-circuit), so **both** `pdf_quench` terms lift and the hybrid **dissolves** — the rung-20 forbid guard is **removed** (`super_eq_o` now combines with **every** closure). The load-bearing result is the rung-20 **inversion generalized to composition variance**: the ideal bell EI(φ) is sharply **peaked near stoich** (φ≈0.97, `T_p≈2424 K`) where `m(T)` is at its **minimum** (`≈1.12`); the β-PDF integral is **EI-weighted** onto that peak, so the effective lift is **≈×1.15** — **below** the primary (**×1.28**), far below the deep-lean **point value** (**×1.91**, cool flame ⇒ big `m` but ≈0 EI), and it even **DECREASES with segregation** (×1.17→×1.14 as `g`: 0.005→0.30 — more variance ⇒ more stoich-peak weight ⇒ smaller fractional lift). The `pdf_quench` **composite** (×1.156) sits **between** term1 (bulk ×1.17) and term2 (bell ×1.14) — the measured proof both carry `m(T)`. A **shape-preserving CONSISTENCY lift**: the optimum **LOCATION** (pinned AT `C_opt`, where `g→0`), the `(H/S)²` shift and the stoich-mean **sign reversal** are **unmoved** (both eq-O and super-eq-O J-sweeps minimise AT `J_opt`); only the magnitude lifts. **Carried:** the super-eq **ratio** stays semi-empirical (rung 19); **prompt** stays a **primary-only** invariant EI (not injected into the composition integrals). No new species/config; a pure diagnostic ⇒ cycle bit-for-bit rung 6. `super_eq_o=False` ⇒ **bit-for-bit** rungs 13/15/18. | `docs/rung21-spec.md` |
| 22 | **Resolved cross-plane / spatial PDF** — the **INVERSION of rung 18** (the deferred spatial/CFD-PDF seam rungs 17 & 18 both named). Rung 18's load-bearing **negative** result was that a **0-D** variance transport **cannot derive** the Holdeman `C_opt` optimum (mean-field `ω(J)` ⇒ **monotone** `g(J)`), so it had to **IMPOSE** the coverage `ω(C)` — the jet **spacing `S`** by hand. Rung 22 **resolves** the y-z dilution cross-plane (`_spatial_segregation`, pure-stdlib) and `C_opt` **EMERGES as an OUTPUT**: the penetration `δ=k_p·√(S·H)·J^(1/4)` couples `S` in (fixed dilution mass ratio ⇒ `d_j∝√(SH)J^(−1/4)`), a **fixed mixing-length** spread + **far-wall reflection** penalize over-penetration, and the uniformity best is where `δ` fills half the height ⇒ **`(S/H)√J=1/(4·k_p²)`**, `S,H`-independent. A `SpatialPDF(S,k_p,…)` config (rides on `mixing`, **≤1-of-SIX** with `unmixedness`/`pdf`/`pdf_quench`/`pocket_quench`/`transported`) feeds the resolved width through the **same** rung-13 ideal bell (`_pdf_mean_ei`) as rung 18 — **only the SOURCE of `g` changes** (imposed ODE → derived cross-plane). **Certified (the collapse):** the `g_min` **VALUE is geometry-independent** (`0.0182` across `2×`-independent `S`/`H`); only `J_opt` moves, **exactly as `(H/S)²`**; the argmin `C` = the closed form `1/(4k_p²)≈2.5`; **NO `C_opt` knob** (the signature of the inversion — `SpatialPDF(C_opt=…)` is a constructor error; `C_opt()` is a derived property); robust to `k_p`; `g_spatial < g_ceiling` always (the rung-18 two-stream ceiling **bounds** the resolved field). **Honest concessions (loud):** the **VALUE** `C_opt≈2.5` rides on the semi-empirical `k_p` (only the **collapse + `(H/S)²` shift** are derived, a *group* not a number); rung 22 derives the **WIDTH** `g(C)`, **NOT the DWELL** `τ_core(C)` (the rung-16 kink stays imported — a **partial** closure of "width AND dwell"); the **emissions** optimum at `C_opt` is only **LOCAL** — through the pure ideal bell the **GLOBAL** `⟨EI⟩` min is at **max segregation** (rung-13's descending far flank, spatialized), because the derived floor `g(C_opt)≈0.018` sits **just below** the ideal-bell `⟨EI⟩(g)` **hump peak ≈0.021** (a **narrow** basin) — which is **WHY uniformity (`g`), not emissions, is the headline** (rung 18 was **not** wrong: it reported the real **local** behaviour). The field is a **Gaussian-plume cartoon** feeding the β-PDF closure — a real PDF-transport/CFD cross-plane (the full **shape**, the **dwell spectrum**, and rung-17's firing **magnitude**) stays the deferred ceiling. `super_eq_o=True` threads the rung-21 `m(T)` lift through the shared bell; default `False`. Pure diagnostic ⇒ cycle bit-for-bit rung 6. `spatial=None` ⇒ exact prior rung. | `docs/rung22-spec.md` |
| 23 | **Derived dwell spectrum** — completing rung 22's **partial** closure (the "width AND dwell" seam, other half). Rung 22 derived the cross-plane **WIDTH** `g(C)` but fed it through the per-pocket quench with the **imported rung-16 KINKED scalar dwell** `τ_core(C)=τ_res·(1+b_u·|ln(C/C_opt)|)` — which **bakes `C_opt` in**. Rung 23 develops the **SAME** cross-plane in **TIME** (`_spatial_dwell_field`, over rung-11's `τ_mix=H/(C_e√J·U_c)`; `σ(t)∝√t`, `δ(t)∝t^⅓`, both terminating at `τ_mix` ⇒ the terminal field **IS** rung 22's, the reduce anchor), so each pocket carries its **OWN** dwell `τ(ξ)` from first principles — **NO `C_opt`, NO `τ_res`, NO `b_u`** (the non-circularity contract; the scale is rung-11's `τ_mix`, so `SpatialDwellPDF` adds **NO new dwell knob**). A `SpatialDwellPDF(S,k_p,…)` config (rides on `mixing`, **≤1-of-SEVEN** with the other six; **no `C_opt` field**, `C_opt()=1/(4k_p²)` derived) feeds `τ(ξ)` through the **same** rung-16 per-pocket quench (`_pocket_quench_mean_ei` gains a `tau_of_xi` callable; `None` ⇒ byte-identical rung 16). **The load-bearing POSITIVE — the ξ–τ CORRELATION:** rich pockets are the **late-arriving** ones ⇒ dwell **correlates** with composition (rung-16's scalar `τ_core` has `corr≡0` by construction — the physics it structurally cannot express). A **MATCHED-MEAN** twin (`τ(ξ)` spectrum vs a scalar `⟨τ⟩_PDF`) isolates it: `corr_ratio=ei_no_spatial_dwell/…_meanfield` (term-2 ratio) `>1` ⇒ the correlation **ADDS NO** (rich pockets dwell long ⇒ re-make more), **ONE-SIGNED across `τ_mix ×0.2–×5`** (the pockets stay **formation-limited**, `max_a<1` — the Jensen concavity of EI-in-`τ` never wins), **CONCENTRATED under-penetration** (~+5.8% at J=4 → ~+2.5% at `C_opt` → a ~+0.8% over-penetration floor; the shipped 32/24 grid). **Honest concessions (rung-18-flavored, loud):** the correlation's absolute **MAGNITUDE/off-optimum TREND** rides on rung-11's **un-anchored** `τ_mix` (∝1/√J) — only the **SHAPE** (sign + under-penetration concentration) is derived (as rung-22's `C_opt≈2.5` rides on `k_p`); the **emissions `C_opt` pin is NOT recovered** (the derived `τ` **FALLS** off-optimum, does not grow like rung-16's kink, so the over-penetration flank is not lifted — **but rung 16 already declined the global-min LOCATION**, its GATE 3, so this is a first-principles *confirmation*, not a discovery; whether the pin survives depends on whether the dwell **grows** (rung-16, imposed) or **falls** (rung-23, derived) — **neither pinned**); rung-16 `τ_core` and rung-23 `⟨τ⟩` **diverge ~3×–70× and trend OPPOSITE** off-optimum, both un-anchored — an **HONEST split**, not "rung 16 is an artifact"; **ONE global `τ_mix`** (a Gaussian-plume cartoon) — a **locally-resolved** mixing time (each cell its own rate), plus the full CFD-PDF **shape** (which would let rung 17 claim a firing **MAGNITUDE**), stays the deferred ceiling. Only the **WIDTH** `g` pins `C_opt` (rung-22's uniformity result, preserved); `g_spatial < g_ceiling` still holds. `super_eq_o=True` threads the rung-21 `m(T)` lift through the shared per-pocket quench; default `False`. Pure diagnostic ⇒ cycle bit-for-bit rung 6. `spatial_dwell=None` ⇒ exact prior rung. | `docs/rung23-spec.md` |

Rungs 7–13, 15, 16, 17, 18, 19, 20, 21, 22 and 23 are **pure diagnostics** — NO/N never enter the cycle solve, so the cycle
stays **bit-for-bit rung 6**. Rung 14 is *also* a pure diagnostic (`Gas.nozzle_flow` only reads the
run's state; the production nozzle stays frozen), so the cycle is still bit-for-bit rung 6. Rung 17 is
*also* a pure diagnostic (`Gas.exhaust_no_clamp` only reads the run's state and composes the rung-8/11/16
+ rung-14 outputs), so the cycle stays bit-for-bit rung 6. Rung 18 is *also* a pure diagnostic
(`zoned_nox(…, transported=…)` only adds a transported-width PDF integral over the existing ideal bell —
NO/N never enter `_equil_solve`), so the cycle stays bit-for-bit rung 6. Rung 19 is *also* a pure
diagnostic (`thermal_nox`/`zoned_nox(…, super_eq_o=…, prompt=…)` only lift the primary [O] and add an
imposed prompt term — NO/N never enter `_equil_solve`), so the cycle stays bit-for-bit rung 6. Rung 20 is
*also* a pure diagnostic (`zoned_nox`/`exhaust_no_clamp(…, super_eq_o=…)` only lift `[O]` inside the
existing `_quench_no` re-making — NO/N never enter `_equil_solve`), so the cycle stays bit-for-bit rung 6. Rung 21 is
*also* a pure diagnostic (`zoned_nox(…, super_eq_o=…)` threads the same `m(T)` lift through the ideal-bell
PDF integrals `ei_no_pdf`/`ei_no_pdf_quench` term 2/`ei_no_transported` — NO/N never enter `_equil_solve`),
so the cycle stays bit-for-bit rung 6. Rung 22 is *also* a pure diagnostic (`zoned_nox(…, spatial=…)` only
adds a resolved cross-plane width `g_spatial` fed through the existing ideal bell — NO/N never enter
`_equil_solve`), so the cycle stays bit-for-bit rung 6. Rung 23 is *also* a pure diagnostic
(`zoned_nox(…, spatial_dwell=…)` only develops that resolved cross-plane in TIME for a per-pocket dwell
`τ(ξ)` fed through the existing per-pocket quench — NO/N never enter `_equil_solve`), so the cycle stays
bit-for-bit rung 6. Each
rung's verified anchor data (textbook / formation / CEA-equilibrium / Zeldovich-kinetics / ICAO-zoning /
rich-RQL / finite-quench / jet-mixing / unmixedness / mixing-PDF / frozen-vs-equilibrium-nozzle /
PDF-through-quench / per-pocket-PDF-quench / super-equilibrium-exhaust / transported-variance /
super-equilibrium-O-and-prompt / super-equilibrium-O-through-quench / ideal-bell-PDF-lift /
resolved-spatial-PDF / derived-dwell-spectrum) lives in `docs/plans/rungN-anchor-*.md`; `docs/plans/` also holds the
living plan/tasks (rungs 1–3).

## Working contract (from SPEC.md — these override convenience)
- **Derive before you code.** For each station, write the governing equation and
  a one-line physical justification (why it holds) *before* implementing it.
- **Show the work.** Every run prints the full station table (Tt, pt, …) so the
  numbers can be watched propagating.
- **Pure components.** Each component is `apply(state, gas) -> state` with no
  hidden state (Turbine and Nozzle diverge their signatures by design).
- **Conservation checks are assertions**, run on every execution (not as
  separate tests). See SPEC.md / docs/rung2-spec.md § Conservation checks.
- **Current scope (rung 23):** all rungs above are cumulative and live (see § The
  rungs). The **cycle solve** is a thermally-perfect, reacting, dissociation-
  equilibrium gas (`Gas.reacting_equilibrium()`) run through ideal + real components
  (isentropic `η_c/η_t` **or** polytropic `e_c/e_t`, mutually exclusive; `π_d/π_b/π_n`,
  `η_b`, `η_m`; dual cold/hot gas; specified exit pressure). The burner is a root-find
  on `f` over the scale-B absolute balance (equilibrium composition re-solved each trial,
  then frozen through turbine + nozzle). Rungs 7–13 add the **NOx diagnostics**
  (`Gas.thermal_nox` / `Gas.zoned_nox`) *beside* the cycle, never inside it; rung 14 adds the
  **nozzle-flow diagnostic** (`Gas.nozzle_flow`) — also beside the cycle (the production nozzle
  stays frozen) — hence bit-for-bit rung 6; rung 15 adds the **PDF-through-quench diagnostic**
  (`zoned_nox(…, pdf_quench=QuenchPDF(…))`) — combining the rung-13 composition β-PDF with the
  rung-10/12 dwell; rung 16 adds the **per-pocket PDF-through-quench diagnostic**
  (`zoned_nox(…, pocket_quench=PocketQuenchPDF(…))`) — carrying each β-PDF pocket through its OWN
  `_quench_no`, retiring rung-15's linearised dwell; rung 17 adds the **exhaust-NO clamp ladder**
  (`Gas.exhaust_no_clamp(…)`) — carrying three combustor-mixing-fidelity exhaust-NO models (rung-8
  mixed-out, rung-11 bulk quench, rung-16 per-pocket) through the **same** rung-14 nozzle collapse to
  show the mixed-out shortcut reads the dropped clamp **dormant** at the rich primary while the fuller
  models **fire** (all still bit-for-bit rung 6); rung 18 adds the **transported-variance diagnostic**
  (`zoned_nox(…, transported=TransportedPDF(…))`) — replacing rung-13's imposed kink with the residual of
  a variance-decay ODE from a **derived** two-stream ceiling, and proving (the load-bearing **negative**
  result) that a 0-D transport **cannot derive** the `C_opt` optimum (mean-field `ω` ⇒ monotone `g(J)`;
  the optimum needs the spatial spacing `S`), so the coverage `ω(C)` stays an **imposed** spatial closure;
  rung 19 adds the **super-equilibrium-O & prompt-NO diagnostic** (`thermal_nox`/`zoned_nox(…,
  super_eq_o=…, prompt=…)`) — lifting the equilibrium-O **lower bound** every NO number since rung 7
  carried, two ways that **both** refute the naive "rich primary explodes" intuition: a **computed**
  T-driven Westenberg super-eq-O multiplier `m(T)∈[1.16,1.50]` (weakest when rich) and an **imposed**
  rich-specific De Soete prompt φ-bump (survives where thermal dies; ~27× less T-sensitive). Both lift
  only the **primary** diagnostic and stay trace, so bit-for-bit rung 6;
  rung 20 adds the **super-equilibrium-O-through-the-quench diagnostic** (`zoned_nox`/`exhaust_no_clamp(…,
  super_eq_o=…)`) — threading the rung-19 lift **into** the `_quench_no` finite-quench re-making, so
  `ei_no_quenched`/`ei_no_pocket_quench` and the rung-17 clamp `a` (which still rode on **equilibrium O**)
  finally carry it. The load-bearing result **inverts** the intuition a third time: the lift is
  **modest & peak-concentrated** (`≈m(T_peak)≈1.14`, even **smaller** than the primary lift) because the
  Zeldovich re-making peaks at the hottest crossing where `m(T)` is minimal; the rung-17 `a` **rise**
  (numerator lifts, the thermodynamic denominator `x_no_e(T9)` untouched) — discharging the "every `a`
  is a lower bound" caveat — but the clamp stays **dormant at station 4** (super-eq O is not the
  burner-clamp lever). Narrow, per-field-consistent scope; the ideal-bell PDF integrals stay eq-O in
  rung 20 (**lifted in rung 21**); prompt rides as an invariant EI. Pure diagnostic, so bit-for-bit rung 6.
  Rung 21 adds the **super-equilibrium-O-through-the-ideal-bell-PDF diagnostic** (`zoned_nox(…,
  super_eq_o=…)` now combines with `pdf`/`pdf_quench`/`transported`) — discharging the **last eq-O seam**
  rung 20 left open. It threads the same `m(T)` lift through the three ideal-bell composition integrals
  (`ei_no_pdf` rung 13, `ei_no_pdf_quench` term 2 rung 15, `ei_no_transported` rung 18) via `super_eq_o=`
  on `_bell_interpolator`/`_pdf_mean_ei` (both the built bell and the `g→0` delta), so **both**
  `pdf_quench` terms lift and rung 20's half-lifted **hybrid** dissolves — the rung-20 **forbid guard is
  removed**. The load-bearing result is the rung-20 **inversion generalized to composition variance**: the
  ideal bell EI(φ) is peaked near stoich (hottest ⇒ `m` minimal), the β-PDF is EI-weighted onto that peak,
  so the effective lift is **≈×1.15** (below the primary ×1.28, far below the deep-lean point value ×1.91,
  and *decreasing* with segregation `g`). A **shape-preserving consistency lift**: the optimum LOCATION
  (AT `C_opt`), the `(H/S)²` shift and the stoich-mean sign reversal are unmoved. `super_eq_o=False` ⇒
  bit-for-bit rungs 13/15/18. Pure diagnostic, so bit-for-bit rung 6.
  Rung 22 adds the **resolved-cross-plane / spatial-PDF diagnostic** (`zoned_nox(…, spatial=SpatialPDF(…))`)
  — the **INVERSION of rung 18**. Where rung 18 proved a **0-D** variance transport **cannot derive** the
  Holdeman `C_opt` (mean-field `ω(J)` ⇒ monotone `g(J)`, so it **imposed** the coverage `ω(C)` — the jet
  spacing `S` by hand), rung 22 **resolves** the y-z dilution cross-plane (`_spatial_segregation`, pure
  stdlib) and `C_opt` **EMERGES as an OUTPUT**: the penetration `δ=k_p·√(S·H)·J^(1/4)` couples `S` in, a
  fixed mixing-length spread + far-wall reflection penalize over-penetration, and the uniformity best is
  where `δ` fills half the height ⇒ `(S/H)√J=1/(4k_p²)`, `S,H`-independent. The resolved width feeds the
  **same** rung-13 ideal bell as rung 18 (only the **source** of `g` changed, imposed → derived).
  Certified: the `g_min` **VALUE** is geometry-independent (the collapse), `J_opt` shifts **exactly** as
  `(H/S)²`, `C_opt`=`1/(4k_p²)` with **no `C_opt` knob** (the signature of the inversion), `g_spatial <
  g_ceiling` (the rung-18 ceiling bounds it). Honest: the **value** `C_opt≈2.5` rides on the semi-empirical
  `k_p` (only the collapse + the `(H/S)²` shift are derived, a *group* not a number); rung 22 derives the
  **width** `g(C)`, **not** the **dwell** `τ_core(C)` (rung-16 kink imported — a **partial** closure); and
  the **emissions** min at `C_opt` is only **LOCAL** (through the pure ideal bell the global `⟨EI⟩` min is
  at max segregation — rung-13's descending far flank spatialized — because the derived floor `≈0.018`
  sits just below the ideal-bell hump peak `≈0.021`, a narrow basin), so uniformity, not emissions, is the
  headline. `spatial=None` ⇒ exact prior rung. Pure diagnostic, so bit-for-bit rung 6.
  Rung 23 adds the **derived-dwell-spectrum diagnostic** (`zoned_nox(…, spatial_dwell=SpatialDwellPDF(…))`)
  — completing rung 22's **partial** closure. Rung 22 derived the cross-plane **width** `g(C)` but fed it
  through the per-pocket quench with the **imported rung-16 kinked scalar dwell** `τ_core(C)` (which bakes
  `C_opt` in); rung 23 develops the **same** cross-plane in **time** (`_spatial_dwell_field`, over rung-11's
  `τ_mix=H/(C_e√J·U_c)`; `σ∝√t`, `δ∝t^⅓`, both terminating at `τ_mix` ⇒ the terminal field **is** rung 22's,
  the reduce anchor), so each pocket carries its **own** dwell `τ(ξ)` from first principles — **no `C_opt`,
  no `τ_res`, no `b_u`** (the non-circularity contract; the scale is rung-11's `τ_mix`, so no new dwell
  knob). The one genuinely new quantity is the **ξ–τ correlation**: rich pockets are the **late-arriving**
  ones ⇒ dwell **correlates** with composition (rung-16's scalar `τ_core` has `corr≡0` by construction). A
  **matched-mean** twin (`τ(ξ)` spectrum vs a scalar `⟨τ⟩_PDF`) isolates it — `corr_ratio>1` ⇒ the
  correlation **adds NO**, **one-signed across `τ_mix ×0.2–×5`** (the pockets stay formation-limited,
  `max_a<1`), **concentrated under-penetration** (~+5.8% at J=4 → ~+2.5% at `C_opt`). Honest
  (rung-18-flavored): the correlation's absolute **magnitude/trend** rides on rung-11's un-anchored `τ_mix`
  (∝1/√J) — only the **shape** (sign + concentration) is derived; the **emissions `C_opt` pin is not
  recovered** (the derived `τ` falls off-optimum — but rung 16 already declined the global-min location, its
  GATE 3); rung-16 `τ_core` and rung-23 `⟨τ⟩` **diverge ~3×–70× and trend opposite** off-optimum, both
  un-anchored — an honest split, not "rung 16 is an artifact"; **one global `τ_mix`** (a cartoon), a
  locally-resolved mixing time staying the ceiling. `super_eq_o=True` threads the rung-21 `m(T)` lift
  through the shared per-pocket quench. `spatial_dwell=None` ⇒ exact prior rung. Pure diagnostic, so
  bit-for-bit rung 6.
  Fork A/B
  (`Gas.reacting()` / `reacting_forkb()`) and the frozen-products `Gas.thermally_perfect()` are kept
  alongside. **Deferred seams** (kept open on purpose): **finite-rate nozzle chemistry** — rung 14
  gives the frozen↔equilibrium *bracket*, not the real Damköhler-number flow *between* the bounds (nor
  a **shifting turbine**); a **spatial / transported-CFD PDF** — rungs 22–23 took the first steps,
  **deriving** the β width `g(C)` (rung 22, hence the optimum LOCATION as an OUTPUT) **and** the **dwell
  spectrum** `τ(ξ)` (rung 23, hence the ξ–τ **correlation**) from a resolved cross-plane with the jet
  **spacing `S`** — the inversion/completion of rung 18's 0-D negative result — but a **real** PDF-transport
  / scalar-flux solve that predicts the full cross-plane **pattern** (the full resolved-PDF *shape*, not just
  its variance `g`, **and** a **locally-resolved mixing time** — each cell its own rate, not rung-23's one
  global `τ_mix` — which is what an *off-optimum dwell growth* that pins the emissions optimum
  non-circularly would need) stays the ceiling; that locally-resolved mixing time is also what would let
  rung 17 claim a firing *magnitude*, not just a direction (rungs 22–23 are a Gaussian-plume **cartoon**
  feeding the β-PDF closure, so the value `C_opt≈2.5` and the dwell *magnitude* still ride on `k_p`/`τ_mix`);
  a regime where a **per-pocket clamp fires AT THE BURNER**
  (`max_a>1` at station 4, not just in the rung-14/17 nozzle where rung 17 already fires it) — the
  lever is a **slow-enough freeze on a cooling pocket** (a long dwell that freezes NO high while the
  local `[NO]_e` collapses), *not* a hotter `Tt4` alone (raising `Tt4` raises the terminal `[NO]_e`
  and *lowers* the ratio) — and rung 20 shows super-eq O is **not** that lever (it speeds *formation*,
  not the `[NO]_e` collapse, so `max_a` stays <1 at station 4; **rung 21 confirms** the ideal-bell PDF
  lift is *also* not this lever — it is a shape-preserving consistency lift, `max_a` unmoved); the
  **ideal-bell PDF lifts** are now **DONE** (rung 21 — the super-eq-O `m(T)` threaded through
  `ei_no_pdf`/`ei_no_pdf_quench` term 2/`ei_no_transported`, the rung-20 forbid guard removed); **detailed Fenimore**
  (`CH+N₂→HCN→…`) and **super-equilibrium-O radical-decay history** (both need new species / a relaxing
  pocket — rung 19 kept the prompt magnitude *imposed* and the super-eq ratio *semi-empirical*, and rung
  20 carried that same semi-empirical ratio through the quench, precisely because a 0-D pool cannot
  derive them); off-design / component maps, a *choked* convergent nozzle,
  afterburner.
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
  Rung 15 adds the `QuenchPDF` config (jet spacing `S` + `C_opt`/`k_g`/`g_max` (rung-13 segregation) +
  `tau_res`/`b_u` (rung-12 dwell) + grid sizes; `C(mixing)=(S/H)√J`, `segregation(C)=min(g_max,
  k_g·|ln(C/C_opt)|)`, `dwell_factor(C,τ_ref)=τ_res·(1+b_u·|ln(C/C_opt)|)/τ_ref`) and gives `zoned_nox`
  a `pdf_quench=` param (**requires `mixing`**, **mutually exclusive with `pdf` and `unmixedness`** — a
  `≤1`-of-three guard) that carries rung-13's β-PDF **through** the finite quench: `ei_no_pdf_quench` =
  **term 1** (the rung-11 mean-field bulk quench `ei_no_quenched` — the **finite floor**) **+ term 2**
  (`dwell_factor(C,tau)·_pdf_mean_ei(…,g)` — the rung-13 β-PDF integral over the ideal bell, **reused
  verbatim**, scaled by the off-optimum-growing dwell). The ≈0 rung-13 optimum **becomes finite bulk
  NO** (at `C_opt`, `g→0` ⇒ term 2→0 ⇒ `=ei_no_quenched`), the min stays **AT `C_opt`** (`(H/S)²`
  shift), and the rung-13 **descending** far flank **CLIMBS** (dwell restored, surviving `J→∞`); the
  nonlinear bell keeps the **stoich-mean sign reversal** (the discriminator a dwell-only closure fails).
  `ZonedNOxState` records `pdf_quench`/`ei_no_pdf_excess`/`ei_no_pdf_quench` (`C_holdeman`/`g_seg`
  reused). `pdf_quench=None` ⇒ exact rung 13. No new chemistry/integrator — only the additive combination
  (a bulk quench already computed + the rung-13 integral × a scalar dwell factor).
  Rung 16 adds the `PocketQuenchPDF` config (same knobs as `QuenchPDF`, but `core_dwell(C)=τ_res·(1+
  b_u·|ln(C/C_opt)|)` is the **absolute** `τ_core` passed INTO each pocket's quench, not rung-15's
  `dwell_factor` RATIO) and the module helper `_pocket_quench_mean_ei` (the rung-16 upgrade of
  `_pdf_mean_ei`: over a fixed ξ-grid on `[0,ξ(φ=2)]`, each **rich-of-mean** pocket is carried through
  its OWN `_quench_no` at `τ_core`; **lean-of-mean/φ>2** pockets reuse `_ideal_bell_ei` — 0 above φ2,
  bit-identical to rung 15 — since they never re-cross stoich; interpolate + integrate over the β-PDF,
  tail→0; returns `(⟨EI⟩, max_a)`). `zoned_nox` gains a `pocket_quench=` param (**requires `mixing`**,
  **≤1-of-four with `pdf_quench`/`pdf`/`unmixedness`**) → `ei_no_pocket_quench` = **term 1**
  (`ei_no_quenched`, the same mean-field floor) **+ term 2** (`_pocket_quench_mean_ei(…)`). Because a
  lingering pocket **COOLS**, term 2 is **SUBLINEAR** in `τ_core` (far-flank ≈×1.3 vs rung-15's LINEAR
  ×1.51 = the dwell ratio), which **ERODES** rung-15's over-penetration far flank ~18–32% into
  **near-degeneracy** with the `C_opt` notch (which **survives**: term 2 →0 at `C_opt` ⇒
  `=ei_no_quenched`). The **global-min LOCATION is NOT claimed** (flips sign across quadrature ~5% / φ>2
  tail / `C_e` 2%→21%) — the honest headline is the cooling-limited erosion + the sublinear slope, not a
  relocated optimum. Clamp **dormant** (`max_a<1`, folded into `max_a_quench` over the pockets) — the
  difference is cooling, not super-eq rollover. `ZonedNOxState` records `pocket_quench`/
  `ei_no_pocket_excess`/`ei_no_pocket_quench` (`C_holdeman`/`g_seg` reused). `pocket_quench=None` ⇒
  exact rung 15. No new chemistry/integrator — only per-pocket reuse of the rung-10 `_quench_no` + the
  rung-13 β-PDF quadrature (so ~`n_bell`× costlier than rung 15's single bell). Rung 17 adds the
  **cycle-side exhaust-NO clamp ladder**: the `ExhaustNOxClampState` dataclass (the three margins
  `a_mixed_out`/`a_bulk_quench`/`a_pocket` + the common `x_no_e_exit`/`no_collapse_ratio`, the three
  numerators, the transparency pair `ei_no_quenched`/`ei_no_pocket_quench` and their ratio
  `gap_pocket_over_bulk`, `max_a_quench`, and the `hides_super_eq`/`ladder_monotone` predicates) and
  `Gas.exhaust_no_clamp(far, Tt3, Tt4, p, Tt9, pt9, p9, phi_primary, mixing, pocket_quench, tau,
  quench_ngrid, quench_nsteps)` — it calls `zoned_nox` **three ways** (rung 8/11/16) for the numerators
  and `nozzle_flow` **once** (rung 14) for the common denominator `x_no_e(T9)`, then forms `a_i =
  x_i/x_no_e(T9)`. The per-pocket mole fraction is `κ·⟨EI⟩_pocket` with `κ=x_no_bulk/ei_no_quenched`
  (`x_no ∝ EI` at fixed far), which is exactly why the nozzle is a no-op on the pocket/bulk ratio.
  **No new chemistry/config** — it only READS state (through the untouched rung-8/11/16 + rung-14
  machinery), so the cycle is bit-for-bit rung 6. Requires the equilibrium gas + both configs. Rung 18
  adds the **transported-variance closure**: the `TransportedPDF` config (rides on `JetMixing`,
  **≤1-of-five** with the other four; knobs `S`/`C_opt`/`C_phi=2.0`/`Da_opt`/`w_cov`/`tau_mix` + grids;
  a `coverage_omega(C)` = the IMPOSED spatial `ω(C)` and a `segregation(C,far,φ_p)` that returns the
  transported width + derived ceiling), the module helpers `_two_stream_ceiling(far,φ_p)` (the DERIVED
  ceiling `(ξ_p−ξ̄)/(1−ξ̄)`, asserts `φ_p>φ_overall`) and `_transport_variance(g_ceiling,ω,τ,C_φ,nsteps)`
  (backward-Euler integration of `dg/dt=−C_φ·ω·g` — positivity-preserving; the negative-result gate drives
  it with mean-field `ω(J)` to show monotone `g(J)`), the `ZonedNOxState` fields
  `transported`/`g_ceiling`/`g_transported`/`ei_no_transported`, and a `transported=` param on `zoned_nox`
  that feeds the transported width through the rung-13 ideal bell (`_pdf_mean_ei`). `transported=None` ⇒
  exact prior path; a pure diagnostic (NO/N never enter `_equil_solve`), so bit-for-bit rung 6. Rung 19
  adds the **super-equilibrium-O & prompt-NO lift** of the equilibrium-O lower bound: the Westenberg
  constants `_WESTENBERG_C1/TH1/C2/TH2` + `_super_eq_o_multiplier(T)` (the dimensionless T-only ratio
  `m(T)=(C2/C1)·T·exp((θ1−θ2)/T)∈[1.16,1.50]`), an `o_multiplier=1.0` param on `_thermal_no` (lifts the
  pool's `[O]` inside the integrator; `1.0` ⇒ byte-identical rung 7) with `NOxState` gaining
  `o_multiplier`/`ei_no_prompt` + an `ei_no_total` property, the `PromptNO` config (the imposed De Soete
  φ-shape `f(φ,n)=4.75+0.0819n−23.2φ+32φ²−12.2φ³`, `ei_prompt(φ,T)=scale·max(f,0)·exp(−Ea/RuT)`, `scale`
  back-solved from an imposed `peak_ei`), the `ZonedNOxState` fields `super_eq_o`/`o_multiplier`/`prompt`/
  `ei_no_prompt` + `ei_no_total`, and `super_eq_o=False`/`prompt=None` params on `thermal_nox`/`zoned_nox`
  (both off ⇒ the exact prior code path). A **summed** trace guard spans both channels; the lift acts only
  on the **primary** diagnostic; a pure diagnostic (NO/N never enter `_equil_solve`), so bit-for-bit rung 6.
  Rung 20 threads that same super-eq-O lift **through the finite quench**: a `_SUPER_EQ_T_FLOOR=1500 K`
  flame-band floor (`m(T)=A·T·exp(B/T)` diverges as `T→0`), a `super_eq_o=` param on `_quench_no`
  (lifts `[O]` by `m(max(T,floor))` INSIDE the cooling `dn_dt`, a standing `1≤m≤2` trajectory assert;
  `False` ⇒ byte-identical rung 10/11/12/16), threaded through `_ideal_bell_ei`/`_pocket_quench_mean_ei`
  (each pocket's initial `_thermal_no` + its own `_quench_no` + the lean/tail bell — no half-eq-O
  pocket), and a `super_eq_o=` param on `zoned_nox`/`exhaust_no_clamp` that lifts the bulk/core/per-pocket
  quench + all three clamp numerators (the thermodynamic denominator `x_no_e(T9)` untouched). `zoned_nox`
  forbids `super_eq_o` with `pdf`/`pdf_quench`/`transported` (rung 20 kept the ideal-bell integrals eq-O
  — **rung 21 removes this guard**); `ZonedNOxState` gains an `ei_no_quenched_total` property (adds the
  invariant prompt). The lift is modest & peak-concentrated (`≈m(T_peak)`); a pure diagnostic, so
  bit-for-bit rung 6.
  Rung 21 threads the SAME super-eq-O `m(T)` lift **through the ideal-bell PDF integrals**: a
  `super_eq_o=` param on `_bell_interpolator` (lifts every bell node's `[O]` via the existing
  `_ideal_bell_ei(…, super_eq_o=…)`) and on `_pdf_mean_ei` (both the built bell AND the `g→0` delta
  short-circuit, so the reduce stays consistent in the limit); `zoned_nox` passes it into the three
  `_pdf_mean_ei` calls (`ei_no_pdf` rung 13, `ei_no_pdf_quench` term 2 rung 15, `ei_no_transported`
  rung 18) and the rung-20 forbid guard is **replaced by an explanatory comment** — `super_eq_o` now
  combines with every closure, and `ei_no_pdf_quench` is no longer a half-lifted hybrid (both terms carry
  `m(T)`). The effective lift is peak-concentrated at ≈×1.15 (EI-weighted onto the near-stoich bell peak
  where `m` is minimal), BELOW the primary ×1.28; `False` ⇒ byte-identical rungs 13/15/18. A pure
  diagnostic, so bit-for-bit rung 6.
  Rung 22 adds the **resolved cross-plane / spatial PDF** — the INVERSION of rung 18: the module helper
  `_spatial_segregation(far, φ_p, S, H, J, k_p, k_y, k_z, ny, nz)` (pure stdlib) builds a separable
  Gaussian air plume over one dilution cell (penetration `y∈[0,H]` × span `z∈[0,S]`; penetration
  `δ=k_p·√(S·H)·J^(1/4)` couples `S` in via the fixed-mass-ratio jet diameter; a **fixed mixing-length**
  spread `σ_y=k_y·H, σ_z=k_z·S` + **both-wall mirror images** so over-penetration piles air at the far
  wall; spanwise-periodic), **root-finds the air scale so `⟨ξ⟩=ξ̄` exactly** (mean-preserving), and
  returns `g=Var[ξ]/(ξ̄(1−ξ̄))` — whose minimum over `J` collapses onto `C=(S/H)√J=1/(4k_p²)` as an
  **OUTPUT**. The `SpatialPDF` config (rides on `JetMixing`, **≤1-of-six** with the other five closures;
  knobs `S`/`k_p`/`k_y`/`k_z` + grids — **no `C_opt` field**, `C_opt()` is a derived property) computes
  the width via `segregation(mixing, far, φ_p)` (returns `(g_spatial, g_ceiling)`), and `zoned_nox` gains
  a `spatial=` param that feeds `g_spatial` through the **same** rung-13 ideal bell (`_pdf_mean_ei`,
  `super_eq_o=` threads the rung-21 lift), asserting `g_spatial < g_ceiling`. `ZonedNOxState` gains
  `spatial`/`g_spatial`/`ei_no_spatial` (`C_holdeman`/`g_ceiling`/`g_seg` reused). The emissions min at
  `C_opt` is only **local** (the derived floor sits just below the ideal-bell hump peak); uniformity is
  the headline. `spatial=None` ⇒ exact prior path; a pure diagnostic (NO/N never enter `_equil_solve`),
  so bit-for-bit rung 6.
  Rung 23 adds the **derived dwell spectrum** — completing rung 22's partial closure: the module helper
  `_spatial_dwell_field(far, φ_p, S, H, J, τ_mix, k_p, k_y, k_z, ny, nz, nt)` (pure stdlib) develops the
  rung-22 plume in **time** (`σ(t)=σ_final·√(t/τ_mix)`, `δ(t)=δ_final·(t/τ_mix)^⅓`, both terminating at
  `τ_mix` so the terminal field == `_spatial_segregation` — the reduce anchor), root-finds the terminal air
  scale so `⟨ξ⟩=ξ̄` (mean-preserving, asserted), integrates each cell's **arrival-time deficit**
  `τ_cell=∫₀^{τ_mix}(1−β(t)/β_final) dt`, and returns `(g_spatial, tau_of_xi)` — `tau_of_xi` a
  monotone-binned interpolator `τ(ξ)` (idiomatic like `_bell_interpolator`). The absolute scale `τ_mix` is
  rung-11's `mixing.tau_q` — **no new dwell knob, no `C_opt`/`τ_res`/`b_u`** (the non-circularity contract).
  `_pocket_quench_mean_ei` gains an optional `tau_of_xi` callable (each pocket quenches at `τ(ξ)`; `None` ⇒
  byte-identical rung 16). The `SpatialDwellPDF` config (rides on `JetMixing`, **≤1-of-seven** with the
  other six; knobs `S`/`k_p`/`k_y`/`k_z`/`nt` + grids — **no `C_opt` field**, `C_opt()` derived) computes
  the width AND the spectrum via `dwell_field(mixing, far, φ_p)` (returns `(g_spatial, tau_of_xi,
  g_ceiling)`), and `zoned_nox` gains a `spatial_dwell=` param → `ei_no_spatial_dwell = ei_no_quenched +
  ⟨EI_pocket_quench(ξ;τ(ξ))⟩_g` (term1 floor + correlated term2) PLUS a **matched-mean twin**
  `ei_no_spatial_dwell_meanfield` (scalar `⟨τ⟩_PDF`) and `corr_ratio` (their term-2 ratio, the correlation
  isolated). `ZonedNOxState` gains `spatial_dwell`/`g_spatial_dwell`/`tau_mean_dwell`/`ei_no_spatial_dwell`/
  `ei_no_spatial_dwell_meanfield`/`ei_no_spatial_dwell_excess`/`corr_ratio` (`C_holdeman`/`g_ceiling`/`g_seg`
  reused). The certified positive is the ξ–τ **correlation** (`corr_ratio>1`, one-signed while
  formation-limited); the emissions `C_opt` pin is not recovered (the derived `τ` falls off-optimum — rung
  16 already declined the global-min location). `super_eq_o=` threads the rung-21 `m(T)` lift through the
  shared per-pocket quench. `spatial_dwell=None` ⇒ exact prior path; a pure diagnostic (NO/N never enter
  `_equil_solve`), so bit-for-bit rung 6.
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
- `tests/test_rung15.py` — rung-15: two reduces (`pdf_quench=None` code-path-identical rung 13, all
  rung-15 fields None; at `C_opt` (`g→0`) `ei_no_pdf_quench` == the **finite bulk quench NO**
  `ei_no_quenched` to <0.01% — NOT rung-13's ≈0) + a helper-vs-production `zoned_nox` pin, the **finite
  floor** (rung-15 optimum finite vs rung-13's ≈0, orders larger), the optimum **pinned AT `C_opt`**
  (both flanks up, the far over-penetration flank **CLIMBING** + staying elevated — the restored dwell,
  global min AT `J_opt`), the **`(H/S)²` shift**, the **stoich-mean sign reversal** (the discriminator a
  dwell-only closure fails), the kinked `g(C)`/`u(C)` + growing `dwell_factor`, cycle-untouched,
  require-`mixing` + `≤1`-of-`{pdf_quench,pdf,unmixedness}` mutual-exclusivity + `QuenchPDF`
  positivity/range guards. (Reuses a cached DP + a shared trajectory + a bell built once.)
- `tests/test_rung16.py` — rung-16: two reduces (`pocket_quench=None` code-path-identical rung 15, all
  rung-16 fields None; at `C_opt` (`g→0`) `ei_no_pocket_quench` == the **finite bulk quench NO**
  `ei_no_quenched` to <0.1%) + a cached-bank helper-vs-production `zoned_nox` pin, **THE MECHANISM** (the
  far-flank term2 ratio is **SUBLINEAR** — rung-16 ≈×1.3 vs rung-15's LINEAR ×1.51 = the dwell ratio to
  <2%), the **EROSION** (`EI16(far) < 0.93·EI15(far)` at every over-penetration `J`), the **far-flank
  CLIMB FLATTENS** (rung 15 climbs >10% over `J=144→625`, rung 16 <½ that — the resolution-robust face
  of the erosion), the composition excess **vanishing AT `C_opt`** (both immediate flanks up — the notch
  survives), clamp dormancy over the pockets, cycle-untouched, require-`mixing` +
  `≤1`-of-`{pocket_quench,pdf_quench,pdf,unmixedness}` mutual-exclusivity + `PocketQuenchPDF`
  positivity/range guards + the kinked `g(C)`/`u(C)`/`core_dwell`. **DELIBERATELY asserts NO global-min
  LOCATION** (it is within the quadrature/tail/`C_e` ambiguity — contrast rung-15's `argmin==J_opt`
  GATE 3). (Reuses a cached DP + a shared trajectory + a per-pocket bank built once.)
- `tests/test_rung17.py` — rung-17: **THE LADDER** (load-bearing: `a_mixed<1<a_bulk<a_pocket` — three
  independent physics composing at the rich `φ_p=1.5` primary, both headline predicates `hides_super_eq`/
  `ladder_monotone` hold), the **rung-14 contrast** (the same mixed-out-through-the-nozzle construction
  FIRES at `φ_p=1.0` but is DORMANT at `φ_p=1.5`, `a10>100·a15`), the **identity witnessed not gated**
  (`a_pocket/a_bulk == gap_pocket_over_bulk == ei_no_pocket_quench/ei_no_quenched` to machine precision —
  documented as algebra, the nozzle no-op), **scale-sensitivity** (the ORDERING holds structurally at
  `C_e∈{0.15,0.20}` while `a_bulk` AND the gap MOVE >5%, `a_mixed_out` scale-independent — firing in-band),
  **reduce-to-components** (`x_no_bulk`/`ei_no_quenched`/`a_bulk`/`ei_no_pocket_quench` are the rung-11/14/16
  values bit-for-bit — it composes, never recomputes), cycle-untouched, clamp dormancy at station 4
  (`max_a_quench<1`), and the guards (requires-equilibrium-gas / requires-both-configs / back-pressure).
  (Reuses a cached DP + cached clamp results per `C_e`.)
- `tests/test_rung18.py` — rung-18: three reduces (`transported=None` prior path untouched + fields None;
  `Da_opt→∞` perfect-mixing recovers the well-mixed point value = the kinked notch; `g_ceiling→0` point
  value), **THE NEGATIVE RESULT** (`_transport_variance` with mean-field `ω(J)` const/√J/J ⇒ monotone/flat
  `g(J)`, NO interior optimum; the spatial `ω(C)` ⇒ an interior optimum AT `J_opt=16` — the optimum ⟺ the
  spatial `S`), the **DERIVED ceiling** (`==(ξ_p−ξ̄)/(1−ξ̄)` from `φ_p`, `<g_max=0.3` by >4×, J/C_e-
  independent; the `φ_p>φ_overall` RQL guard), the **RESIDUAL floor** (`g(C_opt)>0` ⇒ `ei_no_transported`
  elevated well above the point value, min AT `C_opt` with both flanks up), **KINK-non-genericity** (the
  transported width's one-sided slopes →0 vs the kink corner; the basin rounds `O(1)` one step off `J_opt`
  while the kinked ideal-bell notch dives `≫10³×`), cycle-untouched, and the guards (requires-`mixing` /
  ≤1-of-five / `TransportedPDF` positivity). (Reuses a cached DP.)
- `tests/test_rung19.py` — rung-19: the LOAD-BEARING reduce (`super_eq_o=False`+`prompt=None` ⇒
  bit-for-bit the prior rung — the `o_multiplier=1.0` integrator call byte-identical, the default
  `thermal_nox`/`zoned_nox` fields the rung-7 baseline `m=1.0`/`ei_no_prompt=0.0`), the **super-eq units
  cross-validation** (Westenberg `[O]_eq`/`comp["O"]∈[0.94,0.99]` across a (φ,T) grid), **super-eq is
  T-driven not rich** (`m(T)` φ-independent, `∈[1.15,1.55]`, decreasing, `→1` as T→∞; the EI lift `==m(T)`
  to ~1%; the ABSOLUTE lift collapses on the rich flank), the **prompt f(φ) shape** (peak ≈φ1.24, negative
  past φ1.65, EI clamped ≥0), **prompt survives where thermal dies** (prompt/thermal strictly increasing
  0.24→455 across φ_p 0.8→1.5), the **T-sensitivity discriminator** (thermal ×584 / prompt ×21 > 10×), the
  **summed trace guard** (both channels `Σ x_NO<0.02`), and the `PromptNO` positivity/calibratable-`phi_ref`
  guards. (Reuses a cached DP.)
- `tests/test_rung20.py` — rung-20: the LOAD-BEARING reduce (`super_eq_o=False` ⇒ bit-for-bit the prior
  rung — a direct `_quench_no` reduce + the bulk/per-pocket/clamp fields identical), **THE MODEST
  PEAK-CONCENTRATED LIFT** (the bulk-quench lift `∈(1.10,1.25)`, `≥ m(T_peak)` — a formation-weighted
  average of a T-decreasing `m` floored at the hottest point — and **strictly `<` the rung-19 primary
  lift**, `T_peak>T_primary`), **CLAMP DORMANT** (`max_a_quench<1` with the lift — super-eq O is not the
  burner-clamp lever), **THE CERTIFIED SPINE** (the rung-17 `a_bulk`/`a_pocket` **rise** while the
  denominator `x_no_e_exit` is **bit-identical**; ordering + `a_mixed<1` + the rung-17 predicates survive),
  **prompt-through invariance** (`ei_no_quenched_total = ei_no_quenched + ei_no_prompt`, `None` for the
  ideal quench), the **combine guard** (**CONSCIOUSLY INVERTED by rung 21**: `super_eq_o` +
  `{pdf,pdf_quench,transported}` — which rung 20 forbade — must now SUCCEED; still combines OK with
  `mixing`/`unmixedness`/`pocket_quench`), and the **load-bearing FLOOR** (raw `m(1200 K)>2`, floored
  `m(1500 K)<2` — the standing `1≤m≤2` trajectory assert). (Reuses a cached DP + cached clamps per flag.)
- `tests/test_rung21.py` — rung-21: the LOAD-BEARING reduce (`super_eq_o=False` ⇒ bit-for-bit rungs
  13/15/18 — a defaulted-kwarg helper reduce + a production `ei_no_pdf` reduce), **THE MODEST
  PEAK-CONCENTRATED LIFT** (the ideal-bell `⟨EI⟩_pdf` lift `∈(1.10,1.20)`, **strictly `<`** the primary
  ×1.28 and `<` the deep-lean point-value ×1.9 — point > primary > PDF), the **LIFT DECREASES WITH
  SEGREGATION** (more variance ⇒ more stoich-peak weight ⇒ smaller fractional lift), the **HYBRID
  RESOLVED** (`ei_no_pdf_quench` composite lift `∈(1.10,1.20)` and **between** term1 (bulk) and term2
  (ideal-bell) lifts; `super_eq_o` combines with `pdf`/`pdf_quench`/`transported` with no raise — the
  inverted rung-20 gate), the **g→0 CONSISTENCY** (lifted `ei_no_pdf` at `C_opt` == the super-eq-O
  well-mixed point value), and **SHAPE PRESERVED** (eq-O and super-eq-O J-sweeps both minimise AT
  `J_opt` — a consistency lift, not a relocated optimum). (Reuses a cached DP + two bells built once.)
- `tests/test_rung22.py` — rung-22: the reduce (`spatial=None` exact prior path + fields None; the
  primary diagnostic `ei_no`/`x_no_mix` bit-identical), **THE COLLAPSE** (the load-bearing inversion: the
  `g_min` VALUE geometry-independent across `2×`-independent `S`/`H`; `J_opt` shifts EXACTLY as `(H/S)²`;
  the argmin `C` == the closed form `1/(4k_p²)`; **NO `C_opt` knob** — `SpatialPDF(C_opt=…)` is a
  constructor error, `C_opt()` a derived property; robust to `k_p`), the **rung-18 tie** (`g_spatial <
  g_ceiling` at under/at/over-penetration), the **EMISSIONS honesty** (`C_opt` a LOCAL `⟨EI⟩` min — both
  immediate flanks up — but the GLOBAL min at max segregation over a wide `J`-sweep; the derived floor
  sits just below the ideal-bell hump peak), grid convergence (`ny=nz∈{32,48,64}`), cycle-untouched, and
  the guards (requires-`mixing` / ≤1-of-six / RQL `ξ_p>ξ̄` / `SpatialPDF` positivity). (Reuses a cached DP;
  the collapse gates use the bell-free helper directly.)
- `tests/test_rung23.py` — rung-23: the reduce (`spatial_dwell=None` exact prior path + fields None; the
  terminal (`t=τ_mix`) field reproduces rung-22's `g_spatial`, `_spatial_dwell_field==_spatial_segregation`;
  the production `g_spatial_dwell` == `SpatialPDF`'s width), **THE CORRELATION SIGN** (the load-bearing
  positive: the matched-mean twin — `τ(ξ)` spectrum vs a scalar `⟨τ⟩` — gives `corr_ratio>1`, the
  correlation ADDS NO; **ONE-SIGNED across `τ_mix ×0.2–×5`** with `max_a<1` (formation-limited); **CONCENTRATED
  under-penetration** (`corr@J4 > corr@J16 > 1`)), the **rung-18 tie** (`g_spatial < g_ceiling`), clamp
  dormancy (`max_a_quench<1`), cycle-untouched (`primary.ei_no`/`x_no_mix` bit-identical), the
  **NON-CIRCULARITY signature** (**NO `C_opt` knob** — `SpatialDwellPDF(C_opt=…)` a constructor error,
  `C_opt()=1/(4k_p²)` derived), a helper-vs-production pin, and the guards (requires-`mixing` / ≤1-of-seven
  / `SpatialDwellPDF` positivity). **DELIBERATELY asserts NO emissions global-min LOCATION** (rung 16 already
  declined it; the derived `τ` trend rides on the un-anchored `τ_mix`). (Reuses a cached DP + a per-pocket
  bank built once; the correlation gates re-run the cheap NO-only RK4 per dwell.)
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
  so a realistic zoned exhaust is `max_a`≈250 super-equilibrium at the exit, where rung-10's clamp fires),
  and the rung-15 PDF-through-quench panel (a `J`-sweep: the **finite floor** pinned **AT `C_opt`** with
  the far over-penetration flank **CLIMBING** — the restored dwell — vs rung-13's ⟨EI⟩13 **descending**;
  the non-monotone two-mechanism over-flank; plus the **stoich-mean sign-reversal** columns that certify
  term 2 is genuine composition work, not rung 12 in disguise), and the rung-16 per-pocket PDF-quench
  panel (a `J`-sweep: `⟨EI⟩15` vs `⟨EI⟩16` with the **erosion %** column — rung-15's LINEAR far-flank
  **climb** vs rung-16's SUBLINEAR **flat** far flank, `×1.51` vs `≈×1.3` term-2 growth — the
  cooling-limited erosion into near-degeneracy with the `C_opt` notch; with the **honest scope** stated
  loudly: the global-min location is NOT claimed, the clamp is dormant), and the rung-17 exhaust-NO
  clamp-ladder panel (the rich `φ_p=1.5` design point: the three-model ladder MIXED-OUT `a≈0.02`
  **DORMANT** → BULK-QUENCH `a≈3.4` **FIRES** → PER-POCKET `a≈13.6` **FIRES harder**, all through the
  same nozzle collapse; the **rung-14 contrast** `φ_p=1.0→a≈250` vs `φ_p=1.5→a≈0.02`; a `C_e` sweep
  showing the **ordering is structural** while magnitudes move; with the **honest scope** stated loudly:
  the identity is algebra, the firing is in-band not universal (fast quench → dormant), no magnitude is
  pinned, the clamp is dormant at station 4, and rung 17 is a synthesis of rungs 11/16/14 — not new physics),
  and the rung-18 transported-variance panel (the **DERIVED** two-stream ceiling `0.0675` vs rung-13's
  free `g_max=0.3` at 4.4×; the **NEGATIVE-RESULT** table — the real variance ODE giving monotone/flat
  `g(J)` for mean-field `ω(J)` const/√J/J vs an interior optimum AT `J_opt` only for the spatial `ω(C)`,
  the optimum appearing **only** once `S` enters; and the **shape** — the transported width's smooth
  ELEVATED basin vs the imposed kink diving to the well-mixed floor at `C_opt`, the residual floor
  `g(C_opt)>0`; with the **honest scope**: the optimum LOCATION is imposed (0-D can't derive it — the
  spatial/CFD PDF stays the ceiling), only the ceiling/floor/sharpness are what transport buys),
  and the rung-19 super-eq-O & prompt-NO panel (the equilibrium-O **lower bound** lifted two ways — a
  **φ_p sweep** with the thermal (equilibrium-O), super-eq-lifted, and prompt EI columns + the
  prompt/thermal ratio, showing super-eq O **collapsing on the rich flank WITH thermal** (0.24→455 ratio
  as thermal dies but prompt persists — 'prompt survives where thermal dies'); the **m(T) table** (the
  T-only Westenberg multiplier, φ-independent, decreasing → the 'T-driven not rich' first fail); and the
  **T-sensitivity discriminator** thermal ×584 (double exp) vs prompt ×21 (single) ≈ 28× milder; with the
  **two concessions stated loudly**: the prompt magnitude is IMPOSED (only the φ-shape + directional ratio
  certified), the super-eq ratio is semi-empirical (cross-validated to ~5%, the units gate)),
  and the rung-20 super-eq-O-through-the-quench panel (the finite-quench lower bound lifted — the
  **effective-lift table** (`ei_no_quenched` **×1.17**, `ei_no_pocket_quench` ×1.16, vs the rung-19
  **primary** ×1.28), the **why-it-inverts** note (the re-making peaks at the hottest crossing `T_peak`
  where `m(T)` is MINIMAL, so the quench lift is peak-concentrated and *smaller* than the primary), and
  the **certified-spine** clamp table (`a_bulk 3.27→3.83`, `a_pocket 11.06→12.87` rising while the
  thermodynamic denominator `x_no_e(T9)` is **bit-identical**, the clamp still dormant at station 4
  `max_a<1`); with the **honest scope**: the super-eq ratio stays semi-empirical (the lifted `a` is
  better-justified but not pinned), prompt rides as an invariant EI kept OUT of `a`, and the ideal-bell
  PDF integrals deliberately stay eq-O lower bounds in rung 20 (lifted in rung 21)),
  and the rung-21 ideal-bell-PDF-lift panel (the last eq-O seam discharged — the **effective-lift table**
  (`ei_no_pdf` **×1.14**, `ei_no_pdf_quench` ×1.16, `ei_no_transported` ×1.16), the **why-it's-peak-
  concentrated** spread (deep-lean point value **×1.91** with ≈0 EI > primary **×1.28** > EI-weighted
  `⟨EI⟩_pdf` **×1.14** — the bell peak is hotter than the flame, so `m` is smallest where the NO is), and
  the **hybrid-resolved** note (the `pdf_quench` composite ×1.156 sitting **between** term1 (bulk ×1.170)
  and term2 (bell ×1.144) — the measured proof both lift, rung 20's forbidden combination now valid);
  with the **honest scope**: a shape-preserving consistency lift (optimum LOCATION / `(H/S)²` shift /
  sign reversal unmoved), the super-eq ratio semi-empirical, prompt a primary-only invariant EI),
  and the rung-22 resolved-cross-plane / spatial-PDF panel (the INVERSION of rung 18 — **(1) THE COLLAPSE
  table**: vary `S`/`H` independently, the `g_min` VALUE identical (`0.0182`) everywhere while `J_opt`
  shifts EXACTLY as `(H/S)²`, `C_opt=1/(4k_p²)≈2.5` an OUTPUT with no `C_opt` fed in; **(2) the rung-18
  tie** `g_spatial < g_ceiling=0.0675` at under/at/over-penetration; **(3) the honest emissions curve** —
  `C_opt` (J=16) a LOCAL `⟨EI⟩` min (immediate flanks J=9/25 up) but the GLOBAL min at max segregation
  (J=400, the far over-penetration flank descending below the narrow `C_opt` basin — the derived floor
  `≈0.018` sits just below the ideal-bell hump peak `≈0.021`); with the **honest scope**: the VALUE
  `C_opt≈2.5` rides on the semi-empirical `k_p` (only the collapse + `(H/S)²` shift derived), rung 22
  derives the WIDTH not the DWELL (rung-16 kink imported), the field is a Gaussian-plume cartoon feeding
  the β-PDF closure — a real PDF-transport/CFD cross-plane stays the ceiling),
  and the rung-23 derived-dwell-spectrum panel (completing rung 22's partial closure — **(1) THE ξ–τ
  CORRELATION table**: a `J`-sweep with the derived `τ_mix`, `⟨τ⟩`, the correlated `ei_corr` vs the
  matched-mean `ei_mean`, and `corr/mean`>1 EVERYWHERE (largest under-penetration, fading toward `C_opt`)
  with `max_a<1` — the correlation ADDS NO, one-signed while formation-limited, the physics rung-16's
  SCALAR `τ_core` cannot express; **(2) THE DIVERGENCE table**: rung-16's imposed `τ_core` GROWS
  off-optimum vs rung-23's derived `⟨τ⟩` FALLING ∝1/√J, ~3×–70× apart and opposite-trending, BOTH
  un-anchored — an honest split, not "rung 16 is an artifact"; with the **honest scope**: rung 23 derives
  the correlation's SHAPE (sign + concentration), the absolute magnitude/trend rides on the un-anchored
  `τ_mix`, the emissions `C_opt` pin is NOT recovered (rung 16 already declined the global-min location),
  and a locally-resolved mixing time — which would let rung 17 claim a firing MAGNITUDE — stays the ceiling).

## Commands
- Run the model:  `python main.py`
- Run tests:      `pytest`  (or `python tests/test_rung2.py`, etc.)
- Install deps:   `pip install -r requirements.txt`  (matplotlib only)

## Stack
Python (standard library) + matplotlib for the plot. No other dependencies.
