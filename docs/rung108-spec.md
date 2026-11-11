# Rung 108 — THE TWO-SPEED TEMPERATURE LIMITER: rung 52's fast-cut, slow-give-back lag on the REDLINE clip, beside the governor and on the open throttle

**Code:** `rust/src/two_speed_clip.rs` — `try_der` / `try_rk_step` / `march`: rung 106's three-state march `(ν_L, ν_H,
g)` (copied, rung 106's module untouched) with the clip's rate `(r − g)/τ_att` while `r > g` and `(r − g)/τ_rel`
otherwise (`AsymmetricLag::tau`, rung 52's), the clip and its target recorded as `PointExtra::Asym { g, required }`; the
demand rung 106's `Demand` — rung 95's governor or the open ramp. The Controls view (`sandbox_controls.rs`): the
settings `gov_lag_two` / `gov_lag_att`, `two_speed()`, `Route::GovernorTwoSpeed` / `Route::TwoSpeed`,
`two_speed_clip` / `two_speed_der` / `two_speed_step`, the refusals; the governed two-speed route on the 0.005 step
always (`march_ds`). The page (`docs/sandbox/template.html`): the tick box *"…cuts fast, gives back slowly"* under the
limiter's lag, its cut-in slider, the lag knob relabelled *"Giving back"* when ticked; the box **starts ticked on the
page** (the defaults reply), off in the model. **Gates:** `rust/tests/rung108.rs`. **Anchor:**
`docs/plans/rung108-anchor-two-speed-limiter.md`, § 3 frozen at `e14a723` before any rung-108 code (§ 2 amended after
the build, dated). **Scratch:** `docs/plans/scratch-archive/rung108/` (`score.rs`, `score2.rs`, `crash.rs`, `diag2.rs`
and their outputs are the numbers below).

The seam: rung 107's (*"rung 52's asymmetric lag on the REDLINE (106 lagged it with one time constant; 107 left it
instant): does a fast attack on the temperature limiter end rung 106's overshoot while a slow release keeps its
sag?"*). The user's choices (memory `rung108-decisions`): BOTH routes, governed and open; ALL holds; the tick box with
the existing lag as the give-back time; the cut-in floor 0.02 τ; beside the governor ALWAYS the 0.005 step; the cost
measured and reported before shipping (governed 3.3 s on the page against rung 106's 0.6 s — shipped as is); rung
106's own roughness recorded and fixed AFTER this rung; the box ticked by default on the page.

> **THE HEADLINE — a fast cut ends most of the overshoot, but it STORES more, so beside the governor the slow
> give-back is paid in THRUST TIME.** At a 0.02 τ cut-in the governed slam runs 59–63 K over the redline, against
> 265–282 K for rung 106's single lag at 0.2 (and 0 K for the instant limiter); on the open throttle 24 K against
> 135–141 K. The overshoot falls with the cut-in time in the ratio a first-order lag predicts (0.30 governed, 0.26 open,
> 0.02 against 0.1). But the faster cut tracks its target to a higher peak — the cut at the crossing is 2.1–2.2× a 0.2 cut's
> — and giving that back slowly withholds `τ_rel·(g_c − g_end)` of fuel: beside the governor the thrust still SAGS
> after the move (2.0–4.0 % of the step at 0.02, more with a slower give-back), reaches 95 % at 2.1–2.2 τ against 1.2–1.3
> for the instant limiter at give-back 0.2, and **not within the 2.5 τ run at 0.4 or 0.8** (87–88 % / 73–75 % reached). On the
> open throttle it never sags. **The seam's question answered: YES and YES — the overshoot mostly goes, the sag stays,
> and its size is the give-back's.** At rung 106's own lag it sags less than rung 106 at 0.2 and 0.4 — but MORE at 0.8,
> because the slow single lag barely cuts (it lets the engine run 366–399 K hot) and so has little to give back.

## 1. What it adds

One line on rung 106's march: the clip's lag constant chosen per evaluation, `τ_att` while the target is above the
clip, `τ_rel` otherwise (the tie `r = g` takes `τ_rel`, continuous there). The demand, the other legs (schedule, floor,
metal — instant, off the demand), the minimum-fuel hold and the recorded keys are rung 106's. At equal constants it IS
rung 106's march (governed) and rung 47's (open), bit for bit.

**The ledger (D1)** splits by phase: with one crossing at `g_c`, the fuel let through above the instant redline's cut on
the way up is `τ_att·g_c`, the fuel withheld below it on the way down `τ_rel·(g_c − g_end)`. Rung 106's cancellation is
gone: per unit of stored cut they stand in the ratio `τ_rel/τ_att`.

## 2. Gates (anchor § 2) — all pass

* Off ⇒ every earlier run, bit for bit; the tick box alone (lag off) moves nothing.
* Equal constants ⇒ rung 106 (governed, fine step and page step) and rung 47 (open), bit for bit.
* **The pinned trigger, as amended (2026-11-11).** Governed (0.005 step): every point before the first crossing is
  `τ_rel`-free and rung 106's at `τ = τ_att`, bit for bit, the overshoot with it, on every § 3 cell. **Open (0.02
  step): an inner RK4 STAGE reads `τ_rel` before the recorded crossing, in two places** — the crossing step at cut-in
  0.02 (every shape, one step early: rung 107's lesson), and **a new case, the ENGAGEMENT step** (shape 0 at cut-in ≤
  0.05: at s = 0.29 a mid-step stage puts `g` four times above a target just leaving zero, twelve steps early). So the
  claim holds up to the SPLIT (the first point where give-back 0.1 and 0.8 differ), pinned per cell; from it to the
  crossing within 5e−4 relative of rung 47 (measured 2.2e−4); the overshoot at the same point within 1e−3 K (measured
  1.6e−4 K of 24 K; its spread over give-back 2.8e−5 K). The law was kept (per evaluation, as rungs 52 and 107); the
  gate states what the march does (`diag2.txt`).
* Both constants → 0 ⇒ rung 95's instant redline: gaps 8.3e−3, 4.5e−3, 2.3e−3, order 0.89 / 0.97.
* The ledger per phase within its frozen bars: governed worst attack miss 1.2e−3 of `τ_att·g_c` (bar 5e−3), open
  1.3e−2 (bar 3e−2).
* The governed route on the 0.005 step, the reply carrying it; every step re-runs bit for bit (six settings); the
  refusals in words; no minimum-fuel hold, one attack per run, on the default grid.

## 3. Scored (anchor § 3, every run on the 0.005 step) — 144 HIT, 4 MISS, 12 VACUOUS (160 cells)

| Claim | Cells | Result |
|---|---|---|
| H1 the overshoot falls with the cut-in time; 0.02 / 0.1 in [0.2, 0.5] | 16 | **16 HIT** (ratios 0.30–0.31 governed, 0.255–0.257 open) |
| H2 the governor overshoots more than the open ramp | 12 | **12 HIT** (2.0–2.5×) |
| H3 the faster cut stores more (`g_c` falls as `τ_att` rises) | 4 | **4 HIT** (0.02: 1.63–1.72e−2; 0.2: 0.75–0.80e−2) |
| H4 governed, a fast cut still sags (> 0.002); more with `τ_rel` | 36 + 12 | **48 HIT** (0.020–0.059) |
| H5 at rung 106's lag, the fast cut sags less than rung 106 | 12 | **8 HIT, 4 MISS** (every map at 0.8) |
| H6 settles later than instant; later with `τ_rel` | 12 + 4 | **4 HIT, 12 VACUOUS** (0.4 and 0.8 never reach 95 %) |
| H7 the open ramp never sags | 36 | **36 HIT** (drawdown 0 everywhere) |
| H8 the LP minimum before the crossing; rising as `τ_att` falls | 12 + 4 | **16 HIT** |

**H5's miss is the named risk (H3):** rung 106's single lag at 0.8 stores `g_c` 2.7–3.0e−3, the fast cut 1.66–1.72e−2
(5.5–6×); giving that back at 0.8 withholds more than the slow cut ever stored. The drawdowns are unchanged with a 4 τ
settle (`score2.txt`), so it is not the window. **The trade, at 0.8: 59 K over the redline and a 3.5–4.0 % sag, or
366–399 K over and 2.5–3.4 %.**

**H6's vacuity is the headline's cost:** at give-back 0.4 the thrust reaches 87–88 % of its step by the run's end
(2.5 τ), at 0.8 73–75 %. With a 4 τ settle (off protocol) 0.4 settles at 3.0–3.1 τ, 0.8 still not (92–93 % at 4.5 τ);
rung 106 at the same lags reaches 98.6–100 %.

**H8's sign:** the LP's closest approach comes before the crossing and gets closer to the instant limiter's as the cut
quickens (0.02: −0.0009 to −0.0016 of flow coefficient below instant; 0.1: −0.0021 to −0.0036; 0.2: −0.0026 to −0.0046)
— every lagged cut costs the LP margin against the instant redline, a fast one least.

## 4. After scoring (reported, not scored)

* **The ledger, governed and open, at 0.02** (`score.txt`): let-through within 1.2e−3 (relative) of `τ_att·g_c`, withheld within
  1e−4 of `τ_rel·(g_c − g_end)`.
* **The other laws at (0.02, 0.4)** (HP rate 0.1, HP gain 10, the pressure ratio, fan-speed rate 0.15 / gain 7, four
  maps): overshoot 43–96 K against rung 106's 212–492 K at 0.4; every run sags (0.005–0.050), every one LESS than rung
  106 at 0.4.
* **The burner-temperature hold to 1600 K** (its demand crosses the redline): overshoot 24 K against rung 106's 106 K;
  no crossing — the demand stays above the redline, so the cut never gives back and the run ends held at 1480.1 K.
* **Crash map** (`crash.txt`, seed 108, 1 000 requests, half governed): 0 panics; 482 admitted (180 governed, 302
  open), 118 stopped early, every stop a named kind (91 the floor out of reach, 21 rich, 3 lean, 3 the redline beside
  the floor), no minimum-fuel hold; 518 refused (45 two lags at once, 26 the give-back floor beside the governor, 73 the
  speed laws' edges, 57 hot starts, the rest earlier rungs'). **12 runs cut twice** — all open, all throttle pull-backs
  starting above the redline — and that is **rung 47's own**: its single lag at the cut-in time cuts twice on all 12, at
  the give-back time on 8 (`reatt.txt`, read off the applied cut, which matches the recorded clip on all 12). Slowest
  7.3 s natively, 6.7–10.4 s in WebAssembly (`wasmtime.txt`).
* **Cost** (the user's condition; measured on the page in the browser check, the default slam holding thrust with the
  lag at 0.2): governed two-speed 3.3 s per run against rung 106's 0.6 s (the 0.005 step: four times the points);
  open 0.20 s against rung 47's 0.16 s. Natively 0.5–3.0 s (`timing.txt`), and the crash map's five slowest requests
  take 6.7–10.4 s in WebAssembly through the page's own call path (Node, `wasmtime.txt`; not in the browser itself,
  under the page's 15 s line). Shipped as is (the user, 2026-11-11).

## 5. What it does to its neighbours

* **Rung 106** — its overshoot (and most of its sag at 0.2) was the SLOW CUT's: a fast cut takes the overshoot from
  272 K to 59 K at the same give-back and halves the sag. BOUNDED: at a slow give-back (0.8) the fast cut sags more.
* **Rung 107** — "a fast cut leaves the release's" CONFIRMED on a second leg (the redline): at 0.02 the sag rises
  steadily with the give-back on every map. Its finding that the slow release costs TIME is sharper here: 95 % is not
  reached at all within the run at 0.4.
* **Rung 47** — "the cost of realism: it breaks the redline hold (~55 → 190 K)" is a slow cut's: at 0.02 the open
  overshoot is 24 K. Its trailing-edge verdict survives: the LP's closest approach comes before the crossing.
* **Rung 52** — the pinned trigger is exact at the fine step; at the coarse step the march's own stages read the
  release constant early, now also at the engagement step.

## Concessions

* The open route is scored on the 0.005 step (one grid for every comparison) but runs on the page at 0.02; the page's
  open numbers agree within the step check (anchor § 0 G3), and its pinned trigger holds only to the split (§ 2).
* The 2.5 τ run cuts off the give-back at 0.4 and 0.8 (`g_end/g_c` 0.19–0.25 and 0.42–0.52), so H6 there is VACUOUS
  and the longer-settle reading is off protocol.
* The cut-in floor (0.02) and the give-back's floor beside the governor (0.1) are numerical (the step check), not
  hardware times.
* The governed page run is ~5× slower than rung 106's; the slowest settings were timed in WebAssembly under Node (6.7–10.4 s), not in the browser.

## The next seam

**Rung 106's own roughness at the page step** (anchor § 0 G3, sandbox plan § 43's OPEN item): the shipped single lag
beside the governor with the floor or the schedule is rough at one step (to 6.2e−4) — the user's choice is to fix it
next. Then **the governor beside rung 88's slow valve** (still refused).
