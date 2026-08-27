//! RUNGS 57–60 — the VARIABLE STATOR on the FUEL-metered two-shaft transient, and the three
//! rungs that fall out of running a wall-moving lever beside a point-moving one.
//!
//! Rung 57 (`engine.py:7385–8451`) subclasses rung 43/45's [`TwoSpoolFuelTransient`] and arms rung
//! 53's stator from the LIVE state at every closure. Rung 58 puts ONE fuel-side min-select leg
//! beside it and takes a mixed second difference; rung 59 re-derives that leg on the plant it runs
//! on; rung 60 re-references the floor to INCIDENCE. All four are ONE Python class — 35 methods,
//! 923 body lines — which is why they are one module.
//!
//! # The one structural fact, and no census saw it
//!
//! `_arm` assigns `self.map_lp` / `self.map_hp` and **never restores them**. § 5.19 (iv)'s census
//! classified the phase's dynamic scoping by exactly one shape — save, set, call, restore in a
//! `finally` — so a bare permanent assignment was invisible to it. The mutation is observable:
//! § 5.20 (ii) measured a port that SCOPES it (build the armed maps inside `try_close` and leave
//! the caller's core untouched — the natural Rust shape) moving `margin_min_lp`, rung 57's own
//! currency, by **15.4 %**, with all **59** ported gates green. That is the third time this port
//! has found a suite blind to a large error, and the first where the blind spot is in the OBJECT'S
//! OWN STATE rather than in a reader's coordinate.
//!
//! Hence [`crate::two_spool::TwoSpoolMapCore`]'s two map fields are `Cell<ComponentMap>` (slice V
//! step 1b), and hence [`r57_arm`] writes through [`set_map_lp`] from inside a `&self` hook.
//!
//! # Why the march is bit-identical anyway, and why that is a licence rather than luck
//!
//! Probe 6 measured the stale map being read **723 times per march OUTSIDE the close extent** —
//! 687 by `_instant_tail`, 36 by `_powers` — and the trajectory does not move. The reason is
//! algebraic, not a dead path: every one of those reads is [`ComponentMap::eta_t_at`],
//! [`ComponentMap::with_vsv`] sets only `vsv`, and `eta_t_at` reads only `a_t`. **`vsv` cannot
//! reach `a_t`.** So the two shipped, gated cells that read the map inside the march are invariant
//! to the arming BY CONSTRUCTION — which is what makes rungs 57–60 portable without touching
//! them, and which step 5 owes an injection for (arm during a march, assert `try_instant_tail`'s
//! output is bit-identical). Staleness bites only through the channels the mutation drives:
//! `psi` and `phi_surge_at`, exactly the pair `with_vsv`'s own docstring names.
//!
//! # What slice V adds to the tables
//!
//! Three NEW cells ([`StatorTransientHooks`] — `arm`, `v_of`, `stator_march`), two phase-6 cells
//! OPENED at step 1a ([`FuelTransientHooks`]), and one SWAP into a table that already ships
//! ([`TwoSpoolTransientHooks::try_close`]). `at_stator` gets **no cell**: it is a pure sibling
//! constructor, § 5.19 (iii)'s delete.
//!
//! [`set_map_lp`]: crate::two_spool::TwoSpoolMapCore::set_map_lp
//! [`TwoSpoolFuelTransient`]: crate::fuel_transient::TwoSpoolFuelTransient
//! [`FuelTransientHooks`]: crate::fuel_transient::FuelTransientHooks
//! [`TwoSpoolTransientHooks::try_close`]: crate::two_spool_transient::TwoSpoolTransientHooks

use std::cell::Cell;

use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelLimiters, FuelPoint, FuelTransientCore,
    FuelTransientHooks, SurgeLimiter,
};
use crate::gas::{powp, Abort};
use crate::map::ComponentMap;
use crate::spool::SpoolTransient;
use crate::two_spool::{Spool, TwoSpoolEngine};
use crate::two_spool_transient::{
    CloseState, TwoSpoolTransientCore, TwoSpoolTransientHooks, R40,
};

// ---------------------------------------------------------------------------------------------
// Counters — the two arms of `_arm` are the ones P3's reduce gate is written on
// ---------------------------------------------------------------------------------------------

thread_local! {
    static ARM_CALLS: Cell<u64> = const { Cell::new(0) };
    /// `_arm` returned on its FIRST LINE — nothing scheduled. **THE REDUCE, by dispatch.**
    static ARM_UNARMED: Cell<u64> = const { Cell::new(0) };
    /// The LP schedule commanded EXACTLY `0.0`, so the DESIGN map is handed back untouched.
    static ARM_LP_ZERO: Cell<u64> = const { Cell::new(0) };
    /// The LP schedule commanded a nonzero setting: `with_vsv` builds a moved map.
    static ARM_LP_MOVED: Cell<u64> = const { Cell::new(0) };
    static ARM_HP_ZERO: Cell<u64> = const { Cell::new(0) };
    static ARM_HP_MOVED: Cell<u64> = const { Cell::new(0) };
    static VOF_CALLS: Cell<u64> = const { Cell::new(0) };
    static MARCH_CALLS: Cell<u64> = const { Cell::new(0) };
    /// `_read` was handed a CALLER'S `v_of` rather than defaulting to this machine's own. No
    /// shipped caller in rungs 57-63 passes one — a grep of `_read(`'s call sites, NOT a run —
    /// so the ported parameter is COUNTED and `slice_v_smoke.rs` section K gates it at zero.
    static READ_FOREIGN_VOF: Cell<u64> = const { Cell::new(0) };
    /// `_refine_min`'s minimum landed on an ENDPOINT, so no parabola is fitted.
    static REFINE_EDGE: Cell<u64> = const { Cell::new(0) };
    /// `_refine_min`'s three-point vertex had a ZERO second difference.
    static REFINE_FLAT: Cell<u64> = const { Cell::new(0) };
    static RESOLVE_PHI: Cell<u64> = const { Cell::new(0) };
    static RESOLVE_INCIDENCE: Cell<u64> = const { Cell::new(0) };
}

fn bump(c: &'static std::thread::LocalKey<Cell<u64>>) {
    c.with(|x| x.set(x.get() + 1));
}

/// This module's counters — same single-consumer caveat as every other census in the crate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Census {
    pub arm_calls: u64,
    pub arm_unarmed: u64,
    pub arm_lp_zero: u64,
    pub arm_lp_moved: u64,
    pub arm_hp_zero: u64,
    pub arm_hp_moved: u64,
    pub v_of_calls: u64,
    pub march_calls: u64,
    pub read_foreign_v_of: u64,
    pub refine_edge: u64,
    pub refine_flat: u64,
    pub resolve_phi: u64,
    pub resolve_incidence: u64,
}

pub mod counters {
    use super::*;

    pub fn take() -> Census {
        let c = Census {
            arm_calls: ARM_CALLS.with(|x| x.get()),
            arm_unarmed: ARM_UNARMED.with(|x| x.get()),
            arm_lp_zero: ARM_LP_ZERO.with(|x| x.get()),
            arm_lp_moved: ARM_LP_MOVED.with(|x| x.get()),
            arm_hp_zero: ARM_HP_ZERO.with(|x| x.get()),
            arm_hp_moved: ARM_HP_MOVED.with(|x| x.get()),
            v_of_calls: VOF_CALLS.with(|x| x.get()),
            march_calls: MARCH_CALLS.with(|x| x.get()),
            read_foreign_v_of: READ_FOREIGN_VOF.with(|x| x.get()),
            refine_edge: REFINE_EDGE.with(|x| x.get()),
            refine_flat: REFINE_FLAT.with(|x| x.get()),
            resolve_phi: RESOLVE_PHI.with(|x| x.get()),
            resolve_incidence: RESOLVE_INCIDENCE.with(|x| x.get()),
        };
        reset();
        c
    }

    pub fn reset() {
        for k in [&ARM_CALLS, &ARM_UNARMED, &ARM_LP_ZERO, &ARM_LP_MOVED, &ARM_HP_ZERO,
                  &ARM_HP_MOVED, &VOF_CALLS, &MARCH_CALLS, &READ_FOREIGN_VOF, &REFINE_EDGE,
                  &REFINE_FLAT, &RESOLVE_PHI, &RESOLVE_INCIDENCE] {
            k.with(|x| x.set(0));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// RUNG 57's schedule
// ---------------------------------------------------------------------------------------------

/// [`StatorSchedule`]'s shape function `S(x)`.
///
/// A Rust enum where Python has a validated string, so the `__post_init__` membership assert
/// becomes unrepresentable rather than checked — **and the assert is ported anyway**, on
/// [`StatorSchedule::try_from_str`], because rung 57's suite pokes it with a bad string and the
/// refusal is the observable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// `S(x) = x^2(3-2x)` — C1 at BOTH corners. THE DEFAULT, and it matters: the schedule's kink
    /// lives in STATE space, so rung 50's put-the-switch-on-the-grid trick is unavailable and a
    /// C0 corner would cost the RK4 march its order there.
    Smooth,
    /// `S(x) = x` — the C0 alternative, carried ONLY as a shape-robustness control.
    Linear,
}

/// RUNG 57. A variable-stator schedule `v(n)` in the CORRECTED SPEED of its own spool.
///
/// ```text
/// v(n) = v_max * S( (n_ref - n) / (n_ref - n_lo) )        S clipped to [0, 1]
/// ```
///
/// CLOSED at low corrected speed, monotonically opening, and EXACTLY 0 at and above the design
/// speed `n_ref` — which is not cosmetic: the whole hardware capture (`A4`/`A45`/`A8`,
/// `mcorr_*_d`, `tau_*_d`) is taken at `v = 0` (rung 53's discipline), so a schedule holding a
/// nonzero setting at the design speed would silently contradict every design reference.
/// [`with_shape`](Self::with_shape) ASSERTS it rather than relying on the algebra.
///
/// Like `vsv` itself (rung 53), `s_off` (rung 50) and `bleed` (rung 42), this is a swept geometry
/// coordinate, not a fitted constant: it adds no physics beyond rung 53's three derived channels,
/// it only says WHERE on the map they are applied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatorSchedule {
    pub v_max: f64,
    pub n_lo: f64,
    pub n_ref: f64,
    pub shape: Shape,
}

impl StatorSchedule {
    /// Python's default `n_ref`.
    pub const N_REF: f64 = 1.0;

    /// `n_ref = 1.0`, `shape = "smooth"` — Python's two defaults, which is how every shipped
    /// caller builds one.
    pub fn new(v_max: f64, n_lo: f64) -> Self {
        Self::with_shape(v_max, n_lo, Self::N_REF, Shape::Smooth)
    }

    /// The full constructor, carrying `__post_init__`'s two surviving asserts.
    pub fn with_shape(v_max: f64, n_lo: f64, n_ref: f64, shape: Shape) -> Self {
        assert!(n_lo < n_ref,
                "rung-57 StatorSchedule needs n_lo < n_ref: got {n_lo} >= {n_ref}");
        let s = StatorSchedule { v_max, n_lo, n_ref, shape };
        assert!(s.at(n_ref) == 0.0,
                "rung-57 StatorSchedule must be EXACTLY 0 at the design corrected speed n_ref -- \
                 the hardware and both maps' design references are captured at v = 0.");
        s
    }

    /// Python's `shape` membership assert, which a Rust enum otherwise deletes. Rung 57's suite
    /// pokes it with a bad string, so the refusal is a gated observable and not a formality.
    pub fn try_from_str(v_max: f64, n_lo: f64, n_ref: f64, shape: &str) -> Self {
        let sh = match shape {
            "smooth" => Shape::Smooth,
            "linear" => Shape::Linear,
            other => panic!(
                "rung-57 StatorSchedule shape must be 'smooth' (C1, default) or 'linear' \
                 (C0 control), got {other:?}"),
        };
        Self::with_shape(v_max, n_lo, n_ref, sh)
    }

    /// `v(n)` — Python's `__call__`.
    pub fn at(&self, n: f64) -> f64 {
        let x = (self.n_ref - n) / (self.n_ref - self.n_lo);
        // Python's `0.0 if x < 0.0 else (1.0 if x > 1.0 else x)` — a two-arm conditional, NOT
        // `clamp`, which differs on NaN. Spelled the same way.
        let x = if x < 0.0 { 0.0 } else if x > 1.0 { 1.0 } else { x };
        self.v_max
            * match self.shape {
                Shape::Smooth => x * x * (3.0 - 2.0 * x),
                Shape::Linear => x,
            }
    }
}

// ---------------------------------------------------------------------------------------------
// RUNG 60's floor
// ---------------------------------------------------------------------------------------------

/// RUNG 60. Rung 49's floor RE-REFERENCED to INCIDENCE — the *matched phi floor* rung 58 asked
/// for, and the only canonical way to build one.
///
/// ```text
/// M_i  =  T_c - (1/phi - v)  >=  m_lim          [the wall is the METAL]
/// ```
///
/// Rung 58 found a `phi` floor NOT COMPOSABLE with a variable stator at a fixed set point: the
/// admissible bands on the bare and statored machines are DISJOINT, because rung 53's lever moves
/// the `phi` wall by more than the ramp's own `phi` excursion. Its proposed repair — match the set
/// point per machine — is under-determined: matching at fixed `phi`-margin off the moved wall and
/// matching at fixed incidence give DIFFERENT floors, apart by exactly `v*sm/(1+sm)`
/// ([`matching_rules`](ScheduledStatorCore::matching_rules)). There is no second candidate for the
/// canonical rule, because `M_i` is the ONE currency whose wall the stator does not move.
///
/// HOW IT RUNS. There is no new solve. At the live setting `v` the floor IS the `phi` floor
/// `1/(T_c + v - m_lim)`, so [`at`](Self::at) hands back a plain rung-49 [`SurgeLimiter`] and rung
/// 49's set-point solve runs unchanged. That conversion is legal — rather than circular — only
/// because `v` is a function of the SHAFT STATE and not of the fuel, so within a derivative call
/// the floor is a constant and rung 49's monotonicity bracket carries verbatim.
///
/// `m_lim` is the SAME disclaimed rung-36 constant read as an incidence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IncidenceLimiter {
    /// WHICH spool's incidence is floored.
    pub spool: Spool,
    /// The floor, in the incidence-margin currency `M_i`.
    pub m_lim: f64,
}

impl IncidenceLimiter {
    pub fn new(spool: Spool, m_lim: f64) -> Self {
        IncidenceLimiter { spool, m_lim }
    }

    /// The incidence set point a given `phi` floor IS, at stator setting `vsv`. At the design
    /// setting (`vsv = 0`) this is the rung-49 floor read in rung 53's coordinate — the same
    /// instrument, renamed, which is what makes the two comparable.
    pub fn from_phi(cmap: &ComponentMap, spool: Spool, phi_lim: f64, vsv: f64) -> Self {
        Self::new(spool, cmap.tan_beta1_crit() - (1.0 / phi_lim - vsv))
    }

    /// The incidence set point of a floor at surge margin `sm` above the map's own imposed
    /// (design-setting) surge line — rung 49's `from_margin`, re-referenced.
    pub fn from_margin(cmap: &ComponentMap, spool: Spool, sm: f64) -> Self {
        Self::from_phi(cmap, spool, (1.0 + sm) * cmap.phi_surge, 0.0)
    }

