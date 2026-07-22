# Rung 39 anchor — two-spool + component maps

Verified data behind `docs/rung39-spec.md`. Design point: `pi_LPC`=3, `pi_HPC`=6, `Tt4`=1500 K,
`M0`=0.85, `T0`=250 K, `p0`=50 kPa; losses `pi_d`=0.97, `eta_LPC`=0.90, `eta_HPC`=0.88,
`eta_b`=0.99, `pi_b`=0.96, `eta_HPT`=0.92, `eta_LPT`=0.90, `eta_m`=0.99, `pi_n`=0.98;
fixed convergent nozzle (rung 30). Same hardware as the rung-38 anchor, so every number below
is directly comparable to it.

## Disclosed map shapes

Compressor island `eta_c = base - a*(phi-1)^2 - b*(n-1)^2 - c*(phi-1)*(n-1)`; loading
`psi = 1 - sigma*(phi-1)^2 - l*(phi-1)`; turbine `eta_t = base - a_t*(nu_t-1)^2`.

| pair | LP map (a, b, c, sigma, l) | HP map (a, b, c, sigma, l) |
|---|---|---|
| `flow_dom`  | 0.20, 0.05, 0,    0.1, 0.70 | 0.20, 0.05, 0,    0.1, 0.70 |
| `press_dom` | 0.05, 0.20, 0,    0.1, 1.00 | 0.05, 0.20, 0,    0.1, 1.00 |
| `tilted`    | 0.14, 0.10, 0.06, 0.2, 0.85 | 0.14, 0.10, 0.06, 0.2, 0.85 |
| `mixed`     | 0.20, 0.05, 0,    0.1, 0.70 | 0.08, 0.15, 0,    0.1, 1.00 |

`a_t`=0 for the structural (finding-A) readings; `a_t`=0.02 for the back-arrow and slip runs.
**All shapes are representative** (rung-32 methodology): claims verified shape-robust,
magnitudes disclaimed.

## Finding A — the asymmetry (compressor maps only, `a_t`=0)

Perturbation `-0.01` on one efficiency, at a **fixed** `(Tt2, pt2, Tt4, f)` (rung-38 gate-3's
isolation protocol). Relative change in each compressor pressure ratio:

| shape | `Tt4` | `eta_LPC -> pi_HPC` | `eta_HPC -> pi_LPC` | ratio |
|---|---|---|---|---|
| flow_dom  | 1400 | **EXACTLY 0** | −2.592e−04 | ∞ |
| flow_dom  | 1200 | **EXACTLY 0** | −5.639e−04 | ∞ |
| flow_dom  | 1000 | **EXACTLY 0** | −6.610e−04 | ∞ |
| press_dom | 1400 | **EXACTLY 0** | −1.545e−04 | ∞ |
| press_dom | 1200 | **EXACTLY 0** | −3.072e−04 | ∞ |
| press_dom | 1000 | **EXACTLY 0** | −3.293e−04 | ∞ |
| tilted    | 1400 | **EXACTLY 0** | −2.470e−04 | ∞ |
| tilted    | 1200 | **EXACTLY 0** | −5.378e−04 | ∞ |
| tilted    | 1000 | **EXACTLY 0** | −6.291e−04 | ∞ |
| mixed     | 1400 | **EXACTLY 0** | −2.589e−04 | ∞ |
| mixed     | 1200 | **EXACTLY 0** | −5.648e−04 | ∞ |
| mixed     | 1000 | **EXACTLY 0** | −6.640e−04 | ∞ |

Reacting gas (`mixed`): `EXACTLY 0` at `Tt4`=1200/1000, arrow −5.596e−04 / −6.495e−04 — the
cancellation is **gas-independent**, as the algebra requires.

> The exact zeros are a **code-level guarantee**, not a numerical accident: `_hp_eta_loop`
> computes the HP-face corrected flow from the closed form `(†)`, which contains no `pi_LPC`,
> and is called **before** `_lp_eta_loop`. An implementation that iterated all four
> efficiencies jointly leaves ~1e-15 residue instead (measured during the probe stage, and the
> reason the shipped solve is triangular by construction).

**Contrast (asserted in the same gate):** `eta_HPT` and `eta_LPT` move **both** ratios at every
point — e.g. `mixed`/1200: `eta_HPT` → (+1.261e−03, −1.634e−02), `eta_LPT` → (−9.790e−03,
+3.922e−03). Rung 39 does **not** claim the spools stop talking.

## Finding A — the weak back-arrow (`a_t`=0.02)

A representative turbine map opens the closed leaf, via
`eta_LPC -> phi_L -> n_L -> nu_LPT -> eta_LPT -> Tt5 -> Tt25 -> pi_HPC`:

| shape | `Tt4` | `eta_LPC -> pi_HPC` | `eta_HPC -> pi_LPC` | ratio |
|---|---|---|---|---|
| flow_dom  | 1400 | 8.166e−07 | 2.621e−04 | 321× |
| flow_dom  | 1200 | 1.376e−06 | 5.694e−04 | 414× |
| flow_dom  | 1000 | 1.216e−06 | 6.666e−04 | 548× |
| press_dom | 1400 | 1.339e−06 | 1.593e−04 | 119× |
| press_dom | 1200 | 2.273e−06 | 3.163e−04 | 139× |
| press_dom | 1000 | 2.019e−06 | 3.385e−04 | 168× |
| tilted    | 1400 | 1.066e−06 | 2.509e−04 | 235× |
| tilted    | 1200 | 1.770e−06 | 5.450e−04 | 308× |
| tilted    | 1000 | 1.534e−06 | 6.362e−04 | 415× |
| mixed     | 1200 | 1.393e−06 | 5.701e−04 | 409× |

