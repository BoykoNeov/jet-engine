//! SLICE AD step 5 — **THE ORACLE for rung 72**, against PyPy *and* CPython 3.14.
//!
//! `rung72.rs` ports the suite's own 28 gates, and most of them are RELATIONS — a reduce arm equal
//! to an ancestor's march, a gain that is exactly zero, a rank that changes at a hand-over, a
//! residual under a bar. Relations are agreement, not correctness. **This file is the other
//! half**: every value `oracle/dump_slice_ad.py` emits, compared as its IEEE-754 bit pattern.
//!
//! # THIS FILE EXISTS BECAUSE OF ONE MEASUREMENT, AND IT IS BUILT AGAINST IT
//!
//! § 5.28.4 (a): injection **j05** deletes the `|a3|` term from Durand–Kerner's start scale — the
//! term § 5.28 (iii) measured winning `scale`'s max on **1 068 of 1 068** shipped calls. It moves
//! **26 of step 3's 3 216 values**, of which **eight are inside `charpoly_selftest`**, the rung's
//! own instrument-gated-against-itself, written for exactly this class of error. **And all 28
//! gates pass — in Rust AND in Python.** Their bars are one-sided (`< 1e-9`), a different start
//! converges to the same roots, and the residuals move in their last digits without crossing
//! anything. A tolerance gate cannot be made to catch that without inventing a bar nobody
//! measured, so the miss was BOOKED here rather than papered over. Section A and section H are
//! the seats that settle it.
//!
//! Two more misses were booked to this file, and they are NOT the same kind of thing:
//!
//! * **j09** moves exactly **3** keys — `shared_bill`'s incidence-arm stator row — and **no gate
//!   in either language reads any of them**. A *defence with no reader*, in the ledger.
//!   Section F carries all three.
//! * **j06** was NOT booked, and that is the point: it moves **0 of 3 216**, which is a stronger
//!   statement than "missed". Nothing here is added for it.
//!
//! # THE FOUR STRUCTURAL PROPERTIES THIS FILE IS BUILT AROUND
//!
//! **1. AN AGGREGATE IS LOSSY, SO SECTION G EMITS THE MARCH ITSELF.** Step 2 landed a 209-line
//! six-state integrator behind THIRTEEN gates that are all relations, and step 3's 3 216 keys are
//! the readers' aggregates over its trajectory — a `min`, a count and a window can all sit still
//! while the points under them move. The suite's own reduce spine compares **9 of the march's 30
//! recorded fields**. Section G emits every field of every fifth point of every distinct march the
//! readers drive, **plus the `min`, `max` and LAST of every float field over ALL points** — the
//! stride's backstop. **That backstop is real and it is weaker than "nothing can hide"**: it
//! catches a defect moving a column's EXTREME or its FINAL value wherever that defect sits, and
//! NOT one at a hidden point that moves neither. Measured at close-out — perturbing `sp_thrust`
//! at index 137 (`137 % 5 = 2`) moves **0 of 54 116 keys**, injection applied on 24 marches,
//! against a control at index 135 that moves 10. **1 302 of 6 470 points are emitted (20.1 %)**,
//! so what section G pins is every field at one point in five, plus both extremes and the
//! endpoint of all 240 columns.
//!
//! **2. A CELL CAN BREAK BY EMPTYING THE SAMPLE.** § 5.27 (ii) ran a parent's body in a child's
//! slot and the reader **returned successfully with an EMPTY table** — every aggregate `None`, no
//! value differing because there were no values. So every sample-shaped reading emits its ROW
//! COUNT and its SKIPPED counts, and every `Option` carries a PRESENCE FLAG beside it
//! ([`Cmp::opt`]).
//!
//! **3. A REGIME IS THE ONE THING NO FLOAT WITNESSES.** § 5.28.2 (a) measured a wrong label inside
//! a FILTER dropping a rung-72 point and then reporting perfect tracking over an empty set. Every
//! authority label, nozzle branch, stator regime, `ic_order` and ledger cell name is emitted as an
//! FNV-1a hash through [`Cmp::s`], and a discrete key that flips between interpreters is a hard
//! failure and never a rounding.
//!
//! **4. A COUNT IS ONLY EVIDENCE IF SOMETHING RE-DERIVES IT.** Sections G, H and J emit their own
//! census — signature counts, distinct-vector counts, near-double counts, `scale`-winner counts,
//! call counts, label histograms. **Wherever the Rust can recompute one from its own values it
//! does, and asserts it**; where it cannot (a call count over Python's own run) the key is read as
//! an INPUT and tied into a consistency web that must hold. [[rust-port-guessed-census-bars]] is
//! five typed count bars that were every one wrong.
//!
//! # SECTION J EXISTS SO THAT P4's ZERO IS A READING AND NOT AN ABSENCE
//!
//! **P4** predicts that writing `gf == gr` for `abs(gf − gr) <= tol` changes no oracle key, off
//! § 5.28 (iv)'s measurement of 25 702 suite calls with **0** in the open interval. § 5.28.4 (b)
//! ran it as the declared control `c11` and it was missed by all three binaries and all 28 gates —
//! but that is the GATE seat. **The prediction is VACUOUS if this dump's grid never reaches the
//! function at all**, and four instruments in this slice's own history printed a confident zero
//! from a run that reached nothing ([[rust-port-slice-ad-preflight]]). So the grid's own call
//! count, distinct-pair count, exact-zero count, OPEN-INTERVAL count, label histogram and
//! `min_nonzero_gap` are all keys. `J/n_open` is the key P4 lives or dies on, and
//! `J/min_nonzero_gap` is the MARGIN — a number, never the word "unreachable".
//!
//! # THE PRE-FLIGHT's QUARTIC NUMBERS ARE ABOUT A DIFFERENT POPULATION, AND SECTION H SAYS SO
//!
//! **P3** reads *"the oracle's `_quartic_roots_c` section agrees on 375 distinct coefficient
//! vectors, and the 167 near-double cases are where a disagreement lands"*. Both numbers were
//! measured by probe F over the **WHOLE rung-72 SUITE** — 1 068 calls. This dump's grid is the
//! five readers, which is smaller, so both are RE-MEASURED here and emitted as keys. Quoting the
//! pre-flight's against this dump would be § 5.27.6 (i) exactly: a row measured at one stride
//! quoted against a fixture passing another. [`the_quartic_census_is_this_grids_own`] is a
//! tripwire that FAILS if the suite-wide pair is ever transcribed back in.
//!
//! # THE CROSS-INTERPRETER EXEMPTION — a set of NAMES, read off the diff, annotated after
//!
//! § 5.28.3 (h) pre-registered **P7** BEFORE this step could produce a list, because slices Z, AB
//! and AC each carried one and each was falsified in an instructive direction, and a post-hoc list
//! asserts nothing. [`EXEMPT`] is nevertheless read off THIS dump's own diff and annotated
//! afterwards, in that order — [[rust-port-slice-z-step4]] is a pre-registered exemption of TWO
//! keys that measured EIGHT, because it counted quantities where a dump emits names. **The port is
//! held to PyPy**, where nothing is exempt, and the set is checked in BOTH directions: a key that
//! STOPS drifting is as much a change as a new one.
//!
//! # WHAT THIS ORACLE CANNOT SEE, NAMED HERE SO STEP 6 OWNS IT
//!
//! * **The three cells' DISPATCH.** No value key can witness a function-pointer table, and
//!   § 5.28 (vii)'s laundering map already fixes each cell's honest scoring seat: `_shared_rig` on
//!   any rig reader, `_reference` and `_rk4_floor_shared` only on a DIRECT march, because
//!   `_shared_rig` calls `at_lever` at its third line and launders an injected core before a rig
//!   reader sees it. That is `slice_ad_dispatch.rs`'s subject.
//! * **`_reference` is the bitwise identity at this rung** (§ 5.28 (vi), 195 278 of 195 278 calls),
//!   so no value key below can distinguish it from its absence. Its gate is a SENTINEL and the
//!   value break arrives at slice AE.
//! * **The raises** — `_assert_fuel_boundary`'s two bars, `_rk4_floor_shared`'s `ds * rate <= 2.0`,
//!   the four arming asserts, the joint-IC residual. Nothing here passes a `ds` that trips the
//!   floor. Stated rather than implied, because a silent absence reads as coverage.
//! * **The `tie` branch and the three dead roots.** Section J measures **zero** `tie` labels on
//!   this grid where the whole suite has one, and section H measures `|a3|` winning `scale` on
//!   every vector — so the cube root and both even roots are unreachable here. Both are MEASURED
//!   keys below rather than claims.
//! * **`_quad_gains_at`** — § 5.28.3 (a)'s fourth cell, PASSED and never called, deferred to slice
//!   AE. Its VALUES are covered (every row of sections C, D and E comes out of it); its DISPATCH
//!   is not, and cannot be, because no hook exists for it yet.
//!
//! Regenerate both:
//! ```text
//! .venv/Scripts/python.exe rust/oracle/dump_slice_ad.py > rust/oracle/slice_ad_pypy.tsv
//! C:/Python314/python.exe  rust/oracle/dump_slice_ad.py > rust/oracle/slice_ad_cpython.tsv
//! ```
//! **Through a POSIX shell, not PowerShell 5.1** — it writes a UTF-8 BOM that lands in front of
//! the `#` on line 1, so the header parses as data. [[windows-tooling-file-hazards]].

