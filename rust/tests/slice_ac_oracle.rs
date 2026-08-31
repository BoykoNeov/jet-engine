//! SLICE AC step 6 — **THE ORACLE for rungs 70 + 71**, against PyPy *and* CPython 3.14.
//!
//! `rung70.rs` and `rung71.rs` port the two suites' own 57 gates, and most of them are RELATIONS —
//! a reduce arm equal to an ancestor's march, `pair_CV` near 1, a determinant that factors, a
//! window that overlaps. Relations are agreement, not correctness: two marches of the same binary
//! can agree with each other and both be wrong. **This file is the other half**: every value
//! `oracle/dump_slice_ac.py` emits at the SUITES' OWN GRID, nothing coarsened, every float
//! compared as its IEEE-754 bit pattern.
//!
//! # THIS FILE EXISTS BECAUSE OF THREE MEASUREMENTS, AND IT IS BUILT AGAINST EACH
//!
//! **1. A CELL THAT BREAKS BY EMPTYING THE SAMPLE.** § 5.27 (ii) ran rung 68's `_triple_laws` in
//! rung 70's slot and `split_gains` **returned successfully with an EMPTY table** — `len(rows)`
//! 2 → 0, every aggregate `None`, no gain differing because there are no gains. A value-diff gate
//! of the shape every previous slice wrote compares two empty tables and passes. So every
//! sample-shaped reader here emits its **row count and skipped count as keys**, and every
//! `Option` carries a PRESENCE FLAG beside it — [`Cmp::opt`], which is the same shape slice AB
//! used and the reason it is not new machinery.
//!
//! **2. A CLOCK GRID THAT REORDERS.** Step 4's injection i02 permutes `split_modes`' arms; it
//! shifts 25 of 38 printed lines and is caught by NEITHER language's gates at rung 70 (step 5
//! measured the same blindness at rung 71 — an inherited family property). An oracle only sees it
//! if the keys are bound to the grid's own index in the grid's own order, so every arm is
//! `C/arm/<i>/…` / `K/arm/<i>/…` and every aggregate over arms is emitted BESIDE its per-arm
//! parts, never instead of them.
//!
//! **3. A JOINT WINDOW THAT WIDENS.** Step 4's injection took rung 70's joint window 61 → 341
//! points and `joint_fraction` 0.179 → exactly 1.0 with every ported gate green, because their
//! bars are one-sided lower bounds. [`Cmp::span`] emits every window's COUNT as a key of its own
//! and section F emits the fraction, which discharges that booking rather than deferring it
//! again.
//!
//! # SECTION N — THE DECLARED EXTRA GRID, AND WHY P6 COULD NOT BE SETTLED WITHOUT ONE
//!
//! § 5.27 (iv) registered `p = nz[0]*nz[1]` being real inside [`zeta_pair`] as a **gated
//! condition**, measured over the rung-70 READERS (18 of 18). Step 5 falsified it from the
//! shipped test suite: `test_rung71.py`'s damping gate drives the same function on a CONSTRUCTED
//! spectrum where `p = 4462 + 4947i`. **Re-reading P6 off the readers' grid would therefore
//! re-publish the measurement that was already wrong.** Section N carries what the readers
//! cannot:
//!
//! * `N/const` — the three constructed spectra of `test_rung71.py:549-561`, verbatim, through
//!   BOTH damping readers, with `p` and `s` emitted as keys. This is the arm where [`csqrt`]
//!   takes its complex branch and [`c_div`] is a genuine complex division — CPython's Smith
//!   algorithm, the operation § 5.27 (iv) priced at 13 of 18 against a schoolbook spelling.
//! * `N/pair` / `N/ring` — every `_zeta_pair` and `_zeta_ring` call sections A–M make,
//!   INTERCEPTED at the function boundary in Python and never reconstructed (slice Z's leading
//!   finding). The roots arrive here as [`Cmp::input_c`] — read from the golden, marked consumed,
//!   never compared against themselves — and the Rust replays the shipped reader on them with the
//!   plant taken out of the loop.
//!
//! `N/pair/ncalls` and `N/ring/ncalls` are checkable rather than decorative: `split_modes` and
//! `split_floor` are the only `_zeta_pair` callers and `full_modes` the only `_zeta_ring` caller,
//! so each count must equal the rows sections C, E and K emit between them. That equality is
//! asserted here from the RUST's own tallies.
//!
//! # THE CROSS-INTERPRETER EXEMPTION — a set of NAMES, measured from the diff, and **P8 IS
//! FALSIFIED IN BOTH DIRECTIONS**
//!
//! P8 read *"the CPython exemption is the names downstream of `_invariants`' `c1` plus the
//! `cross_identity` subtree that `rung67_control` pulls in"*. Measured off this dump:
//!
//! * **the `cross_identity` subtree contributes ZERO names.** Section B is `rung67_control`, the
//!   only reader that calls it, and **not one B key drifts.** § 5.27 (iv) measured that site
//!   diverging 1 of 1 under CPython — and the divergence does not survive into anything the
//!   reader returns. This is slice AB's P3 one slice on: *a named subtree that does not appear*.
//! * **and 119 of the 234 names are not reader-side at all — THE MARCH ITSELF DIVERGES.** No
//!   `sum()` is involved: `_triple_gains_at` has none, and the drifting keys include raw central
//!   differences (`gains/R_q`, `gains/V_q`) and raw marched values (`phi_at_stator_off`).
//!
//! **The third cause is MEASURED, not inferred** (`probe_ac_step6.py`, sixteen arms marched under
//! both interpreters and diffed point by point, bit for bit):
//!
//! | arm | points | differing `(index, key)` | first |
//! |---|---|---|---|
//! | `C/arm/1` (`split_modes`, `tau_gov` 0.005) | 851 | 3 096 | index 112, in **`v`** |
//! | `D/0/fast_valve` (`c1_clock_swap`, `tau_q` 0.02) | 341 | 1 927 | index 28, in **`v`** |
//! | `H/0/by_q/2` (`window_law`, `tau_q` 0.20) | 341 | 801 | index 30, in **`v`** |
//! | the OTHER THIRTEEN arms, controls included | 341/851 | **0** | — |
//!
//! Every one begins in the STATOR STATE and nowhere else, and at the first differing index the
//! whole previous point and every other state at that point are bit-identical — so the solve's
//! INPUTS agree and its OUTPUT is 10–11 ULPs apart. **That is a solve terminating differently,
//! not a formula rounding differently.** RK4 then carries it into every state, and the loops
//! being contracting, it decays back to bit-equality by the end of the ramp. Which arms it hits
//! is not monotone in any clock (`tau_q` 0.20 diverges, 0.50 does not), which is the bit-pattern
//! property slice AB (i) recorded for the `sum()` question, one level down.
//!
//! [[rust-port-slice-z-step4]] is a pre-registered exemption of TWO keys that measured EIGHT,
//! because it counted quantities where a dump emits names — so [`EXEMPT`] is read off the diff
//! and annotated afterwards, in that order, and carries **119 / 91 / 24** under the three causes.
//! **The set is checked in BOTH directions**: a key that STOPS drifting is as much a change as a
//! new one. **The port is held to PyPy**, where nothing is exempt — which is what makes cause 3
//! an audit-arm note rather than a port defect.
//!
//! # WHAT THIS ORACLE CANNOT SEE, NAMED HERE SO STEP 7 OWNS IT
//!
//! * **The five swaps' DISPATCH.** No value key can witness a function-pointer table — a cell
//!   that computes the same number a different way passes every key here. That is
//!   `slice_ac_dispatch.rs`'s subject, and § 5.27 (v) measured that four of the five break by
//!   PANIC, which no dump reaches.
//! * **The three `_rk4_floor*` guards — P5, and it is SETTLED HERE IN THE NEGATIVE.** They are
//!   raises, not values; § 5.27 (vi) measured that at `ds = 0.05` the rung-71 floor fires first
//!   and the rung-70 one is never reached. Nothing in this dump passes a `ds` that trips either,
//!   so **no key below can see any of the three** — stated rather than implied, because a silent
//!   absence reads like coverage. `rung70.rs` / `rung71.rs` gate them by their RUNG TAG.
//! * **`_gov_max`'s restore policy.** § 5.27 (vii) measured 256 sets with zero overwrites and
//!   per-instance depth 1, and every restore puts a VALUE back (the mirror of slice AB, where
//!   every restore was to `None`). `slice_ac_cells.rs` manufactures it.
//! * **The nine arming guards**, which are raises and therefore not values.
//!
//! Regenerate both:
//! ```text
//! .venv/Scripts/python.exe rust/oracle/dump_slice_ac.py > rust/oracle/slice_ac_pypy.tsv
//! C:/Python314/python.exe  rust/oracle/dump_slice_ac.py > rust/oracle/slice_ac_cpython.tsv
//! ```
//! **Through a POSIX shell, not PowerShell 5.1** — it writes a UTF-8 BOM that lands in front of
//! the `#` on line 1, so the header parses as data. [[windows-tooling-file-hazards]].

use std::collections::{BTreeMap, BTreeSet};

