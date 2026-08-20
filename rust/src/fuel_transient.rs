//! RUNGS 43 + 45 — rung 35's FUEL control on rung 40's two-shaft plant, and that plant marched
//! against rung 41's imposed surge line.
//!
//! Rung 43 (`engine.py:4507–5437`) subclasses rung 40 and inverts the control: **fuel is imposed
//! and `Tt4` is an OUTPUT** of a forward burner, so the closure's one root is still `m_L` but the
//! mixture — and with it the whole hot end — floats. Rung 45 marches that plant against the
//! imposed `phi_surge` and reports the crossing. On top sits the entire **rung-46…52 limiter
//! family**, which is seven keyword arms on ONE method rather than seven classes: the TIT topping
//! governor, its lag, the feedforward `Wf/pt3` schedule, the `phi` feedback floor, a forced
//! release, that release's rate, and the realisable asymmetric lag.
//!
//! # What § 5.16's probes measured before any of this was written
//!
//! 1. **THE FUEL PATH REFUSES AN EQUILIBRIUM GAS, AND THE REFUSAL DOES NOT ESCAPE.**
//!    [`try_tt4_from_f`] asserts against `gas.equilibrium`, but that assert fires inside the
//!    closure's bracket scan, which CATCHES it and walks the low wall in. So an ordinary caller
//!    sees the **bracket** error naming a cause ("off the modeled speed-line region") that is not
//!    the actual one. Measured through `_instant_fuel` and `equilibrium_fuel` alike: **46
//!    swallowed, and the escaping error is the bracket one.** Reproduced here, and gated on the
//!    error's IDENTITY — because on that input class no value key exists at all.
//! 2. **THOSE 46 ARE TWO ARMS, NOT ONE, AND THE SPLIT IS 38 / 8.** § 5.16 recorded them as one
//!    number. They are not: 38 are the equilibrium-gas refusal, and **8 are `inverse: root not
//!    bracketed` out of the HPC ideal-temperature inversion** — a call site slice L measured at
//!    **0** and left panicking. It is a contiguous band of trial flows (`m_lp` 1.739…2.019) where
//!    the HP face has run past `psi < 0`, the ideal enthalpy rise goes negative and the target
//!    leaves the 150–4000 K table. So [`crate::gas::Gas::try_t_from_h_c`] exists as of this slice
//!    — *fallibility is per CALL SITE, not per function* — and the two arms get **two counters**,
//!    never a summed 46 (*a registered SUM is not a gated SPLIT*).
//! 3. **`int(round(s_end/ds))` LANDS ON AN EXACT `.5` TIE HERE, AND THE TWO LANGUAGES' `round`
//!    DISAGREE.** Rung 43's ramps run `s_end = r + 8.0` at `ds = 0.02`, and at `r = 0.25` that is
//!    `8.25/0.02 = 412.5` exactly (`825/2` as a `Fraction`). Python's zero-digit `round` is
//!    half-to-EVEN → 412; Rust's `f64::round` is half-AWAY-FROM-ZERO → 413. **21 of 162 marches
//!    land on a tie and not one is inexact**, so the naive test for this hazard reports agreement
//!    on precisely the cases that matter. [`round_ties_even`](f64::round_ties_even) is INHERITED
//!    from `two_spool_transient.rs:862` rather than re-decided — a spelling slice Q chose when
//!    nothing could see it, load-bearing 47 rungs later. And § 5.16 measured that every reported
//!    value is BLIND to the extra step (`Tt4_peak`, `X`, `E_temp_H`, `E_temp_L`, `complete`
//!    bit-identical in 12 of 12 cells, because `s_settle = 8.0` makes 95 %+ of a march settling
//!    tail and the peak is attained at point 13 of 412): **only the trajectory LENGTH sees it**,
//!    which is why the length is an oracle key.
//! 4. **NO PHASE-6 GATE ARMS A SINGLE LIMITER KEYWORD.** `der` builds zero caps **227 856 times
//!    out of 227 856** — the only keyword either suite passes to any fuel entry point is
//!    `freeze="lp"`. So the three set-point solves, both dispatch twins and all three helper
//!    classes are unreached by every rung gate in this phase, and gating their counters against
//!    zero would be vacuous by construction. The **armed smoke sections are the coverage**, and
//!    they are sized deliberately rather than as an afterthought.
//! 5. **`equilibrium_fuel`'s SHIPPED NOISE-FLOOR CLAIM IS A CPG STATEMENT.** Its comment says the
//!    non-equilibrium gases' "residual floor is ~1e-14 — comfortably under the absolute
//!    `_EQ_TOL`". On the TPG gases it is **9.3e-13 — 65× worse and 8 % under the bar**, and the
//!    exit pass count swings **16-fold** between interpreters (`thermally_perfect` at
//!    `Tt4 = 1400`: PyPy 2 passes, CPython 33). The conclusion survives — no cell exhausts
//!    `EQ_MAX` — its stated reason does not. That makes a TPG `equilibrium_fuel` key the sharpest
//!    single DETECTOR in the slice and simultaneously unusable inside a CPython bit-equality bar.
//! 6. **`collapse_exponent`'s ARGMIN SITS ON A PLATEAU.** The score is piecewise-constant in `q`
//!    (13 distinct bin-fill shapes over 25 samples) and every currency's minimum is attained by
//!    TWO adjacent `q` at a gap of exactly `0.000e+00`. Python's `min` keeps the FIRST of equals;
//!    [`Iterator::min_by`] also keeps the first — but `max_by` keeps the LAST and is one keystroke
//!    away, and **rung 43's own gate 9 cannot tell them apart** (it asserts an ordering a
//!    last-of-equals tie-break satisfies just as well). Only the value dump can.
//!
//! # `_close_fuel` IS NOT `r40_try_close`, and porting it by analogy produces a wrong port
//!
//! Every one of these differs from the rung-40 closure it visually resembles:
//!
//! | | rung 40's `_close` | rung 43's [`try_close_fuel`](FuelTransientCore::try_close_fuel) |
//! |---|---|---|
//! | high wall | `min(2.5, phi_max*n_L)` — two arms | `min(2.5, phi_max*n_L, hi0)` — **three, all live** |
//! | low wall | literal `0.02` | `max(lo0, 0.02)`, `lo0` from `f_cap = 0.065` |
//! | march-in step | `0.02` | **`0.04`** |
//! | the scan | breaks at the FIRST success | keeps the LAST negative, breaks on the first positive AFTER one |
//! | the high end | `g(hi)` pre-evaluated OUTSIDE the try | produced INSIDE the scan |
//! | the bracket check | `glo < 0 && 0 < ghi` | `lo is not None and hi is not None` — **no sign filter** |
//!
//! …and rung 35's single-spool fuel closure is a third thing again (`f_cap = 0.05`, no floor).
//! **Four Illinois tolerances live at four call sites** — `1e-12` in the closure, `1e-9` in
//! [`try_topping_fuel`](FuelTransientCore::try_topping_fuel), `1e-13` in BOTH
//! [`try_sched_fuel`](FuelTransientCore::try_sched_fuel) and
//! [`try_surge_fuel`](FuelTransientCore::try_surge_fuel) — and none of them is `EQ_TOL`.
//!
//! The marcher differs too: rung 40's breaks before the final RK stages and floors both speeds at
//! `0.2`; **rung 43's does NEITHER**. The wasted final stages are invisible in the returned
//! trajectory and visible in the census, which is why the census is dumped.
//!
//! # The three marchers compose the min-select three DIFFERENT ways
//!
//! * [`integrate_fuel`](FuelTransientCore::integrate_fuel)'s own `der` collects caps into a LIST,
//!   filters `c < mf`, takes the `min`, and **re-solves the instant**;
//! * [`integrate_fuel_lagged`](FuelTransientCore::integrate_fuel_lagged) min-selects
//!   SEQUENTIALLY with no filter, and its `faded` references **`mf_sched`** where the bare one
//!   references the already-clipped `mf` — two different functions spelled with the same name;
//! * [`integrate_fuel_asym`](FuelTransientCore::integrate_fuel_asym) builds `required` from
//!   UNFADED caps off `mf_sched`, and tests the unlagged redline on the CLIPPED `mf`.
//!
//! # Float-identity branches that are branches on purpose
//!
//! [`faded`] returns the cap ITSELF at `w >= 1.0`; the two set-point solves return `mf_sched`
//! ITSELF when dormant; [`release_weight`] short-circuits to exactly `1.0` or `0.0`. § 5.16
//! measured what that buys: the dormant `c < mf` comparison is an **EXACT structural zero** — a
//! float compared with itself — so it is unflippable on any interpreter, where the two live
//! predicates carry margins of `5.0e-3` and `1.4e-2` against a `~1e-10` drift. *A reduce
//! discipline paying off as numerical robustness.*

// The turbine / nozzle / thrust tail is rung 40's and is reached through its hook table, so
// this module imports the burner-side pieces ONLY -- `Nozzle` appears nowhere below.
use crate::components::choked_mfp;
use crate::engine::FlightCondition;
use crate::gas::{powp, Abort, Gas};
use crate::map::ComponentMap;
use crate::matcher::Branch;
use crate::spool::{try_illinois, SpoolTransient, ILLINOIS_MAXIT};
use crate::two_spool::{Spool, TwoSpoolEngine};
use crate::two_spool_transient::{
    CloseState, Instant2, TwoSpoolTransientCore, TwoSpoolTransientHooks,
};

use std::cell::Cell;

// ---------------------------------------------------------------------------------------------
// Counters — the dead arms this slice ships are gated against ZERO rather than left absent, and
// the ones a wrapper CANNOT see live here from day one
// ---------------------------------------------------------------------------------------------

thread_local! {
    static CLOSE_CALLS: Cell<u64> = const { Cell::new(0) };
    static CLOSE_BRACKET_FAILS: Cell<u64> = const { Cell::new(0) };
    static CLOSE_G_EVALS: Cell<u64> = const { Cell::new(0) };
    static MARCH_IN_ADVANCES: Cell<u64> = const { Cell::new(0) };
    static MARCH_IN_REFUSAL: Cell<u64> = const { Cell::new(0) };
    static MARCH_IN_INVERSE: Cell<u64> = const { Cell::new(0) };
    static MARCH_IN_OFFMAP: Cell<u64> = const { Cell::new(0) };
    static MARCH_IN_OTHER: Cell<u64> = const { Cell::new(0) };
    static LO_FLOOR_HITS: Cell<u64> = const { Cell::new(0) };
    static HI_WALL_LITERAL: Cell<u64> = const { Cell::new(0) };
    static HI_WALL_MAP: Cell<u64> = const { Cell::new(0) };
    static HI_WALL_HI0: Cell<u64> = const { Cell::new(0) };
    static INSTANT_CALLS: Cell<u64> = const { Cell::new(0) };
    static EQ_CALLS: Cell<u64> = const { Cell::new(0) };
    static EQ_PASSES: Cell<u64> = const { Cell::new(0) };
    static EQ_DAMPED: Cell<u64> = const { Cell::new(0) };
    static EQ_DAMP_FLOOR: Cell<u64> = const { Cell::new(0) };
    static EQ_EXHAUSTED: Cell<u64> = const { Cell::new(0) };
    static MARCH_CALLS: Cell<u64> = const { Cell::new(0) };
    static MARCH_POINTS: Cell<u64> = const { Cell::new(0) };
    static MARCH_BREAK_K1: Cell<u64> = const { Cell::new(0) };
    static MARCH_BREAK_RK: Cell<u64> = const { Cell::new(0) };
    static DER_CALLS: Cell<u64> = const { Cell::new(0) };
    static DER_CAPS_0: Cell<u64> = const { Cell::new(0) };
    static DER_CAPS_1: Cell<u64> = const { Cell::new(0) };
    static DER_CAPS_2: Cell<u64> = const { Cell::new(0) };
    static DER_CAPS_3: Cell<u64> = const { Cell::new(0) };
    static DER_RESOLVES: Cell<u64> = const { Cell::new(0) };
    static TOPPING_CALLS: Cell<u64> = const { Cell::new(0) };
    static TOPPING_SKIPS: Cell<u64> = const { Cell::new(0) };
    static TOPPING_EXHAUSTED: Cell<u64> = const { Cell::new(0) };
    static SCHED_CALLS: Cell<u64> = const { Cell::new(0) };
    static SCHED_DORMANT: Cell<u64> = const { Cell::new(0) };
    static SCHED_SKIPS: Cell<u64> = const { Cell::new(0) };
    static SURGE_CALLS: Cell<u64> = const { Cell::new(0) };
    static SURGE_DORMANT: Cell<u64> = const { Cell::new(0) };
    static SURGE_SKIPS: Cell<u64> = const { Cell::new(0) };
    static RW_CALLS: Cell<u64> = const { Cell::new(0) };
    static RW_ONE: Cell<u64> = const { Cell::new(0) };
    static RW_INTERIOR: Cell<u64> = const { Cell::new(0) };
    static RW_ZERO: Cell<u64> = const { Cell::new(0) };
    static MF_FLOOR_HITS: Cell<u64> = const { Cell::new(0) };
    static INTERP_LOW: Cell<u64> = const { Cell::new(0) };
    static INTERP_MID: Cell<u64> = const { Cell::new(0) };
    static INTERP_HIGH: Cell<u64> = const { Cell::new(0) };
    static INTERP_FALLTHROUGH: Cell<u64> = const { Cell::new(0) };
    static CAP_LOW: Cell<u64> = const { Cell::new(0) };
    static CAP_MID: Cell<u64> = const { Cell::new(0) };
    static CAP_HIGH: Cell<u64> = const { Cell::new(0) };
    static CAP_FALLTHROUGH: Cell<u64> = const { Cell::new(0) };
    static COLLAPSE_NAN: Cell<u64> = const { Cell::new(0) };
    static COLLAPSE_EMPTY: Cell<u64> = const { Cell::new(0) };
    static COLLAPSE_TIES: Cell<u64> = const { Cell::new(0) };
}

/// This module's counters. Read and RESET by [`counters::take`] — same single-consumer caveat as
/// [`crate::two_spool_transient::counters::take`]: they are thread-locals, so two `#[test]`s in
/// one binary would steal each other's tallies and the failure would read as physics rather than
/// harness. **One `#[test]` per binary.**
pub mod counters {
    use super::*;

    /// Every count this module keeps.
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct Census {
        pub close_calls: u64,
        pub close_bracket_fails: u64,
        /// Every `g` evaluation inside the closure's scan, successful or not.
        pub close_g_evals: u64,
        /// Advances of the low-wall march-in — i.e. `g` failures the scan SWALLOWED.
        ///
        /// **DEAD on every CPG grid (0 of 227 889) and HOT on the one input class rung 43's own
        /// suite names** (46 on `Gas::reacting_equilibrium`). A dead arm and a hot loop in the
        /// same file: the census is worthless without the grid attached to it.
        pub march_in_advances: u64,
        /// …of which the equilibrium-gas REFUSAL. Measured 38 of 46.
        pub march_in_refusal: u64,
        /// …of which `inverse: root not bracketed`. Measured **8 of 46**, all from the HPC ideal
        /// temperature. § 5.16 recorded the 46 as one number; it is two arms, and in Rust they
        /// arrive from two different files.
        pub march_in_inverse: u64,
        /// …of which the non-real / off-map guard. **0 on every grid measured so far**, including
        /// the equilibrium-gas one — kept apart from the other two because Python reaches it by
        /// returning a COMPLEX where Rust reaches it by returning NaN.
        pub march_in_offmap: u64,
        /// …anything else. A non-zero here means a fourth arm nobody has classified.
        pub march_in_other: u64,
        /// `max(lo0, 0.02)` taking the literal. `lo0` wins **227 889 / 227 889** — DEAD.
        pub lo_floor_hits: u64,
        /// The high wall's three arms. Measured **24 033 / 200 193 / 3 663** — all three LIVE,
        /// and gated as a three-way split whose total is the call count, never as a sum.
        pub hi_wall_literal: u64,
        pub hi_wall_map: u64,
        pub hi_wall_hi0: u64,
        pub instant_calls: u64,
        pub eq_calls: u64,
        /// Newton passes summed over every `equilibrium_fuel` call — probe 3's 16-fold
        /// interpreter amplifier, and the sharpest single detector in the slice.
        pub eq_passes: u64,
        /// `min(1.0, 0.25/max(...))` taking the DAMPER. Measured **0 of 8 steps** — DEAD.
        pub eq_damped: u64,
        /// …and its `1e-30` floor. Measured **0** — DEAD.
        pub eq_damp_floor: u64,
        /// `_EQ_MAX` exhausted → raise. Measured **0** — DEAD.
        pub eq_exhausted: u64,
        pub march_calls: u64,
        pub march_points: u64,
        /// Both truncation arms. Measured **0 truncations in 162 marches** — DEAD, and gated
        /// against zero rather than left absent.
        pub march_break_k1: u64,
        pub march_break_rk: u64,
        pub der_calls: u64,
        /// How many caps `der` built, by count. Measured **`{0: 227 856}`** on both suites' full
        /// grids: NO phase-6 gate arms a single limiter keyword, so the contested-`min` question
        /// belongs to slices T and U and gating these against zero here would be vacuous. The
        /// armed smoke sections are what makes them non-zero.
        pub der_caps_0: u64,
        pub der_caps_1: u64,
        pub der_caps_2: u64,
        pub der_caps_3: u64,
        /// `der` re-solving the instant because a cap bound.
        pub der_resolves: u64,
        pub topping_calls: u64,
        /// `_topping_fuel`'s halving loop swallowing a closure failure.
        pub topping_skips: u64,
        /// …and that loop running out without the residual ever going negative.
        pub topping_exhausted: u64,
        pub sched_calls: u64,
        /// The leg returning `mf_sched` ITSELF — the float-identity dormant branch.
        pub sched_dormant: u64,
        pub sched_skips: u64,
        pub surge_calls: u64,
        pub surge_dormant: u64,
        pub surge_skips: u64,
        /// [`release_weight`] calls and its three arms. Measured 84/0/0 in six of the nine armed
        /// cases; rung 51's fade is the ONLY case with all three live.
        pub rw_calls: u64,
        pub rw_one: u64,
        pub rw_interior: u64,
        pub rw_zero: u64,
        /// `max(1e-9, mf_sched - g)` taking the literal — exists in the two dispatch twins ONLY.
        pub mf_floor_hits: u64,
        /// [`interp`]'s three arms and its fall-through `return ys[-1]`. Measured low **12** /
        /// interior **2 420** / high **2 752**, fall-through **0**.
        pub interp_low: u64,
        pub interp_mid: u64,
        pub interp_high: u64,
        pub interp_fallthrough: u64,
        /// [`AccelSchedule::cap`]'s three arms and its fall-through — a SEPARATE function from
        /// [`interp`] in Python and kept separate here, because their fall-throughs differ (one
        /// is an initializer, the other a return).
        pub cap_low: u64,
        pub cap_mid: u64,
        pub cap_high: u64,
        pub cap_fallthrough: u64,
        /// [`collapse_exponent`]'s NaN guard firing. Measured **0 of 75** — DEAD, and spelled.
        pub collapse_nan: u64,
        /// …and its `if sp else nan` fall-back. Measured **0** — DEAD, and spelled.
        pub collapse_empty: u64,
        /// Exponents whose score EXACTLY equalled the incumbent minimum. Measured non-zero: every
        /// currency's argmin is a TIE of two adjacent `q` at a gap of `0.000e+00`, and rung 43's
        /// own gate 9 is blind to which one wins.
        pub collapse_ties: u64,
    }

    pub fn take() -> Census {
        let c = Census {
            close_calls: CLOSE_CALLS.with(|x| x.get()),
            close_bracket_fails: CLOSE_BRACKET_FAILS.with(|x| x.get()),
            close_g_evals: CLOSE_G_EVALS.with(|x| x.get()),
            march_in_advances: MARCH_IN_ADVANCES.with(|x| x.get()),
            march_in_refusal: MARCH_IN_REFUSAL.with(|x| x.get()),
            march_in_inverse: MARCH_IN_INVERSE.with(|x| x.get()),
            march_in_offmap: MARCH_IN_OFFMAP.with(|x| x.get()),
            march_in_other: MARCH_IN_OTHER.with(|x| x.get()),
            lo_floor_hits: LO_FLOOR_HITS.with(|x| x.get()),
            hi_wall_literal: HI_WALL_LITERAL.with(|x| x.get()),
            hi_wall_map: HI_WALL_MAP.with(|x| x.get()),
            hi_wall_hi0: HI_WALL_HI0.with(|x| x.get()),
            instant_calls: INSTANT_CALLS.with(|x| x.get()),
            eq_calls: EQ_CALLS.with(|x| x.get()),
            eq_passes: EQ_PASSES.with(|x| x.get()),
            eq_damped: EQ_DAMPED.with(|x| x.get()),
            eq_damp_floor: EQ_DAMP_FLOOR.with(|x| x.get()),
            eq_exhausted: EQ_EXHAUSTED.with(|x| x.get()),
            march_calls: MARCH_CALLS.with(|x| x.get()),
            march_points: MARCH_POINTS.with(|x| x.get()),
            march_break_k1: MARCH_BREAK_K1.with(|x| x.get()),
            march_break_rk: MARCH_BREAK_RK.with(|x| x.get()),
            der_calls: DER_CALLS.with(|x| x.get()),
            der_caps_0: DER_CAPS_0.with(|x| x.get()),
            der_caps_1: DER_CAPS_1.with(|x| x.get()),
            der_caps_2: DER_CAPS_2.with(|x| x.get()),
            der_caps_3: DER_CAPS_3.with(|x| x.get()),
            der_resolves: DER_RESOLVES.with(|x| x.get()),
            topping_calls: TOPPING_CALLS.with(|x| x.get()),
            topping_skips: TOPPING_SKIPS.with(|x| x.get()),
            topping_exhausted: TOPPING_EXHAUSTED.with(|x| x.get()),
            sched_calls: SCHED_CALLS.with(|x| x.get()),
            sched_dormant: SCHED_DORMANT.with(|x| x.get()),
            sched_skips: SCHED_SKIPS.with(|x| x.get()),
            surge_calls: SURGE_CALLS.with(|x| x.get()),
            surge_dormant: SURGE_DORMANT.with(|x| x.get()),
            surge_skips: SURGE_SKIPS.with(|x| x.get()),
            rw_calls: RW_CALLS.with(|x| x.get()),
            rw_one: RW_ONE.with(|x| x.get()),
            rw_interior: RW_INTERIOR.with(|x| x.get()),
            rw_zero: RW_ZERO.with(|x| x.get()),
            mf_floor_hits: MF_FLOOR_HITS.with(|x| x.get()),
            interp_low: INTERP_LOW.with(|x| x.get()),
            interp_mid: INTERP_MID.with(|x| x.get()),
            interp_high: INTERP_HIGH.with(|x| x.get()),
            interp_fallthrough: INTERP_FALLTHROUGH.with(|x| x.get()),
            cap_low: CAP_LOW.with(|x| x.get()),
            cap_mid: CAP_MID.with(|x| x.get()),
            cap_high: CAP_HIGH.with(|x| x.get()),
            cap_fallthrough: CAP_FALLTHROUGH.with(|x| x.get()),
            collapse_nan: COLLAPSE_NAN.with(|x| x.get()),
            collapse_empty: COLLAPSE_EMPTY.with(|x| x.get()),
            collapse_ties: COLLAPSE_TIES.with(|x| x.get()),
        };
        reset();
        c
    }

