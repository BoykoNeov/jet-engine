
#### 5.33.7 STEP 7 — THE DISPATCH GATES, AND **THREE KINDS OF SILENCE: RUNG 79's THREE TABLE SWAPS ARE ONE DELETION TO EVERY SEAT, AND RUNG 80's `at_lever` IS VISIBLE TO POINTER IDENTITY AND TO NOTHING ELSE — IN EITHER LANGUAGE**

**SHIPPED**: `rust/tests/slice_ai_dispatch.rs`, **9 gates**, green in 32 s. Six swaps, each pointed
back at the parent it was re-aimed FROM — rung 79's `at_lever` (→ 78's), `shared_rig`, `cap_fuel`,
`with_coord` (→ 78's cell, rung 74's body); rung 80's `at_lever` and `shared_rig` (→ 79's) — plus a
`MacroNone` row per rung (this file's rebuild helper, which must read `same`) and a `Count` row per
rung wrapping every re-aimed cell at once. **Seats**: rung 79's four readers and rung 74's
`demand_gains` at § (i)'s settings (wall 0.80, `every = 4`) on every row; rung 80's rows add its
four readers. Predictions in `W:\temp\claude\slice-ai-step7\predictions.md`, written before the run.

##### (a) THE DESIGN, AND THE THREE CHANGES FROM AH's TEMPLATE (the advisor's list)

* **A cell is a reading, four call counts AND rung 79's six counters** (reset before each seat,
  read after). § (i)'s hazard is value-invisible by construction, so a verdict on values alone
  would print this slice's headline row as `same` and mean nothing. The purity proof for the
  `MacroNone` and `Count` rows compares the counter vectors as well as the readings.
* **Rows run on parallel spawned threads, with NO lock** — sound because everything the file
  reads is thread-local (rung 79's counters since step 1, this file's call counters, the quiet
  flag). The file reads no `GAUGE_HITS`, so AH step 7's `Mutex` has nothing to protect. 32 s
  wall-clock for 12 rows × 5–9 seats at the shipped `ds`, no coarsening. It is § (iv)'s decision
  paying out: the first file whose design depends on it.
