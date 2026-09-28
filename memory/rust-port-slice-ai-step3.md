---
name: rust-port-slice-ai-step3
description: "Slice AI step 3 (rung 79's coord_march/coord_forced/with_probe) — cargo stops at the first failing test BINARY, so an injection sweep over several --test targets reports only the first catcher; run it with --no-fail-fast before claiming a second one"
metadata:
  node_type: memory
  type: project
---

Slice AI step 3 shipped 2026-09-28 (plan `W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md`
§ 5.33.3): `with_probe`, `coord_march` (+ `coord_march_logged`), `forced_cap`, `coord_forced` in
`rust/src/state_coordinate.rs`, gated by `rust/tests/slice_ai_march.rs` (4 gates: both readings bit for
bit, the counters read in the same test, the probe log as an FNV-1a digest, a manufactured probe NEST
and an unwind). Green on the first run. Sweep: 16 injections, 16 of 16 predicted.

**THE LESSON: `cargo test --test a --test b --test c` STOPS AT THE FIRST BINARY THAT FAILS.** The
first sweep ran without `--no-fail-fast`, and item L's injection (J16) came back killed by
`slice_ai_cells` alone — the new post-march readback in `slice_ai_march`, written precisely to be its
second catcher, had never RUN. Re-run with `--no-fail-fast`: both catch it. Same for J15, whose
"killed by the counters only" needed the panic message to confirm which assert fired.

**Why:** a sweep's "KILLED by X" is only as wide as the set of gates that executed; a fail-fast
harness silently narrows that set to everything before the first failure, in binary order.

**How to apply:** every injection sweep over more than one test binary passes `--no-fail-fast` and
logs the panic lines, so the report names every gate that caught the defect, not just the first.
The sweep script is `W:\temp\claude\slice-ai-step3\inject.py`.

Also recorded: J12 (the REFERENCE march run at incidence) and J14 (`w_shipped` from the forced `phi`
solve) both survive — on this rig the march is value-invisible to the coordinate, and the forced `phi`
solve is a COPY of `_surge_fuel`, so each comparison cannot tell its reference from its instrument.
Only `d_forced` sees the coordinate in §§ 5–5.2. `forced_cap`'s SLACK arm is unreached (all 10 points
bind) — owed to step 5's `:21301` refusal gate. Related: [[rust-port-slice-ai-step2]],
[[rust-port-copy-vs-rederivation]].
