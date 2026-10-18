---
name: rung99-decisions
description: "User scope decisions 2026-10-18 for rung 99 (a flight-speed / Mach axis on the control tables, rung 98's seam): ALL THREE tables; EXTEND the trend beyond the grid; axis ON by default; the exact formula correction ALSO a page setting (none / grid / formula)"
metadata:
  node_type: memory
  type: project
  originSessionId: 88c81e76-8dcf-41fb-90c3-5b92afee2032
  modified: 2026-10-18T11:14:12.899Z
---

Rung 98's seam "a Mach axis: a table made at several flight speeds and read at the one flown". User picked it ("1")
on 2026-10-18, after `/clear`, and answered four scope questions (asked before any probe or anchor, per
[[rung98-corrected-tables]] lesson 2):

- **Tables: ALL THREE get the speed axis** (thrust table, rung 48's schedule, rung 95's steady HP table). I recommended
  thrust-only (rung 98 G3: at fixed inlet totals a Mach change moves only the thrust while choked); user chose all three.
  The probe must still show what the axis does to the other two (expected: nothing while choked; only unchoked rows).
- **Beyond the grid: EXTEND the trend** (linear through the last two slices), not clamp, not refuse. I recommended
  clamp. Report how far out it is extrapolating.
- **Default: speed axis ON.** Saved off-design-Mach runs that read a table move again; "off" = rung 98 bit for bit
  (the reduce setting).
- **Formula correction ALSO a page setting** — a three-way choice: no speed correction (rung 98) / speed grid /
  formula (rung 98 G6's exact split: nozzle pressure term + ram drag recomputed for the flight speed). I recommended
  test-only. The formula is still the non-self-certifying reference for the grid's blending error.
- **Grid: Mach 0.3, 0.6, 0.85, 1.2, 1.6, 2.0, 2.5, 3.0** (user, 2026-10-18, second question). My first default ended at
  2.0; the pre-anchor reach probe showed extending beyond 2.0 needs the 1.6 slice, which cannot be built for Mach
  2.5–3 flights (135 vs rung 98's 130 of 162 built); to 3.0 builds 147–157 of 162. So "extend the trend" acts only
  below Mach 0.3 on the page. Slices in the design altitude's air (250 K, 50 kPa), each carried by its own δ/θ; between slices, read BOTH bracketing slices at the same reading and
  blend the inferred values (advisor), not the rows.

**Open risk for the probe:** whether the steady two-spool equilibrium can solve unchoked rows (CLAUDE.md OPEN item; rung
97 built only the transient branch) — the slow end of the grid may be out of reach.

**How to apply:** default setting = grid; the three-way "speed correction" knob acts only under `table_at: design`
and only when a table is read. Related: [[rung98-decisions]], [[rung97-decisions]].

**Status 2026-10-18:** rung 99 BUILT ([[rung99-speed-axis]]; spec `docs/rung99-spec.md`, plan § 31).
