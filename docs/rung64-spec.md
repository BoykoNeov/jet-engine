# Rung 64 — the φ-REFERENCED BLEED LIMITER: what a control LAW can and cannot buy

Rung 63's named next seam. Every arming of the interstage valve from rung 42 to 63 was OPEN
LOOP — a constant position (42), or a schedule `b(n_L)` read off the state (62). This one
closes the loop on the protected variable itself, which makes it to rung 62 exactly what rung
49's `SurgeLimiter` is to rung 48's feedforward `AccelSchedule`: the same step, one lever over.

**HEADLINE: a limiter's LAW cannot buy PROTECTION, only its PRICE.**

The ceiling on the protected coordinate is `min φ` over the FULLY-OPEN march — a property of
`b_max`, the lever's AUTHORITY, which is hardware — and `b = b_max` is *itself an open-loop
law*. So a closed loop buys nothing on the coordinate, and a floor set above that ceiling
saturates and is VIOLATED. What feedback buys is the BILL: at a coordinate matched EXACTLY,
the closed loop pays 49–52 % of rung 62's schedule's bleed and 25 % of the state-blind law's,
with an end-of-ramp thrust bill that is machine-zero.

That **INVERTS rung 61** without contradicting it, and **BOUNDS rungs 46–52 on a third axis**:
rung 53 bounded that family's CURRENCY, rung 57 its CLOCK, and this its CEILING.

Anchor: `docs/plans/rung64-anchor-phi-bleed-limiter.md` (six predictions HIT, two REFUTED,
both published in full — one of them turned out stronger than the prediction it killed).

---

## 0. THE INSTRUMENT — the tautology that would have counterfeited the rung

A valve commanded to hold `φ_lp ≥ φ_lim`, on a lever whose entire credit runs through `φ` and
whose `v ≡ 0` (so rung 53's two currencies collapse to `M_i = T_c − 1/φ`), **pins the
coordinate it watches**. That is rung 60's tautology verbatim, and rung 63's `floor_dichotomy`
already re-found it on this very plant. Written naively, rung 64 would publish
`min φ_lp == φ_lim` to 1e−15 and call it a finding.

So before any prediction was written, two discriminating probes decided whether the rung has
content the tautology does not already own. They are declared in the anchor's § 0 as the
rung's GIVEN, not as scored predictions, and everything below rests on them.

**The tautology is not discarded — it is repurposed as the MATCHING INSTRUMENT.** It is what
lets § 2 hold the protected coordinate fixed to 1.4e−15 across three different control laws
and let only the price move.

### Scope, pre-checked rather than discovered

Steady flight condition, fully-choked branch throughout (gate 7 checks it at the WIDEST
position any rung-64 law can command — a saturating floor sitting at `b_max` for most of the
ramp). CPG working gas, two-spool map-matched plant, rung 45's accel fuel ramp. The valve is
instantaneous and unlagged.

---

## 1. THE CEILING — what feedback does NOT buy

A controlled valve saturates when even `b = b_max` cannot hold the floor, so its saturation
boundary is `min φ` over a march with the valve fully open throughout. That needs no new
plant. On rung 63's grid (`φ_surge` = 0.55, `b_max` = 0.10, `r` = 0.5, `ds` = 0.005,
`n_lo` = 0.65):

| plant | shaped | tilted |
|---|---|---|
| valve SHUT | 0.735442 (`sm` 0.337167) | 0.737373 (0.340678) |
| rung-62 SCHEDULE | 0.788669 (0.433944) | 0.789699 (0.435816) |
| constant `b = b_max` | **0.809515 (0.471846)** | **0.808539 (0.470071)** |

The schedule commands only `b` = 0.0736 / 0.0760 of `b_max` = 0.10 **at its own `φ` minimum**
— it is not saturated — so there is a real gap of +0.0208 / +0.0188 above rung 63's published
upper band edge. A closed loop can take that gap.

**But the third row is itself an OPEN-LOOP law, and it attains the ceiling.** So the gap is
about the schedule's PLACEMENT, not about the loop being open, and:

> the ceiling on `min φ` is a property of `b_max` — the lever's AUTHORITY — and not of
> feedback. A closed loop buys nothing over the best open-loop law on the protected coordinate.

**The witness.** A floor set 10 % above the fully-open march's own minimum
(`φ_lim` = 0.890467 against a ceiling of 0.809515) saturates and is VIOLATED by −0.083104 —
the first law in this family that cannot deliver its own set point, and it fails on *hardware*,
not on control. Its achieved minimum lands at 0.807363, i.e. **below** the fully-open march by
−2.152e−3: a saturating valve sits at `b_max` only near the minimum and less open before it,
so the fully-open march bounds every admissible `b`-history from above and is not attained.

