# Rung 90 anchor — the CUSTOMER bleed: air the aircraft takes at station 3 (pre-registered 2026-10-15 — § 3)

**Seam:** rung 89 § *The next seam* (and rungs 42, 61, 63, 86, 87, 88 before it): *"the station-3
customer bleed — air taken off the core for the aircraft, not to protect a compressor — a bleed the
valves here would have to work around"*. Rung 42's concession: *"HPC-exit bleed … is a different
sink with a different signature."*

**The user's choices (2026-10-15):** the whole seam in one rung — the steady two-shaft engine, the
throttle-slam engine, the protection valves beside it, and a sandbox knob; the demand set **either**
as a fixed share of the compressor's air **or** as a fixed amount of air, as a switch.

Shipped as KNOBS (the sandbox direction): every prediction below is scored HIT/MISS, and a miss is
a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\jet-rung90-probe`, not in the repo),
on EXISTING code only — no customer-bleed code existed when it ran. § 1–§ 3 written BEFORE any
customer-bleed number was computed. What § 0 saw is a GIVEN and scores no credit.

---

## 0. GIVENS — measured on the `c = 0` plant before anything was written

Rig A (rung 42's): CPG gas (γ 1.4 / 1.3), `π_LPC` 3 × `π_HPC` 6, `Tt4` 1500 K design, rung 42's
losses, 250 K / 50 kPa / Mach 0.85, flat maps and the `flow/press` shapes, `Tt4` 1500 → 650 K.

**G1 — the c = 0 running line** (flat; `flow/press` within 0.03 in `π`):

| `Tt4` | 1500 | 1300 | 1100 | 1000 | 900 | 800 | 700 | 650 |
|---|---|---|---|---|---|---|---|---|
| `π_LPC` | 3.000 | 2.629 | 2.295 | 2.141 | 1.995 | 1.857 | 1.727 | 1.664 |
| `π_HPC` | 6.000 | 5.188 | 4.406 | 4.028 | 3.660 | 3.304 | 2.960 | 2.793 |

**G2 — the § 1 linear model reproduces rung 42's interstage response, on the flat maps.** Measured
`d ln φ / db` (one-sided, `db` 0.02, existing rung-42 code) against § 1's formula:

| | LP model | LP measured | HP model | HP measured |
|---|---|---|---|---|
| flat, 1500 K | +0.965 | +0.980 | +0.117 | +0.117 |
| flat, 1000 K | +1.060 | +1.075 | +0.032 | +0.032 |
| flat, 650 K | +1.158 | +1.173 | −0.016 | −0.017 |
| `flow/press`, 1500 K | +0.965 | +0.722 | +0.117 | +0.078 |
| `flow/press`, 650 K | +1.171 | +1.053 | −0.018 | −0.010 |

On the flat maps the LP residual is a near-constant +0.015 (the fuel-fraction terms § 1 drops) and
the HP agrees to ≤ 5e−4, including the sign change at `π*` (rung 42's finding, recovered by the
model — `a_H = 1` IS rung 41's `s_H = 0`). On the shaped maps the model is 0.75–0.90 of the
measured LP: it ignores the map's loading and efficiency slopes. **Hence: quantitative bars on the
flat maps only; shaped maps get SIGN bars.**

**G3 — rung 64's valve rig** (`docs/rung64-spec.md` § 1): shaped/tilted, `r` 0.5, valve shut
`min φ_LP` 0.7354 / 0.7374, `b_max` 0.10, `Tt4` 1000 → 1400 K. Inherited, not re-measured.

## 1. THE DEVICE — where the customer bleed enters (derived before coding)

Per unit inlet air `ṁ₂`, with the interstage share `b` (rung 42) and the customer share `c` of the
HPC delivery:

```
LPC pumps ṁ₂ · HPC pumps (1−b)ṁ₂ · burner + turbines pass (1−b)(1−c)ṁ₂ (·(1+f) after the burner)
```

`c` enters SIX places — two of them break rung 42's structural claims:

1. **The HP shaft balance — NEW.** `h(Tt3) − h(Tt25) = η_m (1−c)(1+f) Δh_HPT`. Rung 42's `(1−b)`
   cancelled here (both sides core flow); `(1−c)` does NOT — the HP turbine drives its compressor
   with less gas than the compressor pumps. **The HP balance is no longer bleed-invariant.**
2. **The HP face referral (†) — NEW.** The HPT nozzle chokes on burner flow, the HP face passes
   `1/(1−c)` of it: `m_H = A4 π_b π_HPC MFP √(Tt25/Tt4) / [(1+f)(1−c)] / m_H,d`. Rung 42 called
   rung 39's HP loop VERBATIM because its body was bleed-free; **that no longer holds — the HP loop
   is copied with (†-c).** The triangle survives: the HP still closes on itself, given
   `(Tt25, Tt3, c)`.
3. **The LP shaft balance** — `h(Tt25) − h(Tt2) = η_m (1−b)(1−c)(1+f) Δh_LPT` (rung 42's shape).
4. **The LP face referral (‡)** — picks up `1/[(1−b)(1−c)]`.
5. **The forward rebuild** — the extraction booked at station 3, before the burner, so every
   conservation assert downstream sees the burner flow.
6. **Thrust** — the customer air was captured: full ram drag, no exhaust momentum. Per inlet air
   `F/ṁ₂ = (1−b)(1−c)·[(1+f)V9 + pressure − V0] − (b + (1−b)c)·V0` (written so `c = 0` adds
   exact zeros).

Untouched: both turbine pins (same flow through both choked nozzles, extraction upstream of
station 4). And on the TRANSIENT plant the same six enter rung 62's four cells: the closure, the
fuel closure (`f = ṁ_fuel / burner air`, and the rich/lean bracket walls carry the factor —
rung 62's I5), the power loop (the HP term gains `(1−c)` where rung 62's cancelled) and the
thrust tail.

**The two demand modes.** *Share*: `c` fixed. *Amount*: a fixed `W = w · ṁ_design` kg/s, so
`c = W / ṁ_HPC` floats with the state (larger at low power). Steady: `c` is iterated inside the
joint `(f, pt4)` loop from the burner flow the nozzle imposes. Transient: explicit per closure
trial (`ṁ_burner = ṁ_HPC − W`). Refused, in plain words: a share outside `[0, 0.5)` (rung 42's
bound), an amount that leaves less than half the HPC air for the burner at a closure.

**Where it lives.** On the design engine (`TwoSpoolEngine`), copied onto rung 38's core: the
aircraft's load, not a lever, so every sibling a reader builds from the design engine keeps it
(rung 64's sibling trap cannot drop it). The design run IGNORES it: the hardware is captured with
the customer bleed OFF, as rung 42's valve is captured shut (concession). Every lower solver
(rungs 39, 42 steady; 40, 43, 57 transient cells) **refuses** an active customer bleed rather than
silently ignoring it (rung 89's precedent: an explicit object is never silently dropped).

**Reduce (the spine).** An inactive customer bleed (`None`, share 0, amount 0) dispatches to the
earlier code VERBATIM: steady → rung 42's `try_match_bleed` (→ rung 39 at `b = 0`); transient →
rung 62's existing bodies (→ rung 57 at `b = 0`). Bit for bit, gated; no oracle and no CLI golden
may move; the default design run stays rung 6.

### The linear model (the prediction instrument; § 0 G2 validated it on `b`)

Flat maps, CPG, fixed `Tt4`, both turbines pinned. With `τ` each compressor's temperature ratio,
`a = k η(τ−1)/(1+η(τ−1))` its pressure-ratio sensitivity `d ln π / d ln(τ−1)`, `k = γ/(γ−1)`,
`t_L = (τ_L − 1)/τ_L`:

```
LP energy: d ln(τ_L−1) = −dc          Tt25: d ln Tt25 = −t_L dc
HP energy: d ln(τ_H−1) = −(1 − t_L) dc     π_HPC: d ln π_HPC = −a_H (1 − t_L) dc
(‡)      : d ln m_L = d ln π_LPC + d ln π_HPC + dc ;   n_L ∝ √(τ_L−1)
(†)      : d ln m_H = d ln π_HPC + ½ d ln Tt25 + dc ;  n_H ∝ √(τ_H−1)

    d ln φ_L / dc = 1.5 − a_L − a_H (1 − t_L)
    d ln φ_H / dc = 0.5 + (1 − t_L)(1 − a_H)
