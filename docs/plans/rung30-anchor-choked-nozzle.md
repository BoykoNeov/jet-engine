# Rung-30 anchor — the choked convergent nozzle

Two-part anchor, both re-derived and verified against the running model
(`M:/claud_projects/temp` probes `rung30_probe.py`, `rung30_cpg2.py`). Part A pins the
sonic-throat solver against a first-rank textbook result; part B is the design-point
demonstration that carries the finding.

## Part A — the critical pressure ratio (isentropic M = 1)

The choke condition is the standard compressible-flow sonic point. For a calorically
perfect gas (Anderson, *Modern Compressible Flow*; reproduced in every propulsion text —
Mattingly, Hill & Peterson, Cohen–Rogers–Saravanamuttoo):

```
T*/Tt = 2/(γ+1)
p*/pt = ( 2/(γ+1) ) ^ ( γ/(γ−1) )          # critical (choking) static/total ratio
```

| γ | `T*/Tt` = 2/(γ+1) | `p*/pt` | critical ratio `pt/p*` |
|---|---|---|---|
| 1.40 (cold air) | 0.83333 | **0.52828** | 1.8929 |
| 1.33 (design `γ_t(T*)`) | 0.85797 | 0.54382 | 1.8389 |
| 1.30 (Mattingly hot gas) | 0.86957 | **0.54573** | 1.8324 |

The `γ=1.4 → 0.5283` and `γ=1.3 → 0.5457` values are the tabulated gas-dynamics constants.

### Gate 2 — the TPG sonic solver reproduces the closed form BIT-FOR-BIT

The solver root-finds `h_t(Tt9)−h_t(T*) = ½γ_t(T*)R_t·T*` on `T*` (enthalpy-integral
bisection), then `p* = pt9·pr_t(T*)/pr_t(Tt9)`. On a **self-consistent CPG** gas
(`R_t = (γ_t−1)/γ_t·cp_t`, so `cp_t ≡ γ_t R_t/(γ_t−1)` exactly), `Tt9 = 1500 K`,
`pt9 = 300 kPa`, `γ_t = 1.3`:

| | solver | closed form | rel. err |
|---|---|---|---|
| `T*` | 1304.347826086952 | 1304.347826086957 | **3.7e-15** |
| `p*` | 163718.32014422 | 163718.32014422 | **1.6e-14** |

Machine precision. **Caveat (the rounded-R trap):** on the *Mattingly* constants
(`γ_t=1.3, cp_t=1239, R_t=285.9`) `cp_t ≠ γ_t R_t/(γ_t−1)` (1239 vs 1238.9), so the solver
and the `2/(γ+1)` closed form disagree at ~1e-5 — the same sub-1e-4 mismatch the shipped
nozzle's CPG energy-split assert already tolerates (`split_tol = 1e-3`). The solver is the
faithful one (it uses `h` and `γRT` directly); the closed form assumes the exact `cp`
relation. **Gate 2 therefore runs on the self-consistent gas**, where the agreement is
exact.

On the reacting TPG design-point gas the realized critical ratio is `pt9/p* = 1.8395`, which
differs from the `γ_t(T*)=1.3345` closed form (1.8389) by 0.03% — that is the variable-`cp`
physics (γ changes through the finite expansion), not solver error. The M=1 identity holds
regardless: `V*/a* = 1.00000000`.

## Part B — the design-point demonstration (the finding)

Design point `π_c = 10`, `M0 = 0.85`, `Tt4 = 1500 K`, real losses
(`pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98`),
`Gas.reacting_equilibrium()`. Nozzle entry: `Tt9 = 1262.69 K`, `pt9 = 314.27 kPa`,
`p0 = 50 kPa`, `f = 0.02718`, `V0 = 269.79 m/s`.

| quantity | ideal full-expansion (shipped) | choked convergent (rung 30) |
|---|---|---|
| `p9` [kPa] | 50.00 | 170.85 |
| `M9` | 1.8639 | 1.0000 |
| `T9` [K] | 811.71 | 1094.71 |
| `V9` [m/s] | 1039.89 | 642.09 |
| momentum thrust `(1+f)V9−V0` [N·s/kg] | 798.36 | 389.75 |
| pressure thrust [N·s/kg] | 0 | +355.96 |
| specific thrust [N·s/kg] | 798.37 | 745.71 |
| TSFC [kg/(N·s)] | 3.4043e-5 | 3.6447e-5 |

Derived:
- choked ⇒ underexpanded: `p*/p0 = 3.417`.
- exit-velocity drop 38.3%; momentum-thrust drop **51.2%**; **net specific-thrust loss 6.60%**.
- pressure thrust recovers **87.1%** of the 408.6 N·s/kg momentum deficit; it is **47.7%**
  of the choked nozzle's total specific thrust.

## Cross-links

- Reuses the pressure-thrust term validated underexpanded by Mattingly *Elements of
  Propulsion* Example 7.1 (`docs/plans/rung2-anchor-mattingly.md`, `p9 = 2·p0`), which the
  engine already scores to ~0.1%.
- The rung-2 anchor doc itself named this rung as deferred: "The choked convergent nozzle
  (choke detection + branch) stays deferred to a later rung."
