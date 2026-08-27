---
name: rust-port-slice-z-step2
description: "Slice Z step 2 — a shared scope struct silently ignores a field the junior rung has no parameter for, and the route witness that caught it twice could not see the third case"
metadata: 
  node_type: memory
  type: project
  originSessionId: 42ef8dd0-6a5c-41e2-9ad4-9339dddd8cb5
  modified: 2026-08-27T08:49:19.307Z
---

Slice Z step 2 (2026-08-27) ported both marches of rungs 66/67 and all 20 method bodies —
2 576 Rust lines against 1 496 Python, a **1.72× expansion**, which corrects § 5.24 (ii)'s
labelled ~3 000-line estimate downward and comes in under slice W's 2.06×.

**The leading lesson is the mirror of [[rust-port-slice-z-step1]]'s, and this half is the
PORT's fault rather than the source's.** Python adds `_stator_march`'s isolation parameters
ONE PER RUNG, so `m65._stator_march(..., lag=…)` is a `TypeError`. The port's `MarchScope` is
one struct shared by every rung — a deliberate choice, so the cell's signature opens once
instead of four times — so the same call SUCCEEDS and the field is discarded.

**Why:** it bit three of my own typed route bars, and the third bit past the witness that
caught the first two. `key_count` is MANY-TO-ONE: **14** covers the bare march, rung 46's
unlagged redline, rung 47's lagged one, rungs 48–51's legs and rung 64's instantaneous valve;
**16** covers rung 52's clip state and rung 65's valve state, two DIFFERENT pairs of keys.
So a `14 == 14` says only *neither side carried a march state*, and the bar passed on both
sides of a comparison whose floats disagreed. A ROUTE WITNESS IS NOT A RUNG WITNESS.

**And the same conflation cost the write-up a wrong TALLY, caught in review after the
commit.** I published "P2's six reduce arms gated six-for-six" while calling the
`LeverArm::default()` gate the rung-64 arm. It is not — that machine has **no limiter at
all**, which is rung 43/57's; rung 64 is the floored INSTANTANEOUS valve, and the untested
combination (floored + unlagged + no clock) is the only path that runs `r64_solve_b`,
`ForcedBleed` and `b_of` at every closure. Step 1's own comment had hedged it correctly and
step 2 hardened the loose half into a count.

**How to apply:**
- When a port collapses a per-rung signature into one shared struct, ask what the SOURCE does
  when a junior is handed a senior's field. If the answer is "refuses", write it down where a
  caller reads it — the port will not refuse and no value key will say so.
- Never type a count bar. Three in one file, three wrong. See
  [[rust-port-guessed-census-bars]].
- **Do not turn a predecessor's hedge into a count.** If an earlier step wrote "lands two
  rungs lower (X / Y)", that is two candidates, not one — resolve it by building the machine
  and reading which, before publishing a tally.
- **Compare the port to the source BEFORE writing a gate.** Every gate a port step can write
  is either a reduce arm (agrees by dispatch) or self-referential, so neither catches a march
  that is uniformly wrong. A throwaway bit-emitting probe pair over both marches and all ten
  readers was 785 keys bit-exact on the first run; the gates came after.
- **Adding an enum variant breaks the exhaustive matches loudly and leaves the
  `_ => panic!()` arms SILENT — and a silent one is a NARROWING**, stricter than the source,
  with every suite green. Audit each wildcard by hand and ask what Python does; here four
  arms split two–two. Spell the survivors as named arms so the next variant breaks the build.
- A branch that fires on NO shipped grid (here `joint_ic_corners`'s caught-panic arm, 0 of 8)
  cannot be left asserted in prose — the oracle runs the suites' grid and will not reach it
  either. Exhibit it directly; measure the constant on both languages.

Related: [[rust-port-slice-z-preflight]], [[rust-port-slice-z-step1]], [[rust-port-slice-m]],
[[rust-port-copy-vs-rederivation]].
