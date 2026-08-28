# Memory index

One line per entry — the **hook only**, and one lesson per hook. Detail lives in the linked
file; never re-expand these lines. A rung hook carries the **process lesson** (what went wrong,
what to do differently), never the physical headline — that is already in CLAUDE.md's rung
table, which loads beside this file. Rung entries follow that table's families and numbering.

## Working agreements
- [Session-end routine](session-end-routine.md) — at session end: update memory + docs, commit, push
- [Git remote setup](git-remote-setup.md) — github.com/BoykoNeov/jet-engine, branch main, origin over SSH
- [Always commit and push](always-commit-and-push.md) — auto-commit + push green work; gate is bare `pytest`
- [CLAUDE.md is a reference](claude-md-is-a-reference.md) — one line per rung, detail → the spec; a guard test enforces it
- [Two indexes, one spine](two-indexes-one-spine.md) — CLAUDE.md carries the PHYSICAL verdict, this file the PROCESS lesson
- [Never run the gate for timing](never-run-the-gate-for-timing.md) — a quoted run time is documentation, not a signal
- [Test-suite speed policy](test-suite-speed-policy.md) — ONE gate; the tiering's blocker inverts under COST
- [xdist module-fixture cost](xdist-module-fixture-cost.md) — a module fixture is rebuilt PER WORKER
- [Perf: sonic throat + PyPy](perf-sonic-throat-and-pypy.md) — the hot function was hot ALGORITHMICALLY
- [PyPy switch shipped](pypy-switch-shipped.md) — SLOW_SECONDS kept with its reason INVERTED; psutil load-bearing
- [Golden fingerprint gate](golden-fingerprint-gate.md) — the ONLY absolute-value gate; CPython anchor; measure a detector
- Golden gate slices [2](golden-gate-slice2.md) [3](golden-gate-slice3.md) [4](golden-gate-slice4.md) [5](golden-gate-slice5.md) [6](golden-gate-slice6.md) [7](golden-gate-slice7.md) — drift follows CONDITIONING; a STRIDE knob is not a RESOLUTION knob; lead with the reader that BYPASSES the short-circuit; a difference inherits the drift of what was differenced; a band must clear the SMALLEST live value; a root-finding arm needs a go/no-go first
- [Visuals artifact](visuals-artifact.md) — docs/visuals/ page is artifact 56cde230…; update the SAME URL
- [Windows file-tooling hazards](windows-tooling-file-hazards.md) — PyPy unflushed writes, PowerShell double-encoding, backticks in `-m`, and a status read off the runner

