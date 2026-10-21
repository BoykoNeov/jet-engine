# Rung 106 anchor — THE GOVERNOR BESIDE THE SLOW TEMPERATURE LIMITER (pre-registered 2026-10-21 — § 3)

**Seam:** rung 95 § *The next seam*: *"…or **the governor on the lagged marchers** (47/52/88), where its demand meets a
lag's state."*; its concession: *"Plain march only: refused beside rungs 47/52/88's lagged marchers."*

**The user's choices (2026-10-21):**
* **Which lag:** rung 47's slow temperature limiter ONLY. Rung 52's realistic release and rung 88's slow valve stay
  refused (later steps).
* **Rung 101's lagged exhaust sensor beside it:** left for another session — refused in words.
* **Holds:** ALL of rungs 95/96's — burner temperature, thrust, HP speed (both laws), thrust inferred from the pressure
  ratio or the fan speed (both laws).
* **Lean limit** (asked after § 0's sweep): where the lag's leftover cut would take the applied fuel below the leanest
  fuel the engine runs on, HOLD it there (a minimum-fuel stop) and mark the point.
* **Hot start** (asked next): a governed run with the slow limiter whose START is steadily above the limiter's
  temperature is REFUSED in words — a working limiter would never let the engine settle there. Only this new
  combination; rung 47's open route keeps accepting such starts, as today.

Shipped as KNOBS: every prediction below is scored HIT/MISS, and a miss is a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\rung106-probe`, to be archived in
`docs/plans/scratch-archive/rung106/`) holding a PROTOTYPE built from public pieces; it printed REACHABILITY only — run
lengths, stops, clamp counts, which leg held, bit-equality and cost — never a temperature, a thrust, a time, a stall
margin or a cut size a claim below is about. § 1–§ 3 written before any rung-106 code exists in the crate.

---

## 0. GIVENS — measured before anything was written

The Controls engine and default slam (rung 95 § 0: 1000 → 1400 K over 0.5 τ, settle 2 τ, step 0.02, the perfect gas),
the redline 1480 K (rungs 46–49's), each lag `τ` in HP spool time constants.

**G1 — the prototype IS rung 47 when its demand is the open ramp** (`check47.txt`): the prototype's march with the
ramp's fuel as the demand against the Controls view's own lagged route (rung 47), all four maps, `τ` 0.02 / 0.05 / 0.2
/ 0.8 — 126 of 126 points each, **0 points differ** (speeds and applied fuel bit for bit). So every difference below
is the governor's.

**G2 — until the clip first engages, the governed lagged run IS rung 95's** (`reach.txt`): bit for bit at every point
before the first non-zero clip, every law and case of the first probe (336 runs).

**G3 — reachability** (`reach.txt`, 336 runs: seven laws × six cases × flow/press, flat-LP × `τ` 0.02–0.8):
* every acceleration and slowdown ran to the end, except accel + the LP stall floor on flat-LP, which stops at
  s = 0.82–0.84 with the floor out of reach — **rung 95's own governed march stops at the same point** (`reach2.txt`
  (a), 42–43 points both), so that stop is not the lag's;
* the default slam with the redline: the clip engages under every law but the burner-temperature hold (whose demand
  never passes 1400 K) and stays non-zero for 117–122 of 126 points;
* the governor asks for the RICH wall at 0–29 points on the default slam with a fast lag (`τ` 0.02–0.05: HP rate,
  HP proportional, pressure ratio), none at `τ` ≥ 0.2, and the thrust law at none; up to 118 on the hot slam 1000 → 1600 K;
* on the default slowdown 1400 → 1000 K the clip never engages (the start is below the redline).

**G4 — the lean limit** (`reach2.txt` (c), 2 520 runs: five throttle pairs × three redlines × three ramps × four maps ×
seven laws × `τ` 0.02, 0.8): **399 runs** held the applied fuel at the lean wall at some point — **every one a slowdown
that STARTS ABOVE THE REDLINE** (1450 → 1000 K at redline 1300; 1700 → 900 K at every redline). **The open ramp on
the same 399 configurations never reaches the wall** (`open_lean.txt`: the whole 2 520-run sweep with the ramp as the
demand — 0 at the wall; all 399 ran to the end. The prototype's open mode equals rung 47 wherever it runs (G1) and
stops where rung 47's solvable-ceiling fallback would carry on: 679 such stops elsewhere in the sweep, none among the
399). No acceleration and no start below the redline did. With
the hot-start refusal (§ 1) the hold is a safety net the scored runs never reach.

**G5 — cost:** a lagged governed run takes 0.3–1.5× rung 95's governed run on the same settings (one root and one
redline solve per evaluation, as rung 95 with the redline on).

## 1. THE DEVICE — derived before coding

**The march** — rung 95's governed march with rung 47's clip as a third state: at each evaluation, the governor's fuel
`G(ν_L, ν_H, s)` (rung 95/96's laws, unchanged, stateless), then

```
r     = G − T(ν; Tt4_max)   if Tt4 at G is above the redline, else 0     (rung 46/47's guard, off the DEMAND)
dg/ds = (r − g)/τ
u     = min(G − g, the other armed legs solved off G)                     (rung 47's pattern)
```

`T` is rung 46's topping fuel (the fuel that puts `Tt4` on the redline at frozen speeds). The other legs — rung 48's
schedule, 49/60's floor, rung 95's metal leg — stay instant, solved off `G`. An applied fuel below the leanest runnable
one is HELD at that wall (bisected, as rung 95's) and marked. The clip `g` and `r` are RECORDED at every point (a lagged
state carries history: a re-run step starts from the recorded one — rung 52's two keys).

**The demand is a closure**: the march takes the demand as `G(ν, s)`; the open ramp handed in instead gives rung 47's
march (G1) — kept as the gate below, two code paths.

**Scope:** the Controls view's governor with *the temperature limiter's lag* on. Refused in words: beside rung 101's
lagged exhaust sensor (for another session), beside rung 52's release and rung 88's slow valve (as today), and — the
hot-start rule — when the run's start has `Tt4` above the redline. The lag's knob keeps rung 47's page floor (one march
step); the step check (§ 2) confirms it under the governor.

**Where it lives:** a new module `rust/src/governor_lag.rs` (one module per rung) holding the march and its step, the
Controls route `GovernorLag`, nothing else moving.

## 2. REDUCES and GATES (construction — no credit)

* **Off ⇒ every earlier run, bit for bit** (dispatch: the lag off is rung 95/96/101's march; the governor off is rung
  47's). No gate, oracle or golden moves.
* **The open ramp as the demand IS rung 47's march** — the new march with the ramp handed in against the Controls
  view's lagged route, bit for bit at every point, four maps (G1's check, now on the shipped code).
* **Before the clip engages, rung 95's march** bit for bit; **a redline above the governed peak** ⇒ `g ≡ 0`, rung 95's
  run bit for bit (the instant redline then never cuts either).
* **`τ → 0` ⇒ rung 95 with the instant redline:** the gap in both speeds shrinks with `τ` (0.02, 0.01, 0.005 at a step
  `τ/4`), reported with its order.
* **The clip's ledger (D1):** `dg/ds = (r − g)/τ` integrates to `∫(r − g) ds = τ·(g(S) − g(0))` — exact in RK4's own
  stage quadrature, to the trapezoid's O(ds²) on recorded points. Measured against the instant limiter AT THE SAME
  STATES, the extra fuel the lag lets through early and the fuel it withholds later cancel, up to `τ·g` at the end.
  Gated on the recorded points of the default thrust run, four maps, every `τ` of § 3: **bar frozen —
  `|trapz(r − g) − τ·(g_end − g_0)| ≤ 2e−2 · τ · max g`** (exceeded means investigate, not relax), and the miss falls
  when the step is halved (0.02 → 0.01) at `τ` 0.2.
* **The lean hold, at model level:** one of G4's hot starts (1700 → 900 K, redline 1480, flow/press, thrust, `τ` 0.2)
  marched by calling the march DIRECTLY (the page refuses it): it holds at the wall, marks every such point, and runs
  to the end; separately the page refuses the same request in words.
* **Step check:** the default governed lagged run (thrust, `τ` = one step, 0.02) at the step 0.02 against 0.005, the flow
  coefficients' difference reported — sets or confirms the knob's floor.

## 3. PREDICTIONS — frozen here, scored after the build

Cells: the default slam with the redline (1480 K), each of the four maps, the thrust law unless stated; `τ` ∈ {0.05,
0.1, 0.2, 0.4, 0.8} (rung 47's set). **"Open"** = the same settings with the governor off (rung 47's lagged route);
**"instant"** = the governor with the lag off (rung 95 with the redline). The overshoot is `max Tt4 − 1480`.

**How every cell is read (frozen):**
* peaks, minima and maxima are over RECORDED points only (step 0.02);
* the "open" side's `g` and `r` (H3, H5) are read off the NEW march with the open ramp handed in as the demand — licensed
  only by § 2's bit-equality gate against rung 47's route (which does not record them);
* where one run of a pair stops early, both are read over the stretch BOTH reach (rung 88's discipline), and the cell
  says so;
* H4's crossing is interpolated linearly between recorded points; a run that never reaches 95 % scores the cell
  VACUOUS;
* H3: a cell where NEITHER run ever has `g > r` scores VACUOUS, not HIT; H5: a cell where either run never engages
  scores VACUOUS.

**Notation.** `e = r − g` is the lag's error: the fuel let through ABOVE what the instant limiter would allow at the
same state; `−e` where negative is the fuel WITHHELD below it (the stored cut). Linearising, `de/ds = dr/ds − e/τ`, so
`e(s) = ∫ r′(σ)·exp(−(s−σ)/τ) dσ` from engagement — the overshoot is the required cut's GROWTH RATE `r′ = G′ − T′`
filtered at `τ`, and the stored cut its FALL rate.

**H1 — the governor makes the lag's overshoot LARGER than on the open ramp:** governed overshoot > open overshoot at
every `τ`. 20 cells. *Derivation:* the open ramp's `r′ = R′ − T′` with `R′` the ramp's constant slope; the governor's
`G′` exceeds it on average over the ramp (rung 95 H1: `G > R` at every ramp point from a common start), so `r′` is
larger and so is `e`. Limits: `τ → ∞` (no clip) the two runs tend to rung 95's unlimited peaks, ~1930 K governed against
~1700 K open; `τ → 0` both hold the redline. *Risk:* the governed spools run ahead, so `T′` (the topping fuel rising
with airflow) is larger too and eats into `r′` — a fast lag reads mostly the slopes, and there the sign rests on
`G′ − R′ > T′_gov − T′_open`.

**H1b — and under every other law:** HP speed (rate 0.1, proportional 20), the pressure ratio, the fan speed (rate 0.2,
proportional 5), at `τ` 0.2: governed overshoot > open overshoot. 20 cells. Same derivation; risk largest on the fan-speed
laws (the slowest shaft, the gentlest demand).

**H2 — the overshoot rises with the lag:** governed overshoot strictly increasing over the five `τ`. 4 cells. *Derivation:*
at a fixed history with `r′ ≥ 0` up to the peak, `e` rises with `τ` (the kernel `exp(−u/τ)` rises pointwise); the
states differ (more fuel ⇒ faster spools), second order. Rung 47 measured this shape open.

**H3 — the governor STORES a larger cut:** the largest fuel withheld below the instant limiter, `max(g − r)` over the
run, is larger governed than open at every `τ`. 20 cells. *Derivation:* after the move the open ramp stops (`R′ = 0`), so
the open `r` falls only as `T` rises with the spools; the governed demand FALLS once the move ends (rung 95: its fuel
at the move's end exceeds the end's, and `Tt4` peaks there), so `r′ = G′ − T′` is more negative and the stored cut
`≈ −τ·r′` larger. This is rung 95's "nothing winds up" made conditional: the governor holds no state, but the clip
measured off its demand does. *Risk:* a governor still on its RICH wall when the move ends has `G′` set by the wall,
not by the demand — G3 saw none under the thrust law on this slam, some under the HP-speed and pressure-ratio laws at
`τ` ≤ 0.05 (H1b's cells sit at 0.2).

**H4 — so the lag makes the governed run reach the thrust LATER than the instant limiter**: 95 % of the thrust step
later than "instant" at every `τ`. 20 cells. *Derivation:* by D1 the fuel let through early is paid back by the stored
cut later; the governed thrust nears its demand at about 1.3 τ (rung 95 H5), after the move, i.e. inside the payback
window H3 enlarges — the run is starved just as it closes on the thrust. *Risk (named, the likeliest miss):* the early
surplus speeds the spools while the thrust is far off, and spool speed gained early persists; if the crossing comes
before most of the payback, the sign flips to SOONER. A two-way bet — a miss is the finding "front-loading wins".

**H5 — the redline engages EARLIER under the governor:** the first point with `r > 0` comes before the open run's. 4
cells. *Derivation:* rung 95 H1 — more fuel than the ramp at every point of the move, and H2's hotter run.

Reported, not scored: rung 47's two reliefs (the lowest flow coefficient of each compressor, lagged against unlimited)
under the governor — its "the lag never reaches the LP" was a TIMING fact (the LP minimum upstream of engagement), and
H5 moves the engagement; the burner-temperature hold with `T_set` above the redline; the clip's peak; the cost.

## 4. What would change the plan

A MISS is reported as a finding. A run that stops (a failed root, the floor out of reach) is a stop with its reason, as
every route's. **Crash map before the spec** (rung 95 lesson 1): 1 000 governed requests with the lag on, every hold and
law, random limiters, levers, generator, cooling and cap, both ends over the slider range; every "other" stop's message
read. Any lean-wall point in a run the hot-start rule admits is a finding to report, not hide.