use turbojet::bleed_transient::LeverArm;
use turbojet::cross_split::{
    build_cross_split_cascade, c1_clock_swap, rung67_control, split_bill, split_floor,
    split_gains, split_modes, window_overlap, zeta_pair, SplitFloorLive, Span, StateBoundary,
};
use turbojet::engine::FlightCondition;
use turbojet::full_split::{
    band_containment, build_full_split_cascade, full_bill, full_gains, full_modes, ic_contraction,
    window_law, zeta_ring, WindowLawArm,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::{c_add, c_mul, StatorIncidenceLimiter, C64};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::{StatorLimiter, TripleGains};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_ac_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_ac_cpython.tsv");

/// The number of keys `dump_slice_ac.py` emits — **its own stderr tally, not a neighbouring
/// slice's and not an estimate**. `load` bars 95 % of it, so a golden truncated mid-write cannot
/// present as a pass. [[rust-port-guessed-census-bars]]: the first writing of this line said
/// `39_099`, which was a guess from slice AB's 15 957 and was wrong by 7×.
const GOLDEN_KEYS: usize = 5_351;

/// The key NAMES the CPython 3.14 arm is allowed to differ on — **MEASURED off this dump's own
/// diff, never predicted.** § 5.27's P8 names two subtrees: `_invariants`' `c1` (rung 69's,
/// inherited) and the `cross_identity` subtree `rung67_control` pulls in, which belongs to rung
/// 67. [[rust-port-slice-z-step4]] is a pre-registered exemption of TWO keys that measured EIGHT,
/// because it counted quantities where a dump emits names — so this list is read off the diff and
/// annotated afterwards, in that order.
const EXEMPT: [&str; 234] = [
    // --- CAUSE 3: THE MARCH ITSELF DIVERGES (119 names, exactly THREE arms) ---
    "C/arm/1/max_c0_rel",
    "C/arm/1/max_c1_err",
    "C/arm/1/row/0/root/2/im",
    "C/arm/1/row/1/root/2/im",
    "C/arm/1/row/2/c0",
    "C/arm/1/row/2/c0_rel",
    "C/arm/1/row/2/c1",
    "C/arm/1/row/2/c1_err",
    "C/arm/1/row/2/c1_pred",
    "C/arm/1/row/2/c1_rel",
    "C/arm/1/row/2/cyclic",
    "C/arm/1/row/2/pair_CV",
    "C/arm/1/row/2/pair_RC",
    "C/arm/1/row/2/root/0/abs",
    "C/arm/1/row/2/root/0/re",
    "C/arm/1/row/2/root/1/abs",
    "C/arm/1/row/2/root/1/re",
    "C/arm/1/row/2/root/2/abs",
    "C/arm/1/row/2/root/2/im",
    "C/arm/1/row/2/root/2/re",
    "C/arm/1/row/2/worst_zero",
    "C/arm/1/row/2/zeta",
    "C/arm/1/row/3/c0",
    "C/arm/1/row/3/c0_rel",
    "C/arm/1/row/3/c1",
    "C/arm/1/row/3/c1_err",
    "C/arm/1/row/3/c1_pred",
    "C/arm/1/row/3/c1_rel",
    "C/arm/1/row/3/cyclic",
    "C/arm/1/row/3/pair_CV",
    "C/arm/1/row/3/pair_RC",
    "C/arm/1/row/3/pair_RV",
    "C/arm/1/row/3/root/0/abs",
    "C/arm/1/row/3/root/0/re",
    "C/arm/1/row/3/root/1/abs",
    "C/arm/1/row/3/root/1/re",
    "C/arm/1/row/3/root/2/abs",
    "C/arm/1/row/3/root/2/im",
    "C/arm/1/row/3/root/2/re",
    "C/arm/1/row/3/worst_zero",
    "C/arm/1/row/3/zeta",
    "C/arm/1/row/4/c0",
    "C/arm/1/row/4/c0_rel",
    "C/arm/1/row/4/c1",
    "C/arm/1/row/4/c1_err",
    "C/arm/1/row/4/c1_pred",
    "C/arm/1/row/4/c1_rel",
    "C/arm/1/row/4/cyclic",
    "C/arm/1/row/4/pair_CV",
    "C/arm/1/row/4/pair_RC",
    "C/arm/1/row/4/pair_RV",
    "C/arm/1/row/4/root/0/abs",
    "C/arm/1/row/4/root/0/re",
    "C/arm/1/row/4/root/1/abs",
    "C/arm/1/row/4/root/1/re",
    "C/arm/1/row/4/root/2/abs",
    "C/arm/1/row/4/root/2/im",
    "C/arm/1/row/4/root/2/re",
    "C/arm/1/row/4/worst_zero",
    "C/arm/1/row/4/zeta",
    "C/arm/1/row/5/c0",
    "C/arm/1/row/5/c0_rel",
    "C/arm/1/row/5/c1",
    "C/arm/1/row/5/c1_err",
    "C/arm/1/row/5/c1_pred",
    "C/arm/1/row/5/c1_rel",
    "C/arm/1/row/5/cyclic",
    "C/arm/1/row/5/pair_CV",
    "C/arm/1/row/5/pair_RC",
    "C/arm/1/row/5/pair_RV",
    "C/arm/1/row/5/root/0/abs",
    "C/arm/1/row/5/root/0/re",
    "C/arm/1/row/5/root/1/abs",
    "C/arm/1/row/5/root/1/re",
    "C/arm/1/row/5/root/2/abs",
    "C/arm/1/row/5/root/2/im",
    "C/arm/1/row/5/root/2/re",
    "C/arm/1/row/5/worst_zero",
    "C/arm/1/row/5/zeta",
    "C/arm/1/row/6/c0",
    "C/arm/1/row/6/c0_rel",
    "C/arm/1/row/6/c1",
    "C/arm/1/row/6/c1_err",
    "C/arm/1/row/6/c1_pred",
    "C/arm/1/row/6/c1_rel",
    "C/arm/1/row/6/cyclic",
    "C/arm/1/row/6/pair_CV",
    "C/arm/1/row/6/pair_RC",
    "C/arm/1/row/6/pair_RV",
    "C/arm/1/row/6/root/0/abs",
    "C/arm/1/row/6/root/0/re",
    "C/arm/1/row/6/root/1/abs",
    "C/arm/1/row/6/root/1/re",
    "C/arm/1/row/6/root/2/abs",
    "C/arm/1/row/6/root/2/im",
    "C/arm/1/row/6/root/2/re",
    "C/arm/1/row/6/worst_zero",
    "C/arm/1/row/6/zeta",
    "C/arm/1/zeta_lo",
    "D/0/fast_valve/c1_marched",
    "D/0/fast_valve/gains/R_q",
    "D/0/fast_valve/gains/V_q",
    "D/0/fast_valve/gains/cyclic",
    "D/0/fast_valve/gains/pair_CV",
    "D/0/fast_valve/gains/pair_RC",
    "D/0/fast_valve/pair_RC",
    "D/0/held_gains/c1_fast_stator",
    "D/0/held_gains/c1_fast_valve",
    "D/0/held_gains/ratio",
    "D/0/k_null",
    "D/0/marched_ratio",
    "D/0/measured_delta",
    "D/0/null_delta",
    "D/0/one_scalar_null/c1_fast_stator",
    "D/0/one_scalar_null/c1_fast_valve",
    "D/0/one_scalar_null/ratio",
    "D/0/predicted_delta",
    "H/0/by_q/2/phi_at_stator_off",
    "H/0/by_q/2/v_at_stator_off",
    // --- cause 1: `_invariants`' compensated `sum()` (91 names) ---
    "C/arm/0/max_c1_err",
    "C/arm/0/min_c1_rel",
    "C/arm/0/row/0/c1",
    "C/arm/0/row/0/c1_err",
    "C/arm/0/row/0/c1_rel",
    "C/arm/0/row/2/c1",
    "C/arm/0/row/2/c1_err",
    "C/arm/0/row/2/c1_rel",
    "C/arm/0/row/2/root/0/abs",
    "C/arm/0/row/2/root/0/re",
    "C/arm/0/row/2/root/1/abs",
    "C/arm/0/row/2/root/1/re",
    "C/arm/0/row/2/root/2/abs",
    "C/arm/0/row/2/root/2/re",
    "C/arm/0/row/2/worst_zero",
    "C/arm/0/row/2/zeta",
    "C/arm/0/row/6/c1",
    "C/arm/0/row/6/c1_err",
    "C/arm/0/row/6/c1_rel",
    "C/arm/0/row/6/root/0/abs",
    "C/arm/0/row/6/root/0/re",
    "C/arm/0/row/6/root/1/abs",
    "C/arm/0/row/6/root/1/re",
    "C/arm/0/row/6/root/2/abs",
    "C/arm/0/row/6/root/2/re",
    "C/arm/0/row/6/worst_zero",
    "C/arm/0/row/6/zeta",
    "C/arm/0/zeta_lo",
    "C/arm/3/max_c1_err",
    "C/arm/3/row/3/c1",
    "C/arm/3/row/3/c1_err",
    "C/arm/3/row/3/c1_rel",
    "C/arm/3/row/3/root/0/abs",
    "C/arm/3/row/3/root/0/re",
    "C/arm/3/row/3/root/1/abs",
    "C/arm/3/row/3/root/1/re",
    "C/arm/3/row/3/root/2/abs",
    "C/arm/3/row/3/root/2/re",
    "C/arm/3/row/3/worst_zero",
    "E/row/3/mod",
    "E/row/3/zeta",
    "K/arm/1/row/1/c1",
    "K/arm/1/row/1/min_root",
    "K/arm/1/row/1/root/1/abs",
    "K/arm/1/row/1/root/1/im",
    "K/arm/1/row/1/root/2/abs",
    "K/arm/1/row/1/root/2/im",
    "K/arm/1/row/1/zeta",
    "K/arm/1/zeta_lo",
    "K/arm/2/row/3/c1",
    "K/arm/2/row/3/min_root",
    "K/arm/2/row/3/root/0/abs",
    "K/arm/2/row/3/root/0/re",
    "K/arm/2/row/3/root/1/im",
    "K/arm/2/row/3/root/2/im",
    "K/arm/3/row/1/c1",
    "K/arm/3/row/1/ds_lambda",
    "K/arm/3/row/1/max_root",
    "K/arm/3/row/1/min_root",
    "K/arm/3/row/1/mod_ratio",
    "K/arm/3/row/1/root/0/abs",
    "K/arm/3/row/1/root/0/re",
    "K/arm/3/row/1/root/1/abs",
    "K/arm/3/row/1/root/1/re",
    "K/arm/3/row/1/root/2/abs",
    "K/arm/3/row/1/root/2/re",
    "K/arm/4/row/3/c1",
    "K/arm/4/row/3/min_root",
    "K/arm/4/row/3/root/0/abs",
    "K/arm/4/row/3/root/0/re",
    "K/arm/4/row/3/root/1/im",
    "K/arm/4/row/3/root/2/im",
    "K/arm/4/row/4/c1",
    "K/arm/4/row/4/min_root",
    "K/arm/4/row/4/root/0/abs",
    "K/arm/4/row/4/root/0/re",
    "K/arm/4/row/4/root/1/im",
    "K/arm/4/row/4/root/2/im",
    "K/arm/5/row/2/c1",
    "K/arm/5/row/2/ds_lambda",
    "K/arm/5/row/2/max_root",
    "K/arm/5/row/2/min_root",
    "K/arm/5/row/2/mod_ratio",
    "K/arm/5/row/2/root/0/abs",
    "K/arm/5/row/2/root/0/re",
    "K/arm/5/row/2/root/1/abs",
    "K/arm/5/row/2/root/1/im",
    "K/arm/5/row/2/root/1/re",
    "K/arm/5/row/2/root/2/abs",
    "K/arm/5/row/2/root/2/im",
    "K/arm/5/row/2/root/2/re",
    // --- cause 2: CPython 3.14's signed zero, `0.0` vs `-0.0` (24 names) ---
    "C/arm/0/row/0/root/2/im",
    "C/arm/0/row/1/root/2/im",
    "C/arm/0/row/2/root/2/im",
    "C/arm/0/row/3/root/2/im",
    "C/arm/0/row/4/root/2/im",
    "C/arm/0/row/5/root/2/im",
    "C/arm/0/row/6/root/2/im",
    "C/arm/2/row/0/root/2/im",
    "C/arm/2/row/1/root/2/im",
    "C/arm/2/row/2/root/2/im",
    "C/arm/2/row/3/root/2/im",
    "C/arm/2/row/4/root/2/im",
    "C/arm/2/row/5/root/2/im",
    "C/arm/2/row/6/root/2/im",
    "C/arm/2/row/7/root/2/im",
    "C/arm/2/row/8/root/2/im",
    "C/arm/3/row/0/root/2/im",
    "C/arm/3/row/1/root/2/im",
    "C/arm/3/row/2/root/2/im",
    "C/arm/3/row/3/root/2/im",
    "C/arm/3/row/4/root/2/im",
    "C/arm/3/row/5/root/2/im",
    "K/arm/3/row/0/root/2/im",
    "K/arm/3/row/1/root/2/im",
];

fn load(text: &str) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, bits) = (it.next().expect("key"), it.next().expect("bits"));
        let v = bits.parse::<u64>().unwrap_or_else(|e| panic!(
            "slice-AC golden line is not `key<TAB>u64` ({e}): {line:?}. If the second field has \
             text appended, the dump was redirected with `2>&1` and its stderr interleaved. If \
             the FIRST line failed, the file has a UTF-8 BOM: it was redirected through \
             PowerShell, which writes one."));
        assert!(m.insert(k.to_string(), v).is_none(), "dup {k}");
    }
    // MEASURED off THIS dump's own emitted count — never inherited from a neighbouring slice.
    assert!(m.len() > GOLDEN_KEYS - GOLDEN_KEYS / 20,
            "the slice-AC golden did not parse ({} keys, expected about {GOLDEN_KEYS})", m.len());
    m
}

