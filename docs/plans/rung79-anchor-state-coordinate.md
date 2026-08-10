# Rung 79 anchor — THE STATE COORDINATE (rung 78 § 9's fourth seam)

Scored in `docs/rung79-spec.md` § 8. The rule this project runs under: **a prediction that is
edited after the measurement is not a prediction.**

The seam, in rung 78 § 9's own words: ***THE SAME QUESTION ON THE φ LEG'S OWN COORDINATE.** § 4.2
shows the φ leg has no gauge because its set point is a constant. Rung 69's INCIDENCE reference
gave a leg a different coordinate without changing what it watches — a coordinate change is a
gauge candidate on the STATE side rather than the LAW side, and nothing in 69–77 asked whether it
preserves roots.*

Rung 78 re-wrote a leg's **LAW** (`cap_k = w0 + k(cap − w0)`) and found the root preserved and its
**UNIQUENESS DESTROYED**. This rung re-writes a leg's **STATE COORDINATE** — rung 60's incidence
`M_i` in place of rung 49's `φ` — and asks the same question of the other side.

**THIS ANCHOR IS DELIBERATELY LOPSIDED, AND § 0 SAYS WHY.** The derivation below is an
**identity**, provable in four lines, and it is listed as **DERIVATION, UNSCORED**. It is
confirmed numerically because an unconfirmed identity is an unrun code path, not because
confirming it is evidence. **If a D-item disagrees with the plant, this rung has a BUG, not a
finding.** Everything scored is in § 2, and it is all about § 5 — the **march**.

---

## 0. THE CONSTRUCTION — UNSCORED DERIVATION

### 0.1 The two residuals

