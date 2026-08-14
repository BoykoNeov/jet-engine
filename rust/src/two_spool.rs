//! TWO-SPOOL matching — a second shaft, a third choked throat (rungs 38 and 39).
//!
//! Port of `turbojet/engine.py`'s `build_two_spool_turbojet` / `TwoSpoolEngine` /
//! `TwoSpoolMatcher` / `TwoSpoolMapMatcher` (phase 5 slice K of `docs/plans/todo-rust-port.md`,
//! § 5.7). It arrives beside [`crate::matcher`] and [`crate::map`] rather than inside either:
//! rung 38 + 39 is ~940 lines of Python and would push `matcher.rs` past 1 900, and the
//! dependency is strictly one-way — this module consumes [`OffDesignMatcher`], [`MapMatcher`],
//! [`ComponentMap`], the components and the gas, and nothing consumes it until slice L.
//!
//! **THE RUNG (38).** Rung 31 pinned ONE turbine between two choked throats. Split the shaft and
//! there are THREE: `A4` (the HP NGV), `A45` (the LP NGV) and `A8` (the nozzle). The (★) trick
//! chains TWICE, once per throat pair, and the same parameterised solver serves both:
//!
//! ```text
//! (★-HP)  pi_HPT / sqrt(tau_HPT)  =  A4·MFP(Tt4)  / (A45·1   ·MFP(Tt45))
//! (★-LP)  pi_LPT / sqrt(tau_LPT)  =  A45·MFP(Tt45)/ (A8 ·pi_n·MFP(Tt5))
//! ```
//!
//! Both turbines are then pure geometry, and the two shaft balances hand back the two
//! compressors — **in a TRIANGLE, not a 2×2 solve**. Step 3 (`pi_LPC`) reads no HP quantity at
//! all, and step 4 (`pi_HPC`) reads step 3's `Tt25`. Rung 38's finding is that triangularity,
//! and § 5.7's port of it makes the claim structural: [`hp_eta_loop_closed`] below is a FREE
//! function whose parameter list contains no LP efficiency and no LP pressure ratio, so the
//! closure is a scope fact rather than a numerical coincidence.
//!
//! **THE RUNG (39).** Rung 38 predicted a real map would reintroduce the coupling and force a
//! joint solve. It does not. Referring the HP NGV choke to the HP compressor face cancels
//! `pi_LPC` exactly — `pt4/pt25 = pi_b·pi_HPC` — so the HP map coordinate pair is a closed fixed
//! point in `pi_HPC` alone, while the LP face carries the PRODUCT. The map opens **exactly one
//! arrow, HP → LP**: the cascade is not dissolved, it acquires a DIRECTION. Two maps also mean
//! two shaft speeds, hence the slip `N_L/N_H` — the structural novelty rung 38 has no way to
//! compute.
//!
//! # What this module does that slices I and J did not have to
//!
//! **1. `lp_disabled` IS AN ENUM, WHICH MAKES "EXACT DISPATCH" A COMPILER FACT.** Python's
//! constructors return early holding a delegate and set `self._degenerate`; every later method
//! opens with `if self._degenerate is not None: return self._degenerate.match(...)`. The
//! docstring insists this is "exact dispatch, not a knob-to-zero limit". Here that is the type:
//! [`TwoSpoolMatcher::Degenerate`] holds an [`OffDesignMatcher`] and no two-spool field exists on
//! that variant, so the cascade is not merely unentered — it is unreachable. The reduce ladder
//! closes inside this slice: flat + disabled → rung 31, shaped + disabled → rung 32, flat
//! two-spool → rung 38, shaped two-spool → rung 39.
//!
//! **2. THREE HOOKS WITH ZERO LIVE CALL SITES, AND THAT IS DELIBERATE.** § 5.3's census found
//! `match` overridden by rung 42 and called on `self` at three sites — all three inside rung-41
//! methods (`surge_margin`, `running_line_map`, `flow_coefficient_turn`), which are **slice L's**
//! — and `_hp_eta_loop`/`_lp_eta_loop` overridden by rung 55, phase 7. So [`TwoSpoolHooks`]
//! ships today with nothing dispatching through it, exactly as slice I's `solve_turbine` did
//! (unexercised until phase 6). A hook's job is to exist on the day the overriding rung lands; an
//! instrument's job is to fire, which is why slice J deleted a dead `debug_assert_eq!` and this
//! is kept. **The one thing measured and NOT covered:** rung 55's override reads `self.stack_hp`,
//! a phase-7 field. Phase 7 therefore adds a field to [`TwoSpoolMapCore`] (additive) rather than
//! changing this call site (not additive) — recorded here so it is a known bound and not a
//! surprise.
//!
//! **3. THE REDUCE GATE IS A LOOP SHAPE, NOT A VALUE.** Rung 39's gate 1 — flat maps reproduce
//! rung 38 bit-for-bit — holds because both efficiency loops test the residual BEFORE ever
//! calling the secant, so on a flat map they return having done no secant arithmetic at all, and
//! because the outer turbine-efficiency loop returns on its first pass at `a_t == 0`. A
//! `do`-while shape converges to the same place, looks correct, and breaks the reduce for a
//! reason that reads as a solver artefact. Both loops below are check-first.
//!
//! # Two deliberate duplications, preserved
//!
//! [`TwoSpoolCore::cascade`] and [`TwoSpoolMapCore::cascade_map`] are rung 38's and rung 39's own
//! bodies. The Python leaves rung 38's `match`/`_cascade` "LITERALLY untouched" so the rung-38
//! suite still witnesses them, and so does this. Likewise the HP and LP efficiency loops look
//! like one function with a flag and are not: the HP one is CLOSED and the LP one carries
//! `pi_hpc` — the single arrow the whole rung is about. Folding them together would erase the
//! finding into a parameter. Slice F's lesson, twice.

use crate::components::{choked_mfp, ram_recovery, Burner, Component, Compressor, Inlet, Nozzle,
                        Turbine};
use crate::engine::{score, try_score, Engine, EngineResult, FlightCondition, Performance};
use crate::gas::{powp, Abort, FlowState, Gas};
use crate::map::{ComponentMap, MapMatcher, MapOffDesignResult};
use crate::matcher::{OffDesignMatcher, OffDesignResult};

/// Counters that live INSIDE the shipped loops, for the counts § 5.7 gates.
///
/// Same justification as [`crate::map::psi_calls`] and [`OffDesignMatcher::tau_calls`]: the
/// numbers P2/P4 compare against are properties of loops whose bodies are the thing under test,
/// so the only alternatives are a counter in the shipped loop or a copy of the loop in the gate
/// — and a copy would gate the copy. Python observes the same loops through a delegating
/// subclass. All are `u64` increments: no float arithmetic, so an instrument cannot perturb a
/// value.
///
/// Thread-locals rather than struct fields because three of the five live in FREE functions
/// ([`hp_eta_loop_closed`], [`lp_eta_loop_arrow`], [`secant`]) — which is itself the point of
/// those functions being free (§ 5.7 P1).
pub mod counters {
    use std::cell::Cell;

    thread_local! {
        static CASCADE_CALLS: Cell<u64> = const { Cell::new(0) };
        static TURB_MIN: Cell<u64> = const { Cell::new(u64::MAX) };
        static TURB_MAX: Cell<u64> = const { Cell::new(0) };
        static HP_MAX: Cell<u64> = const { Cell::new(0) };
        static LP_MAX: Cell<u64> = const { Cell::new(0) };
        static HP_MIN: Cell<u64> = const { Cell::new(u64::MAX) };
        static LP_MIN: Cell<u64> = const { Cell::new(u64::MAX) };
        static CLAMPS: Cell<u64> = const { Cell::new(0) };
    }

    /// Zero every counter. Called per cell by the oracle gate, so a count is per-cell and not
    /// cumulative-since-process-start.
    pub fn reset() {
        CASCADE_CALLS.with(|c| c.set(0));
        TURB_MIN.with(|c| c.set(u64::MAX));
        TURB_MAX.with(|c| c.set(0));
        HP_MAX.with(|c| c.set(0));
        LP_MAX.with(|c| c.set(0));
        HP_MIN.with(|c| c.set(u64::MAX));
        LP_MIN.with(|c| c.set(u64::MAX));
        CLAMPS.with(|c| c.set(0));
    }

    pub(super) fn bump_cascade() { CASCADE_CALLS.with(|c| c.set(c.get() + 1)); }
    pub(super) fn note_turb(n: u64) {
        TURB_MIN.with(|c| c.set(c.get().min(n)));
        TURB_MAX.with(|c| c.set(c.get().max(n)));
    }
    pub(super) fn note_hp(n: u64) {
        HP_MAX.with(|c| c.set(c.get().max(n)));
        HP_MIN.with(|c| c.set(c.get().min(n)));
    }
    pub(super) fn note_lp(n: u64) {
        LP_MAX.with(|c| c.set(c.get().max(n)));
        LP_MIN.with(|c| c.set(c.get().min(n)));
    }
    pub(super) fn bump_clamp() { CLAMPS.with(|c| c.set(c.get() + 1)); }

    /// Passes of the JOINT `(f, pt4)` fixed point — one per cascade call.
    pub fn cascade_calls() -> u64 { CASCADE_CALLS.with(|c| c.get()) }
    /// Fewest / most passes of the OUTER turbine-efficiency loop in any one cascade.
    pub fn turb_passes_min() -> u64 { TURB_MIN.with(|c| c.get()) }
    pub fn turb_passes_max() -> u64 { TURB_MAX.with(|c| c.get()) }
    /// Most secant steps taken by one HP / LP efficiency loop. These witness that the
    /// `ETA_MAX = 80` cap is nowhere near approached (measured 4), NOT the loop's shape.
    pub fn hp_passes_max() -> u64 { HP_MAX.with(|c| c.get()) }
    pub fn lp_passes_max() -> u64 { LP_MAX.with(|c| c.get()) }
    /// FEWEST secant steps taken by one HP / LP efficiency loop — **and this is the pair
    /// that witnesses the CHECK-FIRST shape.** On a flat map the residual passes on entry,
    /// so the loop returns having called the secant ZERO times; a `do`-while would make the
    /// minimum 1 while leaving the maximum at 4, which is why the maxima alone are blind to
    /// the defect the module note names.
    pub fn hp_passes_min() -> u64 { HP_MIN.with(|c| c.get()) }
    pub fn lp_passes_min() -> u64 { LP_MIN.with(|c| c.get()) }
    /// How often the secant's `[0.3, 1.0]` clamp BOUND. Measured `0` everywhere (§ 5.7 (g)).
    pub fn secant_clamp_hits() -> u64 { CLAMPS.with(|c| c.get()) }
}