    pub fn reset() {
        for c in [
            &CLOSE_CALLS, &CLOSE_BRACKET_FAILS, &CLOSE_G_EVALS, &MARCH_IN_ADVANCES,
            &MARCH_IN_REFUSAL, &MARCH_IN_INVERSE, &MARCH_IN_OFFMAP, &MARCH_IN_OTHER,
            &LO_FLOOR_HITS, &HI_WALL_LITERAL, &HI_WALL_MAP, &HI_WALL_HI0, &INSTANT_CALLS,
            &EQ_CALLS, &EQ_PASSES, &EQ_DAMPED, &EQ_DAMP_FLOOR, &EQ_EXHAUSTED, &MARCH_CALLS,
            &MARCH_POINTS, &MARCH_BREAK_K1, &MARCH_BREAK_RK, &DER_CALLS, &DER_CAPS_0,
            &DER_CAPS_1, &DER_CAPS_2, &DER_CAPS_3, &DER_RESOLVES, &TOPPING_CALLS,
            &TOPPING_SKIPS, &TOPPING_EXHAUSTED, &SCHED_CALLS, &SCHED_DORMANT, &SCHED_SKIPS,
            &SURGE_CALLS, &SURGE_DORMANT, &SURGE_SKIPS, &RW_CALLS, &RW_ONE, &RW_INTERIOR,
            &RW_ZERO, &MF_FLOOR_HITS, &INTERP_LOW, &INTERP_MID, &INTERP_HIGH,
            &INTERP_FALLTHROUGH, &CAP_LOW, &CAP_MID, &CAP_HIGH, &CAP_FALLTHROUGH,
            &COLLAPSE_NAN, &COLLAPSE_EMPTY, &COLLAPSE_TIES,
        ] {
            c.with(|x| x.set(0));
        }
    }
}

fn bump(c: &'static std::thread::LocalKey<Cell<u64>>) {
    c.with(|x| x.set(x.get() + 1));
}

// ---------------------------------------------------------------------------------------------
// The error IDENTITY — because on the refusal path no VALUE key exists
// ---------------------------------------------------------------------------------------------

/// Which arm produced an [`Abort`] on the fuel path.
///
/// Python distinguishes these by MESSAGE inside one `except AssertionError`, and so does this —
/// the classification is the probe's, not a new taxonomy. It exists because § 5.16 prediction 8 is
/// gated on the error's identity: reaching the refusal through an ordinary entry point yields the
/// BRACKET error, and no value key can witness a call that returns no value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FuelAbort {
    /// `try_tt4_from_f` on an equilibrium gas — the refusal itself.
    Refusal,
    /// `inverse: root not bracketed`, out of the HPC ideal-temperature inversion.
    InverseBracket,
    /// The non-real / off-map guard inside `g`.
    OffMap,
    /// The closure's own "does not bracket" assert — what actually ESCAPES.
    Bracket,
    Other,
}

/// Classify an [`Abort`] by the same substrings Python's probe matches on.
pub fn classify(e: &Abort) -> FuelAbort {
    let s = &e.0;
    if s.contains("non-equilibrium") {
        FuelAbort::Refusal
    } else if s.contains("inverse: root not bracketed") {
        FuelAbort::InverseBracket
    } else if s.contains("off-map compressor trial") {
        FuelAbort::OffMap
    } else if s.contains("does not bracket") {
        FuelAbort::Bracket
    } else {
        FuelAbort::Other
    }
}

// ---------------------------------------------------------------------------------------------
// The three helper records, and the free function
// ---------------------------------------------------------------------------------------------

/// RUNG 48. The `Wf/pt3` ACCELERATION FUEL SCHEDULE — the FEEDFORWARD min-select leg.
///
/// ```text
/// Wf  <=  (1 + margin) * kappa_ss(n_H) * pt3
/// ```
///
/// `kappa_ss(n_H) = (Wf/pt3)` read off the plant's OWN steady running line, so the schedule SHAPE
/// is DERIVED and the whole imposition is the one scalar `margin`. Build it with
/// [`FuelTransientCore::accel_schedule`].
///
/// It is EARLY-acting where the rung-46/47 governor is LATE: the governor is feedback on a
/// CONSEQUENCE and cannot fire until `Tt4` reaches the redline, near the END of a ramp; this
/// watches the INPUT, so `Wf` steps up immediately while `pt3` can only rise as the spools spin up.
#[derive(Clone, Debug, PartialEq)]
pub struct AccelSchedule {
    pub margin: f64,
    /// Abscissa: corrected HP speed on the steady running line.
    pub n_h: Vec<f64>,
    /// `kappa_ss(n_H) = (Wf/pt3)` there.
    pub kappa: Vec<f64>,
}

impl AccelSchedule {
    /// The fuel cap at the current `(n_H, pt3)`: linear interpolation on the derived table,
    /// clamped at both ends.
    ///
    /// **A SEPARATE FUNCTION FROM [`interp`], IN PYTHON AND HERE.** They differ where it matters:
    /// this one seeds `k = ys[-1]` and OVERWRITES it inside the loop, so its fall-through is an
    /// INITIALIZER that the final multiply always consumes; `interp`'s is a `return`. Factoring
    /// the two together would be exactly the *deliberate duplication* the port is told not to
    /// remove.
    pub fn cap(&self, n_h: f64, pt3: f64) -> f64 {
        let (xs, ys) = (&self.n_h, &self.kappa);
        let k = if n_h <= xs[0] {
            bump(&CAP_LOW);
            ys[0]
        } else if n_h >= xs[xs.len() - 1] {
            bump(&CAP_HIGH);
            ys[ys.len() - 1]
        } else {
            let mut k = ys[ys.len() - 1];
            let mut hit = false;
            for i in 0..xs.len() - 1 {
                if xs[i] <= n_h && n_h <= xs[i + 1] {
                    let t = (n_h - xs[i]) / (xs[i + 1] - xs[i]);
                    k = ys[i] + t * (ys[i + 1] - ys[i]);
                    hit = true;
                    break;
                }
            }
            if hit {
                bump(&CAP_MID);
            } else {
                bump(&CAP_FALLTHROUGH);
            }
            k
        };
        (1.0 + self.margin) * k * pt3
    }
}

/// RUNG 49. The `phi` / SURGE-MARGIN FEEDBACK limiter — the min-select leg that watches the
/// PROTECTED variable itself.
///
/// ```text
/// Wf  <=  the fuel that holds   phi_spool >= phi_lim
/// ```
///
/// `docs/both-edges-limiter-negative.md` closed the whole `pt3`-filter family with one fact —
/// `pt3`, `Wf`, `n` and every filter of them rise MONOTONICALLY through a ramp, so such a
/// limiter's release edge is structurally POST-ramp. It named the one escape: the only signals
/// with a turnover UPSTREAM of a surge minimum are the surge variables themselves. `phi` has its
/// minimum inside the ramp BY DEFINITION, so a `phi` floor DOES close inside it.
///
/// `phi_lim` is the SAME disclaimed constant the ladder has carried since rung 36 — every relief
/// MAGNITUDE is disclaimed; the SIGNS, the ORDERING and the CROSSING are the claims.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurgeLimiter {
    /// WHICH spool's `phi` is floored.
    pub spool: Spool,
    /// The floor, in the map's own flow-coefficient units.
    pub phi_lim: f64,
}

impl SurgeLimiter {
    pub fn new(spool: Spool, phi_lim: f64) -> Self {
        assert!(phi_lim > 0.0, "rung-49 phi floor is a flow coefficient");
        SurgeLimiter { spool, phi_lim }
    }

    /// `phi_lim = (1+sm)*phi_surge` off the map's OWN imposed surge line — rung 36/41's constant,
    /// not a new one. The magnitude rides on that disclaimed `phi_surge`.
    pub fn from_margin(cmap: &ComponentMap, spool: Spool, sm: f64) -> Self {
        assert!(cmap.phi_surge > 0.0,
                "rung-49 from_margin needs a surge line: build the map with .with_phi_surge(.)");
        assert!(sm >= 0.0, "the rung-49 floor sits AT or ABOVE the surge line");
        Self::new(spool, (1.0 + sm) * cmap.phi_surge)
    }

    /// Python's `key()` — which `phi` of an INSTANT this leg reads. The SOLVE side: the
    /// [`try_surge_fuel`](FuelTransientCore::try_surge_fuel) bracket evaluates this at each trial
    /// fuel.
    pub fn read(&self, i: &FuelInstant) -> f64 {
        match self.spool {
            Spool::Lp => i.base.close.phi_lp,
            Spool::Hp => i.base.close.phi_hp,
        }
    }

    /// The same `key()`, off a MARCHED POINT — the READ side, which
    /// [`surge_relief`](FuelTransientCore::surge_relief)'s `hold_err` needs.
    ///
    /// **PYTHON HAS ONE FUNCTION HERE AND RUST NEEDS TWO, and that is a typing fact and not a
    /// duplication.** `surge.key()` returns the STRING `"phi_lp"` / `"phi_hp"`, which indexes an
    /// instant dict and a marched-point dict indifferently. Rust's [`FuelInstant`] and
    /// [`FuelPoint`] are different types carrying the same two floats, so the one Python function
    /// splits into two accessors that must agree. They are named apart (`read` / `read_point`)
    /// rather than overloaded, so a call site says which side of the leg it is on.
    pub fn read_point(&self, p: &FuelPoint) -> f64 {
        match self.spool {
            Spool::Lp => p.phi_lp,
            Spool::Hp => p.phi_hp,
        }
    }
}

/// RUNG 51. The min-select leg's AUTHORITY at march coordinate `s`.
///
/// ```text
/// w(s) = clamp( (s_off + tau_rel - s) / tau_rel , 0, 1 )
/// ```
///
/// A PURE FUNCTION OF `s` — no state, no latch — so rung 50's RK4 argument carries verbatim (the
/// march is already non-autonomous through the fuel schedule). A boolean LATCH would flip between
/// k1 and k4 and silently destroy the integrator's order.
///
/// **`tau_rel` FALSY MEANS `None` OR `0.0`, and `is_none()` is the WRONG spelling.** Python's
/// `if not tau_rel` is true for both, and that short-circuit returns exactly `1.0` or `0.0` —
/// the identical branch rung 50 takes, which is what makes the reduce bit-for-bit rather than
/// equal-to-tolerance.
pub fn release_weight(s: f64, s_off: Option<f64>, tau_rel: Option<f64>) -> f64 {
    bump(&RW_CALLS);
    let w = match s_off {
        None => 1.0,
        Some(off) => {
            if tau_rel.is_none_or(|t| t == 0.0) {
                if s < off {
                    1.0
                } else {
                    0.0
                }
            } else {
                let t = tau_rel.expect("checked");
                1.0f64.min(0.0f64.max((off + t - s) / t))
            }
        }
    };
    if w >= 1.0 {
        bump(&RW_ONE);
    } else if w == 0.0 {
        bump(&RW_ZERO);
    } else {
        bump(&RW_INTERIOR);
    }
    w
}

/// RUNG 52. The FAST-ATTACK / SLOW-RELEASE lag on a min-select leg — the physically-realisable
/// limiter rungs 50/51 imitated with forced edges.
///
/// ```text
/// required(nu, s) = max(0, mf_sched - leg_cap(nu, mf_sched))
/// dg/ds = (required - g) / tau_att      if required > g       (fast ATTACK)
///         (required - g) / tau_rel      if required < g       (slow RELEASE)
/// mf    = mf_sched - g
/// ```
///
/// `g` (the clip AMOUNT, not the valve position) is a THIRD STATE. **It PINS ITS OWN TRIGGER**:
/// `tau_rel` is never READ while `required > g`, so the entire march up to the first crossing is
/// bit-identical across a `tau_rel` sweep — the property rung 50 had to FORCE with `s_off`. Both
/// branches carry the same vanishing numerator, so the right-hand side has a KINK and not a jump,
/// which is why this form is RK4-legal where rung 51's sketched `max(g, required)` was not.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AsymmetricLag {
    pub tau_att: f64,
    pub tau_rel: f64,
}

impl AsymmetricLag {
    pub fn new(tau_att: f64, tau_rel: f64) -> Self {
        assert!(tau_att > 0.0 && tau_rel > 0.0,
                "rung-52 lag constants are time constants on the march coordinate; the \
                 instantaneous limit is rung 49 (lag=None), not tau=0.");
        AsymmetricLag { tau_att, tau_rel }
    }

