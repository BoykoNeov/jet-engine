---
name: rust-port-phase8-slice-ao
description: "Phase 8 slice AO (2026-10-03): main.py rungs 46–52's 7 panels byte-exact (PORTED=54) on the first compile — read keyword defaults off the Python SIGNATURE, and pytest.ini injects -n so `-p no:xdist` breaks the run"
metadata:
  node_type: memory
  type: project
  originSessionId: e9770811-131d-4beb-9dd1-1e534cae5ab2
  modified: 2026-10-03T07:40:55.401Z
---

Slice AO of phase 8, plan § 8.6. `rust/src/panels/limiters.rs` (rungs 46–52); `PORTED = 54`,
byte-exact on the first compile and first run. No shim change. Rung 52's panel sits out of line in
`main.py` (after rung 56's) but `main()` calls it seventh — port in `main()`'s order, not file order.

**Process lessons:**
- **Read every keyword default off the Python SIGNATURE, not off the panel.** The panels omit
  `ds`, `n = 13`, `eps = (0.05, 0.01)`; `lag_sweep` has no `eps` at all, so `factorization_grid`
  inherits `lag_relief`'s default. One grep of the `def` lines settled all of them up front.
- **AN's injection lesson applied:** a planted error that moves EVERY value on a line (rung 48's
  `pt4 / pi_b` → `pt4`) failed on the first row read, all six numbers moved — no coincident row.
- **Tooling:** `pytest.ini` adds `-n … --dist load`, so `-p no:xdist` makes pytest refuse the
  ini's own arguments. Run the two Python guards plain (`pytest tests/test_rust_line_citations.py
  tests/test_claude_md_reference.py -q`).

**How to apply (AP–AQ):** same checks — signature defaults, an every-row injection, `--nocapture`.
Related: [[rust-port-phase8-slice-an]], [[rust-port-status]].
