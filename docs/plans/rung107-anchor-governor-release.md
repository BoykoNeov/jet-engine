# Rung 107 anchor — THE GOVERNOR BESIDE THE REALISTIC RELEASE (pre-registered 2026-11-10 — § 3)

**Seam:** rung 106 § *The next seam*: *"**The governor beside rung 52's realistic release** (fast attack, slow release)
— the release edge is where the stored cut is paid back, so it decides the sag"*; rung 95/106's concession: *"rung 52's
release … stay[s] refused beside the governor."*

**The user's choices (2026-11-10):**
* **Which limiter:** rung 52 AS BUILT — the fast-attack / slow-release lag on the acceleration schedule (48) and the stall
  floor (49/60); the redline (46) and rung 95's metal leg stay INSTANT on top. (A side note had flagged, before the
  question, that rung 106's sag came from the redline's lag, which this leaves instant — so the sag may not appear here;
  the two-speed temperature limiter stays a possible later seam.)
* **Holds:** ALL of rungs 95/96's — burner temperature, thrust, HP speed (both laws), thrust read from the pressure ratio
  or the fan speed (both laws).
* **A floor above the start** (asked after § 0's sweep): a governed run with the release whose floor ALREADY asks for a
  cut at the start (the cut's target measured off the governor's request at s = 0 is above zero) is REFUSED in words —
  a working floor would never let the engine settle below it (rung 106's hot-start principle). Only this new
  combination; the governor-off route keeps showing such a run.
* Stated, not asked: beside rung 106's temperature lag, rung 88's slow valve or rung 101's lagged sensor stays refused
  (two lags at once); rung 106's minimum-fuel hold carries over on the governed path as a safety net.

Shipped as KNOBS: every prediction below is scored HIT/MISS, and a miss is a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\rung107-probe`, to be archived in
`docs/plans/scratch-archive/rung107/`) holding a PROTOTYPE built from public pieces; it printed REACHABILITY only — run
lengths, stops, clamp counts, which leg held, sign changes of the lag's error, bit-equality, step-check gaps and cost —
never a thrust, a time, a stall margin or a cut size a claim below is about. § 1–§ 3 written before any rung-107 code
exists in the crate.

---

## 0. GIVENS — measured before anything was written

The Controls engine and default slam (1000 → 1400 K over 0.5 τ, settle 2 τ, step 0.02, the perfect gas); the schedule's
margin 0.25 and the LP floor 0.75 (rungs 48/49's page numbers); `(τ_att, τ_rel)` in HP spool time constants.

**G1 — the prototype IS rung 52 when its demand is the open ramp** (`check52.txt`): its march with the ramp's fuel as the
demand against the Controls view's own release route, four maps × {schedule, floor, both + redline, floor on a
slowdown} × `(τ_att, τ_rel)` ∈ {(0.02, 0.1), (0.02, 0.8), (0.1, 0.1), (0.2, 0.05)} — **64 of 64 runs, 0 points differ**
(speeds and applied fuel bit for bit), every run to the end. So every difference below is the governor's.

**G2 — until the lagged legs first ask for a cut, the governed run IS rung 95's** (`reach.txt`): bit for bit at every
point before the first evaluation with a non-zero target, all 560 governed runs.

**G3′ — reachability IN THE ADMITTED REGION** (`reach2.txt`, re-probed after the knobs' floors were chosen — the advisor's
check: G3 below ran mostly at attacks the page now refuses). 1 344 runs: seven laws (HP gain 10, inside rung 106's edge)
× {schedule; floor; both + redline; a fast ramp 0.16 with the schedule; floor on a slowdown 1400 → 1000; schedule +
redline to 1600 K} × four maps × `(τ_att, τ_rel)` ∈ {(0.2, 0.1), (0.2, 0.2), (0.2, 0.4), (0.2, 0.8), (0.2, 1.6), (0.4,
0.1), (0.4, 0.4), (0.4, 1.6)}:
* **no re-engagement in any run, no lean-wall hold in any run**, every run with more than one release: none;
* the schedule (all four schedule cases, 896 runs) engages in every run, releases exactly once, runs to the end;
* the floor never engages under the burner-temperature hold on flow/press, press/flow and tilted; on flat-LP every
  acceleration with the floor asks for the cut at s = 0 (112 runs — the refused start) and those are the ONLY stops (10,
  the floor out of reach);
* on the slowdown the floor never engages except on flat-LP, where it engages and is STILL CUTTING at the end (48 runs,
  every law but the burner-temperature hold): the slowdown's END sits below the floor there. They run to the end, no lean
  hold — reported, not refused (the user's rule is about the start); the governor-off route does the same;
* cost 0.4–2.3× rung 95's governed run.

**G3 — reachability, the first probe** (`reach.txt`, 560 runs: seven laws × {schedule; floor; both + redline; floor on a
slowdown 1400 → 1000; schedule + redline to 1600 K} × four maps × `(τ_att, τ_rel)` ∈ {(0.02, 0.1), (0.02, 0.8), (0.1,
0.4), (0.4, 0.4)} — mostly attacks now refused; kept for what it found at the fast end):
* **the schedule** engages in every run (112 of 112), runs to the end in every run, releases exactly once, never
  re-engages, never at the lean wall;
* **the floor** engages on flow/press, press/flow and tilted under every law but the burner-temperature hold (whose
  demand never reaches it); on **flat-LP the start sits BELOW the floor** — the cut is asked for at s = 0 in every one
  of those runs (56);
* **every re-engagement** (a release followed by a new attack: 4 runs) and **24 of the 25 runs that held the applied
  fuel at the lean wall** are flat-LP floor runs starting below the floor — the user's refusal (above) removes them;
  the 25th is a slowdown on flat-LP (HP rate law, `τ_rel` 0.8) that then stops on the floor out of reach;
* **the stops** (34): every one the floor out of reach on flat-LP ("no fuel this side of flame-out restores it");
  rung 95's own governed march stops on the same runs, EARLIER (42–43 points against 45–64; 97–104 against 105–111);
* the burner-temperature hold to 1600 K with the schedule: the schedule never engages on 14 of 28 runs.

**The prototype vs the frozen device.** The probes worked out the instant legs in sequence (the redline off `G − g`,
then the metal leg off its result); § 1 freezes them solved TOGETHER off `G − g` (rung 95's min-select). No given
below is affected: the open path has no metal leg, and the metal cap was off in every probe.

**G4 — cost:** a governed release run takes 0.3–2× rung 95's governed run on the same settings (0.1–1 s natively).

**G5 — the step check (the knobs' floors).** Step 0.02 against 0.005, the largest gap in either flow coefficient
(`step_*.txt`, `base_*.txt`, `edge_*.txt`; three flights: design, thin 220 K / 20 kPa, dense sea level Mach 0.3):
* rung 95's governed march with the INSTANT schedule / floor — the page's route today — already reaches 1.0e−4 …
  5.6e−4 on the floor case at the dense flight (thrust 4.7e−4, pressure ratio 5.6e−4, HP gain 10 3.6e−4) and 1.0e−4 on
  the schedule (HP rate, dense); the open release (rung 52, governor off) at its page default (0.02/0.1) 3.2e−5 …
  1.1e−4;
* beside the governor, an attack of 0.02 (the page's default, one step) is rough under every law (1.9e−4 … 2.7e−3);
  the speed laws at RUNG 95's edges (rate 0.04, gain 50) are rough at every attack and release tried;
* at rung 106's per-law edges (HP rate 0.1, fan rate 0.15, HP gain 10, fan gain 7, density-scaled) with
  `(τ_att, τ_rel)` = (0.2, 0.4): every law, case and flight under 1e−4 (largest 7.5e−5);
* **the fine sweep** (`edge_*.txt`: attack {0.04, 0.06, 0.08, 0.1, 0.12, 0.14, 0.16, 0.2, 0.3, 0.5} × release {0.02,
  0.04, 0.06, 0.1, 0.2, 0.4, 0.8}, the seven laws at rung 106's per-law edges, three cases × three flights; the worst
  over all of them per cell): every cell with **attack ≥ 0.2 and release ≥ 0.1 is under 1e−4**, and so is attack 0.16
  with release ≥ 0.1; faster attacks are rough at every release (1.0e−4 … 3.2e−4), a release of 0.02–0.04 is rough at
  every attack (to 3.3e−4), and 0.06 is rough at attacks 0.06–0.16 and 0.5 — scattered cells, not a threshold (rung 106
  § 4's pattern); past the slider (`edge16_*.txt`: attack 0.2 / 0.4 × release 1.0 / 1.6, the same 63 settings) every
  cell under 1e−4 (largest 9.8e−5).
* **The user's choice (asked on that table): LIMIT AND NUDGE** — beside the governor the attack must be at least
  **0.2** and the release at least **0.1** (refused in words below), each speed law within rung 106's per-law edges; the
  page MOVES an out-of-range attack or release to its limit when the governor or the release is switched on, and says so
  under the release (rung 106's pattern); a value typed afterwards meets the refusal. The governor off keeps rung 52's
  one-step floor.

## 1. THE DEVICE — derived before coding

**The march** — rung 95's governed march with rung 52's clip as a third state, in `integrate_fuel_asym`'s order: at each
evaluation the governor's fuel `G(ν_L, ν_H, s)` (rungs 95/96's laws, unchanged, stateless), then

```
u₀    = G − g                                                         (the lagged cut)
u     = min(u₀, the instant legs solved off u₀)                       (the redline, rung 95's metal leg; rung 52's
                                                                        "the unlagged redline reads the CLIPPED fuel")
req   = max(0, G − min(schedule cap, floor cap) solved off G)         (rung 52's target, off the DEMAND)
dg/ds = (req − g)/τ_att   if req > g     (fast attack)
        (req − g)/τ_rel   otherwise      (slow release)
```

The instant legs are solved off `u₀` together and the smallest taken (rung 95's min-select), so at `g = 0` the instant
legs are rung 95's, bit for bit. An applied fuel below the leanest runnable one is HELD at that wall (rung 106's
minimum-fuel stop, bisected as rung 95's walls) and marked. `g` and `req` are RECORDED at every point (rung 52's two
keys), so every step re-runs from its recorded point.

**The demand is a closure** (rung 106's `Demand`): the open ramp handed in instead gives rung 52's march (G1) — kept as
the gate below, two code paths. The open path keeps rung 52's `1e−9` floor and its solvable-ceiling fallback; the lean
hold is the governed path's.

**Scope:** the Controls view's governor with *Realistic release* on (`route: "governor_release"`). Refused in words:
beside the temperature limiter's lag, the slow valve, rung 101's lagged exhaust sensor (two lags); a floor that asks for
a cut at the start (the user's choice); the knobs under the floors § 0 G5 sets, and each speed law past rung 106's
per-law edges (the same edges, measured again here). **The nudge** (the user's choice) moves the attack, the release AND
rung 106's out-of-range speed-law gain or time constant to their edges whenever the governor, the release, the hold,
the law or the reading is switched with both on (the page's own HP gain 20 sits past the edge 10), and says so under the
release — with the words that a governor-off run compared beside it should use the SAME attack (a side note: the floor
makes the governed limiter ten times slower to cut than the page's default, and rung 52 § 1 measured a slower attack
crediting less — so a governor-on/off comparison at different attacks mixes two changes; § 3's "open" cells use the
same lags throughout).

**Where it lives:** a new module `rust/src/governor_release.rs` (one module per rung) holding the march and its step, the
Controls route `GovernorRelease`, nothing else moving.

## 2. REDUCES and GATES (construction — no credit)

* **Off ⇒ every earlier run, bit for bit** (dispatch: the release off is rung 95/96/101/106's march; the governor off is
  rung 52's). No gate, oracle or golden moves.
* **The open ramp as the demand IS rung 52's march** — the new march with the ramp handed in against the Controls view's
  release route, bit for bit at every point, four maps × G1's cases (G1, now on the shipped code).
* **Before the lagged legs first ask for a cut, rung 95's march** bit for bit (thrust and HP-speed holds, two maps).
* **The trigger pins itself** (rung 52 § 1, structural): every point recorded before the first crossing (`req < g`)
  identical across `τ_rel` ∈ {0.1, 0.4, 0.8} at a fixed `τ_att`, bit for bit, governed.
* **The lag's ledger (D1)** — `dg/ds = (req − g)/τ` integrates, phase by phase, to `∫_attack (req − g) ds = τ_att·Δg`
  and `∫_release (g − req) ds = −τ_rel·Δg`: with one crossing at `g_c`, the fuel let through above the caps is
  `τ_att·g_c` and the fuel withheld below them `τ_rel·(g_c − g_end)` — their ratio is `τ_rel/τ_att`, governed or not.
  Gated on recorded points: the attack phase is points `0 … k−1` (the last with `req ≥ g`), the release phase `k …`
  end (`k` the first recorded crossing), the straddling interval in neither; trapezoids; `g_c = g` at `k`. **Measured
  before freezing** (`ledger.txt`, the advisor's check — a guessed bar is this project's commonest miss): the default
  schedule run, four maps × `(τ_att, τ_rel)` ∈ {0.2 × (0.2, 0.4, 0.8, 1.6), 0.4 × (0.4, 1.6)}: largest miss **9.5e−4**
  of `τ·g_c`, falling in every cell when the step halves; the floor (three maps, the same lags): largest **5.4e−3**, not
  always falling (its cap has the floor's dormant switch). **Bars frozen: the schedule `≤ 1e−2 · τ · g_c` per phase and
  falling when the step halves (0.02 → 0.01) at `(0.2, 0.4)`; the floor `≤ 2e−2 · τ · g_c` per phase** (exceeded means
  investigate, not relax).
* **`τ → 0` (both) ⇒ rung 95 with the instant schedule:** the gap in both speeds shrinks with `τ` (0.02, 0.01, 0.005 at a
  step `τ/4`), reported with its order.
* **Every step re-runs** from its recorded point, bit for bit (four settings).
* **The refusals** in words: the floor above the start, the two-lag combinations, the knobs' floors and the law edges.

## 3. PREDICTIONS — frozen here, scored after the build

Cells: the default slam with the schedule (margin 0.25), the thrust law unless stated, each of the four maps;
`τ_att` = 0.2 (the floor region G5 measures clean) and `τ_rel` ∈ {0.2, 0.4, 0.8, 1.6} unless stated (1.6 lies past
the page's release slider, which stops at 1.0: it is run by calling the march directly, as rung 106's 0.05 was). **"Open"** = the
same settings with the governor off (rung 52's route, read off the new march with the ramp handed in — licensed by § 2's
bit-equality gate); **"instant"** = the governor with the release off (rung 95 with the instant schedule). The floor
cells (H4, H5) are the LP floor 0.75 on flow/press, press/flow and tilted (flat-LP's start sits below it: refused).

**How every cell is read (frozen):**
* peaks, minima, crossings and maxima over RECORDED points only (step 0.02); "the crossing" is the first recorded point
  with `req < g` after `g > 0`;
* the thrust share is `(F − F_start)/(F_end − F_start)`, `F_end` the end throttle's steady thrust; the 95 % time is
  interpolated linearly between recorded points; a run that never reaches 95 % scores the cell VACUOUS;
* where one run of a pair stops early, both are read over the stretch BOTH reach, and the cell says so;
* a cell where the leg never engages scores VACUOUS, not HIT.

**Notation.** `e = req − g`: positive while attacking (fuel let through ABOVE the cap at that state), negative while
releasing (`−e` fuel WITHHELD below it). The governor is stateless; the clip is not.

**H1 — the governor makes the schedule cut DEEPER: the largest clip `max g` is larger governed than open,** and the cut
is asked for EARLIER (first `req > 0`). 4 maps × 2 = 8 cells (`max g` is `τ_rel`-free by the pinned trigger, so one
`τ_rel`). *Derivation:* rung 95 H1 — from a common start the governor asks for more fuel than the ramp at every point of
the move; the schedule's cap is a function of the state, so `req = G − C` is larger while the states are close. *Risk:*
the governed spools run ahead, so `C` (the schedule's `Wf/pt3` line times a higher `pt3`) is larger too — the sign rests
on `G − R` outgrowing `C_gov − C_open` (rung 106 H3 measured the analogue 1.8–3.1×).

**H2 — the release COSTS the governed thrust: the 95 % time rises with `τ_rel`** (strictly over the four `τ_rel`), and is
later than instant at every `τ_rel`. 4 maps × (1 + 4) = 20 cells. *Derivation:* by D1 the withheld fuel is
`τ_rel·g_c`, and `g_c` does not move with `τ_rel` (pinned trigger) — so the withheld fuel grows in proportion; the
governor does not see the cut (stateless), only the slower spools it leaves. *Risk (named):* the attack lets
`τ_att·g_c` through early, and spool speed gained early persists (rung 106 H4 flipped past `τ` ≈ 0.9); and if the 95 %
crossing comes BEFORE the release begins, `τ_rel` cannot move it at all (the pinned trigger) — those cells score
VACUOUS, and the claim then fails as a claim about this slam.

**H3 — the governor PAYS BACK part of the release: the release's cost is SMALLER governed than open** — `t95(τ_rel = 1.6)
− t95(τ_rel = 0.2)` governed < the same open. 4 cells. *Derivation:* during the release `u = G − g`; spools held back by
the withheld fuel make a thrust governor ask for MORE (`G` rises with the thrust error at the state), and every unit of
`G` reaches the applied fuel one-for-one while `g` only decays — the governor buys back the release on its own clock.
The open ramp's demand is fixed once the move ends. *Risk:* the governor's `g_c` is larger (H1), so it has MORE to pay
back (`τ_rel·g_c`); a two-way bet on which wins.

**H4 — rung 52's "cannot debit the spool it watches" SURVIVES the governor, and for a structural reason:** on the floor
cells the LP's lowest flow coefficient is identical across `τ_rel` ∈ {0.2, 0.4, 0.8, 1.6} (bit for bit), and it lies
BEFORE the first crossing, governed. 3 maps × 2 (`τ_att` 0.2, 0.4) = 6 cells. *Derivation (sharper than rung 52's):*
while releasing, `g > req`, so the applied fuel `G − g` is below the floor's own cap `G − req` at that state ⇒ the LP sits
ABOVE the floor; while attacking it sits below; before engagement above. So the run's lowest LP point (below the floor)
lies in an ATTACK phase — for ANY attack shape that keeps `g < req` while attacking, not only rung 52's first-order one
(rung 52 § 2's precondition). With one attack phase it precedes the first crossing, which `τ_rel` cannot reach. The
argument's preconditions, stated: the LP's flow coefficient FALLS as fuel rises at frozen speeds (the floor's cap is that
monotone root); the floor is the BINDING cap; and during the release neither the lean hold nor the solvable-ceiling
fallback sets the applied fuel (either could put it above the floor's cap). **Vacuity (frozen):** a cell where the floor
is not the binding cap at the run's LP minimum, or where `min φ_lp ≥ 0.75` (no undershoot), scores VACUOUS (rung 52
gate 4). Because the pinned-trigger gate already makes every pre-crossing point `τ_rel`-free, H4's empirical content is
"the minimum lies in the FIRST attack phase"; that rung 52 § 2's first-order precondition is unnecessary is the CLAIM
scored here, not a premise. *Risk:* a re-engagement (a second attack phase, whose timing IS `τ_rel`-dependent).

**H5 — the slower release still debits the OTHER spool LESS: on the floor cells the HP's lowest flow coefficient rises
with `τ_rel`** (strictly over the four), governed (rung 52 § 4's rate verdict, open). 3 maps × 2 = 6 cells. *Derivation:*
rung 52 § 4 — the HP's debit comes from the fuel handed back after the LP's minimum; a slower hand-back is a gentler one.
*Risk:* H3's mechanism — the governor's own demand rises during the release, on ITS clock, and may flatten or reverse
the ordering.

**H6 — THE SAG (the seam's own question): the governed thrust, beside the slow release, FALLS after the move** — its
drawdown (below) exceeds 0.002 at every `τ_rel`; and neither open nor instant ever falls by that much. 4 maps × 4 = 16 cells (+ the two comparisons reported).
**Reading (frozen):** the sag is the largest DRAWDOWN after the move's end, `max over s > s_move of (max share on
[s_move, s] − share(s))`, and a cell sags when it exceeds 0.002 — the same reading for the open and instant comparisons
(a thrust that rises past the move, peaks, and falls back while staying above its move-end share still sags).
*Derivation:* rung 106 § 5: a sag needs a demand that comes back down while a stored cut lags it down. After the move a
thrust governor's demand falls back toward the end's fuel (rung 95); during the release `du/ds = G′ + (g − req)/τ_rel`
with `G′ < 0` and `g − req` small just after the crossing, so the applied fuel FALLS. The open ramp's `G′ = 0` (it only
rises); the instant schedule hands the cap through (`C` rising with `pt3`). *Risk (named, the side note's):* the
crossing may come DURING the move, while `G′ > 0` — then nothing falls; or the release may finish before the governor's
demand turns. A miss is the finding "the sag belongs to the redline's lag, not to any slow cut".

Reported, not scored: the crossing times (governed against open); the let-through and withheld fuels against D1; H1–H6
on the floor + schedule + redline case; under the other laws (HP rate 0.1, HP gain 10, the pressure ratio, the fan
speed rate 0.15 / gain 7) H2 and H6 at `τ_rel` 0.4; the cost.

## 4. What would change the plan

A MISS is reported as a finding. A run that stops (a failed root, the floor out of reach) is a stop with its reason, as
every route's. **Crash map before the spec** (rung 95 lesson 1): 1 000 governed requests with the release on, every hold
and law, random limiters, levers, generator, cooling and cap, both ends over the slider range; every "other" stop's
message read; any lean-wall point or re-engagement in a run the floor rule admits is a finding to report, not hide.
**All HIT ⇒ look past the grid** (rung 106 lesson 3): `τ_rel` past 1.6, `τ_att` past 0.4.