    /// The active constant — CONTINUOUS at the switch.
    pub fn tau(&self, required: f64, g: f64) -> f64 {
        if required > g {
            self.tau_att
        } else {
            self.tau_rel
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The state records
// ---------------------------------------------------------------------------------------------

/// The flow closed at `(nu_L, nu_H, mdot_fuel)` with the burner run FORWARD — Python's
/// `_close_fuel` return dict, whose **23** keys are reproduced field-for-field.
///
/// **23, NOT rung 40's 21.** The inversion adds exactly two: `Tt4`, which is an OUTPUT here rather
/// than the input it is on the `Tt4`-control path, and `mdot_air_face`, the LP-FACE air flow the
/// fuel-air ratio is formed against. Rung 40's closure computes the same face flow as an unnamed
/// LOCAL and returns only the NGV-imposed `mdot4/(1+f)`; rung 43 returns both, because `f` is
/// built from one and the residual from the other. Composing over [`CloseState`] rather than
/// re-declaring 21 fields is what keeps `_instant_tail` reachable through
/// [`TwoSpoolTransientHooks`] — the ONE hook cell this slice dispatches through.
#[derive(Clone, Debug)]
pub struct FuelCloseState {
    pub base: CloseState,
    /// The OUTPUT `Tt4`. `base` carries no `Tt4` at all on the rung-40 path.
    pub tt4: f64,
    /// The LP-FACE air flow — NOT `base.mdot_air`, which is `mdot4/(1+f)`.
    pub mdot_air_face: f64,
}

/// The quasi-steady fuel-control instant — Python's `_instant_fuel` return dict, **45** keys.
///
/// `dict(c)` of the 23-key closure updated with the tail's 23, of which `Tt4` is a duplicate:
/// 23 + 22 = 45, i.e. exactly rung 40's 44 plus `mdot_air_face`. So this composes over
/// [`Instant2`] and carries one extra field, and the dump asserts the count from PYTHON's own
/// `len()` rather than deriving it from 23 + 23.
#[derive(Clone, Debug)]
pub struct FuelInstant {
    pub base: Instant2,
    pub mdot_air_face: f64,
}

/// What a marched point carries BEYOND the fourteen every route emits.
///
/// § 5.16 probe 4 (B) armed all seven limiter keywords for the first time and measured that
/// [`integrate_fuel_asym`](FuelTransientCore::integrate_fuel_asym) returns a **16**-key point
/// where every other route — including rung 47's lagged twin, whose third state `g` is NOT
/// recorded — returns 14. A struct with `Option` fields would let a caller read a field Python
/// would have raised `KeyError` on, so the route is an enum and the dump enumerates keys PER
/// ROUTE.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointExtra {
    /// The bare march, the rung-46…51 arms, and rung 47's lagged twin — 14 keys.
    None,
    /// Rung 52's asymmetric-lag twin — 16 keys.
    Asym { g: f64, required: f64 },
}

/// One instant of a marched FUEL trajectory — Python's per-point dict.
#[derive(Clone, Copy, Debug)]
pub struct FuelPoint {
    pub s: f64,
    pub nu_lp: f64,
    pub nu_hp: f64,
    /// An OUTPUT: it can overshoot the steady value, which is rung 43's whole finding.
    pub tt4: f64,
    pub f: f64,
    pub pi_lpc: f64,
    pub pi_hpc: f64,
    pub phi_lp: f64,
    pub phi_hp: f64,
    pub mdot_air: f64,
    pub sp_thrust: f64,
    pub branch: Branch,
    /// The APPLIED fuel — below `mf_sched` exactly when a limiter leg bound.
    pub mf: f64,
    /// The SCHEDULED fuel.
    pub mf_sched: f64,
    pub extra: PointExtra,
}

impl FuelPoint {
    /// How many keys Python's dict for this route carries — 14 or 16.
    pub fn key_count(&self) -> usize {
        match self.extra {
            PointExtra::None => 14,
            PointExtra::Asym { .. } => 16,
        }
    }
}

/// RUNG 43's `ramp_excursion_fuel` return.
#[derive(Clone, Debug)]
pub struct RampExcursionFuel {
    pub r: f64,
    pub rho: f64,
    pub tt4_peak: f64,
    /// `Tt4_peak - Tt4_hi` — the SPOOL-NEUTRAL currency, because the running-line-referenced ones
    /// are CIRCULAR (they read back whichever spool sits in the denominator).
    pub x: f64,
    pub e_temp_h: f64,
    pub e_temp_l: f64,
    pub complete: bool,
    pub traj: Vec<FuelPoint>,
}

/// RUNG 43's `constant_speed_excursion_fuel` return — the `r → 0` limit, EXACTLY `rho`-free.
#[derive(Clone, Copy, Debug)]
pub struct ConstantSpeedExcursionFuel {
    pub tt4_peak: f64,
    pub e_temp: f64,
    pub e_lp: f64,
    pub e_hp: f64,
    pub f: f64,
}

/// RUNG 43's `freeze_channels` return — THE MECHANISM.
#[derive(Clone, Copy, Debug)]
pub struct FreezeChannels {
    pub both: f64,
    pub lp: f64,
    pub hp: f64,
    /// SIGN / EXISTENCE only: `d_lp` and `d_hp` do NOT sum to the total and are NOT calibrated
    /// weights.
    pub d_lp: f64,
    pub d_hp: f64,
    pub r: f64,
    pub rho: f64,
}

/// RUNG 45's `phi_excursion_fuel` return.
#[derive(Clone, Copy, Debug)]
pub struct PhiExcursionFuel {
    pub ext_lp: f64,
    pub ext_hp: f64,
    pub s_lp: f64,
    pub s_hp: f64,
    pub min_phi_lp: f64,
    pub min_phi_hp: f64,
    pub tt4_peak: f64,
    pub ratio: f64,
    pub npts: usize,
}

/// RUNG 45's `transient_surge_margin_fuel` return.
#[derive(Clone, Copy, Debug)]
pub struct TransientSurgeMarginFuel {
    /// The RAW (reference-free) transient min margin — THE surge object, and the one immune to
    /// the moving-reference currency trap.
    pub margin_min_lp: f64,
    pub margin_min_hp: f64,
    pub steady_min_lp: f64,
    pub steady_min_hp: f64,
    pub min_phi_lp: f64,
    pub min_phi_hp: f64,
    pub crossed_lp: bool,
    pub crossed_hp: bool,
    pub phi_surge_lp: f64,
    pub phi_surge_hp: f64,
    pub npts: usize,
}

/// RUNG 46/47's `topping_relief` return — Python's 14-key dict.
///
/// `tt4_max` and `tau_gov` are echoed back because Python's dict echoes them; a caller that
/// sweeps `tau_gov` reads the knob off the row rather than off its own loop variable.
#[derive(Clone, Copy, Debug)]
pub struct ToppingRelief {
    pub rho: f64,
    pub r: f64,
    pub tt4_max: f64,
    /// `None` is rung 46's INSTANTANEOUS min-select; `Some` is rung 47's lagged governor.
    pub tau_gov: Option<f64>,
    pub tt4_peak_bare: f64,
    pub tt4_peak_top: f64,
    /// `tt4_peak_top - tt4_max`. POSITIVE is rung 47's cost of realism — the lag breaks the hold.
    pub overshoot: f64,
    /// `tt4_peak_top <= tt4_max + 1e-6`. § 5.17 finding 2 measured this decision uncontested by
    /// **5.5e7**: the governor pins the redline to ≤ 1.6e-12 or misses it by ≥ 54.7 K, with
    /// nothing in between, so a flip here is a port defect and never a knife-edge.
    pub held: bool,
    pub min_phi_lp_bare: f64,
    pub min_phi_lp_top: f64,
    pub min_phi_hp_bare: f64,
    pub min_phi_hp_top: f64,
    /// `> 0` ⇔ the topped march's raw min `phi` sits ABOVE (safer than) the bare one.
    ///
    /// **EXACTLY `0.0` at moderate `r`, and that is structural** — § 5.17 finding 3 measured the
    /// two marches bit-identical over their leading points with the LP minimum inside that
    /// prefix, so the subtraction reads one float from itself.
    pub relief_lp: f64,
    pub relief_hp: f64,
}

/// RUNG 47's `topping_command_trace` return — Python's 5-key dict.
#[derive(Clone, Debug)]
pub struct ToppingCommandTrace {
    /// The `(s, mf)` command at each ENGAGED point — where the clip fires, i.e. `Tt4` pinned at
    /// the redline.
    pub engaged: Vec<(f64, f64)>,
    pub n_engaged: usize,
    /// Whether the engaged command rises. **This is what makes a metering-VALVE lag inert**: an
    /// instant-up valve tracks a rising command with no lag, so the topping overshoot must live
    /// in the sensing/limiter LOOP instead.
    pub monotone_nondecreasing: bool,
    pub tt4_max: f64,
    pub r: f64,
}

/// RUNG 48's `schedule_relief` return — Python's 18-key dict.
#[derive(Clone, Copy, Debug)]
pub struct ScheduleRelief {
    pub margin: f64,
    pub r: f64,
    pub rho: f64,
    /// WHEN the leg first engages — **`NaN` when it never does**, which is Python's
    /// `eng[0] if eng else float("nan")`.
    ///
    /// § 5.17 finding 4: that arm is reachable (`margin >= 0.55` at `r = 0.5`) and **dead on every
    /// suite cell** — the lowest any of them drives `n_engaged` is 1 — so the oracle adds a cell
    /// for it rather than inheriting one. The `to_bits()` comparator needs no NaN special case:
    /// PyPy's `float("nan")` and Rust's [`f64::NAN`] are both `7ff8000000000000`, measured.
    pub s_eng: f64,
    pub n_engaged: usize,
    /// WHERE each spool's RAW surge minimum sits on the BARE march — not
    /// [`PhiExcursionFuel::s_lp`], which locates the running-line-REFERENCED extremum.
    pub s_lp_bare: f64,
    pub s_hp_bare: f64,
    pub relief_lp: f64,
    pub relief_hp: f64,
    pub min_phi_lp_bare: f64,
    pub min_phi_lp_lim: f64,
    pub min_phi_hp_bare: f64,
    pub min_phi_hp_lim: f64,
    /// `∫ (schedule − applied) ds`, trapezoid. One of the two keys that EXCLUDE the deflation
    /// "any clip removes fuel and slows the accel, so this is rung 44's ramp-rate lever".
    pub fuel_removed: f64,
    pub tt4_peak_bare: f64,
    pub tt4_peak_lim: f64,
    /// The settled endpoint — the OTHER exclusion key. The crossing is read only where this is
    /// unmoved from `nu_hp_end_bare`.
    pub nu_hp_end: f64,
    pub nu_hp_end_bare: f64,
}

/// RUNG 49's `surge_relief` return — Python's **25**-key dict.
///
/// **A SEPARATE STRUCT FROM [`ScheduleRelief`], AND THE OVERLAP IS WHY.** Fifteen keys are common
/// to rungs 49/50/52's three readers, so the obvious design is one record with `Option` fields.
/// § 5.18 finding 5 measured why that is wrong: the three dicts are **25 / 27 / 34** keys, and
/// they disagree on more than presence. Rung 49 reports `s_min_other` where rungs 50/52 report the
/// PAIR `s_min_lp` / `s_min_hp`; it carries no `ds` and no `margin`; and its `relief_watched` /
/// `relief_other` are plain floats because its `surge` is a required positional, where the other
/// two make them `None`-able. A shared struct would emit phantom keys on one side and renamed ones
/// on the other, and a key-COUNT census would pass on both while comparing nothing — the
/// *documented gate that doesn't exist* family. The **25** is asserted from Python's own `len()`
/// in the step-5 oracle, never derived here.
///
/// The three keys that are rung 49's ALONE beyond that: `hold_err`, `both_edges_inside_ramp` and
/// `Tt4_peak_lim`'s partner `s_min_other`.
#[derive(Clone, Copy, Debug)]
pub struct SurgeRelief {
    pub phi_lim: f64,
    /// WHICH spool the leg watched. Python echoes the string `"lp"` / `"hp"`.
    pub spool: Spool,
    pub r: f64,
    pub rho: f64,
    /// WHEN the floor first engages — **`NaN` when it never does**, Python's
    /// `eng[0] if eng else float("nan")`.
    ///
    /// § 5.18 finding 4: that arm is UNGATED by this slice's cells — the lowest `n_engaged` any
    /// rung-49 floor cell drives is **10** — so the oracle adds a no-engagement cell rather than
    /// inheriting one, exactly as § 5.17 finding 4 did for [`ScheduleRelief::s_eng`].
    pub s_eng: f64,
    /// WHEN it releases — the edge that is the point of the rung, and `NaN` on the same arm.
    pub s_rel: f64,
    pub n_engaged: usize,
    /// `eng` non-empty ∧ `0 < eng[0]` ∧ `eng[-1] < r` — the object
    /// `docs/both-edges-limiter-negative.md` proved no `pt3`-filter limiter can produce.
    ///
    /// **§ 5.18 FINDING 3: THIS BOOLEAN IS DECIDED AT ONE ULP ON ONE MEASURED CELL.** Over rung
    /// 49's eight floor cells the distance `r − s_rel` is `0.06` / `0.16` inside and `−0.10` /
    /// `−0.02` / `−0.42` / `−0.12` / `−0.02` outside — **one grid cell of margin at the tightest,
    /// not orders** — and the eighth (the HP floor `0.8650`) sits at **`−1.11e-16`**. It survives
    /// only because both languages accumulate the march coordinate the same way; see the note at
    /// [`integrate_fuel`](FuelTransientCore::integrate_fuel)'s loop.
    pub both_edges_inside_ramp: bool,
    /// The largest deviation of the WATCHED `phi` from its floor over the engaged window. `0` to
    /// solver tolerance is the SLIDING MODE; a non-zero value would be chatter. Reads the floored
    /// `phi` off a marched POINT — [`SurgeLimiter::read_point`], not [`SurgeLimiter::read`].
    pub hold_err: f64,
    pub s_lp_bare: f64,
    pub s_hp_bare: f64,
    pub relief_lp: f64,
    pub relief_hp: f64,
    /// The WATCHED spool's relief. Definitional under a working set-point solve (`phi_lim − min
    /// phi_bare`), which is why the rung's claims all live on the other one.
    pub relief_watched: f64,
    /// The UNWATCHED spool's relief — **the rung**. NEGATIVE at `r = 0.5`: one clip credits the
    /// spool it watches and DEBITS the other.
    pub relief_other: f64,
    /// WHERE the unwatched spool's raw minimum sits on the LIMITED march — just AFTER `s_rel`,
    /// which is the mechanism.
    pub s_min_other: f64,
    pub min_phi_lp_bare: f64,
    pub min_phi_lp_lim: f64,
    pub min_phi_hp_bare: f64,
    pub min_phi_hp_lim: f64,
    /// `∫ (schedule − applied) ds`, trapezoid — the anti-deflation pair's first half.
    pub fuel_removed: f64,
    pub tt4_peak_bare: f64,
    pub tt4_peak_lim: f64,
    /// The settled endpoint — the other half. The split is read only where this is unmoved from
    /// `nu_hp_end_bare`.
    pub nu_hp_end: f64,
    pub nu_hp_end_bare: f64,
}

/// RUNG 50's `release_relief` return — Python's **27**-key dict, measured with its own `len()`.
///
/// **THE THIRD OF THREE SEPARATE STRUCTS, AND THE DELTAS ARE THE REASON.** § 5.18 finding 5
/// measured rungs 49/50/52's readers at **25 / 27 / 34** keys with only **15** common to all
/// three, so one record with `Option` fields would emit phantom keys on one side and renamed ones
/// on the other while a key-COUNT census passed on both. Against [`SurgeRelief`]:
///
/// * **here and not there** — `s_off`, `tau_rel`, `ds`, `margin`, `deficit_at_release`, and the
///   PAIR `s_min_lp` / `s_min_hp` where rung 49 reports the single `s_min_other`;
/// * **there and not here** — `hold_err`, `both_edges_inside_ramp`, `s_min_other`, and **both**
///   `Tt4_peak_*` keys. This reader has no peak-`Tt4` field at all, so the two injections § 5.18
///   step 1 finding 3 measured ungated on those keys cannot even be spelled against it;
/// * **`Option` where rung 49 is a plain float** — `spool`, `phi_lim`, `relief_watched`,
///   `relief_other` (its `surge` is optional, rung 49's is a required positional) and `margin`
///   (its `accel` is optional too). Both arms are live in shipped gates: gates 6/7 run
///   accel-only, gates 3/4/5 surge-only, so the `Option`s are gated by construction — § 5.18
///   finding 4.
#[derive(Clone, Copy, Debug)]
pub struct ReleaseRelief {
    /// The FORCED release time. `None` is the unforced leg (rung 49 / rung 48).
    pub s_off: Option<f64>,
    /// RUNG 51's release RATE. Lands here COMPLETE (§ 5.18 P6) and is `None` on every rung-50
    /// cell — `release_sweep` does not forward it.
    pub tau_rel: Option<f64>,
    pub r: f64,
    pub rho: f64,
    /// The march step. Rung 49's record carries no such key; gate 10's `ds` convergence is why
    /// this one does.
    pub ds: f64,
    /// WHICH spool the `phi` leg watched — `None` on an accel-only cell.
    pub spool: Option<Spool>,
    pub phi_lim: Option<f64>,
    /// Rung 48's schedule margin — `None` on a surge-only cell.
    pub margin: Option<f64>,
    /// WHEN the leg first engages — `NaN` when it never does.
    ///
    /// § 5.18 finding 4: the no-engagement arm is UNGATED by this slice's cells (the lowest
    /// `n_engaged` any rung-50 `s_off` cell drives is **2**), so the step-5 oracle adds a cell
    /// for it rather than inheriting one.
    pub s_eng: f64,
    /// WHEN it releases — the edge that is the point of the rung. `NaN` on the same arm.
    pub s_rel: f64,
    pub n_engaged: usize,
    /// The instantaneous fractional clip at the LAST engaged point.
    ///
    /// **TWO SENTINELS FOR ONE CONDITION, COPIED NOT REPAIRED.** With nothing engaged this is
    /// `0.0` while `s_eng` / `s_rel` are `NaN` — and `0.0` is a legitimate deficit, so this key
    /// ALONE cannot separate "never engaged" from "engaged with zero deficit". Read `n_engaged`.
    pub deficit_at_release: f64,
    pub s_lp_bare: f64,
    pub s_hp_bare: f64,
    pub relief_lp: f64,
    pub relief_hp: f64,
    /// The WATCHED spool's relief — `None` on an accel-only cell. Rung 49 calls this definitional;
    /// gate 5 measures it going NEGATIVE under an early forced release, which is what BOUNDS rung
    /// 49's identity to the unforced instrument.
    pub relief_watched: Option<f64>,
    /// The UNWATCHED spool's relief — `None` on an accel-only cell.
    pub relief_other: Option<f64>,
    /// WHERE each spool's raw minimum sits on the LIMITED march. **The PAIR is the rung**: the
    /// headline is that BOTH relocate to `s_rel`, which is why rung 49's single `s_min_other`
    /// could not carry it.
    pub s_min_lp: f64,
    pub s_min_hp: f64,
    pub min_phi_lp_bare: f64,
    pub min_phi_lp_lim: f64,
    pub min_phi_hp_bare: f64,
    pub min_phi_hp_lim: f64,
    /// `∫ (schedule − applied) ds`, trapezoid — the anti-deflation pair's first half.
    pub fuel_removed: f64,
    /// The settled endpoint — the other half.
    pub nu_hp_end: f64,
    pub nu_hp_end_bare: f64,
}

/// RUNG 52's `lag_relief` return — Python's **34**-key dict, the widest of the three.
///
/// **THE THIRD RECORD, AND THE ONE WHOSE KEYS ARE RENAMED RATHER THAN ADDED.** § 5.18 finding 5
/// measured rungs 49/50/52's readers at 25 / 27 / **34** keys with only 15 common to all three.
/// Against [`ReleaseRelief`]:
///
/// * **its own, beyond rung 50's**: `tau_att`, `s_cross`, `g_at_cross`, `required_at_cross`,
///   `g_peak`, `n_recross`, `Tt4_peak_bare` / `Tt4_peak_lag`, and the `eps`-indexed
///   `s_eng_<eps>` / `s_rel_<eps>` pairs;
/// * **RENAMED, not absent** — rungs 49/50 report `min_phi_lp_lim` / `min_phi_hp_lim`, this one
///   reports `min_phi_lp_lag` / `min_phi_hp_lag`. The VALUE is the same quantity off the same
///   march. A shared struct would emit one name on both sides and a key-COUNT census would pass
///   while comparing nothing — the *documented gate that doesn't exist* family;
/// * **absent here**: rung 50's `s_off`, `tau_rel`-as-a-forcing (`tau_rel` here is the LAG's
///   release constant, a different object with the same name), `deficit_at_release`, `s_eng`,
///   `s_rel`, `n_engaged`. **A lag never completes, so this reader has no plain `s_rel` at all** —
///   the release edge is DECLARED at each `eps` instead, which is why the pairs are indexed.
#[derive(Clone, Debug)]
pub struct LagRelief {
    pub tau_att: f64,
    /// The LAG's release constant — **not rung 50/51's forced-release rate**, which this reader
    /// refuses to compose with.
    pub tau_rel: f64,
    pub r: f64,
    pub rho: f64,
    pub ds: f64,
    pub spool: Option<Spool>,
    pub phi_lim: Option<f64>,
    pub margin: Option<f64>,
    /// THE CROSSING — the first point at which the leg's own demand falls back through the clip
    /// state. The natural release trigger, and the thing rung 50's `s_off` had to impose.
    /// `NaN` when there is none.
    pub s_cross: f64,
    pub g_at_cross: f64,
    pub required_at_cross: f64,
    /// **THE `armed` SEED IS A DEAD DISTINCTION AND THIS KEY IS ITS ONLY WITNESS.** Python seeds
    /// `armed = None` and guards `if armed is False`, so the FIRST crossing is not counted as a
    /// re-crossing; the natural Rust `let mut armed = false` counts it and puts this one high on
    /// every row. § 5.18 finding 2 measured over six lag cells that the first point with `g > 0`
    /// is ALWAYS still attacking, so both seeds give `1` everywhere and
    /// `test_rung52.py:224`'s `n_recross == 1` passes under the wrong one. The `Option<bool>`
    /// below is therefore load-bearing and untested by any marched cell; the step-5 oracle
    /// carries a MANUFACTURED trajectory for it.
    pub n_recross: usize,
    pub g_peak: f64,
    pub s_lp_bare: f64,
    pub s_hp_bare: f64,
    pub relief_lp: f64,
    pub relief_hp: f64,
    pub relief_watched: Option<f64>,
    pub relief_other: Option<f64>,
    pub s_min_lp: f64,
    pub s_min_hp: f64,
    pub min_phi_lp_bare: f64,
    /// Python's `min_phi_lp_lag` — the `_lag` SUFFIX where rungs 49/50 say `_lim`.
    pub min_phi_lp_lag: f64,
    pub min_phi_hp_bare: f64,
    pub min_phi_hp_lag: f64,
    pub fuel_removed: f64,
    pub tt4_peak_bare: f64,
    pub tt4_peak_lag: f64,
    pub nu_hp_end: f64,
    pub nu_hp_end_bare: f64,
    /// One `(eps, s_eng, s_rel)` triple per requested threshold, in the order given — Python's
    /// f-string keys `s_eng_{e}` / `s_rel_{e}`.
    ///
    /// **BECAUSE AN EXPONENTIAL NEVER COMPLETES, THE RELEASE EDGE IS DECLARED, NOT DETECTED.**
    /// `s_rel_<eps>` is the last point whose fractional clip is at least `eps` — the same currency
    /// `ReleaseRelief::deficit_at_release` uses. Reported at every `eps` so that no verdict rests
    /// on one threshold. § 5.18 finding 3 measured these bars as the SLACKEST-looking and in fact
    /// the tightest in the slice: the nearest value inside `eps = 0.05` is `0.0505` / `0.0522`
    /// (**1–4 %**) and inside `eps = 0.01` is `0.00916` / `0.01020` (**2–8 %**).
    pub eps_edges: Vec<(f64, f64, f64)>,
}

/// RUNG 52's `factorization_grid` return.
///
/// Does rung 49's credit/debit split FACTOR across the two time constants? A real fast-attack /
/// slow-release limiter is DESIGNED on the premise that it does. The answer is ONE WAY only.
#[derive(Clone, Debug)]
pub struct FactorizationGrid {
    pub tau_atts: Vec<f64>,
    pub tau_rels: Vec<f64>,
    /// Row-major, as [`lag_sweep`](FuelTransientCore::lag_sweep) returns them.
    pub rows: Vec<LagRelief>,
    /// `rows` reshaped `[tau_att][tau_rel]`.
    pub grid: Vec<Vec<LagRelief>>,
    /// The additive-separability residual on the DEBIT,
    /// `D(ta,tr) − D(ta,tr0) − D(ta0,tr) + D(ta0,tr0)`.
    pub residual: Vec<Vec<f64>>,
    /// The spread of `relief_watched` across each `tau_att`'s `tau_rel` row — **MACHINE ZERO**,
    /// and § 5.18 P5 asserts it EXACTLY, with no tolerance. `tau_att` owns the credit exactly.
    ///
    /// **A `Vec` OF PAIRS, NOT A MAP, AND THE DIFFERENCE IS REAL.** Python builds a dict keyed on
    /// `tau_att`, so a REPEATED `tau_att` in the input would silently collapse two rows into one
    /// entry; this keeps both, in input order. No shipped cell repeats a `tau_att`, so the two
    /// agree on everything the suite runs — recorded because it is a divergence, not because it
    /// bites.
    pub credit_spread: Vec<(f64, f64)>,
    pub max_residual: f64,
    pub max_main_effect: f64,
    pub r: f64,
    pub ds: f64,
}


/// A marched point's `(g, required)` — rung 52's lag route and nothing else.
///
/// Panics on a [`PointExtra::None`] point, which means the lag leg did not dispatch to
/// [`integrate_fuel_asym`](FuelTransientCore::integrate_fuel_asym): a defect, not a
/// divergence.
pub fn asym_extra(p: &FuelPoint) -> (f64, f64) {
    match p.extra {
        PointExtra::Asym { g, required } => (g, required),
        PointExtra::None => panic!(
            "rung-52 lag_relief marched a trajectory with no `g` / `required`: the lag leg \
             did not dispatch to integrate_fuel_asym"),
    }
}

/// RUNG 52's CROSSING: the index of the first point at which the leg's own demand falls back
/// through the clip state `g`, and how many times the leg RE-crosses after disarming.
///
/// **TWO RULES IN EIGHT LINES, BOTH UNREACHABLE FROM ANY MARCHED CELL — WHICH IS WHY THIS IS
/// A FUNCTION AND NOT A LOOP INSIDE ITS CALLER.** § 5.18 finding 2 registered both and step 4
/// measured both at **zero moved keys over all 18 suite cells**:
///
/// 1. **`armed` is seeded `None`, not `false`.** Python's guard is `if armed is False`, so the
///    FIRST crossing is not counted as a re-crossing. On every marched cell the first point
///    with `g > 0` is still ATTACKING, so it sets `armed = Some(false)` anyway and the two
///    seeds agree — and `test_rung52.py:224`'s `n_recross == 1` passes under the wrong one.
/// 2. **the `g <= 0.0` arm CONTINUES rather than disarming.** An unclipped point does not
///    break an armed run, so folding the guard into one `if / else` is wrong.
///
/// Both are gated on MANUFACTURED trajectories in `release_oracle.rs`, on the same template
/// and for the same reason as [`first_raw_min`]'s tie gate: *a rule no marched cell tests has
/// to be reachable on its own.* Lifting it out is what makes those gates hold the SHIPPED
/// code rather than a re-spelled copy of it in the test file.
pub fn crossing_census(lim: &[FuelPoint]) -> (Option<usize>, usize) {
    let mut cross: Option<usize> = None;
    let mut n_recross = 0usize;
    let mut armed: Option<bool> = None;
    for (i, p) in lim.iter().enumerate() {
        let (g, required) = asym_extra(p);
        if g <= 0.0 {
            continue;
        }
        if required < g {
            if cross.is_none() {
                cross = Some(i);
            }
            if armed == Some(false) {
                n_recross += 1;
            }
            armed = Some(true);
        } else {
            armed = Some(false);
        }
    }
    (cross, n_recross)
}

/// The RAW (reference-free) minimum of one key over a marched trajectory, and the `s` at which it
/// is attained — rung 45's surge object, as [`schedule_relief`](TwoSpoolFuelTransient::
/// schedule_relief) reads it.
///
/// **THE FOLD IS STRICT, AND THAT IS A RULE NO CELL TESTS.** Python's `min(traj, key=…)` returns
/// the FIRST minimum on ties, so `<` is the faithful spelling and `<=` would return the LAST.
/// § 5.17 finding 5 measured that **no suite cell has a tie** — the closest any of them comes is a
/// `1.61e-5` gap to the second-smallest `phi_hp` — so a `<=` here ships past all 31 ported gates
/// undetected. It is therefore gated on a MANUFACTURED trajectory rather than on a marched one:
/// `topping_oracle.rs::the_raw_min_fold_is_first_on_tie`. That gate is the only reason this
/// function is not still nested inside its caller — two marched points cannot be made to bit-tie,
/// so the rule has to be reachable on its own.
pub fn first_raw_min(traj: &[FuelPoint], key: fn(&FuelPoint) -> f64) -> (f64, f64) {
    let mut best = key(&traj[0]);
    let mut at = traj[0].s;
    for p in &traj[1..] {
        if key(p) < best {
            best = key(p);
            at = p.s;
        }
    }
    (best, at)
}

/// Every limiter keyword [`integrate_fuel`](FuelTransientCore::integrate_fuel) accepts — rungs
/// 46 through 52, which are seven arms on ONE method and not seven classes.
///
/// `Default::default()` is the BARE rung-43/45 march, and § 5.16 measured that this is the only
/// configuration any phase-6 gate ever builds.
#[derive(Clone, Debug, Default)]
pub struct FuelLimiters<'a> {
    /// Hold ONE spool's speed at its initial value — the CHANNEL ISOLATION behind rung 43's
    /// finding. `Lp` removes `rho` from the system entirely (it multiplies only the LP ODE): the
    /// `rho → infinity` ceiling.
    pub freeze: Option<Spool>,
    /// RUNG 46. The TIT topping governor's redline.
    pub tt4_max: Option<f64>,
    /// RUNG 47. That governor's response LAG — dispatches to
    /// [`integrate_fuel_lagged`](FuelTransientCore::integrate_fuel_lagged).
    pub tau_gov: Option<f64>,
    /// RUNG 48. The feedforward `Wf/pt3` leg.
    pub accel: Option<&'a AccelSchedule>,
    /// RUNG 49. The `phi` feedback floor.
    pub surge: Option<SurgeLimiter>,
    /// RUNG 50. FORCE the min-select legs to disarm at `s >= s_off`.
    pub s_off: Option<f64>,
    /// RUNG 51. The RATE of that forced release.
    pub tau_rel: Option<f64>,
    /// RUNG 52. The realisable asymmetric lag — dispatches to
    /// [`integrate_fuel_asym`](FuelTransientCore::integrate_fuel_asym).
    pub lag: Option<AsymmetricLag>,
}

// ---------------------------------------------------------------------------------------------
// The object
// ---------------------------------------------------------------------------------------------

/// RUNG 43 / 45. Rung 35's FUEL control on rung 40's two-shaft plant.
///
/// `lp_disabled=True` dispatches to rung 35's [`SpoolTransient`] fuel path — EXACT dispatch, no
/// two-shaft state is ever built. [`crate::two_spool_transient::TwoSpoolTransient`]'s precedent,
/// and the same Python mechanism: `__init__` returns before `super().__init__`.
pub enum TwoSpoolFuelTransient {
    Degenerate(SpoolTransient),
    Full(FuelTransientCore),
}

impl TwoSpoolFuelTransient {
    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap, rho: f64,
    ) -> Self {
        TwoSpoolFuelTransient::Full(FuelTransientCore::new(
            design_engine, flight_design, mdot_design, map_lp, map_hp, rho))
    }

    /// `lp_disabled=True`. Takes a SINGLE-spool design engine and `map_hp`.
    pub fn lp_disabled(
        design_engine: crate::engine::Engine, flight_design: FlightCondition, mdot_design: f64,
        map_hp: ComponentMap,
    ) -> Self {
        TwoSpoolFuelTransient::Degenerate(
            SpoolTransient::new(design_engine, flight_design, mdot_design, map_hp))
    }

    pub fn degenerate(&self) -> &SpoolTransient {
        match self {
            TwoSpoolFuelTransient::Degenerate(s) => s,
            TwoSpoolFuelTransient::Full(_) => panic!("this fuel transient is not lp_disabled"),
        }
    }

    pub fn core(&self) -> &FuelTransientCore {
        match self {
            TwoSpoolFuelTransient::Full(c) => c,
            TwoSpoolFuelTransient::Degenerate(_) => panic!("this fuel transient is lp_disabled"),
        }
    }

    pub fn core_mut(&mut self) -> &mut FuelTransientCore {
        match self {
            TwoSpoolFuelTransient::Full(c) => c,
            TwoSpoolFuelTransient::Degenerate(_) => panic!("this fuel transient is lp_disabled"),
        }
    }

    // --- THE DEGENERATE PATH'S OWN GUARDS ---------------------------------------------------
    //
    // Python's `integrate_fuel` opens with SEVEN asserts before forwarding to rung 35's marcher,
    // and `_fuel_ramp_march` opens with an eighth. None of them can live on [`FuelTransientCore`],
    // which is never degenerate by construction — so they live here, on the enum, and the rung-45
    // gate that exercises them (`test_reduce_lp_disabled_asserts_the_split_is_two_shaft`) has a
    // Rust home before step 2 writes a line against the wrong type.

    /// Python's `integrate_fuel` on an `lp_disabled` object: the seven refusals, then EXACT
    /// dispatch to rung 35's marcher.
    ///
    /// **THE `nu0` TYPE CHANGES ACROSS THE DISPATCH**, which is why this is a separate method
    /// rather than a branch inside [`FuelTransientCore::integrate_fuel`]: Python forwards
    /// whatever the caller passed, and a degenerate caller passes a SCALAR where a two-shaft one
    /// passes a pair. A single Rust signature would have to accept both.
    #[allow(clippy::too_many_arguments)]
    pub fn integrate_fuel_lp_disabled<S>(
        &self, flight: &FlightCondition, fuel_schedule: S, nu0: f64, s_end: f64, ds: f64,
        lim: &FuelLimiters<'_>,
    ) -> Vec<crate::spool::TransientPoint>
    where
        S: Fn(f64) -> f64,
    {
        let d = self.degenerate();
        assert!(lim.freeze.is_none(), "rung-43 channel isolation needs two spools");
        assert!(lim.tt4_max.is_none() && lim.tau_gov.is_none(),
                "the rung-46/47 TIT topping governor is inherently two-shaft (its finding is the \
                 rho-loud surge relief); lp_disabled is not a reduce axis for it.");
        assert!(lim.accel.is_none(),
                "the rung-48 Wf/pt3 accel schedule is inherently two-shaft (its finding is the \
                 PER-SPOOL engagement crossing); lp_disabled is not a reduce axis.");
        assert!(lim.surge.is_none(),
                "the rung-49 phi floor is inherently two-shaft (its finding is the CREDIT on the \
                 watched spool against the DEBIT on the other); lp_disabled is not a reduce axis \
                 for a split BETWEEN spools.");
        assert!(lim.s_off.is_none(),
                "the rung-50 forced release isolates a split BETWEEN spools (both minima relocate \
                 to the release point); lp_disabled is not a reduce axis for it.");
        assert!(lim.tau_rel.is_none(),
                "the rung-51 release RATE rides on rung 50's forced release, which isolates a \
                 split BETWEEN spools; lp_disabled is not a reduce axis for it.");
        assert!(lim.lag.is_none(),
                "the rung-52 asymmetric lag's finding is a split BETWEEN spools (tau_att owns the \
                 credit exactly, the debit is joint); lp_disabled is not a reduce axis for it.");
        d.integrate_fuel(flight, fuel_schedule, nu0, s_end, ds, None)
    }