// =========================================================================================
// THE DESIGN-POINT CYCLE — deliberately NOT an `Engine`
// =========================================================================================

/// The rung-38 loss parameters, all defaulting to IDEAL.
///
/// A separate type from [`crate::engine::Losses`] because a two-spool machine has four component
/// efficiencies where a single spool has two, and collapsing them onto one struct with two
/// ignored fields is how a caller silently sets the wrong turbine. Python spells this as a
/// separate keyword list on a separate factory, for the same reason.
#[derive(Clone, Copy, Debug)]
pub struct TwoSpoolLosses {
    pub pi_d: f64,
    pub eta_lpc: f64,
    pub eta_hpc: f64,
    pub eta_b: f64,
    pub pi_b: f64,
    pub eta_hpt: f64,
    pub eta_lpt: f64,
    pub eta_m: f64,
    pub pi_n: f64,
    pub p_exit: Option<f64>,
    /// RUNG 30's fixed convergent nozzle. **Rung 38 REQUIRES it** — `A8` is the throat area of a
    /// convergent nozzle and there is no such area without one.
    pub nozzle_convergent: bool,
}

impl Default for TwoSpoolLosses {
    fn default() -> Self {
        TwoSpoolLosses {
            pi_d: 1.0, eta_lpc: 1.0, eta_hpc: 1.0, eta_b: 1.0, pi_b: 1.0,
            eta_hpt: 1.0, eta_lpt: 1.0, eta_m: 1.0, pi_n: 1.0, p_exit: None,
            nozzle_convergent: false,
        }
    }
}

/// Factory: wire a plain (no-bypass) two-spool turbojet, LPC+LPT / HPC+HPT.
///
/// Order: Inlet → LPC → HPC → Burner → HPT → LPT → Nozzle. Isentropic knobs only (rung-31
/// parity — no polytropic `e_c`/`e_t` here). A SEPARATE entry point from
/// [`crate::engine::build_turbojet`], so it never touches the single-spool design path.
pub fn build_two_spool_turbojet(
    gas: Gas, pi_lpc: f64, pi_hpc: f64, tt4: f64, p_ambient: f64, losses: TwoSpoolLosses,
) -> TwoSpoolEngine {
    let components: Vec<(&'static str, Component)> = vec![
        ("2", Component::Inlet(Inlet::new(losses.pi_d))),
        ("25", Component::Compressor(Compressor::new(pi_lpc, losses.eta_lpc, None))),
        ("3", Component::Compressor(Compressor::new(pi_hpc, losses.eta_hpc, None))),
        ("4", Component::Burner(Burner::new(tt4, losses.eta_b, losses.pi_b))),
        ("45", Component::Turbine(Turbine::new(losses.eta_hpt, None))),
        ("5", Component::Turbine(Turbine::new(losses.eta_lpt, None))),
        ("9", Component::Nozzle(if losses.nozzle_convergent {
            Nozzle::convergent(p_ambient, losses.pi_n)
        } else {
            Nozzle::new(p_ambient, losses.pi_n, losses.p_exit)
        })),
    ];
    TwoSpoolEngine::new(gas, components, losses.eta_m)
}

/// The two-spool design-point cycle: chains the components, closing BOTH shaft balances.
///
/// **Deliberately not an [`Engine`], and it does not call [`Engine::run`]** — so the
/// single-shaft-balance logic every rung-6-and-below cycle depends on is never touched. Each
/// shaft is closed exactly the way `Engine::run` closes its one shaft (enthalpy + `eta_m`
/// balance, then the closure assert), applied twice: HP (25→3 drives 4→45) and LP (2→25 drives
/// 45→5).
///
/// In Rust the separation is stronger than "not a subclass": `Engine::run`'s shaft step reads
/// stations "3" and "2" by name off its own table, so a two-spool component list handed to it
/// would close the WRONG shaft rather than fail. The type keeps that from being expressible.
pub struct TwoSpoolEngine {
    pub gas: Gas,
    pub components: Vec<(&'static str, Component)>,
    pub eta_m: f64,
    /// A bare [`Engine`] held ONLY to reuse [`Engine::freestream`]. It owns the gas; see
    /// [`TwoSpoolEngine::gas`].
    fs_engine: Engine,
}

impl TwoSpoolEngine {
    pub fn new(gas: Gas, components: Vec<(&'static str, Component)>, eta_m: f64) -> Self {
        let fs_engine = Engine::new(gas.clone(), Vec::new(), eta_m);
        TwoSpoolEngine { gas, components, eta_m, fs_engine }
    }

    /// Propagate the flow 0 → 9, closing both shafts.
    pub fn run(&self, flight: &FlightCondition, mdot: f64) -> EngineResult {
        let gas = &self.gas;
        let (state0, v0) = self.fs_engine.freestream(flight, mdot);

        let by = |label: &str| -> Component {
            self.components.iter().find(|(l, _)| *l == label)
                .unwrap_or_else(|| panic!("no component at station {label}")).1
        };
        let (inlet, lpc, hpc) = (by("2"), by("25"), by("3"));
        let (burner, hpt, lpt, nozzle) = (by("4"), by("45"), by("5"), by("9"));

        let s2 = inlet.apply(&state0, gas);
        let s25 = lpc.apply(&s2, gas);
        let s3 = hpc.apply(&s25, gas);
        let s4 = burner.apply(&s3, gas);
        let f = s4.far;

        // HP shaft: the HPT (station 45) drives the HPC (25 → 3) ALONE.
        let dh_hpc = gas.h_c(s3.tt) - gas.h_c(s25.tt);
        let s45 = match hpt {
            Component::Turbine(t) => t.apply(&s4, gas, dh_hpc / (self.eta_m * (1.0 + f))),
            _ => panic!("station 45 must be a Turbine"),
        };
        let turbine_power_hp = self.eta_m * (1.0 + s45.far)
            * (gas.h_t(s4.tt, s45.far) - gas.h_t(s45.tt, s45.far));
        assert!((turbine_power_hp - dh_hpc).abs() < 1e-6 * dh_hpc, "HP shaft does not close");

        // LP shaft: the LPT (station 5) drives the LPC (2 → 25) ALONE.
        let dh_lpc = gas.h_c(s25.tt) - gas.h_c(s2.tt);
        let s5 = match lpt {
            Component::Turbine(t) => t.apply(&s45, gas, dh_lpc / (self.eta_m * (1.0 + f))),
            _ => panic!("station 5 must be a Turbine"),
        };
        let turbine_power_lp = self.eta_m * (1.0 + s5.far)
            * (gas.h_t(s45.tt, s5.far) - gas.h_t(s5.tt, s5.far));
        assert!((turbine_power_lp - dh_lpc).abs() < 1e-6 * dh_lpc, "LP shaft does not close");

        let exit = match nozzle {
            Component::Nozzle(n) => n.apply(&s5, gas),
            _ => panic!("station 9 must be a Nozzle"),
        };

        let stations = vec![
            ("0", state0), ("2", s2), ("25", s25), ("3", s3), ("4", s4),
            ("45", s45), ("5", s5), ("9", exit.state),
        ];
        let performance = score(gas, &stations, v0, exit.t9, exit.v9, exit.p9,
                                flight.p0, gas.hpr());
        EngineResult {
            stations, performance, v0, v9: exit.v9, m9: exit.m9, t9: exit.t9, p9: exit.p9,
        }
    }
}

// =========================================================================================
// THE RESULTS
// =========================================================================================

/// One matched two-spool off-design operating point (`docs/rung38-spec.md`).
///
/// `pi_lpc`/`pi_hpc` are OUTPUTS of the triangular cascade, exactly as `pi_c` is in rung 31's
/// [`OffDesignResult`] — which this reduces to bit-for-bit when the LP spool is disabled.
#[derive(Clone, Debug)]
pub struct TwoSpoolResult {
    /// Keyed "0", "2", "25", "3", "4", "45", "5", "9", in flow order.
    pub stations: Vec<(&'static str, FlowState)>,
    pub performance: Performance,
    pub v0: f64,
    pub v9: f64,
    pub m9: f64,
    pub t9: f64,
    pub p9: f64,
    /// Absolute thrust `F = mdot_air * specific_thrust`, N.
    pub thrust: f64,
    /// Throttle setting (input).
    pub tt4: f64,
    /// Flight Mach (input).
    pub m0: f64,
    /// LP compressor pressure ratio — OUTPUT.
    pub pi_lpc: f64,
    /// HP compressor pressure ratio — OUTPUT.
    pub pi_hpc: f64,
    pub tau_lpc: f64,
    pub tau_hpc: f64,
    /// `Tt45/Tt4` — pinned by geometry (★-HP).
    pub tau_hpt: f64,
    pub pi_hpt: f64,
    /// `Tt5/Tt45` — pinned by geometry (★-LP).
    pub tau_lpt: f64,
    pub pi_lpt: f64,
    /// Air mass flow — OUTPUT (set by the HP-NGV choke).
    pub mdot_air: f64,
    pub mdot_ratio: f64,
}

impl TwoSpoolResult {
    /// Station lookup by label. Python's `stations` is an insertion-ordered dict.
    pub fn station(&self, label: &str) -> &FlowState {
        &self.stations.iter().find(|(l, _)| *l == label)
            .unwrap_or_else(|| panic!("no station {label}")).1
    }
}

/// A matched two-spool point WITH component maps (`docs/rung39-spec.md`).
///
/// Composition rather than inheritance: Python's `TwoSpoolMapResult` is a dataclass extending
/// `TwoSpoolResult`, and every field it adds is a map READ-OFF. Both efficiencies per spool and
/// both shaft speeds are now OUTPUTS.
#[derive(Clone, Debug)]
pub struct TwoSpoolMapResult {
    pub base: TwoSpoolResult,
    pub eta_lpc: f64,
    pub eta_hpc: f64,
    pub eta_hpt: f64,
    pub eta_lpt: f64,
    /// LP corrected speed (design = 1).
    pub n_lp: f64,
    /// HP corrected speed (design = 1).
    pub n_hp: f64,
    /// Physical LP shaft-speed ratio `N/N_design`.
    pub n_lp_ratio: f64,
    /// Physical HP shaft-speed ratio.
    pub n_hp_ratio: f64,
    /// `N_L/N_H` — the two-shaft novelty. Exactly 1 on CPG + flat maps, structurally.
    pub slip: f64,
    pub phi_lp: f64,
    pub phi_hp: f64,
    pub nu_hpt: f64,
    pub nu_lpt: f64,
}

/// What a `match_point` call returns, given that `lp_disabled` dispatches to a DIFFERENT solver
/// with a different result type.
///
/// Python simply returns whichever object the delegate produced and lets the caller find out.
/// Naming the two cases is what makes rung 38's "exact dispatch, not a knob-to-zero limit"
/// checkable: a caller that wants the two-spool numbers has to say so, and a reduce gate that
/// compares against rung 31 has to unwrap the single-spool arm.
#[derive(Clone, Debug)]
pub enum Matched {
    /// `lp_disabled` — rung 31's own result, forwarded verbatim.
    Single(OffDesignResult),
    /// The two-spool cascade.
    Two(TwoSpoolResult),
}

impl Matched {
    /// The two-spool arm, or panic. For call sites that have already established the matcher is
    /// not degenerate.
    pub fn two(self) -> TwoSpoolResult {
        match self {
            Matched::Two(r) => r,
            _ => panic!("this matcher is lp_disabled: it returned a single-spool result"),
        }
    }
}

// =========================================================================================
// RUNG 38 — THE TWO-SPOOL MATCHER
// =========================================================================================

/// The four numbers an efficiency fixed point returns: `(eta, pi, m, n)`.
#[derive(Clone, Copy, Debug)]
pub struct EtaLoop {
    pub eta: f64,
    pub pi: f64,
    /// Corrected flow at the compressor FACE, normalised on its design value.
    pub m: f64,
    /// Corrected speed.
    pub n: f64,
}

/// Everything rung 38's cascade produces at a fixed `(Tt2, Tt4, f)`.
#[derive(Clone, Copy, Debug)]
pub struct Cascade {
    pub pi_hpt: f64,
    pub tau_hpt: f64,
    pub tt45: f64,
    pub pi_lpt: f64,
    pub tau_lpt: f64,
    pub tt5: f64,
    pub pi_lpc: f64,
    pub tt25: f64,
    pub pi_hpc: f64,
    pub tt3: f64,
}

/// Rung 39's cascade: rung 38's plus the map read-offs.
#[derive(Clone, Copy, Debug)]
pub struct CascadeMap {
    pub c: Cascade,
    pub eta_lpc: f64,
    pub eta_hpc: f64,
    pub eta_hpt: f64,
    pub eta_lpt: f64,
    pub m_l: f64,
    pub m_h: f64,
    pub n_l: f64,
    pub n_h: f64,
    pub nl: f64,
    pub nh: f64,
    pub phi_l: f64,
    pub phi_h: f64,
    pub nu_hpt: f64,
    pub nu_lpt: f64,
    pub slip: f64,
}

/// RUNG 38. Two-spool (LPC + HPC, no bypass) off-design matching.
///
/// ```text
/// let design = build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, p0, losses);
/// let m = TwoSpoolMatcher::new(design, flight_design, 1.0);
/// let od = m.match_point(&flight_od, tt4_od).two();   // pi_lpc, pi_hpc are OUTPUTS
/// ```
pub enum TwoSpoolMatcher {
    /// `lp_disabled` — the REDUCE path. `design_engine` is then a PLAIN single-spool
    /// [`Engine`], no LPC/LPT/`A45` is ever built, and every match is forwarded verbatim.
    Degenerate(OffDesignMatcher),
    Full(TwoSpoolCore),
}

impl TwoSpoolMatcher {
    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    ) -> Self {
        TwoSpoolMatcher::Full(TwoSpoolCore::new(design_engine, flight_design, mdot_design))
    }

