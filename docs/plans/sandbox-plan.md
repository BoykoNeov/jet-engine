# The web sandbox — plan

**Status: SLICE 1 BUILT 2026-10-06** — `docs/sandbox/` (what shipped, and how it is gated: its
README). **Slice 3 (off-design, "Fly it") BUILT 2026-10-07 — § 10**; what the build found beyond
the plan is § 10.8. **Slice 2 (blade speeds, "Size the blades") BUILT 2026-10-07 — § 11**; what the build found
beyond the plan is § 11.8. **Slice 4 (the transient) PLANNED 2026-10-07 — § 12, answered (§ 12.8); part (A), the slam, BUILT
2026-10-07 — what the build found is § 12.9; part (B), the Controls view, BUILT 2026-10-08 — § 12.10.**
**Slice 5 (the combustor) PLANNED + ANSWERED 2026-10-08 — § 13; BUILT 2026-10-10 — what the build found is § 13.9.** Drafted 2026-10-06, after rung 85 shipped. Direction (user,
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
- **Slice 5 — the combustor:** the NOx diagnostics (rungs 7–24) as knobs on the burner zoning. BUILT 2026-10-10 — § 13.

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
  shipped caller — a candidate seam, not done here. **Done 2026-10-16 — § 26.**
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

## 11. Slice 2 — the blade speeds ("size the blades")

Planned 2026-10-07. Rung 85 as a page: the user picks how the compressor blades are designed (hub
size, how fast the air may meet the blade tips, the material, the overspeed rule, the design flow
coefficient, who absorbs the whole-row rounding) and watches the stage counts, blade speeds and the
REDLINE come out — then moves a stator lever and sees whether the shaft passes its redline.

### 11.1 Which engine — a second one, said plainly

`blade_speed::build` takes a `TwoSpoolEngine`; the page's *Design* engine is single-spool. So slice 2
adds a **separate view, *Blades*, with its own two-spool engine** — rung 85's rig (rung 55's): the
perfect gas (cold 1.4 / 1004, hot 1.3 / 1239, `r_c = 0.4/1.4·1004` — the `tests/rung85.rs` rig, not
the panel's `cpg13`, whose `r_c` is 286.9), `π_LPC` 3 × `π_HPC` 6, `Tt4` 1500, convergent nozzle,
the panels' losses, flight 250 K / 50 kPa / Mach 0.85. The page says in words that this is not the
*Design* engine. The design knobs worth offering (the probe below): the two pressure ratios — they
move `K` and `R` hard (`π` 1.5/15 ⇒ `K` 1/8, `R` 1.71/1.34) — and the design `Tt4`.

### 11.2 Measured (2026-10-07, scratch crate `W:\temp\claude\jet-blade-timing`, not in the repo)

Native, thread cycles over a calibrated clock (4.17 GHz):

| Call | Cost | Page use |
|---|---|---|
| `build` (both spools sized) | **~5 µs** | follows every slider |
| one lever reading, one-block (`Lumped`) lever | 5–8 ms | |
| one lever reading, row-by-row (`AllRows` / `FrontRow`) | 11–82 ms (steep shape worst) | |
| one lumped reading, thermally perfect gas | 273 ms | — |
| one lumped reading, reacting gas | 421 ms | — |
| one lumped reading, **equilibrium** gas | **13 s** | — |

(The 1500 K readings cost 0 ms only because they ARE the design point.) **Through the page's own
requests, over the whole knob box** (3 000 random designs, perfect gas; 300, thermally perfect —
`W:\temp\claude\jet-blade-crashmap`): one-block lever median 7.5 ms (max 18), row-by-row levers
median ~90 ms (max 0.6 s) on the perfect gas; on the thermally perfect gas **median 0.32–0.45 s and
worst 10–22 s for ONE point**, native. **In the browser build** (Node 24, the default design's 9-throttle
grid): perfect gas 10–80 ms a point (sweep 0.1–0.5 s); thermally perfect ~0.8 s a point, max ~1 s,
sweep ~6 s — ~2.5× native. The page's labels quote these.

**Gas** (corrected 2026-10-07 — the user chose BOTH, § 11.7 Q2). Before the fix, the table gases ran
but `size` read the scalar `gamma_c` (the table gas's spec default 1.4) and a `cp` derived from it for
the Mach numbers, while the work came from the enthalpy tables. Fixed in `blade_speed::build`
(`face_props`: `γ, cp` at each face's total temperature on a table gas; the perfect gas's scalar path
kept bit for bit), pinned to published air properties in `tests/rung85.rs`. An earlier draft of this
section called the thermally-perfect gas's moved redline "an artefact"; only part of it was — the
design's work and face temperatures come from the tables too, so the redline moves for real.

### 11.3 Knobs and readouts

**Engine knobs:** `π_LPC`, `π_HPC`, design `Tt4`; the map shape — rung 55's five (`flow/press`,
`press/flow`, `tilted`, `steep`, `flat-eta`), equals, as rung 85 ships them.
**Blade knobs, per spool** (rung 85 takes them per spool; a "same for both" link, on by default):
hub-to-tip `h`; the airflow level `M_rel,lim`; the material `σ_y/ρ`; the overspeed factor; `Φ_d`;
the rounding knob `λ`; the droop switch (A / B).
**Lever knobs:** which lever (one block / every row / front row only), which spool; computed over a
throttle grid when a slider is released, streamed point by point as slice 3's running line.

**Readouts — every result, no verdicts.** Per spool: which wall binds (airflow / strength), both
wall speeds, `K*` beside `K`, mean and tip blade speed, `r`, the design pre-swirl (map units and
degrees), the design absolute and relative tip Mach, the capacity `C`, the strength capability tip
speed, the redline `R`. A **staircase chart** — `K` and `R` against the airflow level (a sizing sweep,
instant), showing the steps at `λ` 0 and the smooth `R` at `λ` 1. A **lever chart** — physical
`N/N_d` of both spools against `Tt4`, each redline drawn as a line, bare (lever at design) beside
scheduled, the front-row tip Mach as a reading. Pin & compare, as the other views.

**Honesty lines on the page:** "never reached its target" said plainly (rung 85's V1, read at the
last setting the scan reached); the vane angle in DEGREES beside its travel — the crossings sit at
≈ 66–69°, which no real stator reaches; the strength wall is OPTIMISTIC (untapered blade, no disc, no
temperature derating, yield not ultimate), so "crosses" survives the model's error and "under" is
one-sided; the maps are invented; holding design incidence cancels the map slope (rung 85 § 4), so
`λ` reaches a held schedule only through the droop switch.

### 11.4 Model side (`sandbox_blades.rs`; model code edited only for Q2's fix)

- The five map shapes move from `tests/rung55.rs` / `tests/rung85.rs` into `src` (a `pub fn` the
  sandbox and both tests can share, or a copy pinned to them). Pinned against rung 85's PUBLISHED
  numbers — each shape's `v*` (press/flow 1.0499, tilted 1.4883, flat-eta 0.9620) and § 6.3's `K/R`
  table — never against itself.
- New ops: `blade_defaults`; `blade_size` (the sizing of both spools + the staircase sweep);
  `blade_lever` (one `Tt4` of one lever's schedule, so the page streams the grid).
- Plain words for every refusal: `SizingError` already reports a bad knob and a choking front row
  without panicking; the schedule path does NOT (its bare read and `match_point` are the
  non-fallible versions), so a lever failure goes through the trap path + `explain()`.

### 11.5 What must be measured before the sliders get their ranges

The crash map over the WHOLE knob box — `h`, `M_rel,lim`, `σ_y/ρ`, overspeed, `Φ_d`, `λ`, droop × 5
shapes × the pressure split × design `Tt4` × every lever on both spools × the throttle grid. Each
failure classified by what its message says (slice 3's lesson: one message can mean opposite
things), each class an `explain()` entry with a test that drives it; slider ranges from the measured
safe region. And the rung's own disclosed unchecked assumption — V1 reads an unreached schedule at
its LAST scan point, right only if shaft speed keeps rising with vane travel — checked across the
box: a mid-scan speed peak is shown as such, not silently as "void".

**Measured (2026-10-07).** Through the page's requests, the sweep's failures are all refusals in
plain words except ONE rare crash (the efficiency-bookkeeping check, 1 in 3 000, a near-zero-thrust
design — slice 1's words). Refusals: the unchoked nozzle at a low throttle (5 %, both gases); the
front row choking (0.7 %); the nozzle's gas below outside pressure at a low throttle (1 %, perfect
gas); a turbine past the bottom of the gas tables at a low throttle (3 %, thermally perfect — the only
fallible inverse inside the two-spool match is the turbines' `try_tau_of`). The three low-throttle
ones are caught BEFORE the schedule by the matcher's own `try_match_point` at the lever's design
setting, so they never reach the browser's crash path. **V1, audited** on 120 unreached schedules: the
MOVED spool's speed rose monotonically in every complete scan; the OTHER spool's speed peaks
mid-travel in 42 of 120 but never above its redline where the end reading was under (0 hidden
crossings); in 2 a finer scan found a gap where the matcher fails BETWEEN the coarse scan's points.

### 11.6 Gates

- **Reduce to the rung:** `blade_size` at the defaults ≡ `blade_speed::build` on the rung-85 rig, bit
  for bit, and reproduces the anchor's § 6.2 row for that cell; `blade_lever` ≡ `Machine::schedule`
  bit for bit; the rung-85 panel's lever rows reproduced through the op.
- The shape copies pinned to rung 85's published numbers (§ 11.4).
- Each refusal fires on its case and only it; each new `explain()` entry driven by a test.
- `check.mjs` gains blade requests (bar per slice 1's mechanism, re-measured); the browser drive
  gains the *Blades* view, a knob move, a lever sweep, a refused design, and screenshots
  (light / dark / 390 px).

### 11.7 Questions for the user

**ANSWERED (user, 2026-10-07):** (1) a — a separate *Blades* view on rung 85's two-spool rig;
(2) **b — also the thermally perfect gas**, after the sizing fix (`build` reads `γ, cp` at each face's
total temperature on a table gas; the perfect gas keeps its scalar path bit for bit); (3) a — `σ_y/ρ`
as a number, Ti-6Al-4V the one preset; (4) a — one knob set + an "unlink the spools" switch.
**Found after the answers:** the row-by-row stack (rung 55, `stage.rs` `kc = γ/(γ−1)` off the scalar
`gamma_c`) splits the pressure rise between rows with the FIXED γ on every gas — 1.4 on the
thermally perfect one. The one-block lever never builds a stack. Labelled on the page, rung 55's
plant not changed (not part of the agreed change).

1. **Which engine.** (a) A separate *Blades* view on rung 85's two-spool rig, with its own pressure
   ratios and `Tt4` — *recommended*; (b) the same, but its flight and `Tt4` start from the *Design*
   view's; (c) also size the *Design* view's single compressor (sizing only — no levers, they need
   two spools).
2. **Gas.** (a) Perfect gas only, said on the page — *recommended*; (b) also thermally perfect, after
   the model change that makes the sizing read `γ, cp` at the face temperature (lever readings
   ~0.3 s per point there); equilibrium is out either way (13 s per point).
3. **Material.** (a) `σ_y/ρ` as a raw number, Ti-6Al-4V as the one cited preset — *recommended*;
   (b) also cite one or two more from first-hand data sheets (a nickel alloy, a steel), as rung 85
   cited titanium.
4. **Per-spool knobs.** (a) One set, with an "unlink the spools" switch — *recommended*; (b) always
   two sets.

### 11.8 What the build found beyond the plan (2026-10-07)

- **The low-throttle failures are the lever's, not the design's.** Through the page's requests, every
  "jet below outside pressure" case was a lever throttle (the design had sized fine), so it got its own
  throttle wording; slice 1's words for it name a nozzle-exit-pressure knob this view does not have.
- **The thermally-perfect slow tail is not the unreached schedules** (median 0.40 s reached vs 0.38 s
  unreached, worst 16 vs 17 s, native): it belongs to particular designs. Hence the killable lever
  Worker rather than a per-point budget. Default design's 9-throttle sweep: 2.5 s native.
- **The CLI panel's gas is not the rig's** (cold R 286.9 vs 286.857): at λ = 1 it moves ONE printed
  digit (h 0.7 / M 1.5 / switch A: v* 1.40238 vs 1.40228). Pinned with that cause, one unit of slack.
- **The slider box was wider than the first sweeps.** Re-swept through the page's requests over the
  sliders' own ranges (45 000 sizings, 1 650 lever points): no new failure kind, but the DESIGN run
  itself fails at an overall pressure ratio ≳ 49 with a low design `Tt4` — the jet below outside pressure
  (thermally perfect, 1.7 %) and the efficiency-bookkeeping check (0.4–1.3 %). Kept reachable (a
  physical result), each explained and driven by a sweep design; a lever request's crash now gets the
  throttle's words (`explain` view `blades_lever`), a sizing crash the design's.
- **Browser ≡ native on the blade requests to 9.2e-15** (perfect gas): the lever search's 1e-12
  residual stop landed on the same travel in both builds; slice 1's bars stand (`check.mjs`).
- **Screenshots found** a clipped gas option, mixed "1,200" / "960.0" throttles, a wide "never
  reached" label, unmodelled-throttle reasons running off the table, and the chart marker's style
  leaking into the table (the table sits in a chart card). Two test races found in the browser drive:
  a stale "9 points done" count read before the next sweep began, and a blade result taken for a
  design result (neither has a shaft speed).

## 12. Slice 4 — the transient ("slam the throttle")

Planned 2026-10-07. Slices 1–3 show STEADY points: the engine has already settled. Slice 4 moves the
throttle and lets the engine take its time — the shaft speeds are STATES that lag the fuel — and shows
the result as a plot against time: shaft speeds, turbine-inlet temperature (which can OVERSHOOT), thrust,
and the compressor's path toward its stall line. Then the control switches — the fuel limiters and the
airflow levers the ladder built (rungs 34–63) — change what that path does.

### 12.1 Which engine — two, and the split is the model's, not a choice of convenience

Every fuel limiter (rungs 46–52) and every airflow lever on the transient (57–63) is **two-shaft by
assertion**: each `lp_disabled` path refuses it with the reason that its finding is a split BETWEEN
spools (`fuel_transient.rs` `integrate_fuel_lp_disabled`, seven refusals). On a ONE-shaft engine the
model has rungs 34–36 only. So:

- **(A) "Slam the throttle" — on the user's own engine** (the *Fly it* hardware, single spool, slice 3's
  `fly_solver`, its flight and map shape and stall line). Two ways to move the throttle: **command the
  turbine-inlet temperature** (rung 34 — the textbook idealisation) or **meter the fuel** (rung 35 — what
  a real engine does, where the temperature becomes an OUTPUT and overshoots). Shown side by side on the
  same ramp, because the difference IS rung 35's finding. Up or down (a chop as well as a slam).
- **(B) "Controls" — a second, two-shaft engine**: rung 43's rig, which is the *Blades* view's rig (the
  same `π_LPC` 3 × `π_HPC` 6, `Tt4` 1500, convergent nozzle, two-spool losses, flight 250 K / 50 kPa /
  Mach 0.85, the perfect gas), on rungs 43/45's fuel-metered march, with the switches of § 12.3. Said
  plainly on the page: not the *Design* engine, not the *Fly it* engine.

**Not in slice 4:** rung 37's combustor clocks (plenum fill, metal heat-soak) — its three marches run
their step count unconditionally and panic where rung 34's stops cleanly (`combustor.rs` header), so
they need their own refusal map; rungs 64–84 (the bleed LIMITER, the lagged valve, the cascades and the
reference/rank rungs) — each has its own constructor and its subject is loop structure, not a visible
behaviour; a flight knob on the two-shaft view — every two-spool solver kept the DESIGN ambient
pressure as the nozzle's back-pressure, and `fly_solver`'s override (§ 10.8) was
single-spool (repaired 2026-10-16, § 26; the knob BUILT the same day, § 27).

### 12.2 Measured (2026-10-07, scratch crates `W:\temp\claude\jet-transient-timing` and
`…\jet-transient-wasm`, not in the repo)

One march = the start equilibrium + a ramp + a settling tail, `ds` 0.02 (the rungs' own grid), settle
3 (A) / 2 (B) spool times. Grid: (A) 4 gases × both throttle modes × 3 map shapes × 6 ramps (3 slams
`1000/1100/1200 → 1500 K` at ramp 0.1/0.5/2 spool times, 3 chops `1500 → 1000/1100/900`); (B) 4 map
shapes × {none, LP stator schedule, bleed schedule} × 7 limiter sets × {slam 0.5, slam 0.1, chop}.
Wall clock, native at below-normal priority / browser build under Node 24:

| | native | browser (median / worst) | page use |
|---|---|---|---|
| (A) perfect gas | 2–4 ms | **5–11 / 38 ms** | follows a slider |
| (A) thermally perfect | 70–280 ms | **0.5 / 1.3 s** | on release, spinner |
| (A) reacting, Fork B | 150–390 ms | **0.6 / 2.3 s** | on release, spinner |
| (A) equilibrium, temperature commanded | **9.2 s** (one ramp) | ~20 s (not run) | refused — § 12.4 |
| (B) perfect gas, no limiter | 12–18 ms | **25–75 ms** | follows a slider |
| (B) perfect gas, limiters armed | 25–210 ms | **0.1 / 0.8 s** | on release, spinner |
| (B) thermally perfect | 0.7–12.8 s | **2.3–46 s** | refused — § 12.4 |

Each march is ONE call into the browser build, so it cannot stream point by point as slice 3's running
line does: a killable Worker and a spinner (slice 2's precedent), the newest request winning.

**What the grid found — each a result the page has to word, not a crash:**
- **A fuel slam can outrun the model.** (A), fuel metered, `1000 → 1500 K` in 0.1 spool times, thermally
  perfect / reacting / Fork B: the march stops after **5 points** on two map shapes of three. The
  temperature overshoots — 1955 K at 0.08 spool times against a 1500 K target — and the step's FOURTH
  RK stage (replayed in full: `k1`–`k3` succeed at ~2070 K) fails with rung 35's *"fuel compressor
  closure does not bracket"*: the burner would need a fuel-air ratio above **0.05**, the edge of rung
  35's closure search (`f_cap`, `spool.rs` `try_close_compressor_fuel`; stoichiometric is ~0.068). The
  perfect gas runs on (156 points, peak 2238 K) because its constant `cp` reaches that temperature on
  LESS fuel (peak `f` 0.043) — the cap applies on every gas. This IS rung 35's overshoot at its most
  violent; the page shows the points it has and says why it stopped.
- **A stall floor set above the starting point shuts the fuel off.** (B), the `φ` floor at 0.75 on the
  LP spool, where the engine STARTS below 0.75 (the flat-LP map bare: 0.727; the press/flow LP map with
  the stator schedule on: 0.729): the floor's own solve cuts the applied fuel to 11–13 % of scheduled,
  `Tt4` falls to ~600 K, the spools wind down and the march stops (16–79 points of 126). **Shown, not
  refused** — the limiter starving the engine is what a sandbox should show, and the last point carries
  both fuels: "the floor cut the fuel to 11 % of schedule, because it sits above where the compressor
  already runs at the start" comes straight off the trajectory.
- **The stator schedule moves the starting point across the floor** — rung 58's finding, made visible.
  On the bare machine (press/flow LP map) the same 0.75 floor is NOT dormant: the run's lowest LP flow
  coefficient is 0.7355, so it binds during the ramp and the run completes — rung 49 working. The stator
  moves the start (0.773 → 0.729) below it, so it bites from `s = 0`. Rung 60's INCIDENCE floor is the
  re-referenced version; offered in place of the `φ` floor when the stator is on (§ 12.3).
- **Two-shaft + thermally perfect + stator schedule fails before the march starts** (`fuel_for_tt4`:
  "inverse: root not bracketed", every limiter set, 7–89 ms) — the rung 57–63 suites run their
  two-shaft machines on the perfect gas only (their one other gas is a single-spool equilibrium build).
  With the 2–46 s cost, the (B) view is perfect-gas-only (§ 12.8 Q2).
- **No chop stopped early on (A)** (18 per gas per mode). On (B) one chop did (flat-LP map, stator, all
  three legs: 117 of 126) — to be classified in § 12.5's crash map.

### 12.3 Knobs and readouts

**(A) Slam the throttle** — a panel inside *Fly it* (its hardware, flight, map shape and stall line
unchanged): start throttle, end throttle (temperature, K, as *Fly it*'s slider), ramp length, and the
mode: *temperature commanded* / *fuel metered* / *both, overlaid*. Readouts against time: shaft speed;
turbine-inlet temperature (commanded and actual — the overshoot is the gap); thrust; fuel flow;
compressor pressure ratio. A **map chart**: slice 3's compressor map with the steady running line, the
stall line, and the transient's path across it — the path leaves the running line toward stall on a
slam and away from it on a chop (rung 34's excursion). The stall margin along the path, labelled as
slice 3 labels it ("read the trend, not the size").

**(B) Controls** — engine knobs: map shape (rung 43's four), `ρ` (how much slower the LP shaft responds
than the HP: the ratio of their time constants); the ramp (start, end, length). **Switches**, each with
its one number:
- *Temperature limiter* (rung 46) — the redline `Tt4_max`; + *its response lag* (rung 47).
- *Acceleration schedule* (rung 48) — its margin above the steady fuel/pressure line.
- *Stall floor* (rung 49) — which spool and the floor's level; with the stator schedule on, the
  incidence floor (rung 60) instead — one slot, one or the other, as the model has it.
- *Realistic release* (rung 52) — the asymmetric fast-attack / slow-release lag on the floor or schedule.
- **ONE airflow lever at a time**: *stator schedule* (rung 57, LP or HP, max setting and the speed it
  opens from) or *bleed schedule* (rung 62, max bleed and opening speed). Stator + bleed + a fuel limiter
  together is OPEN physics (CLAUDE.md: "Fuel + bleed + STATOR on one plant") — refused in words.
  *(LIFTED 2026-10-14 by rung 86 — § 15: a fourth choice, "both".)*
- Left out: rung 50/51's forced release (`s_off`, `τ_rel`) — an instrument the rungs used to ISOLATE the
  release edge, not hardware a controller has (§ 12.8 Q3).

Readouts: both shaft speeds; `Tt4` against the redline; scheduled vs applied fuel (the gap is the
limiter acting, and WHICH leg holds it); both compressors' flow coefficients against their stall walls
(the moved wall when the stator is on); thrust. Pin & compare overlays a second run, so "switch on the
schedule" reads as two curves.

**Time on every axis is in SPOOL TIME CONSTANTS** (`τ_spool = I·ω_d²/P_ref`, rung 34) — the model has no
rotor inertia (it is disclaimed), and rung 34's finding is the RATIO of the ramp to that time, not
seconds. The ramp knob is in the same unit; the page says why. (B)'s unit is the HP spool's.

### 12.4 Refusals, in plain words (pre-checks unless marked)

- **(A) equilibrium gas**: fuel metering does not exist on it (the forward burner asserts — rung 35's
  open seam, CLAUDE.md "reacting-gas fuel control"), so the side-by-side the panel exists for cannot be
  drawn; the temperature-commanded march alone runs, but at 9 s native (~20 s in the browser) for one
  ramp. Refused with those two reasons; the panel offers the other four gases.
- **(A)** both throttles must be above the compressor-face temperature (slice 3's pre-check); both
  endpoints must run steady (a below-idle endpoint is slice 3's `explain_fly` wording).
- **(B) thermally perfect**: not offered (§ 12.2).
- **(B) switch combinations the model refuses** (`r43_integrate_fuel`'s asserts, each a pre-check): the
  realistic-release lag needs a floor or schedule to lag; it cannot run with the temperature limiter's
  lag (a two-lag cascade, rung 66 — not this slice); the limiter's lag needs a redline; `φ` floor and
  incidence floor share one slot; stator AND bleed with fuel limiters is OPEN *(lifted by rung 86 — § 15)*.
- **(B) a floor above the start point** (§ 12.2) is NOT refused — it is shown, and worded from the
  trajectory's own applied-vs-scheduled fuel.
- **A run that stops early is a RESULT**: the page shows the points it has and why it stopped. The
  marchers `break` and drop the error, so the reason must be recovered — **(A)**: replay the WHOLE
  failed step from the last recorded point through the PUBLIC `try_instant` / `try_instant_fuel` — `k1`
  to `k4`, then the new state's `k1` (the next step's opening call, at a state the trajectory never
  records) — the same calls in the same order as `spool.rs` `march`, so the same arithmetic; the first
  `Err` is the reason (demonstrated on § 12.2's fuel slam: `k4` fails). **(B)**: the limiter-armed
  derivative is private to the marchers, so the sandbox reports where it stopped and classifies the
  known causes from the trajectory (the floor's applied-vs-scheduled fuel; the overshoot) — no copy of a
  250-line marcher.

### 12.5 Model side (`sandbox_transient.rs`; no model code edited)

- `op:"slam"` → (A): `fly_solver` (fresh per request, slice 3's reason), the start equilibrium, then
  `integrate` and/or `integrate_fuel` with the ramp schedule — rung 35's own `ramp_excursion_fuel`
  shape, inlined so the page gets the points.
- `op:"controls"` → (B): `build_scheduled_bleed` with one `LeverArm`, `fuel_for_tt4` at both ends, the
  accel table from `accel_schedule` when armed, `integrate_fuel` with a `FuelLimiters`.
- **The crash map before the slider ranges** (slice 2's § 11.5 method): every knob over its box, every
  failure classified by what its message says, each class an `explain` entry with a test that drives it.
  **The step size is checked against two knobs first**: `ρ` divides the LP rate (all of § 12.2 ran at
  `ρ` = 1), so at the small end of its box `ds` 0.02 is compared with 0.01 before any failure there is
  read as physics (rung 65's lesson: a "physical" pre-check that was RK4 instability); and a ramp of 0.1
  spool times is only 5 steps, so the ramp knob's floor comes from `ds` too. (A)'s speed labels are
  re-measured over the crash map's design and flight box (§ 12.2 timed the default design only).

### 12.6 Gates

- **Reduce:** `op:"slam"` ≡ a direct `integrate` / `integrate_fuel` call, bit for bit; a ramp whose
  start and end are the same throttle stays on its equilibrium (`ν` constant to the solver's tolerance);
  the temperature-commanded march APPROACHES slice 3's `fly` at the end throttle — after 3 spool times
  it is at `ν` 0.996 of 1.0 (§ 12.2), so the gate is convergence: the gap to `fly`'s `ν` and `π_c`
  shrinks as the settling time grows, with a bar measured from that data, never typed.
- `op:"controls"` ≡ a direct `integrate_fuel` on the same machine, bit for bit; **a dormant limiter ≡
  the bare run** (rung 46's gate 1, through the op: a redline above the bare peak).
- The stop-reason re-run on (A) reproduces the march's own trajectory up to the stop (bit for bit) —
  so the reason it reports belongs to the step that actually failed.
- Each pre-check fires on its case and only it; each `explain` entry driven by a request that raises it.
- `check.mjs` gains slam / controls requests (bar per slice 1's mechanism, re-measured); the browser
  drive gains both panels, a slam, a chop, a switch, a refused combination, an early stop, and
  screenshots (light / dark / 390 px).

### 12.7 Order of work

1. The crash map over both knob boxes; slider ranges from it. 2. `sandbox_transient.rs` + its tests.
3. The page: (A) inside *Fly it*, (B) as a view. 4. `check.mjs`, the browser drive, screenshots.
(A) and (B) are separable — (A) can ship first.

### 12.8 Questions for the user

**ANSWERED (user, 2026-10-07): all four as recommended** — (1) a: both engines, the slam on the
user's *Fly it* engine FIRST, then the two-shaft *Controls* view; (2) a: the slam offers perfect /
thermally perfect / reacting / Fork B and refuses equilibrium with its two reasons, *Controls* is
perfect-gas-only; (3) a: the § 12.3 switch set, no forced-release experiments; (4) a: time in spool
time constants, no seconds knob.

1. **Which engines.** (a) Both — the throttle slam on your own *Fly it* engine, AND a separate two-shaft
   *Controls* view (the *Blades* rig) for the switches, built (A) first — *recommended*; (b) only the
   slam on your engine (no limiters — the model has none for one shaft); (c) only the two-shaft view.
2. **Gases.** (a) Slam: perfect live, thermally perfect / reacting / Fork B on release (0.5–2.3 s),
   equilibrium refused with the reason; *Controls*: perfect gas only — *recommended*; (b) as (a) but the
   equilibrium gas allowed on the slam in temperature-commanded mode only (~20 s, overshoot hidden).
3. **Which switches on *Controls*.** (a) The § 12.3 set: temperature limiter + its lag, acceleration
   schedule, stall floor (incidence version with the stator), realistic release lag, and ONE airflow
   lever (stator or bleed schedule) — *recommended*; (b) also the forced-release "experiment" knobs
   (rungs 50/51), labelled as instruments; (c) a smaller first set — limiters only, levers later.
4. **Time axis.** (a) In spool time constants, said why (the model has no rotor inertia) —
   *recommended*; (b) also a "spool time constant, seconds" knob that only rescales the axis, labelled as
   a guess.

### 12.9 What the build of (A) found beyond the plan (2026-10-07)

- **The commanded-temperature power cut stops on the SOLVER, not the engine.** Rung 34's closure tries
  its LOWEST airflow first (`try_close_compressor`'s literal wall, 0.02); on a fast cut the still-fast
  compressor would heat that trial's air to ~760 K, above a commanded 640–700 K (or within a few kelvin
  of it — a near-zero rise the burner's `f` solve cannot close either), so the trial fails and the
  closure gives up, while the real operating point (compressor exit ~550 K) is fine. The fuel-metered cut
  runs through. Classified by replaying that first trial exactly (`low_wall_trial_fails`, a copy of
  `eval_m`'s opening lines held to the closure's own outcome). That a root EXISTS past the failing trial is
  shown at the measured state (the residual changes sign where the compressor exit, ~536 K, is below the
  commanded 640 K — a pinned test), not over the whole box. HYPOTHESIS, unverified: a model-side fix
  (march the low wall in) would be bit-identical wherever the closure succeeds today — a candidate seam.
  **REPAIRED 2026-10-10 (§ 14.1)** — true for the time march only; and some of these stops were real
  flame-outs, not artefacts.
- **A time step can be too coarse.** At Mach 3.3 the shaft responds far faster than its design τ: the
  march overshoots its end speed and an RK stage lands below zero speed (`Overstep`). Rare (1 in ~150
  thermally-perfect requests); said as a limit of the stepping.
- **The ramp knob is quantised by the step**: whole numbers of 0.02 τ, at least one (§ 12.5's study:
  converged to 0.1 K at half and a quarter of the step; between steps 20–70 K off).
- **A fuel-cap stop has two causes, needing opposite advice** (advisor, checked): the transient
  overshoot (slow the move down) and an ENDPOINT whose own steady fuel-air ratio is above 0.05 (no ramp
  reaches it — the reacting gas at 2250 K needs 0.051). The closure's low wall IS f = 0.05 at the metered
  fuel, so the second is exactly unreachable: refused before the march (`fuel_reach_check`), naming the
  throttle; the stop words now cover only the first.
- **Flow against the stall line, unchoked:** slice 3's tile reads rung 36's STEADY margin, choked only;
  the slam shows the flow coefficient at every instant, and draws the unchoked stretches thin and dashed.
- **Crash map** (through `sandbox::call`, 3 000 perfect-gas + 500 per table gas, fresh seeds after each
  change): every early stop is one of FOUR measured kinds (fuel limit, nozzle gap, first-trial artefact,
  step too coarse) — none unclassified; the "burner" and "other" kinds have words but no measured case
  (fallbacks); the panics are slice 3's
  steady-solve classes, at an endpoint (the page asks each throttle's steady point to name which) or in
  the design capture. Native median per run 5 ms (perfect), 0.19–0.32 s (table gases), worst 2.3 s.
- **The fuel-metered run settles FASTER** than the commanded one (the overshoot spins the shaft up):
  90 % of the speed change at 1.28 τ against 2.24 τ on the opening slam.
- **Browser ≡ native on the slam requests** within slice 1's bars (worst overall unchanged: perfect
  9.2e-15, table 9.4e-11).
- **Screenshots** came out right first time; a `SANDBOX_SHOTS=1` run wrote them into a folder named `1`
  in the repo (the variable is a PATH) — moved out, not committed.

### 12.10 What the build of (B) found beyond the plan (2026-10-08)

Built as `src/sandbox_controls.rs` (its own module, not inside `sandbox_transient.rs`: the two views share
no engine). Measured in a scratch crate (`W:\temp\claude\jet-controls-map`, not in the repo): a 2 000-request
sweep over a wide box, then 3 000 over the slider box, every stop's REAL message read first.

- **The § 12.2 "floor above the start" stop has two messages, both seen**: rung 49's *"floor … UNREACHABLE"*
  (no fuel cut restores the flow coefficient) and rung 43's *"fuel closure does not bracket"*. They split by
  WHICH call failed, not by cause guessed from the trajectory: § 12.4's "classify from the trajectory" was
  replaced by a RE-RUN of the failed step through the public pieces (`try_instant_fuel` and the three legs'
  set-point solves), held bit-equal to every recorded step on the plain route and on rung 52's lagged route
  (which records its third state). Rung 47's lagged governor does NOT record its state: a stop there is
  said as "cause not known", never guessed (2.2 % of the box).
- **A stop the plan did not foresee — the model's every-step check**: the plain march first works the engine
  out at the FULL SCHEDULED fuel (to see whether a limiter is needed); when a limiter holds the fuel far below
  the schedule, that check has no answer at the slowed shafts and the run dies, though the burning fuel is
  fine. Identified by asking the model (advisor, checked): at the failing state the closure is re-run at the
  CUT fuel (the last point's fraction of the scheduled one); the stop is the method's only if that solves.
  89 of 90 such stops in the slider box did; the one that did not (a 4 % cut on a fast slam) is classed rich.
  Said as the method's limit; on CLAUDE.md's open list beside rung 34's low wall. **REPAIRED 2026-10-10
  (§ 14.3)**, behind a switch the Controls view turns on.
- **Without a limiter cutting, a failed closure sits at one of rung 43's two fuel-air walls**: fast cuts at
  0.71–1.05 × `F_FLOOR` (0.004), fast slams at ~1.03 × `F_CAP` (0.065) — so "lean" / "rich" are read off
  the failing call's mixture (its fuel over the last point's face flow) against the walls' geometric middle,
  with a test holding each example within 2× of its wall.
- **Four crashes, all the acceleration schedule's own steady rows** between two working endpoints, all with
  a lever schedule doing its travel just below design speed. The table is now built through the fallible
  equilibrium (the model's arithmetic line for line, held bit-equal to `accel_schedule`) and a gap is refused
  in words. The lever sliders stop at 0.8 of design speed and 0.3 travel; the endpoint failures left are
  low throttles on the flat-LP map (below 690–890 K), refused in words.
- **Step 0.02 kept**: at ρ 0.2 / 1 / 5 the trajectory agrees with step 0.005 to ~1e-4 in flow coefficient,
  but a sharp temperature spike can fall between points (10.7 K low with the floor on) — said on the page.
  An attack time of a quarter step lost accuracy (2.5 K), so both lags start at one step. Runs are capped at
  10 τ (slowest requests 1–3.7 s native, all long and limiter-armed); the two time sliders can add past it,
  which is refused in words. (A)'s whole-step ramp rule re-measured here: the bare slam's peak moves 7.5 K
  with the step at a ramp of 0.51, 0.0 K at 0.5.
- **A floor far out of reach stops at the very first step** (42 of the wide sweep's floor stops): no points,
  the first evaluation re-made, the page shows the words over empty charts.
- **"Which limiter holds" is exact only for a limiter acting WITHOUT a lag on its route**: on the lagged
  routes a lagged cut converges onto its set point, and the first bar (1e-7) separated the classes by a hair.
  Counting only unlagged legs: holding ≤ 4.8e-15, not holding ≥ 3.2e-6; bar 1e-10.
- **Browser ≠ native beyond slice 1's perfect-gas bar**: the Controls numbers come out of iterated solves
  (closure 1e-12, legs 1e-13), so they get their own bar (1e-11; worst 2.1e-13), and the "settled" gap — a
  difference of two speeds — is compared absolutely.
- **The phone screenshot squeezed the table** (a sentence-long cell that would not wrap) — fixed; light, dark,
  stopped and fuel views looked right.

## 13. Slice 5 — the combustor ("light the burner")

Planned 2026-10-08. Slices 1–4 treat the burner as a box that turns `Tt3` into `Tt4`. Slice 5 opens it: the
NOx diagnostics of rungs 7–24 as knobs — how rich the front (primary) zone burns, how long the gas stays
there, how fast and how evenly the dilution air mixes in — with every result shown: the emission index
(g of NO per kg of fuel), the front zone's flame temperature, the quench path's peak, the mixing numbers.
Every rung from 7 up is a diagnostic BESIDE the cycle, so nothing here changes a cycle number.

### 13.1 Which burner — the user's design, read on the equilibrium gas

The diagnostics take four numbers off a design run — `Tt3`, `Tt4`, the overall fuel-air ratio `f`, the
burner pressure `pt4` — and then do their OWN equilibrium chemistry (`equilibrium_composition`,
`primary_aft`), whatever gas the cycle ran on. They check themselves: the diluted mixture's equilibrium
temperature must come back within 5 % of `Tt4` (`nox.rs` `zoned_nox`, the mix-out gate). Measured on the
Design view's default design and eight more (`π_c` 4–40, `Tt4` 1000–2200 K), each on all five gases
(scratch crate `W:\temp\claude\jet-combustor-timing`, not in the repo):

- **Perfect gas: wrong `f`.** Its constant `cp` needs less fuel for the same `Tt4`, so the mix-out lands
  8–14 % LOW and the gate trips (7 of 9 designs); the other two pass at 0.96 — wrong, and silent.
- **Thermally perfect, reacting, Fork B, equilibrium: within 2.5 %** (0.979–1.024; on the equilibrium gas
  itself +0.65 to +1.3 % — consistent with the 0.99 combustion efficiency, which the mix-out does not
  model, but NOT checked).
- **The basic two-zone EI does not depend on the gas at all** — it is set in the front zone by `Tt3`,
  pressure and the front-zone richness; `f` and `Tt4` enter only through how much air the front zone gets
  and the dilution. The four table gases share the air tables, so the same `Tt3`, so the same EI to every
  printed digit.

So the burner reads the **Design view's current design, run on the equilibrium gas** — the CLI's production
gas, what rungs 7–24 ran on — at ~2 ms, re-run only when the design changes (§ 13.8 Q1). On another gas the
page says why the burner shows the equilibrium run's inlet numbers.

**Not in slice 5:** rungs 25–30 (the nozzle and turbine marches); rung 17's three-model exhaust ladder as
a whole (21 s native — and the comparison it makes is the one the model selector of § 13.3 lets the user
make by hand); the OPEN seams (a pocket clamp that fires at the burner, detailed Fenimore chemistry).

### 13.2 Measured (2026-10-08)

The Design default (`π_c` 10, `Tt4` 1500 K, 250 K / 50 kPa / Mach 0.85): `Tt3` 583.5 K, `pt4` 747 kPa,
overall `φ` 0.40. Front zone at `φ_p` 1.5 (rich, as rungs 12–24 ran), jets at `J` 25. Three grid levels:
**L0** = the model's own defaults (4000 Zeldovich steps, a 240-point quench, 200-point curves, 48 × 48
planes); **L1** = Zeldovich 4000 steps, quench 60 points / 400 steps, curves 80 points (40 on the
per-pocket models 16/23/24), quadrature 160, planes 32 × 32, 24 time slices — the charts page's grids
(`visuals.rs`) except that the charts run quadrature 120 on their pocket and dwell blocks; **L2** = quench
32 / 200, Zeldovich 1000, curves 40 (20), quadrature 80, planes 24 × 24, 16 slices. Native wall clock at
below-normal priority, on a machine also running the other measurements (so ±2×); browser = the same calls
under Node 24.

| Front-zone + mixing model (rung) | one point at L1, native / browser | a sweep at L1, native | page use |
|---|---|---|---|
| two zones, instant quench (8, 9), ± fast O atoms + prompt NO (19) | **4 ms / 4 ms** | richness bell, 17 points: **0.07–0.4 s** | follows a slider |
| a set quench time (10), jets (11), two-stream (12) | 0.1–0.2 s / 0.1 s | 14-point `J` sweep 0.9 s; richness bell 1.5 s | on release (slice 4 put 0.1 s there); sweeps stream |
| on a precomputed curve: β-PDF (13), through the quench (15), transported (18), cross-plane (22) | 0.3–1.8 s / 0.3–0.6 s | `J` sweep 4–18 s | on release; sweeps stream |
| per pocket: through the quench per pocket (16), the plane in time (23), local rate (24) | 3.7–5.7 s / 6–10 s | `J` sweep 51–270 s (L0: 31–84 s per POINT) | § 13.8 Q2 |

- **The precomputed-curve models rebuild one curve on every call — and the model gives no way to hand it
  one.** Rungs 13/15/18/22 build the same "ideal bell" (EI against local mixture, `n_bell` flame solves)
  inside `zoned_nox` → `pdf_mean_ei` each time, though it depends only on `Tt3`, pressure, the residence
  time and the O-atom switch — not on the jets. Outside `g = 0`, `pdf_mean_ei` IS `bell_interpolator` then
  the fold `pdf_mean_ei_on_bell` does, so a kept bell would give the same number bit for bit; AT `g ≤ 1e-9`
  (exactly `J_opt`, the notch below) `pdf_mean_ei` returns the EXACT flame solve and `pdf_mean_ei_on_bell`
  the interpolant — different numbers. But `zoned_nox` takes no bell, so a cache means either a model change
  or a copy of those closure branches in the sandbox — § 13.8 Q4. Without one, those sweeps cost 4–18 s,
  streamed. (The richness chart needs no cache: the instant-quench bell is 17 cheap points.)
- **Browser ≡ native to ~1e-15**: worst 1.7e-15 relative over 56 values at L1 on four designs (`π_c`/`Tt4`
  10/1500, 4/1000, 20/1900, 40/1600; every model but the per-pocket three), 4e-15 over 61 at L2. The four
  equilibrium burner inlets came out bit-identical — but slices 1–3 saw the cycle's equilibrium-temperature
  solver differ at ~1e-10 on other designs, which would carry into every EI, so the page's bar is set at the
  cycle's, re-measured over the build's request grid.
- **Grid: L1 keeps every shape, L2 does not.** Against L0, L1 is within ~0.6 % at every `J` (jets 0.4 %,
  two-stream 0.4 %, β-PDF 0.6 %, through-the-quench 0.5 %, transported 0.1 %, cross-plane 0.1 %) with every
  minimum at the same `J`. L2 moves the through-the-quench model's far-side minimum (`J` 100 instead of 144)
  — and **crashes** the cross-plane models (22, 23, 24) at `J` 6 and 36 on the model's own β-PDF check
  (*"quadrature drifted the mean … raise n_quad: the bar needs ≥ 112"*). So the page runs L1, a fixed
  setting, never a knob. Per pocket (rung 16) at L0 — an 8.3-minute sweep — L1 is within 0.4 % at every `J`,
  minimum at the same `J` (225); the set-time quench's richness bell, within 0.4 %. (The plane-in-time and
  local-rate models, 23/24, were compared L2 against L1 only — L0 is 20 min a sweep; the build checks a few
  of their points at L0.)
- **The optimum is a notch, not a valley.** At the default jet spacing `J` 16 puts the jets exactly on the
  Holdeman optimum (`C = (S/H)·√J` = 2.5): the segregation width is zero, so the β-PDF model drops to the
  perfectly-mixed value (0.000 g/kg — the lean overall mixture makes no NO) and the models with a dwell
  term drop to the mean-field floor. That is rungs 12/13's kink, by design. A sweep that steps over `J_opt`
  misses it, so the `J` grid always includes `J_opt` for the current spacing.
- **The exhaust-nozzle readout is cheap** (`Gas::nozzle_flow`, rung 14): frozen vs equilibrium exit
  temperature and velocity, and how far above its own equilibrium the exhaust NO sits at the nozzle exit —
  7–10 ms. At the default, a stoichiometric front zone leaves the exhaust NO ~250× above its exit
  equilibrium (frozen, as rung 27 earned); a rich one, 0.016×.

### 13.3 Knobs and readouts

**The burner inlet** (read-only, from the Design view): `Tt3`, pressure, `Tt4`, overall richness, fuel flow
— with a line saying that the compressor sets the front zone's starting temperature (`π_c` up ⇒ `Tt3` up ⇒
NO up: a stoichiometric front zone makes 5 → 21 → 53 → 88 g/kg as `π_c` goes 4 → 10 → 20 → 40, § 13.1's
grid at varied `Tt4`).

**Knobs.**
- *Front zone*: richness `φ_p` (from the overall richness up to 2, the soot limit) and residence time `τ`.
- *Chemistry*: faster-than-equilibrium O atoms (rungs 19–21, a switch); prompt NO (rung 19, a switch + its
  imposed reference level — "the one number nobody can derive").
- *Quench* — one of: **instant** (rungs 8/9) · **a set time** `τ_q` (rung 10) · **dilution jets** (rung 11:
  jet strength `J`, duct height `H`, crossflow speed, entrainment constant, schedule shape).
- *Mixing model* (jets only) — none (mean-field, rung 11) or ONE of the eight closures the model allows
  (12, 13, 15, 16, 18, 22, 23, 24 — `zoned_nox` asserts at most one), each with its own few knobs (jet
  spacing `S`; the imposed optimum `C_opt` where the model imposes it, rungs 12–18; the plume spread `k_p`
  where it is DERIVED, rungs 22–24 — `C_opt` then shown as an output). Each named in plain words with its
  rung and its finding in one line — rung 18's said plainly: it cannot find the optimum.
- Every guessed constant is labelled as slice 3 labelled the stall line, *"read the trend, not the size"*
  (`τ`, `H`, crossflow speed, entrainment, `S`, the core knobs, `k_p`, the prompt level — the code's own
  docs call each order-of-magnitude). Rung 8's ICAO-band claim is the one absolute anchor.

**Readouts** — every one the model returns, no verdicts: EI (thermal, prompt, total) and exhaust ppm; NOx
flow (EI × fuel flow, g/s); front-zone flame temperature and air share; the diluted temperature; on a finite
quench the quench time, the peak temperature on the way through stoichiometric, and how far the NO came
toward its equilibrium; with jets, `C` against `C_opt`, the segregation width against its ceiling, the core
share and dwell; on rungs 23/24 the correlation ratio. Beside them, rung 7's "if the burner were perfectly
mixed" number — NO at `Tt4` — which is ~0: the reason zoning exists. **Charts:** EI against front-zone
richness (the bell), EI against jet strength (when jets are on), the quench path (temperature against
mixing progress, `quench_trajectory`), each with the current setting marked; pin & compare overlays a
second run, as everywhere. *Optional* (§ 13.8 Q3): the nozzle-exit readout of § 13.2.

### 13.4 Refusals, in plain words (pre-checks unless marked)

- **A front zone leaner than the whole burner** (`φ_p` below the overall richness — the model's *"primary
  air fraction α > 1"*; met in § 13.1's grid at `Tt4` 2100–2200 K with `φ_p` 0.6): "all the fuel burns in
  the front zone, so it cannot be leaner than the burner overall"; the slider's floor follows the design.
- **The design does not run on the equilibrium gas** (Fork B's balance at `Tt4` 2200 K is one seen; any
  design slice 1 already refuses): the Design view's own words.
- **Model combinations** are structural, not refusals: a mixing model only with jets, one at a time.
- **Everything else is found, not guessed** (slice 2's method): a crash map over the design box × the burner
  knobs × every model, every failure read by its message. Candidates from the source: the mix-out gate, the
  O-atom multiplier's flame band `[1, 2]` at a cool lean front zone, the trace guard (NO < 2 %), the β-PDF
  checks, the prompt model's validity. Each class an `explain` entry with a test that drives it.

### 13.5 Model side (`sandbox_burner.rs`; no model code edited)

- `op:"burner_defaults"`; `op:"burner"` → the inlet (the design on the equilibrium gas — slice 1's
  `build_turbojet` + `run`) and ONE `zoned_nox` at the knobs, returning its whole state; `op:"burner_sweep"`
  → one point of a richness or `J` sweep, so the page streams a sweep point by point with progress and drops
  a stale one (slice 3's running line).
- No cache unless § 13.8 Q4 says otherwise (then: its option's route, gated as § 13.6 says).
- `explain` entries from the crash map; the L1 grids as fixed constants, cited to § 13.2.

### 13.6 Gates

- `op:"burner"` ≡ a direct `zoned_nox` on a fixture built in the test from literal numbers, bit for bit, for
  every quench mode and every closure; the inlet ≡ slice 1's equilibrium design run.
- If a cache ships (Q4 b or c): the cached route ≡ `zoned_nox` bit for bit for `g > 1e-9`, and AT `J_opt`
  (`g = 0`) it must take `pdf_mean_ei`'s exact branch — a test at `J_opt` on every closure it touches; a
  stale key rebuilds. Option (b) is a model change: the full gate, every output bit-identical.
- The fixed L1 grid against the model's defaults, at a few `J` per closure: same minimum, within a bar
  measured from § 13.2's data, never typed.
- Each pre-check fires on its case and only it; each `explain` entry driven by a request that raises it.
- `check.mjs` gains burner requests (bar re-measured: ~1e-15 seen); the browser drive gains the view — the
  opening run against native, a richness sweep, jets on, a closure, the notch at `J_opt`, the lean refusal,
  a sweep stopped by a knob move — and light / dark / 390 px screenshots, looked at.

### 13.7 Order of work

1. The crash map; slider ranges from it. 2. `sandbox_burner.rs` + its tests. 3. The page (a *Burner* view).
4. `check.mjs`, the browser drive, screenshots.

### 13.8 Questions for the user

**Answered 2026-10-08 (user: "Go", on being offered the recommendations): all four as recommended** — 1 (a) the
Design view's design read on the equilibrium gas; 2 (a) the per-pocket models as a point on release + a
stoppable streamed `J` sweep; 3 (a) the nozzle-exit readout included; 4 (a) no cache, no model change.
Found before the build: § 13.2 was measured with the jets at `C_e` 0.20 (the charts page's and rung 11's CLI
panel's value), but the model's own default — and rung 12's spec's stated default (`docs/rung12-spec.md`) —
is 0.15. The page opens on the model's default; the grid's L1-vs-L0 check is re-measured there.

1. **Which burner.** (a) The Design view's design, its burner always read off an equilibrium-gas run of it
   (~2 ms extra when the Design view is on another gas; said on the page) — *recommended*; (b) a fixed burner
   at the CLI's design point, the one rungs 7–24 ran on (simplest, but the compressor's effect on NO is
   lost); (c) the burner view gets its own pressure-ratio / temperature / flight sliders.
2. **The three per-pocket mixing models** (rungs 16, 23, 24 — 4–10 s a point in the browser, a `J` sweep
   1–4 minutes). (a) Offered as a single point that runs on release with a spinner; their `J` sweep a button
   that streams with progress and can be stopped — *recommended*, and a DEPARTURE from slice 4, which refused
   2–46 s runs outright (§ 12.4); the difference is that here a sweep streams and can be stopped, and a
   single point is at most ~10 s; (b) a point only, no sweep; (c) left out (the five faster closures only),
   as slice 4 would have done.
3. **The nozzle-exit readout** (rung 14, ~10 ms: does the exhaust NO survive the nozzle; frozen vs
   equilibrium exit temperature). (a) Included under the burner readouts — *recommended*; (b) left out;
   (c) also rung 17's full three-model ladder (21 s).
4. **Speeding up the four precomputed-curve models' sweeps** (§ 13.2: 4–18 s native per `J` sweep; with a
   kept curve ESTIMATED 1–3 s, not measured). (a) No cache — the sweeps stream at their real cost and no model code changes —
   *recommended*; (b) a small model change so `zoned_nox` can take a curve built outside it, shipped on its
   own behind the full gate (slice 3's memo speed-up precedent); (c) the sandbox repeats those four branches
   itself around a kept curve, held equal to `zoned_nox` by a test — a copy of model code, which slice 4
   refused for its marcher.

### 13.9 What the build found beyond the plan (2026-10-08 – 10)

Built as `src/sandbox_burner.rs` (its own module; the inlet is slice 1's `build_turbojet(…).run(…)` on the
equilibrium gas). Crash map in a scratch crate (`W:\temp\claude\jet-burner-map`, archived as
`docs/plans/scratch-archive/jet-burner-map/`): random requests through `sandbox::call` over `π_c` 2–40,
`Tt4` 900–2400 K, five ambients × Mach 0.1–2.5, front-zone richness 0.3–2, and every knob of every dilution
route and mixing model over ranges wider than the sliders — about 3 500 instant-dilution requests, ~700 each
for the set-time and mean-field jets, 150–220 per curve model, ~40 per per-pocket model. Every failure was read
by its message.

- **Two of § 13.2's L1 grid choices failed the crash map and went back to the model's defaults.** The quench's
  400 RK4 steps (the charts page's) went UNSTABLE on rung 12's core at a long core dwell and high pressure — NaN
  where 2 000, 8 000 and 32 000 steps agree to 1e-9 — so the steps stay at 2 000 (they cost little beside the
  path's equilibrium solves). The β-PDF quadrature at 160 failed the model's own mean check on 6 cases its
  default 200 passes, so quadrature stays at each model's default. What is left fixed (60 path points, an
  80-point bell — 40 on the per-pocket models — a 32 × 32 plane, 24 time slices) is within **0.35 %** of the
  model's defaults at every `J` of the sweep at the opening design, every minimum at the same `J`, on all nine
  routes (the per-pocket three at 5 `J` each). That is re-measured at the jets' default `C_e` 0.15 (§ 13.8);
  the mean-field jets alone show the same 0.35 %, so it is the path's 60 points, not a mixing model. Gated at
  1 %, around the notch, on the four curve models.
- **Most failures are the DESIGN, not the burner**: the equilibrium burner balance at extreme `Tt4`, the
  efficiency cascade, the nozzle back-pressure — said in the Design view's own words, prefixed "this design does
  not run on the equilibrium gas". The burner's own classes, each an `explain` entry with a test that drives it:
  NO stops being a trace gas (in the front zone, with prompt NO summed in, or on the dilution path); the fast-O
  correction outside its flame band (a cool lean front zone); the front-zone flame outside 800–3200 K; the
  dilution path past 3200 K (rich gas crossing stoichiometric with hot air); a mixing spread too wide for the
  β-PDF sums at a lean burner (44 of 220 β-PDF requests — so the spread-cap slider stops at the model's default
  0.3); a cross-plane field that cannot hold its mean (narrow plumes); and a cross-plane model with a front
  zone no richer than the burner. The nozzle readout's two (exit colder than its 500 K search floor — the
  commonest readout failure, 431 of ~3 470 instant-dilution requests; an equilibrium solve that does not settle) leave the burner's numbers standing.
- **The per-pocket models: up to ~20 s a point at the box's edges — and the crash map's own timings are NOT
  per-point times.** The crash map logged medians of 4–24 s and a worst of ~2 minutes, but it ran 8 worker
  threads on 16 logical cores beside other measurements. Its nine slowest points (three per model), re-timed one
  at a time in the browser build under Node at below-normal priority (2026-10-10): 2.8–8.1 s, one 20.7 s
  (rung 23) — the crash map's 55–121 s were 8–36× inflated, and the slowest one itself read 9.7 s then 3.3 s on
  two runs (machine load). § 13.2's 4–10 s at the opening design stands; the page says "seconds, up to about
  half a minute", and the point stays stoppable (any knob move kills its worker).
- **A hang, latent since slice 1, in the page's shared worker code.** The model's JSON reader takes ASCII only
  (its writer escapes the rest); `JSON.stringify` does not escape. When a failure's message carried a
  non-ASCII character (β, ξ, Σ, —), the follow-up request for its plain words trapped INSIDE the trap handler,
  the worker never answered, and the page waited forever. The burner's messages are full of them; the Design
  view had one too (the equilibrium burner balance's `Σ`), unseen because slices 1–4's browser checks drove only
  ASCII messages. Fixed in the runner: every request is escaped, and if the plain-words call fails the worker
  still answers, with the model's message. The browser drive gained a per-call ceiling (300 s — a hang now
  FAILS and closes its Chrome; twice it had left Node dropping out with Chrome still running) and a `Σ` check.
- **Browser ≡ native under the table-gas bar**: 55 burner requests, 2 667 numbers, 2 398 bit-identical, worst
  1.5e-15 relative — save the nozzle readout's jet-speed gain (a difference of two nearly equal speeds),
  3.2e-12. The β-PDF model at its notch (EI ~1e-5) came out bit-identical.
- **At the notch the richness chart is flat** — on the β-PDF model at `J_opt` the spread is zero, so the burner
  is perfectly mixed and the front zone's richness drops out. Correct, and the reason the jet chart, not the
  richness chart, is the one to read there.

## 14. Three solver walls repaired (2026-10-10) — not a slice, not a rung

The page met three places where the MODEL gave up though an answer existed (CLAUDE.md's open list).
Each is now repaired in the model; the rule for each was the project's spine — **where the old
code ran, the new code is the same arithmetic, bit for bit**, and a frozen record that moved was
listed key by key before anything was licensed.

### 14.1 Rung 34's low flow wall (§ 12.9's first finding)

- **Fix:** `SpoolTransient::try_close_compressor_marched` walks the closure's 0.02 low wall in, in
  1/64 steps of the bracket, until the trial's burner solve runs; `integrate` (the time march)
  steps with it. Only there: in `find_equilibrium_nu`'s bracket march a failing trial is CONTROL
  FLOW, and letting it succeed moved the converged speed by ~1e-11 on cells that never failed —
  measured, 1 416 of `combustor_oracle`'s 2 066 keys. So the steady searches keep the literal wall,
  and every oracle stays bit-exact.
- **Found:** the old stop kind "first-trial artefact" was WRONG for some of its own cases. A
  random sweep (600 commanded slams, wider than the sliders) left 8 stops past the walked wall. The
  first cut of the repair stepped the wall by 1/64 and claimed "negative fuel" — and the advisor's
  check found 3 of the 8 had a solvable root INSIDE the skipped step. The walked wall's edge is now
  bisected against the last failing trial. Where the root still lies below that edge, the burner
  closes on `f` ≈ 1e-6 there: the commanded temperature has fallen to the compressor's own exit
  temperature, zero or ALMOST zero fuel (not shown negative — the burner solve also refuses a tiny
  positive rise). The model says so, and the page has a FLAME-OUT stop kind, driven by one of them.
- The § 12.9 chop (1500 → 640 K over 0.06 τ) and its near-zero-rise twin now run to the end; the
  page's `low_wall_trial_fails` copy and its stop kind are retired.

### 14.2 Rung 31's turbine wall — rung 33's dispatch floor (`docs/rung33-spec.md` § SUB-IDLE)

- **Fix:** `try_r31_solve_turbine` steps its 0.02 `π_t` wall in by 0.01 while the isentropic
  `Tt5s` there is below the gas tables' 150 K floor. The (★) root is far above it.
- **Moved, and licensed as a rule** (`tests/common/turbine_wall.rs`, beside `eq_floor.rs`): exactly
  the 12 `offdesign_oracle` cells at 400 K whose 0.02 trial is below the tables — the rule NAMES
  them by that physical test, and the named set equals the moved set. Each now ends at a later,
  named guard (SUB-IDLE, the burner, the choked rebuild's nozzle check) instead of the table floor.
  Nothing else moved (checked against the unchanged code, key by key).
- The CLI's rung-33 panel no longer needs its direct-subsonic detour: the dispatch reaches the
  subsonic thrust guard itself, and the segment is the Python golden's own bytes again.

### 14.3 Rung 43's every-step schedule check (§ 12.10's "a stop the plan did not foresee")

- **The law (user decision, 2026-10-10):** applied fuel = min(schedule, limiter cuts), unchanged.
  Where the scheduled fuel has no operating point, each limiter is solved from the most fuel that
  does solve (the CEILING, `try_solvable_ceiling`) — a limiter's cut is a root, so it is the same
  cut. If any limiter cuts below the ceiling, the min is decided; if none does, the step fails as
  before. `r43_cut_below_ceiling` (plain route), and the same through `try_leg_below` on the rung
  47 and rung 52 routes.
- **A SWITCH, off by default** (`FuelLimiters::below_ceiling`). Switched on for every march, it
  crashed a shipped CLI panel: rung 58's floor dichotomy (rung 63's table) marches a floor whose
  run STOPS on this check, and its diagnostics read the scheduled instant at every recorded point.
  So the shipped rungs keep their march bit for bit, and the Controls view switches it on. **For the
  shipped march the seam stays OPEN** (CLAUDE.md), and so does a second one this found: rung 58's
  floor dichotomy is PUBLISHED from a run that stops on this solver check. *(Both CLOSED since:
  the table re-derived in § 14.4, the switch ON by default in § 14.6.)*
- **Measured** on 1 000 random Controls requests over the slider box, the same requests before and
  after: schedule-check stops 24 → 0; complete runs 564 → 587; real floor-unreachable stops
  64 → 68 (runs that went on and met the engine's own limit); one rich and two cause-unknown stops
  also ran on. The page's schedule-check stop kind is retired.

### 14.4 The switch's reach, measured; rung 63's table re-derived (2026-10-14)

- **The census.** A throwaway worktree logged every time a shipped march met the schedule check.
  It then ran the whole gate twice: switch off, and switch forced on everywhere. With the switch
  off, the check fires in test programs for rungs **49, 63, 64 and 66–73**, in their oracle and
  dispatch suites, and in the CLI (8 hits on the lagged route, 1 on the plain one). With the
  switch on, every rung's own verdict suite still passes. What fails is 10 bit-gate programs
  (17 tests: the CLI golden, seven `slice_*_oracle`, `slice_x_smoke`, `slice_v_smoke`) and **one
  reduce** (`slice_z_smoke`: rungs 66/67 → 65, 76 vs 68 points). Two more failures came from the
  instrumented copy, not the switch: `sandbox_controls`' explicit-off test was overridden by the
  census's forcing, and the page went stale because of the logging code. Rungs 65 and up have their OWN marchers, which have no ceiling, so turning the switch
  on in rung 43's marcher splits the ladder at 65.
- **What would move.** Rung 64's fuel bill (`removed_alone` 9.06e−3 → 1.44e−2 on the tilted map,
  and the `below_bare` cells with it). Rungs 66–69 and 73: march lengths 216 → 231 points and a
  few dozen keys each. Rung 63's floor table: one key in the published (bleed) table. Its oracle
  also carries stator-lever and bleed-beside-stator variants, and there 3 more `removed` keys move:
  the stator-armed floored marches stop on the check too, at 0.34 and 0.43.
- **Decided (user, 2026-10-14):** the default stays OFF *(reversed later that day — § 14.6)*. Only rung 63's floor table is
  re-derived, through a core carrier (`TwoSpoolTransientCore::below_ceiling`, guard
  `BelowCeiling`) that `floor_dichotomy` alone sets. Result in `docs/rung63-spec.md` § 3: one
  `removed` moved (0.43: 1.31e−2 → 1.66e−2), the verdicts stand, and both top reference marches
  still end early on the floor's own refusal (now reported). Bit-gate licence:
  `slice_w_oracle.rs` names the moved keys by re-running the old march's failing step
  (`tests/common/schedule_check.rs`) and holds Python's values through the unchanged route. That
  is 5 keys, pinned as MEASURED; the prediction was 2. Every other key, `m_i` and the credit
  included, is still bit-exact against PyPy and CPython.
- **Prerequisite fixed:** rung 58's `leg_residual` re-read each point at the scheduled fuel with
  the panicking call, so it crashed on any march the switch carried past the check. A point with
  no answer there is now NaN, which `s_eng` can never read as a crossing.
- **Left for the switch-over (DONE — §§ 14.5–14.6):** put the ceiling into rungs 65+'s marchers so the
  reduce holds. Then re-derive the published numbers of rungs 64 and 66–73 (and 49's
  boundary gate), licensing each moved key by this same physical test. Other readers re-solve at
  `mf_sched` too (rungs 70–79's ledgers); the census lists the programs, not yet the readers.

### 14.5 The switch-over, step A built and step B measured (2026-10-14)

- **Step A (commit 6a60097):** the ceiling is in every rung-65+ marcher, each in its own shape
  (rung 65's filter-then-fall-back via `r43_cut_below_ceiling`; the lagged fuel leg of 66/68/72
  via `try_clip_caps`; the governor of 67/70/71/72 via `try_topping_required_below`; the demand
  family 74–79 via `cap_free_gated` and the core's carrier, held for the march). One source per
  march: `lagged_bleed::ceiling_on` refuses a carrier ON with the limiters OFF. Default OFF; the
  full gate bit-exact.
- **Step B — the census, forced ON everywhere** (throwaway worktree; every ceiling search logged
  with its test). 24 of 2 403 tests fail. Every ceiling search found a ceiling. The demand
  family's marches (74–79) never reach it on a shipped grid, and rungs 70/71's records do not move.
- **THE REDUCE HOLDS.** Rungs 66/67 → 65 on the unfloored machine: 76 = 76 = 76 points, every
  key equal. § 14.4's "breaks the reduce" was the test's PINNED length (68), not the reduce.
- **EVERY MOVE IS A RUN THAT USED TO STOP ON THE SOLVER AND NOW RUNS ON.** At the points both
  runs share the values are bit-identical (checked point by point for rung 64's four moved runs,
  and key by key in every oracle). What moves is appended points and the totals / end readings
  taken over the run. Per rung:
  - **64** — the bare-plant floor runs (§ 3's denominators): shaped 136 → 151 points, tilted
    145 → 197; `removed_alone` 8.94e−3 → 1.05e−2 (shaped), 9.06e−3 → 1.44e−2 (tilted); the
    control 1.02e−2 → 1.23e−2. The armed side does not move, so § 3's contrast only widens.
  - **65** — the `b0` continuum (§ 3). The members below `b_cmd(0)` stopped on the SCHEDULE
    CHECK (0.5: 195 points; 0.727: 251) while the natural member ran 341, so the published
    table (1.326e−2 … 2.070e−2, "56 % spread") and the gated ratio 1.166 compare runs of
    different lengths — true with the switch OFF too. On a common window the ORDER INVERTS:
    lower `b0` withholds MORE (off, first 195 points: 1.005e−2 / 1.014e−2 / 1.206e−2 / 1.326e−2
    for 1.0 / 0.99 / 0.727 / 0.5; on, first 235: 1.284e−2 / 1.299e−2 / 1.580e−2 / 1.736e−2).
    Switched on, the members stop on the floor's OWN refusal (rung 49: no fuel restores it).
    The headline — the frozen valve, the continuum — holds; the materiality holds (+32–35 %);
    the published direction and the ratio gate are artefacts.
  - **66–69, 72, 73** — one march: rung 64's fuel-only reference cell, re-read by each rung,
    216 → 231 points (end `s` 1.075 → 1.15). Its `removed` 1.46e−2 → 1.62e−2, its end readings
    later in time. No ledger, rank, zero-count or armed-march key moves.
  - **58/60** — the `phi` floor on the stator-armed plant (smoke V): 31 → 35 points, `removed`
    2.10e−2 → 2.50e−2. Not published; the credit, interaction and pinned keys do not move.
  - **CLI** — one panel, rung 65's, two lines (the 0.727 row and the 1.166 ratio sentence).
- **Readers.** Every reader that re-solves at `mf_sched` (rungs 68/70's `R`, 72's `F`/`R`, both
  boundary asserts) was logged on failure across the whole forced gate: ZERO failures (log
  verified by a positive control). They never visit an appended point.
- **Instrument-only failures** (would be re-scoped, not re-derived): rung 63's and slice W's
  licence (`schedule_check.rs` re-runs the OFF route), `sandbox_controls`' on-vs-off comparison,
  `slice_ag_dispatch`'s planted-defect reach test (the ceiling absorbs the planted early stop),
  the stale page, and direct `integrate_fuel` callers built from `FuelLimiters::default()` while
  the carrier is on (the guard firing as designed; a real flip changes both defaults).

### 14.6 The switch ON by default (2026-10-14, user decision after § 14.5)

- **The flip.** `FuelLimiters::default()` and every new machine's carrier take
  `fuel_transient::ceiling_default()` — `true` unless a `CeilingDefault::set(false)` guard is in
  scope on this thread. The guard must enclose the machines' construction as well as their
  marches. Rung 63's `floor_dichotomy` and the sandbox still set it ON explicitly, so the OFF
  route is exactly the pre-flip code.
- **Before the flip, the branches no grid reached** (the advisor's blocker). A census of the
  forced-on gate, every ceiling search tagged with its call chain, showed shipped grids exercising
  rungs 43, 65, 68 and 72's branches and NEVER rung 66's lagged fuel leg, rungs 67/70's governor,
  or rung 74's demand cells (the last only inside a planted-defect test). `tests/ceiling_branches.rs`
  builds one point each where the OFF march stops on the check (a 0.86 floor and a low valve start
  for 66; a 1600 K schedule under a 1100/1200 K redline for 67, 70 and 74) and gates: OFF shorter,
  shared points bit-identical, the scheduled instant unsolvable at the first ON-only point, ON to
  the end (66: to the floor's own refusal, valve fully open).
- **The oracles.** Each Python oracle runs its emitter with the switch OFF and is held to Python
  exactly as before. A third test runs it at the default and licenses every difference by
  `tests/common/ceiling_rule.rs`: no key vanishes (except a sampled trajectory's former end point,
  superseded by a later one); a new key is a trajectory point; a changed key is a whole-run
  reading (sum, extremum, count, end value). Counts pinned from the measured flip as (changed,
  new, superseded): X 16/0/0, Y 6/0/0, Z 7/105/0, AA 4/68/34, AB 6/68/34, AD 6/0/0, AE 12/0/0,
  W 0/0/0, X-smoke 8/0/0, V-smoke 7/4/4 — the census's numbers exactly. Rung 63's oracle does not
  move at all.
- **Published numbers that moved:** rung 64 § 3 (the bare-plant leg alone 8.94e−3 → 1.05e−2 and
  the 0.7945 control 1.02e−2 → 1.23e−2; the 0.7725 control and every armed value unchanged) and
  rung 65 § 3 (corrected first — its table compared unequal runs even with the switch OFF; at the
  default the ±0.01 pair reads 1.292 over 309 points). The CLI's rung-65 panel is re-cut (two
  entries). No verdict moved.
- **Instruments re-scoped to the OFF route, not re-derived:** rung 63's top-two-runs test and slice
  W's licence (their "old route" is now explicitly OFF), slice AG's planted-defect reach test, and
  slice Z smoke's unfloored reduce grid (68 OFF, 76 at the default — the reduce holds at both).
- **Rung 49** was reached (21 ceiling searches in its suite) and nothing it gates moved.
- **The census scripts** (force the switch ON in a throwaway worktree and log every ceiling search;
  then tag each search with its call chain): `docs/plans/scratch-archive/ceiling-rollout/`
  (`force_on.py`, then `tag.py`), run on a worktree's `rust/src`.
- **Found on the way, NOT the switch — TRACED and FIXED the same day:** one rung-70 machine
  REUSED for six marches in a row (redline 1200, 1200, 1100, 1100, 1050, 1050 K) started its
  SIXTH march off-equilibrium — with the switch OFF for all six, ON for all six, or alternating
  alike; marches one to five equal a fresh machine's bit for bit. **Cause: the live LP map.** A
  stator march writes the live maps as the stators move and, by Python's shape, never restores
  them; march 5 is the first to end with the stators off zero (`v` = −8.3e-4), and rung 68's
  `_arm` returns early once the march's stator position is cleared, so march 6's `s = 0`
  equilibrium was solved on march 5's map (re-measured on `ceiling_branches.rs`' rung-70 point:
  `nu_lp` 0.747544 → 0.747260, `Tt4` 1000.29 K). **Fix:** `stator_march_scoped` resets both maps
  to the AS-BUILT ones (design + any constant setting) at the START of every march; a no-op on a
  fresh machine. **Not also at the END** — measured: that moves 19 slice-V oracle keys (every
  reading of the map a march LEAVES, which slice V gates as Python's persistence) and nothing
  else; start-only moves no key. Gate: `rust/tests/reused_machine.rs` (fails with the reset
  removed). The sandbox never builds these machines.

## 15. Both airflow levers at once — rung 86 (2026-10-14) — not a slice

Rung 86 (`docs/rung86-spec.md`) measured the plant § 12.3 refused — the stator schedule and the
bleed schedule together beside any fuel limiter — and found it runs on the model as it stood (rung
62's machine, rung 63's merge). So the refusal is lifted.

- **The knob:** a fourth lever choice, *Both: stators and bleed together (rung 86)*, showing both
  sets of numbers and the floor's incidence option. One select option rather than two switches:
  functionally the same, and every saved `"lever"` setting stays valid. Both levers' prechecks apply.
  The lever chart draws the stator closing and the air bled (as a fraction) together.
- **Gates:** `tests/sandbox_controls.rs` replays four both-lever cases against the model's own march
  bit for bit (accel; redline + its lag; floor + release; incidence floor on the flat-LP map) and
  checks each limiter holds; `check_requests` adds four (incl. a stators-alone run starved at the
  first step); `browser.mjs` checks the bleed rescuing a run the `φ` floor starves with the stators
  alone, and that starved run shown as a result (0 points, plain words, no error card).
- **Crash map** (§ 14.3's instrument, every request both levers, 1 000 requests, seed 11): no
  panic — 613 complete, 309 refused before the run, 78 stopped with a reason (47 floor unreachable,
  21 lean, 4 rich, 6 cause unknown — the kind earlier maps also found).
- **What the page now shows**, rung 86's knob finding: a stall floor read as a flow coefficient
  fights the stators and can starve the engine; the bleed lifts it back.

## 16. The controlled valve — rung 87 (2026-10-15) — not a slice

Rung 87 (`docs/rung87-spec.md`) measured rung 64's controlled bleed valve beside the stator schedule and
gave it an incidence reference. The page gets it as a mode of the bleed lever.

- **The knobs:** the bleed valve moves *on a speed schedule* (today's, the default — every saved setting
  stays valid) or *opens on its own*: the LP flow coefficient it holds (`valve_phi`, at the design stator
  setting, as the floor's), its widest opening (the existing "most air bled", `b_max`), and what it
  watches — the flow coefficient or blade incidence (`valve_ref`). Allowed alone or with the stators
  ("both") and beside every fuel limiter. The LP flow-coefficient chart draws what the valve holds at each
  point (an incidence valve's target moves with the LP stators); the lever chart's bleed is the model's
  own committed opening at the point, re-solved — never `b_of` (rung 64 § 5.22 (ii)).
- **Built through rung 64's builder whenever the valve is on**: rung 62's, which the view used for
  everything, drops a floor SILENTLY (rung 87's G1). Every other setting still builds through rung 62's,
  bit for bit as before.
- **Prechecks:** the valve's widest opening must be above 0 (a valve that cannot open holds nothing) and
  below 0.5; its target above 0. Each fires only on its case.
- **Gates:** `tests/sandbox_controls.rs` replays five valve cases against the model's own march bit for
  bit (and re-reads and re-steps every point), checks the valve holds its own target while it rides and
  that only the incidence valve's target moves (and only with the LP stators), and drives H5 — the
  incidence valve beside a flow-coefficient floor stops at "floor unreachable", the matched floor runs
  on; `check_requests` adds four; `browser.mjs` clicks the valve on and to incidence, checks the target
  line is drawn, and drives the same pair.
- **Found while gating — rung 66 on the page.** The matched pair at the SAME set point (valve and floor
  both 0.78, both on incidence) split one pin by rounding: the browser build and the native model, equal
  to the last place elsewhere, landed 1e−4 apart in shaft speed (65 % in the bleed at one point). The
  check case sets the floor just below (0.77, where it never acts — rung 87's D4) and the page's note
  says so: set the floor a little below the valve.
- **Crash map** (§ 14.3's instrument, every request the controlled valve, levers bleed / both, 1 000
  requests, seed 11): no panic — 559 complete, 345 refused before the run, 96 stopped with a reason (78
  floor unreachable, 9 lean, 9 cause unknown — the kinds earlier maps also found).

## 17. The slow valve — rung 88 (2026-10-15) — not a slice

Rung 88 (`docs/rung88-spec.md`) gave rung 87's controlled valve rung 65's lag, watching either wall. The
page gets it as the valve's speed.

- **The knobs:** the valve *moves instantly* (rungs 64/87, the default — every saved setting stays valid)
  or *moves slowly*, with its time constant (`valve_speed` `instant` / `slow`, `valve_tau`, default 0.05). The lever chart
  draws the valve's RECORDED position: a slow valve's opening is a march state, so the page re-reads each
  point at it (`ControlsSolver::reread`) — re-solving would hand back the command, not the valve.
- **Built through rung 65's builder whenever the valve is slow** (rung 64's refuses a lag); route
  `valve_lag`, whose step re-run (`der_valve_lag`) copies rung 65's derivative in its order and is held to
  the march bit for bit.
- **Prechecks:** the time constant at least one march step (0.02). Measured on this view's engine (five
  cases, `τ` 0.01–0.3, the page's step against a 4× finer one): at half a step (`ds/τ` = 2, rung 65's own
  bound) the LP flow coefficient is 7e-4–1.1e-3 off; at one step ≤ 9e-5 on the default move (≤ 2.4e-4 on a
  0.1 ramp, where the ramp itself sets the error) — the other routes' ~1e-4 (§ 12.10). And a slow valve
  beside rung 47's lagged governor or rung 52's release is refused (rung 65's march refuses both: two lags
  on two levers is rung 66's cascade).
- **Gates:** `tests/sandbox_controls.rs` replays three slow-valve cases against the model's own march bit
  for bit (and re-reads and re-steps every point, the valve state included), checks the page reads the
  recorded position and that the valve lags its command and target; `check_requests` adds three;
  `browser.mjs` clicks *Moves slowly* and checks the route, the knob and that the slow valve reaches its
  widest opening later than the instant one.
- **Crash map** (§ 14.3's instrument, every request the SLOW valve, levers bleed / both, 1 000 requests,
  seed 11): no panic — 585 complete, 328 refused before the run, 87 stopped with a reason (78 floor
  unreachable, 7 lean, 2 rich).

## 18. The asymmetric valve — rung 89 (2026-10-15) — not a slice

Rung 89 (`docs/rung89-spec.md`) gave rung 88's slow valve a second time constant: it opens at one speed and
closes at another. The page gets it as the slow valve's closing speed.

- **The knobs:** a slow valve *closes at the same speed* (rung 88's, the default — every saved setting stays
  valid) or *closes at its own speed*, with its closing time constant (`valve_close` `same` / `own`,
  `valve_tau_close`, default 0.2); "Time to move" is then the OPENING constant. An instant valve does not
  read either. A note under the knob says what the rung found: each valve is priced by one edge.
- **Built through rung 65's builder and march** (route `valve_lag`, unchanged): the valve is
  `lagged(valve_tau).closing(valve_tau_close)`; the step re-run (`der_valve_lag`) reads the same
  `lag_rate`, so the replay stays bit for bit.
- **Prechecks:** the closing constant at least one march step (0.02), as the opening one — RK4's bound reads
  the smaller. Measured at the page's own step (rung 88's lesson) with opening 0.05 and closing 0.02–0.3,
  five cases, against a 4× finer step: at one step the LP flow coefficient is ≤ 9.2e-5 off on the default
  move (≤ 1.7e-4 on a 0.1 ramp, where the ramp sets the error at every closing speed) — rung 88's ≤ 9e-5
  for the opening one.
- **Gates:** `tests/sandbox_controls.rs` replays two closing-own cases (fast open / slow close, and the
  reverse) against the model's own march bit for bit, checks the precheck's floor (refused below one step,
  accepted at it, unread when closing at the same speed or when instant) and the round trip of both keys;
  `check_requests` adds three; `browser.mjs` clicks *Closes at its own speed* and checks the route, the knob
  and that a slower close (0.2 against 0.05) holds bleed to a later point than the symmetric valve.
- **Crash map** (§ 14.3's instrument, every request a slow valve closing at its own speed, levers bleed /
  both, 1 000 requests, seed 11): no panic — 578 complete, 329 refused before the run, 93 stopped with a
  reason (77 floor unreachable, 11 lean, 5 rich).

## 19. The customer bleed — rung 90 (2026-10-15) — not a slice

Rung 90 (`docs/rung90-spec.md`) put the aircraft's customer bleed on the HP compressor's delivery. The page
gets it on the Controls view, beside the engine's own knobs (it is a load on the engine, not a lever).

- **The knobs:** *Aircraft takes no air* (the default — every saved setting stays valid), *Takes a share*
  (`customer` `share`, `customer_share`, default 0.05) or *Takes a fixed amount* (`amount`,
  `customer_amount` × the design airflow, default 0.05). A note under it says what the rung found; the table
  gains "Air the aircraft takes, start / end", and the run carries a `customer` column.
- **Built** by putting the demand on the design engine (`with_customer_bleed`) before the usual builder, so
  every machine the view uses (rungs 62/64/65) and every sibling it makes keep it; rung 62's four cells then
  dispatch to rung 90's.
- **Prechecks:** a share or an amount outside [0, 0.5), in words. An endpoint the demand starves (the model's
  "customer bleed takes … more than half") is refused in words before any march: a fixed amount collapses the
  operating point (taking air lowers the flow, which raises the share), so on this engine 0.3 has no steady
  point even at 1400 K. A run where it happens mid-march stops with its own kind, `customer`.
- **Gates:** `tests/sandbox_controls.rs` replays four customer cases against the model's own march bit for
  bit (alone, with the redline, beside the instant valve, beside the slow valve + stators + schedule), checks
  that the amount mode delivers its amount at both endpoints, the precheck, the endpoint refusal and the stop
  classifier, and the settings round trip; `check_requests` adds four; `browser.mjs` clicks *Takes a share*
  and *Takes a fixed amount* beside the instant valve and checks the valve bleeds more, and more again.
- **Crash map** (§ 14.3's instrument, every request a customer bleed — share or amount over the slider box
  0–0.2 — beside every lever, both bleed modes and both valve speeds, 1 000 requests, seed 11): no panic —
  680 complete, 262 refused before the run (one of them an endpoint the demand starves, an amount of 0.2 on
  a 670 K end), 58 stopped with a reason (42 floor unreachable, 10 lean, 5 rich, 1 cause unknown on the
  governor-lag route). No run stopped mid-march on the customer bleed itself.

## 20. Turbine cooling air — rung 91 (2026-10-15) — not a slice

Rung 91 (`docs/rung91-spec.md`) cooled the turbines with HP-compressor air that comes back: vane air before the HP
rotor, blade air after it. The page gets it on the Controls view, under the customer bleed (it is part of the
engine, not a lever).

- **The knobs:** *Turbine cooling, vane air* and *blade air* (`cool_vane`, `cool_blade`), shares of the HP
  compressor's air, sliders 0–0.2, default 0 (no cooling — every saved setting stays valid). A note under them,
  shown when either is set, says what the rung found; the table gains "Turbine cooling air, vane / blade". The
  numeric knobs refresh the view only for these two (to show the note); the others still only schedule a run.
- **Built** by putting the cooling on the design engine (`with_turbine_cooling`) after the customer bleed, so every
  machine and sibling keeps it; rung 62's four cells then dispatch to rung 91's first.
- **Prechecks:** each share in [0, 0.25), and with a customer SHARE the two together below half, in words. With a
  customer AMOUNT the bound moves with the throttle: an endpoint it starves is refused in words (the model's
  "together take … more than half"), and a run where it happens mid-march stops with rung 90's kind, `customer`.
- **Gates:** `tests/sandbox_controls.rs` replays three cooling cases against the model's own march bit for bit
  (vane alone; both streams with a customer share and the redline; blade air beside the slow valve and stators),
  checks the cooling reaches the closure, the prechecks, the endpoint refusal and the stop classifier, and the
  settings round trip; `check_requests` adds four; `browser.mjs` types 0.05 into the vane box beside the instant
  valve and checks the note shows and the valve bleeds more.
- **Crash map** (§ 14.3's instrument, every request carrying cooling — both shares over the slider box 0–0.2 — and a
  customer bleed off, share or amount, 1 000 requests, seed 11): no panic — 512 complete, 427 refused before the
  run (80 of them an endpoint a customer amount plus the cooling starves, 5 a customer share plus cooling past
  half), 61 stopped with a reason (50 floor unreachable, 5 lean, 3 rich, 2 cause unknown on the governor-lag route,
  1 the customer amount and cooling outgrowing the engine mid-run).

## 21. The aircraft's generator — rung 92 (2026-10-15) — not a slice

Rung 92 (`docs/rung92-spec.md`) put the aircraft's generator load on a shaft: power, no air. The page gets it on the
Controls view, under the turbine cooling (part of the engine, not a lever).

- **The knobs:** three buttons, *No generator load* / *On the HP shaft* / *On the LP shaft* (`generator`: `off`, `hp`,
  `lp`; default off — every saved setting stays valid), then *A fixed power* / *A share of the turbine's work*
  (`generator_by`, default power), with *Power* (× the HP compressor's design power, the same unit on both shafts;
  slider 0–0.1, default 0.02) or *Share of that turbine's work* (slider 0–0.3, default 0.05). A note under them says
  what the rung found, including that the burner temperature does not rise to carry the load; the table gains
  "Generator load, start / end (% of its turbine's work)", read by `shaft_offtake::shares` (held to the steady
  booking on every lower path by rung 92's plant gate).
- **Built** by putting the load on the design engine (`with_shaft_offtake`) after the cooling, so every machine and
  sibling keeps it; rung 62's closure cells then mark their closures and the power and tail cells take the load off.
- **Prechecks:** a share or a power in [0, 0.5), in words, only when the generator is on.
- **An endpoint the load removes is named, not guessed.** A load the engine cannot carry slows a spool until its
  compressor leaves the map, and the equilibrium search then reports only the map. So when an endpoint fails with the
  generator on, the page (i) with no airflow lever asks the steady engine — rung 92's plant gate makes it the same
  machine — and uses its own refusal ("half or more" / "no steady point") when it gives one; (ii) otherwise re-solves
  that throttle with the generator OFF, and only if that has a steady point adds "With the generator off this throttle
  does have a steady point, so the generator's load is what removes it". A run stopped mid-march by the load is the
  stop kind `generator`.
- **Gates:** `tests/sandbox_controls.rs` replays three generator cases against the model's own march bit for bit (HP
  power alone; LP power beside the instant valve; HP share with vane cooling, a customer amount and the redline),
  checks the load reaches and marks the closure, the share reading, the prechecks, both endpoint wordings (and that a
  throttle with no steady point even unloaded is NOT blamed on the load), the stop classifier and the settings round
  trip; `check_requests` adds four; `browser.mjs` presses *On the HP shaft* beside the instant valve, checks the knobs
  and note show and the share reads above 2 %, that the valve bleeds more than with no load, and — pressing *On the
  LP shaft* — less with the same power on the LP shaft.
- **Crash map** (§ 14.3's instrument, every request carrying a generator — HP or LP, by power over 0–0.1 or by share
  over 0–0.3 — a customer bleed off, share or amount, no cooling; 1 000 requests, seed 11; `scratch-archive/rung92/`):
  no panic — 523 complete, 419 refused before the run (72 an endpoint a customer amount starves, 129 an endpoint with
  no steady point: 79 of them named on the load — 76 that run with the generator off, 3 whose OTHER endpoint fails
  unloaded — and 50 that fail unloaded too, not blamed on it), 58 stopped with a reason (46 floor unreachable, 5 lean,
  4 rich, 2 other — both replayed: rung 46's refusal, the redline below the lowest burner temperature the flow
  allows at that speed — 1 cause unknown on the governor-lag route). No run stopped mid-march on the generator itself.
- **A flaky check, not this rung's:** `browser.mjs`'s `chrome_left_nothing_running` failed in 2 of 4 runs on
  2026-10-15 (once in the full gate, once standalone just after it): a Chrome helper of the run outlived the 15 s
  wait, then exited on its own (none left a minute later). Its bar was NOT raised; a rerun passed. If it recurs,
  log which helper type lingers (`--type=`) before changing the wait.

## 22. Cool the turbine — rung 93 (2026-10-16) — a new view, and a readout on Controls

Rung 93 (`docs/rung93-spec.md`) gave the turbine's vanes and rotor blades a metal temperature (Gauntner's 1980 NASA
correlation on rung 91's two streams). The user chose (2026-10-16) a NEW view for the steady trade, an opt-in readout on
the Controls slam, and the limits set by a material year and a part life.

- **The view, *Cool the turbine*** (`sandbox_cooling.rs`): the Controls engine. Knobs: the framing (*this engine, run
  hotter* — rung 91's fixed hardware — or *redesigned at each temperature*), the cooling technology (Gauntner's four
  FACTORs), the material year (1950–2030) and the part life (a dropdown of five lives), the map shape (this engine only),
  and *your cooling* (vane and blade shares, 0–0.24). Requests: `cooling_knob` (the hottest burner temperature with your
  cooling and with none, and which row sets it), then `cooling_point` streamed over 1300 → 2200 K by 50 (the cooling
  that holds both metals at their limits, its thrust and fuel per unit thrust against the same engine uncooled, the
  cooling air's temperature, the shaft speeds), all on the view's own Worker, killed by a knob move. Opens on film
  cooling, 1985, 10 000 h, 5 % vane and 3 % blade air.
- **The knob's search starts where the engine solves.** The crash map found much cooling unchokes the nozzle at the
  search's 700 K floor (the two-spool model's open seam), so a search from there refused a hottest temperature far above
  it: in the first map 114 of 294 knob requests were answered, against 206 after the fix (the rest hold all the way to
  2400 K; how many of those also hit the floor before the fix was not counted). The view now steps the floor up by 50 K to the first throttle the engine
  solves at; the model's `try_max_tt4` stays strict.
- **The Controls readout** (`metal_on`, `metal_tech`, `metal_year`, `metal_life`; off by default — uncooled, the metal is
  over its limit at every opening throttle, so every saved setting stays valid): both metals at every recorded point
  (`MetalInputs::of_shares` at the march's own instant) and at both steady ends, a *Turbine metal* chart button with both
  limits drawn, and table rows (peak, limits, end). It moves nothing else in the run (gated: every other column bit for
  bit). On the default slam with 5 % vane and 3 % blade air the overshoot lifts the blade 137 K above its end metal, to
  within 9 K of its limit (gated).
- **Gates:** `tests/sandbox_cooling.rs` (each request IS rung 93's call, bit for bit, on a rig built in the test; the
  readout and its non-interference; the knob's floor; every refusal in words); `tests/sandbox_page.rs` (the new knobs,
  toggles and dropdowns are the model's settings and choices); `check_requests` adds 20 — 16 for the view, 4 for the readout (under the Controls bar: worst
  4.5e−12 relative over the whole iterated class); `browser.mjs` adds the readout and the view (the opening knob and
  three sweep points against native, pin & compare, the redesigned framing, a refused year, a sweep stopped by a knob
  move).
- **Crash map** (§ 14.3's instrument, 1 000 requests over the view's knob box, seed 11; `scratch-archive/rung93/`): no
  panic — 703 sweep points answered, 3 refused at the stream cap; 294 knob requests, 206 answered and 88 holding all the
  way to 2400 K (much cooling), none refused after the floor fix.

## 23. A burner that carries the load — rung 94 (2026-10-16) — not a slice

Rung 94 (`docs/rung94-spec.md`) lets each end of the Controls move hold the thrust or the HP speed this engine gives at
its throttle with the generator off, the burner temperature floating to carry rung 92's generator; the metal limit
(rung 93) reported, and binding on a switch (the user's choices).

- **The knobs:** *Throttle holds burner temperature / Holds thrust / Holds HP speed* (`hold`: `tt4`, `thrust`, `speed`;
  default `tt4` — rung 92's throttle, so every saved setting stays valid), under the generator's knobs with a note; and
  *Stop the burner at the metal limit* (`hold_cap`, default off) inside the metal readout's box. Table rows: the ends'
  burner temperatures (set → actual) and what they hold (achieved / asked, "metal cap" when it stopped them).
- **Built** in `sandbox_controls::build`: with the generator on and a hold chosen, each end's target is read off the same
  machine built WITHOUT the generator at the set throttle, and `burner_hold::try_hold_root` lands the loaded machine's
  equilibrium on it (600–2000 K, the floor bisected); the cap bisects the loaded equilibrium's metal excess between the
  floor and the hold. The fuel then ramps between the two ends as before.
- **Prechecks, in words:** the cap needs a hold (nothing floats otherwise), the metal read, and cooling (uncooled, the
  1985 metals allow about 990 K). A hold the load cannot meet is refused naming its bound (the least thrust the load
  allows, or the range's top).
- **Gates:** `tests/sandbox_hold.rs` (holding the burner temperature, or any hold with no generator, is the earlier run bit
  for bit; a thrust hold's ends meet the unloaded engine's thrust to 1e−9 at both ends, both shafts; the cap lands on the
  limit and falls short; every refusal); `tests/sandbox_page.rs` (the switch and toggle are settings); `check_requests`
  adds 4; `browser.mjs` presses *Holds thrust* with the HP generator on and checks the hotter ends, the met target, the
  note and the table.
- **On the default slam** (1000 → 1400 K, 2 % generator, thrust held) the ends run at 1034.9 / 1421.7 K (HP shaft) and
  1031.2 / 1421.1 K (LP shaft).
- **Crash map** (§ 14.3's instrument, 1 000 requests, seed 11, every one with the generator on — either shaft, by power
  or share — and a hold, thrust or HP speed, the cap on in 30 %, cooling in half, every map and lever, both ends over
  the slider range; `scratch-archive/rung94/`): **no panic.** 501 ran to the end (14 with the metal cap stopping an end),
  31 stopped early with a reason; refused in words: 161 the cap without cooling, 119 an end whose UNLOADED engine has no
  steady point (so it sets no target), 109 a target below the least the load allows, 47 a search that met a throttle
  where the compressors would leave the maps the model covers (mostly heavy cooling, the HP shaft slow), 24 no solve
  anywhere in range, 8 a target above the range. Native time: median 0.06 s a request, slowest 0.8 s (an LP generator
  with heavy cooling and both levers).
- **The cap with the generator off** was first ALLOWED and did nothing (the final review's catch: with no load nothing
  floats); it is now refused in words, with a test and a check request.

## 24. A governor in the slam — rung 95 (2026-10-16) — not a slice

Rung 95 (`docs/rung95-spec.md`) governs the Controls move itself: the quantity the throttle holds (rung 94's switch —
burner temperature, thrust, HP speed) follows a straight line from the start end's target to the final end's over the
ramp, and the governor's fuel is the demand the limiters cut (the user's choices: both speed laws as a toggle; clamp and
report at the rich and the lean edge; the metal cap cutting all along the move).

- **The knobs:** *Govern during the move* (`govern`, default off — every saved setting stays valid); holding HP speed,
  *Commanded rate* / *Proportional* (`speed_law`: `rate`, `prop`) with *Time to close the gap* (`gov_tau`, default 0.1,
  0.04–1 τ) or *Gain* (`gov_gain`, default 20, 1–50). The hold toggle moved OUT of the generator's box (it matters with
  no generator once the move is governed), with a note that ungoverned and unloaded all three holds are one run.
- **Readouts:** the demand drawn (dotted) on the chart of the held quantity; a second strip above the plot for the
  governor's edges (rich / lean); the fuel chart's dashed line becomes "fuel the governor asks for"; the holder strip
  names the metal cap; table rows *The governor* (what it holds, how long at each edge) and *Time to 95 % of the thrust
  change*. Stop kind `metal_cap` (the metal over its limit even at the leanest fuel), in words.
- **Built** in `sandbox_controls`: `Route::Governor` first in the dispatch; the march and the stop re-run call the
  model's own `governor::try_der` / `try_rk_step` (no copy, so the re-run is bit for bit by construction); the cap's
  ends with no generator are bisected by rung 94's `try_metal_cap`. Thrust is read by `governor::thrust`, which
  `ControlsSolver::thrust` now calls.
- **Prechecks, in words:** the governor beside rung 47's lag, 52's release or 88's slow valve (each its own marcher);
  `gov_tau` below 0.04 τ; a gain outside (0, 50]; the cap on the burner-temperature hold; the cap without the metal
  read or cooling. With the governor on, the cap no longer needs the generator.
- **The two knob limits are the step check's** (spec § 4): at 0.02 against 0.005 the gain 50 is 5e−5 in flow
  coefficient (100: 1e−3), the rate's two steps under 1e−4 (one step: 1.1e−3). On the lean edge the run is ~10× rougher
  (the closure's search bound is not smooth) — the note says so.
- **Gates:** `tests/rung95.rs` (the model's: the reduce to rung 40's temperature-commanded march, the fixed point, the
  identity, the step check, the metal leg, the re-run, H1–H6, the deep slowdown); `tests/sandbox_governor.rs` (off is
  the earlier page bit for bit; the demand, clamp and holder columns; every refusal); `tests/sandbox_controls.rs` (the
  settings round-trip with the new keys); `check_requests` adds 6; `browser.mjs` ticks *Govern during the move* with
  thrust held and checks the route, the demand met, the hotter peak, the drawn demand, the legend and the table row
  (its screenshot un-clips the settings panel, which a full-page shot otherwise shows only from the top).
- **Crash map** (1 000 governed requests, seed 12; `scratch-archive/rung95/crash.txt`): no panic; 536 ran to the end,
  66 stopped with a reason, the rest refused in words. Its first run found 3 stops of the governor's own (no runnable
  fuel to start its search from on a deep slowdown) — fixed by a fallback scan reached only where the first three tries
  fail. Native time: median 0.10 s a request, slowest 1.35 s.

## 25. A governor that infers thrust — rung 96 (2026-10-16) — not a slice

Rung 96 (`docs/rung96-spec.md`) lets rung 95's governed thrust be READ, as a real control reads it: from the engine
pressure ratio or the fan (LP) speed, turned into thrust by a 13-row table of steady points (the setup proposed to the
user, who answered "go": both readings as a toggle; the throttle's straight line in thrust read through the table; rung
95's speed laws on the LP shaft; the table made on the generator-off engine or on this one).

- **The knobs:** under *Govern during the move*, holding thrust, *Thrust read directly* / *From the pressure ratio* /
  *From the fan speed* (`thrust_by`: `true`, `epr`, `n1`; default `true` — every saved setting stays valid); with the fan
  speed, rung 95's *Commanded rate* / *Proportional* and their knobs now act on the LP shaft (same limits, 0.04 τ and 50,
  from the LP laws' own step check); with the generator on, *The control's table was made on the engine with the
  generator off* (`table_clean`, default on). Choosing a reading switches the metal cap off (refused beside it).
- **Readouts:** on the thrust chart, the thrust the control believes (dashed) and the thrust the throttle asks for
  (dotted) beside the true thrust; with the fan speed, the fan speed it asks for on the speed chart; table rows *The
  governor* (names the reading) and *Thrust given over what the control believes, most and least* (% of the thrust
  change); with a calibrated end, *What the ends hold* gives achieved against asked. JSON keys: `reading`, `calibrated`,
  `reading_value`, `thrust_inferred`, `thrust_demand`, `table`.
- **Built** in `sandbox_controls` on the model's `inferred_thrust::{try_table, Table}` and `governor`'s new laws; the
  calibrated ends by rung 94's `try_hold_root` with the reading held; refusals in words for an end the reading cannot
  reach (`inferred_thrust::refusal_words`).
- **Prechecks, in words:** the cap beside an inferred reading; the fan-speed rate below 0.04 τ; its gain outside (0, 50].
- **Gates:** `tests/rung96.rs` (thrust read directly is rung 95; the clean table inert with no generator; the loaded
  table's ends are rung 94's; the pressure-ratio gap IS the ram drag, ≤ 1e−12; the fixed point and hold identity; the
  LP step check; the re-run; H1–H4); `tests/sandbox_inferred.rs` (read directly the page is rung 95's; the reading, the
  belief and the table reach the page; every refusal); `tests/sandbox_controls.rs` (the round-trip with the new keys);
  `check_requests` adds 5; `browser.mjs` presses *From the pressure ratio* on the governed thrust run and checks the
  route, the reading, true over believed thrust through the move, the drawn belief and the table row.
- **Crash map** (1 000 requests, seed 13; `scratch-archive/rung96/crash.txt`): no panic; 594 ran to the end, 53 stopped
  with a reason (51 the stall floor, 2 a throttle above its redline), the rest refused in words. Median 0.23 s; the
  slowest (10 s under load, 1.6 s alone) is the lean wall beside the bleed valve — rung 95's route takes 4.4 s on it.

## 26. The nozzle pushes against the flight's own air (2026-10-16) — a solver repair, not a slice

§ 10.8's candidate seam, closed. Every off-design solver (`OffDesignMatcher`, `TwoSpoolCore`, and every
rung that copies their nozzle call — 32, 33, 34, 39, 40, 42, 62, 90, 91, 92) kept the DESIGN run's ambient
pressure and used it as the nozzle's back-pressure at every altitude, while the thrust's pressure term read the
flight's `p0`. Now the stored field is deleted and every nozzle call and choked/subsonic test reads the
flight's `p0`; `try_subsonic_operating` and `try_turbine_subsonic`, which never saw a flight, take it as an
argument. `fly_solver`'s override is gone — the model does it.

- **Nothing shipped moved — measured, not assumed.** The whole gate ran once with a logger at every changed site,
  recording each call where the flight's `p0` differed from the old stored value **bit for bit**. Hits: the
  sandbox (`sandbox_fly`, `sandbox_transient`, the CLI's `sandbox` page build) — all through `fly_solver`,
  whose override already set exactly the flight's `p0`, except `sandbox_fly`'s 12.5 kPa discriminator, which built
  the solver directly to pin the OLD behaviour and was rewritten here on purpose; and ONE shipped test, rung 41's gate 4b (two shafts at
  101 325 Pa against a 50 kPa capture). There the nozzle is choked under both values, and on a choked nozzle the
  back-pressure decides only the branch, never the arithmetic — bit-identical. Every oracle, golden and panel held.
- **The gate:** `tests/flight_back_pressure.rs` — a similarity law: same `T0` and Mach, `p0` at ¼, ½ and 2× the
  design, on the two temperature-only gases; every ratio unchanged and airflow and thrust ∝ `p0` to 1e−9, on
  rung 31 (both nozzle branches, each gas), 32, 38, 39, 42, 90/91/92 (loads as SHARES — a fixed-POWER generator
  is a bigger share of a smaller engine, real physics) and rung 40's unbalanced instant. With the old code put
  back, all five tests failed — three refused or crashed, two (32's map matcher, 40's instant) gave a wrong number.
- `tests/sandbox_fly.rs`'s discriminator now pins the other side: the model's own solver flies 12.5 kPa.
- **Unlocked, then BUILT (§ 27):** a flight knob on the two-shaft *Controls* view (§ 12.1 refused it on this seam — a
  sandbox slice), and rung 96's next seam (the control's own `δ`/`θ` corrections: a table made at one flight
  condition, flown at another).

## 27. Altitude on the Controls view (2026-10-16) — a sandbox page change, not a rung

§ 26 unlocked it; the user asked for it ("an altitude control on the sandbox's two-shaft Controls view — build it").
The *Flight* fieldset (altitude + "warmer than a standard day", linked both ways to `T0`/`p0` through the standard
atmosphere — § 9.1's convention) now also flies the Controls engine; Mach stays the design's 0.85 (its row is hidden
there). Scratch scripts and outputs: `scratch-archive/sandbox-altitude/`.

- **Built** in `sandbox_controls`: `ControlsSettings` gains `t0`/`p0` (keys `T0`/`p0`, defaults = the design flight, so
  every saved setting stays valid). `build` splits the flight in two: every CONSTRUCTOR takes the DESIGN flight (the
  hardware stays captured at 250 K / 50 kPa / Mach 0.85), every solve and the march the flight FLOWN. Rung 96's control
  table is built at the flight flown, so its open seam (a table made at one flight, flown at another) stays open. The
  reply carries `ambient` (the altitude and deviation the air reads as) and `density_ratio`. The page moves the
  *Flight* fieldset under the engine's own fieldset on this view, adds a table row naming the air and one for its
  density, and the not-settled note names thin air as a cause. Raw `T0`/`p0` need only be above zero (as *Fly it*).
- **Default = the earlier page, character for character**: every Controls check request (61: 48 that run, 12 refusals)
  through a copy of the previous commit and through this one, the new `ambient` key and settings left out — 2 105
  lines identical. The design flight is NOT a standard day: pressure altitude 5 579.3 m, 1.92 K below standard.
- **Time stays the design flight's spool time constant** (`dν/ds = ṁ·Δw/(P_ref,design·ν)`): in thinner air the shafts
  spin up more slowly on the same clock. On the default move (bare, flow/press) the peak `Tt4` is 1 571 K at −1 km,
  1 696 K at 5.6 km (standard day), 1 830 K at 11 km and 2 024 K at 20 km, and the HP shaft at the end of the 2.5 τ run is
  5e−5 / 1e−3 / 7.5e−3 / 8.5e−2 short of its final speed. The same throttle in colder air runs the shafts faster
  (final HP speed at 1 400 K: 0.948 at sea level, 0.973 above 11 km).
- **The governor's two step-checked limits scale with the air's DENSITY** `σ = (p0/p0d)(T0d/T0)` (exactly 1 at the design
  flight): gain ≤ 50/max(σ,1), time constant ≥ 0.04/min(σ,1), on both shafts' laws. Measured (0.02 vs 0.005, every map):
  at the design flight the old limits give ≤ 7e−5; unscaled, gain 50 on a cold sea-level day gave 6.8e−4 (HP) / 1.0e−3
  (LP) and τ 0.04 at 15 km +40 K 1.1e−3. The error follows the airflow at the end more closely than the density (cold days
  are worse than σ says), so with the density rule 14 of 720 governed runs over 0–20 km (all below 10 km) that never touch a fuel edge still
  exceed 1e−4 (worst 2.5e−4, a cold flat-LP day at 7.5 km). In thin air the governor rides its RICH edge, where the march
  is rougher (up to 7e−4 at the scaled limits) — rung 95 § 4's edge roughness (lean 1e−3), met in a new place. Both are
  said on the page's note, not refused. Every other step-checked knob (the lags at one step, the release, the slow valve)
  stays within about 1e−4 over sea level ±40 K and 15 km (worst 1.2e−4, the slow valve on flat-LP; runs that
  stop early are left out, since there the stopping point itself moves with the step).
- **Crash map** (§ 14.3's instrument, 1 000 requests, seed 14, altitude −1 … 25 km and ±45 K, every knob at random; and
  400 more, seed 15, −5 … 47 km and ±100 K): **no panic.** Seed 14: 393 ran to the end, 45 stopped with a reason, the rest
  refused in words. Refusals grow with altitude where a FIXED amount is a larger share of a thinner engine (customer air,
  generator power, cooling beside a customer amount) and where compressors leave their maps. One new end failure: the
  nozzle cannot push the gas out (exhaust pressure below the outside air) at a very low throttle — 3 of 1 400, 600–800 K,
  every one in air above 250 K — now in words (`endpoint_words`). The off-map words were made flight-neutral, measured:
  the flat-LP map's lowest throttle at sea level is 740 / 860 / 980 K on a −40 / 0 / +40 K day (530 / 650 / 770 K at
  11 km); a fixed 5 % HP generator lifts the lowest throttle from 660 K (design flight) to 1 010 K (11 km), none in range at
  20 km. The TOP of the range can leave the maps too away from the design flight (1 700 K at 11 km on the flow/press map;
  1 690 K runs), so the off-map words no longer say "raise the throttle" but "toward the middle of its range". All 14
  "does not settle" refusals had a lever on, as the words said, and every one also a generator load — now said too. The
  governor's limit refusals at the design flight are the earlier words, character for character (a test holds them).
  Native median 0.011 s a request, slowest 1.3 s.
- **Gates:** `tests/sandbox_altitude.rs` — missing keys ≡ naming the design flight; the ends' speeds equal and airflow,
  fuel, thrust ∝ `p0` at ×0.37 and ×1.9 (1e−9); the 1500 K point is design speed only at the design flight (the
  discriminator for a hardware re-captured at the flight flown — red-checked: with that mutation 4 of 6 tests fail); the
  accelerations ∝ `p0` and a march at `p0`×k with time ÷ k retracing the design run; the view's march is the core's at
  the flight flown; the density rule at both ends; every new refusal. `check_requests` adds 5; `browser.mjs` drags the
  altitude to 15 km against a pinned design-flight run and types a pressure (the altitude follows).
- **No Reynolds-number effect** (none in the model): component efficiencies do not fall in thin air — said on the page.

## 28. The unchoked nozzle on two shafts — rung 97 (2026-10-18) — not a slice

Found while adding a Mach knob to this view (now § 29, parked in git stash `sandbox-28-mach-wip`): since § 27 the
Controls engine can run points whose nozzle is unchoked (low throttles in dense or warm air — at sea level +40 K, 9–20 of
126 points from a 700 K start), and the two-shaft transient computed them on rung 38's two-choked-throat pin anyway; the
reply's `choked` flag was ignored by the page. Offered (refuse in words / draw dashed and warn / build the branch), the
user chose **"Fix it properly first"**. Rung 97 (`docs/rung97-spec.md`) builds the branch in the model; this view only
shows it.

- **Readouts:** the chart draws the whole run thin and dashed and its choked stretches full width over it (the *Fly it*
  slam's convention), with "thin dashed: nozzle unchoked" in the legend when any point is; the table row *Nozzle unchoked
  (of the run's moments)* — "never", or "9 of 126, starting end too". The *About* paragraph says what the branch does.
- **Words:** a model refusal `rung-97: …` (the nozzle cannot pass the flow even with the LP turbine nearly unexpanded)
  reads as "the gas leaving the turbines has too little pressure to be pushed out against the air outside, even with the
  low-pressure turbine taking almost no work from it. Raise the throttle" (`endpoint_words`). § 27's "the nozzle cannot
  push it out" stays for the pin's message, met in none of rung 97's 2 400 crash-map requests; § 27's test request now
  refuses with the off-map words.
- **Default = the earlier page:** the default move never unchokes, so every earlier check request is unchanged
  (choked instants are rung 38's pin, bit for bit).
- **Gates:** `check_requests` adds 4 (+40 K at 700 K: bare, the generator on the LP shaft, cooling, governed on the
  pressure ratio); `browser.mjs` runs the +40 K 700 K move and checks native agreement, 9 unchoked points with the start
  unchoked, the dashed path, the legend and the table row.

## 29. Mach on the Controls view (2026-10-18) — a sandbox page change, not a rung

Asked with rung 96's seam (the user, scoping rung 98: "Yes, add it"): corrected tables cannot undo a change of flight
SPEED, so the Controls engine needs a Mach knob before rung 98 can be scored over one. Built after rung 97 (§ 28),
which it waited on: low Mach is where the nozzle unchokes. Scratch scripts and outputs:
`scratch-archive/rung97/mach_*`.

- **Built** in `sandbox_controls`: `ControlsSettings` gains `m0` (key `M0`, default the design flight's 0.85, so every
  saved setting stays valid). As § 27 split the air, every constructor keeps the DESIGN flight (Mach 0.85) and every
  solve and the march fly `m0`. The governor's density rule now reads the density of the air the engine TAKES IN — the
  inlet's total density, `σ = (p0/p0d)(T0d/T0)·k(M0)/k(M0d)`, `k(M) = recovery(M)·(1 + (γ−1)/2·M²)^(1/(γ−1))` — exactly
  § 27's σ at the design Mach (the ratio is x/x). Mach must be above zero (refused in words). The page shows the Mach row
  on this view, a table row for it, and the density row now says "of the air taken in".
- **Default = the earlier page:** all 69 of the previous commit's check requests through a worktree of it and through
  this tree — every reply identical with the new `M0` settings key removed, except two refusals whose WORDS changed on
  purpose (the gain limit says "the air the engine takes in"; the off-map words add that flying faster raises
  the lowest throttle, with the flat-LP map's figures). A test holds a missing key ≡ naming Mach 0.85.
- **Reach** (the default 1000 → 1400 K move, design air): every map runs Mach 0.05–3.0 except flat-LP, whose 1000 K
  start runs to Mach 1.6. The ram heats the air the engine takes in, so the lowest start that runs rises with Mach: on
  flat-LP 1000 K at 1.6, 1100 K at 1.8, 1200 K at 2.0, 1300 K at 2.2, 1600 K at 2.6 (800–1600 K starts tried); on the
  other three maps the 1000 K start runs to Mach 3.0 and a 1300 K start at 3.5 (the ram heats the air to about 860 K).
  A first draft of the page said flat-LP was refused above Mach 1.5 at every throttle — it had been tried only from
  1000 K there; the review caught it. Flown fast at a low throttle the thrust goes negative (Mach 3, 1000 K: −324 N per
  kg/s of design airflow — the ram drag beats the jet); shown as is, said on the page.
- **Unchoked points** (rung 97's branch, drawn dashed): the default move never unchokes at any Mach. From 700 K it
  unchokes more as the flight slows — design air 1/126 at Mach 0.5, 5 at 0.05; sea level 1 at 0.85, 14 at 0.05; sea level
  +40 K 9 at 0.85, 25 at 0.05; none at 11 km.
- **The density rule with Mach** (0.02 vs 0.005, 792 governed runs: Mach 0.05–3.0 × seven airs from sea level −40 K to
  20 km × four maps × both shafts' laws, at the scaled limits): byte-identical to the same check before rung 97 (no
  governed default run unchokes). 76 runs exceed 1e−4; all but 8 ride a fuel edge (worst 6.4e−4, thin air — § 27's rich
  edge); the 8 that do not are cold flat-LP days (worst 1.7e−4). The page note says so (up to about 0.0006; elsewhere
  about 0.0001), not refused.
- **Crash map** (§ 14.3's instrument, 1 500 requests, seed 29, Mach 0.05–3.5, altitude −1 … 20 km, ±45 K, every knob at
  random): **no panic.** 558 ran to the end, 96 stopped with a reason (75 floor unreachable, 8 lean, 7 rich, 4 cause
  unknown — all on the temperature limiter's lag route, as before — and 2 at their very first step, both governed with both levers: at Mach 1.56 the redline below the burner
  temperature of the least fuel that solves — now its own kind, `redline_floor`, in words ("raise the redline, or switch
  the limiter off"; that run goes to the end with the redline at 1 650 K or off) — and at Mach 1.35 a bled closure off
  the maps, left on the catch-all words); the rest refused in words, the off-map words most of all and more so with Mach (11–41 a half-Mach band below 2.5, 70 at
  2.5–3, 102 at 3–3.5). Rung 97's "too little pressure" refusal met 4 times, all below Mach 1. Native median 0.005 s a request,
  slowest 3.3 s.
- **Gates:** `tests/sandbox_mach.rs` — the default is the design Mach and a missing key the earlier reply; the engine reads
  only its inlet (Mach 0.5 in air chosen to give the design inlet: speeds, airflow and fuel equal to 1e−10, thrust not);
  the hardware stays captured at Mach 0.85 (the 1500 K point is design speed there only); the view's march is the core's at
  the Mach flown, and Mach 0.05 from 700 K starts unchoked; the density rule at both ends with Mach; every new refusal;
  the crash map's redline stop in its own words, and running with the redline raised or off.
  `check_requests` adds 7 (Mach 0.3; 1.6 governed beside the redline; 0.05 from 700 K; 2.5 at 20 km governed; the gain
  refused at 1.6; Mach 0; Mach 3.5); `browser.mjs` types Mach 1.6 and checks the density above 1.5× (it is 1.93×), the air unchanged and
  the table's Mach row.

## 30. The corrected tables — rung 98 (2026-10-18) — not a slice

Rung 96's seam, scoped by the user (memory `rung97-decisions`, `rung98-decisions`): rung 96's thrust table, rung 48's
schedule and rung 95's steady speed table made once at the DESIGN flight and carried to the flight flown, each
correction a switch. Anchor `897001b`; spec `docs/rung98-spec.md`; scratch `scratch-archive/rung98/`.

- **Built** in `sandbox_controls` on `corrected_tables.rs`: `ControlsSettings` gains `table_at` (`"design"` default,
  the user's choice asked before the anchor froze; `"flight"` = rungs 96/97), `corr_p`, `corr_t` (both on). Inert where
  no table is read or at the design flight itself (bit for bit, gated). The Flight fieldset shows a toggle and the two
  switches whenever the run reads a table; the results table says where the tables were made, the air at the engine
  against the design's, the corrections, and splits the pressure-ratio control's gap into the air-swallowed part and
  the constant jet-pressure part. The ends' achieved / asked thrust row shows the steady miss.
- **What the new default moved:** of the 77 check requests (before this section's additions) one — rung 97's +40 K
  pressure-ratio run from 700 K — now pinned to `table_at: flight` so it keeps testing rung 97; no Rust test changed
  value (the full gate). The crash map (seed 98, 1 500 requests) against a worktree of `897001b`: 122 outcomes differ
  (after the final review's fix to the clean-table ends, rung 98 spec), every one reading a table off the design flight
  with the tables made there; 53 now refused because the design-flight table cannot reach (47 at Mach 1.5 and above), in words that say to make the tables at this flight. No panic.
- `check_requests` adds 6 (Mach 1.4 on the pressure ratio, design and this-flight tables; the fan speed uncorrected for
  temperature at 290 K; the schedule uncorrected at 216.65 K; the proportional speed law uncorrected at 290 K; Mach 3
  at sea level, out of reach); `browser.mjs` flies the pressure ratio at Mach 1.4 (both ends short of the asked thrust,
  the switches shown, the table row) and flips to this-flight tables (nothing carried, the switches hidden).

## 31. The speed axis — rung 99 (2026-10-18) — not a slice

Rung 98's seam, scoped by the user (memory `rung99-decisions`): the control's three tables made at Mach 0.3, 0.6,
0.85, 1.2, 1.6, 2.0, 2.5 and 3.0 in the design flight's air and blended at the speed flown, the trend carried beyond the
grid; or the thrust table's thrust recomputed for the flight speed; or neither (rung 98). Anchor `7113ed3`; spec
`docs/rung99-spec.md`; scratch `scratch-archive/rung99/`.

- **Built** in `sandbox_controls` on `mach_axis.rs`: `ControlsSettings` gains `speed_corr` (`"grid"` default, the
  user's choice; `"formula"`; `"none"` = rung 98). Inert where nothing is carried; at Mach 0.85 the grid IS rung 98's
  table, and at a grid slice's own flight (250 K, 50 kPa, a grid Mach) it is the table made there (both bit for bit,
  gated). The Flight panel shows a three-way toggle under the tables' place; the results row names the tables blended,
  the weight and whether the line is carried past the grid; under the formula the pressure-ratio gap splits into the
  airflow guess and the rest (zero while the nozzle is choked). `tables.speed` in the reply carries all of it.
- **What the new default moved:** of 83 check requests, 2 — rung 98's Mach-1.4 pressure-ratio run and its sea-level
  Mach-3 run (still refused, now in the grid's words) — both pinned to `speed_corr: none`. The crash map (seed 99, 1 500
  requests) against a worktree of `7113ed3`: 76 outcomes differ, every one in the predicted class; 28 refused by rung 98
  at Mach 1.5–3.5 now run; 3 that ran are refused by a slow slice's reach (a fixed customer amount at Mach 0.3; two low
  rows with no operating point at Mach 0.3 / 0.6). No panic.
- `check_requests` adds 8 (grid and formula at Mach 1.4, grid at 0.5, the fan speed at 0.15, the formula at Mach 0.05
  from 700 K, the schedule and the proportional law at 1.4, Mach 3 at sea level); `browser.mjs` flies Mach 1.4 on the
  grid (the tables named, the low end a little over) and flips to the formula (the rest nothing). `check.mjs` and
  `browser.mjs` compare the gap's two parts ABSOLUTELY (the rest is zero in algebra, rounding in arithmetic; worst
  1.9e-12, bar 1e-10 N per kg/s); the page reads "nothing" off the nozzle's choke flags, not off a rounded zero.

## 32. Below choke — rung 100 (2026-10-18) — not a slice

Rung 99's seam, scoped by the user on measured probes (memory `rung100-decisions`): the formula's jet push worked out
from the pressure ratio (exact choked or not); the fan-speed formula B (the LP shaft rebalanced against the flight's
air on the table's own rows — the user's first approximation measured weak, they asked why, then asked for a slow
grid table); two slow slices, Mach 0.05 and 0.15, in the speed grid. Anchor `4b5c769`; spec `docs/rung100-spec.md`;
scratch `scratch-archive/rung100/`.

- **Built** in `sandbox_controls` on `unchoked_formula.rs` and `mach_axis.rs`: two hidden reduce keys, `low_grid`
  (default on) and `formula_choked` (default off), written to a reply only off their defaults, so every default reply
  keeps rung 99's shape. The toggle reads *ten speeds*; the note says why the slow tables matter (an unchoked engine
  leaves a choked table's line) and what each formula now knows. Under the formula on the pressure ratio the gap's
  rest reads "nothing" off the number itself (≤ 1e−9 N per kg/s; it is zero at every point, choked or not); under the
  fan-speed formula `tables.speed.fan_rows` names the table row it settles on at each end, beside the engine's own
  burner temperature, and the results row says so.
- **What the new defaults moved:** 3 of 91 check requests (rung 99's Mach-1.4 and Mach-0.05 formula requests, its
  Mach-0.15 fan-speed grid request), pinned to the reduce keys. Crash map (seed 99, 1 500) against a clean worktree of
  `f041d2d`: no panic; 13 differ, all in the two predicted classes (grid below Mach 0.3 with a carried table; formula
  with an inferred thrust), none changing outcome class.
- `check_requests` adds 9 (the slow slices at Mach 0.1 from 700 K, both readings and the schedule; the pressure-ratio
  formula at Mach 0.05 from 700 K, at Mach 1.4, and at Mach 1.4 from 700 K; the fan-speed formula at Mach 0.05 from
  700 K and at Mach 1.4; its refusal at sea level from 600 K). `browser.mjs` runs both formulas at Mach 0.05 from 700 K
  on the page (the rest "nothing" with unchoked points; the rows named). Two bars moved, each measured: `check.mjs`'s
  Controls relative bar 1e−11 → 1e−10 (worst 1.36e−11 at the new schedule request's unchoked STEADY START, which reads
  no table — the two builds stop the closure on different iterates below choke); `browser.mjs` now compares
  `settled_lp`/`settled_hp` absolutely at 1e−11, as `check.mjs` always did (a 7.8e−10 gap 4e−16 apart read 6e−7
  "relative").
- **A flaky pair to watch (2026-10-18, not this rung's code):** in rung 100's full gate, `browser.mjs`'s
  `a_knob_move_stops_a_running_controls_run_and_the_new_one_finishes` and `…_the_cooling_sweep_…` failed (the run had
  finished before the knob moved, "kills 8 -> 8"); both passed when `check.ps1` was rerun alone, as did all 403 page
  checks. Neither run reads a table. No earlier record of this pair failing.

## 33. The airflow off the line — rung 101 (2026-10-19) — not a slice

Rung 100's seam, scoped by the user on measured probes (memory `rung101-decisions`): under the *thrust recomputed*
speed correction, where the formula gets the air swallowed — rung 100's table (the default), the speed tables, or an
exhaust-temperature sensor on the nozzle's own flow; the sensor's lag a knob, off by default, in HP-shaft time
constants; both readings. Anchor `604f6a2` (Status corrected `eab1563`); spec `docs/rung101-spec.md`; scratch
`scratch-archive/rung101/`.

- **Built** in `sandbox_controls` on `airflow_source.rs`: `ControlsSettings` gains `airflow` (`"table"` default,
  `"grid"`, `"exhaust"`), `exhaust_lag` (off) and `exhaust_tau` (1.0, 0.05–5), written to a reply only off their
  defaults, so every default reply keeps rung 100's shape; the page fills them in at the model's defaults (gated). The
  Flight panel shows a three-way *air swallowed* toggle under *Thrust recomputed* only, and under the sensor its slow
  checkbox and slider; the note says what each source knows. The gap row names whose guess it is; a new results row
  says how far a slow probe fell behind the gas (the reply's `exhaust_tt5` / `exhaust_sensor`, written only with a
  lagged sensor); `tables.speed.airflow` / `exhaust_tau` name the source off its default. Under the sensor the
  `demand` column is worked out at each point's own instant.
- **What the defaults moved:** nothing. Census against a clean worktree of `54ee954`: all 326 check requests (every
  view) byte-equal. Crash map (seed 99, 1 500, keys at their defaults): 1 500 / 1 500 identical to the worktree, no
  panic; a targeted map (seed 101, 600, every request inferring thrust under the formula) with the keys drawn: no panic,
  118 differ, all on a non-table source (77 new numbers, 14 refusals now running, 23 refused both ways, 4 newly refused —
  3 in the sensor's words, 1 in the formula's own). The map found a second defect in the sensor's end search (a loaded
  engine with no steady point at the set throttle), fixed before the spec.
- `check_requests` adds 9 (the grid's airflow at Mach 1.4 from 700 K on the pressure ratio and Mach 0.1 on the fan
  speed; the sensor at Mach 1.4 from 700 K both readings, with the customer bleed at Mach 0.5, the fan-speed
  proportional law; lagged at 1 τ (pressure ratio) and 0.5 τ (fan speed); a lag of 0.04 τ refused). `browser.mjs`
  presses the sensor at Mach 1.4 from 700 K (the run matches native, the guess is nothing, the lag box shows) and then
  the slow checkbox (matches native, the results show the probe behind the gas).

## 34. The afterburner — rung 102 (2026-10-19) — Part A on the Design view, Part B on Fly it, Part C on Controls

CLAUDE.md's open item, scoped by the user on measured probes (memory `rung102-decisions`): one rung in three parts, the
design half on the Design view, the built-engine half on Fly it AND Controls (a light-off at a chosen moment), the duct
loss derived, too hot refused in words, the nozzle on a built engine a slider with a *match* setting. Anchor `379646c`;
spec `docs/rung102-spec.md`; scratch `scratch-archive/rung102/`.

- **Built (Part A)** in `sandbox.rs` on `afterburner.rs`: `Settings` gains `afterburner` (off) and `tt7` / `eta_ab` /
  `pi_fh` / `m6` (2000 K, 0.95, 0.97, 0.20), written to a reply only off their defaults (the `defaults` op names all
  five, so the page knows their opening values). Lit, `run` continues `build_turbojet`'s own run from station 5: the
  result is the lit engine (stations 0 2 3 4 5 7 9), and the reply gains an `afterburner` block — the main and
  afterburner fuel, the duct's pressure ratio and its heating part, the burning zone's exit Mach, the throat over the
  unlit throat, the same engine unlit (`dry`, `null` where it does not run) and with no afterburner (`none`) — and a
  `ts_afterburner` curve (5 → 7). The fuel flow everywhere is the jet's fuel/air ratio (station 4's without an
  afterburner — the same value). The page: an *Afterburner* panel under *Nozzle and gas* (a switch, then the four
  knobs and a note), a row of four tiles (thrust and fuel rate unlit, the throat, the duct), station 7 in the table, the
  third curve on the T–s chart, and a paragraph in the notes. Precheck words for the three knob ranges; the model's
  refusals (too hot, too cold, a choking duct) are already plain words; any other message gets slice 1's words beside it.
- **What the defaults moved:** nothing. Census against a clean worktree of `884e60a`: all 335 check requests byte-equal;
  a crash map (seed 102, 1 500, afterburner off) identical reply for reply. Two crash maps with the afterburner on
  (seeds 103, 202): no new panic (every panic the core's own, the same on the old build without the afterburner); the
  first found three defects, fixed before the commit (a slow duct's momentum closure, the equilibrium Newton at the
  stoichiometric edge, an unlit readout refusing a lit design) — `census_a.txt`.
- `check_requests` adds 12, appended LAST (`check_ab_requests`); `check.mjs` / `browser.mjs` treat `ts_afterburner`'s
  entropy absolutely; `browser.mjs` lights the afterburner on the opening design (matches native; tiles, knobs and
  station 7 shown; 2600 K refused in words; switched off, the opening run returns bit for bit).
- **Built (Part B)** — Fly it with the afterburner fitted. `FlySettings` gains `ab` (`none` / `unlit` / `lit`, written
  only when fitted), `Tt7` (the temperature flown, written only off the design's), `a8_ratio` (the throat over the dry
  throat, 1–2) and `a8_match` (the rule); the `fly_defaults` op carries all four at their opening values. `fly` solves the
  dry engine (fitted, unlit, the dry throat) and then the point (`afterburner::FlyAfterburner`), rebuilds the station
  table through the afterburner (station 7) and checks the throat passes its flow; the reply gains the same
  `afterburner` block as the Design view's — its `dry` is the fitted engine unlit at THIS throttle and flight, with its
  shaft speed, pressure ratio, airflow and stall margin — plus `mode` and `match`, and the `ts_afterburner` curve. The
  running-line point flies the same relation (a refusal ends the line, as any non-running point does). The slam refuses
  a fitted afterburner in words (`sandbox_transient::slam_precheck`). The page: an *Afterburner* panel under *Fly it*
  (not fitted / fitted, not lit / lit; the temperature; the throat slider; *open by the rule*), the Design view's four
  tiles in fly mode, the unlit stall margin under the stall tile. Entering Fly it from a design lit on the Design view
  flies it lit, opened by the rule (what that view shows).
- **What Part B moved:** check requests 0–334 byte-equal against Part A's commit; Part A's lit replies within 3.2e−11
  (the duct and fuel solvers' bisections became Illinois, for cost — spec Part B). `check_requests` adds 13
  (`check_ab_fly_requests`, appended after Part A's); `browser.mjs` step 11c flies it lit (matches native; less thrust
  held; the panel, tiles and station 7 shown; more thrust by the rule at the dry speed; the slam refused; not fitted
  restores the plain engine bit for bit). `check.mjs`'s table bar re-measured: worst 4.84e−10 (a lit stall margin).
- **Built (Part C)** — the Controls slam with the afterburner fitted, lit at a chosen moment. `ControlsSettings` gains
  `ab` (`none` / `unlit` / `lit`), `Tt7` (2000), `light_at` (0.5 — the opening run's ramp end; on the 0.02 grid),
  `a8_ratio` and `a8_match`, written only off their defaults; the `controls_defaults` op carries all five. The design
  engine carries the fit (`TwoSpoolEngine::with_afterburner`), so every sibling the view builds has it; the core swaps its
  throat and loss when fitted. The reply gains an `afterburner` block (mode, `Tt7`, light-off, ramp, throat setting) and
  four columns (`ab_w` how far lit, `ab_throat` over the dry throat, `ab_far` its own fuel-air ratio, `Tt7` its exit).
  A stop the afterburner causes is its own kind (`afterburner`: its own refusal or the lean wall, or a step that runs
  with it put out), with words. "Holds thrust" governed while lit is refused in words (the user's decision: lit, main
  fuel lowers thrust at the moment). The page: an *Afterburner* box at the foot of the Controls panel (the switch, the
  temperature, the light-off, the throat, *open by the rule*, a note), an *Afterburner* chart (its exit temperature) and
  a summary row.
- **What Part C moved:** check requests 0–359 byte-equal against `9128307` except `controls_defaults` (the five keys);
  a crash map of 1 200 random Controls requests, each not fitted / fitted / lit: not fitted all byte-equal, no panic in
  3 600, every new lit stop named (spec Part C). `check_requests` adds 9 (`sandbox_controls::check_ab_requests`, last);
  `browser.mjs` step 16b-ab (lit matches native; knobs, chart and row shown; "holds thrust" refused; not fitted restores
  the opening run).

## 35. The steady matchers below choke — rung 103 (2026-10-19) — not a slice

- **What the pages gain:** the Blades view's plants (`Machine::below_choke`, carried into each stator sibling), the
  Cooling view's fixed engine (`FixedEngine::below_choke`) and the Controls view's generator check turn rung 103's switch
  on: an unchoked throttle is re-solved with the LP turbine free instead of refused. The library default stays off
  (every older rung, oracle and golden unchanged). Two new refusals with plain words: below idle; the nozzle cannot pass
  the flow. The Controls view's holds already ran on rung 40's equilibrium (rung 97's branch).
- **A pre-existing panic fixed:** a Cooling sweep point whose metal holds with no cooling air (`booking.expect`) —
  226 of 1 200 random low-throttle requests on the code before this rung, none after.
- **What moved** (crash map, seed 103, against `8bf63b9`): 648 byte-equal; 139 refusals now run; 76 refusals change words;
  13 Blades levers scan past their old stop; 66 Cooling knob answers start their search lower. Tests rewritten:
  `sandbox_blades` (`…_is_re_solved_by_the_view`, the 805 K case now below idle, the new words) and `sandbox_cooling`
  (the heavy-cooling knob compared like for like, the no-cooling point, the new words); `browser.mjs` step 14 (its two
  "Not modelled" rows are lever rows now). Spec § 4.

## 36. The two-shaft closure stops at the HP map's own wall (2026-10-19) — a solver repair, not a slice

Rung 103's open seam: rung 40's equilibrium refused running engines. **The cause was not the Newton's start** —
seeded AT the steady matcher's root it still failed. It was the closure (`r40_try_close` and its rung-62/90/91 copies):
the face-flow search runs up to the LP map's own wall, and nothing bounded where that puts the HP face. On the flat-LP
map there is no LP wall (`phi_max`'s sentinel, so the bare 2.5); past the HP map's wall `ψ_H < 0` and `Tt3` goes below
zero yet stays FINITE, the residual turns back negative, and the bracket's sign test refused a closure whose one
physical root sat well inside. At 11 km and high power the same happens on the curved maps (the "top of the range").
- **The repair** (`two_spool_transient::hp_wall_rescue`, shared by the four `Tt4`-driven closures): only where the old
  bracket FAILED (`g(hi)` refused, or the signs did not bracket), cap the search where the HP face reaches its map's
  wall — `ψ_H = 0.1`, the floor the LP wall already uses, no new constant — by bisection on the pre-burner face algebra,
  and solve on `[lo, cap]`. A closure whose bracket held never enters it: bit for bit. The fuel-driven closures (rung
  43's scan up from the rich wall, and its copies) already stop at the first sign change and were never affected.
- **The census and the user's decision.** The whole gate ran with the repair on: the only Python-held keys that moved
  are 4 march lengths (`slice_v_oracle`, `A/*/tsm/npts`, 48–62 → the full 151 — rung 57's `Tt4` marches stopped at an
  off-map trial; every margin and later reader held bit for bit, the minima came before the stop). The user chose
  **on everywhere** (2026-10-19): the 4 are licensed by a RULE in `slice_v_oracle.rs` (rescued in that march, the Rust
  march full length, Python's short). Rewritten: rung 57's `test_offmap_guard_…` (REVERSED: now solves, at rung 39's
  point), `slice_v_dispatch`'s P5 (its 61 → 62 length half retired; the 15.4 % margin half stands), rung 97's panel
  golden (its two flat-LP rows were the bug), `sandbox_controls` (the refused endpoints at 500 K, below the new 520 K
  edge) and `sandbox_altitude` (the warm 600 K case is now refused by the nozzle, rung 97's words; the 11 km 1700 K end
  runs). Rung 97 § 3's two vacuous flat-LP cells now solve (noted in place; not re-scored).
- **The gate:** `tests/hp_wall_repair.rs` — at every flat-LP point the steady matcher solves (four flights, 550–1500 K),
  the equilibrium lands on its speeds (independent: no closure code in common; ≤ 6e-13 measured, bar 1e-10); the 11 km
  top of range likewise; the curved maps never enter the repair over that grid; the Controls page runs flat-LP from
  600/700 K through all four closures (15 of those 16 starts were refused before). Red check (the rescue switched off by
  an env flag, then removed): 4 of 5 red — every one a refusal, which is the bug; the fifth asserts non-entry.
- **What moved on the page** (crash map, seed 36, 1 200 random Controls requests, half on flat-LP, against a worktree of
  `070b729`): 779 runs byte-equal; 378 refusals now run; 19 refusals change words; 24 identical; no panic; slowest
  reply 119 → 161 ms. The 18 still "off the maps": 4 below idle (the steady matcher agrees), 8 where the steady matcher
  also fails, in its efficiency check — measured below idle: specific thrust −1.4 to −53 N/(kg/s), nozzle CHOKED, so
  rung 103's below-idle words (unchoked branch only) never fire there (warm, fast flights at low throttle), 3 that
  run bare but not with their load (cooling, customer bleed, generator), and 3 of the open seam below. The refusal
  words were re-measured: the lowest throttle that solves at the design flight is 510–520 K on every map (flat-LP was
  750 K), ~610 K at Mach 1.6, 670–690 K at Mach 2, 800–820 K at Mach 2.6 (flat-LP was 980/1170/1520 K); the "top of
  the range" clause is gone.
- **Still open:** the Newton starts at full speed. Just above idle (556–562 K in the crash map; 550 K standard day,
  M 0.85 in the gate) the compressor exit there is hotter than the burner target, so the closure has no root AT THE
  START, while the running point has one (seeded there it solves). A start from the steady matcher would close it.
  **Closed by § 37** (a fixed lower start instead — the steady matchers panic on loaded cores).

## 37. Rung 40's equilibrium starts lower where full speed fails (2026-10-19) — a solver repair, not a slice

§ 36's open cell. Just above idle the full-speed start `(1, 1)` has no closure root at all (the compressor exit there
is hotter than the burner's target), while the running point has one.
- **The repair** (`TwoSpoolTransientCore::try_equilibrium`, the Newton itself moved to `try_newton`): only where the
  default start (`start = None`) FAILED, re-run the Newton from `LOW_STARTS` (equal speeds 0.9 … 0.4, a search choice
  like the closures' 0.02 march); the first start that solves gives the root, **kept only if the engine is not below
  idle** (rung 103's words, judged on thrust per INLET air, as the steady matchers judge it). A below-idle root is
  refused with the full-speed refusal's own words FIRST and "; and below idle … more drag than thrust (… N per kg/s)"
  after — so every reader that sorts refusals by their words (`classify`, the oracles' `abort_kind`, the page) sorts
  them as before. A call that solved from full speed never reaches it: bit for bit. Counter `take_low_start_rescues`.
- **Why the filter:** on a 26-flight probe grid the low start also lands on 650 below-idle roots (down to −149 N/(kg/s))
  that every steady matcher refuses. **Why the first start:** every low start that solves lands on one root (spread
  ≤ 1.5e-12 measured), so the order is a choice, not a selection.
- **Not one rule (the user's choice, 2026-10-19 — asked in plain terms after a side agent raised it):** the full-speed
  start still returns choked below-idle points (41 on the probe grid, hot or fast flights; the Controls page starts at
  −64 to −80 N/(kg/s) at Mach 2.6, and at Mach 2 on two maps), and the steady matchers refuse below idle only on the
  unchoked branch. Kept as it was, honest words on the new refusals; **open**: one below-idle rule everywhere (it would
  move existing results, likely Python-held). **Done in the pages, 2026-10-19 — § 38** (off in the old code).
- **The census:** the whole gate with the repair on — no Python-held key moved (2913 passed); the 2 failures were the
  stale page and `sandbox_controls`' refused endpoint (500 K now runs). Rewritten: that endpoint (470 K, the new words),
  its "does not settle" case (crash-map request 7, a lever with a generator in other air) and its acceleration-gap case
  (`accel_table` driven directly from 470 K) — the two lever cases it held refused there **run now** (the full-speed
  Newton cycled on the schedule; every start that settles lands on the rescued root): `low_start_repair.rs`.
  `hp_wall_repair.rs`' 550 K skip removed, its open-seam test deleted.
- **The gate:** `tests/low_start_repair.rs` — 4 shapes × 7 flights × 500–700 K: a rescued point always has a steady
  point with positive thrust at its speeds (≤ 1e-10; the steady matcher shares no closure code), every running steady
  point is solved; the same for the customer-bleed and cooled engines against rung 90/91's matchers; the open cell;
  one root from every start; a below-idle root's reason; the lever cases; the Controls page from 560 K through all
  four closures. Red check (an env flag, then removed): 4 of the first 5 red plus `hp_wall_repair`'s flat-LP test.
- **What moved on the page** (crash map seed 36, 1 200 requests, against a worktree of `534f69e`): 1 185 byte-equal;
  2 refusals now run (560–562 K, just above idle); 13 change words, each "off the maps" → below idle (−13 to −90 N/(kg/s));
  no panic. 3 low-throttle "off the maps" stay (575–598 K, each with a load or the bleed, the search slowing the LP
  shaft to ~0). Time: not measured cleanly on a busy machine — the slowest request moved between two runs of the same
  code (175, then 655 ms; the first run's slowest at 15–19 ms the second time), but the total rose in both (12.7 s →
  14.9 s, 17.5 s), and a refused call now costs up to 7 solves where it cost 1. The page's lowest start, re-measured: 480 K at the
  design flight on every map (was 510–520), 580 K at Mach 1.6 (610–620), 690 K at Mach 2 and 800–820 K at Mach 2.6
  unchanged (there the full-speed start already accepts below-idle points); standard sea-level day 550–560 K, +40 K
  610–620 K. The refusal words carry the running floors; "off the maps" no longer quotes a low-throttle floor.
- Probes archived: `docs/plans/scratch-archive/low-start-repair/`.
## 38. One below-idle rule in the pages (2026-10-19) — a page rule, not a slice, not a rung

§ 37's open item, plus the crash map's last three "off the maps" refusals. The user's choice (asked in plain terms):
**refuse consistently, behind a switch — off in the old code, on in the pages** (as rung 103's), not "show a point
with more drag than thrust, with a warning" (a bigger job: the shared scoring is not built for negative thrust).
- **The three refusals first** (crash map seed 36, requests 158/349/399): a probe of 54 starts over both shafts, a
  residual sign map over `(ν_LP, ν_HP)` and the load swept down — no start solves, no cell has both shaft powers
  positive, and each sweep walks the root down to the LP shaft stopping and off its map. Real: the load takes the
  running point away. Two already said so ("with the generator off …"); the customer-bleed one now does too — the
  same machine re-solved without the aircraft's air, only on the "off the maps" refusal and only when the
  generator's sentence did not fire (`sandbox_controls::build`; test `an_off_map_endpoint_the_customer_bleed_removes_is_named`).
- **The rule:** `TwoSpoolCore::idle_floor` (default `false`). On, (a) rung 40's equilibrium refuses a root its
  first Newton found with no positive thrust per inlet air ("rung-40 equilibrium below idle …" — the page's
  below-idle words), and (b) every steady matcher's OUTER call refuses a point with no positive net thrust
  (`steady_unchoked::dispatch`; a choked point, in rung 103's lead words "rung-103: below idle", without
  "unchoked"). One comparison (`> 0`, NaN refused) on one quantity's sign (net thrust; per inlet air in the
  equilibrium, absolute in the matchers — the same sign). Rung 38's infallible matcher is not reached by a page and
  is untouched. Set by `Machine::for_pages` (Blades), `FixedEngine::for_pages` (Cooling) — each sets rung 103's
  switch and the floor together — and by one `make` in the Controls view for every machine it builds (the flown
  one, and each sibling a refusal, a hold or a calibration re-solves) plus its steady generator check. The
  stator's `at_setting` sibling carries it by hand, as it carries rung 103's.
- **Why refusing is safe:** at 1 022 below-idle points the full-speed start returned (bare, 5 % customer bleed and
  cooled engines; four maps; 34 flights incl. Mach 2.6), 48 starts over both shafts found no second root.
- **What moved on the page** — two crash maps against a worktree of `6e49d33`:
  - *Seed 36, 1 200 requests* (the HP-wall map: Mach 0.2–1.6, no governor, tables or limiters): 1 162 byte-equal;
    **24 runs now refused** below idle (−0.8 to −56 N per kg/s of inlet air) — 23 with the bleed lever, whose dumped
    air pays full ram drag; 14 refusals reworded (12 the floors sentence, 1 the customer-bleed sentence, 1 now refused
    at its start too); nothing refused began to run, no run moved, no panic.
  - *Rung 101's map, seed 99, 1 500 requests* (every knob: altitude −1…25 km, ±45 K, Mach 0.05–3.5, governors,
    tables, limiters): **178 runs now refused below idle** (126 at the start, 49 at the end, 3 a table row) — −2.9 to
    −346 N per kg/s of inlet air, median −74; 149 of them at Mach 2 or above (59 of the 213 requests at Mach 3–3.5),
    on every lever (none 59, stator 41, both 44, bleed 34). The other 151 that moved were refusals before and after
    (94 old below-idle ones in the new floors sentence; the rest now refused below idle, or at another end, first);
    nothing refused began to run; no panic. So **at high Mach the page refuses much more than
    before** — runs that started or ended far below idle.
- **The page's lowest start, re-measured** (10 K steps; the start BUILDS — the whole slam finishes higher, § 39): plain engine 480 K at the design flight, 580 K at Mach 1.6,
  690 K at Mach 2 (unchanged), **960 K at Mach 2.6** (was 800–820 K, below idle), 550–560 K on a standard sea-level
  day, 630 K at +40 K (was 610–620). The bleed lever adds 40–120 K, and at Mach 2.6 it is still below idle at
  1 100 K (−19.6 N per kg/s); a 5 % customer bleed adds 30–90 K; cooling 0–10 K. The stator lever is the plain
  engine's within 10 K. The below-idle words quote these.
- **A test premise corrected:** `low_start_repair.rs`' "the Controls page from just above idle" started the bleed
  lever and the customer bleed at 560 K — below idle with each (−26 N per kg/s with the lever on flat-LP); they start
  at 600 / 590 K now, their measured floors.
- **One shaft:** the Fly view shows no running point with negative thrust (438 points, three air temperatures,
  Mach 0.3–2.8, 500–1250 K; why not was not traced). Unchanged.
- **The gate:** `tests/idle_floor.rs` — off, the full-speed root below idle is returned, on, refused in words, and
  above idle bit for bit; a refused root is the only root (three cells × 48 starts); a steady matcher's choked
  below-idle point (customer bleed, 11 km, Mach 1.6, 510 K, −6.9 N) refused on, bit for bit above idle; the pages
  (Mach 2.6 820 K refused / 960 K runs; the bleed lever at sea level 560 refused / 600 runs; Blades' choked words).
  Red check (a temporary env flag disabling the floor, then removed): 3 of 4 red; the uniqueness test holds without.
- Probes archived: `docs/plans/scratch-archive/idle-floor/`.
- **Rungs 98/99 are built ON the Controls page** and anchored cells below idle (the Mach-3 slice from 1000 K): the
  first gate failed four of their tests. Their cells are NOT moved: the page carries a hidden reduce setting
  `idle_floor` (on; written to a reply only when off, as rung 100's), and `rung99.rs`' `r99()` and rung 98's G4 test
  turn it off; the two "refused in words" tests also assert what the live page says there now ("below idle" at the
  1000 K start). Specs 98/99 say so.
- **The census:** the whole gate — no Python-held key moved; after the hidden setting 2920 passed, 2 failed: the
  browser check's two "a knob move stops a running sweep" races (the sweep finished before the move), which passed
  in the first gate and in two solo re-runs of the browser check — load, not the floor.

## 39. The Controls view warns below idle (2026-10-20) — a page rule, not a slice, not a rung

A side agent's heads-up on § 38 (the Controls page refused much more at high speed — the bleed lever at Mach 2.6
nothing up to 1 100 K); asked in plain terms, the user chose: **the Controls view keeps a point below idle and warns;
Blades and Cooling keep refusing** (their shared scoring cannot take a negative thrust). A below-idle point is a real
state (the shafts hold a steady speed; the drag of the air swallowed is larger than the jet's push).
- **The rule, three-way:** `TwoSpoolCore::idle_rule: IdleRule` — `Old` (default: the old code, § 37), `Refuse` (§ 38,
  Blades/Cooling via `for_pages`), `Accept` (Controls): rung 40's equilibrium keeps a below-idle root from EITHER start
  (§ 37's low start kept it out — otherwise whether a point runs would depend on the start; both root sets are measured
  unique). The steady matchers read `Accept` as `Old`; the Controls generator check is back to its pre-§ 38 setup. The
  view's hidden setting `idle`: `warn` (default) / `refuse` / `old` (rungs 98/99's tests).
- **The warning:** a steady end below idle carries `below_idle` (thrust per kg/s of INLET air — the rule's own number;
  absent otherwise, so every other reply is unchanged) and the page writes a note above the table. Its displayed thrust
  never shows the other sign: 1 210 below-idle starts over six variants (bleed lever, customer share and amount,
  cooling, both), 0 disagree.
- **The tables:** design-flight tables now hold rows below idle where § 37 had none — 642 tables a governed run reads
  (Mach 1.6–3.3), 330 with such rows, every one still one-way (thrust, pressure ratio, fan speed all rising), so each
  reading has one thrust. Rung 98's reach moves: Mach 3 at sea level now runs (from 120–168 N per kg/s below idle).
  The browser-vs-native check's gap bar gains a relative arm (1e-12 of the value): that cell's ram part is −4 441 N per
  kg/s, where 2.0e-9 is rounding.
- **A stop the page could not name — FOUND (the side agent's second note):** the page's quoted "lowest start" floors
  (§§ 37–38) were measured by BUILDING the run (its two steady ends), not by running it. The slam's own fuel closure
  fails at low shaft speeds — 1 854 of the slams between the two floors stop, 1 691 at their first step — so the
  lowest start whose WHOLE default slam finishes is: design flight 530–570 K (builds from 400 K, above idle from
  480 K), Mach 1.6 570–590 K, Mach 2 640–670 K, Mach 2.6 780–800 K, standard sea level 510–520 K, +40 K 550–570 K
  (every variant within these). Such a stop said only "an internal check"; it is now its own kind, `low_speed`, with
  plain words — the closure found no burner state between the walls (lean and rich are named only within 2× of their
  walls; the crash map's sit at 0.71–1.05). **The march's low-speed closure itself is OPEN** — the next repair offered. **CORRECTED by § 40 (2026-10-20):** 1 553 of the 1 691 first-step stops were starts LEANER than the closure's lean wall (0.004), the other 138 roots within one scan step under it — a wall, not "no burner state"; the stop was named for want of a recorded point.
- **Runs past the old reach — CHECKED, kept (a side agent's third note):** a governed run the old code refused with
  "Make the tables at this flight" now runs, and the note read the −4 441 above as its belief's miss. It is not: it is
  the RAM part alone, and the nozzle part (+4 724) nearly cancels it — at that cell the belief misses by +283 N per kg/s
  at the below-idle start and −38 at the end. Over the governed runs at Mach 2–3.3 (three airs, four shapes, starts
  850/1000/1150 K; `scratch-archive/idle-warn/gap.rs`), the worst miss along a run past the old reach is 120–535 N per
  kg/s (median 160–255), against 100–215 (median 100–140) for runs that reached before at the same flights — larger, as
  the reading goes further from where the table was made, not a different kind; every such run finishes. The reach
  refusal fired when a table ROW could not be built — the run's throttle, translated to the design flight, had no
  steady point there (below idle); those rows now build. Do the runs read inside the rows (`outside.rs`, the advisor's
  check)? Under the page's default speed grid, yes as much as before: points past an end row lie within 3.5 % of the
  table's span in both groups. Under no speed correction with the pressure-ratio reading, 17 runs at Mach 3–3.3 read
  5–20 of their 126 points up to 0.25–1.23 spans past an end (before: at most 0.13), where the lookup carries the end
  segment on in a straight line (not a clamp; for the gross push exact, rung 96's G3) — and their worst miss, 121–320,
  is no larger than their neighbours' (the group's largest, 374, stays within 0.2 of a span). So no new refusal or note — the page already shows the belief, the gap and its
  two parts. (Offered to the user: a note when a reading leaves the table, if wanted.)
- **What moved** (both crash maps, against `6e49d33` — before § 38 — and `1f57583` — § 38):
  - against `6e49d33`, rung 101's map (seed 99, 1 500): 118 requests, every one a refusal before — 27 now run to the
    end with the warning, 63 run and stop early (23 at a fuel closure — now `low_speed` —, 5 cause-unknown on the
    lag route, 3 at the acceleration schedule's set point — still "other" —, the rest lean cuts toward a
    below-idle end and floor stops), the rest refused for another reason; no run moved; no panic. The HP-wall map
    (seed 36, 1 200): 24 runs differ only by the new `below_idle` key (they started below idle already); 7 refusals
    run, 6 run and stop, 1 reworded (the customer-bleed sentence, § 38).
  - against `1f57583`: the floor's refusals run again — rung 101's map 172 to the end, 89 stopped early; seed 36's
    24 to the end, 13 stopped.
- **The gate:** `tests/idle_floor.rs` — a full-speed root below idle (kept at `Old`/`Accept`, bit for bit; refused at
  `Refuse`); a low-start root below idle (§ 37's 470 K cell: refused at `Old`/`Refuse`, kept at `Accept` = the first
  low start that solves, bit for bit); uniqueness; the steady matcher (`Accept` is `Old`); the Controls page warns
  (Mach 2.6 820 K, the bleed lever at sea level 560 K, a low-start root at sea level 540 K — each marches to the end,
  its warning the rule's number; `refuse` refuses each). `browser.mjs`: the note appears below idle and is gone at
  960 K. `sandbox_controls.rs`: the `low_speed` stop driven (480 K at the design flight).
- Probes archived: `docs/plans/scratch-archive/idle-warn/`.

## 40. The fuel closures' lean wall follows the run (2026-10-20) — a solver repair, not a slice, not a rung

§ 39's open item, offered and chosen. **The cause (probes 1–2):** rung 43's fuel closure (and rungs 62/90/91's bled
twins) scans the LP-face flow up from the flow where the mixture hits `F_CAP` 0.065 to where it hits `F_FLOOR` 0.004.
That floor is a SEARCH bound — rung 43 (Python `engine.py:4625`) chose it to keep the scan off the region past the root
where the maps go non-physical; nothing anchors it to combustion. A steady start can burn leaner (1.5–4.3 g/kg at the
six probed flights near the lowest throttle that builds, ~0.1 g/kg at Mach 1.6–2), so the march's first closure could
not reach its own start: of the 1 691 first-step stops, 1 553 starts were leaner than the wall and 138 had their root
within one scan step (0.04) under it — every one accounted for. The 163 slams that stopped later all ended at the RICH
wall (last mixture 0.061–0.065).
- **The user's two choices** (asked in plain terms): relax the wall in the pages rather than refuse such starts; and
  tie it to the run — `TwoSpoolCore::lean_wall: Cell<Option<f64>>`, `None` everywhere by default (the old code, every
  Python-held oracle), set by the Controls view to **half the leaner steady end's mixture when that is under 0.004**.
  A run whose ends both burn at 0.008 or richer keeps 0.004, so a fast power cut from such ends still stops lean as
  before. **But a cut to an end leaner than 0.008 does move** (27 + 20 lean stops now run, 15 governed cuts go deeper — the crash maps below): more than the option's words to the user said ("power cuts still stop lean as today"); reported and asked — **the user kept it (2026-10-20)**: the old wall was a search bound, not combustion. Hidden reduce setting `lean: old`. The stator lever's siblings carry it (`at_setting`).
- **Why it is safe:** the scan takes the first sign change upward from the rich wall, so lowering the lean wall only
  extends it past the old one — a closure that bracketed before brackets on the same samples. Run, not argued:
  `tests/lean_wall.rs` holds a 560 K design-flight start (wall lowered to ~3 g/kg, old slam complete) bit for bit
  against `lean: old` on all five closure variants. **A caller that treated a failed lean trial as information does
  move**: in rung 101's crash map 18 governed runs that completed both times differ — 3 at rounding (≤ 3e-14), 15
  governed power cuts whose fuel had been resting AT the old wall (leanest mixture 0.0040–0.0046) and now cut as deep
  as the governor asks (to 1.2–3.7 g/kg). The old wall was silently shaping those runs.
- **What moved (probe 3, five variants × six flights × four maps, starts 400–1300 K):** the lowest start whose whole
  default slam finishes drops 80–190 K at every flight but the design flight, to within 0–20 K of the lowest that
  builds; at the design flight (20 cells) it does not move — the low starts now leave the start and stop RICH after
  11–21 steps (the default slam has no acceleration schedule; with it on, the plain engine's starts 400–560 K on all four maps run — probe 4; the other variants not measured —
  to the end). Of the 1 691 first-step stops one is left: the customer bleed at sea level +40 K from 440 K, whose LP
  shaft starts at 15 % speed (5 K higher, 38 %) — its mixture is above the run's wall, so it keeps `low_speed`, the
  kind now driven by it. Where the first step now solves it lands on the start's own state (mixture to ≤ 2e-15).
  **Rung 104 (2026-10-20):** that crawling start is rung 38's choked vane's; under the LP turbine's own flow law (the
  default) it runs the LP shaft at 37 %, and a search (four maps, three flights, every customer mode, 400–490 K) found no
  slam stopping at once for slow shafts — `low_speed` is now driven on the `pin` setting (`docs/rung104-spec.md` § 5).
- **The crash maps** (against `e4d9756`): rung 101's (seed 99, 1 500) — 129 replies differ: 27 lean stops, 18
  low-speed, 4 cause-unknown and 3 "other" now run to the end; 5 low-speed now reach a later floor-unreachable stop; 9
  floor-unreachable or lean stops come at another step; 41 first-step stops (39 floor-unreachable, 2 redline-floor)
  only gain a mixture read off the start (was null); 3 lean stops only reworded; 1 floor stop's re-run names another
  instant (its fuel search reaches leaner); the 18 governed runs above. No panic. Seed 36's (1 200):
  58 differ, all fast power cuts to a low end that stopped lean — 20 now run, 10 run further and stop lean at the run's
  wall, 28 only reworded; every changed march bit-equal to the old one for as long as the old one ran.
- **The words:** a stop with no recorded point now reads its mixture off the start (`run_mixture`), so § 39's case
  (480 K at the design flight) is named LEAN under `lean: old`; the lean stop's words and the page's help name the run's
  wall ("0.004, or half the leaner steady end's mixture when that is lower").
- **The gate:** `tests/lean_wall.rs` (the default slam sets no wall and is the old run; the lowered wall moves no run
  the old one solved, five closures; sea level 450 K leaves its lean start on every closure, its first point the start;
  the design flight's 480 K now meets the rich wall and the acceleration schedule takes it; the crawling-LP case is
  `low_speed`; the hidden setting round-trips). `sandbox_controls.rs`: `low_speed` driven by the crawling-LP case.
- Probes archived: `docs/plans/scratch-archive/low-speed/`.

## 41. The HP turbine's vanes on the pages — rung 105 (2026-10-20) — a setting, off by default

Rung 105's law (`docs/rung105-spec.md`) is a setting on the four off-design views — **Fly it** (and its slam),
**Controls**, **Size the blades** and **Cool the turbine** (the fixed engine only, beside rung 104's select): "HP turbine's
inlet vanes: choked, as rungs 31–104 assumed (the default) / vane and rotor in series, split by the stage's reaction",
and a reaction box (0.05–0.6, opening at 0.3) shown when the law is chosen. Request keys `hp_vane` (`pin` / `rows`) and
`reaction`, written to a reply only off the default, so every default reply keeps its bytes.

* **What moves.** On the pages' perfect-gas engines with the nozzle choked nothing (the law IS the pin there, bit for
  bit); Fly it's thermally perfect default moves by about a tenth of a percent; below choke the HP turbine expands a
  little further. A cooled engine (Controls with cooling, the Cooling view) moves a little even choked — the coolants'
  share moves the HP turbine's expansion.
* **Refused in words:** a reaction outside 0.05–0.6; Fly it's afterburner with the law (its lit turbine needs a solve not
  built).
* **Cost with the law on** (spec § COST): Fly it a point 5–15× the pin (≤ 60 ms), a slam 2–3 s; Controls 1.4× by default,
  3.5× below choke, ~60× with turbine cooling (under a second).

## 42. Fly it's speed search below the low-speed wall (2026-10-21) — a solver repair, not a slice, not a rung

Found beside rung 33's double root. On a hot day at a low throttle Fly it refused points that run: rung 34's instant
assumes the nozzle choked first, and below about 0.6 of design speed that nozzle cannot reach the ambient pressure, so
the speed search skipped the speeds the running point sits among. Cause, repair and numbers: `docs/rung34-spec.md`
§ The low-speed wall.
- **The user's choices** (asked in plain terms, each after its cost was measured): fix the step everywhere; then, shown
  that lifting it everywhere moves ~4 600 Python-held values in their last digits, **the old search first, the new step
  only where it refuses** — so no oracle and no page reply that ran moves (§ 14.1's precedent).
- **On the page:** at 45 °C sea level, Mach 0.3, the lowest running throttle drops from 728 to 675 K (thermally perfect
  gas; 737 to 680 K perfect). In 800 seeded requests 18 refusals now answer (560–745 K), all on the stable point: 17
  run to the end, one fast fuel-metered slam stops at the fuel cap; the other 28 are below idle (26 byte-identical, two
  reworded — below). **Corrected after the first commit (`358d7d2`, which said all 18 run):** three of them were
  throttle-up slams from a low start that found their start and then stopped at the first time step (an "ok" slam
  reply carrying a stop), because the time march kept the wall. Asked; the user chose to lift it on the page: a
  switch, `SpoolTransient::march_below_choke`, off by default (every Python-held trajectory, some of which end early
  on purpose) and set by the slam (`sandbox_transient.rs`). Measured: only those three slams change. With an afterburner fitted its own copy of the search is repaired the same way
  (725 K now runs unlit on that day; 700 K is below idle there — the duct's loss).
- **Fly it now refuses a point with no positive thrust below idle, in words** (the user's choice, asked after the
  measurement; § 38's rule, which had found "no running point with negative thrust" on this view — the wall was why).
  Such a point had reached the page as "The efficiency bookkeeping failed": the shared scoring's check cannot pass
  below zero thrust. In 3 000 wide random fly requests (300–900 K, Mach 0–2) 65 did before the repair, all negative
  thrust (Mach 0.5–2), and the repair added 20 that had read "spool down" (the shaft does settle there). All 85 now
  read below idle; 82 others now run; nothing else moved. With the afterburner too.
- **Still reachable:** the "No shaft speed gives a workable…" words (very low throttles, ~300–510 K).
- **The gate:** `tests/rung34_low_speed_wall.rs`; `tests/rung33_double_root.rs` now compares all its cells with rung 34;
  `tests/sandbox_fly.rs`' below-idle case.
  Probes archived: `docs/plans/scratch-archive/rung34-low-speed-wall/`.

## 43. The governor beside the slow temperature limiter — rung 106 (2026-10-21) — not a slice

Rung 106 (`docs/rung106-spec.md`) lifts rung 95's refusal of *Govern during the move* beside the temperature limiter's
*…that reacts with a lag* on the Controls view (`route: "governor_lag"`): the lag's cut becomes a state measured off the
governor's fuel. The top strip marks it as *a lag's cut*, and the new `min_fuel` holder (the minimum-fuel stop) where its
leftover would take the fuel below what the engine burns.
* **The user's choices:** rung 47's lag only (the realistic release and the slow valve stay refused beside the
  governor); every hold; refused beside rung 101's lagged exhaust sensor (another session); the lean edge HELD and
  marked; a start above the redline REFUSED in words; and (asked after the crash map) a governor's request above the
  redline at the start, where the metal cap holds the engine lower, refused too.
* **The lag's floor with the governor on is 0.1 τ** (`GOV_LAG_MIN`, the step check — rung 47's open route keeps one
  step), and beside the lag each speed law has its own edge, measured at three flights (the user's "limit each law"):
  HP-speed rate ≥ 0.1 τ, fan-speed rate ≥ 0.15 τ, HP-speed gain ≤ 10, fan-speed gain ≤ 7, scaled by the density as § 27's.
  Refused in words past them; switching the lag (or the governor, hold, law or reading) on moves an out-of-range knob to
  its edge and says so under the lag (the user's choice — two page defaults sat past the edges).
* **What moves:** nothing that ran before (the combination was refused); the request list's former "refused: a lag"
  entry now runs, with two new refusals beside it.
* **Cost:** a governed lagged run costs what rung 95's governed run with the instant limiter does (~0.3 s natively).
* **OPEN — found here, not caused here: governed Controls runs with the HP-shaft generator are 12–160× slower than when
  rung 95 shipped.** The crash map's five slowest requests take 21–37 s today and 0.2–1.8 s on rung 95's commit
  `096367d` (timed back to back, the lag off; `docs/rung106-spec.md` § 4). With rung 104's old pin they take 3.9–5.6 s,
  so most of it is rung 104's default LP-turbine law; the rest (~3–20×) untraced — a bisect is the next step. On the page
  such a run reads as a hang.
  **Traced (2026-10-22, profiled):** the time is the hold ends', not the march's. With the generator on, an end holding
  thrust or HP speed first finds the coldest burner at which the loaded engine still has a steady point (rung 94's
  floor: up from 600 K in 50 K steps, then halved to 1e-9 K), and below it every equilibrium fails — slowly: since the
  low-start repair (`c9a0306`) a failing one retries from six lower starts, each up to 80 Newton passes, and under rung
  104's law each pass costs ~10× the pin's. ~50 failing solves a request, ~82% of its time; both ends repeated the same
  scan. **First step shipped:** the view keeps every steady point it asks of its own engine by burner temperature, so
  each is solved once — exact (an equilibrium carries no history; 400 random requests, byte-identical replies,
  1 598 → 883 s). The five slow requests still take ~14 s each on the page (WebAssembly): **still OPEN**.
  **Not governed runs — held ones:** the same five requests ungoverned take as long (17–28 s before, 7–9 s after,
  natively); the cost is any Controls run holding thrust or HP speed with the generator on. **The user's choice
  (2026-10-22):** start each end's search at its own throttle (answers move only within the search's 1e-9 K tolerance;
  0.3–1.7 s on the page for three of the five), extended to the two cases that still fall back to the floor (the metal
  cap; no steady point at the set throttle); NOT dropping the low start's retries at the floor (a rule change, measured:
  2.7–3.8 s). The extension is written, unverified (`docs/plans/scratch-archive/gov-slow/README.md`, its patch).
