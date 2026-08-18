//! RUNGS 40 + 44 — BOTH shaft speeds become STATES, and that plant marched against the surge line.
//!
//! Rung 40 (`engine.py:3378–3969`) subclasses rung 39's map matcher for the fixed hardware (three
//! throats, both [`ComponentMap`]s, the shared choke solver and the burner `f`-solve) and then
//! runs a DIFFERENT closure: both compressor maps FORWARD with **no shaft balance**, so the two
//! power residuals stop being constraints and become the right-hand sides of two ODEs. `rho =
//! tau_L/tau_H` is the one surviving clock parameter — a RATIO — and it decides which spool leads.
//! Rung 44 marches that plant against rung 41's imposed surge line and reports the crossing.
//!
//! # The five things § 5.15's probes measured before any of this was written
//!
//! 1. **The `steady` cache key COLLIDES between two distinct floats, and the collision is worth
//!    nothing.** [`ramp_march`]'s memo keys on `round(Tt4, 3)` — a DECIMAL key on a float dict.
//!    Over 31 marches / 5 141 points there is exactly **one** collision between distinct `Tt4`
//!    (`1399.9999999999984` and `1400.0`), it FIRES inside the six reported cases, and it moves
//!    **0** reported values, because it happens at the ramp's saturated end while every extremum
//!    is attained early. A bit-exact oracle over the returned values is therefore BLIND to the key
//!    scheme — which is why [`round3`] implements the round rather than hashing the bits, and why
//!    the smoke dumps the **key sequence itself**. *An oracle cannot see a missing gate, applied
//!    before the gate was missing.*
//! 2. **FIVE arms are DEAD** on both suites' grids and are ported and counted rather than left
//!    absent: [`try_close`]'s bracket failure (0 of 20 847), both of [`integrate`]'s truncation
//!    arms (0 in 51 marches), the `max(0.2, ·)` speed floor (0 points), and the low-wall march-in
//!    loop (**0 advances in 6 339 calls**, 69 440 `g` evaluations). The high wall's `min` is the
//!    exception — BOTH arms live, 1 221 literal against 5 118 map.
//! 3. **`equilibrium`'s noise-floor branch is the ORDINARY exit on the reacting gas** — 6 of 12
//!    cells, not the rescue path its shipped comment describes (that comment names a cell list
//!    measured at rung 43's settings, and it does not hold where rung 40 reads it). So `best`
//!    tracking is load-bearing. Its own two discrete branches — the Newton damper and the `1e-30`
//!    floor — are DEAD (0 of 102 steps) and are counted.
//! 4. **CPython vs PyPy is a DETECTOR here, and it flips a discrete branch**: same 12 cells, the
//!    exit-branch classification FLIPS in 5, the iteration count differs in 10, and the converged
//!    speeds differ in 12 at ~1e-11. The mechanism is the reacting-gas equilibrium sub-solve
//!    leaving ~1e-10 of noise against an ABSOLUTE `1e-12` bar, so whether a pass squeaks under is
//!    decided below the solver's own floor. Rung 40's gates cannot see it (gate 1 asserts `1e-9`).
//! 5. **Rungs 40 and 44 reference two DIFFERENT running lines in the same words.**
//!    [`slip_excursion`] subtracts a LINEAR interpolation between two endpoint matches;
//!    [`phi_excursion`] subtracts a match at EVERY instantaneous `Tt4`. Measured in the same
//!    variable on the same trajectory, the pointwise gap reaches 5 % of the extremum while the
//!    extrema agree to seven figures — the extremum is attained early, where the steady schedule
//!    has not yet curved. So rung 40's reference is a bounded approximation, not a defect, **and a
//!    smoke that dumped only the extrema would pass a port that unified the two.**
//!
//! # The marchers are NOT fused, for three independent reasons
//!
//! [`integrate`] marches a TWO-vector with `rho` dividing the LP row;
//! [`crate::spool::SpoolTransient`]'s marcher marches a scalar; [`crate::combustor`]'s three
//! marches run their step count unconditionally. Widening one marcher across them is refused
//! because (i) the signatures differ — a two-state right-hand side with a clock ratio on one row is
//! not the scalar body with a different closure; (ii) rung 37's marches have no `try`, so routing
//! them through a marcher that truncates on failure converts a raise into a silent truncation **no
//! value gate can see**, and the 0 truncations measured here make that difference LATENT rather
//! than absent; (iii) `integrate_fuel` (slice S) is a hook overridden by eleven phase-7 classes, so
//! a shared marcher would put a hook's dispatch inside a body slice P's gates already cover.
//!
//! # Two tables, and neither hard-coded
//!
//! [`TwoSpoolTransientHooks`] carries this rung's own three virtual names (`try_close`,
//! `try_instant_tail`, `powers` — all [`R40`] today), and the INHERITED
//! [`crate::two_spool::TwoSpoolHooks`] is reached through [`TwoSpoolTransientCore::inner`], which
//! is how [`ramp_march`], [`lead_threshold`] and [`slip_excursion`] get at rung 39's `match`.
//! § 5.12 measured that every overrider of the three names is phase 7, so **this table ships with
//! zero cells exercised inside phase 6** — and a table nothing overrides cannot be witnessed by any
//! value key, which is why `tests/slice_r_smoke.rs` manufactures the failure on BOTH tables.
//!
//! [`ramp_march`]: TwoSpoolTransientCore::ramp_march
//! [`try_close`]: TwoSpoolTransientCore::try_close
//! [`integrate`]: TwoSpoolTransientCore::integrate
//! [`slip_excursion`]: TwoSpoolTransientCore::slip_excursion
//! [`phi_excursion`]: TwoSpoolTransientCore::phi_excursion
//! [`lead_threshold`]: TwoSpoolTransientCore::lead_threshold

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use crate::components::{choked_mfp, ram_recovery, Nozzle};
use crate::engine::{Engine, FlightCondition};
use crate::gas::{powp, Abort, FlowState, Gas};
use crate::map::ComponentMap;
use crate::matcher::Branch;
use crate::spool::{try_illinois, SpoolTransient, ILLINOIS_MAXIT};
use crate::two_spool::{Spool, TwoSpoolEngine, TwoSpoolMapCore, TwoSpoolMapResult};

// ---------------------------------------------------------------------------------------------
// Counters — every DEAD arm this slice ships is gated against ZERO rather than left absent
// ---------------------------------------------------------------------------------------------

thread_local! {
    static CLOSE_CALLS: Cell<u64> = const { Cell::new(0) };
    static CLOSE_BRACKET_FAILS: Cell<u64> = const { Cell::new(0) };
    static CLOSE_NONREAL: Cell<u64> = const { Cell::new(0) };
    static MARCH_IN_ADVANCES: Cell<u64> = const { Cell::new(0) };
    static HI_WALL_LITERAL: Cell<u64> = const { Cell::new(0) };
    static HI_WALL_MAP: Cell<u64> = const { Cell::new(0) };
    static EQ_CALLS: Cell<u64> = const { Cell::new(0) };
    static EQ_PRIMARY: Cell<u64> = const { Cell::new(0) };
    static EQ_NOISE: Cell<u64> = const { Cell::new(0) };
    static EQ_DAMPED: Cell<u64> = const { Cell::new(0) };
    static EQ_DAMP_FLOOR: Cell<u64> = const { Cell::new(0) };
    static POWERS_CALLS: Cell<u64> = const { Cell::new(0) };
    static INSTANT_CALLS: Cell<u64> = const { Cell::new(0) };
    static MATCH_CALLS: Cell<u64> = const { Cell::new(0) };
    static MARCH_CALLS: Cell<u64> = const { Cell::new(0) };
    static MARCH_POINTS: Cell<u64> = const { Cell::new(0) };
    static MARCH_BREAK_K1: Cell<u64> = const { Cell::new(0) };
    static MARCH_BREAK_RK: Cell<u64> = const { Cell::new(0) };
    static NU_FLOOR_HITS: Cell<u64> = const { Cell::new(0) };
    static STEADY_CALLS: Cell<u64> = const { Cell::new(0) };
    static STEADY_MISSES: Cell<u64> = const { Cell::new(0) };
    static STEADY_KEYS: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
    static STEADY_TT4: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
    static EIG_REAL: Cell<u64> = const { Cell::new(0) };
    static EIG_COMPLEX: Cell<u64> = const { Cell::new(0) };
}

/// This module's counters. Read and RESET by [`counters::take`] — same single-consumer caveat as
/// [`crate::spool::counters::take`]: they are thread-locals, so two `#[test]`s in one binary would
/// steal each other's tallies and the failure would read as physics rather than harness.
pub mod counters {
    use super::*;

