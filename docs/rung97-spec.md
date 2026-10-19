# Rung 97 — THE UNCHOKED NOZZLE ON TWO SHAFTS: below the unchoke boundary the LP turbine's expansion is the unknown

**Code:** `rust/src/unchoked_lp.rs` — the helper `try_lp_turbine` (rung 38's pin while the nozzle stays choked, else the
subsonic root), `PI_TOP`. `two_spool.rs`'s `TwoSpoolCore` gains `lp_pinned` (the old model, for scoring only). Wired into
every transient LP-turbine site, tails and shaft-power hooks alike: `two_spool_transient.rs` (`r40_try_instant_tail`,
`r40_powers`), `bleed_transient.rs` (`r62_tail_body`, `r62_powers_body`), `customer_bleed.rs` and `turbine_cooling.rs`
(their `turbines`, which rung 90's and 91's power hooks now call with the flight).
**Gates:** `rust/tests/rung97.rs`; `rust/tests/sandbox_altitude.rs`'s refusal test, changed (§ 4).
**Panel:** `print_unchoked_lp_table` (born in Rust, held to `rust/oracle/rust_owned/`; a change detector only).
**Anchor:** `docs/plans/rung97-anchor-unchoked-lp.md`, committed as `36b4908` before any rung-97 code existed.
**Sandbox:** the Controls view draws unchoked stretches thin and dashed and counts them (`docs/plans/sandbox-plan.md`
§ 28). **Scratch scripts and outputs:** `docs/plans/scratch-archive/rung97/`.

