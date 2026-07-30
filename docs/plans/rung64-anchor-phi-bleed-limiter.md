# Rung 64 anchor — the φ-REFERENCED BLEED LIMITER

Rung 63's named next seam, verbatim from `docs/rung63-spec.md` § "The next seam":

> **A `φ`-REFERENCED BLEED LIMITER** — a *controlled* valve, and § 3 is what makes it sharp:
> a floor that watches `φ` and a lever whose entire credit runs through `φ` are, in this
> rung's pair, two objects that cannot compose; made one object, the floor would move its own
> set point as it acted. Rung 60's tautology and the `φ`-rate negative both bound what it can
> say, and rung 52's *"a self-releasing limiter cannot debit the spool it watches"* predicts
> the shape.

Every rung from 42 to 63 armed this valve OPEN-LOOP: rung 42 a constant position, rung 62 a
schedule `b(n_L)`. This one CLOSES THE LOOP on the protected variable itself.

---

## § 0 — THE BLOCKER, and the two discriminator probes that cleared it

**The trap.** A valve commanded to hold `φ_lp ≥ φ_lim`, on a lever whose entire credit runs
through `φ` and whose `v ≡ 0` (so rung 53's two currencies collapse to `M_i = T_c − 1/φ`),
**pins the coordinate it watches**. That is rung 60's tautology verbatim, and rung 63's
`floor_dichotomy` already re-found it on this very plant. Written naively, rung 64 would
publish `min φ_lp == φ_lim` to 1e−16 and call it a finding.

So before ANY prediction was written, two discriminating probes were run to establish whether
the rung has content the tautology does not already own. **Their results are declared here as
the rung's GIVEN, not as scored predictions** — § 2 pre-registers only what they did not
touch.

### Probe A — the CEILING. Does feedback buy AUTHORITY?

A controlled valve saturates when even `b = b_max` cannot hold the floor, so its saturation
boundary is `min φ` over a march with the valve **fully open throughout**. That needs no new
plant — `b ≡ b_max` is already constructible. On rung 63's grid (`φ_surge` = 0.55,
`b_max` = 0.10, `r` = 0.5, `ds` = 0.005, `n_lo` = 0.65):

| plant | valve SHUT | rung-62 SCHEDULE | constant `b = b_max` |
|---|---|---|---|
| shaped | 0.735442 (`sm` 0.337167) | 0.788669 (0.433944) | **0.809515 (0.471846)** |
| tilted | 0.737373 (0.340678) | 0.789699 (0.435816) | **0.808539 (0.470071)** |

The schedule commands only `b` = 0.073604 / 0.076034 of `b_max` = 0.10 **at its own `φ`
minimum** — NOT saturated — so there is a real gap of +0.0208 / +0.0188 in `φ` above rung 63's
published upper band edge.

**But the third column is itself an OPEN-LOOP law**, and it attains that ceiling. So:

> the ceiling on `min φ` is a property of **`b_max`** — the lever's AUTHORITY, which is
> hardware — and **not** of feedback. A closed loop buys nothing over the best open-loop law
> on the protected coordinate.

This is what makes the rung more than rung 60 restated, and it settles rung 63's "would the
floor move its own set point" worry in the only way that matters: it would, and it changes
nothing about where the ceiling is.

### Probe B — the BILL, in rung 61's currency, at a MATCHED coordinate

The advisor's blocking condition, and it is the right one: rung 61 found the bleed's φ-credit
is a **loading rebate** with 73–102 % of the overspeed surviving, so "the bill" is NOT `∫b ds`
— it is the overspeed and the thrust. Three laws matched to the same `min φ_lp` = 0.80
(strictly inside [0.7354, 0.8095], so rung 60's pinning genuinely binds and the match is
exact), shaped map:

