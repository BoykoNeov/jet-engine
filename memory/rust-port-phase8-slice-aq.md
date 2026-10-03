---
name: rust-port-phase8-slice-aq
description: "Phase 8 slice AQ (2026-10-03): main.py rungs 64–84's 21 panels byte-exact (PORTED=85, every print_* panel) on the first compile — a scratch checker outside the repo ran only the new panels, and a reader NAME is not unique in engine.py"
metadata:
  node_type: memory
  type: project
  originSessionId: f3d80295-9d87-4124-90e8-2fcc32d03141
  modified: 2026-10-03T13:19:14.671Z
---

Slice AQ of phase 8, plan § 8.8. Three files, three commits, each green at its own prefix:
`rust/src/panels/cascades.rs` (64–71), `actuator.rs` (72–77), `readers.rs` (78–84). All 21
panels were byte-exact on their first compile. `PORTED = 85`, and only the `plot_ts_diagram`
line is left, owed to slice AR. No shim change.

**Process lessons:**
- **A scratch checker outside the repo paid for itself many times over.** A full `cli_golden`
  run replays all the earlier panels first (~2.5–4 min). Instead, a tiny crate at
  `W:\temp\claude\aq_probe\` takes `turbojet` as a path dependency, runs `PANELS[from..to]`, and
  diffs each panel's segment by bytes. It can also dump each panel's output for a mutant diff.
  The repo tree stays clean. Reuse it for slice AR.
- **The step-size table had THREE values this time (0.005 / 0.0025 / 0.002)**, and the 0.002
  cases are the Jacobian readers. Some panels pass `ds` explicitly over a default of 0.0025,
  and one call (`detector_sensitivity()`) passes nothing and takes 0.0025. Grep every `def`
  first.
- **A reader NAME is not unique in `engine.py`.** `authority_ceiling` (rungs 54/64) and
  `split_gains` (70/80) are each defined twice with different defaults. A grep that stops at the
  first `def` reads the wrong signature. Count the definitions before you trust one.
- **Commit a mid-slice checkpoint from the index when the tree has moved on.** Checkpoint 2's
  `mod.rs` was built with `git hash-object -w` and `git update-index --cacheinfo`, so the commit
  matched the 78-panel state that `cli_golden` had passed. The unregistered file stayed out.
- **A heredoc full of Rust in Git Bash can fail to parse and write NOTHING** (an apostrophe or
  backtick inside it). Write long code to a scratch file and merge it with a small Python script.

**How to apply (AR):** keep the checker; byte-diff whole segments; read defaults off the
right `def`.
Related: [[rust-port-phase8-slice-ap]], [[rust-port-phase8-slice-ao]], [[rust-port-status]].
