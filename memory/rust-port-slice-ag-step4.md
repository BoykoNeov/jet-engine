---
name: rust-port-slice-ag-step4
description: "Slice AG step 4 (rung 76's cap) — an enumeration of what a REFUSAL protects expires at the step that adds its caller, so a defect I proved silent panicked on a refusal shipped three steps early"
metadata: 
  node_type: memory
  type: project
  originSessionId: 43c63c08-3f04-4e75-9793-b06ca343a261
  modified: 2026-09-07T03:46:12.857Z
---

Slice AG step 4, 2026-09-07 (§ 5.31.4). Rung 76's cap ported into
`M:\claud_projects\jet engine\rust\src\sensed_cap.rs` (305 → 578 lines): `cap_march`, `CapScope`
(`_with_cap`), `accel_for`, `c_at`, plus the two carried Python defaults `ACCEL_SCHEDULE_N = 13`
and `C_AT_REL = 1e-6`. No gate file — step 3's precedent. **1 422 keys, `Rust == PyPy` bit for bit,
both key sets equal, no port fix.** Sweep 14 injections, 9 killed, 5 survived, **14 of 14 verdicts
right and three mechanisms/counts wrong** — and the three misses are the whole content.

**THE LESSON: *what reads this?* has a sibling — *what REFUSES this, and when was that enumeration
last taken?*** `cap_march`'s one structural difference from rung 75's march is `accel: Some(accel)`.
I wrote in its doc comment that dropping it is *silent in the worst available way* — the sensed arm
returns the solve arm's floats, which IS the reduce answer, so every reduce gate keeps passing.
**Measured: it PANICS**, on `r76_integrate_fuel`'s third refusal (*`sensed` re-reads rung 48's
schedule, so there must BE one*) — **ported at step 1, three steps before this step wrote the first
caller that could reach it with no schedule.** The claim is right about the `solve` arm and wrong
about the arm the rung exists for. Repaired ADDITIVELY, both domains named.

**Why:** this is the exact mirror of [[rust-port-slice-ag-step3]], which found a plain write I
proved inert with the PREVIOUS rung's evidence and which panicked on a refusal born at THIS rung.
Same defect class, opposite direction on the timeline: a readers enumeration expires at the rung
that adds a reader, and a refusal's coverage enumeration expires at the STEP that adds its caller.
Both answer to [[rust-port-slice-aa-steps2345]]'s *ask what reads a thing, never wait for a
failure*, asked at the right boundary.

**How to apply:** before writing "dropping this is silent", grep the file being edited for asserts
that the deletion would reach — including ones the port shipped in an earlier step of the same
slice, which is precisely the population a step-boundary argument forgets.

**SECOND: neither wrong restore policy on a reload guard moves a single VALUE, and one moves ONE
key in 1 422.** `CapScope` restores the previous law; `c_at`'s two state guards restore `None` —
two policies in one file. Injecting *restore the constant `"solve"`* kills **exactly one** key
(predicted two), because the drive's marched machine already rests at `"solve"`, so the injected
constant coincides with the displaced value at 18 of 19 sites. Injecting *restore nothing* kills 19
— **all of them tag reads, none a cap value** — because every caller sets the law immediately
before reading it. So driving the guard end to end through `_cap_fuel`, the natural gate shape, is
blind to both. A gate must read the field WITHOUT setting it first, on a machine whose resting law
is not the one the guard arms. Booked into the doc as a requirement on step 6.

**THIRD: the package's one expression-first `max` is UNDRIVABLE, and the step that discharges the
obligation is the step that proves it cannot be checked.** Step 1 booked `_c_at`'s
`dw = rel * max(w, 1e-9)` forward by name, with the NaN-faithful spelling
`if 1e-9 > w { 1e-9 } else { w }`. Discharged — and measured: all four sub-hinge `w` values abort
in the plant (the hinge is `1e-9`, the plant refuses fuel below ~1e-2), and replacing the fold with
`w.max(1e-9)` survives at 0 of 1 422. A **defence with no reader**, disclosed at the step that
shipped it rather than found later.

Also measured: `accel_for`'s hook dispatch is a RULE, not a difference — the direct rung-75 pointer
moves 0 of 1 422, because a schedule is read off EQUILIBRIA, and so do its `tau_rel` and `tau_att`
(a lag is a march parameter), while the identical `tau_rel` mutation inside `cap_march` kills 688.
And the pre-flight's published *341 of 341* against this drive's *333* is ONE measurement on TWO
populations — the probe compared an eleven-key tuple, `Tt4` alone gives 333, and the eight agreeing
points are the first eight, where `mf` has already parted. `max|ΔTt4| = 10.254967637311893`
reproduces the pre-flight to every digit through a DIFFERENT entry point, confirming the rig rather
than assuming it. The step shipped THIN (4 methods, 53 Python lines) and was deliberately not merged
with step 5, because merging would settle P7 — the 7-vs-6 step-count prediction — by convenience.

**FOURTH, and it is a PROCESS defect not a port one: the citation guard step 1 built against
documentation decay had been RED since step 2, and neither step ran it.** Step 4's nine new
`engine.py` citations forced a re-bless; the run printed nine unblessed anchors and only EIGHT are
step 4's. The ninth, `engine.py:18694`, was cited in step 2's own commit and never blessed, and the
census constants were still step 1's `9/50/38` against a live `10/59/47` — so two of that guard's
four asserts had been failing for two steps while both shipped green. Steps 2 and 3 each ran
`cargo test --release` and nothing else. **The repair is procedural: a step that writes a doc
comment citing `engine.py` has changed a gated input, so its gate is `pytest`, not `cargo test`.**
