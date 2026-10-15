# Rung 89 anchor — the ASYMMETRIC valve: which edge decides who is cheaper? (pre-registered 2026-10-15 — § 3)

**Seam:** rung 88 § *The next seam* — *"**The ASYMMETRIC valve** (rung 65's named seam, sharpened here):
`τ_open ≠ τ_close` acts on the two halves § 5 separates — the incidence valve's loss lives on the opening
edge, the `φ` valve's gain on the closing edge — so a fast-opening, slow-closing valve should move the
crossover, and which way is a prediction to register."* Rung 65 refused the object on rung 52's argument
(*"`tau_close` is never read while `b_cmd > b`"*); § 2 D2 says exactly where that argument holds.

**The user's choice (2026-10-15):** this seam, next. Shipped as KNOBS: every prediction below is scored
HIT/MISS, and a miss is a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\jet-rung89-probe`, not in the repo; its
probes are archived with the rung) on the device of § 1, BEFORE § 3 was written. Nothing about an
ASYMMETRIC valve (`τ_close ≠ τ_open`) was run before this file was committed — § 0 measures the symmetric
valve (rung 88's object) and the reduce `closing(τ)` ≡ symmetric only.

---

## 0. WHAT EXISTS — measured before anything was written

Rig: rung 88's (= rung 87's) — CPG two-spool, shaped and tilted maps, `φ_surge` 0.55, `Tt4` 1000 → 1400 K,
settle 1.2, `r` ∈ {0.25, 0.5, 1.0}; LP stator schedule 0.20 below `n_lo` 0.65; `b_max` 0.10; rung 88's 19
scored cells; `τ` ∈ {0.01, 0.02, 0.05, 0.1, 0.2}. Every valve below is RE-TUNED (rung 88 § 4: Illinois
over its set point until its run-minimum incidence margin `M_i` is rung 87's `m_lim`, tol 1e-12).

**G0 — the reduce.** `closing(τ)` — a closing constant EQUAL to the opening one, through the new switching
code — marches bit for bit as the symmetric valve (every field of every point, `Debug`-equal): 36 run pairs,
12 876 points, three cells × `τ` {0.01, 0.05, 0.2} × both references × stators on AND off, `ds` 0.005.
`tau_close = None` is rung 65's `(cmd − q)/tau` operation for operation: on this device the full gate
(`rust/test-all.ps1`, 207 programs, 2 473 tests incl. every rung 64–88 gate, the oracles and the CLI goldens)
passed all but the sandbox page's staleness check (its march copy now calls the same rate law, so the
embedded model changed); the page was rebuilt and every reader of it re-run green — the browser checks
(`check.ps1`: 290), `sandbox_page`, `visuals`, `sandbox_controls`.

**G1 — WHERE EACH VALVE'S WORST POINT SITS AGAINST ITS FIRST CLOSING (the decisive given).** A new census
mark records the earliest march coordinate at which ANY RK4 stage finds the command below the position —
where a closing constant would first be read. On the 190 re-tuned symmetric runs (`ds` 0.005; zero clamp
hits on all 190):

| `τ` | incidence valve: `M_i` minimum BEFORE its first closing | `φ` valve: same |
|---|---|---|
| 0.01 | 19 / 19 | 0 / 19 |
| 0.02 | 19 / 19 | 0 / 19 |
| 0.05 | 19 / 19 | 6 / 19 |
| 0.1 | 19 / 19 | 14 / 19 |
| 0.2 | 19 / 19 | 19 / 19 |

The incidence valve's later margin never comes within 1.26e-3 of its minimum (the run's `M_i` after the first
closing read, less its minimum: +1.26e-3 … +9.26e-2). The `φ` valve, where its minimum comes first, has as
little as +4.4e-5 to spare after it.

**G1′ — the same at `ds` 0.0025** (190 runs, zero clamp hits): the `φ` column identical member by member (so
the partition below is step-stable: **39** (cell, `τ`) where BOTH minima come first, **56** where the `φ`
valve's follows its first closing). The incidence column 94/95 — the one: shaped .25/.34 at `τ` 0.02, where
the mark falls at `s` 0.1263, BEFORE its minimum (0.158) — a single RK4 stage at the valve's opening ONSET
(position ~0, command just dropping back to 0) — so D2's bit-for-bit claim fails for that one run at the fine
step; it stays in H1's scored set. Elsewhere the incidence valve's slack after the mark is ≥ +1.28e-3 and the
`φ` valve's ≥ +4.4e-5. D1's residual on the 190 symmetric runs: ≤ 4.2e-6 at `ds` 0.005, ≤ 1.2e-6 at 0.0025.

**G2 — the arithmetic the bills move along** (same 190 runs; `A` = `∫b ds` over the common length, `U`/`D`
= the recorded position's total opening / closing travel). `R = A_φ/A_inc` reproduces rung 88 § 4's table
(e.g. shaped .25/.34: 1.826 / 1.152 / 0.593 / 0.649 / 0.716 against its 1.83 / 1.15 / 0.59 / 0.65 / 0.72).
Each valve rises once and falls once — counted exactly on the recorded positions: 184 of 190 runs are one
strictly-rising stretch then one strictly-falling one; the six others are the `φ` valve starting saturated
(`b0 = b_max`: no rise, one fall). No run re-opens after closing. The ratio `D_φ/D_inc`:
- `τ` 0.01: above 1 in all 19 (1.04–2.06) — and so is `R` (1.10–2.54);
- `τ` ≥ 0.02: on the same side of 1 as `R` in all but six (cell, `τ`): shaped .5/.46, .5/.50 and 1/.42 at
  0.05 (`R` 0.917 / 0.902 / 0.946, `D_φ/D_inc` 1.015 / 1.000 / 1.054), shaped 1/.50, 1/.54 and tilted 1/.54
  at 0.1 (`R` 0.866 / 0.914 / 0.895, `D_φ/D_inc` 1.021 / 1.008 / 1.007) — all six with `R` < 1 and the `φ`
  valve's minimum AFTER its first closing (G1).
- Where the `φ` valve saturates from the start (`b0 = b_max`, 6 runs), `U_φ = 0`.

**G4 — the `φ` valve's FLOOR** (the 56 close-first (cell, `τ`), `ds` 0.005). By D2, the run before the
first closing read is the same at ANY `τ_close`; so re-tuning the symmetric `φ` valve until the minimum of
`M_i` over THOSE points alone is `m_lim` gives the lowest set point any closing speed can buy it (the limit of
an infinitely slow close, where everything after the mark only gains margin). Found in all 56 (Illinois, tol
1e-12), and BELOW the symmetric set point in all 56 — by 0.002 (shaped 1/.46 at 0.1: 0.3436 vs 0.3456) to
0.078 (shaped 1/.54 at 0.01: 0.3967 vs 0.4747), in the margin `sm` = `φ_lim/0.55 − 1`. The floor barely moves
with `τ` (shaped .5/.50: 0.3705 / 0.3718 / 0.3769 at 0.01 / 0.02 / 0.05). **At `ds` 0.0025** the same 56 rows,
every floor within 0.0003 of the coarse one. **The residual** (pre-mark minimum − `m_lim`, read back at the
root — the mark moves in grid steps with the set point, so a bracketing solve can land on a jump, rung 83):
≤ 1.6e-15 in 111 of 112, and **1.4e-7 in one** (tilted .5/.42 at `τ` 0.02, `ds` 0.005 — a landing on a
step; −6.7e-16 at 0.0025). H3a's floor bars are read at each step's own floors.

**G5 — the two bounds on `R`, from symmetric runs only** (the forecast of D4, and its floor twin).
`R_f(τ_o, τ_c)` (both valves at their symmetric set points at `τ_o`, each bill plus `(τ_c − τ_o)·D`) and
`R_floor(τ_o, τ_c)` (the same with the `φ` valve at its G4 floor, its own `D` and air):
- In the 39 both-minimum-first (cell, `τ_o`): `R_f` never crosses 1 from `R(τ_o, τ_o)` — 0 of 156 off-diagonal pairs.
- In the 56 close-first, every `τ_c > τ_o` (164 pairs): where `R(τ_o, τ_o) < 1` (16 (cell, `τ_o`): `τ_o`
  0.02 ×4, 0.05 ×7, 0.1 ×5) both bounds sit below 1 (31 of 31 pairs). Where `R(τ_o, τ_o) > 1` (40: `τ_o` 0.01
  ×19, 0.02 ×15, 0.05 ×6) they STRADDLE 1 in all 133 pairs: `R_f` 1.03–2.52 above, `R_floor` 0.55–0.87 below.
  Whether the verdict flips there is the open question — nothing symmetric can answer it.
- For `τ_c < τ_o` (close-first, 60 pairs) `R_f` never crosses 1; the credit a FAST close strips is not
  bounded by anything symmetric.
- Both steps give the same 39 / 40 / 16 partition, member by member, and the same 0 of 156.

**G6 — what H5(a) actually asks: the BREAK-EVEN set point** (the 40 straddle rows, both steps). Holding the
incidence valve's air at its symmetric value plus `(0.2 − τ_o)·D_inc` (H1–H2), the `φ` set point `y` at which
`φ`'s symmetric air at `y` plus `(0.2 − τ_o)·D_φ(y)` equals it — solved by Illinois between the floor and the
symmetric set point. As a fraction of the way from the symmetric set point down to the floor: **0.11–0.77**
(0.11–0.25 at `τ_o` 0.05, 0.13–0.66 at 0.02, 0.31–0.77 at 0.01; the two steps within 0.02). So H5(a) is,
to the drift, *"at `τ_c` 0.2 the `φ` valve's re-tuned set point has come at least that fraction of the way
to its floor"* — which H3a measures anyway. The rows nearest the floor are the large-`R` ones at `τ_o`
0.01 (shaped 1/.42 0.765, tilted 1/.46 0.753, shaped 1/.46 0.742). **Decided before measuring: all 40 stay
in H5(a).** A 20× slower close against a 0.01 opening holds most of its bleed through the `φ` valve's late
minimum, so D5's floor should bind there; at `τ_o` 0.05 the closing is only 4× slower, but the fractions
needed are small.

**G3 — the census (by compiler).** On a scratch copy with `BleedLimiter::tau` made private, the compiler
lists every reader of the valve's time constant: rung 65's march and predicate (`lagged_bleed.rs`), its
marginal-mode reader of a freshly built symmetric valve, rung 88's reader (`lagged_incidence.rs`), rung 80's
rebuild (`split_wall.rs`), and the marches of rungs 66–75 (`two_lag`, `cross_loop`, `three_loop` ×2,
`cross_split`, `full_split`, `shared_actuator`, `demand_coordinate`, `anti_windup`). Every one of the latter
solves its valve through `r64_solve_b` — the text search had missed three of them (`three_loop` ×2,
`two_lag`).

## 1. THE DEVICE — rung 65's lag with a second time constant

`db/ds = (b_cmd − b)/τ_open` while `b_cmd ≥ b` (opening, or holding), `(b_cmd − b)/τ_close` while `b_cmd < b`
(closing). No new law and no new state: the same command (rung 64's root against the valve's own target —
rung 87/88), the same start (`b(0) = b_cmd(0)`), the same clamp.
- `BleedLimiter::tau_close: Option<f64>` — `None` closes at `tau` (rung 65's valve, bit for bit); set only
  through `closing(τ_c)`, which refuses an unlagged valve. `tau` is then the opening constant.
- `BleedLimiter::lag_rate(cmd, q)` is the rate law, used by rung 65's march and by the sandbox's replay of it
  (so the two stay one spelling). `tau_min()` feeds rung 65's RK4 bound `ds/τ ≤ 2.0` — the SMALLER constant.
- **The refusals (G3).** `r64_solve_b` refuses a valve carrying `tau_close`, as it refuses an incidence
  valve (rung 88): rungs 66–80 march ONE constant. `lagged(τ)` keeps `tau_close` (as it keeps the reference),
  so rung 67's readers carry an asymmetric valve to that refusal; rung 80's rebuild now carries both.
- **Reading:** rung 88's `lag_read` generalises its identity residual (below) and reports `U`, `D`; the
  census marks the first closing read (`first_closing_read`).

## 2. DERIVATIONS

**D1′ — the bill, edge by edge (arithmetic, NO credit; a GATE, its residual bar measured on the first asymmetric runs — not a prediction).** On each edge `ḃ` has the sign of
`b_cmd − b`, so `b − b_cmd = −τ(s)·ḃ` with `τ(s)` that edge's constant, and

```
    ∫ (b − b_cmd) ds = τ_close · D − τ_open · U        (U, D: total opening / closing travel)
