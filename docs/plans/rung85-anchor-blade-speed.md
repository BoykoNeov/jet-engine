# Rung 85 anchor — THE BLADE-SPEED WALLS (DRAFT — pre-registration not yet closed)

**Seam:** `docs/per-row-blading-negative.md` § 5 — *"an anchor supplied by physics rather than by
choice … a row-level stress or tip-Mach limit that pins `U` from outside the stack"*; carried in
`CLAUDE.md`'s OPEN list as *"An ANCHOR for the blading"*.

**The user's choice (2026-10-06):** BOTH walls — an AIRFLOW wall (front-row relative tip Mach) and
a STRENGTH wall (centrifugal blade-root stress) — with whichever binds winning, per spool.

**Status:** § 0 (pre-check probes) and § 1 (sources) are DONE. § 2 (the knob table) is written.
§ 3 (open design decisions) and § 4 (predictions) are DRAFTS awaiting the advisor check-in. **No
code has been written.** Probes: plain arithmetic on the shipped design numbers, no model code.

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

## 1. SOURCES — verified, with what each does and does NOT supply

- **NASA TP 1659** (Moore & Reid, 1980), *Performance of Single-Stage Axial-Flow Transonic
  Compressor With Rotor and Stator Aspect Ratios of 1.19 and 1.26…*, read first-hand
  (ntrs.nasa.gov/api/citations/19800012840): stage 37, rotor tip speed **454 m/s**, rotor-inlet
  relative Mach **1.493 at the tip** to 1.125 at the hub. Its parent design (its ref. 1): an
  **eight-stage 20:1 core compressor, constant meanline diameter, inlet hub-tip 0.7, inlet
  rotor-tip speed 455 m/s** — explicitly "considerably higher than state-of-the-art" (1980).
  *Supplies:* an UPPER, aggressive example of both the tip Mach and `h`. *Does not supply:* a limit.
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
- **Ti-6Al-4V, mill-annealed:** minimum specified yield **827 MPa** (120 ksi), ultimate 896 MPa
  (130 ksi) — quoted from AMS-spec minima in secondary sources; density 4430 kg/m³ (standard).
  **Still owed:** a first-hand datasheet or handbook read (ASM/MMPDS) before any number ships.
- **Still owed:** a TEXTBOOK range for the relative-tip-Mach wall (Mattingly / Cumpsty /
  Saravanamuttoo / Dixon). Two NASA research rotors are design examples, not a limit.

## 2. THE KNOB TABLE — inputs and outputs, per spool

| quantity | rungs 55/56 | rung 85 (walls on) |
|---|---|---|
| cycle `Δh_spool` | output of the design run | unchanged |
| map slope `l` ⇒ `t₂` | input (map shape) | unchanged |
| design sizing `φ_d` = 1 | input | unchanged (disclosed: it forces `Vx = U`) |
| hub-to-tip `h` (front row) | absent | **NEW, disclosed** — ONE shared constant, both walls |
| airflow wall `M_rel,lim` | absent | **NEW, disclosed level**, verdicts as thresholds |
| strength `σ_y/ρ` | absent | **NEW, sourced** (Ti-6Al-4V min spec) |
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
- **D5 — which ROW binds.** Both walls are expected at each spool's FRONT row (coldest, longest
  blade). To be MEASURED per row, not assumed — especially on the HP spool.
- **D6 — the panel.** Rung 85 is the first rung with no Python segment in `cli_golden`. It needs a
  Rust-owned golden and a stated meaning for the panel count. Its own commit, before the physics.

## 4. PREDICTIONS (DRAFT — not yet registered; scored only once closed)

- **Q1** `K` out of the walls at `M0` = 0.85: LP **2–4**, HP **6–9** (from § 0's arithmetic).
- **Q2** the airflow wall binds the design on BOTH spools at `M0` = 0.85 (§ 0 P-B).
- **Q3** redline headroom `N_red/N_d − 1` is LARGE (tens of percent) on both spools.
- **Q4** rung 53's lumped stator overspeed (`+66.7 %` `N_L` at `Tt4` = 1000) CROSSES the LP
  strength redline; rung 55's front-row stator (`+2.3 %`) does NOT — the positional law (rung 55)
  survives as an ADMISSIBILITY result, not just a cost result.
- **Q5** off design, the AIRFLOW wall is reached before the strength wall in every overspeed the
  shipped transients produce (because it is the design-binding one and both scale with `N`).

## 5. Concessions already known

- The strength law is the UNTAPERED blade-root pull `σ = ρ·U_tip²(1−h²)/2`. Real tip-speed limits
  usually come from the DISC, and from temperature-dependent strength; neither is modelled. The
  hot rear rows' material switch (titanium → nickel) is out of scope: both walls are applied at
  the cold front row only.
- `φ_d` = 1 forces `Vx = U`, which raises the relative Mach for a given `U` relative to real
  designs (Rotor 37 runs `U_tip` 454 m/s at `M_rel` 1.49; this sizing hits 1.5 near 377–389 m/s).
