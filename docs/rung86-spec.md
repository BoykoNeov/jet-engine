# Rung 86 — FUEL + BLEED + STATOR on one plant: the three-lever cube

**Code:** `rust/src/three_levers.rs` · **Gates:** `rust/tests/rung86.rs` · **Panel:**
`print_three_levers_table` (born in Rust, held to its own capture `rust/oracle/rust_owned/`, a change
detector only) · **Anchor (derivations, every pre-registered bar, committed `abe7c5a` before any was
measured):** `docs/plans/rung86-anchor-three-levers.md` — § refs below are to it · **Sandbox:** the
Controls view's lever choice "both" (`docs/plans/sandbox-plan.md` § 15).

**Shipped as KNOBS** (the sandbox direction): every result is shown; predictions were registered
first and are scored here, a miss being a finding.

Rung 63 closed by naming this rung — *"**fuel + bleed + stator**, all three on one plant (this rung
held to two, since a third lever breaks the second difference's isolation)"* — and the sandbox
refused the combination as open physics.

> **THE HEADLINE — a third lever adds no new coupling, only a new CLOCK.** The stator and bleed
> interact (rung 62's pair), and they move the incidence-margin minimum LATER and HOTTER, the pair
> latest. A fuel limiter then decides only WHEN that pair interaction is read: the acceleration
> schedule reads it EARLIER, before it has built up, so the three-way term is a fraction of the
> pair (7–16 %) with the opposite sign, and rung 58's predictor — built from runs that never saw
> the limiter — recovers 71–93 % of it; the turbine limiter reads it only on the runs whose minimum
> is hotter than its redline; a floor that pins the currency replaces it by its set point. So the
> three-way term is computable from the limiter-free pair plus the limiter's timing — **rung 63's
> worry was the instrument, not the plant: the eight-cell cube restores the isolation.**
>
> **AND THE KNOB FINDING, which the page now shows:** a stall floor read as a FLOW COEFFICIENT
> fights the stators (they lower the flow coefficient while raising the margin to the metal), is
> violated from the first instant, and cuts fuel until the run starves — down to no run at all at
> the top set point. The bleed lifts the flow coefficient back and rescues it.

## 1. What it adds — an instrument, not plant code (§ 0 P1)

The plant existed: rung 62's machine carries both schedules, rung 63's readers already took a
`neighbour` lever and a fuel leg. What was missing was the reading. Three switches — `S` (LP stator
schedule, 0.20 below 0.65), `B` (bleed schedule, 0.10 below 0.65), `F` (ONE fuel limiter) — make
eight cells, each the LP incidence-margin minimum `M_i` (rung 58's currency: its wall is the metal,
one number in every cell). The three-way term

```
    T = I_SB|F=1 − I_SB|F=0 = I_BF|S=1 − I_BF|S=0 = I_SF|B=1 − I_SF|B=0
```

is what exists only with all three on (its three spellings agree to 1e-15 — arithmetic, gated, not
a finding). Every fuel limiter is a `Limiter`: the acceleration schedule (48) and its realistic
release lag (52), the turbine limiter (46) and its lag (47), the incidence floor (60) and the `φ`
floor (49). The two lags are not `StatorLeg` fields, so the cube marches through `ramp_march` —
rung 57's own march body, split out — after the start-of-march map reset.

**Disciplines (§ 1):** every cell a fresh sibling of the BARE machine, its arming asserted; one leg
object derived once on the bare sibling; every cell's point count kept, and a SUM compared only
between equal-length runs; a run with no points is a result (`T` = NaN), never a crash.

## 2. Reduces

Each two-lever face IS the earlier rung's own reader, bit for bit (`tests/rung86.rs` R1–R5): the
stator-off face is rung 63's `lever_composite` (its published +0.007658 reproduced), the
stator-on face the same with the stator as `neighbour`, the bleed-off face rung 58's
`composite_credit`, the limiter-free face the four leg-free siblings (rung 62's pair); the cube's
march with a plain limiter IS `stator_march`. No model code path changed: `ramp_march` and
`cell_of` are splits of existing bodies, so every oracle stays bit for bit.

## 3. Scoring (§ 3 — bars frozen at `abe7c5a`)

Six cells: shaped / tilted maps × `r` ∈ {0.25, 0.5, 1.0}; floors at six set points each; every
scored `T` claim also at `ds` = 0.0025 (the step-size bar).

| # | verdict | measured |
|---|---|---|
| A1 | **HIT** (5 scored cells + the seen one) | `T > 0`, `I_SB < 0`, `T/−I_SB` = 0.076–0.155, both steps |
| A2 | **HIT** | `r` = 1: `T` = 1.75–1.85e−3 with the bare cell unreached (at `ds` = 0.0025). The bare-cell equality (no credit) is NOT grid-robust on the tilted map: at 0.005 the leg's engagement falls inside the refine stencil |
| A3 | **HIT** | rung 58's predictor across the stator faces recovers 0.71–0.93 of `T`, both steps |
| A4 | **HIT** | the release lag keeps the sign, `T_lag/T` = 0.71–0.90 |
| step bar, A | **HIT** | `T` moves < 3 % between steps in every cell |
| B1 iff | **HIT**, 48/48 readings | a lever cell is reached iff its leg-free run passes 1480 K by the last point of its minimum's stencil |
| B1 "bit-for-bit its leg-free self" | **MISS as written** | the MINIMUM's reading (`M_i`, its time, `v`) is bit-for-bit; the RUN is not — the limiter still trims the later, hotter fuel. The gate was narrowed to the reading AFTER measuring (no credit for the narrowed form) |
| B2 | **HIT** (`r` = 1: `T` = 0.0 exactly; `r` = 0.25: all four reached; `r` = 0.5: SB reached) — its conditional clause **NOT MET** | at `r` = 0.5 the bleed-alone cell is reached too, so it is not "a three-lever effect where each lever alone shows nothing" |
| B3 | **HIT** | the lagged limiter reaches exactly the cells the instant one does |
| step bar, B | met, but the INSTANT limiter's size is **GRID-SENSITIVE** | its `T` moves 25–34 % between steps, and `I_BF|S=0` changes sign at `r` = 0.5; the lagged limiter's `T` is stable to < 1 % |
| C1 | **HIT** | the incidence floor's whole legged face is `max(c_xy0, m_lim)` to machine precision, every cell, every set point; dormant cells withhold exactly zero fuel over the whole run. Its erasure of `I_SB` above all four minima is rung 60's pinning — no credit |
| C2 | **HIT** stator off, **MISS** stator on | binds iff leg-free `min φ` < `φ_lim`, unbound cells untouched (whole run), everywhere. Stator off: the identity holds to 6e−8. Stator on: the incidence minimum is NOT held — it slides to the end of the held stretch (identity off by up to 5e−4), and at `r` = 1 tilted it sits where `φ` is 1.2e−3 above the floor, at both steps |
| C3 | **HIT**, 64/64 | as fractions of each run's own scheduled fuel: each lever makes the incidence floor withhold less (56 comparisons); the stator makes the `φ` floor withhold more (8). **Only 8 `φ` comparisons survive the equal-length filter** — in all 64 excluded pairs the stator-on run is the SHORTER one (it starved), which is the claim's strongest case and is gated separately |
| step bar, C | met except **one GRID-SENSITIVE cell** | shaped `r` = 0.5, `φ` floor `sm` 0.30: `T` 1.3e−5 → 4.6e−4 (a near-zero term). Every other floor `T` within 6 % |

**No credit, as registered:** A2's bare-cell equality, B2's bare cell at `r` = 0.5 (rung 46), C1's
erasure (rung 60), C2's twin clause (decided by P2's gaps).

## 4. The findings, in order of robustness

1. **The acceleration schedule re-reads the pair earlier** (A1/A3/A4). Every lever moves the
   minimum later; the schedule pulls it back to where the stator–bleed interaction has not yet
   built up, so the three-way term erases 8–16 % of the pair and is predicted from limiter-free
   runs. Grid-stable.
2. **The turbine limiter is a ledger of hot minima** (B1/B3). Whether it touches a cell's margin
   minimum is decided by that cell's temperature there, which the levers raise by moving the
   minimum later. The lagged limiter's three-way term is grid-stable; the instant one's SIZE is not.
3. **The incidence floor is a ledger of its set point** (C1) — exactly, from four limiter-free cells.
4. **The `φ` floor fights the stators and can starve the engine; the bleed rescues it** (C2/C3 and
   `a_phi_floor_beside_the_stator_starves_the_engine_and_the_bleed_rescues_it`). The rung-53/58
   coordinate warning, now as a fuel bill and a stopped run.

## What it does to its neighbours

* **Rung 63 — its seam CLOSED, its stated obstacle answered.** A third lever does not break the
  isolation once the cube has all eight cells; each face is rung 63's own reader.
* **Rung 58 — its mechanism EXTENDED to three levers:** the relocation predictor, differenced across
  the stator faces, recovers the three-way term (71–93 %).
* **Rung 46 — its sequencing law reappears as a ledger**, cell by cell, with the levers deciding
  which cells' minima are hot enough.
* **Rung 60 — reproduced** on the whole legged face of the incidence floor.
* **Rungs 53/58 — the `φ`-coordinate warning made concrete:** the `φ` floor starves the statored
  engine.

## Concessions

* ONE placement (`n_lo` = 0.65 for both schedules, rungs 62/63's), the LP currency only, rung 36's
  imposed `φ_surge`, invented maps — inherited from rungs 57–63.
* `M_i` is a parabola-refined minimum, so "reached" is read through its stencil (the A2 and B
  artefacts above are that stencil, disclosed).
* The instant turbine limiter's three-way SIZE is grid-sensitive; only its sign, order and reach
  ledger are claimed.
* The sandbox's "both" choice is one select option, not two independent switches (anchor § 4
  said switches): functionally the same, and every saved setting stays valid.

## The next seam

The cube measures; it does not compose a CONTROLLED valve. A **φ-referenced bleed limiter beside a
stator schedule and a fuel leg** (rungs 64–68's loops on this plant), and the **station-3 customer
bleed** — a different sink into the same two sensed inputs.
