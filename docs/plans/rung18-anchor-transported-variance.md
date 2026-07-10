# Rung-18 external anchors — the transported-variance closure: what a 0-D variance equation CAN and CANNOT derive (numbers-before-code)

Rungs 12–17 pin the dilution-mixing optimum with **hand-prescribed, kinked** functions of the Holdeman
group `C=(S/H)√J`: the β-PDF width `g(C)=min(g_max, k_g·|ln(C/C_opt)|)` and the dwell
`τ_core(C)=τ_res·(1+b_u·|ln(C/C_opt)|)`. Both carry a **sharp L1 corner centred on the empirical
`C_opt≈2.5`** — a modeling convenience, not derived. The deferred seam (named in the rung-13/16/17
specs) is a **transported PDF**: solve the width from a mixture-fraction variance equation instead of
imposing it, which would also let rung 17 report a firing *magnitude*.

**This anchor establishes — before any production code — that the seam is NOT dischargeable in 0-D, and
what a transport closure legitimately adds instead.** Every number was re-derived first; the prototypes
live in `M:\claud_projects\temp\rung18\` (`proto.py`, `proto2.py`, `proto3.py`, `CHARACTERIZATION.md`).

Design point = rung 16's: flight 250 K/50 kPa/M0.85, PR 10, `Tt4→1500 K`, `τ=3 ms`, `φ_p=1.5`,
`C_e=0.20`; `far=0.02718` (`φ_ov=0.402`), `ξ̄=0.02646`, `J_opt=(C_opt·H/S)²=16`.

> **Read `docs/rung13-spec.md` first** (the resolved β-PDF over the ideal bell — rung 18's vehicle) and
> `docs/rung11-spec.md` §"the ceiling" (mean-field ⇒ no mixing optimum). NO stays a **trace, decoupled
> diagnostic**, so the cycle is still **bit-for-bit rung 6**.

---

## 1. The load-bearing NEGATIVE result: 0-D transport cannot derive the `C_opt` optimum

The tempting design — `g_transported(C) = g_ceiling·exp(−Da_opt·exp(−ln²(C/C_opt)/2w²))` with a smooth
coverage `ω(C)` — produces a smooth basin. **But the smoothness is IMPOSED, not derived**: it is the
output of a hand-chosen smooth `ω(C)`. Dressing "the worked example shows a smooth basin" as "transport
derives smoothness" is circular. The discriminating test integrates a **genuine** variance ODE
`dg/dt = −C_φ·ω·g` from the two-stream ceiling and asks whether *any* optimum structure emerges
(`proto3.py`, machine-checked, `C_φ=2`, `g_ceiling=0.0675`, fixed `τ=2.5 ms` to isolate mixing
*quality* from the *time budget*):

```
 case                                             g(J) shape                          optimum?
 (1) mean-field ω(J), FIXED τ:
        ω const           g = 0.01934 ∀J          FLAT (J-independent)                NO
        ω ∝ √J            0.0361→0.00003          MONOTONE (min at J=625 endpoint)    NO
        ω ∝ J             0.0494→0.0000           MONOTONE (min at J=625 endpoint)    NO
 (2) JetMixing τ_q(J)∝1/√J, ω const:
        0.00731→0.05651                           MONOTONE (min at J=4 endpoint)      NO
 (3) SPATIAL coverage ω(C=(S/H)√J) peaked at C_opt, FIXED τ:
        0.0115 → 0.00711(J=16) → 0.0444           INTERIOR OPTIMUM at J=16 = C_opt    YES
```

**The structural argument (this — NOT the three curves — carries the result; the curves merely
ILLUSTRATE it).** A skeptic will say "you fed monotone `ω(J)`, so of course `g(J)` came out monotone."
The airtight form does not depend on the choice of `ω(J)` at all:

> An interior optimum in `J` requires the exponent `C_φ·ω·τ` to be **non-monotone** in `J` (a decay
> that is *fastest at some middle `J`*). A fixed residence makes `τ` constant, so the exponent tracks
> `ω`; the trajectory `τ_q(J)∝1/√J` is itself monotone — so **`ω(J)` must have an interior maximum**.
> But "the mixing rate peaks at a *particular* `J`" is a statement that there is a **preferred length
> scale** — a specific penetration the jet should reach. A mean-field mixing rate `ω(J)` is built only
> from `J`, `τ_q(J)`, `U_c`, `H` — it has **no spacing `S`**, so it has **no scale that singles out a
> `J`**, so it **cannot** have an interior maximum. The optimum can enter *only* through
> `ω(C=(S/H)√J)`, which imports the spacing `S` — the cross-plane geometry a well-stirred model omits.

So the optimum appears ONLY in case (3): once `ω` depends on `C=(S/H)√J`. The three mean-field rows are
the *illustration* — const (flat), `∝J` (monotone), and the sharp `∝√J` case, which makes
`Da = C_φ·ω·τ_q = C_φ·(k√J)·(H/C_e√J·U_c)` **exactly J-independent** (mixing that perfectly tracks the
jet still gives a *flat* residual). None can locate an optimum, because none contains a length scale.

This is the project's **own** result restated: rung-11's spec — *"mean-field ⇒ no mixing optimum (the
variance seam, rung 12)."* A 0-D transported-variance relaxation is **still mean-field**; it cannot
derive the `C_opt` structure for the same reason rung 11 could not. **The Holdeman optimum is a
cross-plane UNIFORMITY criterion — irreducibly spatial.** So rung 18's production `ω(C)` is an
**explicitly imposed** spatial closure (exactly the role rung-13's kinked `g(C)` played), and a
transported / CFD PDF that predicts the spatial pattern stays the **deferred ceiling**.

---

## 2. What transport LEGITIMATELY adds (the certified content — lead with the ceiling)

**(a) The DERIVED two-stream injection ceiling — the one quantity that emerges, not fit.** At injection
the dilution zone is two streams: rich-primary products at `ξ_p` and dilution air at `ξ=0`. The maximum
normalized variance of a two-delta PDF on `{0, ξ_p}` at the fixed overall mean `ξ̄` is

```
g_ceiling = (ξ_p − ξ̄)/(1 − ξ̄),   ξ_p = φ_p·f_st/(1+φ_p·f_st),   ξ̄ = far/(1+far)
          = (0.09216 − 0.02646)/(1 − 0.02646) = 0.0675   (φ_p=1.5)