use std::collections::{BTreeMap, BTreeSet};

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::reference_split::C64;
use turbojet::shared_actuator::{
    authority, authority_law, build_shared_actuator_cascade, charpoly_selftest,
    mask_discriminator, quartic_roots_c, shared_bill, shared_cells, shared_gains, shared_march,
};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_ad_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_ad_cpython.tsv");

/// The number of keys `dump_slice_ad.py` emits — **its own stderr tally, not a neighbouring
/// slice's and not an estimate**. [`load`] bars 95 % of it, so a golden truncated mid-write cannot
/// present as a pass. [[rust-port-guessed-census-bars]].
const GOLDEN_KEYS: usize = 54_116;

/// The key NAMES the CPython 3.14 arm is allowed to differ on — **MEASURED off this dump's own
/// diff, never predicted**, and annotated only afterwards. See the header's P7 paragraph.
///
/// **THE SET IS 180 AND THE TWO GOLDENS DIFFER ON 5 022.** The gap is not slack: section H's
/// coefficient vectors are read as INPUTS, so on the CPython arm the Rust replays Durand–Kerner
/// on CPython's OWN coefficients — and reproduces CPython's roots bit-for-bit, on all 374
/// vectors. **The 4 842 H keys that differ between the two interpreters are therefore entirely
/// upstream of the solver**, in `_charpoly4`'s `sum()`-built polynomial, and not in the root
/// finder. That is a sharper reading of P3 than P3 predicted, and it is only available because
/// the section is input-fed rather than reconstructed.
///
/// # CAUSE 1 — DOWNSTREAM OF A FLOAT OR COMPLEX `sum()` (174 names)
///
/// CPython 3.12+'s `sum()` is Neumaier-COMPENSATED and PyPy's is a naive fold. `_charpoly4`'s two
/// traces are `sum()`, so `coef[4]` moves, and everything read off it moves with it: section C's
/// `det` and `det_range` (121), section D's `det`/`gap`/`pole` (34), section E's poles and
/// `worst_re` (15), and section A's four (4). § 5.28.3 (e) measured that the compensation reaches
/// **complex** too, where five shipped comments in this repo say "for floats" — `sum(roots)` at
/// rung 72 is the ladder's ONLY complex `sum()`.
///
/// # CAUSE 2 — THE PLANT's OWN MARCH DRIFT, AND **IT IS WHAT FALSIFIES P7's HEADLINE** (6 names)
///
/// P7 reads *"exempt on the keys downstream of a float or complex `sum()`, **and on nothing
/// else**"*, with the falsifier *"any exempt key that is not downstream of a `sum()` falsifies
/// P7"*. These six are march values — `required`, `required_gov`, `f`, `g`, `g_gov`, `mf` — at
/// **two points** (`sig/6/pt/465`, `sig/7/pt/395`) of the 1 302 section G emits, in 2 of the 10
/// signatures, differing by **1–4 ULPs**. There is no `sum()` anywhere in the six-state march.
/// This is [[rust-port-slice-ac-step6]]'s cause 3 — *a solve terminating differently* — arriving
/// at rung 72, which is exactly what P7's own clause (ii) says and exactly what its headline
/// forbids. **The headline falls and clause (ii) stands**; it is not reinterpreted to rescue it.
///
/// **No aggregate key drifts.** `G/sig/*/agg/*/{min,max,last}` are computed over EVERY point, not
/// the strided ones, and not one of them differs — so the excursion does not reach either extreme
/// or the final point of any column. That BOUNDS the drift; it does not prove it decays, and the
/// stronger claim is not made here.
///
/// **The port is held to PyPy**, where nothing is exempt — which is what makes cause 2 an
/// audit-arm note and not a port defect.
const EXEMPT: [&str; 180] = [
    // --- CAUSE 1: downstream of `_charpoly4`'s `sum()` (174 names) ---
    "A/general/det_err",
    "A/general/det_vs_a0",
    "A/general/trace_err",
    "A/triangular/trace_err",
    "C/0/det_range/hi",
    "C/0/det_range/lo",
    "C/0/row/0/det",
    "C/0/row/1/det",
    "C/0/row/10/det",
    "C/0/row/11/det",
    "C/0/row/12/det",
    "C/0/row/13/det",
    "C/0/row/14/det",
    "C/0/row/15/det",
    "C/0/row/16/det",
    "C/0/row/17/det",
    "C/0/row/18/det",
    "C/0/row/19/det",
    "C/0/row/2/det",
    "C/0/row/20/det",
    "C/0/row/21/det",
    "C/0/row/22/det",
    "C/0/row/23/det",
    "C/0/row/24/det",
    "C/0/row/25/det",
    "C/0/row/26/det",
    "C/0/row/27/det",
    "C/0/row/28/det",
    "C/0/row/29/det",
    "C/0/row/3/det",
    "C/0/row/30/det",
    "C/0/row/31/det",
    "C/0/row/32/det",
    "C/0/row/33/det",
    "C/0/row/34/det",
    "C/0/row/35/det",
    "C/0/row/36/det",
    "C/0/row/37/det",
    "C/0/row/38/det",
    "C/0/row/39/det",
    "C/0/row/4/det",
    "C/0/row/40/det",
    "C/0/row/41/det",
    "C/0/row/42/det",
    "C/0/row/43/det",
    "C/0/row/44/det",
    "C/0/row/45/det",
    "C/0/row/46/det",
    "C/0/row/47/det",
    "C/0/row/48/det",
    "C/0/row/49/det",
    "C/0/row/5/det",
    "C/0/row/50/det",
    "C/0/row/51/det",
    "C/0/row/52/det",
    "C/0/row/53/det",
    "C/0/row/54/det",
    "C/0/row/55/det",
    "C/0/row/56/det",
    "C/0/row/57/det",
    "C/0/row/58/det",
    "C/0/row/59/det",
    "C/0/row/6/det",
    "C/0/row/60/det",
    "C/0/row/61/det",
    "C/0/row/62/det",
    "C/0/row/63/det",
    "C/0/row/64/det",
    "C/0/row/65/det",
    "C/0/row/66/det",
    "C/0/row/67/det",
    "C/0/row/68/det",
    "C/0/row/69/det",
    "C/0/row/7/det",
    "C/0/row/70/det",
    "C/0/row/71/det",
    "C/0/row/72/det",
    "C/0/row/73/det",
    "C/0/row/74/det",
    "C/0/row/75/det",
    "C/0/row/76/det",
    "C/0/row/77/det",
    "C/0/row/78/det",
    "C/0/row/79/det",
    "C/0/row/8/det",
    "C/0/row/80/det",
    "C/0/row/81/det",
    "C/0/row/9/det",
    "C/1/det_range/hi",
    "C/1/det_range/lo",
    "C/1/row/0/det",
    "C/1/row/1/det",
    "C/1/row/10/det",
    "C/1/row/11/det",
    "C/1/row/12/det",
    "C/1/row/13/det",
    "C/1/row/14/det",
    "C/1/row/15/det",
    "C/1/row/16/det",
    "C/1/row/17/det",
    "C/1/row/18/det",
    "C/1/row/19/det",
    "C/1/row/2/det",
    "C/1/row/20/det",
    "C/1/row/21/det",
    "C/1/row/22/det",
    "C/1/row/23/det",
    "C/1/row/24/det",
    "C/1/row/25/det",
    "C/1/row/26/det",
    "C/1/row/27/det",
    "C/1/row/28/det",
    "C/1/row/29/det",
    "C/1/row/3/det",
    "C/1/row/30/det",
    "C/1/row/31/det",
    "C/1/row/32/det",
    "C/1/row/33/det",
    "C/1/row/34/det",
    "C/1/row/4/det",
    "C/1/row/5/det",
    "C/1/row/6/det",
    "C/1/row/7/det",
    "C/1/row/8/det",
    "C/1/row/9/det",
    "D/arm/0/fuel/det/hi",
    "D/arm/0/fuel/det/lo",
    "D/arm/0/fuel/gap",
    "D/arm/0/gov/det/hi",
    "D/arm/0/gov/det/lo",
    "D/arm/0/gov/gap",
    "D/arm/0/gov/pole",
    "D/arm/1/fuel/det/hi",
    "D/arm/1/fuel/det/lo",
    "D/arm/1/fuel/gap",
    "D/arm/1/fuel/pole",
    "D/arm/1/gov/det/hi",
    "D/arm/1/gov/det/lo",
    "D/arm/1/gov/pole",
    "D/arm/2/fuel/det/hi",
    "D/arm/2/fuel/det/lo",
    "D/arm/2/fuel/pole",
    "D/arm/2/gov/det/hi",
    "D/arm/2/gov/det/lo",
    "D/arm/2/gov/gap",
    "D/arm/2/gov/pole",
    "D/arm/3/fuel/det/hi",
    "D/arm/3/fuel/det/lo",
    "D/arm/3/fuel/pole",
    "D/arm/3/gov/det/hi",
    "D/arm/3/gov/det/lo",
    "D/arm/3/gov/pole",
    "D/cell/0/fuel/gap",
    "D/cell/0/gov/gap",
    "D/cell/0/gov/pole",
    "D/cell/1/fuel/pole",
    "D/cell/1/gov/gap",
    "D/cell/1/gov/pole",
    "D/worst_pole",
    "E/arm/0/max/worst_pole",
    "E/arm/0/max/worst_re",
    "E/arm/0/sum/worst_re",
    "E/arm/1/max/worst_pole",
    "E/arm/1/max/worst_re",
    "E/arm/1/sum/worst_pole",
    "E/arm/1/sum/worst_re",
    "E/arm/2/max/worst_pole",
    "E/arm/2/max/worst_re",
    "E/arm/2/sum/worst_pole",
    "E/arm/2/sum/worst_re",
    "E/max_pole_unmatched",
    "E/max_worst_re",
    "E/sum_pole_unmatched",
    "E/sum_worst_re",
    // --- CAUSE 2: THE MARCH ITSELF, and the 6 names that falsify P7's headline ---
    "G/sig/6/pt/465/required",
    "G/sig/6/pt/465/required_gov",
    "G/sig/7/pt/395/f",
    "G/sig/7/pt/395/g",
    "G/sig/7/pt/395/g_gov",
    "G/sig/7/pt/395/mf",
];

