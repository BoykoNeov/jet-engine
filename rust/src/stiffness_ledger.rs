//! RUNG 77 — **THE STIFFNESS LEDGER.** `StiffnessLedgerTransient`, slice AH.
//!
//! Rung 76 § 3 found that writing rung 48's leg as a set-point solve multiplies its sensitivity
//! to every other state by `1/(1−c)`, and § 8 predicted that *every other set-point solve in this
//! family has one and it has never been read*. **This rung reads all three and REFUTES that
//! wording.** The implicit function theorem gives `dw*/dq = −G_q/G_w` for the accel (48),
//! governor (46) and phi (49) legs alike; substituting the accel leg's residual returns rung 76
//! § 3's identity exactly, with `1/(1−c) = 1/G_a'`. So `1/(1−c)` is not a gain a solve BUYS — it
//! is that leg's own residual slope, and it takes that form for ONE reason: **its set point is a
//! formula for its own actuator.** `Tt4_max` and `phi_lim` are CONSTANTS, so the other two legs
//! have a STIFFNESS and can never have a GAIN.
//!
//! # WHAT STEP 1 OF THIS SLICE ADDS AT THIS RUNG — **ONE RE-AIMED POINTER, AND IT CARRIES NOTHING**
//!
//! | | Python | slot | table |
//! |---|---|---|---|
//! | swap | `at_lever` | `LeverHooks::at_lever` | [`R77`] |
//!
//! That is the whole of it. Rung 77 adds no knob, no state and no plant code — it is a pure
//! reader, so its reduce is by CONSTRUCTION rather than by dispatch, and every march it runs is
//! `SensedCapTransient`'s bit-for-bit because not one of the parent's plant methods is overridden.
//!
//! # THE ONE CELL IS INVISIBLE TO VALUES AND VISIBLE ONLY TO POINTER IDENTITY
//!
//! Plan § 5.32 (ii), probe 8, measured what each `at_lever` in the chain copies forward:
//!
//! | rung | carries | NEW this rung |
//! |---|---|---|
//! | 74 | 2 | — |
//! | 75 | 5 | `_windup_law, _tau_t, _ic_cap` |
//! | 76 | 6 | `_cap_law` |
//! | **77** | **6** | **NONE** |
//! | 78 | 7 | `_gauge_k` |
//!
//! **Rung 77's `at_lever` is the FIRST in the entire chain that carries nothing new.** Its field
//! list is identical to rung 76's; its ONLY difference is the class it constructs — and Python's
//! own docstring says so outright: *"THE FIFTEENTH INSTANCE, and the thing not carried is the
//! CLASS."* In this port the class **is** the hooks table. So a defect at this cell cannot move a
//! copied field, and can only be a wrong table: **loud to pointer identity, silent to every value
//! gate in the crate.**
//!
//! [`r77_at_lever`]'s six `set` lines are therefore kept DELIBERATELY, not by inertia. They are
//! redundant on the VALUE — rung 76's body would copy the same six — and load-bearing on the
//! IDENTITY, because a body that forwards straight to its parent's is folded onto its parent's
//! address by the linker and the pointer gate then cannot tell the two rungs apart. That is slice
//! AG step 1's measured defect (`r76_shared_rig`'s note), live here again **by construction**.
//!
//! # AND THERE IS NO WIDTH TRIPWIRE TO CATCH A MISSED RE-AIM
//!
//! This slice is **0 ADD**: `TripleHooks` stays at 18 fields and `LeverHooks` is untouched
//! (§ 5.32 (ii), probes 1/6/7). Nothing in the crate goes red on an `E0063` if a pointer is left
//! aimed at rung 76's body — the machine simply runs the parent, returns this slice's own reduce-
//! arm answer, and every reduce gate in the crate goes on passing. The step-1 instrument is
//! therefore **function-pointer identity in BOTH directions** — the swap differs from its parent,
//! AND it is the body this module names — and it lives in `tests/slice_ah_cells.rs`.
//!
//! # THE `_b_state`/`_v_state` NEST — MEASURED HERE, DECIDED IN [`residual_gauge`]
//!
//! Slice AC booked a same-field nest of the two frozen-state carriers to this slice. It is real
//! and it is live at this rung: `leg_slopes` calls rung 76's `_c_at` from inside its own frozen
//! block (`engine.py:19829`, inside the freeze opened at `:19821`), **150 times over the rung-77
//! suite** (120 through `stiffness_ledger`, 30 directly). Rung 77 is one of five such sites across
//! the two rungs of this slice. The port's treatment of them, and the reason no value gate can
//! state it, is written once — at [`residual_gauge`](crate::residual_gauge) — because the guard
//! that carries it is shared.
//!
//! [`residual_gauge`]: crate::residual_gauge

use crate::bleed_transient::{LeverArm, LeverHooks};
use crate::demand_coordinate::{cap_free, cap_gov};
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, FuelTransientCore, FuelTransientHooks,
    PointExtra,
};
use crate::gas::Abort;
use crate::map::ComponentMap;
use crate::sensed_cap::{c_at, C_AT_REL};
use crate::shared_actuator::riding4;
use crate::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks};
use crate::three_loop::TripleHooks;
use crate::two_spool::{Spool, TwoSpoolEngine};
use crate::two_spool_transient::{MarchedBleed, MarchedStator, TwoSpoolTransientHooks};

// ---------------------------------------------------------------------------------------------
// THE CASCADE BUILDER
// ---------------------------------------------------------------------------------------------

