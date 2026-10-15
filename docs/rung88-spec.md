# Rung 88 — the SLOW valve on a moving target: does a lag's cost depend on the wall it watches?

**Code:** the device edits in `rust/src/limited_bleed.rs` (`BleedLimiter::lagged` keeps the reference;
`on_incidence` takes a lag) and `rust/src/lagged_bleed.rs` (rung 65's march solves its command against
the valve's own target); the reading in `rust/src/lagged_incidence.rs` · **Gates:** `rust/tests/rung88.rs`
· **Panel:** `print_lagged_incidence_table` (born in Rust, held to its own capture
`rust/oracle/rust_owned/`, a change detector only) · **Anchor (givens, derivations, every pre-registered
bar, committed `81831f4` before any was measured):** `docs/plans/rung88-anchor-lagged-incidence.md` — §
refs below are to it · **Sandbox:** the Controls view's bleed valve, "Moves slowly"
(`docs/plans/sandbox-plan.md` § 17) · **Scratch scripts:** `docs/plans/scratch-archive/rung88/`.

**Shipped as KNOBS** (the sandbox direction): every result is shown; predictions were registered first
and are scored here, a miss being a finding.

Rung 87 closed by naming this seam: *"A **lagged incidence valve** (rung 65's bandwidth on a MOVING
target — does the lag's cost depend on which wall it tracks?)"*. Rung 87 had refused the object; rung
65 measured the lag only on the `φ` valve with the stators off.

> **THE HEADLINE — an exact law has no slack for a slow actuator.** The incidence valve opens exactly
> as far, and exactly as late, as the protection needs; so it must open FAST, and any lag is protection
> lost that it has to buy back with a higher set point. The flow-coefficient valve opens EARLY (the
> stators lower `φ` from the start) and closes late, and that surplus absorbs the lag — at a small lag
> it even GAINS protection. Re-tuned so both hold rung 87's incidence margin exactly, the `φ` valve
> costs 1.27–3.47× the incidence valve's air when instant, and **0.53–0.91× by `τ` = 0.1 in all 19
> cells** (less thrust too, 18 of 19): **rung 87's saving belongs to a FAST valve.** The crossover sits
> between `τ` 0.02 and 0.1 HP-spool time constants on this rig — a measurement on the march's own time
> scale (rung 65's concession), not a claim about real valves: the claim is *a slow enough valve*.
>
> **And on the bleed side, the lag's bill is arithmetic.** Integrating the lag, `∫(b − b_cmd) = τ·(b(0)
> − b(end))`: a slow valve's extra air is `τ` × the opening it STARTS with, plus a drift. With the
> stators on, the incidence valve starts shut in 13 of 21 cells and the `φ` valve open, so the wall sets
> the bill through the start — and the moving target (the incidence valve's, through `v`) adds only a
> small drift (H1). Rung 65 § 1's *"the closing tail dominates the integral"* was this identity.

## 1. What it adds

**The device (anchor § 1).** No new law: rung 87's target `φ*(v) = φ_lim/(1 + v·φ_lim)` chased by rung
65's first-order lag. Three edits, each leaving every `φ`-valve path unchanged to the operation:
`on_incidence` no longer refuses a lag; `lagged` keeps the reference (it rebuilt through `with_tau`, which
reset it to `φ` — a lagged incidence valve came back a `φ` valve, silently: the sibling trap, inside the
device); rung 65's march solves its command against `r87_target`. Rung 67's five readers that rebuilt
`with_tau(own.phi_lim, own.b_max, ..)` now call `own.lagged(τ)`, so they carry an incidence valve to the
refusal instead of quietly converting it. Rungs 66–80 still solve every valve through `r64_solve_b`,
which refuses an incidence valve (a compiler census, `phi_lim` made private on a scratch copy, found
every reader of a carried valve's set point).

**The reading (`lagged_incidence.rs`).** `run_lagged` / `lag_read`: one slow run in rung 87's currencies
plus D1's ledger (`b0`, `b_end`, `∫b`, `∫b_cmd`, the identity's residual, the clamp count, whether the
command rises at the `M_i` minimum); `matched_lagged`: either valve, at a given lag, re-tuned by Illinois
over its set point until its run-minimum `M_i` equals a target (`matched_phi_lagged` is its `φ` half,
H5's instrument).

## 2. Reduces

With the stators off the lagged incidence valve IS the lagged `φ` valve, bit for bit (341 points, every
recorded position and command). With `τ = None` rung 65's builder marches rung 64's machine bit for bit.
`three_levers::march` and `stator_march` agree bit for bit on a lagged machine. No oracle, no
`rung64`–`rung87` gate and no CLI golden moved; rung 87's refusal test lost only the refusal this rung
removes. The default design run stays rung 6.

## 3. Scoring (anchor § 3 — bars frozen at `81831f4`)

19 scored cells (rung 87's), stators on, `τ` ∈ {0.01, 0.02, 0.05, 0.1, 0.2}, `ds` 0.005 AND 0.0025 —
every verdict the same at both steps. D1's preconditions held on every run (zero clamp hits).

| # | verdict | measured |
|---|---|---|
| D1 (no credit) | arithmetic | `∫(b − b_cmd) − τ(b0 − b_end)` ≤ 3.5e-6 on every run at `ds` 0.005, ≤ 9.6e-7 at 0.0025 — the trapezoid's O(ds²) |
| H1 | **HIT**, 19 of 19, both steps | the incidence valve's drift ≤ `max(0.05·τ·b_peak, 1e-5)`: 6.3e-7–4.2e-4, POSITIVE in all 95 (cell, `τ`) runs; the `φ` partner's is mixed in sign (−5.0e-5 to 6.2e-5) and smaller in 82 of 95 — the moving target adds a small one-signed drift, no first-order term |
| H2 | **HIT**, 14 of 14 — *mostly construction* | where the `φ` partner starts more open, its lag costs more air (`τ·b0`) |
| H3 | **HIT**, 19 of 19 | the slow incidence valve's own protection falls below `m_lim`, monotone in `τ`: −1.2e-3 to −8.6e-3 at 0.01, −7.6e-3 to −5.8e-2 at 0.2 |
| H4 | **HIT**, 19 of 19 | its `M_i` minimum sits where its command RISES — the opening edge |
| H5 | **MISS**, 39 of 95 (cell, `τ`) pairs, both steps | matched to the slow incidence valve's OWN minimum, the slow `φ` valve bleeds more in 19/19 at `τ` 0.01, 15/19 at 0.02, 5/19 at 0.05, **0/19 at 0.1 and 0.2**. Predicted: the gap closes but not to zero. It closes and REVERSES |

H5 matched the `φ` valve to the slow incidence valve's degraded margin, which a reader can object prices a
dip below `m_lim` as protection. § 4's table has no such gap, and leads.

## 4. Equal REQUIREMENT — both slow valves re-tuned to hold `m_lim` exactly (after measuring; not scored)

Each valve at each lag set by Illinois so its run-minimum `M_i` is rung 87's `m_lim` (`|ΔM_i|` ≤ 1e-15).
Entry = the `φ` valve's bleed integral over the incidence valve's (bold < 1: the `φ` valve is cheaper);
‡ = the thrust integral disagrees in sign with the bleed (the two currencies cross at different `τ`):

| maps | `r` | `sm` | instant | `τ` 0.01 | 0.02 | 0.05 | 0.1 | 0.2 |
|---|---|---|---|---|---|---|---|---|
| shaped | 0.25 | 0.34 | 3.47 | 1.83 | 1.15 | **0.59** | **0.65** | **0.72** |
| shaped | 0.25 | 0.38 | 1.83 | 1.15 | **0.90** | **0.73** | **0.76** | **0.78** |
| shaped | 0.25 | 0.42 | 1.60 | 1.20 | **0.89** | **0.76** | **0.82** | **0.83** |
| shaped | 0.5 | 0.42 | 2.28 | 1.76 | 1.39 | **0.76** | **0.65** | **0.70** |
| shaped | 0.5 | 0.46 | 1.72 | 1.45 | 1.25 | **0.92 ‡** | **0.74** | **0.77** |
| shaped | 0.5 | 0.50 | 1.30 | 1.20 | 1.11 | **0.90** | **0.80** | **0.82** |
| shaped | 1 | 0.42 | 3.46 | 2.54 | 1.90 | **0.95** | **0.53** | **0.57** |
| shaped | 1 | 0.46 | 1.94 | 1.70 | 1.50 | 1.08 | **0.72** | **0.71** |
| shaped | 1 | 0.50 | 1.64 | 1.53 | 1.43 | 1.17 | **0.87** | **0.75** |
| shaped | 1 | 0.54 | 1.27 | 1.21 | 1.16 | 1.05 | **0.91 ‡** | **0.77** |
| tilted | 0.25 | 0.34 | 3.11 | 1.65 | 1.08 | **0.62** | **0.68** | **0.74** |
| tilted | 0.25 | 0.38 | 1.76 | 1.10 | **0.88** | **0.74** | **0.77** | **0.81** |
| tilted | 0.25 | 0.42 | 1.58 | 1.16 | **0.85** | **0.78** | **0.83** | **0.83** |
| tilted | 0.5 | 0.42 | 2.37 | 1.77 | 1.36 | **0.73** | **0.64** | **0.69** |
| tilted | 0.5 | 0.46 | 1.78 | 1.46 | 1.27 | **0.88** | **0.73** | **0.77** |
| tilted | 0.5 | 0.50 | 1.31 | 1.20 | 1.09 | **0.89** | **0.81** | **0.83** |
| tilted | 1 | 0.46 | 2.10 | 1.77 | 1.52 | 1.02 | **0.66** | **0.70** |
| tilted | 1 | 0.50 | 1.71 | 1.58 | 1.45 | 1.14 | **0.82** | **0.74** |
| tilted | 1 | 0.54 | 1.30 | 1.24 | 1.18 | 1.06 | **0.90** | **0.77** |

(`ds` 0.005; at 0.0025 every crossover, count and ‡ is the same, the slow columns within 0.007 and the
instant column within 0.045 — rung 87's own step spread, e.g. 3.456 / 3.501 — § 9.) Instant it is
rung 87's 1.27–3.47 (its table, reproduced). The crossover comes EARLIEST where the ramp is fast (`r`
0.25: by 0.02) or the set point low, LATEST at high set point and slow ramp (`r` 1, `sm` 0.54: by 0.1).

**Met by the law, not at a stop.** The incidence valve never reaches `b_max`. The `φ` valve does in 19
runs — early, where it fights the stators — but never at its own `M_i` minimum (opening there 0.0002–0.012):
the requirement is held by each valve's control law in every entry.

**The slow `φ` valve is cheaper than the instant one.** Its re-tuned set point FALLS with `τ` at first in
all 19 cells (e.g. shaped `r` 0.25 `sm` 0.34: 0.2609 instant → 0.2505 at `τ` 0.05), lowest at `τ`
0.02–0.1 in 15 cells and still falling at 0.2 in 4; at its best lag it holds the same margin with 0.36–0.91× the instant `φ` valve's air. The
incidence valve's set point rises monotonically with `τ` in every cell, and its slow run always costs
more than its instant one. So rung 65's *"bandwidth is pure loss"* holds in a valve's OWN coordinate:
read in another wall's, a lag can be a gain.

## 5. The mechanism, measured

At each re-tuned slow run, the `M_i` minimum's place and the valve's lag there (`b − b_cmd`):
- **`τ` 0.01:** the incidence valve's minimum sits on its opening edge (`b < b_cmd`); the `φ` valve's
  sits LATER in all 19 cells, where it is closing and OVER-holds (`b > b_cmd` in 19 of 19) — the credit.
- **`τ` ≥ 0.05:** both minima move to the opening edge; the `φ` valve's is at the same point or later in
  57 of 57 runs, already further open in 31 and lagging its command less in 36 — it started opening
  before the margin was needed, the incidence valve only when it was.

## 6. The findings, in order of how much the construction already implied them

1. **Rung 87's order reverses for a slow enough valve** (H5's MISS, § 4). Not implied by either device:
   the instant comparison was nearly construction (rung 87 D3); the slow one is not.
2. **A lag can buy protection in another wall's coordinate** (§ 4's turnover; anchor G3 saw the `M_i`
   gain, this rung prices it): the slow `φ` valve beside the stators beats the instant one.
3. **The moving target adds a small, one-signed drift** (H1) — the incidence valve's command integral
   rises with `τ`, the `φ` valve's does not systematically.
4. **The lag's protection loss sits on the opening edge** (H3/H4) — rung 65 § 1, on a moving target.
5. **The wall sets the lag's air bill through where the valve starts** (H2, D1) — arithmetic plus the
   start positions (anchor G2). Ranked last.

## What it does to its neighbours

* **Rung 87 — BOUNDED.** Its H1/H2 (*the `φ` valve costs 1.27–3.50× at equal protection*) hold for a
  FAST valve only: by `τ` 0.1 the order is reversed in every scored cell. Its advice *watch the wall*
  stands for protection (H5 of rung 87, the starved run, is untouched); its PRICE does not.
* **Rung 65 — EXPLAINED and BOUNDED.** § 1's bill is D1 (`Δ∫b` = 3.78e-4 against `τ·b_cmd(0)` = 3.66e-4
  at `τ` 0.01 … 1.354e-2 against `τ(b0 − b_end)` at 0.4, where the valve is still open at the end).
  *"Bandwidth is pure loss"* holds in the valve's own coordinate; beside a wall-moving lever, read in
  the other wall's coordinate, a lag can be a gain (§ 4).
* **Rung 64 — unchanged.** The ceiling is still the authority; no § 4 entry is met at the stop.

## Concessions

* **`τ` is a swept coordinate on the march's own `s`** (rung 65's): the crossover's location is a
  measurement on this rig, every magnitude disclaimed; the claims are the ORDER, the REVERSAL and the
  SIGNS.
* **D1's bill rests on rung 65's start convention**, `b(0) = b_cmd(0)` (the valve at equilibrium); a
  valve started elsewhere pays `τ` times a different opening.
* **The lag is SYMMETRIC** (rung 65's concession, verbatim) — and § 5 makes it sharper: the incidence
  valve's cost is on its OPENING edge, the `φ` valve's credit on its CLOSING edge, so an asymmetric
  valve would separate the two (the next seam).
* ONE placement (`n_lo` 0.65), the LP currency only, rung 36's imposed `φ_surge`, invented maps, the
  valve on the LP only; no fuel leg scored — beside a slow valve the page refuses rung 47's and 52's
  lagged legs, as rung 65's march does.
* H5's currency matched to a DEGRADED margin; § 4 is the after-measurement re-statement, not scored.

## The next seam

**The ASYMMETRIC valve** (rung 65's named seam, sharpened here): `τ_open ≠ τ_close` acts on the two halves
§ 5 separates — the incidence valve's loss lives on the opening edge, the `φ` valve's gain on the closing
edge — so a fast-opening, slow-closing valve should move the crossover, and which way is a prediction to
register. Still open beside it: the **station-3 customer bleed** (rung 86's).

## 9. Step check and scripts

`ds` 0.005 and 0.0025 give the same H verdicts and counts (H1–H4 19/19, 14/14; H5 39/95) and the same
§ 4 crossovers, counts per `τ` and mixed cells (§ 4's entries move ≤ 0.007 with a lag, ≤ 0.045 instant). The probe (§ 0 givens), the scorer, the equal-requirement runner, the page's step check
and the crash map are archived in `docs/plans/scratch-archive/rung88/`.
