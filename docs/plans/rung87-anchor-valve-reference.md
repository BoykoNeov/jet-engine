# Rung 87 anchor — the CONTROLLED valve beside the stator schedule: what should it watch? (pre-registered 2026-10-15 — § 3)

**Seam:** rung 86 § *The next seam* — *"The cube measures; it does not compose a CONTROLLED valve.
A **φ-referenced bleed limiter beside a stator schedule and a fuel leg** (rungs 64–68's loops on
this plant)"*. Rungs 64–80 put rung 64's valve beside fuel legs and beside a stator LIMITER (68,
69), never beside the stator SCHEDULE: `tests/rung64.rs` builds that machine once (the sibling-trap
gate) and measures nothing on it.

**The user's choice (2026-10-15):** this seam, next. Shipped as KNOBS (the sandbox direction):
every prediction below is scored HIT/MISS, and a miss is a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\jet-rung87-probe`, not in the repo);
§ 1–§ 3 written BEFORE any of § 3's quantities was measured. What § 0 already saw is listed as a
GIVEN and scores no credit.

---

## 0. WHAT EXISTS — measured before anything was written

Rig: rung 86's — CPG two-spool, shaped (`lp`/`hp`) and tilted maps, `φ_surge` = 0.55, `Tt4`
1000 → 1400 K, settle 1.2, `ds` = 0.005, `r` ∈ {0.25, 0.5, 1.0}; LP stator schedule 0.20 below
`n_lo` = 0.65; valve authority `b_max` = 0.10. Every bleed number below is RE-SOLVED through
`b_at_point` (rung 64 § 5.22 (ii): reading `b_of` off a floored march returns exactly 0).

**G1 — THE SIBLING TRAP IS LIVE, AND SILENT.** Rung 86's rig and the sandbox build through rung
62's `build_scheduled_bleed`; its `at_lever(&LeverArm::floored(..))` returns a machine with
`lever.lim = None` and `armed_bleed() = false` — **no panic**. Every rung-87 machine is built by
rung 64's `build_limited_bleed`, whose `at_lever` keeps the floor through `merged(stator, valve)`
(checked: `lim` present, stator scheduled, `armed_bleed`).

**G2 — THE STATORS LOWER THE `φ` VALVE'S CEILING AND RAISE ITS INCIDENCE CEILING.** Rung 64's
ceiling is `min φ_LP` over the FULLY-OPEN march. Measured, valve shut / fully open:

| shape | `r` | `min φ` S0 shut / full | `min φ` S1 shut / full | grid-min `M_i` S0 shut / full | grid-min `M_i` S1 shut / full |
|---|---|---|---|---|---|
| shaped | 0.25 | 0.7118 / 0.7839 | 0.6804 / 0.7415 | 0.4132 / 0.5426 | 0.4497 / 0.5818 |
| shaped | 0.50 | 0.7354 / 0.8095 | 0.6999 / 0.7621 | 0.4585 / 0.5829 | 0.4963 / 0.6237 |
| shaped | 1.00 | 0.7512 / 0.8267 | 0.7125 / 0.7754 | 0.4869 / 0.6085 | 0.5263 / 0.6511 |
| tilted | 0.25 | 0.7103 / 0.7793 | 0.6786 / 0.7370 | 0.4103 / 0.5349 | 0.4487 / 0.5749 |
| tilted | 0.50 | 0.7374 / 0.8085 | 0.7007 / 0.7603 | 0.4620 / 0.5814 | 0.5010 / 0.6222 |
| tilted | 1.00 | 0.7552 / 0.8279 | 0.7148 / 0.7752 | 0.4940 / 0.6104 | 0.5341 / 0.6526 |

(`M_i = T_c − 1/φ_LP + v`, `T_c = 1/φ_surge`; grid minimum over the march points.) So a `φ` valve
loses 0.042–0.053 of ceiling to the stators, while the margin to the metal it is protecting GAINS
0.039–0.043.

**G3 — THE `φ` VALVE BESIDE THE STATORS** (5 set points `sm` 0.30–0.46, `φ_lim = (1+sm)·0.55`,
all 30 cells × both stator faces). Seen, not predicted:
- While it rides, `M_i = T_c − 1/φ_lim + v` to ≤ 3.3e−15 (rung 60's pin, one lever over).
- It bleeds more with the stators on than off in every cell where it opens at all (`b_int`, e.g.
  shaped `r` = 0.5, `sm` 0.38: 9.29e−3 off → 3.71e−2 on), and saturates at lower set points.
- Its incidence minimum moves LATER (as late as s ≈ 0.9), to where the stators have reopened (`v` small)
  and the valve is nearly shut.
- The bill is NOT at the end of the ramp (`b_end` ≈ 0, end thrust within 0.1 %) but in the THRUST
  INTEGRAL: shaped `r` = 0.5, `sm` 0.38, against the same stator face's shut run: −0.74 (S0) and
  −4.78 (S1) on ~863–868.

**G4 — THE `φ` VALVE BESIDE THE `φ` FUEL FLOOR** (rung 49; valve/fuel `sm` 0.38/0.30, 0.38/0.38,
0.38/0.46, 0.46/0.38; both faces, all 6 cells): the pair marches (no panic). Fuel set point BELOW
the valve's ⇒ fuel withheld exactly 0 while the valve holds; the valve saturating hands the
variable to the fuel floor, which then holds its own set point; EQUAL set points ⇒ the fuel floor
cuts some fuel (max 1.3e−3–1.5e−2): ownership of a shared pin is the solver's (rung 66's
degeneracy); fuel set point ABOVE ⇒ the valve never opens and the run starves (rung 86), with the
stators in 8–18 points.

## 1. THE DEVICE — the INCIDENCE-referenced valve

Rung 64's law, with ONE thing changed: the wall it watches (rung 69 did the same to rung 68's
stator limiter; rung 60 to rung 49's fuel floor). Hold `M_i,LP ≥ m_lim`, i.e.

```
    T_c − 1/φ + v ≥ m_lim   ⇔   φ ≥ φ*(v) = 1/(T_c − m_lim + v)
