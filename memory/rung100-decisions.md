---
name: rung100-decisions
description: "User scope decisions 2026-10-18 for rung 100 (rung 99's seam, the formula below choke): exact pressure-ratio push formula; Mach 0.05 AND 0.15 tables added to the speed grid; fan-speed formula = approximation B (LP-shaft balance on the table's gas-generator line); formula REPLACED (rung 99's kept as hidden reduce); thrust at the reading"
metadata:
  node_type: memory
  type: project
  originSessionId: 8918ac28-5a13-4265-984a-bd8007156ca4
  modified: 2026-10-18T18:46:31.665Z
---

Rung 99's seam "the formula with the unchoked nozzle". User picked it 2026-10-18 after `/clear`. The scope moved
three times on measured probes (`W:\temp\claude\rung100-probe`, pre-anchor), each time by the user's answer:

1. First answers: BOTH readings with fan speed APPROXIMATE (A: HP side from the table at the fan speed, LP turbine
   re-solved vs p0); REPLACE the formula (rung 99's kept as a hidden reduce setting); compute AT THE READING.
2. Probe showed A removes only ~25–40 % (plain, 700 K, Mach 0.1: rung 99 +7.1 %, A +5.1 %); a device B (find the
   table row whose HP side, LP turbine re-solved, gives the LP power needed at the fan speed) removes 60–80 % but
   overshoots (−2.2 %). User asked WHY: the unchoked engine leaves the table's one-parameter line (at a fixed burner T
   the fan spins slower, ~1.4 % less core pressure); only a SECOND parameter (this flight's table, or flight speed —
   the grid) can bring it back. With the TRUE HP side the LP re-solve reproduces thrust to 1e-13.
3. User: "try a grid slice low". A Mach-0.05 slice cut the worst low-speed start miss 33 %→1.8 % (fan speed) and
   16 %→3.2 % (pressure ratio) — the grid's straight line below 0.3 was the main error.

**Final scope (user, multi-select, all four):** exact pressure-ratio formula · Mach 0.05 slice in the grid · ALSO a
Mach 0.15 slice (measure before freezing) · fan-speed formula = approximation B.

**How to apply:** grid becomes 0.05, 0.15, 0.3, … (default ON, so default runs below Mach 0.3 move — census by
running). Related: [[rung99-decisions]], [[rung99-speed-axis]], [[rung97-decisions]].

**Status 2026-10-18:** rung 100 BUILT ([[rung100-below-choke]]; spec `docs/rung100-spec.md`, plan § 32).
