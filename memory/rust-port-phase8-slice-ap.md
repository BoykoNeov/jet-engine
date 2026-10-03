---
name: rust-port-phase8-slice-ap
description: "Phase 8 slice AP (2026-10-03): main.py rungs 53–63's 10 panels byte-exact (PORTED=64) on the first compile of each half — the step size is per CALL SITE, and the first line a planted error trips can be its weakest witness"
metadata:
  node_type: memory
  type: project
  originSessionId: f543f13d-8d4f-49cf-8c19-246086470784
  modified: 2026-10-03T11:08:51.321Z
---

Slice AP of phase 8, plan § 8.7. `rust/src/panels/airflow.rs` (53–56, steady) and
`rust/src/panels/schedules.rs` (57–63, transient + bleed); **rung 59 has no panel**, so ten panels
and `PORTED = 64`. Byte-exact on the first compile of each half; no shim change.

**Process lessons:**
- **The march step `ds` is chosen per CALL SITE, off each reader's own signature** — rung 57's and
  62's readers default to 0.01, rung 58's composite and rung 63's retiming/dichotomy to 0.005, and
  rung 60's panel passes 0.01 over a 0.005 default. A per-panel or per-module choice would have
  been wrong in three of the six transient panels. Tabulate it before writing (the advisor did).
- **The first line a planted error trips can be its weakest witness.** Rung 60's `ds` injection
  failed on a TAUTOLOGY row whose printed value is rounding noise (`-2.2e-16` → `-8.9e-16`).
  Running the mutant binary and diffing the whole panel showed 9 of 33 lines moved, with the real
  physics 15 lines lower, and two physics lines unmoved. Diff the whole segment, not just the
  first failure, and diff in BYTES (the segment offsets are byte offsets; a `str` slice misaligns
  after the first non-ASCII character).
- **`format(True, '>8')` is `'       1'`** in Python — that is why the panels write `str(x)`; pass
  `x.py_str()`, never the bool, into a padded field.

**How to apply (AQ, rungs 64–84):** tabulate each reader's defaults first; plant the slice's own
trap as the injection; diff the mutant's whole segment by bytes.
Related: [[rust-port-phase8-slice-ao]], [[rust-port-phase8-slice-an]], [[rust-port-status]].