## The Rust port — decided 2026-08-12; **PHASES 0–6 DONE** — phase 6 authorised 2026-08-17 and **COMPLETED 2026-08-20** with slice U's five steps; **PHASE 7 AUTHORISED 2026-08-20**, slices V, W, X and **Y COMPLETE** — slice Y closed 2026-08-27 with all five steps; **SLICE Z (66+67) COMPLETE 2026-08-27** — all five steps: both marches + 20 method bodies, the **38 ported gates** green first run at **20.0x** PyPy, **35 335 oracle keys bit-exact on both interpreters** (CPython exempt on 8 named keys), and **8 dispatch gates** closing the four measured blind spots; **SLICE AA (68) COMPLETE 2026-08-27** — all five steps: `three_loop.rs` 2 302 lines, **47 gates**, **12 084 oracle keys bit-exact vs PyPy** (4 named CPython exemptions), full gate **124 blocks / 1 199 passed / 0 failed**; **SLICE AB (69) PRE-REGISTERED 2026-08-27** — eleven probes, § 5.26; one cell added, TEN swapped — **STEPS 1-3 SHIPPED**: nine bodies + six readers (1 641 Rust lines) then the **25 ported gates** of `tests/rung69.rs` (851 lines, green first run), full gate **127 binaries / 1 241 passed / 0 failed**; a ten-injection sweep caught 9, and the one hole — the Newton budget — is settleable only by step 4's oracle
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
- Slice L (41/42) steps [1](rust-port-slice-l-step1.md) [3](rust-port-slice-l-step3.md) [4](rust-port-slice-l-step4.md) — fallibility is per CALL SITE; measure the detector before believing a smoke check; a claim in the SHIPPED source was false
- [Slice M: rungs 53/54](rust-port-slice-m.md) — a bar asserted in a doc comment but never measured
- Slice N (55/56) [pre-flight](rust-port-slice-n-preflight.md) + steps [1](rust-port-slice-n-step1.md) [2](rust-port-slice-n-step2.md) [3](rust-port-slice-n-step3.md) [4](rust-port-slice-n-step4.md) [5](rust-port-slice-n-step5.md) — a body-read never tells you what its CARRIER costs; a correction applied only FORWARD leaves its precedent standing; three instruments measured NOTHING; a carrier claim on ONE hook says nothing about the next; two censuses on two GRIDS; two of the SOURCE's own gates could not see the defect they name
- [Slice O: rung 61, phase 5 done](rust-port-slice-o.md) — the defect lived in an EDGE, not a node
- [Phase 6 pre-flight](rust-port-phase6-preflight.md) — the same census run in the OPPOSITE direction hit a scoping bug
- [Slice P: rungs 34/35/36](rust-port-slice-p.md) — a perfectly-placed deferral described a branch that does not exist
- [Slice Q: rung 37](rust-port-slice-q.md) — a dead arm is a property of the GRID, not the code
- Slice R (40/44) steps [1](rust-port-slice-r-step1.md) [2](rust-port-slice-r-step2.md) [3](rust-port-slice-r-step3.md) [4](rust-port-slice-r-step4.md) — a registered margin read off the wrong assertion; a registered SUM is not a gated SPLIT; an injection harness needs its own baseline; a probe feeding BOTH sides the same wrong input sees nothing
- Slice S (43/45) [pre-flight](rust-port-slice-s-preflight.md) + steps [1](rust-port-slice-s-step1.md) [2](rust-port-slice-s-step2.md) [3](rust-port-slice-s-step3.md) [4](rust-port-slice-s-step4.md) — an inherited IOU named a gas the code REFUSES; injections reporting "nothing moved" could not have moved anything; a NON-STRICT ordering assertion is satisfied by inertness; a zero measured before being called a hole; a probe's HEADER claimed the suites' grids and its code ran another
- [Slice U pre-flight](rust-port-slice-u-preflight.md) — sweep the arming COMBINATIONS: three shipped asserts no input can reach
- [Slice U step 1](rust-port-slice-u-step1.md) — bit-exact + green says nothing about GATE POWER; an injection table needs a DID-IT-MOVE column; a knife-edge value protected REDUNDANTLY
- Slice U steps [2](rust-port-slice-u-step2.md) [3](rust-port-slice-u-step3.md) [4](rust-port-slice-u-step4.md) [5](rust-port-slice-u-step5.md) — a defender and an exposure on DISJOINT cells; a function exercised only on cells chosen for INERTNESS; a gate comparing a key with ITSELF cannot see its value; the closing step wrote TWO near-vacuous gates of its own
- [Phase 7 pre-flight](rust-port-phase7-preflight.md) — the plan stated the SAME SET twice and nobody diffed the two; then every write-up defect was a table TYPED instead of EMITTED
- Slice V (57-60) [authorised + steps 1a/1b](rust-port-slice-v.md) [step 2](rust-port-slice-v-step2.md) [step 3](rust-port-slice-v-step3.md) [step 4](rust-port-slice-v-step4.md) — a census matches a SHAPE, so a bare permanent assignment was invisible to one built on `try/finally`; both gates written to CLOSE a step could not fail; an injection whose only trace is OBJECT STATE reads exactly like one that never applied, so a did-it-move probe needs witness keys no gate reads; a channel measured closed at one reader was open at the reader BEFORE it, and fixing one guessed bar is not sweeping for them · [step 5](rust-port-slice-v-step5.md) — ask of every assertion in a manufactured-bug gate WHAT FILE IT READS: the four that read nothing survive a regenerated golden; and a gate that MANUFACTURES a bug is code too — inject its own wrapper
- [Slice W (62/63) steps 1-2](rust-port-slice-w.md) — run a refuted probe over the WHOLE table, not the row; a deferral needs a grep of the SUITES; a smoke section on a path the slice cannot reach is what tells a wrong port from a wrong grid
- [Slice W step 3](rust-port-slice-w-step3.md) — five of six injections pass all 88 gates, and the probe was wrong five times: every one a zero nobody measured, so make the instrument prove it can SEE
- [Slice W step 4](rust-port-slice-w-step4.md) — a second-interpreter arm can disagree because the LANGUAGE differs; a renamed predicate hides in the hundred keys where both spellings agree; and "no suite reaches it" is not evidence a cell measures anything
- [Slice W step 5](rust-port-slice-w-step5.md) — a "did it move" assertion passes a HALF-APPLIED injection; assert the exact delta, and mutate your own gates to find out
- Slice X (64) steps [1](rust-port-slice-x-step1.md) [2](rust-port-slice-x-step2.md) [3](rust-port-slice-x-step3.md) [4](rust-port-slice-x-step4.md) [5](rust-port-slice-x-step5.md) — extending a SHARED helper silently voided an override and 1017 green tests could not see it; a coarse grid flipped a published boolean, so emit BOTH grids; a probe that cannot tell a crash from a compile error calls its best result its own failure; both interpreters bit-exact, so the measured bar was that there is none to type, but the tally I logged did not ADD UP; a zero-count assertion is satisfied by DELETING the branch it names
- Slice Y (65) steps [1](rust-port-slice-y-step1.md) [2](rust-port-slice-y-step2.md) [3](rust-port-slice-y-step3.md) [4](rust-port-slice-y-step4.md) [5](rust-port-slice-y-step5.md) — a design refused twice on body reads was revived by EMITTING the set, and a `-x` census column was measuring the repo's own duration cache; a few-ulp "port defect" was my probe simplifying `(1.4-1.0)/1.4` to `0.4/1.4`, so re-run the committed dump FIRST; `ptr::eq` on a `const` tests the optimiser, and the phase's `slow` bill is 16x, measured; a green 35,994-key oracle says nothing about COVERAGE, so inject — and its three misses were exactly the three pre-registered; mutating my own gates found one testing HALF of what it named
- [Slice Z pre-flight](rust-port-slice-z-preflight.md) — my own probe chunked by the `n_sample` the gate PASSES, not the count the STRIDE delivers, and at the right width the answer INVERTED
- [Slice Z step 1](rust-port-slice-z-step1.md) — a shipped refusal blind to the carrier the rung below installs, and a count bar that had already been WRITTEN DOWN once, which reads like a measurement
- [Slice Z step 2](rust-port-slice-z-step2.md) — one scope struct for a per-rung parameter, so a junior silently ignores what Python refuses; and the route witness that caught it twice was blind the third time
- [Slice Z step 3](rust-port-slice-z-step3.md) — the census reported a 0/0 baseline and called its own CONTROLS invisible; and a gate's doc comment claimed a coverage it did not have
- [Slice Z step 4](rust-port-slice-z-step4.md) — a pre-registered exemption of TWO keys measured EIGHT: it counted quantities, the dump emits names
- [Slice Z step 5](rust-port-slice-z-step5.md) — the bar written to repair one vacuity exposed a second in the same gate, and two 'dead' branches are UNOBSERVABLE rather than unexercised
- [Slice AA step 1](rust-port-slice-aa-step1.md) — a growth prediction asked of ONE struct while the step grew three, and `ptr::eq` on a `const` written a second time
- [Slice AA steps 2-5](rust-port-slice-aa-steps2345.md) — a DEFENCE WITH NO READER appeared four times; ask what reads a thing, never wait for a failure
- [Slice AB pre-flight](rust-port-slice-ab-preflight.md) — a shipped section carried a MECHANISM beside its measurement and both it and my replacement were refuted; and a swap whose only content is a panic string
- [Slice AB step 1](rust-port-slice-ab-step1.md) — two gates that could not fail, and two counts typed instead of added up — one read off a runner reporting exit 0 with EMPTY output
- [Slice AB step 2](rust-port-slice-ab-step2.md) — the smoke refuted its own gate, and a step-1 gate went vacuous the moment the bodies landed
- [Slice AB step 3](rust-port-slice-ab-step3.md) — my injection sweep ran 2 of the slice's 3 binaries and printed MISS, which reads as "nothing sees this"
- Slice T (46/47/48) steps [1](rust-port-slice-t-step1.md) [2](rust-port-slice-t-step2.md) [3](rust-port-slice-t-step3.md) [4](rust-port-slice-t-step4.md) — an EXACT ZERO blinds its own gate to the SIGN; 9/9 green and blind to a 24% error, so tabulate BAR MARGINS; a defect can survive because every READER is invariant under it; an injection matching TWICE applies nothing and still reports green

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
- Margin sweeps — [72–77 march audit](rungs72-77-march-audit.md), [74 arrest interval](rung74-arrest-interval.md), [79 gap](rung79-gap-margin.md), [29 π_c](rung29-pi-c-margin.md), [29 M0](rung29-M0-margin.md), [28 β](rung28-beta-margin-hardened.md) — a liveness counter on a FROZEN plant reports FULL activity; the number that didn't parse WAS the finding; a counter is only as good as the NOUN it counts; ENERGY = INVENTORY × COMPLETION; swing, not headroom; β pressure-invariant

## Investigated, NEGATIVE — not shipped, not rungs
- Negatives — [τ_res](tau-res-negative.md), [mixing scale](mixing-scale-negative.md), [turbine march](turbine-march-negative.md), [JICF anchor](mixing-jicf-anchor-negative.md), [pt3 sensor lag](pt3-sensor-lag-negative.md), [per-row blading](per-row-blading-negative.md), [both-edges limiter](both-edges-limiter-negative.md), [phi-rate limiter](phi-rate-limiter-negative.md) — confirms 26; revisit only when δ(J) is anchored; 25's dodge cannot repeat; a SECOND unanchored exponent; the 12%-gap-inside-my-own-band lesson; OVER-DETERMINED; closes the whole pt3-filter family; the ONE negative with a test