    /// The `phi` floor this incidence floor IS at setting `v` — **fallibly.**
    ///
    /// **AND `Abort` RATHER THAN A PANIC IS THE WHOLE POINT OF THE `try_` SPELLING HERE.** Trace
    /// the call: `der` (inside `integrate_fuel`) → `try_surge_fuel` → rung 57's cell →
    /// [`resolve_floor`](FuelTransientCore::resolve_floor) → here. Python's marcher SWALLOWS the
    /// `AssertionError` at that depth and truncates the trajectory; a Rust panic would abort a
    /// march Python completes. That is § 5.16 probe 4 (A) — the reason
    /// [`FuelTransientCore::try_tt4_from_f`] exists — landing on a site slice V introduces.
    ///
    /// Closing the stators (`v > 0`) LOWERS the `phi` floor, by exactly the amount rung 53 lowers
    /// the wall: the DISTANCE to the metal is held, not the flow coefficient.
    pub fn try_phi_lim_at(&self, t_c: f64, v: f64) -> Result<f64, Abort> {
        let d = t_c + v - self.m_lim;
        if d <= 0.0 {
            let m = self.m_lim;
            return Err(Abort(format!(
                "rung-60 incidence floor m_lim={m:.6} is at or above the critical incidence \
                 T_c={t_c:.6} at v={v:.4}: no phi realises it.")));
        }
        Ok(1.0 / d)
    }

    /// [`try_phi_lim_at`](Self::try_phi_lim_at) for a caller that cannot recover — the spelling
    /// rung 60's own refusal gate pokes directly, where the raise is the observable.
    /// [`FuelTransientCore::tt4_from_f`]'s precedent.
    pub fn phi_lim_at(&self, t_c: f64, v: f64) -> f64 {
        self.try_phi_lim_at(t_c, v).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The equivalent rung-49 leg at setting `v`. **THE REDUCE**: at `v = 0.0` this is
    /// `SurgeLimiter(spool, 1/(T_c - m_lim))`, float-identical to the hand-built rung-49 floor
    /// (`x + 0.0 == x` exactly), so the whole rung-49/58/59 path stays BIT-FOR-BIT.
    pub fn try_at(&self, t_c: f64, v: f64) -> Result<SurgeLimiter, Abort> {
        Ok(SurgeLimiter::new(self.spool, self.try_phi_lim_at(t_c, v)?))
    }

    pub fn at(&self, t_c: f64, v: f64) -> SurgeLimiter {
        self.try_at(t_c, v).unwrap_or_else(|e| panic!("{}", e.0))
    }
}

// ---------------------------------------------------------------------------------------------
// The arming state — carried on rung 40's core, for the reason that field's doc gives
// ---------------------------------------------------------------------------------------------

/// RUNG 57's per-spool arming, and the DESIGN maps every sibling and every arming is rebuilt from.
///
/// Two ways to arm, MUTUALLY EXCLUSIVE per spool:
///
/// * `vsv_lp` / `vsv_hp` — a CONSTANT setting, rung 53's lever transplanted. Applied ONCE at
///   construction, so `equilibrium` and `fuel_for_tt4` see it and the march starts on the
///   STATORED running line.
/// * `sched_lp` / `sched_hp` — a [`StatorSchedule`] read off the LIVE state at every closure, the
///   thing a real engine implements.
///
/// **The constant arm is EXACTLY zero in § 5.20 (ii)'s table** because it is applied in the
/// constructor and only a SCHEDULE ever reaches [`r57_arm`]. So the reduce contract and the
/// staleness sit on DISJOINT arms, which is the second reason no reduce gate can witness the
/// carrier.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatorArming {
    pub vsv_lp: f64,
    pub vsv_hp: f64,
    pub sched_lp: Option<StatorSchedule>,
    pub sched_hp: Option<StatorSchedule>,
    /// The maps AS PASSED IN — Python's `self.map_lp_design`. Every arming is `with_vsv` off
    /// THESE, never off the live (possibly already-armed) map, which is what makes `_arm` a pure
    /// function of the state rather than a latch.
    pub map_lp_design: ComponentMap,
    pub map_hp_design: ComponentMap,
}

impl StatorArming {
    /// What every rung-40/43 object carries: no lever, and the design maps ARE the live maps.
    pub fn unarmed(map_lp: ComponentMap, map_hp: ComponentMap) -> Self {
        StatorArming {
            vsv_lp: 0.0,
            vsv_hp: 0.0,
            sched_lp: None,
            sched_hp: None,
            map_lp_design: map_lp,
            map_hp_design: map_hp,
        }
    }

    /// Python's `_is_armed` — a SCHEDULE on either spool. A constant setting is deliberately NOT
    /// in it: it has already been applied to the maps by the constructor.
    pub fn is_scheduled(&self) -> bool {
        self.sched_lp.is_some() || self.sched_hp.is_some()
    }

    /// Python's `self._is_armed() or self.vsv_lp or self.vsv_hp` — the guard every rung-58/59/60
    /// reader opens with.
    pub fn is_armed(&self) -> bool {
        self.is_scheduled() || self.vsv_lp != 0.0 || self.vsv_hp != 0.0
    }

    pub fn design_map(&self, spool: Spool) -> ComponentMap {
        match spool {
            Spool::Lp => self.map_lp_design,
            Spool::Hp => self.map_hp_design,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The virtual table
// ---------------------------------------------------------------------------------------------

/// The ramp geometry every rung-57/58/59/60 reader marches on — Python's
/// `(Tt4_lo, Tt4_hi, r, s_settle, ds)` argument run, bundled so the cell signature does not carry
/// nine floats.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ramp {
    pub tt4_lo: f64,
    pub tt4_hi: f64,
    pub r: f64,
    pub s_settle: f64,
    pub ds: f64,
}

impl Ramp {
    /// Rung 57's defaults: `s_settle = 1.2`, `ds = 0.01`.
    pub fn new(tt4_lo: f64, tt4_hi: f64, r: f64) -> Self {
        Ramp { tt4_lo, tt4_hi, r, s_settle: 1.2, ds: 0.01 }
    }

    /// The `ds = 0.005` that rungs 58/59/60's READER METHODS declare — a DIFFERENT default on the
    /// same parameter, which is why it is spelled and not shared.
    ///
    /// **AND IT IS NOT WHAT THREE OF THE FOUR SUITES MARCH ON — corrected at step 3, where the
    /// earlier wording (*"rungs 58/59/60's default"*) nearly re-gridded two ported suites.**
    /// A method default and a suite constant are two different claims about one parameter, and
    /// they disagree here. Measured off the four files:
    ///
    /// ```text
    /// test_rung57.py   DS = 0.01    == its readers' default
    /// test_rung58.py   DS = 0.01    passed EXPLICITLY at every call site, overriding 0.005
    /// test_rung59.py   DS = 0.01    passed EXPLICITLY at every call site, overriding 0.005
    /// test_rung60.py   DS = 0.005   == its readers' default
    /// ```
    ///
    /// So `fine` is the right constructor for a caller reproducing a rung-58/59/60 READER's own
    /// default, and the WRONG one for a caller reproducing the rung-58 or rung-59 SUITE. Halving
    /// their step moves every number they assert, and those suites' gates are relational — they
    /// compare quantities to each other, so a finer grid moves both sides and none of them fires.
    pub fn fine(tt4_lo: f64, tt4_hi: f64, r: f64) -> Self {
        Ramp { tt4_lo, tt4_hi, r, s_settle: 1.2, ds: 0.005 }
    }

    pub fn with_ds(self, ds: f64) -> Self {
        Ramp { ds, ..self }
    }

    pub fn with_settle(self, s_settle: f64) -> Self {
        Ramp { s_settle, ..self }
    }

    pub fn with_r(self, r: f64) -> Self {
        Ramp { r, ..self }
    }
}

/// The ONE fuel-side min-select leg rung 58 composes with — Python's `(accel, surge, Tt4_max)`
/// keyword triple, all three defaulting to `None`, which is `integrate_fuel`'s own default and
/// therefore rung 57's reduce.
#[derive(Clone, Copy, Debug, Default)]
pub struct StatorLeg<'a> {
    /// RUNG 48's feedforward `Wf/pt3` table.
    pub accel: Option<&'a AccelSchedule>,
    /// RUNG 49's `phi` floor or RUNG 60's incidence floor — Python's ONE `surge=` slot.
    pub surge: Option<Floor>,
    /// RUNG 46's TIT redline.
    pub tt4_max: Option<f64>,
}

/// **§ 5.19 (iv)'s `Scope`, and this is the cell it was designed for.**
///
/// Python's rungs 65–68 each add ONE parameter to `_stator_march`, and each one is a per-MARCH
/// **isolation diagnostic** rather than a control setting: the body saves the previous value of
/// an instance attribute, assigns the parameter, calls `super()`, and restores in a `finally`.
/// Probe 2 emitted all four and the shape is identical every time:
///
/// | rung | adds | attribute | arrived |
/// |---|---|---|---|
/// | **65** | `b0` | `_b0` — the lagged valve's initial position | slice Y |
/// | **66** | `lag` | `_lag` — the FUEL leg's asymmetric lag | **slice Z** |
/// | **67** | `tau_gov` | `_tau_gov` — the GOVERNOR's clock | **slice Z** |
/// | 68 | `v0`, `ic_order` | `_v0`, `_ic_order` | slice AA |
///
/// So the cell's signature is opened **ONCE, here**, and this struct grows additively at 66/67/68
/// — one non-additive change instead of four. `Scope` had been retired field-by-field twice
/// (`try_close` at slice V, `_b_forced` at slice X) and it survives on the one cell § 5.19 (iv)
/// actually measured it onto.
///
/// # SLICE Z's P1 VERDICT — **the growth is additive in the TYPE and SOURCE-BREAKING in the
/// SYNTAX, and the second half had not been measured**
///
/// § 5.23 (iii) promised rungs 66/67/68 would grow this struct additively, and § 5.24 P1 made
/// that a prediction with a falsification clause: *"all 55 shipped call sites and every
/// `stator_march_scoped` signature stay as they are — falsified if any existing caller has to
/// change."* Both halves of the SIGNATURE claim hold exactly: [`stator_march`] and
/// [`stator_march_scoped`] are character-identical to what slice Y shipped, and **no un-scoped
/// call site moved.**
///
/// **AND "55" WAS ITSELF STALE — THE COUNT IS 82.** It is a number slice Y typed into this doc
/// comment and P1 inherited without re-running it. Counted at slice Z, over `src/` and `tests/`,
/// excluding comments: **82 un-scoped sites** (91 once slice Z's own file lands) and **16 scoped
/// ones**. The verdict does not change — none of the 82 moved — but a count carried forward
/// across a slice on the strength of a doc comment is the shape this port has been caught on
/// repeatedly, so it is re-measured here rather than re-quoted.
///
/// **But adding a field to a struct is a compile error at every EXHAUSTIVE STRUCT LITERAL of
/// it, and the port had NINE** — one in `src/lagged_bleed.rs` and **eight in four test files**
/// (`rung65.rs` ×3, `slice_y_oracle.rs` ×2, `slice_y_smoke.rs` ×2, `slice_y_dispatch.rs` ×1),
/// which is where a `src/`-only grep would have missed them. So P1 is **falsified at its letter**
/// and every one of the nine took the one-token repair `..MarchScope::DEFAULT`.
///
/// **THE COST IS PAID ONCE AND NOT PER RUNG.** A functional-update literal absorbs the next
/// field silently, so slice AA's `v0` / `ic_order` costs zero edits at these nine sites. That is
/// the reason the repair is a spread rather than a pair of `None`s typed nine times — and the
/// reason it is written down here rather than fixed quietly, because the precedent a later slice
/// inherits is *"growth is free"* and the measurement says *"growth is free from the SECOND time
/// on."*
///
/// [`stator_march`]: FuelTransientCore::stator_march
/// [`stator_march_scoped`]: FuelTransientCore::stator_march_scoped
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MarchScope {
    /// RUNG 65's `b0` — an override of the lagged valve's INITIAL position, and `None` is the
    /// physical initial condition (the equilibrium command), which leaves every march
    /// bit-for-bit. It exists because rung 65 § 3's finding is that `b` is a CONSTANT OF THE
    /// MOTION, and a constant of the motion is only demonstrable by moving its value.
    pub b0: Option<f64>,
    /// RUNG 66's `lag` — the FUEL-side leg's [`AsymmetricLag`], armed for ONE march.
    ///
    /// It rides here for rung 65's reason verbatim, and Python says so: `_stator_march` is called
    /// from a dozen rung-57-to-65 readers that know nothing about a fuel lag, and every one of
    /// them must keep reaching the IDENTICAL march. `None` leaves them all bit-for-bit.
    ///
    /// The carrier the value lands on is [`TwoSpoolTransientCore::lag`], through
    /// [`LaggedFuel`](crate::two_spool_transient::LaggedFuel).
    pub lag: Option<AsymmetricLag>,
    /// RUNG 67's `tau_gov` — the TIT topping governor's response clock, armed for ONE march.
    ///
    /// Rung 66's shape with one substitution: the fuel leg's SENSOR moves from `phi_lp` to `Tt4`.
    /// `Tt4_max` needs no plumbing of its own — it has been a rung-58 [`StatorLeg`] field since
    /// slice V, so the governor's REDLINE already reaches the march and only its CLOCK is new.
    ///
    /// The carrier is [`TwoSpoolTransientCore::tau_gov`], through
    /// [`LaggedGovernor`](crate::two_spool_transient::LaggedGovernor).
    pub tau_gov: Option<f64>,
}

impl MarchScope {
    /// What every un-scoped caller passes. A `const` rather than `Default::default()` so the
    /// forwarding methods stay `const`-friendly and the intent reads at the call site.
    ///
    /// **AND WHAT EVERY PARTIAL LITERAL SPREADS.** `MarchScope { b0, ..MarchScope::DEFAULT }` is
    /// the spelling slice Z's P1 verdict (above) settles on, because it survives slice AA's two
    /// further fields without an edit.
    pub const DEFAULT: MarchScope = MarchScope { b0: None, lag: None, tau_gov: None };
}

/// WHICH leg is armed — Python's `_one_leg` return, a string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegKind {
    Accel,
    Surge,
    Topping,
}

impl StatorLeg<'_> {
    /// Python's `_one_leg`: EXACTLY one of the three, asserted.
    pub fn one(&self) -> LegKind {
        let n = self.accel.is_some() as u8
            + self.surge.is_some() as u8
            + self.tt4_max.is_some() as u8;
        assert!(n == 1,
                "rung-58 composes the stator with EXACTLY ONE fuel-side leg. Two fuel legs is \
                 min-select algebra, not a composite: whenever one binds the other contributes \
                 exactly zero, so the interaction term is trivially -credit(other) -- the \
                 tautological-gate failure rungs 40/46 were caught by.");
        if self.accel.is_some() {
            LegKind::Accel
        } else if self.surge.is_some() {
            LegKind::Surge
        } else {
            LegKind::Topping
        }
    }

    /// Whether ANY leg is armed — `_cell`'s `armed` flag, which is a different question from
    /// [`one`](Self::one)'s and is asked on cells where the answer is "none".
    pub fn any(&self) -> bool {
        self.accel.is_some() || self.surge.is_some() || self.tt4_max.is_some()
    }

    /// The limiter set this leg IS, for [`FuelTransientCore::integrate_fuel`].
    fn limiters(&self) -> FuelLimiters<'_> {
        let (surge, incidence) = match self.surge {
            Some(Floor::Phi(s)) => (Some(s), None),
            Some(Floor::Incidence(i)) => (None, Some(i)),
            None => (None, None),
        };
        FuelLimiters {
            tt4_max: self.tt4_max,
            accel: self.accel,
            surge,
            incidence,
            ..Default::default()
        }
    }
}

