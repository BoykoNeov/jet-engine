# The web sandbox — plan

**Status: SLICE 1 BUILT 2026-10-06** — `docs/sandbox/` (what shipped, and how it is gated: its
README). **Slice 3 (off-design, "Fly it") BUILT 2026-10-07 — § 10**; what the build found beyond
the plan is § 10.8. Slice 2 not started. Drafted 2026-10-06, after rung 85 shipped. Direction (user,
2026-10-06): the project becomes a **sandbox** — change the engine's components and design numbers
and watch it respond — delivered as an **interactive web page running the Rust model live**, beside
the charts page and the cutaway (`docs/visuals/`).

§ 9 holds the user's answers. Built beyond the plan: the burner and exhaust legs of the T–s
diagram as curves (`sandbox::ts_curves`), after a screenshot showed a straight 3 → 4 line.

## 1. What the spike measured (2026-10-06)

A throwaway crate (`W:\temp\claude\jet-wasm-spike`, not in the repo) linked this crate by path,
exported two plain functions (specific thrust, TSFC at a given gas model / `π_c` / `Tt4`, the panels'
flight point and `real_losses()`), was built for `wasm32-unknown-unknown`, and run under Node 24
beside the same calls compiled natively.

| Question | Answer |
|---|---|
| Does the crate build for the browser with **no new dependency**? | **Yes.** `wasm32-unknown-unknown` is installed; the library compiled unchanged. Nothing in `src/` but `main.rs` touches files, clocks or the environment. |
| Does it **run**? | **Yes** — perfect, thermally perfect, reacting and equilibrium gas, under Node's WebAssembly engine (Fork B not tried). |
| Size | **120 KB** `.wasm` (≈160 KB as base64) for the design-point cycle. |
| Speed, one design-point run | perfect gas 1–3 µs native / ~1–100 µs wasm (first call warms); thermally perfect ~12 µs / ~45 µs; reacting ~23 µs / ~57 µs; **equilibrium ~2.3 ms / ~1.4–2.6 ms**. Live update while a slider drags is affordable for every gas model at the design point. |
| Bit-identical to native? | **11 of 12 cases, both outputs.** One case differs in the last ~9 units of the last place (perfect gas, `π_c`=4, `Tt4`=1100 K: specific thrust `…76b7` native vs `…76bf` wasm). Cause: the browser build uses Rust's bundled maths library, the Windows build the system one. Relative size ~1e-15 — invisible on a page, but it means **a browser number is not a bit-exact copy of the CLI's**. |
| What does a crash look like? | A Rust panic becomes a JavaScript `RuntimeError` (a trap). A panic hook that stores the message lets JS read it **after** the trap: `"burner fixed point f=g(f) did not converge in 100 steps"` for `π_c`=30, `Tt4`=500 K. In the spike the same module even answered the next call, but the plan does **not** rely on that (a trap can leave allocator or lock state half-updated): **every trap re-creates the module**, which at 120 KB is cheap. |

## 2. How the model reaches the browser