    /// `lp_disabled=True`. Takes a SINGLE-spool design engine, because that is what the Python
    /// contract says it is handed — a fact the signature can carry and a boolean cannot.
    pub fn lp_disabled(
        design_engine: Engine, flight_design: FlightCondition, mdot_design: f64,
    ) -> Self {
        TwoSpoolMatcher::Degenerate(
            OffDesignMatcher::new(design_engine, flight_design, mdot_design))
    }

    pub fn match_point(&self, flight: &FlightCondition, tt4: f64) -> Matched {
        match self {
            TwoSpoolMatcher::Degenerate(m) => Matched::Single(m.match_point(flight, tt4)),
            TwoSpoolMatcher::Full(c) => Matched::Two(c.match_point(flight, tt4)),
        }
    }

    /// The two-spool core, or panic — for gates that built a non-degenerate matcher.
    pub fn core(&self) -> &TwoSpoolCore {
        match self {
            TwoSpoolMatcher::Full(c) => c,
            TwoSpoolMatcher::Degenerate(_) => panic!("this matcher is lp_disabled"),
        }
    }

    /// The core, mutably — so a gate can perturb ONE captured efficiency and re-run the
    /// cascade, which is how rung 38 gate 3 and rung 39 gate 4 establish triangularity.
    /// Python does it by assigning the attribute; the captured throat areas are deliberately
    /// left alone, which is why this is not "rebuild the matcher with a different eta".
    pub fn core_mut(&mut self) -> &mut TwoSpoolCore {
        match self {
            TwoSpoolMatcher::Full(c) => c,
            TwoSpoolMatcher::Degenerate(_) => panic!("this matcher is lp_disabled"),
        }
    }
}

/// The captured hardware and the cascade — rung 38's object once `lp_disabled` is ruled out.
pub struct TwoSpoolCore {
    pub eta_m: f64,
    pub flight_design: FlightCondition,
    pub mdot_air_design: f64,
    pub hf_fuel_molar: Option<f64>,
    pub pi_lpc_design: f64,
    pub eta_lpc: f64,
    pub pi_hpc_design: f64,
    pub eta_hpc: f64,
    pub tt4_design: f64,
    pub eta_b: f64,
    pub pi_b: f64,
    pub eta_hpt: f64,
    pub eta_lpt: f64,
    pub p_ambient: f64,
    pub pi_n: f64,
    pub pi_d_max: f64,
    pub f_design: f64,
    /// HP turbine NGV throat area, m².
    pub a4: f64,
    /// LP turbine NGV throat area, m² — **the third throat, and rung 38's structural novelty.**
    pub a45: f64,
    /// Nozzle throat area, m².
    pub a8: f64,
    /// The design run, kept because rung 39 reads five further stations off it.
    pub reference: EngineResult,
    fs_engine: Engine,
    /// How many times [`tau_of`](TwoSpoolCore::tau_of) has been called — see
    /// [`OffDesignMatcher::tau_calls`], same reason. § 5.7 P3 gates the per-call count, which
    /// lives inside the shipped bisection.
    pub tau_calls: std::cell::Cell<u64>,
}

impl TwoSpoolCore {
    pub const TOL: f64 = 1e-13;
    pub const MAX: usize = 200;

    pub fn gas(&self) -> &Gas { &self.fs_engine.gas }

    pub fn freestream_for(&self, flight: &FlightCondition) -> (FlowState, f64) {
        self.fs_engine.freestream(flight, self.mdot_air_design)
    }

    /// The FALLIBLE twin of [`freestream_for`](Self::freestream_for) — see
    /// [`Engine::try_freestream`].
    pub fn try_freestream_for(
        &self, flight: &FlightCondition,
    ) -> Result<(FlowState, f64), Abort> {
        self.fs_engine.try_freestream(flight, self.mdot_air_design)
    }

    /// Capture the fixed hardware from one design run — **three** throat areas, where rung 31
    /// captured two.
    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    ) -> Self {
        let eta_m = design_engine.eta_m;
        let hf_fuel_molar = design_engine.gas.spec.hf_fuel_molar;

        let by = |label: &str| -> Component {
            design_engine.components.iter().find(|(l, _)| *l == label)
                .unwrap_or_else(|| panic!("no component at station {label}")).1
        };
        let (mut pi_lpc_design, mut eta_lpc, mut e_lpc) = (f64::NAN, f64::NAN, None);
        if let Component::Compressor(x) = by("25") {
            pi_lpc_design = x.pi_c; eta_lpc = x.eta_c; e_lpc = x.e_c;
        }
        let (mut pi_hpc_design, mut eta_hpc, mut e_hpc) = (f64::NAN, f64::NAN, None);
        if let Component::Compressor(x) = by("3") {
            pi_hpc_design = x.pi_c; eta_hpc = x.eta_c; e_hpc = x.e_c;
        }
        let (mut tt4_design, mut eta_b, mut pi_b) = (f64::NAN, f64::NAN, f64::NAN);
        if let Component::Burner(x) = by("4") {
            tt4_design = x.tt4; eta_b = x.eta_b; pi_b = x.pi_b;
        }
        let (mut eta_hpt, mut e_hpt) = (f64::NAN, None);
        if let Component::Turbine(x) = by("45") { eta_hpt = x.eta_t; e_hpt = x.e_t; }
        let (mut eta_lpt, mut e_lpt) = (f64::NAN, None);
        if let Component::Turbine(x) = by("5") { eta_lpt = x.eta_t; e_lpt = x.e_t; }
        let (mut p_ambient, mut pi_n, mut nozzle_convergent) = (f64::NAN, f64::NAN, false);
        if let Component::Nozzle(x) = by("9") {
            p_ambient = x.p_ambient; pi_n = x.pi_n; nozzle_convergent = x.convergent;
        }
        let mut pi_d_design = f64::NAN;
        if let Component::Inlet(x) = by("2") { pi_d_design = x.pi_d; }

        // Scope: isentropic knobs only (rung-31 parity).
        assert!(e_lpc.is_none() && e_hpc.is_none() && e_hpt.is_none() && e_lpt.is_none(),
                "rung 38 two-spool matching uses isentropic eta_c/eta_t maps only; \
                 polytropic is out of scope");
        assert!(nozzle_convergent,
                "rung 38 matching needs the FIXED CONVERGENT nozzle (rung 30): build the design \
                 engine with nozzle_convergent so its throat area A8 is defined");

        let pi_d_max = pi_d_design / ram_recovery(flight_design.m0);

        // Run the design cycle ONCE to capture the reference state + the THREE throat areas.
        let reference = design_engine.run(&flight_design, mdot_design);
        let (s4, s45, s5) = (*reference.station("4"), *reference.station("45"),
                             *reference.station("5"));
        let f_design = s4.far;
        let gas = design_engine.gas;
        let mdot4_r = mdot_design * (1.0 + f_design);   // total mass through every throat
        let a4 = mdot4_r * powp(s4.tt, 0.5) / (s4.pt * choked_mfp(&gas, s4.tt, f_design));
        let a45 = mdot4_r * powp(s45.tt, 0.5) / (s45.pt * choked_mfp(&gas, s45.tt, f_design));
        let (tt9_r, pt9_r) = (s5.tt, pi_n * s5.pt);     // Tt9 = Tt5; pt9 = pi_n * pt5
        let a8 = mdot4_r * powp(tt9_r, 0.5) / (pt9_r * choked_mfp(&gas, tt9_r, f_design));

        TwoSpoolCore {
            eta_m, flight_design, mdot_air_design: mdot_design, hf_fuel_molar,
            pi_lpc_design, eta_lpc, pi_hpc_design, eta_hpc, tt4_design, eta_b, pi_b,
            eta_hpt, eta_lpt, p_ambient, pi_n, pi_d_max, f_design, a4, a45, a8, reference,
            fs_engine: Engine::new(gas, Vec::new(), eta_m),
            tau_calls: std::cell::Cell::new(0),
        }
    }

