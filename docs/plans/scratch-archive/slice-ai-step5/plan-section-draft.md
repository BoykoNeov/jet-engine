#### 5.33.5 STEP 5 — THE TWO PORTED SUITES AND ALL SEVEN REFUSALS, AND **`_a_cap`'s FREEZE IS INVISIBLE AT THE ONE MARGIN THE ZERO-GAP GATE READS: AT THE PLANT'S OWN FUEL THE UNFROZEN RE-SOLVE LANDS ON THE FROZEN STATE**

`rust/tests/rung79.rs` (**28 gates** — the suite's 25 plus three declared) and `rust/tests/rung80.rs`
(**16 gates** — the suite's 12 plus four declared). The suite counts were COLLECTED (`pytest
--collect-only`), not read off the files: an earlier message of this session said 23 and 13.
**No module line changed** except a doc comment (below): P1 stands at 2 184, P6 at 0 ADD. Both files
compiled first time with zero warnings and were green on their first run.

##### (a) THE SEVEN REFUSALS, EVERY ONE NOW GATED

| site | method | gate | how |
|---|---|---|---|
| `:20910` | `_cap_fuel` | `rung79.rs` #23 (suite) | needle sharpened from `"REFUSED"` (8 sites) to the message's own phrase |
| `:21285` | `_forced_cap` | `rung79.rs` +1 | `phi_lim = 50`, both coordinates, an `Abort` |
| `:21301` | `_forced_cap` | `rung79.rs` +2 | `phi_lim = 1e-3`, both coordinates; the walk ends on a failed evaluation at PyPy's own step (`searched to 4.181503e-02`) |
| `:21467` | `_shared_rig` | `rung80.rs` #3 (suite) | + the caller's `sm_air` read back after the unwind |
| `:21486` | `_shared_rig` | `rung80.rs` +1 | **P5 HOLDS**: `+0` ulp builds (exempt), `+1`/`+2` refuse, `+3` builds at `0x3fe999999999999b` — PyPy's bits |
| `:21551` | `_split_row` | `rung80.rs` +2 | a valve-less rung-80 core; and a point with no `b` dies on the `KeyError`-shaped message first |
| `:21647` | `split_arrest` | `rung80.rs` +3, +4 | each half of the `and` broken on its own |

Every needle is unique in `engine.py` (counted). No gate swaps the panic hook: `take_hook`/`set_hook`
are process-wide, and a parallel refusal gate could restore a sibling's silent hook (advisor).
Measured: zero `panicked` lines in either passing run's output.

##### (b) J6 DISCHARGED — `forced_cap`'s SLACK ARM, PINNED TO PYTHON's BITS

Step 3's J6 (the slack walk growing by `shrink`) survived because the shipped `coord_forced` binds at
all ten points. A refusal gate cannot kill it — a walk in the wrong direction also refuses — so
`rung79.rs` +3 pins a slack SUCCESS on a valve-off rung-79 rig at `(0.8, 0.9, 0.02)`: PyPy measured
`Gs = −0.0600`, roots `…2fd2` (`phi`) and `…2fd1` (incidence), ONE ULP apart, and each equal to
`cap_free`'s slack arm to the bit — two instruction sequences, one arithmetic.

##### (c) WHERE THE PORT IS NOT A LINE-BY-LINE COPY

* `test_reduce_phi_is_rung_78_by_dispatch`'s last line, `G.__code__.co_consts is not None`, is TRUE
  OF EVERY PYTHON FUNCTION. Not copied; the dispatch the name claims is gated instead — R79's cell
  returns R78's float to the bit at a slack state AND moves no rung-79 counter, while the same call
  at incidence moves `hits` (the counter is what stops the equality comparing a function with itself).
* `isinstance` held STRICTER (both pointer halves, `rung78.rs` (b)'s lesson).
* The two D3 tests carry `9f1d11a`'s corrected wording: plumbing, not invariance.
* `test_reduce_sm_air_equal_sm_is_bit_for_bit` also asserts the march ran `demand` (every point a
  `Demand` point, `lag_coord` on both built rigs) — `slice_af_dispatch.rs`'s finding: a clip march
  also returns 341 points.
* `_a_cap` is ported WITH its outer freeze (§ (vii)).

##### (d) THE INJECTION SWEEP — 8 injections, predicted in writing first (`predictions.md`), `--no-fail-fast`, **8 of 8 verdicts right, 1 detail wrong**

| # | injection | result |
|---|---|---|
| N1 | `_a_cap`'s outer freeze removed | **KILLED** — by `the_gap_residual_is_rung_77s_stiffness` ONLY. **The detail I got wrong**: I predicted `gap_at_zero_margin_is_exactly_zero` too. Unfrozen, the plant re-solves its valve and stator for each trial fuel; at `margin = 0` the fixed point IS the plant's own fuel, where the re-solve lands on the marched state, so that gate cannot see the freeze. At `margin ± 0.01` it can (`slope 1.2227` vs `1/(1−c) = 1.254`). |
| N2 | `R79.at_lever` left at rung 78's | **KILLED** ×7, all in `rung79.rs` — the carry gate and every march reader (`hits = 0`); `rung80.rs` untouched, as predicted |
| N3 | J6: the slack walk grows by `shrink` | **KILLED** — +2 (walk length) and +3 (a refusal where Python's root was due); nothing else |
| N4 | the valve wall spelled `ps + sm·ps` | **KILLED** by P5 alone: the band NARROWS to one ulp (`+2` builds). Every other rung-80 gate green — the 0.8 wall itself does not move |
| N5 | `split_arrest`'s first clause dropped | **KILLED** — +3 only |
| N6 | `_split_row`'s valve assert hoisted | **KILLED** — +2 only, on its `KeyError` half |
| N7 | `AirScope::drop` restores nothing | **KILLED** — #2, #3 and +1, every one by the `sm_air` READBACK; no value gate |
| N8 | `r79_cap_fuel`'s `phi` early return removed | **KILLED** by `reduce_phi_is_rung_78_by_dispatch`'s COUNTER assertion; its bit equality held — which is why the two are paired |

##### (e) A STALE SENTENCE IN THE CRATE, FOUND BY THE ADVISOR

`state_coordinate.rs`'s `coord_scan` doc still said the spec's D3 row reads the zero as *"not
small, exactly zero"* — false since `ea43d9c`. Corrected, with `slice_ai_scan.rs`'s header, this
plan's § 5.33.2 and its memory entry pointed at `ea43d9c`/`9f1d11a`. Two stale Python lines are
LISTED, not edited: `engine.py:21073`'s comment (*"the SENSITIVITY does not move at all"*) and
`test_rung79.py`'s probe-flag docstring naming `test_the_gap_log_records_distinct_states`, a test
since renamed.

##### (f) WHAT STEP 5 LEAVES

The oracle, carrying § 5.33 (i)'s `demand_gains`-on-R79 arm (step 6); the dispatch gates (step 7).
The citation guard is re-blessed for four new anchors (43/280/178 → 45/305/182: `21049`, `21285`,
`21301`, `21467`), each read by hand, run after the step's last Rust edit. Gate for this step: full
`cargo test --release` + full `pytest`, predicted before the run at 170 blocks / 1 761 passed and
1 387 — the numbers are in the commit.
