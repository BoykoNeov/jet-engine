//! RUNGS 62–63 — **the BLEED SCHEDULE beside the STATOR SCHEDULE, on the transient plant**, and
//! then a FUEL leg beside both. Python's `BleedSchedule` + `ScheduledBleedTransient`
//! (`turbojet/engine.py` 8783–9692, 908 lines).
//!
//! **STEP 1 SHIPS THE CELLS AND NOTHING ELSE.** The bodies are step 2's. What is here is the
//! virtual table ([`LeverHooks`]), the state it dispatches on ([`LeverArming`]), the keyword
//! bundle its constructor cell takes ([`LeverArm`]), rung 62's own schedule type
//! ([`BleedSchedule`]) and the report struct [`legs`](LeverHooks::legs) returns. Every cell
//! defaults to [`NO_LEVER`], whose bodies PANIC — rungs 40/43/57 have no `b_of` and no
//! `_armed_bleed` in Python at all, so a default that silently answered `false` would be a claim
//! no value gate could see. [`crate::stator_transient::NO_STATOR`]'s precedent, and its reason.
//!
//! # § 5.21 (ii)/(iii) — the two decisions this module is shaped by, both MEASURED
//!
//! **THE VALVE STATE LIVES ON [`TwoSpoolTransientCore`], NOT ON A RUNG-62 TYPE, AND A TEST FORCED
//! THAT.** `tests/test_rung63.py::test_the_at_stator_trap_is_real_and_returns_rung59s_zero_for_free`
//! calls the INHERITED rung-59 reader `schedule_invariance` on a bleed-armed machine and asserts
//! it reports the `Wf/pt3` table bit-identical — rung 59's exact headline, *while measuring
//! nothing*, because rung 62 overrode `at_stator` so the reader differences against a sibling
//! carrying this machine's valve. That reader's receiver is [`ScheduledStatorCore`], so
//! `at_stator` must return a [`ScheduledStatorCore`] **that answers `armed_bleed`**. Measured on
//! the shipped Python, forcing rung 57's un-overridden body (which is what a `-> ScheduledStatorCore`
//! returning a BARE sibling would be) flips the gate's two identities from `True/True` to
//! `False/False`, at `9.543e-3` and `1.019e-2`.
//!
//! **AND THERE IS NO RUNG-62 TYPE — ONE TYPE CARRIES RUNGS 57–84.** `at_lever` is overridden by
//! **17** classes and called **46** times; typing its cell `-> ScheduledBleedCore` would re-open a
//! gated signature at twelve later slices. `probe_w5.py` measured what the ladder actually adds:
//! **five arming fields in twenty-three rungs** (`bleed`, `bleed_sched` at 62; `bleed_lim` at 64;
//! `stator_lim` at 68; `stator_inc` at 69), every one a plain scalar, and `at_lever`'s keyword
//! list grows monotonically to nine and then stops. So the cell takes a **struct** ([`LeverArm`])
//! and returns [`ScheduledStatorCore`]; each later slice adds one field with a `Default` and swaps
//! a body, which is additive where a parameter list is not. **The rung is the TABLE, not the
//! type** — § 2's architecture, and the reason it was chosen.
//!
//! *Registered as `-> ScheduledBleedCore` in § 5.21 (iii) and corrected here: a newtype over
//! [`ScheduledStatorCore`] cannot be what `at_stator` returns, so it would have bought a type
//! distinction for `at_lever` alone at the cost of a `Deref` on every inherited reader. The
//! decision (one type, an arm struct, a stable signature) is unchanged; only its spelling is.*
//!
//! # The reduce
//!
//! Per CALL, and it is live MID-MARCH. `b_of` is a pure function of the live state — no history,
//! no latch, RK4-legal exactly as rung 57's `_arm` is — and every overridden closure returns to
//! its rung-57 parent **verbatim** whenever that value is `0.0`. A [`BleedSchedule`] is exactly
//! `0` at and above `n_ref`, so a *scheduled* machine takes both branches inside one march:
//! measured **12 reduced / 53 bled** on `_close` and **0 / 344** on `_close_fuel` over one
//! `equilibrium` + one march at `ds = 0.02`. **A value gate cannot see the second half** — the two
//! branches agree wherever `b` is 0 — which is why slice W owes a dispatch gate (§ 5.21 P4).

use std::cell::Cell;

use crate::components::{choked_mfp, Nozzle};
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AccelSchedule, Floor, SurgeLimiter, FuelCloseState, FuelPoint, FuelTransientCore, FuelTransientHooks,
};
use crate::gas::{powp, Abort, FlowState};
use crate::map::ComponentMap;
use crate::matcher::Branch;
use crate::spool::{try_illinois, ILLINOIS_MAXIT};
use crate::stator_transient::{
    CellRead, ClampAudit, CreditProfile, LegKind, Ramp, ScheduledStatorCore,
    ScheduledStatorTransient, Shape, StatorArm, StatorLeg, StatorTransientHooks,
    R57, R57_FUEL, R57_TWO,
};
use crate::two_spool::{Spool, TwoSpoolEngine};
use crate::two_spool_transient::{
    CloseState, Instant2, TwoSpoolTransientCore, TwoSpoolTransientHooks,
};

// ---------------------------------------------------------------------------------------------
// RUNG 62's SCHEDULE — the deliberate TWIN of rung 57's, ported as its OWN type
// ---------------------------------------------------------------------------------------------

/// RUNG 62. A handling-bleed schedule `b(n_L)` in the LP corrected speed.
///
/// ```text
/// b(n) = b_max * S( (n_ref - n)/(n_ref - n_lo) )        S clipped to [0, 1]
/// ```
///
/// OPEN at low corrected speed, closing monotonically, and EXACTLY 0 at and above the design
/// speed `n_ref` — which is rung 42's *"the valve is SHUT at the design point by construction"*
/// and rung 53/57's hardware-capture discipline saying the same thing from the other side.
/// `__post_init__` ASSERTS it rather than relying on the algebra, and so does
/// [`with_shape`](Self::with_shape).
///
/// # NOT FACTORED AGAINST [`StatorSchedule`], AND THAT IS THE POINT
///
/// Python's docstring says the twinning is DELIBERATE: same functional form, same two shapes,
/// same corner assert, *"the two levers must differ in their PHYSICS and in nothing else, or the
/// rung's headline (their loop gains have opposite SIGNS) would be comparing two schedule
/// definitions rather than two devices."* Rust invites one generic `Schedule<Kind>` here and it
/// must be refused — [[rust-port-copy-vs-rederivation]], *don't factor a deliberate duplication
/// away*. What IS shared is [`Shape`], because the shape FUNCTION is one function in both: `S(x)`
/// is not a device, and giving it two spellings would be the mirror error.
///
/// [`StatorSchedule`]: crate::stator_transient::StatorSchedule
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BleedSchedule {
    pub b_max: f64,
    pub n_lo: f64,
    pub n_ref: f64,
    pub shape: Shape,
}

impl BleedSchedule {
    /// Python's default `n_ref`.
    pub const N_REF: f64 = 1.0;

    /// `n_ref = 1.0`, `shape = "smooth"` — Python's two defaults, which is how every shipped
    /// caller builds one.
    pub fn new(b_max: f64, n_lo: f64) -> Self {
        Self::with_shape(b_max, n_lo, Self::N_REF, Shape::Smooth)
    }

    /// The full constructor, carrying `__post_init__`'s three surviving asserts. The fourth —
    /// the `shape` membership check — is [`try_from_str`](Self::try_from_str)'s, because an enum
    /// deletes it here.
    pub fn with_shape(b_max: f64, n_lo: f64, n_ref: f64, shape: Shape) -> Self {
        assert!(n_lo < n_ref,
                "rung-62 BleedSchedule needs n_lo < n_ref: got {n_lo} >= {n_ref}");
        assert!((0.0..0.5).contains(&b_max),
                "rung-42's own bound: b >= 0.5 starves the core and the choked branch is long \
                 gone by then; got b_max = {b_max}");
        let s = BleedSchedule { b_max, n_lo, n_ref, shape };
        assert!(s.at(n_ref) == 0.0,
                "rung-62 BleedSchedule must be EXACTLY 0 at the design corrected speed n_ref -- \
                 rung 42 captures the hardware with the valve SHUT.");
        s
    }

    /// Python's `shape` membership assert, which a Rust enum otherwise deletes.
    pub fn try_from_str(b_max: f64, n_lo: f64, n_ref: f64, shape: &str) -> Self {
        let sh = match shape {
            "smooth" => Shape::Smooth,
            "linear" => Shape::Linear,
            other => panic!(
                "rung-62 BleedSchedule shape must be 'smooth' (C1, default) or 'linear' \
                 (C0 control), got {other:?}"),
        };
        Self::with_shape(b_max, n_lo, n_ref, sh)
    }

    /// `b(n)` — Python's `__call__`.
    #[allow(clippy::manual_clamp)]
    pub fn at(&self, n: f64) -> f64 {
        let x = (self.n_ref - n) / (self.n_ref - self.n_lo);
        // Python's `0.0 if x < 0.0 else (1.0 if x > 1.0 else x)` — a two-arm conditional, NOT
        // `clamp`, which differs on NaN. Spelled the same way rung 57's twin is.
        let x = if x < 0.0 { 0.0 } else if x > 1.0 { 1.0 } else { x };
        self.b_max
            * match self.shape {
                Shape::Smooth => x * x * (3.0 - 2.0 * x),
                Shape::Linear => x,
            }
    }
}

// ---------------------------------------------------------------------------------------------
// THE STATE — held on the shared core, for § 5.21 (ii)'s measured reason
// ---------------------------------------------------------------------------------------------

/// RUNG 62's valve arming — the two fields Python's `__init__` sets past rung 57's.
///
/// **THIS GROWS, AND THAT IS THE DESIGN.** Rung 64 adds `bleed_lim`, rung 68 `stator_lim`, rung 69
/// `stator_inc`; rungs 65–67 and 70–84 add nothing at all. Adding a field with a `Default` is
/// additive, so no later slice re-opens [`LeverHooks::at_lever`]'s signature — § 5.21 (iii)/P5.
///
/// **NOT [`crate::two_spool::TwoSpoolMapCore::bleed`], which is rung 42's STEADY valve.** The two
/// never collide in Python either: `TwoSpoolBleedMatcher` is a sibling branch, not an ancestor of
/// `ScheduledBleedTransient`, and no transient body reads `self.bleed` before rung 62 defines it
/// (measured — both directions, over the whole ancestry).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LeverArming {
    /// A CONSTANT position — rung 42's lever transplanted. Applied at construction, so
    /// `equilibrium` and `fuel_for_tt4` see it and the march starts on the BLED running line.
    pub bleed: f64,
    /// A schedule read off the live state at every closure — what a real handling-bleed system
    /// implements. Mutually exclusive with `bleed`, asserted at construction.
    pub sched: Option<BleedSchedule>,
    /// RUNG 64's CLOSED loop — a floor on `phi_lp` the valve rides to hold. Mutually exclusive
    /// with BOTH of the above: rung 62's two-way assert is EXTENDED to three, never replaced,
    /// because the three are the legs the rung differences.
    pub lim: Option<crate::limited_bleed::BleedLimiter>,
}

impl LeverArming {
    /// What every rung-40/43/57 object carries: no valve.
    pub fn unarmed() -> Self {
        LeverArming::default()
    }
}

/// The keyword bundle `at_lever` takes — Python's six (rung 62) → nine (rung 69) keywords, as one
/// struct so the cell's signature never changes. [`Default`] is the BARE sibling, which is what
/// `at_lever()` with no arguments builds and what every rung-62 difference is taken against.
///
/// Distinct from [`LeverArming`] on purpose: this carries the STATOR keywords too, because
/// `at_lever` re-arms **both** devices, while the stator half of the state already lives in
/// [`crate::stator_transient::StatorArming`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LeverArm {
    pub stator: StatorArm,
    pub bleed: f64,
    pub bleed_sched: Option<BleedSchedule>,
    /// RUNG 64's keyword — **the FIRST TEST of § 5.21 (iii)/P5**, and it is a field with a
    /// `Default` rather than a parameter, so [`LeverHooks::at_lever`]'s signature is not
    /// re-opened. `at_lever` goes 6 → 7 keywords here, 8 at rung 68, 9 at 69, then stops.
    pub bleed_lim: Option<crate::limited_bleed::BleedLimiter>,
}

impl LeverArm {
    /// The valve alone — `at_lever(bleed_sched=…)`.
    pub fn scheduled(s: BleedSchedule) -> Self {
        LeverArm { bleed_sched: Some(s), ..Default::default() }
    }

    /// The valve alone, at a CONSTANT position — `at_lever(bleed=…)`.
    pub fn constant(b: f64) -> Self {
        LeverArm { bleed: b, ..Default::default() }
    }

    /// The stator alone — `at_lever(vsv_…=…)`, which is how rung 62 spells its neighbour.
    pub fn stator(s: StatorArm) -> Self {
        LeverArm { stator: s, ..Default::default() }
    }