/// Build a rung-77 object, so every sibling re-asserts the whole chain's guards.
///
/// `_ref_law` is set for rung 73's reason, inherited through the chain rather than copied; nothing
/// else is written, because this class declares **no** law of its own — it is a pure reader, and a
/// set that no gate could see is a line that looks like it is doing the `ref_law` job.
pub fn build_stiffness_ledger_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = crate::reference_split::build_split_family_cascade(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm,
        &R77_TWO, &R77_STATOR, &R77_FUEL, &R77, &R77_TRIPLE);
    if let ScheduledStatorTransient::Full(c) = &built {
        c.fuel.inner.ref_law.set(crate::applied_reference::REF_LAW_APPLIED);
    }
    built
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — five, and exactly ONE of them carries anything of this rung's own
// ---------------------------------------------------------------------------------------------

/// RUNG 77's lever table — ONE swap, `at_lever`, and the parent it must differ from is rung 76's.
///
/// The SIXTEENTH construction in this family and the FIFTEENTH instance of the sibling-constructor
/// trap. See the module header for why this one cell is the sharpest structural fact in the slice.
pub const R77: LeverHooks = LeverHooks {
    at_lever: r77_at_lever,
    ..crate::sensed_cap::R76
};

/// RUNG 77's `TwoSpoolTransientHooks` — **ZERO cells swapped**, an alias.
pub const R77_TWO: TwoSpoolTransientHooks = crate::sensed_cap::R76_TWO;

/// RUNG 77's fuel table — **ZERO cells swapped**, an alias.
///
/// Rung 76 re-aimed `integrate_fuel` to carry its own three refusals. This rung declares no law,
/// so it has none to add and the rung-76 body is what a rung-77 machine must take.
pub const R77_FUEL: FuelTransientHooks = crate::sensed_cap::R76_FUEL;

/// RUNG 77's stator table — **ZERO cells swapped**, an alias.
pub const R77_STATOR: StatorTransientHooks = crate::sensed_cap::R76_STATOR;

/// RUNG 77's third-loop table — **ZERO of rung 76's eighteen cells re-aimed**, an alias.
///
/// Written as an alias and NOT as an exhaustive literal, which is a deliberate departure from
/// [`R76_TRIPLE`](crate::sensed_cap::R76_TRIPLE)'s spelling. That literal exists because rung 76
/// re-aims two of the eighteen and a reader needs to see the sixteen that stayed put beside them.
/// Here there is nothing to hold apart: the exhaustive form would be eighteen lines asserting the
/// same thing the alias asserts in one, and the next reader to add a cell would have to edit both.
pub const R77_TRIPLE: TripleHooks = crate::sensed_cap::R76_TRIPLE;

// ---------------------------------------------------------------------------------------------
// THE ONE RE-AIMED BODY
// ---------------------------------------------------------------------------------------------

/// RUNG 77's `at_lever` — **rung 76's sibling constructor returning a RUNG-77 machine, and the
/// first in the chain that carries NOTHING NEW.**
///
/// The six `set` lines below are byte-for-byte rung 76's. That is the point: what this cell
/// changes is the TABLE the machine is built on, and nothing else. Hand back a rung-76 sibling
/// here and every reader in this module measures rung 76's plant — which is a legal plant, answers
/// every reduce gate, and is not this rung.
///
/// **DO NOT FACTOR THE SIX LINES ONTO THE PARENT.** Forwarding to
/// [`R76`](crate::sensed_cap::R76)`.at_lever` and then re-tabling is the same six values and a
/// different function: the linker folds a pass-through body onto its parent's address, and
/// `tests/slice_ah_cells.rs`'s identity gate — the only instrument this 0-ADD slice has — then
/// reads the two rungs as one. Slice AG measured exactly that on `r76_shared_rig`.
fn r77_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    let m = match build_stiffness_ledger_cascade(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    };
    m.fuel.inner.ref_law.set(core.fuel.inner.ref_law.get());
    m.fuel.inner.lag_coord.set(core.fuel.inner.lag_coord.get());
    m.fuel.inner.windup_law.set(core.fuel.inner.windup_law.get());
    m.fuel.inner.tau_t.set(core.fuel.inner.tau_t.get());
    m.fuel.inner.ic_cap.set(core.fuel.inner.ic_cap.get());
    m.fuel.inner.cap_law.set(core.fuel.inner.cap_law.get());
    m
}

// ---------------------------------------------------------------------------------------------
// STEP 2 — THE INSTRUMENT, THE THREE RESIDUALS, THE THREE SET POINTS, AND § 1 / § 2
// ---------------------------------------------------------------------------------------------
//
// This rung reads. Everything below consumes rung 76's plant unchanged — `cap_march`, `c_at`,
// `accel_for`, `cap_free`, `cap_gov` — and adds exactly one differencing primitive and one way of
// laying three set-point solves side by side.

/// `_slope_at`'s `rel` — **`_c_at`'s own step, duplicated ON PURPOSE and not factored.**
///
/// Python spells two independent defaults (`engine.py:19303` and `:19702`) and rung 77's docstring
/// says why they must agree: § 1's instrument check is `1 − G_a' == c`, two readings differenced at
/// the SAME step, so their roundoff cancels and the check tests the ALGEBRA rather than the
/// arithmetic. Reaching for [`C_AT_REL`](crate::sensed_cap::C_AT_REL) here would factor a
/// deliberate duplication away — the port's recorded COPY-vs-REDERIVATION rule — so the value is
/// re-typed and the binding between the two is asserted in `tests/slice_ah_laws.rs` instead, where
/// a later divergence is a red test and not a silently weaker instrument.
pub const SLOPE_AT_REL: f64 = 1e-6;