    /// Every count this module keeps, plus the `steady` memo's KEY SEQUENCE.
    ///
    /// The key sequence is carried and not merely counted because it is the ONE thing the
    /// equivalence relation `round(Tt4, 3)` decides and no value key can see (§ 5.15 probe 1
    /// measured the collision moving 0 reported values, with the collision confirmed to fire
    /// inside the measured set).
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct Census {
        pub close_calls: u64,
        pub close_bracket_fails: u64,
        pub close_nonreal: u64,
        pub march_in_advances: u64,
        pub hi_wall_literal: u64,
        pub hi_wall_map: u64,
        pub eq_calls: u64,
        pub eq_primary: u64,
        pub eq_noise: u64,
        pub eq_damped: u64,
        pub eq_damp_floor: u64,
        pub powers_calls: u64,
        pub instant_calls: u64,
        /// Calls to [`TwoSpoolTransientCore::match_point`] — Python's `self.match`, which is how
        /// every rung-40/44 entry point reaches rung 39's INHERITED table.
        pub match_calls: u64,
        pub march_calls: u64,
        pub march_points: u64,
        pub march_break_k1: u64,
        pub march_break_rk: u64,
        pub nu_floor_hits: u64,
        pub steady_calls: u64,
        pub steady_misses: u64,
        pub eig_real: u64,
        pub eig_complex: u64,
        /// The memo keys in INSERTION order — one entry per cache MISS.
        pub steady_keys: Vec<f64>,
        /// The UNROUNDED `Tt4` behind each of those keys, in the same order — which is what makes
        /// the collision legible (`1399.9999999999984` keyed as `1400.0`).
        pub steady_tt4: Vec<f64>,
    }

    pub fn take() -> Census {
        let c = Census {
            close_calls: CLOSE_CALLS.with(|x| x.get()),
            close_bracket_fails: CLOSE_BRACKET_FAILS.with(|x| x.get()),
            close_nonreal: CLOSE_NONREAL.with(|x| x.get()),
            march_in_advances: MARCH_IN_ADVANCES.with(|x| x.get()),
            hi_wall_literal: HI_WALL_LITERAL.with(|x| x.get()),
            hi_wall_map: HI_WALL_MAP.with(|x| x.get()),
            eq_calls: EQ_CALLS.with(|x| x.get()),
            eq_primary: EQ_PRIMARY.with(|x| x.get()),
            eq_noise: EQ_NOISE.with(|x| x.get()),
            eq_damped: EQ_DAMPED.with(|x| x.get()),
            eq_damp_floor: EQ_DAMP_FLOOR.with(|x| x.get()),
            powers_calls: POWERS_CALLS.with(|x| x.get()),
            instant_calls: INSTANT_CALLS.with(|x| x.get()),
            match_calls: MATCH_CALLS.with(|x| x.get()),
            march_calls: MARCH_CALLS.with(|x| x.get()),
            march_points: MARCH_POINTS.with(|x| x.get()),
            march_break_k1: MARCH_BREAK_K1.with(|x| x.get()),
            march_break_rk: MARCH_BREAK_RK.with(|x| x.get()),
            nu_floor_hits: NU_FLOOR_HITS.with(|x| x.get()),
            steady_calls: STEADY_CALLS.with(|x| x.get()),
            steady_misses: STEADY_MISSES.with(|x| x.get()),
            eig_real: EIG_REAL.with(|x| x.get()),
            eig_complex: EIG_COMPLEX.with(|x| x.get()),
            steady_keys: STEADY_KEYS.with(|x| x.borrow().clone()),
            steady_tt4: STEADY_TT4.with(|x| x.borrow().clone()),
        };
        for z in [&CLOSE_CALLS, &CLOSE_BRACKET_FAILS, &CLOSE_NONREAL, &MARCH_IN_ADVANCES,
                  &HI_WALL_LITERAL, &HI_WALL_MAP, &EQ_CALLS, &EQ_PRIMARY, &EQ_NOISE, &EQ_DAMPED,
                  &EQ_DAMP_FLOOR, &POWERS_CALLS, &INSTANT_CALLS, &MATCH_CALLS, &MARCH_CALLS, &MARCH_POINTS, &MARCH_BREAK_K1,
                  &MARCH_BREAK_RK, &NU_FLOOR_HITS, &STEADY_CALLS, &STEADY_MISSES, &EIG_REAL,
                  &EIG_COMPLEX] {
            z.with(|x| x.set(0));
        }
        STEADY_KEYS.with(|x| x.borrow_mut().clear());
        STEADY_TT4.with(|x| x.borrow_mut().clear());
        c
    }
}

// ---------------------------------------------------------------------------------------------
// Records — the 21 + 23 = 44 keys Python's two dicts carry
// ---------------------------------------------------------------------------------------------

/// The flow closed at `(nu_L, nu_H, Tt4)` by the HPT-NGV choke ALONE — Python's `_close` return
/// dict, whose **21** keys are reproduced field-for-field.
///
/// **`mdot_air` IS NOT THE FACE FLOW.** `_close` computes two different physical air flows: a
/// local `mdot_air = m_lp*mcorr_lp_d*pt2/sqrt(Tt2)`, which exists only to refer the SAME physical
/// air to the HP face, and the RETURNED one, which is `mdot4/(1+f)` — the flow the NGV choke
/// imposes. The returned one is what [`TwoSpoolTransientCore::powers`] normalises both residuals
/// with and what the thrust is built on. They agree only at the root, and the port keeps them as
/// two named locals so nothing collapses them.
#[derive(Clone, Debug)]
pub struct CloseState {
    pub m_lp: f64,
    pub m_imp: f64,
    pub m_hp: f64,
    pub phi_lp: f64,
    pub phi_hp: f64,
    pub tt2: f64,
    pub n_lp: f64,
    pub n_hp: f64,
    pub tau_lpc: f64,
    pub tau_hpc: f64,
    pub tt25: f64,
    pub tt3: f64,
    pub pi_lpc: f64,
    pub pi_hpc: f64,
    pub pt4: f64,
    pub f: f64,
    /// `None` when the working gas IS the design gas — [`TwoSpoolMapCore`]'s `Option` convention.
    pub wgas: Option<Gas>,
    pub eta_lpc: f64,
    pub eta_hpc: f64,
    /// `mdot4/(1+f)` — see the type note. NOT the LP-face flow.
    pub mdot_air: f64,
    pub mdot4: f64,
}

impl CloseState {
    pub fn gas<'a>(&'a self, c: &'a TwoSpoolMapCore) -> &'a Gas {
        self.wgas.as_ref().unwrap_or_else(|| c.gas())
    }
}

/// The quasi-steady instant: [`CloseState`] plus the turbine / power / thrust tail — Python's
/// `_instant_tail` return dict, which `update`s **23** further keys onto the 21 above.
///
/// **`PartialEq` is HAND-WRITTEN over the 42 floats and the branch, and deliberately EXCLUDES
/// `wgas`.** Rung 44 gate 1 compares two of these with `==` over the whole 44-key dict, and § 5.15
/// measured that comparison **as the gate actually makes it** — two SEPARATE transient objects
/// built with differently-armed maps, not one object twice: their two `wgas` are the SAME object
/// (`is` → `True`), because the working gas is memoised upstream of the maps. So Python's dict `==`
/// never exercises gas value equality there. (`Gas` does define `__eq__` and the two compare equal
/// anyway, so the decision is safe either way — it is now measured rather than inferred from a
/// one-object test.)
#[derive(Clone, Debug)]
pub struct Instant2 {
    pub close: CloseState,
    pub nu_lp: f64,
    pub nu_hp: f64,
    pub tt4: f64,
    pub slip: f64,
    /// `Phi_lp` — the LP power residual, i.e. `rho*dnu_L/ds`.
    pub phi_lp_dot: f64,
    /// `Phi_hp` — the HP power residual, i.e. `dnu_H/ds`.
    pub phi_hp_dot: f64,
    pub pt_lp: f64,
    pub pt_hp: f64,
    pub pc_lp: f64,
    pub pc_hp: f64,
    pub tt45: f64,
    pub tt5: f64,
    pub tau_hpt: f64,
    pub tau_lpt: f64,
    pub pi_hpt: f64,
    pub pi_lpt: f64,
    pub eta_hpt: f64,
    pub eta_lpt: f64,
    pub nu_hpt: f64,
    pub nu_lpt: f64,
    pub sp_thrust: f64,
    pub m9: f64,
    pub branch: Branch,
}

impl PartialEq for Instant2 {
    fn eq(&self, o: &Self) -> bool {
        let (a, b) = (&self.close, &o.close);
        a.m_lp == b.m_lp && a.m_imp == b.m_imp && a.m_hp == b.m_hp && a.phi_lp == b.phi_lp
            && a.phi_hp == b.phi_hp && a.tt2 == b.tt2 && a.n_lp == b.n_lp && a.n_hp == b.n_hp
            && a.tau_lpc == b.tau_lpc && a.tau_hpc == b.tau_hpc && a.tt25 == b.tt25
            && a.tt3 == b.tt3 && a.pi_lpc == b.pi_lpc && a.pi_hpc == b.pi_hpc && a.pt4 == b.pt4
            && a.f == b.f && a.eta_lpc == b.eta_lpc && a.eta_hpc == b.eta_hpc
            && a.mdot_air == b.mdot_air && a.mdot4 == b.mdot4
            && self.nu_lp == o.nu_lp && self.nu_hp == o.nu_hp && self.tt4 == o.tt4
            && self.slip == o.slip && self.phi_lp_dot == o.phi_lp_dot
            && self.phi_hp_dot == o.phi_hp_dot && self.pt_lp == o.pt_lp && self.pt_hp == o.pt_hp
            && self.pc_lp == o.pc_lp && self.pc_hp == o.pc_hp && self.tt45 == o.tt45
            && self.tt5 == o.tt5 && self.tau_hpt == o.tau_hpt && self.tau_lpt == o.tau_lpt
            && self.pi_hpt == o.pi_hpt && self.pi_lpt == o.pi_lpt && self.eta_hpt == o.eta_hpt
            && self.eta_lpt == o.eta_lpt && self.nu_hpt == o.nu_hpt && self.nu_lpt == o.nu_lpt
            && self.sp_thrust == o.sp_thrust && self.m9 == o.m9 && self.branch == o.branch
    }
}