    /// RUNG 64's leg — `at_lever(bleed_lim=…)`, the valve under a CLOSED loop.
    pub fn floored(l: crate::limited_bleed::BleedLimiter) -> Self {
        LeverArm { bleed_lim: Some(l), ..Default::default() }
    }

    /// Python's `{**neighbour, **lever}` — the LEVER wins on every field it sets, which is what
    /// `_isolating`'s disjointness assert has already guaranteed cannot overlap.
    pub fn merged(neighbour: &LeverArm, lever: &LeverArm) -> Self {
        LeverArm {
            stator: StatorArm {
                vsv_lp: if lever.stator.vsv_lp != 0.0 { lever.stator.vsv_lp }
                        else { neighbour.stator.vsv_lp },
                vsv_hp: if lever.stator.vsv_hp != 0.0 { lever.stator.vsv_hp }
                        else { neighbour.stator.vsv_hp },
                sched_lp: lever.stator.sched_lp.or(neighbour.stator.sched_lp),
                sched_hp: lever.stator.sched_hp.or(neighbour.stator.sched_hp),
                lp_disabled: lever.stator.lp_disabled || neighbour.stator.lp_disabled,
            },
            bleed: if lever.bleed != 0.0 { lever.bleed } else { neighbour.bleed },
            bleed_sched: lever.bleed_sched.or(neighbour.bleed_sched),
            bleed_lim: lever.bleed_lim.or(neighbour.bleed_lim),
        }
    }

    /// Python's `bool(kw.get("bleed")) or kw.get("bleed_sched") is not None` — the check
    /// `_isolating`'s reference-sibling assert runs on the NEIGHBOUR dict, before any object
    /// exists to ask.
    pub fn arms_valve(&self) -> bool {
        self.bleed != 0.0 || self.bleed_sched.is_some() || self.bleed_lim.is_some()
    }
}

/// WHICH lever a report is about — Python's `"bleed"`/`"stator"` string, chosen by
/// `_armed_bleed()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lever {
    Bleed,
    Stator,
}

/// Python's `_legs` return — rung 57's START / RAMP / FULL generalised to ANY reference machine.
///
/// `loop_` is Python's `loop` key, which is a Rust keyword.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegsReport {
    pub spool: Spool,
    pub r: f64,
    pub reference: f64,
    pub start: f64,
    pub ramp: f64,
    pub full: f64,
    /// `full / ramp`. `< 1` is rung 57's negative feedback; `> 1` is AMPLIFICATION — the rung.
    pub self_cancel: f64,
    pub surrendered: f64,
    pub share_start: f64,
    /// What the START term does NOT explain: `(full - ramp) - start`.
    pub loop_: f64,
    pub nu0_ref: f64,
    pub nu0_armed: f64,
    /// The loop witnessed DIRECTLY: between the two legs a stator schedule commands LESS of
    /// itself and a bleed schedule commands MORE.
    pub cmd_ramp: f64,
    pub cmd_full: f64,
    pub s_ref: f64,
    pub s_ramp: f64,
    pub s_full: f64,
    pub lever: Lever,
}

// ---------------------------------------------------------------------------------------------
// THE TABLE — five cells, and § 5.21 (i) is why two of them are here at all
// ---------------------------------------------------------------------------------------------

/// RUNGS 62–63's five virtual names.
///
/// **TWO OF THESE FIVE ARE NOT IN § 5.19 (x)'s COLUMN.** That column books `at_lever` and
/// `at_stator` as *pure sibling constructors* needing no cell. `at_lever` is overridden by
/// **seventeen** ladder classes and called **forty-six** times — the most-dispatched name in
/// phase 7 — and `at_stator` by one. Re-running the emitter over the WHOLE remaining ladder
/// rather than over this slice found four such names in total (`at_stator` at V, `at_lever` here,
/// `_quad_gains_at` at AD, `_with_coord` at AF) and put the phase's measured cell count at **35**,
/// not 28. § 5.21 (i).
///
/// **THE RECEIVERS DIFFER, WHICH IS WHY ONE TABLE HOLDS ALL FIVE** — [`StatorTransientHooks`]'s
/// note in full. `armed_bleed` and `b_of` are reached from inside rung-40 and rung-43 hook bodies,
/// so their `self` is the shallowest core carrying the state ([`TwoSpoolTransientCore`]);
/// `at_lever`, `isolating` and `legs` re-invoke a CONSTRUCTOR and read the design references, so
/// theirs is [`ScheduledStatorCore`]. A struct of `fn` pointers has no receiver of its own.
///
/// [`StatorTransientHooks`]: crate::stator_transient::StatorTransientHooks
pub struct LeverHooks {
    /// A sibling on the SAME hardware and the same design references, BOTH levers re-armed.
    /// Overridden at rungs **64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80**
    /// — seventeen bodies, each copying one more field of [`LeverArm`] forward. Python's own
    /// docstring at rung 73 calls it *"THE ELEVENTH INSTANCE of the trap rungs 61–72 each hit"*.
    pub at_lever: fn(&ScheduledStatorCore, &LeverArm) -> ScheduledStatorCore,
    /// Does this machine carry a valve at all? Overridden at rung **64** (`or bleed_lim is not
    /// None`). Read by `_legs` to name the lever and by `_isolating`'s reference assert.
    pub armed_bleed: fn(&TwoSpoolTransientCore) -> bool,
    /// The valve position this machine holds at the given state — constant or scheduled. **No
    /// history, no latch**, so it is RK4-legal exactly as rung 57's `_arm` is. Overridden at rung
    /// **64**. Every reader goes through this rather than through the stored `bleed`, which for a
    /// scheduled machine is `0.0` and means nothing.
    pub b_of: fn(&TwoSpoolTransientCore, f64, Option<f64>) -> f64,
    /// RUNG 63's `(reference, armed)` sibling PAIR that isolates one lever in the presence of a
    /// neighbour — and THE GATE that makes every rung-63 reader trustworthy. Overridden at rung
    /// **64**.
    pub isolating: fn(&ScheduledStatorCore, &LeverArm, Option<&LeverArm>)
        -> (ScheduledStatorCore, ScheduledStatorCore),
    /// Rung 57's START / RAMP / FULL against ANY reference machine. Overridden at rung **77**.
    pub legs: fn(&ScheduledStatorCore, &FlightCondition, &ScheduledStatorCore, &Ramp, Spool,
                 &StatorLeg<'_>) -> LegsReport,
    /// RUNG 64's **committed** valve position at a recorded trajectory point — the ONE cell
    /// slice X creates, and the one row of § 5.19 (x)'s hand-written cell column an emitter
    /// confirms (§ 5.22 (iii)).
    ///
    /// **IT RE-SOLVES; IT DOES NOT RECONSTRUCT.** Python's docstring: *"the valve is a pure
    /// function of the state, so this RE-SOLVES it exactly rather than reconstructing it —
    /// which is what makes the bleed integral below a measurement and not an estimate."*
    /// Reconstructing instead drives a floored march's `b_int` and `b_peak` to **exactly 0** and
    /// both of the rung's PUBLISHED ratios (0.2552, 0.5187) to 0 — **and all 111 rung-62/63/64
    /// gates still pass**, because the only assertion that reads them is the ordering
    /// `f < s < c`, which zeroing the SMALLEST term satisfies. § 5.22 (ii); the gate that
    /// catches it is one slice X ADDS, not one it ports.
    ///
    /// Overridden at rung **65**, whose lagged valve marches the position instead.
    pub b_at_point: fn(&ScheduledStatorCore, &FlightCondition, &FuelPoint) -> f64,
}

/// **THE DEFAULT, AND ITS CELLS PANIC.** Rungs 40/43/57 have no `b_of`, no `_armed_bleed` and no
/// `at_lever` in Python at all — an unvalved rung-57 object is not a rung-62 object with the
/// valve shut, it is an object where the names do not exist. Defaulting `armed_bleed` to `false`
/// would silently make every rung-57 machine answer a question it cannot be asked, and **that is
/// a claim no value gate could see**: it agrees with the truth on exactly the machines the suites
/// build. [`crate::stator_transient::NO_STATOR`]'s precedent and its reason.
///
/// Unreachable by construction: the only dispatchers are the four methods on
/// [`TwoSpoolTransientCore`] and [`ScheduledStatorCore`] below, and their only callers are rung
/// 62's own cell bodies and readers.
pub const NO_LEVER: LeverHooks = LeverHooks {
    at_lever: no_lever_at_lever,
    armed_bleed: no_lever_armed_bleed,
    b_of: no_lever_b_of,
    isolating: no_lever_isolating,
    legs: no_lever_legs,
    b_at_point: no_lever_b_at_point,
};

fn no_lever_at_lever(_: &ScheduledStatorCore, _: &LeverArm) -> ScheduledStatorCore {
    panic!("no lever table on this object: at_lever is rung 62's own sibling constructor, and \
            rungs 40/43/57 have no valve to re-arm. Reaching it means a rung-62 body ran on a \
            core built without R62.");
}

fn no_lever_armed_bleed(_: &TwoSpoolTransientCore) -> bool {
    panic!("no lever table on this object: rungs 40/43/57 have no _armed_bleed at all. \
            Answering `false` here would be a claim no value gate could see.");
}

fn no_lever_b_of(_: &TwoSpoolTransientCore, _: f64, _: Option<f64>) -> f64 {
    panic!("no lever table on this object: rungs 40/43/57 have no b_of.");
}

fn no_lever_isolating(_: &ScheduledStatorCore, _: &LeverArm, _: Option<&LeverArm>)
    -> (ScheduledStatorCore, ScheduledStatorCore) {
    panic!("no lever table on this object: _isolating is rung 63's own.");
}

fn no_lever_b_at_point(_: &ScheduledStatorCore, _: &FlightCondition, _: &FuelPoint) -> f64 {
    panic!("no lever table on this object: b_at_point is rung 64's own, and rungs 40/43/57/62 \
            have no committed valve position to report.");
}

fn no_lever_legs(_: &ScheduledStatorCore, _: &FlightCondition, _: &ScheduledStatorCore, _: &Ramp,
                 _: Spool, _: &StatorLeg<'_>) -> LegsReport {
    panic!("no lever table on this object: _legs is rung 62's own generalisation of rung 57's \
            three-leg decomposition.");
}

// ---------------------------------------------------------------------------------------------
// The dispatch points, on the cores the cells' receivers name
// ---------------------------------------------------------------------------------------------

impl TwoSpoolTransientCore {
    /// Rung 62's `_armed_bleed`, **through the virtual table**.
    pub fn armed_bleed(&self) -> bool {
        (self.lever_hooks.armed_bleed)(self)
    }

    /// Rung 62's `b_of`, **through the virtual table**. `tt2 = None` reads against the DESIGN
    /// `Tt2`, which is the convention every rung-62 reader uses.
    pub fn b_of(&self, nu_lp: f64, tt2: Option<f64>) -> f64 {
        (self.lever_hooks.b_of)(self, nu_lp, tt2)
    }
}

impl ScheduledStatorCore {
    /// Rung 62's `at_lever`, **through the virtual table**.
    pub fn at_lever(&self, arm: &LeverArm) -> ScheduledStatorCore {
        (self.fuel.inner.lever_hooks.at_lever)(self, arm)
    }

    /// The BARE sibling — `at_lever()` with no arguments, which is how rung 62 spells its own
    /// reference machine in `loop_decomposition`, `loop_factors`, `pair_interaction` and
    /// `clock_sweep`.
    pub fn bare_lever(&self) -> ScheduledStatorCore {
        self.at_lever(&LeverArm::default())
    }

    /// Rung 63's `_isolating`, **through the virtual table**.
    pub fn isolating(&self, lever: &LeverArm, neighbour: Option<&LeverArm>)
        -> (ScheduledStatorCore, ScheduledStatorCore) {
        (self.fuel.inner.lever_hooks.isolating)(self, lever, neighbour)
    }

    /// Rung 64's `b_at_point`, **through the virtual table**.
    pub fn b_at_point(&self, flight: &FlightCondition, p: &FuelPoint) -> f64 {
        (self.fuel.inner.lever_hooks.b_at_point)(self, flight, p)
    }

    /// Rung 62's `_legs`, **through the virtual table**.
    pub fn legs(&self, flight: &FlightCondition, reference: &ScheduledStatorCore, ramp: &Ramp,
                spool: Spool, leg: &StatorLeg<'_>) -> LegsReport {
        (self.fuel.inner.lever_hooks.legs)(self, flight, reference, ramp, spool, leg)
    }