The shipped φ leg (rung 49, solved in `_cap_fuel`'s surge branch since rung 74) is

    Gs(w) = φ_lim − φ(w)                          [rung 49's floor, in flow-coefficient units]

Rung 60's incidence currency is `M_i = T_c − (1/φ − v)`, with `T_c = tan_beta1_crit() = 1/φ_surge`
the blade **metal** angle (rung 53; zero new constants) and `v` the stator setting. Its floor is
`m_lim = T_c − (1/φ_lim − v)`, which is `IncidenceLimiter.from_phi(cmap, spool, φ_lim, v)` — the
**same** instrument renamed, which is what makes the two comparable at all. So

    Gi(w) = m_lim − M_i(w)
          = [T_c − 1/φ_lim + v] − [T_c − 1/φ(w) + v]
          = 1/φ(w) − 1/φ_lim

**`T_c` AND `v` CANCEL IDENTICALLY.** The blade metal and the stator setting are both absent from
the incidence residual. This is the rung's first structural fact and it is worth stating plainly:
rung 60 chose `M_i` *because* the stator does not move its wall, and in the residual the stator
does not appear at all.

### 0.2 It is a MULTIPLIER, and the multiplier is POSITIVE

    Gi(w) = (φ_lim − φ(w)) / (φ(w)·φ_lim) = Gs(w) · h(w) ,     h(w) := 1/(φ(w)·φ_lim)

`φ` is a flow coefficient, asserted `> 0` everywhere the map is defined, and `φ_lim > 0` by
`SurgeLimiter.__post_init__`. So **`h > 0` strictly, with no zero anywhere on the domain.** Four
consequences, all immediate:

* **D1 — the root SET is preserved POINTWISE**, not just at the set point: `sign Gi ≡ sign Gs`,
  so every root, and every non-root, is shared. Root *counts* must agree on any window.
* **D2 — the slope at the root scales by exactly `1/φ_lim²`.** `Gi′ = Gs′h + Gs h′`, and
  `Gs(w*) = 0` kills the second term, so `Gi′(w*) = Gs′(w*)·h(w*) = Gs′(w*)/φ_lim²` since
  `φ(w*) = φ_lim`. **A DERIVED factor with no fitted content.**
* **D3 — `dw*/dq` is INVARIANT.** `dw*/dq = −G_q/G_w`; at the root both numerator and denominator
  carry the same factor `h(w*)`, which cancels. This is rung 78's headline half arriving on the
  state side.
* **D4 — monotonicity is preserved, so the SHIPPED bracket still applies.** Cutting fuel raises
  `φ` (rung 49), so `Gs` increases in `w`; a positive multiplier cannot invert that, so `Gi`
  increases in `w` too. `_cap_free`'s bracket walk is legal in **both** coordinates.

### 0.3 THE STRUCTURAL POINT, AND IT IS THE HEADLINE

Rung 78's gauge had a **free dial** `k`, and its slope factor `1 − k·c` **passes through zero** —
which is exactly how it destroyed uniqueness (a second root swept in, collided at `k·c = 1`, and
departed). This gauge has **NO DIAL AT ALL**, and the reason is § 0.1: `T_c` and `v` cancel out of
the residual, so there is no parameter left to sweep. "Cannot be driven singular" is therefore not
*we swept and never reached zero* — it is **there is nothing to sweep**, and the multiplier is
positive because a coordinate must be invertible.

**So the expected headline: A COORDINATE CHANGE IS A GAUGE THAT CANNOT GO SINGULAR — and rung
78's lost uniqueness was a property of its AFFINE LAW-SIDE FAMILY, not of gauges as such.** This
**BOUNDS** rung 78 (it supplies the missing hypothesis under which uniqueness survives); it does
**NOT** correct it — rung 78's statement was true of the family it was made about.

D4 is the same statement in the solver: rung 78 had to write a damped Newton **because** its
family could invert `G`'s sign. This rung reuses the shipped bracket. **The root-finder
requirement is itself the diagnostic separating a law-side family from a coordinate.**

---

## 1. WHAT IS BUILT

`StateCoordinateTransient`, the **SEVENTH declared knob** beside `_share_law` (72), `_ref_law`
(73), `_lag_coord` (74), `_windup_law` (75), `_cap_law` (76) and `_gauge_k` (78):

    _phi_ref in {"phi", "incidence"}          "phi" == rung 78, by exact dispatch

**ZERO new constants.** `T_c` is rung 53's `1/φ_surge`; `m_lim` is rung 49's own `φ_lim` read
through rung 60's shipped `from_phi`; both cancel anyway (§ 0.1). Nothing is imposed and nothing
is swept.

**The fallback is REUSED, deliberately.** `_cap_free(Gi, mf_sched, fallback)` keeps rung 49's
`_surge_fuel` as its fallback, unchanged: it solves `φ(w) = φ_lim`, which by D1 is the **same root
set**, so the reuse is legal. Writing an incidence-specific fallback would put two root-finders
behind a rung whose whole claim is that there is one root.

---

## 2. THE SCORED PREDICTIONS — ALL OF THEM ABOUT § 5

Nothing here has been measured. §§ 0.1–0.3 are algebra; the numbers confirming them are reported
in the spec as **confirmations of an identity** and are not scored.

The live question is what a coordinate change does to the **PLANT**. It should do nothing — but
"nothing" has to survive a **discontinuous** selector. `_cap_fuel` returns `min(accel, φ)`, and
the φ cap comes out of a *bracketed Illinois solve* whose iterate sequence depends on the residual's
slope. Same root in exact arithmetic; **different floats**. A min-select can amplify a float
difference into a trajectory difference, and that is the one thing here that can actually fail.

| | prediction | why it can fail |
|---|---|---|
| **P1** | The set point moves, and by SOLVER NOISE ONLY: `0 < δ_rel < 1e−9` at a majority of cells. **Reported as a NUMBER, not as a bound.** | Both halves are live. `δ = 0` exactly would mean the Illinois path is insensitive to the slope (a finding about the solver); `δ > 1e−9` would mean a coordinate change is not root-preserving *in floating point*, which the reduce gate then has to answer for. `1e−15` and `1e−9` mean different things and a `< 1e−6` gate passes both. |
| **P2** | **THE MIN-SELECT NEVER FLIPS**: `flips = 0` over every riding point of the march. | **The rung's one genuinely open question.** If it flips, a coordinate change is inert on the set point and NOT inert on the plant — a real correction to "a gauge is inert", and a better finding than the expected one. |
| **P2n** | **P2's NON-VACUITY GUARD, registered in advance**: report `min |accel − φ| / φ` over the march. **If that gap is ≫ the P1 perturbation, P2 HOLDS VACUOUSLY and the spec must say so.** | Rung 78 § 5.1 hit three vacuity traps *after the fact*. This registers the guard *before*. A large gap makes P2 true and uninformative, and that must be reported as such rather than scored as a pass. |
| **P3** | `binds > 0`, and specifically `binds == hits` — the φ branch wins the `min` at **every** riding point. | Rung 78 § 5 measured the accel leg winning 0 of 15 at `margin` 0.05 and 0.10, i.e. the φ leg winning 15 of 15. If that does not reproduce at these settings, § 5 is masked exactly as rung 78's was — the same failure on the other leg. |
| **P4** | The trajectory's worst relative move is `< 1e−9` over all five tracked keys. | Fails if P2 fails; also fails if the noise of P1 integrates rather than damps over 341 steps. |
| **P5** | Root count is **1**, and **equal in both coordinates**, at every cell over rung 78's `[0.2, 3.0]·w0` window. | D1 forces *equality*; it does **not** force *one*. Two roots in both coordinates would be a real (and inherited) finding about the φ leg that no rung has looked for. |
| **P6** | `_cap_free` converges in **both** coordinates at every cell, with no damped Newton anywhere in this rung. | D4 says monotonicity survives. If the shipped bracket fails in the incidence coordinate, D4 is wrong on the plant and § 0.3's law-side/state-side separation loses its sharpest piece of evidence. |

**Expected: P1 and P3–P6 HOLD, P2 HOLDS, and the honest risk is that P2 holds VACUOUSLY.** That
is stated here, in advance, as the most likely disappointing outcome — and P2n is what will
detect it. A rung whose scored block is six HELDs is the shape of the failure this project has
recorded four times (rung 73's perfect confirmation having measured nothing; rung 77's
`1.000e+00`; rung 75's blind instrument; rung 78 § 5.1's three traps in one section), so the
spec must not dress the derivation up as a result.

---

## 3. THE REDUCE, BOTH DIRECTIONS (rung 73's discipline)

* `_phi_ref = "phi"` dispatches `_cap_fuel` to `super()` on an **exact** comparison ⇒ bit-for-bit
  rung 78. Gated.
* At `_phi_ref = "incidence"` the residual's **SLOPE RATIO must equal `1/φ_lim²`** (else the knob
  is dead — a coordinate that changes nothing is not a coordinate), **and** the **SET POINT must
  not move** beyond P1's noise (else it is not a coordinate at all, it is a device).

Either half alone passes something broken: the first passes a knob that is not wired to the
plant, the second passes a knob that does nothing.

## 4. SETTINGS

Rung 78's verbatim, so the two rungs are differenceable: `phi_lim = 0.80`, `margin = 0.10`,
`taus = (0.05,)*4`, `r = 0.5`, `s_settle = 1.2`, `ds = 0.005`, `v_max = 0.20`, `inc = False`,
coordinate `demand` (rung 74's — the one coordinate that reaches `_cap_fuel` at all; `clip`
dispatches out of the ladder before it, rung 76 § 0). `1/φ_lim² = 1.5625` exactly.
