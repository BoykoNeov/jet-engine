# Rung-15 external anchors — the PDF *through* the finite quench: rung-13's resolved mixture-fraction distribution carried through the rung-10/12 dwell chain, so the two mechanisms combine (composition variance + dwell)

Verified scalings + a machine-checked worked example that anchor the **PDF-through-quench** model
(rung 15): the closure that stops *isolating* the two mixing mechanisms and finally **combines** them.
Rung 12 had the **dwell** effect (an absolute, off-optimum-growing core residence — a **TIME**
mechanism, which made the over-penetration flank *climb*). Rung 13 had the **composition** effect (a
mean-preserving β-PDF of mixture fraction — a **COMPOSITION** mechanism, which pinned the optimum
*location* and, in isolation, dropped the quench so its minimum collapsed to ≈0). Rung 15 carries the
resolved β-PDF **through** the finite quench, so the ≈0 rung-13 optimum floor **becomes the finite
bulk quench NO** and the rung-13 descending far-flank **climbs again** (the dwell restored) — while
the composition mechanism (and its **stoich-mean sign reversal**) survives intact.

**THE CONSTRUCTION (additive — mean + resolved fluctuation).** The one equation:

```
⟨EI⟩₁₅(J) = EI_bulk_quench(τ_mean(J))            [term 1: the rung-11 MEAN-FIELD floor, present at all C]
          + D(u(C)) · ⟨EI_bell⟩(g(C))            [term 2: the rung-13 composition variance × a rung-12 dwell]
```

- **Term 1** is the rung-11 mean-field bulk quench (`_quench_no` at `τ_mean = mixing.tau_q`) — exactly
  the production `ei_no_quenched`. It is present at **every** `C` and is what a perfectly-mixed jet
  makes as it quenches through the stoich crossing. It is the **finite floor** the ≈0 rung-13 optimum
  was missing (rung 13 dropped this transient entirely).