/// One instant of a marched TWO-shaft trajectory — Python's `TwoSpoolTransientPoint`,
/// nondimensional time `s = t/tau_H`.
#[derive(Clone, Copy, Debug)]
pub struct TwoSpoolTransientPoint {
    pub s: f64,
    /// `N_L/N_L,d` — STATE 1.
    pub nu_lp: f64,
    /// `N_H/N_H,d` — STATE 2.
    pub nu_hp: f64,
    pub tt4: f64,
    pub slip: f64,
    pub pi_lpc: f64,
    pub pi_hpc: f64,
    pub phi_lp: f64,
    pub phi_hp: f64,
    pub mdot_air: f64,
    pub f: f64,
    /// `rho*dnu_L/ds` — the LP power residual; 0 on the running line.
    pub phi_lp_dot: f64,
    /// `dnu_H/ds`.
    pub phi_hp_dot: f64,
    pub sp_thrust: f64,
}

/// RUNG 44's `phi_excursion` return.
#[derive(Clone, Copy, Debug)]
pub struct PhiExcursion {
    pub ext_lp: f64,
    pub ext_hp: f64,
    pub s_lp: f64,
    pub s_hp: f64,
    pub min_phi_lp: f64,
    pub min_phi_hp: f64,
    /// `|ext_lp|/|ext_hp|`, or `+inf` when the HP excursion is exactly zero.
    pub ratio: f64,
    pub npts: usize,
}

/// RUNG 44's `transient_surge_margin` return.
#[derive(Clone, Copy, Debug)]
pub struct TransientSurgeMargin {
    pub margin_min_lp: f64,
    pub margin_min_hp: f64,
    pub steady_min_lp: f64,
    pub steady_min_hp: f64,
    pub crossed_lp: bool,
    pub crossed_hp: bool,
    pub phi_surge_lp: f64,
    pub phi_surge_hp: f64,
    pub npts: usize,
}

/// How [`TwoSpoolTransientCore::try_equilibrium`] left its Newton — a DISCRETE oracle key.
///
/// § 5.15 probe 4 measured this classification FLIPPING between CPython and PyPy in 5 of 12 cells
/// on the reacting gas, while every rung-40 gate passes on both. It is dumped so a step-4
/// divergence shows up as a discrete disagreement rather than as a last-bit one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EqExit {
    /// The residual fell under the ABSOLUTE `1e-12` bar.
    Primary,
    /// All 80 passes spent; the best iterate seen was accepted under the `1e-8` noise floor.
    Noise,
}

// ---------------------------------------------------------------------------------------------
// The virtual table
// ---------------------------------------------------------------------------------------------

/// Rung 40's own three virtual names.
///
/// § 5.12's census ran § 5.3's inheritance sweep in the opposite direction and found `_close`,
/// `_instant_tail` and `_powers` all overridden — by `ScheduledStatorTransient`,
/// `ScheduledBleedTransient`, `LimitedBleedTransient` and `LaggedBleedTransient`, **every one of
/// which is phase 7**. So this table ships LIVE and unexercised: no cell is swapped anywhere in
/// phase 6, and no value key can witness a table nobody overrides. `tests/slice_r_smoke.rs`
/// manufactures the failure instead — swap a cell, assert a value breaks — on
/// `rung42.rs::gate_the_dispatch_is_live`'s precedent.
pub struct TwoSpoolTransientHooks {
    /// The FORWARD closure: one root in `m_L`, no shaft balance.
    pub try_close:
        fn(&TwoSpoolTransientCore, f64, f64, f64, f64, f64) -> Result<CloseState, Abort>,
    /// The turbine / power / thrust tail, shared with rung 43's FUEL control (which reaches it
    /// with `Tt4` an OUTPUT of the closure rather than an input).
    pub try_instant_tail: fn(&TwoSpoolTransientCore, &FlightCondition, &CloseState, f64, f64, f64,
                             f64) -> Result<Instant2, Abort>,
    /// `(Phi_L, Phi_H)` from an already-closed flow — the Newton's inner loop, factored out so it
    /// does not rebuild the nozzle/thrust tail at every step.
    pub powers: fn(&TwoSpoolTransientCore, &CloseState, &FlightCondition, f64, f64, f64)
        -> Result<(f64, f64), Abort>,
}

/// RUNG 40's table.
pub const R40: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: r40_try_close,
    try_instant_tail: r40_try_instant_tail,
    powers: r40_powers,
};

// ---------------------------------------------------------------------------------------------
// The object
// ---------------------------------------------------------------------------------------------

/// RUNG 40 / 44. Both shaft speeds are STATES under one clock ratio.
///
/// `lp_disabled=True` dispatches to rung 34's [`SpoolTransient`] — EXACT dispatch, no two-shaft
/// state is ever built. Python's `__init__` returns before `super().__init__`, so only four
/// attributes exist on that path and every inherited method raises `AttributeError` (**not**
/// `AssertionError`, so no caller in the ladder catches it) — which is why this variant carries the
/// single-spool object alone and every two-shaft accessor on it panics. `map_lp`/`map_hp`/`rho` are
/// set-and-never-read there and are NOT carried; slice K's `TwoSpoolMapMatcher::Degenerate` is the
/// precedent.
pub enum TwoSpoolTransient {
    Degenerate(SpoolTransient),
    Full(TwoSpoolTransientCore),
}

impl TwoSpoolTransient {
    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap, rho: f64,
    ) -> Self {
        TwoSpoolTransient::Full(TwoSpoolTransientCore::new(
            design_engine, flight_design, mdot_design, map_lp, map_hp, rho))
    }

    /// `lp_disabled=True`. Takes a SINGLE-spool design engine and `map_hp`, which is exactly what
    /// Python's early return builds.
    pub fn lp_disabled(
        design_engine: Engine, flight_design: FlightCondition, mdot_design: f64,
        map_hp: ComponentMap,
    ) -> Self {
        TwoSpoolTransient::Degenerate(
            SpoolTransient::new(design_engine, flight_design, mdot_design, map_hp))
    }

    /// The rung-34 object `lp_disabled` forwards to — Python's `self._degenerate`.
    pub fn degenerate(&self) -> &SpoolTransient {
        match self {
            TwoSpoolTransient::Degenerate(s) => s,
            TwoSpoolTransient::Full(_) => panic!("this transient is not lp_disabled"),
        }
    }

    pub fn core(&self) -> &TwoSpoolTransientCore {
        match self {
            TwoSpoolTransient::Full(c) => c,
            TwoSpoolTransient::Degenerate(_) => panic!("this transient is lp_disabled"),
        }
    }

    /// `rho` is assigned on a BUILT object by rung 40 gates 5, 6 and 7 — gate 7 inside an 18-step
    /// bisection — so it is a mutable field, not a constructor-only parameter.
    pub fn core_mut(&mut self) -> &mut TwoSpoolTransientCore {
        match self {
            TwoSpoolTransient::Full(c) => c,
            TwoSpoolTransient::Degenerate(_) => panic!("this transient is lp_disabled"),
        }
    }
}

/// Rung 40's object once `lp_disabled` is ruled out: rung 39's map core, the clock ratio, and the
/// two PER-SPOOL design shaft powers the residuals are nondimensionalised on.
pub struct TwoSpoolTransientCore {
    /// Rung 39's matcher. `pub` because the reduce gates need the SAME captured hardware on both
    /// sides — and because the INHERITED [`crate::two_spool::TwoSpoolHooks`] lives on it.
    pub inner: TwoSpoolMapCore,
    /// `rho = tau_L/tau_H` — the ONE surviving clock parameter, and a RATIO.
    pub rho: f64,
    pub p_ref_lp: f64,
    pub p_ref_hp: f64,
    pub hooks: &'static TwoSpoolTransientHooks,
}

