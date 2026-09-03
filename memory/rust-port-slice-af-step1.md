---
name: rust-port-slice-af-step1
description: "Slice AF step 1 — a gate whose doc comment claimed it measured a constant through behaviour still took its reference FROM that constant, and only the mutation sweep saw it; plus a width arrival that fired two tripwires nobody had named"
metadata: 
  node_type: memory
  type: project
  originSessionId: cdf27442-ba5b-411e-9320-062136386f41
  modified: 2026-09-03T18:58:09.499Z
---

Slice AF step 1 (rung 74, `DemandCoordinateTransient`). Plan § 5.30.1. Four added cells
(`_cap_fuel`, `_sensed_cap`, `_windup_tau`, `_with_coord`), four re-aimed pointers, `TripleHooks`
14 → 18. See [[rust-port-status]], [[rust-port-slice-af-preflight]],
[[instrument-fed-by-what-it-certifies]].

**THE LESSON: the standing question — *what supplies the value under test?* — has to be asked of a
gate's REFERENCE, not only of its subject, and a doc comment claiming otherwise is not evidence.**
The bracket-walk gate logged every abscissa `cap_free` probed and asserted the ratio off the log,
and its own comment said this measured the constant *"through the behaviour rather than restating
it"*. It asserted `log[k] == prev * CAP_GROW` — the constant under test. A mutation setting
`CAP_GROW = 1.0/0.85` (`_sched_fuel`'s own shrink, the likeliest wrong neighbour) **SURVIVED all 17
green gates**: the bar was the walk's self-consistency with the constant, never the constant's
value. Repaired to compare against Python's literal transcribed from `engine.py`, plus a compounded
magnitude bar — `(1/0.9)^60 = 557.4` against `(1/0.85)^60 = 1.7e4` — because the one-sided
*"> 500x"* line the gate already had passes every growing constant.

**Why to apply it:** this is the fourth consecutive slice to ship one instance of the
agrees-with-itself defect class, and it is the first where the pre-flight's own standing item was in
front of me while I wrote the gate. Naming the item does not discharge it. The only thing that found
it was mutating the slice's own source and scoring the result against a prediction.

**How to apply it:** for every gate, write down the two values it compares and where each came
from. If either traces back to the code under test, the gate is void however behavioural it looks.
And run the mutation sweep on constants, not only on bodies — a constant is exactly the thing a
self-referential bar cannot see.

## Two mispredictions that were the MUTATION's fault, not the gate's

* One mutation was mislabelled: it deleted the coordinate refusal where the label said *sunk below
  the entry test*, so it tested nothing about placement. Re-run properly (the assert MOVED, by
  splicing the real text out of the file rather than retyping it) it was killed.
* One never applied at all — a `PATCH-MISS` from retyping a message containing a line
  continuation. **A mutation that does not apply reports as a misprediction and looks like a
  finding**; print the match count and treat zero as an instrument failure, never as a result.

**AND THE SAME DEFECT HIT A SECOND TOOL IN THE SAME STEP, WHICH IS THE STRONGER FORM.** The script
that repaired the test file guarded itself with `"drive(&m" not in t` — a substring of the
replacement `forced_release_drive(&m)` it was checking for — so it aborted before writing, and the
test run that followed reported **the previous run's result exactly**. A patch that does not apply
does not look like nothing happened; it looks like the last measurement, which is the shape that
does not prompt a second look. Guard on a regex with word boundaries, print the substitution counts,
and treat an unchanged file as a failure.

## A survivor that is CORRECT, and the comment defending it was wrong

Forwarding the fuel limiters unchanged instead of with the resolved clocks survived. That is right:
the parent re-resolves both with the same `or_else`, so the forwarding is a measured no-op — the
same category as `_shared_rig`'s carry. The code comment I had written claimed the parent *"would
march a different plant"*, which is false. **A survivor is a question, not a verdict: read the
parent before calling it a blind spot or a bug.**

## The other finding: a tripwire class nobody had named

The 14 → 18 arrival was measured slice AD's way (apply, fix the lib, count what is still red) and
came back **four test-target sites where the crate's comments say two**: two `E0063` initializer
literals in slice AB's and AC's cells files, and **two `E0027` exhaustive destructurings in slice
AE's files that nothing calls tripwires** — AE's own gate describes its destructuring firing without
ever naming the class. AD's *"the two width tripwires"* sentence has been stale since AE landed and
was copied verbatim into both AB's and AC's doc comments. [[rust-port-slice-af-preflight]]'s own
lesson, turned on the crate: check the ROW, not just whether the correction exists somewhere.

Also fixed: `applied_reference.rs` named `cross_split.rs`'s `CoordScope`, a type that has never
existed — `git log -S` returns only the commit that wrote the sentence — and credited it with the
opposite of what the type it meant (`GovScope`) documents. Urgent because this step introduces the
crate's first real `CoordScope`, so the stale name would have resolved to a live type.
