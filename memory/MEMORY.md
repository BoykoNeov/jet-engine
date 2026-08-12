# Memory index

One line per entry — the **hook only**, and one lesson per hook. Detail lives in the linked
file; never re-expand these lines. A rung hook carries the **process lesson** (what went wrong,
what to do differently), never the physical headline — that is already in CLAUDE.md's rung
table, which loads beside this file. Rung entries follow that table's families and numbering.

## Working agreements
- [Session-end routine](session-end-routine.md) — at end of batch/planning, or "session end": update memory + docs, commit, push to main
- [Git remote setup](git-remote-setup.md) — github.com/BoykoNeov/jet-engine (public), branch main, origin over SSH
- [Always commit and push](always-commit-and-push.md) — auto-commit + push green work unasked; gate is bare `pytest`; never gate a docs-only change
- [CLAUDE.md is a reference](claude-md-is-a-reference.md) — one line per rung, detail → docs/rungN-spec.md; a guard test enforces the budget
- [Two indexes, one spine](two-indexes-one-spine.md) — CLAUDE.md carries the PHYSICAL verdict, this file the PROCESS lesson; never restate the headline
- [Never run the gate for timing](never-run-the-gate-for-timing.md) — a quoted run time is documentation, not a signal; take it from a run already happening
- [Test-suite speed policy](test-suite-speed-policy.md) — ONE gate; the tiering's blocker was a COUNT argument that inverts under COST
- [xdist module-fixture cost](xdist-module-fixture-cost.md) — a module fixture is rebuilt PER WORKER, so each consumer multiplies it (2:37 added to a 2:59 gate)
- [Perf: sonic throat + PyPy](perf-sonic-throat-and-pypy.md) — the hot function was hot ALGORITHMICALLY (45 bisections for a linear root), not linguistically
- [PyPy switch shipped](pypy-switch-shipped.md) — SLOW_SECONDS kept with its reason INVERTED (bought time → buys determinism); psutil load-bearing
- [Golden fingerprint gate](golden-fingerprint-gate.md) — the ONLY absolute-value gate; goldens are a CPython anchor, never regenerate elsewhere; measure a detector's sensitivity
- [Golden gate slice 2](golden-gate-slice2.md) — drift is set by CONDITIONING, not by rung; the reduced-resolution worry measured backwards
- [Golden gate slice 3](golden-gate-slice3.md) — a STRIDE knob is not a RESOLUTION knob (an arm that guarded nothing); a structural zero needs a two-sided ABS tolerance
- [Golden gate slice 4](golden-gate-slice4.md) — lead an arm with the reader that BYPASSES the short-circuit; when zeros ARE the finding, loosening tolerance loosens the CLAIM
- [Golden gate slice 5](golden-gate-slice5.md) — a finite difference inherits drift from the quantity DIFFERENCED, so relative is the wrong currency
- [Golden gate slice 6](golden-gate-slice6.md) — a band must clear the SMALLEST LIVE value it covers; bit-equality can be PREDICTED from the arithmetic
- [Golden gate slice 7](golden-gate-slice7.md) — a ROOT-FINDING arm needs a go/no-go before it is written; the smallest live value belonged to the SEARCH, not the physics
- [Visuals artifact](visuals-artifact.md) — docs/visuals/ page is artifact 56cde230…; update the SAME URL, regenerate via extract_data.py + build.py
- [Windows file-tooling hazards](windows-tooling-file-hazards.md) — PyPy leaves `open().write()` unflushed, PowerShell Get/Set-Content double-encodes UTF-8; both fail SILENTLY