    /// Python's `equilibrium_fuel` on an `lp_disabled` object: EXACT dispatch to rung 35's fuel
    /// equilibrium, and rung 43's gate 2 compares the two side by side with `==`.
    ///
    /// **IT DROPS `start` ON THE FLOOR, AND THE DROP IS THE PORT.** Python's line is
    /// `return self._degenerate.equilibrium_fuel(flight, mdot_fuel)` — the caller's `start` is
    /// accepted by the signature and then never passed on, so a degenerate caller who supplies one
    /// silently gets rung 35's own bracketing search instead. Rust could simply not offer the
    /// parameter; that would be a nicer API and a WORSE port, because the difference would stop
    /// being visible at the call site. So the parameter is here, `_`-bound, and named in this
    /// comment. Likewise `cmap`: Python passes none, so the held object's own map is used.
    ///
    /// A separate method rather than an arm of [`FuelTransientCore::equilibrium_fuel`] for the
    /// same reason [`integrate_fuel_lp_disabled`](Self::integrate_fuel_lp_disabled) is: the RETURN
    /// type changes across the dispatch — rung 35's 27-field [`Instant`](crate::spool::Instant)
    /// against the two-shaft [`FuelInstant`]. Python's duck typing hides that; Rust cannot.
    pub fn equilibrium_fuel_lp_disabled(
        &self, flight: &FlightCondition, mdot_fuel: f64, _start: Option<(f64, f64)>,
    ) -> crate::spool::Instant {
        self.degenerate().equilibrium_fuel(flight, mdot_fuel, None)
    }

    /// RUNG 45's `phi_excursion_fuel`, through the enum — which REFUSES the degenerate engine.
    ///
    /// The fuel-path transient surge split is inherently two-shaft (rung 44's contract), so
    /// `lp_disabled` is not a reduce axis for it. Python asserts; this panics.
    #[allow(clippy::too_many_arguments)]
    pub fn phi_excursion_fuel(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, r: f64, s_settle: f64, ds: f64,
        tt4_max: Option<f64>, tau_gov: Option<f64>, accel: Option<&AccelSchedule>,
        surge: Option<SurgeLimiter>,
    ) -> PhiExcursionFuel {
        assert!(matches!(self, TwoSpoolFuelTransient::Full(_)),
                "the fuel-path transient surge split is inherently two-shaft (rung 44's contract): \
                 lp_disabled is not a reduce axis for a split BETWEEN spools.");
        self.core().phi_excursion_fuel(flight, tt4_lo, tt4_hi, r, s_settle, ds, tt4_max, tau_gov,
                                       accel, surge)
    }

    /// RUNG 45's `transient_surge_margin_fuel`, through the enum — same refusal.
    #[allow(clippy::too_many_arguments)]
    pub fn transient_surge_margin_fuel(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, r: f64, s_settle: f64, ds: f64,
        tt4_max: Option<f64>, tau_gov: Option<f64>, accel: Option<&AccelSchedule>,
        surge: Option<SurgeLimiter>,
    ) -> TransientSurgeMarginFuel {
        assert!(matches!(self, TwoSpoolFuelTransient::Full(_)),
                "the fuel-path transient surge split is inherently two-shaft (rung 44's contract): \
                 lp_disabled is not a reduce axis for a split BETWEEN spools.");
        self.core().transient_surge_margin_fuel(flight, tt4_lo, tt4_hi, r, s_settle, ds, tt4_max,
                                                tau_gov, accel, surge)
    }

    /// RUNG 46's [`topping_relief`](FuelTransientCore::topping_relief), through the enum — which
    /// REFUSES the degenerate engine.
    ///
    /// **THIS IS THE ONE READER THAT NEEDS THE ENUM.** Of the four this slice ports, only
    /// `topping_relief` is called on an `lp_disabled` object by any suite
    /// (`test_rung46.py::test_reduce_lp_disabled_asserts_the_split_is_two_shaft`); rungs 47 and 48
    /// exercise the same refusal through `integrate_fuel` instead. Python reaches the assert one
    /// level down, inside `phi_excursion_fuel`; the refusal is hoisted here so the panic names the
    /// method the caller actually invoked.
    #[allow(clippy::too_many_arguments)]
    pub fn topping_relief(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64, r: f64,
        s_settle: f64, ds: f64, tau_gov: Option<f64>,
    ) -> ToppingRelief {
        assert!(matches!(self, TwoSpoolFuelTransient::Full(_)),
                "the rung-46/47 TIT topping governor is inherently two-shaft (its finding is the \
                 rho-loud surge relief); lp_disabled is not a reduce axis for it.");
        self.core().topping_relief(flight, tt4_lo, tt4_hi, tt4_max, r, s_settle, ds, tau_gov)
    }
}

/// Rung 43's object once `lp_disabled` is ruled out.
///
/// **COMPOSES over rung 40 through a `pub inner`** — as `combustor.rs` composes over
/// `SpoolTransient` and `two_spool_transient.rs` over `TwoSpoolMapCore`. Rung 40's own class is
/// already 1 271 lines of Rust and this is a different control: fuel imposed, `Tt4` an output,
/// seven limiter arms.
pub struct FuelTransientCore {
    /// Rung 40's transient. `pub` because the reduce gates need the SAME captured hardware on
    /// both sides, and because the INHERITED [`TwoSpoolTransientHooks`] lives on it.
    pub inner: TwoSpoolTransientCore,
}

