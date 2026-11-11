# Rung 108 anchor — THE TWO-SPEED TEMPERATURE LIMITER (pre-registered 2026-11-11 — § 3 FROZEN after review)

**Seam:** rung 107 § *The next seam*: *"**The two-speed temperature limiter beside the governor** — rung 52's asymmetric
lag on the REDLINE (106 lagged it with one time constant; 107 left it instant): does a fast attack on the temperature
limiter end rung 106's overshoot while a slow release keeps its sag?"*

**The user's choices (2026-11-11):**
* **Where:** BOTH — beside the governor (rung 106's route) AND on the plain throttle move (rung 47's route), so the page
  has no switch that works in one mode only and the governed-against-open comparison is on the page.
* **Holds:** ALL of rungs 95/96's, each speed law's edge measured.
* **On the page:** a tick box under the temperature limiter's lag, *"…cuts fast, gives back slowly"*; ticking it gives a
  fast cut-in and keeps the existing lag as the give-back time — the realistic case as the default (the cost measured
  and reported before shipping).
* Stated, not asked (carried over): rung 106's refusals of a start above the redline and of a governor's request above
  it at the start; the minimum-fuel hold on the governed path; beside rung 52's release, rung 88's slow valve or rung
  101's lagged sensor stays refused (two lags at once).

