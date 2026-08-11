# Rung 83 anchor — THE CORRECTOR'S OWN BAR

**Seam:** `docs/rung82-spec.md` § 8, first bullet — *"THE FIXED POINT'S OWN COST … whether a
**single Newton step** off the residual `h` (whose slope § 3a already measures at ±0.044 / −1.83)
reaches the same place for one march is untested, and it is the difference between a diagnosis and
a usable predictor."*

**Plant:** `CorrectorLawTransient` (`turbojet/engine.py`) — **no state, no knob, no constant**, the
fourth reader-only rung (77, 81, 82, 83). **Spec:** `docs/rung83-spec.md`. **Gates:**
`tests/test_rung83.py`.

**Rig:** `tests/test_rung82.py`'s verbatim — `FlightCondition(250.0, 50_000.0, 0.85)`,
`π_LPC/π_HPC/Tt4 = 3.0/6.0/1500.0`, the `REAL` loss set, `FLOOR = 0.55`, `Tt4 lo/hi/max =
1000/1400/1200`, `φ_fuel/φ_air = 0.75/0.77`, both `ComponentMap`s, and
`_lag_coord/_ref_law/_windup_law/_cap_law = "demand"/"sched"/"none"/"solve"`.

**Cost:** 268 marches over four probes — `precheck.json` (173), `iterate.json` (43),
`discriminator.json` (31), `jump.json` (21).

---

## 0. THE PRE-CHECK — RUN BEFORE ANY OF §§ 1–6 WAS WRITTEN

Rungs 81 and 82 both set the precedent: ask first whether the rung's own headline survives contact
with the shipped plant, and take no credit for anything the pre-check settles. Five questions, in
the order that could kill the rung: **E1** the identity, **E2** whether the expensive solve buys
accuracy at all, **E3** whether the slope is a constant of the plant or of the point, **E4** the
kink, **E5** the two estimators against both bars. `M:\claud_projects	empung83\precheck.py`.

**E1 settled the foundation immediately** — `h == τ̂_min − κ·τ` to the last bit (difference exactly
`0.0`) on 4/4 scans, and the RAW `τ_eff` set along the trajectory is a single float (`{0.09}` at
τ = 0.03, `{0.24}` at τ = 0.08). So the fixed point IS the root of the forward reading's own
residual, and everything below is about solving `F(τ) = τ`.

---

# Rung 83 § 1 draft — THE MECHANISM, DERIVED (zero new constants)

> **REVISION NOTE (advisor, before any spec was written).** An earlier draft of this section
> presented the error law below as *"verified to 2.7e-14 across 35 rows"*. That was an **IDENTITY
> ROUND-TRIP**: `c̄` is DEFINED as `(F(τ₀) − τ*)/(τ₀ − τ*)`, so substituting it into the secant root
> reproduces the law in two lines **for any `F`, any slope, any root**. It cannot fail, and the
> agreement measured nothing but floating-point closure on a rearrangement of my own definition —
> rung 70's "gate computing my own formula twice" and rung 77's "perfect 1.000e+00" in a new hat.
> The law is kept as a **DERIVATION**. It is never quoted as evidence and **never gated by a test**
> — such a test would pass on a plant that does not exist. All the empirical content of this rung
> is in the **MEASURED `c̄`** of § 1.5.

## 1.1 The identity the seam did not know it had

Rung 82 ships three readings. Two of them are **one object**, and the shipped code proves it.

`_threshold_scan` builds, at every scored interior four-loop point `s`:

    hat(s)     = ( gap(s) + tau_gov * cdot_r(s) ) / cdot_f(s)     the POINT-WISE threshold
    tau_eff(s) = kappa(s) * tau_f                                  read through `_demand_tau`

and returns

    h           = min_s [ hat(s) - tau_eff(s) ]      the RESIDUAL  (the fixed-point detector)
    tau_hat_min = min_s   hat(s)                     the FORWARD reading's numerator

**If `tau_eff` is constant along `s`** the two minima share an argmin, so

    h(tau) = tau_hat_min(tau) - kappa*tau = kappa * ( F(tau) - tau ),    F = the FORWARD reading

**So the FIXED POINT is not a rival reading of the criterion — it is the ROOT of the FORWARD
reading's own residual**, and rung 82's `fix` bisection is literally solving `F(tau) = tau`.

**MEASURED (E1), not assumed.** On four scans spanning two ramps and both branches, `h` and
`tau_hat_min − κ·τ` agree **to the last bit** (difference exactly `0.0`), and the RAW `tau_eff` set
along the trajectory is a **single float** — `{0.09}` at τ = 0.03, `{0.24}` at τ = 0.08, i.e.
`κ = 3.0` exactly. This is stronger than the shipped `kappa_pure` flag, which rounds to 6 places
before comparing.

