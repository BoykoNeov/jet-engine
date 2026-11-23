---
name: rung110-wip-handoff
description: "Rung 110 (turbofan flown) WIP status: gates 1-5 + 4b green, full gate run 2026-11-23 (2595 pass, only the stale sandbox page fails); remaining: gates 6-8, P1-P8 grid, CLI panel, page rebuild, then spec/row/notes/splice"
metadata:
  node_type: memory
  type: project
  originSessionId: e6648d71-da72-47be-b1d8-c7aee1ca569c
  modified: 2026-11-23T19:40:03.349Z
---

**Status 2026-11-23 (commits dfd3c7d, bfd30c3, pushed):** gate 4b GREEN (a below-choke crawl: fan loops get
FAN_MAX 1000 passes + a FAN_FLOOR 1e-9 rounding stop); rung 104's choked-nozzle shortcut skipped when the design nozzle
was unchoked. **Full gate ran: 2595 passed, the only failure the sandbox page being stale** (no oracle moved, so the
user's "stop if a recorded result moves" condition on the one-nozzle rule has not fired YET — it can still fire at the
page rebuild: diff the page's numbers/browser checks, and if anything moves, STOP and show the user, never re-capture).

**Still to do, in order:** gates 6–8 and the P1–P8 grid (210 cells, anchor § 3 — its own grid, not the test consts;
P4/gate 7 perturb `eta_hpc` on the FROZEN hardware, never a rebuilt design), the CLI panel
`print_flown_turbofan_table` + golden, `rust/sandbox-wasm/build.ps1` (main tree), full gate, commit + push; then
spec / CLAUDE.md row / notes / splice.

Earlier findings (each for the spec): FAN_TOL 1e-10 (table-gas inversion noise ~2e-11 made 1e-13 a 2-cycle); D8 speeds
`n_fan`/`n_hp` are what P1/P6 score (`n_*_map` printed only); ONE nozzle rule (A8 at the actual design exit when
unchoked); one-shaft end flies on rung 31; v0 == 0 scores via `score_or_static`; gate 5's 0.3 %-of-printed was a MISS
(thrust 0.367 %), the explanation asserted instead.

Kept on purpose: `W:\temp\claude\rung110-flown-turbofan\probe\` and `book\` (gate 5 sources, OCR). Related:
[[rung110-decisions]], [[needless-precision]].