Shipped as KNOBS: every prediction below is scored HIT/MISS, and a miss is a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\rung108-probe`, to be archived in
`docs/plans/scratch-archive/rung108/`) holding a PROTOTYPE built from public pieces (rung 106's `try_der` with the one
line `dg = (r − g)/lag.tau(r, g)`); it printed REACHABILITY only — run lengths, stops, edge counts, min-fuel holds,
bit-equality, step-check gaps and cost — never an overshoot, a thrust, a time, a stall margin or a cut size a claim
below is about. § 1–§ 3 written before any rung-108 code exists in the crate. **Reviewed 2026-11-11** (advisor): the
probe's printed lines checked for every scored quantity — `ledger.txt` prints only relative ledger misses, no `g_c`,
peak `Tt4` or drawdown, so H3 stays blind; the old H1 (overshoot `τ_rel`-free, peak before the crossing) was
construction and moved into § 2, a falsifiable H1 (the overshoot's scaling with `τ_att`) took its place; H6's 95 % time
and vacuity rule and H8's scored half were made precise.

---

## 0. GIVENS — measured before anything was written

The Controls engine and default slam (1000 → 1400 K over 0.5 τ, settle 2 τ, the perfect gas), the redline 1480 K;
`(τ_att, τ_rel)` in HP spool time constants. The 63 governed settings below are rung 107's: seven laws at rung 106's
per-law edges (thrust, pressure ratio, burner temperature, HP rate 0.1, HP gain 10, fan-speed rate 0.15, fan-speed gain
7, density-scaled) × {redline alone; + schedule; + floor on press/flow} × {design; thin 220 K / 20 kPa; dense sea level
Mach 0.3}; the open ramp on the same 9 case × flight pairs.

**G1 — the prototype at equal constants IS the page's route** (`sweep.txt`, `same=`): at `(0.2, 0.2)` its march equals
the Controls view's own — rung 106's governed route, rung 47's open route — bit for bit (speeds, `Tt4`, applied fuel at
every point) in all 72 settings. So every difference below is the second constant's.

**G2 — reachability** (`sweep.txt`, 576 runs: the 72 settings × `τ_att` {0.01, 0.02, 0.05, 0.1} × `τ_rel` {0.1, 0.8};
`open2.txt`, 900 open runs: 3 cases × 3 flights × 4 maps × `τ_att` {0.02, 0.03, 0.05, 0.1, 0.2} × `τ_rel` {0.02, 0.05,
0.1, 0.2, 0.8}): **every run that engages attacks exactly once and releases exactly once — no re-attack, no lean-wall
hold, anywhere.** The burner-temperature hold (to 1400 K, below the redline) never engages (18 of 126 per attack). The
only stops are the open floor runs on flat-LP at the design and dense flights (36 and 23 points), the same at every
`(τ_att, τ_rel)` — rung 47's route (G1) stops there too; not the clip's.

**G3 — the step check** (each run against one at a step four times finer, the largest gap in either flow coefficient on
the 0.02 grid; bar 1e−4):
* *governed, one step (0.02):* rough with a fast cut — `τ_att` 0.1: 14 of 126 runs over the bar, 0.05: 32, 0.02: 76,
  0.01: 91 (worst 2.1e−3). **At 0.005 every run is clean** (worst 5.8e−5, `τ_att` 0.01; 3.3e−5 at 0.02).
* *rung 106's own route is already rough at one step beside the floor or the schedule* (`base.txt`, the single lag at
  0.1 / 0.2 on the 63 governed settings): to 6.2e−4 (floor, dense, pressure ratio) — rung 106 § 4's sweep never combined
  the lag with the floor; rung 95's instant route on the same settings is clean but one (1.3e−4). **The user's choice:
  record it now (sandbox plan), fix it after this rung** — so this rung moves nothing that runs today.
* *open, one step:* clean for every `τ_att` ≥ 0.02 and `τ_rel` ≥ 0.02 (worst 9.6e−5 at `τ_att` 0.02, the same at every
  release — the worst point lies in the attack); rough at 0.01 (7.0e−4).

**The user's choice (asked on G3 and G4): the cut-in floor 0.02 τ, and beside the governor the two-speed limiter ALWAYS
on the 0.005 step** (one rule, at every `τ_att`: at 0.1–0.2 the floor cases are rough at one step too); the open route at
the page step (0.02) with its one-step floors (0.02). The give-back time keeps the existing lag knob's floors (0.1 beside
the governor — clean at 0.005, `sweep.txt` — and one step open). The speed laws keep rung 106's edges (clean at 0.005).

**G4 — cost** (`timing.txt`, one thread, natively): a governed two-speed run at `(0.02, 0.2)` on the 0.005 step takes
0.5–3.0 s against rung 106's 0.1–0.9 s on the same settings (3.4–7.8×); the open route the same as rung 47's (0.05–0.24
s). The page (WebAssembly) ran ~2× native in plan § 45 — to be measured on the page before shipping.

**G5 — the ledger, measured** (`ledger.txt`; the default slam, thrust law, four maps, `(τ_att, τ_rel)` ∈ {(0.02, 0.2),
(0.02, 0.8), (0.05, 0.4), (0.1, 1.6), (0.2, 0.2)}; recorded points, trapezoids, the attack phase points `0 … k−1`, the
release `k …` end, `k` the first recorded crossing): governed (0.005) attack misses 7.5e−5 … 1.2e−3 of `τ_att·g_c`,
release 2.9e−6 … 3.2e−5 of `τ_rel·g_c`, all falling when the step halves; open (0.02) attack 8.6e−4 … 1.3e−2, release
5.4e−5 … 5.0e−4, falling when the step halves but one (map 2, (0.1, 1.6): 8.6e−4 → 1.1e−3).

## 1. THE DEVICE — derived before coding

**The march** — rung 106's three-state march `(ν_L, ν_H, g)` with rung 52's two constants on the clip. At each
evaluation the demand `G` (rungs 95/96's governor, stateless — or the open ramp's fuel at `s`), then

```
r     = G − T(ν; Tt4_max)   if Tt4 at G is above the redline, else 0     (rung 46/47's guard, off the DEMAND)
dg/ds = (r − g)/τ_att       if r > g        (fast to cut)
        (r − g)/τ_rel       otherwise       (slow to give back)
u     = min(G − g, the other armed legs solved off G)                     (rung 47/106's pattern)
```

`T` is rung 46's topping fuel; the schedule (48), the floor (49/60) and rung 95's metal leg stay instant, solved off `G`.
The tie `r = g` takes the release constant (`AsymmetricLag::tau`, rung 52's; continuous there, since the error is 0).
An applied fuel below the leanest that runs is held at that wall and marked (rung 106's minimum-fuel stop). `g` and `r`
are recorded at every point (rung 52's two keys), so every step re-runs from its recorded point.

**The demand is a closure** (rung 106's `Demand`): the open ramp handed in gives the open route — rung 47's march when
the two constants are equal.

**The ledger (D1)** — rung 106's `∫(r − g) ds = τ·Δg` splits by phase: with one crossing (the first `r ≤ g` after
`g > 0`) at `g_c`, the fuel let through above the instant redline's cut (at the same states) on the way up is
`∫_attack (r − g) ds = τ_att·g_c`, and the fuel withheld below it on the way down `∫_release (g − r) ds =
τ_rel·(g_c − g_end)`. Rung 106's cancellation (both `τ·…`) is gone: the two are in the ratio `τ_rel/τ_att` per unit of
stored cut.

**The pinned trigger** (rung 52 § 1): `τ_rel` is never read while `r > g`, so every point before the first crossing is
`τ_rel`-free — and identical to rung 106's march at `τ = τ_att`.

**Where it lives:** a new module `rust/src/two_speed_clip.rs` (one module per rung) — the march and its step; the
Controls routes beside it; `governor_lag.rs` untouched.

## 2. REDUCES and GATES (construction — no credit)

* **Off ⇒ every earlier run, bit for bit** (dispatch: the tick box off is rung 47's / 106's route). No gate, oracle or
  golden moves.
* **Equal constants ⇒ rung 106 (governed) and rung 47 (open), bit for bit**, at every point (§ 0 G1 on the shipped code).
* **The pinned trigger:** every point before the first crossing identical across `τ_rel` ∈ {0.1, 0.4, 0.8} at a fixed
  `τ_att`, and identical to rung 106's march at `τ = τ_att`, bit for bit, governed and open.
* **The overshoot is the cut's (construction — moved here from § 3 on review):** with the redline the only leg, `Tt4 >
  1480` exactly while `e = r − g > 0`, an attack phase; one attack per run (G2) ⇒ the peak `Tt4` lies before the first
  crossing and is identical across `τ_rel`, bit for bit, on § 3's cells. Its only failure is this gate's: an inner RK
  stage of the crossing step reading `τ_rel` before the recorded crossing (rung 107 lesson 4) — then it is investigated.
* **Both constants → 0 ⇒ rung 95 with the instant redline** (governed): the gap shrinks with `τ`, reported with its order.
* **The ledger (D1)** per phase, G5's cells: **bars frozen — governed (0.005) `≤ 5e−3 · τ · g_c` per phase; open (0.02)
  `≤ 3e−2 · τ · g_c` per phase** (exceeded means investigate, not relax).
* **The same rule for the page's fine step** (plan § 45's): the governed two-speed route's step is the run's own (0.005),
  read by the march, the re-run of a step, the stop's diagnosis and the expected point count; the reply carries it.
* **Every step re-runs** from its recorded point, bit for bit (four settings, the fine step included).
* **The refusals** in words: carried over (above), and the knobs' floors § 0 sets.

## 3. PREDICTIONS — frozen here, scored after the build

Cells: the default slam, the thrust law (`ThrustBy::True`) unless stated, each of the four maps (flow/press,
press/flow, tilted, flat-LP), the redline alone; `τ_att` ∈ {0.02, 0.05, 0.1} and `τ_rel` ∈ {0.2, 0.4, 0.8} unless
stated. **Every scored run on the 0.005 step, governed and open alike** (one reading grid for every comparison — plan
§ 45's lesson: a peak over marched points compares only at equal grids; the open route's page step 0.02 is G3-clean, so
its page numbers agree within the step check). **"106"** = the new march at equal constants `τ_att = τ_rel = τ`
(rung 106's march, by the G1 gate); **"instant"** = rung 95's governed march with the instant redline; **"open"** = the
same lags with the ramp as the demand.

**How every cell is read (frozen):**
* peaks, minima, crossings over RECORDED points only; the overshoot is `max Tt4 − 1480 K`; "the crossing" is the first
  recorded point with `r ≤ g` after `g > 0`; `g_c` is `g` there;
* the thrust share is `(F − F_start)/(F_end − F_start)`, `F_end` the end throttle's steady thrust; the 95 % time is
  interpolated linearly between recorded points;
* the sag is rung 107's drawdown after the move's end, `max over s > 0.5 of (max share on [0.5, s] − share(s))`; a run
  SAGS when it exceeds 0.002;
* a cell whose clip never engages (`g ≡ 0`) scores VACUOUS; where one run of a pair stops early both are read over the
  stretch both reach, and the cell says so.

**Notation.** `e = r − g`: positive while attacking (fuel let through ABOVE the instant redline's cut at that state —
the engine then runs above the redline), negative while releasing (`−e` WITHHELD — the engine below the redline).

**H1 — THE SEAM'S FIRST HALF: a fast cut shrinks the overshoot in proportion to its time — the overshoot falls strictly
as `τ_att` falls over {0.02, 0.05, 0.1, 0.2}, and overshoot(0.02)/overshoot(0.1) lies in [0.2, 0.5],** governed and
open, at `τ_rel` 0.4 (the overshoot is `τ_rel`-free by § 2's gate). 4 maps × 2 × (ordering + ratio) = 16 cells.
*Derivation:* the instant clip holds `Tt4` at 1480 exactly (rung 46), and `Tt4` is algebraic in the applied fuel at the
states, so the overshoot is the let-through `e = r − g` times `∂Tt4/∂u`. A first-order lag on a target rising at a
constant `r′` from its engagement trails it by `e = τ r′ (1 − exp(−D/τ))` after a window `D`; the ratio at τ 0.02 to
0.1 is `0.2·(1 − e^{−D/0.02})/(1 − e^{−D/0.1})` — 0.2 for a long window, 0.31 at `D` 0.1, 0.47 at `D` 0.05 (the
target's rise lasting at least the move's last tenth). *Risk:* `r′` not constant — a target that rises then peaks gives
the slow lag a smaller average slope (ratio up); a slow cut that runs the engine ahead may steepen `r` (ratio below 0.2
— superlinear, a finding). **Vacuity:** no overshoot at `τ_att` 0.1.

**H2 — the governor still makes a fast cut overshoot MORE than the open ramp:** governed overshoot > open at each
`τ_att`. 4 maps × 3 = 12 cells. *Derivation:* a first-order lag on a target rising at `r′` trails it by about `τ_att·r′`;
the governor's demand rises faster than the ramp (rung 95 H1, rung 106 H1: 272 K against 138 K at 0.2), so its `r′` is
larger. *Risk:* at a fast cut the overshoot is set by the first few steps of the engagement, where the governor's and
the ramp's `r′` may be close.

**H3 — the faster cut STORES MORE: the cut at the crossing `g_c` falls strictly as `τ_att` rises** over {0.02, 0.05, 0.1,
0.2}, governed (at `τ_rel` 0.4). 4 cells. *Derivation:* the advisor's named risk, made a claim — a fast lag tracks its
target to near its peak, a slow one turns down earlier and lower; and the slow cut's overshoot runs the engine ahead,
where the topping fuel `T(ν)` is larger and the thrust governor asks less, both lowering `r`. So the slow release has
MORE to pay back after a fast cut (D1: withheld `τ_rel·(g_c − g_end)`). *Risk:* the slower lag's crossing comes later,
on a different part of `r`'s fall.

**H4 — THE SEAM'S QUESTION, PART 1: beside the governor a fast cut still SAGS — the drawdown exceeds 0.002 at every
cell, and rises strictly with `τ_rel` at each `τ_att`.** 4 maps × 9 = 36 cells + 4 × 3 = 12 ordering cells.
*Derivation:* after the move the thrust governor's demand is the fuel that gives the commanded thrust AT the current
speeds, so the applied `G − g` falls short by about `F_mf·g` (`F_mf` the thrust per unit fuel at those speeds). During
the release `g` decays at `τ_rel`; as the spools speed up `F_mf` grows, and a slow enough decay lets the shortfall GROW —
the thrust falls. A slower release decays slower ⇒ a larger sag. Rung 107 § 45 measured exactly this order on the
schedule's lag at a 0.02 cut (0.5 / 1.5 / 3 % at release 0.1 / 0.4 / 1.6). *Risk (named — the derivation's weak link):*
whether `F_mf` grows faster than `g` decays is not derived; if it does not, the shortfall only shrinks and a fast cut
leaves a slow CLIMB with no fall — the finding would be "the sag was the slow cut's catch-up, not the release's"
(rung 106's sag fell at s = 0.64–0.82, while its cut was possibly still attacking). The run is 2.5 τ long, so at
`τ_rel` 0.8 the release may be cut off by the window: `g_end/g_c` is reported beside every H4 cell, and a strict-ordering
MISS is read against it.

**H5 — PART 2: at rung 106's own lag, a fast cut sags LESS than rung 106** — the drawdown at `(0.02, τ)` < "106" at `τ`,
for `τ` ∈ {0.2, 0.4, 0.8}. 4 maps × 3 = 12 cells. *Derivation:* rung 107 § 4 found most of a slow cut's sag was the CUT's
— the stored cut still growing after the move (an attack phase, `g` catching up toward `r`, the shortfall rising); a fast
cut has caught up before the move ends. *Risk:* H3 — the fast cut stores more, so its release withholds more.

**H6 — the fast cut costs time: the 95 % SETTLE time comes later than instant at every `τ_rel`, and later with `τ_rel`**
(strictly over the three), at `τ_att` 0.02. 4 maps × (3 + 1) = 16 cells. **The settle time (frozen):** the last upward
crossing of share 0.95 after which every recorded point to the run's end stays ≥ 0.95, interpolated linearly (a
sagging run can pass 0.95, fall back and pass it again; the first reach is reported beside it, not scored).
*Derivation:* D1 — the release withholds `τ_rel·(g_c − g_end)`, `g_c` `τ_rel`-free; the withheld fuel is spool speed
not gained. *Risk (named on review):* during the attack the lagged clip lets through `τ_att·g_c` MORE than the instant
one, so the engine runs ahead of instant (rung 107's "runs ahead, then sags"); a settle near the move's end can come
EARLIER than instant for that reason. **Vacuity, split:** "later than instant" is scored wherever both runs settle; the
`τ_rel` ordering is VACUOUS when all three settle before the crossing (then `τ_rel` cannot move it); a run that never
settles makes its cells VACUOUS (rung 107's H3 went all VACUOUS that way — reported with the share reached).

**H7 — the open ramp never sags:** the open drawdown ≤ 0.002 at every `(τ_att, τ_rel)`. 4 maps × 9 = 36 cells.
*Derivation:* after the move the ramp's fuel is constant; during the release `g` falls, so the applied `G − g` RISES and
the spools only accelerate; the attack after the move is short at a fast cut. Rung 106: the open ramp never falls under
the single lag. *Risk:* an attack still running after the move (the shortfall then grows) — largest at `τ_att` 0.1.

**H8 — rung 106's LP relief, extended: the LP's lowest flow coefficient lies BEFORE the first crossing at every
`τ_att`, and RISES as `τ_att` falls** (strictly over {0.02, 0.05, 0.1}), governed, at `τ_rel` 0.4. 4 maps × (3 + 1) =
16 cells. (Before the crossing ⇒ `τ_rel`-free and rung 106's at `τ = τ_att`, by § 2's gate — construction, checked
there, not scored.) *Derivation:* rung 106 § 4 — under the governor the redline engages before the LP's closest
approach, and a faster lag cuts more fuel there (relief +0.0025 at 0.05 against +0.0016 at 0.2). *Risk:* a fast cut
crosses EARLIER, so its crossing may come before the LP's closest approach (the location half MISSES, and the minimum
then moves with `τ_rel`); and the relief may saturate below 0.05. **Vacuity:** none — a minimum always exists; a relief
of exactly zero against "instant" is reported.

Reported, not scored: `g_c`, the let-through and withheld fuels against D1; the overshoot against "106" and "instant";
under the other laws (HP rate 0.1, HP gain 10, the pressure ratio, fan-speed rate 0.15 / gain 7) the overshoot, H4 and
H5 at `(0.02, 0.4)`; the burner-temperature hold to 1600 K (crossing the redline — rung 106's exception); the page's cost.

## 4. What would change the plan

A MISS is reported as a finding. A run that stops is a stop with its reason. **Crash map before the spec**: 1 000
requests with the two-speed limiter on, governed and open, every hold and law, random limiters, levers, generator,
cooling and cap, both ends over the slider range; every "other" stop read; any re-attack or lean-wall hold in an
admitted run reported. **All HIT ⇒ look past the grid.**