```

exact in the continuous march, broken only by the clamp; with `τ_open = τ_close` it is rung 88's D1,
`τ(b0 − b_end)`, since `U − D = b_end − b0`. The equivalent `∫(b − b_cmd)/τ(s) ds = b0 − b_end` is what the
reader checks (its residual is rung 88's, bit for bit, on a symmetric valve). So **a slow CLOSING edge adds
`τ_close·D` of air and a slow OPENING edge takes `τ_open·U` away**, plus the drift (the command re-read along a
moved trajectory). The symmetric valve's bill `τ·b0` was the difference of two edge bills.

**D2 — rung 65's argument, made exact.** The march is bit for bit independent of `τ_close` up to the first
RK4 stage that finds `b_cmd < b` (the census mark): no earlier operation reads it. So a valve's run-minimum is
unchanged by `τ_close`, bit for bit, wherever (i) that minimum is recorded before the mark and (ii) nothing
after the mark falls below it.

**D3 — the incidence valve cannot debit its own wall on its closing edge (construction).** While closing,
`b > b_cmd`, and `φ_LP` rises with `b` at fixed state (rung 64's premise), so the incidence valve holds MORE
than its target there: `M_i ≥ m_lim` pointwise wherever `b ≥ b_cmd` — with the command interior (`φ(b_cmd)` IS
the target) or dormant (`φ(0)` already meets it, and bleed only adds). Rung 52's *"a self-releasing limiter
cannot debit the spool it watches"*, on an airflow lever. So after the mark the incidence valve can lose its
wall only by RE-OPENING (`b < b_cmd` again), and G2 counts no re-opening in any symmetric run. With G1
(minimum first) it gives (ii) of D2 for the incidence valve at ANY `τ_close`: **its protection, and so its
re-tuned set point, depend on `τ_open` alone** — up to G1′'s onset stage and any re-opening a faster close
might start. The `φ` valve over-holds `φ`, not incidence, so D3 says nothing about it — its
closing edge is where rung 88 found its CREDIT (G1: minimum after closing in 56 of 95).

**D4 — the forecast.** Where BOTH valves' minima precede their first closing at `τ_open` (G1: 39 of 95 (cell,
`τ_open`)) and stay the minimum, both set points are those of the symmetric valve at `τ_open`, and D1′ gives
each bill at `τ_close` as the symmetric one plus `(τ_close − τ_open)·D` plus a drift. So

```
    R_f(τ_o, τ_c) = (A_φ(τ_o) + (τ_c − τ_o)·D_φ(τ_o)) / (A_inc(τ_o) + (τ_c − τ_o)·D_inc(τ_o))
