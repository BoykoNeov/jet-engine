---
name: rust-port-phase8-slice-am
description: "Phase 8 slice AM (2026-10-02): main.py rungs 25–37's 13 panels byte-exact (PORTED=39) — a catch-all except printed a cause it never checked (rung 33's SUB-IDLE rows are an equilibrium-solver failure)"
metadata:
  node_type: memory
  type: project
  originSessionId: 54db8690-f191-44c0-888d-14cc2c957667
  modified: 2026-10-02T09:54:16.438Z
---

Slice AM of phase 8, plan § 8.4. `rust/src/panels/marches.rs` (rungs 25–30) +
`offdesign.rs` (31–37); `PORTED = 39`, byte-exact on the FIRST compile. No shim change: every
call hit an already-gated entry point; the only work was mapping Python keyword defaults onto
Rust positional arguments.

**Process lesson:**
- **Read the CAUGHT exception's message, not the label the code prints for it.** Rung 33's
  `except AssertionError` prints "net thrust <= 0: below thrust-neutral idle", but at Tt4 = 440/420
  the assertion is "equilibrium Newton did not converge" — confirmed under PyPy, not just in Rust.
  The bytes match either way, so the byte gate cannot see it; stderr of the `catch_unwind` run
  did. Booked OPEN for the user as a `main.py` honesty item (§ 8.4 (i)), not changed.

**How to apply (AN–AQ):** after each `cli_golden` run, read its stderr for caught panics and
check each one is the failure the panel's text claims.
Related: [[rust-port-phase8-slice-al]], [[rust-port-status]].