/// RUNG 57's own three virtual names — the cells slice V CREATES, as opposed to the two it opens
/// in [`FuelTransientHooks`] and the one it swaps in [`TwoSpoolTransientHooks`].
///
/// **THE RECEIVERS DIFFER, AND THAT IS THE POINT OF ONE TABLE RATHER THAN TWO.** `arm` and `v_of`
/// are reached from INSIDE rung-40 and rung-43 hook bodies, so their `self` is the shallowest core
/// that can carry the state ([`TwoSpoolTransientCore`]); `stator_march` is reached only from
/// rung-57 readers and its body calls `fuel_for_tt4` / `equilibrium` / `integrate_fuel`, so its
/// `self` is [`FuelTransientCore`]. A struct of `fn` pointers has no receiver of its own, so one
/// table holds both — which is also what § 5.20 (vi) enumerates: three cells, one list.
pub struct StatorTransientHooks {
    /// Set BOTH maps from the CURRENT state, permanently. Overridden at rung **68** (slice AA).
    pub arm: fn(&TwoSpoolTransientCore, f64, f64, f64),
    /// The setting this machine HOLDS at a given state — constant or scheduled. The readers all
    /// go through this rather than through the live map, which `arm` leaves at whatever the LAST
    /// sub-step happened to be. Overridden at rung **68** (slice AA).
    pub v_of: fn(&TwoSpoolTransientCore, Spool, f64, f64, Option<f64>) -> f64,
    /// The rung-45 accel FUEL ramp on THIS machine. Overridden at rungs **65/66/67/68**.
    ///
    /// **THE `&MarchScope` IS SLICE Y's, AND IT IS THE ONLY NON-ADDITIVE CHANGE IN THAT SLICE.**
    /// Rungs 65–68 each add one per-march isolation parameter to this name and to no other, so
    /// the signature is opened once and the struct grows — see [`MarchScope`]. Callers do not
    /// pass it: [`FuelTransientCore::stator_march`] supplies [`MarchScope::DEFAULT`].
    #[allow(clippy::type_complexity)]
    pub stator_march: fn(&FuelTransientCore, &FlightCondition, &Ramp, Option<(f64, f64)>,
                         &StatorLeg<'_>, &MarchScope) -> (Vec<FuelPoint>, (f64, f64)),
    /// A sibling on the same hardware with the STATORS re-armed. **OPENED BY SLICE W, and it is
    /// the cell slice V shipped without.** § 5.20's closing note booked it as an inert deferral
    /// on the strength of a body read; § 5.21 (ii) measured it and the reading is the opposite.
    /// Rung 62 overrides it so the sibling carries THIS machine's VALVE, and
    /// `tests/test_rung63.py::test_the_at_stator_trap_is_real_and_returns_rung59s_zero_for_free`
    /// reads that override DIRECTLY: on a bleed-armed machine the inherited rung-59 reader
    /// `schedule_invariance` then compares the plant with ITSELF and reports rung 59's exact
    /// headline while measuring nothing — the counterfeit the gate exists to pin. Forcing rung
    /// 57's body there flips both of its identities from `true` to `false`, at `9.543e-3` and
    /// `1.019e-2`. Overridden at rung **64** as well.
    pub at_stator: fn(&ScheduledStatorCore, StatorArm) -> ScheduledStatorCore,
}

/// **THE DEFAULT, AND ITS CELLS PANIC.** Rung 40 has no `_arm` in Python at all — an unarmed
/// rung-40 or rung-43 object is not a rung-57 object with the lever at zero, it is an object where
/// the name does not exist. Defaulting these to [`r57_arm`]/[`r57_v_of`] would silently make a
/// rung-40 object armable, which is a claim no value gate could see; a panic that is unreachable
/// by construction is one `tests/slice_v_smoke.rs` can assert directly.
///
/// Unreachable because `r40_try_close` and `r43_try_close_fuel` carry NO arming call: the only
/// dispatchers are [`TwoSpoolTransientCore::arm`] and [`TwoSpoolTransientCore::v_of`], and the
/// only callers of those are rung 57's three cell bodies.
pub const NO_STATOR: StatorTransientHooks = StatorTransientHooks {
    arm: no_stator_arm,
    v_of: no_stator_v_of,
    stator_march: no_stator_march,
    at_stator: no_stator_at_stator,
};

/// RUNGS 57–60's table.
pub const R57: StatorTransientHooks = StatorTransientHooks {
    arm: r57_arm,
    v_of: r57_v_of,
    stator_march: r57_stator_march,
    at_stator: r57_at_stator,
};

/// RUNG 57's swap into rung 40's table — `try_close` ARMS first. The other two cells are rung
/// 40's own, untouched, which is the module doc's algebraic licence expressed as a table.
pub const R57_TWO: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: r57_try_close,
    ..R40
};

/// RUNG 57's swap into rung 43's table — BOTH cells, the two step 1a opened.
pub const R57_FUEL: FuelTransientHooks = FuelTransientHooks {
    try_close_fuel: r57_try_close_fuel,
    try_surge_fuel: r57_try_surge_fuel,
    // NOT overridden — rung 65's is the next body, and rung 57 marches through rung 43's. Named
    // rather than spread because slice Y's addition of this cell broke this literal, and a
    // `..R43` here would have made the next such addition silent.
    integrate_fuel: crate::fuel_transient::r43_integrate_fuel,
};

fn no_stator_arm(_: &TwoSpoolTransientCore, _: f64, _: f64, _: f64) {
    panic!("no stator table on this object: rungs 40/43 have no _arm at all, so this cell is \
            unreachable by construction (r40_try_close carries no arming call). Reaching it \
            means a rung-57 body ran on a core built without R57.");
}

fn no_stator_v_of(_: &TwoSpoolTransientCore, _: Spool, _: f64, _: f64, _: Option<f64>) -> f64 {
    panic!("no stator table on this object: rungs 40/43 have no v_of.");
}

fn no_stator_march(_: &FuelTransientCore, _: &FlightCondition, _: &Ramp, _: Option<(f64, f64)>,
                   _: &StatorLeg<'_>, _: &MarchScope) -> (Vec<FuelPoint>, (f64, f64)) {
    panic!("no stator table on this object: _stator_march is rung 57's own march.");
}

fn no_stator_at_stator(_: &ScheduledStatorCore, _: StatorArm) -> ScheduledStatorCore {
    panic!("no stator table on this object: at_stator is rung 57's own sibling constructor.             Its receiver is a ScheduledStatorCore, which cannot exist without R57.");
}

/// RUNG 57's own `at_stator` — the BARE sibling, carrying no valve, which is exactly what made
/// it the counterfeit rung 62 had to override. Unchanged from what slice V shipped inline; only
/// its call site moved behind the table.
fn r57_at_stator(core: &ScheduledStatorCore, arm: StatorArm) -> ScheduledStatorCore {
    match ScheduledStatorTransient::new(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_stator never disables LP"),
    }
}

// ---------------------------------------------------------------------------------------------
// The dispatch points, on the cores the cells' receivers name
// ---------------------------------------------------------------------------------------------

impl TwoSpoolTransientCore {
    /// Rung 57's `_arm`, **through the virtual table**.
    pub fn arm(&self, nu_lp: f64, nu_hp: f64, tt2: f64) {
        (self.stator_hooks.arm)(self, nu_lp, nu_hp, tt2)
    }

    /// Rung 57's `v_of`, **through the virtual table**. `tt2 = None` reads against the DESIGN
    /// `Tt2`, which is the convention every rung-57/58/60 reader uses.
    pub fn v_of(&self, spool: Spool, nu_lp: f64, nu_hp: f64, tt2: Option<f64>) -> f64 {
        (self.stator_hooks.v_of)(self, spool, nu_lp, nu_hp, tt2)
    }
}

impl FuelTransientCore {
    /// Rung 57's `_stator_march`, **through the virtual table** — reached from
    /// `&FuelTransientCore` because that is the cell's receiver, while the table itself lives one
    /// level down on [`TwoSpoolTransientCore`].
    pub fn stator_march(
        &self, flight: &FlightCondition, ramp: &Ramp, nu0: Option<(f64, f64)>,
        leg: &StatorLeg<'_>,
    ) -> (Vec<FuelPoint>, (f64, f64)) {
        self.stator_march_scoped(flight, ramp, nu0, leg, &MarchScope::DEFAULT)
    }

    /// The same march with rungs 65–68's per-march isolation parameters — see [`MarchScope`].
    ///
    /// Split from [`stator_march`](Self::stator_march) so that opening the cell's signature at
    /// slice Y moved **no** caller: every one of the shipped un-scoped sites wants the default.
    /// Slice Y wrote "55" here; re-counted at slice Z it is **82**, and the number now lives at
    /// [`MarchScope`] alone so there is one place to keep honest instead of two.
    pub fn stator_march_scoped(
        &self, flight: &FlightCondition, ramp: &Ramp, nu0: Option<(f64, f64)>,
        leg: &StatorLeg<'_>, scope: &MarchScope,
    ) -> (Vec<FuelPoint>, (f64, f64)) {
        (self.inner.stator_hooks.stator_march)(self, flight, ramp, nu0, leg, scope)
    }

    /// Rung 57's `v_of`, forwarded — the spelling the rung-57 readers use, since their `self` is
    /// this level.
    pub fn v_of(&self, spool: Spool, nu_lp: f64, nu_hp: f64, tt2: Option<f64>) -> f64 {
        self.inner.v_of(spool, nu_lp, nu_hp, tt2)
    }