## 1.2 What that buys — a Newton step, in closed form

Newton on `h`, `kappa` locally constant (it takes two values only, 1 and 3, so it is piecewise
constant by construction):

    h'(tau) = kappa * ( F'(tau) - 1 )

    tau_1 = tau_0 - h(tau_0)/h'(tau_0) = tau_0 + ( F(tau_0) - tau_0 ) / ( 1 - c ),   c = F'(tau_0)

**`kappa` CANCELS.** The correction is the forward reading's own miss, divided by `1 − c`.

> **THIS IS RUNG 77's `1/(1−c)` A THIRD TIME.** Rung 77 found it as a set-point solve's
> STIFFNESS; rung 78 found its slope a GAUGE and its root's uniqueness not. Here it is neither —
> it is the **CORRECTOR** that would turn the cheap reading into the expensive one. Same scalar,
> third role — and § 1.5 is why the third role does not work.

## 1.3 The error law — ALGEBRA, not a measurement

Let `τ*` be the root (`F(τ*) = τ*`), `ĉ` whatever slope the corrector actually uses, and define the
**MEAN slope from the root out to the reference**

    cbar = ( F(tau_0) - tau* ) / ( tau_0 - tau* )

Substituting `F(tau_0) = tau* + cbar*(tau_0 - tau*)` into `tau_hat = (F(tau_0) - chat*tau_0)/(1-chat)`:

    tau_hat - tau*  =  ( cbar - chat ) / ( 1 - chat )  *  ( tau_0 - tau* )
                        \___ SLOPE ERROR ___/            \__ LEVER ARM __/

and in relative terms

    (tau_hat - tau*)/tau*  =  ( cbar - chat )/( 1 - chat )  *  ( tau_0/tau* - 1 )

**This is two lines of algebra and holds for any `F`.** It is a derivation: it explains, it does not
evidence. Its content is that a one-step corrector has exactly two error sources, and they
**MULTIPLY** — so a reference far from the root demands a proportionally more accurate slope.

## 1.4 THE OPPOSITION — and why one step cannot inherit the cheap reading's accuracy

Rung 82 § 3a's finding is that the forward reading gets **BETTER the further ABOVE** the root the
reference sits (its own table: 98.9 % at τ_ref = 0.020 falling to 2.4 % at 0.120). The law of § 1.3
says the correction gets **WORSE** in exactly the same direction, because the lever arm
`τ₀/τ* − 1` is the multiplier.

The two are opposed. Solving § 1.3 for the slope precision a 10 % answer needs:

    |cbar - chat|  <  0.10 * (1 - chat) / ( tau_0/tau* - 1 )

| ramp | `τ*` | `τ₀ = 0.120` | `0.160` | `0.200` |
|---|---|---|---|---|
| | | lever · required `|Δc|` | | |
| 0.20 | 0.01108 | 10.8× · **0.0097** | 14.4× · **0.0071** | 18.1× · **0.0056** |
| 0.25 | 0.01975 | 6.1× · 0.0188 | 8.1× · 0.0135 | 10.1× · 0.0105 |
| 0.35 | 0.03710 | 3.2× · 0.0428 | 4.3× · 0.0289 | 5.4× · 0.0218 |
| 0.50 | 0.06109 | 2.0× · 0.0991 | 2.6× · 0.0591 | 3.3× · 0.0420 |
| 0.70 | 0.09202 | 1.3× · 0.3144 | 1.7× · 0.1294 | 2.2× · 0.0815 |

Rung 82 § 3a's two quoted slopes are **+0.044** and **−1.83**. At the references where the cheap
reading is accurate, the tolerance is **0.006–0.04** — finer than the gap between the two quoted
values, by two orders of magnitude at the tight end.

## 1.5 THE FINDING — `c̄` is not what any local reading measures

This is the rung's one measured fact, and it is not a matter of precision. Along the ladder, the
**local** chord slope and the **mean** slope back to the root differ in **SIGN**:

| ramp | `τ₀` | local chord `c` | mean-back-to-root `c̄` | |
|---|---|---|---|---|
| 0.20 | 0.200 | **+0.0024** | **−0.0005** | opposite sign |
| 0.20 | 0.160 | +0.0038 | −0.0013 | opposite sign |
| 0.20 | 0.120 | +0.0066 | −0.0032 | opposite sign |
| 0.35 | 0.050 | **+0.1200** | **−0.3757** | opposite sign, 3× magnitude |
| 0.50 | 0.080 | +0.1609 | −0.2283 | opposite sign |
| 0.70 | 0.120 | +0.2428 | −0.2666 | opposite sign |
| 0.25 | 0.020 | −0.1131 | **−10.03** | same sign, **89× magnitude** |