/// `dG/dw` at a point — **the ONLY differencing primitive in this rung**, `c_at`'s shape
/// generalised to any residual.
///
/// # THE `max` IS EXPRESSION-FIRST, AND `c_at`'s DOC COMMENT SAYS THIS SITE DOES NOT EXIST
///
/// `dw = rel * max(abs(w), 1e-9)` — argument 0 is an expression, so `w.abs().max(1e-9)` is the
/// WRONG spelling: Python's `max` seeds its fold at argument 0 and replaces only on a strict
/// comparison, so it returns `nan` for a NaN `w` where `f64::max` returns `1e-9`.
///
/// [`c_at`](crate::sensed_cap::c_at)'s doc calls its own `max(w, 1e-9)` *"the single
/// expression-first `1e-9` fold in the whole package"*. **Re-running slice AG step 1's census at
/// this step reproduces its 103-literal-first-of-268 exactly and finds FOUR such folds, not one**
/// — `engine.py:19308` (rung 76, ported), `:19708` (this function), `:20221`
/// (`residual_gauge::gauge_root`, ported at step 1 in the `f64::max` spelling and repaired at this
/// one) and `:21029` (rung 79, unported, booked to slice AI). Every one of those lines was in
/// `engine.py` when the census ran, so the claim was **false at birth** rather than decayed —
/// slice AG step 5's class, and this slice's own *a claim with an expiry date* arriving on the
/// sentence that asserted it had none. Corrected there, recorded here.
pub fn slope_at(
    big_g: &dyn Fn(f64) -> Result<f64, Abort>, w: f64, rel: f64,
) -> Result<f64, Abort> {
    // Python's `max(abs(w), 1e-9)` — the EXPRESSION is argument 0. See this function's doc.
    let aw = w.abs();
    let dw = rel * if 1e-9 > aw { 1e-9 } else { aw };
    Ok((big_g(w + dw)? - big_g(w - dw)?) / (2.0 * dw))
}

/// WHICH of the three set-point solves — Python's dict keys `"accel"`, `"gov"`, `"phi"`.
///
/// **THE DECLARATION ORDER IS LOAD-BEARING TWICE.** § 2 sorts legs by `abs(direct)` with Python's
/// STABLE `sorted`, so ties fall back to the iteration order `("accel", "gov", "phi")`; and it
/// returns `sorted(set(...))` of those tuples, which compares the STRINGS. Alphabetically
/// `"accel" < "gov" < "phi"`, so the derived [`Ord`] on this enum reproduces both — but only
/// because the two orders coincide, which is why it is said here rather than assumed.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Leg {
    Accel,
    Gov,
    Phi,
}

impl Leg {
    /// Python's `("accel", "gov", "phi")` — the iteration order of every loop in this rung.
    pub const ORDER: [Leg; 3] = [Leg::Accel, Leg::Gov, Leg::Phi];

    pub fn name(self) -> &'static str {
        match self {
            Leg::Accel => "accel",
            Leg::Gov => "gov",
            Leg::Phi => "phi",
        }
    }
}

/// The three residuals as closures, **with no solve** — Python's `_residuals`.
///
/// # THE SPLIT FROM [`legs`] IS NOT TIDINESS, IT IS THE LATE-BINDING TRAP
///
/// A residual closes over the plant and reads `_b_state`/`_v_state` **when it is CALLED**, not when
/// it is built — and a Rust closure holding `&FuelTransientCore` has exactly that property, because
/// the two frozen-state fields are `Cell`s on the core it borrows. So a reader that builds one
/// inside a frozen block and evaluates it after the guard has dropped measures the residual on the
/// plant with the valve LOOP CLOSED, whatever `q` it thought it was asking about.
///
/// **Python records that this shipped once.** § 2's first version did exactly that: both `q ± dq`
/// readings landed on the same closed-valve plant, `G_q` came back **identically zero**, and the
/// relative error against `direct` was a clean `1.000e+00` — a number with no noise in it, which is
/// what gave it away. Splitting the closures out makes the repair structural, and
/// [`set_point_gains`] rebuilds them inside each perturbed block for that reason alone.
pub struct Residuals<'a> {
    pub accel: Option<Box<dyn Fn(f64) -> Result<f64, Abort> + 'a>>,
    pub gov: Option<Box<dyn Fn(f64) -> Result<f64, Abort> + 'a>>,
    pub phi: Option<Box<dyn Fn(f64) -> Result<f64, Abort> + 'a>>,
}

impl<'a> Residuals<'a> {
    pub fn get(&self, leg: Leg) -> Option<&(dyn Fn(f64) -> Result<f64, Abort> + 'a)> {
        let b = match leg {
            Leg::Accel => &self.accel,
            Leg::Gov => &self.gov,
            Leg::Phi => &self.phi,
        };
        b.as_deref()
    }
}

