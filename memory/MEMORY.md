# Memory index

One line per entry — the **hook only**, and one lesson per hook. Detail lives in the linked
file; never re-expand these lines. A rung hook carries the **process lesson** (what went wrong,
what to do differently), never the physical headline — that is already in CLAUDE.md's rung
table, which loads beside this file. Rung entries follow that table's families and numbering.

**Line budget: 200 lines.** Raised 2026-09-03 from the ~140 this file had been held to and had
already passed (156 lines, 201 memory files). Stated here, not enforced: the repo's size guard
covers CLAUDE.md, not this file, which the repo only mirrors. **Legitimate growth is one line
per new entry** — and the engine that actually moves the number is the Rust-port section, whose
slice lines *widen* as step links are appended. **Illegitimate growth is a line that restates the
file it links.** If 200 trips, compact — move detail into the entry, or behind a status pointer
as `rust-port-status.md` already does — and judge any further raise against which of the two it
is.

## Working agreements
- [Session-end routine](session-end-routine.md) — at session end: update memory + docs, commit, push
- [Git remote setup](git-remote-setup.md) — github.com/BoykoNeov/jet-engine, branch main, origin over SSH
- [Always commit and push](always-commit-and-push.md) — auto-commit + push green work; gate is bare `pytest`
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
- [Visuals artifact](visuals-artifact.md) — docs/visuals/ page is artifact 56cde230…; update the SAME URL
- [Windows file-tooling hazards](windows-tooling-file-hazards.md) — PyPy unflushed writes, PowerShell double-encoding, backticks in `-m`, a status read off the runner, a log still being written, and a text-mode rewrite that flips every line ending

