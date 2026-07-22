# Rung 40 anchor — the two-shaft transient

Verified data behind `docs/rung40-spec.md`. Design point throughout:
`build_two_spool_turbojet(gas, pi_lpc=3, pi_hpc=6, Tt4=1500, p0=50 kPa, nozzle_convergent=True)`,
`FLIGHT = (T0=250 K, p0=50 kPa, M0=0.85)`, losses
`pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96, eta_hpt=0.92, eta_lpt=0.90,
eta_m=0.99, pi_n=0.98`.

## Disclosed map shapes (compressor islands only, `a_t = 0`)

| key | LP map | HP map |
|---|---|---|
| `flat` | `ComponentMap.flat()` | `ComponentMap.flat()` |
| `flow/press` | `a=.20 b=.05 sigma=.1 l=.7` | `a=.08 b=.15 sigma=.1 l=1.0` |
| `press/flow` | `a=.05 b=.20 sigma=.1 l=1.0` | `a=.20 b=.05 sigma=.1 l=.7` |
| `tilted` | `a=.14 b=.10 c=.06 sigma=.2 l=.85` | same |
| `steep` | `a=.25 b=.12 sigma=.3 l=1.2` | same |
| `lp-only` | `a=.20 b=.05 sigma=.1 l=.7` | **flat** |
| `hp-only` | **flat** | `a=.08 b=.15 sigma=.1 l=1.0` |

`hp-only` is the **discriminator** for the finding: HP shaped, LP flat.

---

## 1. The reduce — the 2-D equilibrium reproduces rung 39

`equilibrium()` solves `Phi_L = Phi_H = 0` by damped Newton from the design start `(1,1)`,
through the FORWARD closure only (it never calls `TwoSpoolMapMatcher.match`).

| gas | `Tt4` | `d nu_L` | `d nu_H` | `d pi_lpc` | `d pi_hpc` | `d mdot` |
|---|---|---|---|---|---|---|
| CPG | 1500 | `+4.4e-15` | `-7.3e-15` | `+1.0e-14` | `-1.1e-14` | `-1.1e-15` |
| CPG | 1200 | `-2.3e-13` | `+1.6e-13` | `-9.4e-13` | `+3.5e-13` | `-5.9e-13` |
| reacting | 1500 | `-6.0e-15` | `-4.4e-16` | `-7.1e-15` | `+2.2e-16` | `-6.9e-15` |
| reacting | 1200 | `-1.4e-13` | `+2.9e-15` | `-6.6e-13` | `-3.6e-15` | `-6.6e-13` |

`lp_disabled=True` -> rung 34 `SpoolTransient`: `==` on `nu, pi_c, tau_c, tau_t, mdot_air, f,
Phi, sp_thrust` (exact dispatch — no two-shaft state is built).

---

## 2. The turbine-split invariance (INHERITED — rung 39 B1)

`Pt_lp/Pt_hp` at frozen speeds, stepping `Tt4` over 1100 -> 1500 (flat maps, so only the gas
differs). The burner cannot choose which spool to feed:

| gas | values | spread |
|---|---|---|
| CPG | `0.38257846` x4 | `4.7e-15` |
| `thermally_perfect` | `0.38269576 … 0.38543381` | `7.2e-3` |
| `reacting_equilibrium` | `0.38182775 … 0.38571618` | `1.0e-2` |

The rung-31-gate-5 mirror: exact on CPG, broken by the `cp(T)` curve.

---

## 3. `sigma_crit` — the identity, the channels, the shape table

**The identity (flat maps + CPG), `d = 25 K`:**

| `Tt4` | 900 | 1100 | 1300 | 1500 |
|---|---|---|---|---|
| `sigma_crit - 1` | `-3.9e-14` | `-4.1e-15` | `+7.3e-15` | `-8.9e-14` |

**The two channels, measured identically at `Tt4 = 1100`:**

| configuration | `sigma_crit - 1` |
|---|---|
| CPG + flat | `-4.1078e-15` |
| `thermally_perfect` + flat | `+4.3120e-02` |
| `reacting_equilibrium` + flat | `+5.1239e-02` |
| CPG + shaped (`flow/press`) | `+2.4920e-01` |

