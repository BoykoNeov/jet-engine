# Rung 103 — THE STEADY TWO-SHAFT MATCHERS BELOW CHOKE: the refusal becomes a solve

**Code:** `rust/src/steady_unchoked.rs` — `SteadyLp` (`Pin` / `Free` / `Held`), `try_lp_steady` (the LP turbine in every
steady cascade), `track_pt4`, `refuse_unchoked` (each guard's three outcomes), `dispatch` (the pinned body, then the
re-solve), `rung38_below_choke`, `below_idle`. `two_spool.rs`'s `TwoSpoolCore` gains `below_choke` (the switch),
`steady_lp`, `dispatching` and `pin_past_choke` (the old model, scoring only). The six matchers' bodies sit behind
`dispatch`: `r39_body`, `bleed_body`, `customer_body`, `cooled_body`, `loaded_body`, and rung 38's guard. The pages:
`blade_speed::Machine::below_choke` (carried into every stator sibling by `VariableStatorCore::at_setting`),
`metal_temperature::FixedEngine::below_choke`, the Controls view's generator check. **Gates:** `rust/tests/rung103.rs`;
two rewritten page tests (§ 4). **Anchor:** `docs/plans/rung103-anchor-steady-unchoked.md`, committed as `8bf63b9` before
any rung-103 code. **Scratch:** `docs/plans/scratch-archive/rung103/`.

CLAUDE.md's open seam *"The unchoked LP branch in the STEADY two-spool matchers (38 flags; the transient BUILT BY 97)"*,
rung 97's next seam. The user's scope (memory `rung103-decisions`): **off in the old code, on in the pages** (every
shipped path, oracle, golden and older rung keeps rung 38's refusal); **all six** steady matchers (38, 39, 42, 90, 91,
92); **below idle refused in words**.

> **THE HEADLINE — below choke the HP spool hears the LP compressor.** Rung 39's finding A is a one-way street: the LP
> compressor's efficiency cannot reach the HP pressure ratio — through the full match, exactly zero on the perfect gas.
> Below choke the LP turbine's expansion reads the pressure the nozzle pushes with, and that pressure carries the LP
> compressor's ratio. A better LP compressor raises it, the LP turbine expands further, the LP shaft takes more work, the
> HP compressor's inlet runs hotter — and with the same work from its own turbine the HP compressor makes a LOWER
> pressure ratio: −2e−4 … −2.8e−3 for one point of LP efficiency, against a pin value of 0 to 3e−11. **Scored 37 of 38**;
> the miss sits 2e−4 below the choke boundary, where the arrow vanishes (§ 3).
>
> And on all six matchers the LP shaft runs slower than the pin said — **174 of 174** — by up to 69 % deep below choke.

## 1. What it adds

**The equation** is rung 97's, on the steady matchers: below the nozzle's unchoke boundary only `A45` stays choked, so
`π_LPT` is the root of rung 33's `(★★)` one throat downstream, at `pt45 = π_HPT·pt4` and the flight's `p0`. The steady
matchers close `pt4` in their joint `(f, pt4)` loop, so each pass hands the LP turbine the `pt4` it starts with and the
root converges with the loop (anchor G2: one root in every residual scan; the loop never caps).

**The dispatch.** Each matcher runs its pinned solve and rebuild unchanged. If that fails — at the unchoked guard or
anywhere earlier — and the switch is on, it re-solves once with the LP turbine free. A point whose pinned path succeeds
never enters the branch, so every choked point is the switch-off point to the bit (R2). A body that forwards to
another matcher's (rung 42 with the valve shut, rung 90 with no customer air, …) does not retry inside the outer one's
attempt. Rung 38's matcher is infallible: it re-solves at its guard only, and its branch's failures panic in their own
words.

**Below idle.** Inside the branch, a point whose core specific thrust or net thrust is not positive is refused:
*"rung-103: below idle at Tt4=…: with the nozzle unchoked the engine makes more drag than thrust here"* — before the
shared scoring, whose efficiency check cannot take a negative thrust.