    /// Whether THIS machine carries a valve — the dispatch [`TwoSpoolTransientCore::armed_bleed`]
    /// reaches, spelled at the level rung 62's readers hold. **This is the method
    /// `test_the_at_stator_trap_is_real_and_returns_rung59s_zero_for_free` calls on
    /// `at_stator()`'s return**, and the reason [`LeverArming`] lives where it does.
    pub fn armed_bleed(&self) -> bool {
        self.fuel.inner.armed_bleed()
    }
}

// ---------------------------------------------------------------------------------------------
// Counters — the reduce is BY DISPATCH, and § 5.21 (v) says no value key can see it
// ---------------------------------------------------------------------------------------------

thread_local! {
    /// `b_of` returned `0.0`, so the cell handed straight back to rung 57's body. **THE REDUCE.**
    static CLOSE_REDUCED: Cell<u64> = const { Cell::new(0) };
    /// `b_of` returned nonzero: the BLED closure ran.
    static CLOSE_BLED: Cell<u64> = const { Cell::new(0) };
    static CLOSE_FUEL_REDUCED: Cell<u64> = const { Cell::new(0) };
    static CLOSE_FUEL_BLED: Cell<u64> = const { Cell::new(0) };
    /// `_powers`/`_instant_tail` dispatch on the CLOSURE'S OWN `bleed`, not on `b_of` — an
    /// ABSENT dict key in Python. These two pairs and the two above are what a wrong spelling
    /// moves, and nothing else is.
    static POWERS_REDUCED: Cell<u64> = const { Cell::new(0) };
    static POWERS_BLED: Cell<u64> = const { Cell::new(0) };
    static TAIL_REDUCED: Cell<u64> = const { Cell::new(0) };
    static TAIL_BLED: Cell<u64> = const { Cell::new(0) };
    static B_OF_CALLS: Cell<u64> = const { Cell::new(0) };
    /// `b_of` read the CONSTANT position (`bleed_sched is None`) rather than a schedule.
    static B_OF_CONSTANT: Cell<u64> = const { Cell::new(0) };
    /// A scheduled `b_of` that came back EXACTLY zero — the schedule at or above `n_ref`. This
    /// is the arm that makes the reduce live MID-MARCH rather than per machine.
    static B_OF_SCHED_ZERO: Cell<u64> = const { Cell::new(0) };
    static B_OF_SCHED_OPEN: Cell<u64> = const { Cell::new(0) };
    static AT_LEVER_CALLS: Cell<u64> = const { Cell::new(0) };
    /// `at_stator` ran rung 62's body — i.e. handed back a sibling CARRYING THIS VALVE.
    static AT_STATOR_R62: Cell<u64> = const { Cell::new(0) };
    static ISOLATING_CALLS: Cell<u64> = const { Cell::new(0) };
    static LEGS_CALLS: Cell<u64> = const { Cell::new(0) };
    /// `_legs` named the lever `bleed` (i.e. `_armed_bleed()` was true) rather than `stator`.
    static LEGS_LEVER_BLEED: Cell<u64> = const { Cell::new(0) };
    /// Rung 62's bled `_close` failed to bracket.
    static CLOSE_BRACKET_FAILS: Cell<u64> = const { Cell::new(0) };
    static CLOSE_FUEL_BRACKET_FAILS: Cell<u64> = const { Cell::new(0) };
    /// The low-wall march-in advanced — rung 40's loop, measured DEAD there. Counted here
    /// against its own call count rather than assumed to inherit that.
    static MARCH_IN_ADVANCES: Cell<u64> = const { Cell::new(0) };
    static FUEL_MARCH_IN_ADVANCES: Cell<u64> = const { Cell::new(0) };
    /// Python's `max(lo0, 0.02)` literal arm, and the three arms of the fuel closure's `min`.
    static LO_FLOOR_HITS: Cell<u64> = const { Cell::new(0) };
    static HI_WALL_LITERAL: Cell<u64> = const { Cell::new(0) };
    static HI_WALL_MAP: Cell<u64> = const { Cell::new(0) };
    static HI_WALL_HI0: Cell<u64> = const { Cell::new(0) };
}

fn bump(c: &'static std::thread::LocalKey<Cell<u64>>) {
    c.with(|x| x.set(x.get() + 1));
}

/// This module's counters. **The four `*_REDUCED`/`*_BLED` pairs ARE § 5.21 P4's gate** — a
/// `_powers` "simplified" to re-read `b_of` instead of the closure's own `bleed` key moves
/// `powers_reduced` and moves NO value, because the two branches agree wherever `b` is 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Census {
    pub close_reduced: u64,
    pub close_bled: u64,
    pub close_fuel_reduced: u64,
    pub close_fuel_bled: u64,
    pub powers_reduced: u64,
    pub powers_bled: u64,
    pub tail_reduced: u64,
    pub tail_bled: u64,
    pub b_of_calls: u64,
    pub b_of_constant: u64,
    pub b_of_sched_zero: u64,
    pub b_of_sched_open: u64,
    pub at_lever_calls: u64,
    pub at_stator_r62: u64,
    pub isolating_calls: u64,
    pub legs_calls: u64,
    pub legs_lever_bleed: u64,
    pub close_bracket_fails: u64,
    pub close_fuel_bracket_fails: u64,
    pub march_in_advances: u64,
    pub fuel_march_in_advances: u64,
    pub lo_floor_hits: u64,
    pub hi_wall_literal: u64,
    pub hi_wall_map: u64,
    pub hi_wall_hi0: u64,
}

pub mod counters {
    use super::*;

    pub fn take() -> Census {
        let c = Census {
            close_reduced: CLOSE_REDUCED.with(|x| x.get()),
            close_bled: CLOSE_BLED.with(|x| x.get()),
            close_fuel_reduced: CLOSE_FUEL_REDUCED.with(|x| x.get()),
            close_fuel_bled: CLOSE_FUEL_BLED.with(|x| x.get()),
            powers_reduced: POWERS_REDUCED.with(|x| x.get()),
            powers_bled: POWERS_BLED.with(|x| x.get()),
            tail_reduced: TAIL_REDUCED.with(|x| x.get()),
            tail_bled: TAIL_BLED.with(|x| x.get()),
            b_of_calls: B_OF_CALLS.with(|x| x.get()),
            b_of_constant: B_OF_CONSTANT.with(|x| x.get()),
            b_of_sched_zero: B_OF_SCHED_ZERO.with(|x| x.get()),
            b_of_sched_open: B_OF_SCHED_OPEN.with(|x| x.get()),
            at_lever_calls: AT_LEVER_CALLS.with(|x| x.get()),
            at_stator_r62: AT_STATOR_R62.with(|x| x.get()),
            isolating_calls: ISOLATING_CALLS.with(|x| x.get()),
            legs_calls: LEGS_CALLS.with(|x| x.get()),
            legs_lever_bleed: LEGS_LEVER_BLEED.with(|x| x.get()),
            close_bracket_fails: CLOSE_BRACKET_FAILS.with(|x| x.get()),
            close_fuel_bracket_fails: CLOSE_FUEL_BRACKET_FAILS.with(|x| x.get()),
            march_in_advances: MARCH_IN_ADVANCES.with(|x| x.get()),
            fuel_march_in_advances: FUEL_MARCH_IN_ADVANCES.with(|x| x.get()),
            lo_floor_hits: LO_FLOOR_HITS.with(|x| x.get()),
            hi_wall_literal: HI_WALL_LITERAL.with(|x| x.get()),
            hi_wall_map: HI_WALL_MAP.with(|x| x.get()),
            hi_wall_hi0: HI_WALL_HI0.with(|x| x.get()),
        };
        reset();
        c
    }

