# Rung 89 — the ASYMMETRIC valve: which edge decides who is cheaper?

**Code:** the device in `rust/src/limited_bleed.rs` (`BleedLimiter::tau_close`, `closing`, `lag_rate`,
`tau_min`; `r64_solve_b` refuses it) and `rust/src/lagged_bleed.rs` (rung 65's march reads the edge's
constant; `first_closing_read`); the reading in `rust/src/asymmetric_valve.rs` (`asym_valve`,
`matched_asym`, `march_marked`, `phi_floor`) and `lagged_incidence.rs` (`LagRead::rise`/`fall`, the
generalised identity, `matched_with`) · **Gates:** `rust/tests/rung89.rs` · **Panel:**
`print_asymmetric_valve_table` (born in Rust, held to its own capture `rust/oracle/rust_owned/`, a change
detector only) · **Anchor (givens, derivations, every pre-registered bar, committed `43ae914` before any
was measured):** `docs/plans/rung89-anchor-asymmetric-valve.md` — § refs below are to it · **Sandbox:** the
Controls view's slow valve, "Closes at its own speed" (`docs/plans/sandbox-plan.md` § 18) · **Scratch
scripts and the scored grid:** `docs/plans/scratch-archive/rung89/`.

**Shipped as KNOBS** (the sandbox direction): every result is shown; predictions were registered first
and are scored here, a miss being a finding.

Rung 88 closed by naming this seam: *"`τ_open ≠ τ_close` acts on the two halves § 5 separates — the
incidence valve's loss lives on the opening edge, the `φ` valve's gain on the closing edge — so a
fast-opening, slow-closing valve should move the crossover, and which way is a prediction to register."*
Rung 65 had named the asymmetric valve first, as its own concession: its lag is symmetric.

> **THE HEADLINE — each valve is priced by ONE edge.** Re-tuned so both hold rung 87's incidence margin
> exactly, the incidence valve's set point is its OPENING speed's: its margin bottoms out before it ever
> starts closing, so its set point ignores the closing speed (to 1e-9 in all 380 pairs, both steps) — the closing edge
> only adds or saves the air it holds on the way down, `(τ_close − τ_open)·D` (D1′). The flow-coefficient
> valve's margin bottoms out AFTER it starts closing (56 of 95 rows), so its set point is its CLOSING
> speed's: across opening speeds at a fixed closing one it moves ≤ 0.001 sm, across closing speeds up to
> 0.073 — falling as the close slows, down to a FLOOR it reaches once the minimum comes first.
>
> **So the closing speed moves the verdict BOTH ways.** A slow close (`τ_c` 0.2) makes the `φ` valve the
> cheaper one at EVERY opening speed in all 19 cells (0 of 95 runs left to the incidence valve; rung 88's
> symmetric valve leaves it 40). A fast close (`τ_c` 0.01) hands the incidence valve 68 of 95 — the slow
> openings too. I registered the first direction (H5a, 40 of 40) and predicted the second would NOT happen
> (H5c: missed in 51 of 190, the same 51 at both steps). Holding the OPENING speed at its fastest or
> slowest instead leaves the incidence valve 54 and 11 of 95: **the closing speed is the stronger dial**,
> and in every row and every column of every cell the verdict changes side at most once — a staircase
> boundary in the plane of the two speeds.

## 1. What it adds

**The device (anchor § 1).** No new law and no new state: rung 65's first-order lag with a SECOND time
constant. `lag_rate(cmd, q)` chases the command at `τ` (opening) while the command is at or above the
position, at `tau_close` below it; `tau_close = None` is rung 65's valve, bit for bit. `closing(τc)` needs
a lagged valve and `τc > 0`; `lagged(τ)` re-sets only the OPENING constant and carries `tau_close` and the
reference; rung 80's rebuild carries both. RK4's stability bound reads the smaller constant (`tau_min`).
Rungs 66–80 march ONE constant through `r64_solve_b`, which refuses a set `tau_close` — even one equal to
`τ` (an explicit object is never silently dropped). The census was by compiler (`tau` made private on a
scratch copy): it found three readers a name grep had missed (`three_loop` ×2, `two_lag`), all now
reading through the two-constant law or refusing.