**Retrying on any pinned failure** (the advisor's catch, measured in anchor G3): ten probe points failed BEFORE the guard
— the pin over-expanding the LP turbine left the nozzle's gas below outside pressure, the very artifact the branch
corrects. On a retry all ten become "the nozzle cannot pass the flow" and 35 generator refusals quote the branch's share:
**the retry adds no running point on the probe grid; it corrects the reason.** On the Blades page the 805 K "nozzle
back-pressure" refusal is now the honest "below idle" (§ 4).

## 2. Gates (anchor § 2, bars frozen at `8bf63b9`) — all pass

| gate | what | measured | bar |
|---|---|---|---|
| R1 | the switch off by default on every matcher; then the whole gate passes unchanged | — | — |
| R2 | switch on at a choked point = off, to the bit (thrust, both speeds and flow coefficients, both ratios) | every choked cell of the H2 grid + 850–1300 K, six variants | bits |
| R3 | an independent root: the LP turbine HELD at a trial `π` (never calling rung 97's helper), the variant's own body to its rebuild, an outer bisection on the nozzle's mass balance read off the result | 5.6e−13 (23–32 unchoked cells per variant); choked: root = pin ≤ 1e−12 | 1e−11 |
| R4 | rung 40's equilibrium (rung 97's branch), plain map matcher | 2.5e−10 over 60 unchoked cells; its choked floor 2.7e−12 | 1e−8 |
| R5 | a hand-written perfect-gas solve of the whole point (rung 38's matcher; rung 39's on flat maps) | 3.6e−14 | 1e−9 |
| R6 | just below the boundary the branch meets the pin (`Tt4` bisected onto it) — **a continuity check only, not evidence the branch is right** (blind to the red check below) | 3.5e−14, 12 boundaries | 1e−9 |
| R7 | mass at the nozzle: `A8·ρ9·V9` against the station-5 flow | 2.2e−13, 61 points | 1e−11 |
| R8 | no re-solve state left after any call — success, refusal, rung 38's panic | clean | — |

**R4 runs on the probe maps** (flow/press, press/flow, tilted): on `steep` and `flat-eta` rung 40's equilibrium does not
converge at any unchoked cell (anchor G2 saw it fail even at choked cells on `flat-lp`). That is rung 40's Newton, not
this rung; it is why R3 and R5 exist.

**The red check** (the branch's LP turbine forced back to the pin, a scratch edit): R3 (no root), R4 (7e−1), R5 and R7
(2.5e−1) fail, and H1/H2 with them. **R6 passes at exactly 0 — blind to that edit by construction**: it compares the branch
to the pin, which the edit makes identical (rung 97's lesson: a boundary gate can be tautological). It still checks the
branch's own continuity.

## 3. Scored (anchor § 3)

| claim | cells | result |
|---|---|---|
| **H1** below choke `Δπ_HPC < 0` for `η_LPC` 0.90 → 0.91, `|Δ| ≥ 1e−6` and ≥ 100× the same cell's pin | 38: rung 39's engine, perfect and equilibrium gas, maps flat / mixed / steep, two flights | **37 HIT, 1 MISS** |
| **H2** the LP shaft slower than the pin | 174: the Controls engine, six variants, maps steep / flat-eta, three flights | **174 HIT** |

**H1's miss** — equilibrium gas, steep maps, 250 K / 50 kPa / M0.85, 650 K: `Δπ_HPC` −2.7e−8, sign right and 18 000× the
pin's −1.5e−12, under the 1e−6 floor. That cell's nozzle exit Mach number is **0.9998**: it sits 2e−4 below the boundary.
The derivation did not foresee that **the arrow vanishes AT the boundary** — a just-sonic nozzle's flow function is flat
in its pressure ratio (`dMFP/dM = 0` at `M = 1`), so there the root barely reads `pt45/p0`, and the LP compressor reaches
the HP spool through a slope that starts at zero. The measured `|Δπ_HPC|` peaks near `M9` 0.65–0.75 and falls from
there toward choke on every flight (e.g. CPG mixed at sea level: −2.6e−3 at `M9` 0.62, −2.8e−3 at 0.71, −1.5e−3 at
0.89; CPG flat at 250 K: −8.1e−4 at 0.81, −2.4e−6 at 0.998 — the next-closest cell, which cleared the floor). The bar
stands; the miss is the finding's edge.

**Rung 39's gate 4 stays exactly true**: it holds `pt4` and calls the cascade directly, so it never sees the branch. The
new arrow exists only through the full match.

**Reported, not scored** (H2's 174 cells, branch against pin; `score_report.txt`):
* the LP shaft −0.0 … −69 % (largest deep below choke with the HP generator);
* the HP shaft slower at 155 of 174, by at most 0.7 % — 3.8 % with the HP generator; faster by ≤ 0.012 % at the rest;
* the LP compressor's flow coefficient HIGHER (further from its stall line) at **174 of 174**, the HP's lower at 161 — so
  below choke the LP spool is safer and the HP spool nearer stall than the pin said (not derived, so not scored;
  anchor G3's four probe exceptions, all with the HP generator, did not recur here);
* **thrust MORE at only 50 of 174** — the difference −12.5 … +16 N per kg/s of design air. This **BOUNDS rung 97's "more
  thrust"** (+3.6–4.8 % at its +40 K cells) to its own cells: the faster jet against the slower LP shaft's smaller
  airflow goes either way, and at most of these cells the airflow wins.

## 4. The pages

The Blades view (its plants), the Cooling view (its fixed engine) and the Controls view's generator check turn the
switch on; the Controls view's holds already run on rung 40's equilibrium, which has carried rung 97's branch since then.
Each newly reachable refusal has plain words, driven by a test: below idle; the nozzle cannot pass the flow.

* **Blades:** 600 K on the defaults — refused before — now reads its lever. The LP vanes then turn 80.6°, but that is the
  stator schedule's own trend on this engine (66.6° at 1000 K, choked, unchanged code), not this rung.
  `a_lever_point_where_the_nozzle_unchokes_is_refused_before_the_schedule` becomes
  `…_is_re_solved_by_the_view` (the library default still refuses; the page reads the switch-on schedule bit for bit).
  `the_low_throttle_refusals_are_named_for_the_throttle`'s 805 K "back-pressure" case is now "below idle" — the class the
  anchor's G3 found (a pinned failure before the guard) and did not list among the expected rewrites.
* **Cooling:** `heavy_cooling_starts_the_knobs_search_where_the_engine_solves` passed unchanged — vacuously for the
  page's side (it compared against a switch-off engine, and its refusal branch accepted the new answer). Rewritten: the
  library default still refuses at the search's floor; the view's engine solves there; the reply equals the search from
  the floor on the view's engine. **A pre-existing panic, fixed:** a sweep point whose metal holds with NO cooling air
  (`cooling_point` at 900 K on the defaults) panicked on `booking.expect` — the inactive matcher forwards to the uncooled
  body, which keeps no booking. Rung 103's re-solve made 700–850 K reach it too. It now reads the bare engine, as the
  knob form already did (`a_point_needing_no_cooling_reads_the_bare_engine`).
* **Controls:** the generator check's steady answer is the branch's below choke — **rung 97's concession discharged**.
* **The browser check** (`browser.mjs` step 14): its design (`π` 4/10, 1200 K) had two "Not modelled: … nozzle unchokes"
  rows at 720 and 780 K; they are now lever rows short of the target, and the check asserts no row says "Not modelled".

**The crash map** (anchor § 5; `score_crash.rs`, seed 103 — `score_crash_summary.txt`): 600 Blades-lever requests (random design pressure ratios,
design `Tt4`, shape, lever, spool, 20 % on the thermally perfect gas; the throttle 35–75 % of the design's) and 600
Cooling requests (random shares, shape, technology, year; a point at 550–1300 K or the knob search), replied through
the page's own `call` on this rung and on `8bf63b9` (a worktree), request by request. **No panic on this rung; 226 on
the code before it — every one the Cooling view's `booking.expect`** (the pre-existing panic above, 19 % of these
low-throttle requests). Of the rest: 648 byte-equal; 139 refusals now run; 76 refusals change words (73 to below idle,
3 to "cannot pass"); 32 unchanged refusals (the gas tables); 79 replies run on both and differ — 13 Blades levers whose
stator scan now passes settings where it used to stop at the refusal (`"reached":0` rows), and 66 Cooling knob answers
whose search starts lower and so lands on another point of its own bisection (e.g. 2270.522003071528 against
2270.52200307171 K).

**Cost** (`score_cost.txt`, best of 9, the page's own `call`): Blades lever ×3.94 (600 K) and ×2.56 (625 K) against 650 K;
Cooling point ×3.05; the heavy-cooling knob ×0.31 (it stops sooner). **The trigger fired once: the generator check's
steady call, ×5.01** (1.95 ms against 0.39 ms). Counted before changing anything (`score_count.txt`): 8 742 turbine
evaluations against 1 880 — the pinned attempt that ends in the refusal (2 068, thrown away; it is what keeps every
choked point bit for bit) and the branch (6 674, ~44 bisection steps a pass in rung 97's helper). Disclosed, not
optimised: 2 ms, run only when an endpoint already failed with the generator on.

## 5. What moves if the switch is turned on everywhere (reported, not re-scored)

The whole gate run in a scratch worktree with the switch's default flipped to ON (`gate_on_summary.txt`): 2 885 passed, 18
failed in 12 programs. What each failure says:

* **The scope gates that asserted the refusal** — rung 38's `gate4_nozzle_unchoke_is_out_of_scope`, rung 39's
  `gate9_scope_guard_unchoke_raises` — and rung 103's own R1/H2 (which read the default): by construction.
* **The Python-held oracles stop at the first new refusal**: `two_spool_oracle` and `slice_l_oracle` meet rung 97's
  "the nozzle cannot pass" where the pin's "UNCHOKED" stood — an abort class they do not know; past it, every cell the
  pin refused would carry a number Python never computed. `slice_n_oracle`: 4 of 72 520 keys — rung 55's schedule
  census (+6 045 marches, +156 speed-line solves, +385 clamps) — the schedule now reads points it skipped.
  `cli_golden`: rung 38's panel line *"600 nozzle UNCHOKES here -- OUT OF SCOPE (flagged, not solved…)"* changes.
  **This is why the switch is off in the old code** (the user's decision).
* **Rung 41 — the LP spool's flow-coefficient TURN** (`p5_…`): on flat maps the LP face was a RAIL — its flow
  coefficient fell all the way to the end of the choked envelope, the scan ending at the refusal. Solved below choke it
  is a MIN: the flow coefficient turns back up. It is § 3's reported finding (the LP compressor further from stall than
  the pin said) seen through rung 41's instrument — **the LP rail was the envelope's edge, not the compressor's.**
  (Rung 41's spec measures the HP turn; the LP's RAIL is that gate's control.)
* **Rung 90's A8** — *"650 K must be refused in the amount mode"*: it now solves. The amount mode still bites harder at
  low power (the rest of A8 holds); the envelope it "shrinks" was the choke's.
* **Rung 42's gate 6b — the valve shrinks the envelope — SURVIVES**: with the branch on the sweep ends at the new
  refusals, and opening the valve still ends it at a higher throttle.
* **Rung 94** (714 of the census's refusals, in its hold searches): every gate passes — its scored holds sit on choked
  points; only the searches' refused trials change.
* The two page tests (§ 4) and the page-currency check, as expected.

## Concessions

* **`A45` stays assumed choked** (rung 38's, carried by rung 97): at the root the LP turbine expands by 1.0–1.45 overall.
* **The switch, not the physics, decides** which answer an older rung's code gets: everything shipped before this rung
  still reads the pin's refusal — the user's decision, so that no Python-held oracle and no older verdict moves.
* The cost of a below-choke steady point includes the thrown-away pinned attempt (§ 4).
* Inherited: invented maps, one `η_m`, rung 40's equilibrium unable to converge on some maps.

## The next seam

**A choked-or-not `A45`** — with the LP turbine expanding under 1.5, its nozzle guide vane is the next throat whose choke
is assumed, not earned (rung 97's, still). And **the arrow's size near the boundary**: H1's slope starts at zero at choke,
so a criterion that reads the HP spool's sensitivity to the LP compressor (a margin, a schedule) is blind to it exactly
where the nozzle first unchokes.

**Rung 40's equilibrium refused running engines** (found in this rung's probe) — **REPAIRED 2026-10-19** (sandbox plan
§ 36, `tests/hp_wall_repair.rs`). On `flat-lp` it failed to bracket ("two-shaft closure does not bracket") where the
steady matcher finds one clean root with positive thrust (`probe1_census.txt`). The cause was not the Newton's start —
seeded AT the steady root it still failed — but the closure's search: with no LP wall on a flat map it ran to the bare
2.5, past the HP map's own wall, where the residual turns back negative. Capping it at the HP wall, only where the old
bracket failed, solves every such point at the steady matcher's speeds (≤ 6e-13). The one cell left (550 K, standard
day, M 0.85, barely above idle) whose full-speed START has no closure root — **REPAIRED 2026-10-19** too (sandbox plan
§ 37, `tests/low_start_repair.rs`): where full speed refuses, the Newton re-starts lower and keeps the root only if it
runs. Rung 97 § 3's "no steady point" is corrected in place. **Not one rule yet** (the user's choice, § 37): the
full-speed start still returns choked below-idle points, as this rung's matchers do.