impl TwoSpoolTransientCore {
    /// Python's `_EQ_TOL` — ABSOLUTE, which is exactly what probe 4's detector turns on.
    pub const EQ_TOL: f64 = 1e-12;
    /// Python's `_EQ_MAX`.
    pub const EQ_MAX: usize = 80;
    /// The noise-floor acceptance bound. Worst accepted residual measured: 6.47e-11, against an
    /// initial residual of 3e-2…3e-1 — the bound is not delicate, and that is now a count.
    pub const NOISE_FLOOR: f64 = 1e-8;
    /// The `_close` root's tolerance — a LITERAL `1e-12` at that call site, NOT `EQ_TOL`.
    pub const CLOSE_TOL: f64 = 1e-12;

    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap, rho: f64,
    ) -> Self {
        Self::with_hooks(design_engine, flight_design, mdot_design, map_lp, map_hp, rho, &R40)
    }

    pub fn with_hooks(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap, rho: f64,
        hooks: &'static TwoSpoolTransientHooks,
    ) -> Self {
        let inner = TwoSpoolMapCore::new(design_engine, flight_design, mdot_design, map_lp, map_hp);
        let (s2, s25, s3) = (*inner.base.reference.station("2"),
                             *inner.base.reference.station("25"),
                             *inner.base.reference.station("3"));
        let gas = inner.gas();
        let p_ref_lp = mdot_design * (gas.h_c(s25.tt) - gas.h_c(s2.tt));
        let p_ref_hp = mdot_design * (gas.h_c(s3.tt) - gas.h_c(s25.tt));
        TwoSpoolTransientCore { inner, rho, p_ref_lp, p_ref_hp, hooks }
    }

    pub fn gas(&self) -> &Gas { self.inner.gas() }

    // --- the inlet state (shared by every entry point below) --------------------------------

    /// `(Tt2, pt2, V0)` — Python's `_inlet`.
    pub fn inlet(&self, flight: &FlightCondition) -> (f64, f64, f64) {
        let pi_d = self.inner.base.pi_d_max * ram_recovery(flight.m0);
        let (state0, v0) = self.inner.base.freestream_for(flight);
        (state0.tt, pi_d * state0.pt, v0)
    }

    // --- THE DISPATCH POINTS ----------------------------------------------------------------

    /// Close the flow at `(nu_L, nu_H, Tt4)` **through the virtual table**.
    pub fn try_close(
        &self, nu_lp: f64, nu_hp: f64, tt4: f64, tt2: f64, pt2: f64,
    ) -> Result<CloseState, Abort> {
        (self.hooks.try_close)(self, nu_lp, nu_hp, tt4, tt2, pt2)
    }

    /// [`try_close`](Self::try_close) for a caller that cannot fail — Python's `_close` reached
    /// from `equilibrium`'s Newton, where nothing catches.
    pub fn close(&self, nu_lp: f64, nu_hp: f64, tt4: f64, tt2: f64, pt2: f64) -> CloseState {
        self.try_close(nu_lp, nu_hp, tt4, tt2, pt2).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The turbine / power / thrust tail, **through the virtual table**.
    #[allow(clippy::too_many_arguments)]
    pub fn try_instant_tail(
        &self, flight: &FlightCondition, c: &CloseState, nu_lp: f64, nu_hp: f64, tt4: f64,
        v0: f64,
    ) -> Result<Instant2, Abort> {
        (self.hooks.try_instant_tail)(self, flight, c, nu_lp, nu_hp, tt4, v0)
    }

    /// `(Phi_L, Phi_H)` from an already-closed flow, **through the virtual table**.
    pub fn powers(
        &self, c: &CloseState, flight: &FlightCondition, nu_lp: f64, nu_hp: f64, tt4: f64,
    ) -> Result<(f64, f64), Abort> {
        (self.hooks.powers)(self, c, flight, nu_lp, nu_hp, tt4)
    }

    /// Rung 39's `match`, reached **through the INHERITED table** — Python's `self.match(...)`
    /// inside a rung-40 method, which resolves to `TwoSpoolMapMatcher.match` and from there to
    /// whatever cell `hooks.try_match_point` holds.
    ///
    /// Counted, because this edge — a TRANSIENT object reaching rung 39's matcher — is the one
    /// structurally new dispatch of the slice, and no value key can witness a table nobody
    /// overrides.
    pub fn match_point(&self, flight: &FlightCondition, tt4: f64) -> TwoSpoolMapResult {
        MATCH_CALLS.with(|x| x.set(x.get() + 1));
        self.inner.match_point(flight, tt4)
    }

    // --- one quasi-steady instant ------------------------------------------------------------

    /// The quasi-steady flow at `(nu_L, nu_H, Tt4)` and the TWO net powers driving the two shaft
    /// ODEs. NOT a matched point — both shafts are deliberately UNBALANCED.
    pub fn try_instant(
        &self, flight: &FlightCondition, nu_lp: f64, nu_hp: f64, tt4: f64,
    ) -> Result<Instant2, Abort> {
        INSTANT_CALLS.with(|x| x.set(x.get() + 1));
        let (tt2, pt2, v0) = self.inlet(flight);
        let c = self.try_close(nu_lp, nu_hp, tt4, tt2, pt2)?;
        self.try_instant_tail(flight, &c, nu_lp, nu_hp, tt4, v0)
    }

    pub fn instant(&self, flight: &FlightCondition, nu_lp: f64, nu_hp: f64, tt4: f64) -> Instant2 {
        self.try_instant(flight, nu_lp, nu_hp, tt4).unwrap_or_else(|e| panic!("{}", e.0))
    }

    // --- the equilibrium: a 2-D root (rung 34's was 1-D) -------------------------------------

    /// Solve `Phi_L = Phi_H = 0` in `(nu_L, nu_H)` — the two-shaft running-line instant.
    ///
    /// **THE REDUCE**: this reproduces rung 39's matched point at the same `(flight, Tt4)`, through
    /// the FORWARD closure only and never by calling that matcher (which would make the reduce
    /// circular).
    ///
    /// **`best` IS THREE SEPARATE TRAPS, and all three are load-bearing on the reacting gas**,
    /// where the residual plateaus for ~75 passes and which iterate wins is decided in the noise:
    /// it is updated AFTER the tolerance check (so the returning iterate never enters it), the
    /// comparison is STRICT (`<`, so ties keep the EARLIEST iterate), and it stores the speeds that
    /// PRODUCED the residual — i.e. pre-step. A `<=`, or a capture after the update, is invisible
    /// on CPG and moves `(nu_L, nu_H)` on the reacting gas.
    ///
    /// Returns the instant, how the Newton left, and the pass count — the last two are DISCRETE
    /// oracle keys, because probe 4 measured them flipping between interpreters.
    pub fn try_equilibrium(
        &self, flight: &FlightCondition, tt4: f64, start: Option<(f64, f64)>,
    ) -> Result<(Instant2, EqExit, usize), Abort> {
        EQ_CALLS.with(|x| x.set(x.get() + 1));
        let (tt2, pt2, _) = self.inlet(flight);
        let big_f = |a: f64, b: f64| -> Result<(f64, f64), Abort> {
            let c = self.try_close(a, b, tt4, tt2, pt2)?;
            self.powers(&c, flight, a, b, tt4)
        };

        let (mut nl, mut nh) = start.unwrap_or((1.0, 1.0));
        let mut best: Option<(f64, f64, f64)> = None;
        for it in 0..Self::EQ_MAX {
            let (fl, fh) = big_f(nl, nh)?;
            let res = fl.abs().max(fh.abs());
            if res < Self::EQ_TOL {
                EQ_PRIMARY.with(|x| x.set(x.get() + 1));
                return Ok((self.try_instant(flight, nl, nh, tt4)?, EqExit::Primary, it));
            }
            if best.is_none() || res < best.unwrap().0 {
                best = Some((res, nl, nh));
            }
            let h = 1e-6;
            let (al, ah) = big_f(nl + h, nh)?;
            let (bl, bh) = big_f(nl, nh + h)?;
            let (j11, j12) = ((al - fl) / h, (bl - fl) / h);
            let (j21, j22) = ((ah - fh) / h, (bh - fh) / h);
            let det = j11 * j22 - j12 * j21;
            assert!(det.abs() > 1e-300, "rung-40 equilibrium Jacobian is singular");
            let dl = (-fl * j22 + fh * j12) / det;
            let dh = (-j11 * fh + j21 * fl) / det;
            // Python's `max(a, b, c)` over three positives. BOTH discrete branches here are DEAD
            // (0 of 102 Newton steps): the damper never binds and the 1e-30 floor never wins.
            // Counted anyway — a `min` on a solver's step is exactly the shape § 5.12 warned about.
            let scale = dl.abs().max(dh.abs()).max(1e-30);
            if scale == 1e-30 {
                EQ_DAMP_FLOOR.with(|x| x.set(x.get() + 1));
            }
            let damp = 1.0f64.min(0.25 / scale);
            if damp < 1.0 {
                EQ_DAMPED.with(|x| x.set(x.get() + 1));
            }
            nl += damp * dl;
            nh += damp * dh;
        }
        // NOISE-FLOOR ACCEPTANCE. `EQ_TOL` is ABSOLUTE, but the residual's noise floor is
        // GAS-dependent: ~1e-14 on CPG (so the primary return always fires), ~1e-10 on the REACTING
        // gas, where the equilibrium sub-solve inside `_close` leaves that much in Phi. The shipped
        // comment calls this a path "reached ONLY by inputs that previously RAISED" and names a
        // cell list; § 5.15 probe 3 re-measured it at rung 40's own settings and it is **the
        // ordinary exit on the reacting gas, 6 of 12 cells** — that list was written at rung 43's
        // settings and does not hold where this rung reads it.
        if let Some((r, bl, bh)) = best {
            if r < Self::NOISE_FLOOR {
                EQ_NOISE.with(|x| x.set(x.get() + 1));
                return Ok((self.try_instant(flight, bl, bh, tt4)?, EqExit::Noise, Self::EQ_MAX));
            }
        }
        Err(Abort(format!("rung-40 two-shaft equilibrium did not converge at Tt4={tt4:.0}")))
    }

    /// [`try_equilibrium`](Self::try_equilibrium) for a caller that cannot fail — Python's
    /// `equilibrium`, whose `AssertionError` no caller in this rung catches.
    pub fn equilibrium(&self, flight: &FlightCondition, tt4: f64) -> Instant2 {
        self.try_equilibrium(flight, tt4, None).unwrap_or_else(|e| panic!("{}", e.0)).0
    }

    // --- THE OBJECT: the lead threshold sigma_crit (dagger) ----------------------------------

    /// `sigma_crit`: the clock ratio at which NEITHER spool leads.
    ///
    /// ```text
    /// sigma_crit = [ (dPhi_L/dTt4)/nu_L ] / [ (dPhi_H/dTt4)/nu_H ]
    /// ```
    ///
    /// HP leads an acceleration iff `rho > sigma_crit`. `== 1` EXACTLY on flat maps + a CPG gas,
    /// INHERITED from rung 39's slip identity — this rung's reduce spine, not its finding. The
    /// finding is that BOTH the `cp(T)` gas curve and the maps move it off 1.
    ///
    /// `nu = None` reaches rung 39's `match` through the INHERITED table.
    pub fn lead_threshold(
        &self, flight: &FlightCondition, tt4: f64, d: f64, nu: Option<(f64, f64)>,
    ) -> f64 {
        let nu = nu.unwrap_or_else(|| {
            let od = self.match_point(flight, tt4);
            (od.n_lp_ratio, od.n_hp_ratio)
        });
        let ip = self.instant(flight, nu.0, nu.1, tt4 + d);
        let im = self.instant(flight, nu.0, nu.1, tt4 - d);
        ((ip.phi_lp_dot - im.phi_lp_dot) / nu.0) / ((ip.phi_hp_dot - im.phi_hp_dot) / nu.1)
    }

    // --- stability: the 2x2 Jacobian of the two-state flow ------------------------------------

    /// `d(dnu/ds)/d(nu)` at `(nu_L, nu_H)` — the two-state analogue of rung 34's "Phi decreasing
    /// through zero". Returns `[[a,b],[c,d]]`, with the LP row carrying `1/rho`.
    pub fn jacobian(
        &self, flight: &FlightCondition, tt4: f64, nu: Option<(f64, f64)>, h: f64,
    ) -> [[f64; 2]; 2] {
        self.jacobian_at_rho(flight, tt4, nu, h, self.rho)
    }

    /// [`jacobian`](Self::jacobian) at an EXPLICIT clock ratio.
    ///
    /// Python writes `rho0, self.rho = self.rho, 1.0` … `finally: self.rho = rho0` around the two
    /// band methods below. That save/restore is a Python idiom for a `&self` method, not physics:
    /// this is a grep, not a probe — `self.rho` is read at exactly ONE site inside the `jacobian`
    /// call tree (`engine.py:3678`), so passing `1.0` explicitly is bit-identical by construction,
    /// and `rho` stays a plain mutable field for the gates that assign it.
    pub fn jacobian_at_rho(
        &self, flight: &FlightCondition, tt4: f64, nu: Option<(f64, f64)>, h: f64, rho: f64,
    ) -> [[f64; 2]; 2] {
        let nu = nu.unwrap_or_else(|| {
            let od = self.match_point(flight, tt4);
            (od.n_lp_ratio, od.n_hp_ratio)
        });
        let big_f = |a: f64, b: f64| -> (f64, f64) {
            let i = self.instant(flight, a, b, tt4);
            (i.phi_lp_dot / rho, i.phi_hp_dot)
        };
        let (fl, fh) = big_f(nu.0, nu.1);
        let (al, ah) = big_f(nu.0 + h, nu.1);
        let (bl, bh) = big_f(nu.0, nu.1 + h);
        [[(al - fl) / h, (bl - fl) / h], [(ah - fh) / h, (bh - fh) / h]]
    }

    /// Real parts of the 2x2 eigenvalues (both negative ⇔ a stable attractor).
    ///
    /// **BOTH branches are LIVE** — the `disc >= 0` guard is the one `** 0.5` site in this class
    /// protected by an explicit sign test rather than by a measurement, and rung 40 gate 5's `rho`
    /// sweep takes the complex arm 7 times against 245 real ON THAT GRID. Counted here so a port
    /// reaching only one arm reads as a count disagreement rather than as silence — and counted on
    /// whatever grid is running, never against gate 5's numbers on a different one.
    pub fn eigenvalues(j: [[f64; 2]; 2]) -> (f64, f64) {
        let tr = j[0][0] + j[1][1];
        let det = j[0][0] * j[1][1] - j[0][1] * j[1][0];
        let disc = tr * tr - 4.0 * det;
        if disc >= 0.0 {
            EIG_REAL.with(|x| x.set(x.get() + 1));
            let r = powp(disc, 0.5);
            return (0.5 * (tr - r), 0.5 * (tr + r));
        }
        EIG_COMPLEX.with(|x| x.set(x.get() + 1));
        (0.5 * tr, 0.5 * tr)
    }

    // --- THE FINDING: the rho-band in which the inter-spool mode goes COMPLEX ----------------
    //
    // Write the Jacobian at rho=1 as (a,b,c,d). At clock ratio rho the LP row carries 1/rho, so
    //
    //     tr   = a/rho + d              det  = (a*d - b*c)/rho
    //     disc = tr^2 - 4*det = (a/rho - d)^2 + 4*b*c/rho
    //
    // STABILITY: tr<0 and det>0 hold for EVERY rho>0 as soon as a<0, d<0 and a*d>b*c — three
    // conditions carrying NO rho. Those signs are MEASURED (gate 5); what is DERIVED is that, given
    // them, rho cannot destabilise the pair. OSCILLATION: disc is NOT rho-free. It vanishes at
    // rho = a/d, leaving disc = 4*b*c/rho, so whenever b*c < 0 a complex pair EXISTS in a band
    // around a/d. Measured: b*c < 0 exactly when the LP compressor map is SHAPED. MAP-created.

    /// The `rho` interval on which the two-shaft mode is COMPLEX, or `None` if there is none.
    ///
    /// The `(B*B - 4*A*C) ** 0.5` here has NO guard, and is safe only GIVEN `a<0, d<0` — measured
    /// `min(B²−4AC) = 2.587e-02 > 0` over the gate grid. The derivation the measurement stands on:
    /// with `A = a²`, `C = d²` and `B = 2ad + 4|bc|`,
    ///
    /// ```text
    /// B² − 4AC = (2ad + 4|bc|)² − 4a²d² = 16|bc|(ad + |bc|) ≥ 0   whenever  a*d > 0,
    /// ```
    ///
    /// which the `a<0, d<0` sign structure supplies (`min(a·d) = 9.438e-01`, both signs 42/42). A
    /// sign flip here turns Python's COMPLEX into a silent Rust NaN, which is why the derivation is
    /// written AT the site rather than in the spec.
    pub fn oscillatory_band(
        &self, flight: &FlightCondition, tt4: f64, nu: Option<(f64, f64)>,
    ) -> Option<(f64, f64)> {
        let j = self.jacobian_at_rho(flight, tt4, nu, 1e-6, 1.0);
        let (a, b, c, d) = (j[0][0], j[0][1], j[1][0], j[1][1]);
        if b * c >= 0.0 {
            return None;
        }
        // disc<0  <=>  a^2 u^2 - (2ad + 4|bc|) u + d^2 < 0,   u = 1/rho
        let (aa, bb, cc) = (a * a, 2.0 * a * d + 4.0 * (b * c).abs(), d * d);
        let root = powp(bb * bb - 4.0 * aa * cc, 0.5);
        Some((2.0 * aa / (bb + root), 2.0 * aa / (bb - root)))
    }

    /// `max` over `rho` of `|Im/Re|` for the two-shaft mode = `sqrt(-b*c/(a*d))`, attained at
    /// `rho = a/d`. Zero when `b*c >= 0`. MAGNITUDE DISCLAIMED (it rides on the maps).
    ///
    /// The `** 0.5` is guarded by the `b*c >= 0` early return GIVEN `a*d > 0` — the same measured
    /// sign structure as [`oscillatory_band`](Self::oscillatory_band).
    pub fn damping_ratio_max(
        &self, flight: &FlightCondition, tt4: f64, nu: Option<(f64, f64)>,
    ) -> f64 {
        let j = self.jacobian_at_rho(flight, tt4, nu, 1e-6, 1.0);
        let (a, b, c, d) = (j[0][0], j[0][1], j[1][0], j[1][1]);
        if b * c >= 0.0 { 0.0 } else { powp(-b * c / (a * d), 0.5) }
    }

    // --- march both shafts (RK4 on a 2-vector) ------------------------------------------------

    /// RK4-march `(dnu_L/ds, dnu_H/ds) = (Phi_L/rho, Phi_H)` with `Tt4 = schedule(s)`.
    ///
    /// **WRITTEN OUT, not routed through [`crate::spool::SpoolTransient`]'s marcher** — see the
    /// module note's three reasons. Trajectory LENGTH is an OUTPUT (rung 34's discipline), and both
    /// truncation arms are counted: **0** on every grid measured, so the difference from rung 37's
    /// unconditional marches is LATENT rather than absent, and the smoke MANUFACTURES a truncation
    /// so the length key is a gate that has fired rather than one that never could.
    ///
    /// `int(round(s_end/ds))` is LIVE here, unlike at rung 37: of the four `(s_end, ds)` pairs the
    /// two suites use, `1.2/0.05 = 23.99999999999999644729` is not exact, so `round` gives 24 and a
    /// truncation gives 23 — a whole missing step. `round_ties_even`, not `f64::round`: Python's
    /// zero-digit `round` is half-to-EVEN.
    pub fn integrate<S>(
        &self, flight: &FlightCondition, schedule: S, nu0: (f64, f64), s_end: f64, ds: f64,
    ) -> Vec<TwoSpoolTransientPoint>
    where
        S: Fn(f64) -> f64,
    {
        MARCH_CALLS.with(|x| x.set(x.get() + 1));
        let der = |a: f64, b: f64, t: f64| -> Result<(f64, f64, Instant2), Abort> {
            let i = self.try_instant(flight, a, b, t)?;
            Ok((i.phi_lp_dot / self.rho, i.phi_hp_dot, i))
        };
        let mut pts: Vec<TwoSpoolTransientPoint> = Vec::new();
        let (mut nl, mut nh) = nu0;
        let mut s = 0.0f64;
        let n_steps = (s_end / ds).round_ties_even() as i64;
        for i_step in 0..=n_steps {
            let tt4 = schedule(s);
            let (k1l, k1h, inst) = match der(nl, nh, tt4) {
                Ok(x) => x,
                Err(_) => {
                    MARCH_BREAK_K1.with(|x| x.set(x.get() + 1));
                    break;
                }
            };
            pts.push(TwoSpoolTransientPoint {
                s, nu_lp: nl, nu_hp: nh, tt4, slip: nl / nh, pi_lpc: inst.close.pi_lpc,
                pi_hpc: inst.close.pi_hpc, phi_lp: inst.close.phi_lp, phi_hp: inst.close.phi_hp,
                mdot_air: inst.close.mdot_air, f: inst.close.f, phi_lp_dot: inst.phi_lp_dot,
                phi_hp_dot: inst.phi_hp_dot, sp_thrust: inst.sp_thrust,
            });
            MARCH_POINTS.with(|x| x.set(x.get() + 1));
            if i_step == n_steps {
                break;
            }
            // The three remaining RK stages sit under ONE `try` in Python, so any of them ending
            // the march ends it at the SAME point — the stages are sequential and each needs the
            // previous slope.
            let stages = (|| -> Result<(f64, f64, f64, f64, f64, f64), Abort> {
                let (k2l, k2h, _) =
                    der(nl + 0.5 * ds * k1l, nh + 0.5 * ds * k1h, schedule(s + 0.5 * ds))?;
                let (k3l, k3h, _) =
                    der(nl + 0.5 * ds * k2l, nh + 0.5 * ds * k2h, schedule(s + 0.5 * ds))?;
                let (k4l, k4h, _) = der(nl + ds * k3l, nh + ds * k3h, schedule(s + ds))?;
                Ok((k2l, k2h, k3l, k3h, k4l, k4h))
            })();
            let Ok((k2l, k2h, k3l, k3h, k4l, k4h)) = stages else {
                MARCH_BREAK_RK.with(|x| x.set(x.get() + 1));
                break;
            };
            nl = 0.2f64.max(nl + ds / 6.0 * (k1l + 2.0 * k2l + 2.0 * k3l + k4l));
            nh = 0.2f64.max(nh + ds / 6.0 * (k1h + 2.0 * k2h + 2.0 * k3h + k4h));
            if nl == 0.2 || nh == 0.2 {
                NU_FLOOR_HITS.with(|x| x.set(x.get() + 1));
            }
            s += ds;
        }
        pts
    }

    // --- THE FINDING: the marched slip excursion, and its sign vs rho ------------------------

    /// Signed extremum of `slip - slip_steady(Tt4)` over a marched acceleration ramp.
    ///
    /// NEGATIVE ⇔ the LP spool falls BEHIND its steady schedule ⇔ the HP spool LEADS.
    ///
    /// **THE REFERENCE IS A LINEAR INTERPOLATION between two endpoint matches**, where rung 44's
    /// [`phi_excursion`](Self::phi_excursion) re-matches at every instantaneous `Tt4` — two
    /// different objects, described by the same words in both docstrings. Probe 5 measured the
    /// pointwise gap reaching 5 % of the extremum while the extrema agree to seven figures, because
    /// the extremum is attained early, where the steady schedule has not yet curved. So this is a
    /// BOUNDED approximation and rung 44's construction is the general one — and the extremum alone
    /// cannot tell the two apart, which is why the smoke dumps POINTWISE keys.
    pub fn slip_excursion(
        &self, flight: &FlightCondition, tt4_lo: f64, dtt4: f64, r_ramp: f64, s_end: f64, ds: f64,
    ) -> f64 {
        let od_lo = self.match_point(flight, tt4_lo);
        let od_hi = self.match_point(flight, tt4_lo + dtt4);
        let (slip_lo, slip_hi) = (od_lo.slip, od_hi.slip);
        let nu0 = (od_lo.n_lp_ratio, od_lo.n_hp_ratio);
        let sched = |t: f64| tt4_lo + dtt4 * 1.0f64.min(t / r_ramp);
        let mut ext = 0.0f64;
        for p in self.integrate(flight, sched, nu0, s_end, ds) {
            let u = (p.tt4 - tt4_lo) / dtt4;
            let e = p.slip - (slip_lo + u * (slip_hi - slip_lo));
            if e.abs() > ext.abs() {
                ext = e;
            }
        }
        ext
    }

    // --- RUNG 44: the TRANSIENT surge line — the phi excursion and the crossing --------------

    /// RUNG 44. March a linear `Tt4` ramp from the running-line start at `Tt4_lo` and return the
    /// marched points beside a running-line-referenced steady-`phi` lookup.
    ///
    /// READ-ONLY: it calls `integrate`/`match` and writes nothing — the surge line, if armed, is
    /// never touched (the rung-41 reduce, one rung on).
    pub fn ramp_march(
        &self, flight: &FlightCondition, tt4_lo: f64, dtt4: f64, r_ramp: f64, s_end: f64, ds: f64,
    ) -> (Vec<TwoSpoolTransientPoint>, SteadyRef<'_>) {
        let od_lo = self.match_point(flight, tt4_lo);
        let nu0 = (od_lo.n_lp_ratio, od_lo.n_hp_ratio);
        let sched = |t: f64| tt4_lo + dtt4 * 1.0f64.min(t / r_ramp);
        let pts = self.integrate(flight, sched, nu0, s_end, ds);
        (pts, SteadyRef { core: self, cache: HashMap::new() })
    }

    /// RUNG 44. Signed extremum of `phi(s) - phi_steady(Tt4(s))` per spool over a marched `Tt4`
    /// ramp, referenced to the RUNNING LINE. NEGATIVE ⇔ `phi` dips BELOW the steady running line ⇔
    /// TOWARD surge.
    ///
    /// The acceleration case swings BOTH spools toward surge, the LP eating ~1.6–2.2× the HP's; the
    /// excursion is SCHEDULE-slaved — `rho`-invariant, ramp-rate-driven — and NOT the LP-map
    /// complex mode. Every magnitude rides on the maps + the ramp; the SIGN and the LP>HP ordering
    /// are the load-bearing content. Needs NO surge line.
    pub fn phi_excursion(
        &self, flight: &FlightCondition, tt4_lo: f64, dtt4: f64, r_ramp: f64, s_end: f64, ds: f64,
    ) -> PhiExcursion {
        let (pts, mut steady) = self.ramp_march(flight, tt4_lo, dtt4, r_ramp, s_end, ds);
        let (mut ext_lp, mut ext_hp) = (0.0f64, 0.0f64);
        let (mut s_lp, mut s_hp) = (0.0f64, 0.0f64);
        let (mut min_phi_lp, mut min_phi_hp) = (f64::INFINITY, f64::INFINITY);
        for p in &pts {
            let e_lp = p.phi_lp - steady.at(flight, p.tt4, Spool::Lp);
            let e_hp = p.phi_hp - steady.at(flight, p.tt4, Spool::Hp);
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
        }
        PhiExcursion {
            ext_lp, ext_hp, s_lp, s_hp, min_phi_lp, min_phi_hp,
            ratio: if ext_hp != 0.0 { ext_lp.abs() / ext_hp.abs() } else { f64::INFINITY },
            npts: pts.len(),
        }
    }

    /// RUNG 44. March the `Tt4` ramp against the IMPOSED `phi_surge` and REPORT the crossing per
    /// spool — the transient analogue of the steady margin, under the rung-36 discipline: REPORT
    /// the crossing, GATE the flip.
    ///
    /// Unlike the steady margin (which ASSERTS the point sits clear), this ALLOWS `phi < phi_surge`
    /// and records it. The crossing DEPTH rides on the imposed `phi_surge` and the ramp rate and is
    /// DISCLAIMED; the load-bearing object is that the transient min margin sits BELOW the steady
    /// one at the same `Tt4`.
    pub fn transient_surge_margin(
        &self, flight: &FlightCondition, tt4_lo: f64, dtt4: f64, r_ramp: f64, s_end: f64, ds: f64,
    ) -> TransientSurgeMargin {
        let (ml, mh) = (self.inner.map_lp, self.inner.map_hp);
        assert!(ml.phi_surge > 0.0 && mh.phi_surge > 0.0,
                "transient_surge_margin needs a surge line on BOTH maps: build each with \
                 with_phi_surge(phi_surge).");
        let (pts, mut steady) = self.ramp_march(flight, tt4_lo, dtt4, r_ramp, s_end, ds);
        let (mut tr_lp, mut tr_hp) = (f64::INFINITY, f64::INFINITY);
        let (mut st_lp, mut st_hp) = (f64::INFINITY, f64::INFINITY);
        for p in &pts {
            tr_lp = tr_lp.min(p.phi_lp - ml.phi_surge);
            tr_hp = tr_hp.min(p.phi_hp - mh.phi_surge);
            st_lp = st_lp.min(steady.at(flight, p.tt4, Spool::Lp) - ml.phi_surge);
            st_hp = st_hp.min(steady.at(flight, p.tt4, Spool::Hp) - mh.phi_surge);
        }
        TransientSurgeMargin {
            margin_min_lp: tr_lp, margin_min_hp: tr_hp, steady_min_lp: st_lp, steady_min_hp: st_hp,
            crossed_lp: tr_lp < 0.0, crossed_hp: tr_hp < 0.0,
            phi_surge_lp: ml.phi_surge, phi_surge_hp: mh.phi_surge, npts: pts.len(),
        }
    }
}

