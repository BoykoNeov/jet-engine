# Rung-12 external anchors — spatial unmixedness: the two-stream variance layer that recovers the Holdeman optimum

Verified scalings + a machine-checked worked example that anchor the **spatial-unmixedness**
model (rung 12): the minimal **variance** layer that finally makes the mean-field rung-11
NO-vs-`J` curve **turn back up**, recovering the classic **Holdeman dilution-jet optimum** — the
emissions minimum landing **AT** the Holdeman group `C = (S/H)·√J ≈ C_opt ≈ 2.5`. Every number was
re-derived **before any production code** (project discipline); the worked example runs on the
*existing* rung-6/7/8/9/10/11 primitives (`_equilibrium_composition`, `_primary_aft`,
`_mixed_out_T`, `_quench_trajectory`, `_thermal_no`, `_quench_no` with a schedule) — rung 12 adds
**no new chemistry and no new integrator**, only a **second stream** and a mass-weight. The
prototype lives in `M:\claud_projects\temp\rung12\proto_final.py`.

> **Read `docs/rung11-spec.md` first.** Rung 11 derived the quench *rate* from the jet momentum-
> flux ratio `J` but on a **single well-mixed core** (mean-field), so its `J`-sweep is **monotone**
> — a stronger jet only ever re-makes *less* NO. Rung 11 named the missing piece explicitly: the
> mixing **optimum** is a **spatial-variance** effect (an over-penetrating jet leaves an un-mixed
> hot near-stoich core), out of reach of a mean field, and deferred here. Rung 12 adds exactly that
> variance — the smallest closure that shows it: **two streams**. NO stays a **trace, decoupled
> diagnostic**, so the cycle is still **bit-for-bit rung 6**.

**The lesson (it completes the RQL mixing story).** Rung 11 said "quick quench = high-momentum
jet" and drew a monotone "more `J` → less NO." But real dilution jets do **not** improve without
limit: there is an **optimum** at the Holdeman number `C = (S/H)·√J ≈ 2.5` (jet spacing `S`, duct
height `H`). Mixing is most **uniform** near `C_opt`; **under**-penetration (low `C`) leaves the
jet hugging the wall with a hot core it never reaches, and **over**-penetration (high `C`) slams
the cold air onto the far wall (or collides opposed jets in the centre), again leaving an
under-mixed hot pocket. **Both** flanks strand a near-stoichiometric core that **misses the fast
jet mixing and lingers** — dwelling at the NO-bell peak and re-making the NO the rich primary
avoided. A mean field has no such pocket (no variance); rung 12 restores it as a **second stream**
and the NO-vs-`J` curve **turns back up**, with the minimum **at the Holdeman optimum**.