The reason is structural, and visible in the shipped `F`: **`F` has an interior MINIMUM just above
the root.** At `r = 0.20`, `F` equals `τ*` = 0.01108 at the root, falls to 0.00918 at τ = 0.020,
then rises monotonically to 0.01098 at τ = 0.200. Anywhere on that rising far branch the local
slope is positive while the chord back across the minimum to the root is negative.

**So the corrector needs `c̄`, and no reading of the plant AT `τ₀` measures `c̄`** — measuring `c̄`
requires `F(τ*)`, i.e. the root, i.e. the solve the corrector was trying to avoid. This kills BOTH
estimators with ONE mechanism: the 2-march secant and the 1-march borrowed slope are not estimating
the right quantity badly, they are estimating the **WRONG QUANTITY**.

## 1.6 The kink is NOT the mechanism

`F(τ) = min_s hat(s)/κ` is a **MINIMUM over trajectory points**, so it is only piecewise smooth, and
the shipped `s_bind` moves along the ladder (0.130 → 0.105 at `r = 0.35`). Before the sweep this was
the hazard the rung looked most exposed to, and V6 registered it: a chord across an argmin switch is
not a slope.

**It did not discriminate.** Under a fixed protocol the same-argmin flag neither predicts nor
anti-predicts accuracy: at `r = 0.20` every chord shares an argmin and the estimates still miss by
5–21 %; the two best single estimates in the whole sweep (1.2 % at `r = 0.25`, 4.1 % at `r = 0.50`)
both come from pairs that STRADDLE a switch. Two lucky rows out of ~30 is not an inversion of V6 —
it is V6 failing to be the mechanism. The mechanism is § 1.5.

## 1.7 And the SIDE is free — one march, not a solve

`h(τ₀) < 0` iff `F(τ₀) < τ₀`, so **`sign(h(τ₀))` — from the SAME single march that gives `F(τ₀)` —
says which side of the ROOT `τ₀` sits on.** This is a **correction to rung 82 § 6**, which says the
reader *"cannot know which side it is on without solving the problem it was trying to avoid"*. The
side is one march; only the ROOT is a solve.