impl FuelTransientCore {
    /// Rung 40's, inherited verbatim — an ABSOLUTE bar, which is exactly what probe 3's detector
    /// turns on.
    pub const EQ_TOL: f64 = TwoSpoolTransientCore::EQ_TOL;
    pub const EQ_MAX: usize = TwoSpoolTransientCore::EQ_MAX;
    /// The closure root's tolerance — a LITERAL `1e-12` at that call site.
    pub const CLOSE_TOL: f64 = 1e-12;
    /// [`try_topping_fuel`](Self::try_topping_fuel)'s — a LITERAL `1e-9`, and NOT the closure's.
    pub const TOPPING_TOL: f64 = 1e-9;
    /// [`try_sched_fuel`](Self::try_sched_fuel)'s and [`try_surge_fuel`](Self::try_surge_fuel)'s
    /// — a LITERAL `1e-13` at BOTH sites, tighter than either of the two above.
    pub const LEG_TOL: f64 = 1e-13;
    /// The forward burner's physical `f` ceiling and floor — the two that set the closure's low
    /// wall and its THIRD high-wall arm. Rung 35's single-spool closure uses `0.05` and has no
    /// floor at all; these are rung 43's own.
    pub const F_CAP: f64 = 0.065;
    pub const F_FLOOR: f64 = 0.004;
    /// The march-in step — rung 40's is `0.02`.
    pub const MARCH_IN_STEP: f64 = 0.04;

    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap, rho: f64,
    ) -> Self {
        FuelTransientCore {
            inner: TwoSpoolTransientCore::new(
                design_engine, flight_design, mdot_design, map_lp, map_hp, rho),
        }
    }

    pub fn with_hooks(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap, rho: f64,
        hooks: &'static TwoSpoolTransientHooks,
    ) -> Self {
        FuelTransientCore {
            inner: TwoSpoolTransientCore::with_hooks(
                design_engine, flight_design, mdot_design, map_lp, map_hp, rho, hooks),
        }
    }

    pub fn gas(&self) -> &Gas { self.inner.gas() }
    pub fn rho(&self) -> f64 { self.inner.rho }

    // --- the helper Python deliberately does NOT inherit -------------------------------------

    /// Linear interpolation on a sorted grid.
    ///
    /// **A SECOND COPY ON PURPOSE.** Its docstring says why: "the two-spool chain does not inherit
    /// `SpoolTransient`'s copy — `TwoSpoolMatcher` is deliberately not a subclass of it". Routing
    /// this at [`SpoolTransient::interp`] would factor away a duplication the source states it
    /// chose, and the port's own COPY-vs-REDERIVATION rule says an "exactly" claim survives a
    /// copied instruction sequence and dies on a second derivation.
    ///
    /// All three arms are LIVE (12 / 2 420 / 2 752); the fall-through `return ys[-1]` is DEAD and
    /// is spelled and counted.
    pub fn interp(xs: &[f64], ys: &[f64], x: f64) -> f64 {
        if x <= xs[0] {
            bump(&INTERP_LOW);
            return ys[0];
        }
        if x >= xs[xs.len() - 1] {
            bump(&INTERP_HIGH);
            return ys[ys.len() - 1];
        }
        for i in 0..xs.len() - 1 {
            if xs[i] <= x && x <= xs[i + 1] {
                bump(&INTERP_MID);
                let t = (x - xs[i]) / (xs[i + 1] - xs[i]);
                return ys[i] + t * (ys[i + 1] - ys[i]);
            }
        }
        bump(&INTERP_FALLTHROUGH);
        ys[ys.len() - 1]
    }

    // --- rung 35's forward burner, on the two-spool matcher ----------------------------------

    /// Forward burner: `Tt4` as the OUTPUT of `f` — the exact inverse of the `f`-solve.
    ///
    /// ```text
    /// h4*(1 + f) = h_c(Tt3) + f*eta_b*hPR   =>   Tt4 = T_from_h_t(h4, f)
    /// ```
    ///
    /// **THE REFUSAL IS AN `Abort`, NOT A PANIC — and § 5.16 probe 4 (A) is why.** Rung 35's
    /// single-spool twin keeps its panic, correctly: nothing there catches it. Here the assert
    /// fires inside the closure's bracket scan, which SWALLOWS it, so a panic would abort a march
    /// Python completes. The panicking spelling stays available as
    /// [`tt4_from_f`](Self::tt4_from_f) for the gate that pokes it directly.
    pub fn try_tt4_from_f(&self, tt3: f64, f: f64) -> Result<f64, Abort> {
        let c = &self.inner.inner;
        let gas = c.gas();
        if gas.is_equilibrium() {
            return Err(Abort(
                "rung-43 fuel control needs the forward burner Tt4(f), built for the \
                 non-equilibrium gas; use Tt4-control (equilibrium/integrate, rung 40) for the \
                 reacting-gas two-spool cycle.".to_string()));
        }
        let h4 = (gas.h_c(tt3) + f * c.base.eta_b * gas.hpr()) / (1.0 + f);
        Ok(gas.t_from_h_t(h4, f))
    }

    /// [`try_tt4_from_f`](Self::try_tt4_from_f) for a caller that cannot recover — the spelling
    /// rung 43's own refusal gate pokes directly, where the raise is the observable.
    pub fn tt4_from_f(&self, tt3: f64, f: f64) -> f64 {
        self.try_tt4_from_f(tt3, f).unwrap_or_else(|e| panic!("{}", e.0))
    }

    // --- THE FORWARD CLOSURE with FUEL imposed: one root in m_L, no shaft balance -------------

    /// Rung 40's closure with the burner run FORWARD — `Tt4` FLOATS.
    ///
    /// `f = mdot_fuel/mdot_air` with `mdot_air` the LP-FACE airflow, so `f` and `Tt4` are OUTPUTS
    /// of the trial flow; the HP-fed NGV choke then implies an airflow and consistency closes
    /// `m_L`. Still ONE unknown, still NO shaft balance — both power residuals stay OUTPUTS, which
    /// is what makes them the two ODE right-hand sides. **This is where the two-shaft airflow LAG
    /// lives.**
    ///
    /// **THE SCAN IS NOT RUNG 40's, AND THE SOURCE SAYS WHY.** Rung 40's global high wall is safe
    /// only because `Tt4` is PINNED there; with `Tt4` floating, far past the root the mixture goes
    /// lean, the HP map leaves its physical branch and the sonic-throat solve fails, so a
    /// wall-to-wall bracket can straddle nonsense. `g` rises monotonically through the physical
    /// root, so this keeps the LAST negative and takes the FIRST crossing after it. Rung 40's own
    /// closure is untouched — this is a consequence of the CONTROL change, not a fix to rung 40.
    ///
    /// # ⚠ PHASE 7 NEEDS THIS BEHIND A HOOK CELL — **4 classes override it**
    ///
    /// `ScheduledStatorTransient`, `ScheduledBleedTransient`, `LimitedBleedTransient` and
    /// `LaggedBleedTransient` each replace Python's `_close_fuel`. One of § 5.12's six crossing
    /// names, and one of the three [`TwoSpoolTransientHooks`] has no cell for — see
    /// [`integrate_fuel`](Self::integrate_fuel) for why the cell is not built here.
    pub fn try_close_fuel(
        &self, nu_lp: f64, nu_hp: f64, mdot_fuel: f64, tt2: f64, pt2: f64,
    ) -> Result<FuelCloseState, Abort> {
        bump(&CLOSE_CALLS);
        let t = &self.inner;
        let c = &t.inner;
        let gas = c.gas();
        let n_lp = nu_lp * powp(c.tt2_d / tt2, 0.5);
        let (h2, pr2) = (gas.h_c(tt2), gas.pr_c(tt2));

        let ev = |m_lp: f64| -> Result<FuelCloseState, Abort> {
            let phi_lp = m_lp / n_lp;
            let tau_lpc = 1.0 + (c.tau_lpc_d - 1.0) * c.map_lp.psi(phi_lp) * n_lp * n_lp;
            let tt25 = tt2 * tau_lpc;
            let eta_lpc = c.map_lp.eta_c_at(c.base.eta_lpc, phi_lp, n_lp);
            let h25 = gas.h_c(tt25);
            // The LPC ideal-temperature inversion. FALLIBLE for the same reason as the HPC one
            // below, and measured DEAD on every grid so far — the two are one `try` scope in
            // Python, so making only the measured one fallible would panic where Python swallows.
            let pi_lpc = gas.pr_c(gas.try_t_from_h_c(h2 + eta_lpc * (h25 - h2))?) / pr2;
            let pt25 = pi_lpc * pt2;
            // THE FACE FLOW — and unlike rung 40, it is RETURNED: `f` is built from it.
            let mdot_air_face = m_lp * c.mcorr_lp_d * pt2 / powp(tt2, 0.5);

            // Same physical air flow, referred to the HP face.
            let m_hp = (mdot_air_face * powp(tt25, 0.5) / pt25) / c.mcorr_hp_d;
            let n_hp = nu_hp * powp(c.tt25_d / tt25, 0.5);
            let phi_hp = m_hp / n_hp;
            let tau_hpc = 1.0 + (c.tau_hpc_d - 1.0) * c.map_hp.psi(phi_hp) * n_hp * n_hp;
            let tt3 = tt25 * tau_hpc;
            let eta_hpc = c.map_hp.eta_c_at(c.base.eta_hpc, phi_hp, n_hp);
            let h3 = gas.h_c(tt3);
            // **THE MEASURED FALLIBLE SITE.** On the equilibrium gas this lands outside the
            // 150-4000 K table 8 times per closure, at m_lp in 1.739...2.019, where psi_H < 0
            // makes the ideal enthalpy rise negative. Slice L left `t_from_h` panicking because
            // the call sites that existed then never reached it; this one does.
            let pi_hpc =
                gas.pr_c(gas.try_t_from_h_c(h25 + eta_hpc * (h3 - h25))?) / gas.pr_c(tt25);
            let pt4 = c.base.pi_b * pi_hpc * pt25;

            // THE INVERSION vs rung 40: fuel imposed => f and Tt4 are OUTPUTS.
            let f = mdot_fuel / mdot_air_face;
            let tt4 = self.try_tt4_from_f(tt3, f)?;
            let wgas = c.base.try_working_gas(f, tt4, pt4)?;
            let wg = wgas.as_ref().unwrap_or(gas);
            let mdot4 = c.base.a4 * pt4 * choked_mfp(wg, tt4, f) / powp(tt4, 0.5);
            let mdot_imp = mdot4 / (1.0 + f);
            let m_imp = (mdot_imp * powp(tt2, 0.5) / pt2) / c.mcorr_lp_d;
            Ok(FuelCloseState {
                base: CloseState {
                    m_lp, m_imp, m_hp, phi_lp, phi_hp, tt2, n_lp, n_hp, tau_lpc, tau_hpc, tt25,
                    tt3, pi_lpc, pi_hpc, pt4, f, wgas, eta_lpc, eta_hpc, mdot_air: mdot_imp,
                    mdot4,
                },
                tt4,
                mdot_air_face,
            })
        };

        // THE OFF-MAP GUARD. Rung 40's closure carries the same one and states the case in full:
        // **Rust returns NaN where Python returns a COMPLEX**, so the port's test is Python's
        // `r == r` inverted. NOT `is_finite()` — Python's guard is `isinstance(r, float) and
        // r == r`, which an INFINITY passes.
        let g = |m: f64| -> Result<f64, Abort> {
            bump(&CLOSE_G_EVALS);
            let r = m - ev(m)?.base.m_imp;
            if r.is_nan() {
                return Err(Abort(format!(
                    "off-map compressor trial at m_lp={m:.4}: the loading law has gone \
                     non-physical (Tt3 < 0 => a complex pressure ratio).")));
            }
            Ok(r)
        };

        // The two walls. `lo0` is the flow at which f hits its physical CEILING, `hi0` the flow at
        // which it hits the FLOOR — and `hi0` is the third arm of a `min` rung 40 spells with two.
        let lo0 = mdot_fuel * powp(tt2, 0.5) / (Self::F_CAP * c.mcorr_lp_d * pt2);
        let hi0 = mdot_fuel * powp(tt2, 0.5) / (Self::F_FLOOR * c.mcorr_lp_d * pt2);
        // Python's `min(a, b, c)` spelled as the fold it IS, so the arm classification and the
        // value come out of one statement. `f64::min` would differ on NaN and would not say which
        // arm bound. Measured 24 033 / 200 193 / 3 663 — all three live.
        let wall_map = c.map_lp.phi_max(0.1) * n_lp;
        let mut cap = 2.5f64;
        let mut arm = &HI_WALL_LITERAL;
        if wall_map < cap {
            cap = wall_map;
            arm = &HI_WALL_MAP;
        }
        if hi0 < cap {
            cap = hi0;
            arm = &HI_WALL_HI0;
        }
        bump(arm);

        // Python's `max(lo0, 0.02)` — the literal arm is DEAD (0 of 227 889) and is spelled.
        let mut m = lo0;
        if 0.02 > m {
            m = 0.02;
            bump(&LO_FLOOR_HITS);
        }

        // THE SCAN. Not rung 40's break-at-first-success: this keeps reassigning `lo` to the LAST
        // negative and stops at the FIRST positive that follows one. `ghi` is produced INSIDE the
        // try, so a failure at the high end advances rather than propagating.
        let (mut lo, mut glo, mut hi, mut ghi) = (None, 0.0f64, None, 0.0f64);
        while m < cap {
            let gm = match g(m) {
                Ok(v) => v,
                Err(e) => {
                    bump(&MARCH_IN_ADVANCES);
                    bump(match classify(&e) {
                        FuelAbort::Refusal => &MARCH_IN_REFUSAL,
                        FuelAbort::InverseBracket => &MARCH_IN_INVERSE,
                        FuelAbort::OffMap => &MARCH_IN_OFFMAP,
                        _ => &MARCH_IN_OTHER,
                    });
                    m += Self::MARCH_IN_STEP;
                    continue;
                }
            };
            if gm < 0.0 {
                lo = Some(m);
                glo = gm;
            } else if lo.is_some() {
                hi = Some(m);
                ghi = gm;
                break;
            }
            m += Self::MARCH_IN_STEP;
        }
        // **NO SIGN FILTER.** Rung 40's guard re-tests `glo < 0 && 0 < ghi`; rung 43's tests only
        // that both endpoints were found, because the scan's own arms already decided the signs.
        let (Some(lo), Some(hi)) = (lo, hi) else {
            bump(&CLOSE_BRACKET_FAILS);
            return Err(Abort(format!(
                "rung-43 fuel closure does not bracket at nu=({nu_lp:.4},{nu_hp:.4}), \
                 mdot_fuel={mdot_fuel:.5} - off the modeled speed-line region.")));
        };
        let root = try_illinois(g, lo, hi, glo, ghi, Self::CLOSE_TOL, ILLINOIS_MAXIT)?;
        ev(root)
    }

    pub fn close_fuel(
        &self, nu_lp: f64, nu_hp: f64, mdot_fuel: f64, tt2: f64, pt2: f64,
    ) -> FuelCloseState {
        self.try_close_fuel(nu_lp, nu_hp, mdot_fuel, tt2, pt2)
            .unwrap_or_else(|e| panic!("{}", e.0))
    }

    // --- one quasi-steady instant ------------------------------------------------------------

    /// The quasi-steady instant at `(nu_L, nu_H, mdot_fuel)` — `Tt4` is an OUTPUT.
    ///
    /// **THIS IS THE SLICE'S ONE LIVE HOOK CELL.** § 5.15 registered that rung 43 overrides none
    /// of rung 40's three virtual names — true — and inferred that rung 40's table "ships with
    /// zero cells exercised inside phase 6". *Overridden and exercised are different claims*: the
    /// tail is reached HERE, on the hot path, 227 856 times per full grid. The other two cells are
    /// genuinely untouched — the closure REPLACES rung 40's rather than calling it, and
    /// [`try_equilibrium_fuel`](Self::try_equilibrium_fuel) runs its own 2-D Newton rather than
    /// calling `powers`.
    pub fn try_instant_fuel(
        &self, flight: &FlightCondition, nu_lp: f64, nu_hp: f64, mdot_fuel: f64,
    ) -> Result<FuelInstant, Abort> {
        bump(&INSTANT_CALLS);
        let (tt2, pt2, v0) = self.inner.inlet(flight);
        let c = self.try_close_fuel(nu_lp, nu_hp, mdot_fuel, tt2, pt2)?;
        let base = self.inner.try_instant_tail(flight, &c.base, nu_lp, nu_hp, c.tt4, v0)?;
        Ok(FuelInstant { base, mdot_air_face: c.mdot_air_face })
    }

    pub fn instant_fuel(
        &self, flight: &FlightCondition, nu_lp: f64, nu_hp: f64, mdot_fuel: f64,
    ) -> FuelInstant {
        self.try_instant_fuel(flight, nu_lp, nu_hp, mdot_fuel)
            .unwrap_or_else(|e| panic!("{}", e.0))
    }

    // --- RUNG 46: the TIT topping governor's set-point solve ---------------------------------

    /// RUNG 46. The instantaneous fuel that pins `Tt4 == Tt4_max` at the CURRENT flow.
    ///
    /// `Tt4` rises monotonically with fuel at fixed spool speeds, so a bracketed Illinois solve is
    /// robust. `mf_over` is the scheduled fuel, KNOWN by the caller to overshoot: it is the UPPER
    /// bracket, and the LOWER is found by halving until `Tt4` falls under the redline.
    ///
    /// **TWO SUBTLETIES THE STRUCTURE FORCES, both reproduced.** `resid(mf_over)` sits OUTSIDE the
    /// halving loop's catch, so a failure there PROPAGATES; and on the catch arm `glo` keeps its
    /// PREVIOUS value while `lo` halves, so if the loop exhausts, `lo` and `glo` are one halving
    /// apart. The tolerance is `1e-9` — neither the closure's `1e-12` nor the legs' `1e-13`.
    pub fn try_topping_fuel(
        &self, flight: &FlightCondition, nu_lp: f64, nu_hp: f64, tt4_max: f64, mf_over: f64,
    ) -> Result<f64, Abort> {
        bump(&TOPPING_CALLS);
        let resid = |mf: f64| -> Result<f64, Abort> {
            Ok(self.try_instant_fuel(flight, nu_lp, nu_hp, mf)?.base.tt4 - tt4_max)
        };
        let hi = mf_over;
        let ghi = resid(hi)?; // > 0 by the caller's guard, and NOT caught
        let mut lo = 0.5 * hi;
        let mut glo: Option<f64> = None;
        let mut broke = false;
        for _ in 0..40 {
            match resid(lo) {
                Err(_) => {
                    bump(&TOPPING_SKIPS);
                    lo *= 0.5;
                    continue;
                }
                Ok(v) => glo = Some(v),
            }
            if glo.expect("just assigned") < 0.0 {
                broke = true;
                break;
            }
            lo *= 0.5;
        }
        if !broke {
            bump(&TOPPING_EXHAUSTED);
        }
        let ok = glo.is_some_and(|v| v < 0.0);
        if !ok {
            return Err(Abort(format!(
                "rung-46 topping cannot reach Tt4_max={tt4_max:.1} at \
                 nu=({nu_lp:.4},{nu_hp:.4}) -- redline below the flow's floor Tt4.")));
        }
        try_illinois(resid, lo, hi, glo.expect("checked"), ghi, Self::TOPPING_TOL, ILLINOIS_MAXIT)
    }

    // --- RUNG 48: the Wf/pt3 accel schedule — DERIVED shape, one imposed scalar ---------------

    /// RUNG 48. Build the `Wf/pt3` accel schedule by reading the plant's OWN steady running line
    /// over the accel band.
    ///
    /// The SHAPE is DERIVED (no curve is imposed); the entire imposition is the one scalar
    /// `margin`. `margin = 0` is "never exceed the steady fuel/pressure ratio".
    pub fn accel_schedule(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, margin: f64, n: usize,
    ) -> AccelSchedule {
        assert!(margin >= 0.0, "rung-48 accel-schedule margin is measured ABOVE the steady line");
        assert!(n >= 2, "the derived schedule needs at least the two band endpoints");
        let mut rows: Vec<(f64, f64)> = Vec::with_capacity(n);
        for k in 0..n {
            let eq = self.inner.equilibrium(
                flight, tt4_lo + (tt4_hi - tt4_lo) * k as f64 / (n as f64 - 1.0));
            let pt3 = eq.close.pt4 / self.inner.inner.base.pi_b;
            rows.push((eq.close.n_hp, eq.close.f * eq.close.mdot_air / pt3));
        }
        // Python sorts TUPLES: ties on n_H fall through to kappa. Spelled the same way.
        rows.sort_by(|a, b| a.partial_cmp(b).expect("running-line rows are finite"));
        AccelSchedule {
            margin,
            n_h: rows.iter().map(|&(a, _)| a).collect(),
            kappa: rows.iter().map(|&(_, b)| b).collect(),
        }
    }

    /// RUNG 48. The applied fuel under the `Wf/pt3` leg at the CURRENT flow.
    ///
    /// The cap is IMPLICIT in `Wf` — `pt3` and `n_H` both move with the fuel through the closure —
    /// so this is a bracketed set-point solve, the same structure as
    /// [`try_topping_fuel`](Self::try_topping_fuel).
    ///
    /// **Returns `mf_sched` ITSELF when dormant** — float-identical, no solve. That is what makes
    /// the rung-48 dormant reduce BIT-FOR-BIT rather than merely equal, and § 5.16 measured what
    /// else it buys: the downstream `c < mf` comparison becomes a float compared with itself, an
    /// EXACT structural zero, unflippable on any interpreter.
    pub fn try_sched_fuel(
        &self, flight: &FlightCondition, nu_lp: f64, nu_hp: f64, mf_sched: f64,
        accel: &AccelSchedule,
    ) -> Result<f64, Abort> {
        bump(&SCHED_CALLS);
        let big_g = |w: f64| -> Result<f64, Abort> {
            let i = self.try_instant_fuel(flight, nu_lp, nu_hp, w)?;
            Ok(w - accel.cap(i.base.close.n_hp, i.base.close.pt4 / self.inner.inner.base.pi_b))
        };
        let (hi, ghi) = (mf_sched, big_g(mf_sched)?);
        if ghi <= 0.0 {
            bump(&SCHED_DORMANT);
            return Ok(mf_sched); // DORMANT -- the leg is not consulted
        }
        let mut lo = mf_sched;
        let mut glo: Option<f64> = None;
        for _ in 0..60 {
            lo *= 0.85;
            match big_g(lo) {
                Err(_) => {
                    bump(&SCHED_SKIPS);
                    continue; // off the modeled speed-line region
                }
                Ok(v) => glo = Some(v),
            }
            if glo.expect("just assigned") < 0.0 {
                break;
            }
            // Python RESETS to None here, so the assert below is "we broke", not "we ever
            // evaluated" -- an arm that reads as redundant and is not.
            glo = None;
        }
        let Some(glo) = glo else {
            let margin = accel.margin;
            return Err(Abort(format!(
                "rung-48 accel-schedule set point does not bracket at \
                 nu=({nu_lp:.4},{nu_hp:.4}), mf_sched={mf_sched:.5}, margin={margin}")));
        };
        try_illinois(big_g, lo, hi, glo, ghi, Self::LEG_TOL, ILLINOIS_MAXIT)
    }

    // --- RUNG 49: the phi / surge-margin FEEDBACK leg -----------------------------------------

    /// RUNG 49. The applied fuel under the `phi` floor at the CURRENT flow.
    ///
    /// `phi` falls MONOTONICALLY with fuel at fixed spool speeds — more fuel ⇒ hotter `Tt4` ⇒ less
    /// choked-NGV corrected capacity ⇒ less flow at the same `n` — so the bracket is sound:
    /// cutting fuel RAISES `phi`.
    ///
    /// Structurally [`try_sched_fuel`](Self::try_sched_fuel)'s twin with **`0.9` where that one
    /// steps `0.85`**, and the same float-identical dormant return.
    ///
    /// # ⚠ PHASE 7 NEEDS THIS BEHIND A HOOK CELL — **`ScheduledStatorTransient` overrides it**
    ///
    /// The third of § 5.12's six crossing names with no [`TwoSpoolTransientHooks`] cell. Only one
    /// overrider, so the cost of missing it is the smallest of the three — and the reason it is
    /// written down anyway is that a single overrider is exactly the one a census skims past. See
    /// [`integrate_fuel`](Self::integrate_fuel) for why the cell is not built here.
    pub fn try_surge_fuel(
        &self, flight: &FlightCondition, nu_lp: f64, nu_hp: f64, mf_sched: f64,
        surge: &SurgeLimiter,
    ) -> Result<f64, Abort> {
        bump(&SURGE_CALLS);
        let big_g = |w: f64| -> Result<f64, Abort> {
            // > 0 when phi is BELOW the floor (the limiter must cut fuel)
            Ok(surge.phi_lim - surge.read(&self.try_instant_fuel(flight, nu_lp, nu_hp, w)?))
        };
        let (hi, ghi) = (mf_sched, big_g(mf_sched)?);
        if ghi <= 0.0 {
            bump(&SURGE_DORMANT);
            return Ok(mf_sched); // DORMANT -- the leg is not consulted
        }
        let mut lo = mf_sched;
        let mut glo: Option<f64> = None;
        for _ in 0..60 {
            lo *= 0.9;
            match big_g(lo) {
                Err(_) => {
                    bump(&SURGE_SKIPS);
                    continue;
                }
                Ok(v) => glo = Some(v),
            }
            if glo.expect("just assigned") < 0.0 {
                break;
            }
            glo = None;
        }
        let Some(glo) = glo else {
            let (lim, sp) = (surge.phi_lim, match surge.spool {
                Spool::Lp => "LP",
                Spool::Hp => "HP",
            });
            return Err(Abort(format!(
                "rung-49 phi floor {lim:.4} on the {sp} spool is UNREACHABLE at \
                 nu=({nu_lp:.4},{nu_hp:.4}) -- no fuel this side of flame-out restores it. \
                 Lower the floor (it must sit below the running-line phi).")));
        };
        try_illinois(big_g, lo, hi, glo, ghi, Self::LEG_TOL, ILLINOIS_MAXIT)
    }

    // --- the equilibrium: a 2-D root at fixed FUEL --------------------------------------------

    /// Solve `Phi_L = Phi_H = 0` in `(nu_L, nu_H)` at fixed FUEL.
    ///
    /// **THE REDUCE, and it is not tautological**: with `mdot_fuel = f_eq*mdot_air_eq` of a
    /// rung-40 `Tt4`-control point this returns THAT point — via the forward-BURNER closure, a
    /// genuinely different code path. Control-invariance: a steady point is the same however it is
    /// named.
    ///
    /// **NO NOISE-FLOOR ACCEPTANCE, unlike rung 40's `equilibrium` — and the shipped reason for
    /// that is a CPG statement read as a general one.** The source says the fuel path refuses an
    /// equilibrium gas outright, "so this loop only ever runs on the non-equilibrium gases, whose
    /// residual floor is ~1e-14 — comfortably under the absolute `_EQ_TOL`". Measured: on CPG,
    /// yes (3.2e-16…1.4e-13, 7×…3 161× of margin). On the TPG gases the floor is **9.3e-13 — 65×
    /// worse and 8 % UNDER the bar it is called comfortably under**, and the pass count swings
    /// 16-fold between interpreters. No cell exhausts [`EQ_MAX`](Self::EQ_MAX), so the
    /// CONCLUSION survives; its stated REASON does not. The pass count is returned because it is
    /// the sharpest detector in the slice.
    ///
    /// Returns `(instant, passes)`.
    pub fn try_equilibrium_fuel(
        &self, flight: &FlightCondition, mdot_fuel: f64, start: Option<(f64, f64)>,
    ) -> Result<(FuelInstant, usize), Abort> {
        bump(&EQ_CALLS);
        let big_f = |a: f64, b: f64| -> Result<(f64, f64), Abort> {
            let i = self.try_instant_fuel(flight, a, b, mdot_fuel)?;
            Ok((i.base.phi_lp_dot, i.base.phi_hp_dot))
        };
        let (mut nl, mut nh) = start.unwrap_or((1.0, 1.0));
        for pass in 0..Self::EQ_MAX {
            let (fl, fh) = big_f(nl, nh)?;
            if fl.abs().max(fh.abs()) < Self::EQ_TOL {
                return Ok((self.try_instant_fuel(flight, nl, nh, mdot_fuel)?, pass));
            }
            // Counted AFTER the exit check, so the counter accumulates the same object the
            // returned `pass` reports and the same one Python's dump recovers from its
            // `_instant_fuel` call count: passes COMPLETED, not residuals evaluated. Bumping it
            // above would be off by exactly one per call — which is how it was caught.
            EQ_PASSES.with(|x| x.set(x.get() + 1));
            let h = 1e-6;
            let (al, ah) = big_f(nl + h, nh)?;
            let (bl, bh) = big_f(nl, nh + h)?;
            let (j11, j12) = ((al - fl) / h, (bl - fl) / h);
            let (j21, j22) = ((ah - fh) / h, (bh - fh) / h);
            let det = j11 * j22 - j12 * j21;
            assert!(det.abs() > 1e-300, "rung-43 fuel equilibrium Jacobian is singular");
            let dl = (-fl * j22 + fh * j12) / det;
            let dh = (-j11 * fh + j21 * fl) / det;
            // Python's `min(1.0, 0.25/max(|dl|, |dh|, 1e-30))`. BOTH arms measured DEAD (0 of 8
            // Newton steps) and both spelled -- the `1e-30` floor and the damper itself.
            let mut den = dl.abs();
            if dh.abs() > den {
                den = dh.abs();
            }
            if 1e-30 > den {
                den = 1e-30;
                bump(&EQ_DAMP_FLOOR);
            }
            let mut damp = 1.0f64;
            if 0.25 / den < damp {
                damp = 0.25 / den;
                bump(&EQ_DAMPED);
            }
            nl += damp * dl;
            nh += damp * dh;
        }
        bump(&EQ_EXHAUSTED);
        Err(Abort(format!(
            "rung-43 fuel equilibrium did not converge at mdot_fuel={mdot_fuel:.5}")))
    }

    pub fn equilibrium_fuel(
        &self, flight: &FlightCondition, mdot_fuel: f64, start: Option<(f64, f64)>,
    ) -> (FuelInstant, usize) {
        self.try_equilibrium_fuel(flight, mdot_fuel, start)
            .unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The steady fuel flow whose running-line equilibrium IS rung 40's `Tt4` point.
    ///
    /// Pins the two control modes to the SAME steady endpoint — no new knob, so the excursions are
    /// apples-to-apples.
    pub fn fuel_for_tt4(&self, flight: &FlightCondition, tt4: f64) -> f64 {
        let eq = self.inner.equilibrium(flight, tt4);
        eq.close.f * eq.close.mdot_air
    }

    // --- the march ---------------------------------------------------------------------------

    /// RK4-march `(dnu_L/ds, dnu_H/ds) = (Phi_L/rho, Phi_H)` under a FUEL schedule. `Tt4` is an
    /// OUTPUT recorded per point — it can overshoot the steady value.
    ///
    /// **THE LOOP IS NOT RUNG 40's.** That one breaks before the final RK stages and floors both
    /// speeds at `0.2`; this does NEITHER. The wasted final stages are invisible in the returned
    /// trajectory and visible in the census, which is why the census is dumped.
    ///
    /// `int(round(s_end/ds))` is not merely live here, it is a TIE: rung 43's ramps put
    /// `8.25/0.02 = 412.5` exactly on the boundary, and Python rounds to 412 where a naive
    /// `f64::round` gives 413. [`round_ties_even`](f64::round_ties_even) is INHERITED from rung
    /// 40's marcher, not re-decided. Only the trajectory LENGTH sees the difference — every
    /// reported value is blind to it, measured — so the length is an oracle key.
    ///
    /// The seven limiter arms are dispatched by [`FuelLimiters`], and the asserts below are
    /// Python's, in Python's order.
    ///
    /// # ⚠ PHASE 7 NEEDS THIS BEHIND A HOOK CELL — **11 classes override it**
    ///
    /// § 5.12's census measured six names called on `self` inside phase 6 and overridden in phase
    /// 7. [`TwoSpoolTransientHooks`] carries three of them (`try_close`, `try_instant_tail`,
    /// `powers`); this is the largest of the three that have no cell — `LaggedBleedTransient`
    /// through `SensedCapTransient` each replace it. **Calling it directly from a phase-7 body
    /// would silently run rung 43's version**, which is the failure mode the ladder architecture
    /// exists to prevent.
    ///
    /// No cell is built here on purpose: phase 7 is unauthorised, and a hook with one
    /// implementation is a guess at what the second one needs. Slice T writes the note instead,
    /// **at the definition rather than only into § 5.17** — slice O's lesson, where what actually
    /// reached the next slice was a panic with a backtrace and the paragraph that had predicted it
    /// correctly was read second.
    pub fn integrate_fuel<S>(
        &self, flight: &FlightCondition, fuel_schedule: S, nu0: (f64, f64), s_end: f64, ds: f64,
        lim: &FuelLimiters<'_>,
    ) -> Vec<FuelPoint>
    where
        S: Fn(f64) -> f64,
    {
        assert!(lim.lag.is_none() || lim.accel.is_some() || lim.surge.is_some(),
                "rung-52 lag lags a min-select LEG's clip -- arm one (accel/surge).");
        assert!(lim.lag.is_none() || (lim.s_off.is_none() && lim.tau_rel.is_none()),
                "rung-52 lag and rung 50/51's s_off/tau_rel are ALTERNATIVE release instruments, \
                 not composable.");
        assert!(lim.lag.is_none() || lim.tau_gov.is_none(),
                "rung-52 lag and rung-47 tau_gov are both a clip AMOUNT carried as a state, on \
                 two different legs. Running both is a two-lag cascade, not this rung.");
        assert!(lim.tau_gov.is_none() || lim.tt4_max.is_some(),
                "rung-47 tau_gov is a governor lag -- it needs a redline (Tt4_max) to lag.");
        assert!(lim.tau_rel.is_none() || lim.s_off.is_some(),
                "rung-51 tau_rel is the RATE of a FORCED release -- it needs the release time \
                 s_off to be pinned.");
        assert!(lim.tau_rel.is_none_or(|t| t >= 0.0), "rung-51 tau_rel is a fade DURATION");
        assert!(lim.s_off.is_none() || lim.accel.is_some() || lim.surge.is_some(),
                "rung-50 s_off forces a min-select LEG to release early -- arm one \
                 (accel/surge).");

        if let Some(lag) = lim.lag {
            return self.integrate_fuel_asym(
                flight, fuel_schedule, nu0, s_end, ds, lim.freeze, lim.tt4_max, lim.accel,
                lim.surge.as_ref(), &lag);
        }
        if let (Some(tt4_max), Some(tau_gov)) = (lim.tt4_max, lim.tau_gov) {
            return self.integrate_fuel_lagged(
                flight, fuel_schedule, nu0, s_end, ds, lim.freeze, tt4_max, tau_gov, lim.accel,
                lim.surge.as_ref(), lim.s_off, lim.tau_rel);
        }

        bump(&MARCH_CALLS);
        // THE MIN-SELECT. Each cap is solved INDEPENDENTLY from the SCHEDULED fuel, so arming one
        // leg cannot perturb the other's bracket -- two Illinois solves off different brackets
        // agree only to tolerance, not bit-for-bit, and rung 43 gate 3 demands bit-for-bit.
        let der = |a: f64, b: f64, mf_in: f64, s: f64|
         -> Result<(f64, f64, f64, FuelInstant), Abort> {
            bump(&DER_CALLS);
            let mut mf = mf_in;
            let mut i = self.try_instant_fuel(flight, a, b, mf)?;
            let mut caps: Vec<f64> = Vec::new();
            // RUNG 50/51: the leg's AUTHORITY `w` is a pure function of s. `s_off = None`
            // short-circuits to 1.0 and a falsy `tau_rel` makes it the rung-50 step, so rungs
            // 49/50 are reached by the IDENTICAL branch and stay bit-for-bit.
            let w = release_weight(s, lim.s_off, lim.tau_rel);
            // RUNG 51. `w == 1.0` returns the cap ITSELF -- float-identical, which is what keeps
            // the rung-50 reduce bit-for-bit. Note it fades toward `mf`, the applied fuel; the
            // lagged twin's same-named closure fades toward `mf_sched` instead.
            let faded = |c: f64| if w >= 1.0 { c } else { mf + w * (c - mf) };

            if let Some(tt4_max) = lim.tt4_max {
                if i.base.tt4 > tt4_max {
                    caps.push(self.try_topping_fuel(flight, a, b, tt4_max, mf)?);
                }
            }
            if let Some(accel) = lim.accel {
                if w > 0.0 {
                    caps.push(faded(self.try_sched_fuel(flight, a, b, mf, accel)?));
                }
            }
            if let Some(surge) = lim.surge.as_ref() {
                if w > 0.0 {
                    caps.push(faded(self.try_surge_fuel(flight, a, b, mf, surge)?));
                }
            }
            bump(match caps.len() {
                0 => &DER_CAPS_0,
                1 => &DER_CAPS_1,
                2 => &DER_CAPS_2,
                _ => &DER_CAPS_3,
            });
            // A dormant leg returns `mf` itself, so this comparison is a float against itself --
            // an exact structural zero rather than a near-miss.
            caps.retain(|&c| c < mf);
            if !caps.is_empty() {
                bump(&DER_RESOLVES);
                let mut m = caps[0];
                for &c in &caps[1..] {
                    if c < m {
                        m = c;
                    }
                }
                mf = m;
                i = self.try_instant_fuel(flight, a, b, mf)?;
            }
            let da = if lim.freeze == Some(Spool::Lp) { 0.0 } else { i.base.phi_lp_dot / self.rho() };
            let db = if lim.freeze == Some(Spool::Hp) { 0.0 } else { i.base.phi_hp_dot };
            Ok((da, db, mf, i))
        };

        let mut pts: Vec<FuelPoint> = Vec::new();
        let (mut a, mut b) = nu0;
        // THE MARCH COORDINATE IS ACCUMULATED, AND A "CLEANER" `k as f64 * ds` WOULD FLIP A
        // PUBLISHED BOOLEAN. Python writes `s += ds` from `0.0`; summing `0.02` twenty-five times
        // gives `0.50000000000000011` where `25 * 0.02` gives exactly `0.5`. § 5.18 finding 3
        // measured a decision that lands inside that difference: rung 49's
        // [`SurgeRelief::both_edges_inside_ramp`] compares the LAST engaged `s` against the ramp
        // end `r`, and on one of its eight floor cells (the HP floor `0.8650`) the distance
        // `r − s_rel` is **`−1.11e-16`** — ONE ULP, and the boolean is `false` only because both
        // languages accumulate. The other cells clear by ONE GRID CELL, not by orders. So this
        // line is a COPY and not a spelling choice; the same holds for the two dispatch twins
        // below, which share the `s` sequence bit for bit (§ 5.18 finding 5b: 201 points at
        // `s_end = 4.0`, 301 at `6.0`, on all three marchers).
        let mut s = 0.0f64;
        let n_steps = (s_end / ds).round_ties_even() as i64;
        for _ in 0..=n_steps {
            let mf = fuel_schedule(s);
            let Ok((k1a, k1b, mf_app, inst)) = der(a, b, mf, s) else {
                bump(&MARCH_BREAK_K1);
                break;
            };
            pts.push(point(s, a, b, &inst, mf_app, mf, PointExtra::None));
            bump(&MARCH_POINTS);
            let stages = (|| -> Result<(f64, f64, f64, f64, f64, f64), Abort> {
                let mfm = fuel_schedule(s + ds / 2.0);
                let (k2a, k2b, _, _) =
                    der(a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, mfm, s + ds / 2.0)?;
                let (k3a, k3b, _, _) =
                    der(a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, mfm, s + ds / 2.0)?;
                let (k4a, k4b, _, _) =
                    der(a + ds * k3a, b + ds * k3b, fuel_schedule(s + ds), s + ds)?;
                Ok((k2a, k2b, k3a, k3b, k4a, k4b))
            })();
            let Ok((k2a, k2b, k3a, k3b, k4a, k4b)) = stages else {
                bump(&MARCH_BREAK_RK);
                break;
            };
            a += ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a);
            b += ds / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b);
            s += ds;
        }
        pts
    }

    /// RUNG 47. The TIT topping governor with a finite response lag — the sensing / limiter-loop
    /// lag of a real temperature limiter, which is the DOMINANT one (far larger than valve slew).
    ///
    /// ```text
    /// required(nu, s) = max(0, schedule(s) - topping(nu, Tt4_max))
    /// dg/ds = (required - g) / tau_gov
    /// mf_applied = schedule(s) - g
    /// ```
    ///
    /// Because `required` GROWS after engagement while `g` TRAILS it, the applied fuel stays ABOVE
    /// `topping` ⇒ `Tt4` OVERSHOOTS the redline — the classic topping overshoot. `g` is NOT the
    /// applied fuel but the REDUCTION: a pure valve-position lag is INERT on an accel, so the
    /// overshoot lives in the LOOP lag.
    ///
    /// **ITS `faded` IS A DIFFERENT FUNCTION FROM THE BARE MARCHER'S, under the same name.** This
    /// one fades toward `mf_sched`; the bare one fades toward the applied `mf`. And it
    /// min-selects SEQUENTIALLY with no `c < mf` filter, computing the instant ONCE at the end.
    /// § 5.16 measured that this route adds NO per-point key — its third state `g` is not
    /// recorded, which is worth knowing because rung 52's is.
    #[allow(clippy::too_many_arguments)]
    pub fn integrate_fuel_lagged<S>(
        &self, flight: &FlightCondition, fuel_schedule: S, nu0: (f64, f64), s_end: f64, ds: f64,
        freeze: Option<Spool>, tt4_max: f64, tau_gov: f64, accel: Option<&AccelSchedule>,
        surge: Option<&SurgeLimiter>, s_off: Option<f64>, tau_rel: Option<f64>,
    ) -> Vec<FuelPoint>
    where
        S: Fn(f64) -> f64,
    {
        bump(&MARCH_CALLS);
        let required = |a: f64, b: f64, mf_sched: f64| -> Result<f64, Abort> {
            let i = self.try_instant_fuel(flight, a, b, mf_sched)?;
            if i.base.tt4 > tt4_max {
                return Ok(mf_sched - self.try_topping_fuel(flight, a, b, tt4_max, mf_sched)?);
            }
            Ok(0.0)
        };
        let der = |a: f64, b: f64, g: f64, s: f64|
         -> Result<(f64, f64, f64, f64, FuelInstant), Abort> {
            bump(&DER_CALLS);
            let mf_sched = fuel_schedule(s);
            let mut mf = mf_sched - g;
            if 1e-9 > mf {
                mf = 1e-9;
                bump(&MF_FLOOR_HITS);
            }
            let w = release_weight(s, s_off, tau_rel);
            // RUNG 51, float-identical at w == 1.0 -- and referencing `mf_sched`, NOT `mf`.
            let faded = |c: f64| if w >= 1.0 { c } else { mf_sched + w * (c - mf_sched) };

            if let Some(accel) = accel {
                if w > 0.0 {
                    let c = faded(self.try_sched_fuel(flight, a, b, mf_sched, accel)?);
                    if c < mf {
                        mf = c;
                    }
                }
            }
            if let Some(surge) = surge {
                if w > 0.0 {
                    let c = faded(self.try_surge_fuel(flight, a, b, mf_sched, surge)?);
                    if c < mf {
                        mf = c;
                    }
                }
            }
            let i = self.try_instant_fuel(flight, a, b, mf)?;
            let da = if freeze == Some(Spool::Lp) { 0.0 } else { i.base.phi_lp_dot / self.rho() };
            let db = if freeze == Some(Spool::Hp) { 0.0 } else { i.base.phi_hp_dot };
            let dg = (required(a, b, mf_sched)? - g) / tau_gov;
            Ok((da, db, dg, mf, i))
        };

        let mut pts: Vec<FuelPoint> = Vec::new();
        let (mut a, mut b) = nu0;
        // `s` ACCUMULATED, never `k as f64 * ds` — see the note in [`Self::integrate_fuel`]'s
        // loop: § 5.18 finding 3 measured a shipped boolean decided at one ulp of that difference.
        let (mut g, mut s) = (0.0f64, 0.0f64);
        let n_steps = (s_end / ds).round_ties_even() as i64;
        for _ in 0..=n_steps {
            let Ok((k1a, k1b, k1g, mf_app, inst)) = der(a, b, g, s) else {
                bump(&MARCH_BREAK_K1);
                break;
            };
            pts.push(point(s, a, b, &inst, mf_app, fuel_schedule(s), PointExtra::None));
            bump(&MARCH_POINTS);
            let stages = (|| -> Result<(f64, f64, f64, f64, f64, f64, f64, f64, f64), Abort> {
                let (k2a, k2b, k2g, _, _) = der(
                    a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, g + ds / 2.0 * k1g, s + ds / 2.0)?;
                let (k3a, k3b, k3g, _, _) = der(
                    a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, g + ds / 2.0 * k2g, s + ds / 2.0)?;
                let (k4a, k4b, k4g, _, _) =
                    der(a + ds * k3a, b + ds * k3b, g + ds * k3g, s + ds)?;
                Ok((k2a, k2b, k2g, k3a, k3b, k3g, k4a, k4b, k4g))
            })();
            let Ok((k2a, k2b, k2g, k3a, k3b, k3g, k4a, k4b, k4g)) = stages else {
                bump(&MARCH_BREAK_RK);
                break;
            };
            a += ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a);
            b += ds / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b);
            g += ds / 6.0 * (k1g + 2.0 * k2g + 2.0 * k3g + k4g);
            s += ds;
        }
        pts
    }

    /// RUNG 52. The march with a min-select leg's clip carried as a state under a FAST-ATTACK /
    /// SLOW-RELEASE lag.
    ///
    /// `required` is computed from the SCHEDULED fuel and from UNFADED caps, so the legs are
    /// solved off the SAME bracket rungs 48/49 use — arming one cannot perturb the other's solve,
    /// and the dormant case returns `mf_sched` itself. The unlagged redline, if armed, min-selects
    /// on top and reads the CLIPPED `mf`.
    ///
    /// **THE STRUCTURAL FACT THIS METHOD EXISTS TO EXHIBIT**: while `required > g` the release
    /// constant is NEVER READ, so the entire march up to the first crossing is bit-identical
    /// across a `tau_rel` sweep. The leg pins its own trigger — everything rung 50 forced with
    /// `s_off`, a realisable limiter does for free. `g` and `required` are recorded per point, so
    /// this is the ONE route emitting 16 keys where every other emits 14.
    #[allow(clippy::too_many_arguments)]
    pub fn integrate_fuel_asym<S>(
        &self, flight: &FlightCondition, fuel_schedule: S, nu0: (f64, f64), s_end: f64, ds: f64,
        freeze: Option<Spool>, tt4_max: Option<f64>, accel: Option<&AccelSchedule>,
        surge: Option<&SurgeLimiter>, lag: &AsymmetricLag,
    ) -> Vec<FuelPoint>
    where
        S: Fn(f64) -> f64,
    {
        bump(&MARCH_CALLS);
        let required = |a: f64, b: f64, mf_sched: f64| -> Result<f64, Abort> {
            let mut caps: Vec<f64> = Vec::new();
            if let Some(accel) = accel {
                caps.push(self.try_sched_fuel(flight, a, b, mf_sched, accel)?);
            }
            if let Some(surge) = surge {
                caps.push(self.try_surge_fuel(flight, a, b, mf_sched, surge)?);
            }
            if caps.is_empty() {
                return Ok(0.0);
            }
            let mut m = caps[0];
            for &c in &caps[1..] {
                if c < m {
                    m = c;
                }
            }
            Ok(0.0f64.max(mf_sched - m))
        };
        let der = |a: f64, b: f64, g: f64, s: f64|
         -> Result<(f64, f64, f64, f64, FuelInstant, f64), Abort> {
            bump(&DER_CALLS);
            let mf_sched = fuel_schedule(s);
            let mut mf = mf_sched - g;
            if 1e-9 > mf {
                mf = 1e-9;
                bump(&MF_FLOOR_HITS);
            }
            if let Some(tt4_max) = tt4_max {
                // The UNLAGGED redline, min-selected on top -- and it reads the CLIPPED mf.
                if self.try_instant_fuel(flight, a, b, mf)?.base.tt4 > tt4_max {
                    let c = self.try_topping_fuel(flight, a, b, tt4_max, mf)?;
                    if c < mf {
                        mf = c;
                    }
                }
            }
            let i = self.try_instant_fuel(flight, a, b, mf)?;
            let req = required(a, b, mf_sched)?;
            let dg = (req - g) / lag.tau(req, g);
            let da = if freeze == Some(Spool::Lp) { 0.0 } else { i.base.phi_lp_dot / self.rho() };
            let db = if freeze == Some(Spool::Hp) { 0.0 } else { i.base.phi_hp_dot };
            Ok((da, db, dg, mf, i, req))
        };

        let mut pts: Vec<FuelPoint> = Vec::new();
        let (mut a, mut b) = nu0;
        // `s` ACCUMULATED, never `k as f64 * ds` — see the note in [`Self::integrate_fuel`]'s
        // loop: § 5.18 finding 3 measured a shipped boolean decided at one ulp of that difference.
        let (mut g, mut s) = (0.0f64, 0.0f64);
        let n_steps = (s_end / ds).round_ties_even() as i64;
        for _ in 0..=n_steps {
            let Ok((k1a, k1b, k1g, mf_app, inst, req)) = der(a, b, g, s) else {
                bump(&MARCH_BREAK_K1);
                break;
            };
            pts.push(point(s, a, b, &inst, mf_app, fuel_schedule(s),
                           PointExtra::Asym { g, required: req }));
            bump(&MARCH_POINTS);
            let stages = (|| -> Result<(f64, f64, f64, f64, f64, f64, f64, f64, f64), Abort> {
                let (k2a, k2b, k2g, ..) = der(
                    a + ds / 2.0 * k1a, b + ds / 2.0 * k1b, g + ds / 2.0 * k1g, s + ds / 2.0)?;
                let (k3a, k3b, k3g, ..) = der(
                    a + ds / 2.0 * k2a, b + ds / 2.0 * k2b, g + ds / 2.0 * k2g, s + ds / 2.0)?;
                let (k4a, k4b, k4g, ..) =
                    der(a + ds * k3a, b + ds * k3b, g + ds * k3g, s + ds)?;
                Ok((k2a, k2b, k2g, k3a, k3b, k3g, k4a, k4b, k4g))
            })();
            let Ok((k2a, k2b, k2g, k3a, k3b, k3g, k4a, k4b, k4g)) = stages else {
                bump(&MARCH_BREAK_RK);
                break;
            };
            a += ds / 6.0 * (k1a + 2.0 * k2a + 2.0 * k3a + k4a);
            b += ds / 6.0 * (k1b + 2.0 * k2b + 2.0 * k3b + k4b);
            g += ds / 6.0 * (k1g + 2.0 * k2g + 2.0 * k3g + k4g);
            s += ds;
        }
        pts
    }

    // --- the excursions ----------------------------------------------------------------------

    /// A FUEL ramp of nondimensional duration `r = tau_fuel/tau_H`.
    ///
    /// Reports the overshoot in the SPOOL-NEUTRAL currency `X = Tt4_peak - Tt4_hi`, because the
    /// running-line-referenced currencies are CIRCULAR — they read back whichever spool sits in
    /// the denominator. `E_temp_H`/`E_temp_L` are returned too, but ONLY so the circularity itself
    /// can be gated.
    #[allow(clippy::too_many_arguments)]
    pub fn ramp_excursion_fuel(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, r: f64, freeze: Option<Spool>,
        s_settle: f64, ds: f64,
    ) -> RampExcursionFuel {
        let mf_lo = self.fuel_for_tt4(flight, tt4_lo);
        let mf_hi = self.fuel_for_tt4(flight, tt4_hi);
        let eq0 = self.inner.equilibrium(flight, tt4_lo);
        let nu0 = (eq0.nu_lp, eq0.nu_hp);
        let schedule = |s: f64| -> f64 {
            if s <= 0.0 {
                mf_lo
            } else if s >= r {
                mf_hi
            } else {
                mf_lo + (mf_hi - mf_lo) * (s / r)
            }
        };
        let s_end = r + s_settle;
        let lim = FuelLimiters { freeze, ..Default::default() };
        let traj = self.integrate_fuel(flight, schedule, nu0, s_end, ds, &lim);
        assert!(!traj.is_empty(), "rung-43 fuel ramp produced no trajectory");
        let complete = traj[traj.len() - 1].s >= s_end - 2.5 * ds;
        let grid: Vec<f64> =
            (0..9).map(|k| tt4_lo + (tt4_hi - tt4_lo) * k as f64 / 8.0).collect();
        let rl: Vec<Instant2> =
            grid.iter().map(|&t| self.inner.equilibrium(flight, t)).collect();
        // Python sorts (nu, Tt4) TUPLES, so a tie on nu falls through to Tt4.
        let mut nl: Vec<(f64, f64)> = rl.iter().map(|p| (p.nu_lp, p.tt4)).collect();
        let mut nh: Vec<(f64, f64)> = rl.iter().map(|p| (p.nu_hp, p.tt4)).collect();
        nl.sort_by(|a, b| a.partial_cmp(b).expect("running-line rows are finite"));
        nh.sort_by(|a, b| a.partial_cmp(b).expect("running-line rows are finite"));
        let xs_l: Vec<f64> = nl.iter().map(|&(x, _)| x).collect();
        let ys_l: Vec<f64> = nl.iter().map(|&(_, y)| y).collect();
        let xs_h: Vec<f64> = nh.iter().map(|&(x, _)| x).collect();
        let ys_h: Vec<f64> = nh.iter().map(|&(_, y)| y).collect();
        let (mut e_th, mut e_tl) = (0.0f64, 0.0f64);
        let mut peak = tt4_lo;
        for p in &traj {
            peak = peak.max(p.tt4);
            e_th = e_th.max(p.tt4 / Self::interp(&xs_h, &ys_h, p.nu_hp) - 1.0);
            e_tl = e_tl.max(p.tt4 / Self::interp(&xs_l, &ys_l, p.nu_lp) - 1.0);
        }
        RampExcursionFuel {
            r, rho: self.rho(), tt4_peak: peak, x: peak - tt4_hi, e_temp_h: e_th, e_temp_l: e_tl,
            complete, traj,
        }
    }

    /// The `r → 0` limit: BOTH spools frozen at the low-power equilibrium, fuel jumps.
    ///
    /// No integration — a pure algebraic map property, hence EXACTLY `rho`-free. It is the
    /// `r_eff → 0` endpoint of the ramp family, not a separate object.
    pub fn constant_speed_excursion_fuel(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64,
    ) -> ConstantSpeedExcursionFuel {
        let eq0 = self.inner.equilibrium(flight, tt4_lo);
        let mf_hi = self.fuel_for_tt4(flight, tt4_hi);
        let inst = self.instant_fuel(flight, eq0.nu_lp, eq0.nu_hp, mf_hi);
        ConstantSpeedExcursionFuel {
            tt4_peak: inst.base.tt4,
            e_temp: inst.base.tt4 / tt4_lo - 1.0,
            e_lp: inst.base.close.pi_lpc / eq0.close.pi_lpc - 1.0,
            e_hp: inst.base.close.pi_hpc / eq0.close.pi_hpc - 1.0,
            f: inst.base.close.f,
        }
    }

    // --- THE MECHANISM -----------------------------------------------------------------------

    /// THE FINDING. March the same fuel ramp three ways — both spools free, LP frozen, HP frozen —
    /// and compare the peak `Tt4`.
    ///
    /// Freezing EITHER spool makes the overshoot WORSE: both sit in the one loop (`f` is set at
    /// the LP face, `Tt4` is metered at the HP-fed NGV throat) and both relieve it. The SHARE
    /// trades with `rho`. **SIGN / EXISTENCE only** — `d_lp` and `d_hp` do not sum to the total and
    /// are NOT calibrated weights. The LP-frozen march is the `rho → infinity` CEILING and is
    /// `rho`-independent bit-for-bit, since `rho` multiplies only the LP ODE.
    #[allow(clippy::too_many_arguments)]
    pub fn freeze_channels(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, r: f64, s_settle: f64, ds: f64,
    ) -> FreezeChannels {
        let peak = |fz: Option<Spool>| {
            self.ramp_excursion_fuel(flight, tt4_lo, tt4_hi, r, fz, s_settle, ds).tt4_peak
        };
        let both = peak(None);
        let lp = peak(Some(Spool::Lp));
        let hp = peak(Some(Spool::Hp));
        FreezeChannels { both, lp, hp, d_lp: lp - both, d_hp: hp - both, r, rho: self.rho() }
    }

    // --- the WITHDRAWN claim, kept measurable -------------------------------------------------

    /// Best-fit `q` for a would-be effective clock ratio `r_eff = r/rho^q`, by minimizing the mean
    /// relative spread of one currency within bins of common `r_eff`.
    ///
    /// `points` = `[(r, rho, value)]`. Pass `q = Some(..)` to EVALUATE that exponent instead of
    /// optimizing, so the single-spool clocks `q = 0` and `q = 1` score on the same metric.
    ///
    /// **This exists so a WITHDRAWN claim stays measurable and asserted-against**: the fitted `q`
    /// DIFFERS across currencies, because the referenced ones read back their own denominator.
    /// Rung 43 claims NO effective clock; this is the guard, not a result.
    ///
    /// **THE TIE-BREAK IS LOAD-BEARING AND THE RUNG'S OWN GATE IS BLIND TO IT.** The score is
    /// piecewise-constant in `q`, and every currency's minimum is attained by TWO adjacent `q` at
    /// a gap of exactly `0.000e+00`. Python's `min` keeps the FIRST of equals and so does
    /// [`Iterator::min_by`] — but `max_by` keeps the LAST, one keystroke away, and gate 9's
    /// ordering assertion is satisfied either way. Only the value dump can tell them apart. The
    /// NaN → `9e9` guard and the `if sp else nan` fall-back are both DEAD on every grid and both
    /// spelled.
    pub fn collapse_exponent(
        points: &[(f64, f64, f64)], nb: usize, q: Option<f64>,
    ) -> (f64, f64) {
        let spread = |q: f64| -> f64 {
            let mut rows: Vec<(f64, f64)> =
                points.iter().map(|&(r, rho, y)| (r / powp(rho, q), y)).collect();
            rows.sort_by(|a, b| a.partial_cmp(b).expect("collapse rows are finite"));
            let (lo, hi) = (rows[0].0.ln(), rows[rows.len() - 1].0.ln());
            let mut bins: Vec<Vec<f64>> = vec![Vec::new(); nb];
            for &(x, y) in &rows {
                let k = (nb - 1).min(((x.ln() - lo) / (hi - lo).max(1e-12) * nb as f64) as usize);
                bins[k].push(y);
            }
            let sp: Vec<f64> = bins
                .iter()
                .filter(|b| b.len() > 1)
                .map(|b| {
                    let mx = b.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                    let mn = b.iter().copied().fold(f64::INFINITY, f64::min);
                    (mx - mn) / (b.iter().sum::<f64>() / b.len() as f64).abs()
                })
                .collect();
            if sp.is_empty() {
                bump(&COLLAPSE_EMPTY);
                return f64::NAN;
            }
            sp.iter().sum::<f64>() / sp.len() as f64
        };
        if let Some(q) = q {
            return (q, spread(q));
        }
        // Python: `min(gen, key=lambda t: t[1] if t[1] == t[1] else 9e9)` -- FIRST of equals.
        let mut best: Option<(f64, f64, f64)> = None; // (q, spread, key)
        for i in 0..25 {
            let qq = i as f64 / 20.0;
            let sp = spread(qq);
            // Python's `t[1] if t[1] == t[1] else 9e9` -- a NaN test written as self-equality.
            // `is_nan()` IS `!= self` for `f64`, and clippy denies the literal spelling; the
            // crate already made this substitution once, in rung 40's off-map guard.
            let key = if !sp.is_nan() {
                sp
            } else {
                bump(&COLLAPSE_NAN);
                9e9
            };
            match best {
                None => best = Some((qq, sp, key)),
                Some((_, _, bk)) => {
                    if key == bk {
                        bump(&COLLAPSE_TIES);
                    }
                    if key < bk {
                        best = Some((qq, sp, key));
                    }
                }
            }
        }
        let (q, sp, _) = best.expect("25 samples");
        (q, sp)
    }

    // --- RUNG 45: the TRANSIENT surge line ON THE FUEL PATH -----------------------------------

    /// RUNG 45. March a FUEL ramp whose steady endpoints are the fuel-equivalents of
    /// `Tt4_lo → Tt4_hi`, and hand back the trajectory beside a COMMANDED running-line `phi`
    /// lookup.
    ///
    /// **THE REFERENCE IS THE COMMAND, NOT THE OUTPUT**, and that is rung 44's discipline applied
    /// where it bites: `Tt4_cmd(s)` is the LINEAR `Tt4` ramp the fuel command corresponds to, not
    /// the overshooting output. Referencing to the output would fold rung 43's `rho`-monotone
    /// overshoot into the baseline — a moving-reference currency trap, the surge-axis echo of
    /// rung 43's own currency circularity. On the `Tt4` path command == output, so this reduces
    /// to rung 44 EXACTLY.
    ///
    /// READ-ONLY: it marches and writes nothing, so an armed surge line is never touched.
    #[allow(clippy::too_many_arguments)]
    pub fn fuel_ramp_march(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, r: f64, s_settle: f64,
        ds: f64, lim: &FuelLimiters<'_>,
    ) -> (Vec<FuelPoint>, CommandedSteady) {
        let mf_lo = self.fuel_for_tt4(flight, tt4_lo);
        let mf_hi = self.fuel_for_tt4(flight, tt4_hi);
        let eq0 = self.inner.equilibrium(flight, tt4_lo);
        let nu0 = (eq0.nu_lp, eq0.nu_hp);
        let sched = |s: f64| -> f64 {
            if s <= 0.0 {
                mf_lo
            } else if s >= r {
                mf_hi
            } else {
                mf_lo + (mf_hi - mf_lo) * (s / r)
            }
        };
        let traj = self.integrate_fuel(flight, sched, nu0, r + s_settle, ds, lim);
        let (lo, hi) = (tt4_lo.min(tt4_hi), tt4_lo.max(tt4_hi));
        let grid: Vec<f64> = (0..9).map(|k| lo + (hi - lo) * k as f64 / 8.0).collect();
        let rl: Vec<Instant2> =
            grid.iter().map(|&t| self.inner.equilibrium(flight, t)).collect();
        let steady = CommandedSteady {
            grid,
            ys_l: rl.iter().map(|p| p.close.phi_lp).collect(),
            ys_h: rl.iter().map(|p| p.close.phi_hp).collect(),
            tt4_lo,
            tt4_hi,
            r,
        };
        (traj, steady)
    }

    /// RUNG 45. Signed extremum of `phi(s) - phi_steady(Tt4_cmd(s))` per spool over a marched FUEL
    /// ramp. NEGATIVE ⇔ below the running line ⇔ TOWARD surge.
    ///
    /// Accel: both spools TOWARD surge, the LP the larger magnitude. The LP-eats-more DOMINANCE
    /// **COMPRESSES** vs rung 44 (~1.2–1.7 against 1.6–2.2), because the `Tt4` overshoot loads the
    /// HP transient lag — so this object gates only the ORDERING; the strong LP asymmetry lives on
    /// the raw [`transient_surge_margin_fuel`](Self::transient_surge_margin_fuel). Needs NO surge
    /// line.
    /// **THE LIMITER SET IS NARROWER THAN [`integrate_fuel`](Self::integrate_fuel)'s, and that is
    /// Python's signature, not a simplification.** This method accepts `Tt4_max`, `tau_gov`,
    /// `accel` and `surge` — rungs 46 through 49 — and nothing else: passing `s_off`, `tau_rel`,
    /// `lag` or `freeze` is a `TypeError` in Python, so taking a whole [`FuelLimiters`] here
    /// would let a caller ask for something the source refuses. [`fuel_ramp_march`] below DOES
    /// take the whole set, because Python's does.
    ///
    /// [`fuel_ramp_march`]: Self::fuel_ramp_march
    #[allow(clippy::too_many_arguments)]
    pub fn phi_excursion_fuel(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, r: f64, s_settle: f64, ds: f64,
        tt4_max: Option<f64>, tau_gov: Option<f64>, accel: Option<&AccelSchedule>,
        surge: Option<SurgeLimiter>,
    ) -> PhiExcursionFuel {
        let lim = &FuelLimiters { tt4_max, tau_gov, accel, surge, ..Default::default() };
        let (traj, steady) =
            self.fuel_ramp_march(flight, tt4_lo, tt4_hi, r, s_settle, ds, lim);
        let (mut ext_lp, mut ext_hp) = (0.0f64, 0.0f64);
        let (mut s_lp, mut s_hp) = (0.0f64, 0.0f64);
        let (mut min_phi_lp, mut min_phi_hp) = (f64::INFINITY, f64::INFINITY);
        let mut tt4_peak = tt4_lo;
        for p in &traj {
            let e_lp = p.phi_lp - steady.at(p.s, Spool::Lp);
            let e_hp = p.phi_hp - steady.at(p.s, Spool::Hp);
            if e_lp.abs() > ext_lp.abs() {
                ext_lp = e_lp;
                s_lp = p.s;
            }
            if e_hp.abs() > ext_hp.abs() {
                ext_hp = e_hp;
                s_hp = p.s;
            }
            min_phi_lp = min_phi_lp.min(p.phi_lp);
            min_phi_hp = min_phi_hp.min(p.phi_hp);
            tt4_peak = tt4_peak.max(p.tt4);
        }
        PhiExcursionFuel {
            ext_lp, ext_hp, s_lp, s_hp, min_phi_lp, min_phi_hp, tt4_peak,
            ratio: if ext_hp != 0.0 { ext_lp.abs() / ext_hp.abs() } else { f64::INFINITY },
            npts: traj.len(),
        }
    }

    /// RUNG 45. March the FUEL ramp against the IMPOSED `phi_surge` and REPORT the crossing per
    /// spool — under the rung-36 discipline: report the crossing, gate the flip.
    ///
    /// **The RAW (reference-free) transient min `phi` IS the surge object**: it is what crosses
    /// `phi_surge`, and unlike the running-line-referenced excursion it is immune to the
    /// moving-reference trap. Its `rho`-invariance is the load-bearing finding — the `Tt4`
    /// overshoot is strongly `rho`-monotone yet does NOT reach `margin_min_lp`, so rung 44's
    /// "`rho` powerless over surge" SURVIVES the control swap on the reference-free object. Fuel
    /// ALSO drives the raw min deeper than `Tt4` control at the same ramp rate. The crossing DEPTH
    /// is disclaimed; the gated object is the flip's SIGN.
    /// The limiter set is rungs 46-49 only — see
    /// [`phi_excursion_fuel`](Self::phi_excursion_fuel)'s note.
    #[allow(clippy::too_many_arguments)]
    pub fn transient_surge_margin_fuel(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, r: f64, s_settle: f64, ds: f64,
        tt4_max: Option<f64>, tau_gov: Option<f64>, accel: Option<&AccelSchedule>,
        surge: Option<SurgeLimiter>,
    ) -> TransientSurgeMarginFuel {
        let lim = &FuelLimiters { tt4_max, tau_gov, accel, surge, ..Default::default() };
        let (ml, mh) = (self.inner.inner.map_lp, self.inner.inner.map_hp);
        assert!(ml.phi_surge > 0.0 && mh.phi_surge > 0.0,
                "transient_surge_margin_fuel needs a surge line on BOTH maps: build each with \
                 .with_phi_surge(phi_surge).");
        let (traj, steady) =
            self.fuel_ramp_march(flight, tt4_lo, tt4_hi, r, s_settle, ds, lim);
        let (mut tr_lp, mut tr_hp) = (f64::INFINITY, f64::INFINITY);
        let (mut st_lp, mut st_hp) = (f64::INFINITY, f64::INFINITY);
        let (mut min_phi_lp, mut min_phi_hp) = (f64::INFINITY, f64::INFINITY);
        for p in &traj {
            tr_lp = tr_lp.min(p.phi_lp - ml.phi_surge);
            tr_hp = tr_hp.min(p.phi_hp - mh.phi_surge);
            st_lp = st_lp.min(steady.at(p.s, Spool::Lp) - ml.phi_surge);
            st_hp = st_hp.min(steady.at(p.s, Spool::Hp) - mh.phi_surge);
            min_phi_lp = min_phi_lp.min(p.phi_lp);
            min_phi_hp = min_phi_hp.min(p.phi_hp);
        }
        TransientSurgeMarginFuel {
            margin_min_lp: tr_lp, margin_min_hp: tr_hp, steady_min_lp: st_lp, steady_min_hp: st_hp,
            min_phi_lp, min_phi_hp, crossed_lp: tr_lp < 0.0, crossed_hp: tr_hp < 0.0,
            phi_surge_lp: ml.phi_surge, phi_surge_hp: mh.phi_surge, npts: traj.len(),
        }
    }

    // =========================================================================================
    // RUNGS 46-48 — the READERS
    //
    // Four thin differencers over the marches above. § 5.16 deliberately left all thirteen
    // rung-46-52 readers unported (they carry no rung-43/45 gate); these are the four
    // `test_rung46-48.py` reach, and the other nine are slice U's.
    // =========================================================================================

    /// RUNG 46. March the SAME accel FUEL ramp twice — BARE and TOPPED (fuel clipped to hold
    /// `Tt4 <= tt4_max`) — and difference the surge object.
    ///
    /// **The finding is a SPLIT, not a relief.** Enforcing the TIT redline rebates surge margin on
    /// the LATE, non-binding HP spool and is MACHINE-ZERO on the EARLY, binding LP one: the surge
    /// debit is paid on early-ramp fuel, upstream of any window a redline-triggered governor can
    /// open. Rung 35's two accel limits are coupled in CAUSE but SEQUENCED in time.
    ///
    /// `tau_gov` (RUNG 47) gives the governor a response LAG. It changes only the TOPPED march —
    /// the bare stays governor-off — so the differential still isolates the governor. A lag is a
    /// TRAILING-edge tool: it cannot reach the early LP minimum, so it erodes the HP rebate and
    /// breaks the redline hold while buying nothing on the LP.
    ///
    /// Magnitudes are disclaimed (imposed maps and `phi_surge`, the fuel step, the band, the
    /// redline); load-bearing are the RELIEF SIGN, that `Tt4` is HELD, and the dormant reduce.
    pub fn topping_relief(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64, r: f64,
        s_settle: f64, ds: f64, tau_gov: Option<f64>,
    ) -> ToppingRelief {
        let bare = self.phi_excursion_fuel(
            flight, tt4_lo, tt4_hi, r, s_settle, ds, None, None, None, None);
        let top = self.phi_excursion_fuel(
            flight, tt4_lo, tt4_hi, r, s_settle, ds, Some(tt4_max), tau_gov, None, None);
        ToppingRelief {
            rho: self.rho(),
            r,
            tt4_max,
            tau_gov,
            tt4_peak_bare: bare.tt4_peak,
            tt4_peak_top: top.tt4_peak,
            overshoot: top.tt4_peak - tt4_max,
            held: top.tt4_peak <= tt4_max + 1e-6,
            min_phi_lp_bare: bare.min_phi_lp,
            min_phi_lp_top: top.min_phi_lp,
            min_phi_hp_bare: bare.min_phi_hp,
            min_phi_hp_top: top.min_phi_hp,
            relief_lp: top.min_phi_lp - bare.min_phi_lp,
            relief_hp: top.min_phi_hp - bare.min_phi_hp,
        }
    }

    /// RUNG 47 (secondary). March the rung-46 INSTANTANEOUS topped accel and read the applied
    /// fuel at each ENGAGED point — the min-select topping SET POINT, where `Tt4` is pinned at the
    /// redline.
    ///
    /// **This gates the valve-vs-loop-lag CONTRAST.** A pure metering-VALVE-position lag is inert
    /// on the accel precisely when this command rises monotonically, because an instant-up valve
    /// tracks a rising command with no lag. So the topping OVERSHOOT lives in the sensing /
    /// limiter-LOOP lag — which lags the clip AMOUNT — and not in the valve. WHERE the lag lives
    /// decides whether it overshoots at all.
    ///
    /// **The engagement selector is `|Tt4 - tt4_max| < 1e-6`, and § 5.17 finding 2 measured it
    /// uncontested by 1.06e6**: an engaged point sits within 9.1e-13 of the redline, the nearest
    /// unengaged one 1.064 K away.
    pub fn topping_command_trace(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64, r: f64,
        s_settle: f64, ds: f64,
    ) -> ToppingCommandTrace {
        let lim = &FuelLimiters { tt4_max: Some(tt4_max), ..Default::default() };
        let (traj, _) = self.fuel_ramp_march(flight, tt4_lo, tt4_hi, r, s_settle, ds, lim);
        let engaged: Vec<(f64, f64)> = traj.iter()
            .filter(|p| (p.tt4 - tt4_max).abs() < 1e-6)
            .map(|p| (p.s, p.mf))
            .collect();
        // Python: `all(eng[i][1] >= eng[i-1][1] - 1e-12 for i in range(1, len(eng)))` — vacuously
        // true on 0 or 1 engaged points, which `windows(2)` reproduces without a special case.
        let monotone_nondecreasing =
            engaged.windows(2).all(|w| w[1].1 >= w[0].1 - 1e-12);
        ToppingCommandTrace {
            n_engaged: engaged.len(),
            engaged,
            monotone_nondecreasing,
            tt4_max,
            r,
        }
    }

    /// RUNG 48. March the SAME accel FUEL ramp twice — BARE and with the `Wf/pt3` leg armed — and
    /// difference the reference-free surge object, exactly as
    /// [`topping_relief`](Self::topping_relief) does for the TIT governor.
    ///
    /// **THE RUNG IS FOUR OF THE EIGHTEEN KEYS.** The finding is the crossing
    /// `relief_* > 0 ⟺ s_eng < s_*`: a fuel-side limiter rebates a spool IFF it engages UPSTREAM
    /// of THAT spool's own minimum. [`fuel_removed`](ScheduleRelief::fuel_removed) and
    /// [`nu_hp_end`](ScheduleRelief::nu_hp_end) are what exclude the deflation that this is rung
    /// 44's ramp-rate lever restated — they vary SMOOTHLY through the crossing at which the relief
    /// switches EXACTLY off, and at a margin where `relief_lp` is exactly 0 the SAME clip still
    /// rebates the HP.
    ///
    /// `tt4_max` / `tau_gov` arm rungs 46/47's governor ON TOP (the min-select composite); the
    /// bare leg stays governor-free so the differential isolates the `Wf/pt3` leg.
    #[allow(clippy::too_many_arguments)]
    pub fn schedule_relief(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, accel: &AccelSchedule, r: f64,
        s_settle: f64, ds: f64, tt4_max: Option<f64>, tau_gov: Option<f64>,
    ) -> ScheduleRelief {
        let bare_lim = &FuelLimiters::default();
        let lim_lim = &FuelLimiters { tt4_max, tau_gov, accel: Some(accel), ..Default::default() };
        let (bare, _) = self.fuel_ramp_march(flight, tt4_lo, tt4_hi, r, s_settle, ds, bare_lim);
        let (lim, _) = self.fuel_ramp_march(flight, tt4_lo, tt4_hi, r, s_settle, ds, lim_lim);
        assert!(!bare.is_empty() && !lim.is_empty(),
                "rung-48 schedule_relief produced no trajectory");

        // Python's `min(traj, key=...)` returns the FIRST minimum on ties, so the fold is STRICT —
        // see [`first_raw_min`], which is module-level PRECISELY so that rule can be gated on a
        // manufactured tie, § 5.17 finding 5 having measured that no suite cell carries one.
        let (mpl_b, s_lp) = first_raw_min(&bare, |p| p.phi_lp);
        let (mph_b, s_hp) = first_raw_min(&bare, |p| p.phi_hp);
        let (mpl_l, _) = first_raw_min(&lim, |p| p.phi_lp);
        let (mph_l, _) = first_raw_min(&lim, |p| p.phi_hp);

        // The trapezoid, in Python's instruction order: `0.5 * h * (prev + cur)` accumulated
        // ascending. NOT hoisted, NOT rearranged — the port's `copy-vs-rederivation` rule, which
        // has cost it an "exactly" claim before.
        let mut removed = 0.0f64;
        for i in 1..lim.len() {
            let h = lim[i].s - lim[i - 1].s;
            removed += 0.5 * h * ((lim[i - 1].mf_sched - lim[i - 1].mf)
                                  + (lim[i].mf_sched - lim[i].mf));
        }
        let eng: Vec<f64> = lim.iter()
            .filter(|p| p.mf < p.mf_sched * (1.0 - 1e-9))
            .map(|p| p.s)
            .collect();
        let peak = |t: &[FuelPoint]| t.iter().fold(f64::NEG_INFINITY, |a, p| a.max(p.tt4));
        ScheduleRelief {
            margin: accel.margin,
            r,
            rho: self.rho(),
            s_eng: eng.first().copied().unwrap_or(f64::NAN),
            n_engaged: eng.len(),
            s_lp_bare: s_lp,
            s_hp_bare: s_hp,
            relief_lp: mpl_l - mpl_b,
            relief_hp: mph_l - mph_b,
            min_phi_lp_bare: mpl_b,
            min_phi_lp_lim: mpl_l,
            min_phi_hp_bare: mph_b,
            min_phi_hp_lim: mph_l,
            fuel_removed: removed,
            tt4_peak_bare: peak(&bare),
            tt4_peak_lim: peak(&lim),
            nu_hp_end: lim[lim.len() - 1].nu_hp,
            nu_hp_end_bare: bare[bare.len() - 1].nu_hp,
        }
    }

    /// RUNG 48 (the finding method). Sweep the schedule margin `m` and report, per `m`, the
    /// engagement time and both reliefs.
    ///
    /// **`m` is an ENGAGEMENT-TIME instrument.** The bare march's `(Wf/pt3)/kappa_ss` ratio rises
    /// MONOTONICALLY through both surge minima, so `m` maps continuously to `s_eng(m)` — one
    /// scalar moves the clip ACROSS the minima with the plant, the band, the ramp rate and the
    /// endpoint all held fixed. Watch `relief_lp` fall to EXACTLY 0 as `s_eng` passes `s_lp`,
    /// while `relief_hp` is still positive and dies only as `s_eng` reaches `s_hp`.
    ///
    /// **The `m → 0` corner is the HONEST BOUNDARY, reported not hidden**: there the leg binds
    /// from the start and never releases, and the leg HAS degenerated into rung 44's ramp-rate
    /// lever. Read the crossing only where `nu_hp_end` is unmoved. § 5.17 finding 7 measured that
    /// this corner COMPLETES rather than refusing — at `m = 0.02` the march runs to the end with
    /// `nu_hp_end` 1.4e-1 below bare — so a Rust-side refusal here is a defect, not a divergence.
    #[allow(clippy::too_many_arguments)]
    pub fn engagement_sweep(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, margins: &[f64], r: f64,
        s_settle: f64, ds: f64, n: usize,
    ) -> Vec<ScheduleRelief> {
        margins.iter()
            .map(|&m| {
                let acc = self.accel_schedule(flight, tt4_lo, tt4_hi, m, n);
                self.schedule_relief(flight, tt4_lo, tt4_hi, &acc, r, s_settle, ds, None, None)
            })
            .collect()
    }

    // --- RUNG 49: the phi floor — read BOTH edges, and both spools ----------------------------

    /// RUNG 49 (the finding method). March the SAME accel FUEL ramp twice — BARE and with the
    /// `phi` FLOOR armed — and difference rung 45's reference-free surge object (raw min `phi`),
    /// exactly as rungs 46/48's `topping_relief` / [`schedule_relief`](Self::schedule_relief) do
    /// for their legs.
    ///
    /// Reports BOTH edges of the engaged window (`s_eng`, `s_rel`) — the point of the rung. A
    /// `pt3`-filter limiter's `s_rel` is structurally POST-ramp (`docs/both-edges-limiter-
    /// negative.md`); a `phi` floor's can close INSIDE it, and when it does the closing edge
    /// RE-OPENS the unwatched spool's descent.
    ///
    /// **THE FINDING** is the SPLIT at fixed clip: `relief_watched > 0` (the truncated descent,
    /// rung 48's term) while `relief_other < 0` (the re-opened one, new).
    /// [`s_min_other`](SurgeRelief::s_min_other) locates the unwatched minimum — it sits just
    /// AFTER `s_rel`, which is the mechanism. `fuel_removed` / `nu_hp_end` are the anti-deflation
    /// pair (rung 48's discipline).
    ///
    /// **IT KEEPS BOTH LIMITED ARGMIN LOCATIONS WHERE [`schedule_relief`](Self::schedule_relief)
    /// DISCARDS THEM.** Rung 48 folds `first_raw_min(&lim, …)` and drops the `s`; rung 49's
    /// `s_min_other` IS that `s`, picked by which spool the leg watches. Copying the neighbour's
    /// body and binding `_` there is the one mechanical way to get this method subtly wrong.
    #[allow(clippy::too_many_arguments)]
    pub fn surge_relief(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, surge: &SurgeLimiter, r: f64,
        s_settle: f64, ds: f64, tt4_max: Option<f64>, tau_gov: Option<f64>,
        accel: Option<&AccelSchedule>,
    ) -> SurgeRelief {
        let bare_lim = &FuelLimiters::default();
        let lim_lim =
            &FuelLimiters { tt4_max, tau_gov, accel, surge: Some(*surge), ..Default::default() };
        let (bare, _) = self.fuel_ramp_march(flight, tt4_lo, tt4_hi, r, s_settle, ds, bare_lim);
        let (lim, _) = self.fuel_ramp_march(flight, tt4_lo, tt4_hi, r, s_settle, ds, lim_lim);
        assert!(!bare.is_empty() && !lim.is_empty(),
                "rung-49 surge_relief produced no trajectory");

        // STRICT fold, first-on-tie — [`first_raw_min`], shared with rung 48 and gated on a
        // manufactured tie in `topping_oracle.rs` because no marched cell carries one.
        let (mpl_b, s_lp) = first_raw_min(&bare, |p| p.phi_lp);
        let (mph_b, s_hp) = first_raw_min(&bare, |p| p.phi_hp);
        let (mpl_l, s_lp_l) = first_raw_min(&lim, |p| p.phi_lp);
        let (mph_l, s_hp_l) = first_raw_min(&lim, |p| p.phi_hp);

        // Python's trapezoid, in Python's instruction order — NOT hoisted, NOT rearranged.
        let mut removed = 0.0f64;
        for i in 1..lim.len() {
            let h = lim[i].s - lim[i - 1].s;
            removed += 0.5 * h * ((lim[i - 1].mf_sched - lim[i - 1].mf)
                                  + (lim[i].mf_sched - lim[i].mf));
        }
        let eng: Vec<f64> = lim.iter()
            .filter(|p| p.mf < p.mf_sched * (1.0 - 1e-9))
            .map(|p| p.s)
            .collect();
        let watched_lp = surge.spool == Spool::Lp;
        // The largest deviation of the WATCHED `phi` from its floor over the engaged window. The
        // filter is re-spelled rather than reusing `eng` because Python re-spells it: `eng` holds
        // the `s` values, this comprehension needs the POINTS.
        let hold = lim.iter()
            .filter(|p| p.mf < p.mf_sched * (1.0 - 1e-9))
            .map(|p| (surge.read_point(p) - surge.phi_lim).abs())
            .fold(0.0f64, f64::max);
        let peak = |t: &[FuelPoint]| t.iter().fold(f64::NEG_INFINITY, |a, p| a.max(p.tt4));
        SurgeRelief {
            phi_lim: surge.phi_lim,
            spool: surge.spool,
            r,
            rho: self.rho(),
            s_eng: eng.first().copied().unwrap_or(f64::NAN),
            s_rel: eng.last().copied().unwrap_or(f64::NAN),
            n_engaged: eng.len(),
            both_edges_inside_ramp: !eng.is_empty()
                && 0.0 < eng[0]
                && eng[eng.len() - 1] < r,
            hold_err: hold,
            s_lp_bare: s_lp,
            s_hp_bare: s_hp,
            relief_lp: mpl_l - mpl_b,
            relief_hp: mph_l - mph_b,
            relief_watched: if watched_lp { mpl_l - mpl_b } else { mph_l - mph_b },
            relief_other: if watched_lp { mph_l - mph_b } else { mpl_l - mpl_b },
            s_min_other: if watched_lp { s_hp_l } else { s_lp_l },
            min_phi_lp_bare: mpl_b,
            min_phi_lp_lim: mpl_l,
            min_phi_hp_bare: mph_b,
            min_phi_hp_lim: mph_l,
            fuel_removed: removed,
            tt4_peak_bare: peak(&bare),
            tt4_peak_lim: peak(&lim),
            nu_hp_end: lim[lim.len() - 1].nu_hp,
            nu_hp_end_bare: bare[bare.len() - 1].nu_hp,
        }
    }

    /// RUNG 49. Sweep the `phi` floor and report, per floor, both window edges and both reliefs.
    ///
    /// **`phi_lim` is a WINDOW instrument where rung 48's `m` was an ENGAGEMENT-TIME one**: a
    /// tighter floor engages EARLIER **and** releases LATER, so it opens the window at both ends
    /// at once. `relief_watched` rises monotonically (it is the definitional
    /// `phi_lim − min phi_bare`) while `relief_other` goes NEGATIVE and peaks in magnitude where
    /// `s_rel` lands at the RAMP END — the two edges answering to different clocks.
    ///
    /// **THE HONEST BOUNDARY, reported not hidden**: a floor at or above the INITIAL running-line
    /// `phi` binds from `s = 0` and never releases (`s_eng == 0`), the accel does not complete
    /// (`nu_hp_end` falls away from `nu_hp_end_bare`) and the leg HAS degenerated into rung 44's
    /// ramp-rate lever. Read the split only where `nu_hp_end` is unmoved.
    #[allow(clippy::too_many_arguments)]
    pub fn floor_sweep(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, floors: &[f64], spool: Spool,
        r: f64, s_settle: f64, ds: f64,
    ) -> Vec<SurgeRelief> {
        floors.iter()
            .map(|&p| {
                let leg = SurgeLimiter::new(spool, p);
                self.surge_relief(flight, tt4_lo, tt4_hi, &leg, r, s_settle, ds, None, None, None)
            })
            .collect()
    }

    // --- RUNG 50/51: the FORCED release edge, and its RATE --------------------------------

    /// RUNG 50 (the finding method). March the SAME accel fuel ramp twice — BARE and with a
    /// min-select leg armed but FORCED to disarm at `s_off` — and difference rung 45's
    /// reference-free surge object, exactly as rungs 46/48/49's relief methods do.
    ///
    /// **WHY `s_off` AND NOT A LAG.** Rungs 48/49 could move a limiter's release edge only by
    /// moving `m` / `phi_lim`, which drags the ENGAGEMENT edge, the window length and the clip
    /// depth along with it — so rung 49 § 3's clock result had to be hedged as WITHIN-FAMILY.
    /// `s_off` slides the release alone, TWO-SIDED, with everything up to it bit-identical. It is
    /// an isolation diagnostic in the project's own tradition (`freeze = Lp` holds a spool's speed
    /// against its own ODE); neither is a control law.
    ///
    /// **THE FINDING**: the release edge RELOCATES BOTH SPOOLS' MINIMA TO ITSELF —
    /// `s_min_lp` / `s_min_hp` == `s_rel` to a grid cell — whenever the DIVE BRANCH WINS on that
    /// spool, which is the conjunction of (a) the release landing at or AFTER that spool's own
    /// bare minimum and (b) that spool's relief being NEGATIVE.
    ///
    /// `s_off = None` reproduces the unforced leg exactly (rung 49 / rung 48). `tau_rel` (RUNG 51)
    /// fades the release over `[s_off, s_off + tau_rel]` instead of stepping it; `tau_rel = None`
    /// is bit-for-bit rung 50. **It lands COMPLETE here rather than at rung 51's step** because it
    /// is a kwarg of this method and not a separate path — § 5.18 P6 — and NO rung-50 cell passes
    /// it, so it is `Option`-typed and value-inert until `tests/rung51.rs`.
    ///
    /// # THE COPY TRAP IS NOT [`surge_relief`](Self::surge_relief)'s, IT IS ITS MIRROR IMAGE
    ///
    /// Rung 49 collects `eng` as the `s` VALUES and folds `hold_err` over a re-spelled filter on
    /// the POINTS. This method needs the opposite: `eng` holds the POINTS, because
    /// `deficit_at_release` reads `eng[-1]`'s `mf` / `mf_sched`, and `s_eng` / `s_rel` take `.s`
    /// off them. Copying rung 49's body gives the wrong collection type and no way to compute the
    /// deficit at all. **And Python builds `eng` BEFORE the trapezoid here and AFTER it in rung
    /// 49** — each file's statement order is kept, not unified.
    ///
    /// # THE FORCED-RELEASE COMPARISON SITE IS A ONE-*CELL* KNIFE EDGE, NOT A ONE-ULP ONE
    ///
    /// [`release_weight`] tests `s < s_off` against the ACCUMULATED march coordinate, and every
    /// `s_off` the suite passes sits ON the `ds` grid — the one place accumulated `s` and a
    /// "cleaner" `k * ds` can straddle the bar. Swept over the suite's eighteen `s_off` values at
    /// `ds ∈ {0.02, 0.01}`, **six comparison sites change the last armed index by a WHOLE GRID
    /// CELL** under the two spellings, and two of them are live cells of this suite: at
    /// `ds = 0.02`, `s_off = 0.20` accumulates to `0.19999999999999998` and `s_off = 0.26` to
    /// `0.25999999999999995`, so the leg stays armed one point LONGER than `k * ds` would keep it.
    /// Gates 5 and 10b read exactly those rows. This is a coarser hazard than § 5.18 finding 3's
    /// one-ulp boolean and it lands on a different site, so it is noted here as well as at the
    /// march loop.
    #[allow(clippy::too_many_arguments)]
    pub fn release_relief(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, s_off: Option<f64>,
        surge: Option<&SurgeLimiter>, accel: Option<&AccelSchedule>, r: f64, s_settle: f64,
        ds: f64, tau_rel: Option<f64>,
    ) -> ReleaseRelief {
        assert!(surge.is_some() || accel.is_some(),
                "rung-50 release_relief needs a leg to release: pass surge= and/or accel=.");
        assert!(s_off.is_none_or(|x| x > 0.0),
                "rung-50 s_off is a release TIME on the march");
        let bare_lim = &FuelLimiters::default();
        let lim_lim = &FuelLimiters {
            accel, surge: surge.copied(), s_off, tau_rel, ..Default::default()
        };
        let (bare, _) = self.fuel_ramp_march(flight, tt4_lo, tt4_hi, r, s_settle, ds, bare_lim);
        let (lim, _) = self.fuel_ramp_march(flight, tt4_lo, tt4_hi, r, s_settle, ds, lim_lim);
        assert!(!bare.is_empty() && !lim.is_empty(),
                "rung-50 release_relief produced no trajectory");

        // STRICT fold, first-on-tie — [`first_raw_min`], shared with rungs 48/49 and gated on a
        // manufactured tie in `topping_oracle.rs` because no marched cell carries one.
        let (mpl_b, s_lp) = first_raw_min(&bare, |p| p.phi_lp);
        let (mph_b, s_hp) = first_raw_min(&bare, |p| p.phi_hp);
        let (mpl_l, s_lp_l) = first_raw_min(&lim, |p| p.phi_lp);
        let (mph_l, s_hp_l) = first_raw_min(&lim, |p| p.phi_hp);

        // `eng` holds the POINTS here — see the copy-trap note above — and Python builds it
        // BEFORE the trapezoid in this method (and AFTER it in rung 49's).
        let eng: Vec<&FuelPoint> =
            lim.iter().filter(|p| p.mf < p.mf_sched * (1.0 - 1e-9)).collect();

        // Python's trapezoid, in Python's instruction order — NOT hoisted, NOT rearranged.
        let mut removed = 0.0f64;
        for i in 1..lim.len() {
            let h = lim[i].s - lim[i - 1].s;
            removed += 0.5 * h * ((lim[i - 1].mf_sched - lim[i - 1].mf)
                                  + (lim[i].mf_sched - lim[i].mf));
        }

        // The INSTANTANEOUS fractional clip at the LAST engaged point — the "deficit at release".
        // **`0.0` is Python's no-engagement sentinel AND a legitimate value**, where the same
        // row's `s_eng` / `s_rel` use `NaN`: two sentinels for one condition in one record
        // (§ 5.18 finding 4). Copied, not repaired — read `n_engaged` to separate them.
        let deficit = match eng.last() {
            Some(last) => (last.mf_sched - last.mf) / last.mf_sched,
            None => 0.0,
        };
        let watched = surge.map(|s| s.spool);
        ReleaseRelief {
            s_off,
            tau_rel,
            r,
            rho: self.rho(),
            ds,
            spool: watched,
            phi_lim: surge.map(|s| s.phi_lim),
            margin: accel.map(|a| a.margin),
            s_eng: eng.first().map_or(f64::NAN, |p| p.s),
            s_rel: eng.last().map_or(f64::NAN, |p| p.s),
            n_engaged: eng.len(),
            deficit_at_release: deficit,
            s_lp_bare: s_lp,
            s_hp_bare: s_hp,
            relief_lp: mpl_l - mpl_b,
            relief_hp: mph_l - mph_b,
            relief_watched: watched
                .map(|w| if w == Spool::Lp { mpl_l - mpl_b } else { mph_l - mph_b }),
            relief_other: watched
                .map(|w| if w == Spool::Lp { mph_l - mph_b } else { mpl_l - mpl_b }),
            s_min_lp: s_lp_l,
            s_min_hp: s_hp_l,
            min_phi_lp_bare: mpl_b,
            min_phi_lp_lim: mpl_l,
            min_phi_hp_bare: mph_b,
            min_phi_hp_lim: mph_l,
            fuel_removed: removed,
            nu_hp_end: lim[lim.len() - 1].nu_hp,
            nu_hp_end_bare: bare[bare.len() - 1].nu_hp,
        }
    }

    /// RUNG 50. Sweep the FORCED release time at a FIXED leg — the deconfounded axis.
    ///
    /// `relief_hp` (or `relief_other`) deepens monotonically as `s_off` walks THROUGH the
    /// unwatched spool's own minimum without noticing it, peaks with the release just inside the
    /// RAMP END, and collapses past it. That ordering is rung 49 § 3's clock claim with the
    /// engagement edge and the clip depth held fixed.
    ///
    /// Pass `s_offs` on the `ds` grid (the switch otherwise straddles a step) — and see
    /// [`release_relief`](Self::release_relief)'s note on why "on the grid" is where the
    /// accumulated coordinate is at its sharpest, not its safest.
    ///
    /// **NO `tau_rel`.** Python's `release_sweep` does not forward it, so a rate sweep is rung
    /// 51's `rate_sweep` and not this loop with an extra argument.
    #[allow(clippy::too_many_arguments)]
    pub fn release_sweep(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, s_offs: &[f64],
        surge: Option<&SurgeLimiter>, accel: Option<&AccelSchedule>, r: f64, s_settle: f64,
        ds: f64,
    ) -> Vec<ReleaseRelief> {
        s_offs.iter()
            .map(|&so| {
                self.release_relief(flight, tt4_lo, tt4_hi, Some(so), surge, accel, r, s_settle,
                                    ds, None)
            })
            .collect()
    }

    /// RUNG 51 (the finding method). Sweep the release RATE at a FIXED trigger `s_off`.
    ///
    /// Rung 50 moved WHEN the withheld fuel is handed back and found the debit monotone in the
    /// DEFICIT at fixed release. It could not move HOW FAST, and said so. This is that axis:
    /// everything up to `s_off` is BIT-IDENTICAL across the sweep (the clip only starts fading
    /// there), so the trigger, the engagement edge and the whole engaged window are held fixed
    /// while the hand-back rate alone varies.
    ///
    /// **DO NOT READ THE SWEEP ALONE.** `fuel_removed` RISES with `tau_rel` (the clip is held
    /// partially on for longer), so the sweep moves the deficit and the rate TOGETHER — the same
    /// confound rung 49 § 4 fell into. The gate is a TWO-SIDED BRACKET against the two HARD
    /// releases at the ends of the fade's own interval, not
    /// [`deficit_curve`](Self::deficit_curve).
    ///
    /// # § 5.18 P6 — THIS ADDS NO NEW LOGIC, AND THAT IS THE CHECK
    ///
    /// `tau_rel` is a kwarg of [`release_relief`](Self::release_relief), not a separate path, so
    /// that method landed COMPLETE at rung 50's step and this is a loop over it. The prediction was
    /// registered before either was written; it is discharged by this body being three statements
    /// long, exactly as slice T registered its zero-source-line steps.
    ///
    /// `s_off` is a plain `f64` because Python's annotation is `s_off: float` and no caller passes
    /// `None` — widening it to an `Option` here would be a wider API than the source's, in the
    /// direction the port is told not to drift.
    #[allow(clippy::too_many_arguments)]
    pub fn rate_sweep(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, s_off: f64,
        tau_rels: &[Option<f64>], surge: Option<&SurgeLimiter>, accel: Option<&AccelSchedule>,
        r: f64, s_settle: f64, ds: f64,
    ) -> Vec<ReleaseRelief> {
        assert!(tau_rels.iter().all(|t| t.is_none_or(|v| v >= 0.0)),
                "rung-51 tau_rel is a fade DURATION on the march coordinate");
        tau_rels.iter()
            .map(|&t| {
                self.release_relief(flight, tt4_lo, tt4_hi, Some(s_off), surge, accel, r, s_settle,
                                    ds, t)
            })
            .collect()
    }

    /// RUNG 51. Rung 50 § 5's fixed-release deficit→depth curve, rebuilt cleanly: rung 50 had to
    /// hand-pick `phi_lim` values whose NATURAL releases happened to coincide, whereas `s_off`
    /// pins the release by construction, so sweeping the floor walks the deficit at a genuinely
    /// FIXED (and hard) release. Every row is a rung-50 point (`tau_rel = None`).
    ///
    /// **NOT THE GATE FOR [`rate_sweep`](Self::rate_sweep), AND KEPT BECAUSE FINDING THAT OUT WAS
    /// THE WORK.** This curve was rung 51's pre-registered gate and it is CONFOUNDED: at matched
    /// release-COMPLETION a faded run always removes LESS fuel than the hard one, and rung 50 § 5
    /// already says less deficit ⇒ shallower dive, so "shallower at matched completion" proves
    /// nothing. The two-sided bracket replaced it. Ported because the source ships it, not because
    /// a gate needs it — *COPY vs REDERIVATION*.
    #[allow(clippy::too_many_arguments)]
    pub fn deficit_curve(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, s_off: f64, floors: &[f64],
        spool: Spool, r: f64, s_settle: f64, ds: f64,
    ) -> Vec<ReleaseRelief> {
        floors.iter()
            .map(|&p| {
                let leg = SurgeLimiter::new(spool, p);
                self.release_relief(flight, tt4_lo, tt4_hi, Some(s_off), Some(&leg), None, r,
                                    s_settle, ds, None)
            })
            .collect()
    }

    // --- RUNG 52: the asymmetric fast-attack / slow-release LAG --------------------------------

    /// RUNG 52 (the finding method). March the SAME accel fuel ramp twice — BARE and with a
    /// min-select leg whose clip is carried under an [`AsymmetricLag`] — and difference rung 45's
    /// reference-free surge object, exactly as rungs 46/48/49/50's relief methods do.
    ///
    /// **THE OBJECT RUNGS 50/51 COULD NOT REACH.** `s_off` / `tau_rel` FORCE a release because
    /// rung 49's family could not pin one; this leg pins its OWN. `s_cross` — the first point
    /// where `required` falls back through the clip state `g` — is the natural release trigger,
    /// and it is INVARIANT in `lag.tau_rel`, structurally, because `tau_rel` is not read before
    /// it. Sweep the rate and everything upstream is BIT-IDENTICAL.
    ///
    /// **BECAUSE AN EXPONENTIAL NEVER COMPLETES, THE RELEASE EDGE IS DECLARED, NOT DETECTED**:
    /// `s_rel_<eps>` is the last point whose fractional clip is at least `eps`. Reported at every
    /// `eps` in the slice so that no verdict rests on a threshold.
    ///
    /// # TWO TRAPS IN EIGHT LINES OF CROSSING LOOP, BOTH REGISTERED BEFORE THE PORT
    ///
    /// § 5.18 finding 2 measured both, so neither is "tidied":
    ///
    /// 1. **`armed` is `Option<bool>`, seeded `None`.** Python seeds `armed = None` and guards
    ///    `if armed is False`, so the FIRST crossing is not counted as a re-crossing. The natural
    ///    `let mut armed = false` counts it and puts [`n_recross`](LagRelief::n_recross) one high
    ///    on every row — and `test_rung52.py:224` asserts `n_recross == 1`, which the WRONG seed
    ///    also satisfies on every marched cell, because the first point with `g > 0` is always
    ///    still attacking. The seed is gated on a MANUFACTURED trajectory at step 5, not here.
    /// 2. **The `g <= 0.0` arm CONTINUES, it does not disarm.** An unclipped point leaves `armed`
    ///    alone, so folding the guard into one `if / else` is wrong.
    #[allow(clippy::too_many_arguments)]
    pub fn lag_relief(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, lag: AsymmetricLag,
        surge: Option<&SurgeLimiter>, accel: Option<&AccelSchedule>, r: f64, s_settle: f64,
        ds: f64, eps: &[f64],
    ) -> LagRelief {
        assert!(surge.is_some() || accel.is_some(),
                "rung-52 lag_relief needs a leg to lag: pass surge= and/or accel=.");
        let bare_lim = &FuelLimiters::default();
        let lim_lim = &FuelLimiters {
            accel, surge: surge.copied(), lag: Some(lag), ..Default::default()
        };
        let (bare, _) = self.fuel_ramp_march(flight, tt4_lo, tt4_hi, r, s_settle, ds, bare_lim);
        let (lim, _) = self.fuel_ramp_march(flight, tt4_lo, tt4_hi, r, s_settle, ds, lim_lim);
        assert!(!bare.is_empty() && !lim.is_empty(),
                "rung-52 lag_relief produced no trajectory");

        let (mpl_b, s_lp) = first_raw_min(&bare, |p| p.phi_lp);
        let (mph_b, s_hp) = first_raw_min(&bare, |p| p.phi_hp);
        let (mpl_l, s_lp_l) = first_raw_min(&lim, |p| p.phi_lp);
        let (mph_l, s_hp_l) = first_raw_min(&lim, |p| p.phi_hp);

        // THE CROSSING — lifted into [`crossing_census`] so the two rules in it are reachable
        // without a march. See that function; the lift is behaviour-neutral.
        let asym = asym_extra;
        let (cross_i, n_recross) = crossing_census(&lim);
        let cross: Option<&FuelPoint> = cross_i.map(|i| &lim[i]);

        // Python's trapezoid, in Python's instruction order.
        let mut removed = 0.0f64;
        for i in 1..lim.len() {
            let h = lim[i].s - lim[i - 1].s;
            removed += 0.5 * h * ((lim[i - 1].mf_sched - lim[i - 1].mf)
                                  + (lim[i].mf_sched - lim[i].mf));
        }
        let watched = surge.map(|s| s.spool);
        let peak = |t: &[FuelPoint]| t.iter().fold(f64::NEG_INFINITY, |a, p| a.max(p.tt4));
        let g_peak = lim.iter().map(asym).fold(f64::NEG_INFINITY, |a, (g, _)| a.max(g));

        let eps_edges: Vec<(f64, f64, f64)> = eps.iter()
            .map(|&e| {
                let on: Vec<f64> = lim.iter()
                    .filter(|p| (p.mf_sched - p.mf) / p.mf_sched >= e)
                    .map(|p| p.s)
                    .collect();
                (e,
                 on.first().copied().unwrap_or(f64::NAN),
                 on.last().copied().unwrap_or(f64::NAN))
            })
            .collect();

        LagRelief {
            tau_att: lag.tau_att,
            tau_rel: lag.tau_rel,
            r,
            rho: self.rho(),
            ds,
            spool: watched,
            phi_lim: surge.map(|s| s.phi_lim),
            margin: accel.map(|a| a.margin),
            s_cross: cross.map_or(f64::NAN, |p| p.s),
            g_at_cross: cross.map_or(f64::NAN, |p| asym(p).0),
            required_at_cross: cross.map_or(f64::NAN, |p| asym(p).1),
            n_recross,
            g_peak,
            s_lp_bare: s_lp,
            s_hp_bare: s_hp,
            relief_lp: mpl_l - mpl_b,
            relief_hp: mph_l - mph_b,
            relief_watched: watched
                .map(|w| if w == Spool::Lp { mpl_l - mpl_b } else { mph_l - mph_b }),
            relief_other: watched
                .map(|w| if w == Spool::Lp { mph_l - mph_b } else { mpl_l - mpl_b }),
            s_min_lp: s_lp_l,
            s_min_hp: s_hp_l,
            min_phi_lp_bare: mpl_b,
            min_phi_lp_lag: mpl_l,
            min_phi_hp_bare: mph_b,
            min_phi_hp_lag: mph_l,
            fuel_removed: removed,
            tt4_peak_bare: peak(&bare),
            tt4_peak_lag: peak(&lim),
            nu_hp_end: lim[lim.len() - 1].nu_hp,
            nu_hp_end_bare: bare[bare.len() - 1].nu_hp,
            eps_edges,
        }
    }

    /// RUNG 52. The `(tau_att, tau_rel)` rows, in ROW-MAJOR order. Sweep one list with the other a
    /// singleton to get a pure attack or pure release sweep.
    ///
    /// **A PURE `tau_rel` SWEEP IS DECONFOUNDED BY CONSTRUCTION** — the property rung 50 needed
    /// `s_off` to manufacture and rung 51 believed a lag could not have. `s_cross` and
    /// `relief_watched` come back invariant; only the hand-back moves.
    ///
    /// **A PURE `tau_att` SWEEP** is rung 48's engagement-time axis in realisable clothing: a
    /// slower attack engages LATER and credits LESS.
    #[allow(clippy::too_many_arguments)]
    pub fn lag_sweep(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tau_atts: &[f64],
        tau_rels: &[f64], surge: Option<&SurgeLimiter>, accel: Option<&AccelSchedule>, r: f64,
        s_settle: f64, ds: f64, eps: &[f64],
    ) -> Vec<LagRelief> {
        let mut out = Vec::with_capacity(tau_atts.len() * tau_rels.len());
        for &ta in tau_atts {
            for &tr in tau_rels {
                out.push(self.lag_relief(flight, tt4_lo, tt4_hi, AsymmetricLag::new(ta, tr), surge,
                                         accel, r, s_settle, ds, eps));
            }
        }
        out
    }

    /// RUNG 52 (the headline method). DOES RUNG 49'S CREDIT/DEBIT SPLIT FACTOR ACROSS THE TWO
    /// TIME CONSTANTS?
    ///
    /// A real fast-attack / slow-release limiter is DESIGNED on the premise that it does — cut
    /// hard to protect, hand back gently, and tune the two independently. This is the first
    /// instrument on which rung 49's two clocks are INDEPENDENTLY DIALABLE on a single
    /// physically-realisable leg, so the premise becomes testable.
    ///
    /// **THE ANSWER IS THAT THE TWO CLOCKS SEPARATE ONE WAY.** `credit_spread` is MACHINE ZERO —
    /// `tau_att` owns the credit EXACTLY — while the debit's additive-separability residual comes
    /// back the SAME ORDER as the main effects. The design premise is HALF TRUE, and the half that
    /// fails is the PROTECTIVE one.
    ///
    /// **§ 5.18 FINDING 7 CORRECTS THE DOCSTRING'S QUOTED RATIO.** The Python says "62–70 % of
    /// them at both ramp rates measured" and `test_rung52.py`'s gate 4 says "70 % at `r = 0.5`
    /// against 62 % at `r = 2.0`". Measured on the gates' own cells at the right settle time:
    /// **65.0 % at `r = 0.5` (`ds = 0.01`) and 58.9 % at `r = 2.0` (`ds = 0.02`)**. Four
    /// alternative denominators were tried and none reproduces both figures, so the correction
    /// stands. Both clear the gate's `0.4` bar comfortably and **no gate reads the quoted
    /// numbers**.
    #[allow(clippy::too_many_arguments)]
    pub fn factorization_grid(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tau_atts: &[f64],
        tau_rels: &[f64], surge: Option<&SurgeLimiter>, accel: Option<&AccelSchedule>, r: f64,
        s_settle: f64, ds: f64, eps: &[f64],
    ) -> FactorizationGrid {
        assert!(surge.is_some(),
                "rung-52 factorization_grid splits WATCHED against OTHER, so it needs a leg with \
                 a watched spool: pass surge=. Rung 48's accel leg watches neither (it is \
                 feedforward on pressure), so `relief_watched`/`relief_other` are undefined for \
                 it -- read it through `lag_sweep` and difference the spools by name instead.");
        let rows = self.lag_sweep(flight, tt4_lo, tt4_hi, tau_atts, tau_rels, surge, accel, r,
                                  s_settle, ds, eps);
        let n = tau_rels.len();
        let grid: Vec<Vec<LagRelief>> =
            (0..tau_atts.len()).map(|i| rows[i * n..(i + 1) * n].to_vec()).collect();
        // `relief_other` is `Some` on every row because the assert above forces a watched leg.
        let other = |x: &LagRelief| x.relief_other.expect("factorization_grid forces surge=");
        let d00 = other(&grid[0][0]);
        let residual: Vec<Vec<f64>> = (0..tau_atts.len())
            .map(|i| (0..n)
                 .map(|j| other(&grid[i][j]) - other(&grid[i][0]) - other(&grid[0][j]) + d00)
                 .collect())
            .collect();
        let watched = |x: &LagRelief| x.relief_watched.expect("factorization_grid forces surge=");
        let credit_spread: Vec<(f64, f64)> = tau_atts.iter().enumerate()
            .map(|(i, &ta)| {
                let hi = grid[i].iter().map(watched).fold(f64::NEG_INFINITY, f64::max);
                let lo = grid[i].iter().map(watched).fold(f64::INFINITY, f64::min);
                (ta, hi - lo)
            })
            .collect();
        // Python's two-step `main = max(...); main = max(main, max(...))`, in that order.
        let mut main = (0..tau_atts.len())
            .map(|i| (other(&grid[i][0]) - d00).abs())
            .fold(f64::NEG_INFINITY, f64::max);
        main = main.max((0..n).map(|j| (other(&grid[0][j]) - d00).abs())
                        .fold(f64::NEG_INFINITY, f64::max));
        let max_residual = residual.iter().flatten().map(|v| v.abs())
            .fold(f64::NEG_INFINITY, f64::max);
        FactorizationGrid {
            tau_atts: tau_atts.to_vec(),
            tau_rels: tau_rels.to_vec(),
            rows,
            grid,
            residual,
            credit_spread,
            max_residual,
            max_main_effect: main,
            r,
            ds,
        }
    }
}

