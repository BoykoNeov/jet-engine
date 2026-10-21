# Rung 106 — THE GOVERNOR BESIDE THE SLOW TEMPERATURE LIMITER: rung 95's governor as the demand rung 47's lagged cut is measured off

**Code:** `rust/src/governor_lag.rs` — `ClipLag` (the redline and its lag), `Demand` (the governor, or the open ramp —
the gate's second code path), `try_der` / `try_rk_step` / `march` (the three-state march, the clip recorded as rung 52's
`PointExtra::Asym { g, required }`); `governor.rs` — `Leg::Lag`, `Leg::MinFuel`, `wall` made public. The Controls view
(`sandbox_controls.rs`): `Route::GovernorLag`, `ControlsSolver::{clip_lag, lag_limiters, gov_lag_der, gov_lag_step}`,
`Holder::MinFuel`, `GOV_LAG_MIN`, the refusals. **Gates:** `rust/tests/rung106.rs`. **Panel:** `print_governor_lag_table`
(born in Rust, `rust/oracle/rust_owned/`, a change detector only). **Anchor:** `docs/plans/rung106-anchor-governor-lag.md`,
committed as `ca5504f` before any rung-106 code. **Scratch:** `docs/plans/scratch-archive/rung106/`.

The seam: rung 95's (*"…the governor on the lagged marchers (47/52/88), where its demand meets a lag's state"*). The
user's choices (memory `rung106-decisions`): rung 47's slow temperature limiter ONLY (rung 52's release and rung 88's
slow valve stay refused, later steps); every hold of rungs 95/96; rung 101's lagged exhaust sensor beside it refused for
now; where the lag's leftover cut would take the fuel below what the engine burns, HOLD it there (a minimum-fuel stop)
and mark it; a run whose start sits above the redline REFUSED in words (a working limiter would never let the engine
settle there) — and, asked again after the crash map, a run whose GOVERNOR'S REQUEST at the start is above the redline
while another limit holds the engine lower (the metal cap), refused too.