/// Accumulates every disagreement so ONE run reports them all, **and reports every golden key the
/// Rust never asked for** — a field missing from the port is invisible until that half fires.
struct Cmp {
    py: BTreeMap<String, u64>,
    seen: BTreeSet<String>,
    bad: Vec<String>,
    cpython: bool,
    exempted: BTreeSet<String>,
    drifts: Vec<(String, f64)>,
    flips: Vec<String>,
}

impl Cmp {
    fn new(py: BTreeMap<String, u64>, cpython: bool) -> Self {
        Cmp { py, seen: BTreeSet::new(), bad: Vec::new(), cpython,
              exempted: BTreeSet::new(), drifts: Vec::new(), flips: Vec::new() }
    }

    fn f(&mut self, key: &str, got: f64) {
        assert!(got.is_finite(), "{key} is not finite: {got}");
        self.raw(key, got.to_bits(), false);
    }

    fn d(&mut self, key: &str, got: usize) { self.raw(key, got as u64, true); }

    fn b(&mut self, key: &str, got: bool) { self.raw(key, got as u64, true); }

    /// The dump's `s(...)` — a string as its FNV-1a 64-bit hash. The off-regime arm names,
    /// `silenced`, the `ic_contraction` sweep orders and the two ledgers' cell names are the
    /// non-floats a rung-70/71 reading carries, and a REGIME is the one thing no float witnesses.
    fn s(&mut self, key: &str, got: &str) { self.raw(key, fnv1a(got), true); }

    /// **THE PRESENCE FLAG IS THE POINT, NOT THE VALUE.** Every `max(…, default=None)` in these
    /// readers returns `None` on an EMPTY sample, which is § 5.27 (ii)'s measured break shape;
    /// `_zeta_ring` returns `None` on a real spectrum for real. A sentinel float would conflate
    /// the two.
    fn opt(&mut self, key: &str, got: Option<f64>) {
        self.b(&format!("{key}?"), got.is_some());
        if let Some(x) = got { self.f(key, x); }
    }

    fn opt_d(&mut self, key: &str, got: Option<usize>) {
        self.b(&format!("{key}?"), got.is_some());
        if let Some(x) = got { self.d(key, x); }
    }

    fn opt_b(&mut self, key: &str, got: Option<bool>) {
        self.b(&format!("{key}?"), got.is_some());
        if let Some(x) = got { self.b(key, x); }
    }

    /// A COMPLEX root — `re`, `im` and `abs`. **`abs` is a KEY and not a convenience**: it is
    /// `hypot`, and `sorted(…, key=abs)` is what selects WHICH root is which in both damping
    /// readers, in `n_zero`, `worst_zero`, `min_root` and `max_root`.
    fn c(&mut self, key: &str, z: C64) {
        self.f(&format!("{key}/re"), z.re);
        self.f(&format!("{key}/im"), z.im);
        self.f(&format!("{key}/abs"), z.abs());
    }

    /// **A GOLDEN KEY READ AS AN INPUT AND NOT COMPARED.** Section N replays the shipped damping
    /// readers on the roots Python's interceptor captured; re-emitting those roots as assertions
    /// would compare a key with itself, which slice U step 4 recorded as a gate that cannot see
    /// its own value. Marking them `seen` keeps the missing-key half of [`Cmp::finish`] honest
    /// without pretending they were checked.
    fn input_f(&mut self, key: &str) -> f64 {
        assert!(self.seen.insert(key.to_string()), "the Rust read {key} twice");
        f64::from_bits(*self.py.get(key)
            .unwrap_or_else(|| panic!("{key}: NO GOLDEN — section N's input is missing")))
    }

    fn input_d(&mut self, key: &str) -> usize {
        assert!(self.seen.insert(key.to_string()), "the Rust read {key} twice");
        *self.py.get(key)
            .unwrap_or_else(|| panic!("{key}: NO GOLDEN — section N's input is missing")) as usize
    }

    /// A root read as an INPUT. `abs` is consumed too — the dump emits it, so leaving it unread
    /// would fire the missing-key half for a key that is genuinely an input.
    fn input_c(&mut self, key: &str) -> C64 {
        let z = C64 { re: self.input_f(&format!("{key}/re")),
                      im: self.input_f(&format!("{key}/im")) };
        let _ = self.input_f(&format!("{key}/abs"));
        z
    }

    fn raw(&mut self, key: &str, got: u64, discrete: bool) {
        assert!(self.seen.insert(key.to_string()), "the Rust emitted {key} twice");
        match self.py.get(key) {
            None => self.bad.push(format!("{key}: NO GOLDEN (the dump never emitted it)")),
            Some(&want) if want != got => {
                if self.cpython && EXEMPT.contains(&key) {
                    self.exempted.insert(key.to_string());
                } else if self.cpython && discrete {
                    self.flips.push(format!("{key}: rust {got} vs cpython {want}"));
                } else if self.cpython {
                    let (a, b) = (f64::from_bits(got), f64::from_bits(want));
                    let rel = if b == 0.0 { (a - b).abs() } else { ((a - b) / b).abs() };
                    self.drifts.push((format!("{key}: {a:.17e} vs {b:.17e}"), rel));
                } else if discrete {
                    self.bad.push(format!("{key}: rust {got} vs python {want}"));
                } else {
                    self.bad.push(format!(
                        "{key}: rust {:.17e} ({got:016x}) != py {:.17e} ({want:016x})",
                        f64::from_bits(got), f64::from_bits(want)));
                }
            }
            Some(_) => {}
        }
    }

