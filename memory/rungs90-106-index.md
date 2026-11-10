---
name: rungs90-106-index
description: "Index of rungs 90-106 (loads, metal, governor, unchoke, tables, speed axis, below choke, airflow, afterburner, LP/HP turbine laws, governor beside the slow limiter, the governor beside the realistic release): one process lesson per rung + the user's scope decisions per rung. Append new rungs of this family HERE, not in MEMORY.md (moved out 2026-11-10 at 19.6 KB)"
metadata:
  node_type: memory
  type: project
  originSessionId: 24769f1e-166a-49a8-9795-5af4eefb98aa
  modified: 2026-11-10T08:26:53.403Z
---

Moved out of `MEMORY.md` on 2026-11-10, when the index reached 19.6 KB. Append new rungs and decisions here.

## Process lessons, one per rung
- [90](rung90-customer-bleed.md) the one miss was the guessed bar
- [91](rung91-turbine-cooling.md) extrapolate a linear reading to zero
- [92](rung92-shaft-offtake.md) an exact identity has a domain
- [93](rung93-metal-temperature.md) a screenshot is not a measurement; derive every sign
- [94](rung94-burner-hold.md) a pre-freeze check must not print the claim; bisect a floor near a fold
- [95](rung95-governor.md) a fixed-guess search fails off its guesses — crash map; disclose a rough search-bound clamp
- [96](rung96-inferred-thrust.md) a prediction's named RISK is a claim — measure it even on a HIT
- [97](rung97-unchoked-lp.md) a flag is not a guard; a boundary gate can be tautological
- [98](rung98-corrected-tables.md) a table read outside its rows is a clamp
- [99](rung99-speed-axis.md) derive from model state, not a hand estimate; zero-in-algebra needs an absolute bar
- [100](rung100-below-choke.md) measure a chosen approximation and answer the why; an exact term can remove a compensating error
- [101](rung101-airflow-source.md) read the build path before claiming a start transfers; a probe on one side of choke misleads the other
- [102](rung102-afterburner.md) a fired cost trigger is fixed by COUNTING calls and instants; print a scored quantity dry before freezing
- [103](rung103-steady-unchoked.md) a floor on a slope that vanishes at a boundary is foreseeable; removing a refusal exposes the code behind it
- [104](rung104-turbine-law.md) math-equal is not bit-equal where readings sit on kinks; re-ask a decision once its cost is measured
- [105](rung105-hp-vane.md) re-ask scope before freezing; read the dear law once; classify a refusal at its source
- [106](rung106-governor-lag.md) re-run a comparison claim on the same configurations; all-HIT ⇒ look past the grid
- [107](rung107-governor-release.md) a HIT can rest on the wrong mechanism: split a two-edge device's edges; scope a comparison to the hold it ran on

## The user's decisions, per rung
- [93/94](rung93-94-decisions.md) user 2026-10-16: two rungs, both framings, hold thrust or HP speed
- [95](rung95-decisions.md) governor in the slam; speed law BOTH; clamp+report rich and lean; metal cap all along
- 96: user said "go" to EPR or fan speed, thrust line via a steady table, clean or loaded table (in [96](rung96-inferred-thrust.md))
- [97](rung97-decisions.md) δ/θ switches, all three tables made once, rows over the run, add Mach
- [98](rung98-decisions.md) tables made at the DESIGN flight by default, "this flight" the reduce setting
- [99](rung99-decisions.md) Mach axis on ALL three tables, extend beyond the grid, ON by default, formula also a setting
- [100](rung100-decisions.md) exact pressure-ratio push, Mach 0.05+0.15 grid slices, fan formula B (scope moved on probes)
- [101](rung101-decisions.md) airflow source a CHOICE (old table / speed tables / exhaust sensor), lag knob off, both readings
- [102](rung102-decisions.md) afterburner, ONE big rung (design + fixed engine), duct loss derived, refuse too-hot; hold-thrust refused lit
- [103](rung103-decisions.md) steady matchers below choke: OFF in old code, ON in pages; all six; sub-idle refused
- [104](rung104-decisions.md) A45 never choked: 3-setting flow-law knob, ON everywhere, one rung, Python oracles keep their unmoved keys, sort every failing test
- [105](rung105-decisions.md) HP vane A4 never choked at a normal reaction: reaction knob (0.3) on both engines, DEFAULT OFF after cost re-ask; rung 33 double-root bug recorded
- [106](rung106-decisions.md) governor beside rung 47's temperature-limiter lag only; all holds; 52/88 and the lagged sensor later
- [107](rung107-decisions.md) governor beside rung 52's release AS BUILT (schedule + floor lagged, redline/metal instant); all holds
