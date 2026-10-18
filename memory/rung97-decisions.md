---
name: rung97-decisions
description: "User scope decisions 2026-10-18 for rung 96's seam (control tables made at one flight, flown at another, corrected by δ/θ): two switches (pressure, temperature); ALL THREE tables made once; rows over the run's own range at the design flight; ADD a Mach knob to Controls"
metadata:
  node_type: memory
  type: project
  originSessionId: f1e86d85-6f2a-4ab4-9593-9fd477e71319
  modified: 2026-10-18T03:25:58.976Z
---

Rung 96's seam "the table made at one flight condition and flown at another, corrected by δ/θ". User picked it
("1") on 2026-10-18 and answered the four scope questions:

- **Corrections: two switches** — pressure (δ) and temperature (θ), each on/off (not one all-or-nothing switch).
- **Tables: ALL THREE made once at the design flight, then corrected** — rung 96's thrust table (`inf::try_table`),
  rung 48's acceleration schedule (`accel_table`), rung 95's speed-proportional steady table (`gv::try_steady_table`).
  (I recommended thrust-only; user chose all three.)
- **Rows: same range as the run**, translated to the design flight (corrected-equivalent throttles) — so at the design
  flight everything is rung 96 bit for bit.
- **Mach: YES, add a Mach knob to the Controls view in this step** (I recommended not now). Corrections cannot undo a
  speed change — the textbook reason schedules carry a Mach axis.

Pre-anchor probe (2026-10-18, `W:\temp\claude\rung97-probe\`), disclosed so steady temperature claims are NOT blind:
pressure similarity exact (0.0); temperature similarity NOT exact — corrected thrust +0.69 %/+10 % θ at 1400 K
(+0.48 % at 1000 K), because `f = (h4−h3)/(η_b·hPR − h4)`: the fuel's fixed heating value makes `f/θ` rise with θ.
Fixed generator power / fixed customer amount break pressure similarity (12 % / 33 % thrust at p0×½).

**Found 2026-10-18, user decision "Fix it properly first":** the Controls view draws two-shaft points whose nozzle is
UNCHOKED (the reply's `choked` flag is ignored by the page) — and the two-spool transient solves the LP turbine on the
`a8` choke regardless (`two_spool_transient.rs` r40 tail), so those numbers are void. Reachable since plan § 27
(e.g. sea level +40 K, 700 K throttle; 9–20 of 126 points), far more at low Mach. Offered: stop/refuse in words,
or draw dashed — user chose to BUILD the two-spool unchoked LP branch (CLAUDE.md OPEN item, rung 38's flag) FIRST.

Mach density-rule check done (inlet total density, `density_ratio` with a Mach factor): 792 runs, unclamped worst
1.7e−4, rich-edge worst 6.4e−4 — same level as § 27's disclosed 2.5e−4 / 7e−4 (`W:\temp\claude\rung97-probe\mach_rulecheck.txt`).
Mach code edits are in git stash "sandbox-28-mach-wip" (sandbox_controls.rs m0 field + density Mach factor, tests/sandbox_controls.rs, template, browser.mjs) — pop it for step (2).

**How to apply:** order is now — (1) RUNG 97 = the two-spool unchoked LP branch on the TRANSIENT family (rung 40 tails
+ powers, every variant; the steady rung 38/39 matcher keeps its refusal — status "transient BUILT, steady OPEN"),
(2) Mach on Controls (sandbox plan § 28), (3) RUNG 98 = the δ/θ corrected tables → anchor → model/page/spec. Related:
[[rung96-inferred-thrust]], [[sandbox-altitude]], [[rung95-decisions]].

**Status 2026-10-18:** step (1) RUNG 97 SHIPPED ([[rung97-unchoked-lp]]; spec `docs/rung97-spec.md`, plan § 28). Next:
pop stash "sandbox-28-mach-wip" → the Mach knob is now plan **§ 29** (its stashed comments say "§ 28" — renumber), then
rung 98 (δ/θ). The Mach page must keep rung 97's dashed-unchoked drawing and table row.