/// The per-`Tt4` steady-`phi` lookup [`TwoSpoolTransientCore::ramp_march`] hands back — Python's
/// `steady` closure over its own `cache` dict.
///
/// **THE KEY IS `round(Tt4, 3)`, A DECIMAL KEY ON A FLOAT DICT**, so the port cannot simply hash
/// the bits: probe 1 measured ONE collision between distinct `Tt4` floats over 31 marches / 5 141
/// points (`1399.9999999999984` and `1400.0`, at the ramp's saturated end), and it FIRES inside the
/// six reported cases while moving **0** reported values. The equivalence relation therefore gets
/// its own gate — the key sequence is a counter and an oracle key — instead of riding on values
/// that structurally cannot see it.
pub struct SteadyRef<'a> {
    core: &'a TwoSpoolTransientCore,
    cache: HashMap<u64, (f64, f64)>,
}

impl SteadyRef<'_> {
    /// One steady lookup. A `match` failure PANICS: Python has no `try` here.
    pub fn at(&mut self, flight: &FlightCondition, tt4: f64, spool: Spool) -> f64 {
        STEADY_CALLS.with(|x| x.set(x.get() + 1));
        let key = round3(tt4);
        let pair = match self.cache.get(&key.to_bits()) {
            Some(v) => *v,
            None => {
                STEADY_MISSES.with(|x| x.set(x.get() + 1));
                STEADY_KEYS.with(|x| x.borrow_mut().push(key));
                STEADY_TT4.with(|x| x.borrow_mut().push(tt4));
                let od: TwoSpoolMapResult = self.core.match_point(flight, tt4);
                let v = (od.phi_lp, od.phi_hp);
                self.cache.insert(key.to_bits(), v);
                v
            }
        };
        match spool {
            Spool::Lp => pair.0,
            Spool::Hp => pair.1,
        }
    }
}

