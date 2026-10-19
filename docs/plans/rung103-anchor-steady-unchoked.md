# Rung 103 anchor — THE STEADY TWO-SHAFT MATCHERS BELOW CHOKE: the refusal becomes a solve (pre-registered 2026-10-19 — § 3)

**Seam:** CLAUDE.md *Still OPEN*: *"The unchoked LP branch in the STEADY two-spool matchers (38 flags; the transient
BUILT BY 97)."* Rung 97's spec § *The next seam*: *"The steady matchers' unchoked branch (rungs 38/39/42/90/91/92's
cascades — the same equation in a different solve structure)."*

**User decisions (2026-10-19, AskUserQuestion; memory `rung103-decisions`):**
* **Default: OFF in the old code, ON in the pages.** Every shipped path, oracle, golden and older rung's test keeps
  rung 38's refusal; the sandbox views that reach a steady matcher (Blades, Cooling, the Controls view's generator
  check) and this rung's own tests turn the branch on. The older rungs' claims are re-run with it on and what moves is
  REPORTED, not re-scored.
* **Scope: all six steady matchers** — rung 38 (no map), 39 (maps), 42 (interstage bleed), 90 (customer air),
  91 (turbine cooling), 92 (shaft power).
* **Below idle: refuse in words** (rung 33's precedent): where the re-solved engine's net thrust is not positive.

**Status of what was seen before this was written.** Two scratch crates and a throwaway worktree of `2688ce8`
(archived: `docs/plans/scratch-archive/rung103/`, README there). Probe 1 rebuilt rung 39's cascade from public pieces;
probe 2 ran a PROTOTYPE of the device (`worktree-probe.patch`) and printed ON values — thrust, both shaft speeds, both
flow coefficients, `π_LPT`, cost — **on the probe cells of § 4 only**. Rung 39's "the LP compressor's efficiency leaves
the HP pressure ratio alone" was measured on CHOKED cells only; **no below-choke value of H1's quantity was printed
anywhere.** The scored cells (§ 3) were pre-flighted printing the PIN side only and the ON outcome's class.

---

## 0. GIVENS — measured before anything was written

**G1 — who reads the refusal today** (`reach_census.tsv`: a one-call logger at the six refusal sites, the whole gate
run in the worktree; it passed except the page-currency check, which the logger's own bytes explain). 1 642 refusals:
rung 92's 894 (rung 94's holds 714, `cli_golden`'s Rust-owned panels 180), rung 39's 501 (`slice_l_oracle` 318,
`two_spool_oracle` 134, rung 41's gates 36), rung 42's 191, rung 38's 48, rung 91's 7, rung 90's 1. **Python-held
oracles, Python-held CLI segments, rungs 41/42/90/94's claims and two page tests read these refusals** — so turning the
branch on everywhere would move them. Hence the switch (decision above).

**G2 — the probe's picture** (`probe1_census.txt`; the Controls engine `BladeSettings::defaults().design()`, rung 39's
matcher, four map shapes × four flights × `Tt4` 550–1150 K):
* 74 of 208 points refused UNCHOKED; every one at a low throttle.
* Rung 97's helper inside the joint `(f, pt4)` loop (`pt45 = π_HPT·pt4` from the current pass) converges at 70 of the
  74 in 8–60 passes (a choked point: 7–8); the loop never caps.
* An outer scan of the nozzle's mass residual over `π_LPT ∈ [π_pin, 0.9995]` (25 points, the joint loop closed at each)
  finds **exactly one sign change** at each of those 70; the joint loop at a held `π` converges in ≤ 8 passes.
* Where rung 40's transient equilibrium (rung 97's branch) converges, it equals the steady result to every printed
  digit. It does NOT converge at 34 cells where the steady solve does — including CHOKED cells on the `flat-lp` map — so
  it cannot be the only cross-check.
* 4 of the 74 have no running point (3 "the nozzle cannot pass the flow", 1 the LP speed line off the map); about 9
  newly solved points have net thrust ≤ 0 (below idle).
* On choked cells probe 1's rebuilt cascade equals the shipped matcher bit for bit.

**G3 — the prototype on all six variants** (`variants.txt`, `variants_retry.txt`; plain, bleed 0.05, customer share
0.05, cooling 0.03/0.03, generator power 0.05 on the HP and on the LP shaft; maps flow/press and tilted; three flights):
* switch ON at a choked point = switch OFF, **bit for bit, 80 of 80**;
* of 157 refused-unchoked points: 112 solve, 22 below idle, 19 "cannot pass", 4 the generator refusal;
* against the pin's own numbers (the guard skipped): the LP shaft slower at **110 of 110**; the HP shaft slower at 91;
  the LP compressor's flow coefficient higher (further from stall) at 106 — the four exceptions all with the generator on
  the HP shaft; the HP compressor's lower at 94; thrust MORE at only 54 of 110 (ratios 0.91–2.2, the large ones beside
  zero thrust);
