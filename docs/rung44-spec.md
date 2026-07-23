# Rung 44 — The transient two-spool surge line: the excursion is SCHEDULE-slaved, and the LP eats it

Rungs 40 and 41 both closed by naming the **same** open seam in almost the same words: *"the
transient surge line — measuring rung 40's complex inter-spool mode against rung 41's stability
boundary — rung 41 is steady-only, so no surge-**survival** claim yet."* Rung 44 marches the
running point against the surge line and measures what actually happens.

> **The two-shaft acceleration drives BOTH compressors toward surge, the LP spool eats
> ~1.6–2.2× the HP's excursion (rung 41's steady exposure split SURVIVES dynamically) — but the
> excursion is SCHEDULE-SLAVED: it is set by the fuel/`Tt4` RAMP RATE against the shaft clock,
> is INVARIANT to the inter-spool clock ratio `rho` (<2 % over a 25× `rho` range), and is
> INDEPENDENT of the LP-map complex mode (the mode-free `hp-only` pair has the LARGEST LP/HP
> ratio). The two objects rung 40 spent its energy on — `rho` and the complex mode — are BOTH
> surge-irrelevant. What governs the transient surge hazard is the mundane one: how fast you
> slam the throttle.**

Like rungs 7–30, 36 and 41 this is a **pure diagnostic beside the cycle**: the rung-44 methods
only *read* rung 40's marched trajectory and rung 41's imposed `phi_surge`; they add **no state**
and feed nothing back. Arming a surge line leaves rung 40's `integrate`/`equilibrium`/`jacobian`
**bit-for-bit** unchanged (the rung-41 witness, one rung on), and the default
`build_turbojet(…).run(…)` design path is untouched (**bit-for-bit rung 6**).

---

## Sign-space only — the hard constraint, not a caveat

Rung 40 **disclaimed** the complex mode's magnitude (`|Im/Re| ≤ 0.25`, "no visible ringing"), and
rung 41 **disclaimed** every surge margin (`phi_surge` is imposed). Neither imposition vanishes in
rung 44, so rung 44 **cannot** make the surge-**survival** claim rung 41 said it could not make.
What it delivers is **signs**: the direction the transient moves the operating point relative to
the steady running line, which spool eats it, and whether the inter-spool dynamics matter. The
**rung-36 discipline is enforced exactly: report the crossing, gate the flip.** `transient_surge_margin`
(unlike the steady `surge_margin`, which *asserts* the point sits clear) **allows** `phi < phi_surge`
and records it — but the absolute crossing rides on the imposed floor and the ramp rate and is
**disclaimed**; the **gated object is the excursion's SIGN**.

---

## The measured object: `phi_excursion` (the phi analogue of rung 40's `slip_excursion`)

March an accel ramp `Tt4_lo → Tt4_lo+dTt4` from the running-line start (`nu0` = the matched speeds
at `Tt4_lo`), and take the signed extremum of

```
    e_spool(s) = phi_spool(s) − phi_spool,steady(Tt4(s))          (referenced to the RUNNING LINE)
```

per spool. Referencing to the running line at the **instantaneous** `Tt4` (not the starting point)
is rung 34/40's discipline: it strips the steady schedule's own drift with `Tt4` so only the
**transient lag** remains. (An early probe that referenced to the *starting* phi read the sign of
sibling excursions backwards, exactly as rung 40's first `slip_excursion` probe did.)

**The mechanism.** On an acceleration the schedule steps `Tt4` up while both shaft speeds **lag**;
the forward-choke closure then sits the operating point at higher `Tt4` / lower relative speed,
which is **lower flow coefficient** — *toward* surge. This is the classic acceleration-surge swing,
and it is here as a **computed** sign, not an assumed one (the closure could have gone either way).

---

## THE THREE FINDINGS

### (1) The excursion is toward surge on BOTH spools, and the LP eats more — rung 41 SURVIVES dynamically

Measured, five shape pairs, accel `Tt4` 1000 → 1400:

| shape pair | `ext_lp` | `ext_hp` | `|ext_lp|/|ext_hp|` |
|---|---|---|---|
| flow/press | −0.194 | −0.102 | 1.90 |
| press/flow | −0.185 | −0.115 | 1.61 |
| tilted | −0.193 | −0.110 | 1.76 |
| steep | −0.175 | −0.097 | 1.81 |
| **hp-only** (LP flat) | **−0.226** | −0.101 | **2.23** |

Both spools swing the **same sign** (`ext < 0`, toward surge), and the **LP eats 1.6–2.2× the HP's
excursion at every point** — rung 41's *steady* exposure asymmetry (`phi_L` takes the throttle
excursion, `phi_H` is shielded and bounded) **extends into the transient**. A **confirmation +
extension**, not a new sign. Decel is the exact mirror (`ext > 0` on both, *away* from surge). Gate 3
asserts the sign and the `> 1.4×` LP-over-HP ratio, shape-robust; the magnitudes are disclaimed.