```

With the set point given as `φ_lim` (its value at `v` = 0, so `m_lim = T_c − 1/φ_lim`), `T_c`
CANCELS:

```
    φ*(v) = φ_lim / (1 + v·φ_lim)            (the shape of map.rs's φ_surge(v))
```

and at `v` = 0 this is `φ_lim/1.0` — **the `φ` valve, bit for bit** (`0·x` and `/1.0` are exact).
`v` is a function of the SHAFT STATE alone (rung 60's argument), so inside one outer solve the
target is a constant and rung 64's monotone bracket is untouched: the device is rung 64's solve
against `φ*(v)` in place of `φ_lim`.

Where it lives: a `reference` on `BleedLimiter` (default `φ`), so every sibling constructor that
carries the floor carries its reference (the trap G1 guards against cannot drop half of it).
Instantaneous only: a lagged valve (rung 65) with the incidence reference is REFUSED — rung 65's
readers read `phi_lim` as a flow coefficient. `r64_solve_b` keeps its signature and refuses an
incidence valve; the three rung-64 cells (`try_close`, `try_close_fuel`, `b_at_point`) solve
against `lim.target(v)` and compute `v` only for the incidence reference, so every `φ`-valve
path does not change by a single operation and every oracle stays bit for bit.

## 2. DERIVATIONS

**D1 — the pin (rung 60's, inherited, NO credit, gated).** While riding: the `φ` valve reads
`M_i = m_lim + v`; the incidence valve reads `M_i = m_lim` exactly.

**D2 — the dormancy ledger (arithmetic, NO credit, gated).** At `b = 0` the closed loop IS the
shut run, so a valve never opens iff the shut run never violates its target: the incidence valve
on stator face `S` is dormant iff grid-min `M_i`(shut, `S`) ≥ `m_lim`. Likewise it is fully open at
the minimum only if `m_lim` > grid-min `M_i`(full-open, `S`) (G2), modulo the closed loop's own
trajectory (rung 64's `bounded_by_full`). Applied to G2, `sm` ∈ {0.30 … 0.58} step 0.04
(`m_lim` = 0.4196 0.4613 0.5007 0.5378 0.5729 0.6061 0.6375 0.6674); D dormant · R rides ·
S saturates; lower case = within 0.005 of an edge (unscored):

| cell | face | 0.30 | 0.34 | 0.38 | 0.42 | 0.46 | 0.50 | 0.54 | 0.58 |
|---|---|---|---|---|---|---|---|---|---|
| shaped 0.25 | S0 | R | R | R | r | S | S | S | S |
| | S1 | D | R | R | R | R | S | S | S |
| shaped 0.50 | S0 | D | r | R | R | R | S | S | S |
| | S1 | D | D | r | R | R | R | S | S |
| shaped 1.00 | S0 | D | D | R | R | R | r | S | S |
| | S1 | D | D | D | R | R | R | R | S |
| tilted 0.25 | S0 | R | R | R | s | S | S | S | S |
| | S1 | D | R | R | R | r | S | S | S |
| tilted 0.50 | S0 | D | d | R | R | R | S | S | S |
| | S1 | D | D | d | R | R | R | S | S |
| tilted 1.00 | S0 | D | D | R | R | R | r | S | S |
| | S1 | D | D | D | r | R | R | R | S |

(The 0.005 band was chosen KNOWING G3's one closed-loop violation of a full-open ceiling: shaped
`r` = 0.25, S0, `sm` 0.42 held 0.780001 under a 0.7839 ceiling. It is disclosed, not hidden.)

So "the stators lower the incidence valve's bill" scores FREE wherever S1 is D and S0 is R (the
stators do the whole job with the valve shut). Those cells are ledger, not content.

**D3 — why the `φ` valve should cost more at EQUAL protection (the content).** Any valve law
delivering a run minimum `M_i ≥ m_lim` must hold `φ ≥ φ*(v(s))` at every point of the run. The
incidence valve holds exactly that and no more. A `φ` valve holds a CONSTANT `φ ≥ φ_lim`; to deliver
the same run minimum, its `φ_lim` must be at least `φ*(v_b)` where `v_b` is `v` at its own binding
point — which G3 found LATE, where `v` is small. Wherever `v(s) > v_b` (the early, low-speed
stretch, where the stators are most closed) it holds `φ` above what protection needs, and pays
bleed for it. This is a pointwise argument on ONE trajectory; the two valves' trajectories differ
(bleed slows the spool, which moves `v`), so it is a prediction, not a theorem.

**D4 — matched references with a fuel floor (arithmetic, NO credit, gated).** A fuel floor set
STRICTLY below the valve's in the SAME coordinate is dormant (withholds exactly 0 over the whole
run) wherever the valve never saturates — the valve's pin keeps the coordinate above it. The same
holds for an INCIDENCE fuel floor beside a `φ` valve at the same `sm`, with the stators on: `M_i ≥
m_lim + v ≥ m_lim`. G4's EQUAL-set-point case is rung 66's degeneracy: no prediction.

## 3. PREDICTIONS — registered before measuring (bars frozen here)

**Scored set.** "R" cells of D2's S1 face, 20 cells. The `φ` valve beside the stators is MATCHED
to each: Illinois over its `φ_lim` until its grid-min `M_i` equals the incidence valve's
(`|Δ| < 1e−7`); a cell whose root does not bracket is reported, not scored. Bill = against the
SAME stator face's shut run: `ΔF_int` (thrust integral loss), `b_int`, `b_peak`, `ν_LP,end`.

**THE STEP-SIZE BAR.** H1 and H2 must hold at `ds` = 0.0025 as well as 0.005 in every scored cell;
a claim that holds at one step only is GRID-SENSITIVE, not HIT.

- **H1 (the bill, bleed).** At matched protection the `φ` valve bleeds MORE: `b_int(φ*) >
  b_int(inc)` in every scored cell.
- **H2 (the bill, thrust).** And costs more thrust: `ΔF_int(φ*) > ΔF_int(inc)` (loss larger) in
  every scored cell.
- **H3 (the stators pay part of the incidence valve's bill).** On cells R on BOTH faces (11:
  shaped 0.25 {0.34, 0.38}, shaped 0.5 {0.42, 0.46}, shaped 1 {0.42, 0.46}, tilted 0.25 {0.34,
  0.38}, tilted 0.5 {0.42, 0.46}, tilted 1 {0.46}): `b_int(inc, S1) < b_int(inc, S0)`, where S0's
  is the `φ` valve's (D1's reduce) — the opposite sign to G3's `φ` valve.
- **H4 (the ceiling).** Outside the 0.005 band, the incidence valve saturates (some point at
  `b_max` with `M_i < m_lim`) iff D2 says S. Scored on the S1 face (all `sm`) and S0 at `sm` ≥ 0.50
  (G3 saw S0 up to 0.46).
- **H5 (mismatched references hand the work to the fuel).** The incidence valve beside a `φ` FUEL
  floor at the SAME `sm`, stators on: the fuel floor withholds fuel (> 0) in every scored cell —
  the valve lets `φ` fall to `φ*(v) < φ_lim` and the floor wakes — and the thrust integral is
  lower than the incidence valve beside an INCIDENCE fuel floor at the same `sm` (dormant by D4).
  A run that stops early counts as lower; that is reported as a stop, not hidden.

**Size: NO bar.** H1/H2's ratios are reported, not scored.

**Reduces (gates, not predictions):** the incidence valve with the stators off ≡ the `φ` valve, bit
for bit (D1/§ 1); a `φ`-referenced `BleedLimiter` takes every rung-64 path unchanged (every
`rung64`–`rung80` gate and oracle stays green with no edit); the default design run stays rung 6.

## 4. THE KNOBS (sandbox, same batch)

The Controls view's bleed lever gains a mode: **"on a schedule"** (today's, the default — every
saved setting stays valid) or **"opens on its own to hold a stall margin"**, with the margin
(`sm`), the valve's largest opening (`b_max`) and **what it watches: the airflow number (`φ`) or
the blade angle (incidence)**. It is allowed with or without the stators and beside every fuel
limiter. The machine is built through rung 64's builder whenever the valve is chosen (G1). The
lever chart draws the RE-SOLVED opening (`b_at_point`), never `b_of`. Gates follow rung 86's:
replay against the model's own march bit for bit, `check_requests` cases, a `browser.mjs` case,
a crash map.