- **Term 2** is the rung-13 **β-PDF integral over the ideal bell** — `⟨EI_bell⟩(g) = ∫ EI_bell(ξ)·
  P_β(ξ; ξ̄, g) dξ` (the exact rung-13 `_pdf_mean_ei`, reused verbatim), scaled by an **off-optimum-
  growing dwell factor** `D(u) = τ_res·(1+b_u·u)/τ_ref`. The β-PDF resolves *which* pockets are
  segregated and *how far off-stoich* they sit (the nonlinear bell — so the sign reversal survives);
  `D(u)` gives those segregated pockets an **absolute residence that grows off-optimum** (rung-12's
  design-choice #1 — the reason the over-penetration flank *climbs* and survives `J→∞`).

**WHY BOTH TERMS ARE LOAD-BEARING (this is the rung).** Term 1 is the finite floor rung 13 lacked;
term 2 is the resolved composition variance rung 12 lacked. Zero either one and rung 15 collapses to
a parent: **term 2 = 0 ⇒ rung 11** (pure mean field); **term 1 dropped ⇒ rung 13** (composition on
the ideal bell, ≈0 optimum). Neither parent shows *both* the finite floor **and** the stoich-mean
sign reversal — that pairing is the certified content.

> **A trap this rung was built to avoid (documented so it stays avoided).** The naïve "PDF through the
> quench" — integrate a *quench-resolved* EI over a **dwell-time** distribution — **silently rebuilds
> rung 12.** The measured `EI_quench(τ) ≈ 0.62·τ_ms` is **linear**, so `Σ wᵢ·EI_quench(τᵢ) = 0.62·⟨τ⟩`:
> a distribution integrated against a *linear* function is just its **mean**, and the "resolved
> distribution" does no work — all the climb then comes from the `(1+b_u·u)` dwell term, which **is**
> rung-12's `core_dwell`. Worse, the variance there rides the mildly **concave** `EI_quench`, so a
> spread *lowers* NO — the **wrong sign** vs rung-13's peaked bell. The fix (this construction) keeps
> the **nonlinear bell integral** as term 2, where variance acts with the **correct sign** (segregation
> *raises* NO off-stoich, *lowers* it at stoich). The discriminator that catches the trap is the
> **stoich-mean sign reversal** (§4) — a dwell-only construction cannot reproduce it.

Every number was re-derived **before any production code** (project discipline); the worked example
runs on the *existing* rung-6/7/9/10/11/13 primitives (`_quench_no`, `_bell_interpolator`,
`_beta_pdf_nodes_weights`, `_pdf_mean_ei`) — rung 15 adds **no new chemistry and no new integrator**,
only the additive combination of term 1 (a bulk quench already computed) and term 2 (the rung-13
integral × a scalar dwell factor). The prototype lives in `M:\claud_projects\temp\rung15\proto4.py`
(and `CONSTRUCTION.md`).

> **Read `docs/rung13-spec.md` and `docs/rung12-spec.md` first.** Rung 13 resolved the mixture-fraction
> distribution as a mean-preserving β-PDF but on the **ideal** bell (dropping the dwell). Rung 12
> resolved the dwell as an **absolute, off-optimum-growing** core residence but with a **two-lump**
> composition split. Rung 15 is the **product**: the rung-13 β-PDF integral **times** the rung-12
> dwell, **plus** the rung-11 mean-field floor. NO stays a **trace, decoupled diagnostic**, so the
> cycle is still **bit-for-bit rung 6**.

---

## 1. The mechanism (numbers-before-code)

**The two Holdeman-group functions carry all the `C`-dependence** (`C = (S/H)·√J`, the same group as
rungs 12–13):

```
g(C) = min(g_max, k_g·|ln(C/C_opt)|)     segregation (β-PDF width; KINKED, 0 at C_opt)   — rung 13
u(C) = |ln(C/C_opt)|                      unmixedness (L1 distance; KINKED, 0 at C_opt)   — rung 12
```

**Term 1 — the mean-field floor.** `EI_bulk_quench(τ_mean)` is the rung-11 clamp-free quench NO
integrated along the shared `_quench_trajectory` at the jet time `τ_mean = H/(C_e·√J·U_c)`. It falls
monotonically with `J` (∝ 1/√J band; a stronger jet escapes the stoich peak faster — the rung-11
result). At `C_opt` (`J = J_opt`) it is the finite value the emissions optimum pins to.

**Term 2 — the resolved composition variance × dwell.**
- `⟨EI_bell⟩(g)` is the rung-13 β-PDF integral over the **ideal primary bell** `EI_bell(ξ)` (peaked at
  stoich `ξ≈0.063`, small in both wings), mean-preserved at `ξ̄ = far_overall/(1+far_overall)`, width
  `g(C)`. It is **humped in `g`** (peaks near `g≈0.02`, descends as the β-PDF goes bimodal — rung-13
  §2b) and, crucially, **NONLINEAR and peaked**, so it carries the **sign reversal** (§4).
- `D(u) = τ_res·(1+b_u·u)/τ_ref` is the dwell factor: the ratio of the segregated pocket's **absolute
  residence** `τ_core(u) = τ_res·(1+b_u·u)` (rung-12's `core_dwell`) to the bell's **reference
  residence** `τ_ref` (the `tau` at which `EI_bell` was evaluated). Because `EI ∝ τ` in the
  dormant-clamp regime (`max_a ≪ 1`; measured `EI_quench(τ) ≈ 0.62·τ_ms`, linear), multiplying the
  reference-time bell EI by `τ_core/τ_ref` **rescales it to the pocket's actual lingering dwell** — a
  disclosed, physically-grounded linearisation, not a free knob. `D(u)` **grows off-optimum** (via
  `u`), which is what makes the far flank climb and survive `J→∞`.

**The knobs** (order-of-magnitude, like every rung-9..13 knob — disclosed, not fit to a book EI):
`C_opt = 2.5` (Holdeman), `k_g = 0.3`, `g_max = 0.3` (rung 13); `τ_res = 2.5 ms`, `b_u = 3.0`
(rung-12-style dwell; `b_u` is larger than rung-12's default because term 2's `⟨EI_bell⟩` caps lower
than rung-12's `EI(τ_core)`, so a steeper off-optimum growth is needed to pin the min at `C_opt` — see
§2c); `τ_ref = 3 ms` (the bell reference); `S = 0.0625 m`, `H = 0.10 m` (rung 12).

---

## 2. The worked example (`main.py` design point)

**Tt3 = 583.5 K, Tt4 = 1500 K, far = 0.02718 (φ_overall = 0.402), p = 7.474 bar; lean overall mean**
(`ξ̄ = 0.02646`). Rich primary `φ_p = 1.5` (the RQL primary, as rungs 10–12). Default kink geometry:
`C_opt = 2.5`, `k_g = 0.3`, `g_max = 0.3`, `τ_res = 2.5 ms`, `b_u = 3.0`, `τ_ref = 3 ms`,
`S = 0.0625 m` (`H = 0.10 m` ⇒ `J_opt = (C_opt·H/S)² = 16`). Bell built once (`n_bell = 200`),
quadrature `n_quad = 200`, quench trajectory `ngrid = 80`.

**(a) The finite floor at the optimum (the headline vs rung 13).** At `C_opt` (`J = 16`, `g = 0`), the
β-PDF is a delta at `ξ̄`, term 2 = `D(0)·EI_bell(ξ̄) = 1.1×10⁻⁵ g/kg` (negligible), so
`⟨EI⟩₁₅ = EI_bulk_quench = 1.034 g/kg` — the **finite bulk quench NO**, NOT rung-13's ≈0
(`1.3×10⁻⁵`). *That* is the ≈0-floor-becomes-finite-bulk-NO result the roadmap named.

**(b) The `J`-sweep — min PINNED AT `C_opt`, both flanks up, far flank CLIMBS (`b_u = 3`):**

| `J` | `C = (S/H)√J` | `g(C)` | term 1 (floor) | term 2 | **⟨EI⟩₁₅** | rung 13 (ideal bell) | rung 12 (two-stream) |
|---|---|---|---|---|---|---|---|
| 4   | 1.25 | 0.208 | 2.059 | 0.955 | 3.015 | 0.372 | 2.445 |
| 9   | 1.88 | 0.086 | 1.376 | 1.147 | 2.524 | 0.739 | 1.805 |
| **16** | **2.50** | **0.000** | **1.034** | **1.1e-5** | **1.034 ← min, AT `C_opt`** | **1.3e-5 (≈0!)** | **1.034** |
| 25  | 3.12 | 0.067 | 0.828 | 1.189 | 2.016 | 0.854 | 1.420 |
| 36  | 3.75 | 0.122 | 0.690 | 1.082 | 1.773 | 0.586 | 1.726 |
| 49  | 4.38 | 0.168 | 0.592 | 1.011 | 1.603 | 0.453 | 1.862 |
| 64  | 5.00 | 0.208 | 0.518 | 0.955 | 1.474 | 0.372 | 1.982 |
| 100 | 6.25 | 0.275 | 0.415 | 0.870 | 1.285 | 0.279 | 2.190 |
| 144 | 7.50 | 0.300 | 0.346 | 0.901 | 1.247 | 0.252 | 2.363 |
| 225 | 9.38 | 0.300 | 0.277 | 1.042 | 1.319 | 0.252 | 2.580 |
| 400 | 12.5 | 0.300 | 0.208 | 1.223 | 1.431 | 0.252 | 2.864 |
| 625 | 15.6 | 0.300 | 0.167 | 1.363 | 1.530 | (0.25) | (3.0) |

⟨EI⟩₁₅ **falls to a minimum AT `J = 16` (`C = C_opt`)** — the finite floor 1.034 — and **rises on both
immediate flanks**. The **over-flank is non-monotone (down then up)**: a shallow interior min at
`J ≈ 144` (1.247, still **20 % above** the floor), then a climb (`144→625`: 1.247→1.530). This is the
**two-mechanism signature**: just off `C_opt` the **composition convexity jump** dominates (rung-13's
sharp shoulder, term 2's `⟨EI_bell⟩` near its hump), far out the **dwell climb** dominates (rung-12,
`D(u)` growth beating the humped `⟨EI_bell⟩` descent). Both parents' fingerprints, in one curve.

> **The contrast that IS the rung.** Rung-13's over-flank **descends** to ≈0.25 (bimodal PDF, dwell
> dropped); rung-15's stays **elevated and climbs** (dwell restored). Rung-13's optimum is ≈0;
> rung-15's is the **finite 1.034**. Rung-12's over-flank climbs monotonically but from a single lumped
> core with **no sign reversal**; rung-15's term 2 samples the nonlinear bell, so it **does** reverse
> (§4). Rung 15 is a wigglier bowl than rung 12 *on purpose* — the wiggle is the second mechanism.

**(c) Why `b_u = 3` (the pin condition).** For the min to land **AT** `C_opt` (not drift to a stronger
jet), term 2's off-optimum growth must beat term 1's ∝1/√J fall on the over-flank. With `b_u = 1` the
min drifts to `J = 400` (the floor sinks below the `C_opt` value); `b_u = 2` → `J = 144`; `b_u = 3`
pins it AT `J = 16` with a 20 % over-flank margin. `b_u` is the same *kind* of order-of-magnitude dwell
knob as rung-12's (there `b_u = 1`); it is larger here only because term 2's `⟨EI_bell⟩` (capped near
0.25 at `g_max`) is a smaller lever than rung-12's `EI(τ_core)`.

**(d) The optimum sits AT the Holdeman group — shrink `S`, it MOVES as `(H/S)²`:**

| `S` (m) | `J_opt = (C_opt·H/S)²` | EI-min lands at `J` |
|---|---|---|
| 0.0625 | 16 | **16** |
| 0.0500 | 25 | **25** |

The min lands **on `J_opt`** for both spacings (the kinked `g`/`u` pin it at `C_opt`), so the emissions
optimum shifts **exactly as `(H/S)²`** — the Holdeman group, now recovered from the *combined* closure.

**(e) The stoich-mean sign reversal (the discriminator — §4).** `⟨EI_bell⟩(g)` at a **lean** mean
**rises** with `g` (segregation raises NO); at a **stoich** mean it **falls** (spreading moves mass off
the peak): `g = 0 → 0.30` gives lean `1.3e-5 → 0.252` (up) but stoich `20.72 → 0.518` (down). Term 2 =
`D(u)·⟨EI_bell⟩` inherits **both signs** — the check a dwell-only rung-12-in-disguise **fails**.

---

## 3. The reduce gates

**Rung 15 is the COMBINATION of rungs 11–13, not "rung 13 + one knob."** Two legitimate reduces:

1. **`pdf_quench=None` ⇒ code-path-identical rung 13 (hence 12/11/10/9/6).** `zoned_nox` gains a
   `pdf_quench` parameter (a `QuenchPDF` config) defaulting to `None`; with it unset the path is the
   **exact rung-13/12/11 result** (per the existing short-circuits). The whole existing suite and
   `main.py` are untouched and stay **bit-for-bit** (cycle bit-for-bit rung 6).
2. **At `C_opt` (`g → 0`) ⇒ the finite bulk quench NO** `EI_bulk_quench = ei_no_quenched` (the rung-11
   mean field), to `< 0.01 %` (the residual `D(0)·EI_bell(ξ̄) ≈ 1×10⁻⁵ g/kg`). This is the **NEW**
   reduce that separates rung 15 from rung 13 — where rung-13's `g → 0` gave the ideal-bell **point
   value ≈ 0**, rung-15's gives the **finite bulk NO**. It is the ≈0-floor-becomes-finite result.

No knob turns rung 15 into rung 12's two lumps (different closure — same as rung 13 claimed no
bit-for-bit reduce to rung 12).

