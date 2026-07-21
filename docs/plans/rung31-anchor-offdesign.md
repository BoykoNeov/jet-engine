# Rung-31 anchor — off-design matching (the choked-hardware running line)

Two-part anchor, both re-derived and verified against the running model
(`M:/claud_projects/temp`-style probes `rung31_probe.py`, `rung31_cpg.py`). Part A pins the
matching solver against the textbook **closed-form referencing** to machine precision (the
rigorous, non-tautological anchor — the rung-30 pattern). Part B is the design-point +
running-line demonstration that carries the finding.

## Part A — the CPG closed-form referencing (the standard method)

With the turbine NGV and the exhaust nozzle **both choked**, the two mass-flow-parameter
constraints force the turbine to run at constant corrected point. For a calorically-perfect
gas this is exact — the standard performance-analysis result in Mattingly *Elements of
Propulsion*, Hill & Peterson, and Cohen–Rogers–Saravanamuttoo (the *equations* are the
anchor; no specific worked example was pinned, and rung 31 derives them independently):

```
τ_t , π_t         =  const   (independent of Tt4, M0)          # choked turbine + choked nozzle
τ_c − 1           =  η_m(1+f)(cp_t/cp_c)(1 − τ_t) · Tt4/(τ_r·T0)   ∝ Tt4/(τ_r·T0)   # slaving
π_c               =  [ 1 + η_c(τ_c − 1) ] ^ ( γc/(γc−1) )      # compressor map (isentropic η_c)
ṁ                 ∝  pt4 / √Tt4                                # choked corrected-flow group
```

### Gate 2 — the TPG matching solver reproduces this BIT-FOR-BIT on a CPG gas

Self-consistent CPG dual gas (`R_t = (γ_t−1)/γ_t·cp_t`, `γ_c=1.4, cp_c=1004, γ_t=1.3,
cp_t=1239`), design `π_c=10, Tt4=1500, M0=0.85`, real losses. Throttle sweep `Tt4 =
1500…1000`:

| `Tt4` | `π_c` | `τ_t` | `π_t` | slaving factor `(τ_c−1)/[(1+f)Tt4/(τ_r T0)]` |
|-------|-------|-------|-------|------|
| 1500 | 10.0000 | 0.839882206 | 0.427878 | 0.195620 |
| 1300 | 7.8626 | 0.839882206 | 0.427878 | 0.195620 |
| 1000 | 5.3257 | 0.839882206 | 0.427878 | 0.195620 |

`τ_t`, `π_t` and the slaving factor are constant to **machine zero** (spread `0.0e0`), and
`π_c = [1+η_c(τ_c−1)]^(γc/(γc−1))` to **< 1e-14**. The sonic-throat matching (`_sonic_throat`
→ `choked_mfp` → the (★) bisection) and the closed-form ratio equations are two entirely
different code paths onto the same operating point. This is the anchor: on the gas the
textbook assumes, the solver *is* the textbook.

## Part B — the reacting-gas design point + running line (the finding)

Design REFERENCE = the choked-**convergent** design point (rung 30): `Gas.reacting_equilibrium()`,
`π_c=10, Tt4=1500, M0=0.85`, real losses
(`pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98`),
`nozzle_convergent=True`. Fixed-hardware throat areas captured from it:
`A4 = 1.35378e-3`, `A8 = 2.94544e-3` (per unit design air mass, m²·… consistent units).

**Reduce-to-design** (gate 1): matching at the design flight + `Tt4` returns `π_c = 10` to
`5.4e-10`, `ṁ/ṁ_R = 1.0`, and specific thrust `745.7145` — identical to the design run (all
stations to ≤1e-9).

### Running line — throttle sweep (M0 = 0.85), the compressor is slaved

| `Tt4` | `π_c` (OUTPUT) | `τ_t` | `ṁ/ṁ_R` | sp. thrust | thrust | nozzle |
|-------|------|-------|---------|-----------|--------|--------|
| 1500 | **10.000** | 0.841794 | 1.000 | 745.7 | 745.7 | choked |
| 1400 | 8.878 | 0.839247 | 0.923 | 687.9 | 635.0 | choked |
| 1300 | 7.857 | 0.836518 | 0.852 | 628.0 | 534.7 | choked |
| 1200 | 6.930 | 0.833560 | 0.785 | 565.5 | 444.0 | choked |
| 1100 | 6.090 | 0.830284 | 0.724 | 500.1 | 362.2 | choked |
| 1000 | 5.328 | 0.826527 | 0.668 | 431.7 | 288.3 | choked |
|  900 | 4.637 | 0.822290 | 0.616 | 360.0 | 221.7 | choked |
|  800 | 4.015 | 0.817875 | 0.569 | 285.0 | 162.1 | choked |

