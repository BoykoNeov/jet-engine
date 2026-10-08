
### 8.8 SLICE AQ — RUNGS 64–84's PANELS, BYTE-EXACT (2026-10-03)

The last 21 panels, in three new files and three commits, each green at its own prefix:
`rust/src/panels/cascades.rs` (rungs 64–71: the bleed limiter, the lagged valve, the two-lag
cascade, cascade A, three loops, the reference, generic and full splits; `main.py` lines
4724–5414), `rust/src/panels/actuator.rs` (72–77: the shared actuator, the applied reference, the
demand coordinate, the anti-windup device, the sensed cap, the stiffness ledger; 5417–6128) and
`rust/src/panels/readers.rs` (78–84: the residual gauge, the state coordinate, the split wall and
the reader-only rungs 81–84; 6131–6855). **`PORTED = 85`: every one of `main()`'s `print_*`
panels, all 21 byte-exact on their first compile**, through byte 229 930 of the golden. Only the
`plot_ts_diagram` line (bytes 229 930–229 987) is left, and it is slice AR's. No shim change.

#### (i) Port decisions, each one the Python's own semantics

* **The march step is read off each reader's SIGNATURE, per call site** (slice AP's rule), and
  this slice has three values, not two. Rung 64's two readers take their `ds = 0.005` default;
  rungs 65–67 pass `ds=DS=0.005` over defaults of 0.005 (65) and **0.0025** (66/67, which the
  panel overrides); rung 67's `detector_sensitivity()` is called bare and takes **0.0025**; rungs
  68–71 pass no `ds`, so most readers take 0.005 but **rung 69's `reference_modes` and rung 71's
  `full_gains` take 0.002**; rungs 72–74's seven Jacobian readers (`shared_gains`, `shared_cells`,
  `mask_discriminator`, `applied_gains`, `applied_cells`, `ref_discriminator`, `demand_gains`)
  take **0.002** and their march readers 0.005; rungs 75–84 are 0.005 throughout (rungs 83/84
  spell it in their `kw`). The table was written before any code, from one grep of the `def`
  lines. Every reader name in it is defined once in `engine.py` except two: `authority_ceiling`
  (rung 54's and rung 64's) and `split_gains` (rung 70's and rung 80's). For those, the port calls
  each class's own, and the defaults were read off THAT definition.
* **The panels' machines are the shipped builders on rung 53's hardware** (`airflow.rs`'s
  `design13` / `lp_map` / `hp_map`, through one `machine(build, d, arm)` helper). Python's
  keyword limiters are `LeverArm` fields: `bleed_lim=BleedLimiter(PHI, B, tau=…)` is
  `BleedLimiter::with_tau(PHI, B, Some(…))`, `.from_margin(LP, B, SM, tau=…)` is
  `from_margin_tau`, and the `rig(phi_lim, inc)` of rungs 74–76 arms `stator_inc` or
  `stator_lim` by `inc`. Rungs 81–84 have no class of their own in the port; they read rung 80's
  `build_split_wall_cascade` machine, as their own tests do.
* **Private attributes set by plain assignment are the shared core's `Cell`s.** Rungs 77/78 set
  five (`_lag_coord`, `_ref_law`, `_windup_law`, `_tau_t`, `_cap_law`) and rungs 79–84 set four
  (no `_tau_t`). The port sets the same ones, so a default that matched by coincidence could not
  hide a missing assignment.
* **Python containers printed whole** go through `py_tuple` / `py_list` / `py_dict`:
  `fa['fracs']` (a float tuple), `str(arm['taus'])` (3- and 4-tuples), `str(a['zeros'])` and
  `str(d0['dzeros_B'])` (lists), rung 77's `order` (a tuple of strings, so the elements are
  quoted), rung 80's `authority` (Python's insertion-ordered count dict, `{'fuel': 1, 'gov': 3}`).
  **Python dicts keyed by a tuple or a formatted string** are Rust `Vec`s searched by key: rung
  72/73's `(inc, authority)` cells, rung 75's `"%s|%s" % (ref, tau_t)` (carried as fields in the
  port, compared exactly), rung 76's `sorted(live)` (bytewise sort of the ASCII keys) and rung
  81's `sorted(by_authority.items())`.
* **Bare floats in an f-string or `%s`** print their `repr` (`{TAU}` → `0.05`, `{PHI}` → `0.8`,
  `{g['worst_F_r']}` → `0.0`); **bools** print `True`/`False` through `py_str()`; an
  `Option<f64>` / `Option<bool>` under `%s` prints Python's `None` where the reader returns one
  (rung 82's `grows_above`, rung 84's `s_bind`/`edge`). Every other `Option` and `Result` is
  unwrapped with a message: Python would raise where the value is absent, so the golden proves it
  present.
* **Two calls kept where a refactor would merge them**: rung 79 calls `coord_forced` twice, each
  on a fresh `rig()`, and prints the first's `d_forced` before the second is made; the port does
  the same. Rung 74's `demand_law` is called once per `(phi, inc)` and filtered by `inc`, as the
  Python does.

#### (ii) Caught failures, read off stderr with `--nocapture`

**One new line, and it is expected.** Rung 74's `windup_law` marches the `demand × applied` cell,
which has no plant: its joint initial condition does not converge
(`rung-74: the joint initial condition did not converge (residual 2.898e-03 after 60
iterations)…`), the Rust march panics, and the reader catches it as an absent cell. That is
Python's `try/except`, and the panel prints `NO` for that row, as the golden does. Stderr
therefore now carries four lines: the three already booked (rung 33's two equilibrium-solve
failures, § 8.4 (i); rung 38's nozzle unchoke, § 8.5 (ii)) and this one.

#### (iii) What the gate saw, measured

One injection, the slice's own trap: rung 71's `full_gains` given the `ds = 0.005` its siblings
take instead of its own 0.002 default. Run through a scratch checker that runs only the
named panels and compares them byte-for-byte with their golden segments: **FAILED, 44 of 74 lines
moved**, the first at line 30, the first row of the gain table. The sample grid itself moves, so
the table loses three rows and every row after it shifts up. Every gain and the determinant's
factoring error (to `1.1e-03`) changed. Unlike slice AP's injection, the first line tripped is a
physics row, not a rounding-noise one. Reverted from a byte copy; `git diff` empty on the file.

#### (iv) Cost

`cli_golden.rs`: **147.2 s at 72 steps**. At 78 steps it took 248.3 s, but that run overlapped the
scratch checker's builds and runs, so the number is not a timing. At 85 steps the gate below
records the value. The new panels are the heaviest in `main.py`: rungs 69, 71, 72, 73, 74 and 84
each take about 20–43 s on their own in the scratch checker.

**The ship gate** — `cargo test --release --no-fail-fast` at below-normal, read to its end:
GATE_RESULT

**Next: slice AR** — the visuals (`data.json`, the cutaway JSON, both splices, the T–s data and
the slimmed matplotlib script), which also owes the golden's last line.