    pub fn reset() {
        for k in [&CLOSE_REDUCED, &CLOSE_BLED, &CLOSE_FUEL_REDUCED, &CLOSE_FUEL_BLED,
                  &POWERS_REDUCED, &POWERS_BLED, &TAIL_REDUCED, &TAIL_BLED, &B_OF_CALLS,
                  &B_OF_CONSTANT, &B_OF_SCHED_ZERO, &B_OF_SCHED_OPEN, &AT_LEVER_CALLS,
                  &AT_STATOR_R62, &ISOLATING_CALLS, &LEGS_CALLS, &LEGS_LEVER_BLEED,
                  &CLOSE_BRACKET_FAILS, &CLOSE_FUEL_BRACKET_FAILS, &MARCH_IN_ADVANCES,
                  &FUEL_MARCH_IN_ADVANCES, &LO_FLOOR_HITS, &HI_WALL_LITERAL, &HI_WALL_MAP,
                  &HI_WALL_HI0] {
            k.with(|x| x.set(0));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// (1) THE Tt4-PINNED CLOSURE — rung 40's body with the extraction at station 25
// ---------------------------------------------------------------------------------------------

/// RUNG 62's rung-40 closure cell.
///
/// **THE REDUCE IS THE FIRST TWO LINES, AND IT IS THE WHOLE CONTRACT.** `b_of` is a pure function
/// of the live state, so a `0.0` hands straight back to rung 57's own body — which ARMS and then
/// calls rung 40's. Not "rung 40's body with `b = 0` substituted": *the same function object*, so
/// a valve-shut machine is rung 57 (hence rungs 43–52) bit-for-bit by DISPATCH rather than by
/// arithmetic. A [`BleedSchedule`] is exactly `0` at and above `n_ref`, so both arms fire inside
/// one march — measured 12 / 53.
pub fn r62_try_close(
    t: &TwoSpoolTransientCore, nu_lp: f64, nu_hp: f64, tt4: f64, tt2: f64, pt2: f64,
) -> Result<CloseState, Abort> {
    let b = t.b_of(nu_lp, Some(tt2));
    if b == 0.0 {
        bump(&CLOSE_REDUCED);
        return (R57_TWO.try_close)(t, nu_lp, nu_hp, tt4, tt2, pt2);
    }
    bump(&CLOSE_BLED);
    t.arm(nu_lp, nu_hp, tt2);
    let c = &t.inner;
    let gas = c.gas();
    let n_lp = nu_lp * powp(c.tt2_d / tt2, 0.5);
    let (h2, pr2) = (gas.h_c(tt2), gas.pr_c(tt2));

    let ev = |m_lp: f64| -> Result<CloseState, Abort> {
        let phi_lp = m_lp / n_lp;
        let tau_lpc = 1.0 + (c.tau_lpc_d - 1.0) * c.map_lp().psi(phi_lp) * n_lp * n_lp;
        let tt25 = tt2 * tau_lpc;
        let eta_lpc = c.map_lp().eta_c_at(c.base.eta_lpc, phi_lp, n_lp);
        let h25 = gas.h_c(tt25);
        let pi_lpc = gas.pr_c(gas.t_from_h_c(h2 + eta_lpc * (h25 - h2))) / pr2;
        let pt25 = pi_lpc * pt2;
        let mdot_face_trial = m_lp * c.mcorr_lp_d * pt2 / powp(tt2, 0.5);
        // THE EXTRACTION, at station 25.
        let mdot_core = (1.0 - b) * mdot_face_trial;

        // Same physical CORE flow, referred to the HP face (rung 40's line, with (1-b)).
        let m_hp = (mdot_core * powp(tt25, 0.5) / pt25) / c.mcorr_hp_d;
        let n_hp = nu_hp * powp(c.tt25_d / tt25, 0.5);
        let phi_hp = m_hp / n_hp;
        let tau_hpc = 1.0 + (c.tau_hpc_d - 1.0) * c.map_hp().psi(phi_hp) * n_hp * n_hp;
        let tt3 = tt25 * tau_hpc;
        let eta_hpc = c.map_hp().eta_c_at(c.base.eta_hpc, phi_hp, n_hp);
        let h3 = gas.h_c(tt3);
        let pi_hpc = gas.pr_c(gas.t_from_h_c(h25 + eta_hpc * (h3 - h25))) / gas.pr_c(tt25);
        let pt4 = c.base.pi_b * pi_hpc * pt25;

        let f = c.base.try_solve_f(tt3, pt4, tt4)?;
        let wgas = c.base.try_working_gas(f, tt4, pt4)?;
        let wg = wgas.as_ref().unwrap_or(gas);
        let mdot4 = c.base.a4 * pt4 * choked_mfp(wg, tt4, f) / powp(tt4, 0.5);
        // CORE air the NGV choke imposes, and the FACE flow that implies.
        let mdot_imp = mdot4 / (1.0 + f);
        let m_imp = (mdot_imp / (1.0 - b) * powp(tt2, 0.5) / pt2) / c.mcorr_lp_d;
        Ok(CloseState {
            m_lp, m_imp, m_hp, phi_lp, phi_hp, tt2, n_lp, n_hp, tau_lpc, tau_hpc, tt25, tt3,
            pi_lpc, pi_hpc, pt4, f, wgas, eta_lpc, eta_hpc, mdot_air: mdot_imp, mdot4,
            bleed: Some(b),
            // **NOT `mdot_face_trial`.** Python's dict key `mdot_face` is `mdot_imp/(1-b)`, the
            // IMPOSED face flow; the local of the same name three lines up is the m_lp-derived
            // TRIAL. They agree only at the root, and the shadowing is Python's own.
            mdot_face: Some(mdot_imp / (1.0 - b)),
        })
    };

    // The off-map guard, rung 40's in full: **Rust returns NaN where Python returns a COMPLEX**,
    // so the port's test is Python's `r == r` inverted.
    let g = |m: f64| -> Result<f64, Abort> {
        let r = m - ev(m)?.m_imp;
        if r.is_nan() {
            return Err(Abort(format!(
                "off-map compressor trial at m_lp={m:.4}: the loading law has gone non-physical \
                 (Tt3 < 0 => a complex pressure ratio).")));
        }
        Ok(r)
    };

    let wall_map = c.map_lp().phi_max(0.1) * n_lp;
    if 2.5 <= wall_map { bump(&HI_WALL_LITERAL); } else { bump(&HI_WALL_MAP); }
    let hi = 2.5f64.min(wall_map);
    // Python evaluates `ghi = g(hi)` OUTSIDE the try: a failure at the high wall PROPAGATES.
    let ghi = g(hi)?;

    let (mut lo, mut glo, mut m) = (None, 0.0f64, 0.02f64);
    while m < hi {
        match g(m) {
            Ok(v) => { glo = v; lo = Some(m); break; }
            Err(_) => { bump(&MARCH_IN_ADVANCES); m += 0.02; }
        }
    }
    let Some(lo) = lo.filter(|_| glo < 0.0 && 0.0 < ghi) else {
        bump(&CLOSE_BRACKET_FAILS);
        return Err(Abort(format!(
            "rung-62 bled two-shaft closure does not bracket at nu=({nu_lp:.4},{nu_hp:.4}), \
             Tt4={tt4:.0}, b={b:.4} — off the modeled speed-line region.")));
    };
    let root = try_illinois(g, lo, hi, glo, ghi, TwoSpoolTransientCore::CLOSE_TOL,
                            ILLINOIS_MAXIT)?;
    ev(root)
}

// ---------------------------------------------------------------------------------------------
// (2) THE FUEL CLOSURE — `Tt4` an OUTPUT, and the ONE place the bleed moves the CONTROL
// ---------------------------------------------------------------------------------------------

/// RUNG 62's rung-43 closure cell.
///
/// **THE BLEED CHANGES THE CONTROL HERE, NOT JUST THE FLOW.** The burner never sees the dumped
/// air, so a metered fuel flow makes a RICHER mixture: `f = mdot_fuel / mdot_core`, where rung 43
/// divides by the FACE flow. That one line is the whole of rung 63's structural channel — it is
/// the LP shaft balance carrying `(1-b)` that moves `Tt25`, and `Tt25` sits upstream of both of
/// the `Wf/pt3` leg's protections.
pub fn r62_try_close_fuel(
    ft: &FuelTransientCore, nu_lp: f64, nu_hp: f64, mdot_fuel: f64, tt2: f64, pt2: f64,
) -> Result<FuelCloseState, Abort> {
    let b = ft.inner.b_of(nu_lp, Some(tt2));
    if b == 0.0 {
        bump(&CLOSE_FUEL_REDUCED);
        return (R57_FUEL.try_close_fuel)(ft, nu_lp, nu_hp, mdot_fuel, tt2, pt2);
    }
    bump(&CLOSE_FUEL_BLED);
    ft.inner.arm(nu_lp, nu_hp, tt2);
    let c = &ft.inner.inner;
    let gas = c.gas();
    let n_lp = nu_lp * powp(c.tt2_d / tt2, 0.5);
    let (h2, pr2) = (gas.h_c(tt2), gas.pr_c(tt2));

    let ev = |m_lp: f64| -> Result<FuelCloseState, Abort> {
        let phi_lp = m_lp / n_lp;
        let tau_lpc = 1.0 + (c.tau_lpc_d - 1.0) * c.map_lp().psi(phi_lp) * n_lp * n_lp;
        let tt25 = tt2 * tau_lpc;
        let eta_lpc = c.map_lp().eta_c_at(c.base.eta_lpc, phi_lp, n_lp);
        let h25 = gas.h_c(tt25);
        let pi_lpc = gas.pr_c(gas.t_from_h_c(h2 + eta_lpc * (h25 - h2))) / pr2;
        let pt25 = pi_lpc * pt2;
        let mdot_face_trial = m_lp * c.mcorr_lp_d * pt2 / powp(tt2, 0.5);
        let mdot_core = (1.0 - b) * mdot_face_trial;

        let m_hp = (mdot_core * powp(tt25, 0.5) / pt25) / c.mcorr_hp_d;
        let n_hp = nu_hp * powp(c.tt25_d / tt25, 0.5);
        let phi_hp = m_hp / n_hp;
        let tau_hpc = 1.0 + (c.tau_hpc_d - 1.0) * c.map_hp().psi(phi_hp) * n_hp * n_hp;
        let tt3 = tt25 * tau_hpc;
        let eta_hpc = c.map_hp().eta_c_at(c.base.eta_hpc, phi_hp, n_hp);
        let h3 = gas.h_c(tt3);
        let pi_hpc = gas.pr_c(gas.t_from_h_c(h25 + eta_hpc * (h3 - h25))) / gas.pr_c(tt25);
        let pt4 = c.base.pi_b * pi_hpc * pt25;

        // THE ONE PLACE THE BLEED CHANGES THE CONTROL — `mdot_core`, not the face flow.
        let f = mdot_fuel / mdot_core;
        let tt4 = ft.try_tt4_from_f(tt3, f)?;
        let wgas = c.base.try_working_gas(f, tt4, pt4)?;
        let wg = wgas.as_ref().unwrap_or(gas);
        let mdot4 = c.base.a4 * pt4 * choked_mfp(wg, tt4, f) / powp(tt4, 0.5);
        let mdot_imp = mdot4 / (1.0 + f);
        let m_imp = (mdot_imp / (1.0 - b) * powp(tt2, 0.5) / pt2) / c.mcorr_lp_d;
        Ok(FuelCloseState {
            base: CloseState {
                m_lp, m_imp, m_hp, phi_lp, phi_hp, tt2, n_lp, n_hp, tau_lpc, tau_hpc, tt25, tt3,
                pi_lpc, pi_hpc, pt4, f, wgas, eta_lpc, eta_hpc, mdot_air: mdot_imp, mdot4,
                bleed: Some(b),
                mdot_face: Some(mdot_imp / (1.0 - b)),
            },
            tt4,
            // Python's `mdot_air_face` key — the m_lp-derived TRIAL face flow, a DIFFERENT
            // quantity from `mdot_face` above.
            mdot_air_face: mdot_face_trial,
        })
    };

    let g = |m: f64| -> Result<f64, Abort> {
        let r = m - ev(m)?.base.m_imp;
        if r.is_nan() {
            return Err(Abort(format!(
                "off-map compressor trial at m_lp={m:.4}: the loading law has gone non-physical \
                 (Tt3 < 0 => a complex pressure ratio).")));
        }
        Ok(r)
    };

    // Rung 43's scan-up-from-the-rich-wall bracket. **The `f` caps are CORE-referenced, so the
    // FACE-flow walls they imply carry `1/(1-b)`** — without it the scan starts INSIDE the
    // physical root at large `b`. That factor is rung 62's, not rung 43's.
    let lo0 = mdot_fuel * powp(tt2, 0.5)
        / (FuelTransientCore::F_CAP * (1.0 - b) * c.mcorr_lp_d * pt2);
    let hi0 = mdot_fuel * powp(tt2, 0.5)
        / (FuelTransientCore::F_FLOOR * (1.0 - b) * c.mcorr_lp_d * pt2);
    let wall_map = c.map_lp().phi_max(0.1) * n_lp;
    let mut cap = 2.5f64;
    let mut arm = &HI_WALL_LITERAL;
    if wall_map < cap { cap = wall_map; arm = &HI_WALL_MAP; }
    if hi0 < cap { cap = hi0; arm = &HI_WALL_HI0; }
    bump(arm);

    let mut m = lo0;
    if 0.02 > m { m = 0.02; bump(&LO_FLOOR_HITS); }

    let (mut lo, mut glo, mut hi, mut ghi) = (None, 0.0f64, None, 0.0f64);
    while m < cap {
        let gm = match g(m) {
            Ok(v) => v,
            Err(_) => {
                bump(&FUEL_MARCH_IN_ADVANCES);
                m += FuelTransientCore::MARCH_IN_STEP;
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
        m += FuelTransientCore::MARCH_IN_STEP;
    }
    let (Some(lo), Some(hi)) = (lo, hi) else {
        bump(&CLOSE_FUEL_BRACKET_FAILS);
        return Err(Abort(format!(
            "rung-62 bled fuel closure does not bracket at nu=({nu_lp:.4},{nu_hp:.4}), \
             mdot_fuel={mdot_fuel:.5}, b={b:.4} — off the modeled speed-line region.")));
    };
    let root = try_illinois(g, lo, hi, glo, ghi, TwoSpoolTransientCore::CLOSE_TOL,
                            ILLINOIS_MAXIT)?;
    ev(root)
}

// ---------------------------------------------------------------------------------------------
// (3) THE NEWTON'S INNER POWER LOOP — the touch point that BITES
// ---------------------------------------------------------------------------------------------

/// RUNG 62's rung-40 power cell.
///
/// **PYTHON'S OWN COMMENT CALLS THIS "THE TOUCH POINT THAT BITES", AND IT IS WHY THE CELL EXISTS
/// AT ALL.** Rung 40 factored `(Phi_L, Phi_H)` out of `_instant_tail` so the equilibrium Newton
/// would not rebuild the nozzle each step. Left bleed-free it still converges to 1e-12 — on a
/// residual **the plant does not use**: `n_L` comes back 5.3 % wrong with `phi_L` still agreeing
/// to 1e-3 and no exception anywhere. What catches it is rung 42's steady cross-check, not any
/// internal consistency.
///
/// **AND THE DISPATCH IS ON THE CLOSURE'S OWN `bleed`, NOT ON `b_of`.** Python reads
/// `c.get("bleed", 0.0)` — an ABSENT key on every rung-40/57 closure. Re-reading `b_of(nu_lp, …)`
/// here would agree on every state the suites reach and disagree in general, and **no value key
/// could see the difference** because the two branches coincide wherever `b` is 0. § 5.21 (v).
/// The counters are the gate.
fn r62_powers(
    t: &TwoSpoolTransientCore, c: &CloseState, flight: &FlightCondition, nu_lp: f64, nu_hp: f64,
    tt4: f64,
) -> Result<(f64, f64), Abort> {
    let b = c.bleed.unwrap_or(0.0);
    if b == 0.0 {
        bump(&POWERS_REDUCED);
        return (R57_TWO.powers)(t, c, flight, nu_lp, nu_hp, tt4);
    }
    bump(&POWERS_BLED);
    let core = &t.inner;
    let (wgas, f) = (c.gas(core), c.f);
    let nu_hpt = nu_hp * powp(core.tt4_d / tt4, 0.5);
    let (_, _, tt45) = core.base.try_solve_choked_turbine(
        wgas, tt4, f, core.base.a4, core.base.a45, 1.0,
        core.map_hp().eta_t_at(core.base.eta_hpt, nu_hpt))?;
    let nu_lpt = nu_lp * powp(core.tt45_d / tt45, 0.5);
    let (_, _, tt5) = core.base.try_solve_choked_turbine(
        wgas, tt45, f, core.base.a45, core.base.a8, core.base.pi_n,
        core.map_lp().eta_t_at(core.base.eta_lpt, nu_lpt))?;
    // HP: both sides are CORE flow, so (1-b) cancels — rung 42's bleed-INVARIANT form.
    let pt_hp = core.base.eta_m * (1.0 + f) * (wgas.h_t(tt4, f) - wgas.h_t(tt45, f));
    let pc_hp = wgas.h_c(c.tt3) - wgas.h_c(c.tt25);
    // LP: the LPT passes CORE gas while the LPC pumps FACE air — rung 42's (1).
    let pt_lp = core.base.eta_m * (1.0 - b) * (1.0 + f) * (wgas.h_t(tt45, f) - wgas.h_t(tt5, f));
    let pc_lp = wgas.h_c(c.tt25) - wgas.h_c(c.tt2);
    let mdot_face = c.mdot_face.expect("rung-62 _powers indexes c['mdot_face'] — a bled closure \
                                        always sets it, and Python KeyErrors if it does not");
    Ok(((mdot_face * (pt_lp - pc_lp)) / (t.p_ref_lp * nu_lp),
        (c.mdot_air * (pt_hp - pc_hp)) / (t.p_ref_hp * nu_hp)))
}

// ---------------------------------------------------------------------------------------------
// (4) THE TURBINE / POWER / THRUST TAIL
// ---------------------------------------------------------------------------------------------

/// RUNG 62's rung-40 tail cell. Same dispatch as [`r62_powers`], on the same absent key.
fn r62_try_instant_tail(
    t: &TwoSpoolTransientCore, flight: &FlightCondition, c: &CloseState, nu_lp: f64, nu_hp: f64,
    tt4: f64, v0: f64,
) -> Result<Instant2, Abort> {
    let b = c.bleed.unwrap_or(0.0);
    if b == 0.0 {
        bump(&TAIL_REDUCED);
        return (R57_TWO.try_instant_tail)(t, flight, c, nu_lp, nu_hp, tt4, v0);
    }
    bump(&TAIL_BLED);
    let core = &t.inner;
    let tt2 = c.tt2;
    let (wgas, f) = (c.gas(core), c.f);

    let nu_hpt = nu_hp * powp(core.tt4_d / tt4, 0.5);
    let eta_hpt = core.map_hp().eta_t_at(core.base.eta_hpt, nu_hpt);
    let (pi_hpt, tau_hpt, tt45) = core.base.try_solve_choked_turbine(
        wgas, tt4, f, core.base.a4, core.base.a45, 1.0, eta_hpt)?;
    let nu_lpt = nu_lp * powp(core.tt45_d / tt45, 0.5);
    let eta_lpt = core.map_lp().eta_t_at(core.base.eta_lpt, nu_lpt);
    let (pi_lpt, tau_lpt, tt5) = core.base.try_solve_choked_turbine(
        wgas, tt45, f, core.base.a45, core.base.a8, core.base.pi_n, eta_lpt)?;

    let mdot_core = c.mdot_air;
    let mdot_face = c.mdot_face.expect("rung-62 _instant_tail indexes c['mdot_face']");
    let pt_hp = core.base.eta_m * (1.0 + f) * (wgas.h_t(tt4, f) - wgas.h_t(tt45, f));
    let pc_hp = wgas.h_c(c.tt3) - wgas.h_c(c.tt25);
    let pt_lp = core.base.eta_m * (1.0 - b) * (1.0 + f) * (wgas.h_t(tt45, f) - wgas.h_t(tt5, f));
    let pc_lp = wgas.h_c(c.tt25) - wgas.h_c(tt2);

    let phi_hp_dot = (mdot_core * (pt_hp - pc_hp)) / (t.p_ref_hp * nu_hp);
    let phi_lp_dot = (mdot_face * (pt_lp - pc_lp)) / (t.p_ref_lp * nu_lp);

    let s5 = FlowState { tt: tt5, pt: pi_lpt * pi_hpt * c.pt4, mdot: mdot_core, far: f };
    let exit = Nozzle::convergent(core.base.p_ambient, core.base.pi_n).try_apply(&s5, wgas)?;
    let press = (1.0 + f) * wgas.r_t_at(f) * exit.t9 * (1.0 - flight.p0 / exit.p9) / exit.v9;
    let sp_thrust = (1.0 + f) * exit.v9 - v0 + press;

    Ok(Instant2 {
        close: c.clone(),
        nu_lp, nu_hp, tt4, slip: nu_lp / nu_hp, phi_lp_dot, phi_hp_dot, pt_lp, pt_hp, pc_lp, pc_hp,
        tt45, tt5, tau_hpt, tau_lpt, pi_hpt, pi_lpt, eta_hpt, eta_lpt, nu_hpt, nu_lpt, sp_thrust,
        // rung 42's (3): the dumped air carries FULL ram drag and returns no exhaust momentum.
        // `sp_thrust` stays CORE-referenced (bit-for-bit at b = 0); this is the honest
        // per-INLET-air figure beside it, and the key rung 40's dict does not have.
        sp_thrust_inlet: Some((1.0 - b) * sp_thrust - b * v0),
        m9: exit.m9,
        branch: if exit.p9 > core.base.p_ambient + 1e-6 { Branch::Choked } else { Branch::Subsonic },
    })
}

// ---------------------------------------------------------------------------------------------
// (5) THE LEVER CELLS — the state, the sibling constructors, and § 5.21 (ii)'s override
// ---------------------------------------------------------------------------------------------

/// RUNG 62's `_armed_bleed`.
fn r62_armed_bleed(t: &TwoSpoolTransientCore) -> bool {
    t.lever.bleed != 0.0 || t.lever.sched.is_some()
}

/// RUNG 62's `b_of` — the valve position this machine holds at the given state.
///
/// `tt2 = None` reads against the DESIGN `Tt2`. **Every reader goes through this rather than
/// through the stored `bleed`**, which for a scheduled machine is `0.0` and means nothing.
fn r62_b_of(t: &TwoSpoolTransientCore, nu_lp: f64, tt2: Option<f64>) -> f64 {
    bump(&B_OF_CALLS);
    let Some(sched) = t.lever.sched else {
        bump(&B_OF_CONSTANT);
        return t.lever.bleed;
    };
    let t2 = tt2.unwrap_or(t.inner.tt2_d);
    let b = sched.at(nu_lp * powp(t.inner.tt2_d / t2, 0.5));
    if b == 0.0 { bump(&B_OF_SCHED_ZERO); } else { bump(&B_OF_SCHED_OPEN); }
    b
}

/// RUNG 62's `at_lever` — a sibling on the SAME hardware and the same design references, BOTH
/// levers re-armed. Every difference rung 62 reports goes through it, so a swept setting can
/// never be confused with a re-designed engine.
fn r62_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    bump(&AT_LEVER_CALLS);
    match build_scheduled_bleed(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    }
}

/// RUNG 62's `at_stator` — **rung 57's sibling constructor, overridden so it carries THIS
/// machine's valve.**
///
/// Rung 57 hard-constructs a BARE machine, and `stator_credit` / `credit_decomposition` /
/// `arrow_toggle` all route their bare leg through it. Left un-overridden, every one of those
/// would have differenced an armed machine against a VALVE-SHUT bare one and silently attributed
/// the valve's whole effect to the stator — rung 61's `at_setting` trap, one ladder over.
///
/// **AND A SHIPPED GATE READS IT.** On a bleed-armed machine the inherited rung-59 reader
/// `schedule_invariance` derives the `Wf/pt3` table on `self` and on `self.at_stator()` — the
/// SAME bleed-armed machine — and returns `ordinate_identical = true`, numerically identical to
/// rung 59's headline result while measuring nothing at all.
/// `test_the_at_stator_trap_is_real_and_returns_rung59s_zero_for_free` pins that counterfeit, and
/// it is the reason every rung-63 reader is built on `_isolating` instead.
fn r62_at_stator(core: &ScheduledStatorCore, arm: StatorArm) -> ScheduledStatorCore {
    bump(&AT_STATOR_R62);
    let lever = LeverArm {
        stator: arm,
        bleed: core.fuel.inner.lever.bleed,
        bleed_sched: core.fuel.inner.lever.sched,
        // `None`, and DELIBERATELY: rung 62's body has no `bleed_lim` at all, so a rung-64
        // machine reaching it would silently lose its floor. That is precisely the trap rung
        // 64's OWN `at_stator` override exists to close — Python's "fourth instance", after
        // rung 61's `at_setting`, rung 62's `at_stator` and rung 63's `_isolating`. Carrying
        // `self`'s limiter here instead would make rung 64's override a no-op and its gate
        // pass for the wrong reason.
        bleed_lim: None,
    };
    r62_at_lever(core, &lever)
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — and the ONE spelling that would drop rung 57 silently
// ---------------------------------------------------------------------------------------------

/// RUNGS 62–63's own table.
pub const R62: LeverHooks = LeverHooks {
    at_lever: r62_at_lever,
    armed_bleed: r62_armed_bleed,
    b_of: r62_b_of,
    isolating: r62_isolating,
    legs: r62_legs,
    // Rung 62 has NO `b_at_point` in Python — the name does not exist below rung 64 — so this
    // slot keeps NO_LEVER's PANIC rather than answering `b_of`. Defaulting it to `b_of` would
    // be right on a rung-62 machine and wrong on a floored one, which is a claim no value gate
    // could see; § 5.22 (ii) is the measurement of exactly that mistake, made from the other
    // side.
    b_at_point: no_lever_b_at_point,
};

/// RUNG 62's swap into rung 40's table — **THREE cells, and `..R57_TWO` is load-bearing.**
///
/// Spelling this `..R40` would compile, pass every value gate that does not arm a stator, and
/// silently drop rung 57's arming from the one cell rung 62 does NOT override
/// (`try_surge_fuel`'s sibling on this table) — § 5.20 (v)'s carrier failure wearing a different
/// hat. The parent's table is what Python's `super()` reaches, so the parent's table is what the
/// spread must name.
pub const R62_TWO: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: r62_try_close,
    try_instant_tail: r62_try_instant_tail,
    powers: r62_powers,
};

/// RUNG 62's swap into rung 43's table — ONE cell. **`..R57_FUEL`, never `..R43`**: rung 62 does
/// not override `_surge_fuel`, so it must inherit rung 60's floor-resolving body, not rung 49's.
pub const R62_FUEL: FuelTransientHooks = FuelTransientHooks {
    try_close_fuel: r62_try_close_fuel,
    ..R57_FUEL
};

/// RUNG 62's swap into rung 57's table — ONE cell, the one slice V shipped without.
pub const R62_STATOR: StatorTransientHooks = StatorTransientHooks {
    at_stator: r62_at_stator,
    ..R57
};

// ---------------------------------------------------------------------------------------------
// THE CONSTRUCTOR
// ---------------------------------------------------------------------------------------------

/// RUNG 62's constructor — Python's `ScheduledBleedTransient.__init__`.
///
/// Returns [`ScheduledStatorTransient`] rather than a type of its own: § 5.21 (iii)'s decision,
/// and the module note's reason. The `lp_disabled` arm is rung 57's unchanged, since rung 62's
/// `__init__` forwards the flag and adds nothing to that path.
///
/// **THE ASSERT ORDER IS PYTHON'S.** `super().__init__(…)` runs FIRST, so rung 57's four
/// capture-discipline asserts fire before rung 62's two — which decides which message a caller
/// arming both a stator schedule and a constant stator AND a double valve actually sees.
pub fn build_scheduled_bleed(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = ScheduledStatorTransient::with_tables(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm.stator,
        &R62_TWO, &R62_STATOR, &R62_FUEL, &R62,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: None });
    // Rung 62's own two asserts, AFTER super()'s — see the note above.
    assert!(!(arm.bleed != 0.0 && arm.bleed_sched.is_some()),
            "rung-62: the valve gets a CONSTANT position or a SCHEDULE, not both -- they are the \
             two legs the rung differences (rung 57's discipline).");
    assert!((0.0..0.5).contains(&arm.bleed),
            "rung-42 bleed fraction must be in [0, 0.5): b>=0.5 starves the core and the choked \
             branch is long gone by then");
    built
}

// ---------------------------------------------------------------------------------------------
// (6) THE RUNG — the LOOP a state-fed schedule closes on itself
// ---------------------------------------------------------------------------------------------

impl LeverArm {
    /// Which of Python's `at_lever` KEYWORDS this arm sets — the list `_isolating`'s two refusals
    /// iterate over.
    ///
    /// **AND THE ONE PLACE A STRUCT IS NOT A DICT.** Python tests key PRESENCE; this tests
    /// non-defaultness, so `at_lever(bleed=0.0)` — a present key with a default value — reads as
    /// absent here and would be refused by `assert lever` where Python accepts it. No shipped
    /// caller passes a defaulted keyword (checked: rung 63's suite builds `BLEED`, `STAT` and
    /// `{}` only), and the divergence is DISCLOSED rather than papered over with a
    /// `HashMap<&str, ...>` that would make every reader allocate.
    pub fn keys(&self) -> Vec<&'static str> {
        let mut k = Vec::new();
        if self.stator.vsv_lp != 0.0 { k.push("vsv_lp"); }
        if self.stator.vsv_hp != 0.0 { k.push("vsv_hp"); }
        if self.stator.sched_lp.is_some() { k.push("vsv_sched_lp"); }
        if self.stator.sched_hp.is_some() { k.push("vsv_sched_hp"); }
        if self.bleed != 0.0 { k.push("bleed"); }
        if self.bleed_sched.is_some() { k.push("bleed_sched"); }
        if self.bleed_lim.is_some() { k.push("bleed_lim"); }
        k
    }
}

/// RUNG 63's `_isolating` — the `(reference, armed)` sibling PAIR that isolates `lever` in the
/// presence of `neighbour`, **and THE GATE that makes every rung-63 reader trustworthy**.
///
/// Rung 62 overrode `at_stator` on purpose, and that override reaches SIX inherited readers
/// (`stator_credit`, `credit_decomposition`, `composite_credit`, `engagement_shift`,
/// `schedule_invariance`, `matched_credit`). `schedule_invariance` is the one that bites: on a
/// bleed-armed machine it derives the `Wf/pt3` table on `self` and on `self.at_stator()` — the
/// SAME bleed-armed machine — and returns `ordinate_identical = true`, numerically identical to
/// rung 59's headline while measuring nothing. **Every rung-63 reader is therefore built here,
/// and rungs 58/59's own methods are left literally unchanged.**
fn r62_isolating(core: &ScheduledStatorCore, lever: &LeverArm, neighbour: Option<&LeverArm>)
    -> (ScheduledStatorCore, ScheduledStatorCore) {
    bump(&ISOLATING_CALLS);
    let empty = LeverArm::default();
    let nb = neighbour.unwrap_or(&empty);
    let lk = lever.keys();
    assert!(!lk.is_empty(), "rung-63 isolates a lever: pass one `at_lever` keyword");
    let nk = nb.keys();
    for k in &lk {
        assert!(!nk.contains(k),
                "rung-63: '{k}' is the LEVER being isolated, so the reference sibling must not \
                 also carry it -- that is exactly the armed-vs-armed comparison rung 62's \
                 `at_stator` override would have produced silently.");
    }
    let reference = core.at_lever(nb);
    let armed = core.at_lever(&LeverArm::merged(nb, lever));
    let want = nb.arms_valve();
    assert!(reference.armed_bleed() == want,
            "rung-63's reference sibling must carry the NEIGHBOUR's valve and nothing else; it \
             reports armed={} against neighbour={want}.", reference.armed_bleed());
    (reference, armed)
}

impl ScheduledStatorCore {
    /// RUNG 62's `_commanded` — the setting the armed schedule COMMANDS at the given point of a
    /// trajectory, **the loop witnessed directly** rather than inferred from a ratio of credits.
    ///
    /// `Tt2` is read from the flight condition, which is fixed along a ramp.
    pub fn commanded(&self, flight: &FlightCondition, traj: &[FuelPoint], s_at: f64, lever: Lever)
        -> f64 {
        // Python's `min(traj, key=…)` — FIRST minimum wins on a tie, which `min_by` also gives.
        let p = traj.iter()
            .min_by(|a, b| (a.s - s_at).abs().total_cmp(&(b.s - s_at).abs()))
            .expect("rung-62 _commanded needs a non-empty trajectory");
        let tt2 = self.fuel.inner.inlet(flight).0;
        match lever {
            Lever::Bleed => self.fuel.inner.b_of(p.nu_lp, Some(tt2)),
            Lever::Stator => self.v_of(Spool::Lp, p.nu_lp, p.nu_hp, Some(tt2)),
        }
    }
}

/// RUNG 62's `_legs` — rung 57's START / RAMP / FULL, **generalised to ANY reference machine**.
///
/// ```text
/// START-ONLY   armed running line, REFERENCE march
/// RAMP-ONLY    reference running line, ARMED march
/// FULL         both -- the machine as it actually runs
/// self_cancel  FULL / RAMP-ONLY
/// ```
///
/// Rung 57 hard-wired the reference to the bare machine, which is right for the one lever it
/// carried. Here the reference is a PARAMETER, because the rung's second finding needs a
/// NEIGHBOUR carried on BOTH sides of the difference — otherwise the difference is the pair, not
/// the lever.
///
/// **RUNG 63 threads ONE fuel-side min-select leg through all four marches**, so a lever's loop
/// can be measured with a LEGGED neighbour on both sides. An empty [`StatorLeg`] is
/// `_stator_march`'s own default, so every rung-62 caller reaches the IDENTICAL four marches:
/// THE REDUCE.
fn r62_legs(
    core: &ScheduledStatorCore, flight: &FlightCondition, reference: &ScheduledStatorCore,
    ramp: &Ramp, spool: Spool, leg: &StatorLeg<'_>,
) -> LegsReport {
    bump(&LEGS_CALLS);
    let (t_ref, nu0_r) = reference.stator_march(flight, ramp, None, leg);
    let r_ref = *reference.read(&t_ref, None).spool(spool);
    let eq = core.fuel.inner.equilibrium(flight, ramp.tt4_lo);
    let nu0_a = (eq.nu_lp, eq.nu_hp);
    let (t_start, _) = reference.stator_march(flight, ramp, Some(nu0_a), leg);
    let (t_ramp, _) = core.stator_march(flight, ramp, Some(nu0_r), leg);
    let (t_full, _) = core.stator_march(flight, ramp, Some(nu0_a), leg);
    let base = r_ref.m_i;
    let r_ramp = *core.read(&t_ramp, None).spool(spool);
    let r_full = *core.read(&t_full, None).spool(spool);
    let start = reference.read(&t_start, None).spool(spool).m_i - base;
    let ramp_only = r_ramp.m_i - base;
    let full = r_full.m_i - base;
    let lever = if core.armed_bleed() {
        bump(&LEGS_LEVER_BLEED);
        Lever::Bleed
    } else {
        Lever::Stator
    };
    LegsReport {
        spool,
        r: ramp.r,
        reference: base,
        start,
        ramp: ramp_only,
        full,
        self_cancel: if ramp_only != 0.0 { full / ramp_only } else { f64::NAN },
        surrendered: if ramp_only != 0.0 { 1.0 - full / ramp_only } else { f64::NAN },
        share_start: if full != 0.0 { start / full } else { f64::NAN },
        loop_: (full - ramp_only) - start,
        nu0_ref: nu0_r.0,
        nu0_armed: nu0_a.0,
        cmd_ramp: core.commanded(flight, &t_ramp, r_ramp.at().s, lever),
        cmd_full: core.commanded(flight, &t_full, r_full.at().s, lever),
        s_ref: r_ref.at().s,
        s_ramp: r_ramp.at().s,
        s_full: r_full.at().s,
        lever,
    }
}

// ---------------------------------------------------------------------------------------------
// RUNG 62's READERS
// ---------------------------------------------------------------------------------------------

/// One row of [`ScheduledStatorCore::loop_factors`] — the two derivatives the headline's SIGN
/// argument rests on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopFactor {
    pub tt4: f64,
    pub n_bare: f64,
    pub dn_db: f64,
    pub dn_dv: f64,
    pub sign_bleed: i32,
    pub sign_stator: i32,
}

/// [`ScheduledStatorCore::pair_interaction`] — the four-cell interaction of two levers on ONE
/// accelerating machine, in BOTH currencies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PairInteraction {
    pub spool: Spool,
    pub r: f64,
    pub credit_a: f64,
    pub credit_b: f64,
    pub credit_pair: f64,
    pub credit_sum: f64,
    pub interaction: f64,
    pub interaction_frac: f64,
    /// **RETURNED RAW, AND THE RATIO IS DELIBERATELY ABSENT.** `cost_bleed` is negative while
    /// `cost_stator` is positive, so a normalised interaction would have a difference of
    /// opposite-signed terms in its denominator — rung 43's currency-circularity trap.
    pub cost_a: f64,
    pub cost_b: f64,
    pub cost_pair: f64,
    pub cost_interaction: f64,
}

