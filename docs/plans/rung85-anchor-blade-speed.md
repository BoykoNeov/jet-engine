# Rung 85 anchor — THE BLADE-SPEED WALLS (DRAFT — pre-registration not yet closed)

**Seam:** `docs/per-row-blading-negative.md` § 5 — *"an anchor supplied by physics rather than by
choice … a row-level stress or tip-Mach limit that pins `U` from outside the stack"*; carried in
`CLAUDE.md`'s OPEN list as *"An ANCHOR for the blading"*.

**The user's choice (2026-10-06):** BOTH walls — an AIRFLOW wall (front-row relative tip Mach) and
a STRENGTH wall (centrifugal blade-root stress) — with whichever binds winning, per spool.

**Status:** § 0 (pre-check probes) and § 1 (sources) are DONE. § 2 (the knob table) is written.
§ 3 (open design decisions) and § 4 (predictions) were revised after the FIRST advisor check-in
(2026-10-06); § 0 P-C BLOCKED the design; the user chose `Φ_d` as a design input — see § 6. No code has been written.
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

## 3. OPEN DESIGN DECISIONS (to settle at the advisor check-in)

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
- **D4 — which experiment shows the crossover.** "Which wall SETS the design" is a DESIGN-SPACE
  question (engines designed at different flight `M0`); "which wall does an OVERSPEED hit first" is
  an OFF-DESIGN question on one engine. Proposal: the off-design one is the rung (it is where
  rungs 53–61's speed bills live); the design-space crossover is reported as § 0's probe only.
  Off design, the airflow wall reads CORRECTED speed/flow (`n`, `m`) and the strength wall reads
  PHYSICAL speed `N/N_d = n·sqrt(θ₂/θ₂,d)` — each spool its own pair.
- **D5** — retracted; see § 0 (true by construction).
- **D6 — the panel.** Rung 85 is the first rung with no Python segment in `cli_golden`. It needs a
  Rust-owned golden and a stated meaning for the panel count. Its own commit, before the physics.

## 4. PREDICTIONS (DRAFT — not registered until P-C is resolved)

- **Q4 (re-posed)** — a LEVEL against a LEVEL, not a cost against a level. On the machine the walls
  produce (its own `K`, re-measured — NOT rung 53/55's published `+66.7 %` / `+2.3 %`, which are
  part-power CHANGES at `K` = 8, and rung 55's cost scales ~`1/K`, so ~2.7× larger at LP `K` ≈ 3):
  read `N_L/N_L,d` WITH the lever at each cell, against `N_red/N_d`. Prediction: the lumped stator
  crosses the strength redline somewhere in the shipped throttle range; the front-row stator
  does not.
- **Q5 (re-posed, two-sided)** — the stator's setting `v` adds face pre-swirl, `tan β₁ = 1/φ − v`,
  which LOWERS the rotor's relative inlet Mach (the purpose of a real IGV). So under a stator lever
  physical `N` can rise while `M_rel,tip` FALLS. The off-design tip Mach is computed from
  `(φ, v, n)` through the stack's own `β₁`. Prediction to be written only after that function
  exists and its design value is checked to equal the wall level.
- **NEGATIVE decision rule (fixed now):** if the walls re-derive `K` and `C` but NO verdict of rungs
  53–61 changes, and strength never binds inside the shipped envelope, this is a NEGATIVE
  (`docs/…-negative.md`), not a rung.

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

**Still open:** re-running P-C's capacity table at `Φ_d` = 0.54 (§ 6's P-C' gives the summary
band; the full per-cell table is not yet written).

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
