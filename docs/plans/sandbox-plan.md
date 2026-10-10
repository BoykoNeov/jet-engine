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
behaviour; a flight knob on the two-shaft view — every two-spool solver keeps the DESIGN ambient
pressure as the nozzle's back-pressure (CLAUDE.md, OPEN), and `fly_solver`'s override (§ 10.8) is
single-spool.

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
  incidence floor share one slot; stator AND bleed with fuel limiters is OPEN.
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
  floor dichotomy is PUBLISHED from a run that stops on this solver check.
- **Measured** on 1 000 random Controls requests over the slider box, the same requests before and
  after: schedule-check stops 24 → 0; complete runs 564 → 587; real floor-unreachable stops
  64 → 68 (runs that went on and met the engine's own limit); one rich and two cause-unknown stops
  also ran on. The page's schedule-check stop kind is retired.
