# The web sandbox

`turbojet-sandbox.html` — **open it with a double-click.** One self-contained, offline file: change
the engine's design and watch it respond. Every number is computed by this repository's Rust model,
compiled for the browser and embedded in the page (no server, no network, no dependencies).

Plan and decisions: `docs/plans/sandbox-plan.md`. The standard atmosphere's derivation and anchor:
`docs/plans/sandbox-anchor-atmosphere.md`.

## Slice 1 — the design point (shipped 2026-10-06)

**Knobs:** altitude and "warmer than a standard day", linked both ways to the air temperature and
pressure (1976 US Standard Atmosphere; typing a pressure makes the altitude its pressure altitude);
flight Mach number; compressor pressure ratio; turbine-inlet temperature; air mass flow; inlet
recovery (the model applies its supersonic recovery law on top, above Mach 1); compressor and
turbine efficiency, isentropic or polytropic; combustion efficiency; burner and nozzle pressure
ratios; shaft efficiency; nozzle (fully expanded / set exit pressure / fixed convergent); gas model
(rungs 1–6, opening on equilibrium chemistry — the production cycle).

**Readouts:** thrust, specific thrust, TSFC, efficiencies, the station table, the T–s diagram (the
burner and exhaust legs as curves), and **pin & compare** (freeze a design, see every change).

**A design that does not run** shows why, in plain words, beside the model's own message. Cheap
cases are caught before the solve (`sandbox::precheck`); the rest are the model's own assertions,
translated by `sandbox::explain` — whose seven entries are the seven messages a 13 000-design sweep
actually produced, each driven by a test.

## Slice 3 — fly the engine you designed (shipped 2026-10-07)

Press **Fly it**: the design's hardware is frozen (turbine and nozzle throat areas, design flow and
speed), and the throttle and the flight move instead. The engine finds its own pressure ratio,
airflow and shaft speed. One solver: rung 34's spool balance on a rung-32 compressor map, with
rung 36's stall margin read off the solved point (`sandbox::fly`, plan § 10).

**Knobs:** throttle (turbine-inlet temperature); the flight knobs (now moving the flight, not the
design); the gas (opens on the thermally perfect gas, live at ~20 ms a point; the equilibrium gas
takes ~1 s and runs when a slider is let go); the compressor map shape (three invented,
representative shapes); the stall line (a chosen number, not data).

**Readouts:** shaft speed, pressure ratio, airflow, stall margin (labelled "trend only"), the
operating point's map efficiencies and flow coefficient, the station table, the T–s diagram, and a
compressor map: speed lines, the stall line, the running line streamed one throttle at a time, and
the current point.

**Three things the sandbox does that the shipped solvers do not:**
- **The nozzle pushes against the flight's own air pressure.** The matchers keep the design run's
  ambient as the nozzle's back-pressure; every shipped caller flies at the design pressure, so it
  never mattered. The sandbox changes altitude with the hardware frozen, so it sets the back-pressure
  per flight (`sandbox::fly_solver`). At the design pressure this is the shipped solver bit for bit;
  without it, a thin-air point fails outright (`tests/sandbox_fly.rs`, which proves both).
- **The stall margin is read off the solved point** instead of re-solving it (rung 36's
  `surge_margin` solves twice), and its constant-flow half uses a non-crashing copy of the map read,
  held bit-equal to the original wherever that answers.
- **Its failures have their own plain words** (`sandbox::explain_fly`): below idle, overspeed, a low
  throttle in fast flight, no workable speed. Each is driven by a measured failing point, from a
  27 610-call sweep.

**Propulsive efficiency can read above 100 %** in fly mode, for two reasons the page names: the
model's efficiency split counts the jet's speed only, and a convergent nozzle high up releases the jet
well above the outside pressure; and near zero thrust (idle) the fuel's own mass, counted as
accelerated, dominates both the thrust and the jet's energy gain (135 % measured at an unchoked idle
point with the jet at outside pressure). Overall efficiency is unaffected.

## Slice 2 — size the blades (shipped 2026-10-07)

Press **Size the blades**: rung 85 as knobs, on its OWN two-spool engine (two compressors on two
shafts — not the *Design* engine, which has one), at 250 K / 50 kPa / Mach 0.85
(`sandbox_blades.rs`, plan § 11).

**Knobs:** the two compressor pressure ratios and the design turbine-inlet temperature; the gas
(rung 85's perfect gas, or thermally perfect); the map shape (rung 55's five, as equals); the blade
knobs — hub-to-tip ratio, airflow level (design tip Mach), material strength ÷ density (titanium
Ti-6Al-4V the one cited preset), overspeed factor, design flow coefficient, the rounding knob λ, the
droop switch — one set for both spools or one each; the stator lever (one block / every row / front
row) and its spool.

