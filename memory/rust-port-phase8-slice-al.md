---
name: rust-port-phase8-slice-al
description: "Phase 8 slice AL (2026-10-02): main.py rungs 7–24's 18 panels byte-exact (PORTED=26) — a surviving injection was an EQUIVALENT mutant (field symmetry), not a blind gate"
metadata:
  type: project
---

Slice AL of phase 8, plan § 8.3. `rust/src/panels/nox.rs` (rungs 7–14) + `mixing.rs` (15–24,
with `main.py`'s `_mean_grad_sq`/`_j_opt_from`); `PORTED = 26`, all byte-exact on the first
compile that built. Pre-flight per § 8.2: containers + `None` added to `dump_pyfmt.py` (15/15),
`Option<T>`/`PyRaw` in the shim; rung 16's `except AssertionError` → `nox::try_primary_aft`.

**Process lessons:**
- **A surviving injection may be an EQUIVALENT mutant — check before calling the gate blind.**
  Periodic → clamped z-boundary in `_mean_grad_sq` PASSED because the field is symmetric about
  S/2 with period S (`xi[nz-1] == xi[0]`). A non-equivalent one (wall difference `(ip-im)·dy` →
  `2·dy`) FAILED at the 4th decimal. Pick injections that MUST move the value at the printed digits.
- **A blanket `impl PyFormat for &T`** replaced the per-type `&str` impl the moment panels passed
  `&f64` from iterators — design the shim for references from the start.

**How to apply (AM–AQ):** same recipe; watch `cli_golden.rs` run time (≈100–135 s at 26 steps;
later rungs are the expensive half of the 1 642 s PyPy run).
Related: [[rust-port-phase8-slice-ak]], [[rust-port-status]].
