# Turbojet Cycle Simulator

A model of a turbojet engine built to be **understood**, not just run. It follows the air station by
station — inlet, compressor, burner, turbine, nozzle — and shows every number on the way: the gas
state at each station, the thrust, the efficiencies, and how the engine behaves when you change it.

The model grew in small steps ("rungs"), each adding one physical effect and checked against a
published case: from the textbook ideal cycle, through real components and real gas chemistry, to
off-design running, throttle slams, two shafts, fuel limiters, variable stators, bleed valves,
cooling air and an afterburner. There are 108 rungs today.

## Try it — no install

**Double-click `Start Jet Engine.cmd`** (or open `docs/sandbox/turbojet-sandbox.html` in a browser).

That page is the sandbox: one self-contained, offline file with the whole model compiled into it.
Change the engine and watch it respond. It has six views:

| View | What you do there |
|---|---|
| **Design the engine** | Pick the pressure ratio, turbine temperature, losses, gas model and flight condition; read the thrust, fuel use, the station table and the T–s diagram. |
| **Fly it** | Freeze that design's hardware, then move the throttle and the flight: the running line, shaft speed, stall margin; slam the throttle and watch the engine lag. |
| **Size the blades** | Two-shaft engine: blade speeds, stage counts, the speed limit, variable stators. |
| **Controls** | A throttle slam on the two-shaft engine with the fuel limiters, bleed valve, stators, generator load, cooling air and afterburner as switches. |
| **Burner** | The combustor's zones and mixing, and the nitrogen oxides they make. |
| **Cool the turbine** | Cooling air against metal temperature. |

Not sure where to start? [`docs/sandbox/EXPERIMENTS.md`](docs/sandbox/EXPERIMENTS.md) lists short
experiments — which knobs to move and what to watch for.

Two older pages sit beside it in `docs/visuals/`: an animated engine cutaway and a page of charts
(they cover the first rungs only).

## Read it

| If you want… | Open |
|---|---|
| The physics in plain language, station by station (rungs 1–22) | [`NOTES.md`](NOTES.md) |
| Rungs 25–108 in plain language, one file per family | [`docs/notes/`](docs/notes/README.md) |
| The formal equations of the base cycle | [`SPEC.md`](SPEC.md) |
| One line on what every rung adds, and the map of the code | [`CLAUDE.md`](CLAUDE.md) |
| A rung's full derivation, assumptions and checks | `docs/rungN-spec.md` |
| Things tried that did **not** work, and why | `docs/*-negative.md` |
| What the sandbox page does and how it is tested | [`docs/sandbox/README.md`](docs/sandbox/README.md) |
| Where the project is going | [`docs/plans/roadmap.md`](docs/plans/roadmap.md) |

## Run the model from the command line

The code is one Rust crate with **no dependencies** (`rust/`). You need a stable Rust toolchain.

```
cargo run --release --manifest-path rust/Cargo.toml          # the design-point tables and one panel per rung
cargo test --release --manifest-path rust/Cargo.toml --test rung31   # one rung's tests
powershell -File rust\test-all.ps1                            # every test, in parallel (about 11 minutes)
```

`python plot_ts_diagram.py` (needs matplotlib) draws the T–s chart from the file the first command
writes. It is the only Python left; it draws and does no physics.

After changing the model or the page template, rebuild the sandbox page with
`powershell -File rust\sandbox-wasm\build.ps1` and commit it — the test suite fails while it is stale.

## How the project works

- **Derive before coding.** Every station gets its governing equation and a one-line reason first.
- **Show the work.** Every run prints the full station table.
- **Conservation checks are assertions** that run on every execution, not separate tests.
- **Every rung reduces to the one before it**, exactly, and a test proves it.
- **A surprise is a finding.** Predictions are written down before the code; a miss is reported,
  not hidden.

## Licence

Boyko Non-Commercial License v1.0 — see [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).