**Readouts:** per spool, which limit binds, both limits' blade speeds, stage counts (continuous and
whole), blade and tip speeds, pre-swirl, design Mach numbers, capacity, the redline; a staircase
chart (stages and redline against the airflow level at λ 0, 1 and yours); and the lever against both
redlines over a throttle grid, streamed one throttle at a time on its own Worker — which a knob move
KILLS mid-point, because one thermally-perfect point can take seconds.

**What it adds to the model:** on a table gas, rung 85's sizing now reads γ and cp at each face's
temperature (before, the gas's scalar 1.4); the perfect gas keeps its path bit for bit
(`tests/rung85.rs`). A lever throttle where the two-spool match fails (nozzle unchoked, the jet below
outside pressure, a turbine past the gas tables — all at low throttle) is refused in plain words
BEFORE the schedule runs, through the matcher's own non-crashing twin.

## Slice 4 (A) — slam the throttle (shipped 2026-10-07)

Inside **Fly it**: move the throttle from a starting setting to the fly throttle and watch the engine
take its time (`sandbox_transient.rs`, plan § 12). The shaft speed is a state that lags (rung 34).
Two ways to move the throttle, side by side: **temperature commanded** (rung 34 — the turbine-inlet
temperature follows the move by fiat) and **fuel metered** (rung 35 — the fuel follows it, the
temperature is an output and overshoots on a slam).

**Knobs:** the starting throttle, how long the move takes and how long the run continues afterwards
(in τ, the shaft's own response time — the model has no rotor inertia, so no seconds), which mode(s).
The move must last a whole number of the march's 0.02 τ steps (measured: one ending between steps, or
an instant step, moves the peak temperature by 20–70 K with the step size).

**Readouts:** turbine temperature, shaft speed, thrust, fuel flow, pressure ratio and flow coefficient
against the stall line, against time, with the steady points before and after; a table (peak
temperature and overshoot, highest pressure ratio, lowest flow coefficient, time to 90 % of the speed
change, the end state); both runs' paths on the compressor map. Each run is computed on its own Worker,
killed when a knob moves. Pin & compare overlays a pinned run.

**A run that stops early is a result.** The model's marcher stops and drops its error; the sandbox
replays the failed step through the same public calls (bit-equal to the march, `tests/sandbox_transient.rs`)
and says why, by kind: the fuel-air ratio passing 0.05, the fuel-metered solver's search edge (the
overshoot outran the model); the unchoked-nozzle solve's known gap; on a commanded power cut, the
airflow search's first trial asking the burner to cool the air (a solver artefact — the fuel-metered cut
runs through); a time step too coarse for a very fast shaft (Mach 3.3). A crash in a steady solve names
the throttle it happened at. The equilibrium gas is refused: the model cannot meter fuel on it.

## Slice 4 (B) — Controls (shipped 2026-10-08)

Press **Controls**: a throttle slam on a SEPARATE two-shaft engine (the *Size the blades* engine at its
opening settings, perfect gas), with the fuel controls and one airflow lever as switches
(`sandbox_controls.rs`, plan § 12, what the build found § 12.10). Both shaft speeds lag; the fuel is metered
and the turbine temperature overshoots.

**Knobs:** map shape (four), the LP shaft's response time against the HP's, the two throttles, how long the
fuel move takes and the run continues. **Switches**, each with its number: turbine temperature limiter and its
lag (rungs 46/47), acceleration schedule (48), stall floor on either compressor, watching the flow coefficient
or — with the stators moving — blade incidence (49/60), realistic fast-in slow-out release (52); one lever:
stators or bleed valve, scheduled on speed (57/62).

**Readouts:** turbine temperature against the redline, both shaft speeds, burnt vs scheduled fuel, each
compressor's flow coefficient against its (moving) stall line and the floor, thrust (the bleed's dumped air
pays its ram drag), the lever's setting; a strip showing which control holds the fuel at each moment; tiles
and a table, with pin & compare. Each run on its own Worker, killed by a knob move.

**A run that stops early is a result**, by kind, re-run step for step through the model's public calls: the
floor could no longer be held; the model's every-step check at the full scheduled fuel had no answer while a
limiter held the fuel below it (the method's limit); the mixture left the fuel solver's range, lean or rich.
With the temperature limiter's lag on, the march does not record its state, and the stop is said without a
cause.

## How it is built

| Piece | What it is |
|---|---|
| `rust/src/sandbox.rs` | The whole bridge, ordinary Rust: settings JSON in → `build_turbojet(…).run(…)` → full-precision JSON out. Tested natively (`tests/sandbox.rs`). |
| `rust/src/sandbox_blades.rs` | Slice 2's ops (`blade_*`), reached through `sandbox::call` (`tests/sandbox_blades.rs`). |
| `rust/src/sandbox_transient.rs` | Slice 4's ops (`slam_defaults`, `slam`), reached through `sandbox::call` (`tests/sandbox_transient.rs`). |
| `rust/src/sandbox_controls.rs` | Slice 4 (B)'s ops (`controls_defaults`, `controls`), reached through `sandbox::call` (`tests/sandbox_controls.rs`). |
| `rust/src/atmosphere.rs` | The 1976 standard atmosphere (`tests/atmosphere.rs`, held to the published table). |
| `rust/sandbox-wasm/` | A tiny separate crate: only the browser exports around `sandbox::call`, and the panic hook that keeps a crash's message readable. Not built by `cargo test`. |
| `template.html` | The page; `/*__SANDBOX_WASM_B64__*/` receives the build as base64. |
| `turbojet-sandbox.html` | The built page — committed, like the charts and cutaway pages. |

**Rebuild after any change that can reach the cycle (`rust/src/`) or the template:**

```
powershell -File rust\sandbox-wasm\build.ps1
```

then commit the page. The gate fails while it is stale.

**Expected red, not a broken model:** the "page is current" check compares the browser build's
BYTES, and those carry the compiler version and the source files' absolute paths (in the model's
crash messages). After a `rustup update`, or with the repo checked out somewhere else, the gate goes
red until the page is rebuilt and committed — the numbers have not changed.

