---
name: rust-port-phase8-slice-an
description: "Phase 8 slice AN (2026-10-02): main.py rungs 38–45's 8 panels byte-exact (PORTED=47) — a planted error can be invisible on a row where two quantities happen to coincide"
metadata:
  type: project
---

Slice AN of phase 8, plan § 8.5. `rust/src/panels/twospool.rs` (rungs 38–45); `PORTED = 47`,
byte-exact on the first compile of each half. No shim change; Python's `setattr` efficiency
perturbations go through `core_mut()` and are restored, as the rung-38/39 gate ports already did.

**Process lesson:**
- **A planted error proves the gate only on rows where it MOVES something.** Rung 42's injection
  (`dL/gL` → `dL/gH`) was equivalent on the 1500 K row, where both gaps are equal, and was caught
  one row later. Pick an injection that differs on most rows, and read WHICH row failed: if it
  had been the last row of a table, a shorter table would have let it pass.
- **Read caught panics with `--nocapture`.** libtest swallows a passing test's stderr, so slice
  AM's "read stderr" rule needs the flag (or the shipped binary). The only new caught panic is
  rung 38's nozzle-unchoke at 600 K, which matches its printed label.

**How to apply (AO–AQ):** same two checks per slice.
Related: [[rust-port-phase8-slice-am]], [[rust-port-status]].
