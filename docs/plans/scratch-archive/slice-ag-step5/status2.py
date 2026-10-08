import io

p = r"C:\Users\boiko\.claude\projects\M--claud-projects-jet-engine\memory\rust-port-status.md"
s = io.open(p, encoding="utf-8", newline="").read()

old = ("`cargo test --release` **151 blocks / 1 565 passed / 0 failed / CARGO_EXIT=0 off disk**, "
       "delta ZERO vs step 4; two new clippy warnings CLOSED; `pytest` green. "
       "See [[rust-port-slice-ag-step5]].")
assert s.count(old) == 1
new = ("`cargo test --release` **151 blocks / 1 565 passed / 0 failed / CARGO_EXIT=0 off disk**, "
       "delta ZERO vs step 4; two new clippy warnings CLOSED; `pytest` **1 373 passed, 0 failed, "
       "20:42, PYTEST_EXIT=0 OFF DISK** (step 4 could not capture that code; `Process.Start` with "
       "both streams taken as `ReadToEndAsync` BEFORE the wait does). **ADDENDUM, same step "
       "number** \u2014 the step's own lesson turned on itself: the `inc` axis was **NARROWED, not "
       "copied**. Every reader call passed `inc = false`, so none of the first 1 814 keys touched "
       "rung 69's incidence plant, while `test_rung76.py` sweeps `inc` in THREE tests, all three "
       "`solve_gain` \u2014 the reader whose gate had just been booked self-certifying. `LeverArm` "
       "already carries `stator_inc` (a drive gap, not a port width gap \u2014 checked FIRST, because "
       "the other answer would have been a step-1 finding). Re-driven and the whole sweep re-run: "
       "**2 691 keys bit-exact, verdicts UNCHANGED**, and three sharpenings \u2014 the `tau_auth` row "
       "(the claim genuinely at risk, since `cap_march` marches past the ramp end at "
       "`tau_rel = 3\u00b7tau_f` and a RELEASING point would split the clocks) survives on a SECOND "
       "plant, `n_tau_split = 0` at all four cells; the two grid-sensitive injections stay confined "
       "to (`phi`, 0.20) and the incidence arm opens NO second route even though the `accel_binds` "
       "guard is LIVE there at the suite's margin \u2014 **a branch being live is not the same claim "
       "as a mutation of it being observable**; and the identity's exact-zero count is 18 of 36 "
       "rows, 1 of 1/2/4 on the incidence arm against 8 of 10 on the suite's, so step 6's sharper "
       "population is also its thinner one. **The sweep's own attribution columns went blind on "
       "that same edit** (bucketed by `E/0.1/`, renamed to `E/i0/0.1/`), caught because two "
       "one-sided rows read zero on both sides beside a nonzero total. Seven **unspelled reader "
       "defaults** named (`CAP_GAINS_REFS`/`LAWS`, `CAP_BILL_TAU_T`/`TAIL`, `SOLVE_GAIN_REF`/`DQ`/"
       "`EVERY`) with step 4 \u00a7 (f)'s PREMISE re-derived, not inherited: no crate caller relies on "
       "them, but step 6's gates are the only call sites and the suite does not write them down. "
       "Two bookkeeping corrections made in place: *16 of 16* is really **15 one-sided plus one "
       "hedged** (injection 15 could not lose), and \u00a7 (a) said *four* where \u00a7 (c) tabulates "
       "five. Citation census \u2192 **10/70/56**. Gates re-run: cargo **151/1 565/0**, `pytest` "
       "green. See [[rust-port-slice-ag-step5]].")
io.open(p, "w", encoding="utf-8", newline="").write(s.replace(old, new))
print("status updated")