```

— the closing edge moves `R` toward `D_φ/D_inc` (G2). Where the `φ` valve's minimum is on its closing edge,
a slower close also buys it protection (rung 88 § 5's credit), lowering its set point and its air: `R` falls
BELOW `R_f`.

**D5 — how far the credit can go: the floor (G4).** A slower close holds more bleed after the mark, which
lifts the `φ` valve's late minimum; re-tuning then lowers its set point — until the late minimum rises above
the minimum BEFORE the mark, which no closing speed touches (D2). From there the set point is the floor,
exactly. So, as `τ_close` grows, the `φ` valve's set point falls and then stops at G4's floor; and with air
rising with the set point, `R ≥ R_floor` (G5) up to the drift. **Nothing like it exists for the incidence
valve (D3): a slow close only costs it air.** The asymmetry of the two walls is the asymmetry of what each
valve's closing edge can do for it.

## 3. PREDICTIONS — registered before measuring (bars frozen here)

**Scored grid:** rung 88's 19 cells × every `(τ_open, τ_close)` ∈ {0.01, 0.02, 0.05, 0.1, 0.2}², the 20
OFF-diagonal pairs scored (the diagonal is rung 88 — the reduce). Both valves re-tuned to `m_lim` (rung 88 §
4's currency, Illinois tol 1e-12), stators on. **THE STEP-SIZE BAR:** every H must hold at `ds` 0.005 AND
0.0025 (`ds/τ_min` ≤ 0.5); one step only = GRID-SENSITIVE. **Equal length:** integrals over the stretch both
runs reach. **Preconditions per run:** clamp hits = 0; a run breaking it is reported and left out. Thrust
integrals reported, not scored.

- **H1 (the incidence valve's protection lives on its opening edge — D2 + D3 + G1; CONSTRUCTION but for an
  onset stage or a re-opening).** Its re-tuned set point at `(τ_o, τ_c)` equals the symmetric valve's at `τ_o`
  to `|Δφ_lim| ≤ 1e-9`, and its minimum stays before its first closing read, in all 380 (cell, pair). G1′'s
  onset-stage run (shaped .25/.34, `τ_o` 0.02, fine step) is in the scored set: if it misses it is scored a
  MISS, with D2's onset stage as the stated reason beside the score. Ranked with H2.
- **H2 (the closing bill — D1′; arithmetic).** At that set point, `A_inc(τ_o, τ_c) − A_inc(τ_o, τ_o) −
  (τ_c − τ_o)·D_inc(τ_o)` is within `max(0.05·|τ_c − τ_o|·b_peak, 1e-5)` in all 380 (`b_peak`, `D` of the
  symmetric run). Ranked last.
- **H3 (the `φ` valve's credit lives on its closing edge, down to a floor — rung 88 § 5, D5).** (a) In the 56
  close-first (cell, `τ_o`) of G1, its re-tuned set point is NON-INCREASING in `τ_c` over all five values,
  never below G4's floor (by more than 1e-9), strictly decreasing while above the floor, and EQUAL to the
  floor (`≤ 1e-9`) at every `τ_c` from the first one at which its minimum precedes its first closing read. Per
  cell, the `τ_c` at which the floor binds is reported. (b) In the 39 min-first, the set point is unchanged
  (`≤ 1e-9`) for every `τ_c > τ_o` — a slower close holds more bleed after the minimum, which raises `φ` at
  each point; NOT construction over the run (the extra bleed moves the spools and the stator setting, and the
  slack can be 4.4e-5). `τ_c < τ_o` there is reported, not scored.
- **H4 (the bounds — D4, D5).** (a) In the 39 min-first (cell, `τ_o`), for every `τ_c > τ_o`: `|R/R_f − 1| ≤
  0.02`. (b) In the 56 close-first: for every `τ_c > τ_o`, `R_floor·0.98 ≤ R < R_f`; for every `τ_c < τ_o`,
  `R > R_f` (the credit lost).
- **H5 (THE CONTENT — a slow CLOSING edge reverses the verdict at a FAST opening).** (a) In the 40 (cell,
  `τ_o`) of G5 where the bounds straddle 1 (the `φ` valve's minimum after its first closing, `R(τ_o, τ_o) >
  1`): `R(τ_o, 0.2) < 1`. A fast-opening, slow-closing valve makes the `φ` valve the cheaper one where the
  symmetric valve at the same opening speed keeps rung 87's verdict: the slow close lifts the `φ` valve to
  (near) its floor and buys the incidence valve nothing (D3, D5). So **the crossover moves to FASTER opening
  as the closing slows** — the seam's "which way". (b) Where both of G5's bounds lie on one side of 1 (every
  `τ_c > τ_o` pair outside (a): 26 min-first + 31 close-first), `R` is on that side; (a)'s 93 intermediate
  `τ_c` are reported, not scored. (c) For every `τ_c < τ_o` (190 (cell, pair)),
  `R` keeps the sign of `R(τ_o, τ_o)`: a FAST close strips the `φ` valve's credit (H4b) but not enough to
  flip the verdict — the least bounded claim here, and the likeliest miss (shaped 1/.42 at `τ_o` 0.05 sits
  at `R` 0.946, `R_f` 0.928).
  The prediction I first drafted was the opposite of (a) — "the OPENING edge decides who is cheaper" (rung
  88 § 5's split read naively). G4–G5, measured before this was frozen, are why it was changed: on the
  close-first cells nothing symmetric bounds the sign, and the floor sits far below 1 in all 40.

**Size: NO bar** beyond H2's and H4's.

**Reduces (gates, not predictions):** `tau_close = None` ≡ rung 65/88 bit for bit (every rung 64–88 gate and
oracle unchanged, no CLI golden moved); `closing(τ)` ≡ symmetric bit for bit (G0); the diagonal of the grid
reproduces rung 88 § 4's table; the default design run stays rung 6.

## 4. THE KNOBS (sandbox, same batch)

The Controls view's slow bleed valve gains a **closing speed**: "closes at the same speed" (today's, the
default — every saved setting stays valid) or its own time constant. Built through rung 65's builder and
march, replayed bit for bit (the replay uses the same `lag_rate`); the step precheck reads the SMALLER
constant at the page's own step (rung 88's lesson: measure the floor at the product's step); fuel legs rung
65 refuses stay refused. Gates follow rung 88's: replay bit for bit, `check_requests` cases, a `browser.mjs`
case, a crash map.
