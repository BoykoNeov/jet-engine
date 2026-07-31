# Memory index

One line per entry — the hook only. Full detail lives in the linked file; do not re-expand
these lines (the same rule CLAUDE.md lives under).

## Working agreements
- [Session-end routine](session-end-routine.md) — at end of batch/planning, or "session end": update memory + docs, commit, push to main
- [Git remote setup](git-remote-setup.md) — repo is github.com/BoykoNeov/jet-engine (public), branch main, origin over SSH
- [Always commit and push](always-commit-and-push.md) — auto-commit + push green work without being asked; the gate is bare `pytest` (it runs everything); never gate a docs-only change, never run it "just to be sure"
- [CLAUDE.md is a reference](claude-md-is-a-reference.md) — one-line-per-rung index, detail → docs/rungN-spec.md; a guard test enforces the size budget; bloated to ~200KB twice before
- [Test-suite speed policy](test-suite-speed-policy.md) — ONE gate: `pytest` runs everything (2:18); the tiering's blocker was a COUNT argument that inverts under COST; the LPT scheduler is worth ~26%
- [xdist module-fixture cost](xdist-module-fixture-cost.md) — a module-scoped fixture is rebuilt PER WORKER, so each extra consumer multiplies it; 5 consumers of a 20 s reader added 2:37 to a 2:59 gate
- [Perf: sonic throat + PyPy](perf-sonic-throat-and-pypy.md) — the hot function was hot ALGORITHMICALLY (45 bisections for a linear root), not linguistically; gate 28:18 → 1:55
- [PyPy switch shipped](pypy-switch-shipped.md) — full gate 17:27 → 2:47; SLOW_SECONDS KEPT with its reason inverted (bought time → buys determinism); psutil load-bearing; print precision protects states, not residuals
- [Golden fingerprint gate](golden-fingerprint-gate.md) — the project's ONLY absolute-value gate; goldens are a CPython anchor, NEVER regenerate them under another interpreter; measure a detector's sensitivity, don't assert it
- [Golden gate slice 2](golden-gate-slice2.md) — 26 arms / 8044 values; drift is set by CONDITIONING not by rung, and the reduced-resolution worry measured backwards
- [Visuals artifact](visuals-artifact.md) — docs/visuals/ page published as artifact 56cde230…; update the same URL, regenerate via extract_data.py + build.py

