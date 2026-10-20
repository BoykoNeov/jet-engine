---
name: rung105-decisions
description: "User scope decisions 2026-10-20 for rung 105 (A4, the HP turbine's NGV, a flow law replacing the choked pin): a REACTION knob (two rows in series, default R 0.3) + the old pin, on both engines and every off-design page — DEFAULT OFF (re-asked once cost 50–110x on Fly it and effect ≤0.4 % were measured); rung 33 double-root bug recorded, fix later"
metadata:
  node_type: memory
  type: project
  originSessionId: dda0c953-01cb-4b73-a8e4-c586c6990234
  modified: 2026-10-20T19:28:07.842Z
---

Picked 2026-10-20 over the afterburner re-check (rung 104's other seam), because the HP vane's answer decides whether
that re-check must be done twice.

**Probe G1 (`W:\temp\claude\rung105-probe\g1.txt`, to be archived under `docs/plans/scratch-archive/rung105/`):** the
vane chokes iff stage reaction R ≤ R* = 1 − (c*²/2 − KE4)/Δh0 (loss-independent: Tt const in the vane, M=1 a static-T
condition; repeating stage). One stage: one-shaft (Fly it, π_c 10) π_t 2.33–2.40 ⇒ R* 0.16–0.19 choked nozzle, 0.04–0.17
below choke; two-shaft rig π_HPT 2.30–2.35 ⇒ R* 0.17–0.20, below choke at low power under rung 104's ellipse π_HPT down
to 1.35 ⇒ R* < 0 (unchoked at ANY reaction). Published single-stage reaction 0.25–0.5 (engine patent US 11566562:
0.15–0.4, 0.25–0.33 preferred, 0.3 example; Cambridge 4A3 2023 exam: 30 % vs 50 %). ⇒ unchoked everywhere at a normal
reaction — a rung, not a negative. Real engines with choked HP vanes run π_stage 3–4; these are lightly loaded.

**Census (throwaway worktree, `a4` private):** 36 outside reads in 13 src files (turbine_cooling 8, customer_bleed 6,
shaft_offtake 4, turbine_law 3, spool 3, stage 2, combustor 2, bleed_transient 2, bleed 2, two_spool_transient,
map, fuel_transient, afterburner 1 each) + ~19 lines inside matcher.rs / two_spool.rs + 13 reads in 10 test files.

**User answers (AskUserQuestion, 2026-10-20):**
- Engines: **both, everywhere** — one-shaft (Fly it, its slam, rungs 31–37, stator/stage family) and two-shaft.
- Law: **a reaction knob** — vane and rotor as two rows in series, the design split set by reaction (0 impulse … 0.5);
  the choke EMERGES at low reaction. Plus the old pin as its own setting.
- Default reaction: **0.3**.
- Default: **ON everywhere** (CLI, tests, pages), older tests that move sorted as in rung 104, moved Python-oracle
  values retired by name.

**RE-ASKED 2026-10-20 once cost + effect were measured (prototype in throwaway worktree `W:\temp\claude\rung105-wt`,
probes `W:\temp\claude\rung105-probe2\p1–p6`) — SUPERSEDES "ON everywhere":** the user chose **a KNOB, DEFAULT OFF** —
the reaction law on both engines and every off-design page as a setting, the old choked vane the DEFAULT (nothing old
moves, no oracle retirement, no test sort); default reaction 0.3 when switched on. Measured: choked nozzle 0 on CPG,
≤0.06 % one-shaft / ≤0.4 % two-shaft on real gases (0 at design); below choke one-shaft ~0.07 %, two-shaft π_HPC
+0.4…+4 % at the lowest throttles; rung 104's share 0.54→0.52. Cost (unoptimised prototype): Fly it point 3–9 ms →
0.2–0.4 s, Fly-it slam 0.15–0.19 s → 16–21 s, Controls slam 12–18 ms → 0.3–0.4 s (even the CPG shortcut, which ran the
pin solves per closure trial). Afterburner re-check runs ONCE, on today's model.
**Old bug found (record now, fix later — user):** rung 33's one-shaft subsonic residual has TWO roots at hot Mach 0.3,
650–900 K (CPG and TPG, pin too: e.g. 700 K 0.525 and 0.846); its two-ended march sees one sign and refuses "SUB-IDLE".

**How to apply:** anchor first, frozen before code (rung 104's pattern). Plan the law≡pin shortcut from day one
(perfect gas, nozzle choked, η at design: the design ratio still solves (★), so the law IS the pin in algebra — not in
bits; [[rung104-turbine-law]] lesson 1). Expect rung 39's one-way HP→LP arrow ((†)/(‡) "π_LPC-free by construction")
to be the likeliest finding to flip. Related: [[rung104-decisions]].