/// RUNG 77's `_residuals` — the three legs' residuals, each armed only if its leg is.
///
/// ```text
/// accel (48)  G_a(w) = w − cap(w)             G_a' = 1 − c     DIMENSIONLESS
/// gov   (46)  G_g(w) = Tt4(w) − Tt4_max       G_g' = dTt4/dw   K per kg/s
/// phi   (49)  G_s(w) = phi_lim − phi_lp(w)    G_s' = −dphi/dw  phi per kg/s
/// ```
///
/// # THE PHI LEG READS ITS SPOOL AT BUILD TIME AND ITS LIMIT AT CALL TIME
///
/// Python does `k = surge.key()` **outside** the closure and `surge.phi_lim` **inside** it. `key()`
/// exists on rung 60's incidence floor too; `phi_lim` does not. So the only observable difference
/// between the two placements is WHEN a rung-60 object raises, and in Python that is at CALL time.
/// [`Floor::spool`](crate::fuel_transient::Floor::spool) is therefore read outside the closure and
/// [`Floor::phi`](crate::fuel_transient::Floor::phi) — whose panic IS that `AttributeError` —
/// inside it. `r78_cap_fuel` resolves both outside because its own source line does; the two
/// bodies are not factored together for that reason.
pub fn residuals<'a>(
    ft: &'a FuelTransientCore, flight: &'a FlightCondition, a: f64, h: f64,
    accel: Option<&'a AccelSchedule>, surge: Option<&'a Floor>, tt4_max: Option<f64>,
) -> Residuals<'a> {
    let pi_b = ft.inner.inner.base.pi_b;
    Residuals {
        accel: accel.map(|accel| {
            let f: Box<dyn Fn(f64) -> Result<f64, Abort> + 'a> = Box::new(move |w: f64| {
                let i = ft.try_instant_fuel(flight, a, h, w)?;
                Ok(w - accel.cap(i.base.close.n_hp, i.base.close.pt4 / pi_b))
            });
            f
        }),
        gov: tt4_max.map(|tt4_max| {
            let f: Box<dyn Fn(f64) -> Result<f64, Abort> + 'a> = Box::new(move |w: f64| {
                Ok(ft.try_instant_fuel(flight, a, h, w)?.base.tt4 - tt4_max)
            });
            f
        }),
        phi: surge.map(|surge| {
            // Python's `k = surge.key()`, OUTSIDE the closure.
            let spool = surge.spool();
            let f: Box<dyn Fn(f64) -> Result<f64, Abort> + 'a> = Box::new(move |w: f64| {
                let i = ft.try_instant_fuel(flight, a, h, w)?;
                let read = match spool {
                    Spool::Lp => i.base.close.phi_lp,
                    Spool::Hp => i.base.close.phi_hp,
                };
                Ok(surge.phi().phi_lim - read)
            });
            f
        }),
    }
}

/// ONE leg of [`legs`] — its residual, its own set point, its own normalisation, and whether it is
/// acting at this point.
pub struct LegReading<'a> {
    /// The residual itself. **Evaluate it inside the block it was built for** — [`Residuals`].
    pub g: Box<dyn Fn(f64) -> Result<f64, Abort> + 'a>,
    /// This leg's set point: the fuel at which its own `G` is zero.
    pub w: f64,
    /// This leg's OWN already-imposed scalar — `w` for the accel leg (whose set point IS a fuel,
    /// so the normalisation is the IDENTITY), `Tt4_max` for the governor, `phi_lim` for phi.
    /// **This rung adds no constant**, and that is the whole reason `scale` is not a knob.
    pub scale: f64,
    /// `G(mf_sched) > 0` — rung 76 § 1.3's switch guard, one level over. Positive means the leg
    /// must CUT. Ordering a DORMANT leg's gain against a live one's compares a limiter that is
    /// acting with one that is not.
    pub live: bool,
}

/// The three set-point solves, side by side and **unmixed** — Python's `_legs`.
pub struct Legs<'a> {
    pub accel: Option<LegReading<'a>>,
    pub gov: Option<LegReading<'a>>,
    pub phi: Option<LegReading<'a>>,
}

impl<'a> Legs<'a> {
    pub fn get(&self, leg: Leg) -> Option<&LegReading<'a>> {
        match leg {
            Leg::Accel => self.accel.as_ref(),
            Leg::Gov => self.gov.as_ref(),
            Leg::Phi => self.phi.as_ref(),
        }
    }

    /// Python indexes `legs["accel"]` and raises `KeyError` on a leg this arming did not build.
    /// Every reader in this rung arms all three, so a miss is a wiring defect and not a state.
    pub fn at(&self, leg: Leg) -> &LegReading<'a> {
        self.get(leg).unwrap_or_else(|| panic!(
            "rung-77 reads leg `{}` off `_legs`, which is Python's KeyError when this arming did \
             not build it. All three readers arm accel, gov and phi.", leg.name()))
    }
}

/// RUNG 77's `_legs` — **THE THREE SET-POINT SOLVES, SIDE BY SIDE AND UNMIXED.**
///
/// # IT IS NOT `LeverHooks::legs`, AND IT NEVER CAN BE
///
/// `_legs` is defined exactly twice in the whole 31-class ladder — rung 63 (`engine.py:9176`,
/// which IS [`LeverHooks::legs`](crate::bleed_transient::LeverHooks)) and rung 77 (`:19711`) — and
/// this signature drops `reference, Tt4_lo, Tt4_hi, r, s_settle, ds, spool` and adds `a, h,
/// mf_sched`. The two are INCOMPATIBLE, so this is a name REUSED and not an override: it is a free
/// function in this module, the cell keeps rung 63's body, and no class after rung 77 defines the
/// name at all. Plan § 5.32 (iii) item D, measured from the source rather than inherited.
///
/// # THE MINIMUM ONE LEVEL DOWN MUST NOT BE READ
///
/// `_cap_fuel` returns `min(accel, phi)`, and a ledger of legs that read it would report whichever
/// leg happened to bind and call it the other's slope — rung 76 § 1.3's own trap, which cost that
/// rung a sweep. So each residual is built from the SAME body `_cap_fuel` uses, ONE LEG AT A TIME,
/// and each set point comes from [`cap_free`](crate::demand_coordinate::cap_free) — the UNFLOORED
/// cap, so a DORMANT leg still has a slope. A floored cap would return `mf_sched` and this ledger
/// would silently become a ledger of the schedule.
pub fn legs<'a>(
    ft: &'a FuelTransientCore, flight: &'a FlightCondition, a: f64, h: f64, mf_sched: f64,
    accel: Option<&'a AccelSchedule>, surge: Option<&'a Floor>, tt4_max: Option<f64>,
) -> Result<Legs<'a>, Abort> {
    let r = residuals(ft, flight, a, h, accel, surge, tt4_max);
    let accel_leg = match (accel, r.accel) {
        (Some(accel), Some(g)) => {
            let wa = cap_free(&*g, mf_sched, &|| {
                ft.try_sched_fuel(flight, a, h, mf_sched, accel)
            })?;
            let live = g(mf_sched)? > 0.0;
            Some(LegReading { g, w: wa, scale: wa, live })
        }
        _ => None,
    };
    let gov_leg = match (tt4_max, r.gov) {
        (Some(tt4_max), Some(g)) => {
            let w = cap_gov(ft, flight, a, h, mf_sched, tt4_max)?;
            let live = g(mf_sched)? > 0.0;
            Some(LegReading { g, w, scale: tt4_max, live })
        }
        _ => None,
    };
    let phi_leg = match (surge, r.phi) {
        (Some(surge), Some(g)) => {
            let ws = cap_free(&*g, mf_sched, &|| {
                ft.try_surge_fuel(flight, a, h, mf_sched, surge)
            })?;
            let live = g(mf_sched)? > 0.0;
            Some(LegReading { g, w: ws, scale: surge.phi().phi_lim, live })
        }
        _ => None,
    };
    Ok(Legs { accel: accel_leg, gov: gov_leg, phi: phi_leg })
}

