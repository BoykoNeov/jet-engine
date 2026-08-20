---
name: rust-port-slice-u-preflight
description: "Slice U's pre-registration — an exhaustive combination sweep found three shipped asserts no input can reach, and two of my own claims were wrong until measured"
metadata: 
  node_type: memory
  type: project
  originSessionId: e65a6e1e-d807-4d1c-b0d3-374a76fc486b
  modified: 2026-08-20T09:17:59.755Z
---

Slice U (rungs 49/50/51/52 gates, the last of phase 6) was pre-registered on 2026-08-20 off four
probes. Three process lessons, all of them about claims I wrote before measuring:

**A refusal census must sweep the ARMING COMBINATIONS, not read the asserts.** Reading said all 14
of the marcher's refusals were present in Rust — true, and useless on its own. Sweeping all 255
combinations of the eight limiter keywords through the degenerate object found that rungs
50/51/52's own `lp_disabled` refusals fire on **zero** of them: arming `s_off`/`tau_rel`/`lag`
requires an armed leg, and the `accel`/`surge` refusals precede them. Three asserts in shipped
Python that no input can reach, and four gates named for four different rungs all firing ONE
assert. A read would never have produced that.

**Grepping a format string is not finding a reader.** I wrote that `main.py` prints the one-ulp
`both_edges_inside_ramp` cell. The format string was there; the sweep feeding it is LP-only, and
the cell is HP. Find the CALL, not the print.

**A consistent offset between a quoted number and a measured one is more often a different formula
than staleness** — so search the denominators before shipping a doc correction. Here four
alternatives were tried and none reproduced both figures, so the correction stood, but the search
is what makes it safe to ship. [[rung63-fuel-bleed]]'s lesson, third outing in this port.

Also: the probe written to honour [[rust-port-slice-s-step4]]'s "run the suites' OWN grids" lesson
used the wrong settle time for two of the four rung files on its first run. The numbers happened
to be settle-invariant, which is exactly why it would not have announced itself.

See `docs/plans/todo-rust-port.md` § 5.18. Related: [[rust-port-slice-t-step2]],
[[rust-port-oracle-cannot-see-a-missing-gate]], [[rust-port-copy-vs-rederivation]].
