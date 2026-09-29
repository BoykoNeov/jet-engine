//! RUNG 82 — **THE THRESHOLD'S OWN LAW.** `ThresholdLawTransient` (`engine.py:22148`), slice AJ.
//!
//! Rung 81 scored its criterion as a LABEL predictor (99%) and never asked whether it predicts
//! the THRESHOLD — the `tau_f` at which the fuel region opens. Headline: **a threshold is a fixed
//! point, not a formula.** The criterion's inputs are read off a trajectory that `tau_f` itself
//! moves, so read FORWARD off a fixed reference march it is wrong by a multiple, and read as the
//! ROOT of its own residual it lands. And the swept knob is not the binding clock: every binding
//! point is in RELEASE, where rung 52's lag runs at `3·tau_f`.
//!
//! # WHAT STEP 1 ADDS HERE — **THIS FILE, AND NOTHING IN IT YET**
//!
//! Reader-only, as [`authority_clock`](crate::authority_clock) says for all four rungs: no `R82`
//! table, no builder — the machine is a rung-80 core. The nine methods (`_scan_cells`, `_hats`,
//! `_threshold_scan`, `_scan`, `_bisect`, `threshold_law`, `_threshold_row`,
//! `threshold_reference`, `threshold_terms`) are **step 3**.
//!
//! # WHAT THE PRE-FLIGHT FIXED FOR STEP 3
//!
//! * **`_scan` is named in THIS module and called fully qualified.** `stator.rs` already has a
//!   `scan` (rung 53's `VariableStatorMatcher._scan`, an unrelated hierarchy — a reused name, not
//!   a cell). No bare `use` of either.
//! * **`id(p)` → indices** in `_scan_cells` (`engine.py:22218`, `:22220`), through
//!   [`riding4_idx`](crate::shared_actuator::riding4_idx).
//! * **`min(hats, key=…)`** (`engine.py:22255`) returns the FIRST minimum in iteration order;
//!   **`round(c.tau_f/tau_f, 6)`** (`:22256`) is the crate's format-and-parse rounding, 0
//!   mismatches over the pre-flight's 229 990-case stress and every argument the suites produce.
//! * **`sorted(…, key=dist)`** (`engine.py:22495`) is STABLE → `sort_by`, never
//!   `sort_unstable_by`.
//! * **`float('nan')` is a returned VALUE** in `_threshold_row` (`engine.py:22414`); the oracle
//!   (step 6) must encode it explicitly.
//! * **`_bisect`'s V3 message** (`engine.py:22305`) is built with `%g` →
//!   [`py_g`](crate::demand_coordinate::py_g), added at step 1.