## The Rust port — phases 0–6 done; phase 7 in flight, slice AF (rung 74) steps 1–4 of six done
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
- [Phase 5 pre-flight](rust-port-phase5-preflight.md) — a "closed set" claim is only as wide as the set you swept
- [Slice I: rungs 31/33](rust-port-slice-i.md) — a bare `except` makes the question REACHABILITY
- [Slice J: rung 32](rust-port-slice-j.md) — exactness bounds the CELLS visited, not the RULES discriminated
- [Slice K: rungs 38/39](rust-port-slice-k.md) — the phase table's scope list had never been ENUMERATED
- Slice L (41/42) steps [1](rust-port-slice-l-step1.md) [3](rust-port-slice-l-step3.md) [4](rust-port-slice-l-step4.md) — fallibility is per CALL SITE (one lesson per file)
- [Slice M: rungs 53/54](rust-port-slice-m.md) — a bar asserted in a doc comment but never measured
- Slice N (55/56) [pre-flight](rust-port-slice-n-preflight.md) + steps [1](rust-port-slice-n-step1.md) [2](rust-port-slice-n-step2.md) [3](rust-port-slice-n-step3.md) [4](rust-port-slice-n-step4.md) [5](rust-port-slice-n-step5.md) — a body-read never tells you what its CARRIER costs (one lesson per file)
- [Slice O: rung 61, phase 5 done](rust-port-slice-o.md) — the defect lived in an EDGE, not a node
- [Phase 6 pre-flight](rust-port-phase6-preflight.md) — the same census run in the OPPOSITE direction hit a scoping bug
- [Slice P: rungs 34/35/36](rust-port-slice-p.md) — a perfectly-placed deferral described a branch that does not exist
- [Slice Q: rung 37](rust-port-slice-q.md) — a dead arm is a property of the GRID, not the code
- Slice R (40/44) steps [1](rust-port-slice-r-step1.md) [2](rust-port-slice-r-step2.md) [3](rust-port-slice-r-step3.md) [4](rust-port-slice-r-step4.md) — a registered margin read off the wrong assertion (one lesson per file)
- Slice S (43/45) [pre-flight](rust-port-slice-s-preflight.md) + steps [1](rust-port-slice-s-step1.md) [2](rust-port-slice-s-step2.md) [3](rust-port-slice-s-step3.md) [4](rust-port-slice-s-step4.md) — an inherited IOU named a gas the code REFUSES (one lesson per file)
- [Slice U pre-flight](rust-port-slice-u-preflight.md) — sweep the arming COMBINATIONS: three shipped asserts no input can reach
- [Slice U step 1](rust-port-slice-u-step1.md) — bit-exact + green says nothing about GATE POWER (one lesson per file)
- Slice U steps [2](rust-port-slice-u-step2.md) [3](rust-port-slice-u-step3.md) [4](rust-port-slice-u-step4.md) [5](rust-port-slice-u-step5.md) — a defender and an exposure on DISJOINT cells (one lesson per file)
- [Phase 7 pre-flight](rust-port-phase7-preflight.md) — the plan stated the SAME SET twice and nobody diffed the two (one lesson per file)
- Slice V (57-60) [authorised + steps 1a/1b](rust-port-slice-v.md) [step 2](rust-port-slice-v-step2.md) [step 3](rust-port-slice-v-step3.md) [step 4](rust-port-slice-v-step4.md) — a census matches a SHAPE, so a bare permanent assignment was invisible to one built on `try/finally` (one lesson per file)
- [Slice W (62/63) steps 1-2](rust-port-slice-w.md) — run a refuted probe over the WHOLE table, not the row (one lesson per file)
- [Slice W step 3](rust-port-slice-w-step3.md) — five of six injections pass all 88 gates, and the probe was wrong five times: every one a zero nobody measured, so make the instrument prove it can SEE
- [Slice W step 4](rust-port-slice-w-step4.md) — a second-interpreter arm can disagree because the LANGUAGE differs (one lesson per file)
- [Slice W step 5](rust-port-slice-w-step5.md) — a "did it move" assertion passes a HALF-APPLIED injection; assert the exact delta, and mutate your own gates to find out
- Slice X (64) steps [1](rust-port-slice-x-step1.md) [2](rust-port-slice-x-step2.md) [3](rust-port-slice-x-step3.md) [4](rust-port-slice-x-step4.md) [5](rust-port-slice-x-step5.md) — extending a SHARED helper silently voided an override and 1017 green tests could not see it (one lesson per file)
- Slice Y (65) steps [1](rust-port-slice-y-step1.md) [2](rust-port-slice-y-step2.md) [3](rust-port-slice-y-step3.md) [4](rust-port-slice-y-step4.md) [5](rust-port-slice-y-step5.md) — a design refused twice on body reads was revived by EMITTING the set (one lesson per file)
- Slice Z (66/67) [pre-flight](rust-port-slice-z-preflight.md) [1](rust-port-slice-z-step1.md) [2](rust-port-slice-z-step2.md) [3](rust-port-slice-z-step3.md) [4](rust-port-slice-z-step4.md) [5](rust-port-slice-z-step5.md) — a probe chunked by the width the gate PASSES, not the one the stride delivers, and at the right width the answer INVERTED (one lesson per file)
- Slice AA (68) [1](rust-port-slice-aa-step1.md) [2-5](rust-port-slice-aa-steps2345.md) — a DEFENCE WITH NO READER appeared four times; ask what reads a thing, never wait for a failure (one lesson per file)
- Slice AB (69) [pre-flight](rust-port-slice-ab-preflight.md) [1](rust-port-slice-ab-step1.md) [2](rust-port-slice-ab-step2.md) [3](rust-port-slice-ab-step3.md) [4](rust-port-slice-ab-step4.md) [5](rust-port-slice-ab-step5.md) — an exemption measured between the two DUMPS was 67 names wider than one measured against the PORT (one lesson per file)
- Slice AC (70/71) [pre-flight](rust-port-slice-ac-preflight.md) [1](rust-port-slice-ac-step1.md) [2](rust-port-slice-ac-step2.md) [3](rust-port-slice-ac-step3.md) [4](rust-port-slice-ac-step4.md) [5](rust-port-slice-ac-step5.md) [6](rust-port-slice-ac-step6.md) [7 closed](rust-port-slice-ac-step7.md) — a predicate by NAME cannot tell an override from a name reused, and every reader launders an injection through `at_lever` (one lesson per file)
- Slice AD (72) [pre-flight](rust-port-slice-ad-preflight.md) [1](rust-port-slice-ad-step1.md) [2](rust-port-slice-ad-step2.md) [3](rust-port-slice-ad-step3.md) [4](rust-port-slice-ad-step4.md) [5](rust-port-slice-ad-step5.md) [6 closed](rust-port-slice-ad-step6.md) — a shipped block documents a method with ZERO definitions, and a gate compared the plant against the function that produced it (one lesson per file)
- Slice AE (73) [pre-flight](rust-port-slice-ae-preflight.md) [1](rust-port-slice-ae-step1.md) [2](rust-port-slice-ae-step2.md) [3](rust-port-slice-ae-step3.md) [4](rust-port-slice-ae-step4.md) [5 closed](rust-port-slice-ae-step5.md) — a FIXTURE is a claim nobody re-reads as one; and a control, an exact-bits gate and an aggregate each measured nothing (one lesson per file)
- Slice AF (74) [pre-flight](rust-port-slice-af-preflight.md) [1](rust-port-slice-af-step1.md) [2](rust-port-slice-af-step2.md) [3](rust-port-slice-af-step3.md) [4](rust-port-slice-af-step4.md) — *is that measured?* belongs to the sentences that say a thing MATTERS, not only to the ones that say it cannot be seen (one lesson per file)
- Slice T (46/47/48) steps [1](rust-port-slice-t-step1.md) [2](rust-port-slice-t-step2.md) [3](rust-port-slice-t-step3.md) [4](rust-port-slice-t-step4.md) — an EXACT ZERO blinds its own gate to the SIGN (one lesson per file)

