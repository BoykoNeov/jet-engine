# Rung 15 — The PDF through the finite quench: the resolved mixture-fraction β-PDF carried through the rung-10/12 dwell chain (composition variance AND dwell, combined)

Rungs 11–13 built the dilution-mixing story one mechanism at a time and, by design, kept them
**isolated**. Rung 12 had the **dwell** effect (an absolute, off-optimum-growing core residence — a
**TIME** mechanism; the over-penetration flank *climbs*) but a **two-lump** composition split. Rung 13
had the **composition** effect (a mean-preserving β-PDF of mixture fraction — a **COMPOSITION**
mechanism; the optimum *location* pinned AT `C_opt`) but on the **ideal** bell — it **dropped the
quench**, so its optimum minimum collapsed to the well-mixed lean value **≈ 0** and its
over-penetration flank **descended** (bimodal PDF). Rung 13 named its own successor out loud: **carry
the PDF through the finite quench**, so the two combine. Rung 15 is exactly that.

**THE RESULT (the ≈0 floor becomes finite bulk NO; the descending flank climbs).** Rung 15 carries the
resolved β-PDF through the rung-10/12 `_quench_no` dwell chain: the ≈0 rung-13 optimum floor **becomes
the finite bulk quench NO** (the rung-11 mean field), and the rung-13 **descending** far-flank
**climbs again** (the dwell restored). The composition mechanism — and its **stoich-mean sign
reversal** — survives intact. So rung 15 is **distinguishable from BOTH parents**: it is not rung 13
(it has the finite floor and the climbing flank) and it is not rung 12 (its variance samples the
nonlinear bell, so it reverses sign at a stoich mean — which a lumped dwell cannot).

**THE CONSTRUCTION (additive — mean + resolved fluctuation).**

```
⟨EI⟩₁₅(J) = EI_bulk_quench(τ_mean(J))          [term 1: rung-11 MEAN-FIELD floor, present at all C]
          + D(u(C)) · ⟨EI_bell⟩(g(C))          [term 2: rung-13 β-PDF integral × a rung-12 dwell]
```

Term 1 is the finite floor rung 13 lacked (the mean-field quench transient). Term 2 is the resolved
composition variance rung 12 lacked (the rung-13 `_pdf_mean_ei`, reused verbatim) — scaled by an
**off-optimum-growing dwell factor** `D(u) = τ_res·(1+b_u·u)/τ_ref` (rung-12's `core_dwell` as a
dimensionless rescaling of the bell's reference-time NO to the pocket's actual lingering dwell, exact
while `EI ∝ τ` in the dormant-clamp regime). **Zero either term and rung 15 collapses to a parent** —
that pairing is the rung.