* cost: an unchoked ON call up to 17× the refused OFF call at that point.
* **Ten points fail BEFORE the guard** (customer air and the HP generator at 550–650 K: the pinned rebuild's
  "nozzle back-pressure exceeds total pressure" — the pin over-expands the LP turbine below outside pressure, the very
  artifact the branch corrects). Retrying the branch on ANY pinned failure: all ten become "the nozzle cannot pass" (no
  running point); 35 generator refusals keep their kind but quote the branch's share. **The retry adds no point on this
  grid; it corrects the reason.**

**G4 — an independent root for every variant** (`indep.txt`). The LP turbine HELD at a trial `π` (no call to rung 97's
helper), the variant's own body run to its forward rebuild, and an outer bisection on the nozzle's mass balance read off
the RESULT (station-5 flow vs `A8·ρ9·V9`). Unchoked: equal to the prototype's root to **≤ 1.7e−13** in `π` (plain,
bleed, customer, cooling); choked: the residual at the pin is ±1e−15 and the root is the pin. On the two generator
variants the bracket's top (`π` 0.9995) is not evaluable — an LP turbine barely expanding cannot carry the load — so the
gate marches the top down (rung 33's march).

**G5 — rung 39's asymmetry on the CHOKED branch, through the FULL match** (`arrow.txt`; rung 39's own engine, `π`
3/6, `Tt4` 1500, 250 K / 50 kPa / M0.85, flat and mixed maps, 700–1400 K, `η_LPC` 0.90 → 0.91): on the perfect gas
`Δπ_HPC` = 0 bit for bit at 11 of 12 cells and −4.4e−16 at one; on the equilibrium gas |`Δπ_HPC`| 4e−13 … 6e−8,
rising with `Tt4` (dissociation makes `f` read `pt4`). `Δπ_LPC` +0.0094 … +0.028.

