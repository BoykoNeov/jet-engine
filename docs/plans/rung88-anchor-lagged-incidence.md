# Rung 88 anchor — the SLOW valve on a moving target: does a lag's cost depend on the wall it watches? (pre-registered 2026-10-15 — § 3)

**Seam:** rung 87 § *The next seam* — *"A **lagged incidence valve** (rung 65's bandwidth on a MOVING
target — does the lag's cost depend on which wall it tracks?)"*. Rung 87 refused the object (a lagged
incidence valve panicked in `on_incidence`); rung 65 measured the lag only on the `φ` valve with the
stators off.

**The user's choice (2026-10-15):** this seam, next. Shipped as KNOBS: every prediction below is scored
HIT/MISS, and a miss is a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\jet-rung88-probe`, not in the repo; its
probe is archived with the rung) on the device of § 1, BEFORE § 3 was written. Nothing about a LAGGED
INCIDENCE valve beside the stators was measured before this file was committed — except § 0's G1,
which runs it with the stators OFF (where it is the `φ` valve).

---

## 0. WHAT EXISTS — measured before anything was written

Rig: rung 87's (= rung 86's) — CPG two-spool, shaped and tilted maps, `φ_surge` 0.55, `Tt4` 1000 →
1400 K, settle 1.2, `r` ∈ {0.25, 0.5, 1.0}; LP stator schedule 0.20 below `n_lo` 0.65 (S1) or none (S0);
`b_max` 0.10. Every machine is a sibling of a bare machine built by rung 65's `build_lagged_bleed`
(rung 64's refuses a lagged valve; with `τ = None` the two builders march bit for bit — G1). Bleed is the
RECORDED position on a lagged march (rung 65 corrected rung 64: a lagged position cannot be re-solved),
the RE-SOLVED one on an instantaneous march.

**G1 — the reduces and the entry points (bit for bit, 341 points × 11 keys, `ds` 0.005).** Stators off:
lagged incidence valve ≡ lagged `φ` valve (`τ` 0.05). Stators on, lagged `φ` and lagged incidence valve:
`three_levers::march` ≡ `stator_march` (no scope dropped). Stators on, instantaneous `φ` and incidence
valve: rung 65's builder ≡ rung 64's. `.on_incidence().lagged(τ)` == `with_tau(.., τ).on_incidence()`
(the reference survives `lagged` — § 1).

**G2 — WHERE EACH VALVE STARTS.** Rung 65 starts a lagged valve at its EQUILIBRIUM position, `b(0) =
b_cmd(0)` (§ 0 there: starting at 0 would inject a transient). Measured on the instantaneous valves, S1,
rung 87's 21 tabulated cells (19 scored + the 2 band cells ¹), incidence valve at `sm`, matched `φ`
valve at rung 87's `sm*`:

| cell | `b0` incidence | `b0` matched `φ` | | cell | `b0` incidence | `b0` matched `φ` |
|---|---|---|---|---|---|---|
| shaped .25 / .34 | 0 | 0 | | tilted .25 / .34 | 0 | 0 |
| shaped .25 / .38 | 0 | 0 | | tilted .25 / .38 | 0 | 0 |
| shaped .25 / .42 | 0 | 1.918e-2 | | tilted .25 / .42 | 0 | 8.943e-3 (S) |
| shaped .5 / .38 ¹ | 0 | 0 | | tilted .5 / .42 | 0 | 5.777e-3 |
| shaped .5 / .42 | 0 | 1.764e-2 | | tilted .5 / .46 | 0 | 4.818e-2 (S) |
| shaped .5 / .46 | 3.622e-3 | 5.708e-2 (S) | | tilted .5 / .50 | 2.266e-2 | 8.637e-2 (S) |
| shaped .5 / .50 | 3.070e-2 | 9.392e-2 (S) | | tilted 1 / .42 ¹ | 0 | 0 |
| shaped 1 / .42 | 0 | 0 | | tilted 1 / .46 | 0 | 3.322e-2 |
| shaped 1 / .46 | 3.622e-3 | 4.480e-2 | | tilted 1 / .50 | 2.266e-2 | 7.810e-2 (S) |
| shaped 1 / .50 | 3.070e-2 | 8.759e-2 (S) | | tilted 1 / .54 | 4.983e-2 | 0.1 (S) |
| shaped 1 / .54 | 5.645e-2 | 0.1 (S) | | | | |

(S) = the matched `φ` valve saturates somewhere in its run. The incidence valve starts SHUT in 13 of 21
cells; the `φ` valve starts at or above it in all 21 and strictly above in 14 of the 19 scored. The
stators are most closed at the start: that raises the incidence margin (the incidence valve has nothing
to do) and lowers `φ` (the `φ` valve is already working).

**G3 — the lagged `φ` valve beside the stators runs today, and three things about it are given** (all
21 cells, `τ` ∈ {0.01, 0.02, 0.05, 0.1, 0.2}, `ds` 0.005):
- **Its bleed integral grows by `τ·b0`** (to within 10 % of `τ·b0`, by D1 below; the rest, the drift, ≤ 0.9 % of `τ·b_peak`): e.g. shaped .5/.46, `b0`
  5.71e-2: Δ∫b 5.77e-4 / 2.88e-3 / 1.14e-2 at `τ` 0.01 / 0.05 / 0.2, against `τ·b0` 5.71e-4 / 2.85e-3
  / 1.14e-2. Where `b0` = 0 (6 cells) |Δ∫b| ≤ 7.2e-5 at every `τ` and ≤ 8e-6 at `τ` ≤ 0.02 — the latter
  within the trapezoid residual, so no sign is claimed there.
- **In the INCIDENCE currency its lag is a CREDIT at small `τ`**: grid-min `M_i` RISES at `τ` 0.01 in
  all 21 cells (e.g. shaped .5/.42: 0.53778 → 0.54245). Its `M_i` minimum sits LATE, in the stretch
  where the stators have reopened and the valve is CLOSING (`ḃ < 0` at the minimum in all 21); a lagged
  valve closes late, so it holds more bleed there than its law asks.
- **The credit turns over**: at larger `τ` the minimum JUMPS to the opening edge (`s` 0.1–0.3), where a
  lag under-opens, and `M_i` falls back — below the instantaneous value by `τ` 0.2 in 6 cells (every `r` 0.25 cell).

**G4 — rung 65's own case** (S0, `φ_lim` 0.80, `r` 0.5, its `τ` list): Δ∫b against `τ·b_cmd(0)`
(`b_cmd(0)` = 0.036626) — 3.78e-4 / 3.66e-4 (`τ` 0.01) … 7.44e-3 / 7.33e-3 (0.2) … 1.354e-2 / 1.465e-2
(0.4, where the valve has not closed by the end: `b_end` 3.3e-3, so the bill is `τ(b0 − b_end)`).
Zero clamp hits. Rung 65 § 1's *"the closing tail dominates the integral"* IS D1.

**G5 — the matcher's object.** The lagged `φ` valve's grid-min `M_i` over `φ_lim` (S1, all six
shape × `r`, `τ` ∈ {None, 0.01, 0.05, 0.2}, 16 set points from the shut run's `min φ` to 1.5) is
non-decreasing, with a flat top below the full-open value (the valve saturates at its binding point)
— so a target below that top has ONE root, and a target on it has none worth scoring. Zero clamp hits.

## 1. THE DEVICE — the incidence valve, given rung 65's lag

No new law: rung 87's target `φ*(v) = φ_lim/(1 + v·φ_lim)`, chased by rung 65's first-order lag
`db/ds = (b_cmd − b)/τ`, `b_cmd` = rung 64's root at the state against `φ*(v)` at that state. Three
edits, each `φ`-valve path unchanged to the operation (oracles and every rung 64–87 gate untouched):
- `BleedLimiter::on_incidence` no longer refuses a lag.
- `BleedLimiter::lagged` KEEPS the reference. It rebuilt through `with_tau`, which resets it to `φ` — so
  a lagged incidence valve came back a `φ` valve, silently (the sibling trap, inside the device). Rung
  67's five readers that rebuilt `with_tau(own.phi_lim, own.b_max, ..)` now call `own.lagged(τ)`, so
  they carry an incidence valve to the refusal below instead of quietly converting it.
- Rung 65's march solves its command against `r87_target` (rung 64's three cells' spelling): `φ_lim` for
  the `φ` valve, no `v` read.
- Rungs 66–80 still solve every valve through `r64_solve_b`, which REFUSES an incidence valve (a
  compiler census, making `phi_lim` private on a scratch copy, found every reader of a carried valve's
  set point; all are on machines whose marches go through that refusal).

## 2. DERIVATIONS

**D1 — the lag's bleed identity (arithmetic, NO credit, gated).** Integrate `db/ds = (b_cmd − b)/τ`
over the run:

```
    ∫ (b − b_cmd) ds = τ · (b(0) − b(end))
