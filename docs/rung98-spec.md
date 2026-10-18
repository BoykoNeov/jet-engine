# Rung 98 — THE CORRECTED TABLES: the control's three tables made once at the design flight, read elsewhere through the air's pressure and temperature

**Code:** `rust/src/corrected_tables.rs` — `TableAt`, `Carry` (the flight's `δ`, `θ`, the row translation, the three
carries, the gap's two parts), the design-flight builders `try_design_table` / `try_design_steady_table` /
`try_design_accel`, the words `row_words`. `sandbox_controls.rs` gains the settings `table_at`, `corr_p`, `corr_t`,
routes rungs 48/95/96's table builds through the carry, and calibrates an inferred thrust's ends on the carried table.
**Gates:** `rust/tests/rung98.rs`; `rust/tests/sandbox_controls.rs`'s round trip takes the three settings.
**Panel:** `print_corrected_tables_table` (born in Rust, held to `rust/oracle/rust_owned/`; a change detector only).
**Anchor:** `docs/plans/rung98-anchor-corrected-tables.md`, committed as `897001b` before any rung-98 code existed.
**Sandbox:** the Controls view's Flight panel, the results table's two rows (`docs/plans/sandbox-plan.md` § 30).
**Scratch scripts and outputs:** `docs/plans/scratch-archive/rung98/`.

Rung 96's seam: *"the table made at one flight condition and flown at another, its readings corrected by `δ`/`θ` as a
real schedule's are."* Until now rungs 48, 95 and 96 built their tables at the flight flown, so a correction had nothing
to do. The user chose all three tables, two switches (pressure, temperature), rows over the run's own range translated
to the design flight, and — asked before the anchor froze — **the design flight as the default**, with "tables made at
this flight" kept as the setting that reproduces rungs 96/97.

> **THE HEADLINE — a correction can only undo what changes the engine's inside; flight speed changes only the thrust.**
> Pressure correction is EXACT on this model (≤ 3.6e−11) and the temperature one nearly so (the pressure-ratio control
> misses by under 0.1 %, the fan speed by under 0.3 %). But at fixed inlet air a Mach change leaves the core identical
> to the bit while the jet's push against the outside air and the ram drag move: a table made at Mach 0.85 gives
> **8.4–9.1 % too much thrust at Mach 0.5 and 4.6–9.7 % too little at Mach 1.4**, with both corrections on, every map,
> both readings. That is the textbook reason a real schedule carries a Mach axis.
>
> Each correction is needed where it is: without the pressure one the thrust is off by the pressure ratio itself
> (the pressure-ratio control −47 % in air at 0.53 of the design's pressure, +60 % at 1.6). Without the temperature one the fan-speed control misses
> by 22–43 % on a day 16 % warmer or 13 % colder at the engine — while the pressure-ratio control has nothing for it to
> correct (`EPR` and thrust over `δ` contain no temperature: identical runs, bit for bit), and the acceleration schedule
> has nothing for the PRESSURE switch to correct (fuel over compressor pressure is already a ratio).

## The device (anchor § 1)

* **Rows.** Each table is built at the design flight on the run's own row throttles × `Tt2_d/Tt2` (the
  corrected-equivalent throttle), whatever the switches. Its end rows are NOT the run's ends.
* **The carry** to the flight flown, with `δ = pt2/pt2_d`, `θ = Tt2/Tt2_d` (inlet totals, each switchable): thrust table
  thrust `×δ`, `EPR` as is, LP speed `×√θ`, fuel `×δ√θ`, face airflow `×δ/√θ`; HP-speed table speed `×√θ`, fuel `×δ√θ`;
  acceleration schedule `Wf/pt3 ×√θ` (its abscissa is already the model's corrected HP speed). Every downstream read is
  rungs 48/95/96's, unchanged.
* **Calibrated ends.** An inferred thrust's ends land where this engine's steady reading equals the carried table's
  reading at rung 96's target; the end's thrust minus the target is the **steady miss** the page shows. The proportional
  laws are not calibrated; they settle where their own law balances.
* **Reach.** A row with no steady point at the design flight is refused in words (*"a table made at the design flight
  cannot cover this flight. Make the tables at this flight instead."*): the translated throttle below about 500 K —
  roughly Mach 2.5 and above for the default move, lower for low throttles.
* **Reduce.** Tables made at this flight, a run flown AT the design flight, or a run that reads no table: the old code,
  bit for bit, by dispatch.

## Gates (anchor § 2) — measured margins (`margins.txt`, all four maps, seven variants)

| Gate | Bar | Measured |
|---|---|---|
| Tables at this flight / flown at the design flight / no table read ⇒ old code | bit for bit | bit for bit |
| Temperature switch on `EPR`; pressure switch on the schedule | bit for bit | bit for bit |
| Pressure-only flight: carried design tables vs tables built AT the flight by their own solves | 1e−9 | 3.6e−12 |
| Pressure-only flight: the governed `EPR` move vs the same with tables made there | 1e−7 | 9.3e−13 |
| The gap's NOZZLE part constant through a run (hot, Mach 1.4; choked points) | 1e−12 × thrust | 2.1e−13 (cooled) |
| Pressure correction off: thin air under, dense over, both readings, every map, both ends | sign | 32 of 32 |

**Red check:** a 0.1 % error put into the carried thrust column failed all three non-bitwise gates; restored.

## Scored (anchor § 3) — 68 HIT, 4 MISS of 72 (`score.txt`)

* **H1 — fan speed, temperature uncorrected: cold over, hot under. 16 / 16 HIT.** Cold +22…+43 %, hot −24…−37 % (the
  flat-LP map worst).
* **H2 — Mach, both corrections on: faster under, slower over. 32 / 32 HIT.** Mach 0.5: +8.4…+9.1 %. Mach 1.4: −4.6…−5.4 %
  at the 1000 K end, −9.4…−9.7 % at 1400 K. The named risk (the low throttle nearly balanced) showed as a miss about half
  the size at 1000 K, but no sign flipped.
* **H3 — the schedule, temperature uncorrected: cooler and slower in hot air, hotter and faster in cold. 16 / 16 HIT.**
  Hot: peak `Tt4` 50–64 K lower, and the thrust never reaches 95 % of its change in the 2.5 τ run (corrected: by
  2.2–2.3 τ). Cold: peak 57–66 K higher, 95 % reached 0.18 τ sooner. The leg held the fuel in all 16 runs.
* **H4 — the proportional HP-speed law, temperature uncorrected: predicted to settle FASTER in hot air, slower in cold.
  4 HIT (cold), 4 MISS (hot).** In hot air it settles SLOWER too (0.9441 against 0.9472 on flow/press). **The finding**
  (`h4why.txt`): the rows lie where the CORRECTED reading falls (0.785–0.946 in hot air), so the uncorrected reading
  (0.786–0.948 physical, against a table of 0.729–0.879 uncarried) runs past the top row at 111 of 126 points, where rung
  95's steady table HOLDS its end value — too little fuel, not too much. The derivation's own assumption held (steady
  fuel ∝ HP speed^4.5–4.8); what it missed is that a table read outside its rows is a clamp. In cold air the reading
  stays inside the table (9 of 126 beyond), so the cold HITs are the derivation's mechanism, not the clamp's. The
  CORRECTED baseline clamps too, barely: the hot engine's corrected state sits slightly above the design's (G2), so its
  demand ends just past the carried top row (101 of 126 points, by under 0.2 % in speed) and reads a fuel close to right.
  Rung 48's schedule does not: in every H3 run it held the fuel only inside its rows (`capcheck.txt`).

## Reported, not scored

* **The temperature residual, both corrections on** (both ends): `EPR` +0.08 % hot, −0.07 % cold; fan speed
  +0.06…+0.25 % hot, −0.04…−0.28 % cold. The fuel's fixed heating value (anchor G2).
* **The proportional law's droop on the carried table**, both on: −0.03…−0.04 % HP speed hot, +0.006 % cold — the hot
  figure partly the top-row clamp above.
* **Fixed amounts** (pressure similarity broken, anchor G1), `EPR`, ends start / final: a generator by POWER +1.1 % /
  +0.3 % thin, −0.5 % / −0.1 % dense; customer air by AMOUNT −4.6 % / −1.4 % thin, +1.2 % / +0.5 % dense.
* **The thrust gap through the move** (`F_true − F̂` over the thrust change, ramp points; design-flight tables against
  tables made there): hot and cold, the same within 0.4 points (`EPR` +16 / +3 hot, +9 / +2 cold). Mach 0.5: `EPR`
  +20 / +7 against +7 / +1; Mach 1.4: +10 / −1.4 against +19 / +4.5 — the steady Mach miss rides on rung 96's dynamic gap.
  The page splits the `EPR` gap into the air-swallowed part and the constant jet-pressure part (anchor § 1).

## Crash map, compared request by request (`crash_compare.txt`)

1 500 random Controls requests (seed 98: rung 97 § 29's map plus the tables' place and switches at random), through this
code and through a worktree of `897001b`, which ignores the new keys. **No panic on either.** 122 outcomes differ —
**every one** reads a table, flies off the design flight, and has its tables made there (0 outside that class): 59 run
with different numbers; 53 now end in the refusal that the design-flight table cannot reach (47 at Mach 1.5 and above,
the rest low throttles in hot or fast air; 5 were refused before, by the run's own schedule rows or an end); 4 that ran
now have a calibrated end out of the 600–2000 K search; 6 were rung 96's calibrated-end refusals and are the same
refusal on the design-flight table. **The final review caught a deviation here:** the first build, with the clean table
and a generator, ran rung 94's thrust-hold root to find the ends' targets — which rung 96 never runs there — so 2 runs
that had run were refused and 6 refusals used the wrong words (thrust-hold words for a control holding a reading).
Fixed (the targets come from the generator-off engine at the set throttle, as anchor § 1 says; gated, and crash
request #26 runs again); the counts here are after the fix. The four pre-existing odd stops are the same requests in both. The Controls check requests: 1 of 77 moved (rung 97's +40 K
`EPR` run from 700 K), now pinned to tables made at this flight; six rung-98 requests added.

**Cost** (`cost.txt`, best of 9): the governed `EPR` run 0.155 → 0.194 s (the calibrated ends' root solves); the
schedule and the proportional law unchanged.

## What it does to its neighbours

* **Rung 96 — its concession DISCHARGED.** "The table is made at the run's own flight condition" is now a setting, and
  the default. Its headline survives a design-flight table: the dynamic gap through the move is unchanged within 0.4
  points at another temperature; at another Mach the steady Mach miss adds to it.
* **Rung 48 — the ratio unit explained.** `Wf/pt3` against corrected HP speed needs no pressure correction, by
  construction; uncorrected for temperature it is a different limiter (H3: ~60 K on the peak).
* **Rung 95 — a table read outside its rows is a clamp** (H4's MISS). Its steady table holds its end values; read
  uncorrected it runs off them.

## Concessions

* The exponent is the textbook ½ on `θ`; the residual it leaves is reported, not fitted away.
* One reference station (the inlet totals) for every table, including the HP-speed one; in exact similarity every
  station's ratio is the same.
* No Mach axis: a table carries one flight speed. Rows are the run's own range, not a fixed schedule.
* The fixed-amount loads break pressure similarity; they are run, not corrected.
* Inherited: invented maps, rung 36's imposed stall line, the perfect gas, rung 95's lean-wall roughness.

## The next seam

**A Mach axis**: a table made at several flight speeds and read at the one flown — the correction rung 98 shows the
`δ`/`θ` pair cannot make. Or the steady matchers' unchoked branch (rung 97's seam), or the governor on the lagged
marchers (rung 95's).