Reader: `LimitedBleedTransient.authority_ceiling`.

---

## 2. THE BILL — what feedback DOES buy

The advisor's blocking condition on this rung, and it was the right one: rung 61 found the
bleed's φ-credit is a **loading rebate** with 73–102 % of the overspeed surviving, so "the
bill" is NOT `∫b ds` — it is the overspeed and the thrust. A law that saved 40 % of the bleed
and 5 % of the overspeed would be buying back the thing rung 61 says is not the bill.

Three laws matched to the same `min φ_lp` = 0.80 — strictly inside [0.7354, 0.8095], so the
floor genuinely rides and rung 60's pinning gives the match for free; the two open-loop
settings are DRIVEN to it by an outer root over marches. Shaped map:

| law | `min φ_lp` | `∫b ds` | `b` peak | Δ`nu_lp_end` | Δ`F_end` | Δ`∫F ds` | Δ`min φ_hp` |
|---|---|---|---|---|---|---|---|
| 1 constant `b*` = 0.088163 | 0.800000 | 0.149876 | 0.08816 | −0.022024 | −5.476 % | −5.398 % | **+2.327e−3** |
| 2 schedule `b_max*` = 0.118220 | 0.800000 | 0.073741 | 0.10130 | −0.004037 | −0.464 % | −1.605 % | −2.850e−4 |
| 3 **φ FLOOR** `φ_lim` = 0.80 | 0.800000 | **0.038249** | 0.09202 | **−0.000688** | **+0.028 %** | −0.564 % | −5.307e−4 |
| 0 valve SHUT (reference) | 0.735442 | 0 | 0 | — | — | — | — |

Match error across all three: **1.44e−15**. Tilted map: `∫b` ratios 0.2484 / 0.4906, floor
Δ`F_end` +0.030 %, schedule −0.488 % — the same picture.

Three things are the rung:

**(a) The ordering is the ladder's own information ordering.** State-blind (42) → state-fed
open loop (62) → closed loop on the protected variable (64), and the bill falls monotonically
along it in `∫b ds`, in the overspeed AND in the integrated thrust — the three that rung 61
showed need not track.

| | fuel path | airflow path |
|---|---|---|
| state-blind | — | rung 42 constant `b` |
| open loop, state-FED | rung 48 `Wf/pt3` feedforward | rung 62 `b(n_L)` schedule |
| CLOSED loop on the protected variable | rung 49 `φ` floor | **rung 64** |

**(b) The honest comparator is law 2, not law 1.** A constant bleed through a transient is a
straw man — it bleeds hardest where `φ` is already highest. The number this rung publishes is
the **2×** against rung 62's own schedule, not the 4× against the state-blind law.

**(c) The closed loop's end-of-ramp thrust bill is MACHINE-ZERO** (+0.028 % / +0.030 %, inside
settling) where the schedule still pays −0.46 % / −0.49 %. It self-releases, so it has left
the machine by settle: its bill is transient-only, and that is the practical content of the
2× — not that it bleeds less, but that it has stopped bleeding.

**Why this inverts rung 61 without contradicting it.** Rung 61 compared two LEVERS with
nothing matched and found the compensating one bought back the COORDINATE and not the BILL.
This compares three LAWS of ONE lever at a matched coordinate. The sentences turn over because
the matched quantity moved from the bill to the coordinate; both are the same fact seen from
its two sides.

Reader: `LimitedBleedTransient.matched_bill`.

### The spool split, and two prior rungs transferring to an airflow lever

The LP debit is not merely small but **structurally unavailable**: `min φ_lp` IS `φ_lim` to
1.4e−15 while the floor rides, so no LP debit is even expressible. That is rung 52's *"a
self-releasing limiter cannot debit the spool it watches"*, transferred from a fuel lever to an
airflow one. The HP **is** debited (−5.3e−4 / −9.1e−4) — rung 49's *"an LP floor debits the
HP"*, same transfer — while the state-blind constant valve **CREDITS** it (+2.3e−3 / +2.2e−3),
because it is still open at the HP's own LATE minimum where both state-fed laws have shut.

The HP debit is O(1e−4) against an LP move of O(1e−1), so it is gated for grid-independence:
sign holds and magnitude moves 1.9 % across a 4× refinement, converged to all printed digits
between the last two grids.

---

## 3. THE THIRD FINDING — a closed loop DELETES a second limiter's plant

Rung 63 § 3 found a `φ` FUEL floor and the IMPOSED valve have no composable middle, over a
BAND whose edges are the two plants' own minimum `φ`. With both objects watching `φ_lp`, the
band collapses — and the reason is stronger than disarming.

