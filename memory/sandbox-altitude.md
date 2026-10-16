---
name: sandbox-altitude
description: "Controls view altitude knob (plan § 27, 2026-10-16) — process lessons: a step-checked knob limit is only valid at the flight it was measured at; split capture vs flown flight; an empty-vs-empty compare reads IDENTICAL"
metadata:
  node_type: memory
  type: project
  originSessionId: abf24555-fe89-4ee4-90ca-bde1797c76e3
  modified: 2026-10-16T18:30:07.810Z
---

Built 2026-10-16: the *Flight* fieldset flies the two-shaft Controls engine (T0/p0 in `ControlsSettings`, Mach fixed),
hardware captured at the DESIGN flight, every solve + the march at the flight FLOWN. Detail: `docs/plans/sandbox-plan.md` § 27.

**Lessons (process):**
- A knob limit set by a STEP CHECK at one operating condition does not carry to another: the march's clock (design-flight
  spool τ) is fixed, so denser air makes the loop stiffer. Re-run the step check at the box's corners BEFORE choosing slider
  ranges, then verify AT the limits a rule implies (the density rule left 14/720 cold-day runs above the bar — disclosed).
- A similarity test at ×½/×2 was bit-exact (powers of two) and could not see a hardware RE-CAPTURE at the flown flight
  (same corrected engine). The discriminator was the design throttle sitting at design speed ONLY at the design flight —
  red-checked by mutating `fd` back to the flown flight (4 of 6 tests failed).
- `cmp` of two EMPTY outputs printed IDENTICAL (a bad path expanded to nothing) — check line counts before trusting a
  diff. And `python -` is the Store stub that hangs (see [[windows-tooling-file-hazards]]) — it ate a 10-min timeout.
- A new settings FIELD broke the gate's build: `tests/sandbox_controls.rs` spells the struct out in full (round-trip
  test). After adding a field, `cargo test --no-run` before the gate. A background `…; echo exit $?` reported 0 for that
  failed gate — read the gate's own result lines, never the wrapper's exit.

**Why:** these are the traps that would have shipped a wrong "unchanged" or a silently mis-wired knob.
**How to apply:** any new flight/condition knob on a transient view: split constructor vs method flights, re-step-check,
and gate with a discriminator a re-capture cannot pass. Related: [[sandbox-direction]], [[rung95-governor]].
