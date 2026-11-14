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
| 53–56 and 61 airflow levers | — | to write |
| 57–63 schedules | — | to write |
| 64–68 valve and cascades | — | to write |
| 69–84 control-loop structure | — | to write, last, one short page |
| 85 blade speeds | — | to write |
| 86–89 levers together | — | to write |
| 90–94 loads | [90-94-loads.md](90-94-loads.md) | written 2026-11-12 |
| 95–101 governors and inferred thrust | [95-101-governors-and-inferred-thrust.md](95-101-governors-and-inferred-thrust.md) | written 2026-11-12 |
| 102 afterburner | [102-afterburner.md](102-afterburner.md) | written 2026-11-12 |
| 103–105 below choke | [103-105-below-choke.md](103-105-below-choke.md) | written 2026-11-12 |
| 106–108 governor beside the limiters | [106-108-governor-beside-limiters.md](106-108-governor-beside-limiters.md) | written 2026-11-12 |

Order (roadmap A4): the families a sandbox user meets first — 31–37, 46–52, 90–108 — then the rest.

Before committing a note, run `powershell -NoProfile -File docs\notes\check-numbers.ps1` (expect "misses: 0"):
it checks every number in each rung section against that rung's own spec, or a spec the section names.