**Scope, stated:** the side that is free is the side of the **residual's own root**, not of the
plant's measured threshold. Those are different objects, 2.7–9.4 % apart, and there is a live
counter-instance — `r = 0.25`, τ = 0.020: `h = −8.14e−3` (above the root, 0.01975) while
`n_fuel = 0` (below the plant's threshold, 0.02033). Registered as V7, not filed as a miss.

## 1.8 Cost, paired explicitly (rung 63's recorded over-claim)

- rung 82's **fixed-point bisection alone**: 2 endpoints + 10 midpoints + 1 re-scan = **13 marches**
- rung 82's `threshold_law` **row**: measured (13) + fixed (13) + reference (1) = **27**, or **40**
  with the `ds_fine` control — the seam's "~40 marches a threshold"
- the one-step estimators: **2 marches** (measured slope) · **1 march** (borrowed slope)

The honest comparison is **13 → 2** and **13 → 1**, against the FIXED POINT. The "~40" is a row's
whole cost including the measured bisection a predictor never has, and is never used as the bar.

---

# Rung 83 §§ 2-5 draft — PRE-REGISTRATION, written BEFORE the sweeps

Written before the pre-check output was read. § 0 gets filled from the probe; nothing below is
edited afterwards (rung 82's § 6a discipline — scored by appending, never by rewriting).

## 2. THE PREDICTIONS, IN THE ORDER THEY WILL BE SCORED

**P1 — THE IDENTITY IS EXACT, NOT APPROXIMATE.** On every scan where `kappa_pure` is True,
`h == tau_hat_min − kappa·tau_f` holds **to the last bit**, and the RAW set of `tau_eff` along the
trajectory is a **single float** (not merely equal after `round(…, 6)`, which is all `kappa_pure`
asserts). If this fails the derivation collapses and rung 83 is a NEGATIVE, reported as one.

**P2 — THE SIDE IS FREE; ONLY THE ROOT IS A SOLVE.** `sign(h(τ₀))` — read off the *same single
march* that yields the forward reading — agrees with which side of the threshold `τ₀` sits on, at
**every** ladder point of **every** admissible ramp, excluding points inside the measured
bisection bracket (V7). **This CORRECTS rung 82 § 6**, which says the reader *"cannot know which
side it is on without solving the problem it was trying to avoid"*. It is scored against the
FIXED POINT's side (`h`'s own root) and against the MEASURED threshold's side separately — they
differ by 2.7-9.4 %, and a point between them is a genuine disagreement, not a miss.

**P3 — TWO MARCHES REPRODUCE THIRTEEN.** Estimator (A) — secant slope, no prior knowledge — lands
within **10 %** of the 13-march bisected fixed point at every admissible ramp, *provided the pair
shares an argmin* (V6). This is **bar 1** and it is an arithmetic claim about the solve, not a
physical one about the plant.

**P4 — THE BORROWED SLOPE FAILS AWAY FROM WHERE IT WAS MEASURED.** Estimator (B) — one march,
rung 82 § 3a's quoted `+0.044` / `−1.83` — is **worse than (A)** at a majority of admissible
ramps, because `c` is a constant of the **POINT**, not of the plant. Rung 82's own § 3a already
shows the chord at **0.12** on 0.05→0.08 against **0.044** on 0.08→0.12 *within one ramp*.
REFUTED if (B) matches (A) within (A)'s own spread across ramps — which would make `c` a plant
constant and the seam's literal one-march version the right answer.

**P5 — THE SOLVE WAS NOT BUYING ACCURACY.** Against **bar 2** (the MEASURED threshold), the cheap
forward reading from a reference far ABOVE is **no worse** than the 13-march fixed point at ≥ 3 of
5 ramps. Rung 82's own § 2/§ 3a rows already have one instance (`r = 0.35`: forward-from-0.12 at
**2.4 %** against the fixed point's **3.7 %**). If this confirms, the rung's headline is about
**what the thirteen marches were buying**, not about cost — and the cost result becomes secondary
rather than being promoted past a result that contradicts its framing.

## 3. THE VACUITY CONDITIONS — REGISTERED IN ADVANCE

| # | condition | consequence |
|---|---|---|
| **V1** | four-loop window empty at a bracket end (`n_riding4 = 0`) — rung 82's own V1 | ramp **void**, counted, reported |
| **V2** | `riding4_valid` False — the plant never left `Tt4_lo` | row **void** |
| **V3** | the threshold sits **at** a bracket endpoint | row **void** — censored, never a value |
| **V4** | `kappa` impure on a scan | that scan's `F` is **undefined** (the map to the swept clock is not a single factor); estimate void, never computed with one of the two |
| **V5** | `\|1 − c\| ` below the point where the step is meaningful | the Newton step **DIVIDES BY ~ZERO**; the estimate is **void**, never clipped or winsorised |
| **V6** | the secant pair does **not** share an argmin (`s_bind` differs at its two ends) | the chord is **not a slope**. Those estimates are reported in their **own column** and never averaged with same-argmin ones — the advisor's discriminator, and the one this rung is most exposed to |
| **V7** | a ladder point lies **inside** the measured bisection bracket | its side is **unresolved**; P2 is not scored there. NOTE this is a censor, not a pass/fail bar — rung 82's P1 died of *scoring* against a loop-count width, and using it to **exclude** is the legitimate use |

## 4. THE CONTROLS, FIXED IN ADVANCE

1. **RUNG 82's IDENTITY REDUCE.** No state, no knob, no constant: at `r = 0.5` and rung 81's
   clocks this class's march is **bit-for-bit** `ThresholdLawTransient`'s. A reader-only rung whose
   march moved would be a rung-82 regression wearing a new class name.
2. **THE PAIR MOVES AT FIXED RAMP.** Rung 82 § 3a exists because a fixed reference across a ramp
   sweep makes the reference's side collinear with `r`. The ladder is therefore swept **within**
   each ramp, and any cross-ramp statement is made on the per-ramp results, never on one pair.
3. **THE SECANT SPACING IS SWEPT, NOT CHOSEN.** The pair separation is a READER setting (not a
   plant constant), so its effect on the estimate is measured rather than fixed by fiat — a single
   spacing would leave "the answer depends on my spacing" untested.
4. **BOTH BARS, ALWAYS SIDE BY SIDE.** Every estimate carries its error against the fixed point
   AND against the measured threshold. A pass on one must never be quotable as a pass on the other.
5. **COST IS COUNTED IN MARCHES AND NAMED AGAINST ITS BASELINE.** 13 = the fixed-point bisection;
   27 = a `threshold_law` row without the step control; 40 = with it. The predictor's 1 or 2 is
   compared to **13**, and the "~40" is never used as the comparison (rung 63's recorded
   over-claim was exactly this substitution).

## 5. WHAT WOULD REFUTE EACH PREDICTION

| # | refuted by |
|---|---|
| **P1** | any `kappa_pure` scan where `h` and `tau_hat_min − κ·τ_f` differ by more than 0 ULP, or a raw `tau_eff` set with more than one member |
| **P2** | a resolved ladder point (outside the bracket) where `sign(h)` disagrees with the side |
| **P3** | a same-argmin secant estimate more than 10 % from the bisected fixed point at any admissible ramp |
| **P4** | (B) matching (A) within (A)'s spread at a majority of ramps — `c` would then be a plant constant, and the seam's one-march version wins |
| **P5** | the fixed point beating the far-above forward reading at ≥ 3 of 5 ramps — the solve would then be buying real accuracy and the cost framing stands as the headline |

## 6. THE REDUCE CONTRACT

No state, no knob, no constant — the fourth reader-only rung (77, 81, 82, 83). `c` is measured, not
introduced; `1/(1−c)` is rung 77's own scalar; the ladder and the spacing are reader settings in the
sense rung 82's `tau_refs` and `bracket` are. The reduce is therefore an **IDENTITY**: at `r = 0.5`,
rung 80's walls and rung 81's clocks, the march is bit-for-bit rung 82's and its four-loop set is
rung 81's own (`n_riding4 = 33`, `n_fuel = 0`).

---

# Rung 83 — PRE-REGISTRATION OF THE ITERATION PROBE

Written and saved **BEFORE** `iterate.py` is run. Nothing below is edited afterwards; the probe is
scored by appending. This exists because P5 died of exactly this exposure — a bar that said
"a reference far above" without ever FIXING the reference, so a winner could be picked per ramp
after the fact. The start rule below is therefore fixed for all five ramps, in advance.

## The start rule — ONE rule, no per-ramp knowledge

The predictor is given exactly what rung 82's `_bisect` is given: the bracket `[lo, hi] =
[0.004, 0.30]`. Nothing else. From it:

    tau_0 = sqrt(lo * hi) = sqrt(0.004 * 0.30) = 0.034641...      (geometric mean — the natural
                                                                   centre of a positive quantity
                                                                   spanning ~2 decades)
    tau_1 = 1.25 * tau_0  = 0.043301...                            (the second point of the first
                                                                   secant; ratio fixed, not tuned)

Then plain secant on `g(t) = F(t) - t`, always using the two most recent iterates:

    t_{n+1} = t_n - g(t_n) * (t_n - t_{n-1}) / ( g(t_n) - g(t_{n-1}) )

This start SPANS the root set by accident, not by design: `tau_0 = 0.0346` sits ABOVE the fixed
point at `r = 0.20` (0.01108) and `r = 0.25` (0.01975), and BELOW it at `r = 0.50` (0.06109) and
`r = 0.70` (0.09202); at `r = 0.35` (0.03710) it is 6.6 % below. Both branches are therefore
exercised without choosing anything per ramp.

## Registered safeguards — declared now, counted in the result

| | rule | why it is declared rather than applied silently |
|---|---|---|
| **S1** | an iterate outside `[lo, hi]` is **CLAMPED to the violated end**, and the clamp is **COUNTED** | a bare secant can throw an iterate to infinity; a clamp is standard, but an uncounted clamp turns the probe into a bisection wearing a secant's name |
| **S2** | `\|g(t_n) - g(t_{n-1})\| < 1e-12` (a flat pair) ⇒ **ABORT that ramp as NON-CONVERGED**, never nudge | `r = 0.70` already has an exactly flat chord (`F(0.020) = F(0.030) = 0.27786`), so this WILL fire if the iteration wanders there |
| **S3** | a scan with `kappa_pure` False ⇒ `F` undefined ⇒ **ABORT that ramp**, counted | rung 82's own V4 |
| **S4** | cap **6 iterations** (≤ 8 marches including the two starts) | 8 < 13 is the whole point; a cap chosen after seeing the traces would be the P5 defect again |

## The score — fixed now

Marches to reach a tolerance, against rung 82's 13-march bisection **on the same bracket**:

- **primary:** marches until `|t_n − tau_fix| / tau_fix < 1 %`
- **secondary:** marches until `< 5 %`
- a ramp that hits the cap, aborts on S2/S3, or oscillates is **NON-CONVERGED** and is reported as
  such — never as "converged with a looser tolerance chosen afterwards"
- **clamps are reported per ramp**; a run that clamps on most iterations is reported as a
  safeguarded bisection, not as a secant

`tau_fix` is rung 82's own 13-march bisected fixed point (already measured: 0.01108 / 0.01975 /
0.03710 / 0.06109 / 0.09202). The iteration is scored against **bar 1** only — the arithmetic bar.
Bar 2 (the measured threshold) is not reachable by any solve of `h`, because the fixed point is
itself 2.7–9.4 % from it; that is rung 82's finding, not this probe's business.

## The predictions

**P6 — THE ITERATION CONVERGES WHERE THE SINGLE STEP DID NOT.** The safeguarded secant reaches
1 % of the fixed point within the 8-march cap at **≥ 4 of 5** ramps. Reasoning: the single step
fails because `chat` (local) is not `cbar` (mean back to the root) — but each iterate SHRINKS the
lever arm `|t_n − tau_fix|`, and `cbar` → `c_local` as the arm → 0. The mechanism that kills one
step is self-curing under iteration. REFUTED if it converges at ≤ 3 ramps.

**P7 — THE BELOW BRANCH IS THE WELL-CONDITIONED ONE.** Ramps started BELOW the root
(`r = 0.35 / 0.50 / 0.70`) reach 1 % in **strictly fewer** marches than ramps started above
(`r = 0.20 / 0.25`). Reasoning: below the root the measured chords are steeply negative
(−1.8 to −4.4), so `1/(1−c)` ∈ [0.18, 0.36] — a strongly damped, well-conditioned step; above it
`c ≈ 0` gives `1/(1−c) ≈ 1` on a nearly FLAT `F`, which is the ill-conditioned direction.
REFUTED if the above-started ramps are no slower.

**P8 — THE NON-MONOTONICITY BITES AT LEAST ONE RAMP.** `F` has an interior MINIMUM just above the
fixed point (at `r = 0.20`, `F` falls from 0.01108 at the root to 0.00918 at τ = 0.020 and then
rises monotonically to 0.01098 at τ = 0.200). A secant that lands in that basin sees a chord whose
sign is opposite to the one it needs. So **at least one ramp either clamps (S1) or overshoots past
the root before converging**. REFUTED if every ramp descends monotonically to the root with zero
clamps — which would mean the non-monotonicity is invisible to the iteration and the single-step
failure has nothing to do with curvature.

## The second question this probe also answers — P2's missing branch

The ladder in the first probe started at τ = 0.020, which is ABOVE the fixed point at `r = 0.20`
(0.01108) and `r = 0.25` (0.01975). So P2 — `sign(h)` gives the side of the residual's own root —
was tested on **one branch only** at 2 of 5 ramps. Two extra rungs at **τ = 0.008 and τ = 0.014**
are scanned at those two ramps (4 marches) so both branches are exercised everywhere.

**P2's scope is stated here explicitly:** the side that is free is the side of the **residual's own
root** (`tau_fix`), NOT the side of the plant's measured threshold (`tau_star`). There is already a
live counter-instance for the plant's side — `r = 0.25`, τ = 0.020: `h = −8.14e−3 < 0` (above the
residual's root, 0.01975) while `n_fuel = 0` (below the plant's threshold, 0.02033). That is the
2.8 % gap between the two objects, and it is registered as V7, not as a miss.

---

# Rung 83 — THE SCORED RESULTS (appended to the registrations, never edited into them)

Three probes, 247 marches total: `precheck.json` (173), `iterate.json` (43), `discriminator.json` (31).

## A. The registrations, scored

| # | prediction | verdict |
|---|---|---|
| **P1** | the identity `h = κ(F−τ)` is exact; raw `τ_eff` a single float | **CONFIRMED** — difference exactly `0.0` on 4/4 scans; raw set `{0.09}` at τ=0.03, `{0.24}` at τ=0.08, so `κ = 3.0` exactly |
| **P2** | `sign(h)` gives the side, from one march | **CONFIRMED** — 100 % at every ladder point of every ramp, **both branches at all five ramps** after the 3 extra below-root scans. **CORRECTS rung 82 § 6.** |
| **P3** | a 2-march secant lands within 10 % of the 13-march fixed point at every ramp | **REFUTED** — no fixed protocol reaches it; best worst-case is 19.6 % (at τ₀ = 0.050) |
| **P4** | the borrowed slope is worse than the measured one at a majority of ramps | **REFUTED as stated** — (B) beats (A) at the two near starts (29.7/35.0 % vs 202/111 %) and loses at the four far ones. The premise ("`c` is of the point") is right; the ranking it predicted is not |
| **P5** | the cheap reading is no worse than the solve at ≥ 3 of 5 ramps | **REFUTED** — see § B. The bar was mis-specified |
| **P6** | the iterated secant reaches 1 % within 8 marches at ≥ 4 of 5 ramps | **REFUTED — 1 of 5** (4 of 5 at 5 %) |
| **P7** | starting below the root is strictly faster than above | **REFUTED** — "below" holds both the best (r = 0.35, 3 marches to 5 %) and the only total failure (r = 0.70) |
| **P8** | the non-monotonicity bites ≥ 1 ramp (clamp or overshoot) | **CONFIRMED via the overshoot arm only** — `clamps = 0` at all five ramps, and **0 of 5** descend monotonically |

## B. P5 died of its own wording — and it is the second bar in this lineage to die that way

The bar said *"the cheap forward reading from a reference far ABOVE"* and never **FIXED** the
reference. Read with hindsight it passes 3 of 5 (2.4 % / 0.3 % / 0.4 % beating the solve's 3.7 % /
3.6 % / 2.7 %). But the winning reference is a **different one at every ramp** — 0.200, 0.200,
0.120, 0.120, 0.160 — and knowing which requires knowing the answer.

At any single fixed reference the 13-march solve wins:

| `τ_ref` | 0.020 | 0.030 | 0.050 | 0.080 | 0.120 | 0.160 | 0.200 |
|---|---|---|---|---|---|---|---|
| ramps where the cheap reading beats the solve | 0/5 | 0/5 | 0/5 | 0/5 | **2/5** | 1/5 | 0/5 |

**The twelve extra marches WERE buying accuracy.** Rung 82's P1 died of scoring against a loop-count
width; this one died of a bar that permitted picking the reference afterwards. Same shape, second
time: **a bar that names a DIRECTION ("far above") is not a bar until it names a POINT.**

## C. The one-step result, and its mechanism

Derived (§ 1.3 of the derivation — **algebra, never quoted as a measurement**):

    (tau_hat - tau*)/tau*  =  (cbar - chat)/(1 - chat) * (tau_0/tau* - 1)
                              \_ slope error _/          \_ LEVER ARM _/

The two error sources **MULTIPLY**, so the reference that makes rung 82's forward reading accurate
(far above) is the one that demands the finest slope. Required |Δc| for a 10 % answer at τ₀ = 0.200:
**0.0056** (r = 0.20) … 0.0815 (r = 0.70). Rung 82 § 3a's two quoted slopes are **+0.044** and
**−1.83**.

**And the corrector does not need a slope it could ever read.** It needs `c̄`, the MEAN slope back to
the root; every local reading gives `c_local`, and the two differ in **SIGN** at six of the sampled
rows (r = 0.20 τ₀ = 0.120/0.160/0.200; r = 0.35 τ₀ = 0.050; r = 0.50 τ₀ = 0.080; r = 0.70
τ₀ = 0.120), because **`F` has an interior MINIMUM just above the root**. Measuring `c̄` needs
`F(τ*)`, i.e. the root, i.e. the solve being avoided.

**The kink was NOT the mechanism.** V6's same-argmin flag neither predicts nor anti-predicts
accuracy under a fixed protocol; the two best single estimates in the sweep both straddle a switch.
Two lucky rows out of ~30 is V6 failing to discriminate, not inverting.

## D. The iteration, and the causal discriminator

Under the registered fixed start (`τ₀ = √(lo·hi) = 0.034641`, `τ₁ = 1.25 τ₀`, cap 6, 0 clamps):

| ramp | root | start/root | 1 %@ | 5 %@ | final \|g\| | final err | bisection's own res. |
|---|---|---|---|---|---|---|---|
| 0.20 | 0.01108 | 3.13× | — | 6 | 8.2e−04 | 5.35 % | ±1.30 % |
| 0.25 | 0.01975 | 1.75× | — | 5 | 3.5e−03 | 4.55 % | ±0.73 % |
| **0.35** | 0.03710 | **0.93×** | **5** | 3 | **1.7e−15** | **0.10 %** | ±0.39 % |
| 0.50 | 0.06109 | 0.57× | — | 7 | 1.5e−02 | 1.58 % | ±0.24 % |
| 0.70 | 0.09202 | 0.38× | — | — | 1.5e−01 | 21.63 % | ±0.16 % |

At `r = 0.35` — the one ramp whose root the fixed start happens to sit on — the secant drives the
residual to **1.7e−15 in 5 marches**. Its 0.10 % "error" is **inside the 13-march bisection's own
±0.39 % resolution**: the secant is the *more* accurate of the two there, and the disagreement is
the bisection's.

**THE CONTROL (an intervention, scores nothing).** Re-running the same secant with the start moved
ONTO each root converts **3 of the 4 failures**:

| ramp | final \|g\| | final err | bisection res. | verdict |
|---|---|---|---|---|
| 0.20 | 8.0e−16 | 0.49 % | ±1.30 % | **converged, inside the bisection's resolution** |
| 0.50 | 1.5e−14 | 0.08 % | ±0.24 % | **converged, inside** |
| 0.70 | 6.2e−08 | 0.12 % | ±0.16 % | **converged, inside** |
| **0.25** | 5.3e−03 | 9.59 % | ±0.73 % | **STILL FAILS — oscillates from the root itself** |

So the five outcomes had **two different causes**, and the intervention separates them:

1. **Three were the START** — the lever arm again, curable only by knowing the answer.
2. **One was the SHAPE** — and it is not what I inferred. See § E.

## E. THE SHAPE, MEASURED — AND `r = 0.25` HAS NO ROOT AT ALL

> **CORRECTION, recorded not rewritten.** An earlier § E called `F` at `r = 0.25` a **sawtooth**,
> from a slope of ≈ −9 read off two points the *secant happened to visit*. That was the
> identity-round-trip defect a second time — a number from an iteration's incidental path, not a
> measurement built to answer the question. Probes 3 and 4 (110 marches) replace it, and **`F` is
> not a sawtooth**: it is smooth and monotone on each branch, ~1 % per 0.25 % step, with **ONE**
> argmin handover. The real finding is stronger than the wrong one.

**Probe 3 — `F` resolved through the root at `r = 0.25`, at both of rung 82's own steps:**

| | `ds = 0.005` (shipped) | `ds = 0.0025` (rung 82's `ds_fine`) |
|---|---|---|
| branch behaviour | smooth, monotone, ≈ −1 % per step | same |
| argmin handover | **one**, at τ ≈ 0.0198, `s` 0.0850 → 0.0800 | **one**, same place, `s` 0.0825 → 0.0800 |
| jump size in `F` | **−19.40 %** | **−11.25 %** |
| bisected answer | 0.019754 | 0.019465 — **moved by 2.89e−4 = exactly ONE bracket width** |

**Probe 4 — 21 points at `1.25e−5` spacing (50× finer than probe 3, 23× finer than the bracket)
across the gap the bisection's final bracket lives in:**

    tau = 0.0197750   g = F - tau = +1.6498e-03   s_bind = 0.0850
    tau = 0.0197875   g = F - tau = -2.4264e-03   s_bind = 0.0800   <- argmin HANDOVER

**The residual JUMPS ACROSS ZERO.** Over a τ step of `1.25e−5` it goes from `+1.65e−3` to
`−2.43e−3`; the smaller of the two is **132× the step**. Exactly one sign change in the interval,
and it coincides with the argmin handover.

**So at `r = 0.25`, on rung 82's shipped grid, THE FIXED POINT DOES NOT EXIST.** The 13-march
bisection's answer, 0.019754, is the location of a **discontinuity** — a perfectly well-defined
sign-change locator, and not a root. That is why the secant fails there even when started at the
bisection's own answer (§ D): there is nothing to converge to.

**And existence is RESOLUTION-DEPENDENT, not a fixed property of the plant.** At `ds = 0.0025` the
upper branch does cross zero smoothly (`g` = +3.18e−4 at τ = 0.019250, −1.31e−4 at 0.019500, root
≈ 0.019427, matching the fine bisection's 0.019465 inside its width) — refining the march step
**creates** the root, moves the answer by a full bracket width, and opens a **new** jump at
τ = 0.023.

**The contrast that closes it (probe 3, Q4).** At `r = 0.35` — the one ramp where the secant reached
`1.7e−15` — `F` is smooth through the crossing (`g` = +1.56e−4 at τ = 0.037000, −6.83e−4 at
0.037333) and the nearest argmin handover is far away at τ = 0.038667. **A genuine root on a smooth
branch is exactly where the secant works, and it is the only such ramp of the five.**

## F. THE HEADLINE

**A BRACKETING SOLVE LOCATES A *SIGN CHANGE*; A CORRECTOR NEEDS A *ROOT* — AND ON A RESIDUAL BUILT
AS A MINIMUM, THOSE ARE DIFFERENT OBJECTS.**

`h` is a `min` over a discrete set of trajectory points, so each argmin handover is a jump. Bisection
reads only `sign(h)`, which is **free from one march** (§ A/P2 — the correction to rung 82 § 6) and
is defined whether or not a root exists. A Newton or secant step reads the residual's **value and
slope**, which presuppose a root — and at 1 of 5 ramps there isn't one.

So the thirteen marches were never buying resolution. They buy **an answer that exists**, and that
is the one thing a single step cannot borrow. The seam is answered **NO**, with the consolation
stated exactly: given a root on a smooth branch, the secant reaches it to machine precision in ~5
marches, beating the bisection's own ±0.39 % — it is a **polisher, not a predictor**, and the
bracket that tells it a root is there is the expense it was meant to remove.

**Cross-rung.** Rung 78 found *a residual's SLOPE is a GAUGE, its root's UNIQUENESS is not.*
Rung 83 finds **its root's EXISTENCE is not either.**

---