`_surge_fuel` (`engine.py`:4743) solves `G(w) = φ_lim − φ(w) = 0` in the fuel `w`, on its own
stated premise that *"φ falls MONOTONICALLY with fuel at fixed spool speeds"*. Where this valve
RIDES it re-pins `φ_lp` to `φ_lim` at **any** fuel. So on the armed plant

    dφ_lp/dWf  =  0        and        G ≡ 0  across the entire bracket

and the fuel leg's set-point solve is DEGENERATE: it returns an arbitrary point of a
continuum.

> **A closed-loop lever does not DISARM a second limiter on the same variable — it DELETES
> that limiter's PLANT.**

Its authority over `φ` is not inverted (`docs/phi-rate-limiter-negative.md`, where fuel's
authority flips sign between level and derivative) but **zero**.

**What may be read from this, and what may not.** At exact tangency `_surge_fuel` chooses
between its dormant early return and a 60-iteration degenerate hunt on **the sign of one ulp**
of `φ_lim − φ_lp`. So the residual it removes there (measured 2.53e−4 against 8.94e−3 for the
leg alone) has a roundoff coin flip for its very *existence*. **No number about it is a
result.** What is stable, and what the gate asserts:

- **Inertness** — the composite's `m_i` and `min φ` agree with the valve-alone march, and the
  leg's credit is machine-zero, against a bare-plant credit of O(1e−2) in the same currency:
  inert by five orders. To MACHINE precision and deliberately not to the bit — one run gave
  exactly 0.0 and the next −4.4e−16, because the degenerate solve returns an arbitrary point
  of a continuum. Demanding bit-equality here would assert on the very roundoff § 3 exists to
  expose.
- **The control** — a fuel floor set strictly BELOW the valve's set point is EXACTLY dormant on
  the armed plant (0.000000e+00 at both 0.7725 and 0.7945) while removing 1.87e−2 / 1.02e−2 on
  the bare one. The residual appears only at tangency.

The degeneracy also has a cost worth recording: a single armed-plus-armed cell runs ~1e3× a
normal one, because every one of those 60 iterations is an outer root over closures. **A
limiter whose plant has been deleted does not fail — it grinds.**

Reader: `LimitedBleedTransient.floor_refusal`.

---

## 4. A RIDING FLOOR DESTROYS THE LOCATION OF THE MINIMUM

This came out of a refuted prediction (anchor P2) and it bounds four rungs.

A floor that rides pins `φ_lp` to `φ_lim` over an **interval**, so the minimum's VALUE is a
result (rung 60) and its LOCATION is not one — the argmin is a 1-ulp tie among many points.
Measured: doubling and quadrupling an *untouched* `b_max` moves every physical quantity by
≤ 6.6e−16 relative (`Tt4_peak`, `thrust_end`, `thrust_int`, `min φ_lp`, `min φ_hp`, `∫b ds`,
`nu_lp_end` — the last exactly 0), while three ARGMIN keys move by O(1): `s_at_min_lp` by a
factor 3.3, `b_at_min_lp` by 48 %.

The 1e−15 perturbation that flips them is itself an artifact — `_solve_b` brackets on
`[0, b_max]`, so the clamp is the Illinois solve's upper endpoint and enters the iterate
sequence even when it never binds (three clamps, three roots ~1e−15 apart, all delivering the
set point to roundoff). But the *sensitivity* is real and structural.

**This bounds rungs 44–52 on their own terms.** Those rungs report WHERE a surge minimum sits;
rung 50's entire finding is that a release edge **relocates both spools' minima to itself**. On
a plant with a riding floor, that object does not exist. `_bill_cell` therefore reports
`plateau_span` / `plateau_pts` — the finite stretch of ramp over which `φ` sits on the floor —
and marks the three argmin keys as diagnostics rather than results. The valve-shut march has
`plateau_pts == 1`; the floored one spans a finite stretch.

Rungs 44–52's own readings are **not** invalidated: their floors are fuel-side legs whose
windows close inside the ramp, and rung 49/50's minima are read where the leg is NOT riding.
What is bounded is the general claim, and the boundary is stated rather than assumed.

---

## Verification gates (`tests/test_rung64.py`)

**Gate 1 — THE REDUCE.** `bleed_lim=None` marches bit-for-bit rung 63 (341 points × 7 keys);
a floor below every `φ` on the march dispatches to the parent AT EVERY STATE, witnessed against
the valve-shut march; both rung-42/62 arming modes reproduce rung 63 exactly through the new
class; the single-spool design run is bit-for-bit rung 6.

**Gate 2 — THE OBJECT.** `b_max = 0` is refused by assertion — a limiter that cannot act is a
DIFFERENT object from an absent one, and that distinction is the whole rung. Rung 62's two-way
exclusivity assert is extended to three, not replaced.

