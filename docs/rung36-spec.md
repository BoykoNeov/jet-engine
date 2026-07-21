# Rung 36 — The surge line: the excursion gets a boundary to be measured against

Rungs 32, 34 and 35 all reported the transient **excursion** as a distance **above the running
line** — the compressor-map displacement `E = max_t[π_c(t)/π_c,rl(ν(t)) − 1]` toward surge — and
every one of them filed the same concession: **no surge line, no surge-margin claim.** Rung 32 stated
the reason precisely: *"a representative efficiency island is not a surge boundary; the magnitude
rides on where you draw the line."* The excursion could say **how far toward surge**, never **how far
from surge**.

Rung 36 draws the line. The result is a **cross-rung CONFIRMATION + SHARPENING** (the rung-28 /
rung-29-margin move):

> **The excursion above the running line was only half of a surge statement.** Surge risk is the
> excursion measured against the **surge margin**, and the surge margin is **not constant along the
> running line** — it is **thin at part power, wide at design**, because the choked hardware drives
> the running-line flow coefficient `φ_op(Tt4)` *down toward the stall flow coefficient* as you
> throttle back. That **thin-at-low-power schedule** — sign-robust despite the one imposed constant —
> is the rung. And it **confirms and sharpens** rung 34's implicit worst case rather than moving it:
> rung 34's excursion `E0` is *already* largest for the **low-power burst**, and the surge line shows
> the steady margin `SM` it consumes is *thinnest* exactly there — the two acceleration limits
> **reinforce**. Rung 34's `E(r)` supplied the excursion; the surge line supplies the **margin it is
> measured against** — genuinely new information, not a rescale of `E`.

Like every rung 7→35 this is a **pure diagnostic beside the cycle**: a separate entry point
(surge methods on `SpoolTransient`), the surge line **never touches the running line or the
transient** (`E`, `ν(s)` are unchanged — it only *measures* against them), the default
`build_turbojet(…).run(…)` design path is untouched (bit-for-bit rung 6), and it **reduces to
rung 34/35 bit-for-bit when the surge line is off** (`phi_surge = 0`).

---

## The honest problem rung 32 named — and why it does NOT sink the rung

A surge line is empirical hardware data. Consistent with the no-data-file discipline and the
rungs 12–24 / rung 32 methodology (parametric closures with **disclosed** shapes, **load-bearing
claims shape-robust, magnitudes disclaimed**), the natural hope was a **zero-new-constant** anchor:
stall where the map's own **loading law peaks**, `dψ/dφ = 0`, i.e. `φ_peak = 1 − l/(2σ)`.

**That anchor is dead** — arithmetic, checked against the shipped surge-realistic shapes:

| shape            | σ    | l    | `φ_peak = 1 − l/(2σ)` |
|------------------|------|------|------------------------|
| `surge_flow`     | 0.10 | 0.70 | **−2.5**               |
| `surge_pressure` | 0.10 | 1.00 | **−4.0**               |
| `surge_tilted`   | 0.20 | 0.85 | **−1.125**             |

All at `φ < 0` — non-physical. For these shapes the linear term dominates, so `ψ` is
**monotone-rising across the whole physical `φ > 0` range**: the loading peak never occurs in range,
there is **no free in-range stall point to inherit**. A surge criterion must be **imposed** as a
disclosed constant — a **stall flow coefficient `φ_surge`** (`ComponentMap.with_phi_surge`). That
reintroduces exactly the free parameter rung 32 objected to.

**So the rung lives or dies on one question: does a shape-robust SIGN survive variation of that
imposed constant?** No margin *magnitude* can ever be load-bearing here — rung 32 was right about
that. Only a **trend / sign** can. It does survive, for a structural reason:

> `SM(Tt4)` has the form `[pressure ratio at `φ_surge`] / [pressure ratio at `φ_op(Tt4)`] − 1` on
> the same speed line. The imposed `φ_surge` sets the **level**. The **trend** is set by
> `φ_op(Tt4)` — the running-line flow coefficient — and **that is determined by the choked
> hardware** (rung 31/32), *not* by the imposed floor. The floor cannot flip a sign it does not
> control.

