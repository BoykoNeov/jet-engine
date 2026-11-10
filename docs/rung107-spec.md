# Rung 107 — THE GOVERNOR BESIDE THE REALISTIC RELEASE: rung 95's governor as the demand rung 52's fast-attack / slow-release lag is cut from

**Code:** `rust/src/governor_release.rs` — `try_der` / `try_rk_step` / `march` (the three-state march `(ν_L, ν_H, g)`,
the lag and its target recorded as rung 52's `PointExtra::Asym { g, required }`; the demand rung 106's `Demand` — the
governor, or the open ramp, the gate's second code path); `governor_lag.rs` — `Demand::try_fuel` made `pub(crate)`,
nothing else. The Controls view (`sandbox_controls.rs`): `Route::GovernorRelease`, `release_lag` / `gov_rel_der` /
`gov_rel_step`, the knobs' floors `GOV_REL_*`, the refusals (rung 106's "the governor runs with the instant limiters
only" LIFTED for the release). The page (`docs/sandbox/template.html`): the nudge `relNudge()` and its note
`ctl-relnudge`; the attack slider's top 0.2 → 0.5. **Gates:** `rust/tests/rung107.rs`. **Panel:**
`print_governor_release_table` (born in Rust, `rust/oracle/rust_owned/`, a change detector only). **Anchor:**
`docs/plans/rung107-anchor-governor-release.md`, committed as `7b15c6b` before any rung-107 code. **Scratch:**
`docs/plans/scratch-archive/rung107/` (`score.rs`, `crash.rs`, `sagcause.rs` and their outputs are the numbers below).

The seam: rung 106's (*"the governor beside rung 52's realistic release — the release edge is where the stored cut is
paid back, so it decides the sag"*). The user's choices (memory `rung107-decisions`): rung 52 AS BUILT — the
acceleration schedule's and the stall floor's cut lagged, fast in, slow out; the redline and rung 95's metal leg
instant on top; every hold of rungs 95/96; a run whose floor already asks for a cut at the start REFUSED in words;
beside the governor the attack at least 0.2 τ and the release at least 0.1 τ (the step check, anchor § 0 G5), the
page MOVING an out-of-range knob to its floor and saying so; beside rung 106's lag, rung 88's slow valve or rung 101's
slow sensor refused (two lags).

> **THE HEADLINE — holding thrust, the governed thrust SAGS beside the realistic release, and the sag belongs mostly
> to the slow CUT, not to the slow release.** The cut is measured off the governor's demand, which asks for more than
> the open ramp from the first step, so the limiter cuts more than twice as deep (2.35–2.40×) and asks for the cut
> earlier (s = 0.06 against 0.12–0.14). But a slow attack lets fuel through before the cut catches up: at the page's
> attack (0.2 τ) the governed engine is at 78 % of its thrust step when the move ends, against 33 % with the instant
> schedule. It has run ahead; the governor's demand falls back, and the thrust falls with it — by 3.3–4.9 % of the step
> (the drawdown after the move), deeper the slower the release, but only a little: the release adds 0.4 points at
> 0.4 τ and 1.1 at 1.6 τ to what an instant release already sags. On the open ramp, and with the instant schedule, the
> thrust never falls. **The seam's premise — "the release edge decides the sag" — is refuted at the page's attacks.**
> Each edge alone makes a sag (attack alone 3.7 %, release alone up to 3.0 %), but they do not add.
>
> **Scope: the direct thrust hold only.** Under the HP-speed holds and the read thrust (pressure ratio, fan speed) the
> governor with the INSTANT schedule already sags 5–13 % (rung 95's own overshoot), and the lag can sag less than that
> (fan-speed rate 0.15: 3.9–4.4 % against 5.6–9.6 %) — there the sign of the lag's contribution itself changes.

## 1. What it adds

At each evaluation (anchor § 1, `integrate_fuel_asym`'s order):

```
u₀    = G − g                                                   (the governor's fuel less the lagged cut)
u     = min(u₀, the redline and the metal leg solved off u₀)    (instant, on top)
req   = max(0, G − min(schedule cap, floor cap) solved off G)   (the target, off the DEMAND)
dg/ds = (req − g)/τ_att   if req > g     (fast attack)
        (req − g)/τ_rel   otherwise      (slow release)
```

The governor `G` is rungs 95/96's, unchanged and stateless. The instant legs are solved off `u₀` together (rung 95's
min-select), so at `g = 0` the run is rung 95's. Below the leanest fuel the engine runs on, the applied fuel is held at
that wall and marked (rung 106's minimum-fuel stop). `g` and `req` are recorded at every point, so every step re-runs.

**Why the attack makes a sag (the model).** While attacking, `g < req`: the applied fuel `G − g` sits ABOVE the
schedule's cap by `req − g`, and by D1 the fuel so let through is `τ_att·g_c`. The governor sees only the thrust error;
an engine that has been given extra fuel early is ahead of its line, the thrust error closes early, and once the move
ends the demand comes DOWN toward the end's steady fuel — taking the applied fuel with it. With the instant schedule the
cap holds the applied fuel BELOW `G` all along the climb (`C` rising with `pt3`), so the engine is never ahead and the
thrust never falls. The release adds a second, smaller channel: after the crossing `du/ds = G′ + (g − req)/τ_rel`, with
`G′ < 0` once the move ends, the anchor's mechanism — real (an instant attack with a slow release sags up to 3.0 %), but
at the page's attack it rides on top of the first.

## 2. Gates (anchor § 2, bars frozen at `7b15c6b`) — all pass

* **Off ⇒ every earlier run, bit for bit** (dispatch). No oracle or golden moved.
* **The open ramp as the demand IS rung 52's march** — 64 runs (4 maps × 4 cases × 4 lag pairs) against the Controls
  view's release route, bit for bit at every point, the lag included.
* **Before the lag first asks for a cut, rung 95's march**, bit for bit (thrust and HP-speed holds, two maps, schedule
  and floor); a schedule that never cuts is rung 95's whole run.
* **The trigger pins itself** — every point before the first crossing identical across `τ_rel` ∈ {0.1, 0.4, 0.8}, the
  crossing index identical, and the runs differ after it.
* **D1, the lag's ledger** per phase, on recorded points: the schedule ≤ 1e−2·τ·g_c (falling when the step halves), the
  floor ≤ 2e−2·τ·g_c. Reported on the scored grid: the fuel let through matches `τ_att·g_c` within 0.2 %, the fuel
  withheld matches `τ_rel·(g_c − g_end)` within 0.1 %.
* **`τ → 0` (both)** tends to rung 95 with the instant schedule (gaps shrinking at 0.02, 0.01, 0.005, order ~0.96).
* **Every step re-runs** from its recorded point, bit for bit (four settings, including the fan-speed and HP-gain laws).
* **The refusals** in words: the knobs' floors, the speed laws past rung 106's edges, the two-lag combinations (rung
  106's lag, the slow sensor), the floor asking for a cut at the start (the governor-off route keeps showing it).
* **The sag's decomposition** (added after scoring, the headline rests on it; step 0.005): both edges instant ⇒ no sag;
  an instant attack with a 1.6 τ release sags; a 0.2 τ attack with an instant release sags more than 0.02 and carries
  more than 85 % of the 0.2 / 0.4 run's sag.
* **Dropped:** the anchor's governed lean-hold gate. The hold never fired in the admitted region — 0 of 1 344 runs in
  § 0 G3′, 0 of the crash map's 270 admitted runs — so a test of it had nothing to hold (it was vacuous, 0 of 126
  points); the minimum-fuel stop stays as a safety net, tested on rung 106's path.

## 3. Scored (anchor § 3) — 38 HIT, 10 MISS, 12 VACUOUS

Cells: the default slam (1000 → 1400 K over 0.5 τ, settle 2 τ, step 0.02), the schedule (margin 0.25), the thrust
hold, the four maps, `τ_att` 0.2, `τ_rel` ∈ {0.2, 0.4, 0.8, 1.6}. Reading rules as frozen; the vacuity rules coded
before the first run (`score.rs` header).

| Claim | Result | What was measured |
|---|---|---|
| **H1** the cut deeper governed (`max g`) | **4 HIT** | 2.35–2.40× the open ramp's |
| **H1** the cut asked for earlier | **4 HIT** | first `req > 0` at s = 0.06 governed, 0.12–0.14 open |
| **H2** the 95 % time rises with `τ_rel` | **4 VACUOUS** | at 1.6 τ the thrust never reaches 95 % in the 2.5 τ run; over the three that do it rises (1.17, 1.56, 2.33 on flow/press) |
| **H2** later than instant at each `τ_rel` | **8 HIT, 4 MISS, 4 VACUOUS** | every MISS is `τ_rel` 0.2: governed 1.14–1.17 against instant 1.23–1.26 — the attack's let-through arrives first; VACUOUS at 1.6 |
| **H3** the release costs less governed | **4 VACUOUS** | the 1.6 τ runs (governed and open) never reach 95 % |
| **H4** the LP minimum `τ_rel`-free and before the crossing (floor) | **6 HIT** | at points 8–9, the crossing at 16–18, bit-identical over the four `τ_rel`; below 0.75 while the floor cuts; no lean hold in any release |
| **H5** the HP minimum rises with `τ_rel` (floor) | **6 MISS** | the HP minimum ALSO lies before the crossing (points 13–15), so it is `τ_rel`-free, bit for bit |
| **H6** the governed thrust sags (> 0.002) | **16 HIT** | 0.033–0.049, rising with `τ_rel` on every map |
| H6's comparisons: open / instant never sag | 16 / 4 HIT | 0 drawdown every one |

**H1's premise, checked:** the anchor took `max g` as `τ_rel`-free by the pinned trigger. It is not exactly: the
deepest cut sits AT the crossing, and that point's step already read `τ_rel` in its inner stages — a hair (≤ 4e−4
relative), gated.

**H5's miss is the governor's, not the slower attack's** (reported, `score.txt`): with the governor OFF at the same
attacks (0.2, 0.4) the HP minimum lies AFTER the crossing and rises with `τ_rel` (flow/press 0.8641 → 0.8660), as rung
52 § 4 found at its own 0.02. The governor's early demand pulls the HP's lowest point forward into the attack, where the
release cannot reach it — so beside a governor the release debits NEITHER spool. **This BOUNDS rung 52's rate verdict to
the open ramp.**

## 4. After scoring (reported, not scored)

* **The sag's two edges** (`sagcause.txt`, the thrust hold, step 0.005, flow/press; flat-LP the same shape): both lags
  0.005 — no sag; attack 0.005 with release 0.1 / 0.4 / 1.6 — 0.5 / 1.5 / 3.0 %; attack 0.2 with release 0.005 / 0.4 /
  1.6 — 3.7 / 4.1 / 4.8 %. Against the attack at release 0.4: 0.02 → 1.5 %, 0.05 → 1.9 %, **0.1 → 4.9 %**, 0.2 →
  4.1 %, 0.4 → 2.9 % — NOT monotone; the page's floor (0.2) sits just past the worst. The engine's thrust at the move's
  end climbs steadily with the attack (0.33 instant, 0.67 at 0.1, 0.78 at 0.2, 0.86 at 0.4): a slower attack lets more
  through, but a slow-enough one also cuts less deep (the cut at 0.4 is 60 % of 0.2's), so less is left to sag.
* **H3 with a longer settle** (4 τ, off protocol): the release's cost (1.6 − 0.2) is LARGER governed (2.69 τ) than open
  (2.11 τ) — the opposite of H3's bet — and at 1.6 τ the governed and open 95 % times coincide (3.86 τ): a slow-enough
  release erases the governor's head start.
* **Floor + schedule + redline** (three maps): the sag is there and grows with `τ_rel` (0.0017–0.0054 at 0.2, to
  0.017–0.023 at 1.6 — press/flow at 0.2 just under the 0.002 bar); the redline holds 25–32 points; the minima as on
  the floor alone.
* **The other laws at `τ_rel` 0.4:** every run sags — and so does every INSTANT run, by 5–13 %, except the fan-speed
  laws on flat-LP. The HP rate law (0.1) on flow/press and flat-LP reaches 95 % before the crossing (0.49 τ, its own
  overshoot). The lag's sag is larger than the instant governor's under the pressure ratio and (three maps) HP gain
  10, smaller under the fan-speed laws on most maps. One release, no re-attack, in every run.
* **Crash map** (`crash.txt`, seed 13, 1 000 random governed requests with the release on, every hold and law, random
  limiters, levers, generator, cooling and cap, both ends over the slider range): 0 panics; 270 admitted — 231 to the
  end, 39 stopped, every stop the floor out of reach; **0 "other" stops, 0 minimum-fuel holds, 0 re-attacks** (each
  admitted run re-read through the library: 51 released once, 219 never let go within the run). Refused: 333 the
  attack's floor, 113 the speed laws' edges, 37 the cut asked for at the start, the rest the earlier rungs' refusals.
  Slowest 6.3 s natively.
* **Cost:** a governed release run 0.2–0.34 s natively, against rung 95's 0.29–0.58 s.

## 5. What it does to its neighbours

* **Rung 106** — its sag is CONFIRMED to need no slow temperature limiter: any stored cut beside a thrust governor sags.
  Its seam's premise (the RELEASE decides the sag) is REFUTED at the page's attacks: the slow ATTACK carries most of it.
* **Rung 52** — "cannot debit the spool it watches" SURVIVES the governor (H4), on the structural argument (a minimum
  below the floor lies in an attack phase). Its rate verdict (a slower release debits the other spool less) is BOUNDED
  to the open ramp: governed, the HP minimum moves before the release.
* **Rung 95** — beside the governor the realistic release can reach 95 % thrust EARLIER than the instant schedule (a
  fast release, H2's misses): the let-through is a credit as well as a sag.

## Concessions

* H4's third precondition — that the solvable-ceiling fallback never sets the applied fuel during the release — is not
  recorded by the march, so it was NOT checked; the lean-hold half was (0 holds).
* The knobs' floors (attack 0.2, release 0.1) come from a step check at three flights; past the slider (1.6) only by
  calling the march.
* The scored grid is the thrust hold; the other laws are reported at one release time.
* H2's rise and H3 are unscored at the frozen settle; the longer-settle reading is off protocol.

## The next seam

**The two-speed temperature limiter beside the governor** — rung 52's asymmetric lag on the REDLINE (106 lagged it with
one time constant; 107 left it instant): does a fast attack on the temperature limiter end rung 106's overshoot while a
slow release keeps its sag? And **the governor beside rung 88's slow valve** (still refused).
