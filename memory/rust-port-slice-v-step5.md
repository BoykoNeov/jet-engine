---
name: rust-port-slice-v-step5
description: "Slice V step 5 — a gate that manufactures the bug needs assertions that read nothing on disk, and a prediction too narrow was corrected by the anti-vacuity assert it forgot"
metadata: 
  node_type: memory
  type: project
  originSessionId: 09c79c64-03cd-47b1-952e-2cea12ecb6db
  modified: 2026-08-26T08:07:48.303Z
---

Slice V step 5 (2026-08-26) shipped `rust/tests/slice_v_dispatch.rs` — the manufactured carrier
gate P5 had owed since the slice was pre-registered — plus a `--dispatch` mode on
`inject_slice_v.py` and a COMMITTED evidence emitter. Crate: 102 suites, 921 passed, 0 failed,
0 ignored; `git diff -- rust/src/` empty. **Slice V is complete; slice W (rung 62) is next.**

**The process lesson, and it is about what a gate READS, not what it asserts.** Four instruments
in this slice look at the same carrier and only one of them is a gate on it: the 59 ported gates
are relational (0/59), the carrier witness proves the mechanism not the need, the oracle catches
it at 87 keys but is a GOLDEN — regenerable. So when writing a manufactured-bug gate, ask of each
assertion *what file does this need?* Two of the six here pin their clean column against the
committed TSV by key (worth having: it also fails if the golden is ever regenerated against buggy
code). The other **four read nothing on disk** — they compute both branches in Rust and assert
the difference — and those are the ones that survive a regenerated golden. **Record the split per
test in the evidence, don't claim it in prose.**

**And the sharpest form of a finding often needs no bar.** The percentage (15.4 % on
`margin_min_lp`) is the plan's number, but the assertion with no tolerance in it is: under the
scoped port, `surge_margin` reads a SCHEDULED machine **bit-for-bit as an UNSTATORED one**, same
bit pattern for all three scheduled armings. Look for that statement before reaching for a band.

**Two more, both caught rather than shipped:**
- **The anti-vacuity gate must cover the arming the other gates never use.** The advisor blocked
  the plan on it before a line existed: the constructor applies a CONSTANT setting on a line no
  scheduled arming reaches, so a botched hand-rebuild is invisible on three armings and then
  hollows out the constant-arming negative control. Pin the rebuild on ALL cells, not the one the
  headline uses. Same shape as [[rust-port-slice-u-step4]].
- **A prediction that was too NARROW was corrected by the anti-vacuity assert I forgot I had
  written.** I2 was predicted to fail 3 gates and failed 4: dropping the HP arm makes that arming
  effectively unarmed, so the *exactly 3 of 4 armings must move* half of a different test fired.
  An anti-vacuity clause is a detector in its own right — count it when predicting.

**A gate that MANUFACTURES a bug is itself code, and needs the same did-it-move treatment.**
All six injections patched `rust/src/` — none touched the two wrapper cells the gate itself
carries. The advisor caught it after they were green. A wrapper restoring only ONE of the two maps
is a partial carrier bug in the INSTRUMENT; booked as a fourth harness mode (`--self`) so it
re-runs, and both variants fire 4 of 6. The advisor's own prediction (*only the HP armings would
move*) was refuted by the measurement — the two spools are coupled through the shaft state.

**And a failed `io.open(path, "w")` STILL TRUNCATES.** A bad `newline=` argument raised after the
open had already emptied the 519-line test file to 0 bytes, and the `finally` hit the same error,
so the guard that makes injections safe destroyed what it was guarding. Write a `.bak` BEFORE
patching, never rely on holding the text in memory. The other half of
[[windows-tooling-file-hazards]]: that one is *the write never lands*, this is *the truncation
lands and the write does not*.

And [[rust-port-phase7-preflight]]'s rule caught its own author a third time in this slice: the
doc-comment bar-margin table typed `1.3e-13 (3 %)` off a hand calculation where the test's own
`println!` said **1.26e-12 (25 %)**. See also [[rust-port-slice-v-step4]], [[rust-port-slice-v-step3]].