// ============================================================================== the grid
//
// `tests/test_rung72.py`'s module constants, verbatim, and then the five readers' OWN defaults
// read off `turbojet/engine.py`'s `def` lines. See `rung72.rs`'s header table: three distinct
// `ds`, two distinct `every`, three distinct clock grids, and the suite passes NONE of them.

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const V_MAX: f64 = 0.20;
const SM: f64 = PHI / FLOOR - 1.0;
const TAU: f64 = 0.05;
const TT4_MAX: f64 = 1200.0;

const CLOCKS: [(f64, f64, f64, f64); 2] = [(0.05, 0.05, 0.05, 0.05), (0.20, 0.01, 0.50, 0.05)];
const MD_CLOCKS: [(f64, f64, f64, f64); 3] = [
    (0.05, 0.05, 0.05, 0.05), (0.05, 0.08, 0.05, 0.05), (0.02, 0.09, 0.05, 0.05),
];
const AL_DS: f64 = 0.005;
const SG_TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const SG_DS: f64 = 0.002;
const SG_EVERY: usize = 2;
const SC_DS: f64 = 0.002;
const SC_EVERY: usize = 2;
const MD_DS: f64 = 0.002;
const MD_EVERY: usize = 4;
const SB_TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const SB_DS: f64 = 0.005;

/// Section G's stride, and it is **chosen to be coprime to the readers' own `every`** (2 and 4):
/// a stride of 2 or 4 would emit exactly the points the gain rows already cover, which is an extra
/// grid that is not extra. Pinned by [`the_march_stride_is_coprime_to_the_readers_sampling`].
const STRIDE: usize = 5;

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
    build_two_spool_turbojet(cpg(), PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

/// Python's `_shared(design, bleed_lim=_valve())` — the ONE machine every reader below is driven
/// on, exactly as the dump builds it once and calls all five on it.
fn shared() -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp(), B, SM, Some(TAU))),
        ..LeverArm::default()
    };
    match build_shared_actuator_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, &arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

// ============================================================================== the comparator

fn load(text: &str) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, bits) = (it.next().expect("key"), it.next().expect("bits"));
        let v = bits.parse::<u64>().unwrap_or_else(|e| panic!(
            "slice-AD golden line is not `key<TAB>u64` ({e}): {line:?}. If the second field has \
             text appended, the dump was redirected with `2>&1` and its stderr interleaved. If \
             the FIRST line failed, the file has a UTF-8 BOM: it was redirected through \
             PowerShell, which writes one."));
        assert!(m.insert(k.to_string(), v).is_none(), "dup {k}");
    }
    assert!(m.len() > GOLDEN_KEYS - GOLDEN_KEYS / 20,
            "the slice-AD golden did not parse ({} keys, expected about {GOLDEN_KEYS})", m.len());
    m
}