```

exact in RK4's own stage quadrature, to the trapezoid's O(ds²) on recorded points; broken only by the
position clamp (`Census65::clamp_hits` must read 0) and read with `b(end)` reported. So against the
INSTANTANEOUS valve's bleed integral:

```
    Δ∫b(τ) = τ·(b0 − b_end) + drift,     drift = ∫b_cmd(lagged run) − ∫b(instantaneous run)
```

and with rung 65's convention `b(0) = b_cmd(0)`, **the lag's bleed bill is `τ` × the opening it STARTS
with, plus the drift.** A valve started anywhere else pays a different bill; the convention is
load-bearing and it is rung 65's, not this rung's.

**D2 — the first-order sign rule (a reading, not a theorem).** For small `τ`, `b − b_cmd ≈ −τ·ḃ_cmd`:
the valve UNDER-opens where its command rises and OVER-holds where it falls. A valve whose minimum is a
PINNED PLATEAU in its own coordinate loses protection there (the plateau starts with the command
rising — rung 65). A valve whose minimum in ANOTHER coordinate is a single point on a FALLING command
gains protection there at small `τ` (G3). So "bandwidth is pure loss" (rung 65) is a statement in the
valve's OWN coordinate.

**D3 — where the moving target enters.** The incidence valve's target moves with `v`; the lag moves the
shaft state, which moves `v`, which moves the target. All of that lives in the DRIFT of D1 (the command
integral re-read along the lagged trajectory); the `τ·b0` term does not see it. So the seam's question
— *does a moving target change the lag's cost?* — is, on the bleed side, a question about the drift.

## 3. PREDICTIONS — registered before measuring (bars frozen here)

**Scored set:** rung 87's 19 scored cells (G2 minus the two ¹), S1, the incidence valve at `sm`, its
matched `φ` partner at rung 87's `sm*`. `τ` ∈ {0.01, 0.02, 0.05, 0.1, 0.2}. **THE STEP-SIZE BAR:**
every H must hold at `ds` 0.005 AND 0.0025 (`ds/τ` ≤ 0.5 throughout); one step only = GRID-SENSITIVE.
**Equal length:** every integral is compared over the stretch both runs reach; `npts` reported.
**Preconditions per run (D1):** clamp hits = 0 and `b_end` reported; a run breaking them is reported and
left out of H1–H2.

- **H1 (the moving target adds no first-order drift — THE CONTENT on the bleed side).** For the lagged
  incidence valve: `|drift| ≤ max(0.05·τ·b_peak, 1e-5)` in every scored cell at every `τ`, `b_peak` its
  instantaneous run's. (G3 puts the `φ` valve's |drift| at ≤ 0.9 % of `τ·b_peak`; the incidence valve
  has the extra channel of D3.) Its SIGN, and its size against the `φ` partner's drift on the same cell,
  are reported, not scored.
- **H2 (so the wall decides the lag's bleed bill — mostly construction: G2 + D1 + H1).** In the 14
  scored cells where the `φ` partner starts strictly more open, Δ∫b(incidence, `τ`) < Δ∫b(`φ`, `τ`) at
  every `τ`. Ranked last.
- **H3 (in its OWN coordinate the incidence valve's lag is pure loss).** Grid-min `M_i`(incidence, `τ`)
  < `m_lim`, and strictly decreasing in `τ`, in every scored cell — rung 65 § 1 on a moving target; the
  opposite sign to its `φ` partner's in the same currency at `τ` 0.01 (G3).
- **H4 (the opening edge carries it).** The lagged incidence valve's `M_i` minimum sits where its
  command RISES (`b_cmd` increasing between the neighbouring points) in every scored cell at every `τ`
  — D2's rule read on the device built to make its own coordinate a plateau.
- **H5 (rung 87 survives a slow valve — THE CONTENT on the protection side).** At each `τ`, the lagged
  `φ` valve MATCHED (same `τ`; Illinois over `φ_lim`, `|ΔM_i| < 1e-7`) to the lagged incidence valve's
  grid-min `M_i` bleeds MORE (`b_int`) and loses MORE thrust integral than the lagged incidence valve,
  in every scored cell whose match brackets below G5's flat top. The lag takes protection from the
  incidence valve and gives it to the `φ` valve (G3, H3), so the matched `φ` valve gets a lower set
  point and a smaller bill than rung 87's — the gap closes; the prediction is that it does not close
  to zero by `τ` 0.2. Ratios reported, not scored.

**Size: NO bar** beyond H1's.

**Reduces (gates, not predictions):** stators off, the lagged incidence valve ≡ the lagged `φ` valve, bit
for bit (G1); `τ → 0` converges to rung 87's instantaneous incidence valve (a limit, not an equality, as
rung 65); every rung 64–87 gate and oracle green without an edit beyond rung 87's refusal test (now the
refusal of `r64_solve_b` alone); the default design run stays rung 6.

## 4. THE KNOBS (sandbox, same batch)

The Controls view's "opens on its own" bleed valve gains its **speed**: instant (today's, the default —
every saved setting stays valid) or a time constant `τ`. A slow valve is built through rung 65's builder
(rung 64's refuses it); the lever chart draws the RECORDED position; prechecks refuse `ds/τ` above rung
65's 2.0 and the fuel legs rung 65's march refuses beside a lagged valve (rung 47's lagged governor, rung
52's lag, rungs 50/51's forced edges — if the page offers them). Check cases use no equal set points
(rung 87's lesson). Gates follow rung 87's: replay bit for bit, `check_requests` cases, a `browser.mjs`
case, a crash map.