**Gate 3 — THE TRAP, fourth instance.** `at_stator` carries the floor (rung 61's `at_setting`,
rung 62's `at_stator`, rung 63's `_isolating` — same failure mode each time); `_isolating`
counts the floor as an arming mode; and a trial position never leaks out of the outer solve
(the committed `b`, re-run as a rung-42 constant, reproduces the same state).

**Gate 4 — THE CEILING.** `shut < schedule < ceiling`; the schedule is NOT saturated at its own
minimum; the over-set floor is VIOLATED and lands strictly below the fully-open march by less
than 1e−2. Plus the clamp's invisibility to ≤ 1e−14 on every physical key, the plateau, and
the tautology's exactness (< 1e−9) at three grids.

**Gate 5 — THE BILL.** `∫b`, overspeed and integrated thrust all order
constant > schedule > floor, on BOTH map shapes, at a match error < 1e−9 with the floor
unsaturated; the floor's end thrust bill machine-zero where the schedule's is not; the HP sign
split; and the HP debit's grid-independence.

**Gate 6 — THE DELETION.** Inertness (credit exactly 0.0, `m_i` and `min φ` bit-identical to
the valve-alone march) plus the strictly-below control (exactly dormant armed, biting bare).
It DELIBERATELY does not assert on the tangent residual in either direction.

**Gate 7 — the modelling floor.** Every march stays on the choked branch, checked at the widest
position each law can command including a floor saturated throughout.

---

## Concessions

Every one rungs 62/63 list, all inherited, plus:

- **The valve is INSTANTANEOUS and unlagged.** Rungs 47/51/52 spent three rungs on what a
  finite actuator does to a FUEL-side leg; nothing here repeats that, and the lag's shape
  remains rung 52's open seam. A lagged bleed valve is this rung's most obvious next step and
  it is not taken.
- **`φ_lim` and `b_max` are both imposed.** `φ_lim` rides on rung 36's disclaimed `phi_surge`
  exactly as rung 49's does; `b_max` is rung 42's valve size. The MAGNITUDE of every bill is
  therefore disclaimed — the ORDERING and the SIGNS are the claims.
- **The floor watches the LP and only the LP.** Rung 42 established the valve is a degree of
  freedom on the LP spool and NOT the HP, and the outer solve needs `φ` monotone in `b`, which
  the choked-`A4` argument gives for the LP face flow and does not give for the HP. Disclosed
  rather than parameterised, so no untested branch ships.
- **The outer root carries its bracket's upper endpoint into its answer** at ~1e−15 (§ 4).
  Measured and disclosed rather than fixed: making it exact costs closure evaluations inside an
  RK4 march, and nothing physical moves by more than 3e−15 relative. This is a third instance
  of the family `docs/plans/todo-solver-tolerance-audit.md` closed as NEGATIVE — real, benign,
  and now documented in a place a future rung will find it.
- **`_surge_fuel` is not well-posed on a plant whose `φ` another loop pins** (§ 3). Left as
  found: repairing it would mean giving rung 49's leg a degeneracy test it has never needed,
  and the ill-posedness IS this rung's finding.

## What it does to its neighbours

- **INVERTS rung 61** — same fact, opposite side (§ 2).
- **BOUNDS rungs 46–52 on a third axis** — CURRENCY (rung 53), CLOCK (rung 57), now CEILING.
- **BOUNDS rungs 44–52's minimum-LOCATION readings** on a floored plant (§ 4), without
  invalidating their own (their legs are not riding where they read).
- **SHARPENS rung 63 § 3** — "no composable middle" becomes "the plant is deleted" (§ 3).
- **EXTENDS rung 49/52 to an airflow lever** — both spool-side laws transfer intact.
- **CONFIRMS rung 42** — the valve remains a DoF on the LP and not the HP: the HP effect is
  three orders below the LP one under every law.

## The next seam

**THE LAGGED VALVE.** § 3's deletion and § 4's plateau both rest on the valve being
INSTANTANEOUS — it pins `φ` at every sub-evaluation, which is what makes `dφ/dWf` exactly zero
and the minimum exactly flat. Give it rung 47's first-order lag and neither can survive
unchanged: a trailing-edge actuator cannot pin what it has not caught up to, so the plateau
should break into a genuine minimum and the second limiter should get part of its plant back.
Rung 52's *"a self-releasing limiter pins its own trigger"* predicts the shape, and rung 51's
two-sided bracket is the instrument. That also finally reaches rung 52's own standing seam
(the lag's SHAPE and the two-lag CASCADE) from the airflow side.

Still open beside it: **fuel + bleed + STATOR**, all three on one plant (rung 63's, untouched
here).

## Anchor

`docs/plans/rung64-anchor-phi-bleed-limiter.md` — the derivation written before any probe, the
two discriminating probes declared as the rung's given, eight numbered predictions and their
scoring with both refutations published in full.
