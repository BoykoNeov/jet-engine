# Rung 85 — THE BLADE-SPEED WALLS

**Code:** `rust/src/blade_speed.rs` · **Gates:** `rust/tests/rung85.rs` · **Panel:**
`print_blade_speed_table` (the first panel born in Rust — held to its own capture,
`rust/oracle/rust_owned/`, a change detector only) · **Anchor (derivations, sources, every
pre-registered bar):** `docs/plans/rung85-anchor-blade-speed.md` — § refs below are to it.

**Shipped as KNOBS** (the user's sandbox decision, anchor § 4.4 A1): every result is shown for
whatever is chosen; predictions were registered before any code and are scored here as HIT/MISS,
a miss being a finding, never a reason to withhold a knob.

## 1. What it adds

Rungs 55/56's stack never named a blade speed; it was IMPLIED twice — by the stage count `K` and
by the capacity level `C` — and the two disagreed by ~30 % on the LP spool (anchor § 0 P-A). Rung
85 sizes the blade from two walls, per spool, and `K`, `C` and a REDLINE `R = N_red/N_d` become
OUTPUTS:

| knob (per spool) | default | source |
|---|---|---|
| hub-to-tip `h` (front row) | 0.5 | disclosed (grid 0.5 / 0.7) |
| airflow design level `M_rel,lim` | 1.4 | 1.3 sourced as typical civil (Biollo & Benini 2011); 1.4/1.5 disclosed |
| material `σ_y/ρ` | Ti-6Al-4V AMS 4911 min, 827.4 MPa / 4429 kg/m³ | Rolled Alloys sheet, first-hand |
| overspeed factor | 1.2 | 14 CFR 33.27 |
| design flow coefficient `Φ_d` | 0.537 | NASA TP 1659, Rotor 37 mean line |
| rounding knob `λ` | 0 | the user's (anchor § 4.5) |
| droop switch | A (with blade speed) | the user's, after scoring (§ 4 below) |

**Roles (the user's decision, anchor § 3):** the airflow level is an efficiency TARGET — it SIZES
the design (`U_d`, `K`, `C`); the STRENGTH wall is the only off-design wall. Off design the
front-row relative tip Mach is a READING. A lever CROSSES when a spool's PHYSICAL speed
`N/N_d` passes that spool's `R`.

**The rounding knob `λ`.** `K = ceil(K*)` leaves the design below the wall. `λ = 0` slows the
blades (rungs 55/56's machine); `λ = 1` keeps them at the wall and loads each row lighter
(`r` = row work vs the unswirled row) through a design pre-swirl `v_d = (1 − r)/(1 + l)`.
That pre-swirl is EXACTLY a reshaped map of the shipped family — `l' = (1+l)/r − 1`,
`σ' = σ/r` (switch A) or `σ` (switch B) — so no solver changed; the lever's `vsv` became travel
from design, and every reading that needs a physical angle adds `v_d` back (anchor § 4.5 A9).

## 2. Reduce

`λ = 0` (or an exactly-integer `K*`) gives `r == 1.0` exactly, and the plants receive the
SHIPPED map objects (`lambda_zero_hands_the_plants_the_shipped_maps`, bit for bit, all shapes).
The lumped lever is rung 55's one-row stack — rung 55's own proven identity with rung 53's plant —
and reproduces rung 53/55's PUBLISHED schedule (`v*` 1.2436, `N_L` 1.26006 at `Tt4` 1000; the
other shapes' `v*` from rung 55's table), the independent check on the route. It is used instead
of rung 53's own schedule because that one PANICS when unreached; a crash behind a sandbox knob is
not acceptable. The sizing reproduces the anchor's § 6.2 / § 6.3 tables (computed by a separate
script before the code existed), typed into the test, all 12 cells × 8 columns and all shapes.

## 3. Scoring (anchor § 4.1 and § 4.5 — bars frozen before code)

| # | verdict | measured |
|---|---|---|
| P0 | settled by construction (A2), no credit | `Φ_d` enters no schedule residual |
| P1 | **MISS** (existence) | LP all-rows at `K` = 2 reaches at 1500/1300 only; at 1100/1000 the scan hits the map edge (`v` 2.1/2.4) unreached — **crosses before target** under V1 (passes 1.395 at travel ≈ 1.2–1.4) |
| P2 | **MISS** | same at `K` = 3: unreached, crosses 1.708 before target |
| P3 | **HIT** at its one scorable point | `(1,5) < (2,5) < (3,5)` at 1300; built so only `K_L` moves; `K_L` = 1 is not wall-produced (V4) — disclosed |
| P4 | **HIT** | HP lumped max `N_H/N_H,d` = 1.000 (at 1500); 0.8848 at 1000 |
| P5 | **MISS** by 1.6 % | HP all-rows at `K_H` = 4: 1.1073 < the tightest 1.125; at `K_H` = 5: 1.1442 < 1.258. No HP cell crosses at `λ` = 0 |
| P6 | **HIT** | HP front-row max 1.000 |
| P7 | **HIT**, all four clauses | `tilted` 1.4123 crosses its 1.337 and not 1.837; `press/flow` 1.1948, `flat-eta` 1.1648 cross nothing |
| P8 | **HIT** | front-row lever: tip Mach falls 13–16 % at 1000 in every cell while `N_L` rises; V3 holds on the PLANT (≤ 1.4e-14) |
| Q0, Q1 | no credit (identity / construction) | `λ` = 1 redline = § 6.2's `R at K*` column |
| Q2 | **HIT — for an unregistered reason** | the LP lumped bill rises strictly in `λ` in all six cells (all reached); the registered mechanism ("steeper map") is REFUTED — see § 4 |
| Q3 | **MISS** | far cell at `λ` = 1: 1.3392, far below the band [1.60, 2.60], under 1.601 |
| old-Q3 | no credit; crosses by **0.09 %** | 1.2872 vs 1.286 — and its reasoning's 1.43 was wrong, by § 4's mechanism |

**Where the rung's crossings are, on the default shape.** At `λ` = 0: only the LP all-rows lever,
and only before a target it never reaches (V1) — at a vane travel ≈ 66–69° physical. At `λ` = 1
(switch A): also the lumped LP lever in the `h` = 0.5, `M_rel,lim` = 1.5 cell, by a hair. The HP
never crosses. Off the default shape, `tilted` crosses at `λ` = 0.

## 4. The finding — holding design incidence cancels the map slope (lumped plant)

A constant-incidence schedule holds `1/φ − v = 1`. Substituting into the shipped loading law:

```
    ψ = 1 − σu² − l·u − v(1+l)φ,   v = 1/φ − 1   ⇒   v(1+l)φ = −(1+l)u   ⇒   ψ = 1 + u − σu²
```

The held machine's work — hence its speed and setting — carries **no `l`**. Measured on the
lumped plant: `l` from 0.3 to 2.0 moves the held `N_L` by ≤ 1.2e-11 (the bisection tolerance),
while the BARE speed moves 0.78 → 0.70. So a bill quoted against bare (rungs 53/55's "+66.73 %")
depends on `l` only through its DENOMINATOR, and rung 55's `tilted`-vs-`flow/press` gap is its
curvature (σ 0.2 vs 0.1) and η island, not its slope — which is what sank Q3's extrapolation and
the anchor's A4 / old-Q3 estimates. **Scope:** exact on the lumped plant only; on a stack only
stage 0 is held and the rows behind keep their `l`.

**Consequence for the knob, and the user's switch.** `λ` reaches a held schedule's speed ONLY
through `σ'`. A9 disclosed `σ' = σ/r` (droop fixed in absolute work units — a loss scaling with
blade speed squared, which `λ` keeps high). Told this, the user chose a SWITCH: **A** (default)
`σ' = σ/r` — `λ` lowers the redline AND raises the schedule speed (up to +6 %); **B** `σ' = σ` —
`λ` lowers the redline and leaves the LP schedule within 0.09 % (the residue is rung 39's HP→LP
arrow: the HP map's `l'` still moves the HP running point). Both are gated.

## 5. Cross-rung verdicts

- **Rung 55's "`K` is a RESOLUTION" — BOUNDED** (D2): true of the stack as an instrument at fixed
  `U`; under the walls `K` is a design OUTPUT.
- **Rung 53's lumped bill — CORRECTED in reading:** its `+66.73 %` is a ratio to a slope-dependent
  bare; the held speed itself is slope-free (§ 4).
- **Rung 55's all-rows non-existence below 1300 — CONFIRMED** at the walls' `K` = 2/3 (P1/P2
  predicted the opposite).
- **Rungs 60/69–71 untouched** by homogeneity (anchor § 6.1).

## 6. Concessions

- Strength: untapered blade-root pull, no disc, no temperature derating, yield not ultimate — an
  OPTIMISTIC wall, so "crosses" survives the model's error and "under" is one-sided.
- Strength and airflow are applied at the cold front row only.
- `λ > 0` holds the blades at the ZERO-swirl wall; the pre-swirl's lower tip Mach would let a
  real designer spin faster still (conservative on the redline). Lighter loading changes no
  efficiency (the η island reads `(φ, n)` only).
- `φ_surge` is unchanged by the pre-swirl IF stall sits at a fixed incidence above the blade's own
  design incidence — disclosed, not derived.
- The V1 crossings sit at vane travels (≈ 66–69° physical at `Φ_d` 0.537) no real stator reaches;
  rung 53's own lumped schedule was already at 1.24 (66°).
- Rung 54/56's throat law is never called (it reads `vsv` as a physical tangent at `Φ_d` = 1); the
  rung reports its own design capacity `C = MFP(M_abs)/MFP(1)` and refuses a design with
  `M_abs ≥ 1` as an error.

## 7. Seams

- The pre-swirl margin spent on more blade speed (a wall solve WITH swirl) — the realistic `λ = 1`.
- The slope cancellation on a STACK (rows behind stage 0) — is the residual `l`-dependence there
  the whole of rung 55's all-rows non-existence?
- A temperature-dependent strength (the hot rear rows, titanium → nickel) and a disc limit.
