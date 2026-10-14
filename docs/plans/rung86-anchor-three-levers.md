# Rung 86 anchor — FUEL + BLEED + STATOR on one plant (pre-registered 2026-10-14 — § 3)

**Seam:** rung 63 § *The next seam* — *"**fuel + bleed + stator**, all three on one plant (this rung
held to two, since a third lever breaks the second difference's isolation)"*; carried in
`CLAUDE.md`'s OPEN list as *"Fuel + bleed + STATOR on one plant"*, and refused by the sandbox's
Controls view as open physics (`docs/plans/sandbox-plan.md` § 12.3).

**The user's choices (2026-10-14):** (1) **every** fuel limiter, not only the one rung 63 found
composes — the acceleration schedule (48) and its realistic release lag (52), the turbine
temperature limiter (46) and its lag (47), the stall floor read as a flow coefficient (49) and
as an incidence (60); (2) both schedules placed at `n_lo` = 0.65 (rungs 62/63's placement);
(3) the sandbox's Controls view gets the combined lever in the same batch. Shipped as KNOBS
(the sandbox direction): every prediction below is scored HIT/MISS, and a miss is a finding.

**Status:** § 0 measured; § 1–§ 3 written BEFORE any of § 3's quantities was measured, except
the one cell § 0 (P1) discloses. No model code exists for this rung yet.

---

## 0. WHAT EXISTS — measured before anything was written (scratch crate, not in the repo)

**P1 — the plant is already built.** Rung 63's readers take a `neighbour` lever and a fuel leg.
`lever_composite(bled, accel leg, LP, neighbour)` with `neighbour = None` and
`= Some(LP stator schedule)` runs all eight on/off cells to the end (341 points each, 0.4 s per
four cells), every march through `stator_march_scoped` and so through today's start-of-march map
reset (`reset_built_maps`). **The rung is therefore an INSTRUMENT and KNOBS, not plant code.**

**P1 also SAW THE HEADLINE QUANTITY at ONE cell** (shaped maps, `r` = 0.5, accel leg) before this
file existed, and that cell is excluded from every bar in § 3 (no credit):

| stator | neither | bleed | fuel | both | `I_BF` |
|---|---|---|---|---|---|
| off | 0.4584523 | 0.5502227 | 0.4735210 | 0.5729499 | +7.658485e−3 (rung 63's published +0.007658, reproduced) |
| on | 0.4962544 | 0.5631320 | 0.5154940 | 0.5929605 | +1.058892e−2 |

so `T` = +2.930e−3 there, and the stator–bleed pair (leg-free) is `I_SB` = −2.489e−2.

**P2 — the LEG-FREE lever cells**, measured as instrument inputs (the turbine limiter's redline
must bite; the floors' set points must span the cells): `M_i` minimum, its time, the run's peak
`Tt4`, `min φ_LP`, all 24 (shape × `r` × lever) cells, every one full-length:

| shape | `r` | cell | `M_i` | `s_min` | peak `Tt4` | `min φ` |
|---|---|---|---|---|---|---|
| shaped | 0.25 | none / S / B / SB | 0.41323 / 0.44967 / 0.50354 / 0.51437 | 0.191 / 0.203 / 0.224 / 0.238 | 1823 / 1811 / 1856 / 1833 | 0.7118 / 0.6804 / 0.7607 / 0.7112 |
| shaped | 0.50 | | 0.45845 / 0.49625 / 0.55022 / 0.56313 | 0.234 / 0.253 / 0.285 / 0.311 | 1695 / 1683 / 1720 / 1698 | 0.7354 / 0.6999 / 0.7887 / 0.7335 |
| shaped | 1.00 | | 0.48691 / 0.52632 / 0.58075 / 0.59595 | 0.275 / 0.306 / 0.349 / 0.395 | 1567 / 1556 / 1581 / 1563 | 0.7512 / 0.7125 / 0.8081 / 0.7483 |
| tilted | 0.25 | | 0.41030 / 0.44865 / 0.49878 / 0.51139 | 0.196 / 0.207 / 0.226 / 0.238 | 1839 / 1823 / 1874 / 1847 | 0.7103 / 0.6786 / 0.7579 / 0.7083 |
| tilted | 0.50 | | 0.46202 / 0.50099 / 0.55187 / 0.56584 | 0.241 / 0.258 / 0.287 / 0.310 | 1702 / 1688 / 1729 / 1704 | 0.7374 / 0.7008 / 0.7897 / 0.7334 |
| tilted | 1.00 | | 0.49399 / 0.53415 / 0.58582 / 0.60164 | 0.284 / 0.311 / 0.351 / 0.392 | 1569 / 1557 / 1584 / 1565 | 0.7552 / 0.7148 / 0.8115 / 0.7499 |

Read off it before § 3, as plain observation: **the stator LOWERS `min φ` while RAISING `M_i`**
(rung 53's wall-move — the margin to the metal grows while the flow coefficient falls), the bleed
raises both, and **every lever moves the minimum LATER**, the pair latest (+0.04 to +0.12 τ).

## 1. THE INSTRUMENT — the cube

Three switches on one machine: `S` (rung 57's LP stator schedule, `v_max` 0.20 below `n_lo`
0.65), `B` (rung 62's bleed schedule, `b_max` 0.10 below 0.65), `F` (ONE fuel limiter). Eight
cells `c_SBF`, each the LP incidence-margin minimum `M_i` (rung 58's currency: its wall is the
METAL, one number in all eight cells, so differencing crosses no moving wall).

```
    I_SB|F=f  =  c_11f − c_01f − c_10f + c_00f          (and I_SF|B, I_BF|S alike)
    T         =  I_SB|F=1 − I_SB|F=0  =  I_BF|S=1 − I_BF|S=0  =  I_SF|B=1 − I_SF|B=0
```

`T` is the THREE-WAY term — what exists only when all three are on. The three spellings are one
alternating sum over the cube; their agreement is ARITHMETIC (gated at ~1e−15), not a finding.
**The two-lever faces are earlier rungs' instruments:** `I_BF|S=0` is rung 63's
`lever_composite`, `I_SF|B=0` rung 58's `composite_credit`, `I_SB|F=0` rung 62's leg-free pair.

**Disciplines, each from a trap this ladder already fell into:**
- Every cell a FRESH `at_lever` sibling, its arming ASSERTED (stator scheduled or not, valve
  armed or not, leg on or off) — rung 63 § 0: an inherited reader can hand back an identical
  sibling and report a perfect zero.
- ONE leg object, derived once on the BARE machine, passed to all four legged cells (rung 58).
- Every cell reports its point count. `M_i` is a minimum over the run, so a cell that stops
  early is flagged; any SUM (`removed` fuel) is compared only between cells of equal length
  (the rung-65 lesson, 2026-10-14).
- Lagged limiters (47's governor clock, 52's attack/release) are not `StatorLeg` fields, so the
  rung's march is rung 57's ramp spelled out with the full `FuelLimiters`; with a plain leg it
  must equal `stator_march` bit for bit (reduce R5).

**Grid.** Rung 63's rig: shaped (rung 63's LP/HP maps) and tilted; `r` ∈ {0.25, 0.50, 1.00};
`Tt4` 1000 → 1400, settle 1.2, `ds` 0.005, `ρ` = 1, LP currency. Limiters: accel schedule (margin
0.25, 13 rows — rung 63's); its release lag (attack 0.02, release 0.10 — rung 52's, the Controls
default); the turbine limiter at 1480 K (rung 46's) and with its 0.2 τ lag (47's); the incidence
floor and the `φ` floor, each at `sm` ∈ {0.30, 0.34, 0.38, 0.42, 0.46, 0.50} above the design
stall line (`φ_lim` = 0.55 (1 + `sm`); the incidence set point is that `φ` at the design setting,
`m_lim` = `T_c` · `sm`/(1 + `sm`) = 0.420 … 0.606). The set points were chosen to straddle P2's
`M_i` column — an instrument choice made on leg-free data, disclosed.

## 2. DERIVATIONS — what each kind of limiter can do to the cube

**A — a leg that CUTS without pinning (the accel schedule, with or without its lag).** Rung 58's
mechanism: the leg does not change a lever's credit PROFILE in time, it changes WHICH POINT of it
is the minimum (it relocates the minimum earlier). So `I_BF|S` is the bleed's leg-free credit
profile read at the relocated minimum, on whichever stator face. Rung 62 found the stator–bleed
pair interaction is a LOOP that builds over the ramp (the bleed's amplification of the stator's
surrender). P2 shows every lever moves the minimum later and the pair latest — so the pair's
interaction is read at its latest, most-built-up point. A leg that pulls the reading EARLIER reads
it before it has built up: **`I_SB|F` should be SMALLER in size than `I_SB|F=0`, same sign** — so
`T` has the sign of `−I_SB|F=0`, and cannot exceed it in size (the leg cannot read the
interaction before it began). Rung 48's law, cell by cell: a leg that engages AFTER a cell's
minimum leaves that cell bit-for-bit its leg-free self.

**B — a leg that trims LATE (the turbine limiter, with or without its lag).** Rung 46: the
governor cuts only once `Tt4` passes the redline, so it reaches a cell's `M_i` minimum only if that
minimum sits at a `Tt4` above the redline; otherwise the cell is bit-for-bit leg-free. The levers
move the minimum LATER (P2), i.e. to a hotter point of the ramp, and the bleed burns richer — so
lever cells can be reached where the bare cell is not. **`T` is then a LEDGER of which of the four
lever cells the governor reaches**, and a lag only delays the cut (rung 47: trailing edge), so it
can reach no cell the instant governor misses.

**C — a leg that PINS (the floors).** Rung 60: a floor that binds holds its own coordinate at the
set point. The incidence floor's coordinate IS the currency: a cell it reaches reads exactly
`m_lim`, a cell it does not is bit-for-bit leg-free. So the whole legged face is
`c_xy1 = max(c_xy0, m_lim)` — computable from the four leg-free cells and the set point alone —
and above all four (`m_lim` > every `c_xy0`) **the floor erases the stator–bleed interaction
entirely**: `I_SB|F=1 = 0`, `T = −I_SB|F=0`. The `φ` floor pins `φ`, not `M_i`; with the stator
on, `M_i = T_c − 1/φ_lim + v` at a pinned point, so a binding cell reads `T_c − 1/φ_lim + v_at_min`
(rung 60's "= v"), and it BINDS by `min φ` — which the stator LOWERS (P2). **So the `φ` floor
binds MORE on stator cells — it fights the stator — while the incidence floor binds LESS.**
In fuel: a floor withholds fuel to hold its set point, so a lever that lifts the leg-free minimum
in the floor's own coordinate lets it withhold less.

## 3. PREDICTIONS — registered before measuring (bars frozen here)

Cells: 6 (shape × `r`). **"Reached"** = the legged cell's `M_i` or its time `s` is not
bit-equal to its leg-free self — read off the instrument itself, because `refine_min` fits a
parabola through the grid minimum AND its two neighbours, so a limiter acting anywhere in that
stencil moves the reading (advisor, pre-commit).

**THE STEP-SIZE BAR (applies to every A verdict, and to B's/C's `T` where nonzero).** `T` is a
difference of two interactions each ~3× its size — rung 63 § 1's shape, where a halved `ds` moved
the shares 11–12 % and they were withdrawn. So each scored `T` claim must hold at `ds` = 0.0025 as
well as 0.005: same sign, same order of magnitude, and A1's / A3's ratios still inside their
bars. A claim that holds at one step only is scored **GRID-SENSITIVE**, not HIT.

**No credit (settled before this file, kept as gates, never scored):** A2's bare-cell equality
at `r` = 1 (rung 63 gates `s_eng` > the bare minimum on this rig, and P2 gives the minimum); B2's
"bare cell not reached at `r` = 0.5" (rung 46's published 1374 K minimum on `flow/press`, which
IS this rig's shaped map); C1's erasure of `I_SB` above all four minima (rung 60's pinning,
reproduced — a confirmation, not content); C2's "binds on a stator cell and not its twin" (P2's
`min φ` gaps between B and SB, ~0.05, are wider than the `φ_lim` grid spacing 0.022 in every
cell, so it cannot miss).

**A — accel schedule (margin 0.25).**
- **A1 (sign + bound).** In every cell where `|T|` > 1e−5: `T`·`I_SB|F=0` < 0 and
  `|T|` < `|I_SB|F=0|`. Scored on the 5 cells P1 did not see.
- **A2 (rung 48 cell by cell).** At `r` = 1.00, both shapes: `T` ≠ 0 — at least one lever cell,
  whose minimum P2 puts later, is reached although the bare cell is not (the bare-cell equality
  is no credit, above).
- **A3 (mechanism).** Rung 58's predictor on both stator faces — each face's leg-free bleed credit
  profile read at its legged vs leg-free minimum, differenced across faces — recovers `T` with
  ratio in [0.5, 1.5] in every cell where `|T|` > 1e−4.
- **A4 (release lag 0.02 / 0.10).** Same sign as A's `T` in every cell where A's `|T|` > 1e−4,
  ratio `T_lag/T` in [0.5, 1.5].

**B — turbine limiter (1480 K).**
- **B1 (ledger).** In every cell, a lever cell is reached IFF the leg-free run's `Tt4` exceeds
  1480 K at some point up to and including the LAST point of its `M_i` stencil (the grid minimum's
  later neighbour); an unreached cell is bit-for-bit its leg-free self.
- **B2 (where).** `r` = 1.00: no cell reached ⇒ `T` = 0.0 exactly, both shapes. `r` = 0.50: the
  SB cell IS reached ⇒ `T` ≠ 0 — and if the S and B cells are not, this is a three-lever effect
  at a ramp rate where each lever alone (with the limiter) shows nothing on the LP spool.
  `r` = 0.25: all four reached (rung 46's fast-ramp limit).
- **B3 (lag 0.2 τ).** Every cell unreached by the instant limiter is unreached by the lagged one.

**C — floors, `sm` ∈ {0.30 … 0.50}.**
- **C1 (incidence ledger).** Every legged cell `|c_xy1 − max(c_xy0, m_lim)|` < 1e−6, every `sm`,
  every cell; hence `T` = the ledger value to 2e−6.
- **C2 (`φ` ledger), two parts.** A cell binds IFF its leg-free `min φ` < `φ_lim`; an unbound cell
  is bit-for-bit leg-free. On a binding cell: (a) `φ` AT the `M_i` minimum equals `φ_lim` to 1e−6
  — the minimum sits on the pinned stretch — and (b) the minimum reads `T_c − 1/φ_lim + v_at_min`
  to 1e−6. (a) can fail with the stator on: `v` falls as speed rises, so a later point with
  `φ` > `φ_lim` and a smaller `v` can read lower. A miss on (a) is the finding that the `φ` floor
  does not hold the incidence minimum.
- **C3 (fuel).** Between equal-length legged cells that both bind, withheld fuel taken as a
  FRACTION of that cell's own scheduled fuel (each machine's ramp is re-derived to the same `Tt4`
  endpoints, so the bled and statored machines are scheduled different fuel — rung 63 § 2's
  third channel, the one that reversed its sign): the incidence floor withholds a SMALLER
  fraction with each lever on than off; the `φ` floor withholds a LARGER fraction with the
  stator on than off.

**Reduces (gates, not predictions):** R1 the stator-off face ≡ rung 63's `lever_composite`
cells; R2 the stator-on face ≡ the same with `neighbour` = the stator; R3 the bleed-off face ≡
rung 58's `composite_credit` cells; R4 the leg-free face ≡ the four `at_lever` siblings' leg-free
cells; R5 the rung's march with a plain leg ≡ `stator_march`; all bit for bit. The default design
run stays bit-for-bit rung 6 (no model code changes).

## 4. THE KNOBS (sandbox, same batch)

The Controls view's "ONE airflow lever" becomes two independent switches — stator schedule and
bleed schedule — each with its existing numbers, and every fuel limiter allowed beside them;
the "open physics" refusal goes. Nothing else on the page changes.