---

## 4. The invariants (discriminating checks, not book digits)

- **Reduce is exact** (§3): `pdf_quench=None` is the rung-13 path (bit-for-bit); at `C_opt` ⟨EI⟩₁₅ = the
  finite bulk quench NO (`ei_no_quenched`), NOT ≈0.
- **The finite floor** (§2a): the optimum minimum is the mean-field bulk quench NO (≈1.03 g/kg), NOT
  rung-13's ≈0. The ≈0 floor **becomes finite bulk NO** — the headline seam the roadmap named.
- **The emissions minimum is PINNED AT `C_opt`** (§2b): a bowl with the minimum at `J_opt`, **both
  immediate flanks lifting** and the far over-penetration flank **CLIMBING** (`144→625`: 1.25→1.53) —
  NOT rung-13's descent to ≈0. The climb is the restored dwell (`D(u)` growth surviving `J→∞`).
- **The over-flank is non-monotone (the two-mechanism signature)** (§2b): a shallow interior min
  (≈`J=144`, 20 % above the floor) where the composition convexity jump hands off to the dwell climb.
  A *feature*: it is the fingerprint of rung-13 (near `C_opt`) and rung-12 (far out) combining.
- **The STOICH-MEAN SIGN REVERSAL survives** (§2e): `⟨EI_bell⟩` rises with `g` at a lean mean, **falls**
  at a stoich mean; term 2 inherits both. **THE discriminator** — a dwell-only construction (where
  variance rides the concave `EI_quench`) shows the wrong sign and cannot reverse. This certifies term
  2 is doing genuine *composition* work, i.e. rung 15 is not rung 12 in disguise.