```

(The same algebra for `b` gives `1.5 − a_L + a_H t_L` and `t_L (a_H − 1)` — § 0 G2's rows.)

**What it says, before any measurement.** The HP is helped everywhere (`a_H` would need ~1.7,
`π_HPC` ≈ 10, to reverse it — outside the band): **no `π*` for the customer bleed.** The LP is
pushed TOWARD surge at high power — the HP's pressure-ratio drop starves the LP face more than the
`1/(1−c)` feeds it — and helped at low power, the sign changing where `a_L + a_H(1 − t_L) = 1.5`:
on G1's line, at **`Tt4` ≈ 900 K**. The opposite arrow from rung 42's bleed on the LP, at the top
of the band.

Model values on G1's line (flat):

| `Tt4` | 1500 | 1400 | 1300 | 1200 | 1100 | 1000 | 900 | 850 | 800 | 750 | 700 | 650 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| LP | −0.438 | −0.379 | −0.315 | −0.246 | −0.171 | −0.089 | +0.001 | +0.049 | +0.099 | +0.153 | +0.209 | +0.268 |
| HP | +0.215 | +0.240 | +0.269 | +0.301 | +0.339 | +0.382 | +0.432 | +0.460 | +0.490 | +0.522 | +0.556 | +0.594 |

## 2. THE READINGS

**Steady (rig A).** `d ln φ / dc` one-sided at `c` 0.02 on the c = 0 line; the bled point against
the c = 0 line at the same `x = Tt4/Tt_face` (rung 42's off-line test, per spool); thrust per inlet
air and TSFC at `c` 0.05 against rung 42's `b` 0.05 at the same `Tt4`; the amount mode at
`w` 0.05.

**Transient (rig B = G3's, rung 64's).** CPG, shaped/tilted × `r` ∈ {0.25, 0.5, 1}, `ds` 0.005,
`Tt4` 1000 → 1400 K, settle 1.2, rung 64's `φ` valve (`b_max` 0.10, no stators, no fuel leg),
set points `sm` ∈ {0.30, 0.34, 0.38, 0.42}; customer share `c` ∈ {0, 0.05} and amount `w` 0.05.
Every valve position re-solved through `b_at_point` (rung 64 § 5.22 (ii)). Readings: valve-shut
`min φ_LP` and `min φ_HP` over the march; the valve's bleed integral `∫b ds` over the stretch both
runs reach; the ceiling (`min φ_LP` with the valve fully open).

## 3. PREDICTIONS (frozen here; scored in the spec)

**Construction (gated, no credit):** the reduce (inactive ⇒ prior code bit for bit, steady and
transient, both modes); transient equilibrium ≡ steady match at the same `(Tt4, b, c)` to 1e−9
(rung 62's plant gate — the only check that catches a missing factor in the power loop); the
mass identities `ṁ_burner = (1−b)(1−c)ṁ₂`; `x_L = Tt4/Tt2` exactly c-invariant; lower solvers
refuse an active customer bleed; siblings keep it.

**Steady — flat maps (quantitative):**
- **A1 (HP, magnitude).** Measured `d ln φ_H/dc` within 0.03 of § 1's table at every `Tt4` of G1.
- **A2 (HP, no `π*`).** `d ln φ_H/dc > 0` at every point (rung 42's reverses at `π*`; this does not).
- **A3 (LP, magnitude).** Measured `d ln φ_L/dc` within 0.05 of § 1's table at every `Tt4`.
- **A4 (LP, the crossing).** The LP response changes sign ONCE in the band, negative above,
  positive below, the crossing within ±50 K of 900 K.
- **A5 (the HP gets a new freedom).** At the same `x_H = Tt4/Tt25`, the `c` = 0.05 HP point is off
  the c = 0 line by more than 1 % in `φ_H` at every `Tt4` (rung 42's interstage bleed: ≤ 0.016 %).

**Steady — shaped (`flow/press`, sign only):**
- **A6.** `d ln φ_H/dc > 0` everywhere; `d ln φ_L/dc < 0` at 1500 K and `> 0` at 650 K.

**Steady — the trade (flat, `c` 0.05 vs `b` 0.05, same `Tt4`):**
- **A7.** The customer bleed costs MORE thrust per inlet air and MORE TSFC than the interstage bleed
  at every `Tt4` (the customer air carries the HPC's work as well).
- **A8 (amount mode).** At `w` 0.05 the share `c` rises monotonically as the throttle falls, and at
  650 K the LP flow-coefficient shift is positive and larger than the share mode's at `c` 0.05.

**Transient — the valves (rig B; the least bounded claims here, from the steady model at the
ramp's low end, `Tt4` 1000–1100 K, where § 1 gives LP −0.09 to −0.17 and HP +0.34 to +0.38):**
- **B1.** Valve shut, `c` 0.05 LOWERS `min φ_LP` in all 6 cells.
- **B2.** Valve shut, `c` 0.05 RAISES `min φ_HP` in all 6 cells, by more than it moves `min φ_LP`.
- **B3.** The `φ` valve's bleed integral RISES with `c` 0.05 at every set point where it rides in
  both runs (the valve works around the customer bleed).
- **B4.** The ceiling (fully-open `min φ_LP`) FALLS with `c` 0.05 in all 6 cells.
- **B5.** The amount mode (`w` 0.05) costs a larger bleed integral than the share mode (`c` 0.05)
  at every riding set point — at the ramp's low-power start it is a larger share.

**The likeliest miss, named in advance:** B1/B3/B5 rest on the LP sign at the ramp's low end, and
§ 1 puts the steady crossing only ~100 K below the ramp's start. If the march's LP minimum sits at
a state the steady line reads as below ~900 K, all three flip together — the customer bleed would
then LOWER the valve's bill.
