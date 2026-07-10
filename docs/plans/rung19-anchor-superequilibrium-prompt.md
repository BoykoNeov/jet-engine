# Rung-19 external anchors — super-equilibrium O & prompt (Fenimore) NO

Verified forms + worked checks that anchor the **two lower-bound-lifting NO channels** of rung 19:
a **computed, T-driven super-equilibrium-O multiplier** on the thermal rate, and an **imposed,
rich-peaking prompt-NO φ-bump** (De Soete global rate). Every number was re-derived **before any
production code** (project discipline), in `M:\claud_projects\temp\rung19\`
(`check_super_eq_o.py`, `check_prompt_no.py`, `check_corrected.py`).

> **The honest scope, up front (§5).** Rungs 7–18 read the rung-6 **equilibrium** `[O]` and so are
> a NO **lower bound**. Rung 19 lifts that bound two ways — and the load-bearing result is **the two
> lifts behave *opposite* to the "rich primary explodes" intuition**, twice over:
>   1. **Super-eq O is a *computed* T-driven multiplier** `m(T)∈[1.16,1.50]` (hot→small), **not** a
>      rich effect — it is `[O₂]`-independent, so it is *weakest* in the O₂-depleted rich primary.
>      Reduces to rung 7 exactly as `m→1`. The one thing that is cleanly derived.
>   2. **Prompt NO is an *imposed* rich-peaking φ-shape** (De Soete's fitted `f(φ)`), whose **absolute
>      magnitude is not derivable in a 0-D burnt pool** — it is set by one imposed scale (framed like
>      rung-18's imposed coverage `ω(C)`). Its lesson is the **prompt/thermal ratio** (grows
>      monotonically rich; prompt *survives where thermal dies*), certified as **directional**.
> This is a harder concession than rung 7 (which computed an absolute that landed in a band): here the
> super-eq **ratio** is computed but the prompt **magnitude** is imposed outright. Stated plainly.

> **Read `docs/rung7-spec.md` and `docs/rung9-spec.md` first.** Rung 19 rides on rung 7's Zeldovich
> integrator (super-eq O multiplies its `[O]`) and the rung-8/9 zoned primary (where the φ-sweep
> lives). It adds **no new species** and touches **no cycle path** — a decoupled diagnostic, so the
> cycle stays **bit-for-bit rung 6**.

---

## 1. The two channels — forms and where the numbers come from

### (a) Super-equilibrium O — a dimensionless multiplier on `[O]_eq`
Fluent's thermal-NO model offers three O-atom closures (Theory Guide §9.1.3 / §13.1.3): **equilibrium**
(Eq 9-11) and **partial-equilibrium** (Eq 9-13, adding the 3-body `O+O+M⇌O₂+M`), both of the Westenberg
(1971) form
```
[O]_eq = C1 · T^(−0.5) · [O₂]^0.5 · exp(−θ1/T)      (Westenberg equilibrium)
[O]_pe = C2 · T^(+0.5) · [O₂]^0.5 · exp(−θ2/T)      (partial equilibrium, super-eq)
```
with the standard constants `C1≈3.970e5, θ1≈31090 K; C2≈36.64, θ2≈27123 K` (SI, `[O₂]` as mol/m³).

**The load-bearing move (avoids every absolute-constant risk).** We do **not** plug Westenberg `[O]`
into the rate. Both expressions carry the same `[O₂]^0.5`, so their **ratio is dimensionless and a
function of T alone**:
```
m(T) = [O]_pe/[O]_eq = (C2/C1) · T · exp((θ1−θ2)/T)
```
Only `C2/C1` and `θ1−θ2` survive — far more robust than the absolutes, and the `[O₂]^0.5` cancels. We
apply `[O]_super = m(T)·comp["O"]` — **our own rung-6 equilibrium `[O]`, lifted**. So `m≡1` (super-eq
off) is **bit-for-bit rung 7**, and the whole thing needs no absolute-magnitude anchor.

| T (K) | 1800 | 2000 | 2200 | 2400 |
|---|---|---|---|---|
| **m(T)** | **1.505** | 1.342 | 1.232 | **1.157** |

Modest, **T-driven, `[O₂]`-independent** (measured identical across φ=0.8→2.0), **grows as T falls**.
Sanity: literature O-overshoots run ~1.2–10× — we sit at the low, conservative end (equilibrium-O
partial-eq, no radical-decay history).

**A free cross-validation (the units gate).** Westenberg equilibrium O (Eq 9-11) vs our `comp["O"]`
agrees to **0.94–0.99** across the whole (φ,T) grid — our rung-6 equilibrium `[O]` independently
reproduces the standard Westenberg correlation to ~5%. That both certifies our O pool *and* confirms
the SI/concentration units are right (a ~10× miss would flag a mole-fraction/concentration slip).

### (b) Prompt (Fenimore) NO — De Soete's global rate, reduced to a φ-shape
De Soete (1975), Fluent-adapted (Theory Guide §9.1.4 / §13.1.4):
```
d[NO]/dt|prompt = f(φ,n)·A·exp(−Ea/RuT)·[O₂]^a·[N₂]·[FUEL]        (full CFD-cell form)
f(φ,n) = 4.75 + 0.0819·n − 23.2·φ + 32·φ² − 12.2·φ³               (φ∈[0.6,1.6], aliphatic alkanes)
a(X_O₂):  1.0                    X_O₂≤4.1e-3
          −3.95−0.9·ln X_O₂      4.1e-3<X_O₂≤1.11e-2
          −0.35−0.1·ln X_O₂      1.11e-2<X_O₂≤0.03
          0.0                    X_O₂>0.03