    fn finish(&self, arm: &str) {
        let missed: Vec<&String> = self.py.keys().filter(|k| !self.seen.contains(*k)).collect();
        assert!(self.flips.is_empty(),
                "{} DISCRETE keys flipped between interpreters -- a flipped count, flag, regime \
                 label, cell name or off-regime arm name is a different physical reading, never a \
                 rounding:\n  {}",
                self.flips.len(),
                self.flips.iter().take(12).cloned().collect::<Vec<_>>().join("\n  "));
        let worst = self.drifts.iter().map(|(_, r)| *r).fold(0.0_f64, f64::max);
        assert!(self.drifts.is_empty(),
                "{} float keys drifted against CPython OUTSIDE the declared exemption (worst \
                 {worst:.3e}). The exemption is a NAMED LIST rooted in TWO `sum()` sites (§ 5.27 \
                 (iv)) -- read this file's header before widening it, and never replace it with a \
                 tolerance:\n  {}",
                self.drifts.len(),
                self.drifts.iter().take(12).map(|(l, r)| format!("{l} (rel {r:.3e})"))
                    .collect::<Vec<_>>().join("\n  "));
        if self.cpython {
            let want: BTreeSet<String> = EXEMPT.iter().map(|s| (*s).to_string()).collect();
            assert_eq!(self.exempted, want,
                       "the CPython exemption set MOVED. Expected exactly the names in `EXEMPT`; \
                        got {} names. A key that STOPPED drifting is a change too -- it would \
                        mean the port's fold, the dump or CPython's `sum()` moved.\n\
                        only-in-EXEMPT: {:?}\nonly-measured: {:?}",
                       self.exempted.len(),
                       want.difference(&self.exempted).take(20).collect::<Vec<_>>(),
                       self.exempted.difference(&want).take(20).collect::<Vec<_>>());
        }
        if self.bad.is_empty() && missed.is_empty() {
            println!("slice_ac_oracle ({arm}): {} values compared, {} exempt",
                     self.seen.len(), self.exempted.len());
            return;
        }
        panic!(
            "{} of {} compared keys differ:\n  {}\n\n{} golden keys the Rust never asked for (a \
             field missing from the port is invisible until this fires):\n  {:?}",
            self.bad.len(), self.seen.len(),
            self.bad.iter().take(20).cloned().collect::<Vec<_>>().join("\n  "),
            missed.len(), missed.iter().take(20).collect::<Vec<_>>());
    }
}

fn fnv1a(text: &str) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for byte in text.as_bytes() {
        h = (h ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

// ------------------------------------------------------------------------------------ the grid
//
// COPIED from `tests/test_rung70.py` and `tests/test_rung71.py`, which carry the same block
// character for character. Nothing here is chosen.
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const V_MAX: f64 = 0.20;
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TT4_MAX: f64 = 1200.0;

/// `split_modes`' own default — FOUR arms, written `(tau_q, tau_gov, tau_s)` and reported back as
/// `taus = (tau_g, tau_q, tau_s)`, the `(g, q, v)` order of the STATE VECTOR.
const CLOCKS70: [(f64, f64, f64); 4] =
    [(0.05, 0.05, 0.05), (0.05, 0.005, 0.05), (0.05, 0.5, 0.05), (0.02, 0.05, 0.10)];

/// `full_modes`' own default — SIX arms, same convention.
const CLOCKS71: [(f64, f64, f64); 6] = [
    (0.05, 0.05, 0.05), (0.05, 0.005, 0.05), (0.05, 0.5, 0.05),
    (0.005, 0.05, 0.05), (0.05, 0.05, 2.0), (0.10, 0.10, 0.05),
];

/// `split_floor`'s own NINE-point grid.
const FLOOR_GRID: [(f64, f64, f64); 9] = [
    (0.05, 0.05, 0.05), (0.05, 0.05, 0.025), (0.05, 0.05, 0.10),
    (0.10, 0.10, 0.05), (0.02, 0.20, 0.05), (0.20, 0.02, 0.05),
    (2.00, 0.05, 0.05), (0.05, 0.05, 2.00), (0.05, 2.00, 2.00),
];

const TAU_QS: [f64; 5] = [0.005, 0.05, 0.20, 0.50, 2.00];
const TAU_SS: [f64; 4] = [0.005, 0.05, 0.20, 0.50];
const IC_ORDERS: [&str; 6] = ["gqv", "gvq", "qgv", "qvg", "vgq", "vqg"];
const IC_FRACS: [f64; 4] = [0.0, 0.25, 0.6, 1.0];

/// The eight ledger cells IN PYTHON's ORDER. Both `split_bill` and `full_bill` build this list.
const CELLS8: [&str; 8] = ["bare", "G", "V", "S", "GV", "GS", "VS", "GVS"];

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn core(b: ScheduledStatorTransient) -> ScheduledStatorCore {
    match b {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this grid never disables LP"),
    }
}

fn valve() -> BleedLimiter { BleedLimiter::from_margin_tau(&lp(), B, SM, Some(TAU)) }

fn phi_stator() -> StatorLimiter { StatorLimiter::from_margin(&lp(), V_MAX, SM, Some(TAU_S)) }

fn inc() -> StatorIncidenceLimiter {
    StatorIncidenceLimiter::from_margin(&lp(), V_MAX, SM, Some(TAU_S))
}

/// THE rung-70 machine — the governor beside the valve and the `phi` stator. Python's `cross`
/// module fixture.
fn cross() -> ScheduledStatorCore {
    let arm = LeverArm { bleed_lim: Some(valve()), stator_lim: Some(phi_stator()),
                         ..Default::default() };
    core(build_cross_split_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, &arm))
}

/// THE rung-71 machine — the governor, the valve and the INCIDENCE stator. Python's `full`
/// module fixture.
fn full() -> ScheduledStatorCore {
    let arm = LeverArm { bleed_lim: Some(valve()), stator_inc: Some(inc()), ..Default::default() };
    core(build_full_split_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, &arm))
}

// -------------------------------------------------------------------------------- the emitters
//
// One per shape the dump has a helper for, so the two files can be diffed side by side.