Map channel ~5.8x the gas channel — dominant, **not sole** (rung 39 B2's shape).

**The shape table (CPG), speed-normalized `sigma_crit`:**

| shapes | `Tt4=900` | 1100 | 1300 | 1500 |
|---|---|---|---|---|
| `flat` | `1.000000` | `1.000000` | `1.000000` | `1.000000` |
| `flow/press` | `1.304464` | `1.249148` | `1.198245` | `1.148276` |
| `press/flow` | `1.250030` | `1.176179` | `1.103298` | `1.031937` |
| `tilted` | `1.283428` | `1.218315` | `1.155555` | `1.093735` |
| `steep` | `1.384746` | `1.297890` | `1.216750` | `1.138681` |
| `lp-only` | `0.954391` | `0.885124` | `0.809874` | `0.734420` |
| `hp-only` | `1.223258` | `1.230396` | `1.250532` | `1.277047` |

**The refutation, kept visible.** "The map favours the LP spool" is FALSE: `lp-only` sits
**below** 1 at every throttle, `hp-only` **above**. Only the existence of a material shift is
claimed; the direction rides on which spool is shaped.

---

## 4. THE FINDING — the Jacobian structure

`(a,b,c,d) = d(Phi_L,Phi_H)/d(nu_L,nu_H)` at `rho = 1`, on the running line, `Tt4 = 1200`:

| gas | shapes | `a` | `b` | `c` | `d` | `ad-bc` | `bc` |
|---|---|---|---|---|---|---|---|
| CPG | `flow/press` | `-1.5956` | `+0.5738` | `-0.0478` | `-0.9959` | `1.6165` | `-0.0274` |
| CPG | `press/flow` | `-1.6247` | `+0.8003` | `-0.0392` | `-1.0721` | `1.7733` | `-0.0313` |
| CPG | `tilted` | `-1.6100` | `+0.6763` | `-0.0439` | `-1.0295` | `1.6871` | `-0.0297` |
| CPG | `steep` | `-1.6446` | `+0.8150` | `-0.0515` | `-0.9491` | `1.6029` | `-0.0420` |
| CPG | `lp-only` | `-1.6194` | `+0.8427` | `-0.0035` | `-1.4740` | `2.3899` | `-0.0029` |
| CPG | **`hp-only`** | `-1.4740` | **`-0.0058`** | `-0.0462` | `-1.0177` | `1.4998` | **`+0.0003`** |
| reacting | `flat` | `-1.4846` | **`-0.0114`** | `-0.0264` | `-1.5004` | `2.2272` | **`+0.0003`** |

**Shaping the LP map flips the sign of `b = dPhi_L/dnu_H`** (small-negative -> large-positive).
`c < 0` always. So `bc < 0` exactly for the shaped-LP pairs.

### (i) Stability — the `rho`-free sign conditions

252 `(shape, Tt4, rho, gas)` points: 7 shapes x 3 throttles (1500/1200/950) x `rho` in
`{0.05, 0.2, 1, 5, 20, 100}` x {CPG, reacting}.

- `a<0`, `d<0`, `ad>bc` — **zero violations**.
- Worst (largest) eigenvalue real part over all 252: **`-0.011175`** — ALL STABLE.

Since none of the three conditions contains `rho`, `tr = a/rho + d < 0` and
`det = (ad-bc)/rho > 0` hold for every `rho>0`. The measured part is the sign structure; the
`rho`-freeness is algebra on top.

### (ii) The complex mode — created by the LP map

`disc = (a/rho - d)^2 + 4bc/rho` vanishes at `rho = a/d`, leaving `4bc/rho`, so `bc<0` <=> a
complex band exists.

| shapes | band exists? | band (`Tt4=1200`) | centre `a/d` | `disc` at centre |
|---|---|---|---|---|
| `flow/press` | **yes** | `[1.2329, 2.0822]` | `1.6023` | `-0.0685` |
| `hp-only` | **no** | — | `1.4483` | `+0.0007` |

Smallest discriminant over the whole 252-point sweep: `-2.66e-01` (a complex pair does occur
at `rho = 1` on several shaped-LP pairs on the reacting gas).

**Mode strength (DISCLAIMED MAGNITUDE):** `|Im/Re|_max = sqrt(-bc/(ad))`, attained at
`rho = a/d`:

| shapes | `|Im/Re|_max` |
|---|---|
| `lp-only` | `0.045` |
| `flow/press` | `0.131` |
| `press/flow` | `0.198` |
| `steep` (CPG) | `0.234` |
| `steep` (reacting) | `0.246` |

All `<= 0.25` in the sampled maps — under a quarter cycle before e-folding, so no visible
ringing here. **Reported, not gated**: per rung-32/36/39 methodology the magnitude rides on the
representative shapes and is disclaimed. "It does not ring on these maps" is not "hunting is
impossible."

---

## 5. The withdrawn claims (recorded so they cannot creep back)

**(a) "`sigma_crit` predicts the marched crossover" — TAUTOLOGICAL.** From the running line
`Phi=0`, so `Phi(Tt4+dT) ~ dT * dPhi/dTt4` and the first-instant crossover condition collapses
to `rho = sigma_crit` by definition. The measured "convergence" (ratio -> 1 as `O(dT)`:
`0.99585 -> 0.99894 -> 0.99978`) is finite-difference self-consistency, not two paths meeting.

**(b) "`sigma_crit` is the amplitude->0 limit of the marched threshold `rho*`" — REFUTED.**
`rho*` bisected on the sign of the running-line-referenced slip excursion:

| shapes | `dTt4=200` | `dTt4=50` | `dTt4=12` |
|---|---|---|---|
| `flow/press` (`sigma_crit=1.2491`) | no sign change | `rho*/sigma = 0.6018` | `0.6118` |
| `hp-only` (`sigma_crit=1.2304`) | `1.4755` | `1.4195` | `1.4039` |

`rho*` converges — but not to `sigma_crit`.

**Why:** the running-line-referenced excursion is **schedule-slaved**. At `Tt4 = 1100 + 50`
with the speeds still on the `Tt4=1100` line, the excursion is negative at the first step for
*every* `rho` tested (`0.70x`, `0.95x`, `1.05x`, `1.40x` of `sigma_crit` all give
`-3.6e-4 … -4.1e-4`), because `slip_ss(Tt4)` moves while the speeds lag. The steady schedule's
own drift dominates the lead dynamics.

**(c) "The two-shaft pair does not oscillate" — NOT claimed either.** The complex mode exists;
only its strength is small on these maps, and that is disclaimed.

---

## 6. Scope reminders

- `rho` is a **disclaimed clock group, doubled** (`I_L, I_H, w_L,d, w_H,d` unmodelled); only the
  ratio enters and no wall-clock time is claimed.
- The oscillation claim is scoped to **INTER-SPOOL** — rung 37's shaft+metal Jacobian is not
  audited, so "first oscillatory mode in the project" is NOT claimed.
- Fully-choked branch only, both NGVs assumed choked, one `eta_m`, no bypass/bleed/reheat,
  isentropic knobs only, `Tt4` control (not rung-35 fuel) — all inherited from rungs 38/39.
- No surge line on either spool (rung 36's machinery is single-spool).