## What binds the page to the model

- `tests/sandbox.rs` — the sandbox reproduces the CLI's design runs **bit for bit**; its T–s points
  reduce to `visuals::cycle_points` where that function is right; ideal components make no entropy
  on every gas (on the perfect gas, exactly the leftover its rounded rung-1 constants predict).
- `tests/sandbox_fly.rs` — slice 3: flown at its own design point the engine lands on its design
  (every gas); at the design pressure the fly point IS rung 34's equilibrium, bit for bit; the
  back-pressure follows the flight (a pressure-homogeneity check, and proof the shipped back-pressure
  fails there); the stall reading IS rung 36's margin; each fly pre-check and plain-words branch is
  driven by its case.
- `tests/sandbox_blades.rs` — slice 2: at its defaults the view IS rung 85's default cell, and each
  lever request IS `Machine::schedule`, bit for bit, against a rig built in the test (never the
  module's own copy) and against rung 85's published rows; each refusal driven by its case.
- `tests/sandbox_transient.rs` — slice 4: the slam IS rung 34's `integrate` and rung 35's
  `integrate_fuel`, bit for bit; both steady ends are slice 3's fly points; a held throttle holds and
  a settling run approaches its end (measured bars); the stop replay reproduces the march step for step;
  every kind of early stop the crash map found is driven by a request that raises it.
- `tests/sandbox_controls.rs` — slice 4 (B): every run IS rung 43's `integrate_fuel` on rung 62's machine,
  built in the test from literal numbers; every switch is checked where it binds and the holder names it; a
  dormant redline is the bare run; the acceleration table is `accel_schedule`'s, bit for bit; the re-read and
  the stop re-run reproduce the march point for point; each stop kind, refusal and precheck driven by its case.
- `tests/sandbox_page.rs` — the joints: the page is its template + its build; every element the
  script looks up exists; every knob (design and fly) is a setting and back; the dropdowns offer the
  model's choices; every export and op the page uses exists.
- `rust/sandbox-wasm/check.ps1`, **in the gate** (`rust/test-all.ps1`) — rebuilds the page and
  compares it with the committed one; runs the page's build against the native model over a grid
  (`check.mjs`, design and fly requests); and drives the real page in a headless Chrome
  (`browser.mjs`: Worker, panels, linked knobs, pin & compare, and fly it — the design point, the
  running line, below idle, the flight knobs moving the flight, the slow gas running on release; and size
  the blades — the default cell, the streamed lever, unreached and unmodelled throttles, blades that cannot
  be built, a slow sweep stopped by a knob move; and the slam — the opening run against native, the
  fuel overshoot, an early stop in words, a below-idle start naming its throttle, a run stopped by a knob
  move, the refusal on the equilibrium gas; and Controls — the opening run against native, the temperature
  limiter switched on, pin & compare, a refused combination, an early stop in words, the incidence floor, a run
  stopped by a knob move).
  Needs Node ≥ 22 and Chrome. `SANDBOX_SHOTS=<folder>` also saves light, dark and phone screenshots
  of the fly view — look at them: a layout bug passes every behaviour check.

**Browser vs native numbers.** Not bit-identical, by measurement: the browser uses Rust's bundled
maths library, Windows its own. On the perfect gas they agree to ~1e-14; on every table gas the
temperature solver (`gas::SOLVE_TOL` = 1e-11) stops at a different iterate inside its tolerance, so
they agree to ~1e-10 (worst at the constant-flow stall margin, a ratio minus 1). `check.mjs` holds them to 1e-13 / 1e-9 (entropy absolutely: 1e-10 / 1e-6 J/(kg·K)).
The user accepted this difference (plan § 9.4); bit-exactness stays the CLI's job. The Controls runs, though on
the perfect gas, come out of iterated solves (stopping at 1e-12), so they get their own bar: 1e-11 (worst 2.1e-13).

## Next slices

The combustor (slice 5) — plan § 5.