| law | `min φ_lp` | `min φ_hp` | `∫b ds` | `b` peak | `nu_lp_end` | `nu_hp_end` | `F_end` |
|---|---|---|---|---|---|---|---|
| 1 constant `b*` = 0.088163 | 0.800000 | 0.863496 | 0.149876 | 0.08816 | 0.919682 | 0.953647 | 573.00 |
| 2 schedule `b_max*` = 0.118220 | 0.800000 | 0.860884 | 0.073741 | 0.10130 | 0.937669 | 0.955011 | 603.38 |
| 3 **φ FLOOR** `φ_lim` = 0.80 | 0.800000 | 0.860638 | **0.038249** | 0.09202 | 0.941018 | 0.953556 | 606.36 |
| 0 valve SHUT (reference) | 0.735442 | 0.861169 | 0.000000 | 0.00000 | 0.941706 | 0.952714 | 606.19 |

Bill against the valve-shut reference:

| law | Δ`nu_lp_end` | Δ`nu_hp_end` | Δ`F_end` | Δ`∫F ds` |
|---|---|---|---|---|
| 1 constant | −0.022024 | +0.000934 | −5.476 % | −5.398 % |
| 2 schedule | −0.004037 | +0.002298 | −0.464 % | −1.605 % |
| 3 φ FLOOR | −0.000688 | +0.000842 | **+0.028 %** | −0.564 % |

The bill is bought back, in rung 61's own currency and not merely in `∫b ds`: 26 % of the
state-blind law's bleed and 52 % of rung 62's schedule's, with the end-of-ramp thrust bill
**machine-zero** (+0.028 %, inside settling) where the schedule still pays −0.46 %.

**The honest comparator is law 2, not law 1.** A constant bleed through a transient is a straw
man — it bleeds hardest where `φ` is already highest. The number this rung publishes is the
2× against rung 62's own schedule, not the 4× against the state-blind law.

### What § 0 fixes — the axis, before predictions

The three laws are the ladder's own information ordering, one lever over from the fuel side:

| | fuel path | airflow path |
|---|---|---|
| state-blind | — | rung 42 constant `b` |
| open loop, state-FED | rung 48 `Wf/pt3` feedforward | rung 62 `b(n_L)` schedule |
| CLOSED loop on the protected variable | rung 49 `φ` floor | **rung 64** |

---

## § 1 — The derivation, before any further probe

**(a) The lever is a function of the closure's own ROOT — a first in the ladder.** Every lever
from rung 42 to 63 is a function of the STATE VECTOR (`b_of(nu_lp, Tt2)`, `_arm(...)`); that
is precisely what made them RK4-legal. A φ-referenced valve is a function of `φ_lp = m_lp/n_lp`,
which is what the closure solves for. It stays RK4-legal for the same reason rung 50's `s`
did: it carries **no history and no latch** — the root is re-solved from scratch at every
sub-evaluation, so it is still a pure function of the state, just an implicitly-defined one.

**(b) `φ_lp` is monotone increasing in `b`, so the outer solve is a scalar bracketed root.**
The choked `A4` imposes the CORE flow; the FACE flow the closure must find to feed it carries
`1/(1−b)` (`m_imp = mdot_imp/(1−b) · √Tt2/pt2 / mcorr_lp_d`, `engine.py`:9020). More extraction
⇒ more face flow at the same `n_lp` ⇒ higher `φ_lp`. One `_illinois` root on `b ∈ [0, b_max]`
with two clamps — no nested Newton, no 2×2.

**(c) The two clamps ARE the two regimes.** `b = 0` is DORMANT (`φ` already above the floor,
and the closure dispatches to the rung-57 parent bit-for-bit); `b = b_max` is SATURATED (the
floor is violated and the valve can do no more). Between them the valve RIDES the floor and
rung 60's tautology pins `min φ_lp = φ_lim` exactly. Saturation is the only regime in which
the limiter's own set point is not what it delivers — which is why probe A's ceiling is a
statement about `b_max` and not about the law.

