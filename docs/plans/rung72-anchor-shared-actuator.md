# Rung 72 anchor — TWO LOOPS ON ONE ACTUATOR (`n = 4`, the last unoccupied SHAPE)

Scored in `docs/rung72-spec.md` § 9. The rule this project runs under: **a prediction that is
edited after the measurement is not a prediction.**

**AND THIS ANCHOR DISCLOSES ITS OWN ORDER, as rung 71's did.** Rung 71 § 11 named this plant as
one of two `n = 4` seams and asked one question of it — *does § 1's `m` count constraints or
actuators?* — and the plant could have been infeasible: rung 71's own joint window was already
down to 2.05 %, and a fourth loop can only intersect it further. So **§ 0's kill-check was run
before this document existed** (`M:\claud_projects\temp\rung72\killcheck.py`), and it is
reported as measurement, not prediction.

§ 2 is therefore split. **§ 2A is DERIVED** — worked out on paper from the two inherited laws
after § 0 and before any 4×4 gain existed, so it is listed as derivation and **is not scored as
prediction**. **§ 2B is genuinely open** at the time of writing and is what § 9 scores.

**AND § 2B RECORDS ONE PREDICTION THIS ANCHOR ALREADY REFUTED WITH PAPER.** The first design of
this rung had the SUM instrument moving the ZERO COUNT (2 → 1) and made that the discriminator.
§ 2A.4 works out that it does not: the count is the same under both laws, and what separates
them is the SPECTRUM. That is recorded here rather than quietly dropped, because the discarded
design is the reason § 3's instrument exists at all.

---

## 0. THE KILL-CHECK — the window, and the two things it found

Rung 52's `φ` FUEL leg armed BESIDE rung 47's `Tt4` governor, on rung 70's plant and on rung
71's. Six states, four clocks, four loops, **three actuators**:

    dgf/ds = ( F(ν,gr,q,v) − gf ) / τ_f   F = rung 52's clip,  φ_lp ≥ φ_lim   [FUEL   ]
    dgr/ds = ( R(ν,gf,q,v) − gr ) / τ_g   R = rung 47's clip,  Tt4 ≤ Tt4_max  [GOV    ]
    dq/ds  = ( C(ν,gf,gr,v) − q ) / τ_q   C = rung 65's b_cmd, φ_lp ≥ φ_lim   [VALVE  ]
    dv/ds  = ( V(ν,gf,gr,q) − v ) / τ_s   V = rung 68/69's,   φ_lp or M_i     [STATOR ]

**The composition law on the shared actuator is this rung's one modelling decision**, and it is
declared here rather than discovered later:

    mf = mf_sched − max(gf, gr)      MIN-SELECT in clip coordinates.  THE PLANT.
    mf = mf_sched − gf − gr          the SUM law.  AN INSTRUMENT (§ 3), never the plant.