Ea = 303474 J/mol,  A = 6.4e6 (SI; 1.2e7 for C≥2 fuels),  n = carbon number (12, the C12 Jet-A surrogate)
```

**Why we DROP `[O₂]^a` and `[FUEL]` (the model-form correction — see §5).** The full rate is fit
against flame-**cell** (partially-reacted) concentrations, where a rich front still holds O₂. Evaluated
on a **burnt equilibrium pool** it *double-counts* O₂ depletion: rich → `X_O₂` collapses → the rate
craters → the prompt EI peaks *lean* and dies by φ=1.2 — the **exact inverse** of the textbook rich
peak. The rich-peak physics lives **only in `f(φ)`** (De Soete's fitted correction). So the honest 0-D
form keeps the shape and the Arrhenius temperature factor and imposes the scale:
```
EI_prompt(φ,T) = scale · max(f(φ,n),0) · exp(−Ea/RuT)
```

**`f(φ)` shape (n=12) — the transcription check.** Peaks at φ=1.24, symmetric-ish, **goes negative
past φ≈1.65** (why `max(·,0)` and why φ>1.6 is *unusable*, not merely low-confidence):

| φ | 0.6 | 0.8 | 1.0 | 1.2 | 1.24 | 1.4 | 1.6 | 1.8 | 2.0 |
|---|---|---|---|---|---|---|---|---|---|
| f | 0.70 | 1.41 | 2.33 | 2.89 | **2.91** | 2.50 | 0.56 | −3.50 | −10.3 |

**`a(X_O₂)` continuity check** (a second transcription check — the piecewise is **continuous** at every
breakpoint, which a slipped coefficient would break): a(4.1e-3)=1.000, a(1.11e-2)=0.101 (both sides),
a(0.03)=0.001. Confirmed. *(We keep `a(X_O₂)` documented and self-checked even though the final prompt
form drops it — it certifies the transcription and marks the seam if a future rung resolves the flame
structure.)*

---

## 2. The load-bearing result — the intuition fails *both* ways, and prompt survives where thermal dies

At the rung-8/9 zoned primary (p≈13 atm, preheat Tt3≈700 K, τ=3 ms), primary AFT `T_p` from
`_primary_aft`, thermal EI from the rung-7 integrator, super-eq lift `= EI_therm·(m(T_p)−1)`, prompt
from the imposed φ-shape (scale set for a ~2 g/kg reference at φ=1.24):

| φ | T_p (K) | EI_therm | EI_superO | EI_prompt | **prompt/therm** | Σ NO (ppm) |
|---|---|---|---|---|---|---|
| 0.80 | 2303 | 17.18 | 3.26 | 2.03 | 0.12 | 1110 |
| 1.00 | 2513 | 48.20 | 6.00 | 12.65 | 0.26 | 4041 |
| 1.10 | 2509 | 16.63 | 2.09 | 14.27 | 0.86 | 2153 |
| 1.20 | 2436 | 2.76 | 0.40 | 9.93 | **3.59** | 911 |
| 1.30 | 2351 | 0.38 | 0.07 | 5.70 | **15.0** | 452 |
| 1.50 | 2184 | 0.006 | 0.001 | 1.07 | **168** | 87 |
| 1.80 | 1951 | ~0 | ~0 | 0 | — | 0 |

Three certified lessons:
1. **Super-eq O is T-driven, not rich.** `EI_superO/EI_therm = m(T)−1` — same fraction rich or lean;
   in absolute terms it is *tiny* in the rich primary (thermal is already dead there). At φ_p=1.5 the
   rich primary is carried by **prompt NO alone**; super-eq O lifts ~nothing.
2. **Prompt survives where thermal dies.** The prompt/thermal ratio grows **monotonically rich**
   (0.12→168): thermal collapses on the rich flank (T and `[O]` fall), but prompt — riding `f(φ)` — is
   the channel that keeps NO from going to zero just rich of stoich. *This* is the rich-specific lift.
3. **Everything stays trace.** Σ NO (thermal + super-eq + prompt) peaks at **4041 ppm = 0.40% < 2%**,
   so the decoupled-diagnostic assumption holds and the cycle stays **bit-for-bit rung 6**.

### The T-sensitivity discriminator (a real gate)
Prompt carries a **single** Arrhenius exp (`Ea/Ru≈36500 K`); thermal carries a **double** dependence
(`k1f·[O]_eq ~ exp(−38370/T)·[O]_eq`). Measured rise 2000→2400 K at stoich:

| channel | EI(2000 K) | EI(2400 K) | factor |
|---|---|---|---|
| thermal | 0.026 | 14.73 | **×566** |
| prompt  | 0.305 | 6.40 | **×21** |

Prompt is **~27× milder** in T — the quantitative face of "prompt survives where thermal dies," and a
clean gate (a bug that made prompt as T-steep as thermal would fail it).

---

## 3. Reduce-to-lower-rung (load-bearing, exactly like rungs 8–18)
- **super-eq off ⇒ `m≡1`**: `[O]_super = 1·comp["O"]` — the rung-7 integrator called on the *identical*
  pool → **bit-for-bit rung 7/8**. No NO/N added to `_equil_solve`; every rung 1–18 suite stays green.
- **`prompt=None`**: the additive prompt term is skipped entirely (code-path-identical to the prior
  rung), all rung-19 fields `None`.
- **Cycle**: NO/N never enter the cycle solve, and neither channel feeds `f`/`Tt`/thrust → **bit-for-bit
  rung 6** (asserted via the summed-trace guard < 0.02).

## 4. Tolerances (from the measured gaps, not guessed)
- **super-eq units gate:** Westenberg 9-11 / `comp["O"]` ∈ [0.94, 0.99] across (φ,T) — our O reproduces
  the standard correlation to ~5%.
- **super-eq multiplier band:** `m(T)` ∈ [1.15, 1.55] over 1800–2400 K; **φ-independent** to machine
  precision (the certified "T-driven not rich" claim); `m→1` as T→∞.
- **prompt f(φ) transcription:** peak at φ=1.236 (analytic df/dφ=0), negative past φ≈1.65; `a(X_O₂)`
  continuous at all three breakpoints (1.000/0.101/0.001).
- **prompt/thermal monotone rich:** ratio strictly increasing in φ over [0.8, 1.5] (0.12→168) — the
  "survives where thermal dies" lesson.
- **T-sensitivity discriminator:** thermal/prompt rise-factor ratio 2000→2400 K > 10× (measured 566/21
  ≈ 27×).
- **summed trace guard:** Σ x_NO < 0.02 across both channels (measured max 0.0040).
- **reduce (tight):** `m=1` bit-for-bit rung 7; `prompt=None` code-path-identical; cycle bit-for-bit
  rung 6.

> **Calibration note (final implementation).** The derive table above used the standalone check-script
> conditions (Tt3≈700 K, `T_ref=2200` K) — so its EI_prompt column shows the raw reference exploration.
> The shipped `PromptNO` back-solves `scale` from `peak_ei≈2 g/kg` imposed at a **realistic near-peak
> primary AFT `T_ref=2400 K`**, so the *delivered* prompt peak lands near ~2 g/kg (the ~1–5 g/kg
> literature band) at the production design point. `peak_ei` is the **reference** value at (`phi_ref`,
> `T_ref`), **not a cap**: because prompt carries `exp(−Ea/RuT)`, a hotter primary (`T_p > T_ref`)
> nudges the delivered EI above it. The **certified** results — the prompt/thermal ratio and the
> T-sensitivity contrast — cancel `scale` and are unchanged by this calibration.

## 5. What stays UN-anchored (state it plainly — the two concessions)
- **Prompt magnitude is IMPOSED, not derived.** A 0-D burnt pool has no flame structure and (after the
  O₂-double-count fix) no self-consistent absolute prompt rate — so the `scale` is set to a literature
  prompt level (a ~2 g/kg **reference** at a realistic primary AFT), **not** computed. This is the
  rung-18-style imposed closure (the honest successor of a flame-front / transported-PDF prompt model),
  and a **harder** concession than rung 7's (whose absolute landed in a band). What IS certified: the
  **φ-shape** (De Soete `f(φ)`, transcribed + self-checked) and the **directional** prompt/thermal ratio
  and T-contrast — not the g/kg number.
- **Super-eq O magnitude is semi-empirical.** `m(T)` is Westenberg's *fitted* partial-eq/eq ratio, not
  derived from our pool — a full-equilibrium major pool **cannot self-yield** super-equilibrium O (the
  same impossibility rung 18 hit: equilibrium in ⇒ equilibrium out). The lift lives in the fitted
  constants `C2/C1, θ1−θ2`, stated as imposed. What IS certified: the **reduce** (`m→1`≡rung 7), the
  **units cross-validation**, and that the lift is **T-driven and modest** (the debunking of the
  rich-intuition).
- **De Soete φ-validity 0.6–1.6.** φ>1.6 is genuine **extrapolation** (f goes negative) — clamped to 0
  and flagged; the deep-rich flank (φ_p up to the rung-9 soot bound 2.0) is *outside* the prompt model.
- **Fluent/Westenberg constants are IMAGE-LOCKED** on every primary web source (ENEA TLS, cfd-online
  403, ansyshelp SVG); the forms above are **transcribed from the standard published form and
  cross-checked for self-consistency** (f(φ) sign/peak, a(X_O₂) continuity, the units gate), **not**
  independently digit-verified. Source-of-record: ANSYS Fluent Theory Guide §9.1.3/§9.1.4; De Soete,
  *Proc. Combust. Inst.* 15 (1975) 1093; Westenberg, *Combust. Sci. Tech.* 4 (1971) 59.
- **No radical-decay history / prompt HCN chemistry.** Super-eq O here is a static T-multiplier, not a
  pocket that relaxes over the rung-10 quench; prompt is a global rate, not the detailed Fenimore
  `CH+N₂→HCN→…` path (which would need HCN/CN/NCO species and break the no-new-species reduce). Both
  are explicit next seams.