/// RUNG 77's `_ledger_march` — **one rig, one march, ACCEL-ARMED.**
///
/// Rung 76's [`cap_march`](crate::sensed_cap::cap_march) at this rung's own settings — `clip` ×
/// `sched` × `none` × `solve` — with the schedule built by
/// [`accel_for`](crate::sensed_cap::accel_for) **on the rig that will march it**, which is rung 76
/// § 7's trap whose cousin is a TABLE and not a knob. The `_lag_coord = "demand"` after the march
/// is a PLAIN assignment, exactly as Python spells it.
#[allow(clippy::too_many_arguments)]
pub fn ledger_march(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    sm: f64, taus: (f64, f64, f64, f64), r: f64, s_settle: f64, ds: f64, v_max: f64, inc: bool,
    margin: f64,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>, Vec<FuelPoint>, AccelSchedule) {
    let accel = crate::sensed_cap::accel_for(
        core, flight, tt4_lo, tt4_hi, sm, tt4_max, taus, v_max, inc, margin);
    let (m, surge, lag, traj) = crate::sensed_cap::cap_march(
        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc,
        crate::demand_coordinate::LAG_COORD_CLIP, crate::shared_actuator::REF_LAW_DEFAULT,
        crate::anti_windup::WINDUP_LAW_NONE, None, crate::sensed_cap::CAP_LAW_SOLVE, &accel, None);
    m.fuel.inner.lag_coord.set(crate::demand_coordinate::LAG_COORD_DEMAND);
    (m, surge, lag, traj, accel)
}

// ---------------------------------------------------------------------------------------------
// § 1 — THE THREE SLOPES, AND THE INSTRUMENT CHECKED AGAINST RUNG 76
// ---------------------------------------------------------------------------------------------

/// Which `(b, v)` a filtered point was riding at — rung 76's own destructuring, narrowed to the
/// two fields this rung reads.
fn bv_of(p: &FuelPoint) -> (f64, f64) {
    match p.extra {
        PointExtra::Demand { b, v, .. } => (b, v),
        PointExtra::Shared { b, v, .. } => (b, v),
        _ => panic!("rung-77 reads `b`/`v` off every filtered point; `_ledger_march` marches the \
                     demand coordinate, so every point carries them."),
    }
}

/// ONE leg's reading in a [`SlopeRow`].
#[derive(Clone, Copy, Debug, Default)]
pub struct SlopeLeg {
    /// This leg's set point.
    pub w: f64,
    /// `G_w` at that set point — **and the three do not share a unit.**
    pub gw: f64,
    /// `w/scale · G_w` — each slope normalised by its OWN set point.
    pub norm: f64,
    /// `|scale / (w · G_w)|` — the STIFFNESS, the reciprocal of the normalised slope.
    pub stiff: f64,
}

/// One point of [`leg_slopes`].
#[derive(Clone, Copy, Debug)]
pub struct SlopeRow {
    pub s: f64,
    /// Indexed by [`Leg`] in [`Leg::ORDER`].
    pub legs: [SlopeLeg; 3],
    /// Rung 76's `c` at the accel leg's own set point, through
    /// [`c_at`](crate::sensed_cap::c_at).
    pub c: f64,
    /// `|(1 − G_a') − c|` — **the instrument, and it is a check on the ALGEBRA.** The two readings
    /// share a step size, so their roundoff cancels; § 1 of the spec says exactly that rather than
    /// quoting agreement to eleven figures.
    pub c_err: f64,
}

impl SlopeRow {
    pub fn leg(&self, leg: Leg) -> &SlopeLeg {
        &self.legs[leg as usize]
    }
}

/// [`leg_slopes`]'s return — § 1 of the spec.
#[derive(Clone, Debug)]
pub struct LegSlopes {
    pub phi_lim: f64,
    pub margin: f64,
    pub inc: bool,
    pub n: usize,
    pub rows: Vec<SlopeRow>,
    /// The instrument's worst reading over the march.
    pub c_err: Option<f64>,
    /// `(min, max)` of the accel leg's `G_w` — which IS `1 − c`.
    pub c: Option<(f64, f64)>,
    /// `(min, max)` of each leg's RAW slope, in [`Leg::ORDER`]. They do not share a unit.
    pub gw: Option<[(f64, f64); 3]>,
    /// `(min, max)` of each leg's slope normalised by its own set point.
    pub norm: Option<[(f64, f64); 3]>,
    /// `(min, max)` of each leg's stiffness.
    pub stiff: Option<[(f64, f64); 3]>,
    /// **P5's NON-VACUITY gate**: the smallest gap between any two legs' normalised slopes at any
    /// point. If this were zero the three would be one quantity in three costumes.
    pub sep: Option<f64>,
}