fn put_off(c: &mut Cmp, p: &str, names: &[&'static str]) {
    c.d(&format!("{p}/n_off"), names.len());
    for (i, n) in names.iter().enumerate() {
        c.s(&format!("{p}/off/{i}"), n);
    }
}

/// One `_triple_gains_at` return. **The interior arm carries no `s`** — Python's interior dict
/// does not have one, only its off-regime early return does, so a key for it would assert
/// against a field that does not exist.
fn put_gains(c: &mut Cmp, p: &str, gg: &TripleGains) {
    c.b(&format!("{p}/interior"), gg.interior);
    put_off(c, p, &gg.off_regime);
    if !gg.interior {
        c.f(&format!("{p}/s"), gg.s);
        c.f(&format!("{p}/v_base"), gg.v_base);
        return;
    }
    for (n, v) in [("R_q", gg.r_q), ("R_v", gg.r_v), ("C_g", gg.c_g), ("C_v", gg.c_v),
                   ("V_g", gg.v_g), ("V_q", gg.v_q), ("v_base", gg.v_base),
                   ("cyclic", gg.cyclic), ("pair_RC", gg.pair_rc), ("pair_RV", gg.pair_rv),
                   ("pair_CV", gg.pair_cv)] {
        c.f(&format!("{p}/{n}"), v);
    }
}

/// A `(lo, hi, n)` window. **THE COUNT IS A KEY IN ITS OWN RIGHT** — see this file's header,
/// measurement 3.
fn put_span(c: &mut Cmp, p: &str, sp: &Span) {
    c.opt(&format!("{p}/lo"), sp.0);
    c.opt(&format!("{p}/hi"), sp.1);
    c.d(&format!("{p}/n"), sp.2);
}

fn put_boundary(c: &mut Cmp, p: &str, rows: &[StateBoundary]) {
    c.d(&format!("{p}/n"), rows.len());
    for (i, x) in rows.iter().enumerate() {
        c.f(&format!("{p}/{i}/s"), x.s);
        c.f(&format!("{p}/{i}/live/R_q"), x.live_r_q);
        c.f(&format!("{p}/{i}/live/R_v"), x.live_r_v);
        c.f(&format!("{p}/{i}/dead/R_q"), x.dead_r_q);
        c.f(&format!("{p}/{i}/dead/R_v"), x.dead_r_v);
    }
}

fn put_skipped(c: &mut Cmp, p: &str, rows: &[(f64, Vec<&'static str>)]) {
    c.d(&format!("{p}/n"), rows.len());
    for (i, (s, off)) in rows.iter().enumerate() {
        c.f(&format!("{p}/{i}/s"), *s);
        put_off(c, &format!("{p}/{i}"), off);
    }
}

fn put_flist(c: &mut Cmp, p: &str, xs: &[f64]) {
    c.d(&format!("{p}/n"), xs.len());
    for (i, x) in xs.iter().enumerate() {
        c.f(&format!("{p}/{i}"), *x);
    }
}

fn put_split_floor_live(c: &mut Cmp, p: &str, x: &SplitFloorLive) {
    for (n, v) in [("s", x.s), ("pair_RC", x.pair_rc), ("pair_RV", x.pair_rv), ("u", x.u),
                   ("w", x.w), ("quiet_share", x.quiet_share), ("a_over_loud", x.a_over_loud),
                   ("det2", x.det2), ("zeta_pred", x.zeta_pred), ("floor", x.floor),
                   ("mod", x.modulus), ("mod_pred", x.mod_pred), ("rate_sum", x.rate_sum)] {
        c.f(&format!("{p}/{n}"), v);
    }
    c.s(&format!("{p}/silenced"), x.silenced);
    c.opt(&format!("{p}/zeta"), x.zeta);
    c.b(&format!("{p}/complex_pair"), x.complex_pair);
}

fn put_window_arm(c: &mut Cmp, p: &str, a: &WindowLawArm) {
    for (i, t) in [a.taus.0, a.taus.1, a.taus.2].iter().enumerate() {
        c.f(&format!("{p}/taus/{i}"), *t);
    }
    c.d(&format!("{p}/n"), a.n);
    c.f(&format!("{p}/phi_lim"), a.phi_lim);
    c.opt(&format!("{p}/phi_at_stator_off"), a.phi_at_stator_off);
    c.opt(&format!("{p}/v_at_stator_off"), a.v_at_stator_off);
    put_span(c, &format!("{p}/gov"), &a.gov);
    put_span(c, &format!("{p}/valve"), &a.valve);
    put_span(c, &format!("{p}/stator"), &a.stator);
    put_span(c, &format!("{p}/joint"), &a.joint);
    c.d(&format!("{p}/n_interior"), a.n_interior);
    c.f(&format!("{p}/v_hi"), a.v_hi);
    c.f(&format!("{p}/min_phi"), a.min_phi);
    c.opt(&format!("{p}/stator_off"), a.stator_off);
    c.opt(&format!("{p}/phi_recovers_marched"), a.phi_recovers_marched);
}

// ------------------------------------------------------------------------------------ the arms

#[allow(clippy::too_many_lines)]
fn run(golden: &str, arm: &str, cpython: bool) {
    let mut c = Cmp::new(load(golden), cpython);
    let (fl, x70, x71) = (flight(), cross(), full());
    // The Rust's OWN tally of every damping-reader call sections A–M make, checked against the
    // dump's intercepted counts at the foot of this function.
    let (mut n_pair, mut n_ring) = (0usize, 0usize);

    // ===================================================================== A -- split_gains
    let a = split_gains(&x70, &fl, LO, HI, TT4_MAX, SM, R, SETTLE, DS, TAU, TAU_GOV, TAU_S,
                        V_MAX, 10);
    c.d("A/n_riding", a.n_riding);
    c.d("A/n_sampled", a.n_sampled);
    c.d("A/n_rows", a.rows.len());
    c.b("A/s_window?", a.s_window.is_some());
    if let Some((lo, hi)) = a.s_window {
        c.f("A/s_window/lo", lo);
        c.f("A/s_window/hi", hi);
    }
    for (i, row) in a.rows.iter().enumerate() {
        c.f(&format!("A/row/{i}/s"), row.s);
        put_gains(&mut c, &format!("A/row/{i}/gov"), &row.gov);
        put_gains(&mut c, &format!("A/row/{i}/fuel"), &row.fuel);
        c.f(&format!("A/row/{i}/pair_gap"), row.pair_gap);
        c.f(&format!("A/row/{i}/cyclic_is_RC"), row.cyclic_is_rc);
    }
    put_skipped(&mut c, "A/skipped", &a.skipped);
    put_boundary(&mut c, "A/boundary", &a.boundary);
    c.opt("A/worst_CV", a.worst_cv);
    c.opt("A/worst_RC_is_1", a.worst_rc_is_1);
    c.opt("A/worst_RV_is_1", a.worst_rv_is_1);
    c.opt("A/min_pair_gap", a.min_pair_gap);
    c.opt("A/max_pair_gap", a.max_pair_gap);
    c.opt("A/worst_cyclic_is_RC", a.worst_cyclic_is_rc);
    c.opt("A/worst_RC_fuel", a.worst_rc_fuel);
    c.opt("A/worse_pair", a.worse_pair);
    put_flist(&mut c, "A/pair_RC", &a.pair_rc);
    put_flist(&mut c, "A/pair_RV", &a.pair_rv);

    // ================================================================== B -- rung67_control
    let bc = rung67_control(&x70, &fl, LO, HI, TT4_MAX, SM, TAU, TAU_GOV, TAU_S, V_MAX, R,
                            SETTLE, DS, 10);
    c.d("B/n", bc.n);
    c.opt("B/P70_lo", bc.p70_lo);
    c.opt("B/P70_hi", bc.p70_hi);
    c.f("B/P67_lo", bc.p67_lo);
    c.f("B/P67_hi", bc.p67_hi);
    c.opt_b("B/both_negative", bc.both_negative);
    c.opt("B/ratio", bc.ratio);

    // ====================================================================== C -- split_modes
    let cm = split_modes(&x70, &fl, LO, HI, TT4_MAX, SM, &CLOCKS70, R, SETTLE, 0.002, V_MAX, 20);
    c.f("C/ds", cm.ds);
    c.d("C/n_clocks", cm.clocks.len());
    c.d("C/n_arms", cm.arms.len());
    for (i, cl) in cm.clocks.iter().enumerate() {
        for (j, t) in [cl.0, cl.1, cl.2].iter().enumerate() {
            c.f(&format!("C/clock/{i}/{j}"), *t);
        }
    }
    for (i, ar) in cm.arms.iter().enumerate() {
        let k = format!("C/arm/{i}");
        for (j, t) in [ar.taus.0, ar.taus.1, ar.taus.2].iter().enumerate() {
            c.f(&format!("{k}/taus/{j}"), *t);
        }
        c.f(&format!("{k}/rate_sum"), ar.rate_sum);
        c.d(&format!("{k}/n"), ar.n);
        c.d(&format!("{k}/n_sampled"), ar.n_sampled);
        c.d(&format!("{k}/skipped"), ar.skipped);
        c.d(&format!("{k}/n_rows"), ar.rows.len());
        c.d(&format!("{k}/n_zeros"), ar.zeros.len());
        for (j, z) in ar.zeros.iter().enumerate() {
            c.d(&format!("{k}/zeros/{j}"), *z);
        }
        c.opt(&format!("{k}/max_c0_rel"), ar.max_c0_rel);
        c.opt(&format!("{k}/min_c1_rel"), ar.min_c1_rel);
        c.opt(&format!("{k}/max_c1_err"), ar.max_c1_err);
        c.opt_b(&format!("{k}/any_complex"), ar.any_complex);
        c.opt(&format!("{k}/zeta_lo"), ar.zeta_range.0);
        c.opt(&format!("{k}/zeta_hi"), ar.zeta_range.1);
        for (j, x) in ar.rows.iter().enumerate() {
            let rk = format!("{k}/row/{j}");
            for (n, v) in [("s", x.s), ("c2", x.c2), ("c1", x.c1), ("c0", x.c0),
                           ("c1_pred", x.c1_pred), ("pair_RC", x.pair_rc),
                           ("pair_RV", x.pair_rv), ("pair_CV", x.pair_cv),
                           ("cyclic", x.cyclic), ("worst_zero", x.worst_zero),
                           ("c1_rel", x.c1_rel), ("c0_rel", x.c0_rel)] {
                c.f(&format!("{rk}/{n}"), v);
            }
            c.opt(&format!("{rk}/c1_err"), x.c1_err);
            c.opt(&format!("{rk}/zeta"), x.zeta);
            c.b(&format!("{rk}/complex_pair"), x.complex_pair);
            c.d(&format!("{rk}/n_zero"), x.n_zero);
            for (t, z) in x.roots.iter().enumerate() {
                c.c(&format!("{rk}/root/{t}"), *z);
            }
            n_pair += 1;
        }
    }

    // ==================================================================== D -- c1_clock_swap
    for (gi, (tau_g, fast, slow)) in [(0.05, 0.02, 0.10), (0.05, 0.05, 0.05)].iter().enumerate() {
        let sw = c1_clock_swap(&x70, &fl, LO, HI, TT4_MAX, SM, *tau_g, *fast, *slow, R, SETTLE,
                               DS, V_MAX);
        let k = format!("D/{gi}");
        for (n, ar) in [("fast_valve", &sw.fast_valve), ("fast_stator", &sw.fast_stator)] {
            let ak = format!("{k}/{n}");
            for (j, t) in [ar.taus.0, ar.taus.1, ar.taus.2].iter().enumerate() {
                c.f(&format!("{ak}/taus/{j}"), *t);
            }
            c.f(&format!("{ak}/s"), ar.s);
            c.f(&format!("{ak}/c1_marched"), ar.c1_marched);
            c.f(&format!("{ak}/pair_RC"), ar.pair_rc);
            c.f(&format!("{ak}/pair_RV"), ar.pair_rv);
            put_gains(&mut c, &format!("{ak}/gains"), &ar.gains);
        }
        for (n, pr) in [("held_gains", &sw.held_gains), ("one_scalar_null", &sw.one_scalar_null)] {
            c.f(&format!("{k}/{n}/c1_fast_valve"), pr.c1_fast_valve);
            c.f(&format!("{k}/{n}/c1_fast_stator"), pr.c1_fast_stator);
            c.f(&format!("{k}/{n}/ratio"), pr.ratio);
        }
        for (n, v) in [("k_null", sw.k_null), ("marched_ratio", sw.marched_ratio),
                       ("predicted_delta", sw.predicted_delta),
                       ("measured_delta", sw.measured_delta), ("null_delta", sw.null_delta)] {
            c.f(&format!("{k}/{n}"), v);
        }
    }

    // ====================================================================== E -- split_floor
    let sf = split_floor(&x70, &fl, LO, HI, TT4_MAX, SM, &FLOOR_GRID, R, SETTLE, DS, V_MAX);
    c.d("E/n_rows", sf.rows.len());
    c.d("E/n_live", sf.rows.iter().filter(|x| x.live.is_some()).count());
    c.b("E/holds", sf.holds);
    c.b("E/strict", sf.strict);
    c.b("E/any_complex", sf.any_complex);
    c.opt("E/floor_lo", sf.floor_range.0);
    c.opt("E/floor_hi", sf.floor_range.1);
    c.opt("E/worst_pred_err", sf.worst_pred_err);
    c.f("E/max_ds_lambda", sf.max_ds_lambda);
    c.opt("E/max_mod_ratio", sf.max_mod_ratio);
    for (i, row) in sf.rows.iter().enumerate() {
        let k = format!("E/row/{i}");
        for (j, t) in [row.taus.0, row.taus.1, row.taus.2].iter().enumerate() {
            c.f(&format!("{k}/taus/{j}"), *t);
        }
        c.d(&format!("{k}/n"), row.n);
        put_off(&mut c, &k, &row.off_regime);
        c.b(&format!("{k}/live"), row.live.is_some());
        if let Some(live) = &row.live {
            put_split_floor_live(&mut c, &k, live);
            n_pair += 1;
        }
    }
    c.b("E/tightest?", sf.tightest.is_some());
    if let Some(t) = &sf.tightest {
        for (n, v) in [("s", t.s), ("zeta_pred", t.zeta_pred), ("floor", t.floor),
                       ("mod", t.modulus), ("rate_sum", t.rate_sum)] {
            c.f(&format!("E/tightest/{n}"), v);
        }
        c.opt("E/tightest/zeta", t.zeta);
        c.s("E/tightest/silenced", t.silenced);
    }

    // =================================================================== F -- window_overlap
    let wo = window_overlap(&x70, &fl, LO, HI, TT4_MAX, SM, R, SETTLE, DS, TAU, TAU_GOV, TAU_S,
                            V_MAX);
    put_span(&mut c, "F/gov", &wo.gov);
    put_span(&mut c, "F/valve", &wo.valve);
    put_span(&mut c, "F/stator", &wo.stator);
    put_span(&mut c, "F/joint", &wo.joint);
    c.d("F/n", wo.n);
    c.b("F/overlaps", wo.overlaps);
    c.f("F/joint_fraction", wo.joint_fraction);

    // ======================================================================= G -- split_bill
    let sb = split_bill(&x70, &fl, LO, HI, TT4_MAX, SM, R, SETTLE, DS, TAU, TAU_GOV, TAU_S,
                        V_MAX);
    c.f("G/Tt4_max", sb.tt4_max);
    c.d("G/n_cells", sb.cells.len());
    assert_eq!(sb.cells.len(), CELLS8.len(), "Python's ledger has eight cells");
    for (i, name) in CELLS8.iter().enumerate() {
        let cell = sb.cell(name);
        let k = format!("G/cell/{i}");
        c.s(&format!("{k}/name"), name);
        for (n, v) in [("I", cell.i), ("E", cell.e), ("min_phi", cell.min_phi),
                       ("max_Tt4", cell.max_tt4)] {
            c.f(&format!("{k}/{n}"), v);
        }
        c.d(&format!("{k}/n"), cell.n);
        c.opt(&format!("{k}/credit_phi"), cell.credit_phi);
        c.opt(&format!("{k}/credit_Tt4"), cell.credit_tt4);
    }
    for (n, m) in [("marginal_phi", &sb.marginal_phi), ("marginal_Tt4", &sb.marginal_tt4)] {
        c.f(&format!("G/{n}/gov"), m.gov);
        c.f(&format!("G/{n}/valve"), m.valve);
        c.f(&format!("G/{n}/stator"), m.stator);
    }
    c.opt("G/delivered_phi", sb.delivered_phi);
    c.opt("G/delivered_Tt4", sb.delivered_tt4);

    // ======================================================================== H -- window_law
    for gi in 0..2 {
        let (tqs, tss): (&[f64], &[f64]) = if gi == 0 {
            (&TAU_QS, &TAU_SS)
        } else {
            (&[TAU], &[TAU_S])
        };
        let wl = window_law(&x71, &fl, LO, HI, TT4_MAX, SM, tqs, tss, R, SETTLE, DS, TAU,
                            TAU_GOV, TAU_S, V_MAX);
        let k = format!("H/{gi}");
        put_window_arm(&mut c, &format!("{k}/base"), &wl.base);
        c.d(&format!("{k}/n_tau_qs"), wl.tau_qs.len());
        c.d(&format!("{k}/n_tau_ss"), wl.tau_ss.len());
        for (j, t) in wl.tau_qs.iter().enumerate() {
            c.f(&format!("{k}/tau_q/{j}"), *t);
        }
        for (j, t) in wl.tau_ss.iter().enumerate() {
            c.f(&format!("{k}/tau_s/{j}"), *t);
        }
        for (j, ar) in wl.by_tau_q.iter().enumerate() {
            put_window_arm(&mut c, &format!("{k}/by_q/{j}"), ar);
        }
        for (j, ar) in wl.by_tau_s.iter().enumerate() {
            put_window_arm(&mut c, &format!("{k}/by_s/{j}"), ar);
        }
        for (n, xs) in [("edge_q", &wl.edge_q), ("edge_s", &wl.edge_s)] {
            c.d(&format!("{k}/{n}/n"), xs.len());
            for (j, x) in xs.iter().enumerate() {
                c.opt(&format!("{k}/{n}/{j}"), *x);
            }
        }
        c.b(&format!("{k}/q_monotone"), wl.q_monotone);
        c.opt(&format!("{k}/q_span"), wl.q_span);
        c.opt(&format!("{k}/s_span"), wl.s_span);
        c.f(&format!("{k}/joint_fraction"), wl.joint_fraction);
        c.opt(&format!("{k}/phi_short_at_off"), wl.phi_short_at_off);
        c.opt(&format!("{k}/v_at_off"), wl.v_at_off);
    }

    // ================================================================== I -- band_containment
    let bc2 = band_containment(&x71, &fl, LO, HI, TT4_MAX, SM, R, SETTLE, DS, TAU, TAU_GOV,
                               TAU_S, V_MAX);
    c.d("I/n", bc2.n);
    c.d("I/n_delivering", bc2.n_delivering);
    c.d("I/riding_while_delivering", bc2.riding_while_delivering);
    c.d("I/n_riding", bc2.n_riding);
    c.opt("I/min_slack_delivering", bc2.min_slack_delivering);
    c.opt("I/worst_slack_minus_v", bc2.worst_slack_minus_v);
    c.f("I/min_slack_all", bc2.min_slack_all);

    // ======================================================================== J -- full_gains
    let fg = full_gains(&x71, &fl, LO, HI, TT4_MAX, SM, R, SETTLE, 0.002, TAU, TAU_GOV, TAU_S,
                        V_MAX, 2);
    c.f("J/ds", fg.ds);
    c.d("J/n_riding", fg.n_riding);
    c.d("J/n_sampled", fg.n_sampled);
    c.d("J/n_rows", fg.rows.len());
    c.b("J/s_window?", fg.s_window.is_some());
    if let Some((lo, hi)) = fg.s_window {
        c.f("J/s_window/lo", lo);
        c.f("J/s_window/hi", hi);
    }
    for (i, row) in fg.rows.iter().enumerate() {
        let k = format!("J/row/{i}");
        c.f(&format!("{k}/s"), row.s);
        put_gains(&mut c, &format!("{k}/gains"), &row.gains);
        put_gains(&mut c, &format!("{k}/phi_rig"), &row.phi_rig);
        for (n, v) in [("x", row.x), ("y", row.y), ("det", row.det), ("det_pred", row.det_pred),
                       ("y_is_RV", row.y_is_rv), ("x_is_product", row.x_is_product),
                       ("det_err", row.det_err)] {
            c.f(&format!("{k}/{n}"), v);
        }
        c.opt(&format!("{k}/cross_rung"), row.cross_rung);
    }
    put_skipped(&mut c, "J/skipped", &fg.skipped);
    put_boundary(&mut c, "J/boundary", &fg.boundary);
    c.opt("J/closest_to_1", fg.closest_to_1);
    c.opt("J/worst_y_is_RV", fg.worst_y_is_rv);
    c.opt("J/worst_x_is_product", fg.worst_x_is_product);
    c.opt("J/worst_det_err", fg.worst_det_err);
    c.opt("J/det_scale", fg.det_scale);
    c.opt("J/worst_cross_rung", fg.worst_cross_rung);
    put_flist(&mut c, "J/pair_RC", &fg.pair_rc);
    put_flist(&mut c, "J/pair_RV", &fg.pair_rv);
    put_flist(&mut c, "J/pair_CV", &fg.pair_cv);

    // ======================================================================== K -- full_modes
    let fm = full_modes(&x71, &fl, LO, HI, TT4_MAX, SM, &CLOCKS71, R, SETTLE, 0.002, V_MAX, 4);
    c.f("K/ds", fm.ds);
    c.d("K/n_clocks", fm.clocks.len());
    c.d("K/n_arms", fm.arms.len());
    for (i, cl) in fm.clocks.iter().enumerate() {
        for (j, t) in [cl.0, cl.1, cl.2].iter().enumerate() {
            c.f(&format!("K/clock/{i}/{j}"), *t);
        }
    }
    c.d("K/n_zeros_everywhere", fm.zeros_everywhere.len());
    for (j, z) in fm.zeros_everywhere.iter().enumerate() {
        c.d(&format!("K/zeros_everywhere/{j}"), *z);
    }
    c.d("K/arms_with_ring", fm.arms_with_ring);
    c.d("K/arms_real", fm.arms_real);
    c.d("K/arms_below_r69", fm.arms_below_r69);
    c.opt("K/max_c0_err", fm.max_c0_err);
    c.opt("K/min_routh", fm.min_routh);
    c.opt("K/max_mod_ratio", fm.max_mod_ratio);
    c.b("K/all_stable", fm.all_stable);
    for (i, ar) in fm.arms.iter().enumerate() {
        let k = format!("K/arm/{i}");
        for (j, t) in [ar.taus.0, ar.taus.1, ar.taus.2].iter().enumerate() {
            c.f(&format!("{k}/taus/{j}"), *t);
        }
        c.f(&format!("{k}/rate_sum"), ar.rate_sum);
        c.d(&format!("{k}/n"), ar.n);
        c.d(&format!("{k}/n_sampled"), ar.n_sampled);
        c.d(&format!("{k}/skipped"), ar.skipped);
        c.d(&format!("{k}/n_rows"), ar.rows.len());
        c.d(&format!("{k}/n_zeros"), ar.zeros.len());
        for (j, z) in ar.zeros.iter().enumerate() {
            c.d(&format!("{k}/zeros/{j}"), *z);
        }
        c.opt(&format!("{k}/min_root_rel"), ar.min_root_rel);
        c.opt(&format!("{k}/max_c0_err"), ar.max_c0_err);
        c.opt(&format!("{k}/min_routh"), ar.min_routh);
        c.opt(&format!("{k}/max_mod_ratio"), ar.max_mod_ratio);
        c.opt_b(&format!("{k}/all_stable"), ar.all_stable);
        c.opt_b(&format!("{k}/any_complex"), ar.any_complex);
        c.opt_b(&format!("{k}/any_below_r69"), ar.any_below_r69);
        c.opt(&format!("{k}/zeta_lo"), ar.zeta_range.0);
        c.opt(&format!("{k}/zeta_hi"), ar.zeta_range.1);
        for (j, x) in ar.rows.iter().enumerate() {
            let rk = format!("{k}/row/{j}");
            for (n, v) in [("s", x.s), ("c2", x.c2), ("c1", x.c1), ("c0", x.c0),
                           ("c0_pred", x.c0_pred), ("u", x.u), ("w", x.w), ("z", x.z),
                           ("routh", x.routh), ("pair_RC", x.pair_rc), ("pair_RV", x.pair_rv),
                           ("pair_CV", x.pair_cv), ("min_root", x.min_root),
                           ("max_root", x.max_root), ("ds_lambda", x.ds_lambda),
                           ("mod_ratio", x.mod_ratio)] {
                c.f(&format!("{rk}/{n}"), v);
            }
            c.opt(&format!("{rk}/c0_err"), x.c0_err);
            c.opt(&format!("{rk}/zeta"), x.zeta);
            c.opt(&format!("{rk}/r69_floor"), x.r69_floor);
            c.b(&format!("{rk}/below_r69"), x.below_r69);
            c.b(&format!("{rk}/complex_pair"), x.complex_pair);
            c.b(&format!("{rk}/stable"), x.stable);
            c.d(&format!("{rk}/n_zero"), x.n_zero);
            for (t, z) in x.roots.iter().enumerate() {
                c.c(&format!("{rk}/root/{t}"), *z);
            }
            n_ring += 1;
        }
    }

    // ===================================================================== L -- ic_contraction
    let ic = ic_contraction(&x71, &fl, LO, HI, TT4_MAX, SM, &IC_ORDERS, &IC_FRACS, R, SETTLE, DS,
                            TAU, TAU_GOV, TAU_S, V_MAX);
    for (n, rig) in [("full", &ic.full), ("shared", &ic.shared)] {
        let k = format!("L/{n}");
        c.d(&format!("{k}/n"), rig.n);
        c.d(&format!("{k}/n_rows"), rig.rows.len());
        c.d(&format!("{k}/n_converged"), rig.n_converged);
        c.d(&format!("{k}/members"), rig.members);
        c.b(&format!("{k}/spread?"), rig.spread.is_some());
        if let Some((g, q, v)) = rig.spread {
            c.f(&format!("{k}/spread/g"), g);
            c.f(&format!("{k}/spread/q"), q);
            c.f(&format!("{k}/spread/v"), v);
        }
        for (j, v) in [rig.marched.0, rig.marched.1, rig.marched.2].iter().enumerate() {
            c.f(&format!("{k}/marched/{j}"), *v);
        }
        c.opt_d(&format!("{k}/max_iters"), rig.max_iters);
        for (j, x) in rig.rows.iter().enumerate() {
            let rk = format!("{k}/row/{j}");
            c.s(&format!("{rk}/order"), x.order);
            for (t, v) in [x.start.0, x.start.1, x.start.2].iter().enumerate() {
                c.f(&format!("{rk}/start/{t}"), *v);
            }
            for (nm, v) in [("band", x.band), ("g", x.g), ("q", x.q), ("v", x.v),
                            ("res", x.res)] {
                c.f(&format!("{rk}/{nm}"), v);
            }
            c.d(&format!("{rk}/iters"), x.iters);
        }
    }

    // ========================================================================= M -- full_bill
    let fb = full_bill(&x71, &fl, LO, HI, TT4_MAX, SM, R, SETTLE, DS, TAU, TAU_GOV, TAU_S,
                       V_MAX);
    c.f("M/Tt4_max", fb.tt4_max);
    c.d("M/n_cells", fb.cells.len());
    assert_eq!(fb.cells.len(), CELLS8.len(), "Python's ledger has eight cells");
    for (i, name) in CELLS8.iter().enumerate() {
        let cell = &fb.cells.iter().find(|(n, _)| n == name)
            .unwrap_or_else(|| panic!("full_bill has no cell {name}")).1;
        let k = format!("M/cell/{i}");
        c.s(&format!("{k}/name"), name);
        for (n, v) in [("I", cell.i), ("E", cell.e), ("M", cell.m), ("min_phi", cell.min_phi),
                       ("max_Tt4", cell.max_tt4), ("v_hi", cell.v_hi)] {
            c.f(&format!("{k}/{n}"), v);
        }
        c.d(&format!("{k}/n"), cell.n);
        c.opt(&format!("{k}/credit_phi"), cell.credit_phi);
        c.opt(&format!("{k}/credit_Tt4"), cell.credit_tt4);
        c.opt(&format!("{k}/credit_inc"), cell.credit_inc);
    }
    for (i, name) in CELLS8[1..].iter().enumerate() {
        let k = format!("M/degrades/{i}");
        c.s(&format!("{k}/cell"), name);
        let got = &fb.degrades.iter().find(|(n, _)| n == name)
            .unwrap_or_else(|| panic!("full_bill has no degrades row {name}")).1;
        c.d(&format!("{k}/n"), got.len());
        for (j, g) in got.iter().enumerate() {
            c.s(&format!("{k}/{j}"), g);
        }
    }
    c.opt("M/inc_credit_valve_alone", fb.inc_credit_valve_alone);
    c.opt("M/inc_credit_stator_alone", fb.inc_credit_stator_alone);
    for (n, t) in [("marginal", &fb.marginal), ("alone", &fb.alone),
                   ("marginal_phi", &fb.marginal_phi), ("marginal_Tt4", &fb.marginal_tt4),
                   ("marginal_inc", &fb.marginal_inc)] {
        c.f(&format!("M/{n}/gov"), t.gov);
        c.f(&format!("M/{n}/valve"), t.valve);
        c.f(&format!("M/{n}/stator"), t.stator);
    }
    c.opt("M/kept/gov", fb.kept.gov);
    c.opt("M/kept/valve", fb.kept.valve);
    c.opt("M/kept/stator", fb.kept.stator);
    c.opt("M/delivered/phi", fb.delivered_phi);
    c.opt("M/delivered/Tt4", fb.delivered_tt4);
    c.opt("M/delivered/inc", fb.delivered_inc);

    // ============================================== N -- THE DECLARED EXTRA GRID (see header)
    //
    // The three CONSTRUCTED spectra of `test_rung71.py:549-561`, verbatim. The middle one is the
    // arm where `p` is genuinely complex — the branch § 5.27 (iv)'s P6 declared unreachable and
    // step 5 found in the shipped suite.
    let consts: [(&str, [C64; 3]); 3] = [
        ("ok", [C64 { re: -18.0, im: 0.0 }, C64 { re: -21.0, im: 28.0 },
                C64 { re: -21.0, im: -28.0 }]),
        ("bad", [C64 { re: -194.0, im: 0.0 }, C64 { re: -23.0, im: 25.5 },
                 C64 { re: -23.0, im: -25.5 }]),
        ("real", [C64 { re: -20.0, im: 0.0 }, C64 { re: -82.0, im: 0.0 },
                  C64 { re: -138.0, im: 0.0 }]),
    ];
    c.d("N/const/n", consts.len());
    for (i, (name, roots)) in consts.iter().enumerate() {
        let k = format!("N/const/{i}");
        c.s(&format!("{k}/name"), name);
        for (t, z) in roots.iter().enumerate() {
            c.c(&format!("{k}/root/{t}"), *z);
        }
        c.opt(&format!("{k}/pair"), zeta_pair(*roots));
        c.opt(&format!("{k}/ring"), zeta_ring(*roots));
        // `p` and `s` THEMSELVES, so the complex branch is a KEY and not an inference. Python's
        // `sorted(roots, key=abs)[1:]` is a STABLE sort on the modulus, which `sort_by` is too.
        let mut nz = *roots;
        nz.sort_by(|a, b| a.abs().partial_cmp(&b.abs()).expect("no NaN modulus"));
        c.c(&format!("{k}/p"), c_mul(nz[1], nz[2]));
        c.c(&format!("{k}/s_sum"), c_add(nz[1], nz[2]));
    }

    // N/pair, N/ring — the INTERCEPTED call stream of sections A–M, replayed with the plant taken
    // out of the loop. The roots are INPUTS; the returned value is the assertion.
    let np = c.input_d("N/pair/ncalls");
    assert_eq!(np, n_pair,
               "the dump intercepted {np} `_zeta_pair` calls and this file's own readers made \
                {n_pair}. `split_modes` and `split_floor` are the only callers, so the two counts \
                are the SAME rows counted twice — a difference is a row the port did not build, \
                not a bookkeeping slip.");
    for i in 0..np {
        let k = format!("N/pair/{i}");
        let roots = [c.input_c(&format!("{k}/in/0")), c.input_c(&format!("{k}/in/1")),
                     c.input_c(&format!("{k}/in/2"))];
        c.opt(&format!("{k}/out"), zeta_pair(roots));
    }
    let nr = c.input_d("N/ring/ncalls");
    assert_eq!(nr, n_ring,
               "the dump intercepted {nr} `_zeta_ring` calls and this file's own readers made \
                {n_ring}. `full_modes` is the only caller.");
    for i in 0..nr {
        let k = format!("N/ring/{i}");
        let roots = [c.input_c(&format!("{k}/in/0")), c.input_c(&format!("{k}/in/1")),
                     c.input_c(&format!("{k}/in/2"))];
        c.opt(&format!("{k}/out"), zeta_ring(roots));
    }

    c.finish(arm);
}

#[test]
fn rungs70_and_71_are_bit_exact_against_pypy() {
    run(ORACLE_PYPY, "pypy", false);
}

/// **P6, SETTLED — and NOT off the readers' grid.** § 5.27 (iv) registered `p.im == 0` inside
/// [`zeta_pair`] as a gated condition measured over the rung-70 readers; step 5 falsified it from
/// `test_rung71.py`'s damping gate. This gate re-states the falsification as a VALUE, so the
/// complex branch is covered by an oracle key and not only by the `assert!` step 5 replaced.
///
/// The `bad` spectrum's two largest moduli are the REAL root and ONE MEMBER of the pair, so
/// `p = 4462 + 4947i` — and both `csqrt`'s complex branch and [`c_div`]'s Smith algorithm are
/// exercised. `real` is the arm where `_zeta_ring` returns `None` and `_zeta_pair` does not.
#[test]
fn the_constructed_spectrum_reaches_the_complex_branch_the_readers_do_not() {
    let bad = [C64 { re: -194.0, im: 0.0 }, C64 { re: -23.0, im: 25.5 },
               C64 { re: -23.0, im: -25.5 }];
    let mut nz = bad;
    nz.sort_by(|a, b| a.abs().partial_cmp(&b.abs()).expect("no NaN modulus"));
    let p = c_mul(nz[1], nz[2]);
    assert_ne!(p.im, 0.0,
               "the constructed spectrum is the whole point of section N: if `p` came back real \
                the grid moved and P6's branch is uncovered again");
    // The two readers disagree here — that is `test_rung71.py`'s damping gate, and the pair's
    // value is the one the real-only spelling could not produce.
    let (pair, ring) = (zeta_pair(bad).expect("a pair"), zeta_ring(bad).expect("a ring"));
    assert!((pair - ring).abs() > 0.5, "{pair} vs {ring}");
    let real = [C64 { re: -20.0, im: 0.0 }, C64 { re: -82.0, im: 0.0 },
                C64 { re: -138.0, im: 0.0 }];
    assert!(zeta_ring(real).is_none(), "no ring in a real spectrum");
    assert!(zeta_pair(real).is_some(), "rung 70's reader returns a number anyway — the defect");
}

/// **P5, SETTLED IN THE NEGATIVE, AND STATED RATHER THAN IMPLIED.** None of the three
/// `_rk4_floor*` guards is a cell, none is a value, and no `ds` this dump passes trips any of
/// them — so a silent absence from the golden would read like coverage. The check is that the
/// two `ds` values sections A–M use are both BELOW every floor's own bar, computed from the same
/// `ds * rate <= 2.0` the guards spell.
#[test]
fn no_value_key_in_this_oracle_can_see_an_rk4_floor() {
    // The fastest clock any section passes is `tau = 0.005` (`window_law`'s `tau_qs[0]` and
    // `full_modes`' arm 3), so the largest `rate` any guard sees is `1/0.005 = 200`.
    let fastest = 0.005_f64;
    for ds in [DS, 0.002] {
        assert!(ds / fastest <= 2.0,
                "ds = {ds} against the fastest clock {fastest} would TRIP a floor, so this \
                 oracle's grid does reach one after all and the header is wrong");
    }
}

/// **STEP 5's BOOKED ITEM, DISCHARGED — AND ITS BOOKING WAS WRONG BY ONE.**
///
/// [`zeta_ring`]'s doc comment quotes four reader disagreements: *"0.960 vs 0.686, 1.279 vs
/// 0.670, 1.045 vs 0.924, and 1.035 on an arm whose spectrum is entirely REAL"*. Step 5 measured
/// that the SECOND is reached by a shipped gate (the constructed spectrum) and booked *"the other
/// three"* here as coming from a **12-arm** clock grid that appears in no shipped signature.
///
/// Measured off this dump's own intercepted `_zeta_ring` stream — the 32 rows `full_modes` makes
/// on its OWN six-arm default, both readers driven on the SAME roots — **the fourth is on the
/// shipped grid too**, so it is the other **TWO** that are off it:
///
/// | rows | `zeta_pair` | `zeta_ring` | the doc comment's |
/// |---|---|---|---|
/// | 4, 5 | 1.278 | 0.670, 0.669 | **second** pair |
/// | 15, 16 | 1.035, 1.033 | `None` | **fourth** item, a REAL spectrum |
/// | — | — | — | `0.960 vs 0.686` and `1.045 vs 0.924` are NOT reachable here |
///
/// This gate pins the shipped-grid half so a doc-comment number stops being prose. It reads the
/// PyPy golden directly rather than through [`Cmp`] — those keys are section N's INPUTS, and a
/// second consumer of them in the same `Cmp` would trip its read-twice assertion.
#[test]
fn the_two_damping_readers_disagree_on_exactly_four_of_the_shipped_grids_32_rows() {
    let g = load(ORACLE_PYPY);
    let get = |k: &str| f64::from_bits(*g.get(k).unwrap_or_else(|| panic!("{k} missing")));
    let n = g["N/ring/ncalls"] as usize;
    assert_eq!(n, 32, "`full_modes`' six arms make 32 rows on its own default grid");
    let mut disagree = Vec::new();
    for i in 0..n {
        let roots: [C64; 3] = std::array::from_fn(|t| C64 {
            re: get(&format!("N/ring/{i}/in/{t}/re")),
            im: get(&format!("N/ring/{i}/in/{t}/im")),
        });
        match (zeta_pair(roots), zeta_ring(roots)) {
            (Some(p), None) => disagree.push((i, p, None)),
            (Some(p), Some(r)) if (p - r).abs() > 1e-9 => disagree.push((i, p, Some(r))),
            _ => {}
        }
    }
    let idx: Vec<usize> = disagree.iter().map(|(i, _, _)| *i).collect();
    assert_eq!(idx, vec![4, 5, 15, 16], "the disagreeing rows moved: {disagree:?}");
    // The doc comment's SECOND pair, on the two rows that carry it.
    for (i, p, r) in disagree.iter().take(2) {
        assert!((p - 1.278).abs() < 5e-4 && (r.expect("a ring") - 0.670).abs() < 2e-3,
                "row {i}: {p} vs {r:?} is no longer the doc comment's `1.279 vs 0.670`");
    }
    // …and its FOURTH item: a number where there is no ring at all, which is the whole reason
    // rung 71 rebuilt the reader.
    for (i, p, r) in disagree.iter().skip(2) {
        assert!(r.is_none(), "row {i} was supposed to be a REAL spectrum");
        assert!((p - 1.034).abs() < 2e-3, "row {i}: {p} is no longer the doc comment's `1.035`");
    }
}

#[test]
fn rungs70_and_71_against_cpython_with_the_declared_exemption() {
    run(ORACLE_CPYTHON, "cpython", true);
}