    // --- a gas whose station-4 mixture is frozen at THIS trial burn condition --------------

    /// See [`OffDesignMatcher::working_gas`] — identical need, same solution. `None` means "use
    /// the shared design gas".
    ///
    /// **Slice K shipped this INFALLIBLE, with the reason written down, and SLICE L IS WHERE THE
    /// REASON EXPIRES.** § 5.7 (e) argued "rungs 38/39 contain no `try`/`except` anywhere, so no
    /// caller here marches past a failure and there is nothing for an [`Abort`] to be control
    /// flow *for*". Correct then, and it is a statement about the CALLERS, not about this
    /// function — rung 41 adds the first three (`surge_margin_schedule`, `running_line_map`,
    /// `flow_coefficient_turn`), and the equilibrium Newton raises **14 times** inside their
    /// caught scope on the dump grid. The twin below is what that costs. Kept as a pair of
    /// functions rather than a rewrite, so slice K's gated body is the one still running.
    ///
    /// [`Abort`]: crate::gas::Abort
    pub fn working_gas(&self, f: f64, tt4: f64, pt4: f64) -> Option<Gas> {
        self.try_working_gas(f, tt4, pt4).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE twin of [`working_gas`](Self::working_gas) — see [`Abort`].
    ///
    /// [`Abort`]: crate::gas::Abort
    pub fn try_working_gas(&self, f: f64, tt4: f64, pt4: f64) -> Result<Option<Gas>, Abort> {
        if !self.gas().is_equilibrium() {
            return Ok(None);
        }
        let g = Gas::reacting_equilibrium_with(
            self.hf_fuel_molar.expect("an equilibrium gas carries hf_fuel_molar"), 0.0);
        g.try_freeze_equilibrium(f, tt4, pt4)?;
        Ok(Some(g))
    }

    // --- the shared (★) mechanism: one choked-throat PAIR pins one turbine's tau -----------

    /// `tau_t` and `Tt_out` from the isentropic-efficiency map at a trial `pi_t`.
    ///
    /// Split out of the bisection below because it is called once per residual **and once more
    /// after the loop** — which is why the instrument reads 47 where the loop runs 44 times
    /// (§ 5.7 (d)).
    pub fn tau_of(&self, gas: &Gas, tt_in: f64, f: f64, pi_t: f64, eta: f64) -> (f64, f64) {
        self.try_tau_of(gas, tt_in, f, pi_t, eta).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE twin of [`tau_of`](Self::tau_of) — see [`Abort`].
    ///
    /// **The whole of slice L's new fallible surface below the matcher is this one line**: the
    /// isentropic `t_from_pr_t` at a trial expansion ratio. The two other inversions in this
    /// body (`t_from_h_t`, twice) reach the same solver and raise **0** times on the dump grid,
    /// so they keep their panics — see [`crate::gas::try_solve`].
    ///
    /// The counter is bumped BEFORE the fallible call, so a raising call still counts as a call
    /// — which is what Python's frame count would say, and what the (★) census compares.
    ///
    /// [`Abort`]: crate::gas::Abort
    pub fn try_tau_of(
        &self, gas: &Gas, tt_in: f64, f: f64, pi_t: f64, eta: f64,
    ) -> Result<(f64, f64), Abort> {
        self.tau_calls.set(self.tau_calls.get() + 1);
        let tt_outs = gas.try_t_from_pr_t(gas.pr_t(tt_in, f) * pi_t, f)?;
        let dh_ideal = gas.h_t(tt_in, f) - gas.h_t(tt_outs, f);
        let tt_out = gas.t_from_h_t(gas.h_t(tt_in, f) - eta * dh_ideal, f);
        Ok((tt_out / tt_in, tt_out))
    }

    /// Bisect `pi_t` so `pi_t/sqrt(tau_t) = A_in·MFP(Tt_in) / (A_out·pi_loss·MFP(Tt_out))`.
    ///
    /// **THE (★) TRICK, PARAMETERISED SO IT SERVES BOTH TURBINES** — (★-HP) is
    /// `A_in = A4, A_out = A45, pi_loss = 1` (no loss modelled in the inter-turbine duct);
    /// (★-LP) is `A_in = A45, A_out = A8, pi_loss = pi_n` (the nozzle's real loss). That ONE
    /// function serving two throats pairs is rung 38's whole structural content: a second shaft
    /// costs a third throat, not a second mechanism.
    ///
    /// Same monotone bracket and tolerance as [`OffDesignMatcher`]'s single-spool solve. `TOL`
    /// is used **absolutely** here, so the loop runs `ceil(log2(0.979/1e-13)) = 44` times at
    /// every call, with no spread — measured over 10 502 calls on both interpreters.
    /// Returns `(pi_t, tau_t, Tt_out)`.
    pub fn solve_choked_turbine(
        &self, gas: &Gas, tt_in: f64, f: f64, a_in: f64, a_out: f64, pi_loss: f64, eta: f64,
    ) -> (f64, f64, f64) {
        self.try_solve_choked_turbine(gas, tt_in, f, a_in, a_out, pi_loss, eta)
            .unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE twin of [`solve_choked_turbine`](Self::solve_choked_turbine) — see
    /// [`Abort`].
    ///
    /// The bracket-straddle guard STAYS an `assert!` inside the fallible body, and that is not
    /// an oversight: it fires **0** times across slice K's and slice L's grids, so making it an
    /// `Abort` would add a control-flow path with no reachable failure — a gate that measures
    /// nothing (the [`Abort`] rule's own words). What propagates is the residual's inversion.
    ///
    /// [`Abort`]: crate::gas::Abort
    #[allow(clippy::too_many_arguments)]
    pub fn try_solve_choked_turbine(
        &self, gas: &Gas, tt_in: f64, f: f64, a_in: f64, a_out: f64, pi_loss: f64, eta: f64,
    ) -> Result<(f64, f64, f64), Abort> {
        let mfp_in = choked_mfp(gas, tt_in, f);
        let resid = |pi_t: f64| -> Result<f64, Abort> {
            let (tau_t, tt_out) = self.try_tau_of(gas, tt_in, f, pi_t, eta)?;
            let mfp_out = choked_mfp(gas, tt_out, f);
            let rhs = a_in * mfp_in / (a_out * pi_loss * mfp_out);
            Ok(pi_t / powp(tau_t, 0.5) - rhs)
        };

        let (mut lo, mut hi) = (0.02, 0.999);
        let (mut flo, fhi) = (resid(lo)?, resid(hi)?);
        assert!(flo < 0.0 && 0.0 < fhi,
                "rung-38 turbine choke-match bracket does not straddle the root");
        for _ in 0..Self::MAX {
            let mid = 0.5 * (lo + hi);
            let fm = resid(mid)?;
            if flo * fm <= 0.0 {
                hi = mid;
            } else {
                lo = mid;
                flo = fm;
            }
            if hi - lo <= Self::TOL {
                break;
            }
        }
        let pi_t = 0.5 * (lo + hi);
        let (tau_t, tt_out) = self.try_tau_of(gas, tt_in, f, pi_t, eta)?;
        Ok((pi_t, tau_t, tt_out))
    }

    // --- the burner f-solve (reuses the shipped burner formulas) ---------------------------

    pub fn solve_f(&self, tt3: f64, pt4: f64, tt4: f64) -> f64 {
        self.try_solve_f(tt3, pt4, tt4).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE twin of [`solve_f`](Self::solve_f) — see [`Abort`]. Both arms can raise
    /// inside rung 41's caught scope: the equilibrium burner 0 times and this loop's own
    /// non-convergence 3 times on the dump grid.
    ///
    /// [`Abort`]: crate::gas::Abort
    pub fn try_solve_f(&self, tt3: f64, pt4: f64, tt4: f64) -> Result<f64, Abort> {
        let gas = self.gas();
        if gas.is_equilibrium() {
            return Burner::new(tt4, self.eta_b, self.pi_b)
                .try_solve_equilibrium(tt3, pt4, gas);
        }
        let h3 = gas.h_c(tt3);
        let mut f = 0.0;
        for _ in 0..Self::MAX {
            let h4 = gas.h_t(tt4, f);
            let f_new = (h4 - h3) / (self.eta_b * gas.hpr() - h4);
            if (f_new - f).abs() <= Self::TOL * (f_new + 1e-30) {
                return Ok(f_new);
            }
            f = f_new;
        }
        Err(Abort("rung-38 off-design burner f did not converge".to_string()))
    }

    // --- the triangular cascade at a FIXED (Tt2, Tt4, f) -----------------------------------

    /// Steps 1–4 of `docs/rung38-spec.md`, at a FIXED scalar `f` — the one shared state.
    ///
    /// **THE TRIANGULARITY IS A CODE-LEVEL GUARANTEE, NOT A NUMERICAL COINCIDENCE.** Step 3
    /// (`pi_lpc`) reads only `eta_lpc`, `A45`, `A8`, `eta_lpt`, `eta_m` and `(Tt2, Tt4, f)` —
    /// never `eta_hpc` and never `pi_hpc_design`. Step 4 then reads step 3's `Tt25`. Exposed as
    /// its own method, as the Python's is, so a gate can perturb one spool's constants at a
    /// FIXED `(Tt2, Tt4, f)` and the outer joint loop cannot confound the reading.
    pub fn cascade(&self, wgas: &Gas, tt2: f64, tt4: f64, f: f64) -> Cascade {
        self.try_cascade(wgas, tt2, tt4, f).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE twin of [`cascade`](Self::cascade) — see [`Abort`].
    ///
    /// [`Abort`]: crate::gas::Abort
    pub fn try_cascade(
        &self, wgas: &Gas, tt2: f64, tt4: f64, f: f64,
    ) -> Result<Cascade, Abort> {
        counters::bump_cascade();
        // Step 1 (★-HP): tau_HPT from (A4, A45) alone.
        let (pi_hpt, tau_hpt, tt45) =
            self.try_solve_choked_turbine(wgas, tt4, f, self.a4, self.a45, 1.0, self.eta_hpt)?;
        // Step 2 (★-LP): tau_LPT from (A45, A8) alone — needs the nozzle choked.
        let (pi_lpt, tau_lpt, tt5) = self.try_solve_choked_turbine(
            wgas, tt45, f, self.a45, self.a8, self.pi_n, self.eta_lpt)?;

        // Step 3: LP shaft balance -> pi_LPC. NO reference to the HP spool.
        let dh_lpt = self.eta_m * (1.0 + f) * (wgas.h_t(tt45, f) - wgas.h_t(tt5, f));
        let tt25 = wgas.t_from_h_c(wgas.h_c(tt2) + dh_lpt);
        let (h2, h25) = (wgas.h_c(tt2), wgas.h_c(tt25));
        let tt25s = wgas.t_from_h_c(h2 + self.eta_lpc * (h25 - h2));
        let pi_lpc = wgas.pr_c(tt25s) / wgas.pr_c(tt2);

        // Step 4: HP shaft balance -> pi_HPC. Needs Tt25, just solved in step 3.
        let dh_hpt = self.eta_m * (1.0 + f) * (wgas.h_t(tt4, f) - wgas.h_t(tt45, f));
        let tt3 = wgas.t_from_h_c(wgas.h_c(tt25) + dh_hpt);
        let (h25b, h3) = (wgas.h_c(tt25), wgas.h_c(tt3));
        let tt3s = wgas.t_from_h_c(h25b + self.eta_hpc * (h3 - h25b));
        let pi_hpc = wgas.pr_c(tt3s) / wgas.pr_c(tt25);

        Ok(Cascade { pi_hpt, tau_hpt, tt45, pi_lpt, tau_lpt, tt5, pi_lpc, tt25, pi_hpc, tt3 })
    }

    // --- match one operating point ---------------------------------------------------------

    /// Match the two-spool engine at `(flight, Tt4)`. `pi_lpc`, `pi_hpc` are OUTPUTS.
    ///
    /// Python spells this `match`, which is a Rust keyword.
    ///
    /// The joint `(f, pt4)` fixed point is the ONE place the two spools share state; everything
    /// under it is the triangular cascade with no 2×2 solve. **It caps out far more often than
    /// slice I's single-spool one** — 23 of 105 matched cells against two — and when it does,
    /// both halves of the stopping rule cycle TOGETHER over 2 to 6 distinct values, where the
    /// single spool failed the two halves for different reasons (§ 5.7 (b)). So the answer is
    /// the 200th iterate of a fixed count, and every pass is reproduced deliberately. Worse:
    /// **whether it caps is interpreter-dependent** — CPython and PyPy disagree at 29 of 126
    /// cells, between ~8 passes and never (§ 5.7 (c)).
    pub fn match_point(&self, flight: &FlightCondition, tt4: f64) -> TwoSpoolResult {
        let pi_d = self.pi_d_max * ram_recovery(flight.m0);
        let (state0, _v0) = self.freestream_for(flight);
        let (tt2, pt2) = (state0.tt, pi_d * state0.pt);

        let (mut f, mut pt4) =
            (self.f_design, self.pi_b * self.pi_hpc_design * self.pi_lpc_design * pt2);
        let mut c = Cascade { pi_hpt: f64::NAN, tau_hpt: f64::NAN, tt45: f64::NAN,
                              pi_lpt: f64::NAN, tau_lpt: f64::NAN, tt5: f64::NAN,
                              pi_lpc: f64::NAN, tt25: f64::NAN, pi_hpc: f64::NAN,
                              tt3: f64::NAN };
        for _ in 0..Self::MAX {
            let owned = self.working_gas(f, tt4, pt4);
            let wgas = owned.as_ref().unwrap_or(self.gas());
            c = self.cascade(wgas, tt2, tt4, f);

            let pt4_new = self.pi_b * c.pi_hpc * c.pi_lpc * pt2;
            let f_new = self.solve_f(c.tt3, pt4_new, tt4);
            let done = (f_new - f).abs() <= Self::TOL * (f_new + 1e-30)
                && (pt4_new - pt4).abs() <= Self::TOL * pt4_new;
            f = f_new;
            pt4 = pt4_new;
            if done {
                break;
            }
        }

        assert!(c.pi_lpc > 1.0 && c.pi_hpc > 1.0 && 0.0 < c.tau_hpt && c.tau_hpt < 1.0
                    && 0.0 < c.tau_lpt && c.tau_lpt < 1.0,
                "rung-38 two-spool match unphysical");

        let owned = self.working_gas(f, tt4, pt4);
        let wgas = owned.as_ref().unwrap_or(self.gas());
        let mdot4 = self.a4 * pt4 * choked_mfp(wgas, tt4, f) / powp(tt4, 0.5);
        let mdot_air = mdot4 / (1.0 + f);

        // Rung 38's `match` is DELIBERATELY the infallible one: Python's rung 38 has no caller
        // that catches, and the twin rule is about the CALLERS. Rung 39's — which rung 41 does
        // catch — is the fallible one below.
        let r = self.try_rebuild(flight, pi_d, c.pi_lpc, c.pi_hpc, tt4, mdot_air,
                                 self.eta_lpc, self.eta_hpc, self.eta_hpt, self.eta_lpt)
            .unwrap_or_else(|e| panic!("{}", e.0));
        let nozzle_choked = r.exit_p9 > self.p_ambient + 1e-6;

        // SCOPE GUARD (`docs/rung38-spec.md` § Scope). Unchoke relocates rung 33's inversion one
        // throat upstream onto the LP spool — a genuinely different solve, not built here. FLAG,
        // DON'T LIE, where rung 33 could DISPATCH. It fires on 23 of the 147-cell dump grid, so
        // it is a live band along the cold/slow edge and not a corner (§ 5.7 (e)).
        assert!(nozzle_choked,
                "rung-38 two-spool match at Tt4={tt4:.0}, M0={:.2}: nozzle UNCHOKED -- OUT OF \
                 SCOPE (docs/rung38-spec.md 'Scope'). The LP turbine's geometric tau_LPT pin \
                 (*-LP) is only valid while the nozzle stays choked; a rung-33-shaped follow-on \
                 would resolve the LP spool's own subsonic branch.", flight.m0);

        r.into_result(self, flight, tt4, mdot_air, &c)
    }

    /// The FORWARD rebuild both rungs end with — freestream through nozzle at the derived
    /// `(pi_lpc, pi_hpc, mdot_air)` and the four efficiencies, on a fresh gas.
    ///
    /// Fires every shipped conservation assert (both compressors, burner, both turbines,
    /// nozzle) — rung 31's discipline on two shafts. The four efficiencies are PARAMETERS for
    /// the reason slice J made rung 31's two parameters: rung 38 passes its fixed design values
    /// and reads exactly as before, rung 39 passes the map's values at the operating point, and
    /// the Python duplicates the whole body across the two `match` methods.
    #[allow(clippy::too_many_arguments)]
    fn try_rebuild(
        &self, flight: &FlightCondition, pi_d: f64, pi_lpc: f64, pi_hpc: f64, tt4: f64,
        mdot_air: f64, eta_lpc: f64, eta_hpc: f64, eta_hpt: f64, eta_lpt: f64,
    ) -> Result<TwoSpoolRebuilt, Abort> {
        let rgas = if self.gas().is_equilibrium() {
            Gas::reacting_equilibrium_with(
                self.hf_fuel_molar.expect("an equilibrium gas carries hf_fuel_molar"), 0.0)
        } else {
            self.gas().clone()
        };
        let (state0, v0) = self.fs_engine.try_freestream(flight, mdot_air)?;
        let s2 = Inlet::new(pi_d).apply(&state0, &rgas);
        let s25 = Compressor::new(pi_lpc, eta_lpc, None).apply(&s2, &rgas);
        let s3 = Compressor::new(pi_hpc, eta_hpc, None).apply(&s25, &rgas);
        let s4 = Burner::new(tt4, self.eta_b, self.pi_b).apply(&s3, &rgas);
        let dh_hpt_reb = (rgas.h_c(s3.tt) - rgas.h_c(s25.tt)) / (self.eta_m * (1.0 + s4.far));
        let s45 = Turbine::new(eta_hpt, None).apply(&s4, &rgas, dh_hpt_reb);
        let dh_lpt_reb = (rgas.h_c(s25.tt) - rgas.h_c(s2.tt)) / (self.eta_m * (1.0 + s4.far));
        let s5 = Turbine::new(eta_lpt, None).apply(&s45, &rgas, dh_lpt_reb);
        let exit = Nozzle::convergent(self.p_ambient, self.pi_n).try_apply(&s5, &rgas)?;
        Ok(TwoSpoolRebuilt {
            state0, v0, s2, s25, s3, s4, s45, s5,
            exit_state: exit.state, exit_m9: exit.m9, exit_t9: exit.t9, exit_v9: exit.v9,
            exit_p9: exit.p9, gas: rgas,
        })
    }
}

/// The forward rebuild's eight stations plus the nozzle exit.
struct TwoSpoolRebuilt {
    state0: FlowState,
    v0: f64,
    s2: FlowState,
    s25: FlowState,
    s3: FlowState,
    s4: FlowState,
    s45: FlowState,
    s5: FlowState,
    exit_state: FlowState,
    exit_m9: f64,
    exit_t9: f64,
    exit_v9: f64,
    exit_p9: f64,
    gas: Gas,
}

impl TwoSpoolRebuilt {
    /// Score the rebuilt cycle and assemble the result both rungs return.
    fn into_result(
        self, core: &TwoSpoolCore, flight: &FlightCondition, tt4: f64, mdot_air: f64,
        c: &Cascade,
    ) -> TwoSpoolResult {
        self.try_into_result(core, flight, tt4, mdot_air, c)
            .unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE twin — see [`Abort`]. The only thing that can raise here is
    /// [`try_score`]'s efficiency cascade, and it does: **27 cells** of the dump grid, every one
    /// a cell rung 41's schedule methods skip.
    ///
    /// [`Abort`]: crate::gas::Abort
    fn try_into_result(
        self, core: &TwoSpoolCore, flight: &FlightCondition, tt4: f64, mdot_air: f64,
        c: &Cascade,
    ) -> Result<TwoSpoolResult, Abort> {
        let TwoSpoolRebuilt { state0, v0, s2, s25, s3, s4, s45, s5,
                              exit_state, exit_m9, exit_t9, exit_v9, exit_p9, gas: rgas } = self;
        let stations = vec![
            ("0", state0), ("2", s2), ("25", s25), ("3", s3), ("4", s4),
            ("45", s45), ("5", s5), ("9", exit_state),
        ];
        let perf = try_score(&rgas, &stations, v0, exit_t9, exit_v9, exit_p9, flight.p0,
                             rgas.hpr())?;
        let thrust = mdot_air * perf.specific_thrust;
        Ok(TwoSpoolResult {
            stations, performance: perf, v0, v9: exit_v9, m9: exit_m9, t9: exit_t9, p9: exit_p9,
            thrust, tt4, m0: flight.m0, pi_lpc: c.pi_lpc, pi_hpc: c.pi_hpc,
            tau_lpc: s25.tt / s2.tt, tau_hpc: s3.tt / s25.tt,
            tau_hpt: c.tau_hpt, pi_hpt: c.pi_hpt, tau_lpt: c.tau_lpt, pi_lpt: c.pi_lpt,
            mdot_air, mdot_ratio: mdot_air / core.mdot_air_design,
        })
    }
}

// =========================================================================================
// RUNG 39 — TWO-SPOOL + COMPONENT MAPS: the cascade acquires a DIRECTION
// =========================================================================================
//
// THE ALGEBRA. The HPT NGV choke fixes the corrected flow at station 4; refer it to the HP
// compressor face at station 25. Since pt4 = pi_b*pi_HPC*pi_LPC*pt2 and pt25 = pi_LPC*pt2, the
// ratio pt4/pt25 = pi_b*pi_HPC — pi_LPC CANCELS:
//
//   (†)  mdot_corr,25 = A4 * pi_b * pi_HPC          * MFP*(Tt4,f) * sqrt(Tt25/Tt4) / (1+f)
//   (‡)  mdot_corr,2  = A4 * pi_b * pi_HPC * pi_LPC * MFP*(Tt4,f) * sqrt(Tt2 /Tt4) / (1+f)
//
// The LP compressor raises pressure and mass flow PROPORTIONALLY, so the HP core sees the same
// CORRECTED flow whatever the LP spool delivers, and no modelled loss between 25 and 4 puts
// pi_LPC back. Tt25/Tt3 come from rung 38's ENERGY cascade (no compressor efficiency anywhere),
// so the HP compressor's whole map coordinate pair is a closed fixed point in pi_HPC alone. The
// LP face (‡) DOES carry pi_HPC.
//
// So the map opens EXACTLY ONE arrow, HP -> LP. Rung 38's VERDICT survives; rung 38's stated
// REASON for expecting it to fail is refuted — the rung-28 shape. The solve below is written
// triangular ON PURPOSE, with (†)/(‡) in exactly those closed forms, so the closed leaf is a
// bit-for-bit code-level guarantee rather than the ~1e-15 noise a jointly-iterated
// implementation would leave behind.

/// The virtual table § 5.3's census requires rung 39 to ship.
///
/// **Slice K shipped it with NOTHING dispatching through it; SLICE L MAKES `try_match_point`
/// THE PORT'S FIRST LIVE VIRTUAL DISPATCH.** Rung 42 overrides it, and rung 41's `surge_margin`
/// / `running_line_map` / `flow_coefficient_turn` call it on `self` — so a rung-42 object
/// running `bleed_trade` reaches `surge_margin` (rung 41's code) which reaches rung 42's own
/// override. That chain is the reason the table exists, and
/// `rung42.rs::gate_the_dispatch_is_live` is what witnesses it: no value key can.
/// `hp_eta_loop`/`lp_eta_loop` are still unexercised — they are overridden by rung 55 (phase 7).
pub struct TwoSpoolHooks {
    /// Match one point, FALLIBLY. Rung 42's `TwoSpoolBleedMatcher` overrides this.
    ///
    /// The table holds the `try_` half of the twin, not the panicking half, because the
    /// panicking half is a two-line wrapper that must NOT be overridable independently — a cell
    /// that overrode only one of the pair would give `surge_margin_schedule` and
    /// `surge_margin` different physics.
    pub try_match_point:
        fn(&TwoSpoolMapCore, &FlightCondition, f64) -> Result<TwoSpoolMapResult, Abort>,
    /// The CLOSED HP efficiency fixed point. Rung 55's `StageStackMatcher` overrides it.
    pub hp_eta_loop:
        fn(&TwoSpoolMapCore, &Gas, f64, f64, f64, f64, f64, &ComponentMap) -> EtaLoop,
    /// The LP efficiency fixed point — the one that reads `pi_hpc`.
    pub lp_eta_loop:
        fn(&TwoSpoolMapCore, &Gas, f64, f64, f64, f64, f64, f64, &ComponentMap) -> EtaLoop,
}

/// RUNG 39's table.
pub const R39: TwoSpoolHooks = TwoSpoolHooks {
    try_match_point: r39_try_match_point,
    hp_eta_loop: r39_hp_eta_loop,
    lp_eta_loop: r39_lp_eta_loop,
};

/// RUNG 39. Two-spool off-design matching WITH a [`ComponentMap`] on EACH spool.
///
/// ```text
/// let mm = TwoSpoolMapMatcher::new(design, flight, 1.0,
///                                  ComponentMap::flow_dominated(),
///                                  ComponentMap::pressure_dominated());
/// let od = mm.match_point(&flight, 1200.0).two_map();   // both etas AND both N are OUTPUTS
/// ```
pub enum TwoSpoolMapMatcher {
    /// `lp_disabled` forwards to rung 32's [`MapMatcher`], which itself reduces to rung 31 on a
    /// flat map — so ONE dispatch completes the whole four-rung ladder. The single remaining
    /// compressor plays the HPC role, so it carries `map_hp`.
    Degenerate(MapMatcher),
    Full(TwoSpoolMapCore),
}

impl TwoSpoolMapMatcher {
    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap,
    ) -> Self {
        TwoSpoolMapMatcher::Full(
            TwoSpoolMapCore::new(design_engine, flight_design, mdot_design, map_lp, map_hp))
    }

    /// `lp_disabled=True`. Takes a SINGLE-spool design engine and `map_hp`, which is exactly
    /// what the Python's early return builds.
    pub fn lp_disabled(
        design_engine: Engine, flight_design: FlightCondition, mdot_design: f64,
        map_hp: ComponentMap,
    ) -> Self {
        TwoSpoolMapMatcher::Degenerate(
            MapMatcher::new(design_engine, flight_design, mdot_design, map_hp))
    }

    pub fn match_point(&self, flight: &FlightCondition, tt4: f64) -> MatchedMap {
        match self {
            TwoSpoolMapMatcher::Degenerate(m) => MatchedMap::Single(m.match_point(flight, tt4)),
            TwoSpoolMapMatcher::Full(c) => MatchedMap::Two(c.match_point(flight, tt4)),
        }
    }

    pub fn core(&self) -> &TwoSpoolMapCore {
        match self {
            TwoSpoolMapMatcher::Full(c) => c,
            TwoSpoolMapMatcher::Degenerate(_) => panic!("this matcher is lp_disabled"),
        }
    }

    /// See [`TwoSpoolMatcher::core_mut`] — rung 39 gate 4 perturbs `eta_lpc`/`eta_hpc` and the
    /// two turbine efficiencies at a FIXED operating point.
    pub fn core_mut(&mut self) -> &mut TwoSpoolMapCore {
        match self {
            TwoSpoolMapMatcher::Full(c) => c,
            TwoSpoolMapMatcher::Degenerate(_) => panic!("this matcher is lp_disabled"),
        }
    }
}

/// Rung 39's two dispatch arms — see [`Matched`], same reasoning.
#[derive(Clone, Debug)]
pub enum MatchedMap {
    Single(MapOffDesignResult),
    Two(TwoSpoolMapResult),
}

impl MatchedMap {
    pub fn two(self) -> TwoSpoolMapResult {
        match self {
            MatchedMap::Two(r) => r,
            MatchedMap::Single(_) => panic!("this matcher is lp_disabled"),
        }
    }
}

/// Rung 39's object once `lp_disabled` is ruled out: rung 38's core plus the two maps and the
/// per-FACE design references the two sets of map coordinates are normalised on.
pub struct TwoSpoolMapCore {
    pub base: TwoSpoolCore,
    pub map_lp: ComponentMap,
    pub map_hp: ComponentMap,
    pub tt2_d: f64,
    pub tt25_d: f64,
    pub tt4_d: f64,
    pub tt45_d: f64,
    /// Design corrected flow at the LPC face (station 2).
    pub mcorr_lp_d: f64,
    /// Design corrected flow at the HPC face (station 25).
    pub mcorr_hp_d: f64,
    pub tau_lpc_d: f64,
    pub tau_hpc_d: f64,
    pub hooks: &'static TwoSpoolHooks,
}

impl TwoSpoolMapCore {
    /// Per-spool efficiency secant tolerance (rung 32's).
    pub const ETA_TOL: f64 = 1e-11;
    /// Secant step cap (positive-feedback edge guard, rung 32's).
    pub const ETA_MAX: usize = 80;
    /// Outer turbine-efficiency loop cap — INERT when `a_t == 0`.
    pub const TURB_MAX: usize = 60;

    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap,
    ) -> Self {
        Self::with_hooks(design_engine, flight_design, mdot_design, map_lp, map_hp, &R39)
    }

    pub fn with_hooks(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap, hooks: &'static TwoSpoolHooks,
    ) -> Self {
        let base = TwoSpoolCore::new(design_engine, flight_design, mdot_design);
        let (s2, s25, s3) = (*base.reference.station("2"), *base.reference.station("25"),
                             *base.reference.station("3"));
        let (s4, s45) = (*base.reference.station("4"), *base.reference.station("45"));
        TwoSpoolMapCore {
            tt2_d: s2.tt, tt25_d: s25.tt, tt4_d: s4.tt, tt45_d: s45.tt,
            mcorr_lp_d: mdot_design * powp(s2.tt, 0.5) / s2.pt,
            mcorr_hp_d: mdot_design * powp(s25.tt, 0.5) / s25.pt,
            tau_lpc_d: s25.tt / s2.tt,
            tau_hpc_d: s3.tt / s25.tt,
            base, map_lp, map_hp, hooks,
        }
    }

    pub fn gas(&self) -> &Gas { self.base.gas() }

    // --- THE DISPATCH POINT ----------------------------------------------------------------

    /// Match one point **through the virtual table** — rung 39's body, or rung 42's override.
    ///
    /// This pair is the port's first LIVE dispatch: rung 41's [`surge_margin`](Self::
    /// surge_margin), [`running_line_map`](Self::running_line_map) and
    /// [`flow_coefficient_turn`](Self::flow_coefficient_turn) all reach the concrete cell
    /// through here, which is exactly what Python's `self.match(...)` does inside those three
    /// methods.
    pub fn match_point(&self, flight: &FlightCondition, tt4: f64) -> TwoSpoolMapResult {
        self.try_match_point(flight, tt4).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE half — see [`Abort`]. Rung 41's three schedule methods call THIS one and
    /// skip the point on `Err`, which is what Python's `except AssertionError: continue` means.
    ///
    /// [`Abort`]: crate::gas::Abort
    pub fn try_match_point(
        &self, flight: &FlightCondition, tt4: f64,
    ) -> Result<TwoSpoolMapResult, Abort> {
        (self.hooks.try_match_point)(self, flight, tt4)
    }

    // --- the triangular map cascade at a FIXED (Tt2, pt2, Tt4, f) --------------------------

    /// Rung 38's steps 1–4 with both maps live, TRIANGULAR by construction.
    ///
    /// Order: geometry (★-HP, ★-LP) → ENERGY (`Tt25`, `Tt3`; map-free) → HP eta loop (closed) →
    /// LP eta loop (reads `pi_HPC`), wrapped in an OUTER turbine-efficiency loop that is INERT
    /// when both `a_t == 0` — `eta_t_at` then returns its base, so the loop converges on its
    /// first pass and the closed leaf stays exact. Measured: 1 pass at `a_t = 0`, 3 to 4 at
    /// `a_t = 0.02`.
    ///
    /// Exposed as its own method so the finding is testable at a fixed `(Tt2, pt2, Tt4, f)` —
    /// rung 38 gate 3's isolation protocol, so the outer `f` loop cannot confound it.
    ///
    /// `pt2` is UNUSED, in the Python too: the whole map cascade works in corrected coordinates
    /// normalised on the design faces, and the only absolute pressure it would need — `pt4` —
    /// cancels out of (†) and (‡). It is kept in the signature because rung 38 gate 3's protocol
    /// hands a caller the converged `(Tt2, pt2, Tt4, f)` four-tuple, and dropping one element
    /// here would make the two isolation entry points differ for no reason.
    pub fn cascade_map(&self, wgas: &Gas, tt2: f64, pt2: f64, tt4: f64, f: f64) -> CascadeMap {
        self.try_cascade_map(wgas, tt2, pt2, tt4, f).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE twin of [`cascade_map`](Self::cascade_map) — see [`Abort`]. Both efficiency
    /// loops and the outer turbine loop keep their panics: measured **0** raises.
    ///
    /// [`Abort`]: crate::gas::Abort
    pub fn try_cascade_map(
        &self, wgas: &Gas, tt2: f64, _pt2: f64, tt4: f64, f: f64,
    ) -> Result<CascadeMap, Abort> {
        counters::bump_cascade();
        let mfp4 = choked_mfp(wgas, tt4, f);
        let (mut eta_hpt, mut eta_lpt) = (self.base.eta_hpt, self.base.eta_lpt);
        for turb_pass in 0..Self::TURB_MAX {
            // Steps 1–2: both turbines pinned by geometry, at the current turbine efficiencies.
            let (pi_hpt, tau_hpt, tt45) = self.base.try_solve_choked_turbine(
                wgas, tt4, f, self.base.a4, self.base.a45, 1.0, eta_hpt)?;
            let (pi_lpt, tau_lpt, tt5) = self.base.try_solve_choked_turbine(
                wgas, tt45, f, self.base.a45, self.base.a8, self.base.pi_n, eta_lpt)?;

            // ENERGY (map-free): the LP balance fixes Tt25, the HP balance fixes Tt3 onto it.
            let dh_lpt = self.base.eta_m * (1.0 + f) * (wgas.h_t(tt45, f) - wgas.h_t(tt5, f));
            let tt25 = wgas.t_from_h_c(wgas.h_c(tt2) + dh_lpt);
            let dh_hpt = self.base.eta_m * (1.0 + f) * (wgas.h_t(tt4, f) - wgas.h_t(tt45, f));
            let tt3 = wgas.t_from_h_c(wgas.h_c(tt25) + dh_hpt);

            // THE TRIANGLE: HP closes on itself, THEN LP closes onto pi_HPC.
            let hp = (self.hooks.hp_eta_loop)(self, wgas, tt4, f, tt25, tt3, mfp4, &self.map_hp);
            let lp = (self.hooks.lp_eta_loop)(
                self, wgas, tt2, tt4, f, tt25, mfp4, hp.pi, &self.map_lp);

            // Two physical shaft speeds — the structural novelty (rung 38 computes none).
            let nl = lp.n * powp(tt2 / self.tt2_d, 0.5);
            let nh = hp.n * powp(tt25 / self.tt25_d, 0.5);
            let nu_hpt = nh * powp(self.tt4_d / tt4, 0.5);
            let nu_lpt = nl * powp(self.tt45_d / tt45, 0.5);

            let out = CascadeMap {
                c: Cascade { pi_hpt, tau_hpt, tt45, pi_lpt, tau_lpt, tt5,
                             pi_lpc: lp.pi, tt25, pi_hpc: hp.pi, tt3 },
                eta_lpc: lp.eta, eta_hpc: hp.eta, eta_hpt, eta_lpt,
                m_l: lp.m, m_h: hp.m, n_l: lp.n, n_h: hp.n, nl, nh,
                phi_l: lp.m / lp.n, phi_h: hp.m / hp.n, nu_hpt, nu_lpt, slip: nl / nh,
            };

            // OUTER turbine-efficiency loop. With a_t == 0 these targets ARE the current
            // values, so this returns on the FIRST pass and the leaf above stays exact.
            let t_hpt = self.map_hp.eta_t_at(self.base.eta_hpt, nu_hpt);
            let t_lpt = self.map_lp.eta_t_at(self.base.eta_lpt, nu_lpt);
            if (t_hpt - eta_hpt).abs() <= Self::ETA_TOL
                && (t_lpt - eta_lpt).abs() <= Self::ETA_TOL {
                counters::note_turb(turb_pass as u64 + 1);
                return Ok(out);
            }
            eta_hpt = t_hpt;
            eta_lpt = t_lpt;
        }
        panic!("rung-39 turbine-efficiency loop did not converge at Tt4={tt4}; moderate a_t");
    }
}

/// One rung-32 secant step on the fixed-point residual `R(eta) = eta_map(eta) - eta`.
///
/// **The `[0.3, 1.0]` clamp is DEAD on every cell measured** — 0 bindings across all 72 matched
/// cells of a 144-cell shaped grid (§ 5.7 (g)). Ported as written, because it guards a path this
/// grid does not reach, and recorded as dead so a reader does not infer it is load-bearing.
///
/// **The one place the two languages genuinely differ, and it is unreachable:** Python's
/// `min(max(x, 0.3), 1.0)` propagates a NaN `x`, while Rust's `f64::max`/`min` return the
/// non-NaN operand and would hand back `0.3`. Written down rather than papered over — the clamp
/// never binds at all on any measured cell, and a NaN reaching here would already have poisoned
/// the residual that produced it.
pub fn secant(eta: f64, eta_prev: Option<f64>, r: f64, r_prev: f64, target: f64) -> f64 {
    let nxt = match eta_prev {
        // `abs(R - R_prev) < 1e-300`: a flat residual, so the secant slope is unusable and
        // Python falls back to plain substitution — the same branch the first step takes.
        Some(ep) if (r - r_prev).abs() >= 1e-300 => eta - r * (eta - ep) / (r - r_prev),
        _ => target,
    };
    if nxt < 0.3 || nxt > 1.0 {
        counters::bump_clamp();
    }
    nxt.max(0.3).min(1.0)
}

/// **THE CLOSED LEAF, AS A FREE FUNCTION — this signature IS rung 39's finding A.**
///
/// Solve `(eta_HPC, pi_HPC)` self-consistently on the HP map. It reads NO LP quantity **except
/// `Tt25`**, which the LP energy balance produced and which is map-free by rung 38's cascade;
/// no LP EFFICIENCY and no LP PRESSURE RATIO is in scope at all, because none is a parameter.
/// That narrow statement is what rung 39's gate 4 actually tests, and the first draft of
/// § 5.7's P1 over-claimed it as "no LP quantity" — false on the signature itself.
///
/// Check-first: the residual is tested BEFORE the secant is called, so on a flat map this
/// returns having done no secant arithmetic at all. That is what makes rung 39's flat-map
/// reduce to rung 38 bit-for-bit rather than merely close.
#[allow(clippy::too_many_arguments)]
pub fn hp_eta_loop_closed(
    wgas: &Gas, tt4: f64, f: f64, tt25: f64, tt3: f64, mfp4: f64, cmap: &ComponentMap,
    eta_hpc_base: f64, a4: f64, pi_b: f64, mcorr_hp_d: f64, tau_hpc_d: f64,
) -> EtaLoop {
    let (h25, h3, pr25) = (wgas.h_c(tt25), wgas.h_c(tt3), wgas.pr_c(tt25));
    let tau_hpc = tt3 / tt25;
    let (mut eta, mut eta_prev, mut r_prev) = (eta_hpc_base, None, f64::NAN);
    for pass in 0..TwoSpoolMapCore::ETA_MAX {
        let pi = wgas.pr_c(wgas.t_from_h_c(h25 + eta * (h3 - h25))) / pr25;
        // (†): pi_LPC-FREE by construction.
        let m = (a4 * pi_b * pi * mfp4 * powp(tt25 / tt4, 0.5) / (1.0 + f)) / mcorr_hp_d;
        let n = cmap.solve_n(m, tau_hpc, tau_hpc_d);
        let tgt = cmap.eta_c_at(eta_hpc_base, m / n, n);
        let r = tgt - eta;
        if r.abs() <= TwoSpoolMapCore::ETA_TOL {
            counters::note_hp(pass as u64);
            return EtaLoop { eta, pi, m, n };
        }
        let nxt = secant(eta, eta_prev, r, r_prev, tgt);
        eta_prev = Some(eta);
        r_prev = r;
        eta = nxt;
    }
    panic!("rung-39 HP efficiency secant did not converge at Tt4={tt4}; moderate the HP map \
            coefficients or the throttle.");
}

/// The LP efficiency fixed point — **it reads `pi_hpc`, and that is THE ONE ARROW the map
/// opens.**
///
/// Deliberately not folded together with [`hp_eta_loop_closed`]: the two bodies differ by one
/// factor in `m`, and that one factor IS the rung. A shared function with a
/// `pi_hpc: Option<f64>` parameter would turn a finding into a flag.
#[allow(clippy::too_many_arguments)]
pub fn lp_eta_loop_arrow(
    wgas: &Gas, tt2: f64, tt4: f64, f: f64, tt25: f64, mfp4: f64, pi_hpc: f64,
    cmap: &ComponentMap, eta_lpc_base: f64, a4: f64, pi_b: f64, mcorr_lp_d: f64,
    tau_lpc_d: f64,
) -> EtaLoop {
    let (h2, h25, pr2) = (wgas.h_c(tt2), wgas.h_c(tt25), wgas.pr_c(tt2));
    let tau_lpc = tt25 / tt2;
    let (mut eta, mut eta_prev, mut r_prev) = (eta_lpc_base, None, f64::NAN);
    for pass in 0..TwoSpoolMapCore::ETA_MAX {
        let pi = wgas.pr_c(wgas.t_from_h_c(h2 + eta * (h25 - h2))) / pr2;
        // (‡): carries pi_hpc — the ONE arrow.
        let m = (a4 * pi_b * pi_hpc * pi * mfp4 * powp(tt2 / tt4, 0.5) / (1.0 + f)) / mcorr_lp_d;
        let n = cmap.solve_n(m, tau_lpc, tau_lpc_d);
        let tgt = cmap.eta_c_at(eta_lpc_base, m / n, n);
        let r = tgt - eta;
        if r.abs() <= TwoSpoolMapCore::ETA_TOL {
            counters::note_lp(pass as u64);
            return EtaLoop { eta, pi, m, n };
        }
        let nxt = secant(eta, eta_prev, r, r_prev, tgt);
        eta_prev = Some(eta);
        r_prev = r;
        eta = nxt;
    }
    panic!("rung-39 LP efficiency secant did not converge at Tt4={tt4}; moderate the LP map \
            coefficients or the throttle.");
}

// --- the hook bodies: state off the core, arithmetic in the free functions -----------------

#[allow(clippy::too_many_arguments)]
fn r39_hp_eta_loop(
    core: &TwoSpoolMapCore, wgas: &Gas, tt4: f64, f: f64, tt25: f64, tt3: f64, mfp4: f64,
    cmap: &ComponentMap,
) -> EtaLoop {
    hp_eta_loop_closed(wgas, tt4, f, tt25, tt3, mfp4, cmap,
                       core.base.eta_hpc, core.base.a4, core.base.pi_b,
                       core.mcorr_hp_d, core.tau_hpc_d)
}

#[allow(clippy::too_many_arguments)]
fn r39_lp_eta_loop(
    core: &TwoSpoolMapCore, wgas: &Gas, tt2: f64, tt4: f64, f: f64, tt25: f64, mfp4: f64,
    pi_hpc: f64, cmap: &ComponentMap,
) -> EtaLoop {
    lp_eta_loop_arrow(wgas, tt2, tt4, f, tt25, mfp4, pi_hpc, cmap,
                      core.base.eta_lpc, core.base.a4, core.base.pi_b,
                      core.mcorr_lp_d, core.tau_lpc_d)
}

/// RUNG 39's `match`. The outer `(f, pt4)` fixed point is rung 38's, unchanged — the one place
/// the two spools share state. `pi_lpc`, `pi_hpc`, all four efficiencies AND both shaft speeds
/// are OUTPUTS. Scope (inherited, re-asserted): the nozzle must stay choked.
fn r39_try_match_point(
    core: &TwoSpoolMapCore, flight: &FlightCondition, tt4: f64,
) -> Result<TwoSpoolMapResult, Abort> {
    let b = &core.base;
    let pi_d = b.pi_d_max * ram_recovery(flight.m0);
    let (state0, _v0) = b.try_freestream_for(flight)?;
    let (tt2, pt2) = (state0.tt, pi_d * state0.pt);

    let (mut f, mut pt4) = (b.f_design, b.pi_b * b.pi_hpc_design * b.pi_lpc_design * pt2);
    let mut c: Option<CascadeMap> = None;
    for _ in 0..TwoSpoolCore::MAX {
        let owned = b.try_working_gas(f, tt4, pt4)?;
        let wgas = owned.as_ref().unwrap_or(b.gas());
        let cm = core.try_cascade_map(wgas, tt2, pt2, tt4, f)?;
        let pt4_new = b.pi_b * cm.c.pi_hpc * cm.c.pi_lpc * pt2;
        let f_new = b.try_solve_f(cm.c.tt3, pt4_new, tt4)?;
        let done = (f_new - f).abs() <= TwoSpoolCore::TOL * (f_new + 1e-30)
            && (pt4_new - pt4).abs() <= TwoSpoolCore::TOL * pt4_new;
        c = Some(cm);
        f = f_new;
        pt4 = pt4_new;
        if done {
            break;
        }
    }
    let c = c.expect("the joint loop runs at least once");

    assert!(c.c.pi_lpc > 1.0 && c.c.pi_hpc > 1.0 && 0.0 < c.c.tau_hpt && c.c.tau_hpt < 1.0
                && 0.0 < c.c.tau_lpt && c.c.tau_lpt < 1.0,
            "rung-39 two-spool map match unphysical");

    let owned = b.try_working_gas(f, tt4, pt4)?;
    let wgas = owned.as_ref().unwrap_or(b.gas());
    let mdot_air = b.a4 * pt4 * choked_mfp(wgas, tt4, f) / powp(tt4, 0.5) / (1.0 + f);

    // Rebuild FORWARD at the MAP-CONSISTENT efficiencies.
    let r = b.try_rebuild(flight, pi_d, c.c.pi_lpc, c.c.pi_hpc, tt4, mdot_air,
                          c.eta_lpc, c.eta_hpc, c.eta_hpt, c.eta_lpt)?;

    // SCOPE GUARD (inherited from rung 38 — unchoke is still a rung-33-shaped follow-on).
    // An `Abort` rather than a panic FROM SLICE L ON: rung 41's schedule methods SKIP such a
    // point, and this is much the commonest reason they do — 23 of the 147-cell dump grid at
    // `b = 0`, rising to 25 at `b = 0.10`, which IS rung 42 gate 6's shrinking envelope, as a
    // count.
    if r.exit_p9 <= b.p_ambient + 1e-6 {
        return Err(Abort(format!(
            "rung-39 two-spool map match at Tt4={tt4:.0}, M0={:.2}: nozzle UNCHOKED -- OUT \
             OF SCOPE (docs/rung38-spec.md 'Scope'). The LP turbine's geometric tau_LPT pin \
             (*-LP) is only valid while the nozzle stays choked.", flight.m0)));
    }

    let base = r.try_into_result(b, flight, tt4, mdot_air, &c.c)?;
    Ok(TwoSpoolMapResult {
        base,
        eta_lpc: c.eta_lpc, eta_hpc: c.eta_hpc, eta_hpt: c.eta_hpt, eta_lpt: c.eta_lpt,
        n_lp: c.n_l, n_hp: c.n_h, n_lp_ratio: c.nl, n_hp_ratio: c.nh, slip: c.slip,
        phi_lp: c.phi_l, phi_hp: c.phi_h, nu_hpt: c.nu_hpt, nu_lpt: c.nu_lpt,
    })
}