**(d) Rung 62's `_powers` trap applies directly.** The outer root is on the same `φ_lp` the
readers report. Rung 62 converged an inner Newton to 1e−12 on a residual the plant did not
use and got `n_L` back 5.3 % wrong with no exception anywhere. The guard is the same one that
caught it: an independent cross-check (here, that the committed `c["bleed"]` reproduces the
reported `φ_lp` and that the dormant branch is bit-for-bit rung 63).

**(e) This does NOT contradict rung 61.** Rung 61 compared two LEVERS with nothing matched and
found the compensating one bought back the coordinate and not the bill. Rung 64 compares three
LAWS of ONE lever at a matched coordinate. Different objects; the sentences invert because the
matched quantity moved from the bill to the coordinate.

**(f) The prediction rung 63 hands over.** Rung 52: *a self-releasing limiter cannot debit the
spool it watches*. This valve self-releases (it shuts as `φ` recovers) and it watches LP. On
the fuel path that law was proved for a fuel lever; rung 42 says a bleed is a degree of freedom
on the LP spool and **not** the HP, and rung 49 says an LP floor DEBITS the HP. Those three do
not obviously agree for an AIRFLOW lever.

---

## § 2 — PREDICTIONS, pre-registered

Everything below is untouched by § 0's probes. Numbered, scored in § 3, misses published.

**P1 — the ceiling is a WALL, and the b≡b_max march bounds it from above.**
With `φ_lim` set ABOVE 0.809515 (say 0.85), the floor saturates and the achieved `min φ_lp`
is (a) strictly BELOW `φ_lim` — the floor is VIOLATED, the first law in this family that
cannot deliver its own set point — and (b) `≤ 0.809515`, the `b ≡ b_max` march being an upper
bound on every admissible `b`-history. I predict (b) is *strict* but by less than 1e−2: a
saturating valve is at `b_max` only near the minimum and less open before it.

**P2 — the invisible authority (the exact one).**
With `φ_lim` = 0.80 fixed and `b_cap` raised 0.10 → 0.20, every reported quantity is
**bit-for-bit identical** (`b` peak was 0.09202 < 0.10, so the clamp is never touched). Once
unsaturated, the lever's authority limit is INVISIBLE to the law — the exact analogue of rung
60's "a floor pins its own coordinate", now on the authority axis rather than the currency one.

**P3 — the second map shape.**
On the tilted map (`a`=0.14, `b`=0.10, `c`=0.06, `σ`=0.2, `l`=0.85 both spools), matched at
`min φ_lp` = 0.80: the floor's `∫b ds` stays between 0.35 and 0.70 of the SCHEDULE's (shaped:
0.5187), and the floor's Δ`F_end` stays machine-zero (|Δ`F_end`/`F`| < 0.1 %) while the
schedule's stays ≥ 0.2 %. *(My rung-63 memory's own lesson: run the second shape BEFORE the
headline, not after.)*

**P4 — rung 52's law TRANSFERS to an airflow lever.**
The LP debit is not merely small but **structurally unavailable**: `min φ_lp` = `φ_lim`
exactly, so no LP debit is even expressible while the floor rides. The HP is debited — the
floor's `min φ_hp` sits BELOW the valve-shut reference — and the debit survives ds-refinement
(`ds` 0.005 → 0.0025 changes it by < 30 %) and the tilted map (same sign). Rung 49's
"an LP floor debits the HP" therefore holds for an AIRFLOW lever, and rung 52's
watched-spool immunity holds with it.

**P5 — the SIGN SPLIT between state-blind and state-fed.**
The constant-`b` law CREDITS the HP (`min φ_hp` above the valve-shut reference) while both
state-fed laws DEBIT it. Prediction: this survives ds-refinement and the tilted map, and the
mechanism is placement — a constant valve is still open at the HP's own (LATE, rung 41/49)
minimum where the state-fed laws have already shut.

**P6 — the ds-exactness of the tautology.**
`min φ_lp` − `φ_lim` is < 1e−9 at every `ds` in {0.01, 0.005, 0.0025} — the pinning is exact
arithmetic, not a grid artifact, because the floor is enforced INSIDE the closure rather than
between steps.