    /// RUNG 60. The rung-49 leg a min-select floor IS at the CURRENT stator setting.
    ///
    /// A [`Floor::Phi`] is returned BY VALUE and UNCHANGED — Python returns it by IDENTITY (`is`,
    /// not `==`), and [`SurgeLimiter`] is `Copy`, so the Rust claim is field-wise equality plus a
    /// dispatch count. That weakening is stated here because a reduce gate that silently answers a
    /// smaller question is the *ported test can go VACUOUS* failure.
    ///
    /// A [`Floor::Incidence`] is converted through [`IncidenceLimiter::try_at`], which is legal
    /// rather than circular because `v` is a function of the SHAFT STATE alone: rung 49's bracket
    /// (*cutting fuel raises phi*) needs the floor to be constant in the fuel, and it is.
    ///
    /// The setting is read through `v_of`, i.e. against the DESIGN `Tt2` — the same convention
    /// rungs 57/58 already use. It is exact at the design flight condition, which is where every
    /// claim is made.
    pub fn resolve_floor(
        &self, floor: &Floor, nu_lp: f64, nu_hp: f64,
    ) -> Result<SurgeLimiter, Abort> {
        match floor {
            Floor::Phi(s) => {
                bump(&RESOLVE_PHI);
                Ok(*s)
            }
            Floor::Incidence(inc) => {
                bump(&RESOLVE_INCIDENCE);
                let cmap = self.inner.stator.design_map(inc.spool);
                inc.try_at(cmap.tan_beta1_crit(), self.v_of(inc.spool, nu_lp, nu_hp, None))
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// RUNG 57's CELLS
// ---------------------------------------------------------------------------------------------

/// RUNG 57. Set both maps from the CURRENT state.
///
/// A pure function of `(nu_L, nu_H, Tt2)` — no history, no latch, so it is RK4-legal exactly as
/// rung 50's `s`-threading was. Returns immediately when nothing is scheduled: **THE REDUCE.**
///
/// **AND IT WRITES THROUGH A `Cell` AND NEVER RESTORES**, which is the module doc's whole subject.
/// Both arms are counted because P3's reduce gate is written on them: a `v_max = 0.0` schedule
/// hands back the DESIGN map, and Python asserts that by object IDENTITY (`is`) — a test that does
/// not survive a `Copy` type as written, so the port re-gates it as equality PLUS the dispatch
/// count [`Census::arm_lp_zero`].
pub fn r57_arm(t: &TwoSpoolTransientCore, nu_lp: f64, nu_hp: f64, tt2: f64) {
    bump(&ARM_CALLS);
    let a = &t.stator;
    if !a.is_scheduled() {
        bump(&ARM_UNARMED);
        return;
    }
    if let Some(s) = a.sched_lp {
        let v = s.at(nu_lp * powp(t.inner.tt2_d / tt2, 0.5));
        if v == 0.0 {
            bump(&ARM_LP_ZERO);
            t.inner.set_map_lp(a.map_lp_design);
        } else {
            bump(&ARM_LP_MOVED);
            t.inner.set_map_lp(a.map_lp_design.with_vsv(v));
        }
    }
    if let Some(s) = a.sched_hp {
        // See `ScheduledStatorCore`'s CONCESSIONS: this reads the HP SHAFT speed, not its
        // corrected speed, because `Tt25` is an OUTPUT of the very root the schedule arms.
        let v = s.at(nu_hp);
        if v == 0.0 {
            bump(&ARM_HP_ZERO);
            t.inner.set_map_hp(a.map_hp_design);
        } else {
            bump(&ARM_HP_MOVED);
            t.inner.set_map_hp(a.map_hp_design.with_vsv(v));
        }
    }
}

/// RUNG 57. The setting this machine holds at the given state — constant or scheduled.
pub fn r57_v_of(
    t: &TwoSpoolTransientCore, spool: Spool, nu_lp: f64, nu_hp: f64, tt2: Option<f64>,
) -> f64 {
    bump(&VOF_CALLS);
    let a = &t.stator;
    match spool {
        Spool::Lp => match a.sched_lp {
            None => a.vsv_lp,
            Some(s) => {
                let t2 = tt2.unwrap_or(t.inner.tt2_d);
                s.at(nu_lp * powp(t.inner.tt2_d / t2, 0.5))
            }
        },
        Spool::Hp => match a.sched_hp {
            None => a.vsv_hp,
            Some(s) => s.at(nu_hp),
        },
    }
}

/// RUNG 57. The rung-45 accel FUEL ramp on THIS machine.
///
/// Deliberately NOT `fuel_ramp_march`: that one references the commanded running line and reads
/// the FIELD `phi_surge`, which rung 53 pinned to the DESIGN setting so rungs 41/44/45's readers
/// stay literally unchanged. Under a moving stator that field is the wrong wall, so rung 57 reads
/// its own ([`ComponentMap::phi_surge_at`]) through a march of its own. `nu0 = None` starts on THIS
/// machine's own running line.
///
/// RUNG 58 threads ONE fuel-side min-select leg through. All three default to `None`, which is
/// `integrate_fuel`'s own default, so every rung-57 caller reaches the IDENTICAL march: THE
/// REDUCE.
pub fn r57_stator_march(
    ft: &FuelTransientCore, flight: &FlightCondition, ramp: &Ramp, nu0: Option<(f64, f64)>,
    leg: &StatorLeg<'_>, _scope: &MarchScope,
) -> (Vec<FuelPoint>, (f64, f64)) {
    bump(&MARCH_CALLS);
    let mf_lo = ft.fuel_for_tt4(flight, ramp.tt4_lo);
    let mf_hi = ft.fuel_for_tt4(flight, ramp.tt4_hi);
    let nu0 = nu0.unwrap_or_else(|| {
        let eq = ft.inner.equilibrium(flight, ramp.tt4_lo);
        (eq.nu_lp, eq.nu_hp)
    });
    let r = ramp.r;
    let sched = move |s: f64| -> f64 {
        if s <= 0.0 {
            mf_lo
        } else if s >= r {
            mf_hi
        } else {
            mf_lo + (mf_hi - mf_lo) * (s / r)
        }
    };
    let traj = ft.integrate_fuel(flight, sched, nu0, ramp.r + ramp.s_settle, ramp.ds,
                                 &leg.limiters());
    (traj, nu0)
}

/// RUNG 57's rung-40 cell: ARM, then rung 40's own body.
///
/// `r40_try_close` and not `t.try_close` — the latter is this cell, i.e. infinite recursion. That
/// is what step 1a's move of the bodies out of the `impl` blocks bought.
pub fn r57_try_close(
    t: &TwoSpoolTransientCore, nu_lp: f64, nu_hp: f64, tt4: f64, tt2: f64, pt2: f64,
) -> Result<CloseState, Abort> {
    t.arm(nu_lp, nu_hp, tt2);
    (R40.try_close)(t, nu_lp, nu_hp, tt4, tt2, pt2)
}

/// RUNG 57's rung-43 closure cell: ARM, then rung 43's own body.
pub fn r57_try_close_fuel(
    ft: &FuelTransientCore, nu_lp: f64, nu_hp: f64, mdot_fuel: f64, tt2: f64, pt2: f64,
) -> Result<crate::fuel_transient::FuelCloseState, Abort> {
    ft.inner.arm(nu_lp, nu_hp, tt2);
    (crate::fuel_transient::R43.try_close_fuel)(ft, nu_lp, nu_hp, mdot_fuel, tt2, pt2)
}

/// RUNG 60's leg cell: rung 49's set-point solve, on the floor RESOLVED at the live setting.
pub fn r57_try_surge_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, nu_lp: f64, nu_hp: f64, mf_sched: f64,
    floor: &Floor,
) -> Result<f64, Abort> {
    let resolved = Floor::Phi(ft.resolve_floor(floor, nu_lp, nu_hp)?);
    (crate::fuel_transient::R43.try_surge_fuel)(ft, flight, nu_lp, nu_hp, mf_sched, &resolved)
}
// ---------------------------------------------------------------------------------------------
// THE OBJECT
// ---------------------------------------------------------------------------------------------

/// How a rung-57 object is armed — Python's five constructor keywords past the hardware.
///
/// `Default::default()` is the BARE machine, which is exactly what
/// [`ScheduledStatorCore::at_stator`] builds with no arguments and what every rung-58/59/60
/// composite differences against.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StatorArm {
    pub vsv_lp: f64,
    pub vsv_hp: f64,
    pub sched_lp: Option<StatorSchedule>,
    pub sched_hp: Option<StatorSchedule>,
    /// Carried ONLY so rung 57's fourth assert is expressible — see
    /// [`ScheduledStatorTransient::new`], which refuses a BARE `lp_disabled` object for a TYPING
    /// reason rather than a physical one.
    pub lp_disabled: bool,
}

impl StatorArm {
    pub fn constant(vsv_lp: f64, vsv_hp: f64) -> Self {
        StatorArm { vsv_lp, vsv_hp, ..Default::default() }
    }

    pub fn scheduled_lp(s: StatorSchedule) -> Self {
        StatorArm { sched_lp: Some(s), ..Default::default() }
    }

    pub fn scheduled_hp(s: StatorSchedule) -> Self {
        StatorArm { sched_hp: Some(s), ..Default::default() }
    }
}

/// RUNGS 57–60. Rung 53's VARIABLE STATOR on rungs 43/45's FUEL-metered two-shaft plant — the
/// first lever that moves the surge FLOOR *during* an acceleration.
///
/// `lp_disabled=True` dispatches to rung 35's [`SpoolTransient`] fuel path, exactly as rungs 40
/// and 43 do — but Python's early return hands `SpoolTransient` a SINGLE-spool design engine while
/// rung 57's own signature is typed for a two-spool one, so the two constructors split here as
/// they already do on [`crate::fuel_transient::TwoSpoolFuelTransient`]. Rungs 57–60 build the
/// degenerate arm **zero** times: their only `lp_disabled` call site is the refusal gate, which
/// arms the lever and therefore never reaches construction.
pub enum ScheduledStatorTransient {
    Degenerate(SpoolTransient),
    Full(ScheduledStatorCore),
}

impl ScheduledStatorTransient {
    /// Rung 57's constructor, carrying all four of Python's asserts.
    ///
    /// `map_lp`/`map_hp` are the DESIGN-SETTING maps: rung 57 moves the stators itself (rung 53's
    /// capture discipline), so a map already carrying `.with_vsv(.)` is refused.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: StatorArm,
    ) -> Self {
        Self::with_tables(design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm,
                          &R57_TWO, &R57, &R57_FUEL, &crate::bleed_transient::NO_LEVER,
                          crate::bleed_transient::LeverArming::unarmed())
    }

    /// [`new`](Self::new) with the four tables and the valve arming named — the constructor every
    /// DESCENDANT goes through, and the reason rung 62 needs no object type of its own.
    ///
    /// **A DESCENDANT MUST NAME ITS PARENT'S TABLES IN ITS SPREADS, NOT RUNG 40's/43's.** Rung
    /// 62's `R62_FUEL` is `{ try_close_fuel: …, ..R57_FUEL }`, because rung 62 does not override
    /// `_surge_fuel` and Python's `super()` reaches rung 60's floor-resolving body there. Spelling
    /// `..R43` compiles and silently drops it. § 5.21 (vi).
    #[allow(clippy::too_many_arguments)]
    pub fn with_tables(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: StatorArm,
        two_hooks: &'static TwoSpoolTransientHooks,
        stator_hooks: &'static StatorTransientHooks,
        fuel_hooks: &'static FuelTransientHooks,
        lever_hooks: &'static crate::bleed_transient::LeverHooks,
        lever: crate::bleed_transient::LeverArming,
    ) -> Self {
        let base_lp = map_lp.unwrap_or_else(ComponentMap::flat);
        let base_hp = map_hp.unwrap_or_else(ComponentMap::flat);
        assert!(base_lp.vsv == 0.0 && base_hp.vsv == 0.0,
                "rung-57 takes the DESIGN-SETTING maps and moves the stators itself (rung 53's \
                 capture discipline). Pass vsv_lp/vsv_sched_lp, not a map already carrying \
                 .with_vsv(.).");
        assert!(!(arm.vsv_lp != 0.0 && arm.sched_lp.is_some()),
                "rung-57: a spool gets a CONSTANT setting or a SCHEDULE, not both -- they are the \
                 two legs the rung differences.");
        assert!(!(arm.vsv_hp != 0.0 && arm.sched_hp.is_some()),
                "rung-57: a spool gets a CONSTANT setting or a SCHEDULE, not both.");
        assert!(!(arm.lp_disabled
                  && (arm.vsv_lp != 0.0 || arm.vsv_hp != 0.0 || arm.sched_lp.is_some()
                      || arm.sched_hp.is_some())),
                "rung-57's findings are per-SPOOL and inter-spool (it corrects rung 53's P5 \
                 arrow); lp_disabled is not a reduce axis for them.");
        assert!(!arm.lp_disabled,
                "a BARE lp_disabled rung-57 object forwards to rung 35's SINGLE-spool fuel path, \
                 which needs a single-spool design engine -- Python's early return re-reads its \
                 `design_engine` parameter as one. Build it through \
                 ScheduledStatorTransient::lp_disabled. Rungs 57-60 never do.");

        let arming = StatorArming {
            vsv_lp: arm.vsv_lp,
            vsv_hp: arm.vsv_hp,
            sched_lp: arm.sched_lp,
            sched_hp: arm.sched_hp,
            map_lp_design: base_lp,
            map_hp_design: base_hp,
        };
        let fuel = FuelTransientCore {
            inner: TwoSpoolTransientCore::with_lever_hooks(
                design_engine.clone(), flight_design, mdot_design, base_lp, base_hp, rho,
                two_hooks, stator_hooks, arming, lever_hooks, lever),
            hooks: fuel_hooks,
        };
        // A CONSTANT setting is applied ONCE, HERE -- after the design capture above, exactly as
        // rung 53 does it, so `equilibrium` sees the statored machine and the march starts on the
        // STATORED running line. (Arming only the fuel closure instead is the error probe E made
        // and probe G caught; see the anchor doc.) It writes through the same `Cell` `_arm` does,
        // which is why rung 53's own constructor lost its `let mut` at step 1b.
        if arm.vsv_lp != 0.0 {
            fuel.inner.inner.set_map_lp(base_lp.with_vsv(arm.vsv_lp));
        }
        if arm.vsv_hp != 0.0 {
            fuel.inner.inner.set_map_hp(base_hp.with_vsv(arm.vsv_hp));
        }
        ScheduledStatorTransient::Full(ScheduledStatorCore {
            fuel,
            design_engine,
            flight_design,
            mdot_design,
            rho,
        })
    }

    /// `lp_disabled=True`, BARE — Python's early return, which builds rung 34/35's single-spool
    /// object off the design engine it was handed.
    pub fn lp_disabled(
        design_engine: crate::engine::Engine, flight_design: FlightCondition, mdot_design: f64,
        map_hp: ComponentMap,
    ) -> Self {
        ScheduledStatorTransient::Degenerate(
            SpoolTransient::new(design_engine, flight_design, mdot_design, map_hp))
    }

    pub fn core(&self) -> &ScheduledStatorCore {
        match self {
            ScheduledStatorTransient::Full(c) => c,
            ScheduledStatorTransient::Degenerate(_) => panic!("this transient is lp_disabled"),
        }
    }

    pub fn degenerate(&self) -> &SpoolTransient {
        match self {
            ScheduledStatorTransient::Degenerate(s) => s,
            ScheduledStatorTransient::Full(_) => panic!("this transient is not lp_disabled"),
        }
    }
}

/// Rung 57's object once `lp_disabled` is ruled out: rung 43's fuel transient, plus Python's
/// `_ctor` tuple, which exists for exactly one reason — [`at_stator`](Self::at_stator) re-invokes
/// the constructor. [`crate::stator::VariableStatorCore`]'s three held fields, one ladder on.
pub struct ScheduledStatorCore {
    /// Rung 43's core. The arming and rung 57's table live on `fuel.inner`, not here — see
    /// [`crate::two_spool_transient::TwoSpoolTransientCore::stator`] for why that level.
    pub fuel: FuelTransientCore,
    design_engine: TwoSpoolEngine,
    flight_design: FlightCondition,
    mdot_design: f64,
    rho: f64,
}

impl ScheduledStatorCore {
    /// The arming this machine carries.
    pub fn arming(&self) -> &StatorArming {
        &self.fuel.inner.stator
    }

    /// The DESIGN map of a spool — the reference every wall in rungs 57–60 is read off.
    pub fn design_map(&self, spool: Spool) -> ComponentMap {
        self.arming().design_map(spool)
    }

    /// Python's `v_of`.
    pub fn v_of(&self, spool: Spool, nu_lp: f64, nu_hp: f64, tt2: Option<f64>) -> f64 {
        self.fuel.v_of(spool, nu_lp, nu_hp, tt2)
    }

    /// A sibling on the SAME hardware and the same design references, stators re-armed — rung
    /// 53's `at_setting`, one ladder on. Every difference in rungs 57–60 goes through this, so a
    /// swept setting can never be confused with a re-designed engine.
    ///
    /// **NO CELL.** § 5.19 (iii) classes it with `at_lever` as a pure sibling constructor: it
    /// dispatches nothing and reads no state a descendant could redefine. **AND THAT IS A
    /// DEFERRAL WITH A COUNT, NOT AN INTENTION** — rungs **62 and 64** override it in Python, and
    /// **eight** reader bodies below call `self.at_stator()` (`stator_credit`,
    /// `credit_decomposition`, `composite_credit`, `engagement_shift`, `schedule_invariance`,
    /// `matched_credit`, `set_point_bands`, `floor_composite`). Those eight are INHERITED by
    /// rungs 62/64, so a rung-64 object running the inherited `stator_credit` would build a
    /// rung-**57** bare sibling here. Not slice V's problem — rungs 57–60 construct only rung-57
    /// objects — ~~and slice W's first job.~~
    ///
    /// **SLICE W DID THAT JOB, AND THE DEFERRAL'S REASONING WAS HALF WRONG.** It is a cell now
    /// ([`StatorTransientHooks::at_stator`]), and the half that was wrong is *"inert here"* being
    /// read as *"inert until someone overrides it"*: rung 62's override is read by a **shipped
    /// rung-63 gate** whose whole content is that this constructor carries the valve. The
    /// measurement is in the cell's own note. What the deferral got right is the count — eight
    /// reader bodies, of which the suites exercise exactly **one**.
    pub fn at_stator(&self, arm: StatorArm) -> ScheduledStatorCore {
        (self.fuel.inner.stator_hooks.at_stator)(self, arm)
    }

    // --- what the cell bodies need, since the four held fields are private ------------------

    pub fn design_engine(&self) -> &TwoSpoolEngine { &self.design_engine }
    pub fn flight_design(&self) -> &FlightCondition { &self.flight_design }
    pub fn mdot_design(&self) -> f64 { self.mdot_design }
    pub fn rho(&self) -> f64 { self.rho }

    /// The BARE sibling — `at_stator()` with no arguments, which is how Python spells it eight
    /// times below.
    pub fn bare(&self) -> ScheduledStatorCore {
        self.at_stator(StatorArm::default())
    }

    /// Python's `_stator_march`, through the table.
    pub fn stator_march(
        &self, flight: &FlightCondition, ramp: &Ramp, nu0: Option<(f64, f64)>,
        leg: &StatorLeg<'_>,
    ) -> (Vec<FuelPoint>, (f64, f64)) {
        self.fuel.stator_march(flight, ramp, nu0, leg)
    }

    /// [`FuelTransientCore::stator_march_scoped`], forwarded — the spelling rung 65's readers use.
    pub fn stator_march_scoped(
        &self, flight: &FlightCondition, ramp: &Ramp, nu0: Option<(f64, f64)>,
        leg: &StatorLeg<'_>, scope: &MarchScope,
    ) -> (Vec<FuelPoint>, (f64, f64)) {
        self.fuel.stator_march_scoped(flight, ramp, nu0, leg, scope)
    }
}

// ---------------------------------------------------------------------------------------------
// The reading instruments — Python's return dicts, one struct each
// ---------------------------------------------------------------------------------------------

/// WHERE a trajectory's incidence minimum sits, and at what setting — Python's `at` sub-dict.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtRow {
    pub s: f64,
    pub phi: f64,
    pub v: f64,
    pub nu_lp: f64,
    pub nu_hp: f64,
}

/// BOTH rung-53 currencies on ONE spool, minimised over a trajectory.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReadRow {
    /// `M_phi = phi_op - phi_surge(v)` — the wall MOVES with `v`.
    pub m_phi: f64,
    /// `M_i = T_c - (1/phi_op - v)` — the wall is the METAL.
    pub m_i: f64,
    pub t_c: f64,
    /// `None` only on a trajectory no point of which produced a finite `M_i` — Python's
    /// `row = None` initial, which no shipped march reaches.
    pub at: Option<AtRow>,
    pub min_phi: f64,
}

impl ReadRow {
    /// Python's `d["at"]["s"]`, which raises on `None`. Same here, loudly.
    pub fn at(&self) -> &AtRow {
        self.at.as_ref().expect(
            "rung-57 _read found no finite incidence minimum on this trajectory")
    }
}

/// [`ScheduledStatorCore::read`]'s return — both spools plus the point count.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatorRead {
    pub lp: ReadRow,
    pub hp: ReadRow,
    pub npts: usize,
}

impl StatorRead {
    pub fn spool(&self, spool: Spool) -> &ReadRow {
        match spool {
            Spool::Lp => &self.lp,
            Spool::Hp => &self.hp,
        }
    }
}

/// RUNG 57's reading instrument — [`ScheduledStatorCore::stator_transient_margin`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransientMargin {
    pub read: StatorRead,
    pub nu0_lp: f64,
    pub nu0_hp: f64,
    pub r: f64,
}

/// THE FINDING (rung 57) — [`ScheduledStatorCore::stator_credit`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatorCredit {
    pub spool: Spool,
    pub r: f64,
    pub bare: f64,
    pub armed: f64,
    pub pointwise: f64,
    pub credit: f64,
    pub credit_pointwise: f64,
    /// FALSE for a SCHEDULE, and the flag is the disclosure: a state-fed schedule's `pointwise`
    /// leg is referenced to the setting the schedule would command ON THE BARE TRAJECTORY while
    /// the net leg carries the setting it actually commands, so `erosion` mixes the work channel
    /// with the self-cancellation instead of isolating it. Every erosion number rung 57 publishes
    /// is a constant-`v` one.
    pub pointwise_exact: bool,
    pub erosion: f64,
    /// Rung 53's design-point closed form for the surviving share, `1/(2+l)`.
    pub closed_form: f64,
    pub v_at_min: f64,
    pub s_at_min: f64,
    pub s_at_min_bare: f64,
    pub nu0_bare: f64,
    pub nu0_armed: f64,
    pub min_phi_bare: f64,
    pub min_phi_armed: f64,
    pub m_phi_bare: f64,
    pub m_phi_armed: f64,
}

