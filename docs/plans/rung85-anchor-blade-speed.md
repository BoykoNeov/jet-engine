# Rung 85 anchor — THE BLADE-SPEED WALLS (pre-registered 2026-10-06 — § 4; no code yet)

**Seam:** `docs/per-row-blading-negative.md` § 5 — *"an anchor supplied by physics rather than by
choice … a row-level stress or tip-Mach limit that pins `U` from outside the stack"*; carried in
`CLAUDE.md`'s OPEN list as *"An ANCHOR for the blading"*.

**The user's choice (2026-10-06):** BOTH walls — an AIRFLOW wall (front-row relative tip Mach) and
a STRENGTH wall (centrifugal blade-root stress) — with whichever binds winning, per spool.

**Status:** § 0 (pre-check probes) and § 1 (sources) are DONE. § 2 (the knob table) is written.
§ 3 (open design decisions) and § 4 (predictions) were revised after the FIRST advisor check-in
(2026-10-06); § 0 P-C BLOCKED the design; the user chose `Φ_d` as a design input — see § 6. No code has been written.
2026-10-06 later: § 6.1 (incidence loops invariant), § 1 sourced first-hand, and the user's ROLES
decision (§ 3: airflow SIZES, strength is the only off-design wall).
**2026-10-06, step 3: § 4 REGISTERED** (P0–P8, voids, the NEGATIVE rule), against § 6.2's
per-cell redline table — written BEFORE any code. The draft Q4 ("the lumped stator crosses")
was found already decided by published numbers, the other way, and moved to § 4.0 (no credit).
**Amended the same day, still pre-code (§ 4.4):** the user's SANDBOX decision retires the
ship-or-NEGATIVE gate — rung 85 ships as knobs, all five shapes equal — and five bar defects the
advisor found are fixed (P0 by path, P3's grid, P7 on each shape's own redlines in § 6.3, P8's
identity, the bare-speed note). Next: D6 (the Rust-owned golden, its own commit), then code.
Probes: plain arithmetic on the shipped design numbers (rung 56's CPG rig, `cp` = 1004 exactly, so
`Δh = cp·ΔT` IS the model's own enthalpy there; default shape `flow/press`, `l` = 0.7 LP / 1.0 HP).

---

## 0. THE PRE-CHECK — what the shipped model already implies

The stack (rungs 55/56) is dimensionless: `U` never appears. But it is not absent — it is
IMPLIED twice, by two different disclosed inputs.

**(a) By the stage count `K`.** At design (`φ_k` = 1, `β₁` = 45°, uniform blades) Euler work per
row is `Δh_k = U²(1 − t₂)`, `t₂ = l/(1+l)` (`docs/per-row-blading-negative.md` eq. 1), so

```
    U_d² = Δh_spool / (K · (1 − t₂))                                           ...(1)
```

Shipped design: `Tt2` = 286.125 K, `τ_lpc,d` = 1.409709, `τ_hpc,d` = 1.759671, `cp` = 1004 ⇒
`Δh` = 117.7 (LP) / 307.6 (HP) kJ/kg. At rungs 55/56's `K` = 8:

| `l` | `U_d` LP | `U_d` HP |
|---|---|---|
| 0.70 | 158.1 m/s | 255.7 m/s |
| 0.85 | 165.0 | 266.7 |
| 1.00 | 171.5 | 277.3 |

**(b) By rung 56's capacity level `C`.** `C` = 0.90 ⇒ front-row Mach 0.678; `φ_d` = 1 ⇒ `Vx = U`,
so `U = ν·sqrt(γRTt)` = **220.0 m/s (LP) / 261.2 m/s (HP)**.

**FINDING P-A — the two disclosed inputs describe DIFFERENT MACHINES on the LP spool.** HP: `C`
and `K` = 8 agree to within the `l` spread (261 vs 256–277). LP: `C` implies a blade ~30 % faster
than `K` = 8 can use; a consistent LP has `K ≈ 8·(158/220)² ≈ 4`. Rungs 55/56 never noticed because
neither input was ever converted to a speed. This is the over-determination
`per-row-blading-negative.md` § 4 found, one level down: `U`, `K`, `C` are ONE degree of freedom
carried as THREE knobs.

Implied front-row tip numbers from `C` = 0.90 (hub-to-tip `h` ∈ {0.4, 0.5, 0.6}):
`U_tip` = 275–373 m/s, relative tip Mach **1.08–1.18** — transonic, below NASA Rotor 37's 1.49.

**FINDING P-B — the strength wall does NOT bind at the design flight speed.** Airflow-limited
design `U_tip` at `M0` = 0.85 (wall 1.3–1.5, `h` 0.4–0.7): **314–389 m/s**. Strength redline
(below, § 1): **556–713 m/s**. The strength wall overtakes the airflow wall only above
`M0` ≈ **2.6 – 5.0** (`Tt2` 584–1473 K) — where titanium's strength itself falls with temperature,
which the simple law ignores. **So the strength wall's role at this design point is the REDLINE,
not the design speed.**

**FINDING P-C — BLOCKING: the airflow wall and `φ_d` = 1 cannot coexist with a capacity margin.**
`φ_d` = 1 forces `Vx = U`, so an airflow-limited `U` forces a near-sonic AXIAL Mach at the front
row. The capacity level each wall-derived design implies (`C` = MFP(M)/MFP(1), rung 54's own
definition), over `M_rel,lim` ∈ {1.3, 1.4, 1.5} × `h` ∈ {0.4, 0.5, 0.7}, both spools:

```
    at the wall (continuous K*)   C = 0.939 … 0.9993
    at the integer K              C = 0.908 … 0.9975      (LP K = 3–4, HP K = 5–8)
```

Every cell sits at or past rung 56's shipped 0.90, most of them in the near-choke band where
`per-row-blading-negative.md` § 3 found its `+463 %` artifact and against rung 54's own `C < 1`
assertion. Real transonic rotors run `Vx` well below `U` (a flow coefficient around 0.5–0.6 is the usual
textbook figure — **NOT yet sourced first-hand**); this model has `Vx = U` by construction since
rung 55. **So the rung cannot simply "add the walls": either `φ_d` becomes a design input
(re-founding rung 55's kinematics, `β₁` = 45°, and `t₂ = l/(1+l)`), or the walls are read with the
capacity channel switched off, or P-C itself is the result (a NEGATIVE: the stack's sizing cannot
host a physical blade speed).** Taken to the user.

**Settled by the pre-check — no credit taken (moved here from § 4 at the advisor's check-in):**
- `K` out of the walls at `M0` = 0.85: LP **3–4**, HP **5–8** (P-C's table).
- The AIRFLOW wall binds the design on both spools at `M0` = 0.85 in every cell (P-B, P-C).
- Redline headroom `N_red/N_d − 1` is +43 … +127 % (P-B); rounding `K` up only enlarges it.
- **Which ROW binds is true BY CONSTRUCTION, not a measurement** (retracting draft D5): `U` is the
  same at every row (constant mean radius), rear rows are hotter (lower relative Mach) and their
  blades shorter (higher `h`, lower root stress), and strength is temperature-independent here. So
  both walls bind at each spool's front row under these laws — stated, never "measured".

## 1. SOURCES — verified, with what each does and does NOT supply

- **NASA TP 1659** (Moore & Reid, 1980), *Performance of Single-Stage Axial-Flow Transonic
  Compressor With Rotor and Stator Aspect Ratios of 1.19 and 1.26…*, read first-hand
  (ntrs.nasa.gov/api/citations/19800012840): stage 37, rotor tip speed **454 m/s**, rotor-inlet
  relative Mach **1.493 at the tip** to 1.125 at the hub. Its parent design (its ref. 1): an
  **eight-stage 20:1 core compressor, constant meanline diameter, inlet hub-tip 0.7, inlet
  rotor-tip speed 455 m/s** — explicitly "considerably higher than state-of-the-art" (1980).
  *Supplies:* an UPPER, aggressive example of both the tip Mach and `h`. *Does not supply:* a limit.
- **NASA TP 1659, Table I and Table II(a) — READ FIRST-HAND, and they supply `Φ_d`.** Table I
  (stage 37 design overall parameters): **flow coefficient 0.453** (eq. B23: `(Vz/U_t)` at the
  rotor leading edge, i.e. TIP-referenced), tip speed **454.136 m/s**, hub-tip **0.70**, rotor head
  rise coefficient 0.333, 17 188.7 rpm, airflow 20.188 kg/s. Table II(a) (rotor 37 blade
  elements): at **50 % span** wheel speed 391.7 m/s and meridional inlet velocity 210.2 m/s ⇒
  **mean-line `Φ_d` = 0.537**; absolute (≈ axial — zero inlet swirl) inlet Mach **0.573 (tip) …
  0.661**; relative inlet Mach **1.493 tip … 1.125 hub**. The design work coefficient on the
  mean wheel speed is `cp·ΔT/U_m²` = 1004 × 288.2 × 0.270 / 391.7² ≈ **0.51**, against this
  model's `1/(1+l)` = 0.588 (`l` = 0.7) / 0.500 (`l` = 1.0) — same band.
- **NASA TP 1337** (Reid & Moore, 1978) — the four inlet stages' design report; Rotor 37's design
  (`U_tip` 454 m/s, `h` 0.70, `M_rel,tip` 1.48) as quoted by secondary sources. Not yet read
  first-hand.
- **NASA CR-797** (Miller & Bryans, Allison, 1967) — treats relative inlet Mach ("i.e., tip
  speed") as a TECHNOLOGY LEVEL, not a fixed limit. No stress data. Supports framing the airflow
  wall as a disclosed LEVEL with verdicts as thresholds on it (rung 54's pattern).
- **14 CFR 33.27** (rotor overspeed; text via law.cornell.edu): every rotor must survive
  **120 % of maximum permissible rotor speed** for 5 minutes (115 % for short OEI ratings; 105 % of
  the worst single-failure overspeed). *Supplies:* the gap between the strength capability and
  the REDLINE (`N_red = N_cap/1.2`). *Does NOT supply:* the gap between DESIGN speed and redline —
  that is the model's OUTPUT here (§ 3 D3).
- **Ti-6Al-4V, annealed — READ FIRST-HAND (2026-10-06):** Rolled Alloys, *Data Sheet — 6Al-4V
  Titanium*, Bulletin No. 1052USe 09/16 (rolledalloys.com/wp-content/uploads/6AL4V_Data-sheet.pdf),
  table "Mechanical Properties Specified, AMS 4911, annealed sheet & plate", thickness
  > 0.1874 … ≤ 4.000 in: **0.2 % yield 120 ksi = 827.4 MPa, tensile 130 ksi = 896.3 MPa
  (MINIMA)**; thinner gauges 126/134 ksi. Physical properties: **density 0.160 lb/in³ =
  4429 kg/m³**. The sheet lists "turbine blades, discs and rings" among applications and "high
  strength to 600 °F". *Supplies:* `σ_y/ρ` = 827.4e6/4429 = **1.868e5 m²/s²** (the § 0 P-B
  probe's 827 MPa / 4430 kg/m³ stand, Δ < 0.03 %). *Does NOT supply:* (i) the FORGING minima —
  blades are forged from bar (AMS 4928, listed on the sheet but its minima are not tabulated
  there); AMS 4911 is plate, used as the disclosed stand-in; (ii) any temperature derating (§ 5);
  (iii) typical values — the sheet's "typical range" bar data (yield 128–147 ksi) are NOT used,
  the wall takes the specified MINIMUM.
- **The airflow-wall LEVEL — a peer-reviewed review chapter, READ FIRST-HAND (2026-10-06):**
  Biollo & Benini, *State-of-Art of Transonic Axial Compressors*, ch. 2 of *Advances in Gas
  Turbine Technology* (ed. E. Benini, InTech, 2011), p. 25: *"especially in civil aircraft
  engines, the relative flow tip Mach number of the rotor is limited to maintain high
  efficiencies. A typical value for the rotor inlet relative flow at the tip is Mach ≈ 1.3"*;
  p. 26–27: above ~1.3 the design intent becomes reducing the pre-shock Mach number *"due to the
  strongly rising pressure losses with increasing pre-shock Mach number"* and shock/boundary-layer
  interaction. Same chapter: today's high-efficiency transonic stages run *"tip speed in the order
  of 500 m/s"*. *Supplies:* the LOW end of the swept band, **`M_rel,lim` = 1.3, as the typical
  CIVIL level**, and the mechanism (shock loss ⇒ efficiency, not a hard wall). Only that
  end is SOURCED as a typical level. The sweep's upper values **1.4 and 1.5 are DISCLOSED levels**,
  not sourced limits: TP 1659's Rotor 37 (1.49, a research rotor whose own report calls its parent
  design beyond the state of the art) is an EXAMPLE that a design ran there — a design example, not
  a limit. The whole sweep `{1.3, 1.4, 1.5}` stays a disclosed LEVEL, verdicts as thresholds on it
  (CR-797's framing, rung 54's pattern).
- **NOT read first-hand, NOT cited as a source:** a standard TEXTBOOK statement (Mattingly /
  Cumpsty / Saravanamuttoo / Dixon). A search-engine summary attributes to Dixon & Hall *"rotor
  inlet relative Mach numbers of up to 1.7 are now used"*; the excerpt page (ScienceDirect) and an
  MIT open-access paper (*Performance Limits of Axial Compressor Stages*) both refused the fetch.
  If a textbook is wanted on top of the review chapter, it needs a library read; nothing in the
  band depends on it (1.7 would only widen the sweep's high end, past Rotor 37).

## 2. THE KNOB TABLE — inputs and outputs, per spool

| quantity | rungs 55/56 | rung 85 (walls on) |
|---|---|---|
| cycle `Δh_spool` | output of the design run | unchanged |
| map slope `l` ⇒ `t₂` | input (map shape) | unchanged |
| design sizing `φ_d` = 1 | input | unchanged (disclosed: it forces `Vx = U`) |
| hub-to-tip `h` (front row) | absent | **NEW, disclosed** — ONE shared constant, both walls |
| airflow wall `M_rel,lim` | absent | **NEW, disclosed level**, verdicts as thresholds |
| strength `σ_y/ρ` | absent | **NEW, sourced first-hand** (AMS 4911 min, 827.4 MPa / 4429 kg/m³) |
| overspeed factor 1.2 | absent | **NEW, sourced** (14 CFR 33.27) |
| design speed `U_d` | implied twice, inconsistent (§ 0) | **OUTPUT** — `min(airflow wall, strength wall)`, then the rounding of (1) |
| stage count `K` | input ("a resolution") | **OUTPUT** — `ceil(Δh/(U_wall²(1−t₂)))` |
| capacity level `C` | disclosed input (0.90) | **OUTPUT** — from `U_d` via `Vx = U` |
| redline `N_red/N_d` | absent | **OUTPUT** — `U_cap/(1.2·U_d,tip)` |

**Demoted from input to output: `K` and `C`.** Net constant count: +4 new (`h`, `M_rel,lim`,
`σ_y/ρ`, 1.2), −1 disclosed (`C`), and `K` stops being a free choice.

**Reduce:** walls `None` ⇒ `K`, `C` as supplied ⇒ the rung-56 code path, bit-for-bit.

## 3. DESIGN DECISIONS — status as of 2026-10-06

**The user's decision on the walls' ROLES (2026-10-06):** the airflow level is an EFFICIENCY
TARGET (§ 1, Biollo & Benini: limited "to maintain high efficiencies"), not a hard wall, and the
model attaches no loss to exceeding it. So **the airflow level SIZES the design** (`U_d`, `K`, `C`)
and **the strength redline is the ONLY off-design wall.** Off design the front-row relative tip
Mach is REPORTED as a reading; exceeding the design level is not a "crossing". This narrows the
first decision ("both walls, whichever binds") to the DESIGN point, where it stands unchanged.

| | status |
|---|---|
| D1 integer `K` | **SETTLED** — adopted as proposed |
| D2 rung 55's "resolution" | **SETTLED** — adopted as proposed (BOUNDS, not corrects) |
| D3 vacuity condition | **SETTLED** — registered; does not fire at `M0` = 0.85 (airflow binds the design in every cell, § 0 P-B and § 6 P-C') |
| D4 the experiment | **SETTLED (rewritten)** — the user's roles decision above |
| D5 | retracted |
| D6 the panel | **OPEN** — the golden's meaning for the panel count is settled in its own commit, before the physics |

- **D1 — integer `K`.** `K = ceil(…)` puts the design BELOW the wall by a rounding gap that jumps
  as any input moves (rung 84's staircase). Proposal: every verdict is quoted at the INTEGER `K`
  (that is the machine) AND at the continuous `K*`, with the staircase in redline headroom
  pre-registered as an expected artifact, not a finding.
- **D2 — rung 55's "`K` is a RESOLUTION, not a knob".** Under the walls `K` is a design OUTPUT.
  Proposal: this BOUNDS rung 55 (true for the stack as an instrument at a fixed `U`; false for the
  machine once `U` is anchored), not corrects it.
- **D3 — the overspeed question and its vacuity condition.** Registered BEFORE any result: *if the
  design is strength-bound, `N_red/N_d` = 1/1.2 < 1 — the design itself violates the redline, and
  the rung must instead size `U_d = U_cap/1.2`, whereupon EVERY overspeed crosses the wall by
  construction.* At `M0` = 0.85, § 0 P-B says the design is airflow-bound, so the headroom is a
  derived number; the vacuity condition is exactly "strength binds at design".
- **D4 — which experiment (REWRITTEN 2026-10-06, the user's roles decision).** "Which wall SETS
  the design" is a DESIGN-SPACE question (engines designed at different flight `M0`), answered by
  § 0's probe only: the airflow level, in every cell at `M0` = 0.85. ~~"Which wall does an
  OVERSPEED hit first"~~ — withdrawn: it compared a soft efficiency target with a certification
  failure as if they were one kind of limit. **The rung's off-design question is: does a lever's
  speed bill (rungs 53–61) cross the STRENGTH redline**, which reads PHYSICAL speed
  `N/N_d = n·sqrt(θ₂/θ₂,d)`, each spool its own. The front-row relative tip Mach, which reads
  corrected speed/flow and the stator setting, is reported beside it as a reading, never a verdict.
- **D5** — retracted; see § 0 (true by construction).
- **D6 — the panel.** Rung 85 is the first rung with no Python segment in `cli_golden`. It needs a
  Rust-owned golden and a stated meaning for the panel count. Its own commit, before the physics.

## 4. THE PRE-REGISTERED PREDICTIONS (2026-10-06 — written before any code)

Scored in `docs/rung85-spec.md`, refuted or confirmed. Every bar is read against § 6.2's per-cell
redline `R = N_red/N_d` at the **INTEGER** `K` (the machine, D1); the continuous-`K*` column is
reported beside it, never scored.

**The reads.** Strength reads PHYSICAL speed against design: LP `N_L/N_L,d = n_L` (flight fixed,
`Tt2 = Tt2,d`); HP `N_H/N_H,d = n_H·sqrt(Tt25/Tt25,d)` — `Tt25` falls with throttle, so at part
power the HP's physical speed sits BELOW its corrected one. A **crossing** is `N/N_d > R` for that
(spool, `h`, `M_rel,lim`) cell. Each cell runs BOTH stacks at that cell's own pair of `K`s (§ 6.2
row pair — e.g. `h` = 0.5, `M_rel,lim` = 1.4 is LP 2 / HP 5), because the HP reaches the LP
(rung 39's one arrow).

**The plants and the grid — named, not "somewhere".** Rung 53's lumped steady matcher and rung 55's
stage stack (ALL-ROWS and FRONT-ROW levers), both spools; `Tt4` ∈ {1500, 1300, 1100, 1000} (rung
55's P3 grid); rung 53's constant-incidence schedule (target = the face's / stage 0's design
incidence, read off the matcher, zero new constants); default shape `flow/press`. Scored on the
default shape; the four other disclosed shapes are robustness, P7 excepted. **Not scored:** rung
56's capacity plant (its `C` is now an OUTPUT, 0.649–0.785, and its throat law reads `v/Φ_d`, § 6
— its numbers move by construction; reported as a reading), the transient plants 57+ (`v_max` =
0.20 in map units, every accel ending at `n` = 1), and `Tt4` = 800 (not on rung 53/55's schedule
grid).

### 4.0 Settled by published numbers — NO CREDIT (the draft Q4, retired)

Converted to the strength wall's reference — design speed, not bare-at-throttle (rung 55's
"one currency reconciliation": a referenced excursion reads back its own denominator) — default
shape:

| lever (source) | `N_L/N_L,d` | smallest LP `R` | verdict |
|---|---|---|---|
| rung 53 lumped schedule, `Tt4` = 1000 (rung 55 spec: `N_L(v*)` = 1.26006 against design) | **1.260** | 1.395 | under, by 10.7 % |
| rung 61 stator alone / compensated, `Tt4` = 1500, `v` ≤ 0.30 | ≤ **1.2025** | 1.395 | under |
| rung 55 front row, `K` = 2 (bare 0.7557 × 1.1238) | **0.849** | 1.395 | far under |

So the draft Q4 — *"the lumped stator crosses the strength redline somewhere in the shipped
throttle range"* — is **contradicted** on the LP spool before any code, and *"the front-row stator
does not"* is arithmetic. Both stand only if P0 holds. Also pre-check, not scored: Q5's
design-point SIGN — rung 53's exact `dφ/dv = −(1+l)/(2+l)` with `tan β₁ = (1/φ − v)/Φ_d` — the
relative inlet angle falls while `n` rises; and on rung 53's lumped schedule at `Tt4` = 1000
(its published `φ` = 0.4457, `n_L` = 1.260, `v*` = 1.2436; bare `φ` = 0.708 from its `M_φ`) the
tip relative velocity, lever vs bare, is **−2 %** (`h` = 0.5) / **−10 %** (`h` = 0.7).

### 4.1 The predictions

| # | prediction | bar |
|---|---|---|
| **P0** | **`Φ_d` cannot reach the speed bills** — the map family and `ψ` are `Φ_d`-free (§ 6), and the schedule's target and its read both scale by `1/Φ_d` | rung 53's schedule (`v*`, `n_L`, `n_H`, `φ`) and rung 55's stack schedule at `Φ_d` = 0.537 are **bit-identical (`==`)** to `Φ_d` = 1 at every grid point, both spools |
| **P1** | the LP ALL-ROWS schedule at the walls' **`K` = 2** (5 of the 6 LP cells) **exists**, and lands in a band the 1.395 redline cuts | reaches its target at all four `Tt4` (rung 55's `K` = 8 did not, below 1300); at `Tt4` = 1000, `N_L/N_L,d` ∈ **[1.30, 1.60]** (point estimate **1.40**: log-`K` interpolation between `K` = 1's 1.667× and `K` = 8's > 2.28× of bare 0.7557) |
| **P2** | at **`K` = 3** (cell `h` = 0.5, `M_rel,lim` = 1.3, `R` = 1.708) the same schedule exists and stays **under** its own redline | reaches its target at all four `Tt4`; at 1000, `N_L/N_L,d` ∈ **[1.35, 1.70]** (point **1.49**). One-sided (§ 5): "under" does not survive the optimistic wall |
| **P3** | the all-rows bill is **monotone in `K`** | at every `Tt4` where all three are reached: `N(K=1) < N(K=2) < N(K=3)`, strict |
| **P4** | the HP LUMPED schedule (`vsv_hp`) crosses **no** HP cell | max over the grid of `N_H/N_H,d` (physical) ≤ **1.10** (point ≈ 1.0) < the smallest HP `R`, 1.125. Reason registered: the HP's `φ` droops less with throttle (rung 41: the exposure sits on the LP), and `Tt25` falls |
| **P5** | the HP ALL-ROWS schedule at the walls' `K` (4 or 5) **crosses the tightest HP cell and not the `h` = 0.7 cells** | at `Tt4` = 1000: `N_H/N_H,d` > **1.125** (cell `h` = 0.5, `M_rel,lim` = 1.5, `K` = 4) and < **1.546** (all three `h` = 0.7 cells, `K` = 4) |
| **P6** | the HP FRONT-ROW schedule is far under | max over the grid of `N_H/N_H,d` ≤ **1.00** |
| **P7** | **shape decides the LP lumped verdict** — the one place § 4.0 does not reach | on `tilted` (rung 55: lumped lever +88.79 % vs bare, against `flow/press`'s +66.73 %) the LP lumped schedule at `Tt4` = 1000 **crosses** the 1.395 cells; on `flow/press`, `press/flow` and `flat-eta` it does **not** (`steep` has no rung-53 schedule — inherited). Scored on its own row; never the rung's crossing (§ 4.3) |
| **P8** | **Q5 — the two readings move OPPOSITE under the front-row lever** | design-point identity first: `M_rel,tip(φ=1, n=1, v=0)` equals § 6.2's `M_rel,d` to `1e-12`. Then on the stack at the walls' `K`, front-row lever vs bare at the same `Tt4` ∈ {1300, 1100, 1000}: the front-row relative TIP Mach **falls** (by ≥ 3 % at 1000) in every `h` cell, while physical `N_L` **rises** |

**P1 is registered as a BAND on purpose, not a direction.** Its point estimate (1.40) sits on the
1.395 redline, so "crosses" vs "does not" is not a forecast this anchor can make — naming one
would be a coin-flip dressed as a prediction (rung 83's two void bars died of naming a direction
rather than a point). Both outcomes are named now. **Above 1.395:** the walls turn rung 55's
all-rows verdict from "unreachable at `K` = 8" into "reachable, and over the redline, at the
machine's own `K`" — a changed verdict, and the rung's LP result. **Below:** the LP spool has no
crossing on the default shape, and the rung rests on P5.

**P5 is the riskiest bar, written to lose either way.** No published number gives an HP lever's
speed against design. If `N_H` stays under 1.125 the HP has no crossing; if it passes 1.546,
every HP cell crosses and "the tightest cell only" is refuted.

### 4.2 The voids — a row that trips one is not reported, it is voided

* **V1 an unreachable schedule has no `v*`.** Its redline verdict is read along the SCAN in `v`
  (rung 55's schedule hit its scan edge before its target): `N/N_d > R` at any valid scan point ⇒
  scored "crosses before target"; otherwise the cell is **VOID**, never "under".
* **V2 HP physical speed reads `Tt25` at THAT point.** A verdict read off corrected `n_H` is void.
* **V3 P8's design-point identity must hold**, else every Q5 row is void.
* **V4 `K` comes from § 6.2, per cell, as a PAIR.** A run at a `K` the walls did not produce
  scores nothing.
* **V5 P0 failing voids P1–P6** until the `Φ_d` channel it exposes is named.

### 4.3 ~~The NEGATIVE rule~~ — RETIRED by § 4.4 A1 (the user's sandbox decision, pre-code)

The rung ships as a RUNG iff, **on the default shape**, at least one LP cell (P1/P2) or HP cell
(P4–P6) **crosses** its redline at a reached point or under V1. If none crosses, this is a NEGATIVE
— `docs/blade-speed-walls-negative.md` — whose content is the walls' re-derived `K`, `C` and
redline. A crossing **only** on a non-default shape (P7) is that negative's boundary and does
**not** rescue the rung. Every "crosses" survives § 5's optimistic wall; every "under" is worded
one-sided.

### 4.4 AMENDMENT — 2026-10-06, after commit `7b1d552`, BEFORE ANY CODE

Made after the second advisor check-in and the user's direction change, with no code written and no
run made, so no bar below was moved toward a result. The P-table of § 4.1 stands as written except
where an item here supersedes it.

* **A1 — THE SANDBOX DECISION (user, 2026-10-06).** The project's direction is a SANDBOX — the user
  changes components and designs and watches the engine respond — not a sequence of pass/fail
  lessons. So rung 85 ships as **KNOBS**: blade hub-to-tip `h`, the tip-Mach design level
  `M_rel,lim`, the material `σ_y/ρ`, the overspeed factor, the design flow coefficient `Φ_d` and the
  map shape, with `K`, `C`, `U_d`, the redline and every lever's `N/N_d`-vs-`R` reported for
  whatever is chosen. **§ 4.3's ship-or-NEGATIVE gate is retired, and so is its "default shape
  only" clause: all five disclosed shapes are equal.** The predictions stay and are scored HIT/MISS
  in the spec as recorded expectations; a miss is a finding, never a reason to withhold the knob.
* **A2 — P0 declared by PATH (fixes a bar that could not work either way).** The code takes the
  MAP-UNITS route: `Φ_d` enters ONLY the new sizing / readout code (the walls, `C`, the tip Mach,
  rung 54's throat law `1/sqrt(1+(v/Φ_d)²)`), never rung 53's or rung 55's schedule residual, which
  stays in map units — § 6.1's homogeneity says the solved `v` is identical, so there is nothing
  to recompute. **P0 is therefore SETTLED BY CONSTRUCTION, no credit.** It is replaced by a
  STRUCTURAL gate — the schedule functions take no `Φ_d` and read no struct that carries it
  (checked by the compiler, not by a name grep) — with the `==` comparison kept as a regression
  test only. Had the physical residual `(1/φ − v)/Φ_d` been used, `INC_TOL` would have stopped the
  bisection at a different point and a last-digit miss would have voided the rung via V5. **V5 is
  retired.**
* **A3 — P3 restricted to `Tt4` ∈ {1300, 1100, 1000}.** At 1500 `v*` = 0 and every `K` gives
  `N` = 1 exactly, so a strict `<` fails for a reason with no physics in it.
* **A4 — P7 re-read against EACH SHAPE'S OWN redlines (§ 6.3).** `K`, `U_d` and `R` all depend on
  the map slope `l`; `tilted` is `l` = 0.85 on both spools (`rust/tests/rung55.rs` `maps`), not the
  default 0.7 / 1.0, so § 6.2's 1.395 was the wrong machine's limit. **New P7 bar:** at
  `Tt4` = 1000 the LP lumped schedule on `tilted` crosses ITS tightest LP cell (`h` = 0.5,
  `M_rel,lim` = 1.5, `K` = 2, `R` = **1.337**) and NOT its `h` = 0.7 cells (`R` = **1.837**); on
  `press/flow` (smallest LP `R` **1.286**) and `flat-eta` (same `l` as default, **1.395**) it
  crosses no LP cell. Estimates assume each shape's bare `N_L` ≈ 0.7557 (not published per shape —
  which is why this is a prediction): `tilted` ≈ 1.43, `press/flow` ≈ 1.22 (**within 6 % of its
  bar, flagged**), `flat-eta` ≈ 1.16.
* **A5 — P8's identity is against the FORMULA, not the print.** § 6.2 prints `M_rel,d` to three
  decimals; the `1e-12` check compares the plant's design-point tip Mach with § 6.2's conventions
  recomputed in code.
* **A6 — the bare speed under P1 and § 4.0's front-row row is the LUMPED one** (0.7557, rung 53's
  plant). The stack enters the solver (rung 55), so its bare `N_L` may differ with `K`; P1's band
  absorbs a difference of that size and the front-row verdict (0.849 vs ≥ 1.286 on any shape)
  survives it. No bar changes.
* **A7 — the table's record.** § 6.2/§ 6.3 were computed by a throwaway script outside the repo
  (the repo carries no Python model). **The conventions paragraph is the record**; the rung's Rust
  code recomputes the table, and a test pins its 12 default-shape cells to the printed digits.

## 6. THE USER'S DECISION ON P-C, AND THE DERIVATION IT NEEDS (2026-10-06)

**Decision:** the design flow coefficient `Φ_d` becomes a DESIGN INPUT (sourced: Rotor 37's
mean-line 0.537, § 1); `Φ_d` = 1 is the reduce to rungs 55/56, bit-for-bit.

**Derivation — the map family does NOT change.** The map's `φ` is NORMALISED (`φ = Φ/Φ_d`,
`ψ(1)` = 1). Euler with inlet swirl `v_p = tan α₁` (physical):

```
    ψ_E = U²·[1 − Φ_d·φ·(t₂ + v_p)]
    normalise on design work, match dψ/dφ|₁ = −l    ⇒    Φ_d·t₂ = l/(1+l)        (pinned PRODUCT)
    ⇒   ψ(φ) = [1 − l·(φ − 1)] − v_p·Φ_d·(1+l)·φ      (σ term unchanged, non-Euler)
    ⇒   design work per row   Δh_row = U²·(1 − Φ_d·t₂) = U²/(1+l)        Φ_d-FREE
```

So (i) the shipped `psi` is EXACTLY right with its stator field read as `v = Φ_d·tan α₁` — no
map code changes; (ii) eq. (1)'s `(1 − t₂)` is really `1/(1+l)`, so § 0's `K`-implied speeds
STAND; (iii) the relative inlet angle is `tan β₁ = (1/φ − v)/Φ_d`, so every incidence quantity
`T_c − tan β₁` scales by `1/Φ_d` — RATIOS and zero-crossings invariant, absolute differences not.

**Where `Φ_d` reaches shipped code — found by search, not assumed** (`atan|tan_beta|alpha_1` over
`stator.rs`, `stage.rs`, `map.rs`, `stator_transient.rs`, `stator_bleed.rs`):
- **rung 54's stator-throat law** `A_th(v)/A_th(0) = cos α₁ = 1/sqrt(1+v²)` reads `v` as the
  PHYSICAL tangent. With `Φ_d` ≠ 1 it must read `1/sqrt(1+(v/Φ_d)²)`. **Moves rung 54/56's
  capacity numbers.**
- **the incidence margin** `M_i = T_c − tan β₁` (rungs 53, 55, 56) and the INCIDENCE-referenced
  loops (`stator_transient.rs:279`, rungs 60/69–71): the error scales by `1/Φ_d`. Margins quoted
  as ratios/percent are invariant. ~~a loop's effective GAIN on that error is not~~ — **CORRECTED
  below (§ 6.1): those loops have no gain on the error, and their behaviour is invariant.**
- **capacity** `C`: the front-row axial Mach is now `Φ_d·U/a`, which is the point of the change.

**P-C' — P-C re-run at `Φ_d` = 0.537 (same arithmetic; walls at `v` = 0, `h` ∈ {0.5, 0.7},
`M_rel,lim` ∈ {1.3, 1.4, 1.5}; strength redline = Ti-6Al-4V yield / 1.2):**

```
    capacity C at the integer K :  LP 0.649 … 0.761    HP 0.723 … 0.785      (was 0.908 … 0.9975)
    K                           :  LP 2 – 3            HP 4 – 5
    design U_tip                :  LP 344 … 422 m/s    HP 461 … 523 m/s      (Rotor 37: 454)
    redline headroom N_red/N_d−1:  LP +39 … +92 %      HP +12 … +55 %
    binding wall at design      :  AIRFLOW in every cell
```

So `Φ_d` dissolves P-C (`C` back below rung 56's 0.90 with real margin) and brings the strength
wall CLOSE on the HP spool (+12 % at `h` = 0.5, `M_rel,lim` = 1.5). **The staircase is already
visible:** several `M_rel,lim` cells share one `K` and hence one `U_d` — integer `K`, not the wall
level, sets the machine across most of the band (D1).

### 6.1 SETTLED — the incidence loops are OUT of scope because `Φ_d` cannot reach them (2026-10-06)

The question was whether the incidence-referenced loops' "`1/Φ_d` gain change" is in scope. It
dissolves: **there is no gain.** Rungs 60 and 69–71 are FLOOR loops — each solves for the smallest
`v` holding `M_i ≥ m_lim` (`reference_split.rs:623`, `stator_transient.rs:279`), it does not
multiply the error. Both sides of that inequality come from the ONE normalised map:
`T_c = 1/φ_surge` (`map.rs:376`, zero constants) and `m_lim = T_c − 1/φ_lim`
(`reference_split.rs:213`, built from a `φ` floor). Under `Φ_d` the physical incidence is
`M_i/Φ_d` and the physical floor `m_lim/Φ_d`, so the `1/Φ_d` cancels and the solved `v` is
identical. Hence the trajectory, the Jacobian, rank, zero count and ring are identical too.
**Rungs 60/69–71 are untouched, by homogeneity, not by fiat.**

The two routes by which a PHYSICAL constant could break the homogeneity were checked (both
raised by the advisor):

- **The stator travel `v_max` = 0.20** (rungs 57/58's, inherited by 64–71). If it were a physical
  vane angle, `Φ_d` = 0.537 would grant ~1.9× the physical tangent and, per rung 64 (the ceiling
  IS the authority), move 64–71's protection numbers. **It is not:** every spec and anchor states
  it as IMPOSED and inherited, in map units (`rung68-anchor` l. 128, `rung69-anchor` § 0.2,
  `rung70/71-anchor`). The ONE prose tie to degrees, `rung57-spec.md` l. 76's "20° stator
  rotation", was a slip — `v = tan α₁ = 0.20` is 11.3° at `Φ_d` = 1 — and is corrected there. So
  rung 85 states: **`v_max` is in map units (`v = Φ_d·tan α₁`); its physical travel is
  `atan(v_max/Φ_d)`, which `Φ_d` changes, and no shipped verdict reads it.**
- **Rung 54's stator-throat law** (the one law that reads `v` as a physical tangent) lives on the
  STEADY stator core only (`stator.rs` `throat_margin`/`ThroatRead`, a read-only margin). The
  transient plant the incidence loops run on never builds it (no throat term in
  `stator_transient.rs` or `reference_split.rs`). It moves rung 54/56's capacity numbers, as § 6
  already says, and nothing downstream.

**Units labelling (a documentation job, not scope):** any number quoted in incidence units —
`m_lim`, `M_i`, a cross-gain per unit incidence — is in NORMALISED (map) units; rung 85 states
this once in its spec, and quotes a physical incidence only as `M_i/Φ_d`.

### 6.2 THE PER-CELL TABLE (P-C' in full) — what § 4 is scored against (2026-10-06)

Conventions (plain arithmetic; reproduces § 6's P-C' summary exactly): `γ` = 1.4, `cp` = 1004;
arithmetic mean radius, `U_tip = U_m·2/(1+h)`; uniform `Vx = Φ_d·U_m`, zero inlet swirl; static
`T = Tt − Vx²/2cp`; the airflow wall solves `M_rel,tip = M_rel,lim` for `U_m`; `K* = Δh(1+l)/U_m²`,
`K = ceil(K*)`, `U_d = sqrt(Δh(1+l)/K)`; `U_cap = sqrt(2(σ_y/ρ)/(1−h²))`, `R = U_cap/(1.2·U_d,tip)`.
`Φ_d` = 0.537, `σ_y/ρ` = 1.8681e5 m²/s², faces at `Tt2` = 286.125 K / `Tt25` = 403.354 K.

| spool | `h` | `M_rel,lim` | `K*` | `K` | `U_tip` wall | `U_tip,d` | `C` at `K` | `M_rel,d` | **`R` at `K`** | `R` at `K*` |
|---|---|---|---|---|---|---|---|---|---|---|
| LP | 0.5 | 1.3 | 2.23 | 3 | 399.5 | 344.3 | 0.649 | 1.114 | **1.708** | 1.472 |
| LP | 0.5 | 1.4 | 1.94 | 2 | 428.6 | 421.7 | 0.761 | 1.376 | **1.395** | 1.372 |
| LP | 0.5 | 1.5 | 1.70 | 2 | 457.5 | 421.7 | 0.761 | 1.376 | **1.395** | 1.286 |
| LP | 0.7 | 1.3 | 1.82 | 2 | 389.7 | 372.1 | 0.761 | 1.238 | **1.917** | 1.830 |
| LP | 0.7 | 1.4 | 1.59 | 2 | 417.8 | 372.1 | 0.761 | 1.238 | **1.917** | 1.707 |
| LP | 0.7 | 1.5 | 1.39 | 2 | 445.6 | 372.1 | 0.761 | 1.238 | **1.917** | 1.601 |
| HP | 0.5 | 1.3 | 4.86 | 5 | 474.3 | 467.7 | 0.723 | 1.281 | **1.258** | 1.240 |
| HP | 0.5 | 1.4 | 4.22 | 5 | 508.9 | 467.7 | 0.723 | 1.281 | **1.258** | 1.156 |
| HP | 0.5 | 1.5 | 3.71 | 4 | 543.2 | 522.9 | 0.785 | 1.441 | **1.125** | 1.083 |
| HP | 0.7 | 1.3 | 3.98 | 4 | 462.7 | 461.4 | 0.785 | 1.296 | **1.546** | 1.542 |
| HP | 0.7 | 1.4 | 3.46 | 4 | 496.1 | 461.4 | 0.785 | 1.296 | **1.546** | 1.438 |
| HP | 0.7 | 1.5 | 3.04 | 4 | 529.1 | 461.4 | 0.785 | 1.296 | **1.546** | 1.348 |

(speeds m/s.) **The staircase in one look (D1):** all three LP `h` = 0.7 levels build ONE machine
(`K` = 2), as do all three HP `h` = 0.7 levels (`K` = 4); the LP `h` = 0.5, 1.3 cell's rounding
to `K` = 3 drops its design tip Mach to 1.114, far below the level it was sized on. The 12
spool-cells collapse to **6 distinct spool machines** (3 per spool), and the 6 (`h`, `M_rel,lim`)
cells to **4 distinct LP/HP pairs**: integer `K`, not the wall level, sets the redline.

### 6.3 THE PER-SHAPE TABLE (§ 4.4 A4) — `K` and `R` at integer `K`, by map slope

§ 6.2's conventions with each shape's own `l` (`rust/tests/rung55.rs` `maps`); the design `Δh` is
shape-free. `flat-eta` has the default slopes, so it IS § 6.2. Cells in (`h`, `M_rel,lim`) order
(0.5, 1.3) (0.5, 1.4) (0.5, 1.5) (0.7, 1.3) (0.7, 1.4) (0.7, 1.5); each entry `K` / `R`.

| shape (`l` LP / HP) | LP | HP |
|---|---|---|
| `flow/press`, `flat-eta` (0.7 / 1.0) | 3/1.708 · 2/1.395 · 2/1.395 · 2/1.917 · 2/1.917 · 2/1.917 | 5/1.258 · 5/1.258 · 4/1.125 · 4/1.546 · 4/1.546 · 4/1.546 |
| `press/flow` (1.0 / 0.7) | 3/1.575 · 3/1.575 · 2/1.286 · 3/2.164 · 2/1.767 · 2/1.767 | 5/1.364 · 4/1.220 · 4/1.220 · 4/1.677 · 3/1.452 · 3/1.452 |
| `tilted` (0.85 / 0.85) | 3/1.637 · 3/1.637 · 2/1.337 · 2/1.837 · 2/1.837 · 2/1.837 | 5/1.308 · 4/1.169 · 4/1.169 · 4/1.607 · 4/1.607 · 3/1.392 |
| `steep` (1.2 / 1.2) | 3/1.502 · 3/1.502 · 3/1.502 · 3/2.064 · 3/2.064 · 2/1.685 | 6/1.313 · 5/1.199 · 5/1.199 · 5/1.648 · 4/1.474 · 4/1.474 |

A steeper loading slope `l` lowers the per-row work coefficient `1/(1+l)`, so it needs MORE rows
at the same wall — and the integer staircase then moves `R` in both directions. The tightest cell
anywhere is still the default HP's 1.125.

## 5. Concessions already known

- The strength law is the UNTAPERED blade-root pull `σ = ρ·U_tip²(1−h²)/2`. Real tip-speed limits
  usually come from the DISC, and from temperature-dependent strength; neither is modelled. The
  hot rear rows' material switch (titanium → nickel) is out of scope: both walls are applied at
  the cold front row only.
- **The strength wall is an OPTIMISTIC upper bound** (no disc, no temperature, yield rather than
  the no-burst rule's ultimate — the latter would lift the redline ~4 %; yield vs ultimate is a
  disclosed choice). So its verdicts are ONE-SIDED: "crosses the redline" survives the model's
  error; "stays under it" does not, and is worded as such.
- `φ_d` = 1 forces `Vx = U`, which raises the relative Mach for a given `U` relative to real
  designs (Rotor 37 runs `U_tip` 454 m/s at `M_rel` 1.49; this sizing hits 1.5 near 377–389 m/s).
