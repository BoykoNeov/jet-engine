# Memory index

One line per entry — the **hook only**, and one lesson per hook. Detail lives in the linked
file; never re-expand these lines. A rung hook carries the **process lesson** (what went wrong,
what to do differently), never the physical headline — that is already in CLAUDE.md's rung
table, which loads beside this file. Rung entries follow that table's families and numbering.

**Line budget: 140 lines** (lowered 2026-10-16 to match the memory hook; it was 200 from 2026-09-03). Rung entries are ONE LINE PER FAMILY. Earlier: raised 2026-09-03 from the ~140 this file had been held to and had
already passed (156 lines, 201 memory files). Stated here, not enforced: the repo's size guard
covers CLAUDE.md, not this file, which the repo only mirrors. **Legitimate growth is one line
per new entry**. **Illegitimate growth is a line that restates the file it links.** If 200 trips,
compact — move detail into the entry, or behind a status pointer as `rust-port-status.md` already
does — and judge any further raise against which of the two it is. **A BYTE limit binds too**: the
index is read up to ~24.4 KB, and on 2026-09-26 it reached 22.8 KB, almost all of it the Rust-port
slice lines (which *widen* as step links are appended). Those moved to `rust-port-slice-index.md`;
new slices and step links go THERE, not here.

## Working agreements
- [Sandbox direction](sandbox-direction.md) — user 2026-10-06: a SANDBOX, not pass/fail lessons; ship knobs, prefer realistic absorbers; web page slice 1 BUILT
- [Sandbox slice 1](sandbox-slice1.md) — screenshots found what 122 green checks missed; a bar measured on a toy spike was 4 decades too tight
- [Sandbox slice 3](sandbox-slice3.md) — a first new caller exposed a constant no test varied (design back-pressure); time solves as the product runs them
- [Sandbox slice 2](sandbox-slice2.md) — 'it runs' hid a scalar gamma; classify failures through the product's own path; a browser wait met by the PREVIOUS run
- [Sandbox slice 4A](sandbox-slice4a.md) — a cause written before its message was seen; a proxy classifier hid a case; SANDBOX_SHOTS is a path
- [Sandbox slice 4B](sandbox-slice4b.md) — my own plan's trajectory classifier was a proxy; a hair-thin tolerance gap meant a mixed class
- [Sandbox slice 5](sandbox-slice5.md) — the error path had its own error (non-ASCII message hung the page since slice 1); every await on an external process needs a ceiling
- [Solver walls repair](solver-walls-repair.md) — a fix to failing cases moved 1 416 oracle keys (failure was control flow); a continuation crashed a diagnostic, so it ships default-OFF
- [Back-pressure repair](back-pressure-repair.md) — 2026-10-16: a census logger in a threaded test process must write each record in ONE call; red-check by stashing src only
- [Sandbox altitude](sandbox-altitude.md) — a step-checked limit holds only at its flight; a ×2 similarity cannot see a re-capture; empty-vs-empty cmp says IDENTICAL
- [Sandbox Mach](sandbox-mach.md) — "refused at every throttle" written from ONE throttle tried there; an archived probe source was a copy of another (equal output sizes)
- [Schedule-check census](schedule-check-census.md) — census a fix's reach by RUNNING, not by the seam's stated scope (it said 63; it was 49–73)
- [Schedule-check flip](schedule-check-flip.md) — a total over a run compares only at equal run length (rung 65 § 3 did not)
- [Reused-machine map reset](reused-machine-map-reset.md) — leftover state can be oracle-gated Python behaviour; fix where it harms (next march's start), not everywhere
- [Session-end routine](session-end-routine.md) — at session end: update memory + docs, commit, push
- [Git remote setup](git-remote-setup.md) — github.com/BoykoNeov/jet-engine, branch main, origin over SSH
- [Always commit and push](always-commit-and-push.md) — auto-commit + push green work; gate is `rust/test-all.ps1`: cargo tests run in parallel, below-normal (pytest until 2026-10-05)
- [CLAUDE.md is a reference](claude-md-is-a-reference.md) — one line per rung, detail → the spec; a guard test enforces it
- [Two indexes, one spine](two-indexes-one-spine.md) — CLAUDE.md carries the PHYSICAL verdict, this file the PROCESS lesson
- [Instrument fed by what it certifies](instrument-fed-by-what-it-certifies.md) — AC/AD/AE all shipped a gate that agreed with itself; ask what SUPPLIES the value
- [Never run the gate for timing](never-run-the-gate-for-timing.md) — a quoted run time is documentation, not a signal
- [Run tests below normal](run-tests-below-normal.md) — launch every suite at BELOW-NORMAL priority, and capture its full output
- [Test-suite speed policy](test-suite-speed-policy.md) — ONE gate; the tiering's blocker inverts under COST
- [xdist module-fixture cost](xdist-module-fixture-cost.md) — a module fixture is rebuilt PER WORKER
- [Perf: sonic throat + PyPy](perf-sonic-throat-and-pypy.md) — the hot function was hot ALGORITHMICALLY
- [PyPy switch shipped](pypy-switch-shipped.md) — SLOW_SECONDS kept with its reason INVERTED; psutil load-bearing
- [Golden fingerprint gate](golden-fingerprint-gate.md) — the ONLY absolute-value gate; CPython anchor; measure a detector
- Golden gate slices [2](golden-gate-slice2.md) [3](golden-gate-slice3.md) [4](golden-gate-slice4.md) [5](golden-gate-slice5.md) [6](golden-gate-slice6.md) [7](golden-gate-slice7.md) — drift follows CONDITIONING (one lesson per file)
- [Visuals artifact](visuals-artifact.md) — the CHARTS page, re-minted 2026-10-06 at G5uobeC9…; read its prose for staleness before a republish
- [Cutaway artifact](cutaway-artifact.md) — the animated engine cutaway is a SECOND artifact (968af1ea…, favicon ⚙️); file:// is refused, serve locally and kill by PID
- [Visuals ↔ model binding](visuals-model-binding.md) — the pages had NO test and looked fine; in SYNC is not BOUND, and the page's own design point was typed markup
- [Windows file-tooling hazards](windows-tooling-file-hazards.md) — PyPy unflushed writes, PowerShell double-encoding, backticks in `-m`, a status read off the runner, a log still being written, and a text-mode rewrite that flips every line ending, `cmd`'s parse-time `%ERRORLEVEL%`, a `start /b` batch that never exits, the EOL check that was itself the hazard, a silent `pytest.exe` launcher, a scripted Windows path that becomes a TAB, and a `Copy-Item` restore whose OLD mtime cargo never rebuilds, `python` being the Store stub that hangs, and a bash-built `cmd start` path with a literal `$b`

## The Rust port — DONE 2026-10-05 (phases 0–8; slice AU deleted the Python on top of tag `python-final`; the gate is `rust/test-all.ps1`)
- [Rust port status](rust-port-status.md) — the running tally: phases, slices, and the numbers each closed slice landed on. **Update THAT file, not this line.**
- [Rust port decided](rust-port-decided.md) — plan is docs/plans/todo-rust-port.md; a new PHASE needs authorisation
- [Ladder architecture](rust-port-ladder-architecture.md) — a const table of fn pointers per rung; generics lost
- [Rust arithmetic IS PyPy](rust-port-arithmetic-is-pypy.md) — size a solver claim by DISTINCT ROOTS, not row count
- [Power spelling is split](rust-port-power-spelling.md) — multiply the square, pow above it
- [Shape keys](rust-port-shape-keys.md) — only a GRIDDED argmax pins a location
- [Location keys REFUTE](rust-port-location-keys-refute.md) — the argmax that pays is the one that DISAGREES
- [A ported test can go VACUOUS](rust-port-ported-test-vacuity.md) — a better factorisation turns a real pin into self-comparison
- [Inside/outside exactness](rust-port-inside-outside-exactness.md) — exact in algebra ≠ in arithmetic; a residual needs an ABSOLUTE bar
- [Measure before registering](rust-port-measure-before-registering.md) — probe FIRST; a bit-equality gate is blind to a SHARED assumption
- [COPY vs REDERIVATION](rust-port-copy-vs-rederivation.md) — don't factor a deliberate duplication away
- [A documented gate that doesn't exist](rust-port-documented-gate-that-doesnt-exist.md) — a count guard is blind to a class absent from BOTH sides
- [An oracle cannot see a MISSING GATE](rust-port-oracle-cannot-see-a-missing-gate.md) — bit-exactness says nothing about COVERAGE
- [Guessed census bars](rust-port-guessed-census-bars.md) — five typed count bars, five wrong; measure counts
- [Census by compiler, not by name](census-by-compiler-not-by-name.md) — the sonic_throat repair: a name grep missed the turbine hook; renaming the panicking wrapper on a scratch copy listed the real holdouts
- [Solver audit, Rust era](solver-audit-rust-era.md) — a CLAMP between step and convergence test is a floor no precision fixes; removing a false failure can expose a true one on a panicking route
- [Phase 5 pre-flight](rust-port-phase5-preflight.md) — a "closed set" claim is only as wide as the set you swept
- [Rust port slice index](rust-port-slice-index.md) — every slice I…AI and its step files, one line each with its process lessons; append new slices THERE (moved out 2026-09-26 at 22.8 KB)

## Shipped rungs — one line per family (CLAUDE.md's), one short lesson per linked file
- 25–30 marches — [25](rung25-finite-rate-nozzle.md) inverted into a three-state picture · [26](rung26-freeze-out.md) density, not T; refutes 25's framing · [27](rung27-no-freeze-out.md) an assumption DERIVED · [28](rung28-coupled-no-march.md) a verdict confirmed, BOTH reasons corrected · [29](rung29-shifting-turbine.md) RATIO ≠ ENERGY; bound first · [30](rung30-choked-nozzle.md) full expansion not earned
- 31–33 steady matching — [31](rung31-offdesign-matching.md) reduce-by-construction · [32](rung32-component-maps.md) corrects "hardware IS the map" · [33](rung33-subsonic-matching.md) coupling through pi_c, not p0
- 34–37 one-spool transient — [34](rung34-spool-transient.md) the RATIO of clocks, not the tautology · [35](rung35-fuel-metering.md) the two limits COUPLED · [36](rung36-surge-line.md) never gate the crossing · [37](rung37-combustor-dynamics.md) the two clocks SPLIT
- 38–45 two spools — [38](rung38-two-spool-matching.md) caught my own over-claim · [39](rung39-two-spool-maps.md) refutes the prediction, confirms the verdict · [40](rung40-two-shaft-transient.md) rho SPLITS · [41](rung41-two-spool-surge-line.md) mechanism corrected, verdict survives · [42](rung42-interstage-bleed.md) my hypothesis refuted · [43](rung43-two-shaft-fuel-metering.md) currency circularity · [44](rung44-transient-surge-line.md) SCHEDULE-slaved · [45](rung45-transient-fuel-surge.md) never reaches the reference-free object
- 46–52 fuel limiters — [46](rung46-tit-topping-governor.md) limits SEQUENCED in time · [47](rung47-lagged-topping-governor.md) a lag is trailing-edge · [48](rung48-accel-schedule.md) engages upstream of ITS minimum · [49](rung49-phi-feedback-limiter.md) both predicted signs wrong · [50](rung50-release-edge-isolated.md) refuse your own first confirmation · [51](rung51-release-rate.md) two-sided bracket · [52](rung52-asymmetric-lag.md) surface a conflict, don't comply
- 53–56 airflow levers — [53](rung53-variable-stator.md) a margin is a DISTANCE · [54](rung54-stator-throat.md) shape derived, level disclosed · [55](rung55-stage-stack.md) an INTERIOR optimum · [56](rung56-per-row-capacity.md) a lever's COST is coordinate-dependent
- 57–63 schedules — [57](rung57-stator-schedule-transient.md) bounds the timing family · [58](rung58-composite-minselect.md) check the SUM, not the term · [59](rung59-matched-schedule.md) a concession VACUOUS · [60](rung60-matched-floor.md) the blocker became the headline · [61](rung61-stator-bleed.md) my binding constant is not "derived" · [62](rung62-bleed-schedule.md) Newton on a residual the plant ignores · [63](rung63-fuel-bleed.md) a number taken at THIS rung's settings?
- 64–68 valve & cascades — [64](rung64-phi-bleed-limiter.md) discriminator before the anchor · [65](rung65-lagged-valve.md) a pre-check that was RK4 instability · [66](rung66-two-lag-cascade.md) locate an extremum before quoting it · [67](rung67-cascade-a.md) zero cross-gain is saturation · [68](rung68-three-loops.md) check what is INDEPENDENT
- 69–76 splits & rank — [69](rung69-reference-split.md) a null space absorbs a moved start · [70](rung70-generic-split.md) a gate computing my formula twice · [71](rung71-full-split.md) rank ≠ constraint independence · [72](rung72-shared-actuator.md) instruments agreeing with themselves · [73](rung73-applied-reference.md) a perfect confirmation that measured nothing · [74](rung74-demand-coordinate.md) a closed-loop difference cannot isolate a forcing · [75](rung75-antiwindup-device.md) the inherited instrument was blind · [76](rung76-fuel-dependent-cap.md) the 0.7 % miss WAS the finding
- 77–84 reader-only — [77](rung77-stiffness-ledger.md) UNITS before normalising · [78](rung78-residual-gauge.md) only COUNTING caught three vacuity traps · [79](rung79-state-coordinate.md) register the VACUITY CONDITION · [80](rung80-split-wall.md) the seam named the wrong noun · [81](rung81-authority-clock.md) the first 100 % was the weaker measurement · [82](rung82-threshold-law.md) a quantity vs a loop count is void · [83](rung83-corrector-law.md) an identity round-trip is not verification · [84](rung84-staircase-law.md) a small COUNT cannot carry a RATE
- 85–89 knobs — [85](rung85-blade-speed.md) list EVERY parameter two cases differ in; a (1,1) identity fed by hand · [86](rung86-three-levers.md) the READING is not the RUN · [87](rung87-valve-reference.md) a prediction vs a pointwise optimum is construction · [88](rung88-slow-valve.md) no slack ⇒ a perturbation FLIPS it; predict the sign · [89](rung89-asymmetric-valve.md) a fixed-set-point forecast cannot bound a case-switching perturbation
- 90–99 loads, metal, governor, unchoke, tables & speed axis — [90](rung90-customer-bleed.md) the one miss was the guessed bar · [91](rung91-turbine-cooling.md) extrapolate a linear reading to zero · [92](rung92-shaft-offtake.md) an exact identity has a domain · [93](rung93-metal-temperature.md) a screenshot is not a measurement; derive every sign · [94](rung94-burner-hold.md) a pre-freeze check must not print the claim; bisect a floor near a fold · [95](rung95-governor.md) a fixed-guess search fails off its guesses — crash map; disclose a rough search-bound clamp · [96](rung96-inferred-thrust.md) a prediction's named RISK is a claim — measure it even on a HIT · [97](rung97-unchoked-lp.md) a flag is not a guard; a boundary gate can be tautological · [98](rung98-corrected-tables.md) a table read outside its rows is a clamp · [99](rung99-speed-axis.md) derive from model state, not a hand estimate; zero-in-algebra needs an absolute bar
- Decisions — [93/94](rung93-94-decisions.md) user 2026-10-16: two rungs, both framings, hold thrust or HP speed · [95](rung95-decisions.md) governor in the slam; speed law BOTH; clamp+report rich and lean; metal cap all along · 96: user said "go" to EPR or fan speed, thrust line via a steady table, clean or loaded table (in [96](rung96-inferred-thrust.md)) · [97](rung97-decisions.md) δ/θ switches, all three tables made once, rows over the run, add Mach · [98](rung98-decisions.md) tables made at the DESIGN flight by default, "this flight" the reduce setting · [99](rung99-decisions.md) Mach axis on ALL three tables, extend beyond the grid, ON by default, formula also a setting


## Margin sweeps — confirmations, not rungs
- Margin sweeps — [72–77 march audit](rungs72-77-march-audit.md), [74 arrest interval](rung74-arrest-interval.md), [79 gap](rung79-gap-margin.md), [29 π_c](rung29-pi-c-margin.md), [29 M0](rung29-M0-margin.md), [28 β](rung28-beta-margin-hardened.md) — a liveness counter on a FROZEN plant reports FULL activity (one lesson per file)

## Investigated, NEGATIVE — not shipped, not rungs
- Negatives — [τ_res](tau-res-negative.md), [mixing scale](mixing-scale-negative.md), [turbine march](turbine-march-negative.md), [JICF anchor](mixing-jicf-anchor-negative.md), [pt3 sensor lag](pt3-sensor-lag-negative.md), [per-row blading](per-row-blading-negative.md), [both-edges limiter](both-edges-limiter-negative.md), [phi-rate limiter](phi-rate-limiter-negative.md) — confirms 26 (one lesson per file)
