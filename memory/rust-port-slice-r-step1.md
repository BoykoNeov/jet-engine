---
name: rust-port-slice-r-step1
description: "Slice R step 1 (rungs 40/44) — a registered margin was read off the wrong assertion, and the guard I added was unreachable in the exact case it was built for"
metadata: 
  node_type: memory
  type: project
  originSessionId: 26f13b82-6764-40dd-9696-208fabfe5da4
  modified: 2026-08-18T12:20:17.634Z
---

Phase 6 slice R step 1 (rungs 40 + 44, `TwoSpoolTransient` → `rust/src/two_spool_transient.rs`)
shipped 2026-08-18: the port + `oracle/dump_slice_r_smoke.py` + `tests/slice_r_smoke.rs` +
`tests/slice_r_dispatch.rs`. **1 182 values bit-exact against PyPy**, crate 691 run / 0 failed.
Steps 2–4 remain (rung 40's 9 items, rung 44's 8, the oracle).

**A PRE-REGISTERED MARGIN CAN BE READ OFF THE WRONG ASSERTION.** Prediction 8 said a truncated
step count would be *invisible to every Python gate*, reasoning from the one gate whose grid makes
it live: a 0.2 threshold against a measured 0.398, "a 2× margin". Injected, that gate **fails** —
four lines earlier, at `assert elo * ehi < 0.0`, a bracket-existence check that needs a SIGN
CHANGE and has no margin at all. **Before quoting a gate's margin, find which of its assertions
runs FIRST.** The same prediction's other half ("the oracle catches it through the length AND the
values") was also false: the extremum on the ramp that gate runs is bit-for-bit unchanged.

**AND THE REASON IS EXACT, NOT NUMERICAL.** Probe 5 had measured rung 40's linear running-line
reference and rung 44's per-instant one agreeing "to seven figures". On the shipped ramp they agree
**bit-for-bit**, because the extremum is attained at the instant the ramp SATURATES, where `u == 1`
and the interpolation IS the endpoint match. So the section built to gate that choice reported the
unification injection as one census key and ZERO values until a non-saturating cell was added.
**When two constructions coincide at the point you measure, the coincidence may be structural —
check where the extremum lands before believing a value gate covers the choice.**

**A GUARD CAN BE UNREACHABLE IN EXACTLY THE CASE IT WAS BUILT FOR.** The comparator got a
"never-compared golden key" check so a field missing from the PORT could not hide (the dump
enumerates Python's dict keys). It asserted value diffs FIRST, so it could only fire when nothing
else did — and the one injection it was for (a short march, which omits a whole point) always moves
values too. Merged into one panic, plus `pts.get()` where the section had indexed blind, then
**watched to fire** by commenting out one line. Same family as
[[rust-port-documented-gate-that-doesnt-exist]] and slice Q's *manufacture the failure* rule
([[rust-port-slice-q]]).

**"BOTH ARMS ARE LIVE" IS NOT "THE CHOICE IS WORTH SOMETHING".** The slice's pre-flight made a
headline of a `min` being genuinely contested (1 221 vs 5 118) where the previous rung's bound
15 of 15. Deleting the contested arm moves **no value**: two `illinois_evals` counts change and
every root comes back bit-identical from a different bracket. **A branch census says which arm is
TAKEN; only an injection says what taking it is WORTH.** Related: [[rust-port-slice-j]].

**COUNT THE THING YOU NAME.** The pre-registration wrote "16 tests (9 + 7), counted, not taken from
the header" — collected: **17 (9 + 8)**, and three different counts are in play (gates named in a
docstring, functions defined, items collected). The paragraph existed *because* an earlier count was
wrong. Related: [[rung79-gap-margin]], [[rust-port-slice-n-step4]].

Two of nine injections stay INVISIBLE and are recorded as such rather than papered over: `best`'s
strict-`<` tie-break (no two Newton passes tie on this grid) and the march-in ladder's spelling
(the loop is dead). Detail: `docs/plans/todo-rust-port.md` § 5.15.
