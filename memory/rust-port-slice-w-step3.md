---
name: rust-port-slice-w-step3
description: "Slice W step 3 — five of six injections pass all 88 ported gates, and the probe that measured it was wrong five times, every one a zero nobody had looked at"
metadata: 
  node_type: memory
  type: project
  originSessionId: 4773ae7c-4013-4311-8fa3-564477a7ba9c
  modified: 2026-08-26T11:55:35.177Z
---

Slice W step 3 of the Rust port (rungs 62/63), 2026-08-26. The 88 ported gates are green
(58 + 30 against Python's 88 collected, a complete name bijection), and the substance of the step
is what green does NOT establish: **five of six injected defects pass all 88**, including two that
move 312 and 151 probe keys.

**EVERY WAY AN INSTRUMENT CAN REPORT A ZERO NOBODY MEASURED — FIVE OF THEM IN ONE PROBE.** In
order: (1) the header claimed the key set was "what the 88 gates compare" while dumping a different
reader on a different machine, with nine readers missing — the suites' headline calls
`loop_decomposition` on an ARMED core, the probe called `marginal_loop` on a bare one; (2) the
section named for the floor dumped only the leg-FREE cells and the input grid echoed back, so it
held no floor-armed reading at all; (3) the witness block ran LAST, so the one injection that
panics reported "0 witness keys moved" having emitted none — 444 absent; (4) the coherence check
written to catch exactly that filtered on `status == "OK"`, which excluded the only row with
`caught > 0`, so it examined ZERO rows and printed success; (5) the witness workload ran
`equilibrium` at Tt4 = 1200 where the pre-registered table was measured at 1000, so ONE machine's
row reproduced exactly and another missed by 3 — which reads as a port defect.
**Why:** each failure is invisible in the output, because a zero from a blind instrument and a zero
from an inert defect are the same character. **How to apply:** make the instrument prove it can
see. Emit a `guard/<section>` flag per section, run witnesses FIRST, carry a detector on a path the
subject cannot reach, and make the coherence check print HOW MANY ROWS IT EXAMINED.
See [[rust-port-slice-s-step4]], [[rust-port-slice-v-step2]], [[rust-port-slice-n-step4]].

**A PRE-REGISTERED GATE CAN NAME THE WRONG INSTRUMENT.** § 5.21 (v)/P4 predicted that `_powers`
re-reading `b_of` would be visible ONLY to a dispatch gate counting reduced-vs-bled. Measured: all
eight such counters DO NOT MOVE — the two spellings agree at every call, not merely where the valve
is shut — and the only thing that moves is `b_of`'s CALL COUNT (409 → 818). The verdict survived
and its instrument did not. **How to apply:** when a plan pre-registers *which* instrument will see
a defect, inject the defect and check that instrument moves, before building the gate on it.

**AND A GATE CAN BE BLIND BECAUSE THE SUITE NEVER BUILDS THE DISCRIMINATING INPUT.** Spreading
`R62_FUEL` from `..R43` drops a floor-RESOLVING wrapper — but the resolution is the identity on a
`Floor::Phi`, and both suites build only `Phi`, never `Floor::Incidence`. So "0 of 88 catch it" is
a fact about the suites' inputs, not about the defect. A deliberately un-ported `Incidence` cell
PANICS under the injection, and that panic is the only evidence in 957 keys that the channel exists.

**TWO SMALLER ONES.** A gate failed on the message it exists to accept: `_isolating`'s two refusals
unwind DIFFERENT payload types (`String` from an interpolated `assert!`, `&'static str` from a
literal one), and downcasting to `String` alone read a matched refusal as the empty string. And
`cargo test … | tail -60` returned **exit 0 over a seven-error build failure** — a pipeline's status
is the last command's; derive a gate from the LOG BODY and check the decomposition, never a
remembered total (this section's own steps-1+2 record understates its gate by 5 suites / 12 tests).

Gate: 105 result lines, 1010 passed, 0 failed, 0 ignored; 922 + 58 + 30 = 1010 decomposes exactly.
Remaining: step 4 (the oracle), step 5 (dispatch + manufactured-bug gates — which must count
`b_of_calls` and build a `Floor::Incidence`, per the two corrections above).
See [[rust-port-slice-w]], [[rust-port-decided]].
