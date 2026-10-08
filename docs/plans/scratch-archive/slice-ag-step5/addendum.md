
##### (k) ADDENDUM, SAME STEP NUMBER — **THE `inc` AXIS WAS NARROWED, NOT COPIED, AND § (a)'s OWN LESSON APPLIED TO § (a) FOUND IT**

§ (a) says a grid copied from the test suite inherits the suite's coverage rather than the code's.
**On the `inc` axis this drive did not copy the suite's grid — it NARROWED it.** Every reader call
in both halves passed `inc = false`, and the Rust `rig()` armed `stator_lim` only, so none of the
1 814 keys touched rung 69's incidence-referenced plant. `test_rung76.py` sweeps `for inc in (False,
True)` in **three** tests, and all three are `solve_gain` — the reader whose gate § (d) had just
booked as self-certifying. The port would have gone into step 6 with no golden behind half of each
of those loops.

**`LeverArm` carries `stator_inc` already** (rung 69's ninth keyword), so this was a gap in the
drive and not a width gap in the port — checked first, because *if it does not, that is a step-1
finding in its own right* and not a reason to skip the arm.

Both halves re-driven with the axis added: **2 691 keys, `Rust == PyPy` bit for bit, both key sets
equal, no port fix.** The full sixteen-injection sweep was then re-run on the wider golden rather
than reasoned about, because the table is a property of the grid — which is this step's own
headline, and would be worth nothing if it were not applied here.

**THE VERDICTS DO NOT MOVE — 12 KILLED, 4 SURVIVED, the same sixteen — and three things sharpen:**

* **§ (c)'s `tau_auth`/`tau_masked` row was the claim at risk, and it survives on a second plant.**
  `cap_march` sets `tau_rel = 3·tau_f = 0.15` and marches to `s_settle = 1.2`, past the ramp end
  `r = 0.5`, so a RELEASING point among the `riding4` survivors would have made the two clocks
  differ and killed injection 6 — a property of the trajectory, and the incidence arm is a
  different trajectory. Measured: `n_tau_split = 0` at all **four** (arm, margin) cells, and the
  swap still moves 0 of 2 691 keys. The row keeps its verdict and gains its population.
* **§ (a)'s two grid-sensitive injections stay grid-sensitive, and the incidence arm does NOT open
  a second route — which is the more interesting outcome.** Re-attributed per cell: injection 3's
  33 keys are **all** at (`phi` arm, `margin = 0.20`) and injection 10's 4 keys likewise. Yet on
  the incidence arm the `accel_binds` guard is **LIVE at the suite's own margin** — `n_inert = 1`
  of its 2 rows, where the `phi` arm has 0 of 10 — and the injection still moves **0 keys there**,
  because that one filtered row is filtered under both folds. So **a branch being live is not the
  same claim as a mutation of it being observable**, and exactly one of the four cells can score
  that expression at all. `row_err` stays reachable only at (`phi`, `0.20`): the incidence arm has
  **no `gov`-authoritative rows in any cell**.
* **§ (d)'s self-certifying gate gets its better population named.** `fixed_point` is exactly
  `+0.0` at **18 of the 36 driven rows**, and the split is what a step-6 gate needs: 8 of 10 at the
  suite's own cell, but **1 of 1, 1 of 2 and 1 of 4** on the incidence arm. Injection 15 moves 28
  keys, of which the suite's own cell contributes 4. The incidence arm is the sharper population
  and the thinner one, which is itself a constraint on how that gate is written.

**AND THE ATTRIBUTION COLUMNS IN § (g)'s TABLE WENT BLIND WHEN THE AXIS WAS ADDED.** The sweep
buckets differing keys by the prefixes `E/0.1/` and `F/0.1/`; the new axis renamed every key to
`E/i0/0.1/`, so the per-margin counters read `0 @0.10, 0 @0.20` on all sixteen rows — including the
two whose whole point is the split. Caught because two rows that must be one-sided reported zero on
both sides, which is arithmetically impossible next to a nonzero total. Re-measured with a
corrected bucketer (`W:\\temp\\claude\\slice-ag-step5\\attrib.py`) rather than by patching the
number in prose. **An instrument that partitions is an instrument whose partition can stop matching
the data**, and it stopped matching on the same edit that made the partition worth having — the
sibling of § (g)'s classifier mislabelling a panic as a compile error, in the same sweep.

**THE COUNTS, restated on the 2 691-key golden** (verdict unchanged in every row): 1 → 216, 3 → 33,
5 → 713, 8 → 18, 10 → 4, 11 → 14, 12 → 14, 14 → **exactly 6** (three bill cells now, two arms
each — P2's shape unchanged), 15 → 28, 16 → 42; injections 6, 7, 9 and 13 remain at **0**.

##### (l) AND THE READERS' UNSPELLED DEFAULTS ARE NAMED — **step 4 § (f)'s rule with its PREMISE re-derived rather than inherited**

Step 4 named `ACCEL_SCHEDULE_N` and `C_AT_REL` because *the crate itself relies on them*:
`accel_for` calls `accel_schedule` with no `n`. **Not one of step 5's four readers is called from
anywhere in `rust/src`**, so on that premise alone nothing here needs a name — and stopping there
would have been the inherited half of the rule rather than the rule.

The other half still bites: *a bare literal at the call site is a number nobody can check against a
signature.* **Step 6's ported gates are the call sites**, and they will type these values from
`test_rung76.py`, which does not spell them; nor does `main.py` (which passes `phi_lim` and `margin`
and nothing else, at all three readers); nor `engine.py`'s own `Usage:` block (four positional
arguments). Seven constants, each a fact about a signature no caller in the repository writes down:
`CAP_GAINS_REFS` / `CAP_GAINS_LAWS` (`engine.py:19403`), `CAP_BILL_TAU_T` / `CAP_BILL_TAIL`
(`19497` / `19500`), `SOLVE_GAIN_REF` / `SOLVE_GAIN_DQ` / `SOLVE_GAIN_EVERY` (`19550` / `19552`).

`CAP_GAINS_LAWS` is spelled out rather than aliased to
[`WINDUP_LAWS_DECLARED`](crate::anti_windup::WINDUP_LAWS_DECLARED) even though the two arrays are
equal, because they are equal by COINCIDENCE and not by construction — one is the set rung 75's
`windup_tau` refuses outside, the other is the set this reader sweeps — and an alias would make a
later widening of either silently widen the other. The defaults the suite DOES spell (`phi_lim`,
`margin`, `taus`, `r`, `s_settle`, `ds`, `v_max`, `inc`, `cap_bill`'s `ref`/`law`, `cap_gains`'s
`every`) are deliberately NOT named: a const for one of those would be a second home for a number
step 6 transcribes from the suite, which is how two copies of one value start to disagree.

Citation census `10 / 63 / 51` → **`10 / 70 / 56`**, all five new anchors verified by hand against
the sentences that cite them.

##### (m) THE TWO BOOKKEEPING CORRECTIONS THIS SECTION OWED ITSELF

* **"16 of 16 verdicts right" over-counts by one.** Injection 15 was registered two-sidedly —
  *KILLED, small… but if identity (1) holds to the last bit, it SURVIVES* — and a prediction that
  admits both outcomes cannot lose. The honest score is **15 one-sided predictions, all right, plus
  one hedged**, and [[rung81-authority-clock]]'s *the first grid's 100% was the weaker measurement*
  is the reason to say so rather than to keep the round number. The hedge was not idle — it is what
  made § (d) a finding instead of a row — but it is not a verdict.
* **§ (a)'s closing sentence said FOUR where § (c) tabulates FIVE.** It read *"beside § (c)'s four
  genuine unreachabilities — a defence with no reader, four times over, with two of the four
  false"*; had injections 3 and 10 survived, the set would have been **seven with two false**. A
  section contradicting itself before anyone looked is § 5.31 (i)'s own leading lesson, so it is
  corrected in place rather than left to a reader to reconcile: **five genuine, and the two false
  ones would have made seven.**

##### (n) GATES, ADDENDUM

`cargo test --release`: **150 `Running` + 1 `Doc-tests` = 151 blocks, 151 of 151 ok, 1 565 passed /
0 failed / 0 ignored, 0 `error[E`, `CARGO_EXIT=0` READ OFF DISK** — delta ZERO against § (i), the
widened harness having been archived and deleted again. `clippy`: unchanged, the one
`sensed_cap.rs` warning still step 4's `c_at` at 9 arguments. `pytest`: **1 373 passed, 0 failed**,
run because § (l) added five `engine.py` citations and § (f)'s rule makes that a gated input.
