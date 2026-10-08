
#### 5.31.5 SLICE AG step 5 — rung 76's readers, and **TWO OF SIXTEEN SWEEP VERDICTS WERE PROPERTIES OF THE GRID I COPIED FROM THE SUITE, NOT OF THE CODE — AND THE CITATION GUARD CAUGHT A CITATION THAT WAS WRONG AT BIRTH, WHICH ITS OWN DOCSTRING SAYS IT CANNOT DO**

**SHIPPED**: `src/sensed_cap.rs` **578 → 1 319 lines** — `cap_rows`, `cap_gains`, `cap_bill`,
`solve_gain`, their four row/cell types (`CapRow`, `CapCell` + `CapCellRead`, `CapBill`,
`SolveGainRow` + `SolveGain`), the two Python-faithful folds `py_max`/`py_min`, and the `REF_SCHED`
constant `cap_gains`'s target branch compares against. **NO gate file**: steps 3 and 4's precedent
— a readers step proves itself by DRIVING every reader end to end, and the ported gates are step
6's. **`Rust == PyPy` bit for bit on all 1 814 keys, on the first run that ever compiled, both key
sets equal, no port fix.** Sweep: **16 injections, 12 KILLED, 4 SURVIVED, 16 of 16 VERDICTS
RIGHT** — and, as at step 4, the verdict axis is the WEAK reading. The strong one is that **two of
those sixteen verdicts would have been WRONG on the grid this drive started with**.

##### (a) THE LEADING FINDING — **A MUTATION SWEEP'S VERDICT IS A PROPERTY OF THE GRID, AND A GRID COPIED FROM THE SUITE INHERITS THE SUITE'S COVERAGE, NOT THE CODE'S**

The drive was built the way steps 3 and 4 built theirs: `tests/test_rung76.py`'s arguments, copied
one by one. It came back bit-exact on 1 266 keys. **Then the golden was read for what it could
SEE, and it could not see two things:**

* **`n_inert = 0` in every cell.** At the suite's `margin = 0.10`, 10 of 10 rows bind under both
  cap laws — so `_cap_rows`'s three-`_cap_fuel` min-select guard, the most elaborate block in the
  method, computes three caps in order to decide a constant.
* **`row_err = None` in every live cell.** All four live cells are FUEL-authoritative, and
  `row_err` is stated only where the GOVERNOR holds. So its `Some` arm never runs — and the
  `sched`-versus-`applied` target split *inside* that arm, which is § 1's **P6**, is two levels
  unreachable.

`margin = 0.20` was added for exactly that reason (§ 5.31 (i)'s own published sweep says the accel
leg stops winning everywhere there): 7 of 9 rows bind, all 8 cells go live, the `gov` cells
populate, and both `row_err` targets evaluate. **The consequence is measured, not argued:**

| injection | @ margin 0.10 | @ margin 0.20 |
|---|---|---|
| `accel_binds` folds `min` instead of `max` | **0 keys** | **33 keys** |
| `row_err`'s two targets INVERTED | **0 keys** | **4 keys** |

Both are real defects in load-bearing expressions. On the suite's grid alone both score
**SURVIVED**, and this section would have written them up beside § (c)'s four genuine
unreachabilities — *a defence with no reader*, four times over, with two of the four false.

**The generalisation.** Step 4 § (b) found a drive that *picks one cell to keep the runtime down*
scoring a reload-guard bug as correct. This is the same failure with the cell chosen by INHERITANCE
rather than by economy: a test suite's grid is chosen to make its own assertions fire, and the
assertions a suite makes are not the expressions a port has to get right. So the question
[[rust-port-slice-w-step3]] asks of a probe — *make the instrument prove it can SEE* — has to be
asked of the GOLDEN before the sweep runs, by reading the golden for zeros and for absent keys.
Here the two tells were an aggregate that was `0` in all eight cells and an optional that was
absent in all four live ones, and both are visible in the emitted TSV without running anything.

##### (b) THE SECOND — **STEP 4 § (a)'s DOMAIN, BOUNDED ON A THIRD SIDE, AND THIS TIME THE PREDICTION WAS RIGHT**

`cap_rows` passes `Some(accel)` into `rhs_gains_at` where rung 75's `windup_rows` passes `None`
(`engine.py:19352`). Copying the parent's call site is the single highest-value defect this step
can ship: the reader still returns well-formed Jacobians at every point, the accel branch of
`_cap_fuel` simply never runs, both laws agree, and § 1's whole headline reads as REFUTED.