fn fnv1a(text: &str) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for byte in text.as_bytes() {
        h = (h ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

fn regime_str(r: Regime) -> &'static str {
    match r {
        Regime::Dormant => "dormant",
        Regime::Riding => "riding",
        Regime::Saturated => "saturated",
    }
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

    /// A STRING as its FNV-1a hash. **A REGIME IS THE ONE THING NO FLOAT WITNESSES** — see the
    /// header's structural property 3.
    fn s(&mut self, key: &str, got: &str) { self.raw(key, fnv1a(got), true); }

    /// **THE PRESENCE FLAG IS THE POINT, NOT THE VALUE** — header property 2.
    fn opt(&mut self, key: &str, got: Option<f64>) {
        self.b(&format!("{key}?"), got.is_some());
        if let Some(x) = got { self.f(key, x); }
    }

    fn opt_s(&mut self, key: &str, got: Option<&str>) {
        self.b(&format!("{key}?"), got.is_some());
        if let Some(x) = got { self.s(key, x); }
    }

    fn opt_span(&mut self, key: &str, got: Option<(f64, f64)>) {
        self.b(&format!("{key}?"), got.is_some());
        if let Some((lo, hi)) = got {
            self.f(&format!("{key}/lo"), lo);
            self.f(&format!("{key}/hi"), hi);
        }
    }

    /// A COMPLEX root — `re`, `im` and `abs`. **`abs` is a KEY and not a convenience**: § 5.28
    /// (iii) measured Durand–Kerner leaving an ASYMMETRIC last-bit imaginary residue, which is what
    /// makes bit-exactness the only achievable bar here.
    fn c(&mut self, key: &str, z: C64) {
        self.f(&format!("{key}/re"), z.re);
        self.f(&format!("{key}/im"), z.im);
        self.f(&format!("{key}/abs"), z.abs());
    }

    /// **A GOLDEN KEY READ AS AN INPUT AND NOT COMPARED.** Sections G, H and J replay shipped
    /// functions on arguments Python's interceptors captured; re-emitting those arguments as
    /// assertions would compare a key with itself, which slice U step 4 recorded as a gate that
    /// cannot see its own value. Marking them `seen` keeps [`Cmp::finish`]'s missing-key half
    /// honest without pretending they were checked.
    fn input_f(&mut self, key: &str) -> f64 {
        assert!(self.seen.insert(key.to_string()), "the Rust read {key} twice");
        f64::from_bits(*self.py.get(key)
            .unwrap_or_else(|| panic!("{key}: NO GOLDEN — a declared-grid input is missing")))
    }

    fn input_d(&mut self, key: &str) -> usize {
        assert!(self.seen.insert(key.to_string()), "the Rust read {key} twice");
        *self.py.get(key)
            .unwrap_or_else(|| panic!("{key}: NO GOLDEN — a declared-grid input is missing"))
            as usize
    }

    fn input_b(&mut self, key: &str) -> bool { self.input_d(key) != 0 }

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
                "{} DISCRETE keys flipped between interpreters -- a flipped count, flag, authority \
                 label, nozzle branch, stator regime or ledger cell name is a different physical \
                 reading, never a rounding:\n  {}",
                self.flips.len(),
                self.flips.iter().take(12).cloned().collect::<Vec<_>>().join("\n  "));
        let worst = self.drifts.iter().map(|(_, r)| *r).fold(0.0_f64, f64::max);
        assert!(self.drifts.is_empty(),
                "{} float keys drifted against CPython OUTSIDE the declared exemption (worst \
                 {worst:.3e}). The exemption is a NAMED LIST measured off this dump's own diff and \
                 pre-registered as P7 (§ 5.28.3 (h)) -- read this file's header before widening it, \
                 and never replace it with a tolerance:\n  {}",
                self.drifts.len(),
                self.drifts.iter().take(12).map(|(l, r)| format!("{l} (rel {r:.3e})"))
                    .collect::<Vec<_>>().join("\n  "));
        if self.cpython {
            let want: BTreeSet<String> = EXEMPT.iter().map(|s| (*s).to_string()).collect();
            assert_eq!(self.exempted, want,
                       "the CPython exemption set MOVED. Expected exactly the names in `EXEMPT`; \
                        got {} names. A key that STOPPED drifting is a change too -- it would mean \
                        the port's fold, the dump or CPython's `sum()` moved.\n\
                        only-in-EXEMPT: {:?}\nonly-measured: {:?}",
                       self.exempted.len(),
                       want.difference(&self.exempted).take(20).collect::<Vec<_>>(),
                       self.exempted.difference(&want).take(20).collect::<Vec<_>>());
        }
        if self.bad.is_empty() && missed.is_empty() {
            println!("slice_ad_oracle ({arm}): {} values compared, {} exempt",
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

// ============================================================================== the walk

/// The march point's THIRTY fields, split by type exactly as `dump_slice_ad.py` splits them.
/// Spelled out on both sides rather than looped generically, because a generic loop on one side
/// and a hand-written list on the other is the pair that silently drifts.
fn emit_point(cmp: &mut Cmp, q: &str, p: &FuelPoint) {
    let (g, required, b, b_cmd, v, v_cmd, v_regime, ic_iters, ic_res, ic_order, g_fuel, g_gov,
         required_fuel, required_gov, auth, share_law) = match p.extra {
        PointExtra::Shared { g, required, b, b_cmd, v, v_cmd, v_regime, ic_iters, ic_res,
                             ic_order, g_fuel, g_gov, required_fuel, required_gov, authority,
                             share_law } =>
            (g, required, b, b_cmd, v, v_cmd, v_regime, ic_iters, ic_res, ic_order, g_fuel, g_gov,
             required_fuel, required_gov, authority, share_law),
        _ => panic!("{q}: section G marches the SIX-STATE integrator; this point is not `Shared`, \
                     which means the dispatch left through an inherited marcher"),
    };
    for (name, x) in [
        ("s", p.s), ("nu_lp", p.nu_lp), ("nu_hp", p.nu_hp), ("Tt4", p.tt4), ("f", p.f),
        ("pi_lpc", p.pi_lpc), ("pi_hpc", p.pi_hpc), ("phi_lp", p.phi_lp), ("phi_hp", p.phi_hp),
        ("mdot_air", p.mdot_air), ("sp_thrust", p.sp_thrust), ("mf", p.mf),
        ("mf_sched", p.mf_sched), ("g", g), ("required", required), ("g_fuel", g_fuel),
        ("g_gov", g_gov), ("required_fuel", required_fuel), ("required_gov", required_gov),
        ("b", b), ("b_cmd", b_cmd), ("v", v), ("v_cmd", v_cmd), ("ic_res", ic_res),
    ] {
        cmp.f(&format!("{q}/{name}"), x);
    }
    cmp.s(&format!("{q}/branch"), p.branch.label());
    cmp.s(&format!("{q}/authority"), auth.as_str());
    cmp.s(&format!("{q}/ic_order"), ic_order);
    cmp.s(&format!("{q}/share_law"), share_law);
    cmp.opt_s(&format!("{q}/v_regime"), v_regime.map(regime_str));
    cmp.d(&format!("{q}/ic_iters"), ic_iters);
}

/// The 24 float fields of a point, for section G's whole-trajectory aggregates.
fn point_float(p: &FuelPoint, i: usize) -> f64 {
    let (g, required, b, b_cmd, v, v_cmd, ic_res, g_fuel, g_gov, required_fuel, required_gov) =
        match p.extra {
            PointExtra::Shared { g, required, b, b_cmd, v, v_cmd, ic_res, g_fuel, g_gov,
                                 required_fuel, required_gov, .. } =>
                (g, required, b, b_cmd, v, v_cmd, ic_res, g_fuel, g_gov, required_fuel,
                 required_gov),
            _ => panic!("section G's aggregates need a six-state point"),
        };
    [p.s, p.nu_lp, p.nu_hp, p.tt4, p.f, p.pi_lpc, p.pi_hpc, p.phi_lp, p.phi_hp, p.mdot_air,
     p.sp_thrust, p.mf, p.mf_sched, g, required, g_fuel, g_gov, required_fuel, required_gov, b,
     b_cmd, v, v_cmd, ic_res][i]
}

/// `PT_FLOAT`'s names, in the dump's own order — the aggregates' key names.
const PT_FLOAT: [&str; 24] = [
    "s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp", "phi_hp", "mdot_air",
    "sp_thrust", "mf", "mf_sched", "g", "required", "g_fuel", "g_gov", "required_fuel",
    "required_gov", "b", "b_cmd", "v", "v_cmd", "ic_res",
];

fn walk(cmp: &mut Cmp) {
    let core = shared();
    let fl = flight();

    // ---------------------------------------------------------------- A: charpoly_selftest
    let cp = charpoly_selftest();
    cmp.d("A/n_arms", cp.len());
    for (name, a) in cp.iter() {
        // Python's keys sorted: det_err, det_vs_a0, diag_err?, max_imag?, resid, trace_err — the
        // two optional ones exist on the triangular arm only, and Python simply has no key for
        // them on the general arm. The dump emits what the dict HAS, so the port must too.
        cmp.f(&format!("A/{name}/det_err"), a.det_err);
        cmp.f(&format!("A/{name}/det_vs_a0"), a.det_vs_a0);
        if let Some(x) = a.diag_err { cmp.f(&format!("A/{name}/diag_err"), x); }
        if let Some(x) = a.max_imag { cmp.f(&format!("A/{name}/max_imag"), x); }
        cmp.f(&format!("A/{name}/resid"), a.resid);
        cmp.f(&format!("A/{name}/trace_err"), a.trace_err);
    }

    // ---------------------------------------------------------------- B: authority_law
    let al = authority_law(&core, &fl, LO, HI, TT4_MAX, SM, &CLOCKS, R, SETTLE, AL_DS, V_MAX);
    cmp.b("B/both_cells_everywhere", al.both_cells_everywhere);
    cmp.b("B/one_handover", al.one_handover);
    cmp.f("B/ds", al.ds);
    cmp.d("B/n_arms", al.arms.len());
    for (i, a) in al.arms.iter().enumerate() {
        let p = format!("B/arm/{i}");
        cmp.b(&format!("{p}/inc"), a.inc);
        cmp.d(&format!("{p}/n"), a.n);
        for (lab, n) in a.census.iter() {
            cmp.d(&format!("{p}/census/{}", lab.as_str()), *n);
        }
        cmp.d(&format!("{p}/handovers/n"), a.handovers.len());
        for (j, h) in a.handovers.iter().enumerate() {
            cmp.f(&format!("{p}/handovers/{j}"), *h);
        }
        for (w, sp) in [("fuel", &a.fuel), ("gov", &a.gov), ("valve", &a.valve),
                        ("stator", &a.stator), ("joint", &a.joint)] {
            cmp.opt(&format!("{p}/{w}/lo"), sp.lo);
            cmp.opt(&format!("{p}/{w}/hi"), sp.hi);
            cmp.d(&format!("{p}/{w}/n"), sp.n);
        }
        cmp.f(&format!("{p}/joint_fraction"), a.joint_fraction);
        cmp.d(&format!("{p}/both_want"), a.both_want);
        cmp.d(&format!("{p}/in_joint/fuel"), a.in_joint_fuel);
        cmp.d(&format!("{p}/in_joint/gov"), a.in_joint_gov);
        cmp.b(&format!("{p}/handover_inside"), a.handover_inside);
        cmp.f(&format!("{p}/min_phi"), a.min_phi);
        cmp.f(&format!("{p}/max_Tt4"), a.max_tt4);
    }

    // ---------------------------------------------------------------- C: shared_gains
    for inc in [false, true] {
        let g = shared_gains(&core, &fl, LO, HI, TT4_MAX, SM, SG_TAUS, inc, R, SETTLE, SG_DS,
                             V_MAX, SG_EVERY).expect("§ 1's rig marches");
        let p = format!("C/{}", inc as usize);
        cmp.opt(&format!("{p}/worst_F_r"), g.worst_f_r);
        cmp.opt(&format!("{p}/worst_R_f"), g.worst_r_f);
        cmp.opt(&format!("{p}/worst_pair_FR"), g.worst_pair_fr);
        cmp.opt(&format!("{p}/worst_mask_leak"), g.worst_mask_leak);
        cmp.opt(&format!("{p}/min_live_gain"), g.min_live_gain);
        cmp.d(&format!("{p}/n_riding"), g.n_riding);
        cmp.d(&format!("{p}/n_sampled"), g.n_sampled);
        cmp.d(&format!("{p}/n_rows"), g.rows.len());
        cmp.d(&format!("{p}/skipped/switch"), g.skipped_switch);
        cmp.d(&format!("{p}/skipped/regime"), g.skipped_regime);
        cmp.d(&format!("{p}/by_authority/fuel"), g.by_authority_fuel);
        cmp.d(&format!("{p}/by_authority/gov"), g.by_authority_gov);
        cmp.opt_span(&format!("{p}/s_window"), g.s_window);
        cmp.opt_span(&format!("{p}/det_range"), g.det_range);
        cmp.d(&format!("{p}/n_boundary"), g.boundary.len());
        for (j, bd) in g.boundary.iter().enumerate() {
            let q = format!("{p}/boundary/{j}");
            cmp.f(&format!("{q}/s"), bd.s);
            for (h, x) in [("live/F_q", bd.live_f_q), ("live/F_v", bd.live_f_v),
                           ("live/R_q", bd.live_r_q), ("live/R_v", bd.live_r_v),
                           ("dead/F_q", bd.dead_f_q), ("dead/F_v", bd.dead_f_v),
                           ("dead/R_q", bd.dead_r_q), ("dead/R_v", bd.dead_r_v)] {
                cmp.f(&format!("{q}/{h}"), x);
            }
        }
        for (j, row) in g.rows.iter().enumerate() {
            let q = format!("{p}/row/{j}");
            cmp.f(&format!("{q}/s"), row.s);
            cmp.f(&format!("{q}/det"), row.det);
            cmp.opt_s(&format!("{q}/authority"), row.authority.map(|a| a.as_str()));
            cmp.opt_s(&format!("{q}/masked"), row.masked.map(|a| a.as_str()));
            cmp.opt(&format!("{q}/mask_leak"), row.mask_leak);
            let gg = &row.gains;
            for (name, x) in [
                ("F_r", gg.f_r), ("F_q", gg.f_q), ("F_v", gg.f_v), ("R_f", gg.r_f),
                ("R_q", gg.r_q), ("R_v", gg.r_v), ("C_f", gg.c_f), ("C_r", gg.c_r),
                ("C_v", gg.c_v), ("V_f", gg.v_f), ("V_r", gg.v_r), ("V_q", gg.v_q),
                ("pair_FR", gg.pair_fr), ("pair_RC", gg.pair_rc), ("pair_CV", gg.pair_cv),
                ("pair_RV", gg.pair_rv), ("v_base", gg.v_base),
            ] {
                cmp.f(&format!("{q}/{name}"), x);
            }
        }
    }

    // ---------------------------------------------------------------- D: shared_cells
    let c = shared_cells(&core, &fl, LO, HI, TT4_MAX, SM, &CLOCKS, R, SETTLE, SC_DS, V_MAX,
                         SC_EVERY).expect("§ 2's rig marches");
    cmp.b("D/law_holds", c.law_holds);
    cmp.b("D/all_four_cells", c.all_four_cells);
    cmp.f("D/worst_parent_gap", c.worst_parent_gap);
    cmp.f("D/worst_v_gap", c.worst_v_gap);
    cmp.f("D/worst_pole", c.worst_pole);
    cmp.d("D/n_cells", c.cells.len());
    cmp.d("D/n_arms", c.arms.len());
    for ((inc, auth), sc) in c.cells.iter() {
        let p = format!("D/cell/{}/{}", *inc as usize, auth.as_str());
        cmp.d(&format!("{p}/parent"), sc.parent[5..].parse::<usize>()
            .expect("Python's parent label is `rung NN`"));
        cmp.d(&format!("{p}/zeros/n"), sc.zeros.len());
        for (j, z) in sc.zeros.iter().enumerate() { cmp.d(&format!("{p}/zeros/{j}"), *z); }
        cmp.f(&format!("{p}/gap"), sc.gap);
        cmp.f(&format!("{p}/vgap"), sc.vgap);
        cmp.f(&format!("{p}/pole"), sc.pole);
        cmp.d(&format!("{p}/n"), sc.n);
    }
    for (i, a) in c.arms.iter().enumerate() {
        let p = format!("D/arm/{i}");
        cmp.b(&format!("{p}/inc"), a.inc);
        cmp.d(&format!("{p}/n_riding"), a.n_riding);
        cmp.d(&format!("{p}/n_sampled"), a.n_sampled);
        cmp.d(&format!("{p}/skipped/switch"), a.skipped_switch);
        cmp.d(&format!("{p}/skipped/regime"), a.skipped_regime);
        cmp.d(&format!("{p}/skipped/parent"), a.skipped_parent);
        cmp.d(&format!("{p}/n_cells"), a.cells.len());
        for (auth, st) in a.cells.iter() {
            let q = format!("{p}/{}", auth.as_str());
            cmp.d(&format!("{q}/n"), st.n);
            cmp.d(&format!("{q}/n_parent"), st.n_parent);
            cmp.f(&format!("{q}/gap"), st.gap);
            cmp.f(&format!("{q}/vgap"), st.vgap);
            cmp.f(&format!("{q}/pole"), st.pole);
            cmp.opt_span(&format!("{q}/det"), st.det);
            cmp.opt_span(&format!("{q}/s"), st.s);
            cmp.d(&format!("{q}/zeros/n"), st.zeros.len());
            for (j, z) in st.zeros.iter().enumerate() { cmp.d(&format!("{q}/zeros/{j}"), *z); }
        }
    }

    // ---------------------------------------------------------------- E: mask_discriminator
    let md = mask_discriminator(&core, &fl, LO, HI, TT4_MAX, SM, &MD_CLOCKS, false, R, SETTLE,
                                MD_DS, V_MAX, MD_EVERY).expect("§ 3's rig marches");
    cmp.opt("E/max_pole_unmatched", md.max_pole_unmatched);
    cmp.opt("E/sum_pole_unmatched", md.sum_pole_unmatched);
    cmp.opt("E/sum_pole_matched", md.sum_pole_matched);
    cmp.f("E/sum_worst_re", md.sum_worst_re);
    cmp.f("E/max_worst_re", md.max_worst_re);
    cmp.d("E/n_arms", md.arms.len());
    for (i, a) in md.arms.iter().enumerate() {
        let p = format!("E/arm/{i}");
        cmp.b(&format!("{p}/matched"), a.matched);
        cmp.d(&format!("{p}/n"), a.n);
        for (law, lr) in [("max", &a.law_max), ("sum", &a.law_sum)] {
            let q = format!("{p}/{law}");
            cmp.opt(&format!("{q}/worst_pole"), lr.worst_pole);
            cmp.f(&format!("{q}/worst_re"), lr.worst_re);
            cmp.d(&format!("{q}/n_auth"), lr.authority.len());
            for (j, lab) in lr.authority.iter().enumerate() {
                cmp.s(&format!("{q}/authority/{j}"), lab.as_str());
            }
            cmp.d(&format!("{q}/n_zerokeys"), lr.zeros.len());
            for (auth, zs) in lr.zeros.iter() {
                cmp.d(&format!("{q}/zeros/{}/n", auth.as_str()), zs.len());
                for (j, z) in zs.iter().enumerate() {
                    cmp.d(&format!("{q}/zeros/{}/{j}", auth.as_str()), *z);
                }
            }
        }
    }

    // ---------------------------------------------------------------- F: shared_bill
    for inc in [false, true] {
        let bl = shared_bill(&core, &fl, LO, HI, TT4_MAX, SM, SB_TAUS, inc, R, SETTLE, SB_DS,
                             V_MAX);
        let p = format!("F/{}", inc as usize);
        for (name, x) in [
            ("fuel_marginal_phi", bl.fuel_marginal_phi),
            ("fuel_marginal_Tt4", bl.fuel_marginal_tt4),
            ("Tt4_full", bl.tt4_full), ("Tt4_no_fuel", bl.tt4_no_fuel),
            ("phi_full", bl.phi_full), ("phi_no_fuel", bl.phi_no_fuel),
        ] {
            cmp.f(&format!("{p}/{name}"), x);
        }
        cmp.opt(&format!("{p}/handover"), bl.handover);
        cmp.opt(&format!("{p}/delivered/phi"), bl.delivered_phi);
        cmp.opt(&format!("{p}/delivered/Tt4"), bl.delivered_tt4);
        cmp.opt(&format!("{p}/delivered/inc"), bl.delivered_inc);
        for k in 0..4 {
            let leg = bl.marginal[k].0;
            cmp.f(&format!("{p}/marginal/{leg}"), bl.marginal[k].1);
            cmp.f(&format!("{p}/alone/{leg}"), bl.alone[k].1);
            cmp.opt(&format!("{p}/kept/{leg}"), bl.kept[k].1);
        }
        cmp.d(&format!("{p}/n_cells"), bl.cells.len());
        for (name, cell) in bl.cells.iter() {
            let q = format!("{p}/cell/{name}");
            for (kk, x) in [("I", cell.i), ("E", cell.e), ("M", cell.m),
                            ("min_phi", cell.min_phi), ("max_Tt4", cell.max_tt4)] {
                cmp.f(&format!("{q}/{kk}"), x);
            }
            cmp.d(&format!("{q}/n"), cell.n);
            cmp.d(&format!("{q}/auth_fuel"), cell.auth_fuel);
            cmp.opt(&format!("{q}/handover"), cell.handover);
            cmp.opt(&format!("{q}/credit_phi"), cell.credit_phi);
            cmp.opt(&format!("{q}/credit_Tt4"), cell.credit_tt4);
            cmp.opt(&format!("{q}/credit_inc"), cell.credit_inc);
        }
    }

    // ---------------------------------------------------------------- G: the six-state march
    //
    // The signatures are INPUTS — Python intercepted them at `_shared_march`'s own boundary, so
    // the arms are the readers' own by construction and a reader that changes its grid changes
    // this section with it. `G/n_calls` is Python's own tally and cannot be recomputed here;
    // `G/n_sigs` CAN be, and is, by walking exactly that many blocks and letting the missing-key
    // half of `finish` fire if the count is short.
    let n_calls = cmp.input_d("G/n_calls");
    let n_sigs = cmp.input_d("G/n_sigs");
    let stride = cmp.input_d("G/stride");
    assert_eq!(stride, STRIDE, "the golden's stride and this file's disagree");
    assert!(n_calls >= n_sigs,
            "G/n_calls ({n_calls}) < G/n_sigs ({n_sigs}) — the dump counted distinct signatures \
             it never called");
    for i in 0..n_sigs {
        let p = format!("G/sig/{i}");
        let taus = (cmp.input_f(&format!("{p}/in/tau/0")), cmp.input_f(&format!("{p}/in/tau/1")),
                    cmp.input_f(&format!("{p}/in/tau/2")), cmp.input_f(&format!("{p}/in/tau/3")));
        let r = cmp.input_f(&format!("{p}/in/r"));
        let s_settle = cmp.input_f(&format!("{p}/in/s_settle"));
        let ds = cmp.input_f(&format!("{p}/in/ds"));
        let v_max = cmp.input_f(&format!("{p}/in/v_max"));
        let inc = cmp.input_b(&format!("{p}/in/inc"));
        let (_m, _surge, _lag, traj) =
            shared_march(&core, &fl, LO, HI, TT4_MAX, SM, taus, r, s_settle, ds, v_max, inc);
        cmp.d(&format!("{p}/n_points"), traj.len());
        cmp.d(&format!("{p}/n_emitted"), traj.len().div_ceil(STRIDE));
        // THE AGGREGATES ARE OVER EVERY POINT, NOT THE STRIDED ONES — the stride's backstop.
        for (k, name) in PT_FLOAT.iter().enumerate() {
            let col: Vec<f64> = traj.iter().map(|p| point_float(p, k)).collect();
            // Python's `min`/`max` over a list, which return the FIRST extreme and propagate a
            // NaN — `f64::min` does neither, so the fold is written the long way.
            let mut lo = col[0];
            let mut hi = col[0];
            for x in col.iter().skip(1) {
                if *x < lo { lo = *x; }
                if *x > hi { hi = *x; }
            }
            cmp.f(&format!("{p}/agg/{name}/min"), lo);
            cmp.f(&format!("{p}/agg/{name}/max"), hi);
            cmp.f(&format!("{p}/agg/{name}/last"), col[col.len() - 1]);
        }
        for j in (0..traj.len()).step_by(STRIDE) {
            emit_point(cmp, &format!("{p}/pt/{j}"), &traj[j]);
        }
    }

    // ---------------------------------------------------------------- H: the quartic solver
    //
    // Every count here is RE-DERIVED from the Rust's own roots and asserted, except `H/n_calls`
    // and `H/n_near_double_calls`, which are properties of Python's run. See the header: the
    // pre-flight's 375 / 167 were measured over the WHOLE SUITE and are NOT these numbers.
    let hn_calls = cmp.input_d("H/n_calls");
    let hn_distinct = cmp.input_d("H/n_distinct");
    let hn_near_calls = cmp.input_d("H/n_near_double_calls");
    assert!(hn_calls >= hn_distinct && hn_near_calls <= hn_calls,
            "H's census is not internally consistent: {hn_calls} calls, {hn_distinct} distinct, \
             {hn_near_calls} near-double");
    let mut near_seen = 0usize;
    let mut winners = [0usize; 5];
    for i in 0..hn_distinct {
        let p = format!("H/v/{i}");
        let mut coef = [0.0f64; 5];
        for (j, cf) in coef.iter_mut().enumerate() {
            *cf = cmp.input_f(&format!("{p}/in/coef/{j}"));
        }
        let roots = quartic_roots_c(&coef);
        for (j, z) in roots.iter().enumerate() { cmp.c(&format!("{p}/root/{j}"), *z); }
        // Python: `min(abs(r[i] - r[j]) for i < j)` — the FIRST minimum, in index order.
        let mut sep = f64::INFINITY;
        for a in 0..4 {
            for b in (a + 1)..4 {
                let d = C64 { re: roots[a].re - roots[b].re, im: roots[a].im - roots[b].im }.abs();
                if d < sep { sep = d; }
            }
        }
        cmp.f(&format!("{p}/min_sep"), sep);
        cmp.b(&format!("{p}/near_double"), sep < 1e-6);
        near_seen += usize::from(sep < 1e-6);
        // WHICH term wins `scale = max(1.0, |a3|, |a2|**0.5, |a1|**(1/3.), |a0|**0.25)`, by
        // Python's `max` semantics — FIRST of equal arguments, so ties go to the lower index.
        let cand = [1.0, coef[1].abs(), coef[2].abs().powf(0.5), coef[3].abs().powf(1.0 / 3.0),
                    coef[4].abs().powf(0.25)];
        let mut best = 0usize;
        for k in 1..5 { if cand[k] > cand[best] { best = k; } }
        cmp.d(&format!("{p}/scale_winner"), best);
        winners[best] += 1;
        cmp.d(&format!("{p}/n_complex"), roots.iter().filter(|z| z.im != 0.0).count());
    }
    cmp.d("H/n_near_double_distinct", near_seen);
    for (k, n) in winners.iter().enumerate() { cmp.d(&format!("H/scale_winner/{k}"), *n); }

    // ---------------------------------------------------------------- J: `_authority`'s margin
    //
    // The census keys are Python's own run and are read as INPUTS, then tied into a consistency
    // web that must hold — a count nothing re-derives is not evidence. The REPLAYED pairs are
    // compared for real: the Rust calls the shipped `authority` on each and must produce Python's
    // label, and `J/gap` is recomputed from the two inputs rather than read.
    let jn_calls = cmp.input_d("J/n_calls");
    let jn_distinct = cmp.input_d("J/n_distinct");
    let mut labels = 0usize;
    for lab in ["dormant", "tie", "fuel", "gov"] {
        labels += cmp.input_d(&format!("J/label/{lab}"));
    }
    let jn_zero = cmp.input_d("J/n_zero");
    let jn_tol = cmp.input_d("J/n_within_tol");
    let jn_open = cmp.input_d("J/n_open");
    let jmin = cmp.input_f("J/min_nonzero_gap");
    let jmin_over = cmp.input_f("J/min_nonzero_gap_over_tol");
    let jn_replay = cmp.input_d("J/n_replay");
    let jn_replay_zero = cmp.input_d("J/n_replay_zero");
    assert_eq!(labels, jn_calls,
               "J's label histogram sums to {labels} over {jn_calls} calls — the partition is not \
                exhaustive, so one of them is counting something else");
    assert!(jn_calls > 0 && jn_distinct > 0,
            "**P4 IS VACUOUS**: this dump's grid never reached `_authority` ({jn_calls} calls). \
             A prediction that a tolerance is inert is a reading only if something lands in the \
             function — [[rust-port-slice-ad-preflight]], four instruments that printed a \
             confident zero from a run that reached nothing.");
    assert_eq!(jn_open, jn_tol - jn_zero, "J/n_open is not `n_within_tol - n_zero`");
    assert_eq!(jn_replay, jn_replay_zero + 20,
               "J's replay set is `every exact zero + the twenty smallest non-zero gaps`; the \
                dump emitted {jn_replay} with {jn_replay_zero} zeros");
    assert_eq!(jn_replay_zero, jn_zero, "the replayed zeros are not all of them");
    assert_eq!(jn_open == 0, jmin > 1e-12,
               "the OPEN-INTERVAL count and the MARGIN disagree: n_open = {jn_open} but the \
                smallest non-zero gap is {jmin:.6e}");
    assert!((jmin_over - jmin / 1e-12).abs() <= 0.0,
            "J/min_nonzero_gap_over_tol is not the gap over the tolerance");
    for i in 0..jn_replay {
        let p = format!("J/pair/{i}");
        let gf = cmp.input_f(&format!("{p}/in/gf"));
        let gr = cmp.input_f(&format!("{p}/in/gr"));
        cmp.f(&format!("{p}/gap"), (gf - gr).abs());
        cmp.s(&format!("{p}/label"), authority(gf, gr).as_str());
    }
}

// ============================================================================== the gates

#[test]
fn rust_equals_pypy_on_every_key() {
    let mut cmp = Cmp::new(load(ORACLE_PYPY), false);
    walk(&mut cmp);
    cmp.finish("pypy");
}

#[test]
fn rust_equals_cpython_outside_the_named_exemption() {
    let mut cmp = Cmp::new(load(ORACLE_CPYTHON), true);
    walk(&mut cmp);
    cmp.finish("cpython 3.14");
}

/// **THE TWO GOLDENS MUST HAVE THE SAME KEY SET.** A dump that emitted a key on one interpreter
/// and not the other would make the exemption list a comparison between differently-shaped
/// files — § 5.28.4 (a)'s discipline, where each injected dump was checked to carry the same
/// 3 216-key SET as the clean one before any count was taken off it.
#[test]
fn the_two_goldens_have_the_same_key_set() {
    let (a, b) = (load(ORACLE_PYPY), load(ORACLE_CPYTHON));
    let (ka, kb): (BTreeSet<&String>, BTreeSet<&String>) = (a.keys().collect(), b.keys().collect());
    assert_eq!(ka, kb, "the two arms' goldens have different key sets: {} only in PyPy, {} only \
                        in CPython", ka.difference(&kb).count(), kb.difference(&ka).count());
    assert_eq!(a.len(), GOLDEN_KEYS, "GOLDEN_KEYS is stale");
}

/// **THE STRIDE IS COPRIME TO THE READERS' OWN SAMPLING, AND THAT IS THE WHOLE REASON IT IS 5.**
/// `shared_gains` and `shared_cells` sample the march at `every = 2` and `mask_discriminator` at
/// `every = 4`. A section-G stride of 2 or 4 would emit exactly the points their gain rows already
/// carry — an extra grid that is not extra. Written as a gate rather than a comment because a
/// later edit to `STRIDE` would silently undo it.
#[test]
fn the_march_stride_is_coprime_to_the_readers_sampling() {
    fn gcd(a: usize, b: usize) -> usize { if b == 0 { a } else { gcd(b, a % b) } }
    for every in [SG_EVERY, SC_EVERY, MD_EVERY] {
        assert_eq!(gcd(STRIDE, every), 1,
                   "STRIDE {STRIDE} shares a factor with a reader's `every` of {every}: section \
                    G would re-emit the points that reader's rows already cover");
    }
    assert!(STRIDE > 1, "a stride of 1 is not a stride");
}

/// **THE PRE-FLIGHT's QUARTIC CENSUS IS ABOUT THE WHOLE SUITE AND THIS ONE IS ABOUT FIVE READERS
/// — a tripwire against ever transcribing the first into the second.**
///
/// § 5.28 (iii) measured **1 068 calls, 375 distinct vectors, 167 near-double**, by intercepting
/// every call the whole rung-72 SUITE makes. This dump drives the five readers only. Quoting the
/// suite-wide pair against this grid would be [[rust-port-slice-ac-step6]]'s `every = 40`-vs-`10`
/// defect: a number measured at one population, asserted against another. **P3's second clause is
/// scoreable only because the near-double flag is emitted per vector**, which is what makes "the
/// near-double cases are where a disagreement lands" a checkable statement rather than a hope.
#[test]
fn the_quartic_census_is_this_grids_own() {
    let py = load(ORACLE_PYPY);
    let get = |k: &str| *py.get(k).unwrap_or_else(|| panic!("{k} missing")) as usize;
    let (calls, distinct) = (get("H/n_calls"), get("H/n_distinct"));
    let near = get("H/n_near_double_calls");
    assert_ne!((calls, near), (1_068, 167),
               "section H's census is the SUITE-WIDE pair (1 068 / 167) from § 5.28 (iii). Those \
                were measured over every call the whole rung-72 suite makes; this dump drives the \
                five readers. If they have genuinely converged, delete this tripwire and SAY SO.");
    assert!(calls > 0 && distinct > 0, "section H reached the solver on 0 inputs");
    assert!(distinct <= calls, "more distinct vectors than calls");
    // § 5.28 (iii): `|a3|` wins on every shipped call, so the cube root and both even roots are
    // DEAD. Measured here rather than claimed — and if it ever stops being true, the three risky
    // roots have become reachable and the disclosure in the header is wrong.
    assert_eq!(get("H/scale_winner/1"), distinct,
               "`|a3|` no longer wins `scale` on every distinct vector — the cube root and the \
                two even roots are no longer dead, and § 5.28 (iii)'s disclosure needs re-taking");
    for k in [0usize, 2, 3, 4] {
        assert_eq!(get(&format!("H/scale_winner/{k}")), 0, "scale winner {k} became reachable");
    }
}

/// **THE `tie` BRANCH IS NOT REACHED ON THIS GRID, AND A SILENT ABSENCE WOULD READ AS COVERAGE.**
///
/// § 5.28 (iv) measured the whole suite at 25 702 `_authority` calls with exactly ONE `tie`. This
/// dump's grid has none, so no key in it can witness the `tie` branch — stated as a gate so that
/// a later reader cannot mistake section J's green for coverage of all four labels.
#[test]
fn section_j_reaches_three_of_the_four_authority_labels() {
    let py = load(ORACLE_PYPY);
    let get = |k: &str| *py.get(k).unwrap_or_else(|| panic!("{k} missing")) as usize;
    assert_eq!(get("J/label/tie"), 0,
               "the `tie` branch is now reached on this grid. That is an IMPROVEMENT, not a \
                failure — update this gate and the header's `what this oracle cannot see` list.");
    for lab in ["dormant", "fuel", "gov"] {
        assert!(get(&format!("J/label/{lab}")) > 0,
                "section J stopped reaching the `{lab}` branch");
    }
}