### (2) The excursion is SCHEDULE-slaved — NOT `rho`, NOT the complex mode

This is the load-bearing finding and the cross-rung correction. Rung 40's two dynamically
interesting objects are **both surge-irrelevant**:

- **`rho`-INVARIANT.** Over `rho` ∈ [0.2, 5.0] (a 25× range) `ext_lp` moves from −0.191 to −0.194 —
  **<2 %** — and the extremum sits at `s = r_ramp` (the end of the ramp) for **every** `rho`. The
  inter-spool clock ratio, decisive over *which spool leads* (rung 40's `sigma_crit`) and over
  *whether the mode is complex* (rung 40's band), is **powerless over the surge excursion**. This is
  rung 40's own *"the finite-ramp excursion is schedule-slaved, not lead-governed"* scope-limit,
  now demonstrated on the **surge** axis.
- **RAMP-RATE-driven.** Over `r_ramp` ∈ [0.1, 5.0] `ext_lp` sweeps −0.231 → −0.047 (**~5×**,
  monotone): the faster the `Tt4` slam relative to the shaft clock, the deeper the swing toward
  surge. *This* is the governing variable — the schedule against the shaft response, the real
  slam-accel hazard.
- **MODE-INDEPENDENT.** The airtight leg is the **damping ratio**: every sampled
  `|Im/Re|max ≤ 0.164 < 0.25`, so the ring e-folds well before a quarter cycle and **cannot** carry
  the operating point across a line the steady point clears — the mode is surge-irrelevant on these
  maps by its own strength, independent of the excursion. **Corroborating (not necessary), a partial
  artifact NAMED in rung 41's register:** `hp-only` (LP map flat) has **no complex band at all**
  (rung 40's discriminator) yet shows the **LARGEST** LP/HP ratio (2.23×) — the asymmetry is present,
  indeed largest, with *no* mode, so the mode is **not** its cause. But swapping LP-shaped → LP-flat
  also changes the LP loading, so the 2.23-vs-1.90 *value* confounds mode-absence with a loading
  change and is **not** claimed as a proof; the damping ratio carries the claim. Gate 4 asserts
  `rho`-invariance, ramp-rate monotonicity, `|Im/Re| < 0.25` on every pair, and — as corroboration —
  the mode-free pair's band is `None` while its ratio is the largest.

**The cross-rung correction.** The tempting headline — *"the LP-map complex mode is the transient
surge story"* — is **false**. The mode exists (rung 40), but the surge excursion is governed by the
schedule lag, which is present with or without it. Rung 44 is to the surge axis what rung 40's
scope-limit was to the lead threshold: the exotic inter-spool dynamics do not govern the observable.

### (3) The transient CAN cross a line the steady running line clears — report the crossing, gate the flip

`transient_surge_margin` marches the ramp against the imposed `phi_surge` and returns, per spool, the
minimum `phi(s) − phi_surge` **allowing it to go negative**. Measured (flow/press, accel
1000 → 1400, `r_ramp = 0.3`): steady min `phi_lp` over the ramp = **0.7731**, transient min `phi_lp`
= **0.7408**. A floor set between them (`0.7408 < phi_surge < 0.7731`) is **cleared by every steady
point but CROSSED by the transient** — the first surge-relevant transient statement in the project.

Per the rung-36 discipline this is **reported, not gated as a magnitude**: the crossing depth rides
on the imposed `phi_surge` and the ramp rate, both disclaimed. What gate 5 asserts is the **flip's
sign** — that `transient_surge_margin`'s min margin is **strictly below** the steady margin at the
same instantaneous `Tt4` on the accel (and strictly above on the decel), on the LP spool, every
shape pair. The method **records** a crossing when the floor is placed to force one; it never
**asserts** the point is clear.

---

## Reduce-to-prior contract (the spine)

- **Read-only ⇒ rung 40 bit-for-bit.** The rung-44 methods (`phi_excursion`,
  `transient_surge_margin`) only *read* `integrate`/`match`; they add no state and never write
  `phi_surge`. So arming a surge line leaves `TwoSpoolTransient.integrate`, `.equilibrium` and
  `.jacobian` **`==`** identical (the rung-41 witness, extended one rung), and the rung 38–41 suites
  pass **unchanged**.
- **`phi_surge = 0` ⇒ the excursion object is unchanged; `transient_surge_margin` requires an armed
  map** (asserts, like the steady `surge_margin`) — the surge line is genuinely off when absent.
- **The split methods are inherently two-shaft.** `lp_disabled` is not a reduce axis for a
  *split between spools*; the methods assert the full two-shaft state (rung 40's `lp_disabled`
  dispatch is untouched and still reduces to rung 34 on its own gate).
- **Cycle untouched ⇒ rung 6 bit-for-bit.** The default design run is unchanged; rung 44 is
  read-only.

---

## Verification gates (`tests/test_rung44.py`)

1. **REDUCE — pure diagnostic.** A `phi_surge`-carrying map ⇒ rung 40 `integrate`/`equilibrium`/
   `jacobian` **bit-for-bit** (`==`), several shape pairs; default design run bit-for-bit rung 6.
2. **NON-TAUTOLOGICAL — an INDEPENDENT bare-math CPG two-shaft accel closure** (no `Gas` /
   `Component` / `ComponentMap` / `TwoSpoolTransient` calls; own closed-form CPG thermodynamics, own
   choke bisection, own forward speed lines, own 2-D running-line root, own Euler shaft march)
   reproduces the shipped excursion's **SIGN (negative, both spools), the LP-over-HP ordering, and
   the schedule-slaving (`rho`-invariance + ramp-rate monotonicity)** on shaped maps. Reproducing a
   flat-map value would only re-check the reduce; the shaped-map sign + ordering is what ties the
   object down (mirrors rung 40 gate 3).
3. **THE SPLIT SURVIVES DYNAMICALLY** — `ext_lp < 0` and `ext_hp < 0` on the accel (both toward
   surge) and `> 0` on the decel; `|ext_lp| > 1.4·|ext_hp|` at every shape pair, incl. the mode-free
   `hp-only`. Shape-robust; magnitudes disclaimed.
4. **SCHEDULE-SLAVED, NOT `rho`, NOT THE MODE** — (a) `ext_lp` varies <5 % over `rho` ∈ [0.2, 5.0];
   (b) `|ext_lp|` is monotone-increasing as `r_ramp` falls (faster ⇒ deeper); (c) `hp-only` has
   `oscillatory_band = None` yet the **largest** LP/HP ratio, and `|Im/Re|max < 0.25` on every pair.
5. **REPORT THE CROSSING, GATE THE FLIP** — `transient_surge_margin`'s min LP margin is strictly
   **below** the steady margin at the same instantaneous `Tt4` on the accel (and strictly above on
   the decel); with a floor placed in the gap the method **records** `crossed = True` while the
   steady point is clear — the flip's SIGN is asserted, the crossing DEPTH is not.
6. **CYCLE UNTOUCHED** — the default single-spool design path is bit-for-bit rung 6.

---

## Concessions

- **`phi_surge` imposed (doubled) — no survival claim.** Rung 41's disclosed constant, unchanged.
  Every margin magnitude, and every absolute crossing, is disclaimed; only the excursion's SIGN and
  the flip under a disclosed comparison are load-bearing. Rung 44 does **not** claim the engine
  surges or survives — it claims the transient moves the point the wrong way, on the LP spool,
  governed by the ramp rate.
- **`rho` is a DISCLAIMED clock group, doubled** — inherited from rung 40; only the ratio enters and
  no wall-clock time is claimed. The finding is precisely that `rho` is **powerless** over the
  surge excursion, so this concession costs the finding nothing.
- **Every magnitude rides on the representative maps and the imposed floor** — the excursion depths,
  the LP/HP ratio's value, the crossing depth. Rung-32 methodology: shapes disclosed, claims
  shape-robust, magnitudes disclaimed.
- **`Tt4` is the control** (rung 40's plant). Rung 35/43's **fuel** metering — where the TIT
  overshoot drives the excursion and fuel control *enlarges* it on the single spool — is the natural
  **extension**, deliberately deferred so the surge question is isolated from the fuel-control
  question. The excursion sign and schedule-slaving are expected to survive onto the fuel path
  (the plant is the same); that is not claimed here.
- **Fully-choked branch, both NGVs choked, no bypass/bleed, one `eta_m`, isentropic knobs** —
  inherited from rungs 38–41 unchanged. The subsonic/unchoked LP branch's transient remains open.
- **The complex mode's surge-irrelevance is claimed only on these maps** (`|Im/Re| ≤ 0.164`). A real
  front-stage-stalling map with a stronger inter-spool mode could ring harder; rung 44 claims the
  mode is irrelevant **at the disclosed magnitudes**, not universally (rung 40's disclaimer, carried).
- **Diagnostic beside the cycle.** The transient surge line reads the running point; it never feeds
  back.

---

## Anchor

`docs/plans/rung44-anchor-transient-surge.md`. The **method** is again Cohen–Rogers–Saravanamuttoo
*Gas Turbine Theory* Ch. 9: during an acceleration the working line swings **above** the steady
equilibrium running line toward the surge line, because the fuel step outruns the shaft's inertial
response; the margin consumed is the gap to the low-flow boundary. Rung 44 applies that transient
working-line statement to **two** characteristics and finds the swing **concentrated on the LP
spool** (rung 41's `(†)` shielding, now transient) and **governed by the ramp rate** rather than the
inter-spool clock ratio. Rung 44's own contributions are the **quantified transient split** (LP eats
1.6–2.2×), the **schedule-slaving triple** (`rho`-invariant, ramp-rate-driven, mode-independent —
the surge-axis form of rung 40's scope-limit), and the **report-crossing/gate-flip** transient
surge object that finally discharges rungs 40/41's shared deferred seam.
```