/// The COMMANDED running-line `phi` lookup [`FuelTransientCore::fuel_ramp_march`] hands back —
/// Python's `steady(s, spool)` closure.
///
/// **NO MEMO, deliberately.** Rung 44's `steady` caches on `round(Tt4, 3)` because it re-matches
/// at every instantaneous `Tt4`; this one interpolates a 9-point grid built ONCE, so there is
/// nothing to cache and no equivalence relation to get wrong. The two objects are described in
/// nearly the same words and are not the same object — the same trap § 5.15 measured between rungs
/// 40 and 44, one rung on.
pub struct CommandedSteady {
    grid: Vec<f64>,
    ys_l: Vec<f64>,
    ys_h: Vec<f64>,
    tt4_lo: f64,
    tt4_hi: f64,
    r: f64,
}

impl CommandedSteady {
    /// `phi_steady(Tt4_cmd(s))` — the COMMAND's running line, never the output's.
    pub fn at(&self, s: f64, spool: Spool) -> f64 {
        let u = if self.r > 0.0 { 1.0f64.min(s / self.r) } else { 1.0 };
        let tt4_cmd = self.tt4_lo + (self.tt4_hi - self.tt4_lo) * u;
        match spool {
            Spool::Lp => FuelTransientCore::interp(&self.grid, &self.ys_l, tt4_cmd),
            Spool::Hp => FuelTransientCore::interp(&self.grid, &self.ys_h, tt4_cmd),
        }
    }
}

/// The fourteen fields every route records, in Python's own order.
fn point(
    s: f64, nu_lp: f64, nu_hp: f64, inst: &FuelInstant, mf: f64, mf_sched: f64,
    extra: PointExtra,
) -> FuelPoint {
    FuelPoint {
        s,
        nu_lp,
        nu_hp,
        tt4: inst.base.tt4,
        f: inst.base.close.f,
        pi_lpc: inst.base.close.pi_lpc,
        pi_hpc: inst.base.close.pi_hpc,
        phi_lp: inst.base.close.phi_lp,
        phi_hp: inst.base.close.phi_hp,
        mdot_air: inst.base.close.mdot_air,
        sp_thrust: inst.base.sp_thrust,
        branch: inst.base.branch,
        mf,
        mf_sched,
        extra,
    }
}