**The reading (`asymmetric_valve.rs`).** `asym_valve` (equal constants build the SYMMETRIC object, so the
grid's diagonal is rung 88's own solve, bit for bit); `matched_asym` (rung 88's re-tune on it);
`march_marked` (a march with its first closing read — the earliest RK4 stage with `cmd < q` — and its
clamp count); `phi_floor` (anchor D5: the symmetric `φ` valve re-tuned until its minimum over the stretch
BEFORE its first closing read is the target — the lowest set point any closing speed can buy it, returned
with its residual because the mark moves in grid steps, rung 83).

## 2. Reduces

`closing(τ)` at `τ_close = τ_open` marches bit for bit as the symmetric valve through the switching code
(anchor G0: 36 pairs, 12 876 points; the gate re-checks four). D2: before the first closing read the
march is bit for bit independent of `τ_close`, and parts after it (gated). The grid's diagonal reproduces
rung 88 § 4's table. No oracle, no `rung64`–`rung88` gate and no CLI golden moved; the default design run
stays rung 6.

## 3. Scoring (anchor § 3 — bars frozen at `43ae914`)

19 cells (rung 87/88's) × the 20 off-diagonal `(τ_open, τ_close)` pairs of {0.01, 0.02, 0.05, 0.1, 0.2}²,
stators on, both valves re-tuned to `m_lim` (tol 1e-12), `ds` 0.005 AND 0.0025. Zero clamp hits on every
run; every pair bracketed.

| # | verdict | measured |
|---|---|---|
| D1′ (no credit) | arithmetic | `∫(b − b_cmd)·τ_o/τ(s) − τ_o(b0 − b_end)` ≤ 0.19·`max(1, τ_o/τ_c)`·`ds²` on every run and across five halvings — O(ds²) with an erratic coefficient (where the valve turns inside a step: rung 84's sawtooth) |
| H1 | **HIT at `ds` 0.005** (380/380); **GRID-SENSITIVE** — 376/380 at 0.0025 | the set point clause holds at both steps (`|Δφ_lim|` 0 at 0.005, ≤ 6.6e-10 at 0.0025); the ORDER clause fails in one row at the fine step — shaped .25/.34 at `τ_o` 0.02, all four `τ_c`: an onset-stage closing read at s 0.1263, before the minimum at 0.1575. The anchor named this run (G1′) and the reason in advance; the closing constant does reach that minimum, by ≤ 6.6e-10 |
| H2 | **HIT**, 380/380, both steps — *arithmetic* | the incidence bill moves by `(τ_c − τ_o)·D_inc` to within 0.59 of the bar (drift −2.9e-4 to 2.8e-4) |
| H3a | **HIT**, 56/56 rows, both steps | the `φ` set point falls with `τ_c`, never below its floor, and sits on it once its minimum comes first: in 11 rows inside the grid (`τ_o` 0.05 from `τ_c` 0.1 in 5 cells, from 0.2 in 1; `τ_o` 0.1 from 0.2 in 5); the other 45 are 0.0003–0.0052 sm above it at `τ_c` 0.2. Every floor a root (residual ≤ 1.6e-15, one 1.4e-7 — anchor G4) |
| H3b | **HIT**, 26/26, both steps | where the `φ` valve's minimum comes first, a slower close leaves its set point unchanged |
| H4a | **HIT**, 26/26, both steps | there, `R` is the symmetric forecast `R_f` to 0.07% (bar 2%) |
| H4b | **HIT**, 224/224, both steps | close-first rows: `R_floor·0.98 ≤ R < R_f` for a slower close, `R > R_f` for a faster one |
| H5a | **HIT**, 40/40, both steps | in every row where the symmetric valve keeps rung 87's verdict but the `φ` valve closes before its minimum, `τ_c` 0.2 reverses it: `R` 0.68–0.89 |
| H5b | **HIT**, 57/57, both steps | where both bounds lie on one side of 1, `R` stays there |
| H5c | **MISS**, 139/190, both steps (the same 51 runs) | predicted: a FAST close never flips the verdict. It flips it in 51, every one the same way — the symmetric `φ` valve cheaper, the fast-closing one dearer (`R` 1.0006–1.73). The anchor's "likeliest miss" (shaped 1/.42 at `τ_o` 0.05) is one of them, at 1.73 |

The two steps give the same verdict on every scored run but H1's one row: the same H5a hits, the same
51 H5c misses, the same H3a floor rows, the same sign of `R` in every one of the 475 runs.

## 4. Who is cheaper, across the two speeds (after measuring; not scored)

The slowest OPENING speed at which the incidence valve is still the cheaper one, per closing speed ("—":
the `φ` valve is cheaper at every opening). Column "sym" is rung 88's symmetric valve (the diagonal):

| maps | `r` | `sm` | sym | `τ_c` 0.01 | 0.02 | 0.05 | 0.1 | 0.2 |
|---|---|---|---|---|---|---|---|---|
| shaped | 0.25 | 0.34 | 0.02 | 0.05 | 0.02 | — | — | — |
| shaped | 0.25 | 0.38 | 0.01 | 0.02 | 0.01 | — | — | — |
| shaped | 0.25 | 0.42 | 0.01 | 0.02 | 0.01 | — | — | — |
| shaped | 0.5 | 0.42 | 0.02 | 0.05 | 0.05 | 0.01 | — | — |
| shaped | 0.5 | 0.46 | 0.02 | 0.1 | 0.05 | 0.02 | — | — |
| shaped | 0.5 | 0.50 | 0.02 | 0.1 | 0.05 | 0.01 | — | — |
| shaped | 1 | 0.42 | 0.02 | 0.1 | 0.1 | 0.02 | — | — |
| shaped | 1 | 0.46 | 0.05 | 0.2 | 0.2 | 0.05 | — | — |
| shaped | 1 | 0.50 | 0.05 | 0.2 | 0.2 | 0.1 | 0.02 | — |
| shaped | 1 | 0.54 | 0.05 | 0.2 | 0.2 | 0.1 | 0.01 | — |
| tilted | 0.25 | 0.34 | 0.02 | 0.02 | 0.02 | — | — | — |
| tilted | 0.25 | 0.38 | 0.01 | 0.01 | 0.01 | — | — | — |
| tilted | 0.25 | 0.42 | 0.01 | 0.02 | — | — | — | — |
| tilted | 0.5 | 0.42 | 0.02 | 0.05 | 0.05 | 0.01 | — | — |
| tilted | 0.5 | 0.46 | 0.02 | 0.1 | 0.05 | 0.02 | — | — |
| tilted | 0.5 | 0.50 | 0.02 | 0.1 | 0.05 | 0.01 | — | — |
| tilted | 1 | 0.46 | 0.05 | 0.2 | 0.1 | 0.05 | — | — |
| tilted | 1 | 0.50 | 0.05 | 0.2 | 0.2 | 0.1 | 0.01 | — |
| tilted | 1 | 0.54 | 0.05 | 0.2 | 0.2 | 0.1 | 0.01 | — |

(The same table at both steps.) In every cell, every row and every column of the 5×5 grid changes side
at most once (190 lines, both steps): fast-and-fast is the incidence valve's corner, and slowing EITHER
edge moves toward the `φ` valve. Counted over the 19 cells × the 5 speeds of the other edge, the
incidence valve is cheaper in:

| held edge | 0.01 | 0.02 | 0.05 | 0.1 | 0.2 |
|---|---|---|---|---|---|
| CLOSING held at | 68 | 58 | 32 | 5 | **0** |
| OPENING held at | 54 | 43 | 33 | 22 | 11 |

The closing speed sweeps the verdict from 68 of 95 to none; the opening speed only from 54 to 11. `R`
itself is NOT monotone (deep on the `φ` side it rises again, e.g. shaped .5/.46: 0.719 at (0.05, 0.1),
0.741 at (0.05, 0.2)) — the claim is the SIDE, not the size. The thrust integral agrees in side with the
bleed in 348 of 380 off-diagonal runs (reported, not scored).

## 5. The mechanism, measured

* **The incidence valve — the opening edge's.** Its minimum comes before its first closing read in 95 of
  95 symmetric rows (anchor G1), and stays there at every closing speed (H1): its set point is the same
  to 1e-9 across closing speeds, and differs by up to 0.099 sm across opening speeds. A slower close only
  keeps it open longer on the way down, and D1′ prices that exactly: `(τ_c − τ_o)·D` (H2). This is rung
  52's *a self-releasing limiter cannot debit the spool it watches*, on the airflow lever (anchor D3).
* **The `φ` valve — the closing edge's, down to a floor.** In the 56 rows where it closes before its
  margin bottoms out, its set point moves ≤ 0.001 sm across opening speeds and up to 0.073 across closing
  speeds. A slower close holds bleed through the minimum, so a lower target protects the same margin —
  down to the floor, the margin before its first closing read, which no closing speed touches (D2).
* **Why the fast close flips it (H5c's miss).** The bounds were symmetric forecasts: `R_f` moves both
  bills by `Δτ·D` at FIXED set points. A faster close RAISES the `φ` valve's set point (122 of 190
  fast-close pairs; 0.004–0.063 sm in the 51 misses) — in 62 of the 130 pairs from rows where its minimum
  came first, the fast close moves its closing AHEAD of the minimum, so the closing speed starts pricing
  it. The incidence valve's set point does not move. H4b registered only the side (`R > R_f`, 224 of
  224), not the size; `R_f` sat at 0.48–0.93 where `R` measured 1.00–1.73. The symmetric bounds could not
  see the set point move, and the anchor said so (*"the least bounded claim here"*).

## 6. The findings, in order of how much the construction already implied them

1. **A fast close reverses the verdict too** (H5c's MISS) — not implied: the registered forecast had the
   direction and missed the size by a factor up to 2.6.
2. **The closing speed is the stronger dial; the boundary is a staircase** (§ 4) — measured after the
   bars, not registered.
3. **A slow close makes the `φ` valve cheaper at every opening speed** (H5a, extended by § 4's 0 of 95) —
   partly construction: G5–G6 measured the bounds and the break-even before it was frozen.
4. **The `φ` valve's set point falls with the closing speed, to a floor** (H3) — the floor is D5's
   construction; that it is REACHED inside the grid in 11 rows, and the strictness above it, are not.
5. **The incidence valve is the opening edge's** (H1) — construction (D2 + D3) but for an onset stage,
   which the finer step found in one row.
6. **The closing edge's bill is `(τ_c − τ_o)·D`** (H2, D1′) — arithmetic. Ranked last.

## What it does to its neighbours

* **Rung 88 — SPLIT.** Its crossover (*the `φ` valve is cheaper for a slow enough valve*) is the
  diagonal of a two-speed plane, and "slow" was two dials: the closing one decides more. Its § 5
  (*the incidence valve's loss lives on the opening edge, the `φ` valve's gain on the closing edge*)
  holds as SET POINTS, exactly for the incidence valve and to 0.001 sm for the `φ` valve.
* **Rung 65 — its symmetric-lag concession DISCHARGED.** The two edges price differently; a single
  constant hides which one a verdict rests on.
* **Rung 52 — CONFIRMED on the airflow lever.** Its fast-attack / slow-release law on the fuel side
  found a self-releasing limiter cannot debit what it watches; here the incidence valve's closing edge
  never reaches its own minimum (D3), so its closing speed is pure bill.
* **Rung 87 — unchanged.** Its instant comparison is the corner where both speeds go to zero; its
  advice *watch the wall* still holds for protection.

## Concessions

* **The time constants are swept on the march's own `s`** (rung 65's): every location in § 4 is a
  measurement on this rig; the claims are the SIDES, the one-change staircase and the SIGNS.
* **H1 is grid-sensitive in one row**: at the finer step an onset-stage closing read precedes the
  minimum in shaped .25/.34 at `τ_o` 0.02. The set points still agree to 6.6e-10.
* **D1′ is an O(ds²) identity with an erratic coefficient**: the weight `τ_o/τ_c` jumps where the valve
  turns, so the residual is an envelope, not a clean ×4 per halving.
* **The opening start convention is rung 65's** (`b(0) = b_cmd(0)`); six `φ` runs start saturated
  (anchor G2) and have no opening edge at all.
* ONE placement (`n_lo` 0.65), the LP currency only, rung 36's imposed `φ_surge`, invented maps, the
  valve on the LP only, no fuel leg; `R` compared over the stretch both runs reach.
* The § 4 counts and the "stronger dial" table were defined after scoring and are reported, not scored.

## The next seam

**The station-3 customer bleed** (rung 86's, rung 88's): air taken off the core for the aircraft, not to
protect a compressor — a bleed the valves here would have to work around. Beside it, still open: **the
two-lag cascade with two speeds each** — rung 66's degeneracy read edge by edge (rung 66 refuses an
asymmetric valve today, by `r64_solve_b`).

## 9. Step check and scripts

`ds` 0.005 and 0.0025 give the same verdicts and counts on every H but H1 (§ 3), the same § 4 table and
counts, and the same staircase. The scorer (run as 15 below-normal pieces), the merged grids and logs,
the after-scoring readings, the D1′ step study, the page's step check and the crash map are archived in
`docs/plans/scratch-archive/rung89/`.