**P7 — THE REDUCE.**
`bleed_lim=None` ⇒ rung 63 bit-for-bit at every state (exact dispatch, per call), and a floor
whose `φ_lim` is below every `φ` on the march dispatches AWAY at every state rather than
computing a 0.0 position — the rung-62 discipline. `b_cap = 0.0` is refused by assertion, not
silently reduced (a limiter with no authority is a different object from an absent one).

**P8 — the composite is REFUSED, and rung 63 § 3 says why.**
A `φ`-referenced FUEL floor (rung 49's `surge`) and this `φ`-referenced VALVE cannot compose:
rung 63 § 3 already showed the fuel floor and the imposed valve have no composable middle
(the valve DISARMS the floor over a band, and above it the valve's credit is exactly zero).
With BOTH watching `φ_lp`, whichever acts first pins the coordinate and the other reads its
own set point as satisfied. Prediction: armed together at the same `φ_lim`, the fuel leg's
`fuel_removed` is EXACTLY 0.0 — rung 63's disarming, now structural rather than banded. The
three-lever plant (fuel + bleed + stator) stays rung 63's open seam.

---

## § 3 — SCORING

**Six HIT, two REFUTED.** Both refutations are published in full below, and one of them
(P8) turned out stronger than the prediction it killed.

### P1 — HIT, in all three parts.

Floor set 10 % above the fully-open march's own minimum (`φ_lim` = 0.890467 against a ceiling
of 0.809515), shaped map, `ds` = 0.005:

| law | `min φ_lp` | `sm` | `b` at the `φ` minimum | `b` peak |
|---|---|---|---|---|
| shut | 0.735442 | 0.337167 | 0.000000 | 0.00000 |
| schedule | 0.788669 | 0.433944 | 0.073604 | 0.08454 |
| **full `b = b_max`** | **0.809515** | 0.471846 | 0.100000 | 0.10000 |
| **over-set floor** | 0.807363 | 0.467933 | 0.100000 | 0.10000 |

The floor SATURATES and is VIOLATED by −0.083104 (a), lands BELOW the fully-open march (b),
and the gap is −2.152e−3 — strictly negative and under 1e−2, as predicted including the
reason (a saturating valve is at `b_max` only near the minimum and less open before it).

### P3 — HIT. The second map shape moves nothing that matters.

| | shaped | tilted |
|---|---|---|
| `∫b ds` floor / schedule | 0.5187 | 0.4906 |
| `∫b ds` floor / constant | 0.2552 | 0.2484 |
| floor Δ`F_end` | +0.028 % | +0.030 % |
| schedule Δ`F_end` | −0.464 % | −0.488 % |

Predicted band was 0.35–0.70 (both inside), floor |Δ`F_end`| < 0.1 % (both), schedule
≥ 0.2 % (both).

### P4 — HIT.

`ds` 0.01 → 0.005 → 0.0025, floor at `φ_lim` = 0.80 against the valve-shut plant:

| `ds` | `min φ_lp` − `φ_lim` | `∫b ds` | Δ`min φ_hp` | Δ`nu_lp_end` |
|---|---|---|---|---|
| 0.01 | −5.551e−16 | 0.038250 | −5.206554e−4 | −0.000688 |
| 0.005 | −1.443e−15 | 0.038249 | −5.307096e−4 | −0.000688 |
| 0.0025 | −1.332e−15 | 0.038250 | −5.307096e−4 | −0.000688 |

The HP debit's sign holds and it moves 1.9 % across a 4× refinement (predicted < 30 %); it is
converged to all printed digits between the last two grids. And the LP debit is indeed
*structurally unavailable* — `min φ_lp` IS `φ_lim` to 1.4e−15.

### P5 — HIT, both shapes.

Δ`min φ_hp` against the valve-shut reference at the matched coordinate:

| law | shaped | tilted |
|---|---|---|
| constant | **+2.326899e−3** | **+2.179103e−3** |
| schedule | −2.849687e−4 | −7.952209e−4 |
| floor | −5.307096e−4 | −9.121452e−4 |

The state-blind law CREDITS the HP; both state-fed laws DEBIT it. Sign split as predicted, on
both shapes, and the floor debits hardest of the three.

### P6 — HIT. `min φ_lp` − `φ_lim` ∈ {−5.6e−16, −1.4e−15, −1.3e−15} across the three grids
(predicted < 1e−9): the pinning is exact arithmetic, because the floor is enforced inside the
closure and not between RK steps.

### P7 — HIT. `bleed_lim=None` marches bit-for-bit rung 63 (341 points, seven keys each), a
floor below every `φ` on the march dispatches away at every state, and both rung-42/62 arming
modes reproduce rung 63 exactly through the new class. `b_max = 0.0` is refused by assertion.

### P2 — **REFUTED**, and the refutation SPLITS into two different roundoff mechanisms.

Predicted bit-for-bit under `b_cap` 0.10 → 0.20 with the clamp never touched (`b` peak
0.092025 under all of 0.10 / 0.20 / 0.40). Measured: **13 of 17 float keys differ.**

**(i) The values — the SOLVER PATH, and the prediction's substance survives.** `_solve_b`
brackets on `[0, b_max]`, so the clamp is the Illinois solve's UPPER ENDPOINT and enters the
iterate sequence even when it never binds. At one state, `φ_lim` = 0.877009559683206:

| `b_cap` | committed `b` | `φ_lp` − `φ_lim` | Δ`b` vs 0.10 |
|---|---|---|---|
| 0.10 | 0.02471202858122729 | −3.331e−16 | — |
| 0.20 | 0.02471202858122653 | +0.000e+00 | −7.598e−16 |
| 0.40 | 0.02471202858122732 | +0.000e+00 | +3.816e−17 |

Two paths, two roots ~1e−15 apart, both delivering the set point to roundoff. Marched over
341 steps, **every physical key agrees to ≤ 6.6e−16 relative** across a 4× clamp:

| key | base | worst rel. diff |
|---|---|---|
| `Tt4_peak` | 1714.3766799151 | 6.63e−16 |
| `thrust_end` | 606.3605844082 | 2.63e−15 |
| `thrust_int` | 858.0177340777 | 3.98e−16 |
| `min_phi_lp` | 0.8000000000 | 8.33e−16 |
| `min_phi_hp` | 0.8606378964 | 5.16e−16 |
| `nu_lp_end` | 0.9410179039 | **0** |
| `b_int` | 0.0382488803 | 2.90e−15 |

So "bit-for-bit" was mine to get wrong; "invisible" is right, and no bill in this rung moves
by more than 3e−15 relative when the lever's authority is quadrupled.

**(ii) The LOCATIONS — a genuine structural fact, and it is content.** Three keys differ by
O(1), not by roundoff:

| key | `b_cap` 0.10 | rel. diff at 0.20 | at 0.40 |
|---|---|---|---|
| `s_at_min_lp` | 0.0600000000 | **3.333** | **2.167** |
| `nu_at_min_lp` | 0.7488669823 | 2.90e−2 | 1.51e−2 |
| `b_at_min_lp` | 0.0618771497 | 4.80e−1 | 4.55e−1 |

All three are ARGMIN keys, and a floor that RIDES pins `φ_lp` to `φ_lim` over an INTERVAL. So
the minimum's VALUE is a result (rung 60) and its LOCATION is not one — the argmin is decided
by which point happens to sit one ulp lower, which the (i) perturbation then flips.

> **A riding floor destroys the LOCATION of the minimum it pins.**

That BOUNDS rungs 44–52 on their own terms: those rungs report WHERE a surge minimum sits, and
rung 50's entire finding is that a release edge RELOCATES both spools' minima to itself. On a
plant with a riding floor that object does not exist. `_bill_cell` therefore reports
`plateau_span` / `plateau_pts` and marks the three argmin keys as diagnostics.

### P8 — **REFUTED, and the truth is stronger than the prediction.**

Predicted `fuel_removed` EXACTLY 0.0 beside the valve at the same set point — rung 63's
disarming, made structural. Measured **2.532218e−4**, against 8.939858e−3 for the fuel leg
alone: the leg is *not* dormant. It removes 2.8 % of its solo clip.

But the credit is **machine-zero**, and `m_i` agrees to 15 digits across the `fuel` / `valve` /
`both` cells (0.568181818, i.e. `T_c` − 1/0.80). So the leg ACTS, and its action is inert.

*(Correction, caught by the gate rather than by me: the first run gave a credit of exactly
0.0 and the gate run gave −4.4e−16. Inertness here is MACHINE-precision, not bit-equality —
the degenerate solve returns an arbitrary point of a continuum, so its output wobbles at
1e−16 between runs. Publishing "exactly zero" off one run would have been asserting on
roundoff a second time, in the very section about roundoff.)*

The mechanism is in `_surge_fuel` (`engine.py`:4743): it solves `G(w) = φ_lim − φ(w) = 0` in
the fuel `w`, on the premise (its own docstring) that *"φ falls MONOTONICALLY with fuel at
fixed spool speeds"*. On a valve-armed plant that premise is **destroyed** — the valve re-pins
`φ_lp` to `φ_lim` at ANY fuel, so `G ≡ 0` to roundoff across the whole bracket and the solve
is DEGENERATE: it returns an arbitrary point of a continuum.

So the correct statement is not "the valve disarms the fuel leg." It is:

> **a closed-loop lever does not disarm a second limiter on the same variable — it DELETES
> that limiter's PLANT.** The fuel leg's authority over `φ` is annihilated where the valve
> rides, its set-point solve becomes ill-posed, and whatever it then removes is inert.

That is the sharper form of rung 63 § 3's "no composable middle", and it connects directly to
`docs/phi-rate-limiter-negative.md` (fuel's authority over `φ` inverts between level and
derivative): here that authority is not inverted but **zero**.

**Consequence for what may be published.** The advisor's blocking correction, and it is
sharper than my own reading: at exact tangency `_surge_fuel` chooses between its dormant
return (`ghi <= 0.0`) and the 60-iteration degenerate hunt on **the sign of one ulp** of
`φ_lim − φ_lp` (`engine.py`:4763). So the 2.53e−4's very EXISTENCE is a roundoff coin flip,
not merely its magnitude. **No number about it is a result** — the `G ≡ 0` derivation carries
the claim, and the residual is evidence the instrument is ill-posed at tangency.

**The control that separates tangency chatter from a broken leg** — a fuel floor set strictly
BELOW the valve's set point, armed plant, `ds` = 0.005:

| fuel `φ_lim` | removed on BARE | removed on ARMED |
|---|---|---|
| 0.772500 | 1.869663e−2 | **0.000000e+00** (exactly) |
| 0.794500 | 1.019138e−2 | **0.000000e+00** (exactly) |
| 0.800000 (= the valve's) | 8.939858e−3 | 2.532218e−4 *(chatter)* |

Strictly below, the leg is exactly dormant on the armed plant while still biting hard on the
bare one. The residual appears only at tangency. Gated as inertness + this control; the
tangent value is asserted in neither direction.

### § 3a — the cost the degeneracy imposes, recorded

The degenerate hunt is not free: `_surge_fuel` runs its full 60 iterations and each evaluation
is an OUTER root over closures, so a single `both` cell costs ~1e3× a normal one. Probe D's
three-grid sweep of it was effectively non-terminating and was killed; the rung's `both` cell
is measured once, at `ds` = 0.005. That cost IS the finding wearing its working clothes — a
limiter whose plant has been deleted does not fail, it grinds.