- **The optimum is AT the Holdeman group `C_opt`** (§2d): `J_min = J_opt = (C_opt·H/S)²`, shifting
  **exactly** as `(H/S)²` when `S` changes.
- **Cycle bit-for-bit rung 6** (NO/N never enter `_equil_solve`; `zoned_nox` is a pure diagnostic; the
  PDF-quench layer is opt-in via `pdf_quench`).
- **K-check + trace + clamp-dormancy gates bind** along the shared trajectory (reused unchanged from
  rungs 7–13; `max_a < 1` at the design point).

---

## 5. What stays UN-anchored / deferred (state it plainly)

- **The dwell rescaling `EI_pocket = EI_bell·(τ_core/τ_ref)` is a LINEARISATION** — exact only while the
  clamp is dormant (`EI ∝ τ`, `max_a ≪ 1`, which holds here). A pocket that goes super-equilibrium on a
  slow cooling path (`max_a > 1`) would decompose NO and break the proportionality; carrying the **full
  per-pocket `_quench_no` trajectory** (instead of the reference-bell × dwell-ratio) is the next
  refinement. Disclosed, not hidden.
- **The β-PDF is a PRESUMED shape** and its width `g(C)` and dwell `τ_core(C)` are **modeled** functions
  of the Holdeman group, not solved from a PDF-transport / mixing-frequency equation. A **transported /
  CFD PDF** (predict `g` and the dwell spectrum from a mixing equation) is the deeper seam.