Measured (fast `thermally_perfect` gas): across **3 map shapes × imposed `φ_surge ∈ {0.55,…,0.75}`
× a speed-slope `k` on `φ_surge(n)` including negative `k`** — every cell gives **the same sign**,
`SM` thin at low power. And it holds under **both** margin definitions (constant-speed and the
CRS-default constant-flow). Direction anchored to **CRS Ch. 9**: the equilibrium running line of a
single-spool turbojet approaches the surge line at **low corrected speed** (why starting and
low-speed acceleration are the surge-critical maneuvers, and why bleed valves / variable stators
exist).

---

## The two margin definitions — and why constant-speed is the primary currency

At a running-line point (corrected speed `n0`, flow coefficient `φ_op`, pressure ratio `π_c,op`),
with the stall flow coefficient `φ_surge < φ_op`:

```
SM_N    (constant SPEED)  =  π_c(n0, φ_surge)/π_c,op − 1                          # same speed line n0
SM_flow (constant FLOW)   =  π_c(n_s, φ_surge)/π_c,op − 1 ,   n_s = φ_op·n0/φ_surge   # surge at m_op
```

`π_c(n, φ)` is the **same forward speed-line + efficiency-island arithmetic** the shipped
`_close_compressor` uses; at `φ = φ_op` it reproduces the shipped `π_c` **bit-for-bit** (two code
paths, one `π_c` — so the margin is measured on the very map that sets the running line).

**`SM_N` is the load-bearing currency**, because it is *exactly what a frozen-spool fuel step
consumes*. At `r → 0` (a fuel step faster than the spool can respond) the operating point jumps in
`π_c` **at constant `n0`** — the rung-34 constant-speed excursion `E0`. Both `E0` and `SM_N` are
`π_c` ratios at the frozen `n0` to the **same** denominator `π_c,op`, so

```
the step reaches SURGE   ⇔   E0 ≥ SM_N   ⇔   φ_step ≤ φ_surge          (airtight, gated)
```

(`π_c` is monotone in `φ` along a fixed speed line between `φ_op` and `φ_surge`, so the pressure and
flow-coefficient statements are the same statement.) `SM_flow` is the CRS textbook default and is
reported **only** as **weak corroboration** that the sign is not an artifact of the constant-speed
definition — **not** as an independent confirmation. Its magnitude is not trustworthy: at high power
the surge point sits on a much higher speed line (`n_s = φ_op·n0/φ_surge ≈ 1.54` at design), where the
Euler/efficiency-island **quadratics are extrapolated well past their calibrated range** (`SM_flow`
reaches ~968% — a `π_surge ≈ 100` that is plainly non-physical), and part of the "falling trend" is
that absurd high-power extrapolation coming back to earth. **Constant-speed carries the claim**
(`n0` fixed, `φ ∈ [φ_surge, φ_op]`, no extrapolation); constant-flow is a sign check only.

---

## The finding: the SM schedule is thin at low power — and BOTH acceleration axes agree the low-power burst is worst (confirmation + sharpening, NOT relocation)

**The headline is the schedule itself.** `SM_N(Tt4)` is thin at part power (widest at design), and
the sign of that schedule is **robust to the imposed `φ_surge`** because it rides on the choke-pinned
`φ_op(Tt4)`, not the floor. That is the new, structural, load-bearing result — a surge-margin object
the cycle did not previously carry — together with the **currency equivalence** below.

The **compounding with the excursion** is a **sharpening**, not a relocation. For a full-throttle
burst to a fixed `Tt4_hi` starting from a variable `Tt4_lo`:

- **`E0` rises as start power falls** — a bigger burst from a lower spool speed. *This was already
  available to rung 34* (its `constant_speed_excursion(Tt4_lo, Tt4_hi)` is largest for a low-power
  start), so rung 34 **already** implies the low-power burst has the biggest excursion.
- **`SM_N` falls as start power falls** — the running line has walked `φ_op` down toward `φ_surge`.
  This is the surge line's **unique** contribution: the schedule the excursion is divided by.