- **No wasm-bindgen** (the crate's no-dependency rule). Plain `#[no_mangle] extern "C"` exports, an
  `alloc`/`free` pair for passing text, and a hand-written JS loader (~50 lines).
- **Settings in, results out, as JSON text.** The crate already parses and prints JSON
  (`visuals::Json`), so the bridge needs no new serialisation code. **The result carries FULL
  precision** — never `r6`/`cycle_blocks`, whose 6-figure rounding would make the Node check (§ 6)
  compare two rounded strings and "measure" a zero drift. Checked: `Json` writes a float with
  `repr_f64` (shortest round-trip) and reads it with Rust's correctly-rounded `parse`, so a float
  survives print→parse exactly — `tests/sandbox.rs` pins that, so the bit-for-bit gate tests the
  model and not the printer.
- **Where the code lives — split in two:**
  1. `rust/src/sandbox.rs` — a **pure** module in the main crate: `run_json(settings: &str) -> String`
     (parse settings, build the engine, run it, print the result). Ordinary Rust, so it is tested
     natively by the normal gate like any module. No `extern "C"`, no `unsafe`.
  2. `rust/sandbox-wasm/` — a **tiny sibling crate** (its own `Cargo.toml`, `crate-type = ["cdylib"]`,
     a path dependency on `turbojet`). Only the export shell: `init` (panic hook), `alloc`, `free`,
     `run`, `msg_ptr/len`. **Not a workspace member**, so `cargo test` of the main crate never
     builds it and the 193-program gate is unchanged.
- **One self-contained HTML file**, the `.wasm` embedded as base64: it opens from a double-click
  (`file://` refuses a fetch of a sibling `.wasm`), stays fully offline like the two existing
  pages, and needs no answer to whether a host serves `application/wasm`.
- **The model runs in a Web Worker** (built from a Blob URL inside the same file), so a slow solve
  never freezes the page; the newest slider value wins, stale answers are dropped.
- **Build:** a script `rust/sandbox-wasm/build.ps1` — build the `.wasm` (below-normal priority),
  splice it into `docs/sandbox/template.html` as `docs/sandbox/turbojet-sandbox.html` (a base64
  encoder is ~20 lines; no dependency), then run the Node check (§ 6).

## 3. A design that does not run is a RESULT

The conservation asserts stay (working contract) — and a slider will trip them often. So:

1. **Pre-checks with plain words**, before the solve, for the failures a user will actually hit:
   burner exit not hotter than compressor exit ("the burner would have to cool the air"), turbine
   unable to drive the compressor, an exit pressure the nozzle cannot reach, a mutually-exclusive
   pair of efficiency knobs. Each is a `Result` from `sandbox.rs`, not a panic.
2. **Anything else traps**; the loader reads the stored panic message, re-creates the module, and
   the page shows *"This design does not run:"* + the model's own message, verbatim, with the
   settings that caused it. The last good result stays on screen, greyed, for comparison.
3. **No `try_run` is added to the engine for slice 1** — the trap path covers it without touching
   any gated code. (Off-design slices may need the matchers' existing `try_` twins; decide there.)

## 4. Slice 1 — the design-point sandbox

**Knobs.**
- Flight: ambient `T0`, `p0` (or an altitude knob mapping to a standard atmosphere — § 9), `M0`.
- Cycle: `π_c`, `Tt4`, air mass flow (so thrust shows in newtons, not only per kg/s).
- Component losses: `π_d`, `η_c` **or** `e_c` (a toggle — the code forbids both), `η_b`, `π_b`,
  `η_t` **or** `e_t`, `η_m`, `π_n`.
- Nozzle: fully expanded · specified exit pressure · fixed convergent (rung 30 — the flow picks `p9`).
- Gas model: perfect (rung 1–2) · thermally perfect (3) · reacting (4) · Fork B (5) · equilibrium
  (6, the production cycle). Shown openly, because the existing pages use the perfect gas while
  the CLI's production cycle is equilibrium.
- **The inlet trap.** `Losses.pi_d` is set once, at one flight Mach number. If `M0` is a slider
  and `π_d` is a plain knob, the inlet stops reacting to speed above Mach 1. So the knob is
  `π_d,max`, and the model is fed `π_d,max · ram_recovery(M0)` (the code's own MIL-E-5008B law),
  shown as a derived readout.

**Readouts — every result, no verdicts.** The full station table (Tt, pt, f at 0/2/3/4/5/9);
performance (specific thrust, thrust, TSFC, Brayton / thermal / propulsive / overall efficiency,
`V9`, `M9`, `T9`, `p9`); the T–s diagram; and a **pin & compare** button — freeze the current design and show every number's change
against it, which is the sandbox's main way of "seeing what a part does".

**The T–s diagram needs new code.** `visuals::cycle_points` (`src/visuals.rs:355`) cannot be
reused: it computes entropy with `Gas::default()` (the perfect gas) and from `panels::flight()`'s
fixed datum, whatever engine it is handed — so it is right only for the perfect gas at the panels'
flight point. Slice 1 adds `sandbox::ts_points(gas, result, flight)`: entropy on the run's OWN gas
(the closed form on the perfect gas; the NASA `φ(T)` integral, cold section for stations 0–3 and
the hot section at the run's `f` for 4–9) from the run's OWN ambient datum. Gates: it reduces to
`cycle_points` bit-for-bit on the perfect gas at the panels' flight; an ideal (isentropic)
compressor and turbine give zero entropy change on the thermally-perfect gas. **Honest
concession to state on the page:** the burner changes the gas's composition, so the jump in
entropy across it mixes heat addition with a change of reference mixture.

**Not in slice 1:** off-design, two spools, blade speeds, NOx, transients.

## 5. Later slices (each planned when reached; order is a § 9 question)

- **Slice 2 — rung 85's blade-speed knobs** on the two-spool design: hub-to-tip ratio, tip-Mach
  level, material, overspeed factor, design flow coefficient, the rounding knob `λ` and the droop
  switch ⇒ stage counts, blade speeds, redline, which wall binds. Needs the two-spool design run
  and `blade_speed::build`; time it first.
- **Slice 3 — off-design:** freeze the hardware at the design, then move the throttle (`Tt4`) and
  flight ⇒ the running line, spool speed, surge margin (rungs 31/32/36, and 38/39 for two spools).
  Matchers nest solves; time them before promising live dragging.
- **Slice 4 — the transient:** throttle steps with the fuel limiters and stator/bleed levers
  (rungs 34–68) as switches — a time plot rather than a point.
- **Slice 5 — the combustor:** the NOx diagnostics (rungs 7–24) as knobs on the burner zoning.

## 6. Gates

- **`tests/sandbox.rs`** (native, in the normal gate): `run_json` on fixed settings equals a direct
  `build_turbojet(…).run(…)` **bit for bit** (same machine, same maths library); settings JSON
  round-trips; each pre-check fires on its case and *only* it; the inlet trap — `M0` above 1 moves
  `π_d`.
- **Page joints**, as `tests/visuals.rs` does for the other two: every export the JS calls exists in
  the shell; every `getElementById` target is declared; the built page is the splice of its
  template and the committed `.wasm`.
- **Node check** (in `build.ps1`): the `.wasm` run on a fixed grid of settings, compared with
  `run_json` natively. The bar is **measured, not typed** — § 1 found last-place differences, so the
  bar is a relative tolerance set from the measured worst case on that grid (with headroom stated),
  not bit equality. Whether this step joins `rust/test-all.ps1` is § 9.
- **The page in a real browser** — the one layer nothing above reaches (the loader, the Worker, the
  "does not run" panel, the DOM). The project's pages once had no test and *looked* fine
  (`memory/visuals-model-binding.md`). A script serves `docs/sandbox/` on a local port (the
  browser tooling refuses `file://`), drives a headless Chrome with its OWN profile folder under
  `W:\temp\claude` and its OWN debugging port, PID recorded at launch: load the page, read the
  design-point thrust off the DOM and compare with `run_json`; set an impossible design and check
  the "does not run" panel shows and the next good design recovers. Shut down through its own port,
  then by that PID only — never by name.
- **Unchanged by construction:** `cli_golden`, `visuals`, every oracle — the sandbox adds files and
  one module, and edits none of the model. Checked 2026-10-06: no test in `rust/tests/` lists
  `src/`'s modules (no `read_dir`, no module census), so a non-rung `sandbox.rs` trips no guard.

## 7. Risks still open

- **Hosting, and the two browser features the page leans on.** Whether a page's security policy
  lets (a) WebAssembly compile and (b) a Worker start from a `blob:` URL is **unmeasured** — for
  the claude.ai artifact host AND for a double-clicked `file://` page (browsers treat `file://`
  origins strictly). Check both, in both places, with a tiny test page before slice 1's template is
  written. Fallback if (b) is refused: run the model on the main thread (fine at ≤3 ms per run).
- **Equilibrium near its edges** (very lean / very hot) may be slower or fail more than the
  three spike points show — the Worker and the trap path absorb it, but time a grid, not three.
- **Ranges.** Slider limits decide how often a user lands on a non-running design; set them from
  where the model holds, measured on a grid.

## 8. Order of work for slice 1

0. The hosting / `file://` / `blob:`-Worker check (§ 7) — it decides the page's shape.
1. `sandbox.rs` + `tests/sandbox.rs` (settings → result at full precision, pre-checks, inlet law,
   the gas-aware `ts_points`).
2. `rust/sandbox-wasm/` shell + Node check; measure the bar.
3. `docs/sandbox/template.html` — knobs, readouts, T–s, pin & compare, the "does not run" panel.
4. Joint tests; the headless-browser test; `build.ps1`; README beside the page.
5. Publish if the user wants it published (§ 9).

## 9. Decisions for the user

**ANSWERED (user, 2026-10-06):**
1. **Both, connected to one another.** Built as the aviation convention: *altitude* + *deviation
   from the standard day*, wired both ways to raw `T0`/`p0` — move altitude and `T0`/`p0` follow
   the standard atmosphere (plus the deviation); type `p0` and altitude becomes its PRESSURE
   altitude; type `T0` and the deviation becomes `T0 − T_std(alt)`. The page opens on the panels'
   design point (250 K, 50 kPa ⇒ ≈5.6 km, ≈−2 K off standard). The standard atmosphere is
   anchored to the published 1976 table (`docs/plans/sandbox-anchor-atmosphere.md`).
2. **Realistic** — the page opens on the equilibrium gas.
3. **Local file only** — no claude.ai publish. § 7's hosting check narrows to `file://`.
4. **OK** — browser numbers may drift in the last digits; the Node check uses a measured tolerance.
5. **Include it** — the wasm build + Node check join `rust/test-all.ps1`.
6. **Doesn't matter, both** — slices 2 (blade speeds) and 3 (off-design) both follow, either order.

The original questions, as asked:

1. **Slice 1's knobs** — is § 4 the right first set? In particular: flight as raw ambient
   temperature + pressure, or a single **altitude** knob (standard atmosphere)? *Recommended:
   altitude, with T0/p0 shown as readouts and an "override" switch.* **Cost:** a standard
   atmosphere is NEW PHYSICS, not wiring — under the working contract it needs a cited source
   (the 1976 US Standard Atmosphere), a written derivation, and a test against the published
   table. Raw `T0`/`p0` costs nothing new.
2. **Default gas model** when the page opens. *Recommended: equilibrium (the real production
   cycle, ~2 ms per run), with perfect gas one click away for the textbook numbers.* This rests on
   § 4's gas-aware T–s diagram, which slice 1 builds anyway (every non-perfect gas needs it).
3. **Local file only, or also a published (private) claude.ai page** like the charts and cutaway?
   *Recommended: both, once § 7's hosting check passes.*
4. **Browser numbers may differ from the CLI's in the last digit or so** (§ 1). Acceptable, with
   the measured tolerance gate? *Recommended: yes — bit-exactness stays the CLI's job.*
5. **Node check in the full gate** (`test-all.ps1`, adds a ~1 min wasm build) or only in
   `build.ps1`? *Recommended: in the gate, so a model change that breaks the page is caught.*
6. **Slice order after slice 1** — blade speeds (2) or off-design (3) first?

## 10. Slice 3 — off-design ("fly the engine you designed")

Planned 2026-10-06. The user designs an engine on slice 1's knobs, then **freezes its hardware** and
moves the throttle and the flight condition. The engine finds its own operating point: pressure
ratio, airflow, shaft speed, thrust, fuel burn, stall margin. Slice 1 *redesigns* the engine at every
knob move; slice 3 *flies* one engine.

### 10.1 Which solver — one, measured

Three off-design solvers exist. Run over 4 gases × 3 flights × 12 throttles (2026-10-06, scratch
crates `W:\temp\claude\jet-offdesign-timing`, `…-wasm`, not in the repo):

| Solver | Gives | Verdict |
|---|---|---|
| rung 31/33 `OffDesignMatcher` | `π_c`, airflow, thrust; both nozzle branches | no shaft speed, no map, no stall margin |
| rung 32 `MapMatcher` | + shaft speed, map efficiencies | **does not dispatch to the unchoked nozzle** (`map.rs` module note 3): at 11 km, Mach 0.85, `Tt4` 700–1000 it returned points flagged unchoked but solved on the two-choked-throat pin — numbers its own flag voids. **Kept off the page.** |
| rung 34 `SpoolTransient::equilibrium` + rung 36 `surge_margin` | shaft speed, map, both nozzle branches, stall margin (choked branch) | **the one the page uses** |

### 10.2 Speed — what decides the page's shape

Rung 34 + 36, a **fresh solver per point** (see 10.5), per operating point:

| Gas | native | browser build (Node) |
|---|---|---|
| perfect | < 1 ms | < 1 ms |
| thermally perfect | 4–13 ms | **10–45 ms** — follows a slider |
| reacting / Fork B | 30–470 ms | 40–360 ms |
| equilibrium (the production gas) | 0.4–1.4 s | **0.5–2.5 s** (median 0.75 s) |

Native ms are thread CPU cycles over a calibrated clock (the calibration itself moved 2.7–4.2 GHz
between runs, so compare solvers, not absolute times); browser ms are wall clock under Node 24.
Rung 31 alone, for reference: ~24 ms median on equilibrium, but up to 1.6 s where its joint
`(f, pt4)` loop runs all 200 passes (the documented unmeetable stopping rule, `matcher.rs` header
note 3 — not to be "fixed").

**The equilibrium cost is the chemistry, not bookkeeping.** A behaviour-identical change to the gas
memo caches (a hash lookup instead of a linear scan, `gas.rs` `ReactingSection` /
`EquilibriumSection`) made reacting and Fork B ~10× faster — down to thermally-perfect speed — and
left equilibrium unchanged (measured on a scratch copy, against the thermally-perfect rows it does
not touch). It is a model-code change, so it ships only with the full gate proving every output
bit-identical; offered separately (§ 10.7 Q3).

### 10.3 Knobs and readouts

**Mode switch:** *Design* (slice 1) ↔ *Fly it*. Entering *Fly it* captures the current design as
hardware (turbine and nozzle throat areas, design flow and speed references).

**Fly-it knobs:** throttle (`Tt4`); the flight knobs (altitude, deviation, Mach — the same linked set);
the compressor map shape (the three surge-realistic shapes `surge_flow` / `surge_pressure` /
`surge_tilted`, shown as equals, as rung 85's shapes are); the **stall-line position** `φ_surge`.

**Readouts — every result, no verdicts:** thrust, TSFC, fuel flow; compressor pressure ratio; airflow
and its lapse from design; shaft speed (% of design) and corrected speed; nozzle choked / unchoked;
stall margin (two definitions, constant speed and constant flow) **beside the operating and stall
flow coefficients**; the station table; the T–s diagram (slice 1's `ts_points` on the rebuilt run);
pin & compare across throttle/flight moves; and a **compressor-map chart** — pressure ratio vs
corrected flow, the running line swept over the throttle range, the stall line, the current point.

**The stall margin is labelled for what it is** (user's forwarded note, 2026-10-06; rung 36's own
disclaimer): the model has no measured compressor map, the stall line is one chosen number, and only
its TREND (thin at low power) is load-bearing. Measured on the default shape at `φ_surge` = 0.65 the
margin reads 7 % at idle and 66–110 % at full power — no reader would believe the second. Hence a
knob, the map shape beside it, and the plain-words label: *"depends on the stall line you set and
the map shape you pick — read the trend, not the size."*

### 10.4 What the solver refuses, turned into plain words

Each is a pre-check (before the solve) or an `explain()` entry driven by a test, as slice 1's:
- **Fixed convergent nozzle only** — capturing hardware needs a throat. *Fly it* re-runs the design
  with a convergent nozzle, so its design numbers differ from slice 1's default "fully expanded"
  design; the page says so and shows both.
- **Isentropic efficiencies only** — the matcher asserts on polytropic. Pre-check with plain words.
- **Stall margin on the choked branch only** — it asserts elsewhere, i.e. exactly where the margin is
  thinnest. Shown as "not modelled while the nozzle is unchoked", never a blank.
- **Stall line past the operating point** (`φ_surge ≥ φ_op`, an assert) — a new trap the knob
  creates; pre-checked: "the stall line you set is at or beyond this operating point".
- **Below idle** is a physical result: two distinct messages (rung 33's subsonic "does not bracket",
  rung 34's "equilibrium does not bracket") — both into `explain()` with driving tests. At cruise
  (11 km, Mach 0.85) `Tt4` ≤ 800 K is below idle on every gas; the throttle slider's range comes
  from a measured envelope, not a guess.
- **Mach 0** stays refused, as in slice 1 (the efficiency bookkeeping divides by flight speed).

### 10.5 Model side (`sandbox.rs`, no model code edited)

- New op `{"op":"fly", "design":{…}, "throttle":…, "flight":{…}, "map":…, "phi_surge":…}`.
- Builds a **fresh** `SpoolTransient` per request: the reacting/equilibrium memo caches never
  shrink, so a long-lived solver slows as it runs (measured: the same kind of point 3 ms early,
  300 ms+ later, in cycles). Capture costs ~1.5 ms — cheaper than any cache policy.
- The station table comes from `OffDesignMatcher::rebuild` at the equilibrium's `(π_c, ṁ, η_c, η_t)`
  — the same forward rebuild every matcher ends with (crate-visible; `sandbox.rs` is in the crate).
- The running line: `op:"running_line"` sweeps the throttle grid; on the slow gases the page streams
  it point by point with progress, and drops a stale sweep when the hardware or flight changes.

### 10.6 Gates

- **Reduce to the design point:** flying at the design flight and design `Tt4` returns shaft speed 1,
  `π_c` = design, airflow = design — the project's reduce-to-prior spine, in the sandbox.
- **Bit-equal to the direct call:** `op:"fly"` ≡ `SpoolTransient::equilibrium` + `surge_margin`, and
  the rebuilt station table's thrust ≡ the equilibrium's `thrust` (a joint between two code paths).
- Each pre-check fires on its case and only it; each new `explain()` entry driven by a design that
  raises it.
- The browser grid (`check.mjs`) gains fly requests (bar per slice 1's mechanism, re-measured on the
  real grid); the headless-browser drive gains a mode switch, a throttle move, a below-idle
  throttle, and screenshots (light / dark / 390 px) — slice 1's lesson.

### 10.7 Questions for the user

**ANSWERED (user, 2026-10-06): all three as recommended** — (1) a: *Fly it* opens on the
thermally-perfect gas, live; equilibrium one click away, computed on release with progress.
(2) the stall-line knob + map-shape choice, labelled. (3) yes — the memo-cache speed-up ships FIRST,
as its own commit, proven bit-identical by the full gate.

1. **Speed on the realistic gas.** Equilibrium off-design takes 0.5–2.5 s per point in the browser;
   thermally perfect 10–45 ms. (a) *Fly it* opens on the thermally-perfect gas (live dragging), the
   equilibrium gas one click away and computed when the slider is released, with a progress mark —
   *recommended*; (b) equilibrium default, computed on release; (c) offer only the fast gases.
2. **Stall margin** — a knob for the stall line + a map-shape choice, labelled as above
   (*recommended*); or show only the trend (an arrow, no percentage); or leave it off.
3. **The memo-cache speed-up** (§ 10.2) — ship it now as its own change (full gate must stay
   bit-identical; reacting/Fork B ~10× faster everywhere, not only the page), or leave it.
   **Shipped 2026-10-07 (b3a5e3b), full gate bit-identical.**

### 10.8 What the build found beyond the plan (2026-10-07)

- **The off-design nozzle's back-pressure is the DESIGN ambient.** `OffDesignMatcher::p_ambient` is
  captured from the design run and used only as the nozzle's back-pressure (every use in
  `matcher.rs` / `spool.rs`); the thrust's pressure term reads the flight's `p0`. Every shipped
  caller flies off design at the design `p0` (rung 33's `p0` test re-captures per `p0`), so it never
  mattered. The sandbox is the first to change altitude with the hardware frozen: `fly_solver` sets
  the back-pressure to the flight's `p0`. At the design `p0` it is the shipped solver bit for bit;
  at 12.5 kPa without it, the solve fails outright (both pinned, `tests/sandbox_fly.rs`, and the
  override removed once to see that test go red). A model-side fix would be bit-identical for every
  shipped caller — a candidate seam, not done here.
- **Rung 36's constant-flow margin crashes the gas tables with a low stall line** (it reads a speed
  line at `n·φ_op/φ_surge`, >4× design at `φ_surge` 0.3). The sandbox reads it through
  `try_pi_c_map`, a copy of `pi_c_map` with the fallible inverse, held bit-equal to the original at
  >1000 map points and refusing exactly where it panics. A 160 % speed cap was tried first and
  rejected: at the default stall line it hid the reading just above design power.
- **Rung 36's `surge_margin` re-solves the equilibrium**; `stall_reading` reads the solved point
  (bit-equal to `surge_margin`, half the cost).
- **"Equilibrium does not bracket" means opposite things.** The 27 610-call sweep found it for the
  shaft slowing at every workable speed (below idle), speeding up to the search's top (overspeed),
  and speeding up where only the SLOWEST speeds are workable (fast flight, low throttle: above them
  the compressor heats the air past the throttle setting). `explain_fly` reads the search's ends.
  A throttle not above the compressor-face temperature is now a pre-check.
- **Propulsive efficiency passes 100 %** with an underexpanded convergent nozzle: rung 2's split
  counts jet KINETIC energy only, while the thrust carries the pressure push. It ALSO passes 100 % near
  zero thrust with the jet fully expanded (135 % at an unchoked idle point): with `V9 ≈ V0` the fuel
  mass term `f·V0` dominates both `F` and `ΔKE/2 ≈ ½f·V0² + V0·ΔV`, so the ratio tends to 2. Both
  causes labelled on the page (slice 1 showed the first too, unlabelled); the model is unchanged.
- **The frozen line shows both designs** (the plan's own § 10.4 promise, missed by the first build
  and caught on review): the design as set (its nozzle, its gas) and the convergent re-run.
- **Speed labels re-measured in the page's build after the speed-up** (fly point, median): perfect
  0.7 ms, thermally perfect 21 ms, reacting 32 ms, Fork B 27 ms, equilibrium 0.42 s (max 0.87 s) —
  the reacting gases now run live; only equilibrium runs on release.
- **Measured browser-vs-native drift with the fly requests:** perfect 6.8e-15, table 9.4e-11 (worst
  at the constant-flow stall margin). Slice 1's bars stand.
