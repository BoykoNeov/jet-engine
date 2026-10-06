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

## How it is built

| Piece | What it is |
|---|---|
| `rust/src/sandbox.rs` | The whole bridge, ordinary Rust: settings JSON in → `build_turbojet(…).run(…)` → full-precision JSON out. Tested natively (`tests/sandbox.rs`). |
| `rust/src/atmosphere.rs` | The 1976 standard atmosphere (`tests/atmosphere.rs`, held to the published table). |
| `rust/sandbox-wasm/` | A tiny separate crate: only the browser exports around `sandbox::call`, and the panic hook that keeps a crash's message readable. Not built by `cargo test`. |
| `template.html` | The page; `/*__SANDBOX_WASM_B64__*/` receives the build as base64. |
| `turbojet-sandbox.html` | The built page — committed, like the charts and cutaway pages. |

**Rebuild after any change that can reach the cycle (`rust/src/`) or the template:**

```
powershell -File rust\sandbox-wasm\build.ps1
```

then commit the page. The gate fails while it is stale.

## What binds the page to the model

- `tests/sandbox.rs` — the sandbox reproduces the CLI's design runs **bit for bit**; its T–s points
  reduce to `visuals::cycle_points` where that function is right; ideal components make no entropy
  on every gas (on the perfect gas, exactly the leftover its rounded rung-1 constants predict).
- `tests/sandbox_page.rs` — the joints: the page is its template + its build; every element the
  script looks up exists; every knob is a setting and back; every export and op the page uses exists.
- `rust/sandbox-wasm/check.ps1`, **in the gate** (`rust/test-all.ps1`) — rebuilds the page and
  compares it with the committed one; runs the page's build against the native model over a grid
  (`check.mjs`); and drives the real page in a headless Chrome (`browser.mjs`: Worker, panels,
  linked knobs, pin & compare). Needs Node ≥ 22 and Chrome.

**Browser vs native numbers.** Not bit-identical, by measurement: the browser uses Rust's bundled
maths library, Windows its own. On the perfect gas they agree to ~4e-16; on every table gas the
temperature solver (`gas::SOLVE_TOL` = 1e-11) stops at a different iterate inside its tolerance, so
they agree to ~2e-11. `check.mjs` holds them to 1e-13 / 1e-9 (entropy absolutely: 1e-10 / 1e-6 J/(kg·K)).
The user accepted this difference (plan § 9.4); bit-exactness stays the CLI's job.

## Next slices

Blade speeds (rung 85's knobs) and off-design (fixed hardware, moving throttle) — both planned,
either order (plan § 5).