`π_c` and `ṁ` fall together along a single fixed line — the compressor has **no freedom**;
the choked hardware sets the running line (a pumping characteristic produced **without a
compressor map**).

### The finding: `τ_t` DRIFTS on the real gas — driven by the `γ_t(T)` curve

The textbook says `τ_t` is *exactly* constant. On the reacting gas it drifts
**0.841794 → 0.817875 (−2.8%)** over the 2:1 throttle range (1500→800 K, both choked), while
the **CPG** gas holds it constant to machine zero (Part A). On the `M0` axis (`Tt4` fixed) the
drift is only **0.16%** (`0.841964 → 0.840600` over `M0` 0.5→2.0) — `Tt4`, and so the burner
composition, barely moves.

**Kill-test — the `γ_t(T)` curve is the driver.** A three-gas ladder, drift over the same
choked 1500→800 range (the CPG contrast alone holds *both* `γ(T)` and composition fixed, so it
proves existence but not the driver):

| gas | `τ_t` drift | isolates |
|-----|-----------|----------|
| CPG (fixed `γ`) | **0.000%** | the pure-constant `MFP*` ratio |
| `thermally_perfect` (var `cp(T)`, **frozen** comp) | **+2.30%** | the `γ_t(T)`-curve effect alone |
| `reacting_equilibrium` (var `cp` + comp) | **+2.84%** | the full real gas |

The `γ_t(T)` curve carries **81%** of the drift; the composition shift adds the minority
~19%. It is cleanly a `γ(T)`-*curve* effect because within one operating point both throats
carry the **same** frozen station-4 composition, so `R_t` cancels in `MFP4/MFP9` — the residual
is only the different `γ_t(T)` sampled at `Tt4` vs `Tt9`. Same species as rung 30's "0.03% is
the physics, not error."

### The choked envelope (branch boundary)

Throttling back lowers `pt9/p0`; below the critical ratio (~1.85) the nozzle **unchokes** and
the pin is lost. On the reacting gas at `M0=0.85` the nozzle stays choked down to
`Tt4 ≈ 700` (`pt9/p0 = 2.14`) and **unchokes near `Tt4 ≈ 600`** (`pt9/p0 = 1.83`). The
matcher reports `nozzle_choked = False` past this rather than quoting the (now invalid)
choked-branch match — the subsonic-nozzle matching mode is not modeled (deferred).

### M0 sweep (Tt4 = 1500) — the ram lapse

| `M0` | `π_c` | `ṁ/ṁ_R` | sp. thrust |
|------|-------|---------|-----------|
| 0.50 | 11.606 | 0.857 | 833.2 |
| 0.85 | 10.000 | 1.000 | 745.7 |
| 1.20 | 8.245 | 1.238 | 661.6 |
| 1.60 | 6.474 | 1.658 | 562.6 |
| 2.00 | 5.108 | 2.324 | 458.3 |

`π_c` falls with flight Mach (ram raises `Tt2`, so `Tt4/(τ_r T0)` drops) while `ṁ` rises
(higher `pt4` passes more choked flow) — the textbook single-spool trends. (`M0>1` folds in
`ram_recovery`.)

## Cross-links

- **Method / physics anchor:** Mattingly *Elements of Propulsion* Ch. 8 (off-design
  performance analysis / referencing); same textbook family as the rung-2 design anchor
  (`docs/plans/rung2-anchor-mattingly.md`). The constant-`τ_t` choked-turbine result and the
  `Tt4/(τ_r T0)` slaving are reproduced to machine precision on a CPG gas (Part A).
- **Hardware anchor:** the fixed nozzle is rung 30's choked convergent nozzle
  (`docs/rung30-spec.md`), whose sonic-throat solver `_sonic_throat` supplies both choked
  mass-flow parameters here (`choked_mfp`); the design reference IS the rung-30 choked
  design point (specific thrust 745.7).