CLAUDE.md's open seam *"the subsonic / unchoked LP branch in the two-spool solves (38 flags, unsolved) and its
transient"*, built for the transient family. It came up while adding a Mach knob to the Controls view: that engine ran
points whose nozzle was unchoked and computed them on rung 38's two-choked-throat pin anyway — the tail only flagged
them, and the page drew them. Offered three repairs (refuse in words, draw dashed and warn, build the branch), the user
chose to build it ("Fix it properly first", 2026-10-18). The steady matchers (rungs 38/39/42/90/91/92's cascades) keep
their refusal: **transient BUILT, steady still OPEN** — the steady half BUILT BY RUNG 103, behind a switch (off in the old code, on in the pages).

> **THE HEADLINE — below the unchoke boundary the nozzle's back-pressure takes work off the LP shaft ALONE.** The
> compressors, the HP nozzle guide vane and the HP turbine never read the LP turbine, so at a given instant the HP
> shaft's power is unchanged to the bit and only the LP turbine expands less. The LP shaft therefore settles slower —
> and slowing lowers the pressure the nozzle has to push with, which unchokes it further, a loop that runs one way.
>
> On a +40 K day at sea level, at a 700 K throttle, rung 38's pin over-expanded the LP turbine by 2.4 %: the real LP
> shaft runs **1.6–1.8 % slower** than the old model said, the HP shaft 0.02–0.03 % slower, and the engine gives
> **3.6–4.8 % MORE thrust** — the jet leaves faster from the higher turbine-exit pressure. Through the 700 → 1400 K slam
> its peak `Tt4` is about 10 K hotter, and its LP shaft stays behind the old model's through the whole ramp. On a standard day
> the 700 K end is barely unchoked and every difference is about a hundred times smaller.

## 1. What it adds

**The equation.** Below the unchoke boundary only `A45` (the LP turbine's nozzle guide vane) stays choked; the
convergent nozzle expands fully (`p9 = p0`, `M9 < 1`) and passes `A8·ρ9·V9` at the ACTUAL `pt9/p0`:

```
resid(π_LPT) = A45·pt45·MFP*(Tt45, f45)/√Tt45 − A8·ρ9·V9 = 0,     pt9 = π_n·π_LPT·pt45
```

rung 33's `(★★)`, one throat downstream. `π_LPT` is the unknown; `τ_LPT` and `Tt5` follow from the LP turbine's map
efficiency at the instant.

**The helper** `try_lp_turbine(core, gas, Tt45, f45, η_LPT, pt45, p0)`:
1. rung 38's pin, exactly as before;
2. if the nozzle at the pin's exit is choked (the tails' own test, `p9 > p0 + 1e−6`), the pin, untouched;
3. else bisection on `[π_pin, 0.9995]` (anchor G3: one root there at all 1 667 unchoked instants probed), rung 38's
   tolerance (1e−13, 200 steps); a trial `π` at which the nozzle cannot push the gas out has zero demand;
4. if even `π` = 0.9995 leaves supply over demand, a refusal in words (`rung-97: …`), never a clamp.

One helper serves every variant: each tail builds the nozzle inlet as `pt5 = π_LPT·π_HPT·pt4` at its own `(Tt45, f45)`
(rung 91's cooled HPT supplies its own), and no variant adds mass between `A45` and `A8` (anchor G5).

**On the page:** the Controls chart draws the unchoked stretches thin and dashed, and its table counts them ("Nozzle
unchoked (of the run's moments)"). The numbers there are now the model's own.

## 2. Reduces and gates

* **Choked ⇒ bit for bit** (dispatch at step 2): the default move — which never unchokes — marches identically with
  the switch on or off, bit for bit, and no oracle or Python-held golden meets an unchoked instant (anchor G2's census
  over the whole gate). Every other gate passes unchanged except the one § 4 names.
* **`lp_pinned` ⇒ the pin, bit for bit:** at an unchoked end the switched instant's `π_LPT` is `try_solve_choked_turbine`'s
  to the bit, and its HP turbine and HP power residual are the unswitched instant's to the bit.
* **An independent solve:** a hand-written perfect-gas solve — closed-form turbine, sonic throat and subsonic nozzle, its
  own bisection — matches the shipped root to ≤ 1e−10 on three maps.
* **The boundary, every variant** (plain; the generator on the HP or the LP shaft; customer air; cooling; the bleed and
  stator levers). Bisected onto the unchoke boundary (808.6–849.0 K at sea level +40 K), the root meets the pin to
  **5.2e−14** (bar 1e−9), and the equilibrium's Newton converges on its primary bar in 5–6 steps on both sides of the kink.
* **Mass conservation at the nozzle, every variant** — added beyond the anchor, because the boundary gate cannot see a
  wrong `pt45` (it cancels from the pin and the root alike): at every unchoked point of the 700 → 1400 K move, `A8·ρ9·V9`
  at the tail's own `pt5` equals the gas the engine delivers (burner gas, plus cooling air), a flow the helper never
  reads. Worst **7.6e−14** (bar 1e−11).
* **The fixed point, every variant:** a march held at an unchoked steady end's fuel drifts ≤ 1.4e−13 in 2 τ (bar 1e−9) —
  so no tail branched without its power hook.
* **The red check:** with the switch's default flipped to the pin, three of the five gates fail, each on a wrong VALUE
  (the independent solve, mass conservation, the expansion gap).
* **The step check** (0.02 against 0.005, max flow-coefficient difference) on the unchoked 700 → 1400 K runs, three maps,
  +40 K and standard day: the open march ≤ 2.2e−8; governed runs ≤ 1.4e−6 (thrust), 6.4e−5 (HP proportional),
  9.4e−5 (HP rate), 5.4e−5 (pressure ratio); the fan-speed rate law 1.3–1.45e−4 unclamped and up to 4.0e−4 at its rich
  edge. **The same with the pin**, to the second digit (e.g. 1.45e−4 against 1.44e−4): the bisection adds no march noise;
  the dense-air roughness is plan § 27's, already disclosed.

## 3. Scoring (anchor § 3 — bars frozen at `36b4908`)

Cells: the 700 K throttle (H1) and the 700 → 1400 K slam over 0.5 τ (H2), sea level at Mach 0.85 on a +40 K and a
standard day, each map; "old" is the same run with `lp_pinned`. Floor: a speed difference counts only above 1e−9.
Flat-LP has no steady point at 700 K on either day: **2 cells VACUOUS** each.

| # | verdict | measured |
|---|---|---|
| H1 the LP shaft runs slower at an unchoked steady end | **HIT** 6/6 | +40 K: −1.80e−2 / −1.64e−2 / −1.79e−2 (flow/press, press/flow, tilted); standard: −2.01e−4 / −1.18e−4 / −2.30e−4 |
| H2 it stays behind the old model's LP shaft through the ramp | **HIT** 6/6 | behind at all 25 ramp points of every cell; largest gap = the start's |

**The named risks, measured even on the HITs** (`score.txt`):
* the HP shaft also settles slower, by 2.4–3.1e−4 (+40 K) and 1.2–2.6e−6 (standard) — the same sign, a hundred times
  smaller: H1's cross-coupling did not reverse it;
* the two LP trajectories never cross over the whole run, settle included, above the floor — except once, on the
  standard day's press/flow map at s = 2.16 τ, where the gap is 2.1e−9, at the floor, in the decaying tail after both
  runs have choked.

## 4. After scoring (reported, not scored)

* **Thrust at the unchoked end: MORE than the old model said** — 33.58 / 34.40 / 33.29 against 32.04 / 33.19 / 31.79
  N per kg/s of design airflow (+40 K; +3.6 to +4.8 %); equal to the second decimal on the standard day. The LP turbine
  takes less, so the gas reaches the nozzle at a higher pressure (`pt5/p0` 1.590 / 1.596 / 1.589 against 1.580 / 1.587 /
  1.579) and leaves faster: each kg of air gives 5.5–6.7 % more thrust, while the slower LP shaft passes 1.7–1.8 % less
  air (`followup.txt`). The faster jet wins. No sign was derived for this — and it does not generalise: on rung 103's
  174 steady cells (six variants, other maps and flights) thrust is MORE than the pin's at only 50 (`docs/rung103-spec.md` § 3).
* **Through the slam** (+40 K): the peak `Tt4` is 9.7–10.5 K higher; the LP compressor's lowest flow coefficient is
  0.0004–0.0017 HIGHER (further from its stall line) and the HP compressor's 0.0018–0.0023 LOWER (nearer). At the start
  end the LP's is 0.007–0.008 higher and the HP's 0.001 lower. The unchoked stretch is 9–10 of 126 points.
* **§ 27's "the nozzle cannot push it out" refusal** (`sandbox_altitude`'s request: 600 K in warm, dense air with a large
  customer amount) no longer fails at the nozzle. The end's search now runs the LP shaft slower, and it leaves the
  compressor map — refused with the off-map words. **The anchor expected "it now runs, or fails with the rung-97
  words"; it did neither** — a third outcome, which H1's mechanism explains. Its test now asserts the off-map refusal.
* **Rungs 95 and 96: unmoved.** Their archived scoring scripts (rung 96's `score`, `report`, `claims`; rung 95's `score`,
  `ends`) re-run against this rung print the same, line for line, timings aside (`rerun.txt`). The only lines that
  differ are rung 95's two step checks at gains 200 and 100, which the page's later gain limit (50) now refuses — not this
  rung. So the census's hits in those tests changed no printed number; no verdict of either spec moves.
* **The crash map, against the code just before this rung.** Sandbox § 27's script (seeds 14 and 15: 1 400 requests,
  −5 … 47 km, ±100 K) and a new seed 16 (1 000 requests at −1 … 3 km, where the nozzle unchokes), run back to back on
  this rung and on `36b4908` in a worktree, and compared request by request (`crash_compare.txt`). **No panic on
  either.**
  * Seed 15: one request differs — a refused thrust hold now quotes a different thrust.
  * Seed 14: 15 differ — 7 refusals of the same kind quoting moved numbers (thrust holds whose unloaded target sits at an
    unchoked end); 5 refusals now of another kind (among them the pin's "the nozzle cannot push it out", now the off-map
    words — the test's request above — and the customer-air words); one stopped run and one off-map refusal now run to
    the end; one stopped run is now refused in this rung's words (14.9 km, +29 K, 760 K, the generator on the HP shaft).
  * Seed 16: 28 differ. **15 runs the pin had run (12 to the end, 3 stopping early) are now REFUSED**: 5 in this rung's
    words, every one with the generator on the HP shaft (power 0.05–0.09), at 670–940 K in warm air; 6 by the acceleration schedule, whose steady line now lacks a point; 2 with the generator
    words; 2 by a hold. 4 stopped runs now run to the end; 4 refusals change kind; 5 quote moved numbers.
  * **The pin invented steady points.** For seed 16's #107 (the generator on the HP shaft at power 0.09, 940 K, sea level +20 K), a scan of the
    LP-balanced curve for an HP balance finds one equilibrium on the pin and none with the branch (`miss107.txt`) — the
    refusal is the model's, not the Newton's. At 1 000 K both have one, the branch's LP shaft 1.7 % slower.
  * Rung 97's own refusal appears 1 / 1 / 6 times (seeds 14 / 15 / 16) and reads in words; where the generator is on, the
    generator-off check finds a steady point and names the load.
  * Median request on the same machine, back to back: 0.014 / 0.002 / 0.035 s against 0.012 / 0.002 / 0.033 s.
* **The generator-refusal check stays on the pin.** The Controls view still asks rung 92's STEADY matcher whether a
  load removes a steady point; at an unchoked throttle its answer is the pin's. In seed 16 two runs the pin had run now
  refuse with the generator words and two earlier generator refusals take other words — where the branch removes a
  steady point under a load, those words name the right cause, but any figure they quote comes from the pinned cascade.
* **Cost:** a choked instant pays one nozzle evaluation more in its power hook; an unchoked one ~44 bisection steps,
  each a turbine and a nozzle evaluation. The march, best of 15, interleaved: **+1 %** on the default move (never
  unchoked), **+5 %** on the +40 K 700 K move (`cost.txt`).

## 5. Why — the model

At an instant `(ν_L, ν_H, Tt4)` the close (both compressors on their maps, the HP nozzle guide vane's choke) sets the
airflow, `pt4` and the HP turbine without reading the LP turbine — rung 38's triangular cascade. The pin then forced the
LP turbine to the expansion that keeps the nozzle sonic. Below the unchoke boundary that expansion is too deep: the
nozzle could not pass the flow at the pressure the pin left it. The root expands the LP turbine less (by up to 7 % in
`π` over the probes), so its work per unit gas falls, the LP shaft's power residual falls, and the HP shaft's does not
move. At the old equilibrium the LP shaft is then short of power, and it slows; slowing lowers `π_LPC`, so `pt45` and
`pt9/p0` fall, the nozzle unchokes further and the LP turbine loses more expansion. The loop has a fixed point in every
cell measured — the equilibrium converges in 5–6 Newton steps either side of the boundary.

This is rung 33's inversion relocated one throat upstream, as rung 38 predicted: on the choked branch the LP turbine's
expansion is geometry; below the boundary it re-couples to the compressors and to the air outside.

## What it does to its neighbours

* **Rung 38 / 40 — their concession DISCHARGED for the transient.** "Nozzle unchoke is out of scope" is now solved
  wherever the two-shaft engine is marched; the steady matchers still refuse.
* **Rung 33 — its result extended to two shafts:** the decoupling breaks one throat further down, and only for the LP
  shaft.
* **Rungs 95/96 — unmoved** (§ 4): their scored slams never unchoke, and no printed number sat on an unchoked instant.
  Rung 96's G2 ("equal pressure ratio ⇒ equal gross thrust") SURVIVES the unchoked branch, measured: 60 pairs of
  unchoked instants matched in pressure ratio give gross thrusts equal to ≤ 9.2e−14 (`followup.txt`). A fully expanded
  convergent nozzle's gross push, `A8·p0·(2γ/(γ−1))·((pt9/p0)^g − 1)` on the perfect gas, depends on `pt9` alone, as the choked one's does. What
  does NOT survive is its G3 — gross thrust AFFINE in the pressure ratio — so on unchoked stretches the table's linear
  rows carry an interpolation error in the gross part (not measured). Its controllers keep one root: at frozen shaft
  speeds thrust, the pressure ratio and both accelerations still rise strictly with fuel at every one of 14 394 fuel
  steps over the +40 K 700 → 1400 K move (plain and LP generator, three maps), 1 362 of them unchoked and 22 sweeps
  crossing the boundary.
* **Sandbox § 27** — its one new end failure is gone; the off-map words now cover that request.

## Concessions

* **`A45` stays assumed choked** (rung 38's concession, carried). At the root the LP turbine expands by only 1.36–1.45
  overall; a real vane row would be well short of sonic there — and on the choked branch alike.
* **The steady matchers keep their refusal** — the status was "transient BUILT, steady still OPEN"; **BUILT BY RUNG 103**, behind a switch.
* **The generator-refusal check on the Controls view still reads rung 92's steady (pinned) cascade** — DISCHARGED BY RUNG 103 (the check turns the branch on).
* Inherited: invented maps, the perfect gas, one `η_m`, and the Controls view's flight at Mach 0.85 (a Mach knob is the
  next sandbox change).

## The next seam

**The steady matchers' unchoked branch** (BUILT BY RUNG 103) (rungs 38/39/42/90/91/92's cascades — the same equation in a different solve
structure), and **a choked-or-not `A45`**: with the LP turbine expanding under 1.5, its nozzle guide vane is the next
throat whose choke is assumed rather than earned.