/// One row of [`ScheduledStatorCore::clock_sweep`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClockRow {
    pub r: f64,
    pub bare: f64,
    pub credit: f64,
    pub per_setting: f64,
}

/// [`ScheduledStatorCore::commanded_level`] — what this machine's schedule actually commands over
/// the ramp.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CommandedLevel {
    pub lever: Lever,
    pub at_min: f64,
    pub mean: f64,
    pub peak: f64,
    pub s_min: f64,
}

impl ScheduledStatorCore {
    /// **THE HEADLINE (rung 62).** Rung 57's decomposition against the BARE machine, for whichever
    /// lever this one carries.
    ///
    /// Rung 57 measured `FULL/RAMP` = 0.754–0.896 for a stator schedule and named the mechanism:
    /// closing the stators raises the speed at fixed power, the schedule reads the higher `n` and
    /// opens back up — loop gain `(dn/dv)(dv/dn) = (+)(-) < 0`. For a handling bleed BOTH factors
    /// flip one sign (rung 61 § 2's `dn_L/db < 0`, and an open-at-low-speed schedule has
    /// `db/dn_L < 0`), so the product is POSITIVE and `self_cancel > 1`: **the schedule AMPLIFIES
    /// itself.** `cmd_ramp`/`cmd_full` witness the loop directly.
    pub fn loop_decomposition(&self, flight: &FlightCondition, ramp: &Ramp, spool: Spool)
        -> LegsReport {
        assert!(self.arming().is_armed() || self.armed_bleed(),
                "rung-62 loop_decomposition needs an armed machine to decompose.");
        let reference = self.bare_lever();
        self.legs(flight, &reference, ramp, spool, &StatorLeg::default())
    }

