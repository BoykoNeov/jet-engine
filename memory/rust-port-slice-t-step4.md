---
name: rust-port-slice-t-step4
description: "Slice T step 4 — an injection matching TWICE applies nothing and reports green; and \"two files disagree about one march\" can mean two different gases"
metadata: 
  node_type: memory
  type: project
  originSessionId: 65b2edbc-b84c-49b9-a164-8f7c62324131
  modified: 2026-08-20T07:27:24.838Z
---

Slice T step 4 (2026-08-20) shipped `topping_oracle.rs` — 1 729 keys bit-exact vs PyPy first run,
over rungs 46/47/48's three grids, plus a CPython arm and two rule gates. Two process lessons.

**An injection whose pattern matches TWICE applies NOTHING, and the run still says green.** The
RK4 accumulate line in `integrate_fuel_lagged` is byte-identical to rung 52's in
`integrate_fuel_asym`, so the text substitution hit both and was refused. Four suites then reported
`ok` — which reads exactly like "the defect survives and nothing caught it". The `count == 1`
assert in the harness was the only thing separating those two stories.

**Why:** this port has now had three injections that could not have moved anything. The previous
two were caught in the write-up, afterwards; this one was caught up front only because the harness
asserted its own precondition.

**How to apply:** in an injection harness, always assert the match COUNT before substituting, and
when a twin exists target by LINE with the twin asserted untouched. A green suite after an
injection is not evidence until you have evidence the injection landed. See
[[rust-port-slice-s-step3]] and [[rust-port-slice-r-step3]].

**"Two files disagree about the same march" can mean they are running different GASES.** The
slice's own § 5.17 finding 6 measured the bare `Tt4` peak on the CPG gas, found neither of two
quoted comments matching, and called both stale. Measured on both gases: rung 46's `~1645` is
CORRECT (its gates run `Gas.thermally_perfect()`, peaks 1641-1651 K) and only rung 47's `~1670` is
wrong (CPG peaks 1690-1703 K).

**Why:** *a census is a property of the grid* has cost this port a slice several times, but every
prior instance meant the cell list. This is the first where "the grid" meant the GAS — and the
tell that looked like proof (two files, one march, two numbers) was the thing that was false.

**How to apply:** before deciding which of two disagreeing figures is stale, check they describe
the same configuration — gas included. Then correct the wrong one and write the RIGHT one's
justification beside it, so the next reader does not "fix" it back. See [[rust-port-slice-t-step1]]
and [[rung63-fuel-bleed]].

One more, smaller: the CPython arm's relative bar failed on `overshoot`, which is
`Tt4_peak_top − Tt4_max` pinned to machine zero — a relative deviation on a quantity meant to be
zero is a ratio of two rounding errors. Same class as slice S's Newton residuals, and found only by
running the arm. See [[rust-port-slice-s-step4]].