/// Python's `round(x, 3)` — correctly-rounded to three DECIMAL digits, ties to even.
///
/// Format-and-parse, not `(x*1000.0).round()/1000.0`: the latter is a DIFFERENT function (it rounds
/// the SCALED value, and the scaling is itself inexact). Validated against PyPy on all 11 keys that
/// occur on the gate grids and on 7 adversarial half-way ties (`0.0005`, `1399.9995`, `2.6755`, …):
/// **0 mismatches**. Rust's `{:.3}` and CPython/PyPy's `round` are both correctly-rounded with
/// ties-to-even, so they agree by construction on every finite input; the measurement is what makes
/// that a fact rather than a belief about two libraries.
pub fn round3(x: f64) -> f64 {
    format!("{x:.3}").parse::<f64>().expect("formatted float parses")
}

// ---------------------------------------------------------------------------------------------
// R40's three bodies
// ---------------------------------------------------------------------------------------------

/// Close the flow at `(nu_L, nu_H, Tt4)` by the HPT-NGV choke ALONE.
///
/// Both compressor maps run FORWARD (rung 34's move, applied per spool); the HP face's corrected
/// flow follows from the SAME physical air flow through the LP face, so `m_H` is determined by
/// `m_L` — one unknown, one equation. NO shaft balance is used anywhere here: that residual is the
/// whole point of the rung.
fn r40_try_close(
    t: &TwoSpoolTransientCore, nu_lp: f64, nu_hp: f64, tt4: f64, tt2: f64, pt2: f64,
) -> Result<CloseState, Abort> {
    CLOSE_CALLS.with(|x| x.set(x.get() + 1));
    let c = &t.inner;
    let gas = c.gas();
    let n_lp = nu_lp * powp(c.tt2_d / tt2, 0.5);
    let (h2, pr2) = (gas.h_c(tt2), gas.pr_c(tt2));

    let ev = |m_lp: f64| -> Result<CloseState, Abort> {
        let phi_lp = m_lp / n_lp;
        let tau_lpc = 1.0 + (c.tau_lpc_d - 1.0) * c.map_lp.psi(phi_lp) * n_lp * n_lp;
        let tt25 = tt2 * tau_lpc;
        let eta_lpc = c.map_lp.eta_c_at(c.base.eta_lpc, phi_lp, n_lp);
        let h25 = gas.h_c(tt25);
        let pi_lpc = gas.pr_c(gas.t_from_h_c(h2 + eta_lpc * (h25 - h2))) / pr2;
        let pt25 = pi_lpc * pt2;
        // THE FACE FLOW — a LOCAL, and NOT the `mdot_air` this function returns. It exists only to
        // refer the same physical air to the HP face; the returned one is the NGV-imposed
        // `mdot4/(1+f)` below. See [`CloseState`].
        let mdot_air_face = m_lp * c.mcorr_lp_d * pt2 / powp(tt2, 0.5);

        let m_hp = (mdot_air_face * powp(tt25, 0.5) / pt25) / c.mcorr_hp_d;
        let n_hp = nu_hp * powp(c.tt25_d / tt25, 0.5);
        let phi_hp = m_hp / n_hp;
        let tau_hpc = 1.0 + (c.tau_hpc_d - 1.0) * c.map_hp.psi(phi_hp) * n_hp * n_hp;
        let tt3 = tt25 * tau_hpc;
        let eta_hpc = c.map_hp.eta_c_at(c.base.eta_hpc, phi_hp, n_hp);
        let h3 = gas.h_c(tt3);
        let pi_hpc = gas.pr_c(gas.t_from_h_c(h25 + eta_hpc * (h3 - h25))) / gas.pr_c(tt25);
        let pt4 = c.base.pi_b * pi_hpc * pt25;

        let f = c.base.try_solve_f(tt3, pt4, tt4)?;
        let wgas = c.base.try_working_gas(f, tt4, pt4)?;
        let wg = wgas.as_ref().unwrap_or(gas);
        let mdot4 = c.base.a4 * pt4 * choked_mfp(wg, tt4, f) / powp(tt4, 0.5);
        let mdot_imp = mdot4 / (1.0 + f);
        let m_imp = (mdot_imp * powp(tt2, 0.5) / pt2) / c.mcorr_lp_d;
        Ok(CloseState {
            m_lp, m_imp, m_hp, phi_lp, phi_hp, tt2, n_lp, n_hp, tau_lpc, tau_hpc, tt25, tt3,
            pi_lpc, pi_hpc, pt4, f, wgas, eta_lpc, eta_hpc, mdot_air: mdot_imp, mdot4,
        })
    };

    // THE OFF-MAP GUARD (found by rung 57). The high wall below is the LP map's OWN limit, and
    // nothing bounds where that puts the HP FACE: at phi_L = 2.11 it lands at phi_H > 4, where
    // psi_H < -3, tau_hpc < 0 and Tt3 goes NEGATIVE. Python's `gas.pr_c()` then raises a float to a
    // fractional power on a negative base and returns a COMPLEX, which reaches the bracket
    // comparison as a `TypeError` while every caller catches `AssertionError` only.
    //
    // **RUST RETURNS NaN WHERE PYTHON RETURNS A COMPLEX**, so the port's test is Python's `r == r`
    // inverted — `is_nan()`, which for `f64` IS `r != r` (clippy denies the literal spelling). And
    // NOT `is_finite()`: Python's guard is `isinstance(r, float) and r == r`, which an INFINITY
    // passes. Narrowing it to finiteness here would be a different function on an input class
    // neither language rejects.
    let g = |m: f64| -> Result<f64, Abort> {
        let r = m - ev(m)?.m_imp;
        if r.is_nan() {
            CLOSE_NONREAL.with(|x| x.set(x.get() + 1));
            return Err(Abort(format!(
                "off-map compressor trial at m_lp={m:.4}: the loading law has gone non-physical \
                 (Tt3 < 0 => a complex pressure ratio).")));
        }
        Ok(r)
    };

    // `g` is monotone-increasing (more flow -> lower psi -> lower pi_c -> lower pt4 -> less imposed
    // flow), so it brackets cleanly. BOTH arms of the high wall are LIVE — 1 221 literal against
    // 5 118 map over the gate grids, unlike rung 37's comparable ceiling, which bound on one arm 15
    // of 15 times. Counted, not assumed.
    let wall_map = c.map_lp.phi_max(0.1) * n_lp;
    if 2.5 <= wall_map {
        HI_WALL_LITERAL.with(|x| x.set(x.get() + 1));
    } else {
        HI_WALL_MAP.with(|x| x.set(x.get() + 1));
    }
    let hi = 2.5f64.min(wall_map);
    // Python evaluates `ghi = g(hi)` OUTSIDE the try: a failure at the high wall PROPAGATES.
    let ghi = g(hi)?;

    // March the LOW wall IN: at very small m_lp the pressure ratio explodes and the reacting-gas
    // equilibrium solve can fail there — an off-map bracket artifact, not a physical bound.
    // MEASURED DEAD: 0 advances in 6 339 calls (69 440 `g` evaluations, histogram {0: 6339}), so
    // the loop is spelled exactly — `g(m)` evaluated BEFORE `lo` is bound, and `m += 0.02` as
    // repeated addition rather than `0.02*(k+1)` — and counted against zero beside its call count.
    let (mut lo, mut glo, mut m) = (None, 0.0f64, 0.02f64);
    while m < hi {
        match g(m) {
            Ok(v) => {
                glo = v;
                lo = Some(m);
                break;
            }
            Err(_) => {
                MARCH_IN_ADVANCES.with(|x| x.set(x.get() + 1));
                m += 0.02;
            }
        }
    }
    let Some(lo) = lo.filter(|_| glo < 0.0 && 0.0 < ghi) else {
        CLOSE_BRACKET_FAILS.with(|x| x.set(x.get() + 1));
        return Err(Abort(format!(
            "rung-40 two-shaft closure does not bracket at nu=({nu_lp:.4},{nu_hp:.4}), \
             Tt4={tt4:.0} — off the modeled speed-line region.")));
    };
    let root =
        try_illinois(g, lo, hi, glo, ghi, TwoSpoolTransientCore::CLOSE_TOL, ILLINOIS_MAXIT)?;
    ev(root)
}