    /// **THE SECOND FINDING (rung 62).** One lever's OWN loop, measured with a NEIGHBOUR carried
    /// on both sides of the difference.
    ///
    /// Comparing a PAIR's composite `self_cancel` against the two singles' does NOT test this —
    /// the composite is a credit-weighted blend of two different quantities, and that distinction
    /// is why rung 62's P3 scored REFUTED rather than confirmed.
    ///
    /// **RUNG 63**: the neighbour may instead be a FUEL-SIDE min-select `leg`, carried on both
    /// sides the same way. A leg has no state-feed of its own — it reads the state but emits a
    /// fuel cap, not a setting that re-enters through `dn/d(setting)` — so it is the control for
    /// *"does a loop answer to its neighbour's LOOP, or merely to its neighbour's trajectory?"*
    pub fn marginal_loop(
        &self, flight: &FlightCondition, ramp: &Ramp, lever: &LeverArm,
        neighbour: Option<&LeverArm>, spool: Spool, leg: &StatorLeg<'_>,
    ) -> LegsReport {
        let (reference, armed) = self.isolating(lever, neighbour);
        armed.legs(flight, &reference, ramp, spool, leg)
    }

    /// What this machine's schedule actually commands over the ramp — the value at its own surge
    /// minimum and the trajectory mean. **Without it, [`marginal_loop`](Self::marginal_loop)'s
    /// constant leg is comparing a schedule against a strictly larger lever and proves nothing.**
    pub fn commanded_level(&self, flight: &FlightCondition, ramp: &Ramp, spool: Spool)
        -> CommandedLevel {
        let (traj, _) = self.stator_march(flight, ramp, None, &StatorLeg::default());
        let rd = *self.read(&traj, None).spool(spool);
        let lever = if self.armed_bleed() { Lever::Bleed } else { Lever::Stator };
        let tt2 = self.fuel.inner.inlet(flight).0;
        let vals: Vec<f64> = traj.iter().map(|p| match lever {
            Lever::Bleed => self.fuel.inner.b_of(p.nu_lp, Some(tt2)),
            Lever::Stator => self.v_of(Spool::Lp, p.nu_lp, p.nu_hp, Some(tt2)),
        }).collect();
        CommandedLevel {
            lever,
            at_min: self.commanded(flight, &traj, rd.at().s, lever),
            // Python's `sum(vals)/len(vals)` — left-to-right accumulation, which is what a plain
            // `iter().sum()` also gives.
            //
            // **AND THAT IS TRUE OF PyPy AND FALSE OF CPython 3.12+**, measured by slice W's
            // step-4 oracle: CPython's `sum()` uses Neumaier COMPENSATED summation for floats,
            // so on a constant valve it returns exactly `0.1` where PyPy (and this line)
            // accumulate to three ULPs below it. This crate matches PyPy, which is the project's
            // interpreter and the one every golden is generated on; `slice_w_oracle.rs` carries
            // the nine `D/cl/*/mean` keys as its single declared cross-interpreter exemption,
            // seven of which actually differ. No shipped gate reads `mean`
            // (`test_rung62.py:374` reads `at_min`), so nothing downstream turns on it.
            mean: vals.iter().sum::<f64>() / vals.len() as f64,
            // Python's `max` — FIRST maximum on a tie, and it PROPAGATES NaN differently from
            // `f64::max`. `fold` with `>` reproduces the comparison chain exactly.
            peak: vals.iter().fold(f64::NEG_INFINITY, |a, &b| if b > a { b } else { a }),
            s_min: rd.at().s,
        }
    }

    /// The two derivatives the headline's SIGN argument rests on, **measured rather than quoted**:
    /// `dn_L/db` and `dn_L/dv` on the steady running line at each throttle.
    ///
    /// The check that matters is that NEITHER REVERSES over the band — rung 42's own `dphi_H/db`
    /// passes through zero at `pi* = 3.24674` and reverses below, so a sign argument in this
    /// machine is not safe without looking.
    pub fn loop_factors(&self, flight: &FlightCondition, tt4_grid: &[f64], db: f64, dv: f64)
        -> Vec<LoopFactor> {
        tt4_grid.iter().map(|&tt4| {
            let n0 = self.bare_lever().fuel.inner.equilibrium(flight, tt4).close.n_lp;
            let nb = self.at_lever(&LeverArm::constant(db))
                .fuel.inner.equilibrium(flight, tt4).close.n_lp;
            let nv = self.at_lever(&LeverArm::stator(StatorArm::constant(dv, 0.0)))
                .fuel.inner.equilibrium(flight, tt4).close.n_lp;
            LoopFactor {
                tt4,
                n_bare: n0,
                dn_db: (nb - n0) / db,
                dn_dv: (nv - n0) / dv,
                sign_bleed: if nb < n0 { -1 } else { 1 },
                sign_stator: if nv > n0 { 1 } else { -1 },
            }
        }).collect()
    }

