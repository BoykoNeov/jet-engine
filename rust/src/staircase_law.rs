//! RUNG 84 — **THE MARCHED MINIMUM'S STAIRCASE.** `StaircaseLawTransient` (`engine.py:22809`),
//! slice AJ.
//!
//! Rung 83 asked which ramps have roots and at which `ds`. Headline: **a minimum over a MARCHED
//! set is a reading on a MOVING GRID BOUNDARY, so the residual carries the march's own
//! sawtooth.** A `min` over a fixed set of continuous functions can kink but not jump; rung 83's
//! jump is the four-loop window opening one march step earlier, and the entering point binds at
//! once. So `h(tau; ds) = h_true(tau) + hat'·delta(tau, ds)` with `delta` in `[0, ds)`, and a
//! missing root is a crossing that lands on a step. Rise and tread share the factor `ds`, so the
//! staircase number `Lambda = rise/tread` has no `ds` in it.
//!
//! # WHAT STEP 1 ADDS HERE — **THIS FILE, AND NOTHING IN IT YET**
//!
//! Reader-only, as [`authority_clock`](crate::authority_clock) says for all four rungs: no `R84`
//! table, no builder. Every march it runs is rung 82's, through `_scan_cells`. `edge_read`,
//! `classify`, `staircase_scan`, `lattice_count`, `staircase_number` and `root_class` are
//! **step 4**, with rung 83.
//!
//! # WHAT THE PRE-FLIGHT FIXED FOR STEP 4
//!
//! * **`classify` is named in THIS module and called fully qualified** — `fuel_transient` already
//!   has a `classify`. No bare `use` of either.
//! * **`round(s, 9)` IS A MERGE KEY** (`engine.py:22880`, `:22881`). Within one march it merges
//!   nothing (minimum spacing `1.25e-3`, which is what `:22879`'s comment claims); across the
//!   three `ds` marches the pre-flight counted 339 distinct values collapsing to 195 — an
//!   intended merge that the port must reproduce bit for bit, keyed on the rounded `f64`'s bits.
//! * **`min(summ, key=summ.get)`** (`engine.py:22883`) returns the FIRST minimum in `summ`'s
//!   INSERTION order, so the Rust map is a `Vec` of pairs — never a `HashMap` or `BTreeMap`.
//! * **`int(round(edge / ds))`** (`engine.py:22898`) is 1-argument `round`: ties-to-even,
//!   returning an int → `round_ties_even` and a cast. No exact `.5` was measured; the nearest
//!   value was `309.99999999999994`.
//! * **`round(c.tau_f/tau_f, 6)`** (`engine.py:22876`) as at rung 82.
//! * **Two `%g` messages** (`engine.py:23030`, `:23061`) → [`py_g`](crate::demand_coordinate::py_g),
//!   including `root_class`'s default `eps = 1e-7`, which Python prints `1e-07`.
//! * **`lattice_count`'s `V3: an edge off the march grid`** MISNAMES its cause at `r = 1.0` (no
//!   window, so no edge exists to be off the grid). Reproduced word for word, as rung 83's V4.