- **Both point the same way**, so the consumed-margin ratio `E0/SM_N` **rises monotonically toward
  the low-power end** — the two acceleration limits (excursion magnitude and thinness of margin)
  **reinforce**, and the low-power burst is the most surge-critical on **both** axes.

So the surge line **confirms** rung 34's implicit worst case and **sharpens** it (the margin is
thinnest exactly where the excursion is largest). It does **not** *relocate* the binding constraint:
the binding burst is the low-power one **with or without** the surge line. **Relocation would require
the two schedules to point *opposite* ways** — `E0` largest at one end but `SM_N` thinnest at the
other, so the margin *overrides* the excursion ordering and `argmax(E0/SM_N) ≠ argmax(E0)`. On these
representative maps they are parallel; a real map with a steeply speed-dependent surge line *could*
oppose them, and that is the case a reader should look for. What is nonetheless genuinely new (and
**not** a rescale of `E`) is `SM_N` itself — information that varies independently of the ramp ratio
`r` and turns "excursion above the running line" into "fraction of the stability margin consumed."

**What is NOT claimed — the disclaimed magnitude (the rung-32 discipline, enforced).** *Whether* a
given burst actually reaches surge — the **crossing** `E0 ≥ SM_N` — rides entirely on the disclaimed
`φ_surge`. `E0` is independent of the floor (a pure map displacement); only `SM_N` moves with it, so
the crossing location slides with `φ_surge` (e.g. `surge_flow` at `Tt4_lo = 700`: `E0 = 0.098` fixed,
`SM_N = 0.109` at `φ_surge = 0.55` → **no surge**, but `0.073` at `0.65` → **surge**). The crossing is
therefore **deliberately not gated**. Only the **monotone rise of `E0/SM_N`** (the relocation) and
the **sign of the `SM` schedule** (thin at low power) are load-bearing.

---

## The mitigation this closes (why fuel ramps are scheduled AT LOW POWER)

Rung 34 explained *why fuel ramps are scheduled* — slow the fuel (raise `r = τ_fuel/τ_spool`) and
`E(r)` falls toward zero, keeping the excursion clear of surge. Rung 36 says *where the schedule must
be tightest*: at the low-power end, where the steady margin `SM_N` is already thin and the step
excursion `E0` is already large. This is the acceleration-schedule "bottom" of a real engine — the
region where a bleed valve or variable stator is opened to **push the surge line away** (raise
`φ_surge`) precisely because the running line has crept up to it. Rung 36 does not model those
devices; it exhibits the margin they exist to protect.

---

## Reduce-to-prior contract (the spine)

- **Surge line off ⇒ rung 34/35 bit-for-bit.** `phi_surge` defaults to `0.0` and is read **only**
  by the rung-36 surge methods; nothing on the running line, the transient, or `_close_compressor`
  touches it. `is_flat()` deliberately ignores `phi_surge` (a flat map *with* a surge floor still
  reduces `MapMatcher` to rung 31). The rung 31–35 suites pass **unchanged** — the bit-for-bit
  witness.
- **`π_c` reproduction (non-tautological).** `_pi_c_map(n, φ_op)` reproduces the shipped running-line
  `π_c` to machine zero — the surge margin is computed on the same forward map that sets the running
  line, not a parallel re-derivation.
- **Cycle untouched ⇒ rung 6 bit-for-bit.** The default design run is not touched; the surge line is
  a read-only diagnostic. The rungs-7+ invariant holds.

---

## Verification gates (`tests/test_rung36.py`)

1. **REDUCE — surge line off ⇒ rung 34/35.** With `phi_surge = 0` the `SpoolTransient` equilibrium /
   transient / excursion outputs equal rung 34/35's to machine zero; the rung 31–35 suites are the
   bit-for-bit witness (surge methods add no cycle knob).
2. **`π_c` REPRODUCTION (non-tautological).** `_pi_c_map` at the operating `(n, φ_op)` equals the
   shipped `equilibrium` `π_c` to machine zero, on ≥2 shapes and several `Tt4` — the margin is
   measured on the running-line map itself.