The first is what a fuel control does — every leg computes the fuel it would allow, the lowest
wins, which in clip coordinates is the largest cut. The second double-clips; rung 70's own
integrator refuses it in so many words ("applying both would clip twice and the redline would be
held by an instrument that is not the loop under study"). It is carried only as § 3's isolation
instrument, the role rungs 50/51's forced release edges played.

### 0.1 The windows (`ds = 0.005`, `τ = (0.05, 0.05, 0.05, 0.05/0.15)`, `Tt4_max = 1200 K`)

| leg | `φ` stator arm | INCIDENCE stator arm |
|---|---|---|
| fuel leg riding | 0.005 … 1.700 (340) | 0.005 … 1.700 (340) |
| governor riding | 0.110 … 1.700 (319) | 0.105 … 1.700 (320) |
| valve riding | 0.000 … 0.610 (123) | 0.000 … 0.645 (130) |
| stator riding | 0.005 … 0.435 (87) | 0.005 … 0.245 (49) |
| **ALL FOUR** | **0.110 … 0.435 (66) = 19.35 %** | **0.105 … 0.245 (29) = 8.50 %** |

**The window is WIDER than rung 71's, not narrower** — 19.35 % and 8.50 % against rung 71's
2.05 % — because the fourth loop is a fuel-side one and the fuel side is live over essentially
the whole march. A rung whose feasibility was the open question has the most comfortable sample
in the family since rung 68.

### 0.2 AUTHORITY CHANGES HANDS, ONCE, INSIDE THE JOINT WINDOW

| arm | fuel leg holds | governor holds | hand-over at |
|---|---|---|---|
| `φ` | `s` ≤ 0.205 (40 pts) | `s` ≥ 0.205 (300 pts) | **`s` = 0.205** |
| incidence | `s` ≤ 0.245 (48 pts) | `s` ≥ 0.245 (292 pts) | **`s` = 0.245** |

and **both legs want a cut over 319 of 341 points** (`s ≥ 0.11`), so the masked leg is a leg
that is *riding and reaching nothing*, not a dormant one. The hand-over sits **inside** the joint
window on both arms, so every claim below is measurable on **both sides of the switch, on one
trajectory** — which is why this rung needs no second plant to make its comparison.

### 0.3 What the kill-check did NOT settle

* The SUM instrument's own march terminates early (84 points against 341) and holds `Tt4` at
  1177 K — it over-clips, exactly as expected of a non-physical law. **So § 3 does not march it.**
  It is evaluated as a Jacobian AT THE MIN-SELECT TRAJECTORY'S OWN BASE POINTS, which is rung
  71's `m70 = _split_rig(…)`-at-identical-points device, one law over instead of one rung over.
  A comparison of two laws on two different trajectories would confound the law with the state.
* The joint initial condition converges in **one** iteration with residual **exactly 0**, because
  both fuel legs open dormant (`gf0 = gr0 = 0`). The two new states therefore add nothing to
  rungs 68/70/71's IC story, and § 2B.9 predicts that rather than assuming it.

---

## 1. THE DERIVATION

Rung 69 § 1, which rungs 70 and 71 both re-read unchanged:

    row_i(M) = −(1/c⁽ⁱ⁾_i)·∇c⁽ⁱ⁾ᵀ      ⇒      rank M = dim span{∇c⁽ⁱ⁾} =: m ,   zeros = n − m

**That formula has a precondition nothing before this rung could violate: the loop must own the
actuator it solves for.** Every plant from rung 62 to rung 71 has one law per actuator, and § 1
says so in passing ("one actuator per loop") without ever needing it. Here two laws drive one
actuator, and the precondition becomes the subject.

### 1.1 What a shared actuator does to a row

Both fuel-side laws are inherited verbatim and both compute their clip **from the SCHEDULED
fuel** — rung 47's discipline ("`required` is what the clip WOULD have to be, not what the
current clip makes it") and rung 52's ("solved from the scheduled fuel so arming one leg cannot
perturb another's bracket"). Neither was written with a second clip in mind. Consequences:

    F does not depend on gr   and   R does not depend on gf        ⇒   F_r = R_f = 0, EXACTLY

so each fuel row carries a `−1` on its own axis and **zero on the other fuel axis**. A row of
that shape is not `−(1/c_i)∇c` for any constraint the plant has: under min-select `φ` and `Tt4`
depend on the fuel states only through `max(gf, gr)`, so the true gradient is *flat* in the
masked one, and the masked leg's own `c⁽ⁱ⁾_i` is **0**. **Its row formula is UNDEFINED — not
parallel to another's, undefined** — while `J` itself stays perfectly finite, every row being
`(∂R_i/∂x_j − δ_ij)/τ_i`. The closed form `zeros = n − m` has no value to take here; `det J`,
the zero count, `c1` and the spectrum all remain measurable. That is the honest shape of the
finding and it rhymes with rung 71's own *counts gradients, not live loops*.

### 1.2 The masked COLUMN, and why the spectrum splits

Order the states `(gf, gr, q, v)` and take the governor holding authority (`gr > gf`). Then
`max(gf, gr)` is flat in `gf`, so **nothing downstream of the actuator sees the masked state**:

    C_f = V_f = 0   and   R_f = 0 (§ 1.1)      ⇒     column_gf(M) = (−1, 0, 0, 0)ᵀ

`M` is block **upper**-triangular, and therefore

    eig(M₄) = { −1 }  ∪  eig(M₃) ,     det M₄ = −det M₃ ,     zeros(72) = zeros(parent)

with `M₃` **the parent rung's own 3×3 block, entry for entry**. In `J = diag(1/τ)·M` the split
survives with its clock attached:

    a pole at EXACTLY −1/τ_f, independent of every gain, every other clock and the plant
    c0(72) = −c0(parent)/τ_f

**The masked leg is running open loop.** It is a first-order lag being driven by a reference it
cannot act on — min-select windup, seen in the spectrum.

### 1.3 The rank law that replaces `zeros = n − m`

A loop that holds authority contributes its constraint's gradient; a loop that does not
contributes its own axis, which no other row occupies. So

    rank M = m_live + n_masked        ⇒        zeros = n_live − m_live

with `n_live` the loops **holding authority** and `m_live` the distinct constraint gradients
among them. `n_live = 3` here on both arms, and `m_live` = 2 (`φ` arm) or 3 (incidence arm).

### 1.4 The two readings rung 70 § 6.1 / rung 71 § 11 offered, scored on paper

| arm | constraints | actuators | constraint reading `n − m` | actuator reading | § 1.3 |
|---|---|---|---|---|---|
| `φ` stator | 2 | 3 | **2** | **1** | **1** |
| incidence stator | 3 | 3 | **1** | **1** | **0** |

**Neither is the answer.** The constraint reading is wrong by exactly one on *both* arms; the
actuator reading is right on the `φ` arm and wrong on the incidence one, i.e. right by
coincidence — on the `φ` arm the masked leg and the missing gradient happen to cancel. The
question rung 71 § 11 posed has a third answer, and the second arm is what proves it: **without
the incidence arm this rung would have shipped "`m` counts actuators" and been wrong.**

---

## 2A. DERIVED, NOT SCORED

* **D1** The composition law is min-select in clip coordinates; the SUM law is an instrument.
* **D2** `F_r = R_f = 0` exactly, at every point, on both arms — a property of the two INHERITED
  laws' scheduled-fuel discipline, not a modelling choice made here.
* **D3** Under min-select the masked state's column is `(−1, 0, 0, 0)ᵀ`, so `M` is block
  triangular, `eig(M₄) = {−1} ∪ eig(M₃)`, `det M₄ = −det M₃`, and there is a pole at exactly
  `−1/τ_f`.
* **D4** `zeros = n_live − m_live`; the constraint reading is wrong by one on both arms.
* **D5** The SUM law does **not** move the zero count. Under SUM neither fuel row is a
  constraint gradient either (§ 1.1's precondition fails for *both* legs rather than one), so
  each still contributes its own axis: rank `= 2 + rank{∇C, ∇V}` = 3 on the `φ` arm (the valve
  and the `φ` stator share a gradient) and 4 on the incidence arm — **the same 1 and 0**. This
  refutes the first design of § 3 and is why the discriminator is the spectrum.

## 2B. THE PREDICTIONS — genuinely open, and what § 9 scores

**P1 — THE ZERO COUNT.** Measured `zeros` = **1** on the `φ` arm and **0** on the incidence arm,
at every interior sampled point of the joint window, **on both sides of the authority hand-over**
— against the constraint reading's 2 and 1. This is the rung.

**P2 — THE SPECTRUM SPLITS, AND THIS IS THE STRONGEST TEST.** The four eigenvalues of `J₄`
reproduce the parent rung's three (rung 70's on the `φ` arm, rung 71's on the incidence arm,
read at the IDENTICAL base point) plus one root at exactly `−1/τ_f`, to the differencing floor.
Predicted to hold at every interior point; predicted to hold with the **roles swapped** before
the hand-over, where it is the GOVERNOR that is masked and the free pole is `−1/τ_g`.

**P3 — THE DETERMINANT.** `c0(72) = −c0(parent)/τ_f` at identical base points: identically 0 on
the `φ` arm (rung 70's own `det J = 0`), and `+(1 − pair_RC)(1 − pair_CV)/(τ_f τ_g τ_q τ_s)` on
the incidence arm — **rung 71's factorisation with its sign flipped by the masked leg**, which
would make rung 71 § 1.3's *one factor per rung* survive a rung that adds no factor.

**P4 — THE MASKED COLUMN IS EXACTLY ZERO, AND IT SWAPS.** `C_f = V_f = 0` to machine precision
while the governor holds; `C_r = V_r = 0` while the fuel leg holds. Not "small": exactly zero,
because `max()` is flat. Predicted to be the one measurement in this family where rung 67's *a
zero cross-gain is saturation, never decoupling* is **false in a new way** — it is neither
saturation nor decoupling here but MASKING, and the masked leg is nowhere near a stop.

**P5 — THE SUM INSTRUMENT: the count holds, the pole goes.** At the same base points, the SUM
law's spectrum contains **no** root at `−1/τ_f` and its zero count is **unchanged** (1 and 0,
per D5). So a count cannot separate masking from the rank law, and the free pole can. If instead
the count moves, D5 is wrong and § 1.3's accounting needs re-deriving.

**P6 — THE LEDGER.** In the 16-cell ledger (four loops, both arms), the fuel leg's marginal
`φ`-credit is **positive but small**, and it is delivered **entirely inside its own authority
window** (`s ≤ 0.205`). Predicted corollary: adding the fuel leg to rung 70's plant improves
`min φ_lp` while leaving `max Tt4` **unmoved to within the differencing floor**, because a leg
that is masked wherever the governor is binding cannot spend the governor's currency.

**P7 — RUNG 66's IDENTITY, AT ITS OPPOSITE CORNER.** `pair_FR = F_r · R_f = 0` **exactly** — two
loops on ONE ACTUATOR give the pair product 0, where rung 66's two loops on ONE VARIABLE gave
exactly 1. Predicted as the sharpest single contrast in the rung: sharing a *variable* makes two
loops maximally redundant, sharing an *actuator* makes them maximally exclusive.

**P8 — THE REDUCE, FOUR ARMS, BIT-FOR-BIT BY DISPATCH.** No fuel leg ⇒ rung 71 / rung 70;
`tau_gov=None` ⇒ rung 69 / rung 68; no stator ⇒ rung 67 / 66; neither fuel leg ⇒ 65/64/62. On
341 points and 9 recorded keys, worst difference **0.0**.

**P9 — THE INITIAL CONDITION INHERITS AND ADDS NOTHING.** Both fuel legs open dormant, so the
`s = 0` fixed point is the parent's unchanged: a one-parameter FAMILY on the `φ` arm (the valve
and stator share `φ`, `|C_v V_q| = 1`) and a POINT on the incidence arm. The four-way sweep is
predicted to converge in ONE iteration at residual 0 in every one of the 24 orders, and the
`max()`-cycling failure mode is predicted **not** to fire here — reported as an untested guard,
never as a passing test.