/// Python's `min(xs)` / `max(xs)` over a list — the FOLD, not `f64::min`/`f64::max`.
///
/// Python seeds at element 0 and replaces only on a strict comparison, so a NaN never displaces the
/// incumbent and a NaN at element 0 wins everything. `f64::min` has the opposite NaN rule. This
/// family's recorded hazard, spelled out once here and reused by both readers.
fn py_span(xs: impl Iterator<Item = f64>) -> Option<(f64, f64)> {
    let mut it = xs;
    let first = it.next()?;
    let (mut lo, mut hi) = (first, first);
    for x in it {
        if x < lo {
            lo = x;
        }
        if x > hi {
            hi = x;
        }
    }
    Some((lo, hi))
}

fn py_min(xs: impl Iterator<Item = f64>) -> Option<f64> {
    py_span(xs).map(|(lo, _)| lo)
}

fn py_max(xs: impl Iterator<Item = f64>) -> Option<f64> {
    py_span(xs).map(|(_, hi)| hi)
}

/// RUNG 77 § 1 — **the three residual slopes, and the ONE of them that is dimensionless.**
///
/// Each leg's `G_w` is read at ITS OWN set point, with the valve and stator FROZEN at the
/// trajectory's states — which is how every reader from rung 64 to rung 76 has read this plant, and
/// is the OPEN loop. § 3 reads the phi leg the other way, and that is the whole difference between
/// them.
///
/// # THE NEST — SITE 1 OF SIX, AND IT IS SAFE BY THE DEAD-WINDOW CRITERION AND NOT BY CONSTRUCTION
///
/// [`c_at`](crate::sensed_cap::c_at) freezes both state fields in its OWN body and clears them on
/// return, so the call below (`engine.py:19829`, inside the freeze opened at `:19821`) leaves this
/// block THAWED for the rest of its life. Plan § 5.32 (i) measures it firing **150 times** over the
/// rung-77 suite. It is safe because the only statement after it is the `c_err` arithmetic over
/// numbers already computed — nothing reads the plant in that window. **That is a property of
/// statement order, and Python states it nowhere.** It is stated here, and the port keeps Python's
/// clobber rather than restoring the previous value, because `tests/slice_y_dispatch.rs` already
/// pins the clearing policy on these two guards deliberately (step 1's measurement).
#[allow(clippy::too_many_arguments)]
pub fn leg_slopes(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, margin: f64, taus: (f64, f64, f64, f64), inc: bool, r: f64, s_settle: f64,
    ds: f64, v_max: f64, every: usize,
) -> LegSlopes {
    let sm = phi_lim / core.arming().map_lp_design.phi_surge - 1.0;
    let (m, surge, _lag, traj, accel) = ledger_march(
        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc, margin);
    let b_max = m.fuel.inner.lever.lim.expect("`_shared_rig` arms the valve").b_max;
    let pts = riding4(&traj, b_max);
    let mut rows: Vec<SlopeRow> = Vec::new();
    for p in pts.iter().step_by(every) {
        let (a, h, ms) = (p.nu_lp, p.nu_hp, p.mf_sched);
        let (q, v) = bv_of(p);
        let _sb = MarchedBleed::set(&m.fuel.inner, q);
        let _sv = MarchedStator::set(&m.fuel.inner, v);
        let ls = legs(&m.fuel, flight, a, h, ms, Some(&accel), surge.as_ref(), Some(tt4_max))
            .unwrap_or_else(|e| panic!("{}", e.0));
        let mut out = [SlopeLeg::default(); 3];
        for leg in Leg::ORDER {
            let l = ls.at(leg);
            let gw = slope_at(&*l.g, l.w, SLOPE_AT_REL).unwrap_or_else(|e| panic!("{}", e.0));
            out[leg as usize] = SlopeLeg {
                w: l.w,
                gw,
                norm: l.w / l.scale * gw,
                stiff: (l.scale / (l.w * gw)).abs(),
            };
        }
        // THE NEST. `c_at` freezes and clears both fields itself — see this function's doc.
        let c = c_at(&m, flight, a, h, &accel, ls.at(Leg::Accel).w, q, v, C_AT_REL)
            .unwrap_or_else(|e| panic!("{}", e.0));
        let c_err = ((1.0 - out[Leg::Accel as usize].gw) - c).abs();
        rows.push(SlopeRow { s: p.s, legs: out, c, c_err });
    }
    let span = |leg: Leg, f: fn(&SlopeLeg) -> f64| -> Option<(f64, f64)> {
        py_span(rows.iter().map(|x| f(x.leg(leg))))
    };
    let three = |f: fn(&SlopeLeg) -> f64| -> Option<[(f64, f64); 3]> {
        Some([span(Leg::Accel, f)?, span(Leg::Gov, f)?, span(Leg::Phi, f)?])
    };
    let sep = py_min(rows.iter().flat_map(|x| {
        [(Leg::Accel, Leg::Gov), (Leg::Gov, Leg::Phi), (Leg::Accel, Leg::Phi)]
            .into_iter()
            .map(move |(i, j)| (x.leg(i).norm - x.leg(j).norm).abs())
    }));
    LegSlopes {
        phi_lim,
        margin,
        inc,
        n: rows.len(),
        c_err: py_max(rows.iter().map(|x| x.c_err)),
        c: span(Leg::Accel, |l| l.gw),
        gw: three(|l| l.gw),
        norm: three(|l| l.norm),
        stiff: three(|l| l.stiff),
        sep,
        rows,
    }
}