/// WHERE a state-fed schedule's credit is delivered — three legs on one ramp.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CreditDecomposition {
    pub spool: Spool,
    pub r: f64,
    pub bare: f64,
    /// `nu0` from the ARMED running line, then marched at the DESIGN setting — the head start
    /// ALONE.
    pub start: f64,
    /// `nu0` from the BARE running line, marched with the schedule live.
    pub ramp: f64,
    pub full: f64,
    pub share_start: f64,
    pub share_ramp: f64,
    /// `full/ramp` below 1 is the schedule's SELF-CANCELLATION — the one thing a constant setting
    /// cannot do.
    pub self_cancel: f64,
    pub nu0_bare: f64,
    pub nu0_armed: f64,
}

/// RUNG 53's P5 on the TRANSIENT closure — [`ScheduledStatorCore::arrow_toggle`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArrowToggle {
    pub spool: Spool,
    pub v: f64,
    pub s: f64,
    pub state: (f64, f64, f64),
    pub nu_lp: f64,
    pub nu_hp: f64,
    pub d_phi_lp: f64,
    pub d_phi_hp: f64,
    pub d_n_hp: f64,
    pub d_tt25: f64,
    pub phi_lp: f64,
    pub phi_hp: f64,
}

/// One cell of rung 58/59/60's composite — [`ScheduledStatorCore::cell`].
#[derive(Clone, Debug, PartialEq)]
pub struct CellRead {
    /// The PARABOLA-REFINED incidence minimum.
    pub m_i: f64,
    /// The same minimum read at grid points, which is rung 57's number.
    pub m_i_grid: f64,
    pub m_phi: f64,
    pub s: f64,
    /// The incidence margin as a PROFILE in `s`.
    pub prof: Vec<(f64, f64)>,
    pub v: f64,
    pub s_grid: f64,
    pub min_phi: f64,
    pub nu0: f64,
    pub nu_lp_end: f64,
    pub nu_hp_end: f64,
    pub tt4_peak: f64,
    pub fuel_removed: f64,
    /// Sub-grid engagement time, or NaN when no leg is armed — **an `f64` carrying NaN and not an
    /// `Option`**, because [`ScheduledStatorCore::pin_audit`]'s `from_zero` is a self-inequality
    /// test on it and it is a dumped key.
    pub s_eng: f64,
    pub npts: usize,
}

/// Rung 58/60's four cells.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositeCells {
    pub neither: CellRead,
    pub stator: CellRead,
    pub fuel: CellRead,
    pub both: CellRead,
}

/// THE RUNG (58) — [`ScheduledStatorCore::composite_credit`].
#[derive(Clone, Debug, PartialEq)]
pub struct CompositeCredit {
    /// The interaction PREDICTED from the two fuel-leg-free marches alone.
    pub predicted: f64,
    pub profile_bare: f64,
    pub profile_fuel: f64,
    pub spool: Spool,
    pub r: f64,
    pub ds: f64,
    pub leg: LegKind,
    pub cells: CompositeCells,
    pub credit_bare: f64,
    pub credit_fuel: f64,
    pub interaction: f64,
    pub share: f64,
    pub v_bare: f64,
    pub v_fuel: f64,
    pub v_ratio: f64,
    pub relocation: f64,
    pub relocation_bare: f64,
    pub leg_cost_bare: f64,
    pub leg_cost_armed: f64,
    pub fuel_removed_bare: f64,
    pub fuel_removed_armed: f64,
}

/// RUNG 58's CONVERSE reading — [`ScheduledStatorCore::engagement_shift`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EngagementShift {
    pub r: f64,
    pub ds: f64,
    pub leg: LegKind,
    pub bare_limited: f64,
    pub bare_dormant: f64,
    pub armed_limited: f64,
    pub armed_dormant: f64,
    pub d_limited: f64,
    pub d_dormant: f64,
    pub rel_limited: f64,
    pub rel_dormant: f64,
}

/// One row of rung 58's MECHANISM sweep.
#[derive(Clone, Debug, PartialEq)]
pub struct InteractionRow {
    pub tag: String,
    pub credit_bare: f64,
    pub credit_fuel: f64,
    pub interaction: f64,
    pub share: f64,
    pub v_bare: f64,
    pub v_fuel: f64,
    pub v_ratio: f64,
    pub relocation: f64,
    pub leg_cost_bare: f64,
    pub leg_cost_armed: f64,
}

/// RUNG 59. The three factors `kappa_ss` is BUILT from, at one steady point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProofChain {
    pub tt4: f64,
    pub tt25: f64,
    pub tt3: f64,
    pub f: f64,
    pub mfp: f64,
    pub ratio: f64,
    pub n_hp: f64,
    pub nu_lp: f64,
    pub kappa: f64,
}

/// One `Tt4` of [`ScheduledStatorCore::schedule_invariance`]'s proof chain, as RELATIVE
/// differences armed-minus-bare.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChainRow {
    pub tt4: f64,
    pub d_tt25: f64,
    pub d_tt3: f64,
    pub d_f: f64,
    pub d_mfp: f64,
    pub d_ratio: f64,
    pub d_kappa: f64,
    pub d_n_hp: f64,
    pub d_nu_lp: f64,
}

/// RUNG 59, FIRST HALF — [`ScheduledStatorCore::schedule_invariance`].
#[derive(Clone, Debug, PartialEq)]
pub struct ScheduleInvariance {
    pub bare: AccelSchedule,
    pub matched: AccelSchedule,
    /// Tuple-level identity on the ORDINATE — an LP stator cannot reach it AT ALL.
    pub ordinate_identical: bool,
    /// Tuple-level identity on the ABSCISSA — an LP stator leaves it alone (rung 39's ONE arrow);
    /// an HP stator moves it.
    pub abscissa_identical: bool,
    pub d_ordinate: f64,
    pub d_abscissa: f64,
    pub chain: Vec<ChainRow>,
}

/// RUNG 59's standing BLOCKER check — is a leg consulted OUTSIDE its own derived bracket, where
/// `AccelSchedule::cap` clamps and the number is an envelope edge rather than the derived shape?
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClampAudit {
    pub lo: f64,
    pub hi: f64,
    pub n_min: f64,
    pub n_max: f64,
    pub n_cuts: usize,
    pub cut_lo: f64,
    pub cut_hi: f64,
    pub clamped: usize,
}

/// Rung 59's seven cells.
#[derive(Clone, Debug, PartialEq)]
pub struct MatchedCells {
    pub neither: CellRead,
    pub stator: CellRead,
    pub fuel: CellRead,
    pub both_bare_leg: CellRead,
    pub both_matched: CellRead,
    /// ARMED index, BARE values.
    pub both_reindexed: CellRead,
    /// BARE index, ARMED values.
    pub both_revalued: CellRead,
}

/// THE RUNG (59) — [`ScheduledStatorCore::matched_credit`].
#[derive(Clone, Debug, PartialEq)]
pub struct MatchedCredit {
    pub spool: Spool,
    pub r: f64,
    pub ds: f64,
    pub margin: f64,
    pub cells: MatchedCells,
    pub audit_fuel: ClampAudit,
    pub audit_both_bare_leg: ClampAudit,
    pub audit_both_matched: ClampAudit,
    pub ordinate_identical: bool,
    pub abscissa_identical: bool,
    pub d_ordinate: f64,
    pub d_abscissa: f64,
    pub credit_bare: f64,
    pub interaction_bare_leg: f64,
    pub interaction_matched: f64,
    pub delta_match: f64,
    pub delta_index: f64,
    pub delta_value: f64,
    pub abscissa_share: f64,
    pub ordinate_share: f64,
    pub share_bare_leg: f64,
    pub share_matched: f64,
    pub s_eng_bare_leg: f64,
    pub s_eng_matched: f64,
    pub removed_bare_leg: f64,
    pub removed_matched: f64,
    pub relocation: f64,
}

/// RUNG 60. The two ways to MATCH a `phi` set point to a stator-armed machine, and the DERIVED
/// gap between them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MatchingRules {
    pub sm: f64,
    pub v: f64,
    pub t_c: f64,
    pub phi_bare: f64,
    pub m_bare: f64,
    /// Fixed `phi`-MARGIN off the moved wall.
    pub phi_rel: f64,
    /// Fixed INCIDENCE.
    pub phi_inc: f64,
    pub gap: f64,
    pub gap_closed_form: f64,
    pub residual: f64,
}

/// RUNG 60. The ADMISSIBLE SET-POINT band of one machine, in BOTH coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Band {
    pub phi_0: f64,
    pub phi_min: f64,
    pub phi_exc: f64,
    pub m_0: f64,
    pub m_min: f64,
    pub m_exc: f64,
    pub t_c: f64,
    pub v_0: f64,
}

/// RUNG 60, FIRST HALF — [`ScheduledStatorCore::set_point_bands`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SetPointBands {
    pub spool: Spool,
    pub r: f64,
    pub ds: f64,
    pub bare: Band,
    pub armed: Band,
    /// `> 0` ⇒ DISJOINT (bare band above armed).
    pub gap_phi: f64,
    /// `> 0` ⇒ DISJOINT (armed band above bare).
    pub gap_m: f64,
    pub gap_phi_bands: f64,
    pub gap_m_bands: f64,
    pub credit: f64,
    pub excursion: f64,
    /// `credit - excursion` — a fixed incidence set point is admissible IFF this is negative.
    pub criterion: f64,
    pub identity_residual: f64,
    pub phi_admissible: bool,
    pub m_admissible: bool,
    pub overlap_lo: f64,
    pub overlap_hi: f64,
}

/// One row of rung 60's composability ladder.
#[derive(Clone, Debug, PartialEq)]
pub struct LadderRow {
    pub tag: String,
    pub r: f64,
    pub credit: f64,
    pub excursion: f64,
    pub criterion: f64,
    pub gap_m: f64,
    pub gap_m_bands: f64,
    pub gap_phi: f64,
    pub gap_phi_bands: f64,
    pub m_admissible: bool,
    pub phi_admissible: bool,
}

/// WHICH axis [`ScheduledStatorCore::composability_ladder`] walks. Python takes two mutually
/// exclusive keyword lists and asserts `(legs is None) != (rates is None)`; an enum makes the
/// exclusion unrepresentable, and the finding is that the two axes carry DIFFERENT halves of the
/// criterion.
pub enum LadderAxis<'a> {
    /// A ladder of stator legs at FIXED ramp rate.
    Legs(&'a [(String, StatorArm)]),
    /// A ladder of ramp RATES at a fixed leg.
    Rates(&'a [(f64, StatorArm)]),
}

/// RUNG 60's BLOCKER check — the artifact most likely to counterfeit the rung.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PinAudit {
    pub m_set: f64,
    pub m_min: f64,
    pub residual: f64,
    /// The minimum IS the set point, to solver tolerance — the tautology.
    pub pinned: bool,
    /// The leg removed no fuel at all, so the cell is bit-identical to its leg-free sibling.
    pub dormant: bool,
    /// The leg CUTS but has no upward crossing at all. It does NOT discriminate between the two
    /// causes (set point above the `s = 0` value, or first binding past the last grid point) —
    /// only that there is no engagement inside the ramp.
    pub from_zero: bool,
    pub admissible: bool,
    pub s_eng: f64,
    pub removed: f64,
}

/// Which of a floor's three regimes a composite is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Regime {
    BothPinned,
    ArmedClears,
    Mixed,
}

/// Which coordinate the floor leg is written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorKind {
    Phi,
    Incidence,
}

/// THE RUNG (60) — [`ScheduledStatorCore::floor_composite`].
#[derive(Clone, Debug, PartialEq)]
pub struct FloorComposite {
    pub spool: Spool,
    pub r: f64,
    pub ds: f64,
    pub cells: CompositeCells,
    pub audit_fuel: PinAudit,
    pub audit_both: PinAudit,
    pub regime: Regime,
    pub floor: FloorKind,
    pub admissible: bool,
    pub credit_bare: f64,
    pub credit_fuel: f64,
    pub interaction: f64,
    /// The DERIVED value the tautology must take: `v` for a `phi` floor, EXACTLY `0` for an
    /// incidence one. The gate is that the measurement meets it at machine precision, which is
    /// the OPPOSITE of the usual gate and is the point.
    pub pinned_prediction: f64,
    pub pinned_residual: f64,
    /// The half that is NOT pinned — a time has no wall.
    pub s_eng_bare: f64,
    pub s_eng_armed: f64,
    pub d_s_eng: f64,
    pub removed_bare: f64,
    pub removed_armed: f64,
    pub v_at_min: f64,
}
// ---------------------------------------------------------------------------------------------
// RUNG 57 — the two currencies, the credit, and rung 53's P5 transplanted
// ---------------------------------------------------------------------------------------------