- **The absolute knobs stay order-of-magnitude / un-anchored** — `S`, `k_g`, `g_max`, `τ_res`, `b_u`,
  `C_e`, `H`, `U_c` (as in rungs 9–13). `C_opt ≈ 2.5` is Holdeman's uniformity value; the *pin* of the
  emissions optimum at `C_opt` is a modeling choice (the kink), disclosed. Certified: the finite floor,
  the min AT `C_opt` (both flanks up, far flank climbing), the `(H/S)²` shift, and the sign reversal.
- **Super-equilibrium O / prompt (Fenimore) NO** (rung-7 seam) still deferred — matters **most** in
  exactly the near-stoich pockets this PDF resolves *and* dwell-weights, so rung-15 ⟨EI⟩ is still an
  equilibrium-O **lower bound** (carried from rungs 7–13).
- **The soot bound (φ ≤ 2.0)** carried from rung 9 — the bell is 0 for φ>2 (O-starved, 5-species basis
  invalid), so the rich PDF tail beyond φ=2 contributes no NO.
- **The cycle-side nozzle seam** is **rung 14** (`docs/rung14-spec.md`, done); its finite-Damköhler flow
  between the frozen/equilibrium bounds is still open.

---

## Sources
- R. B. Pompei & J. B. Heywood, "The role of mixing in burner-generated carbon monoxide and nitric
  oxide," *Combustion and Flame* 19 (1972) — fuel-air **unmixedness raises NO** above the well-mixed
  value at fixed mean stoichiometry (the sign anchor, carried from rung 13).
- J. D. Holdeman, "Mixing of multiple jets with a confined subsonic crossflow," *Prog. Energy Combust.
  Sci.* 19 (1993) — the `C = (S/H)·√J ≈ 2.5` uniformity optimum (the group both `g` and the dwell ride).
- R. W. Bilger; N. Peters — the **presumed β-PDF of mixture fraction** as the standard closure for
  turbulent non-premixed combustion (term 2's object; the mean/variance parameterisation).
- A. H. Lefebvre & D. R. Ballal, *Gas Turbine Combustion* — dilution-zone jets, `J`, the RQL
  rich-burn/quick-quench, and the finite mixing/residence time that IS the dwell (term 1 and `τ_core`).
- S. R. Turns, *An Introduction to Combustion*; J. B. Heywood, *ICE Fundamentals* — the NO-vs-φ bell
  and its sharp stoich peak; NO ∝ residence in the kinetically-limited (dormant-clamp) regime (the
  `EI ∝ τ` linearisation that grounds the dwell rescaling).
