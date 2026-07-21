# Rung-37 anchor — the two internal clocks (volume-filling + heat-soak)

Two-part anchor, mirroring rungs 31–36. Part A is the **method** (the intercomponent-volume model for
volume-filling; the lumped-metal heat-soakage model) and the numbers it produces on the project's own
machinery. Part B is the **reduce + finding data** — both effects OFF ⇒ rung 35, the two equilibria ==
rung 35 via the independent closures, and the two load-bearing signs (peak = `E0`; `cold < hot <
adiabatic` + the accel-lag).

Design REFERENCE = the rung-34 choked-convergent point: `π_c=10, Tt4=1500, M0=0.85`, real losses
(`pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98`),
`nozzle_convergent=True`, on the fast `thermally_perfect` gas (the transient physics is
gas-independent — rungs 32–35 precedent). Maps: rung 34's surge-realistic `ComponentMap` shapes
(`surge_flow`, `surge_pressure`, `surge_tilted`). Acceleration probed: `Tt4` 1100→1400.

## Part A — the methods

### Volume-filling — the intercomponent-volume model

The canonical gas-turbine transient volume method (Fawke & Saravanamuttoo, *Digital computer
simulation of the dynamic response of a twin-spool turbofan*, ASME/CIMAC 1971; Cohen–Rogers–
Saravanamuttoo *Gas Turbine Theory* Ch. 9): a control volume `V` between the compressor and the
turbine whose pressure is a **state** driven by the compressor/turbine **mass-flow imbalance**:

```
m_cv = pt4·V/(R·Tt4) ,   dpt4/dt = (R·Tt4/V)·(ṁ_c + ṁ_fuel − ṁ_NGV)   (mass storage; Tt4 quasi-steady)
```

The compressor is run from the **back-pressure** the plenum imposes (`π_c = pt4/(π_b·pt2)`, inverted
for the corrected flow `m` on the stable branch), the NGV is the choke (`ṁ_NGV = A4·pt4·MFP*/√Tt4`),
and their difference charges the volume — the **first rung where `ṁ_c ≠ ṁ_NGV`**. The nondimensional
clock `r_v = τ_fill/τ_spool` scales `dpt4/ds` so the design-point linearized drain rate is `1/r_v`.

### Heat-soak — the lumped-metal heat-soakage model

The standard transient-performance heat-soakage model (CRS Ch. 9; Walsh & Fletcher *Gas Turbine
Performance*, transient chapter): a lumped metal thermal capacitance between burner-exit and
turbine-inlet, lagging the gas:

```
Tt4,turb = Tt4,burner − G·(Tt4,burner − Tm) ,   G = hA/(ṁ4·cp)
m·c·dTm/dt = hA·(Tt4,burner − Tm)  ⇒  dTm/ds = (Tt4,burner − Tm)/r_m ,   r_m = τ_soak/τ_spool
```

The documented consequences: heat-soak **steals turbine work → slows the acceleration** (the
thrust-response lag, the primary transient-performance effect), and — because the metal retains its
temperature — a **hot reslam** (a re-acceleration from a hot engine, the "bodie") is more
surge-critical than a cold first acceleration.

## Part B — reduce + finding data

**Reduce 1 — both OFF ⇒ rung 35 bit-for-bit.** `plenum_ratio=0` and `soak_gain=0` are the defaults;
the inherited `equilibrium_fuel`/`integrate_fuel` never read them, so they equal a plain
`SpoolTransient`'s outputs bit-for-bit (dispatch, not re-solve). The rung 31–36 suites pass unchanged
(19/19 in the neighbour run) — the bit-for-bit witness.

**Reduce 2 — plenum equilibrium == rung 35 (non-tautological).** `equilibrium_plenum` reproduces
`equilibrium_fuel` through the **back-pressure** closure (invert `π_c(m)`), a different code path than
rung 35's NGV-continuity root-find. Observed (fast gas): `|Δπ_c|/π_c ≤ 1.5e-11`, `|Δν| ≤ 8.5e-12`,
mass balance `|ṁ_c+ṁ_fuel−ṁ_NGV|/ṁ_NGV ≤ 2e-14` at the fixed point, across 3 shapes × `Tt4 ∈
{1400,1100,900}`.