/// The turbine / power / thrust tail of the instant, shared with rung 43's FUEL control.
///
/// Factored exactly as rung 35 factored the single-spool tail; the rung-40 suite passing unchanged
/// is the bit-for-bit witness.
#[allow(clippy::too_many_arguments)]
fn r40_try_instant_tail(
    t: &TwoSpoolTransientCore, flight: &FlightCondition, c: &CloseState, nu_lp: f64, nu_hp: f64,
    tt4: f64, v0: f64,
) -> Result<Instant2, Abort> {
    let core = &t.inner;
    let tt2 = c.tt2;
    let (wgas, f) = (c.gas(core), c.f);

    // Both turbines pinned by GEOMETRY (rung 38's choke chained twice) — no shaft balance.
    let nu_hpt = nu_hp * powp(core.tt4_d / tt4, 0.5);
    let eta_hpt = core.map_hp.eta_t_at(core.base.eta_hpt, nu_hpt);
    let (pi_hpt, tau_hpt, tt45) = core.base.try_solve_choked_turbine(
        wgas, tt4, f, core.base.a4, core.base.a45, 1.0, eta_hpt)?;
    let nu_lpt = nu_lp * powp(core.tt45_d / tt45, 0.5);
    let eta_lpt = core.map_lp.eta_t_at(core.base.eta_lpt, nu_lpt);
    let (pi_lpt, tau_lpt, tt5) = core.base.try_solve_choked_turbine(
        wgas, tt45, f, core.base.a45, core.base.a8, core.base.pi_n, eta_lpt)?;

    // Specific powers, per unit AIR mass, per shaft.
    let pt_hp = core.base.eta_m * (1.0 + f) * (wgas.h_t(tt4, f) - wgas.h_t(tt45, f));
    let pt_lp = core.base.eta_m * (1.0 + f) * (wgas.h_t(tt45, f) - wgas.h_t(tt5, f));
    let pc_hp = wgas.h_c(c.tt3) - wgas.h_c(c.tt25);
    let pc_lp = wgas.h_c(c.tt25) - wgas.h_c(tt2);

    let phi_hp_dot = (c.mdot_air * (pt_hp - pc_hp)) / (t.p_ref_hp * nu_hp);
    let phi_lp_dot = (c.mdot_air * (pt_lp - pc_lp)) / (t.p_ref_lp * nu_lp);

    let s5 = FlowState { tt: tt5, pt: pi_lpt * pi_hpt * c.pt4, mdot: c.mdot_air, far: f };
    let exit = Nozzle::convergent(core.base.p_ambient, core.base.pi_n).try_apply(&s5, wgas)?;
    let press = (1.0 + f) * wgas.r_t_at(f) * exit.t9 * (1.0 - flight.p0 / exit.p9) / exit.v9;
    let sp_thrust = (1.0 + f) * exit.v9 - v0 + press;

    Ok(Instant2 {
        close: c.clone(),
        nu_lp, nu_hp, tt4, slip: nu_lp / nu_hp, phi_lp_dot, phi_hp_dot, pt_lp, pt_hp, pc_lp, pc_hp,
        tt45, tt5, tau_hpt, tau_lpt, pi_hpt, pi_lpt, eta_hpt, eta_lpt, nu_hpt, nu_lpt, sp_thrust,
        m9: exit.m9,
        branch: if exit.p9 > core.base.p_ambient + 1e-6 {
            Branch::Choked
        } else {
            Branch::Subsonic
        },
    })
}