**P10 — THE REFUSALS.** `_share_law` outside `{max, sum}` refused; `tau_gov` without `Tt4_max`
refused; rungs 50/51's forced release edges refused; an instantaneous valve refused; `ds` past
the four-clock RK4 floor caught by the guard.

---

## 3. THE INSTRUMENT, AND THE ONE THING THAT CAN GO SILENTLY WRONG

**THE SWITCH-PROXIMITY FILTER.** A central difference in `gf` of step `dg` taken where
`|gf − gr| < dg` straddles the `max()` kink and returns the slope of neither branch — the
authority-hand-over version of rung 68's regime filter, and it is not caught by it, because both
legs are comfortably riding and nowhere near a stop at the switch. Points with
`|gf − gr| ≤ 4·dg` are SKIPPED and the count is REPORTED, never silently dropped (rung 68's rule:
a dropped point is a coverage claim).

**THE BOUNDARY, SIXTH RELOAD.** Rung 70's `_assert_state_boundary` measures the governor's
cross-gains against a deliberately blind version, because losing `_b_state`/`_v_state` around
`required` makes the odd loop decouple and *nothing fails*. Here that trap has a twin: if the
FUEL leg's `required` lost the same boundary, `F_q = F_v = 0` and the fuel row would look exactly
like a masked one — the rung would "confirm" its own headline through a bug. **The boundary is
asserted for both fuel legs, against both blind versions.**

**WHAT WOULD FALSIFY THE RUNG.** A measured zero count of 2 on the `φ` arm (the constraint
reading, and § 1.3 wrong); or a spectrum that does not contain `−1/τ_f` under min-select (§ 1.2
wrong); or `C_f` merely small rather than exactly zero (the composition law not doing what § 1.2
says it does).