    /// The four-cell interaction of two levers on ONE accelerating machine, in BOTH currencies:
    /// the incidence credit `M_i` and the shaft-speed cost (peak `nu_L`).
    ///
    /// Rung 61 ran this on the STEADY matcher and found the credits additive to ≤ 2.3 % with an
    /// adverse SPEED interaction in all 30 rows. Here the credit interaction is 8× larger, and the
    /// reason is the shared speed STATE that a steady matcher re-solves.
    pub fn pair_interaction(
        &self, flight: &FlightCondition, ramp: &Ramp, lever_a: &LeverArm, lever_b: &LeverArm,
        spool: Spool,
    ) -> PairInteraction {
        let pair = LeverArm::merged(lever_a, lever_b);
        let mut cells = [(0.0f64, 0.0f64); 4];
        for (i, kw) in [&LeverArm::default(), lever_a, lever_b, &pair].into_iter().enumerate() {
            let m = self.at_lever(kw);
            let eq = m.fuel.inner.equilibrium(flight, ramp.tt4_lo);
            let (traj, _) = m.stator_march(flight, ramp, Some((eq.nu_lp, eq.nu_hp)),
                                           &StatorLeg::default());
            let peak = traj.iter().fold(f64::NEG_INFINITY,
                                        |a, p| if p.nu_lp > a { p.nu_lp } else { a });
            cells[i] = (m.read(&traj, None).spool(spool).m_i, peak);
        }
        let (bare, a, b, p) = (cells[0], cells[1], cells[2], cells[3]);
        let (c_a, c_b, c_p) = (a.0 - bare.0, b.0 - bare.0, p.0 - bare.0);
        let (n_a, n_b, n_p) = (a.1 - bare.1, b.1 - bare.1, p.1 - bare.1);
        let s = c_a + c_b;
        PairInteraction {
            spool,
            r: ramp.r,
            credit_a: c_a,
            credit_b: c_b,
            credit_pair: c_p,
            credit_sum: s,
            interaction: c_p - s,
            interaction_frac: if s != 0.0 { (c_p - s) / s } else { f64::NAN },
            cost_a: n_a,
            cost_b: n_b,
            cost_pair: n_p,
            cost_interaction: n_p - (n_a + n_b),
        }
    }

    /// Credit per unit CONSTANT setting against ramp rate — rung 57's invariance test, run on
    /// whichever lever `lever` arms.
    ///
    /// **REPORTED AS A CONTROL, NOT A FINDING** (`docs/rung62-spec.md` § 0). The signature to read
    /// is MONOTONICITY, not the size of the swing: a wall-mover's floor channel contributes
    /// exactly `v` whatever the trajectory does, while a point-mover's entire credit runs through
    /// `phi` and inherits the trajectory's own ramp-rate dependence.
    pub fn clock_sweep(
        &self, flight: &FlightCondition, ramp: &Ramp, lever: &LeverArm, setting: f64, rates: &[f64],
        spool: Spool,
    ) -> Vec<ClockRow> {
        let bare = self.bare_lever();
        rates.iter().map(|&r| {
            let rr = ramp.with_r(r);
            let (t0, _) = bare.stator_march(flight, &rr, None, &StatorLeg::default());
            let base = bare.read(&t0, None).spool(spool).m_i;
            let m = self.at_lever(lever);
            let eq = m.fuel.inner.equilibrium(flight, rr.tt4_lo);
            let (t, _) = m.stator_march(flight, &rr, Some((eq.nu_lp, eq.nu_hp)),
                                        &StatorLeg::default());
            let credit = m.read(&t, None).spool(spool).m_i - base;
            ClockRow { r, bare: base, credit, per_setting: credit / setting }
        }).collect()
    }
}

// =============================================================================================
// RUNG 63 — FUEL + BLEED on one plant. Rung 62's named seam.
//
// Rung 58 measured a ONE-WAY arrow between a variable stator and a `Wf/pt3` accel leg: the leg
// moved the stator's credit by +9.51 %, the stator moved the leg's engagement time by −0.162 %
// — a factor of 59. Rung 59 then explained the small number exactly. The leg senses TWO things
// and the stator reaches NEITHER:
//
//     ORDINATE  kappa_ss = Wf/pt3 = pi_b*f(Tt3,Tt4)*MFP_A4 / [(1+f)*sqrt(Tt4)].  A4 is CHOKED so
//               MFP_A4 is hardware; Tt3 is pinned by two MAP-FREE shaft balances, so
//               kappa_ss = kappa_ss(Tt4) alone.                       [rung 59's proof chain]
//     ABSCISSA  n_H(Tt4): the HP-face corrected flow carries pt4 ~ pi_LPC over pt25 ~ pi_LPC, so
//               pi_LPC CANCELS.                                       [rung 39's ONE arrow]
//
// A BLEED BREAKS BOTH, and the algebra says exactly where. Of the two shaft balances only the LP
// one carries the valve (`r62_powers`: the HP has core flow on both sides, so (1-b) cancels):
//
//     dh_LPC = eta_m*(1-b)*(1+f)*dh_LPT   =>  Tt25 FALLS with b
//     dh_HPC = eta_m*(1+f)*dh_HPT         =>  Tt3 falls by the SAME enthalpy
//                                         =>  f RISES  =>  kappa_ss RISES   (the ORDINATE)
//     and m_hp ~ sqrt(Tt25)*pi_HPC/(1+f)  =>  n_H(Tt4) MOVES               (the ABSCISSA)
//
// `pi_LPC` still cancels out of `m_hp`: rung 39's arrow is not repealed. What moves the abscissa
// is that the bleed moves `Tt25` ITSELF, which no stator can do. **The valve is the ladder's only
// lever that breaks `mdot_face == mdot_core`, and that identity sits UPSTREAM of both
// protections.**
//
// SCOPE OF THAT CLAIM — it is about the TABLE, and this rung got it wrong twice by over-reaching
// from it. `kappa_ss` and `n_H(Tt4)` are STEADY properties; `s_eng` is a property of the
// TRAJECTORY through them, and a stator moves the trajectory with its table bit-identical (up to
// +1.28 % measured). So *"a stator cannot re-time the leg"* is FALSE; what holds is that the
// bleed's channel is STRUCTURAL and the stator's is trajectory-mediated.
// =============================================================================================

/// One plant's three terms of `g = Wf_sched - (1+m)*kappa(n_H)*pt3`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CapRow {
    pub s: f64,
    pub n_hp: f64,
    pub pt3: f64,
    pub cap: f64,
    pub kappa: f64,
    pub mf_sched: f64,
    pub g: f64,
}

/// [`ScheduledStatorCore::cap_channels`] — the re-timing's sign ATTRIBUTED rather than asserted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CapChannels {
    pub reference: CapRow,
    pub armed: CapRow,
    pub s_at: f64,
    pub d_kappa: f64,
    pub d_pt3: f64,
    pub d_cap: f64,
    pub d_mf_sched: f64,
    pub d_g: f64,
}

/// **THE RUNG (63)** — [`ScheduledStatorCore::leg_retiming`]'s return.
#[derive(Clone, Debug, PartialEq)]
pub struct LegRetiming {
    pub r: f64,
    pub ds: f64,
    pub leg: LegKind,
    pub ref_limited: f64,
    pub ref_dormant: f64,
    pub armed_limited: f64,
    pub armed_dormant: f64,
    pub audits: Option<(ClampAudit, ClampAudit)>,
    pub d_limited: f64,
    pub d_dormant: f64,
    pub rel_limited: f64,
    pub rel_dormant: f64,
    pub channels: Option<CapChannels>,
}

/// One row of [`ScheduledStatorCore::sensed_inputs`]'s proof-chain difference.
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

/// [`ScheduledStatorCore::sensed_inputs`] — rung 59's `schedule_invariance` **with a GENUINELY
/// BARE reference**.
#[derive(Clone, Debug, PartialEq)]
pub struct SensedInputs {
    pub reference: AccelSchedule,
    pub armed: AccelSchedule,
    pub chain: Vec<ChainRow>,
    pub ordinate_identical: bool,
    pub abscissa_identical: bool,
    pub d_ordinate: f64,
    pub d_abscissa: f64,
    pub signed_ordinate: f64,
    pub signed_abscissa: f64,
    /// The control that must stay at MACHINE ZERO for ANY lever: `A4` is choked, so the corrected
    /// group is hardware and nothing on the compressor side can reach it. If it moves, the chain
    /// has broken somewhere else and the rest is meaningless.
    pub d_mfp: f64,
}

/// [`ScheduledStatorCore::matched_leg_deltas`] — rung 59's SPLICE, for a lever that moves BOTH
/// halves of the table.
#[derive(Clone, Debug, PartialEq)]
pub struct MatchedLegDeltas {
    pub spool: Spool,
    pub r: f64,
    pub ds: f64,
    pub margin: f64,
    pub bare_leg: CellRead,
    pub matched: CellRead,
    pub reindexed: CellRead,
    pub revalued: CellRead,
    /// **THE SHARES ARE DELIBERATELY NOT RETURNED.** With the two halves opposite in sign,
    /// `delta_match` is a small difference of two larger terms and the shares move by ~10 % under
    /// an `ds` halving while their sum barely moves — rung 43's currency-circularity shape. The
    /// three RAW deltas carry the claim, and `delta_index` is the grid-robust member.
    pub delta_match: f64,
    pub delta_index: f64,
    pub delta_value: f64,
    pub clamped: usize,
}

/// [`ScheduledStatorCore::lever_composite`] — rung 58's four-cell mixed second difference, built
/// on `at_lever` siblings so it can isolate the VALVE.
#[derive(Clone, Debug, PartialEq)]
pub struct LeverComposite {
    pub spool: Spool,
    pub r: f64,
    pub ds: f64,
    pub leg: LegKind,
    pub neither: CellRead,
    pub lever: CellRead,
    pub fuel: CellRead,
    pub both: CellRead,
    pub credit_bare: f64,
    pub credit_fuel: f64,
    pub interaction: f64,
    pub share: f64,
    pub predicted: f64,
    pub profile_bare: f64,
    pub profile_fuel: f64,
    pub recovered: f64,
    pub relocation: f64,
    pub relocation_bare: f64,
    pub removed_bare: f64,
    pub removed_armed: f64,
}

/// One set-point row of [`ScheduledStatorCore::floor_dichotomy`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloorRow {
    pub sm: f64,
    pub phi_lim: f64,
    pub m_i_fuel: f64,
    pub m_i_both: f64,
    pub min_phi_fuel: f64,
    pub min_phi_both: f64,
    pub removed_fuel: f64,
    pub removed_both: f64,
    pub credit: f64,
    /// DORMANT on the armed plant means **BIT-FOR-BIT its own leg-free march** — the strongest
    /// available witness, and the one a tolerance would blur.
    pub disarmed: bool,
}

/// [`ScheduledStatorCore::floor_dichotomy`] — rung 49's `phi` floor beside the valve, swept.
#[derive(Clone, Debug, PartialEq)]
pub struct FloorDichotomy {
    pub spool: Spool,
    pub r: f64,
    pub ds: f64,
    pub phi_surge: f64,
    pub min_phi_ref: f64,
    pub min_phi_armed: f64,
    pub band: (f64, f64),
    pub rows: Vec<FloorRow>,
}

impl ScheduledStatorCore {
    /// **THE RUNG (63).** Rung 58's `engagement_shift`, on a lever the leg can FEEL.
    ///
    /// Sub-grid engagement time on the reference and the armed plant, on BOTH the limited march
    /// and the DORMANT one — the dormant leg is where `g` is defined everywhere and no clip has
    /// yet perturbed the states, so it is the clean reading, and the two agree to 6 decimals.
    /// ONE leg object is used on both plants (rung 58's discipline).
    ///
    /// A bleed schedule moves this by +2.9 to +4.2 %, LATER, at every ramp rate and on both map
    /// shapes. A STATOR moves it too (up to +1.28 %) even though its TABLE is bit-identical —
    /// `s_eng` is a TRAJECTORY quantity.
    ///
    /// **THE SIGN IS NOT THE OBVIOUS ONE** and `channels` says why: the bleed LOWERS `pt3` (which
    /// would engage the leg EARLIER) but RAISES `kappa(n_H)` through the abscissa shift, and the
    /// two nearly cancel in the cap. What decides the sign is the third term — the COMMANDED fuel
    /// ramp, re-derived on the bled plant, falls further than the cap does, so the crossing
    /// arrives LATER.
    #[allow(clippy::too_many_arguments)]
    pub fn leg_retiming(
        &self, flight: &FlightCondition, ramp: &Ramp, lever: &LeverArm, leg: &StatorLeg<'_>,
        neighbour: Option<&LeverArm>,
    ) -> LegRetiming {
        let kind = leg.one();
        let (reference, armed) = self.isolating(lever, neighbour);
        let dormant = StatorLeg::default();
        let mut out = [0.0f64; 4];
        let mut audits: Vec<ClampAudit> = Vec::new();
        for (i, mach) in [&reference, &armed].into_iter().enumerate() {
            for (j, l) in [leg, &dormant].into_iter().enumerate() {
                let (traj, _) = mach.stator_march(flight, ramp, None, l);
                out[i * 2 + j] =
                    ScheduledStatorCore::s_eng(&mach.leg_residual(flight, &traj, leg));
                if j == 0 {
                    if let Some(a) = leg.accel {
                        audits.push(mach.clamp_audit(flight, &traj, a));
                    }
                }
            }
        }
        let (ref_limited, ref_dormant, armed_limited, armed_dormant) =
            (out[0], out[1], out[2], out[3]);
        let d_lim = armed_limited - ref_limited;
        let d_dor = armed_dormant - ref_dormant;
        LegRetiming {
            r: ramp.r,
            ds: ramp.ds,
            leg: kind,
            ref_limited, ref_dormant, armed_limited, armed_dormant,
            audits: if audits.len() == 2 { Some((audits[0], audits[1])) }
                    else { None },
            d_limited: d_lim,
            d_dormant: d_dor,
            rel_limited: d_lim / ref_limited,
            rel_dormant: d_dor / ref_dormant,
            channels: leg.accel.map(|a| ScheduledStatorCore::cap_channels(
                flight, &reference, &armed, a, ramp, ref_dormant)),
        }
    }