> **Honest scope up front:** the dwell rescaling `EI_pocket = EI_bell·(τ_core/τ_ref)` is a
> **linearisation** — exact only while the clamp is dormant (`max_a ≪ 1`, so `EI ∝ τ`, which holds at
> this tower's altitude). The β-PDF is **presumed** and `g(C)`/`τ_core(C)` are **modeled** functions of
> the Holdeman group, not transported. The **certified** content is the finite floor, the min **pinned
> AT `C_opt`** (both flanks up, far flank **climbing**), the **`(H/S)²` shift**, and the **stoich-mean
> sign reversal** — the combination no single parent shows.

> **Read `docs/rung13-spec.md` and `docs/rung12-spec.md` first**, and
> `docs/plans/rung15-anchor-pdf-quench.md` (numbers-before-code: the additive construction, the
> machine-checked worked example — the finite floor, the min AT `C_opt` with the climbing far flank,
> the two-mechanism over-flank wiggle, the `(H/S)²` shift, the sign reversal, the two reduces, and **the
> documented trap** the naïve dwell-only "PDF through the quench" falls into). This file states only
> what *changes*; the Zeldovich rates, the clamp-free schedule-aware `_quench_no`, the τ-independent
> trajectory, the β-PDF quadrature (`_beta_pdf_nodes_weights`) and ideal-bell interpolator
> (`_bell_interpolator`/`_pdf_mean_ei`), the `JetMixing`/`Unmixedness`/`MixingPDF` configs and the
> Holdeman group, and the "derive before you code" / conservation-assert contract all carry over
> **unchanged**.

---

## What rung 15 adds (and what it deliberately does not)

**Adds:**

- **A PDF-through-quench config** (`QuenchPDF`) — combines the rung-13 **segregation** geometry (`S`,
  `C_opt`, `k_g`, `g_max`, `n_bell`, `n_quad`) with the rung-12 **dwell** geometry (`τ_res`, `b_u`, and
  the bell reference `τ_ref`). It **rides on** a `JetMixing` (needs `J` and `H` for `C`), so
  `pdf_quench` is passed *with* `mixing`, and is **mutually exclusive** with both `pdf` and
  `unmixedness` (three closures of the same variance physics).
- **The additive combination** in `zoned_nox`: term 1 = the rung-11 bulk quench already computed
  (`ei_no_quenched`); term 2 = `D(u(C)) · _pdf_mean_ei(…, g(C))`. `⟨EI⟩₁₅ = term1 + term2`. Rung 15
  adds **no new chemistry and no new integrator** — only the scalar dwell factor `D(u)` and the sum.
- **The payoff** — a bowl whose minimum is the **finite bulk quench NO** (not ≈0) **pinned AT `C_opt`**,
  with **both immediate flanks lifting** and the **far over-penetration flank CLIMBING** (the dwell
  restored, surviving `J→∞`), and a **non-monotone over-flank** (the two-mechanism signature: the
  composition convexity jump near `C_opt`, the dwell climb far out). `J_min = J_opt`, shifting as
  `(H/S)²`. And the **stoich-mean sign reversal** carried by term 2's nonlinear bell.
- **`main.py` panel + `NOTES.md` section + `tests/test_rung15.py`.**

**Deliberately does NOT:**

- **Touch the cycle.** NO is still trace and decoupled; the PDF-quench layer is opt-in via
  `pdf_quench`. Every cycle station is **bit-for-bit rung 6** (the whole rung 1–14 suite stays green).
  The reduce is a **short-circuit**: `pdf_quench=None` runs the exact rung-13 path (and its
  `pdf`/`unmixedness`/`mixing`/`tau_q` reduces below it).
- **Reduce bit-for-bit to rung 12 or rung 13.** Rung 15 is the **combination** of the two mechanisms,
  not either alone plus a knob. The only legitimate reduces are the **`pdf_quench=None` short-circuit**
  (code-path-identical rung 13) and the **`C_opt` (`g→0`) limit** ⇒ the **finite bulk quench NO** (the
  new reduce that separates it from rung-13's ≈0 point value). No knob turns it into a parent curve.
- **Carry the full per-pocket trajectory.** Term 2 uses the reference-bell EI **rescaled** by the dwell
  ratio (`EI ∝ τ`, dormant clamp) — not a per-pocket `_quench_no` integration. Exact here; the
  per-pocket trajectory (for a pocket that goes super-equilibrium) is the next refinement.
- **Transport the PDF.** The β shape is **presumed**; `g(C)` and `τ_core(C)` are **modeled** functions
  of the Holdeman group, not solved from a PDF-transport / mixing-frequency equation.
- **Add super-equilibrium O / prompt (Fenimore) NO** — still deferred (rung-7 seam); the ⟨EI⟩ stays an
  equilibrium-O lower bound. Held **φ ≤ 2.0** (soot bound, rung 9): the bell is 0 for φ>2.

---

## The two things that make it work (stated loudly — they ARE the rung)

**(1) Both terms are load-bearing — zero either and it collapses to a parent.** Term 1 (the mean-field
quench) is the finite floor rung 13 dropped: at `C_opt` (`g→0`) term 2 vanishes and `⟨EI⟩₁₅ =
EI_bulk_quench = ei_no_quenched`, the rung-11 mean field — a **finite** ≈1.03 g/kg, not ≈0. Term 2 (the
rung-13 β-PDF × dwell) is the resolved composition variance rung 12 lacked. The rung is the **pairing**:
the finite floor **and** the sign reversal, which no single parent shows.

**(2) Term 2 must sample the NONLINEAR bell — that is what keeps the sign, and defeats the trap.** The
naïve "PDF through the quench" (integrate a *quench-resolved* EI over a **dwell-time** distribution)
**silently rebuilds rung 12**: `EI_quench(τ)` is **linear** (`≈0.62·τ_ms`, dormant clamp), so a
distribution integrated against it is just its **mean**, the resolved distribution does no work, and
all the climb comes from `(1+b_u·u)` — literally rung-12's `core_dwell` — with the variance riding the
**concave** `EI_quench` and thus acting with the **wrong sign** (a spread *lowers* NO). Keeping term 2
as the **nonlinear bell integral** (`⟨EI_bell⟩`, peaked at stoich) is what makes segregation *raise* NO
off-stoich and *lower* it at stoich — the **sign reversal** that is the discriminator (gate 5). The
dwell factor `D(u)` only **scales** that composition signal; it never provides the sign.

---

## The equations — an additive combination, no station changes

Every cycle station is **bit-for-bit rung 6**. `zoned_nox` is the rung-8..13 flow; rung 15 only adds
term 2's scalar dwell factor and the sum when a `QuenchPDF` is passed:

```
PDF-IDEAL   (pdf_quench=None):  the exact rung-13 β-PDF (or rung-12/11/10/9 per short-circuits)  → rung 13
PDF-QUENCH  (pdf_quench=QuenchPDF(…), REQUIRES mixing=JetMixing(J,…)):
   C   = (S/H)·√J                                   Holdeman group (H, J from the jet)
   g   = min(g_max, k_g·|ln(C/C_opt)|)              segregation (KINKED; 0 at C_opt)         — rung 13
   u   = |ln(C/C_opt)|                              unmixedness (KINKED; 0 at C_opt)         — rung 12
   term1 = ei_no_quenched                           the rung-11 mean-field bulk quench (the FLOOR)
   term2 = [τ_res·(1+b_u·u)/τ_ref] · ⟨EI_bell⟩(g)   composition variance × off-optimum dwell
   ei_no_pdf_quench = term1 + term2                 → min AT C_opt (finite floor); flanks up; far CLIMBS
   g → 0 (at C_opt)  ⇒  term2 → 0  ⇒  ei_no_pdf_quench = ei_no_quenched   (finite bulk NO, NOT ≈0)
```

- **`pdf_quench` REQUIRES `mixing`** — it needs the jet's `J` and `H` to form `C` **and** the derived
  `τ_mean` for term 1 (assert). `pdf_quench=None` keeps the exact rung-13 path.
- **`QuenchPDF`, `pdf`, and `unmixedness` are mutually exclusive** — three closures of the same
  variance physics (assert at most one).
- **Standing asserts (rung-15 deltas):** the rung-7 **K-check** + **trace guard** bind at every
  trajectory `T` (term 1's `_quench_no`) and every bell node `T` (term 2's bell build); the
  **mean-preservation assert** (`⟨ξ⟩≈ξ̄`, variance) on term 2's quadrature (reused from rung 13);
  `g∈[0,g_max]`, `u≥0`, both 0 at `C_opt`; `S,C_opt,τ_res,τ_ref>0`, `k_g,b_u≥0`, `0<g_max<1`;
  `pdf_quench ⇒ mixing`; at most one of `{pdf_quench, pdf, unmixedness}`; the clamp-dormancy `max_a<1`
  gate on term 1's quench (carried from rung 10–13).

---

## Verification gates (priority order)

1. **Reduce (load-bearing, exact by construction).** `pdf_quench=None` short-circuits to the rung-13
   path *before* any rung-15 code — every existing call is **bit-for-bit rung 13** (hence 12/11/10/9/6;
   the whole rung 1–14 suite stays green). Second reduce: at `C_opt` (`g→0`) ⟨EI⟩₁₅ = the **finite bulk
   quench NO** `ei_no_quenched` to `<0.01%` (the NEW reduce vs rung-13's ≈0 point value).
2. **The finite floor (THE headline).** The optimum minimum is the mean-field bulk quench NO (≈1.03
   g/kg), NOT rung-13's ≈0. The ≈0 floor **becomes finite bulk NO** — the roadmap's named seam.
3. **The optimum pinned AT `C_opt`, both flanks up, far flank CLIMBS.** ⟨EI⟩₁₅ falls to its minimum AT
   `J_opt`; **both immediate flanks lift**; the **far over-penetration flank CLIMBS** (`144→625`:
   1.25→1.53) — NOT rung-13's descent to ≈0. A separate gate pins the **non-monotone over-flank** (the
   two-mechanism signature) and confirms the far climb is the restored dwell.
4. **The optimum is AT the Holdeman group `C_opt`.** `J_min == J_opt = (C_opt·H/S)²` — shrink `S` and
   the EI-min moves to higher `J` **exactly as `(H/S)²`** (the kink pins it at `C_opt` for every `S`).
5. **The STOICH-MEAN SIGN REVERSAL survives (THE discriminator).** `⟨EI_bell⟩` rises with `g` at a lean
   mean and **falls** at a stoich mean; term 2 inherits both. **The check a dwell-only construction
   fails** — it certifies term 2 is genuine composition work (rung 15 ≠ rung 12 in disguise).
6. **`g(C)` and `u(C)` are the Holdeman kinks** — 0 at `C_opt`, rising (kinked) on both flanks,
   symmetric in `ln C`; `g` capped at `g_max`.
7. **Cycle untouched.** Re-running the cycle after a `pdf_quench` `zoned_nox` call leaves station 4
   bit-for-bit (pure diagnostic).
8. **K-check + trace + clamp-dormancy gates bind** at every trajectory/bell node (reused from 7–13;
   `max_a < 1` on term 1's quench).

## Conservation asserts (rung-15 deltas)
Carry over rung 6/7/8/9/10/11/12/13's, plus: `pdf_quench ⇒ mixing`; at most one of
`{pdf_quench, pdf, unmixedness}`; the `QuenchPDF` positivity/range guards (`S,C_opt,τ_res,τ_ref>0`,
`k_g,b_u≥0`, `0<g_max<1`); `g∈[0,g_max]`, `u≥0` with both 0 at `C_opt`; the rung-13 mean-preservation
assert on term 2's quadrature; the rung-10 clamp-dormancy `max_a<1` on term 1's quench.

## Done when
The reduce holds exactly (`pdf_quench=None` short-circuit; rungs 1–14 green, untouched; cycle
bit-for-bit rung 6) and at `C_opt` gives the **finite bulk quench NO** (not ≈0); the emissions minimum
is the **finite floor PINNED AT `C_opt`** (both flanks lifting, the **far flank CLIMBING**) with the
**non-monotone over-flank** tested, `J_min = J_opt` **shifting as `(H/S)²`**; the **stoich-mean sign
reversal** survives (the discriminator); the K-check/trace/clamp-dormancy gates hold. `main.py` gains a
rung-15 PDF-quench panel (the `J`-sweep: the finite floor + the climbing far flank vs rung-13's
descent, the two-mechanism wiggle, the `(H/S)²` shift, the sign reversal); `NOTES.md` gains a rung-15
section (combining composition variance and dwell; the additive mean+fluctuation decomposition; the
documented dwell-only trap and why the nonlinear bell defeats it); `CLAUDE.md` scope + rung table +
deferred seams updated (PDF-through-quench **done** — with the transported/CFD PDF, the full per-pocket
trajectory, super-equilibrium O / prompt NO, and the finite-Damköhler nozzle flow still carved out).

## The rung-16+ seam (keep it additive)
Rung 15 combines the presumed β-PDF with the modeled dwell on the reference bell. Next seams, all still
additive on this substrate: (a) **the full per-pocket trajectory** — replace the `EI_bell·(τ_core/τ_ref)`
linearisation with a per-pocket `_quench_no` (matters once a pocket goes super-equilibrium, `max_a>1`,
and NO decomposes); (b) **a transported / CFD PDF** — solve `g(C)` and the dwell spectrum from a
mixing-frequency / PDF-transport equation rather than presuming β and modeling the kinks;
(c) **super-equilibrium O / prompt (Fenimore) NO** — the richer radical pool in exactly the near-stoich
pockets this PDF now resolves *and* dwell-weights, above the equilibrium-O lower bound; (d) the
**finite-Damköhler nozzle flow** between the rung-14 frozen/equilibrium bounds. Only *how the PDF is
obtained*, *what each pocket rides through*, *on what radical pool*, and *how the nozzle reacts* changes.
