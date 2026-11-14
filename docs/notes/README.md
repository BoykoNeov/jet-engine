# Plain-language notes, rungs 25–108

`NOTES.md` at the repository root tells rungs 1–22 in everyday words. This folder carries the story on, one
file per family of rungs (the families of `CLAUDE.md`'s rung table). Each rung gets: what it found, in one
sentence; how it works, without symbols that are not explained; a number or two copied from its spec, with the
case it belongs to; any later rung that corrected it; and where the sandbox page shows it.

The formal record stays in each `docs/rungN-spec.md`. If a note and a spec disagree, the spec is right and the
note is a bug.

| Rungs | File | Status |
|---|---|---|
| 25–30 nozzle and turbine: which shortcuts were earned | [25-30-nozzle-and-turbine.md](25-30-nozzle-and-turbine.md) | written 2026-11-14 |
| 31–33 running the engine you built | [31-33-off-design.md](31-33-off-design.md) | written 2026-11-12 |
| 34–37 one shaft in time | [34-37-one-shaft-in-time.md](34-37-one-shaft-in-time.md) | written 2026-11-12 |
| 38–45 two shafts | [38-45-two-shafts.md](38-45-two-shafts.md) | written 2026-11-12 |
| 46–52 fuel limiters | [46-52-fuel-limiters.md](46-52-fuel-limiters.md) | written 2026-11-12 |
| 53–56 and 61 airflow levers | [53-56-61-airflow-levers.md](53-56-61-airflow-levers.md) | written 2026-11-14 |
| 57–63 schedules (61 is in the airflow-levers file) | [57-63-schedules.md](57-63-schedules.md) | written 2026-11-14 |
| 64–68 valve and cascades | [64-68-valve-and-cascades.md](64-68-valve-and-cascades.md) | written 2026-11-14 |
| 69–84 control-loop structure | [69-84-control-loop-structure.md](69-84-control-loop-structure.md) | written 2026-11-14, one short page |
| 85 blade speeds | [85-blade-speeds.md](85-blade-speeds.md) | written 2026-11-14 |
| 86–89 levers together | [86-89-levers-together.md](86-89-levers-together.md) | written 2026-11-14 |
| 90–94 loads | [90-94-loads.md](90-94-loads.md) | written 2026-11-12 |
| 95–101 governors and inferred thrust | [95-101-governors-and-inferred-thrust.md](95-101-governors-and-inferred-thrust.md) | written 2026-11-12 |
| 102 afterburner | [102-afterburner.md](102-afterburner.md) | written 2026-11-12 |
| 103–105 below choke | [103-105-below-choke.md](103-105-below-choke.md) | written 2026-11-12 |
| 106–108 governor beside the limiters | [106-108-governor-beside-limiters.md](106-108-governor-beside-limiters.md) | written 2026-11-12 |

Every family from 25 to 108 is written (roadmap A4, done 2026-11-14). A later rung that corrects one of these notes
should add a "Later" line to it.

**Gap, outside A4's scope:** `NOTES.md` has no section of its own for rungs 17, 20, 21, 23 and 24 (rung 17 is only
mentioned in passing); those exist only as specs and as their rows in `CLAUDE.md`'s rung table.

Before committing a note, run `powershell -NoProfile -File docs\notes\check-numbers.ps1` (expect "misses: 0"):
it checks every number in each rung section against that rung's own spec, or a spec the section names.