// ---------------------------------------------------------------------------------------------
// § 2 — THE SENSITIVITY ITSELF, IN A CURRENCY ALL THREE LEGS SHARE
// ---------------------------------------------------------------------------------------------

/// Python's `at(qq)` — **the three legs on the plant AS THE VALVE IS AT `qq`.**
///
/// A free function and not a closure so the returned [`Legs`] can name the lifetime it borrows.
/// The two guards are local to this call, which is Python's own `try/finally`.
#[allow(clippy::too_many_arguments)]
fn legs_at<'a>(
    m: &'a ScheduledStatorCore, flight: &'a FlightCondition, a: f64, h: f64, ms: f64,
    accel: &'a AccelSchedule, surge: Option<&'a Floor>, tt4_max: f64, qq: f64, v: f64,
) -> Legs<'a> {
    let _sb = MarchedBleed::set(&m.fuel.inner, qq);
    let _sv = MarchedStator::set(&m.fuel.inner, v);
    legs(&m.fuel, flight, a, h, ms, Some(accel), surge, Some(tt4_max))
        .unwrap_or_else(|e| panic!("{}", e.0))
}

/// Python's `G_at(qq, name, w)` — **the residual at `w`, REBUILT inside this block and never
/// carried out of it.**
///
/// This is [`Residuals`]' whole reason. Building the closure outside and evaluating it here would
/// typecheck, run, and return the closed-valve plant's answer at both `q ± dq` — `G_q` identically
/// zero and a relative error of exactly `1.000e+00`, which is the defect Python's docstring records
/// shipping.
#[allow(clippy::too_many_arguments)]
fn residual_at(
    m: &ScheduledStatorCore, flight: &FlightCondition, a: f64, h: f64, accel: &AccelSchedule,
    surge: Option<&Floor>, tt4_max: f64, leg: Leg, qq: f64, v: f64, w: f64,
) -> f64 {
    let _sb = MarchedBleed::set(&m.fuel.inner, qq);
    let _sv = MarchedStator::set(&m.fuel.inner, v);
    let r = residuals(&m.fuel, flight, a, h, Some(accel), surge, Some(tt4_max));
    let g = r.get(leg).expect("rung-77 § 2 differences all three legs; this arming built them");
    g(w).unwrap_or_else(|e| panic!("{}", e.0))
}

/// ONE leg's reading in a [`GainRow`].
#[derive(Clone, Copy, Debug, Default)]
pub struct GainLeg {
    pub w: f64,
    /// The whole solve, re-run at `q ± dq` — exactly as the plant would.
    pub direct: f64,
    /// `−G_q/G_w`, the two partials differenced SEPARATELY at the unperturbed set point.
    pub ift: f64,
    pub gq: f64,
    pub gw: f64,
    pub live: bool,
    /// `|direct − ift| / max(|direct|, 1e-30)`. **P2 is that the two agree** — a reader that
    /// computed `ift` and called it `direct` would be rung 70's gate computing its own formula
    /// twice, which is why both are returned.
    pub err: f64,
}

/// One point of [`set_point_gains`].
#[derive(Clone, Debug)]
pub struct GainRow {
    pub s: f64,
    /// Indexed by [`Leg`] in [`Leg::ORDER`].
    pub legs: [GainLeg; 3],
    /// The legs actually ACTING here, in [`Leg::ORDER`].
    pub live: Vec<Leg>,
}

impl GainRow {
    pub fn leg(&self, leg: Leg) -> &GainLeg {
        &self.legs[leg as usize]
    }
}

/// [`set_point_gains`]'s return — § 2 of the spec.
#[derive(Clone, Debug)]
pub struct SetPointGains {
    pub phi_lim: f64,
    pub margin: f64,
    pub inc: bool,
    pub n: usize,
    pub rows: Vec<GainRow>,
    /// **P2**: the worst disagreement between the two computations of the same number.
    pub ift_err: Option<f64>,
    /// `(min, max)` of each leg's `dw*/dq`, in [`Leg::ORDER`].
    pub gain: Option<[(f64, f64); 3]>,
    /// **P3**: the ordering by `|dw*/dq|`, smallest first, at the first point.
    pub order: Option<[Leg; 3]>,
    pub order_stable: Option<bool>,
    /// How many points have more than one LIVE leg to order.
    pub n_guarded: usize,
    pub guarded: Option<Vec<Leg>>,
    pub guarded_stable: Option<bool>,
    /// The SET of guarded orderings seen, sorted — returned whole so a single inverting point
    /// cannot hide behind a representative.
    pub guarded_orders: Vec<Vec<Leg>>,
    /// The half of P3 that has to hold alone: wherever phi is live, it is the LARGEST gain.
    pub phi_top: bool,
}