```

set by the **primary richness `φ_p`** — no free knob. It exposes rung-13's `g_max=0.3` as **4.4× too
large**: that oversize is exactly what let rung-13's `⟨EI⟩(g)` reach its *humped/descending* regime (the
"far flank descends"), an artifact of an unphysical width. The derived ceiling keeps `g` in the
monotone-rising part of `⟨EI⟩(g)`.

**(b) The RESIDUAL-unmixedness floor.** A finite best-jet mixing `Da_opt = C_φ·ω_opt·τ` leaves
`g(C_opt) = g_ceiling·exp(−Da_opt) = 0.0675·exp(−2) = 0.00913 > 0` — perfect mixing is never reached, so
the emissions optimum is **elevated off the well-mixed value** rather than touching it. Through the
rung-13 ideal bell (`proto2.py`, derived ceiling, `n_bell=n_quad=200`):

```
   J    C     g_kink   g_tr   EIbell(kink)  EIbell(transp)
   9  1.88   0.0863  0.0099     0.73910        0.94690
  16  2.50   0.0000  0.0091     0.00001        0.89522   ← C_opt: kink touches floor; transp ELEVATED
  25  3.12   0.0669  0.0096     0.85445        0.92705
```

Both have the minimum **AT `C_opt`** (the imposed coverage pins the location), but the kink dives to the
well-mixed `≈1.3e-5` while the transported basin sits at `0.895` (residual `g=0.0091`). Magnitude
un-pinned via `Da_opt` (§4).

**(c) KINK-is-non-generic (a smoothness ARGUMENT, not a derivation).** The imposed `|ln(C/C_opt)|` is a
**corner**: one-sided slopes `∂g/∂C = ∓k_g/C_opt = ±0.12` at `C_opt` (a symmetric difference *cancels*
them — the kink hides unless probed one-sided). Any analytic mixing rate rounds it: the transported `g`
has both one-sided slopes `→0`. The single killer statistic — the EIbell ratio **one step off `J_opt`**:
kink **×5.8e4** (dives to the `≈0` floor) vs transported **×1.05** (genuinely smooth). So the **sharpness
of the pin** (not its location) was the artifact; a corner needs a special mechanism, and mixing rates
don't have one.

**(d) The canonical `C_φ≈2` decay law** — the mechanical-to-scalar timescale ratio, the one genuinely
anchored constant in the closure (the scalar-dissipation workhorse of presumed-PDF combustion).

---

## 3. The reduce (exact by construction)

`Da_opt → ∞` (perfect best-jet mixing) ⇒ `g(C_opt) → 0` ⇒ the transported basin **sharpens back to the
kinked notch** = the well-mixed point value (`proto2.py`: `Da_opt=25` ⇒ `g(C_opt)=9.4e-13` ⇒
`EIbell(16)=0.00001`, the exact rung-13 `g→0` value). So the kinked model **IS** the infinite-mixing
limit of the transported one — the pin was the `Da_opt→∞` idealization. Second reduce: `g_ceiling→0`
(no injected segregation) ⇒ `g≡0` ⇒ the well-mixed point value at every `J` (= rung-13 `k_g=0`).

---

## 4. The knobs (order-of-magnitude — disclosed, not fit)

`C_opt=2.5` (Holdeman), `S=0.0625 m`, `H=0.10 m` (the Holdeman group, shared with rungs 12–17); the
**derived** `g_ceiling` (from `φ_p`, NOT a knob); `C_φ=2.0` (canonical, anchored); `Da_opt` (optimum
Damköhler — how many e-folds of variance the best jet decays, un-pinned, sets the residual floor depth)
and `w_cov` (coverage width in `ln C`, un-pinned, sets the basin breadth). The **imposed coverage**
`ω(C)=ω_opt·exp(−ln²(C/C_opt)/2w_cov²)` is the spatial closure §1 proves cannot be derived in 0-D —
its *shape* is the modeling boundary, honestly the successor of rung-13's kinked `g(C)`.

---

## 5. Vehicle + scope

**Vehicle: the rung-13 ideal bell** (`_pdf_mean_ei`) — it isolates the `g`-shape cleanly (composition
variance only, no dwell). Carrying transported `g` through the rung-16 per-pocket quench makes the far
flank *rise*, but that rise is contaminated by the **still-kinked `τ_core`** (rung 18 is shape-only on
`g`; the dwell kink is untouched — the "also transport the dwell" seam was declined), so it is **kept
out of the headline**. Held `φ ≤ 2.0` (soot bound). Pure diagnostic: NO/N never enter `_equil_solve`,
cycle **bit-for-bit rung 6**.

## 6. What rung 18 leaves open (the still-deferred ceiling)

The **spatial / transported-CFD PDF** that predicts the cross-plane mixing pattern (and hence `g(C)`
*and* the dwell spectrum, and hence rung 17's firing *magnitude*) — §1 shows this is exactly what 0-D
cannot reach. Also still open: **super-equilibrium O / prompt (Fenimore) NO** (every `⟨EI⟩` here is an
equilibrium-O lower bound), the **burner-side clamp-fires** regime, and **finite-rate nozzle chemistry**.