3. **THE SCHEDULE — `SM` thin at LOW power, SIGN-ROBUST.** `SM_N(Tt4)` decreases monotonically as
   `Tt4` falls, **same sign across ≥3 map shapes × ≥3 imposed `φ_surge`** (and, sub-gate, under the
   constant-flow definition too). Magnitude disclaimed.
4. **THE COMPOUNDING — confirmation + sharpening (NOT relocation).** For bursts to a fixed `Tt4_hi`,
   `E0/SM_N` rises monotonically as the start `Tt4_lo` falls (both `E0 ↑` and `SM_N ↓`, reinforcing),
   across ≥3 shapes — the low-power burst is most surge-critical on **both** axes. The gate checks the
   monotone rise; it does **not** assert relocation (the schedules are parallel, so `argmax(E0/SM_N)`
   = `argmax(E0)` — rung 34's implicit worst case is confirmed and sharpened, not moved).
5. **CURRENCY EQUIVALENCE (airtight).** `reaches_surge == phi_step_le_surge` at every tested point —
   `E0 ≥ SM_N ⇔ φ_step ≤ φ_surge` — certifying `SM_N` is the exact currency the constant-speed
   excursion consumes.
6. **CROSSING IS DISCLAIMED (the anti-overclaim gate).** With `E0` held fixed, varying `φ_surge`
   flips `reaches_surge` at a fixed `Tt4_lo` — the test **asserts the flip exists** (the crossing is
   floor-dependent), certifying the rung claims the *trend*, never the crossing. This is the gate
   that keeps rung 36 honest about what rung 32 warned of.
7. **CYCLE UNTOUCHED.** The default design run is bit-for-bit rung 6; constructing/using the surge
   methods does not perturb it.

---

## Concessions

- **One imposed constant `φ_surge`.** The loading-law peak is out of range (see the table), so the
  surge line cannot be inherited; `φ_surge` is a disclosed knob. Every load-bearing claim is verified
  robust to it; every margin **magnitude** — and the **crossing** into surge — is disclaimed.
- **Constant `φ_surge` (in `n`).** Modeled as speed-independent (the standard first-cut stall
  criterion). The sign is verified robust to a mild `n`-dependence `φ_surge(n) = φ0 + k(n−1)`
  (including `k < 0`); a hardware surge line's exact `(ṁ, π)` shape is not claimed.
- **Constant-speed margin is primary.** It is the transient's currency; the CRS-default constant-flow
  margin is reported only for sign-robustness. Neither magnitude is claimed.
- **No bleed valve / variable stator.** Rung 36 exhibits the margin these devices protect; it does
  not model them raising `φ_surge` at low speed (a further seam).
- **Choked branch only.** The surge margin is a choked-branch diagnostic (rung 31/32 hardware); below
  nozzle unchoke (rung 33 subsonic branch) it is out of scope, and `surge_margin` asserts rather than
  lies. Isentropic knobs, NGV-choke, single spool — inherited from rungs 31–34.
- **Diagnostic beside the cycle.** The production run is untouched; the surge line reads the
  running-line / transient state, it does not feed back into it.

---

## Anchor

`docs/plans/rung36-anchor-surge-line.md`. The **method** is the Cohen–Rogers–Saravanamuttoo *Gas
Turbine Theory* Ch. 9 surge-margin-along-the-running-line construction (superimpose the equilibrium
running line on the compressor characteristic; the surge line is the low-flow stability boundary; the
margin is the pressure-ratio gap between them) — the same textbook family as the rung-2/31/34 anchor.
Rung 36's specific result is that for a **choked-turbine / choked-nozzle single spool** the
running-line flow coefficient `φ_op(Tt4)` — pinned by the choked hardware (rung 31/32) — walks down
toward a fixed stall `φ_surge` as the engine throttles back, so the margin schedule's **sign** is
hardware-determined even though its magnitude rides on the imposed floor; and that the rung-34
constant-speed excursion `E0` and this margin `SM_N` share a currency, so the low-power burst — which
rung 34 already knew has the largest excursion — is confirmed to also face the thinnest margin (a
reinforcing sharpening, not a relocation). The reduce (surge off ⇒ rung 34/35, and `_pi_c_map` == the
shipped `π_c`) is the rigorous, non-tautological anchor.