/// Python's `sorted(keys, key=lambda k: abs(x[k]["direct"]))` — **STABLE, so ties fall back to
/// [`Leg::ORDER`]**.
///
/// `sort_by` is Rust's stable sort, and the comparator returns [`Ordering::Equal`] on a NaN key so
/// a non-comparison leaves the incumbent order exactly as Python's Timsort does when `<` is false.
fn order_of(row: &GainRow, keys: &[Leg]) -> Vec<Leg> {
    let mut v: Vec<Leg> = keys.to_vec();
    v.sort_by(|x, y| {
        row.leg(*x).direct.abs()
            .partial_cmp(&row.leg(*y).direct.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    v
}

/// RUNG 77 § 2 — **`dw*/dq = −G_q/G_w`, measured per leg, in kg/s per unit valve.**
///
/// THIS IS THE LEDGER'S LEGAL CURRENCY AND IT NEEDS NO NORMALISATION. § 1's three slopes carry
/// three different units, so ordering them is not a comparison; `dw*/dq` is a fuel per valve
/// position for all three legs, so ordering it IS one. § 1's normalised table survives only because
/// it is what reproduces rung 76 § 3's gain on the accel column.
///
/// # EVERY PLANT READ IS INSIDE THE BLOCK IT BELONGS TO, AND THAT IS THE RUNG-62 `_powers` TRAP
///
/// See [`residual_at`]. The one closure that legitimately escapes its block is `mid[name]["G"]`,
/// which is re-evaluated inside a FRESH `q` block for `G_w`; Python does exactly that and the port
/// keeps it, because the alternative — rebuilding at `q` — is a different arrangement of the same
/// three reads and a later reader would have nothing to diff against.
#[allow(clippy::too_many_arguments)]
pub fn set_point_gains(
    core: &ScheduledStatorCore, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, tt4_max: f64,
    phi_lim: f64, margin: f64, taus: (f64, f64, f64, f64), inc: bool, r: f64, s_settle: f64,
    ds: f64, v_max: f64, dq: f64, every: usize,
) -> SetPointGains {
    let sm = phi_lim / core.arming().map_lp_design.phi_surge - 1.0;
    let (m, surge, _lag, traj, accel) = ledger_march(
        core, flight, tt4_lo, tt4_hi, tt4_max, sm, taus, r, s_settle, ds, v_max, inc, margin);
    let b_max = m.fuel.inner.lever.lim.expect("`_shared_rig` arms the valve").b_max;
    let pts = riding4(&traj, b_max);
    let sg = surge.as_ref();
    let mut rows: Vec<GainRow> = Vec::new();
    for p in pts.iter().step_by(every) {
        let (a, h, ms) = (p.nu_lp, p.nu_hp, p.mf_sched);
        let (q, v) = bv_of(p);
        let lo = legs_at(&m, flight, a, h, ms, &accel, sg, tt4_max, q - dq, v);
        let mid = legs_at(&m, flight, a, h, ms, &accel, sg, tt4_max, q, v);
        let hi = legs_at(&m, flight, a, h, ms, &accel, sg, tt4_max, q + dq, v);
        let mut out = [GainLeg::default(); 3];
        for leg in Leg::ORDER {
            let direct = (hi.at(leg).w - lo.at(leg).w) / (2.0 * dq);
            let w0 = mid.at(leg).w;
            let gw = {
                let _sb = MarchedBleed::set(&m.fuel.inner, q);
                let _sv = MarchedStator::set(&m.fuel.inner, v);
                slope_at(&*mid.at(leg).g, w0, SLOPE_AT_REL)
                    .unwrap_or_else(|e| panic!("{}", e.0))
            };
            // `G_q` at FIXED `w`: the residual RE-READ on the perturbed plant.
            let gq = (residual_at(&m, flight, a, h, &accel, sg, tt4_max, leg, q + dq, v, w0)
                - residual_at(&m, flight, a, h, &accel, sg, tt4_max, leg, q - dq, v, w0))
                / (2.0 * dq);
            let ift = -gq / gw;
            let ad = direct.abs();
            // Python's `max(abs(direct), 1e-30)` — expression-first, so the explicit fold.
            let den = if 1e-30 > ad { 1e-30 } else { ad };
            out[leg as usize] = GainLeg {
                w: w0, direct, ift, gq, gw, live: mid.at(leg).live,
                err: (direct - ift).abs() / den,
            };
        }
        let live: Vec<Leg> = Leg::ORDER.into_iter().filter(|k| out[*k as usize].live).collect();
        rows.push(GainRow { s: p.s, legs: out, live });
    }
    let all: Vec<Leg> = Leg::ORDER.to_vec();
    let order: Vec<Vec<Leg>> = rows.iter().map(|x| order_of(x, &all)).collect();
    // THE GUARDED ORDER: only legs that are actually acting at this point (rung 76 § 1.3).
    let g_order: Vec<Vec<Leg>> = rows
        .iter()
        .filter(|x| x.live.len() > 1)
        .map(|x| order_of(x, &x.live))
        .collect();
    let mut guarded_orders: Vec<Vec<Leg>> = g_order.clone();
    guarded_orders.sort();
    guarded_orders.dedup();
    let span = |leg: Leg, f: fn(&GainLeg) -> f64| -> Option<(f64, f64)> {
        py_span(rows.iter().map(|x| f(x.leg(leg))))
    };
    SetPointGains {
        phi_lim,
        margin,
        inc,
        n: rows.len(),
        ift_err: py_max(rows.iter().flat_map(|x| Leg::ORDER.map(|k| x.leg(k).err))),
        gain: match (span(Leg::Accel, |l| l.direct), span(Leg::Gov, |l| l.direct),
                     span(Leg::Phi, |l| l.direct)) {
            (Some(a), Some(g), Some(s)) => Some([a, g, s]),
            _ => None,
        },
        order: order.first().map(|o| [o[0], o[1], o[2]]),
        order_stable: if order.is_empty() {
            None
        } else {
            Some(order.iter().all(|o| o == &order[0]))
        },
        n_guarded: g_order.len(),
        guarded: g_order.first().cloned(),
        guarded_stable: if g_order.is_empty() {
            None
        } else {
            Some(g_order.iter().all(|o| o == &g_order[0]))
        },
        guarded_orders,
        phi_top: rows
            .iter()
            .filter(|x| x.live.contains(&Leg::Phi))
            .all(|x| *order_of(x, &x.live).last().expect("live is non-empty here") == Leg::Phi),
        rows,
    }
}
