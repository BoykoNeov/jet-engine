# Rung 102 — THE AFTERBURNER: a second burner behind the turbine, on the design, on the engine you fly, and lit during the slam

**Code:** `rust/src/afterburner.rs` — `Afterburner` (the knobs: `tt7`, `eta_ab`, `pi_fh`, `m6`), `AbMode` (`Unlit` /
`Lit`), `try_afterburn` (station 5 → 7), `fuel_frozen` / `fuel_equilibrium` (the fuel), `rayleigh` (the heated duct),
`try_score_total` (thrust on the total fuel), `build_afterburning_turbojet` / `AfterburningTurbojet` (Part A's builder,
continuing `build_turbojet`'s own run from station 5). `gas.rs`: the equilibrium section's burn-condition guard kept PER
fuel/air ratio, and `Gas::try_t_from_h_t`. **Gates:** `rust/tests/rung102.rs`. **Panel:** `print_afterburner_table` (born
in Rust, held to `rust/oracle/rust_owned/`; a change detector only). **Anchor:** `docs/plans/rung102-anchor-afterburner.md`,
committed as `379646c` before any rung-102 code. **Sandbox:** the Design view's *Afterburner* panel,
Fly it's and the Controls view's (`docs/plans/sandbox-plan.md` § 34). **Scratch:** `docs/plans/scratch-archive/rung102/`.

CLAUDE.md's open item *"Afterburner"*, SPEC.md's *"a second burner between turbine and nozzle, with the nozzle re-sized"*.
The scope was set on measured probes, each answer the user's (memory `rung102-decisions`): ONE rung in three parts —
**(A)** the design point, **(B)** the one-shaft engine flown off design (*Fly it*), **(C)** the two-shaft slam
(*Controls*) with a light-off at a chosen moment; the duct loss DERIVED; a request past the oxygen REFUSED in words naming
the ceiling; on a built engine the nozzle a slider (throat over the dry throat) with a *match* setting; rungs 96–101's
thrust guesses refused with the afterburner lit.

**Status: Parts A, B and C shipped** (2026-10-19). Part C's predictions were frozen in the anchor's § 5 addendum
(`9128307`) before its code; 335 of its 360 cells hit.

> **THE HEADLINE (Part A) — the afterburner pays most flown fast, the nozzle must open MORE than the square-root rule,
> and dissociation sets the ceiling.** Lit to the same temperature, the same engine gains more thrust at higher flight
> speed for a smaller fuel-rate penalty (every one of 36 orderings across four flights; the project's design at
> 1900 K: ×1.35 thrust for ×1.39 fuel per thrust at Mach 0.85, ×1.64 for ×1.36 at Mach 2). The throat must open by
> `√(Tt7/Tt5)` over the heat-addition loss AND by another 1.1–5.9 %: the added fuel's mass and the hotter gas's lower
> choked flow per area. On the equilibrium gas burning all the oxygen left reaches only 2211–2244 K — 60–79 K short of
> the reacting gas, which keeps no dissociation — and near the ceiling dissociation costs 1.7–3.1 % more afterburner
> fuel. Mattingly's § 8.4 reference engine runs through the shipped model to his own equations at 1e−12.

## The device (anchor § 1)

Stations 5 → 6 → 7 → 9 (Mattingly Fig. 7.8); 5 → 6 a diffuser, isentropic, so 6 has no row. **The fuel:**
`(1+f5)·h_t(Tt5, f5) + η_AB·(f7 − f5)·h_PR = (1+f7)·h_t(Tt7, f7)`, `f7` the TOTAL per unit air — on the perfect gas
Mattingly's (7.29); on the reacting gas the fixed point `f7 = g(f7)` (increasing, a contraction, so iterates seeded at
`f5` rise onto the root and an iterate past the cap means the root is past it); on the equilibrium gas the scale-B
absolute balance with the station-4 FROZEN mixture at `Tt5` as reactant and the equilibrium products at `(f7, Tt7,
pt7)`, iterated with the duct's `pt7` (2–4 passes measured on Part A's grid, cap 50; the anchor's trigger was 10). **The
stoichiometric cap** `f_stoich·(1 − 1e−6)` on every gas. **The duct:** `π_AB = π_fh·π_R`, `π_R` the constant-area
heat-addition (Rayleigh) ratio from the three conservation laws (mass with the fuel added, momentum, energy), the exit
state parametrised by its kinetic energy; `M6 = 0` ⇒ `π_R = 1` by dispatch. **The nozzle** sees station 7; on the
equilibrium gas the afterburner freezes its own mixture. **Thrust, TSFC and efficiencies on `f7`** (Mattingly
7.30–7.32), through `try_score_total`, a deliberate copy of `engine::try_score` taking the fuel explicitly. **Defaults:**
`Tt7` 2000 K, `η_AB` 0.95, `π_fh` 0.97, `M6` 0.20 (anchor G5: 0.944 lit at a doubling, Mattingly Problem 7.9's 0.94).

**What the build changed from § 1 (disclosed):**
* The duct is solved by the general three-law solve on EVERY gas, the perfect gas included (§ 1 said "closed forms on
  CPG"): the closed form has no fuel-mass term, and one code path keeps the two gases comparable. The textbook closed
  form is the GATE's instrument instead (mass factor 1, below).
* The exit state is parametrised by its speed (`u = V7²/2` read off it), not `T7`: the first crash map found ducts at
  Mach 0.001–0.002 failing the momentum closure — a bracket on `T7` resolves `Tt7 − T7` (a millionth of `T7`) only to
  1e−8. (Part B solves it for `V7` on the momentum residual times `V7`, finite at stagnation, by Illinois false position
  — the duct and the equilibrium fuel search lost their bisections there; Part A's replies moved ≤ 3.2e−11.)
* The equilibrium fuel search never asks for the cap up front: the rung-6 Newton fails at isolated points hugging
  stoichiometric (φ 0.999999 at 1500 K, 3.2 bar; 1540–1575 K at 5.8–8.6 bar in the crash map) where no afterburner that
  runs ever probes. The bisection approaches the cap only when the root is up there; the cap test steps back to
  `1e−5`, `1e−4`, `1e−3` below stoichiometric only when the Newton fails at the edge (a ceiling within about a kelvin).
* The ceiling and the choke ceiling are named at the burning-zone entry pressure `π_fh·pt5` (the duct's few-percent drop
  moves dissociation by a few kelvin at most).

**Refusals, in words:** too hot (naming the ceiling); too cold (`Tt7 ≤ Tt5`; on the equilibrium gas below the frozen
mixture's own recombination heat, naming that floor — anchor G2); a choking duct (naming the hottest it takes); on the
Design view the knob ranges (`η_AB` 0.5–1, `π_fh` 0.85–1, `M6` 0–0.35).

## Givens (anchor § 0)

G1–G6 as frozen: the design point on four gases (G1), the equilibrium ceiling and the recombination self-heat (G2), the
Mattingly § 8.4 reference (G3: the book's printed thrust 0.34 % below its own equations, source not found), the held
nozzle on the built one-shaft engine (G4: thrust BELOW dry), the derived duct loss (G5), the steady two-shaft matcher
(G6, not the Controls plant). The probe slip the anchor disclosed (G4's "opened" line re-ran the dry engine) stands as
written: Part B's gate measures it.

## Gates (anchor § 2) — measured (`tests/rung102.rs`)

* **Mattingly's equations through the shipped model** — `f`, `f_AB`, `F/ṁ0`, `S` against (7.20)/(7.33) at G3's inputs
  and at Mach 2 / 11 km: **1e−12 relative** (`M6` 0, `π_fh` 0.94, dual perfect gas, `M0` 1e−3 both sides).
* **The duct solve is the textbook Rayleigh line** on a consistent perfect gas (`cp = γR/(γ−1)`), mass factor 1: `π_R`
  to 1e−10, `M7` to 1e−8, over `M6` 0.05–0.3 × heat ratios 1.1–2.5, and thermal choke exactly where the closed form
  chokes. (On `Gas::default()`, whose rounded constants disagree by 0.05 %, the gate would read that disagreement.)
* **`Unlit` with `π_fh` 1 is the shipped engine bit for bit** — station 9 and every performance number, four gases (so
  `try_score_total` at station 4's fuel IS `try_score`).
* **The per-`far` guard:** a second fuel/air ratio at its own burn condition is allowed; one ratio at two conditions
  still fires (rung 6's own gate unchanged). It moves no number: every oracle bit-exact (the gate).
* **Conservation every run:** energy (1e−9; the equilibrium absolute balance 1e−6), mass, the duct's momentum (1e−9).
* **Refusals:** too hot on every gas, the named ceiling itself running 2 K below; too cold on every gas, and on the
  equilibrium gas `Tt5 + 0.01 K` (the reacting gas runs it); a choking duct on every gas, its ceiling named; a
  Mach-0.001 duct closes.
* **The Design view:** the knobs inert while off (the reply, settings aside, identical); the default reply carries none
  of the new keys; the `defaults` op names all five and round-trips; a lit reply carries the block, station 7, the
  `ts_afterburner` curve.

## Scored, Part A (anchor § 3) — 261 HIT of 261 (`score_a.txt`; H1 in `tests/rung102.rs`)

| claim | cells | HIT | measured |
|---|---|---|---|
| H1 Mattingly § 8.4 printed | 1 | 1 | `f` 0.03581, `f_AB` 0.01955 (book 0.0358, 0.0195), `F/ṁ0` +0.31 %, `S` −0.30 %, `M9` −1.3 %, wet/dry 1.194 |
| H2 pays more flown fast | 36 | 36 | e.g. TPG `π_c` 15, 1400/2100: ×1.44 → 1.84 thrust, ×1.78 → 1.53 TSFC across the flights |
| H3 opens more than `√` | 144 | 144 | +1.14 … +5.93 % |
| H4 the ceiling | 16 | 16 | equilibrium 2211–2244 K; reacting 60–79 K higher |
| H5 dissociation's fuel | 16 | 16 | +1.72 … +3.09 % at 2100 K |
| H6 an unlit duct's cost | 48 | 48 | −1.30 … −0.59 % |

**The named risks, measured even on the hits** (rung 96's lesson):
* **H4's risk was WRONG.** It named 11 km Mach 1.6 (the lowest ambient pressure) as the lowest ceiling; it is
  2232–2241 K, mid-range. The lowest is 3 km Mach 0.5 (2211–2222 K): dissociation reads the burning zone's pressure, and
  Mach 1.6's ram rise holds that up. The highest ceilings are at 250 K Mach 1.2.
* H3's risks were right on both ends: the top (5.93 %) is the equilibrium gas at 2100 K, the bottom (1.14 %) a small rise
  (1700 K from `Tt4` 1600).
* H6's risk was right: sea level static (and 3 km) bite hardest, −1.30 %; the low-thrust 11 km flight least (−0.59 %).
* H5 landed in the lower half (the bar was [1.5, 8] %): at 2100 K the gas is still well below the ceiling.

## Census (anchor § 2) — what the defaults moved

* **Nothing.** Against a clean worktree of `884e60a`: all **335** earlier check requests (every view) byte-equal; a crash
  map of 1 500 random designs with the afterburner off (seed 102): 1 500 / 1 500 replies identical, including the 85 the
  old build panics on.
* **The afterburner's own crash maps** (seeds 103 and 202, 1 500 each, knobs randomized over their ranges): no new
  panic — every one of the 85 / 95 panics is the core engine's own, the same message on the old build with the
  afterburner removed. The first map found the three defects above (the slow duct, the Newton at the cap, an unlit
  readout at near-zero thrust refusing a lit design that runs — now the dry readout is left out); after the fixes every
  refusal is in words: too hot 297 / 292, too cold 225 / 229, a choking duct 20 / 20, the burner's precheck 206 / 195,
  and one efficiency-bookkeeping refusal in slice 1's words.
* **The check grid** gains 12 requests (appended last): a lit afterburner on every gas, two nozzles, `M6` 0, a hot Mach
  1.6 design, three refusals. `browser.mjs` lights it on the opening design (matches native; tiles, knobs and station 7
  shown; too hot refused; off restores the opening run bit for bit).

## Concessions

* The afterburner reads the hot section's properties (exact for Mattingly § 8.4, where `cp_AB = cp_t`; his Example 7.5
  uses a third set). Fuel enters with no enthalpy or axial momentum of its own (the main burner's convention).
* The flame holders are a fixed ratio (not a drag coefficient on the dynamic pressure); the diffuser 5 → 6 is lossless.
* The duct is constant-area with no friction; the afterburner burns completely to equilibrium (no finite-rate flame,
  no screech, no rich burning — the cap is stoichiometric).
* The chart's 5 → 7 jump mixes heat added with a change of reference mixture, as the main burner's does.
* Inherited: the frozen nozzle (rung 14 brackets it), the equilibrium Newton's isolated failures near stoichiometric.

## Part B — Fly it (shipped 2026-10-19; anchor § 3's H7–H9 and its dated § 3b, written on dry runs before the code)

> **THE HEADLINE (Part B) — lit behind a held nozzle, an afterburner makes LESS thrust than not lighting it, and walks
> the compressor to its stall line; opening the throat buys both back, crossing the unlit thrust about halfway to the
> rule.** On the page's opening hardware at four flights off its design point, two throttles and two temperatures, a
> held throat gives ×0.62–0.90 of the unlit thrust at ×1.07–1.22 of its fuel, the shaft slows to 72–86 % of its unlit
> speed, and the stall margin falls by 64–99 % (H7, H8: 64 / 64). Over eleven equal steps to the rule's throat, thrust
> and margin rise at every step and the lit thrust passes the unlit one at `s` 1.08–1.24 of 1.28–1.40 (H9: 16 / 16).
> At the rule's size (*match*) the core runs where it runs unlit — to 1e−10 — and the throat opens 1.19–1.40×, more
> than the square-root rule's 1.16–1.32 (Part A's H3, now on a flown engine).

**Code** (`afterburner.rs`, Part B): `FlyAfterburner` on rung 34's captured plant — `capture` (the duct's area `A6` from
the DRY design run at the `M6` knob; the dry throat `A8_core/π_fh`), `try_instant` (rung 34's instant with the nozzle
relation (★AB) in place of (★): the compressor closed exactly as the shipped instant closes it, the turbine solved so
the throat passes station 7, rung 33's dispatch on station 7's nozzle, the power balance unchanged, thrust on the total
fuel), `try_equilibrium` (the speed search, re-run at the root and required to balance), `try_point` / `try_point_after_dry`
(the throat held at `s` × the dry one, or *match*), `match_area`, `name_refusal`. `try_find_equilibrium_nu` is the
shipped search COPIED fallible (same march, sentinel and Illinois, its panic an `Err` with the same message). In
`sandbox.rs`: `FlySettings` gains `ab` (not fitted / unlit / lit), `tt7`, `a8_ratio`, `a8_match`; `fly_ab`; the
running-line point; `check_ab_fly_requests`. `sandbox_transient.rs`: the slam refuses a fitted afterburner.

**The device.** Choked, `π_t·π_AB·√(Tt4/Tt7) = A4·MFP4·(1+f7)/((1+f5)·A8·π_n·MFP7)`; unchoked `ṁ_air·(1+f7) = A8·ρ9·V9`
at `p9 = p0`. Unlit (`Tt7 = Tt5`, `π_AB = π_fh`) it is rung 31's (★) with the throat `A8·π_fh` — so the fitted engine's
dry throat is the core's over `π_fh`, and the unlit engine sits where the engine without an afterburner sits wherever
the nozzle is choked (the flame holders still cost 0.4–1.1 % of the thrust, through the jet's pressure). Off design
the burning zone's Mach FLOATS with the turbine-exit corrected flow through the hardware `A6` (held lit: 0.15–0.17 from
the knob's 0.20). *Match* solves the dry engine first, sizes the throat from the lit exit at that core (`ṁ7/(ρ9·V9)`,
choked or not), and runs the lit instant at the dry speed; with the throat set by the user the full lit speed search runs.
The station table is the shipped forward rebuild at the point's core, continued through `try_afterburn` and the
convergent nozzle; the throat's mass balance is checked there (`FLY_THROAT_TOL` 1e−7 — the shipped rebuild never reads
the throat area, so a wrong relation would pass every older check).

**Gates (`tests/rung102.rs`, `part_b_*`):** unlit at the dry throat ≡ the engine without an afterburner, choked, at four
flights (speed, pressure ratio, airflow, `Tt5`, `π_t` within 1e−9; measured ≤ 5e−11 — `π_fh·(A8/π_fh)` is not
bit-exact); *match* ⇒ the lit engine solved THROUGH (★AB) at the matched throat lands on the dry core (1e−9; measured
≤ 1.3e−10), and matched at the dry point the core cannot move; an independent instrument — the SHIPPED solver with its
throat scaled by G4's effective factor, iterated to a fixed point — agrees on the held lit point (1e−8; measured
≤ 7e−11); the fly reply (not fitted is the shipped reply byte for byte whatever the keys say; the block, station 7 and
the curve when fitted; the keys round-trip); the refusals in words; one matched point on the equilibrium gas.

**Scored, Part B — 80 HIT of 80** (`score_b.txt`; thermally-perfect gas, the opening design, maps `flow` and `pressure`;
`tilted` reported and hits every cell too):

| claim | cells | HIT | measured |
|---|---|---|---|
| H7 held: lit thrust below unlit | 32 | 32 | ×0.620–0.902 |
| H8 held: the margin collapses (> 60 % at 1900 K, > 40 % at 1700 K) | 32 | 32 | −79 … −99 % at 1900 K, −64 … −89 % at 1700 K |
| H9 opening buys both back, crossing unlit inside | 16 | 16 | strictly rising at every step; crossing at `s` 1.08–1.24 |

**The named risks, measured:** H7's (fast flight, a small rise) was right — 11 km Mach 1.4 at 1700 K is the closest,
×0.880–0.902. H8's (`Tt4` 1300 conservative; the second map's stall line) — `Tt4` 1300 does drop more (83–99 % against
64–84 %), and the `pressure` map's larger margins still collapse past both bars. H9's (a turning operating point) did
not appear: the margin rises at every one of the 160 steps. **§ 3b's stated direction held:** at G4's own flight and
throttle (Mach 0.85, `Tt4` 1500) Part B's held throat — 3 % larger than G4's, against an unlit denominator — gives
×0.847 at 1700 K and ×0.775 at 1900 K, where G4's numbers interpolate to about ×0.75 and ×0.68: less severe, as
stated. (The scored cells' wider range, ×0.61–0.90, is their own flights and throttles, not comparable to G4's.)

**Cost (anchor § 4's trigger, 3× the dry point, best of 9).** The first build missed it: a lit point with the nozzle
held cost 6–12× the shipped dry point, even unlit 3.5×. Measured, not guessed (the census of trials and instants), and
re-planned before shipping: the duct's three bisections to Illinois false position (88 → 16 µs a duct), the burning
zone's Mach left unsolved on unlit trials, the unchoked turbine bracketed from the choked root instead of marched,
the turbine's lower bracket found by bisection past the band where low `π_t` chokes the duct's entry, the equilibrium
afterburner fuel's bisection to Illinois once bracketed (5.7× → 2.1× there), each instant's turbine bracketed near the
last instant's root within one search, and the speed search started at the design speed (the dry point) or the dry
speed (a lit one), the shipped search the fallback. **On the page's own reply, against the shipped dry reply, at the 16
scored H9 cells: held ×1.03–2.31, matched ×0.38–0.96.** Equilibrium gas at the design flight: held ×1.65, matched ×0.43.
Part A's replies moved by the solver change at most 3.2e−11 relative (entropy 1.9e−8 J/(kg·K)); its golden panel did
not move.

**Census (against Part A's commit `7de9419`):** check requests 0–334 byte-equal; Part A's 12 — the 8 lit ones moved
within the solvers' tolerances (above), the 4 refusals and the no-heat-loss one byte-equal. **A crash map of 1 200 random
Fly-it requests** (seed 112, the afterburner keys randomized, every gas and map): **no new panic** — all 16 panics with
the keys on reproduce, the same message, with them removed (the shipped speed search, the efficiency bookkeeping,
Fork B's burner balance at `Tt4` 2024 K: the core's own). With the afterburner fitted, the speed search's failures come
back as `explain_fly`'s words instead of a panic (the shipped plant panicked on 65 of the 1 200). Refusals, all in
words: too hot 190, too cold 33, a choking duct 10, the duct's entry choking 4, the speed search's own words 135, the
efficiency bookkeeping 4 (`census_b.txt`). The check grid gains 13 requests (`check_ab_fly_requests`); `browser.mjs`
step 11c flies it lit, matched, slammed and removed.

**Concessions (Part B):**
* "Dry" is the fitted engine unlit (its flame holders cost pressure), not the engine without an afterburner; the
  difference is the flame holders' pressure loss, 0.4–1.1 % of the thrust (the 32 H7 cells), and nothing in the core
  while choked.
* The unchoked branch is bracketed, not marched as the shipped one is; its root is unique, so the answer is the same to
  the solve's tolerance — and the scored grid never reaches it (the dry pre-flight).
* The slam (one shaft) refuses a fitted afterburner, lit or not, rather than flying the plain engine under a label that
  says otherwise; the afterburner lit in a slam is Part C's (Controls, two shafts).
* A lit request the afterburner refuses is named at THIS throttle and flight's dry point (its turbine exit is what the
  afterburner is asked to heat); the lit engine's own turbine exit differs, so the named ceiling is the dry one's.

## Part C — Controls (shipped 2026-10-19; the anchor's § 5 addendum, frozen as `9128307` before its code)

> **THE HEADLINE (Part C) — what the throttle holds decides the sign. On the two-shaft slam, where the throttle meters
> FUEL, a light-off behind a held nozzle gives MORE thrust than not lighting (+19 … +27 %) — Part B, holding `Tt4`, got
> less — because it buys it from the core: the LP shaft slows by 30–42 %, the main burner runs 163–190 K hotter on the
> same fuel and the HP shaft overspeeds 8–13 %; the LP compressor dips toward stall straight after light-off, then its
> flow margin (the page's) recovers past where it started while its pressure margin stays lower.** Opened by the rule at
> every moment, no shaft moves — bit for bit — and thrust rises ×1.48–1.54. And lit at a fixed exhaust temperature, more
> MAIN fuel at the same shaft speeds LOWERS thrust (less air through the turbine throats; the afterburner just burns
> less), so a governor holding thrust with it pushes the wrong way: refused in words (the user's decision).

> **CORRECTED BY RUNG 104 (2026-10-20, `docs/rung104-spec.md` § 5).** The held light-off's mechanism is rung 38's
> choked vane's (rung 104's `pin` setting). Under the vane's own flow law (the default) the back-pressure reaches the HP
> turbine too: the HP shaft SLOWS 3–6 % instead of overspeeding (C3 flips), the LP shaft slows 10–16 % (C2 still holds,
> nearer its floor), the burner still runs ~200 K hotter and thrust still rises (+22 … +27 %) — the sign the throttle
> decides survives. C12's three misses were the pin's: under every law the redline holds through a late held light-off.
> One row, two rows and the ellipse alike.

**Code** (Part C): `afterburner.rs` — `TwoShaftFit` (the page's settings), `TwoShaftAb` (captured on the core: `A6`, the
dry throat, the nozzle's own `π_n`; `w_at`), `LitAb` (`burn`, station 5 → 7 at the light-off fraction; `try_lp_turbine`,
the LP relation lit, its lean wall and refusals named; `s_now`), `lp_exit` (each thrust tail's exit), `AbAt` (the
readout), `LIGHT_RAMP`, `take_census_c`. `two_spool.rs` — `TwoSpoolEngine::with_afterburner`; the core swaps its throat and
loss when fitted (`A8/π_fh`, `π_n·π_fh`) and carries `afterburner`, `ab_w` (`light_at`, `lit_ab`), `ab_out`, `ab_last`.
`unchoked_lp.rs` — `try_lp_turbine` dispatches lit-and-held there. The four transient thrust tails (rungs 40, 62, 90, 91)
read `lp_exit`. Every march derivative sets the light-off from its own time (`fuel_transient.rs` ×3, `lagged_bleed.rs`,
`governor.rs`), as do the view's replays and re-reads. `sandbox_controls.rs` — `ControlsSettings` gains `ab`, `tt7`,
`light_at`, `a8_ratio`, `a8_match`; the precheck; `StopKind::Afterburner` and `afterburner_stopped` (the cause test);
the reply's `afterburner` block and `ab_w` / `ab_throat` / `ab_far` / `Tt7` columns; `check_ab_requests`.

**The device** (anchor § 5.1). Fitted and unlit, the core's nozzle throat opens by `1/π_fh` and the flame holders join the
nozzle's loss, so every shipped path — the tails, the steady matchers, the tables, the formula — runs the fitted engine
unchanged. Lit, the LP turbine reads station 7 one throat downstream of `A45`: choked `π_LPT·π_AB·√(Tt45/Tt7) =
A45·MFP45·(1+f7)/((1+f45)·A8·π_n·MFP7)`, else `ṁ7 = A8·ρ9·V9` at `p9 = p0`. Light-off at a chosen time on the march's
0.02 grid ramps over 0.10 τ (five steps): `Tt7(w) = Tt5 + w·(Tt7 − Tt5)`, the throat `A8_dry·(1 + w·(s − 1))`; gas
arriving hotter than `Tt7(w)` is not heated. *Match* keeps the LP turbine where the fitted engine has it unlit and books
the lit exit (Mattingly's law in time — an ideal nozzle scheduler). The lean wall: with the throat held, below some main
fuel the gas cannot leave the throat even with the LP turbine nearly unexpanded — no running point (the model keeps both
turbine throats choked; a real engine would stall), answered at once without searching below it.

**Gates (`tests/rung102.rs`, `part_c_*`; anchor § 5.2):** not fitted is the shipped reply byte for byte, whatever the keys
say; fitted-unlit = the probe's hooked relation to the last digit (both slams; `unlit_dump`), and = not fitted where the
nozzle stays choked (the default slam: every point equal); *match* ⇒ the march is the fitted-unlit one bit for bit (`ν`,
`Tt4`; all 24 scored cells too); the held lit LP root agrees with the effective-throat fixed point through the SHIPPED pin
to 1e−8; a re-read sets the light-off from its point's time (every recorded point bit for bit); DS vs DS/2 under 1e−6
(off the grid it was 1.5e−4 — the probe, P7); the refusals and the afterburner's stops in words; the reply's keys.

**Scored, Part C — 335 HIT, 25 MISS of 360** (`score_c.txt`; the grid unseen: three map shapes, `ρ` 0.5 / 3, the design
flight and 5 km Mach 0.6, 900 → 1350 K, `Tt7` 1900, light-off 0.30 and 3.50; ×1, ×1.2, *match*):

| claim | cells | HIT | measured |
|---|---|---|---|
| C1 *match*: thrust × more than √(Tt7/Tt5) | 24 | 24 | ×1.481–1.537 against 1.347 |
| C2 held: the LP slows, by 8–50 % | 48 | 48 | −29.7 … −42.1 % |
| C3 held: the HP overspeeds 2–20 % | 24 | 24 | +8.3 … +12.6 % |
| C4 held: the main burner hotter | 24 | 24 | +163 … +190 K |
| C5 held: thrust above unlit | 24 | 24 | +19.3 … +27.1 % |
| C6 held: the LP's pressure margin dips below unlit's after light-off | 24 | 10 | MISS ×14 (below) |
| C7 held, end: LP flow margin above unlit, pressure margin below | 48 | 40 | flow 24/24; pressure MISS ×8 (`flat-lp`) |
| C8 held, end: both HP margins above unlit | 48 | 48 | |
| C9 ×1.2 strictly between ×1 and *match* | 72 | 72 | |
| C10 light-off below choke (650/700 K, Mach 0.5) | 15 | 15 | the held lit nozzle chokes within two steps — the unchoked lit branch ran only there, and no gate checks it independently (the instrument reads choked instants) |
| C11 the HP-speed governor holds HP speed, the LP slows | 6 | 6 | HP within 0.26 %; LP −70 … −89 % |
| C12 the redline through a late held light-off | 3 | 0 | MISS ×3: the lean wall (below) |

**The misses, and the named risks.** C6's 14: six are mid-slam light-offs where the slam's OWN minimum lands exactly at
the light-off (0.30 is still inside the ramp's dip), so nothing after it is lower — the named risk, word for word; the
other eight are the `flat-lp` map, where the LP speed lines are flat, so the pressure ratio at the stall line equals the
running one and that margin is identically 0 (the coordinate cannot read the map; rung 53). C7's 8 are the same flat map.
C12's 3: a late held light-off with the redline on stops all three runs — the redline cuts the main fuel toward the lean
wall as the LP slows (named: "the lit afterburner stopped the run"); the named risk was exactly this. C2's risk (`ρ` 3
late, the least settled) held (−29.7 % is its smallest); C1's (the slow flight) held with margin; C3's (`ρ` 0.5 mid-slam)
held.

**The headline's own check (run after scoring, at the final review; reported, not scored — `tt4hold.txt`).** Part B
and Part C differ in TWO things, the throttle and the shaft count, so "the throttle decides the sign" needed a run
that changes only the throttle: the same two-shaft Controls engine, governed to HOLD `Tt4` all along the move, lit at
0.5 behind a held nozzle. Lit thrust is BELOW unlit at all three slams tried (×0.871, ×0.885, ×0.893 — the default
slam, settle 6, and `tilted` 900 → 1350), crosses above at ×1.15 (×1.110–1.130), and ×1.549–1.593 by the rule — Part B's
picture on two shafts. With `Tt4` held the LP shaft slows much further (to 29–40 % of its speed against 60–62 % metered):
the core has no extra heat to give. So on one engine the throttle alone flips the sign.

**Cost** (§ 5.4: 3× the same request fitted-unlit, the page's reply, best of 9). Default slam held ×1.42, ×1.15 ×1.56,
*match* ×1.04; with the redline / schedule / floor / release on ×0.37–1.54 (a held run that stops is cheap); settle 6
×1.44. **The trigger fired once: the HP-speed governor with the nozzle held, ×3.76** (378 ms against 101) — and it
shipped above the bar, a DEVIATION from § 5.4's frozen rule ("sped up before shipping"), not its outcome. The census says
why, and it is not the LP relation: the governor rides the lean wall for the rest of the run, evaluating 3.3× the
instants (20 181 against 6 089), each 1.28× dearer; the planned fix (Part B's bracket) cannot reach a cost made of more
instants. Two deterministic changes were measured: a bracket from the unlit root (44 bisections a solve — slower, removed)
and the top-first lean-wall answer (kept: 449 → 378 ms). A history-dependent bracket was not used: it would break the
re-read's bit-for-bit match.

**Census (against `9128307`):** check requests 0–359 byte-equal except `controls_defaults` (it now carries the five keys
the page needs); a crash map of 1 200 random Controls requests (seed 102; shapes, `ρ`, throttles, ramp, settle, Mach,
redline, schedule, floor, release, levers, the governor; each run not fitted, fitted unlit and lit with a random `Tt7`,
light-off and throat): **not fitted, all 1 200 byte-equal to the base; no panic in any of the 3 600.** Lit against unlit:
1 066 the same outcome; 66 lit runs stop where the unlit one ran and 7 stop earlier than it — all 73 classed
`afterburner`; 59 refused in words ("hold thrust" lit). The check grid gains 9 requests (`check_ab_requests`, last);
`browser.mjs` step 16b-ab runs it lit, shows its knobs, chart and row, refuses "holds thrust" and removes it.

**Concessions (Part C):**
* The light-off is a fixed 0.10 τ ramp of temperature and throat together — no actuator lag of its own; *match* is an
  ideal scheduler that sizes the throat afresh at every instant.
* The lean wall is a model limit, not a stall prediction: with both turbine throats choked the gas has nowhere to go, so
  there is no running point; a real engine would back the gas up and stall, which the model cannot show.
* Fitted and unlit, the duct's entry Mach is not checked (Part B's unlit point does): the swapped core never computes it.
* The steady endpoints (and every table) are the fitted-unlit engine's: a run lights only during the march.
* `two_spool.rs:936`'s assert is not converted: the census never reached it (an unreachable assert made an `Abort`
  measures nothing); the anchor's § 1 line is retracted (§ 5.0 P9).