**G6 — the pages.** The Blades view's lever request asks rung 85's plant (rung 39's body through the stator hooks); the
Cooling view asks rung 91's matcher through `metal_temperature`'s functions (which rungs 93/94 also test); the Controls
view's generator check asks rung 92's matcher. Two page tests assert the refusal:
`sandbox_blades::a_lever_point_where_the_nozzle_unchokes_is_refused_before_the_schedule` and
`sandbox_cooling::heavy_cooling_starts_the_knobs_search_where_the_engine_solves` — **expected rewrites**, not breaks.

---

## 1. THE DEVICE

**The switch.** `TwoSpoolCore::below_choke` (default `false` — every shipped path exactly as before). Not rung 97's
`lp_pinned`, which its own scoring and red check set.

**The dispatch — only where the pinned path fails.** Each of the six matchers runs its pinned solve and rebuild
unchanged. If that fails — at the unchoked guard or ANY earlier error — and the switch is on, it re-solves once with
the LP turbine free: the same joint `(f, pt4)` loop, the LP pin replaced by rung 97's `try_lp_turbine` at
`pt45 = π_HPT·pt4` (each variant's own `π_HPT`, cooled or not) and the flight's `p0`. A point whose pinned path
succeeds never enters the branch, so **every choked point is bit for bit the switch-off point**. If the branch fails
too, its own reason is given (rung 97's "the nozzle cannot pass", the generator's, the map's).

**Below idle.** In the branch, where the core's specific thrust or the engine's net thrust is not positive:
`rung-103: below idle at Tt4=…: with the nozzle unchoked the engine makes more drag than thrust here`. Checked before
the shared scoring (which cannot take a negative thrust: its efficiency check fails on it).

**State.** The branch's `(pt4, p0)` lives on the core for the length of one re-solve and is cleared on every exit,
panics included (a drop guard). Rung 38's matcher stays infallible: its branch failures panic with their own words.

**`A45` stays assumed choked** — rung 38's concession, carried (rung 97 § Concessions).

## 2. GATES (bars frozen here)

* **R1 — switch off ⇒ nothing moves:** the whole gate passes unchanged with the switch's default.
* **R2 — switch on, choked ⇒ bit for bit:** at every choked cell of § 3's grids, all six variants, ON = OFF to the bit
  (thrust, both speeds, both flow coefficients, `π_LPC`, `π_HPC`).
* **R3 — the independent root** (G4's instrument, shipped as a test): at the unchoked cells of § 3's H2 grid, every map
  variant, the held-`π` outer bisection's root equals the shipped `π_LPT` to **≤ 1e−11**; calibrated first on choked
  cells (root = pin to ≤ 1e−12). Generator variants: the top marched down in 0.01 steps until evaluable.
* **R4 — rung 40's equilibrium:** the plain map matcher against `TwoSpoolTransientCore::try_equilibrium` (rung 97's
  branch) at the cells where that converges — both speeds and `π_LPT` to ≤ 1e−8 relative, after measuring the same
  comparison on choked cells (its own noise floor, reported); rungs 90/91/92's existing plant gates (steady =
  the Controls equilibrium with no lever armed) re-run at unchoked cells with the switch on, at their own bars.
* **R5 — a hand-written perfect-gas solve** (closed-form turbines, sonic `A45`, subsonic nozzle, its own bisection on
  `π_LPT` around its own fixed point on the two shaft balances) for rung 38's matcher and rung 39's on flat maps:
  `π_LPT`, `π_LPC`, `π_HPC` to ≤ 1e−9.
* **R6 — the boundary:** `Tt4` bisected onto the unchoke boundary (each variant, two flights); just below it the branch's
  `π_LPT` meets the pin's to ≤ 1e−9.
* **R7 — mass at the nozzle:** at every ON unchoked point, `A8·ρ9·V9` from the rebuilt exit equals the station-5 flow to
  ≤ 1e−11 relative (a flow the branch never reads).
* **R8 — state cleared:** after every call — success, refusal, panic — no re-solve state is left on the core.
* **R9 — the red check:** with the branch's LP turbine forced back to the pin (scratch edit), R3, R5 and R7 fail.
* **R10 — the pages:** each newly reachable refusal (below idle, cannot pass, the generator in the branch) has plain
  words driven by a test; the two G6 tests are rewritten to the new behaviour, and the Cooling path takes the switch
  from the page, never from a library default.

## 3. SCORED (bars frozen here; cells never probed — `preflight.txt`)

Scored cells are those where, with the switch OFF, the matcher refuses UNCHOKED; with it ON it solves (not below idle,
not "cannot pass"); and the guard-skipped pin runs. The pre-flight printed only the pin side.

**H1 — THE HEADLINE: below choke, the HP spool hears the LP compressor.** Rung 39's finding A is a one-way street:
the LP compressor's efficiency cannot reach the HP pressure ratio (G5: exactly zero through the full match on the
perfect gas). Below choke the LP turbine's expansion reads `pt45/p0`, `pt45 = π_HPT·π_b·π_HPC·π_LPC·pt2`, so a better LP
compressor raises `π_LPC`, raises the pressure the nozzle pushes with, lets the LP turbine expand further (lower
`π_LPT`), gives the LP shaft more work, raises `Tt25` — and the HP compressor, with the same work from its own pinned
turbine onto a hotter inlet, has a smaller temperature ratio and so **a LOWER pressure ratio**.
* Quantity: `Δπ_HPC` for `η_LPC` 0.90 → 0.91 (the core's base value; a flat map reads it directly, a shaped map shifts
  its island with it), switch ON, against the same cell with the guard skipped (the pin).
* **Bar: at every scored cell `Δπ_HPC < 0`, `|Δπ_HPC| ≥ 1e−6`, and `|Δπ_HPC| ≥ 100 ×` the same cell's pin value.**
* Cells (38): rung 39's own engine on the perfect gas and the equilibrium gas, maps flat / mixed / steep, flights
  250 K / 50 kPa / M0.85 and sea level M0.50, `Tt4` 450–750 K step 50 where scored.
* Rung 39's gate 4 stays exactly true: it holds `pt4` and calls the cascade directly, so it never sees the branch. The
  new arrow exists only through the full match.

**H2 — the LP shaft runs slower than the pin said** (rung 97's mechanism, now on the steady matchers): at every scored
cell, switch-ON `N_L/N_L,d` < the pin's.
* Cells (174): the Controls engine, maps `steep` and `flat-eta`, variants plain / bleed 0.08 / customer share 0.03 /
  cooling 0.02-0.04 / generator power 0.02 on the HP and on the LP shaft, flights sea level +20 K M0.60, 2 km standard
  M0.50, sea level −10 K M0.40, `Tt4` 575–825 K step 50 where scored (11–16 cells per variant and shape; none vacuous).

**Reported, not scored** (on H2's cells, against the pin): the HP shaft's speed; both compressors' flow coefficients
(G3: the LP's higher at 106 of 110 probe points, the exceptions with the HP generator — not derived, so not scored); the
thrust DIFFERENCE (not a ratio) — G3 already shows it below the pin at about half the points, which BOUNDS rung 97's
"more thrust" to its own cells (a cross-rung correction, written into both specs).

## 4. Probe cells (excluded from scoring)

The Controls engine on maps flow/press, press/flow, tilted, flat-lp; flights sea level +40 K M0.85, sea level M0.85,
sea level M0.30, the design flight; `Tt4` 550–1150 K; variants bleed 0.05, customer share 0.05, cooling 0.03/0.03,
generator power 0.05 on each shaft. Rung 39's engine at 700–1400 K (choked cells only), flat and mixed maps.

## 5. After scoring — reported, not scored

* **The older rungs with the switch on:** rung 41's surge-margin schedules and turn (they skip a refused point), rung
  42's "the valve shrinks the choked envelope" count, rung 90's amount-mode envelope, rung 94's holds, rung 92's
  generator refusals — each re-run with the switch on; what moves is listed, no verdict re-scored.
* **The pages:** a seeded crash map of Blades-lever and Cooling requests at low throttles, switch on against the code
  before this rung, request by request; no panic allowed.
* **Cost (trigger, not a bar):** on the page's own requests (a Blades lever reply, a Cooling sweep reply, the Controls
  generator check) the switch-on reply at an unchoked throttle ≤ 5× the same request's reply at the nearest choked
  throttle. If exceeded: count the joint-loop passes and helper calls before changing anything (rung 102 lessons 1, 5).