> **Honest scope (say it at the headline, not just the footnote).** The optimum's *location* is an
> **input**, not a prediction: we place the unmixedness kink at Holdeman's empirical `C_opt≈2.5`, so
> the two-stream variance *reproduces* an emissions optimum **there** (a calibrated model at this
> tower's altitude — `C_e`, `τ_q` were un-pinned too). The **certified** content is the both-flanks
> **turn-up** and the **`(H/S)²` shift** of the min with the spacing — not a *derivation* of 2.5.

---

## 1. The mechanism (numbers-before-code — and the two things that make it land at C_opt)

**Two streams, mass-weighted — the CORE carries the off-optimum penalty (in two ways); the bulk is
a fixed mean-field reference:**

```
EI_total(J) = (1 − w)·EI(τ_mean)  +  w·EI(τ_core)
```

- **BULK** (fraction `1 − w`): the rung-11 mean field, quenched at the jet time
  `τ_mean(J) = H/(C_e·√J·U_c) ∝ 1/√J` — what a mean field says: monotone-**falling**, and NOT a
  function of `C_opt`. This is the fixed **reference** curve (`ei_no_quenched`), still descending at
  `J = 100`.
- **CORE** (fraction `w`): the under-mixed pocket that **misses the jet** and lingers, quenched at
  `τ_core(C) = τ_res·(1 + b_u·u)` — an **ABSOLUTE** dwell (`~ τ_res`, a few ms, NOT the vanishing
  jet time, so its NO penalty **survives `J → ∞`**) that **grows** off-optimum (`b_u`). So off the
  optimum the core worsens **two ways**: more gas segregates (`w↑`) AND it lingers longer (`τ_core↑`).
- **The unmixedness** `u(C) = |ln(C/C_opt)|` — an **L1 (KINKED)** distance from the optimum, `0`
  at `C_opt`, symmetric in `ln C`. It drives both the segregated fraction `w = min(w_max, k_u·u)`
  and the core dwell.

**Why the two design choices, both load-bearing:**

1. **The core dwell is ABSOLUTE, not a multiple of `τ_mean`.** The naïve instinct — a
   *mean-preserving spread* of `τ_q` justified by convexity (Jensen) — is **wrong**: with the
   equilibrium clamp **dormant** (max_a ≪ 1), NO accumulates as **rate × dwell**, so `EI ∝ τ_q`
   (the rung-10 reduce table: `EI/τ_q` = 1.107 → 1.092 → 1.083, mildly *concave*). A spread adds
   ≤ 0; a *multiplicative* core `τ_core ∝ τ_mean ∝ 1/√J → 0` vanishes as `J → ∞`. An **absolute**
   `τ_core` keeps the penalty finite (and here growing) at strong jets — that is what turns the
   curve up.

2. **The unmixedness is KINKED (`|ln|`), not smooth (`ln²`), so the min lands AT `C_opt`.** With a
   smooth (parabolic) `w`, `w'(C_opt) = 0`, so at `C_opt` the total derivative is just the bulk's
   `dB/dlnC < 0` (still falling) — the min drifts to a **stronger** jet than the uniformity
   optimum. The **kink** gives `w` a non-zero slope at `C_opt`, so the turn-up starts **there** the
   moment the unmixedness beats the penetration benefit:

   ```
   k_u·[EI(τ_core) − EI(τ_mean)] > EI(τ_mean)      at C_opt   (an emissions optimum EXISTS at C_opt)
   ```

   That inequality is exactly the condition for an emissions optimum to exist at the uniformity
   point (if penetration always won, there would be no optimum). The default `k_u = 2.5` clears it,
   so the **EI-min pins at `C_opt` for ALL `S`** ⇒ `J_min = J_opt = (C_opt·H/S)²`, which shifts
   **exactly as `(H/S)²`** — the Holdeman group made literal.

`w → 0` at `C_opt` ⇒ the optimum point sits **EXACTLY** on the rung-11 mean-field curve (a clean
invariant); `k_u = 0` ⇒ `w ≡ 0` ⇒ bit-for-bit the mean field at *every* `J`.

---

## 2. The worked example (`main.py` design point)

**Tt3 = 583.5 K, Tt4 = 1500 K, far = 0.0272 (φ=0.402), p = 7.47 bar; rich primary φ_p = 1.5**
(primary AFT ≈ 2110 K, α ≈ 0.268; the τ-independent trajectory peaks at ≈ 2452 K at β≈0.16 — the
stoich crossing, reused verbatim from rung 10/11). Default `Unmixedness`: `τ_res = 2.5 ms`,
`k_u = 2.5`, `b_u = 1.0`, `w_max = 0.7`, `C_opt = 2.5`, entrainment `shape_n = 2`, `C_e = 0.20`.

**(a) The turn-up — `S = 0.0625 m` (uniformity optimum `J_opt = (C_opt·H/S)² = 16`):**

| `J` | `C=(S/H)√J` | `w(C)` | `EI_bulk` (mean field) | **`EI_unmixed`** |
|---|---|---|---|---|
| 4   | 1.25 | 0.700 | 2.059 | **2.445** |
| 9   | 1.88 | 0.700 | 1.376 | **1.805** |
| 16  | 2.50 | 0.000 | 1.034 | **1.034**  ← **EI-min, AT C_opt** (`w=0`, on the mean-field curve) |
| 25  | 3.12 | 0.558 | 0.828 | **1.420** |
| 36  | 3.75 | 0.700 | 0.690 | **1.726** |
| 49  | 4.38 | 0.700 | 0.592 | **1.862** |
| 64  | 5.00 | 0.700 | 0.518 | **1.982** |
| 100 | 6.25 | 0.700 | 0.415 | **2.190** |

`EI_unmixed` **FALLS to a minimum AT `J=16` (`C=C_opt=2.5`) then RISES monotonically** — the
recovered Holdeman optimum. The mean-field `EI_bulk` (rung 11) is still monotone-falling at `J=100`
(0.415) — the **variance** is what turns the total up (to 2.190, ~2× the minimum). (`ngrid=80`; the
shape settles by `ngrid≈32`, so the tests use 32.)

**(b) The optimum sits AT the Holdeman group — shrink `S` and it MOVES as `(H/S)²`:**

| `S` (m) | `J_opt = (C_opt·H/S)²` | EI-min lands at `J` |
|---|---|---|
| 0.0625 | 16 | **16** |
| 0.0500 | 25 | **25** |

The EI-min lands **on `J_opt`** for both spacings (the kinked unmixedness pins it at `C_opt`), so
the emissions optimum shifts **exactly as `(H/S)²`** (16 → 25 = (0.0625/0.05)²). This is the
discriminating check that rung 12 recovers the **Holdeman group `(S/H)√J`**, not a bare dip in `J`.

**(c) The core penalty survives strong jets and grows off-optimum.** `EI_core = EI(τ_core(C))` is
**minimised at `C_opt`** (τ_core = τ_res there) and larger on both flanks; at `J = 100` it far
out-emits the fast bulk (the core dwell ≈ 4.8 ms vs τ_mean ≈ 0.67 ms). That is the load-bearing
physics: an absolute, growing core dwell keeps the turn-up alive as the jet strengthens. `max_a`
stays ≈ 0.04–0.07 ≪ 1 across the sweep — the dropped clamp is dormant-on-numbers at this lean point.

---

## 3. The reduce-to-rung-11 gate (exact by construction)

The production reduce is a **short-circuit**, not an empirical limit: `zoned_nox` gains an
`unmixedness` parameter (an `Unmixedness` config) defaulting to `None`, and **requiring** a
`mixing` (it needs the jet's `J` and duct `H`). With `unmixedness=None` the path is **exactly
rung 11** (single mean-field quench); every existing rung-1..11 call — the whole suite, `main.py` —
is untouched and stays **bit-for-bit rung 11** (hence rung 6, cycle untouched). Second level:
`Unmixedness(k_u=0)` ⇒ `w(C) ≡ 0` ⇒ the two-stream total collapses onto the mean-field bulk at
**every** `J`, bit-for-bit. And at a jet whose group is **exactly** `C_opt`, `w = 0`, so the total
sits precisely on the rung-11 curve.

---

## 4. The invariants (discriminating checks, not book digits)

- **Reduce is exact** (§3): `unmixedness=None` is the rung-11 path; `k_u=0` is bit-for-bit the
  mean-field bulk; `C=C_opt` ⇒ total = bulk.
- **The `J`-sweep TURNS BACK UP** (§2a): non-monotone — falls then rises, an interior minimum. The
  load-bearing rung-12 result (a monotone curve fails it). The mean-field bulk is still falling.
- **The optimum is AT the Holdeman group `C_opt`** (§2b): `J_min = J_opt = (C_opt·H/S)²`, shifting
  **exactly** as `(H/S)²` when `S` changes (the kink pins it at `C_opt` for every `S`).
- **The core penalty survives `J → ∞` and grows off-optimum** (§2c): `EI_core` at a strong jet far
  exceeds the fast bulk, and is minimised at `C_opt` — the reason the turn-up persists (a J-scaled
  core would vanish and the curve would stay monotone).
- **`w(C)` is the unmixedness**: 0 at `C_opt`, rising (kinked) on both flanks, symmetric in `ln C`,
  capped at `w_max`.
- **Clamp dormancy persists** (`max_a ≈ 0.04–0.07 ≪ 1` across the sweep) — the dropped equilibrium
  cap stays correct-on-principle, dormant-on-numbers at this lean point (carried from rung 10/11).
- **`K`-check + trace guards bind along the trajectory** (reused unchanged from rung 10/11).
- **Cycle bit-for-bit rung 6** (NO/N never enter `_equil_solve`; `zoned_nox` is a pure diagnostic;
  the variance layer is opt-in via `unmixedness`).

---

## 5. What stays UN-anchored / deferred (state it plainly)

- **This is a TWO-STREAM (bulk + core) closure** — the *minimal* variance model, not a resolved PDF
  of the mixing field. It captures that *some* gas dwells far longer than the mean (the physics the
  mean field misses) and that both the segregated fraction and its dwell are governed by the
  Holdeman group; it does **not** resolve the spatial structure (a CFD/PDF-transport model would).
  The **turn-up**, the **optimum AT `C_opt`**, and the **`(H/S)²` shift** are the certified content
  — the same "shape, not book digits" altitude the whole tower is built at.
- **The absolute knobs are order-of-magnitude / un-anchored** — `S`, `τ_res`, `k_u`, `b_u`,
  `w_max`, `C_e`, `H`, `U_c` (like `α`, `φ_p`, `τ`, `τ_q`). `C_opt ≈ 2.5` is Holdeman's uniformity
  value; the *pin* at `C_opt` is a modeling choice (the kink + `k_u` above the existence threshold),
  disclosed — not an empirical fit to a book EI. What is certified is the turn-up + the optimum at
  the Holdeman group + the `(H/S)²` scaling. `EI_total` is **not** a function of `C` alone (the bulk
  rides `J` directly); it is the optimum *location* that pins at `C_opt`.
- **The core uses the same entrainment schedule shape as the bulk** (`mixing.schedule`) — the
  slower, more diffusive mixing of the missed pocket is a residual choice; the **dwell** (`τ_core`),
  not the shape, drives the penalty (shape is ~2× per rung 11, dwell is orders).
- **Super-equilibrium O / prompt (Fenimore) NO** (rung-7 seam) still deferred — and matters **most**
  in the rich primary **and** the radical-rich mixing shear layer that *is* this under-mixed core,
  so even the two-stream spike is an equilibrium-O **lower bound** (carried from rungs 7–11).
- **The soot bound (φ_p ≤ 2.0)** carried from rung 9 unchanged.
- **Equilibrium-vs-frozen nozzle expansion** (the rung-6 seam) still open.

---

## Sources
- J. D. Holdeman, "Mixing of multiple jets with a confined subsonic crossflow," *Prog. Energy
  Combust. Sci.* 19 (1993) — the `C = (S/H)·√J ≈ 2.5` **uniformity optimum** for dilution jets in
  crossflow (the group this rung recovers; a *uniformity* criterion, correctly framed as a variance
  effect and **now** modeled — with the emissions optimum landing at `C_opt` — rather than deferred).
- A. H. Lefebvre & D. R. Ballal, *Gas Turbine Combustion* — dilution-zone jets, the momentum-flux
  ratio `J`, the under-/over-penetration failure modes, the RQL quick-quench and its residence times.
- R. J. Margason, "Fifty years of jet in cross flow research" (AGARD) — the `y/d ∝ √J` penetration
  scaling that puts `√J` in the Holdeman group.
- S. R. Turns, *An Introduction to Combustion*; J. B. Heywood, *ICE Fundamentals* — the NO-vs-φ bell
  and super-equilibrium NO freeze on cooling (carried from rungs 7–11); segregation / unmixedness as
  a driver of NO above the mean-field value.
