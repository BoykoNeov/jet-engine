# Rung-16 external anchors — the PDF through the finite quench, PER POCKET: retiring rung-15's linearised dwell (numbers-before-code)

Verified scalings + a machine-checked worked example that anchor the **per-pocket PDF-through-quench**
model (rung 16): the closure that **retires rung-15's one acknowledged approximation**. Rung 15's
term 2 = `D(u)·⟨EI_bell⟩(g)` rescaled each pocket's **constant-temperature** ideal-bell NO by a
**scalar** dwell ratio `D(u)=τ_core/τ_ref` — exact only while `EI ∝ τ` (the dormant clamp), which
**ignores that a lingering pocket COOLS**. Rung 16 carries **each rich-of-mean pocket through its own
finite quench** (`_quench_no` at the dwell `τ_core`), so the dwell acts **inside the cooling
chemistry**. Every number below was re-derived **before any production code** (project discipline); the
prototypes live in `M:\claud_projects\temp\rung16\` (`proto.py`…`proto8.py`, `RUNG16-CHARACTERIZATION.md`).

**THE CONSTRUCTION (additive — mirrors rung 15; only term 2's internals change).**

```
⟨EI⟩₁₆(J) = EI_bulk_quench(τ_mean(J))               [term 1: rung-11 MEAN-FIELD floor — UNCHANGED]
          + ⟨EI_pocket_quench(ξ; τ_core(C))⟩_g       [term 2: PER-POCKET quench β-PDF integral]
```

Compare rung 15's `term 2 = D(u)·⟨EI_bell⟩(g)`. The **only** change is inside term 2: the constant-`T`
bell scaled by the scalar `D(u)` becomes the **per-pocket quenched** `EI_pocket_quench(ξ; τ_core)` — the
dwell `τ_core = τ_res·(1+b_u·u)` passed straight into each pocket's `_quench_no`. **No new chemistry, no
new integrator** — it reuses the rung-10 `_quench_no` and the rung-13 β-PDF quadrature.

> **Read `docs/rung15-spec.md` and `docs/plans/rung15-anchor-pdf-quench.md` first.** Rung 16 is rung
> 15's term 2, **resolved**. NO stays a **trace, decoupled diagnostic**, so the cycle is still
> **bit-for-bit rung 6**.

---

## 1. Why the honest headline is EROSION, not a re-sharpened or un-pinned optimum

The investigation (8 prototypes, 4 advisor rounds) killed three tempting-but-wrong headlines. The
record matters more than the conclusion:

**(a) NOT "un-pins the optimum."** An early proto (default `C_e=0.15`) showed rung-16's *total* min
drifting to `J=400`, 21% below the `C_opt` notch — a dramatic "the pin was an artifact" story. **It was
a knob mismatch.** Shipped rungs 11–15 (and `test_rung15`, `main.py`) are anchored at **`C_e=0.20`**;
the default `C_e=0.15` was never the ladder's regime. `C_e` scales `τ_q=H/(C_e√J·U_c)`, so it lowers the
**floor** (term 1) but leaves term 2 (`C=(S/H)√J`, `g`, `τ_core` — all `C_e`-independent) unchanged. A
lower floor drops `total(C_opt)` **more** than the far flank (the floor is larger at low `J`), which
pins rung-15's total at `C_opt` — matching its shipped `test_rung15` GATE 3. At the anchored `C_e=0.20`
the drift is a **mild ~2%**, not 21%.

**(b) NOT "re-sharpens the pin."** A second proto (a `vals[-1]` clamp on the `φ>2` β-PDF tail) showed
rung 16 pinning *sharply* at `C_opt`. **That was a tail artifact.** The β-PDF places **6.79%** of its
weight above `φ=2` (the soot bound). Rung-15's `_ideal_bell_ei` returns **0** there (out of scope), so
rung 15 *zeroes* that tail; the clamp instead put the `φ=2` pocket's large near-stoich quench value on
all of it. A machine-checked decomposition at `J=225` isolated it exactly:

```
term2_16 DECOMPOSITION (J=225, g=0.300):     tail=ZERO (rung-15-consistent)   tail=CLAMP (artifact)
   lean (ξ<ξ̄)                                       0.0000                        0.0000
   in-grid rich                                     0.7337                        0.7337
   above-grid tail (φ>2)                            0.0000                        0.7971   ← the whole artifact
   TOTAL                                            0.7337                        1.5308
```

The **`tail=zero`** choice (identical to rung 15's `φ>2→0` scope) is the one that keeps the reduce
exact. The other defensible choice — truncate+renormalize the β-PDF at `φ=2` — lifts term 2 by only
`1/(1−0.068) = +7.3%`, so the qualitative result is unchanged; only the indefensible clamp flips it.

**(c) The robust conclusion — EROSION.** With the anchored `C_e` and the rung-15-consistent tail, the
per-pocket quench makes term 2 **sublinear** in `τ_core` and **erodes** rung-15's far-flank basin into
**near-degeneracy** with the `C_opt` notch. The global-min *location* is **not claimed** — it flips
sign across the β-PDF quadrature (~5%), the tail treatment, and `C_e` (2%→21%), all comparable to the
margin.

---

## 2. The machine-checked worked example (anchored, `C_e=0.20`, design `Tt4=1500 K`)

Design point: `far=0.02718` (`φ_ov=0.402`), `ξ̄=0.02646`, `p=7.47 bar`, `J_opt=(C_opt·H/S)²=16`.
Production `zoned_nox`, `n_bell=120`, `n_quad=160`, `quench_ngrid=80`, `quench_nsteps=700`:

```
   J     C     g    floor    EI15     EI16    t2_16   max_a
   4  1.25 0.208   2.059   3.016    2.854   0.795   0.644
   9  1.88 0.086   1.376   2.525    2.578   1.202   0.595
  16  2.50 0.000   1.034   1.034    1.034  1.6e-05  0.559   ← C_opt: term2→0 ⇒ EI16 = finite bulk floor
  25  3.12 0.067   0.828   2.017    2.141   1.313   0.587
  64  5.00 0.208   0.518   1.476    1.313   0.795   0.644
 144  7.50 0.300   0.346   1.243    0.987   0.641   0.693
 225  9.38 0.300   0.277   1.314    0.976   0.699   0.722
 400 12.50 0.300   0.208   1.425    0.980   0.772   0.758
 625 15.62 0.300   0.167   1.523    0.995   0.828   0.786