/// `(Phi_L, Phi_H)` from an already-closed flow — the Newton's inner loop.
///
/// `flight` is UNUSED, in the Python too: the tail's nozzle and thrust are what read it, and this
/// body deliberately stops short of them. Kept in the signature because it is a hook every phase-7
/// override implements against.
fn r40_powers(
    t: &TwoSpoolTransientCore, c: &CloseState, _flight: &FlightCondition, nu_lp: f64, nu_hp: f64,
    tt4: f64,
) -> Result<(f64, f64), Abort> {
    POWERS_CALLS.with(|x| x.set(x.get() + 1));
    let core = &t.inner;
    let (wgas, f) = (c.gas(core), c.f);
    let nu_hpt = nu_hp * powp(core.tt4_d / tt4, 0.5);
    let (_, _, tt45) = core.base.try_solve_choked_turbine(
        wgas, tt4, f, core.base.a4, core.base.a45, 1.0,
        core.map_hp.eta_t_at(core.base.eta_hpt, nu_hpt))?;
    let nu_lpt = nu_lp * powp(core.tt45_d / tt45, 0.5);
    let (_, _, tt5) = core.base.try_solve_choked_turbine(
        wgas, tt45, f, core.base.a45, core.base.a8, core.base.pi_n,
        core.map_lp.eta_t_at(core.base.eta_lpt, nu_lpt))?;
    let pt_hp = core.base.eta_m * (1.0 + f) * (wgas.h_t(tt4, f) - wgas.h_t(tt45, f));
    let pt_lp = core.base.eta_m * (1.0 + f) * (wgas.h_t(tt45, f) - wgas.h_t(tt5, f));
    let pc_hp = wgas.h_c(c.tt3) - wgas.h_c(c.tt25);
    let pc_lp = wgas.h_c(c.tt25) - wgas.h_c(c.tt2);
    Ok(((c.mdot_air * (pt_lp - pc_lp)) / (t.p_ref_lp * nu_lp),
        (c.mdot_air * (pt_hp - pc_hp)) / (t.p_ref_hp * nu_hp)))
}