Gate 5 asserts only `>50×` — the **ratio is disclaimed** (it rides on `a_t`). The mechanism is
rung 32's own sub-finding ("the turbine is pinned in corrected speed") on the LP spool.

## Finding B — the speed slip `N_L/N_H`

| gas / map | `Tt4`=1500 | 1300 | 1100 | 900 |
|---|---|---|---|---|
| CPG, **flat** | 1.000000000 | 1.000000000 | 1.000000000 | 1.000000000 |
| thermally-perfect, flat | 1.000000000 | 0.995767632 | 0.990994820 | 0.985184874 |
| reacting, flat | 1.000000000 | 0.995188254 | 0.989857143 | 0.983508436 |
| CPG, `mixed` shaped (`a_t`=0.02) | 1.000000000 | 0.980781 | 0.964264 | 0.949717 |

Channel sizes at `Tt4`=900: gas curve **1.48%** (thermally-perfect) / **1.65%** (reacting);
map **5.02%** — the map is **~3.4×** the gas channel, i.e. the dominant of the two, but **not
the sole** one (the spec states it that way; an earlier "100% map content" reading was
refuted by this table during the probe stage).

**B1 is structural.** Both shaft works are `eta_m*(1+f)*cp_t*Tt4*[pure geometry]` once both
NGVs choke, so `(1+f)` and `Tt4` cancel in `N_L/N_H`. Verified directly by forcing `f` to
0.5×/1×/2×/4× its solved value at `Tt4`=1200 — slip stays 1 to <1e−9 at every one (gate 6).

**B3 direction (`a_t`=0.02, CPG):** monotone-decreasing at every shape pair —

| shape | 1500 | 1300 | 1100 | 900 |
|---|---|---|---|---|
| flow_dom  | 1.000000 | 0.978122 | 0.959220 | 0.942861 |
| press_dom | 1.000000 | 0.971603 | 0.946803 | 0.925341 |
| tilted    | 1.000000 | 0.974880 | 0.953427 | 0.935217 |
| mixed     | 1.000000 | 0.980781 | 0.964264 | 0.949717 |

Sign gated; **magnitude (5.1%–7.5% at `Tt4`=900) disclaimed.**

## Running line (CPG, `mixed` shaped, `a_t`=0)

| `Tt4` | `pi_LPC` | `pi_HPC` | `eta_LPC` | `eta_HPC` | `N_L/N_Ld` | `N_H/N_Hd` | slip |
|---|---|---|---|---|---|---|---|
| 1500 | 3.0000 | 6.0000 | 0.90000 | 0.88000 | 1.00000 | 1.00000 | 1.000000 |
| 1300 | 2.6234 | 5.1833 | 0.89773 | 0.87935 | 0.89998 | 0.91761 | 0.980792 |
| 1100 | 2.2785 | 4.3892 | 0.89124 | 0.87722 | 0.80328 | 0.83302 | 0.964301 |
|  900 | 1.9697 | 3.6303 | 0.88162 | 0.87334 | 0.70839 | 0.74585 | 0.949783 |

## Reduce ladder (all verified)

| configuration | reduces to | status |
|---|---|---|
| flat maps, two-spool | rung 38 `TwoSpoolMatcher` | **bit-for-bit** (`==`) on the reacting gas at `Tt4`=1500/1300/1100/900 |
| flat map, `lp_disabled` | rung 31 `OffDesignMatcher` | **bit-for-bit**, by exact dispatch |
| shaped map, `lp_disabled` | rung 32 `MapMatcher` | **bit-for-bit**, by exact dispatch |
| default `build_turbojet(...).run(...)` | rung 6 | untouched |

The flat-map reduce landing on `==` (rather than the ≤1e-9 the spec conservatively promised)
is because `ComponentMap.flat()` returns each `eta_*_at` base exactly, so both efficiency
secants and the turbine loop return on their first pass at the design efficiencies, and the
remaining arithmetic is rung 38's `_cascade` with two independent sub-expressions reordered.

## Non-tautological anchor

Gate 3: an independent bare-math CPG two-spool **map** cascade — no `Gas`, `Component`,
`ComponentMap` or `TwoSpoolMapMatcher` calls; closed-form CPG thermodynamics, its own turbine
and speed-line bisections, and efficiency fixed points by **damped substitution** rather than
the shipped secant — reproduces `(pi_LPC, pi_HPC, eta_LPC, eta_HPC, n_L, n_H)` to ≤1e−8 across
`Tt4` = 1500/1300/1100/1000. Two code paths, one operating point. This is the only anchor that
ties the *map* cascade's numbers down, since the flat-map reduce holds every efficiency at
design and the `lp_disabled` reduce never enters the two-spool path at all.

## Scope reminders

Fully-choked branch only (nozzle unchoke raises rung 38's documented error — verified at
`Tt4`≈600 and below); both NGVs assumed choked; steady only; one `eta_m`; no bypass/bleed/
interstage loss/reheat; no surge line on either spool.
