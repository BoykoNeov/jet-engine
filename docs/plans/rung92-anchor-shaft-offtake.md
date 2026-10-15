# Rung 92 anchor — SHAFT-POWER OFFTAKE: the aircraft's generator load on a shaft (pre-registered 2026-10-15 — § 3)

**Seam:** rung 91 § *The next seam*: *"Shaft-power offtake — the aircraft's generator load on the HP
shaft: the other half of what an aircraft takes (rung 90 took its air)."* Rung 90's concession: *"No
shaft-power offtake (the other half of an aircraft's demand)."*

**The user's choices (2026-10-15):** the load can come off EITHER shaft — the HP shaft (today's
engines) or the LP shaft (the more-electric-aircraft proposal) — as a switch; the demand set either
as a SHARE of that shaft's turbine work or as a fixed POWER, as a switch; the whole seam in one rung
as rungs 90/91 — the steady two-shaft engine, the throttle-slam engine, the protection valve beside
it, and a sandbox knob.

Shipped as KNOBS (the sandbox direction): every prediction below is scored HIT/MISS, and a miss is
a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\jet-rung92-probe`, not in the repo;
its source and output archived in `docs/plans/scratch-archive/rung92/`), on EXISTING code only — no
offtake code existed when it ran. § 1–§ 3 written BEFORE any offtake number was computed by offtake
code. What § 0 saw is a GIVEN and scores no credit.

---

## 0. GIVENS — measured on the unloaded plant before anything was written

Rig A (rungs 42/90/91's): CPG gas (γ 1.4 / 1.3), `π_LPC` 3 × `π_HPC` 6, `Tt4` 1500 K design, rung
42's losses, 250 K / 50 kPa / Mach 0.85, flat maps and the `flow/press` shapes, `Tt4` 1500 → 650 K,
`ṁ_design` 1 kg/s. The running line is rung 90's G1.

**G1 — the two shafts' powers.** The design HP compressor power `P_ref` = 307.6 kW, the LP's
117.7 kW (ratio **2.61**). Along the line (flat), each compressor's power over `P_ref`:

| `Tt4` | 1500 | 1400 | 1300 | 1200 | 1100 | 1000 | 900 | 850 | 800 | 750 | 700 | 650 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| HP | 1.000 | 0.843 | 0.706 | 0.585 | 0.481 | 0.391 | 0.314 | 0.280 | 0.249 | 0.220 | 0.194 | 0.170 |
| LP | 0.383 | 0.323 | 0.270 | 0.224 | 0.184 | 0.150 | 0.120 | 0.107 | 0.095 | 0.084 | 0.074 | 0.065 |

A fixed power `P` is therefore a share `P/P_C` of a shaft's work that grows ~6× (HP) from 1500 K
to 650 K — and on the LP shaft is 2.6× the HP share of the same power at every point.

**G2 — the measured bleed responses** (`d ln φ / d·`, Richardson `2D(0.01) − D(0.02)`, existing
rung-42 and rung-90 code), flat:

| `Tt4` | 1500 | 1300 | 1100 | 1000 | 900 | 800 | 750 | 700 | 650 |
|---|---|---|---|---|---|---|---|---|---|
| rung 42 `b`: LP | +0.966 | +0.999 | +1.038 | +1.060 | +1.085 | +1.112 | +1.126 | +1.141 | +1.158 |
| rung 42 `b`: HP | +0.116 | +0.081 | +0.047 | +0.031 | +0.016 | +0.002 | −0.005 | −0.011 | −0.017 |
| rung 90 `c`: LP | −0.433 | −0.312 | −0.170 | −0.089 | +0.000 | +0.099 | +0.152 | +0.208 | +0.267 |
| rung 90 `c`: HP | +0.212 | +0.266 | +0.337 | +0.380 | +0.430 | +0.487 | +0.519 | +0.554 | +0.591 |

## 1. THE DEVICE — where the offtake enters (derived before coding)

A generator (and pumps) draw a power `P_x` off a shaft, after the bearings (`η_m` already taken).
Per unit of that shaft's compressor air, `q = P_x / ṁ_compressor`; the shaft balances become

```
HP:  h(Tt3) − h(Tt25)  = η_m (gas per HPC air) Δh_HPT  − q_H
LP:  h(Tt25) − h(Tt2)  = η_m (gas per face air) Δh_LPT − q_L
```

**Two demands, per shaft.** *Share*: `P_x = s · (that shaft's turbine shaft power)`, so the
compressor gets `(1−s)` of it. *Power*: `P_x = w · P_ref` with `P_ref` the design HP compressor
power — ONE unit for both shafts (so an HP and an LP load of the same `w` are the same kilowatts),
and `s = P_x / P_turbine` floats with the state. Both shafts may carry a load at once (the device is
general; the sandbox shows a switch).

Where it enters — and where it does NOT:

1. **The two shaft balances** — the only steady change. The offtake moves no gas: **both turbine
   pins are untouched** (same flows through A4, A45, A8), so on the CPG gas `π_HPT`, `τ_HPT`,
   `π_LPT`, `τ_LPT` stay EXACTLY where they were, as do the face referrals (†)/(‡), the burner's
   air and the thrust bookkeeping. The state moves only through the compressors' work.
2. **The transient closure is untouched** (it reads compressor maps and the A4 choke, never a
   shaft balance), and so is the fuel closure. Only rung 62's **power cell** and **tail cell**
   change: `Φ̇ = (P_turbine − P_compressor − P_x)/(P_ref,shaft · ν)`.

**Two exact identities (flat maps, CPG) — the structure this rung exposes.** With constant map
efficiencies and loading (`ψ`), a compressor's `π`, `τ` and speed follow from its work alone, and
a share of a turbine's work enters a balance exactly as a turbine-gas deficit of the same size does:

- **An LP-shaft share `s` IS rung 42's interstage bleed `b = s`, minus the dumped air.** Rung 42's
  LP balance carries `(1−b)` on the LP turbine's gas — the same factor as `(1−s)` on its work; its
  HP balance cancels `(1−b)`; the offtake never touches the HP. Every `π`, `τ`, `N` is identical;
  only the LP face's corrected flow differs, by the bleed's `1/(1−b)`:
  `φ_L(offtake s) = (1−s) φ_L(bleed b=s)`, `φ_H` identical.
- **An HP-shaft share `s` plus an LP-shaft share `s` IS rung 90's customer bleed `c = s`, minus the
  dumped air.** Rung 90's `(1−c)` on the HP turbine's gas and `(1−b)(1−c)` on the LP's are those
  two shares; both faces differ by `1/(1−c)`.

So **a bleed is a power offtake plus air thrown away.** Gated as construction (§ 3), exact at
finite share; at first order they give the offtake's response from G2's MEASURED rows
(`LP-shaft = (b_L − 1, b_H)`, `HP-shaft = (c_L − b_L, c_H − 1 − b_H)`), which agree with the linear
model below to ≤ 0.0032 at every `Tt4` (the probe's table).

**Where it lives.** On the design engine (`TwoSpoolEngine`), beside the customer bleed and the
cooling, copied onto rung 38's core so every sibling keeps it; the design run IGNORES it (hardware
sized unloaded — the knob is a load MODULATED on fixed hardware, as rungs 90/91). Every body that
cannot model it refuses an ACTIVE one in words: rungs 38, 39, 42, 90, 91 steady (the new body is a
copy of rung 91's, which carries 90's and 42's), rungs 40, 43 (hence 57) transient. Rungs 62/90/91's
transient cells are WRAPPED, not refused (the closure is untouched). **The perfect gas only** — the
steady body is a copy of rung 91's cooled cascade, validated there only on the perfect gas.

**Bounds**, in words, checked at the ROOT (rung 90's lesson 2): each share in `[0, 0.5)`; a power
load that takes half its turbine's work or more at the converged point is refused ("the generator
takes more than half the HP turbine's work here").

**Reduce (the spine).** No load (`None`, share 0, power 0) dispatches to the earlier code VERBATIM:
steady → rung 91's `try_match_cooled` (→ 90 → 42 → 39); transient → rung 62's cells as they stand
(→ 91 / 90 / 62 / 57). Bit for bit, gated, with a counter per path; no oracle and no CLI golden may
move; the default design run stays rung 6.

### The linear model (the prediction instrument)

Flat maps, CPG, fixed `Tt4`, `b = c = ε = 0`, both throats choked, `f`'s response dropped. Rung 91's
compressor algebra (`h`, `ℓ` the fractional changes of the HP and LP compressor work; no referral
term — no air leaves): `d ln φ_L = a_L ℓ + a_H(h − t_L ℓ) − ½ℓ`, `d ln φ_H = (a_H − ½)(h − t_L ℓ) +
½ t_L ℓ`. An HP share is `h = −1, ℓ = 0`; an LP share `h = 0, ℓ = −1`:

```
HP shaft:  d ln φ_L/ds = −a_H                     d ln φ_H/ds = ½ − a_H
LP shaft:  d ln φ_L/ds = ½ − a_L + a_H t_L        d ln φ_H/ds = t_L (a_H − 1)
per unit POWER w:  multiply by ds/dw = P_ref / P_C,shaft   (G1)
```

(Rung 90's customer row is HP + LP + 1 on each face; rung 42's bleed row is LP + 1 on the LP face.)

**What it says, before any offtake code.** The HP shaft load pushes **both compressors toward surge
at every power** — no crossing (`a_H` > ½ throughout): the HP compressor gets less work, its
pressure ratio falls (`a_H`), and that starves the LP face too. **This is what the customer bleed's
relief hid**: rung 90's HP spool was helped only because its air left (+1); the power it took
pushed it the other way. The LP shaft load is nearly FREE per unit share: on the LP spool its own
loss of loading (`½ − a_L`) is offset by the HP's pressure rise (`a_H t_L`), crossing near
**1290 K**; on the HP it is rung 42's exact HP response, crossing at rung 42's `π*` near **783 K**.
Per unit POWER the LP shaft's share is 2.6–2.9× larger, and still the HP shaft costs more on both
spools at every `Tt4`.

Model values on the line (flat):

| `Tt4` | 1500 | 1400 | 1300 | 1200 | 1100 | 1000 | 900 | 850 | 800 | 750 | 700 | 650 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| HP shaft, LP, per `s` | −1.402 | −1.360 | −1.313 | −1.263 | −1.209 | −1.149 | −1.084 | −1.049 | −1.012 | −0.974 | −0.933 | −0.890 |
| HP shaft, HP, per `s` | −0.902 | −0.860 | −0.813 | −0.763 | −0.709 | −0.649 | −0.584 | −0.549 | −0.512 | −0.474 | −0.433 | −0.390 |
| LP shaft, LP, per `s` | −0.035 | −0.019 | −0.002 | +0.017 | +0.038 | +0.060 | +0.085 | +0.098 | +0.112 | +0.126 | +0.142 | +0.158 |
| LP shaft, HP, per `s` | +0.117 | +0.099 | +0.082 | +0.065 | +0.048 | +0.032 | +0.016 | +0.009 | +0.002 | −0.004 | −0.011 | −0.016 |
| HP shaft, LP, per `w` | −1.402 | −1.613 | −1.862 | −2.158 | −2.513 | −2.938 | −3.450 | −3.744 | −4.067 | −4.422 | −4.809 | −5.235 |
| HP shaft, HP, per `w` | −0.902 | −1.020 | −1.153 | −1.304 | −1.473 | −1.660 | −1.859 | −1.960 | −2.059 | −2.151 | −2.232 | −2.294 |
| LP shaft, LP, per `w` | −0.093 | −0.060 | −0.006 | +0.077 | +0.205 | +0.402 | +0.705 | +0.913 | +1.173 | +1.499 | +1.910 | +2.431 |
| LP shaft, HP, per `w` | +0.306 | +0.308 | +0.303 | +0.289 | +0.260 | +0.212 | +0.137 | +0.085 | +0.023 | −0.052 | −0.143 | −0.251 |

## 2. THE READINGS

**Steady (rig A).** `d ln φ / ds` per shaft (share mode) and `d ln φ / dw` per shaft (power mode),
Richardson-extrapolated to zero load from one-sided differences at 0.01 and 0.02 (rung 91's
reading); thrust per inlet air and TSFC at `s` 0.05 and `w` 0.05 (reported); the power mode's
floating share along the line (reported).

**Transient (rig B = rungs 90/91's, i.e. rung 64's).** CPG, shaped/tilted × `r` ∈ {0.25, 0.5, 1},
`ds` 0.005, `Tt4` 1000 → 1400 K, settle 1.2, rung 64's `φ` valve (`b_max` 0.10, no stators, no fuel
leg), set points `sm` ∈ {0.30, 0.34, 0.38, 0.42}; three demands: none, an HP-shaft power `w` 0.02,
an LP-shaft power `w` 0.02 (the same kilowatts; at the ramp's 1000 K start an HP share ≈ 0.05 and
an LP share ≈ 0.13). Readings as rungs 90/91's (valve-shut `min φ_LP`, `min φ_HP`; the valve's
bleed integral over equal-length runs; the ceiling), plus **the spool's acceleration**: on the
valve-shut march, the march time at which `ν_H` first covers 90 % of its own rise (start
equilibrium → the march's last point).

## 3. PREDICTIONS (frozen here; scored in the spec)

**Construction (gated, no credit):** the reduce (inactive ⇒ prior code bit for bit, steady and
transient, every dispatch path counted); **both identities of § 1 at finite share** on the flat
maps (`π`, `τ`, `N` and the HP turbine point to 1e−9; `φ` with the `(1−s)` factors); the turbine
pins and `τ_LPT`, `π_LPT`, `τ_HPT`, `π_HPT` unchanged by a load (no cooling) to round-off; transient
equilibrium ≡ steady match to 1e−9 (rung 62's plant gate) for HP and LP shaft × share and power ×
every lower path (none, interstage valve open, customer bleed, turbine cooling); both shaft balances
close on the rebuilt stations with the load booked; the power mode delivers its power; lower solvers
refuse an active load; siblings keep it; non-CPG gases refused.

**The bars are derived.** The model's residual on the flat maps is read from § 0 (the first-order
identity rebuilt from MEASURED rows): ≤ 0.0032 on every entry. The magnitude bar is **0.01** (3×);
a crossing's bar is 0.01 over the model's local slope in `Tt4`.

**Steady — flat maps:**
- **A1 (HP shaft, magnitude).** `d ln φ_L/ds` and `d ln φ_H/ds` within 0.01 of § 1's table at every
  `Tt4`.
- **A2 (HP shaft, no relief).** Both negative at every `Tt4` — no crossing.
- **A3 (LP shaft, magnitude).** Both within 0.01 of § 1's table at every `Tt4`.
- **A4 (LP shaft, HP crossing).** The HP response changes sign ONCE, positive above, at
  **783 ± 75 K** (slope 1.3e−4 /K).
- **A5 (LP shaft, LP crossing).** The LP response changes sign ONCE, negative above, at
  **1290 ± 55 K** (slope 1.9e−4 /K).
- **A6 (equal power).** Per unit `w` (power mode, both shafts), the HP-shaft load lowers BOTH
  `φ_L` and `φ_H` more than the LP-shaft load at every `Tt4`.

**Steady — shaped (`flow/press`, sign only):**
- **A7.** HP shaft: both spools negative at 1500 and 650 K. LP shaft: HP positive at 1500 K and
  negative at 650 K; LP negative at 1500 K and positive at 650 K.

**Transient — rig B** (from the steady model over the ramp, 1000–1400 K: per `w` the HP-shaft load
gives LP −2.9 to −1.6 and HP −1.7 to −1.0; the LP-shaft load gives HP +0.21 to +0.31 and LP
+0.40 to −0.06 flat — on `flow/press` the § 0 decomposition gives LP −0.56 to −0.76, so its LP sign
is map-dependent and **not scored**):
- **B1.** Valve shut, the HP-shaft load LOWERS `min φ_LP` and `min φ_HP` in all 6 cells.
- **B2.** Valve shut, the LP-shaft load RAISES `min φ_HP` in all 6 cells.
- **B3.** The valve's bleed integral RISES with the HP-shaft load at every set point where it rides
  in both runs; the ceiling FALLS in all 6 cells.
- **B4 (which shaft pays).** The HP-shaft load costs the valve MORE than the LP-shaft load of the
  same power at every set point where both ride.
- **B5 (acceleration).** The HP-shaft load DELAYS `ν_H`'s 90 % rise time in all 6 cells. Basis (the
  least bounded claim here): at an equilibrium a fixed power adds no damping term (the load's
  `1/ν` derivative multiplies a zero net power), so it acts through the state — a lower HP speed at
  the same `Tt4`, where the spool is slower.

**The likeliest miss, named in advance:** B5 — its basis is qualitative and the 90 % time also
depends on how the load reshapes the speed's end point. Among the steady claims, A5: the LP-shaft
LP response is a near-cancellation of two `O(1)` terms, so its crossing is the most sensitive
reading in the table.