    /// RUNG 63. The THREE terms of `g = Wf_sched - (1+m)*kappa(n_H)*pt3`, read on **both DORMANT
    /// marches at the REFERENCE plant's own engagement time**, so the sign of the re-timing is
    /// attributed rather than asserted.
    ///
    /// `mf_sched` is **not a constant across the two plants**, because `stator_march` pins both to
    /// the same `Tt4` endpoints (rung 35's apples-to-apples discipline) and a bled machine burns
    /// different fuel to reach them. That third term is what decides the sign.
    pub fn cap_channels(
        flight: &FlightCondition, reference: &ScheduledStatorCore, armed: &ScheduledStatorCore,
        accel: &AccelSchedule, ramp: &Ramp, s_at: f64,
    ) -> CapChannels {
        let row = |m: &ScheduledStatorCore| -> CapRow {
            let (traj, _) = m.stator_march(flight, ramp, None, &StatorLeg::default());
            let p = traj.iter()
                .min_by(|a, b| (a.s - s_at).abs().total_cmp(&(b.s - s_at).abs()))
                .expect("rung-63 cap_channels needs a non-empty march");
            let i = m.fuel.instant_fuel(flight, p.nu_lp, p.nu_hp, p.mf_sched);
            let pt3 = i.base.close.pt4 / m.fuel.inner.inner.base.pi_b;
            let cap = accel.cap(i.base.close.n_hp, pt3);
            CapRow {
                s: p.s,
                n_hp: i.base.close.n_hp,
                pt3,
                cap,
                kappa: cap / ((1.0 + accel.margin) * pt3),
                mf_sched: p.mf_sched,
                g: p.mf_sched - cap,
            }
        };
        let (a, b) = (row(reference), row(armed));
        CapChannels {
            reference: a,
            armed: b,
            s_at,
            d_kappa: b.kappa / a.kappa - 1.0,
            d_pt3: b.pt3 / a.pt3 - 1.0,
            d_cap: b.cap / a.cap - 1.0,
            d_mf_sched: b.mf_sched / a.mf_sched - 1.0,
            d_g: b.g - a.g,
        }
    }

    /// RUNG 63. Rung 59's `schedule_invariance` **with a GENUINELY BARE reference** — the
    /// `Wf/pt3` table derived on both plants and compared HALF BY HALF, plus the proof-chain
    /// residuals that say which factor carries the difference.
    ///
    /// Rung 59's verdicts, to compare against: an LP stator moves NEITHER half (both ≤ 1e-13, its
    /// own published tolerance); an HP stator moves ONLY the abscissa, ordinate exactly 0.
    pub fn sensed_inputs(
        &self, flight: &FlightCondition, ramp: &Ramp, lever: &LeverArm, margin: f64, n: usize,
        neighbour: Option<&LeverArm>,
    ) -> SensedInputs {
        let (reference, armed) = self.isolating(lever, neighbour);
        let l_ref = reference.fuel.accel_schedule(flight, ramp.tt4_lo, ramp.tt4_hi, margin, n);
        let l_arm = armed.fuel.accel_schedule(flight, ramp.tt4_lo, ramp.tt4_hi, margin, n);
        let mut chain = Vec::with_capacity(n);
        for k in 0..n {
            let tt4 = ramp.tt4_lo + (ramp.tt4_hi - ramp.tt4_lo) * k as f64 / (n as f64 - 1.0);
            let a = reference.proof_chain(flight, tt4);
            let b = armed.proof_chain(flight, tt4);
            let d = |x: f64, y: f64| (y - x) / x;
            chain.push(ChainRow {
                tt4,
                d_tt25: d(a.tt25, b.tt25),
                d_tt3: d(a.tt3, b.tt3),
                d_f: d(a.f, b.f),
                d_mfp: d(a.mfp, b.mfp),
                d_ratio: d(a.ratio, b.ratio),
                d_kappa: d(a.kappa, b.kappa),
                d_n_hp: d(a.n_hp, b.n_hp),
                d_nu_lp: d(a.nu_lp, b.nu_lp),
            });
        }
        let mid = n / 2;
        let worst = |x: &[f64], y: &[f64]| x.iter().zip(y.iter())
            .map(|(&a, &b)| (a - b).abs() / b)
            .fold(f64::NEG_INFINITY, |m, v| if v > m { v } else { m });
        SensedInputs {
            ordinate_identical: l_arm.kappa == l_ref.kappa,
            abscissa_identical: l_arm.n_h == l_ref.n_h,
            d_ordinate: worst(&l_arm.kappa, &l_ref.kappa),
            d_abscissa: worst(&l_arm.n_h, &l_ref.n_h),
            signed_ordinate: l_arm.kappa[mid] / l_ref.kappa[mid] - 1.0,
            signed_abscissa: l_arm.n_h[mid] / l_ref.n_h[mid] - 1.0,
            d_mfp: chain.iter().map(|r| r.d_mfp.abs())
                .fold(f64::NEG_INFINITY, |m, v| if v > m { v } else { m }),
            reference: l_ref,
            armed: l_arm,
            chain,
        }
    }

    /// RUNG 63. Rung 59's SPLICE, for a lever that moves BOTH halves of the table.
    ///
    /// The armed cell is run against four legs — the reference-derived one, the MATCHED one, and
    /// the two `synthetic_leg` splices — so the matched leg's effect can be read per half. Rung 59
    /// always had one half exactly zero, which made the split trivially additive; **here both
    /// halves are live and they carry OPPOSITE SIGNS.**
    #[allow(clippy::too_many_arguments)]
    pub fn matched_leg_deltas(
        &self, flight: &FlightCondition, ramp: &Ramp, lever: &LeverArm, margin: f64, spool: Spool,
        n: usize, neighbour: Option<&LeverArm>,
    ) -> MatchedLegDeltas {
        let (reference, armed) = self.isolating(lever, neighbour);
        let l_b = reference.fuel.accel_schedule(flight, ramp.tt4_lo, ramp.tt4_hi, margin, n);
        let l_a = armed.fuel.accel_schedule(flight, ramp.tt4_lo, ramp.tt4_hi, margin, n);
        let l_s = ScheduledStatorCore::synthetic_leg(&l_a, &l_b);   // ARMED index, REFERENCE values
        let l_c = ScheduledStatorCore::synthetic_leg(&l_b, &l_a);   // REFERENCE index, ARMED values
        let mut cells = Vec::with_capacity(4);
        let mut clamped = 0usize;
        for leg in [&l_b, &l_a, &l_s, &l_c] {
            let sl = StatorLeg { accel: Some(leg), surge: None, tt4_max: None };
            cells.push(armed.cell(flight, ramp, spool, &sl));
            let (traj, _) = armed.stator_march(flight, ramp, None, &sl);
            clamped = clamped.max(armed.clamp_audit(flight, &traj, leg).clamped);
        }
        let base = cells[0].m_i;
        MatchedLegDeltas {
            spool,
            r: ramp.r,
            ds: ramp.ds,
            margin,
            delta_match: cells[1].m_i - base,
            delta_index: cells[2].m_i - base,
            delta_value: cells[3].m_i - base,
            clamped,
            bare_leg: cells[0].clone(),
            matched: cells[1].clone(),
            reindexed: cells[2].clone(),
            revalued: cells[3].clone(),
        }
    }

    /// RUNG 63. Rung 58's four-cell mixed second difference, built on `at_lever` siblings so it
    /// can isolate the VALVE (`composite_credit` cannot — see [`r62_isolating`]).
    ///
    /// ```text
    /// interaction = [M_i(both) - M_i(fuel)] - [M_i(lever) - M_i(neither)]
    /// ```
    ///
    /// **THE CURRENCY IS `M_i`** for rung 58's reason, and for a bleed it is cleaner still: the
    /// valve is a pure POINT-mover (`v = 0` identically), so `M_i = T_c - 1/phi` with `T_c` the
    /// blade metal off the DESIGN map — ONE fixed wall in all four cells, and no moving-wall
    /// coordinate artifact is even possible.
    pub fn lever_composite(
        &self, flight: &FlightCondition, ramp: &Ramp, lever: &LeverArm, leg: &StatorLeg<'_>,
        spool: Spool, neighbour: Option<&LeverArm>,
    ) -> LeverComposite {
        let kind = leg.one();
        let (reference, armed) = self.isolating(lever, neighbour);
        let free = StatorLeg::default();
        let neither = reference.cell(flight, ramp, spool, &free);
        let lever_cell = armed.cell(flight, ramp, spool, &free);
        let fuel = reference.cell(flight, ramp, spool, leg);
        let both = armed.cell(flight, ramp, spool, leg);
        let c_bare = lever_cell.m_i - neither.m_i;
        let c_fuel = both.m_i - fuel.m_i;
        let d_i = c_fuel - c_bare;
        let prof = CreditProfile::new(&neither.prof, &lever_cell.prof);
        let (p_bare, p_fuel) = (prof.at(neither.s), prof.at(both.s));
        LeverComposite {
            spool,
            r: ramp.r,
            ds: ramp.ds,
            leg: kind,
            credit_bare: c_bare,
            credit_fuel: c_fuel,
            interaction: d_i,
            share: if c_bare != 0.0 { d_i / c_bare } else { f64::NAN },
            predicted: p_fuel - p_bare,
            profile_bare: p_bare,
            profile_fuel: p_fuel,
            recovered: if d_i != 0.0 { (p_fuel - p_bare) / d_i } else { f64::NAN },
            relocation: both.s - lever_cell.s,
            relocation_bare: fuel.s - neither.s,
            removed_bare: fuel.fuel_removed,
            removed_armed: both.fuel_removed,
            neither,
            lever: lever_cell,
            fuel,
            both,
        }
    }

    /// **THE SECOND FINDING (rung 63).** Rung 49's `phi` floor beside the valve, swept over the
    /// set point — and the pair has **no composable middle**.
    ///
    /// A bleed's credit runs ENTIRELY through `phi` (a pure point-mover: `v = 0`, so
    /// `M_i = T_c - 1/phi` exactly). A `SurgeLimiter` PINS `phi`. Rung 60 found a floor beside a
    /// STATOR gives `= v` in `phi` and `= 0` in incidence, both exact; with `v = 0` those two
    /// collapse onto each other. So there are only two regimes, and the boundary is **not fitted**
    /// — it is the two plants' OWN minimum `phi`:
    ///
    /// ```text
    /// phi_lim < min phi(reference)   both plants clear; the leg is DORMANT in BOTH
    /// in between                     the floor is DISARMED by the lever: dormant on the armed
    ///                                plant, BIT-FOR-BIT its leg-free march
    /// phi_lim > min phi(armed)       BOTH bind, the floor pins the currency, and the lever's
    ///                                credit is EXACTLY zero -- rung 60's tautology, in both
    ///                                currencies at once
    /// ```
    ///
    /// `s_eng` is deliberately NOT reported: a floor above the initial `phi` is violated from
    /// `s = 0`, where `s_eng` finds no upward crossing and returns NaN.
    pub fn floor_dichotomy(
        &self, flight: &FlightCondition, ramp: &Ramp, lever: &LeverArm, sm_grid: &[f64],
        spool: Spool, neighbour: Option<&LeverArm>,
    ) -> FloorDichotomy {
        let (reference, armed) = self.isolating(lever, neighbour);
        let cmap = self.arming().design_map(spool);
        let free = StatorLeg::default();
        let free_ref = reference.cell(flight, ramp, spool, &free);
        let free_arm = armed.cell(flight, ramp, spool, &free);
        let rows = sm_grid.iter().map(|&sm| {
            let lim = SurgeLimiter::from_margin(&cmap, spool, sm);
            let sl = StatorLeg { accel: None, surge: Some(Floor::Phi(lim)), tt4_max: None };
            let cf = reference.cell(flight, ramp, spool, &sl);
            let cb = armed.cell(flight, ramp, spool, &sl);
            FloorRow {
                sm,
                phi_lim: lim.phi_lim,
                m_i_fuel: cf.m_i,
                m_i_both: cb.m_i,
                min_phi_fuel: cf.min_phi,
                min_phi_both: cb.min_phi,
                removed_fuel: cf.fuel_removed,
                removed_both: cb.fuel_removed,
                credit: cb.m_i - cf.m_i,
                disarmed: cb.fuel_removed == 0.0 && cf.fuel_removed > 0.0
                    && cb.m_i == free_arm.m_i && cb.min_phi == free_arm.min_phi,
            }
        }).collect();
        FloorDichotomy {
            spool,
            r: ramp.r,
            ds: ramp.ds,
            phi_surge: cmap.phi_surge,
            min_phi_ref: free_ref.min_phi,
            min_phi_armed: free_arm.min_phi,
            band: (free_ref.min_phi / cmap.phi_surge - 1.0,
                   free_arm.min_phi / cmap.phi_surge - 1.0),
            rows,
        }
    }
}