* **Declared narrowings**: the `phi` stator arm only (the suites'; the oracle sweeps both);
  rung-80 readers not seated on rung-79 machines (Python cannot call them there); `split_gains`
  in `clip` (its non-vacuous coordinate here).

##### (b) THE ROWS — **every verdict row landed as pre-registered**

| rung | row | scan | census | march | forced | demand_gains | liveness | arrest | saturation | gains |
|---|---|---|---|---|---|---|---|---|---|---|
| 79 | `at_lever → 78` | same | same | **DIFF** | same | same, hits **0** | | | | |
| 79 | `shared_rig → 78` | same | same | same | same | same, hits 128 | | | | |
| 79 | `cap_fuel → 78` | same | same | **DIFF** | same | same, hits **0** | | | | |
| 79 | `with_coord → 78` | same | same | **DIFF** | same | same, hits **0** | | | | |
| 80 | `at_lever → 79` | same | same | same | same | same | same | same | same | same |
| 80 | `shared_rig → 79` | same | same | same | same | same | **DIFF** | **DIFF** | **DIFF** | **DIFF** |

* **Rung 79's three table swaps are ONE deletion to every seat.** `coord_march` reads one
  identical string under all three (a `phi` march against a `phi` march, empty log, all six
  counters zero), and `demand_gains` reads the shipped values under all three with § (i)'s 128
  incidence hits gone to zero — P2b's masks, holding on every row. The `with_coord` row is, in
  effect, the Python repair § (i)(c) declined to make; at this seat it moves no value.
* **The scan/census/forced counters equal the shipped ones on every row**, as predicted: those
  readers call `phi_cap` with an EXPLICIT coordinate, table-free.
* **Rung 80's `shared_rig` row**: no split reader refuses — `engine.py:21486`'s guard lives in the
  deleted cell — and all four DIFF. Every rung-80 counter vector equals shipped.

##### (c) THREE KINDS OF SILENCE, NOT AH's TWO

AH § 5.32.7 (b) separated *ran, no difference* from *never entered*. This slice adds a third:

1. **Ran, no difference** — rung 79's `shared_rig`: the injected `at_lever` had already carried
   `phi_ref`, so the row matches shipped on values AND counters at every seat.
2. **Never entered** — `cap_fuel` at every CLIP-only seat (the three gauge-point readers,
   `split_gains` in `clip`): 0 entries, AH's *only the DEMAND march calls it*.
3. **Entered, built a DIFFERENT machine, never dispatched it** — rung 79's `at_lever → 78` at
   scan/census/forced. `the_rung_79_at_lever_rig_differs_where_no_reader_dispatches_it` reads the
   rig `gauge_points` builds: it carries RUNG 78's `cap_fuel` and `with_coord`, and the `Count`
   row enters neither at those seats. A nonzero `at_lever` count there does NOT mean *ran and made
   no difference*; the difference was built and then never consulted.

##### (d) THE TALLIES — PINNED, AND ONE DETAIL MISPREDICTED

`[scan, census, march, forced, demand_gains, liveness, arrest, saturation, gains]`:

| cell | rung 79 | rung 80 |
|---|---|---|
| `at_lever` = `shared_rig` | 2, 2, 4, 2, 1 | 2, 2, 4, 2, 1, 6, 18, 8, 3 |
| `cap_fuel` | 0, 0, 2 732, 0, 128 | (the same five), 4 098, 24 588, 10 931, 0 |
| `with_coord` | 0, 0, **8**, 0, **32** | (the same five), 0, 0, 0, 0 |

* **`with_coord` was predicted at 4 and ~16 and measured at 8 and 32.** The prediction counted
  SCOPES; every `CoordScope` dispatches the cell TWICE, on set and on drop — the restore goes back
  THROUGH the table. That second dispatch is exactly the path item L's survivor bypassed (step 6).
* `cap_fuel` at `demand_gains` is 128 = the incidence hits: every entry there took rung 79's branch.
* `split_saturation`'s 10 931 is 3 more than 8 × 1 366: one of its eight marches is three calls
  longer. Pinned, not explained.
* Rung 80's `at_lever` is ENTERED at every one of the nine seats, so its all-`same` row is
  REDUNDANCY. `the_rung_80_at_lever_row_keeps_the_split_on_a_rung_79_rig` reads the mechanism:
  under the swap the marched rig is a RUNG-79 machine (its `shared_rig` and `at_lever` pointers)
  that still carries the requested `sm_air` with the air wall above the fuel wall, because
  `r80_shared_rig` re-reads the knob off the CORE.

##### (e) THE CLOSING SOURCE MUTATION — **delete `at_lever: r80_at_lever,`** (compiles: `..R79` fills it)

Predicted in writing first; eight binaries, `--no-fail-fast`, restored by `git checkout` with the
blob hash verified (`f18dcef1`).

| binary | result | what failed |
|---|---|---|
| `slice_ai_cells.rs` | **8 / 10** | the pointer census and the `sm_air` carry gate — predicted |
| `slice_ai_dispatch.rs` | **0 / 9** | the pointer gate, the rebuild-helper proof, the install proof (so every matrix gate), the mechanism gate — predicted |
| **`rung80.rs`** | **16 / 16** | **nothing — PREDICTED TO FAIL, and wrong** |
| `slice_ai_oracle.rs` | 6 / 6, **0 of 37 945 keys moved** | — predicted |
| `rung79`, `slice_ai_split`, `_scan`, `_march` | green | — predicted |

Plus rustc's *function `r80_at_lever` is never used* — AH's free tripwire, present again.

**The miss is the finding.** `tests/test_rung80.py` has no `at_lever`, `isinstance` or carry test
at all (grepped), so the ported suite is faithfully blind, not under-ported. The override is
redundant on every shipped path in PYTHON too — `_shared_rig` re-sets `_sm_air` off `self` — so
rung 80's `at_lever` is visible to Rust pointer identity and to nothing value-bearing in either
language. This measures slice AJ's booking (§ 5.33 (ii)) at its first rung: at rung 80 no reader
depends on the rebuilt machine being its own rung. Whether one at 81–84 does stays AJ's question.
The Python half is inferred from the grep and the port's fidelity, not re-run.

##### (f) THE PREDICTIONS, SETTLED — SLICE AI CLOSES

* **P1 — REFUTED as registered; the PER-LINE mechanism wins.** Raw `wc -l`, AH's count:
  `state_coordinate.rs` 1 290 + `split_wall.rs` 905 = **2 195**, below the per-class band
  (2 700–3 100) and below its own 2 600 threshold. The per-line figure was ≈ 2 420; 2 195 is 225
  under it and 705 under the per-class 2 900. AH's two mechanisms were 140 lines apart and AI's
  ~480, so this is the first slice where they separated, and the port's size followed the Python's
  BODY, not its class count. Growth after step 4's 2 184 is doc edits only.
* **P2** — scored at step 6: P2a CONFIRMED, P2b CONFIRMED, item L's *"and by the counters"*
  REFUTED. Step 7 adds P2b on three more rows ((b)).
* **P3 — CONFIRMED** at step 6, and used here as a design premise ((a)).
* **P4, P5 — HELD** (steps 1 and 5).
* **P6 — 0 ADD, `TripleHooks` at 18 fields**, all seven steps.
* **P7 — NOT SCORED**, as registered: seven steps by numbering.

##### (g) GATES

The citation guard is re-blessed 46/308/182 → **47/309/182**: one new file, one site, no new
anchor (`21486`). The first draft cited it as a bare `` `:21486` `` in a comment block that never
names `engine.py`, which the scanner does not open — an unwatched citation by FORM, caught only
because the file count did not move after the re-bless. Rewritten in the `engine.py:N` form.