## Shipped rungs

### 25–30 · nozzle & turbine marches
- [Rung 25 finite-rate nozzle](rung25-finite-rate-nozzle.md) — inverted into a three-state picture
- [Rung 26 freeze-out](rung26-freeze-out.md) — density-driven, not T-driven; refutes rung 25's own framing
- [Rung 27 NO freeze-out](rung27-no-freeze-out.md) — an assumption DERIVED rather than asserted
- [Rung 28 coupled NO march](rung28-coupled-no-march.md) — confirms a verdict while correcting BOTH its reasons
- [Rung 29 shifting turbine](rung29-shifting-turbine.md) — the RATIO ≠ ENERGY correction; the bound-first method
- [Rung 30 choked nozzle](rung30-choked-nozzle.md) — full expansion NOT earned; the pressure term rescues 87%

### 31–33 · off-design steady matching
- [Rung 31 off-design matching](rung31-offdesign-matching.md) — first STRUCTURAL rung; reduce-by-construction
- [Rung 32 component maps](rung32-component-maps.md) — CORRECTS rung 31's "choked hardware IS the map"
- [Rung 33 subsonic matching](rung33-subsonic-matching.md) — coupling through pi_c, not p0 (the advisor's fix)

### 34–37 · the single-spool transient
- [Rung 34 spool transient](rung34-spool-transient.md) — the finding is the RATIO of clocks, not the tautology
- [Rung 35 fuel metering](rung35-fuel-metering.md) — fuel ENLARGES the excursion; the two limits are COUPLED
- [Rung 36 surge line](rung36-surge-line.md) — the zero-constant anchor was DEAD; never gate the crossing
- [Rung 37 combustor dynamics](rung37-combustor-dynamics.md) — the two clocks SPLIT

### 38–45 · two spools
- [Rung 38 two-spool matching](rung38-two-spool-matching.md) — I caught my own over-claim pre-ship
- [Rung 39 two-spool + maps](rung39-two-spool-maps.md) — REFUTES the prediction while CONFIRMING the verdict
- [Rung 40 two-shaft transient](rung40-two-shaft-transient.md) — rho SPLITS: powerless over stability, decisive over oscillation
- [Rung 41 two-spool surge line](rung41-two-spool-surge-line.md) — corrects a mechanism while its verdict survives
- [Rung 42 interstage bleed](rung42-interstage-bleed.md) — my hypothesis REFUTED; state self-targeting in phi-space
- [Rung 43 two-shaft fuel metering](rung43-two-shaft-fuel-metering.md) — the CURRENCY-CIRCULARITY trap
- [Rung 44 transient surge line](rung44-transient-surge-line.md) — the excursion is SCHEDULE-slaved
- [Rung 45 transient fuel surge](rung45-transient-fuel-surge.md) — a monotone overshoot NEVER reaches the reference-free object

### 46–52 · the fuel-side limiter family
- [Rung 46 TIT topping governor](rung46-tit-topping-governor.md) — the limits are SEQUENCED in time
- [Rung 47 lagged topping governor](rung47-lagged-topping-governor.md) — a lag is TRAILING-edge
- [Rung 48 accel schedule](rung48-accel-schedule.md) — a limiter rebates a spool IFF it engages upstream of ITS minimum
- [Rung 49 phi feedback limiter](rung49-phi-feedback-limiter.md) — both my predicted signs were wrong
- [Rung 50 release edge isolated](rung50-release-edge-isolated.md) — refusing my own first CONFIRMING result was the key move
- [Rung 51 release rate](rung51-release-rate.md) — two-sided bracket, after the pre-registered gate was confounded
- [Rung 52 asymmetric lag](rung52-asymmetric-lag.md) — surfacing a conflict beat complying

### 53–56 · airflow levers, on the steady matcher
- [Rung 53 variable stator](rung53-variable-stator.md) — a margin is a DISTANCE, so it is coordinate-dependent
- [Rung 54 stator throat](rung54-stator-throat.md) — the constant SPLIT: shape derived, level disclosed
- [Rung 55 stage stack](rung55-stage-stack.md) — the row count has an INTERIOR optimum
- [Rung 56 per-row capacity](rung56-per-row-capacity.md) — a LEVER'S COST is coordinate-dependent too

### 57–63 · schedules on the transient — except 61, which is steady
- [Rung 57 stator schedule](rung57-stator-schedule-transient.md) — bounds the timing family as 53 bounded the currency
- [Rung 58 composite min-select](rung58-composite-minselect.md) — refuted by MY OWN table: check the SUM, not the term
- [Rung 59 matched schedule](rung59-matched-schedule.md) — discharges rung 58's concession as VACUOUS
- [Rung 60 matched floor](rung60-matched-floor.md) — the advisor's blocker BECAME the headline
- [Rung 61 stator + bleed](rung61-stator-bleed.md) — a "derived" scaling whose binding constant is mine is NOT derived
- [Rung 62 bleed schedule](rung62-bleed-schedule.md) — Newton converging on a residual the plant does not use
- [Rung 63 fuel + bleed](rung63-fuel-bleed.md) — check a quoted number was taken at THIS rung's settings

### 64–68 · the bleed valve, its lag, and cascades
- [Rung 64 phi bleed limiter](rung64-phi-bleed-limiter.md) — the discriminator-before-the-anchor move
- [Rung 65 lagged valve](rung65-lagged-valve.md) — a § 0 pre-check that was itself RK4 instability
- [Rung 66 two-lag cascade](rung66-two-lag-cascade.md) — check where an extremum sits before quoting it
- [Rung 67 cascade A](rung67-cascade-a.md) — a zero cross-gain is saturation, never decoupling
- [Rung 68 three loops](rung68-three-loops.md) — check what is INDEPENDENT before quoting it

### 69–84 · reference splits, rank, and the reader-only rungs
- [Rung 69 reference split](rung69-reference-split.md) — a null space ABSORBS a moved start
- [Rung 70 generic split](rung70-generic-split.md) — I caught a gate computing my own formula twice
- [Rung 71 full split](rung71-full-split.md) — rank independence is NOT constraint independence
- [Rung 72 shared actuator](rung72-shared-actuator.md) — two instruments silently agreed with themselves
- [Rung 73 applied reference](rung73-applied-reference.md) — a bug returning a PERFECT confirmation having measured nothing
- [Rung 74 demand coordinate](rung74-demand-coordinate.md) — a closed-loop difference cannot isolate a forcing
- [Rung 75 anti-windup device](rung75-antiwindup-device.md) — the inherited instrument was BLIND
- [Rung 76 fuel-dependent cap](rung76-fuel-dependent-cap.md) — the 0.7% miss on a pre-registered identity WAS the finding
- [Rung 77 stiffness ledger](rung77-stiffness-ledger.md) — check UNITS before choosing a normalisation
- [Rung 78 residual gauge](rung78-residual-gauge.md) — THREE vacuity traps, only COUNTING caught them
- [Rung 79 state coordinate](rung79-state-coordinate.md) — registering the VACUITY CONDITION beat registering the result
- [Rung 80 split wall](rung80-split-wall.md) — the seam named the wrong NOUN
- [Rung 81 authority clock](rung81-authority-clock.md) — the first grid's 100% was the WEAKER measurement
- [Rung 82 threshold law](rung82-threshold-law.md) — two bars VOID for comparing a physical quantity to a loop count
- [Rung 83 corrector law](rung83-corrector-law.md) — an identity round-trip sold as verification
- [Rung 84 staircase law](rung84-staircase-law.md) — a small integer COUNT cannot carry a RATE

## Margin sweeps — confirmations, not rungs
- Margin sweeps — [72–77 march audit](rungs72-77-march-audit.md), [74 arrest interval](rung74-arrest-interval.md), [79 gap](rung79-gap-margin.md), [29 π_c](rung29-pi-c-margin.md), [29 M0](rung29-M0-margin.md), [28 β](rung28-beta-margin-hardened.md) — a liveness counter on a FROZEN plant reports FULL activity (one lesson per file)

## Investigated, NEGATIVE — not shipped, not rungs
- Negatives — [τ_res](tau-res-negative.md), [mixing scale](mixing-scale-negative.md), [turbine march](turbine-march-negative.md), [JICF anchor](mixing-jicf-anchor-negative.md), [pt3 sensor lag](pt3-sensor-lag-negative.md), [per-row blading](per-row-blading-negative.md), [both-edges limiter](both-edges-limiter-negative.md), [phi-rate limiter](phi-rate-limiter-negative.md) — confirms 26 (one lesson per file)