## The Rust port — decided 2026-08-12; PHASES 0–3 DONE (slices A–E); PHASE 4 authorised, slice F done (330 Rust tests)
- [Rust port decided](rust-port-decided.md) — plan is docs/plans/todo-rust-port.md; slices are free inside an authorised phase, **phase 5 needs fresh authorisation**
- [Ladder architecture](rust-port-ladder-architecture.md) — a const table of fn pointers per rung; generics lost by COMPILING and returning a silently different number
- [Rust arithmetic IS PyPy](rust-port-arithmetic-is-pypy.md) — 100% bit-exact vs PyPy on gas AND cycle; size a solver claim by DISTINCT ROOTS, not row count
- [Power spelling is split](rust-port-power-spelling.md) — multiply the square, pow above it; a tolerance bar hid the defect for a whole phase
- [Shape keys](rust-port-shape-keys.md) — dump a finding's ARGMAX: a peak's VALUE drifts between interpreters and its LOCATION does not; an analytic cancellation read through a solver is exact to the solver's GRID
- [Location keys REFUTE](rust-port-location-keys-refute.md) — the argmax that pays is the one that DISAGREES; a group is earned by the factor appearing twice; sweep wider than the source's own gate
- [A ported test can go VACUOUS](rust-port-ported-test-vacuity.md) — a better factorisation turns the source's real pin into a self-comparison; a location key on a scheme boundary needs a COARSER grid
- [Inside/outside exactness](rust-port-inside-outside-exactness.md) — an op applied INSIDE an accumulation and removed OUTSIDE is exact in algebra, not arithmetic, and the source says “exactly”; a residual needs an ABSOLUTE bar
- [Measure before registering](rust-port-measure-before-registering.md) — probe FIRST, then pre-register; a slack bar in the source is a lead to a wrong exactness claim, not a number to copy; and a bit-equality gate is BLIND to an assumption both sides share
- [COPY vs REDERIVATION](rust-port-copy-vs-rederivation.md) — an "exactly" claim survives a copied instruction sequence and dies on a second derivation; so don't factor a deliberate duplication away
- [A documented gate that doesn't exist](rust-port-documented-gate-that-doesnt-exist.md) — the key-COUNT guard is blind to a class ABSENT FROM BOTH sides; make a dead key earn its place instead of deleting it

## Shipped rungs

### 25–30 · nozzle & turbine marches
- [Rung 25 finite-rate nozzle](rung25-finite-rate-nozzle.md) — inverted into a three-state picture (an irreversible-fast ceiling below the reversible bound)
- [Rung 26 freeze-out](rung26-freeze-out.md) — density-driven, not T-driven; refutes rung 25's own seam framing
- [Rung 27 NO freeze-out](rung27-no-freeze-out.md) — an assumption DERIVED rather than asserted; the kill test INVERTS rung 26
- [Rung 28 coupled NO march](rung28-coupled-no-march.md) — confirms a verdict while correcting BOTH its reasons; the precedent for editing a shipped rung
- [Rung 29 shifting turbine](rung29-shifting-turbine.md) — the RATIO ≠ ENERGY correction; the bound-first method
- [Rung 30 choked nozzle](rung30-choked-nozzle.md) — full expansion NOT earned; the pressure term rescues 87%

### 31–33 · off-design steady matching
- [Rung 31 off-design matching](rung31-offdesign-matching.md) — first STRUCTURAL rung; reduce-by-construction
- [Rung 32 component maps](rung32-component-maps.md) — CORRECTS rung 31's "choked hardware IS the map"
- [Rung 33 subsonic matching](rung33-subsonic-matching.md) — the INVERSION of rung 31; coupling through pi_c, not p0 (advisor's fix)

### 34–37 · the single-spool transient
- [Rung 34 spool transient](rung34-spool-transient.md) — the finding is the RATIO of clocks, not the tautological I-independent shape
- [Rung 35 fuel metering](rung35-fuel-metering.md) — CORRECTS rung 34: fuel ENLARGES the excursion, the two accel limits are COUPLED
- [Rung 36 surge line](rung36-surge-line.md) — the zero-new-constant anchor was DEAD so phi_surge is imposed, but the SIGN survives; never gate the crossing
- [Rung 37 combustor dynamics](rung37-combustor-dynamics.md) — the two clocks SPLIT: one confirms rung 34's concession, the other corrects it

### 38–45 · two spools
- [Rung 38 two-spool matching](rung38-two-spool-matching.md) — I caught my own "spools don't talk" over-claim pre-ship
- [Rung 39 two-spool + maps](rung39-two-spool-maps.md) — REFUTES rung 38's prediction while CONFIRMING its verdict
- [Rung 40 two-shaft transient](rung40-two-shaft-transient.md) — rho SPLITS: powerless over stability, decisive over oscillation
- [Rung 41 two-spool surge line](rung41-two-spool-surge-line.md) — corrects rung 36's mechanism while its verdict survives
- [Rung 42 interstage bleed](rung42-interstage-bleed.md) — my "penalizes HP" hypothesis REFUTED; state self-targeting in phi-space
- [Rung 43 two-shaft fuel metering](rung43-two-shaft-fuel-metering.md) — the CURRENCY-CIRCULARITY trap
- [Rung 44 transient surge line](rung44-transient-surge-line.md) — the excursion is SCHEDULE-slaved: ramp-rate-driven, mode-independent
- [Rung 45 transient fuel surge](rung45-transient-fuel-surge.md) — a rho-monotone overshoot NEVER reaches the reference-free object

### 46–52 · the fuel-side limiter family
- [Rung 46 TIT topping governor](rung46-tit-topping-governor.md) — relief SPLITS by spool; rung 35's limits are SEQUENCED in time
- [Rung 47 lagged topping governor](rung47-lagged-topping-governor.md) — a lag is TRAILING-edge, refuting "slow governor reaches earlier"
- [Rung 48 accel schedule](rung48-accel-schedule.md) — UNIFIES 46/47: a limiter rebates a spool IFF it engages upstream of THAT spool's own minimum
- [Rung 49 phi feedback limiter](rung49-phi-feedback-limiter.md) — both edges act on DIFFERENT clocks; both my predicted signs were wrong
- [Rung 50 release edge isolated](rung50-release-edge-isolated.md) — refusing my own first CONFIRMING result was the key move
- [Rung 51 release rate](rung51-release-rate.md) — two-sided bracket, after the pre-registered gate turned out confounded
- [Rung 52 asymmetric lag](rung52-asymmetric-lag.md) — rung 50's debit was an ARTIFACT OF FORCING; surfacing a conflict beat complying

### 53–56 · airflow levers, on the steady matcher
- [Rung 53 variable stator](rung53-variable-stator.md) — a margin is a DISTANCE, so a floor-moving lever makes it coordinate-dependent
- [Rung 54 stator throat](rung54-stator-throat.md) — the constant SPLIT: shape derived, level disclosed
- [Rung 55 stage stack](rung55-stage-stack.md) — the row count has an INTERIOR optimum
- [Rung 56 per-row capacity](rung56-per-row-capacity.md) — a LEVER'S COST is coordinate-dependent too

### 57–63 · schedules on the transient — except 61, which is steady
- [Rung 57 stator schedule on the transient](rung57-stator-schedule-transient.md) — bounds rungs 46-52's timing family as rung 53 bounded their currency
- [Rung 58 composite min-select](rung58-composite-minselect.md) — my headline was refuted by MY OWN table: check the SUM, not the term
- [Rung 59 matched schedule](rung59-matched-schedule.md) — discharges rung 58's concession as VACUOUS
- [Rung 60 matched floor](rung60-matched-floor.md) — the advisor's blocker BECAME the headline
- [Rung 61 stator + bleed](rung61-stator-bleed.md) — a "derived" scaling whose binding constant is mine is NOT derived
- [Rung 62 bleed schedule](rung62-bleed-schedule.md) — the _powers trap: Newton converging on a residual the plant does not use
- [Rung 63 fuel + bleed](rung63-fuel-bleed.md) — I over-claimed 3x; check a quoted number was taken at THIS rung's settings

### 64–68 · the bleed valve, its lag, and cascades
- [Rung 64 phi bleed limiter](rung64-phi-bleed-limiter.md) — two predictions refuted and both BECAME content; the discriminator-before-the-anchor move
- [Rung 65 lagged valve](rung65-lagged-valve.md) — a § 0 pre-check that was itself RK4 instability; not every third state starts at zero
- [Rung 66 two-lag cascade](rung66-two-lag-cascade.md) — my anchor was right for the WRONG reason; check where an extremum sits before quoting it
- [Rung 67 cascade A](rung67-cascade-a.md) — a zero cross-gain is saturation, never decoupling; two-branch registration produced the surprise
- [Rung 68 three loops](rung68-three-loops.md) — check what is INDEPENDENT before quoting it; three of my claims corrected mid-build

### 69–84 · reference splits, rank, and the reader-only rungs
- [Rung 69 reference split](rung69-reference-split.md) — det J was BLIND to the split and c1 the discriminator; a null space ABSORBS a moved start
- [Rung 70 generic split](rung70-generic-split.md) — a predicted NULL refuted into an invariance; I caught a gate computing my own formula twice
- [Rung 71 full split](rung71-full-split.md) — rank independence is NOT constraint independence; my own headline quoted the wrong window
- [Rung 72 shared actuator](rung72-shared-actuator.md) — first seam closed by REFUTING its premise; two instruments silently agreed with themselves
- [Rung 73 applied reference](rung73-applied-reference.md) — a bug that returned a PERFECT confirmation having measured nothing; weakening an instrument was worth 5 orders
- [Rung 74 demand coordinate](rung74-demand-coordinate.md) — a closed-loop difference cannot isolate a forcing; the clip floor was an ACCIDENTAL anti-windup device
- [Rung 75 anti-windup device](rung75-antiwindup-device.md) — the inherited instrument was BLIND and would have refuted the headline having measured nothing
- [Rung 76 fuel-dependent cap](rung76-fuel-dependent-cap.md) — the 0.7% miss on a pre-registered identity WAS the finding; guard the second min-select
- [Rung 77 stiffness ledger](rung77-stiffness-ledger.md) — check UNITS before choosing a normalisation; a stale closure returned a perfect 1.000e+00
- [Rung 78 residual gauge](rung78-residual-gauge.md) — THREE vacuity traps in one section, only COUNTING caught them; `ok` is not a correctness guard
- [Rung 79 state coordinate](rung79-state-coordinate.md) — registering the VACUITY CONDITION beat registering the result
- [Rung 80 split wall](rung80-split-wall.md) — the seam named the wrong NOUN; all three pre-registered predictions were refuted
- [Rung 81 authority clock](rung81-authority-clock.md) — the first grid's 100% was the WEAKER measurement; a pre-check answered the seam before the anchor existed
- [Rung 82 threshold law](rung82-threshold-law.md) — two registered bars were VOID for comparing a physical quantity to a loop count
- [Rung 83 corrector law](rung83-corrector-law.md) — an identity round-trip sold as verification; a bar naming a DIRECTION instead of a POINT died
- [Rung 84 staircase law](rung84-staircase-law.md) — a small integer COUNT cannot carry a RATE; the defective estimator sat in the SHIPPED reader, not just the probe

## Margin sweeps — confirmations, not rungs
- [Rungs 72–77 march audit](rungs72-77-march-audit.md) — a seam's section number does not survive the rungs it points at; a liveness counter on a FROZEN plant reports FULL activity
- [Rung 74 arrest interval](rung74-arrest-interval.md) — the advisor killed my framing with a row from my own table; the number that didn't parse WAS the finding
- [Rung 79 gap margin](rung79-gap-margin.md) — a counter is only as good as the NOUN it counts; a limiter armed at the initial point has no transient
- [Rung 29 π_c margin](rung29-pi-c-margin.md) — verdict confirmed but π_c NOT protective; the ENERGY = INVENTORY × COMPLETION sharpening
- [Rung 29 M0 margin](rung29-M0-margin.md) — monotone-protective (opposite of π_c); the swing-not-headroom correction
- [Rung 28 β margin hardened](rung28-beta-margin-hardened.md) — β exactly pressure-invariant, π_c protective

## Investigated, NEGATIVE — not shipped, not rungs
- [τ_res negative](tau-res-negative.md) — rung 26's seam (a) negative on both counts; confirms rung 26
- [Mixing-scale negative](mixing-scale-negative.md) — revisit only when δ(J) is anchored
- [Turbine-march negative](turbine-march-negative.md) — I_turb≡S because turbine entry is at equilibrium; rung 25's dodge cannot repeat
- [Mixing JICF-anchor negative](mixing-jicf-anchor-negative.md) — confirms rung 22 but rides on a SECOND unanchored exponent
- [pt3 sensor-lag negative](pt3-sensor-lag-negative.md) — corrects rung 48's seam SIGN; the 12%-gap-inside-my-own-band lesson
- [Per-row blading negative](per-row-blading-negative.md) — the request is OVER-DETERMINED (proof); capacity inert on the well-posed anchor
- [Both-edges limiter negative](both-edges-limiter-negative.md) — closes the WHOLE pt3-filter family; UPGRADES rung 48
- [phi-rate limiter negative](phi-rate-limiter-negative.md) — authority INVERTS between LEVEL and DERIVATIVE; the ONE negative carrying a test