Step 4 deleted the same schedule from `cap_march` and the drive **PANICKED**, on
`r76_integrate_fuel`'s third refusal — from which § 5.31.4 (a) wrote the domain as *the blindness
is the `solve` arm's, and the arm this rung is about is defended by a refusal already in the file*.
**Deleting it HERE is silent**: pre-registered as *KILLED by VALUE, and NOT by panic*, measured at
**144 of 1 814 keys, no panic**, because a reader reaches `_cap_fuel` directly and never enters
`integrate_fuel`. So that refusal defends the MARCH's callers and **not the readers, and nothing
else does** — what catches it is a POSITIVE count of what the two laws differ by, which is what
§§ E/F of the drive are. Three steps, three answers to *what refuses this*, and the third is
*nothing does*.

##### (c) THE THIRD — **FIVE DEFENCES WITH NO READER IN ONE STEP, EVERY ONE PRE-REGISTERED AS SUCH**

Step 4 shipped one (`c_at`'s expression-first `max`, § 5.31.4 (c)). Step 5 ships five, and each was
typed into `predictions.md` as SURVIVES *with its reason* before the sweep ran:

| expression | why it is written the long way | measured |
|---|---|---|
| `tau_auth` / `tau_masked` per row | the clock the authoritative row divides by is not the masked row's | **0 keys** — `taus[1]` is `0.05` and rung 52's lag returns its ATTACK clock, also `0.05`, at every riding point of a ramp, so the two are the same number |
| `WindupScope` OUTSIDE, `CapScope` INSIDE | Python's `_with_windup(…, m._with_cap, …)` nests them | **0 keys** — the guards write disjoint fields, so neither restore can observe the other |
| `masked_moved`'s conditional | RELATIVE where the base is nonzero, ABSOLUTE where it is not | **0 keys** — a live 4×4's masked diagonal is never within `1e-30` of zero |
| `s_tail`'s `max(taus)` | Python's fold over the four clocks | **0 keys** — every shipped grid sets all four to `0.05`, so `min == max` |
| `py_max` / `py_min` | Python's `max()` propagates a NaN that arrived FIRST; `f64::max` discards it always | `dS == 0.0` at **0 of 29 rows** across all three margins, so the one NaN reachable by construction in this slice is not reachable on this plant |

**That is not a regression against step 4's one — it is what a READERS step IS.** Four small
methods over a plant produce one undrivable spelling; four AGGREGATES over trajectories in general
position produce five, because an aggregate's edge cases are exactly the states a converged march
does not visit. All five are disclosed in the doc comments at the step that shipped them.

##### (d) THE FOURTH — **IDENTITY (1) IS EXACT AT 8 OF 10 ROWS, WHICH MAKES THE OBVIOUS GATE ON IT SELF-CERTIFYING**

`solve_gain`'s `fixed_point` is `|cap_sensed(cap_solve) − cap_solve|`, and § 3's first identity is
that this is **exactly zero** — `cap_solve` is by construction the fixed point of `cap_sensed`.
Injection 15 replaced `sensed(q, w0)` with `solve(q)`, which is `w0` itself: a reader comparing the
solve with itself.

**KILLED — but at 17 keys of 1 814, and none of them at the suite's own margin's eight zeros.**
Measured: `fixed_point` is `+0.0` bit for bit at **8 of 10 rows at `margin = 0.10`**, 4 of 10 at
`0.05`, 3 of 9 at `0.40`. At every one of those rows the wrong expression and the right one return
the same float. So a step-6 gate that asserts identity (1) at `margin = 0.10` — the suite's own
setting, and the natural choice — is reading a quantity that **cannot distinguish the implementation
from one that measures nothing**, at four fifths of its points. That is
[[instrument-fed-by-what-it-certifies]] in its purest form, and the twist is that the exactness
doing the certifying is the very fact being certified. **Booked as a requirement on step 6: assert
identity (1) on a row where `fixed_point` is NONZERO.**

##### (e) `fuel_int`'s FOLD DIRECTION IS EXACTLY 4 KEYS, WHICH IS **P2** ASKED OF THE PORT

Folding the 341-point `mf` trajectory right-to-left instead of left-to-right moves **exactly 4
keys** — the two arms of the two driven bill cells, and nothing else in the file. § 5.31 (iv)
measured the CPython arm needing an exemption for precisely `cap_bill/fuel_int/0` and `/1`, 2 of
83 273 keys, on the grounds that those are the only two `sum()` calls in either class that add a
trajectory rather than a literal `1`. This is the same fact seen from the port's side: summation
ORDER is observable in exactly two places per bill cell and nowhere else, so the fold is written as
`fold(0.0, +)` and the reason is in `CapBill::fuel_int`'s own doc rather than left implicit.

##### (f) THE CITATION GUARD PAID ON ITS FIRST INTENDED USE — **ON THE FAILURE MODE ITS OWN DOCSTRING SAYS IT CANNOT CATCH**

Four new citations, all step 5's own, **no arrears** — step 4 § (k)'s procedural repair held, and
this step ran the guard BEFORE shipping rather than two steps later. Census `10 / 59 / 47` →
`10 / 63 / 51`.

`test_rust_line_citations.py`'s docstring is explicit that it detects **DRIFT, not wrongness**: *a
citation blessed while pointing at the wrong line stays wrong, and no instrument can know what a
comment MEANT to point at.* **It caught one anyway.** § (b)'s sentence was typed with
`engine.py:19353`; the re-bless printed the text it lands on —

> `19353: None -> 'g = read("sensed")'`

— and the argument the sentence is about is one line up (`19352`, `flight, p, accel, surge,
Tt4_max, tau_f, taus[1]`). The mechanism is not the drift check: it is that **a NEW anchor prints
the line it lands on**, so blessing is a moment where a human compares the sentence with the line
for every citation the step added. The guard's stated limit is real for a citation blessed
silently; it does not bind when the blessing prints. That distinction is now in the file's own
census comment, because it is the difference between an instrument that only catches decay and one
that also catches birth defects — and nobody had noticed the second was free.

##### (g) THE SWEEP — 16 injections, predictions typed first, **12 KILLED / 4 SURVIVED, verdicts 16 of 16**

Instrument: patch `src/sensed_cap.rs`, rebuild, re-run the drive, diff its 1 814 keys against the
PyPy golden, restore, verify SHA-256 back to pristine
(`e6133184b55e44ce52dc800e291800c7c9e3bd3c17933ca07abba8c4394bf28d` — step 4 § (g)'s correction
observed: that is the SWEPT file, not the shipped one, which is
`43766021cc75e0d9adecf4b6e804ecfff39f7741d6aae2905559b0241dec4736` after §§ (a)–(e)'s additive doc
repairs, two clippy closures and § (f)'s citation fix, and the drive was re-run against it three
times — after the doc repairs, after the clippy fix, and after the citation fix — because *a
reasoned "this cannot move a float" is worth exactly one run* and there were three of them).

| # | injection | predicted | measured |
|---|---|---|---|
| 1 | `cap_rows` passes `None` to `rhs_gains_at` | KILLED by VALUE, **not** by panic | **KILLED**, 144 keys, no panic → § (b) |
| 2 | `cap_rows` drops `lag_coord.set(DEMAND)` | KILLED by PANIC | **KILLED by PANIC**, `anti_windup.rs:261` — rung 75's refusal, at the two `track` cells |
| 3 | `accel_binds` folds `min` | KILLED, **0.20 only** | **KILLED**, 33 keys, all at 0.20 → § (a) |
| 4 | the SENSED cap call drops `mf_app` | KILLED by PANIC | **KILLED by PANIC**, `sensed_cap.rs:221` — `r76_sensed_cap`'s threading refusal |
| 5 | the PHI cap call passes `Some(accel)` | KILLED, large | **KILLED**, 485 keys |
| 6 | `tau_auth` / `tau_masked` SWAPPED | **SURVIVES** — one clock | **SURVIVED** → § (c) |
| 7 | the two scopes nest the other way | **SURVIVES** — disjoint fields | **SURVIVED** → § (c) |
| 8 | `auth_err` targets `(c+1)/tau_auth` | KILLED | **KILLED**, 12 keys |
| 9 | `masked_moved` loses its conditional | **SURVIVES** — the `else` is unreachable | **SURVIVED** → § (c) |
| 10 | `row_err`'s targets INVERTED | KILLED, **0.20 only** | **KILLED**, 4 keys, all at 0.20 → § (a) |
| 11 | `det_err` scores against `(1+c)` | KILLED | **KILLED**, 8 keys |
| 12 | `n_inert` counts the BINDING rows | KILLED at both margins | **KILLED**, 12 keys |
| 13 | `s_tail` folds `min(taus)` | **SURVIVES** — four equal clocks | **SURVIVED** → § (c) |
| 14 | `fuel_int` folds RIGHT to LEFT | KILLED, **exactly 4** | **KILLED, exactly 4** → § (e) |
| 15 | `fixed_point` reads `solve(q)` | KILLED — *unless the identity is exact* | **KILLED, 17 keys**, and the caveat is the finding → § (d) |
| 16 | `predicted = 1/(1+c)` | KILLED | **KILLED**, 32 keys |

**AND THE SWEEP'S OWN CLASSIFIER MISLABELLED TWO ROWS.** It scored injections 2 and 4 as *KILLED
(compile)* because it tests `"error: " in out`, and cargo prints `error: test failed, to rerun…`
above every panicking test. Both are PANICS, which is what both were predicted to be, so no verdict
moved — but the label was wrong on the one axis the row exists to report, and it is recorded
rather than quietly corrected. An instrument that classifies is an instrument that can classify
wrongly; this one is a throwaway and was not repaired, which is a decision and not an oversight.

##### (h) SIZING — **2.48× by region and 2.35× by body, both INSIDE P1's band**

744 lines added against the four Python readers' 300 (`engine.py:19320-19619`), of which 315 are
comment and 411 are code against Python's 175. So **2.48×** by file region and **2.35×** by body,
where step 4's four small methods came in at 5.09× and 3.15×. § 5.31.4 (h) predicted exactly this
shape: a doc-dominated small step pulls the ratio up and a body-dominated large one pulls it back.
**P1** is a slice total and is settled at the last step; this is the data point that says the two
kinds of step have to be weighted by SIZE and not averaged.

##### (i) GATES

`cargo test --release`: **150 `Running` + 1 `Doc-tests` = 151 blocks, 151 of 151 ok, 1 565 passed /
0 failed / 0 ignored, 0 `error[E`, `CARGO_EXIT=0` READ OFF DISK** — **delta ZERO against step 4**,
which is this step's own prediction and holds because the throwaway harness
(`rust/tests/slice_ag_step5_drive.rs`, archived to `W:	emp\claude\slice-ag-step5\`) was
deleted before the gate ran. A change in the delta would have been the finding.

`clippy --all-targets`: the step's two new warnings — `redundant_closure` on `py_max` and `py_min`,
`.map(|x| f(x))` for `.map(f)` — are CLOSED, and the drive was re-run against the closure to
confirm neither moved a float. The remaining `sensed_cap.rs` warning is step 4's `c_at` at 9
arguments, and the two crate-wide `clippy::eq_op` errors are still the deliberate NaN
self-comparisons § 5.31.4 (i) recorded as a lint-configuration debt.

`pytest` — **run because § (f) made it the right gate, and BEFORE the step shipped rather than two
steps after it** — **1 373 passed, 0 failed, 20:42** at BelowNormal priority, **`PYTEST_EXIT=0` READ OFF DISK** — which step 4 could not do: its `Start-Process -PassThru` left `$p.ExitCode` empty after `WaitForExit()`, so its verdict rested on the summary line alone. Launching through `Process.Start` with both streams taken as `ReadToEndAsync` tasks BEFORE the wait captures the code, and that is now the shape to copy. CLAUDE.md's timing anchor is refreshed from THIS run and from step 4's together — `~15:18 at 1373` becomes **15–21 min on a quiet box**, the 83:20 outlier kept — because a single fast figure is a number a reader plans around and then meets three times over, which is exactly what § 5.31.4 (k) said when it wrote the spread in.

The step touches no Python except `tests/test_rust_line_citations.py`'s anchors and census, so the run is here for § (f)'s reason and not as a formality.

##### (j) WHAT STEP 6 INHERITS

* **The two ported gate files** (`tests/rung75.rs`, `tests/rung76.rs`) **and the oracle** — the
  three things this slice has deferred since step 1, and the whole of what step 6 is.
* **§ 5.31.4 (b)'s requirement**, restated because it is still owed: a gate on `CapScope` must read
  `_cap_law` WITHOUT setting it first, on a machine whose resting law is not the one the guard
  arms. Driving the guard through `_cap_fuel` sees neither wrong restore policy.
* **§ (d)'s requirement, new**: a gate on `solve_gain`'s identity (1) must assert on a row where
  `fixed_point` is nonzero, or it certifies the exactness with the exactness.
* **§ (a)'s requirement, new and the widest**: the ported gates inherit `test_rung76.py`'s grid by
  construction, and that grid is measured here to leave `accel_binds` and `row_err` unexercised.
  A ported gate that reproduces the suite exactly reproduces its blind spots exactly; the second
  margin belongs in step 6's file, not only in this step's deleted harness.
* **§ 5.31.4 (c)'s disclosure and § (c)'s five**, which step 6 cannot repair and should not try to:
  no value gate on this plant can distinguish either `max` spelling, the two clock conditions, the
  two scope orders, `masked_moved`'s two arms, or the two NaN folds.
* **P2**, still open and now sharpened from the port's side by § (e): the oracle's exemption list
  is predicted to stay exactly `cap_bill/fuel_int/0` and `/1`, and this step measured that those
  are the only two keys in the whole reader set where summation order is observable at all.