## Shipped rungs
- [Rung 25 finite-rate nozzle](rung25-finite-rate-nozzle.md) — finite-rate nozzle chemistry; inverted into a three-state picture (an irreversible-fast ceiling below the reversible bound)
- [Rung 26 freeze-out](rung26-freeze-out.md) — anchored local Da(T,p) clock; the freeze point MOVES with Tt4, density-driven; refutes rung 25's own seam framing
- [Rung 27 NO freeze-out](rung27-no-freeze-out.md) — the frozen-NO assumption DERIVED (Da_NO≪1 from entry); the kill test INVERTS rung 26
- [Rung 28 coupled NO march](rung28-coupled-no-march.md) — confirms rung 27's verdict, corrects BOTH its reasons; the precedent for editing a shipped rung
- [Rung 29 shifting turbine](rung29-shifting-turbine.md) — frozen turbine EARNED at design, bites hot; the RATIO ≠ ENERGY correction; bound-first method
- [Rung 30 choked nozzle](rung30-choked-nozzle.md) — full expansion NOT earned for a convergent engine; the pressure term rescues 87%
- [Rung 31 off-design matching](rung31-offdesign-matching.md) — FIRST STRUCTURAL rung (pi_c becomes an OUTPUT); tau_t drifts on real gas, const on CPG; reduce-by-construction
- [Rung 32 component maps](rung32-component-maps.md) — CORRECTS rung 31's "choked hardware IS the map": work is map-free but pi_c/mdot/N need it; N enters via speed lines
- [Rung 33 subsonic matching](rung33-subsonic-matching.md) — the INVERSION of rung 31: subsonic tau_t varies even on CPG (structural coupling through pi_c, not p0 — advisor's fix)
- [Rung 34 spool transient](rung34-spool-transient.md) — N becomes a STATE (first DYNAMIC rung); the finding is the ratio tau_fuel/tau_spool, not the tautological I-independent shape
- [Rung 35 fuel metering](rung35-fuel-metering.md) — fuel is the control, Tt4 an OUTPUT; CORRECTS rung 34 (fuel ENLARGES the surge excursion; the two accel limits are COUPLED)
- [Rung 36 surge line](rung36-surge-line.md) — SM thin at LOW power; the zero-new-constant anchor was DEAD so phi_surge is imposed, but the SIGN survives; never gate the crossing
- [Rung 37 combustor dynamics](rung37-combustor-dynamics.md) — the two internal clocks SPLIT: volume-filling confirms rung 34's concession, heat-soak corrects it (a 2nd state, history-dependent, surge-PROTECTIVE)
- [Rung 38 two-spool matching](rung38-two-spool-matching.md) — FIRST TWO-SHAFT rung; rung 31's trick chained twice via a 3rd choked throat A45; I caught my own "spools don't talk" over-claim pre-ship
- [Rung 39 two-spool + maps](rung39-two-spool-maps.md) — REFUTES rung 38's prediction while CONFIRMING its verdict: the map opens ONE arrow HP→LP (pi_LPC cancels); two speeds ⇒ slip
- [Rung 40 two-shaft transient](rung40-two-shaft-transient.md) — the LP map opens a COMPLEX inter-spool mode; rho SPLITS (powerless over stability, decisive over oscillation)
- [Rung 41 two-spool surge line](rung41-two-spool-surge-line.md) — the exposure SPLITS onto the LP; a LIVE zero-new-constant closed form; corrects rung 36's mechanism, its verdict survives
- [Rung 42 interstage bleed](rung42-interstage-bleed.md) — the valve is a DoF on ONE spool (LP yes, HP no); my "penalizes HP" hypothesis REFUTED; state self-targeting in phi-space
- [Rung 43 two-shaft fuel metering](rung43-two-shaft-fuel-metering.md) — the spools sit at DIFFERENT points in ONE overshoot loop so neither clock governs it; the CURRENCY-CIRCULARITY trap
- [Rung 44 transient surge line](rung44-transient-surge-line.md) — LP eats ~1.6-2.2x (rung 41 survives dynamically) but the excursion is SCHEDULE-slaved: ramp-rate-driven, mode-independent
- [Rung 45 transient fuel surge](rung45-transient-fuel-surge.md) — rung 43's rho-monotone overshoot NEVER reaches the reference-free surge object; fuel enlarges the approach, compresses the ratio
- [Rung 46 TIT topping governor](rung46-tit-topping-governor.md) — first fuel-side FEEDBACK; relief SPLITS (rebates the late HP, machine-zero on the early binding LP); rung 35's limits are SEQUENCED in time
- [Rung 47 lagged topping governor](rung47-lagged-topping-governor.md) — a lag is TRAILING-edge, so it REFUTES rung 46's "slow governor reaches earlier"; cost of realism = it breaks the redline hold
- [Rung 48 accel schedule](rung48-accel-schedule.md) — UNIFIES 46/47: a fuel-side limiter rebates a spool IFF it engages upstream of THAT spool's own surge minimum; m is an engagement-TIME dial
- [Rung 49 phi feedback limiter](rung49-phi-feedback-limiter.md) — watches the PROTECTED variable; a limiter acts through BOTH edges on DIFFERENT clocks, so an LP floor DEBITS the HP; both predicted signs were wrong
- [Rung 50 release edge isolated](rung50-release-edge-isolated.md) — the closing edge RELOCATES both spools' minima to itself; rung 48's immunity is TIMING, not clip SHAPE; refusing my own first confirming result was the key move
- [Rung 51 release rate](rung51-release-rate.md) — the debit is NOT a functional of the applied-fuel trajectory (two-sided bracket, after the pre-registered gate turned out confounded)
- [Rung 52 asymmetric lag](rung52-asymmetric-lag.md) — a self-releasing limiter CANNOT DEBIT THE SPOOL IT WATCHES, so rung 50's watched-side debit is an ARTIFACT OF FORCING; surfacing a conflict beat complying
- [Rung 53 variable stator](rung53-variable-stator.md) — the FIRST floor-moving lever; a margin is a DISTANCE, so such a lever makes it coordinate-dependent — BOUNDS rungs 36-52's currency
- [Rung 54 stator throat](rung54-stator-throat.md) — the constant SPLIT (shape derived / level disclosed); BIND-NEVER-RELIEVE; a CONSTRAINT'S SEVERITY is coordinate-dependent too
- [Rung 55 stage stack](rung55-stage-stack.md) — a POSITIONAL lever buys relief from the part it does not move; the row count has an INTERIOR optimum
- [Rung 56 per-row capacity](rung56-per-row-capacity.md) — the binding row MIGRATES with power; the machine's two binding rows differ by END *and* SPOOL; a LEVER'S COST is coordinate-dependent too
- [Rung 57 stator schedule on the transient](rung57-stator-schedule-transient.md) — a wall-moving lever has NO CLOCK, bounding rungs 46-52's timing family as rung 53 bounded their currency
- [Rung 58 composite min-select](rung58-composite-minselect.md) — two levers do not superpose (ONE-WAY); my "inherits a clock" headline was refuted by MY OWN table — check the SUM, not the term
- [Rung 59 matched schedule](rung59-matched-schedule.md) — a schedule's ORDINATE cannot see a stator, only its INDEX can; discharges rung 58's concession as VACUOUS
- [Rung 60 matched floor](rung60-matched-floor.md) — a floor PINS the coordinate it watches, so the composite is a TAUTOLOGY in ANY currency; the advisor's blocker BECAME the headline
- [Rung 61 stator + bleed](rung61-stator-bleed.md) — a compensating lever buys back the COORDINATE, not the BILL; a "derived" scaling whose binding constant is mine is not derived
- [Rung 62 bleed schedule](rung62-bleed-schedule.md) — a state-fed schedule's LOOP has a SIGN; two loops through one state do not compose; the _powers trap (Newton converging on a residual the plant does not use)
- [Rung 63 fuel + bleed](rung63-fuel-bleed.md) — a fuel schedule's TABLE has two guards and only a MASS-extracting lever reaches them; I over-claimed the consequence 3x — check a quoted number was taken at THIS rung's settings
- [Rung 64 phi bleed limiter](rung64-phi-bleed-limiter.md) — a limiter's LAW cannot buy PROTECTION, only its PRICE (the ceiling is b_max, i.e. hardware); two predictions refuted and both BECAME content; the discriminator-before-the-anchor move
- [Rung 65 lagged valve](rung65-lagged-valve.md) — a lag repairs the SOLVE without removing the DEGENERACY; a § 0 pre-check that was itself RK4 instability; not every third state starts at zero
- [Rung 66 two-lag cascade](rung66-two-lag-cascade.md) — two loops on one variable are ONE loop with the rates ADDED (R_q·C_g ≡ 1 identically); my anchor was right for the wrong reason and my own stability floor was unsafe; check where an extremum sits before quoting it
- [Rung 67 cascade A](rung67-cascade-a.md) — one scalar P sets BOTH the ringing window and the damping, so the mode is admissible but UNOBSERVABLE; a zero cross-gain is saturation, never decoupling; two-branch registration is the device that produced the surprise
- [Rung 68 three loops](rung68-three-loops.md) — n loops on one variable are RANK ONE, so only the CYCLIC product is independent; check what's independent before quoting it; three of my own claims corrected by measurement mid-build
- [Rung 69 reference split](rung69-reference-split.md) — the rank counts CONSTRAINTS not loops (zeros = n − m); det J was BLIND to the split and c1 the discriminator; a null space ABSORBS a moved start
- [Rung 70 generic split](rung70-generic-split.md) — the split buys the RANK, the RING needs one lever on TWO walls; a predicted NULL refuted into an invariance; I caught a gate computing my own formula twice
- [Rung 71 full split](rung71-full-split.md) — n=m=3: rank independence is NOT constraint independence (`zeros = n−m` counts gradients, not LIVE loops); det J finally alive and STILL blind to the one new gain; my own headline quoted the wrong window

## Margin sweeps — confirmations, not rungs
- [Rung 29 π_c margin](rung29-pi-c-margin.md) — verdict confirmed at 9.4× but π_c NOT protective; the ENERGY = INVENTORY × COMPLETION sharpening
- [Rung 29 M0 margin](rung29-M0-margin.md) — monotone-protective (opposite of π_c), 8.8×; the delta_h-swing-not-headroom correction; double-edged envelope
- [Rung 28 β margin hardened](rung28-beta-margin-hardened.md) — β exactly pressure-invariant, π_c protective

## Investigated, NEGATIVE — not shipped, not rungs
- [τ_res negative](tau-res-negative.md) — rung 26's seam (a) negative on both counts; confirms rung 26
- [Mixing-scale negative](mixing-scale-negative.md) — the locally-resolved-SCALE ceiling attack; revisit only when δ(J) is anchored
- [Turbine-march negative](turbine-march-negative.md) — I_turb≡S because turbine entry is at equilibrium; rung 25's dodge cannot repeat
- [Mixing JICF-anchor negative](mixing-jicf-anchor-negative.md) — confirms rung 22 but rides on a SECOND unanchored exponent
- [pt3 sensor-lag negative](pt3-sensor-lag-negative.md) — confirms rung 48, corrects its seam's SIGN (a sensor lag engages EARLIER); the 12%-gap-inside-my-own-ds-band lesson
- [Per-row blading negative](per-row-blading-negative.md) — the request is OVER-DETERMINED (proof); capacity inert on the well-posed anchor
- [Both-edges limiter negative](both-edges-limiter-negative.md) — closes the WHOLE pt3-filter family (the ramp is the only clock); UPGRADES rung 48 to the truncated-descent law
- [phi-rate limiter negative](phi-rate-limiter-negative.md) — fuel's authority over phi INVERTS between LEVEL and DERIVATIVE; the ONE negative carrying a test
