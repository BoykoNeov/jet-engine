# Rung 18 — The transported-variance closure: what a 0-D variance equation CAN and CANNOT derive (the kink is imposed, the ceiling is derived)

Rungs 12–17 pin the dilution-mixing optimum with a **hand-prescribed kink**: the β-PDF segregation width
`g(C)=min(g_max, k_g·|ln(C/C_opt)|)` (and the dwell `τ_core(C)`), a sharp L1 corner on the empirical
`C_opt≈2.5`. The rung-13/16/17 specs named the successor: a **transported PDF** — derive `g(C)` from a
mixture-fraction variance equation instead of imposing it (which would also let rung 17 report a firing
*magnitude*). Rung 18 attempts exactly that — and its headline is the **honest limit** of the attempt.

**THE LOAD-BEARING RESULT (a negative one — and stronger for it).** A 0-D mixture-fraction variance
transport, integrated along the `JetMixing` trajectory, **CANNOT derive the `C_opt` optimum**. Integrate
`dg/dt = −C_φ·ω·g` from the two-stream ceiling: with any **mean-field** `ω(J)` (or the trajectory
`τ_q(J)∝1/√J`) the residual `g(J)` is **monotone/flat — no interior optimum**. An optimum appears *only*
when `ω` is given a **spatial** coverage dependence `ω(C=(S/H)√J)` peaked at `C_opt` — i.e. only once the
jet **spacing `S`** is injected by hand. The Holdeman optimum is a cross-plane **uniformity** criterion,
**irreducibly spatial**; a 0-D relaxation over a mean-field trajectory (all monotone in `J`) structurally
cannot be non-monotone. This is the project's own rung-11 result — *"mean-field ⇒ no mixing optimum (the
variance seam, rung 12)"* — made literal. So rung 18's production coverage `ω(C)` is an **explicitly
imposed** spatial closure (the honest successor of rung-13's kinked `g(C)`), and a transported/CFD PDF
that predicts the spatial pattern stays the **deferred ceiling**.

**WHAT TRANSPORT LEGITIMATELY ADDS (the certified content — lead with the ceiling).**

- **A DERIVED two-stream injection ceiling** `g_ceiling=(ξ_p−ξ̄)/(1−ξ̄)` — the max normalized variance of
  a two-delta PDF on `{0 (air), ξ_p (primary)}` at the fixed overall mean, set by the **primary richness
  `φ_p`**, NOT a free knob. `=0.0675` at `φ_p=1.5`, which exposes rung-13's `g_max=0.3` as **4.4× too
  large** (that oversize is what let rung-13's `⟨EI⟩(g)` reach its humped/descending regime).
- **A RESIDUAL-unmixedness floor** `g(C_opt)=g_ceiling·exp(−Da_opt)>0` — perfect mixing is never reached,
  so the emissions optimum is **elevated off the well-mixed value** (through the rung-13 ideal bell:
  `EIbell(C_opt)=0.895` vs the kink's `≈1.3e-5` touch-the-floor).
- **KINK-is-non-generic** — the imposed corner has one-sided slopes `±k_g/C_opt` at `C_opt`; any analytic
  mixing rate rounds it (transported one-sided slopes `→0`). The EIbell ratio one step off `J_opt`: kink
  **×5.8e4** vs transported **×1.05**. The **sharpness** of the pin (not its location) was the artifact.
- **The canonical `C_φ≈2`** scalar-dissipation decay law — the one genuinely anchored constant.

> **Read `docs/rung13-spec.md`** (the ideal-bell β-PDF — rung 18's vehicle) **and `docs/rung11-spec.md`
> §the ceiling** (mean-field ⇒ no optimum), and `docs/plans/rung18-anchor-transported-variance.md`
> (numbers-before-code: the machine-checked negative test, the derived ceiling, the residual floor, the
> reduce). This file states only what *changes*. The Zeldovich rates, the equilibrium bell primitives,
> the `_pdf_mean_ei` quadrature, the `JetMixing`/`MixingPDF` configs + Holdeman group, and the "derive
> before you code" / conservation-assert contract carry over **unchanged**.

---

## What rung 18 adds (and what it deliberately does not)

**Adds:**

- **A transported-variance config** (`TransportedPDF`) — rides on a `JetMixing`, **≤1-of-five** mutually
  exclusive with `unmixedness`/`pdf`/`pdf_quench`/`pocket_quench` (five closures of the same variance
  physics). It **derives** `g_ceiling` from `φ_p`, integrates the variance ODE with an **explicitly
  imposed** coverage `ω(C)=ω_opt·exp(−ln²(C/C_opt)/2w_cov²)`, and returns `g(C)` — a smooth basin with the
  residual floor. Knobs: `S`, `C_opt`, `C_phi=2.0` (anchored), `Da_opt` (optimum Damköhler — residual
  floor depth), `w_cov` (coverage breadth), grid sizes. `g_ceiling` and `C_φ` are **not** fit.
- **A module helper** `_transport_variance(g_ceiling, omega, tau, C_phi, nsteps)` — integrates
  `dg/dt=−C_φ·ω·g` (a genuine ODE solve; analytic for constant `ω`, but integrated so the negative-result
  gate can drive it with any `ω`). And the derived-ceiling helper.
- **The combination** in `zoned_nox`: `ei_no_transported = _pdf_mean_ei(far, …, g(C))` — the rung-13
  ideal bell at the transported width. `ZonedNOxState` records
  `transported`/`g_ceiling`/`g_transported`/`ei_no_transported` (`C_holdeman` reused).
- **`main.py` panel + `tests/test_rung18.py`.**

**Deliberately does NOT:**

- **Touch the cycle.** NO is still trace and decoupled; the layer is opt-in via `transported`. Every
  cycle station is **bit-for-bit rung 6** (the whole rung 1–17 suite stays green). `transported=None`
  runs the exact prior path.
- **Claim to DERIVE the optimum.** The `C_opt` location rides entirely on the **imposed** coverage `ω(C)`
  — the negative-result gate proves it (swap `ω(C)`→mean-field `ω(J)` ⇒ the optimum vanishes). Rung 18
  does not discharge the transported-PDF seam; it bounds it.
- **Transport the dwell.** `τ_core(C)` stays the rung-16 kink (out of scope — the declined seam). Rung 18
  is shape-only on `g`, on the **ideal bell** (no dwell), so the change is isolated.
- **Add super-equilibrium O / prompt NO.** `⟨EI⟩` stays an equilibrium-O lower bound; held `φ ≤ 2.0`.

---

## The one thing that makes it work (stated loudly — it IS the rung)

**The optimum is irreducibly spatial, so a 0-D transport can only IMPOSE it — but it can DERIVE the
ceiling.** The discriminating test (integrate a real variance ODE; mean-field `ω` ⇒ monotone `g(J)`;
only spatial `ω(C)` ⇒ an optimum) separates the two cleanly: the `C_opt` **location** is imposed (as it
always was, since rung 13), while the injection **ceiling** — the thing rungs 13–16 left as a free
`g_max` — is now **derived** from the primary richness, and the **sharp pin** is exposed as the artifact
of a corner no mixing physics produces. Transport tightened the closure at both ends (a physical ceiling,
a rounded elevated optimum) without over-claiming the one thing it cannot reach.

---

## The equations — a transported width over the ideal bell, no station changes

Every cycle station is **bit-for-bit rung 6**. `zoned_nox` is the rung-8..17 flow; rung 18 only adds the
transported-width PDF integral when a `TransportedPDF` is passed:

```
TRANSPORTED (transported=TransportedPDF(…), REQUIRES mixing=JetMixing(J,…)):
   ξ̄       = far/(1+far),   ξ_p = φ_p·f_st/(1+φ_p·f_st)
   g_ceiling = (ξ_p − ξ̄)/(1 − ξ̄)                     DERIVED two-stream ceiling (from φ_p; assert >0)
   C        = (S/H)·√J                                Holdeman group (H, J from the jet)
   ω(C)     = ω_opt·exp(−ln²(C/C_opt)/2w_cov²)         IMPOSED spatial coverage (peaked at C_opt)
   g(C)     = ∫ dg = −C_φ·ω(C)·g  over [0,τ]  from g_ceiling   (≡ g_ceiling·exp(−Da(C)); assert 0<g≤g_ceiling)
   ei_no_transported = _pdf_mean_ei(far, …, g(C))     rung-13 ideal-bell β-PDF at the transported width
   Da_opt → ∞  ⇒  g(C_opt) → 0  ⇒  the kinked notch (well-mixed point value)   [reduce]
   g_ceiling → 0  ⇒  g ≡ 0  ⇒  well-mixed point value ∀J                       [reduce]
```

- **`transported` REQUIRES `mixing`** (needs `J`, `H` for `C`; assert). `transported=None` keeps the
  exact prior path. **≤1-of-five** with `unmixedness`/`pdf`/`pdf_quench`/`pocket_quench` (assert).
- **Standing asserts (rung-18 deltas):** the rung-7 **K-check** + **trace guard** at every bell node `T`
  (reused via `_pdf_mean_ei`); the rung-13 **mean-preservation** on the β-PDF quadrature; the derived
  `g_ceiling>0` (guards `φ_p>φ_overall`); `0<g(C)≤g_ceiling`; `S,C_opt,C_phi,Da_opt,w_cov>0`;
  `transported ⇒ mixing`; at most one of the five closures.

---

## Verification gates (priority order)

1. **Reduce (load-bearing, exact by construction).** `transported=None` short-circuits to the prior path
   — every existing call is **bit-for-bit rung 17** (the whole rung 1–17 suite stays green). Second:
   `Da_opt→∞` ⇒ `g(C_opt)→0` ⇒ `ei_no_transported = the well-mixed point value` (the rung-13 `g→0`
   value) to `<1%`. Third: `g_ceiling→0` (via a leaner `φ_p`→`φ_overall` limit, test-only) ⇒ `g≡0` ⇒
   point value ∀J.
2. **THE NEGATIVE RESULT (the headline).** `_transport_variance` with **mean-field** `ω(J)` (const / √J /
   J) and fixed `τ` gives `g(J)` **flat or monotone** (min at an endpoint) — NO interior optimum; the
   **spatial** `ω(C)` peaked at `C_opt` gives an **interior optimum at `J_opt`**. The optimum ⟺ the
   spatial `S`. (Certifies §1 of the anchor — the machine-checked anti-derivation.)
3. **The DERIVED ceiling (lead result).** `g_ceiling==(ξ_p−ξ̄)/(1−ξ̄)` from `φ_p` to machine precision,
   `>0` for `φ_p>φ_overall`, and `< g_max=0.3` (the rung-13 free width, ~4.4× larger). Independent of
   `J`/`C_e` (a composition quantity).
4. **The RESIDUAL floor.** `g(C_opt)=g_ceiling·exp(−Da_opt)>0` (never the kink's exact 0), so
   `ei_no_transported(C_opt)` sits **above** the well-mixed point value (elevated optimum), while the
   rung-13 kinked `g` touches it. Both minima **AT `C_opt`** (the imposed coverage pins the location).
5. **KINK-is-non-generic (smoothness).** `g_transported(C)` has both one-sided slopes `→0` at `C_opt`
   (smooth), vs the kink's `±k_g/C_opt` corner; the `ei_no_transported` ratio one step off `J_opt` is
   `O(1)` (`~1.05×`) vs the kink's `≫10³×` dive to the floor.
6. **Cycle untouched.** A `transported` call leaves station 4 bit-identical — rung 6.
7. **K-check + trace + mean-preservation** bind at every bell node (reused from rungs 7/13).
8. **Guards.** `transported ⇒ mixing`; the **≤1-of-five** mutual exclusivity; `TransportedPDF`
   positivity (`S,C_opt,C_phi,Da_opt,w_cov>0`, grids `>1`); the derived-ceiling `φ_p>φ_overall` guard.

## Conservation asserts (rung-18 deltas)
Carry over rungs 6–17's, plus: `transported ⇒ mixing`; at most one of
`{transported,pocket_quench,pdf_quench,pdf,unmixedness}`; `TransportedPDF` positivity/range; the derived
`g_ceiling∈(0,1)` (asserts `φ_p>φ_overall`); `0<g(C)≤g_ceiling` from the ODE; and the reused rung-13
mean-preservation + rung-7 K-check/trace at every bell node.

## Done when
The reduces hold (`transported=None` bit-for-bit rung 17; `Da_opt→∞` = the kinked notch; `g_ceiling→0` =
point value); the **negative-result gate** certifies mean-field `ω` ⇒ monotone / spatial `ω(C)` ⇒
optimum; the **derived ceiling** matches `(ξ_p−ξ̄)/(1−ξ̄)` and beats `g_max` by ~4.4×; the **residual
floor** elevates the optimum; the **kink-non-genericity** smoothness contrast holds; the cycle is
bit-for-bit rung 6. `main.py` gains a rung-18 panel (the derived ceiling vs `g_max`; the smooth elevated
basin vs the kinked notch; the negative-result table — mean-field monotone vs spatial optimum);
`NOTES.md` gains a rung-18 section; `CLAUDE.md` scope + rung table + layout + deferred seams updated
(transported-variance closure **done** — the 0-D limit reached, the spatial/CFD PDF still the ceiling).

## The rung-19+ seam (keep it additive)
- **A spatial / transported-CFD PDF** — predict the cross-plane mixing pattern (and hence `g(C)` *and*
  the dwell spectrum, and hence rung 17's firing **magnitude**) from a PDF-transport / scalar-flux
  equation with the spatial `S`. Rung 18 proves this is exactly what 0-D cannot reach.
- **Super-equilibrium O / prompt (Fenimore) NO** — every `⟨EI⟩` here is an equilibrium-O lower bound.
- **Burner-side super-equilibrium** (`max_a>1` at station 4) and **finite-rate nozzle chemistry**.