> **THE HEADLINE — a stateless governor beside a lagged limiter WINDS UP: the cut it stores while it climbs is paid
> back after the move, so the thrust it holds climbs and then SAGS.** Rung 95's "nothing winds up" was a fact about the
> governor; the limiter's lagged cut is measured off the governor's demand, and that demand rises faster than the open
> ramp, so the cut both lets through MORE (the redline overshoot about doubles: 272 K against rung 47's 138 K at the
> default lag) and stores MORE (the fuel later withheld below the instant limiter: about twice the open ramp's). After
> the move the governor's demand falls back and the stored cut lags it down: on the default slam the thrust reaches 83 %
> of its step at the end of the move, falls to 78 %, and reaches 95 % at 1.53 τ against 1.32 τ with an instant limiter —
> later, and hotter on the way. On the open ramp the thrust never falls; with an instant limiter it never falls. The
> fuel ledger is exact: the extra fuel let through and the fuel withheld, measured against the instant limiter at the
> same states, cancel up to `τ·g` at the end (`∫(r − g) ds = τ·Δg`).
>
> **AND IT MOVES RUNG 47'S REFUTATION — which was timing.** Rung 47's lag never reached the LP compressor (relief exactly
> 0) because the LP's closest approach to stall comes before the limiter engages. Under the governor the limiter engages
> at s = 0.14 instead of 0.30 (every map), before that approach, and the lagged limiter now buys the LP a small relief
> (+0.0004 to +0.004 in flow coefficient, shrinking with the lag).

## 1. What it adds

**The march** — rung 95's governed march with rung 47's clip `g` as a third state. At each evaluation the governor's
fuel `G(ν_L, ν_H, s)` (rungs 95/96's laws, unchanged and stateless), then

```
r     = G − T(ν; Tt4_max)   if Tt4 at G is above the redline, else 0     (rung 46/47's guard, off the DEMAND)
dg/ds = (r − g)/τ
u     = min(G − g, the other armed legs solved off G)                     (rung 47's pattern)
```

`T` is rung 46's topping fuel. The schedule (48), the floor (49/60) and the metal leg (95) stay instant, solved off `G`.
An applied fuel below the leanest that runs is held at that wall (`Leg::MinFuel`, bisected as rung 95's walls).

**On the page** (the Controls view): *Govern during the move* and the temperature limiter's *…that reacts with a lag*
now run together (`route: "governor_lag"`); the top strip marks the clip (`lag`) and the minimum-fuel stop (`min_fuel`).
Refused in words: beside the realistic release or the slow valve (as before); beside rung 101's lagged exhaust sensor
(two lags at once — another session); a start above the redline, or a governor's request above it at the start; a lag
under **0.1 τ** (`GOV_LAG_MIN`, the step check
below — rung 47's open route keeps its one-step floor).

## 2. Gates (anchor § 2, bars frozen at `ca5504f`) — all pass

| gate | measured |
|---|---|
| the open ramp as the demand IS rung 47's march | bit for bit at every point, four maps × lag 0.02 / 0.05 / 0.2 / 0.8 |
| before the clip engages, rung 95's march; a redline above the governed peak, rung 95 whole | bit for bit (thrust and HP speed, flow/press and flat-LP; redline 2500 K) |
| the burner-temperature hold below the redline never engages | `g ≡ r ≡ 0` |
| a vanishing lag tends to rung 95 with the instant redline | largest speed gap 8.3e−3 / 4.5e−3 / 2.3e−3 at `τ` 0.02 / 0.01 / 0.005 (step `τ/4`) — first order (0.89, 0.97) |
| the clip's ledger (D1), bar `2e−2·τ·max g` | 1.3e−3 … 5.9e−3 of `τ·max g`, four maps × five lags; 3.2e−6 → 8.3e−7 when the step halves (second order) |
| the lean hold at model level (1700 → 900 K, redline 1480, called directly) | holds at the wall (runs; 1e−9 leaner does not), marked, runs to the end; the page refuses the request |
| every lagged governed step re-runs from its recorded point | bit for bit (four settings) |
| step check at the floor | under 1e−4 in flow coefficient on every hold at `τ` 0.1 (largest 6.9e−5, the fastest speed law); see § 4 |

## 3. Scored (anchor § 3) — every claim HIT

The default slam with the redline 1480 K, the thrust law unless stated, "open" = rung 47 (the new march with the ramp
handed in, licensed by the first gate), "instant" = rung 95 with the instant redline. The 0.05 lag is run by calling the
march directly (the page's floor is 0.1). Full table: `scratch-archive/rung106/score.txt`.

| # | claim | verdict | measured |
|---|---|---|---|
| H1 | governed overshoot > open | **20/20** | flow/press: 123 / 194 / 272 / 340 / 388 K against 56 / 95 / 138 / 170 / 191 K (`τ` 0.05 … 0.8); the other maps within 7 % |
| H1b | the same under every other law (`τ` 0.2) | **20/20** | HP rate 353–368 K, HP proportional 465–477, pressure ratio 310–329, fan speed rate 143–267, fan speed proportional 223–322, against 135–141 open; smallest margin the fan-speed rate law on flat-LP (143 vs 135) |
| H2 | the overshoot rises with the lag | **4/4** | strictly, every map |
| H3 | governed stored cut > open | **20/20** | `max(g − r)` 0.95e−3 … 1.93e−3 against 0.30e−3 … 0.99e−3 (fuel, kg/s per kg/s of design air) |
| H4 | the 95 % thrust later than instant | **20/20** | flow/press 1.40 / 1.46 / 1.53 / 1.57 / 1.47 τ against 1.32 |
| H5 | the redline engages earlier | **4/4** | point 7 (s = 0.14) against 15 (0.30), every map |

H4's named risk, measured past the grid (reported): the delay peaks near `τ` 0.4 and the sign FLIPS between 0.8 and 1.0
(flat-LP: 0.8 and 0.9) —
there the thrust's first climb, before the stored cut pays back, already passes 95 % (0.954 at `τ` 1.0), and the crossing
jumps to ~0.50 τ (the unlimited governor's 0.475). "Later" is a property of lags up to ~0.9 τ; a longer lag reaches 95 %
during the move and sags after it (to 92 % at `τ` 1.0).

## 4. After scoring (reported, not scored)

* **The sag** (flow/press; press/flow and tilted within 0.003, flat-LP a little shallower): thrust share at the move's
  end → lowest after it: 0.829 → 0.776 (`τ` 0.2), 0.897 → 0.850 (0.4), 0.943 → 0.910 (0.8) — flat-LP 0.840 → 0.797,
  0.904 → 0.868, 0.947 → 0.922 — the lowest at s = 0.64–0.82. Rung 47's open ramp: never falls; rung 95's
  instant limiter and the unlimited governor: reach the thrust and hold it. Pinned in
  `h4_the_lag_makes_the_governed_thrust_later_and_the_thrust_sags_after_the_move`.
* **Rung 47's reliefs under the governor** (lowest flow coefficient, lagged against the unlimited governor): LP
  +0.0025 / +0.0016 / +0.0011 / +0.0008 / +0.0004 at `τ` 0.05 … 0.8 (flow/press; the open ramp's exactly 0), HP +0.015 …
  +0.002 (open +0.0036 … +0.0009). H5's earlier engagement reaches the LP's approach — rung 47 § 2's mechanism, now
  read from the other side.
* **The burner-temperature hold is the exception** (1000 → 1600 K, so its demand crosses the redline): the governed
  overshoot is SMALLER than the open ramp's (76 K against 200 K at `τ` 0.1; 106 against 419 at 0.4) — a temperature
  demand has no fuel overshoot of its own for the clip to chase (rung 95: "a commanded `Tt4` cannot overshoot itself").
  The peak is 1556.2 K on every map to 0.07 K: the clip's error in temperature depends on the demand line, not the maps.
* **The step check set the page's floor.** At one step (0.02, rung 47's own floor) the governed march moves the flow
  coefficients by ~2.0e−4 against a step four times finer under thrust and HP speed (the open ramp 9.4e−5); at three
  steps the fastest speed law (`τ_N` 0.04) still 2.0e−4 — two fast clocks together — and from 0.1 every hold is under
  1e−4. So the governed lag starts at 0.1 τ.
* **Cost:** a governed lagged run 0.23–0.30 s natively, the same as rung 95 with the instant redline (0.26–0.32 s).
* **The crash map** (`scratch-archive/rung106/crash.txt`, 1 000 requests, seed 12 — every one governed with the
  redline and its lag on, the lag 0.02–1.0, the redline 1200–1800 K): **no panic, no "other" stop.** Its first run: 499
  ran (430 to the end, 69 stopped early — 67 the stall floor out of reach, 2 the redline's own floor), 501 refused in
  words (149 a start above the redline, 78 the lag under 0.1, the others rung 94–96's refusals and endpoints with no
  steady point). **One run reached
  the minimum-fuel stop** with its start BELOW the redline (#896): the metal cap held the start at 978 K while the thrust
  governor's request was already above 1480 K, so the clip charged from nothing under the metal leg and the fast slowdown
  (1250 → 690 K over 0.1 τ) held the burner at the lean edge for ~0.6 τ — a limiter out of authority still winding up.
  Asked; the user refused that start too (gated: `the_page_refuses_a_governor_whose_request_starts_above_the_redline`).
  **Re-run with that refusal** (`crash.txt`): no panic, no "other" stop, **no minimum-fuel stop**; 5 requests newly
  refused by it (#896 among them; 425 ran to the end, 69 stopped early as before). Its timings are not quoted (the
  whole gate ran beside it).
* **Its slowest requests (15–20 s) are not the lag's — they are a REGRESSION of rung 95's route since it shipped.** The
  five slowest, all with the HP-shaft generator, take the same time with the lag off (14.6–17.4 s). The same requests
  (stripped of the keys rung 95 did not know) on rung 95's own commit `096367d`: 0.23 / 0.42 / 1.37 / 1.84 s (one refused
  there, by a rule since lifted), against 21–37 s on today's code, timed back to back — 20–150× slower. Not this rung's;
  not traced here (candidates: rungs 103–105's turbine laws, the solver repairs of sandbox plan §§ 36–42). On the page
  such a run reads as a hang. Recorded as an open item (`docs/plans/sandbox-plan.md` § 43).

## 5. Why — the model

Anchor § 3. The lag's error `e = r − g` obeys `de/ds = r′ − e/τ`, so the overshoot is the required cut's growth rate
`r′ = G′ − T′` filtered at `τ`, and the stored cut its fall rate. The governor's demand rises faster than the ramp (rung
95 H1) — larger `r′`, larger overshoot — and, once the move ends, falls back toward the end's fuel while the open ramp
only stops — a more negative `r′`, a larger stored cut. The ledger `∫ e ds = τ·Δg` makes the two the same fuel: what
the lag lets through on the way up it withholds on the way down, and a governor, unlike a ramp, has a way down.

## What it does to its neighbours

* **Rung 95 — "nothing winds up" BOUNDED** to the governor itself: beside a lagged limiter the system winds up through
  the limiter's state. Its concession "plain march only" discharged for rung 47's lag.
* **Rung 47 — its LP refutation is TIMING, confirmed from the other side:** move the engagement upstream (here by the
  governor) and the lagged limiter does reach the LP, a little. Its cost-of-realism finding (the redline broken, growing
  with the lag) holds and roughly doubles.
* **Rung 46/47 — the same clip, a different demand:** the open ramp's thrust never falls under the lag; the governor's
  does — the sag needs a demand that comes back down.

## Concessions

* Rung 47's lag only; rung 52's release, rung 88's slow valve and rung 101's lagged sensor stay refused beside the
  governor.
* The minimum-fuel stop is the fuel closure's lean wall (fuel-air 0.004), not a flame-out model; only a start above the
  redline reached it (anchor G4), and the page refuses that start.
* Inherited: rung 95's (the governor reads thrust directly unless rung 96's reading is chosen; stateless laws; the
  clamps are the closure's walls; a straight-line demand), rung 47's (a first-order lag on the clip AMOUNT, the
  limiter-loop lag, not the valve), invented maps, the perfect gas.

## The next seam

**The governor beside rung 52's realistic release** (fast attack, slow release) — the release edge is where the stored
cut is paid back, so it decides the sag; or **beside rung 88's slow valve** (a lag on the air, not the fuel); or **beside
rung 101's lagged exhaust sensor** (two lags on the governor's own loop).
