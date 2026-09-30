---
name: rust-port-slice-aj-step5
description: "Slice AJ step 5 (rungs 81–84's ported suites + ten driven voids) — an order of two checks is visible only on an input that FAILS BOTH; 'falls through' was reasoned one check short. And two reduce tests would have compared split_march with itself"
metadata:
  node_type: memory
  type: project
  originSessionId: 56ea0c62-2539-4e36-afbd-b228950831e7
  modified: 2026-09-30T07:53:08.483Z
---

Slice AJ step 5 COMPLETE 2026-09-30 (plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.34.5). `rust/tests/rung81.rs` … `rung84.rs`, 55 gates = the suites' 45 (collected 12/16/8/9)
+ the ten § (vi) voids, strings written from `engine.py` and measured on PyPy first
(`W:\temp\claude\slice-aj-step5\probe_voids.py`, with a `sys.version` entry control). No `src/`
change. Green 55/55 on first run. Opening gates (step 4's unfinished ones): cargo 177 blocks,
1 824/0; pytest 1 386 + 1 failure that was THIS step's new files tripping the citation census
(it scans `rust/tests` at run time) — re-blessed 61/420/245. Next: step 6, the oracle.

**THE LESSON: the order of two checks is visible only on an input that FAILS BOTH.** Injection
I13 moved `_bisect`'s straddle test above its window test. I predicted the V1 gate at `r = 1.0`
would kill it because "the straddle holds, so the reordered code falls through" — true, and it
falls through TO THE WINDOW TEST, which returns the same V1. Survived all four suites. Step 3 had
recorded this exact fact (*"a void's order is visible only where both bracket ends are empty"*) and
its `slice_aj_threshold.rs` reading killed I13 alone in a follow-up predicted in writing first.
**How to apply:** when predicting a kill for a reordering, trace the mutated code to its RETURN on
the gate's input, not to the first branch that differs; and for an order of N checks, the gate
needs an input that trips at least the two being swapped. **And apply it to your OWN gates the
moment you write it down**: the advisor found two more unlisted instances in this step's own void
gates (`corrector_step` V4/V5, `root_class` carry/V6); both got a both-conditions arm, measured on
PyPy first, and their reorder injections I14/I15 were killed by those arms ONLY (sweep 15, 14 right).

Also:
- **Two of four reduce tests were self-comparisons in Rust** (the advisor's catch, before code):
  Python compares two CLASSES; the port has one `R80` core, so both sides were one `split_march`.
  Substituted by a THREADING check — the reader's output at DISTINCT clocks vs an independent
  march read off the points' own fields. It killed the clock-swap injections (I1, I2, I4) ONLY
  where the clocks differ; a q/s swap in `authority_clock` (fixture q = s) survives, recorded.
  [[rust-port-ported-test-vacuity]] applied at the moment of porting, not after.
- **An added arm closed a coverage gap step 4 had listed**: `(0.05, 0.30)` at `r = 1.0` — first
  start empty, second VALID — separates `break` from `continue` (C13); the pre-flight's `(0.05,
  0.06)` failed at both. Found by combining step 3's "0.30 is open at r = 1.0" with step 4's list.
- Three formatted voids (`0.004`/`0.3`, `0.0198`, `1`) print the same under `%g` and `{}`; stated
  as predicted survivors before the sweep, not discovered after. Sweep 13, 12 of 13 right.
- A one-line `python3 - <<EOF … || venv-python <<EOF2` fallback HUNG (the `python3` Store stub);
  write scripts to a file and run the venv interpreter by path.

Related: [[rust-port-slice-aj-step4]], [[rust-port-slice-aj-step3]], [[rust-port-status]].
