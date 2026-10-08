//! SLICE AJ step 4, rung 84 — `edge_read`, `classify`, `staircase_scan`, `lattice_count`,
//! `staircase_number` and `root_class`, **every returned value bit for bit, in Python's own key
//! order.**
//!
//! # WHAT IS PINNED
//!
//! Twenty-two readings on `tests/test_rung84.py`'s rig, each against
//! `oracle/probe_slice_aj_step4.py`'s output (`oracle/slice_aj_step4_pypy.tsv`, PyPy, the repo
//! venv; rung 83's readings share the file and are gated in `slice_aj_corrector.rs`). The suite's
//! own calls where it has one — rung 83 § 3.2's and § 3.4's read pairs and their `classify`,
//! `test_p4`'s ladder, `test_p3`'s count at two steps, `test_p7`'s coarse `root_class` — and, for
//! branches the suite never takes:
//!
//! * `classify_rev` — the jump pair REVERSED: `left` non-empty, the sign change mirrored.
//! * `edge_v1` / `classify_v1` — the EMPTY window: no summands, every Option `None`, `kind = None`.
//! * `lattice_v3` — Python's misnamed `V3: an edge off the march grid` at `r = 1.0`.
//! * `sn_ok` / `sn_bare` / `sn_zero` — one edge move with a spacing, with none, and with `0.0`
//!   (Python's `if spacing` is TRUTHINESS: no tread, but `spacing = 0.0` reported).
//! * `sn_v5` at `0.0198123456` — `%g` prints `0.0198123`, `{}` would not; `sn_v2` at `r = 1.0`.
//! * `root_cross` — a CROSSING (`root_exists = True`, `d_membership = 0.0`) at the shipped step;
//!   `root_v1` — the bisection's V1 carried; `root_v6` — `eps = 0.123456789` (`%g`: `0.123457`).
//!
//! `summands`, Python's float-keyed dict, is flattened as a LIST of `[key, value]` pairs: the key
//! bits pinned, insertion order kept, and no float spelled into a path.
//!
//! # UNREACHED ON THIS RIG — recorded, not implied by a green run (plan § 5.34.4)
//!
//! `n_slope_excluded > 0` (so `n_scored` = the cell count here, and a port counting cells would
//! pass), a riding point at a trajectory END (so `edge` = the first scored cell here), a merged
//! `round(s, 9)` key within one march, an edge off the grid on an OPEN window, a non-monotone
//! edge, and a summand exactly `0.0`.

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::split_wall::build_split_wall_cascade;
use turbojet::staircase_law::{
    self, Classified, EdgeRead, LatticeCount, RootClass, StaircaseNumber, StaircaseScan, EPS,
    N_BISECT,
};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::threshold_law::{ScanKw, BRACKET};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE: &str = include_str!("../oracle/slice_aj_step4_pypy.tsv");

