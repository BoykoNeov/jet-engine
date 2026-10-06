# Sandbox anchor — the 1976 US Standard Atmosphere

The sandbox's flight knobs are **altitude + deviation from the standard day**, wired both ways to
raw ambient `T0`/`p0` (user decision, `sandbox-plan.md` § 9.1). That needs a standard atmosphere —
new physics, not wiring — so it is derived and anchored here before it is coded
(`rust/src/atmosphere.rs`).

## Source

*U.S. Standard Atmosphere, 1976*, NOAA/NASA/USAF, NASA-TM-X-74335 (DTIC ADA035728). Constants
cross-checked against the Wikipedia summary of the same document; tabulated values read from the
PDAS reproduction of its Table 1 (`https://www.pdas.com/bigtables.html`, fetched 2026-10-06).

## Derivation

1. **Hydrostatics + ideal gas.** `dp/dH = −ρ g0`, `ρ = p M0/(R* T)`, so `dp/p = −(g0 M0/R*) dH/T`.
   Using **geopotential** altitude `H` absorbs the fall of `g` with height into the coordinate, so
   `g0` is a constant — that is the whole reason the 1976 model is written in `H`.
2. **Temperature is piecewise linear in `H`**: `T = T_b + L_b (H − H_b)` in layer `b`.
3. **Integrate across a layer.** `L_b ≠ 0`: `p = p_b (T_b/T)^(g0 M0/(R* L_b))`.
   `L_b = 0` (isothermal): `p = p_b exp(−g0 M0 (H − H_b)/(R* T_b))`.
   Each layer's base pressure `p_b` is the previous layer's formula at its top — chained from
   `p = 101325 Pa` at `H = 0`, so no base pressure is typed in.
4. **Geometric ↔ geopotential.** `H = r0 Z/(r0 + Z)`, `Z = r0 H/(r0 − H)`, `r0 = 6 356 766 m`.
   The sandbox's altitude knob is GEOMETRIC `Z` (what the table is printed against).
5. **Pressure altitude** (the inverse used when the user types `p0`): find the layer bracketing `p`
   and invert step 3 in closed form — `L_b ≠ 0`: `T = T_b (p_b/p)^(R* L_b/(g0 M0))`,
   `H = H_b + (T − T_b)/L_b`; `L_b = 0`: `H = H_b + (R* T_b/(g0 M0)) ln(p_b/p)`.
6. **Deviation.** `T0 = T_std(Z) + ΔT`, `p0 = p_std(Z)` — the aviation convention: a hot or cold
   day shifts temperature at a fixed pressure altitude.

**Constants** (1976): `g0 = 9.80665 m/s²`, `R* = 8314.32 J/(kmol·K)`, `M0 = 28.9644 kg/kmol`,
`T = 288.15 K` and `p = 101325 Pa` at `H = 0`.

**Layers used** (geopotential base `H_b`, lapse `L_b`): 0 m, −6.5 K/km · 11 000 m, 0 ·
20 000 m, +1.0 K/km · 32 000 m, +2.8 K/km · 47 000 m, 0 (to 51 000 m). Valid sandbox range:
geometric `Z` from −5 km (the table's lower edge) to 47 km.

## Anchor rows (Table 1, geometric Z; T in K, p in Pa, printed to 5 significant figures)

| Z (km) | T (K) | p (Pa) |
|---|---|---|
| 0 | 288.150 | 1.0132E+05 |
| 5 | 255.676 | 5.4048E+04 |
| 10 | 223.252 | 2.6500E+04 |
| 15 | 216.650 | 1.2112E+04 |
| 20 | 216.650 | 5.5293E+03 |
| 25 | 221.552 | 2.5492E+03 |
| 30 | 226.509 | 1.1970E+03 |

**Gate bars, set from the table's printing, not from the code:** `|T − T_tab| ≤ 0.0005 K` (half a
unit in the third decimal) and `|p − p_tab| ≤` half a unit in the fifth significant figure.
Plus: pressure altitude inverts `p_std` (round trip ≤ 1e-6 m), the layer-base pressures are
continuous by construction, and `T0/p0 → (Z, ΔT) → T0/p0` round-trips.

## What the sandbox opens on

The panels' flight point `T0 = 250 K`, `p0 = 50 000 Pa` is not a standard day: it reads as a
pressure altitude of about 5.6 km and a deviation of about −2 K. The page shows that, rather than
moving the project's design point onto the standard atmosphere.
