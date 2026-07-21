# Rung-34 anchor — the spool transient

Two-part anchor, mirroring rungs 31–33. Part A is the **method** (the standard quasi-steady-shaft
transient model) and the design values it produces. Part B is the **reduce gate** — the
equilibrium of the shaft ODE reproduces the rung 31/32 running line via an independent forward
closure (the rigorous, non-tautological anchor) — plus the finding data (`E(r)`) and the spool-down.

Design REFERENCE = the choked-**convergent** design point (rung 30): `π_c=10, Tt4=1500, M0=0.85`,
real losses (`pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98`),
`nozzle_convergent=True`. Same fixed throats `A4, A8` as rung 31; the compressor/turbine maps are
rung 32's `ComponentMap` (with rung 34's linear loading slope `l` for the surge-realistic shapes).

## Part A — the method (quasi-steady components, dynamic shaft)

Standard gas-turbine transient model (Cohen–Rogers–Saravanamuttoo *Gas Turbine Theory* Ch. 9;
Walsh & Fletcher *Gas Turbine Performance*, transient chapter): the gas path is acoustically fast
(choked throats + combustion re-establish in ms) so the flow is **quasi-steady**, while the rotor
carries all the inertia. The single dynamic state is the shaft speed `N`, governed by

```
I·ω·(dω/dt) = η_m·P_turbine(N,Tt4) − P_compressor(N,Tt4)
```

**The forward closure at (N, Tt4)** — the structural novelty (rungs 31–32 ran the map backward):

```
n      = (N/√Tt2)/(N/√Tt2)_d = ν·√(Tt2_d/Tt2)                 # corrected speed from ν=N/N_d
τ_c    = 1 + (τ_c,d−1)·ψ(φ)·n² ,  ψ(φ)=1−σ(φ−1)²−l(φ−1) ,  φ=m/n   # Euler speed line, FORWARD
π_c    = pr_c(Tt3s)/pr_c(Tt2) , Tt3s = T_from_h_c(h_c(Tt2)+η_c·[h_c(Tt3)−h_c(Tt2)])
ṁ(1+f) = A4·pt4·MFP*(Tt4,f)/√Tt4 ,  pt4 = π_b·π_c·pt2           # NGV choke CLOSES m (no shaft balance)
```

The turbine expansion is `(★)`-pinned when the nozzle is choked (rung 31) and set by nozzle
continuity `A8·ρ9·V9 = ṁ4` when it is subsonic (rung 33) — dispatched by the same rung-33 logic. The
nondimensional shaft ODE (time `s = t/τ_spool`, `τ_spool = I·ω_d²/P_ref`):

```
dν/ds = Φ(ν,Tt4) = (ṁ_air·[η_m·P_t,spec − P_c,spec]) / (P_ref·ν)
```

### The linear loading slope `l` (rung 34's map addition)

Rung 32 used the speed line **backward** (solve_n) near design, where the parabola `1−σ(φ−1)²`
(which peaks at `φ=1`) was adequate. Run **forward**, the parabola's zero slope at design gives the
**wrong sign** on the low-flow (surge) side — a real compressor speed line has `π_c` **rising**
toward low flow. The linear term `l>0` supplies the physical negative slope `dψ/dφ|_1 = −l`. It
defaults to 0 (rung 32 bit-for-bit; `solve_n` calls `psi()` which is arithmetically identical at
`l=0`), and the rung-34 surge-realistic shapes (`surge_flow/pressure/tilted`) turn it on.

## Part B — the reduce gate (the rigorous, non-tautological anchor)

The equilibrium of the shaft ODE (`Φ=0`, i.e. the `ν` where `η_m·P_t = P_c`) is solved by the
forward closure alone (it never calls `OffDesignMatcher.match`/`MapMatcher.match`). It reproduces:

| gas | map | matched at | Δπ_c | Δ(N/ν) |
|-----|-----|-----------|------|--------|
| thermally_perfect | flat | `Tt4` 1500→520 (incl. subsonic) | ≤3e-13 | — |
| thermally_perfect | surge_flow | `Tt4` 1500,1200,900 | ≤3e-12 | ≤1e-12 |
| reacting_equilibrium | flat | design | 6e-11 | — |

Two genuinely separate code paths — the steady matcher's **choke + shaft-balance** closure and the
transient's **forward-map + NGV-continuity + power-balance** closure — onto one operating point. At
design the flat-map equilibrium returns `π_c=10, ν=1` to <1e-7. This is the tightest, most
tautology-free anchor (the rungs 29/31/32/33 move: reduce via an independent construction).

### The finding — `E(r)`, peak acceleration excursion above the running line (surge_flow map)

Acceleration `Tt4` 1100→1400, `M0=0.85`; excursion `E = max_t[π_c/π_c,rl(ν) − 1]`:

| `r = τ_fuel/τ_spool` | `E` | `E/E0` |
|------|------|------|
| 0 (algebraic constant-N limit) | +5.39% | 1.000 |
| 0.2 | +4.92% | 0.913 |
| 0.5 | +4.31% | 0.800 |
| 1.0 | +3.47% | 0.643 |
| 2.0 | +2.33% | 0.432 |
| 5.0 | +1.04% | 0.194 |

Monotone-decreasing from the constant-`N` map displacement to ~0, knee near `r≈1`. The `r→0` limit
equals the **algebraic** constant-`N` displacement `π_c(ν0,1400)/π_c(ν0,1100)−1` computed with **no
integration** — the step excursion is a map property; the dynamical content is the ratio.

**Direction, shape-robust** (constant-`N` `E0`): accel `+5.4/+7.0/+6.1%`, decel `−7.0/−8.2/−7.6%`
across `surge_flow/pressure/tilted`. Sign robust; magnitude disclaimed (no surge line drawn — rung
32's concession).

### Spool-down (the rung-33 handshake)

Fuel cut `900→460` over `r=6`: `N` coasts `ν` 0.732→0.503; at `s≈4.6` `pt9/p0` falls through
critical, the branch flips **choked→subsonic** at `M9≈0.996` (continuous), and the trajectory
approaches thrust-neutral idle (specific thrust `360→9` N·s/kg). A too-fast fuel chop instead drives
the point off the map at the flameout boundary (reported by the integrator stopping) — the decel
analogue of the accel-toward-surge excursion.

## Cross-links

- **Method anchor:** Cohen–Rogers–Saravanamuttoo Ch. 9 / Walsh & Fletcher transient chapter — the
  acceleration/deceleration excursion between the surge and flameout lines, and the scheduled fuel
  ramp that keeps it clear. Same textbook family as the rung-2/30/31/32/33 design/off-design anchors.
- **Hardware anchor:** the fixed throats `A4, A8` (rung 31 capture), the convergent nozzle (rung 30),
  the compressor/turbine maps (rung 32 `ComponentMap`, + rung 34's `l`).
- **Reduce anchor:** the forward-closure equilibrium == rung 31 (flat) / rung 32 (shaped), the
  independent-path gate (two closures, one point).