/// Python's `f"{x:g}"`, which only ever tags a ladder row here. Six significant digits, trailing
/// zeros stripped, exponent form outside `[1e-4, 1e6)` — the `%g` rule, spelled out because Rust
/// has no `{:g}` and a tag that silently differed would make two runs' rows incomparable.
fn fmt_g(x: f64) -> String {
    if x == 0.0 {
        return "0".to_string();
    }
    let e = x.abs().log10().floor() as i32;
    if !(-5..6).contains(&e) {
        let s = format!("{:.5e}", x);
        return s;
    }
    let dec = (5 - e).max(0) as usize;
    let s = format!("{:.*}", dec, x);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// RUNG 58. The stator's credit as a PROFILE in `s` — armed minus bare, point by point — as a
/// linearly-interpolating reader. Both marches must be on the same `s` grid, which
/// `stator_march` guarantees (same `ds`, same `s_end`).
#[derive(Clone, Debug, PartialEq)]
pub struct CreditProfile {
    xs: Vec<f64>,
    ys: Vec<f64>,
}

impl CreditProfile {
    /// **THE ASSERT COMES AFTER THE ZIP, AS IT DOES IN PYTHON.** `zip` truncates silently, so a
    /// short march would build a short `ys` and only then be refused; keeping the order keeps the
    /// failure identical.
    pub fn new(prof_bare: &[(f64, f64)], prof_armed: &[(f64, f64)]) -> Self {
        let xs: Vec<f64> = prof_bare.iter().map(|&(a, _)| a).collect();
        let ys: Vec<f64> = prof_bare
            .iter()
            .zip(prof_armed.iter())
            .map(|(&(_, a), &(_, b))| b - a)
            .collect();
        assert!(prof_bare.len() == prof_armed.len(),
                "rung-58 credit profile needs the two marches on ONE grid; one of them broke out \
                 of the loop early (an off-map guard), so they cannot be differenced.");
        CreditProfile { xs, ys }
    }

    pub fn at(&self, s: f64) -> f64 {
        let (xs, ys) = (&self.xs, &self.ys);
        if s <= xs[0] {
            return ys[0];
        }
        if s >= xs[xs.len() - 1] {
            return ys[ys.len() - 1];
        }
        for i in 0..xs.len() - 1 {
            if xs[i] <= s && s <= xs[i + 1] {
                let t = (s - xs[i]) / (xs[i + 1] - xs[i]);
                return ys[i] + t * (ys[i + 1] - ys[i]);
            }
        }
        ys[ys.len() - 1]
    }
}

fn phi_of(p: &FuelPoint, spool: Spool) -> f64 {
    match spool {
        Spool::Lp => p.phi_lp,
        Spool::Hp => p.phi_hp,
    }
}

impl ScheduledStatorCore {
    /// BOTH rung-53 currencies, per spool, minimised over a trajectory, with the wall read at the
    /// LIVE setting:
    ///
    /// ```text
    /// phi-margin        M_phi = phi_op - phi_surge(v)      [the wall MOVES with v]
    /// incidence margin  M_i   = T_c - tan_beta1(phi_op, v) [the wall is the METAL]
    /// ```
    ///
    /// `v_of` defaults to THIS machine's own setting; a caller may pass one to read a trajectory
    /// against a DIFFERENT machine's wall (the floor-only isolation leg). **No shipped caller in
    /// rungs 57–63 passes one** — established by grepping the 16 `_read(` call sites, not by
    /// running anything — so the ported parameter is counted through
    /// [`Census::read_foreign_v_of`] and gated at zero in `slice_v_smoke.rs` section K, rather
    /// than left absent or asserted inert.
    pub fn read(
        &self, traj: &[FuelPoint], v_of: Option<&dyn Fn(Spool, &FuelPoint) -> f64>,
    ) -> StatorRead {
        if v_of.is_some() {
            bump(&READ_FOREIGN_VOF);
        }
        let one = |spool: Spool| -> ReadRow {
            let cmap = self.design_map(spool);
            assert!(cmap.phi_surge > 0.0,
                    "rung-57 needs the rung-36 floor on this map as its incidence anchor: build \
                     it with .with_phi_surge(phi_surge).");
            let t_c = cmap.tan_beta1_crit();
            let mut m_phi = f64::INFINITY;
            let mut m_i = f64::INFINITY;
            let mut row: Option<AtRow> = None;
            for p in traj {
                let v = match v_of {
                    Some(f) => f(spool, p),
                    None => self.v_of(spool, p.nu_lp, p.nu_hp, None),
                };
                let phi = phi_of(p, spool);
                let a = phi - cmap.phi_surge / (1.0 + v * cmap.phi_surge);
                let b = t_c - (1.0 / phi - v);
                // FIRST-STRICT, as Python's `if b < m_i` is: on a tie the EARLIER point keeps the
                // row, and the row feeds the reported `s_at_min` / `v_at_min`.
                if b < m_i {
                    m_i = b;
                    row = Some(AtRow { s: p.s, phi, v, nu_lp: p.nu_lp, nu_hp: p.nu_hp });
                }
                // Python's `min(m_phi, a)` returns the FIRST argument on a tie.
                if a < m_phi {
                    m_phi = a;
                }
            }
            let mut min_phi = phi_of(&traj[0], spool);
            for p in &traj[1..] {
                let x = phi_of(p, spool);
                if x < min_phi {
                    min_phi = x;
                }
            }
            ReadRow { m_phi, m_i, t_c, at: row, min_phi }
        };
        StatorRead { lp: one(Spool::Lp), hp: one(Spool::Hp), npts: traj.len() }
    }

    /// RUNG 57's reading instrument: both surge currencies, per spool, minimised over a marched
    /// accel ramp, against the wall THIS machine's stators actually put there.
    pub fn stator_transient_margin(
        &self, flight: &FlightCondition, ramp: &Ramp,
    ) -> TransientMargin {
        let (traj, nu0) = self.stator_march(flight, ramp, None, &StatorLeg::default());
        TransientMargin {
            read: self.read(&traj, None),
            nu0_lp: nu0.0,
            nu0_hp: nu0.1,
            r: ramp.r,
        }
    }

    /// THE FINDING (rung 57). March BARE and ARMED and split the incidence credit into
    ///
    /// ```text
    /// pointwise  the FLOOR channel alone -- the BARE trajectory read against THIS machine's
    ///            wall. Tautological by construction, and that is the point: it is the reference
    ///            the path term is measured against.
    /// net        the real credit, ARMED trajectory against ARMED wall.
    /// erosion    1 - net/pointwise -- the share the lever's own WORK channel eats by pushing the
    ///            running line down as it lowers the wall.
    /// ```
    ///
    /// For a CONSTANT setting `pointwise` is EXACTLY `v`, so nothing is estimated and `erosion` is
    /// a clean floor-vs-work split; rung 53's design-point closed form predicts the surviving
    /// share as `1/(2+l)`. FOR A SCHEDULE IT IS NOT THAT QUANTITY, and
    /// [`pointwise_exact`](StatorCredit::pointwise_exact) says so — use
    /// [`credit_decomposition`](Self::credit_decomposition) there.
    pub fn stator_credit(
        &self, flight: &FlightCondition, ramp: &Ramp, spool: Spool,
    ) -> StatorCredit {
        let bare = self.bare();
        let leg = StatorLeg::default();
        let (t_bare, nu0_b) = bare.stator_march(flight, ramp, None, &leg);
        let (t_armed, nu0_a) = self.stator_march(flight, ramp, None, &leg);
        let base = *bare.read(&t_bare, None).spool(spool);
        // BARE trajectory, ARMED wall.
        let pw = *self.read(&t_bare, None).spool(spool);
        let net = *self.read(&t_armed, None).spool(spool);
        let c_net = net.m_i - base.m_i;
        let c_pw = pw.m_i - base.m_i;
        let cmap = self.design_map(spool);
        let a = self.arming();
        let exact = match spool {
            Spool::Lp => a.sched_lp.is_none(),
            Spool::Hp => a.sched_hp.is_none(),
        };
        StatorCredit {
            spool,
            r: ramp.r,
            bare: base.m_i,
            armed: net.m_i,
            pointwise: pw.m_i,
            credit: c_net,
            credit_pointwise: c_pw,
            pointwise_exact: exact,
            erosion: if c_pw != 0.0 { 1.0 - c_net / c_pw } else { f64::NAN },
            closed_form: 1.0 / (2.0 + cmap.l),
            v_at_min: net.at().v,
            s_at_min: net.at().s,
            s_at_min_bare: base.at().s,
            nu0_bare: nu0_b.0,
            nu0_armed: nu0_a.0,
            min_phi_bare: base.min_phi,
            min_phi_armed: net.min_phi,
            m_phi_bare: base.m_phi,
            m_phi_armed: net.m_phi,
        }
    }

    /// WHERE a state-fed schedule's credit is delivered. Three legs on one ramp — START-ONLY (the
    /// head start a schedule already closed at idle has before `s = 0`), RAMP-ONLY, and FULL.
    ///
    /// `FULL/RAMP-ONLY` below 1 is the schedule's SELF-CANCELLATION: closing the stators raises
    /// the speed the machine sits at for the same power, the schedule reads that higher speed and
    /// opens back up. It is the one thing a constant setting cannot do.
    pub fn credit_decomposition(
        &self, flight: &FlightCondition, ramp: &Ramp, spool: Spool,
    ) -> CreditDecomposition {
        assert!(self.arming().is_armed(),
                "rung-57 credit_decomposition needs an armed machine to decompose.");
        let bare = self.bare();
        let leg = StatorLeg::default();
        let (t_bare, nu0_b) = bare.stator_march(flight, ramp, None, &leg);
        let base = bare.read(&t_bare, None).spool(spool).m_i;
        let eq = self.fuel.inner.equilibrium(flight, ramp.tt4_lo);
        let nu0_a = (eq.nu_lp, eq.nu_hp);
        let (t_start, _) = bare.stator_march(flight, ramp, Some(nu0_a), &leg);
        let (t_ramp, _) = self.stator_march(flight, ramp, Some(nu0_b), &leg);
        let (t_full, _) = self.stator_march(flight, ramp, Some(nu0_a), &leg);
        let start = bare.read(&t_start, None).spool(spool).m_i - base;
        let ramp_only = self.read(&t_ramp, None).spool(spool).m_i - base;
        let full = self.read(&t_full, None).spool(spool).m_i - base;
        CreditDecomposition {
            spool,
            r: ramp.r,
            bare: base,
            start,
            ramp: ramp_only,
            full,
            share_start: if full != 0.0 { start / full } else { f64::NAN },
            share_ramp: if full != 0.0 { ramp_only / full } else { f64::NAN },
            self_cancel: if ramp_only != 0.0 { full / ramp_only } else { f64::NAN },
            nu0_bare: nu0_b.0,
            nu0_armed: nu0_a.0,
        }
    }

    /// RUNG 53's P5, on the TRANSIENT closure. Take a physical state off the bare march (its LP
    /// surge minimum), then toggle ONE spool's stator and re-close AT THAT SAME STATE.
    ///
    /// Must be called on the BARE machine — it builds both siblings itself. `state` supplies the
    /// toggle point instead of marching for it, which is REQUIRED for the eta-mediation control:
    /// the flat-eta and shaped-eta islands have different running lines, so each finding its OWN
    /// minimum would compare two toggles at two different states.
    pub fn arrow_toggle(
        &self, flight: &FlightCondition, ramp: &Ramp, v: f64, spool: Spool,
        state: Option<(f64, f64, f64)>,
    ) -> ArrowToggle {
        assert!(!self.arming().is_armed(),
                "rung-57 arrow_toggle is a FIXED-STATE toggle: call it on the BARE machine, it \
                 builds both siblings itself.");
        let (tt2, pt2, _) = self.fuel.inner.inlet(flight);
        let (state, s_at) = match state {
            Some(st) => (st, f64::NAN),
            None => {
                let (traj, _) = self.stator_march(flight, ramp, None, &StatorLeg::default());
                // Python's `min(traj, key=...)` — FIRST on a tie.
                let mut best = &traj[0];
                for p in &traj[1..] {
                    if p.phi_lp < best.phi_lp {
                        best = p;
                    }
                }
                ((best.nu_lp, best.nu_hp, best.mf), best.s)
            }
        };
        let a = self.fuel.close_fuel(state.0, state.1, state.2, tt2, pt2);
        let sib = match spool {
            Spool::Lp => self.at_stator(StatorArm::constant(v, 0.0)),
            Spool::Hp => self.at_stator(StatorArm::constant(0.0, v)),
        };
        let b = sib.fuel.close_fuel(state.0, state.1, state.2, tt2, pt2);
        ArrowToggle {
            spool,
            v,
            s: s_at,
            state,
            nu_lp: state.0,
            nu_hp: state.1,
            d_phi_lp: b.base.phi_lp - a.base.phi_lp,
            d_phi_hp: b.base.phi_hp - a.base.phi_hp,
            d_n_hp: b.base.n_hp - a.base.n_hp,
            d_tt25: b.base.tt25 - a.base.tt25,
            phi_lp: a.base.phi_lp,
            phi_hp: a.base.phi_hp,
        }
    }

    // --- RUNG 58: the COMPOSITE ----------------------------------------------------------------

    /// RUNG 58. The armed leg's ENGAGEMENT residual `g(s)`, evaluated at the SCHEDULED fuel on the
    /// marched states: `g > 0` exactly when the leg must cut, one sign convention for all four
    /// legs.
    ///
    /// WHY IT EXISTS. `mf < mf_sched` can only locate the engagement to a GRID CELL, and the thing
    /// rung 58 has to measure — whether a wall-moving lever re-times a point-moving one — is two
    /// parts in a thousand. `g` is CONTINUOUS and the march is bit-identical to the unclipped one
    /// up to its first crossing, so interpolating it is exact there.
    pub fn leg_residual(
        &self, flight: &FlightCondition, traj: &[FuelPoint], leg: &StatorLeg<'_>,
    ) -> Vec<(f64, f64)> {
        leg.one();
        let pi_b = self.fuel.inner.inner.base.pi_b;
        let mut out = Vec::with_capacity(traj.len());
        for p in traj {
            let i = self.fuel.instant_fuel(flight, p.nu_lp, p.nu_hp, p.mf_sched);
            let g = if let Some(accel) = leg.accel {
                p.mf_sched - accel.cap(i.base.close.n_hp, i.base.close.pt4 / pi_b)
            } else if let Some(floor) = leg.surge.as_ref() {
                // RUNG 60: an incidence floor is resolved to the phi floor it IS at the live
                // setting, so `g` keeps ONE sign convention across all four legs. A rung-49
                // SurgeLimiter passes through unchanged -- bit-for-bit.
                let r = self.fuel
                    .resolve_floor(floor, p.nu_lp, p.nu_hp)
                    .unwrap_or_else(|e| panic!("{}", e.0));
                r.phi_lim - r.read(&i)
            } else {
                i.base.tt4 - leg.tt4_max.expect("one() proved a leg is armed")
            };
            out.push((p.s, g));
        }
        out
    }

    /// Sub-grid engagement time: the linearly-interpolated first upward zero of `g`, or NaN.
    pub fn s_eng(residual: &[(f64, f64)]) -> f64 {
        for w in residual.windows(2) {
            let ((s0, g0), (s1, g1)) = (w[0], w[1]);
            if g0 <= 0.0 && 0.0 < g1 {
                return s0 + (s1 - s0) * (0.0 - g0) / (g1 - g0);
            }
        }
        f64::NAN
    }

    /// RUNG 58. The incidence minimum, PARABOLA-refined off the `ds` grid.
    ///
    /// Rung 57 read `M_i` at grid points because its findings were per-trajectory levels. Rung
    /// 58's mechanism is the RELOCATION of that minimum and the setting the schedule commands
    /// THERE, and the relocation it leans on is one or two cells — so the argmin and `v` at it are
    /// both quantized by `ds` unless they are interpolated.
    pub fn refine_min(&self, traj: &[FuelPoint], spool: Spool) -> (f64, f64, f64, f64, f64) {
        let cmap = self.design_map(spool);
        let t_c = cmap.tan_beta1_crit();
        let ys: Vec<f64> = traj
            .iter()
            .map(|p| t_c - (1.0 / phi_of(p, spool) - self.v_of(spool, p.nu_lp, p.nu_hp, None)))
            .collect();
        // Python's `min(range(len(ys)), key=...)` — FIRST index on a tie.
        let mut j = 0usize;
        for k in 1..ys.len() {
            if ys[k] < ys[j] {
                j = k;
            }
        }
        if !(0 < j && j < ys.len() - 1) {
            bump(&REFINE_EDGE);
            let v = self.v_of(spool, traj[j].nu_lp, traj[j].nu_hp, None);
            return (traj[j].s, ys[j], traj[j].s, 0.0, v);
        }
        let (y0, y1, y2) = (ys[j - 1], ys[j], ys[j + 1]);
        let den = y0 - 2.0 * y1 + y2;
        let t = if den != 0.0 {
            0.5 * (y0 - y2) / den
        } else {
            bump(&REFINE_FLAT);
            0.0
        };
        let h = traj[j + 1].s - traj[j].s;
        let (a, b, w) = if t >= 0.0 {
            (&traj[j], &traj[j + 1], t)
        } else {
            (&traj[j - 1], &traj[j], 1.0 + t)
        };
        let nl = a.nu_lp + (b.nu_lp - a.nu_lp) * w;
        let nh = a.nu_hp + (b.nu_hp - a.nu_hp) * w;
        (traj[j].s + t * h, y1 - 0.25 * (y0 - y2) * t, traj[j].s, t, self.v_of(spool, nl, nh, None))
    }

    /// One cell of the composite: march, read both currencies, refine the minimum, and carry the
    /// DEFLATION EXCLUSION (`fuel_removed`, both `nu_*_end`).
    pub fn cell(
        &self, flight: &FlightCondition, ramp: &Ramp, spool: Spool, leg: &StatorLeg<'_>,
    ) -> CellRead {
        let (traj, nu0) = self.stator_march(flight, ramp, None, leg);
        let d = *self.read(&traj, None).spool(spool);
        let (s_ref, m_i_ref, s_grid_ref, _cells, v_ref) = self.refine_min(&traj, spool);
        let armed = leg.any();
        let mut removed = 0.0;
        for i in 1..traj.len() {
            let hh = traj[i].s - traj[i - 1].s;
            removed += 0.5 * hh
                * ((traj[i - 1].mf_sched - traj[i - 1].mf) + (traj[i].mf_sched - traj[i].mf));
        }
        let cmap = self.design_map(spool);
        let t_c = cmap.tan_beta1_crit();
        let prof: Vec<(f64, f64)> = traj
            .iter()
            .map(|p| {
                (p.s,
                 t_c - (1.0 / phi_of(p, spool) - self.v_of(spool, p.nu_lp, p.nu_hp, None)))
            })
            .collect();
        let mut tt4_peak = traj[0].tt4;
        for p in &traj[1..] {
            if p.tt4 > tt4_peak {
                tt4_peak = p.tt4;
            }
        }
        let s_eng = if armed {
            Self::s_eng(&self.leg_residual(flight, &traj, leg))
        } else {
            f64::NAN
        };
        let _ = s_grid_ref;
        CellRead {
            m_i: m_i_ref,
            m_i_grid: d.m_i,
            m_phi: d.m_phi,
            s: s_ref,
            prof,
            v: v_ref,
            s_grid: d.at().s,
            min_phi: d.min_phi,
            nu0: nu0.0,
            nu_lp_end: traj[traj.len() - 1].nu_lp,
            nu_hp_end: traj[traj.len() - 1].nu_hp,
            tt4_peak,
            fuel_removed: removed,
            s_eng,
            npts: traj.len(),
        }
    }

    /// THE RUNG (58). The stator lever and ONE fuel-side min-select leg on ONE plant — four cells
    /// and their MIXED SECOND DIFFERENCE:
    ///
    /// ```text
    /// interaction  =  [M_i(both) - M_i(fuel)] - [M_i(stator) - M_i(neither)]
    /// ```
    ///
    /// THE CURRENCY IS `M_i`, NOT `M_phi`, and that is a finding rather than a convention. `M_i`'s
    /// wall is the METAL — one number, bit-identical in all four cells — while `M_phi`'s wall
    /// MOVES with the stator, so differencing four cells in it crosses two walls and the
    /// non-additivity would be a coordinate artifact. `m_phi` is reported per cell and never
    /// differenced.
    ///
    /// THE FUEL LEG MUST BE ONE OBJECT, DERIVED ONCE, AND PASSED IN — so that a leg which differed
    /// between cells could never make the second difference isolate nothing. That discipline
    /// stands; rung 59 corrects only its stated REASON.
    pub fn composite_credit(
        &self, flight: &FlightCondition, ramp: &Ramp, spool: Spool, leg: &StatorLeg<'_>,
    ) -> CompositeCredit {
        let kind = leg.one();
        assert!(self.arming().is_armed(),
                "rung-58 composite_credit differences an ARMED stator against its own bare \
                 sibling -- call it on the machine carrying the stator leg.");
        let bare = self.bare();
        let none = StatorLeg::default();
        let cells = CompositeCells {
            neither: bare.cell(flight, ramp, spool, &none),
            stator: self.cell(flight, ramp, spool, &none),
            fuel: bare.cell(flight, ramp, spool, leg),
            both: self.cell(flight, ramp, spool, leg),
        };
        let c_bare = cells.stator.m_i - cells.neither.m_i;
        let c_fuel = cells.both.m_i - cells.fuel.m_i;
        let d_i = c_fuel - c_bare;
        let (vb, va) = (cells.stator.v, cells.both.v);
        // THE MECHANISM, predicted from the two FUEL-LEG-FREE marches alone. The stator's credit
        // is a PROFILE in `s`, not a scalar; the fuel leg does not change that profile, it changes
        // WHICH POINT of it is read. So re-reading the no-leg profile at the RELOCATED minimum
        // must reproduce the interaction -- from two trajectories that never saw the leg.
        let prof = CreditProfile::new(&cells.neither.prof, &cells.stator.prof);
        let p_bare = prof.at(cells.neither.s);
        let p_fuel = prof.at(cells.both.s);
        CompositeCredit {
            predicted: p_fuel - p_bare,
            profile_bare: p_bare,
            profile_fuel: p_fuel,
            spool,
            r: ramp.r,
            ds: ramp.ds,
            leg: kind,
            credit_bare: c_bare,
            credit_fuel: c_fuel,
            interaction: d_i,
            share: if c_bare != 0.0 { d_i / c_bare } else { f64::NAN },
            v_bare: vb,
            v_fuel: va,
            v_ratio: if vb != 0.0 { va / vb } else { f64::NAN },
            relocation: cells.both.s - cells.stator.s,
            relocation_bare: cells.fuel.s - cells.neither.s,
            leg_cost_bare: cells.fuel.nu_hp_end - cells.neither.nu_hp_end,
            leg_cost_armed: cells.both.nu_hp_end - cells.stator.nu_hp_end,
            fuel_removed_bare: cells.fuel.fuel_removed,
            fuel_removed_armed: cells.both.fuel_removed,
            cells,
        }
    }

    /// RUNG 58's CONVERSE reading: does the wall-moving lever re-time the point-moving one?
    ///
    /// Sub-grid engagement time on the BARE and the ARMED machine, on BOTH the limited march and
    /// the unlimited one (where `g` is defined everywhere and no clip has yet perturbed the
    /// states). This is the half `composite_credit` cannot see: the credit is a property of the
    /// stator, `s_eng` is a property of the fuel leg, and the rung's headline is that the
    /// influence runs ONE WAY between them.
    pub fn engagement_shift(
        &self, flight: &FlightCondition, ramp: &Ramp, leg: &StatorLeg<'_>,
    ) -> EngagementShift {
        let kind = leg.one();
        assert!(self.arming().is_armed(),
                "rung-58 engagement_shift needs an ARMED stator to shift anything.");
        let bare = self.bare();
        let none = StatorLeg::default();
        let mut out = [0.0f64; 4];
        for (mi, mach) in [&bare, self].into_iter().enumerate() {
            for (hi, armed_leg) in [leg, &none].into_iter().enumerate() {
                let (traj, _) = mach.stator_march(flight, ramp, None, armed_leg);
                out[mi * 2 + hi] = Self::s_eng(&mach.leg_residual(flight, &traj, leg));
            }
        }
        let (bare_limited, bare_dormant) = (out[0], out[1]);
        let (armed_limited, armed_dormant) = (out[2], out[3]);
        let d_lim = armed_limited - bare_limited;
        let d_dor = armed_dormant - bare_dormant;
        EngagementShift {
            r: ramp.r,
            ds: ramp.ds,
            leg: kind,
            bare_limited,
            bare_dormant,
            armed_limited,
            armed_dormant,
            d_limited: d_lim,
            d_dormant: d_dor,
            rel_limited: d_lim / bare_limited,
            rel_dormant: d_dor / bare_dormant,
        }
    }

    /// RUNG 58's MECHANISM sweep. Each stator leg is armed on a SIBLING of this machine (same
    /// hardware, same design references) and run through [`composite_credit`](Self::
    /// composite_credit) against the SAME fuel-leg object.
    ///
    /// A CONSTANT setting has no state-feed, so if the interaction is the relocation acting
    /// THROUGH the schedule's state-feed, sweeping the schedule's knee must move the interaction
    /// while the constant legs sit at a floor. Called on the bare machine — it builds every
    /// sibling itself.
    pub fn interaction_sweep(
        &self, flight: &FlightCondition, ramp: &Ramp, legs: &[(String, StatorArm)], spool: Spool,
        leg: &StatorLeg<'_>,
    ) -> Vec<InteractionRow> {
        assert!(!self.arming().is_armed(),
                "rung-58 interaction_sweep builds every stator sibling itself: call it on the \
                 BARE machine so no leg can inherit a setting it did not declare.");
        legs.iter()
            .map(|(tag, kw)| {
                let d = self.at_stator(*kw).composite_credit(flight, ramp, spool, leg);
                InteractionRow {
                    tag: tag.clone(),
                    credit_bare: d.credit_bare,
                    credit_fuel: d.credit_fuel,
                    interaction: d.interaction,
                    share: d.share,
                    v_bare: d.v_bare,
                    v_fuel: d.v_fuel,
                    v_ratio: d.v_ratio,
                    relocation: d.relocation,
                    leg_cost_bare: d.leg_cost_bare,
                    leg_cost_armed: d.leg_cost_armed,
                }
            })
            .collect()
    }

    // --- RUNG 59: the MATCHED schedule ---------------------------------------------------------

    /// RUNG 59. The three factors `kappa_ss` is BUILT from, at one steady point.
    ///
    /// ```text
    /// kappa_ss  =  f * mdot/pt3  =  pi_b * f(Tt3,Tt4) * MFP_A4 / [(1+f)*sqrt(Tt4)]
    /// ```
    ///
    /// `A4` is CHOKED, so the corrected group is hardware and nothing the stators do can reach it;
    /// `Tt3` is pinned by the TWO SHAFT BALANCES, which are MAP-FREE with every throat choked.
    /// Hence `kappa_ss` is a function of `Tt4` ALONE — a schedule's ORDINATE cannot see a stator
    /// on EITHER spool, exactly. This reader returns the factors so the claim is checked rather
    /// than asserted.
    ///
    /// DOMAIN: a fully-choked machine on the CPG branch. Rung 33's unchoked nozzle branch is the
    /// named boundary; on a reacting gas `f` picks up composition dependence. Neither is claimed.
    pub fn proof_chain(&self, flight: &FlightCondition, tt4: f64) -> ProofChain {
        let eq = self.fuel.inner.equilibrium(flight, tt4);
        let pt3 = eq.close.pt4 / self.fuel.inner.inner.base.pi_b;
        ProofChain {
            tt4,
            tt25: eq.close.tt25,
            tt3: eq.close.tt3,
            f: eq.close.f,
            mfp: eq.close.mdot_air * (1.0 + eq.close.f) * powp(tt4, 0.5) / eq.close.pt4,
            ratio: eq.close.mdot_air / pt3,
            n_hp: eq.close.n_hp,
            nu_lp: eq.nu_lp,
            kappa: eq.close.f * eq.close.mdot_air / pt3,
        }
    }

    /// RUNG 59, FIRST HALF. Derive rung 48's `Wf/pt3` schedule on THIS (stator-armed) machine and
    /// on its bare sibling, and compare the two tables HALF BY HALF.
    ///
    /// ```text
    /// LP stator   n_H(Tt4) untouched (rung 39's ONE ARROW)  =>  BIT-IDENTICAL  =>  matching is
    ///             a NO-OP.
    /// HP stator   n_H(Tt4) moves                            =>  the SAME CURVE, RE-INDEXED.
    /// ```
    pub fn schedule_invariance(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, margin: f64, n: usize,
    ) -> ScheduleInvariance {
        assert!(margin >= 0.0, "rung-59 inherits rung 48's above-the-steady-line margin");
        let bare = self.bare();
        let l_bare = bare.fuel.accel_schedule(flight, tt4_lo, tt4_hi, margin, n);
        let l_matched = self.fuel.accel_schedule(flight, tt4_lo, tt4_hi, margin, n);
        let mut chain = Vec::with_capacity(n);
        for k in 0..n {
            let tt4 = tt4_lo + (tt4_hi - tt4_lo) * k as f64 / (n as f64 - 1.0);
            let a = bare.proof_chain(flight, tt4);
            let b = self.proof_chain(flight, tt4);
            chain.push(ChainRow {
                tt4,
                d_tt25: (b.tt25 - a.tt25) / a.tt25,
                d_tt3: (b.tt3 - a.tt3) / a.tt3,
                d_f: (b.f - a.f) / a.f,
                d_mfp: (b.mfp - a.mfp) / a.mfp,
                d_ratio: (b.ratio - a.ratio) / a.ratio,
                d_kappa: (b.kappa - a.kappa) / a.kappa,
                d_n_hp: (b.n_hp - a.n_hp) / a.n_hp,
                d_nu_lp: (b.nu_lp - a.nu_lp) / a.nu_lp,
            });
        }
        let d_ord = l_matched.kappa.iter().zip(l_bare.kappa.iter())
            .map(|(a, b)| (a - b).abs() / b)
            .fold(f64::NEG_INFINITY, f64::max);
        let d_abs = l_matched.n_h.iter().zip(l_bare.n_h.iter())
            .map(|(a, b)| (a - b).abs() / b)
            .fold(f64::NEG_INFINITY, f64::max);
        ScheduleInvariance {
            ordinate_identical: l_matched.kappa == l_bare.kappa,
            abscissa_identical: l_matched.n_h == l_bare.n_h,
            d_ordinate: d_ord,
            d_abscissa: d_abs,
            bare: l_bare,
            matched: l_matched,
            chain,
        }
    }

    /// RUNG 59's ISOLATION instrument: the ABSCISSA of one table carrying the ORDINATE of the
    /// other. Running it against the two real legs splits `delta_match` into the half that
    /// re-indexes and the half that re-values, with nothing else changed.
    pub fn synthetic_leg(index: &AccelSchedule, values: &AccelSchedule) -> AccelSchedule {
        assert!(index.margin == values.margin,
                "rung-59 splices two tables of ONE schedule margin -- a margin difference would \
                 reintroduce the very leg-change the splice exists to exclude.");
        AccelSchedule {
            margin: values.margin,
            n_h: index.n_h.clone(),
            kappa: values.kappa.clone(),
        }
    }

    /// RUNG 59's standing BLOCKER check. `AccelSchedule::cap` CLAMPS at both ends of its abscissa,
    /// so a leg consulted outside its own bracket is running on `kappa[0]` or `kappa[-1]` — the
    /// envelope edge, not the DERIVED shape. Rung 59 re-indexes that very abscissa, so this is
    /// exactly the artifact that could counterfeit the finding: audited, never assumed.
    pub fn clamp_audit(
        &self, flight: &FlightCondition, traj: &[FuelPoint], leg: &AccelSchedule,
    ) -> ClampAudit {
        let (lo, hi) = (leg.n_h[0], leg.n_h[leg.n_h.len() - 1]);
        let mut n_cut: Vec<f64> = Vec::new();
        let mut n_all: Vec<f64> = Vec::with_capacity(traj.len());
        for p in traj {
            let i = self.fuel.instant_fuel(flight, p.nu_lp, p.nu_hp, p.mf_sched);
            n_all.push(i.base.close.n_hp);
            if p.mf_sched - p.mf > 1e-15 {
                n_cut.push(i.base.close.n_hp);
            }
        }
        let n_min = n_all.iter().copied().fold(f64::INFINITY, f64::min);
        let n_max = n_all.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        ClampAudit {
            lo,
            hi,
            n_min,
            n_max,
            n_cuts: n_cut.len(),
            cut_lo: if n_cut.is_empty() {
                f64::NAN
            } else {
                n_cut.iter().copied().fold(f64::INFINITY, f64::min)
            },
            cut_hi: if n_cut.is_empty() {
                f64::NAN
            } else {
                n_cut.iter().copied().fold(f64::NEG_INFINITY, f64::max)
            },
            clamped: n_cut.iter().filter(|&&x| x < lo || x > hi).count(),
        }
    }

    /// THE RUNG (59). Rung 58's composite re-run with the fuel leg MATCHED to the plant it runs on
    /// — what a FADEC actually burns in — plus the splice that says which half of the table
    /// carries the difference.
    ///
    /// THE ALGEBRA IS EXACT AND IS THE WHOLE LICENSE FOR THIS RUNG. The matched leg is derived on
    /// the ARMED machine, so it is a no-op on the two BARE cells by construction. Therefore
    /// `dI_matched - dI_bare_leg = delta_match` with NO residual term: a FIRST difference on ONE
    /// machine, same stator, same grid, same `T_c`.
    pub fn matched_credit(
        &self, flight: &FlightCondition, ramp: &Ramp, margin: f64, spool: Spool, n: usize,
    ) -> MatchedCredit {
        assert!(self.arming().is_armed(),
                "rung-59 matched_credit differences an ARMED stator against its own bare \
                 sibling -- call it on the machine carrying the stator leg.");
        let bare = self.bare();
        let inv = self.schedule_invariance(flight, ramp.tt4_lo, ramp.tt4_hi, margin, n);
        let (l_b, l_a) = (inv.bare.clone(), inv.matched.clone());
        // ARMED index, BARE values / BARE index, ARMED values.
        let l_s = Self::synthetic_leg(&l_a, &l_b);
        let l_c = Self::synthetic_leg(&l_b, &l_a);

        let none = StatorLeg::default();
        let leg_b = StatorLeg { accel: Some(&l_b), ..Default::default() };
        let leg_a = StatorLeg { accel: Some(&l_a), ..Default::default() };
        let leg_s = StatorLeg { accel: Some(&l_s), ..Default::default() };
        let leg_c = StatorLeg { accel: Some(&l_c), ..Default::default() };
        let cells = MatchedCells {
            neither: bare.cell(flight, ramp, spool, &none),
            stator: self.cell(flight, ramp, spool, &none),
            fuel: bare.cell(flight, ramp, spool, &leg_b),
            both_bare_leg: self.cell(flight, ramp, spool, &leg_b),
            both_matched: self.cell(flight, ramp, spool, &leg_a),
            both_reindexed: self.cell(flight, ramp, spool, &leg_s),
            both_revalued: self.cell(flight, ramp, spool, &leg_c),
        };
        // THE BLOCKER, on every cell that actually consults a leg.
        let mut audits: Vec<ClampAudit> = Vec::with_capacity(3);
        for (tag, leg, mach) in [("fuel", &l_b, &bare), ("both_bare_leg", &l_b, self),
                                 ("both_matched", &l_a, self)] {
            let armed = StatorLeg { accel: Some(leg), ..Default::default() };
            let (traj, _) = mach.stator_march(flight, ramp, None, &armed);
            let a = mach.clamp_audit(flight, &traj, leg);
            assert!(a.clamped == 0,
                    "rung-59: cell {:?} consults its schedule OUTSIDE the derived bracket \
                     [{:.6}, {:.6}] at {} of {} cutting points -- the cap is CLAMPED there, so \
                     the number is an envelope edge and not the derived shape. Widen the Tt4 band \
                     or lower the stator setting.",
                    tag, a.lo, a.hi, a.clamped, a.n_cuts);
            audits.push(a);
        }

        let credit_bare = cells.stator.m_i - cells.neither.m_i;
        let d_i_bare = (cells.both_bare_leg.m_i - cells.fuel.m_i) - credit_bare;
        let d_i_match = (cells.both_matched.m_i - cells.fuel.m_i) - credit_bare;
        let d_match = cells.both_matched.m_i - cells.both_bare_leg.m_i;
        let d_index = cells.both_reindexed.m_i - cells.both_bare_leg.m_i;
        let d_value = cells.both_revalued.m_i - cells.both_bare_leg.m_i;
        MatchedCredit {
            spool,
            r: ramp.r,
            ds: ramp.ds,
            margin,
            audit_fuel: audits[0],
            audit_both_bare_leg: audits[1],
            audit_both_matched: audits[2],
            ordinate_identical: inv.ordinate_identical,
            abscissa_identical: inv.abscissa_identical,
            d_ordinate: inv.d_ordinate,
            d_abscissa: inv.d_abscissa,
            credit_bare,
            interaction_bare_leg: d_i_bare,
            interaction_matched: d_i_match,
            delta_match: d_match,
            delta_index: d_index,
            delta_value: d_value,
            abscissa_share: if d_match != 0.0 { d_index / d_match } else { f64::NAN },
            ordinate_share: if d_match != 0.0 { d_value / d_match } else { f64::NAN },
            // rungs 43/45/49: the RAW second differences carry the claim; these are reported and
            // never leaned on -- `credit_bare` is a denominator from another regime.
            share_bare_leg: if credit_bare != 0.0 { d_i_bare / credit_bare } else { f64::NAN },
            share_matched: if credit_bare != 0.0 { d_i_match / credit_bare } else { f64::NAN },
            s_eng_bare_leg: cells.both_bare_leg.s_eng,
            s_eng_matched: cells.both_matched.s_eng,
            removed_bare_leg: cells.both_bare_leg.fuel_removed,
            removed_matched: cells.both_matched.fuel_removed,
            relocation: cells.both_matched.s - cells.both_bare_leg.s,
            cells,
        }
    }

    // --- RUNG 60: the MATCHED phi FLOOR --------------------------------------------------------

    /// RUNG 60. The two ways to MATCH a `phi` set point to a stator-armed machine, and the DERIVED
    /// gap between them — the proof that rung 58's proposed repair (*match the set point*) was
    /// never a well-posed instruction.
    ///
    /// ```text
    /// fixed phi-MARGIN off the moved wall     phi = (1+sm) / (T_c + v)
    /// fixed INCIDENCE                         phi = 1 / (T_c + v - M_B)
    /// ```
    ///
    /// with `M_B = T_c - 1/[(1+sm)*phi_surge]` the bare floor's own incidence margin. In the
    /// incidence coordinate they are apart by `v*sm/(1+sm)` exactly — zero new constants, and zero
    /// at either `v = 0` or `sm = 0`.
    pub fn matching_rules(&self, sm: f64, v: f64, spool: Spool) -> MatchingRules {
        let cmap = self.design_map(spool);
        let t_c = cmap.tan_beta1_crit();
        let phi_b = (1.0 + sm) * cmap.phi_surge;
        let m_b = t_c - 1.0 / phi_b;
        let phi_rel = (1.0 + sm) / (t_c + v);
        let phi_inc = 1.0 / (t_c + v - m_b);
        let gap = 1.0 / phi_inc - 1.0 / phi_rel;
        MatchingRules {
            sm,
            v,
            t_c,
            phi_bare: phi_b,
            m_bare: m_b,
            phi_rel,
            phi_inc,
            gap,
            gap_closed_form: v * sm / (1.0 + sm),
            residual: gap - v * sm / (1.0 + sm),
        }
    }

    /// RUNG 60. The ADMISSIBLE SET-POINT band of this machine, in BOTH coordinates.
    ///
    /// A floor is an instrument only strictly between two limits: it must sit BELOW the value at
    /// `s = 0`, or it binds from the start and the "acceleration" is a deceleration, and ABOVE the
    /// ramp's own minimum, or it never binds. The width of that band is the ramp's EXCURSION. ONE
    /// leg-free march — both coordinates come off it.
    pub fn band(&self, flight: &FlightCondition, ramp: &Ramp, spool: Spool) -> Band {
        let (traj, _) = self.stator_march(flight, ramp, None, &StatorLeg::default());
        let cmap = self.design_map(spool);
        let t_c = cmap.tan_beta1_crit();
        let phis: Vec<f64> = traj.iter().map(|p| phi_of(p, spool)).collect();
        let mis: Vec<f64> = traj
            .iter()
            .map(|p| t_c - (1.0 / phi_of(p, spool) - self.v_of(spool, p.nu_lp, p.nu_hp, None)))
            .collect();
        let phi_min = phis.iter().copied().fold(f64::INFINITY, f64::min);
        let m_min = mis.iter().copied().fold(f64::INFINITY, f64::min);
        Band {
            phi_0: phis[0],
            phi_min,
            phi_exc: phis[0] - phi_min,
            m_0: mis[0],
            m_min,
            m_exc: mis[0] - m_min,
            t_c,
            v_0: self.v_of(spool, traj[0].nu_lp, traj[0].nu_hp, None),
        }
    }

    /// RUNG 60, FIRST HALF. Can ONE set point be the same instrument on the bare and the statored
    /// machine — in `phi` (rung 49's coordinate) and in incidence (rung 60's)?
    ///
    /// Rung 58 measured the `phi` bands DISJOINT and stopped there. Re-referenced to incidence the
    /// wall no longer moves, so the bands can only be pushed apart by the lever's own CREDIT, and
    /// the gap collapses to an exact identity: `gap = CREDIT - EXCURSION`. So a fixed incidence
    /// set point is admissible IFF THE LEVER'S CREDIT IS SMALLER THAN THE RAMP'S OWN EXCURSION — a
    /// criterion, not a magnitude.
    pub fn set_point_bands(
        &self, flight: &FlightCondition, ramp: &Ramp, spool: Spool,
    ) -> SetPointBands {
        assert!(self.arming().is_armed(),
                "rung-60 set_point_bands compares an ARMED machine with its own bare sibling -- \
                 call it on the machine carrying the stator leg.");
        let b = self.bare().band(flight, ramp, spool);
        let a = self.band(flight, ramp, spool);
        // > 0 => DISJOINT (bare band above armed) / (armed band above bare)
        let gap_phi = b.phi_min - a.phi_0;
        let gap_m = a.m_min - b.m_0;
        let credit = a.m_min - b.m_min;
        let exc = b.m_exc;
        SetPointBands {
            spool,
            r: ramp.r,
            ds: ramp.ds,
            bare: b,
            armed: a,
            gap_phi,
            gap_m,
            gap_phi_bands: gap_phi / b.phi_exc.min(a.phi_exc),
            gap_m_bands: gap_m / b.m_exc.min(a.m_exc),
            credit,
            excursion: exc,
            criterion: credit - exc,
            identity_residual: (credit - exc) - gap_m,
            phi_admissible: gap_phi < 0.0,
            m_admissible: gap_m < 0.0,
            overlap_lo: b.m_min.max(a.m_min),
            overlap_hi: b.m_0.min(a.m_0),
        }
    }

    /// RUNG 60. The threshold `credit < excursion` walked until it is CROSSED — over a ladder of
    /// stator legs at fixed ramp rate, or over ramp rate at a fixed leg.
    ///
    /// The two axes are not equivalent, and that is the finding: the CREDIT is rung 57's
    /// clock-free number and the EXCURSION is the ramp's, so the threshold is crossed by the RAMP
    /// with the lever standing still. Called on the BARE machine, which builds every sibling
    /// itself.
    pub fn composability_ladder(
        &self, flight: &FlightCondition, ramp: &Ramp, axis: LadderAxis<'_>, spool: Spool,
    ) -> Vec<LadderRow> {
        assert!(!self.arming().is_armed(),
                "rung-60 composability_ladder builds every stator sibling itself: call it on the \
                 BARE machine so no leg can inherit a setting it did not declare.");
        let rows: Vec<(String, StatorArm, f64)> = match axis {
            LadderAxis::Legs(legs) => {
                legs.iter().map(|(t, k)| (t.clone(), *k, ramp.r)).collect()
            }
            LadderAxis::Rates(rates) => rates
                .iter()
                .map(|&(x, k)| (format!("r={}", fmt_g(x)), k, x))
                .collect(),
        };
        rows.into_iter()
            .map(|(tag, kw, rr)| {
                let d = self.at_stator(kw).set_point_bands(flight, &ramp.with_r(rr), spool);
                LadderRow {
                    tag,
                    r: rr,
                    credit: d.credit,
                    excursion: d.excursion,
                    criterion: d.criterion,
                    gap_m: d.gap_m,
                    gap_m_bands: d.gap_m_bands,
                    gap_phi: d.gap_phi,
                    gap_phi_bands: d.gap_phi_bands,
                    m_admissible: d.m_admissible,
                    phi_admissible: d.phi_admissible,
                }
            })
            .collect()
    }

    /// RUNG 60's BLOCKER check, and the artifact most likely to counterfeit this rung — rung 59's
    /// [`clamp_audit`](Self::clamp_audit) one ladder on.
    ///
    /// A floor that BINDS holds its own coordinate AT the set point, so that cell's minimum is the
    /// SET POINT and not the march. All three degenerate regimes are named: `pinned` (the
    /// tautology), `dormant` (the leg removed no fuel at all), and `from_zero` (the leg CUTS but
    /// has no upward crossing at all).
    pub fn pin_audit(&self, cell: &CellRead, floor: &Floor, spool: Spool) -> PinAudit {
        let cmap = self.design_map(spool);
        let t_c = cmap.tan_beta1_crit();
        let m_set = match floor {
            // the floor IS in the currency
            Floor::Incidence(i) => i.m_lim,
            Floor::Phi(s) => t_c - (1.0 / s.phi_lim - cell.v),
        };
        let res = cell.m_i - m_set;
        let dormant = cell.fuel_removed <= 0.0;
        // Python's `cell["s_eng"] != cell["s_eng"]` — a self-inequality NaN test, which is why
        // `s_eng` is an `f64` and not an `Option`.
        let from_zero = (cell.s_eng != cell.s_eng) && !dormant;
        PinAudit {
            m_set,
            m_min: cell.m_i,
            residual: res,
            pinned: res.abs() < 1e-9,
            dormant,
            from_zero,
            admissible: !(dormant || from_zero),
            s_eng: cell.s_eng,
            removed: cell.fuel_removed,
        }
    }

    /// THE RUNG (60). Rung 58's four-cell composite with a FLOOR leg — rung 49's `phi` floor, or
    /// rung 60's incidence floor, ONE object across all four cells — and the proof that NEITHER
    /// can carry it.
    ///
    /// THE THEOREM, derived before it is measured. `M_i = T_c - (1/phi - v)`. A floor that binds
    /// holds its own coordinate at the set point, so on every leg-armed cell the minimum IS the
    /// set point:
    ///
    /// ```text
    /// leg floors phi    M_i(both) - M_i(fuel)  =  [T_c - 1/phi_lim + v] - [.. + 0] = v
    /// leg floors M_i    M_i(both) - M_i(fuel)  =  m_lim - m_lim                   = 0
    /// ```
    ///
    /// so a `phi` floor reports the FULL POINTWISE credit with rung 57's erosion annihilated, and
    /// an incidence floor reports NO credit at all. Both are exact, neither is a measurement, and
    /// RE-REFERENCING THE LEG MOVES THE TAUTOLOGY RATHER THAN REMOVING IT. The gate is that the
    /// measurement meets [`pinned_prediction`](FloorComposite::pinned_prediction) at machine
    /// precision, which is the OPPOSITE of the usual gate and is the point.
    ///
    /// WHAT IS NOT TAUTOLOGICAL is the TIMING half: `s_eng` is a time, has no wall, and is pinned
    /// by nothing.
    pub fn floor_composite(
        &self, flight: &FlightCondition, ramp: &Ramp, floor: &Floor, spool: Spool,
    ) -> FloorComposite {
        assert!(self.arming().is_armed(),
                "rung-60 floor_composite differences an ARMED stator against its own bare \
                 sibling -- call it on the machine carrying the stator leg.");
        let bare = self.bare();
        let none = StatorLeg::default();
        let leg = StatorLeg { surge: Some(*floor), ..Default::default() };
        let cells = CompositeCells {
            neither: bare.cell(flight, ramp, spool, &none),
            stator: self.cell(flight, ramp, spool, &none),
            fuel: bare.cell(flight, ramp, spool, &leg),
            both: self.cell(flight, ramp, spool, &leg),
        };
        let audit_fuel = bare.pin_audit(&cells.fuel, floor, spool);
        let audit_both = self.pin_audit(&cells.both, floor, spool);
        let c_bare = cells.stator.m_i - cells.neither.m_i;
        let c_fuel = cells.both.m_i - cells.fuel.m_i;
        let is_inc = matches!(floor, Floor::Incidence(_));
        // THE DERIVED VALUE the tautology must take, per regime.
        let (regime, pred) = if audit_fuel.pinned && audit_both.pinned {
            (Regime::BothPinned, if is_inc { 0.0 } else { cells.both.v })
        } else if audit_both.dormant {
            // If the floor binds on the bare cell but the armed machine clears it, `both` is
            // bit-identical to `stator` and the difference is a property of the floor and one
            // leg-FREE march, with no armed-cell dynamics in it either.
            (Regime::ArmedClears, cells.stator.m_i - audit_fuel.m_set)
        } else {
            (Regime::Mixed, f64::NAN)
        };
        FloorComposite {
            spool,
            r: ramp.r,
            ds: ramp.ds,
            regime,
            floor: if is_inc { FloorKind::Incidence } else { FloorKind::Phi },
            admissible: audit_fuel.admissible && audit_both.admissible,
            credit_bare: c_bare,
            credit_fuel: c_fuel,
            interaction: c_fuel - c_bare,
            pinned_prediction: pred,
            pinned_residual: c_fuel - pred,
            s_eng_bare: cells.fuel.s_eng,
            s_eng_armed: cells.both.s_eng,
            d_s_eng: cells.both.s_eng - cells.fuel.s_eng,
            removed_bare: cells.fuel.fuel_removed,
            removed_armed: cells.both.fuel_removed,
            v_at_min: cells.both.v,
            audit_fuel,
            audit_both,
            cells,
        }
    }
}