**Reduce 3 — heat-soak equilibrium == rung 35 (transient-only).** At steady state `Tm = Tt4,burner`
⇒ `Q = 0` ⇒ `Tt4,turb = Tt4,burner`, so `equilibrium_soak` reproduces `equilibrium_fuel` (`|Δπ_c|/π_c
≤ 1.6e-11`, `ν` matched to 5 digits) — heat-soak **never moves the running line**.

### Finding 1 — volume-filling CONFIRMS `E0` (the peak) + the mass-flow SPLIT

Frozen-spool (`r→0`) fuel step `Tt4` 1100→1400, `M0=0.85`:

| shape          | `E0` (rung 35) | plenum peak | peak − `E0` | max split |
|----------------|----------------|-------------|-------------|-----------|
| surge_flow     | 10.156%        | 10.156%     | −0.00000%   | 22.0%     |
| surge_pressure | 13.062%        | 13.062%     | −0.00000%   | 22.1%     |
| surge_tilted   | 11.389%        | 11.389%     | −0.00000%   | 22.1%     |

The peak lands on rung-35's algebraic `E0` **to machine zero**, and is **identical at `r_v = 0.03` and
`0.1`** (a frozen-spool map fact — the plenum fills to full quasi-steady `pt4` before `ν` can move).
So volume-filling **confirms** the concession; the load-bearing content is the **~22% `ṁ_c ≠ ṁ_NGV`
split** the plenum stores (the first rung to break rung 34's rigid `pt4 = π_b·π_c·pt2` coupling).

### Finding 2 — heat-soak CORRECTS: `cold < hot-reslam < adiabatic` + the accel-lag

Acceleration `Tt4` 1100→1400, `G=0.15`, `r_m=3.0`, `surge_flow`:

| θ₀ (initial metal) | E_surge (peak) | t_accel (99% speed rise) |
|--------------------|----------------|--------------------------|
| adiabatic (rung 35)| 10.16%         | 2.15                     |
| cold first-accel   |  8.75%         | 6.65   (~3× slower)      |
| hot reslam (bodie) |  9.42%         | 2.00   (≈ adiabatic)     |

Ordering `cold < hot-reslam < adiabatic` — the cold metal's heat sink depresses `Tt4,turb` → colder
NGV passes **more** corrected flow → higher `φ` → **away** from surge (channel a wins). rung-34/35's
adiabatic combustor is the conservative **worst case**; a hot reslam recovers most of it. The
**primary** cost is the accel-time **lag** (cold ~3× slower; `t_accel → >s_end` as `r_m` grows). `E =
E(r, θ₀)` — history-dependent, not a function of `r` alone.

**Sign robustness** (the load-bearing claims). `cold < hot-reslam < adiabatic` holds across every
`G ∈ {0.05, 0.10, 0.15, 0.20} × r_m ∈ {1, 3, 5, 10} ×` 3 shapes in the probe (channel a always wins
the peak; the peak is nearly `r_m`-independent — an early-time effect). The accel-lag grows monotone
in `G`.

## What this anchor deliberately does NOT establish

- No **magnitude** of the plenum path-cushion, the heat-soak surge protection, or the accel-lag — all
  ride on the disclaimed `r_v`, `G`, `r_m` (the `I`/`L`/`τ_res` concession, twice). Only the peak =
  `E0` identity, the `ṁ_c≠ṁ_NGV` existence, and the two signs (`cold<hot<adiabatic`, accel-lag) are
  load-bearing.
- No **combined** 3-state (`ν, pt4, Tm`) model — the effects are exhibited separately (the contrast is
  the point); the interaction is a further seam.
- No **energy-storage** plenum (`Tt4` is quasi-steady in the volume), no distributed/flow-varying
  `hA`, no tip-clearance transients, no two-spool dynamics — further dynamic seams.
- No surge-line **crossing**: heat-soak *reduces* the excursion (protective); the crossing is not
  claimed. Fuel control, choked/subsonic dispatch, NGV choke, isentropic knobs — inherited rungs 31–35.
