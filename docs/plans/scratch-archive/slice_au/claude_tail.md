## Layout
A compact map. The code is the Rust crate `rust/`. The Python was deleted at phase 8's last slice;
tag **`python-final`** keeps it, and every `engine.py:N` / `test_rungN.py` citation means that tag.
- `rust/src/gas.rs` — **the core.** `FlowState`; the dual-section `Gas` (cold/hot) with the CPG
  closed-form / TPG NASA-integral property interface, the gas factories (thermally perfect /
  reacting / Fork B / equilibrium) and the equilibrium Newton. `components.rs` — the five pure
  components (`Inlet … Nozzle`, rung 30's choke, rung 31's `choked_mfp`, the burner's `f = g(f)` /
  equilibrium solve). `engine.rs` — chains them, solves the shaft balance, scores performance.
- **Beside the cycle:** `nox.rs` (rungs 7–24 — NOx, zoning, the mixing closures, the nozzle
  bracket, the clamp ladder) and `march.rs` (25–30).
- **The matcher / transient ladders**, one module per family, then per rung: `matcher.rs` (31, 33)
  → `map.rs` (32) → `spool.rs` (34–36) → `combustor.rs` (37) → `two_spool.rs` (38, 39, 41) →
  `two_spool_transient.rs` (40, 44) → `bleed.rs` (42) → `fuel_transient.rs` (43, 45, and the
  fuel-side limiters 46–52) → `stator.rs` (53–54) → `stage.rs` (55–56) → `stator_transient.rs`
  (57–60) → `stator_bleed.rs` (61) → `bleed_transient.rs` (62–63) → `limited_bleed.rs` (64) →
  `lagged_bleed.rs` (65) → **exactly ONE module per rung, 66→84** (`two_lag.rs` …
  `staircase_law.rs`). A rung is a core plus a `const` table of function pointers, and reduces to
  its predecessor; **each module's header names its rung, and what it adds is in its spec** — so
  this entry never grows.
- `rust/src/main.rs` + `panels/` + `pyfmt.rs` — the CLI: the design-point tables and **one panel per
  rung**, held byte-equal to the last Python run's output by `tests/cli_golden.rs`. It writes
  `ts_diagram.json`; `plot_ts_diagram.py` (matplotlib, the one Python file — it draws, no physics)
  turns that into `ts_diagram.png`.
- `rust/tests/` — per-rung `rungN.rs`; the `*_oracle.rs` gates, Rust ≡ PyPy bit for bit against the
  Python's committed outputs in `rust/oracle/`; `fingerprint.rs`, the only **ABSOLUTE-value** gate
  (its CPython anchor is kept as the audit record); `phi_rate_limiter_negative.rs`, the only
  NEGATIVE with a gate; `coverage_ledger.tsv` — where every Python test went.
- `docs/visuals/` — two **BUILT** pages (charts, cutaway): `cargo run --release -- visuals` writes
  `data.json` and splices both (`-- splice` re-splices only). Cycle change ⇒ rebuild **and
  republish**; `tests/visuals.rs` gates the joints.
- `docs/rungN-spec.md` (contents: see the banner); `docs/plans/rungN-anchor-*.md` — that rung's
  verified anchor data. `docs/plans/` holds the plan/tasks.

## Commands
Every `cargo` command below runs in `rust/` (or add `--manifest-path rust/Cargo.toml`).
- Run the model: `cargo run --release` · then the chart: `python plot_ts_diagram.py` (matplotlib).
- **The gate: `cargo test --release`** — **EVERYTHING**, TESTS tests, **TIME** on a quiet box.
  Launch it at below-normal priority. ONE gate; nothing is ever deselected.
- **Iterate: `cargo test --release --test rungN`** — one binary. Run what a change can reach.
- **WHEN to run the gate:** at session end (unless run shortly before), and after a code change.
  NOT at session start, NOT on a docs-only change, NOT "just to be sure", and **NEVER to refresh
  a timing** — take that from a run already happening, or leave it stale.

## Stack
**Rust**, stable, **no dependencies** (by decision). Python survives only as `plot_ts_diagram.py`
(`requirements.txt`: matplotlib). The Python model, its pytest suite and the PyPy venv live at tag
`python-final`; the port's record is `docs/plans/todo-rust-port.md`.