```

The curve is **bimodal**: a sharp `C_opt` notch (`1.034`), a hump peaking at `J=25` (`2.141`), then an
eroded far basin (`~0.98`). The robust, certified numbers:

- **Reduce at `C_opt` (`g→0`):** `EI16 = 1.03373` vs floor `1.03371`, **rel_err 0.0015%** — the single
  lean pocket at `ξ̄` ≈ 0, so `⟨EI⟩₁₆` = the **finite bulk quench NO** (NOT rung-13's ≈0). Design-point
  specific (`ξ̄` is lean here), exactly like rung 15's reduce (`CONSTRUCTION.md`).
- **SUBLINEAR DWELL (the mechanism):** term 2 ratio `J=144→625`: **rung 15 ×1.513** (= the
  `dwell_factor` ratio `D(625)/D(144)` **exactly** — the linearisation) vs **rung 16 ×1.293**
  (sublinear — the pocket cools).
- **FAR-FLANK EROSION (the headline):** `EI₁₆(J) < EI₁₅(J)` on the whole over-penetration flank:
  `+20.6%` (`J=144`), `+25.7%` (`J=225`), `+31.2%` (`J=400`), `+34.7%` (`J=625`).
- **The far-flank CLIMB flattens:** `EI₁₅` climbs `144→625` (`1.243→1.523`, **+22.6%** — the linear
  dwell); `EI₁₆` is flat (`0.987→0.995`, **+0.8%** — the cooling saturates). Same endpoints, opposite
  slope. This is the resolution-robust face of the erosion (the fine-vs-coarse magnitude of the
  `C_opt`-vs-far *gap* is grid-sensitive; the **slope contrast** is not).
- **Clamp DORMANT:** `max_a = 0.786 < 1` across the whole sweep (and every pocket). The rung-15↔16
  difference is **cooling within the dwell**, not super-equilibrium rollover — a `max_a` scan shows the
  clamp fires only at `τ_core ≈ 50 ms`, ~3× the maximum physical far-flank dwell (~16 ms). The
  clamp-free per-pocket integrator is the *capability* (it would roll a super-eq pocket over), dormant
  here exactly like rung 10's own clamp (`0.677`).

---

## 3. Convergence + the near-degeneracy caveat (why no global-min location is claimed)

The `tail=zero` term 2 at the far flank (`J=225`) is converged in `n_bell` but only to **~5%** in
`n_quad` (`0.699 / 0.734 / 0.711` at `n_quad = 200 / 320 / 500`) — the peaked, near-stoich pocket
integrand is harder to quadrature than rung-15's smooth bell. That ~5% is **larger** than the ~2%
`C_opt`-vs-far margin at the anchored `C_e`. Together with the tail-treatment ambiguity (`tail=zero` →
far wins by 2.4%; truncate+renorm → `C_opt` wins by 3%) and the `C_e` sensitivity (2%→21% over
`0.20→0.15`), the **global-min location is genuinely unresolved**. So the tests certify the **erosion /
sublinear slope / near-degeneracy**, and deliberately assert **no** `argmin` (unlike rung-15's GATE 3,
valid there because rung 15's far basin sits a clear ~20% above `C_opt` at fine resolution).

---

## 4. The knobs (order-of-magnitude — identical to rung 15, disclosed, not fit)

`C_opt = 2.5` (Holdeman), `k_g = 0.3`, `g_max = 0.3` (rung 13); `τ_res = 2.5 ms`, `b_u = 3.0` (rung-12
dwell); `S = 0.0625 m`, `H = 0.10 m`, `C_e = 0.20` (the anchored jet). `n_bell = 120`, `n_quad = 160`
default smaller than rung 15's 200/200 — each `n_bell` node is a full `_quench_no`, so the diagnostic
is ~`n_bell`× costlier than rung 15's single bell integral.

---

## 5. What rung 16 leaves open (the still-additive seam)

Rung 16 resolves the **dwell** per-pocket. Still additive on this substrate: (a) a **transported / CFD
PDF** — solve `g(C)` and the dwell spectrum from a mixing/PDF-transport equation rather than presuming
β and modeling the kinks; (b) **super-equilibrium O / prompt (Fenimore) NO** in exactly the near-stoich
pockets this closure now resolves *and* dwell-quenches, above the equilibrium-O lower bound; (c) a
regime where the per-pocket clamp actually **fires** (hotter `Tt4` / longer dwell — the rung-14
exhaust-NO corollary showed the clamp fires on cooling paths); (d) the **finite-Damköhler nozzle flow**
between the rung-14 frozen/equilibrium bounds. Only *how the PDF is obtained*, *on what radical pool*,
*whether a pocket goes super-equilibrium*, and *how the nozzle reacts* changes.
