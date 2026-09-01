//! RUNG 68 — **THREE LOOPS ON ONE VARIABLE**: a `phi`-referenced STATOR limiter beside rung 65's
//! valve and rung 52's fuel leg, all three holding `phi_lp` to the SAME set point `phi_lim`.
//!
//! Headline: *`n` loops on one variable are ONE loop with ALL `n` RATES ADDED* — the actuator
//! block is rank ONE at every `n`, so only the CYCLIC product tests it. See `docs/rung68-spec.md`.
//!
//! # What this module is, at STEP 1
//!
//! **The nine cells and the four dynamically-scoped fields, and nothing else.** § 5.19 (x)'s rule
//! for the phase is that *step 1 of every slice is the cell addition*, so a slice that forgets a
//! cell fails at its own first gate rather than at a value key nine rungs downstream. Every cell
//! below therefore exists, is dispatched through [`TripleHooks`], and defaults to [`NO_TRIPLE`],
//! whose bodies PANIC. The rung-68 bodies land at step 2.
//!
//! # The nine cells, EMITTED rather than typed (§ 5.25 (ii))
//!
//! `probe_aa1.py` enumerates them: a name is a cell iff it is **new here** (no definition below in
//! the MRO) and **overridden above**. The answer is nine — `_check_v0`, `_clamp_v`,
//! `_lagged_stator`, `_manifold_v`, `_rk4_floor`, `_solve_v`, `_stator_leg`, `_triple_laws`,
//! `_triple_rig` — which agrees with § 5.19 (x)'s hand-written column **name for name**, the
//! FIFTH row of that column an emitter confirms.
//!
//! **Eight of the nine are overridden by rung 69 and one — `_triple_laws` — by rung 70.** That is
//! why the widest step in phase 7 is rung 68 and not rung 69: rung 68 is the CALLER of all nine.
//! And every one of those overrides has a signature **identical** to the body here (measured over
//! all three classes), so opening the cells at this width is the last non-additive move the
//! family needs.
//!
//! # The four scoped fields, and why none of them needs § 5.19 (iv)'s `Scope`
//!
//! Rung 68 brings **four** of § 5.19 (iv)'s nine dynamically-scoped fields live at once — the
//! phase's largest single arrival — and every one takes a precedent already in the crate:
//!
//! | Python | Rust | precedent | guard restores to |
//! |---|---|---|---|
//! | `_v_forced` | [`TwoSpoolTransientCore::v_forced`] | `b_forced` / `ForcedBleed` | `None` |
//! | `_v_state` | [`TwoSpoolTransientCore::v_state`] | `b_state` / `BleedState` | `None` |
//! | `_v0` | [`TwoSpoolTransientCore::v0`] | `b0` / `InitialBleed` | the PREVIOUS value |
//! | `_ic_order` | [`TwoSpoolTransientCore::ic_order`] | `b0` / `InitialBleed` | the PREVIOUS value |
//!
//! **AND THE NESTING MEASUREMENT IS WHAT LICENSES THE FIRST TWO.** A guard that restores to
//! `None` CLOBBERS if it ever nests, so `ForcedBleed`'s spelling is only safe on a field that
//! provably does not. `probe_aa3.py` instruments all four plus the inherited `_b_state` and runs
//! the rung-68 suite under its own fixtures: **0 overwrites in 811 632 sets** across
//! `_v_forced` / `_v_state` / `_b_state`, where an "overwrite" — a set to non-`None` while the
//! previous value was non-`None` — is *precisely* a same-field nest.
//!
//! **THE PROBE'S OWN DEPTH COLUMN WAS AN ARTIFACT, AND IT IS RECORDED RATHER THAN QUIETLY
//! FIXED.** It reported `_ic_order` nesting to depth **106**. `_ic_order`'s guard restores to the
//! PREVIOUS value, which is `"gqv"` and never `None`, so a nullity-driven depth counter pushes
//! twice per march and pops never. A restore-to-previous field cannot clobber by construction;
//! the discriminating column for the other three is `OVERWRITE`, not `MAXDEPTH`. § 5.25 (iii).
//!
//! **THE MEASUREMENT'S SCOPE IS RUNG-68 MACHINES**, because that is the class the property was
//! installed on. § 5.22 (vii)'s booking of a same-field `_b_state` / `_v_state` nest at slice AH
//! is **not** discharged here, and both panics below say which rungs would invalidate it.
//!
//! # The arithmetic surface — a cube root was the named risk and there is no cube root
//!
//! `_cubic_roots` (step 2) is the one body in this slice with no precedent in the crate, and the
//! risk raised against it before it was opened was Cardano's trigonometric branch and a cube-root
//! spelling — `x ** (1/3)` is a libm `pow`, `f64::cbrt` is a different instruction, and the two
//! disagree in the last bit, under a rule `lib.rs`'s three-spellings note does not yet carry.
//! `probe_aa6.py` read the body instead of assuming: **the whole rung has ONE `**` site and its
//! exponent is `0.5`**, no `math.*` calls at all, and no cube root. The solver is Newton on the
//! dominant root plus exact deflation. A named risk measured to zero, recorded so a later reader
//! does not re-raise it.
//!
//! What the same sweep DID find, and what step 2 owes: a `round(x, 12)` inside a set-membership
//! key (`round6`/`round3`'s format-and-parse precedent, not `(x*1e12).round()/1e12`), two keyed
//! sorts whose ties are REAL (`_cubic_roots`'s complex branch returns `-0.5*p` twice, so the sort
//! must be stable), and ten dict-ordered iterations of which one feeds a float `sum`.
//!
//! # The CPython exemption this slice owes, PRE-MEASURED
//!
//! Rung 68 has **nine** float `sum()` sites against slice Z's one, which reads like nine times the
//! exposure and is not. `probe_aa4.py` intercepts what the readers actually sum (a module-level
//! `sum` shadowing the builtin in `turbojet.engine`'s globals, recording and delegating — slice
//! Z's leading finding was a probe that RECONSTRUCTED the summands at the wrong width and
//! inverted its own answer), and `probe_aa5.py` re-sums those lists under both interpreters
//! against a naive left fold, which is what a Rust fold is. **Eight of the nine sum three or four
//! numbers and agree everywhere; the ninth — `ic_family`'s `withheld` — sums 101 and differs on
//! CPython in 2 of 10 instances.** The exemption surface is one READER, not nine sites.

use std::cell::Cell;

use crate::bleed_transient::{r62_try_close_fuel, LeverArm, LeverArming, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    asym_extra, point, AccelSchedule, AsymmetricLag, Floor, FuelCloseState, FuelInstant,
    FuelLimiters, FuelPoint, FuelTransientCore, FuelTransientHooks, PointExtra, SurgeLimiter,
};
use crate::gas::{powp, Abort};
use crate::lagged_bleed::{lagged, py_max3};
use crate::limited_bleed::{BleedLimiter, Regime};
use crate::map::ComponentMap;
use crate::spool::{try_illinois, ILLINOIS_MAXIT};
use crate::stator_transient::{
    r57_arm, r57_v_of, MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
    StatorTransientHooks,
};
use crate::two_spool::{Spool, TwoSpoolEngine};
use crate::two_spool_transient::{
    DeclaredOrder, ForcedBleed, ForcedStator, InitialStator, MarchedBleed, MarchedStator,
    TwoSpoolTransientCore, TwoSpoolTransientHooks,
};

// ---------------------------------------------------------------------------------------------
// THE DEVICE
// ---------------------------------------------------------------------------------------------

/// Python's class attribute `_ic_order = "gqv"` — **the DECLARED member of the `s = 0` family**.
///
/// § 2 makes the actuator block rank ONE, so the `s = 0` fixed points are a CURVE and a
/// Gauss-Seidel sweep lands on whichever member its ORDER selects: solving `q` first puts `phi` on
/// the floor and leaves the stator DORMANT at its own fixed point, solving `v` first lands on a
/// different member with the valve dormant. Both are legitimate initial conditions **and they are
/// not the same trajectory.**
///
/// `"gqv"` is rung 66's order with the new actuator appended last, so the rung-66 arm is reached
/// unchanged and the stator takes up only what the pair leaves. It is a CONSTANT and not a
/// default parameter because Python reads it off `self`, which is what makes it overridable per
/// march (see [`DeclaredOrder`](crate::two_spool_transient::DeclaredOrder)).
pub const IC_ORDER_DECLARED: &str = "gqv";

/// RUNG 68's control law: **the smallest `|v|` in `[-v_max, 0]` that holds `phi_lp >= phi_lim`.**
///
/// The THIRD loop on `phi_lp`, and the last lever on this plant with authority over it. It exists
/// to answer rung 66's seam: `det J == 0` for two laws on one constraint suggests a rank
/// deficiency that GROWS with the loop count, and testing that needs a third law holding the SAME
/// variable to the SAME set point — `phi_lim`, rung 49/64's, shared verbatim with rung 52's fuel
/// leg and rung 65's valve.
///
/// **TWO OF THE THREE REGIMES ARE INVERTED relative to [`BleedLimiter`], and the inversion is
/// silent if you get it wrong.** `phi_lp` is DECREASING in `v` (measured `dphi_lp/dv ~ -0.42`)
/// where it is INCREASING in `b`, so `solve_v`'s bracket orientation and BOTH clamp tests are
/// mirrored — and a wrong orientation returns a wrong REGIME LABEL with nothing raising. Python
/// calls that rung 62's `_powers` trap in its fourth reload.
///
/// **WHY NEGATIVE IS THE PROTECTIVE DIRECTION AND WHY THAT IS NOT THE PHYSICAL ONE**, disclosed
/// rather than defended: CLOSING the stators (`v > 0`) LOWERS `phi_lp`, so a loop referenced to a
/// fixed `phi_lim` must OPEN them. A real VSV schedule closes at low corrected speed, for the
/// reason rung 53 published — closing lowers the WALL `phi_surge(v) = 1/(T_c+v)` faster than it
/// lowers `phi`. A `phi`-referenced loop cannot see the wall, so it moves the lever the other way,
/// PROTECTING `phi` while ERODING incidence margin (`dM_phi/dv = -0.115` against
/// `dM_i/dv = +0.344`). It is the law the rank question requires, because that question needs all
/// three loops on the SAME constraint; re-referencing it to the metal wall is rung 69.
///
/// `v_max` is the lever's AUTHORITY and is hardware exactly as `b_max` is — rungs 57/58's swept
/// setting `V = 0.20`, INHERITED rather than chosen, so this rung adds no new constant. `tau` is
/// the actuator's bandwidth and makes the position a FIFTH march state.
///
/// `Copy` for [`LeverArm`](crate::bleed_transient::LeverArm)'s reason, which is what keeps
/// § 5.21 (iii)'s *"the signature is never re-opened"* true when this field is added to it.
///
/// [`BleedLimiter`]: crate::limited_bleed::BleedLimiter
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatorLimiter {
    /// The floor, in the map's own flow-coefficient units — **SHARED** with rung 52's fuel leg
    /// and rung 65's valve, which is what makes § 2's identity a statement about ONE SET POINT
    /// rather than about three numbers that happen to agree.
    pub phi_lim: f64,
    /// The AUTHORITY. The admissible band is `[-v_max, 0]`, one-sided.
    pub v_max: f64,
    /// The actuator's BANDWIDTH — hardware, like `v_max`. `None` is refused by the integrator,
    /// not silently dropped: rung 66's discipline is that a lagged loop beside an instantaneous
    /// one is not a control but a different plant.
    pub tau: Option<f64>,
}

impl StatorLimiter {
    /// Python's `__post_init__` — all three asserts, in Python's order.
    pub fn new(phi_lim: f64, v_max: f64, tau: Option<f64>) -> Self {
        assert!(phi_lim > 0.0, "rung-68 phi floor is a flow coefficient");
        assert!(
            v_max > 0.0 && v_max < 1.0,
            "rung-68 needs stators with AUTHORITY: v_max = 0 is a limiter that cannot act, which \
             is a DIFFERENT object from an absent one (that is `stator_lim=None`); and |v| >= 1 \
             is far outside the setting range rungs 53-58 swept (V = 0.20). Got v_max = {v_max}"
        );
        assert!(
            tau.is_none_or(|t| t > 0.0),
            "rung-68 tau is a time constant on the march coordinate; an INSTANTANEOUS stator loop \
             is a different object and is not built (rung 66's discipline: a lagged loop against \
             an instantaneous one is not a control but a different plant). Got {tau:?}"
        );
        StatorLimiter { phi_lim, v_max, tau }
    }

    /// `phi_lim = (1+sm)*phi_surge` off the map's OWN imposed surge line — rung 49's and rung
    /// 64's `from_margin` **verbatim**, which is what makes all three floors ONE set point rather
    /// than three numbers that happen to agree. § 2's identity needs exactly that.
    pub fn from_margin(cmap: &ComponentMap, v_max: f64, sm: f64, tau: Option<f64>) -> Self {
        assert!(
            cmap.phi_surge > 0.0,
            "rung-68 from_margin needs a surge line: build the map with .with_phi_surge(.)"
        );
        assert!(sm >= 0.0, "the rung-68 floor sits AT or ABOVE the surge line");
        Self::new((1.0 + sm) * cmap.phi_surge, v_max, tau)
    }
}

/// What [`TripleHooks::stator_leg`] hands back — **the two fields every caller of it reads, and
/// no more.**
///
/// Python's `_stator_leg` returns the limiter OBJECT, and rung 69 returns a **different type**
/// from it (`StatorIncidenceLimiter`, whose limit field is `m_lim` rather than `phi_lim`). Read
/// body by body, the callers of `_stator_leg` touch exactly `.tau` (the integrator's clock and
/// `_lagged_stator`) and `.v_max` (`_clamp_v`, `_check_v0`) — never the limit itself, because
/// `_solve_v` reads its own limiter straight off `self` on BOTH rungs. So the cell's return is
/// narrowed to those two fields and slice AB's incidence limiter feeds the same shape.
///
/// **That is a deliberate narrowing and not an oversight**: returning an enum over the two
/// limiter types would put an exhaustive `match` at every call site for slice AB to break, and
/// buy nothing, since no caller can use the limit it would carry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatorLegArm {
    /// The lever's AUTHORITY. The band's SIGN is the reference's, not this field's — rung 68's is
    /// `[-v_max, 0]` and rung 69's is `[0, +v_max]`, which is why [`TripleHooks::clamp_v`] is a
    /// cell rather than a shared expression.
    pub v_max: f64,
    /// The actuator's bandwidth. `None` is refused by the integrator.
    pub tau: Option<f64>,
}

impl From<StatorLimiter> for StatorLegArm {
    fn from(l: StatorLimiter) -> Self {
        StatorLegArm { v_max: l.v_max, tau: l.tau }
    }
}

// ---------------------------------------------------------------------------------------------
// THE NINE CELLS
// ---------------------------------------------------------------------------------------------

/// The three control laws of § 2, as closures of `(g, q, v)` — what [`TripleHooks::triple_laws`]
/// returns.
///
/// Each solves `phi_lp = phi_lim` for ITS OWN actuator given the other two, each through a
/// SHIPPED closure, and **none knows the others exist**. That mutual ignorance is what makes
/// their products a MEASUREMENT of § 2's algebra rather than a restatement of it.
///
/// Boxed rather than generic because these ride through a `const` table: a `fn` pointer cannot be
/// generic, and the alternative — three concrete closure types named in the cell's signature —
/// would pin the cell to rung 68's bodies and defeat the table.
#[allow(clippy::type_complexity)]
pub struct TripleLaws<'a> {
    /// **R** — the FUEL law, `(q, v) -> (clip, regime)`. It trials NEITHER other actuator, so it
    /// sees BOTH states. Rung 52's leg is a `max(0, .)`, so it has a KINK at its own dormant edge
    /// and a central difference straddling that kink returns the slope of neither branch — which
    /// is why the regime is CARRIED and never re-derived from the float.
    pub r: Box<dyn Fn(f64, f64) -> Result<(f64, LegRegime), Abort> + 'a>,
    /// **C** — the VALVE law, `(g, v) -> (b, regime)`. It trials `b`, so it must NOT see
    /// `b_state`, and it MUST see `v_state`: it solves against the plant as the STATORS actually
    /// are. Getting the pair backwards converges a solver on a residual the plant never uses —
    /// rung 62's `_powers` trap, generalised.
    pub c: Box<dyn Fn(f64, f64) -> Result<(f64, Regime), Abort> + 'a>,
    /// **V** — the STATOR law, `(g, q) -> (v, regime)`, and the exact mirror of `c`: it trials
    /// `v`, so NO `v_state`, but `b_state = q` because it solves against the plant as the VALVE
    /// actually is.
    pub v: Box<dyn Fn(f64, f64) -> Result<(f64, Regime), Abort> + 'a>,
}

/// Which branch the FUEL leg is on — Python's `("riding" if raw > 0.0 else "dormant")`.
///
/// A separate type from [`Regime`] on purpose: the fuel leg has no SATURATED branch (its clip is
/// a `max(0, .)` with no upper stop), and folding it into the three-valued enum would let a
/// reader ask a question the leg cannot answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegRegime {
    /// `raw <= 0` — the leg is not clipping, and the `max(0, .)` is on its flat branch.
    Dormant,
    /// `raw > 0` — the leg is clipping, and only here is it evidence of anything.
    Riding,
}

/// The nine keyword arguments [`TripleHooks::triple_rig`] takes, as one struct so the cell's
/// signature is never re-opened — [`LeverArm`](crate::bleed_transient::LeverArm)'s rule.
///
/// ONE constructor for every ledger cell, so a cell can never differ from another by anything
/// except which loops are armed — rung 63's lesson, and the reason the credits are differenceable
/// at all. Every floor comes from the SAME `from_margin(cmap, ., sm)`, which is what makes this
/// ONE set point rather than three numbers that happen to agree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TripleRigArm {
    pub sm: f64,
    /// The VALVE's clock.
    pub tau: f64,
    /// The STATOR's clock.
    pub tau_s: f64,
    pub v_max: f64,
    pub tau_att: f64,
    pub tau_rel: f64,
    /// Arm rung 52's fuel leg (its `SurgeLimiter` floor and its `AsymmetricLag`).
    pub fuel: bool,
    /// Arm rung 65's lagged valve.
    pub valve: bool,
    /// Arm rung 68's lagged stator.
    pub stator: bool,
}

impl Default for TripleRigArm {
    /// Python's own defaults for the six clocks, all three loops armed. `sm` has no Python
    /// default — every caller passes it — and is zeroed here rather than guessed.
    fn default() -> Self {
        TripleRigArm {
            sm: 0.0,
            tau: 0.05,
            tau_s: 0.05,
            v_max: 0.20,
            tau_att: 0.05,
            tau_rel: 0.15,
            fuel: true,
            valve: true,
            stator: true,
        }
    }
}

/// **RUNG 68's NINE VIRTUAL NAMES** — the widest cell addition in the port.
///
/// Eight are overridden by rung 69 (`ReferenceSplitTransient`, slice AB) and one — `triple_laws`
/// — by rung 70 (`CrossSplitTransient`, slice AC). Rung 68 is the CALLER of all nine, which is
/// why § 5.19 (x) puts the addition here rather than at the rung that overrides them.
///
/// Every override's Python signature is **identical** to the body here — measured over all three
/// classes, not assumed — so this width is final for the family.
pub struct TripleHooks {
    /// Python's `_stator_leg` — WHICH limiter is armed. Rung 69 returns its INCIDENCE limiter in
    /// preference, which is a different Python type; see [`StatorLegArm`] for why the return is
    /// narrowed to the two fields every caller actually reads.
    pub stator_leg: fn(&TwoSpoolTransientCore) -> Option<StatorLegArm>,
    /// Python's `_lagged_stator` — is a lagged stator loop armed at all? **The reduce is by
    /// DISPATCH through this name**: `stator_lim is None` means the five-state integrator is not
    /// entered and `arm` never touches a map, so every inherited arm (rungs 67, 66's three, 65,
    /// 64, 52) leaves through the parent bit-for-bit.
    pub lagged_stator: fn(&TwoSpoolTransientCore) -> bool,
    /// Python's `_clamp_v` — the stator's own hardware stops, applied to the STATE and never to a
    /// command (rung 65, verbatim).
    ///
    /// **THE BAND IS ONE-SIDED AND ITS DORMANT STOP IS ZERO** — the design setting — which is why
    /// the clamp is asymmetric where the valve's is not. WHICH side is open depends on the
    /// REFERENCE: `phi` is DECREASING in `v` (rung 68, band `[-v_max, 0]`) and `M_i` is
    /// INCREASING (rung 69, band `[0, +v_max]`). That is the whole reason this is a cell.
    pub clamp_v: fn(&TwoSpoolTransientCore, f64, &StatorLegArm) -> f64,
    /// Python's `_check_v0` — the same band, asserted on an OVERRIDDEN initial position.
    pub check_v0: fn(&TwoSpoolTransientCore, f64, &StatorLegArm),
    /// Python's `_rk4_floor` — **THE MODELLING FLOOR, and it is a cell because a SHIPPED TEST
    /// SWAPS IT.**
    ///
    /// § 2 makes `J` rank one with its non-zero eigenvalue exactly `-sum_i 1/tau_i`, so the
    /// explicit-RK4 bound is on the SUM over however many clocks are armed. At three matched
    /// clocks that reads `ds/tau <= 2/3` against rung 66's `1.0` and rung 65's `2.0`: **a sweep
    /// inheriting rung 66's constant would run at 1.5x the admissible step.**
    ///
    /// `test_rung68.py:381` subclasses the rung to override this to a **no-op** and then measures
    /// the band the guard refuses — at `ds = 0.05`, admitted by rung 66's own constant, the march
    /// reports `min phi_lp` EXACTLY at the floor and a violation integral of **zero**. It does not
    /// blow up the way rung 65's retraction did; it counterfeits PERFECT PROTECTION, which is
    /// worse. An assert nobody has run past is a tautology, so the port cannot inline this one.
    ///
    /// `static` in Python, so no receiver.
    pub rk4_floor: fn(f64, f64, usize, f64),
    /// Python's `_solve_v` — **the stator's outer solve**, and `_solve_b`'s structure with BOTH
    /// CLAMP TESTS AND THE BRACKET ORIENTATION INVERTED, because `phi_lp` is DECREASING in `v`
    /// where it is INCREASING in `b`. Get that backwards and the regime label is wrong with
    /// nothing failing.
    ///
    /// **THE REGIME IS RETURNED, NEVER INFERRED BY A READER COMPARING FLOATS** — this rung's own
    /// trap is that a SATURATED loop counterfeits INDEPENDENCE, so no reader may recover the
    /// regime from `v` against a stop.
    ///
    /// Takes `&dyn Fn` rather than a generic closure because a `fn` pointer in a `const` table
    /// cannot be generic; the concrete state is [`FuelCloseState`] at both call sites (the
    /// integrator's `stator` law and `triple_laws`'s `V`), both built off the **rung-62 pin**
    /// `super(LimitedBleedTransient, self)._close_fuel`.
    #[allow(clippy::type_complexity)]
    pub solve_v: fn(
        &TwoSpoolTransientCore,
        &dyn Fn(f64) -> Result<FuelCloseState, Abort>,
    ) -> Result<(FuelCloseState, f64, Regime), Abort>,
    /// Python's `_manifold_v` — the base point § 2's algebra is STATED at.
    ///
    /// At rung 68 all three laws hold ONE constraint, so the stator's OWN root IS the shared
    /// manifold and the body is `V(g, q)[0]`. At rung 69 they do not, and there is no point where
    /// all three hold at once — which is the entire content of that rung's § 0.3 and the reason
    /// the four arguments rung 68 ignores are carried rather than dropped.
    #[allow(clippy::type_complexity)]
    pub manifold_v: fn(
        &ScheduledStatorCore,
        &FlightCondition,
        f64,
        f64,
        f64,
        f64,
        f64,
        &dyn Fn(f64, f64) -> Result<(f64, Regime), Abort>,
    ) -> Result<f64, Abort>,
    /// Python's `_triple_laws` — see [`TripleLaws`]. Overridden at rung **70**, alone among the
    /// nine.
    #[allow(clippy::type_complexity)]
    pub triple_laws: for<'a> fn(
        &'a ScheduledStatorCore,
        &'a FlightCondition,
        f64,
        f64,
        f64,
        Option<&'a AccelSchedule>,
        Option<&'a Floor>,
    ) -> Result<TripleLaws<'a>, Abort>,
    /// Python's `_triple_rig` — a machine with any SUBSET of the three loops armed, plus the fuel
    /// leg and lag that go with it. See [`TripleRigArm`].
    #[allow(clippy::type_complexity)]
    pub triple_rig: fn(
        &ScheduledStatorCore,
        &TripleRigArm,
    ) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>),
    /// RUNG **69**'s `_with_ref` — **THE ONE CELL SLICE AB ADDS, and it lives in rung 68's table
    /// because that is where every cell this family dispatches lives.**
    /// [`LeverHooks::b_at_point`](crate::bleed_transient::LeverHooks::b_at_point)'s precedent:
    /// slice X added rung 64's one cell to rung 62's table and gave rung 62 the panicking body,
    /// because a table per rung would put a second `&'static` pointer on the core for one name.
    ///
    /// **IT IS THE SETTER, NOT THE CALL.** Python's `_with_ref(self, ref, fn, *a, **kw)` is
    /// higher-order over a return type that varies by call site — a tuple from `_triple_rig`, a
    /// dict from `triple_bill`, a tuple again from rung 73's `_quad_gains_at`. A `fn` pointer in a
    /// `const` table cannot be generic over that, and `&dyn Fn` does not rescue it because it is
    /// the RETURN type that differs. Read against rung 73's override, the only thing that override
    /// changes is **WHICH FIELD THE GUARD WRITES** — so the cell sets and returns the displaced
    /// value, the RAII guard [`RefScope`](crate::reference_split::RefScope) is shared, and each
    /// reader opens its own scope.
    ///
    /// **RUNG 68's SLOT PANICS**: `_with_ref` does not exist below rung 69 in Python at all.
    pub with_ref: fn(&TwoSpoolTransientCore, Option<&'static str>) -> Option<&'static str>,
    /// RUNG **72**'s `_reference` — **WHICH FUEL A LEG COMPUTES ITS CLIP FROM.**
    ///
    /// Rung 72's body is `return req` and § 5.28 (vi) measured it the **bitwise identity on
    /// 195 278 of 195 278 calls**, so this cell has no value break at the rung that introduces it.
    /// It is a cell because rung **73** overrides it to `g_own + req - clip` — the reading that
    /// makes `F_r = R_f = 0` stop being exact — and because rung 72 is the earliest CALLER, which
    /// is where the phase's rule says a cell must exist.
    ///
    /// The float-identical branch is load-bearing at rung 73 and stated there, not here.
    pub reference: fn(&TwoSpoolTransientCore, f64, f64, f64, f64) -> f64,
    /// RUNG **72**'s `_rk4_floor_shared` — **THE SECOND CELL NO VALUE KEY CAN SEE, and this time
    /// the SHIPPED PYTHON GATE CANNOT SEE IT EITHER.**
    ///
    /// [`rk4_floor`](Self::rk4_floor)'s shape, one family along: the condition is
    /// `ds * rate <= 2.0` in rungs 72, 73 and 74 **character for character**, and the entire cell
    /// is the reason the assertion gives. § 5.28 (v) measured the difference — rung 72 argues from
    /// *a bare pole at `-1/tau_f`*, rung 73 from *a pole EXACTLY at the origin*, rung 74 from
    /// *which states are LIVE*.
    ///
    /// **AND THE NEEDLE THE PYTHON SUITE USES IS IN ALL THREE MESSAGES.**
    /// `tests/test_rung72.py:445` fires this floor once, under `match=r"FOUR actuator states"` —
    /// a phrase rungs 73 and 74 both carry, so that gate passes with either successor's floor
    /// installed. Rung 69's analogue does not have the defect (`match="rank TWO"` is unique to
    /// it). **The ported gate must therefore be written on a token that discriminates**, and it is
    /// the one place in this slice where the port's gate is strictly stronger than the source's.
    ///
    /// `static` in Python, so no receiver — and no `n_states`/`tau_s` either, because unlike
    /// [`rk4_floor`](Self::rk4_floor) this message interpolates only `ds` and `ds * rate`.
    pub rk4_floor_shared: fn(f64, f64),
    /// RUNG **72**'s `_shared_rig` — a machine with any SUBSET of the FOUR loops armed, plus the
    /// fuel leg and lag that go with it. See [`SharedRigArm`](crate::shared_actuator::SharedRigArm).
    ///
    /// [`triple_rig`](Self::triple_rig) with two flags added: `inc` selects rung 71's INCIDENCE
    /// stator over rung 70's `phi` one, and `gov` decides whether the sibling carries `_gov_max`.
    /// Rung 63's one-constructor rule is why it is a single function and not five — a cell may
    /// differ from another only by which loops are armed and which coordinate the stator watches.
    ///
    /// **EIGHT CLASSES DEFINE IT (rungs 72–80)**, all with an identical parameter list (§ 5.28
    /// (ii)), so this width is final for the family.
    #[allow(clippy::type_complexity)]
    pub shared_rig: fn(
        &ScheduledStatorCore,
        &crate::shared_actuator::SharedRigArm,
    ) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>),
    /// RUNG **72**'s `_quad_gains_at` — **THE FOURTEENTH CELL, ADDED BY SLICE AE STEP 2, AND THE
    /// ONE SLICE AD BOOKED FORWARD AS "UNREACHABLE".**
    ///
    /// Two definers (rungs 72 and 73) with an identical signature, so it is a cell by every filter
    /// the phase uses. Slice AD § (b) measured it a cell and then declined to install it, having
    /// asked *does shipped code sit in the seat* — all 19 call sites of rung 72's five readers
    /// build a rung-72 machine, so the answer was no. **That is the weaker question, and § 5.29
    /// (iv) refuted the booking BY VALUE**: hold the machine at rung 73, swap only the pointer, and
    /// 32 keys move while 70 vanish — `rows[10].gains.F_r` going `-1.000000000002735` to `0.0`,
    /// which is the rung's own headline number replaced by rung 72's block-triangular zero.
    ///
    /// **THE DIFFERENCE BETWEEN THE TWO BODIES IS THE REFERENCE, AND IT IS FOUR EXTRA ARMS —
    /// WHICH ARE MEASURED, AND WHICH OBSERVE NOTHING.**
    /// Rung 72 takes TWELVE central differences and evaluates 24 arms; rung 73 takes FOURTEEN
    /// (adding `F_f` and `R_r`) and evaluates 28, because each leg's law is wrapped in
    /// [`reference`](Self::reference) before it is differenced — which makes `F` a function of
    /// `gf` where at rung 72 it was not. Probe M drove both bodies on the SAME receiver at every
    /// sampled point of both `applied_gains` arms: the arm count differs by **exactly 4 at every
    /// one of the 101 points**, and `interior` **DISAGREES ON ZERO OF THEM**, with the twelve
    /// shared gains identical to the bit. So the extra arms can in principle drop a point the
    /// parent keeps, and on the shipped grid they never do — written down because the first
    /// draft of this comment offered that possibility as the discriminator, which would have been
    /// a gate with nothing to catch.
    ///
    /// **THE OBSERVABLE IS DISCRETE AND IT IS THE ABSENT KEYS.** Rung 72's dict has no `F_f`,
    /// `R_r`, `self_masked`, `cross_masked` or `self_live` — five per point, **505 over the same
    /// 101 points** — and a key that is absent cannot be passed by a one-sided bar. That is
    /// § 5.29 (iv)'s 70 shipped-only keys, re-measured on this step's own grid.
    ///
    /// **AND IT IS DISPATCHED, NOT CALLED**, at every one of rung 72's own reader sites: Python
    /// reaches it as `m._quad_gains_at` on the machine the rig hands back, so an inherited rung-72
    /// reader run on a rung-73 machine takes rung 73's body. A census restricted to `self.NAME`
    /// scored this method at ZERO readers when it has ELEVEN call sites — § 5.29 (x)'s first
    /// instrument defect — and the port must not repeat the same reading by calling
    /// [`quad_gains_at`](crate::shared_actuator::quad_gains_at) directly.
    #[allow(clippy::type_complexity)]
    pub quad_gains_at: fn(
        &ScheduledStatorCore,
        &FlightCondition,
        &FuelPoint,
        Option<&AccelSchedule>,
        Option<&Floor>,
        f64,
        f64,
        f64,
        f64,
        bool,
        f64,
    ) -> Result<crate::shared_actuator::QuadGains, Abort>,
}

/// **THE DEFAULT, AND ITS CELLS PANIC.** [`NO_STATOR`](crate::stator_transient::NO_STATOR) and
/// [`NO_LEVER`](crate::bleed_transient::NO_LEVER)'s precedent, and the same reason: rungs 40–67
/// have no `_stator_leg`, no `_solve_v` and no `_rk4_floor` in Python **at all**. An unfloored
/// rung-67 object is not a rung-68 object with the stator loop shut; it is an object where the
/// names do not exist.
///
/// Defaulting `stator_leg` to `None` and `lagged_stator` to `false` would be the tempting choice
/// and is exactly the move this family has been caught on: it agrees with the truth on precisely
/// the machines the suites build, so **no value key could ever see it**. A panic that is
/// unreachable by construction is one a smoke test can assert directly.
pub const NO_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: no_triple_stator_leg,
    lagged_stator: no_triple_lagged_stator,
    clamp_v: no_triple_clamp_v,
    check_v0: no_triple_check_v0,
    rk4_floor: no_triple_rk4_floor,
    solve_v: no_triple_solve_v,
    manifold_v: no_triple_manifold_v,
    triple_laws: no_triple_triple_laws,
    triple_rig: no_triple_triple_rig,
    with_ref: no_triple_with_ref,
    reference: no_triple_reference,
    rk4_floor_shared: no_triple_rk4_floor_shared,
    shared_rig: no_triple_shared_rig,
    quad_gains_at: no_triple_quad_gains_at,
};

const NO_TRIPLE_MSG: &str = "no triple table on this object: rungs 40-67 have no third loop on \
                             `phi_lp` at all. Answering `None`/`false` here would be a claim no \
                             value gate could see, because it agrees with the truth on exactly \
                             the machines the suites build.";

fn no_triple_stator_leg(_: &TwoSpoolTransientCore) -> Option<StatorLegArm> {
    panic!("{NO_TRIPLE_MSG} (_stator_leg)");
}

fn no_triple_lagged_stator(_: &TwoSpoolTransientCore) -> bool {
    panic!("{NO_TRIPLE_MSG} (_lagged_stator)");
}

fn no_triple_clamp_v(_: &TwoSpoolTransientCore, _: f64, _: &StatorLegArm) -> f64 {
    panic!("{NO_TRIPLE_MSG} (_clamp_v)");
}

fn no_triple_check_v0(_: &TwoSpoolTransientCore, _: f64, _: &StatorLegArm) {
    panic!("{NO_TRIPLE_MSG} (_check_v0)");
}

fn no_triple_rk4_floor(_: f64, _: f64, _: usize, _: f64) {
    panic!("{NO_TRIPLE_MSG} (_rk4_floor)");
}

fn no_triple_solve_v(
    _: &TwoSpoolTransientCore,
    _: &dyn Fn(f64) -> Result<FuelCloseState, Abort>,
) -> Result<(FuelCloseState, f64, Regime), Abort> {
    panic!("{NO_TRIPLE_MSG} (_solve_v)");
}

#[allow(clippy::too_many_arguments)]
fn no_triple_manifold_v(
    _: &ScheduledStatorCore,
    _: &FlightCondition,
    _: f64,
    _: f64,
    _: f64,
    _: f64,
    _: f64,
    _: &dyn Fn(f64, f64) -> Result<(f64, Regime), Abort>,
) -> Result<f64, Abort> {
    panic!("{NO_TRIPLE_MSG} (_manifold_v)");
}

fn no_triple_triple_laws<'a>(
    _: &'a ScheduledStatorCore,
    _: &'a FlightCondition,
    _: f64,
    _: f64,
    _: f64,
    _: Option<&'a AccelSchedule>,
    _: Option<&'a Floor>,
) -> Result<TripleLaws<'a>, Abort> {
    panic!("{NO_TRIPLE_MSG} (_triple_laws)");
}

fn no_triple_triple_rig(
    _: &ScheduledStatorCore,
    _: &TripleRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    panic!("{NO_TRIPLE_MSG} (_triple_rig)");
}

/// **RUNG 68's OWN SLOT AS WELL AS THE DEFAULT'S**, which is why the message names rung 69 rather
/// than the table. `_with_ref` arrives at rung 69; rungs 40–68 have no reference to select, and
/// answering `None` here would be the same invisible claim [`NO_TRIPLE`] exists to refuse — it
/// agrees with the truth on exactly the machines the rung-68 suite builds.
///
/// [`LeverHooks::b_at_point`](crate::bleed_transient::LeverHooks::b_at_point)'s shape: rung 62's
/// shipped table carries the panicking body of rung 64's added cell for this reason.
/// The refusal all three of slice AD's cells share.
///
/// **A DEFAULT WOULD BE THE DANGEROUS ANSWER HERE, AND ONE OF THE THREE SHOWS WHY.**
/// `reference` returning `req` is rung 72's own body, so a rung-40..71 object silently answering
/// it would agree with rung 72 on every input and no value gate anywhere could see the slot was
/// wrong. That is [`NO_TRIPLE`]'s stated reason, and this is its clearest instance yet.
const NO_SHARED_MSG: &str = "this name is RUNG 72's and does not exist below it: a rung-40..71 \
                             object has no SHARED actuator, so there is no second leg to compute \
                             a reference from, no fourth clock to floor, and no four-loop rig to \
                             build. Answering a default would agree with rung 72 on every input \
                             the suites reach, which is exactly the claim no value gate could see.";

fn no_triple_reference(_: &TwoSpoolTransientCore, _: f64, _: f64, _: f64, _: f64) -> f64 {
    panic!("{NO_SHARED_MSG} (_reference)");
}

fn no_triple_rk4_floor_shared(_: f64, _: f64) {
    panic!("{NO_SHARED_MSG} (_rk4_floor_shared)");
}

fn no_triple_shared_rig(
    _: &ScheduledStatorCore, _: &crate::shared_actuator::SharedRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    panic!("{NO_SHARED_MSG} (_shared_rig)");
}

/// The FOURTH member of [`NO_SHARED_MSG`]'s family — slice AE step 2's added cell.
///
/// Same reason as its three siblings, and the sharpest instance of it: rung 72's body is a
/// perfectly well-typed answer for a rung-68..71 object, and it would return the twelve gains
/// those rungs' own readers never ask for. A default here agrees with rung 72 on every input,
/// which is exactly the claim no value gate can see.
#[allow(clippy::too_many_arguments)]
fn no_triple_quad_gains_at(
    _: &ScheduledStatorCore, _: &FlightCondition, _: &FuelPoint, _: Option<&AccelSchedule>,
    _: Option<&Floor>, _: f64, _: f64, _: f64, _: f64, _: bool, _: f64,
) -> Result<crate::shared_actuator::QuadGains, Abort> {
    panic!("{NO_SHARED_MSG} (_quad_gains_at)");
}

fn no_triple_with_ref(_: &TwoSpoolTransientCore, _: Option<&'static str>) -> Option<&'static str> {
    panic!("_with_ref is RUNG 69's and does not exist below it: a rung-40..68 object has no             REFERENCE to select, and answering `None` would be a claim no value gate could see             because it agrees with the truth on exactly the machines those suites build.");
}

// ---------------------------------------------------------------------------------------------
// The dispatch points, on the cores the cells' receivers name
// ---------------------------------------------------------------------------------------------

impl TwoSpoolTransientCore {
    /// Rung 68's `_stator_leg`, **through the virtual table**.
    pub fn stator_leg(&self) -> Option<StatorLegArm> {
        (self.triple_hooks.stator_leg)(self)
    }

    /// Rung 68's `_lagged_stator`, **through the virtual table**.
    pub fn lagged_stator(&self) -> bool {
        (self.triple_hooks.lagged_stator)(self)
    }

    /// Rung 68's `_clamp_v`, **through the virtual table**.
    pub fn clamp_v(&self, v: f64, lim_s: &StatorLegArm) -> f64 {
        (self.triple_hooks.clamp_v)(self, v, lim_s)
    }

    /// Rung 68's `_check_v0`, **through the virtual table**.
    pub fn check_v0(&self, v0: f64, lim_s: &StatorLegArm) {
        (self.triple_hooks.check_v0)(self, v0, lim_s)
    }

    /// Rung 68's `_rk4_floor`, **through the virtual table**. `static` in Python, so the receiver
    /// exists only to reach the table.
    pub fn rk4_floor(&self, ds: f64, rate: f64, n_states: usize, tau_s: f64) {
        (self.triple_hooks.rk4_floor)(ds, rate, n_states, tau_s)
    }

    /// Rung 68's `_solve_v`, **through the virtual table**.
    pub fn solve_v(
        &self,
        closer: &dyn Fn(f64) -> Result<FuelCloseState, Abort>,
    ) -> Result<(FuelCloseState, f64, Regime), Abort> {
        (self.triple_hooks.solve_v)(self, closer)
    }

    /// Rung **69**'s `_with_ref` setter, **through the virtual table** — sets the reference and
    /// hands back what it displaced.
    ///
    /// Not called directly by anything but [`RefScope::set`](crate::reference_split::RefScope):
    /// a set without a matching restore is exactly the leak Python's `finally` exists to prevent,
    /// so the only public way to reach this is the guard.
    pub fn with_ref(&self, r: Option<&'static str>) -> Option<&'static str> {
        (self.triple_hooks.with_ref)(self, r)
    }
}

impl ScheduledStatorCore {
    /// Rung 68's `_manifold_v`, **through the virtual table**.
    #[allow(clippy::too_many_arguments)]
    pub fn manifold_v(
        &self,
        flight: &FlightCondition,
        a: f64,
        h: f64,
        mf_sched: f64,
        g: f64,
        q: f64,
        v_law: &dyn Fn(f64, f64) -> Result<(f64, Regime), Abort>,
    ) -> Result<f64, Abort> {
        (self.triple_hooks().manifold_v)(self, flight, a, h, mf_sched, g, q, v_law)
    }

    /// Rung 68's `_triple_laws`, **through the virtual table**.
    pub fn triple_laws<'a>(
        &'a self,
        flight: &'a FlightCondition,
        a: f64,
        h: f64,
        mf_sched: f64,
        accel: Option<&'a AccelSchedule>,
        surge: Option<&'a Floor>,
    ) -> Result<TripleLaws<'a>, Abort> {
        (self.triple_hooks().triple_laws)(self, flight, a, h, mf_sched, accel, surge)
    }

    /// Rung 68's `_triple_rig`, **through the virtual table**.
    pub fn triple_rig(
        &self,
        arm: &TripleRigArm,
    ) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
        (self.triple_hooks().triple_rig)(self, arm)
    }

    /// The triple table this machine carries — it lives on the shared transient core, for
    /// [`TwoSpoolTransientCore::stator_hooks`]'s reason exactly: `solve_v` and `stator_leg` are
    /// reached from inside rung-62's `_close_fuel` closures, so `&TwoSpoolTransientCore` is the
    /// shallowest type they must be reachable from.
    pub fn triple_hooks(&self) -> &'static TripleHooks {
        self.fuel.inner.triple_hooks
    }
}

// ---------------------------------------------------------------------------------------------
// THE CONSTRUCTOR — rung 67's, plus the third loop and its three refusals
// ---------------------------------------------------------------------------------------------

/// Python's `ThreeLoopCascadeTransient.__init__` — rung 67's object with `stator_lim` added, and
/// **the three asserts that make the LP stators' three legs mutually exclusive.**
///
/// The first mirrors rung 64's three-way assert on the valve one lever over: the LP stators get a
/// CONSTANT setting (rung 53), a SCHEDULE (57) or a FLOOR (68) — **exactly one**, because those
/// three are precisely the legs this family differences. The second is § 2's own scope: one
/// VARIABLE is not one SET POINT, and rung 66 § 2 measured a −2.5 % offset moving the identity's
/// product to 0.951. The third is arithmetic — a floor that watches the LP needs an LP.
///
/// **THE INHERITED ASSERTS ARE RE-LISTED RATHER THAN REACHED THROUGH RUNG 67's BUILDER**, which is
/// [`build_cross_loop_cascade`]'s own shape: each rung's builder installs its OWN five tables and
/// spells the ancestors' refusals in Python's order. Routing through the parent builder and then
/// overwriting five table fields would work and would leave a machine that was, for a moment, a
/// rung-67 object — which is exactly the *silently hands back the wrong class* trap this rung's
/// `at_lever` docstring calls its sixth instance.
///
/// [`build_cross_loop_cascade`]: crate::cross_loop::build_cross_loop_cascade
pub fn build_three_loop_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    let built = ScheduledStatorTransient::with_triple_tables(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm.stator,
        &R68_TWO, &R68_STATOR, &R68_FUEL, &R68,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim },
        &R68_TRIPLE, arm.stator_lim);
    // Rung 62's two, in Python's order.
    assert!(!(arm.bleed != 0.0 && arm.bleed_sched.is_some()),
            "rung-62: the valve gets a CONSTANT position or a SCHEDULE, not both -- they are the \
             two legs the rung differences (rung 57's discipline).");
    assert!((0.0..0.5).contains(&arm.bleed),
            "rung-42 bleed fraction must be in [0, 0.5): b>=0.5 starves the core and the choked \
             branch is long gone by then");
    // Rung 64's three-way arming exclusion.
    assert!(!(arm.bleed_lim.is_some() && (arm.bleed != 0.0 || arm.bleed_sched.is_some())),
            "rung-64: the valve gets a CONSTANT position (42), a SCHEDULE (62) or a FLOOR (64) -- \
             exactly one. They are the three legs this rung differences, and rung 62's two-way \
             assert is extended rather than replaced.");
    // RUNG 68's own three, in Python's order.
    assert!(!(arm.stator_lim.is_some()
              && (arm.stator.vsv_lp != 0.0 || arm.stator.sched_lp.is_some())),
            "rung-68: the LP stators get a CONSTANT setting (53), a SCHEDULE (57) or a FLOOR \
             (68) -- exactly one. This mirrors rung 64's three-way assert on the valve, one \
             lever over, and the three are exactly the legs this family differences.");
    if let (Some(s), Some(b)) = (arm.stator_lim, arm.bleed_lim) {
        assert!(s.phi_lim == b.phi_lim,
                "rung-68 s 2's identity needs ONE SET POINT, not merely one variable: rung 66 s 2 \
                 measured a -2.5 % offset moving the product to 0.951. Got stator {} vs valve {}. \
                 Build both with the same `from_margin(cmap, ., sm)`.", s.phi_lim, b.phi_lim);
    }
    assert!(arm.stator_lim.is_none() || !arm.stator.lp_disabled,
            "rung-68's stator floor watches the LP, which a disabled LP spool does not have.");
    built
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — five of them, and the fifth is this rung's own
// ---------------------------------------------------------------------------------------------

/// RUNG 68's lever table — ONE cell, `at_lever`, which grows to its EIGHTH keyword here.
pub const R68: LeverHooks = LeverHooks {
    at_lever: r68_at_lever,
    ..crate::cross_loop::R67
};

/// RUNG 68's `TwoSpoolTransientHooks` — **ZERO cells swapped**, named for `R66_TWO`'s reason: a
/// spread of the parent would make the NEXT addition to that table silent here.
pub const R68_TWO: TwoSpoolTransientHooks = crate::cross_loop::R67_TWO;

/// RUNG 68's fuel table — ONE cell, `integrate_fuel`.
pub const R68_FUEL: FuelTransientHooks = FuelTransientHooks {
    integrate_fuel: r68_integrate_fuel,
    ..crate::cross_loop::R67_FUEL
};

/// RUNG 68's stator table — **THREE cells**, and the first rung in the family to swap all three:
/// the march that carries `v0`/`ic_order`, plus `arm` and `v_of`, which the live limiter position
/// now overrides.
pub const R68_STATOR: StatorTransientHooks = StatorTransientHooks {
    arm: r68_arm,
    v_of: r68_v_of,
    stator_march: r68_stator_march,
    ..crate::cross_loop::R67_STATOR
};

/// **RUNG 68's OWN table — the nine cells step 1 opened, now filled.**
pub const R68_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: r68_stator_leg,
    lagged_stator: r68_lagged_stator,
    clamp_v: r68_clamp_v,
    check_v0: r68_check_v0,
    rk4_floor: r68_rk4_floor,
    solve_v: r68_solve_v,
    manifold_v: r68_manifold_v,
    triple_laws: r68_triple_laws,
    triple_rig: r68_triple_rig,
    // RUNG 68 HAS NO `_with_ref` — the name arrives at rung 69. Rung 62's table carries rung 64's
    // `b_at_point` panic for exactly this reason, and this is that precedent's second use.
    with_ref: no_triple_with_ref,
    // AND NONE OF SLICE AD's THREE — all three names arrive at rung 72. Same precedent, third use.
    reference: no_triple_reference,
    rk4_floor_shared: no_triple_rk4_floor_shared,
    shared_rig: no_triple_shared_rig,
    // AND SLICE AE STEP 2's ADDED CELL — the FOURTH name of that family, arriving at rung 72
    // as well. Fourth use of the same precedent.
    quad_gains_at: no_triple_quad_gains_at,
};

// ---------------------------------------------------------------------------------------------
// THE NINE CELL BODIES
// ---------------------------------------------------------------------------------------------

/// RUNG 68's `_stator_leg` — WHICH limiter is armed. One of the four seams rung 69 reaches
/// through, and the identity of what it replaced, so every rung-68 arm is unchanged.
pub(crate) fn r68_stator_leg(t: &TwoSpoolTransientCore) -> Option<StatorLegArm> {
    t.stator.lim.map(StatorLegArm::from)
}

/// RUNG 68's `_lagged_stator` — `stator_lim is not None and stator_lim.tau is not None`.
///
/// **THE REDUCE IS BY DISPATCH THROUGH THIS NAME.** With it false, the five-state integrator is
/// not entered and `arm` never touches a map, so every inherited arm — rung 67, rung 66's three,
/// rung 65, rung 64, rung 52 — leaves through the parent bit-for-bit.
pub(crate) fn r68_lagged_stator(t: &TwoSpoolTransientCore) -> bool {
    t.stator.lim.is_some_and(|l| l.tau.is_some())
}

/// RUNG 68's `_clamp_v` — `min(0, max(-v_max, v))`.
///
/// The band is ONE-SIDED and its dormant stop is ZERO — the design setting — which is why the
/// clamp is asymmetric where the valve's is not. Rung 69 flips the open side, which is the whole
/// reason this is a cell.
pub(crate) fn r68_clamp_v(_: &TwoSpoolTransientCore, v: f64, lim_s: &StatorLegArm) -> f64 {
    0.0f64.min((-lim_s.v_max).max(v))
}

/// RUNG 68's `_check_v0` — the same band, on an OVERRIDDEN initial position.
pub(crate) fn r68_check_v0(_: &TwoSpoolTransientCore, v0: f64, lim_s: &StatorLegArm) {
    assert!((-lim_s.v_max..=0.0).contains(&v0),
            "rung-68 v0 is a stator POSITION on the one-sided band: {v0} is outside [{}, 0]",
            -lim_s.v_max);
}

/// RUNG 68's `_rk4_floor` — **THE MODELLING FLOOR, and it is TIGHTER AGAIN.**
///
/// § 2 makes `J` rank one with its non-zero eigenvalue exactly `-sum_i 1/tau_i`, so the
/// explicit-RK4 bound is on the SUM over however many clocks are armed. At three matched clocks
/// that reads `ds/tau <= 2/3` against rung 66's `1.0` and rung 65's `2.0`: **a sweep inheriting
/// rung 66's constant would run at 1.5x the admissible step.**
///
/// **IT IS A SEPARATE CELL SO THE REFUSAL CAN BE MEASURED RATHER THAN TRUSTED.** An assert nobody
/// has run past is a tautology, and rung 65 published a RETRACTION for exactly this failure mode.
/// What the band does here is WORSE than rung 65's, because it fails toward ZERO: at `ds = 0.05`
/// — admitted by rung 66's two-clock constant — the march reports the floor EXACTLY held,
/// `min phi_lp = 0.800000` and a violation integral of 0. **It counterfeits PERFECT PROTECTION.**
fn r68_rk4_floor(ds: f64, rate: f64, n_states: usize, tau_s: f64) {
    assert!(ds * rate <= 2.0,
            "rung-68: ds*sum(1/tau_i) = {:.3} is outside the explicit RK4 stability region for \
             the {n_states} actuator states (ds = {ds}, tau_s = {tau_s}). THE RATES ADD over \
             EVERY armed clock -- J has rank one, so the non-zero eigenvalue is exactly \
             -sum(1/tau_i) -- and bounding the fastest clock, or even rung 66's two of them, is \
             optimistic. Refine the grid or slow a clock; every tau -> 0 limit is APPROACHED on \
             this integrator and never reached.", ds * rate);
}

/// RUNG 68's `_solve_v` — **the stator's outer solve: the smallest `|v|` in `[-v_max, 0]` holding
/// `phi_lp >= phi_lim`.**
///
/// [`r64_solve_b`]'s structure with **BOTH CLAMP TESTS AND THE BRACKET ORIENTATION INVERTED**,
/// because `phi_lp` is DECREASING in `v` (measured `dphi/dv ~ -0.42`) where it is INCREASING in
/// `b`. The dormant stop is `v = 0` — the DESIGN setting, not an extreme — and the saturated one
/// is `-v_max`. Get that backwards and the regime label is wrong with nothing failing: rung 62's
/// `_powers` trap, fourth reload.
///
/// **THE REGIME IS RETURNED, NEVER INFERRED** — this rung's own trap is that a SATURATED loop
/// counterfeits INDEPENDENCE, so no reader may recover it by comparing `v` against a stop.
///
/// [`r64_solve_b`]: crate::limited_bleed::r64_solve_b
pub(crate) fn r68_solve_v(
    t: &TwoSpoolTransientCore,
    closer: &dyn Fn(f64) -> Result<FuelCloseState, Abort>,
) -> Result<(FuelCloseState, f64, Regime), Abort> {
    bump(&SOLVE_V_CALLS);
    let lim = t.stator.lim.expect("rung-68's `_solve_v` on a machine with no stator floor");
    let c0 = closer(0.0)?;
    if c0.base.phi_lp >= lim.phi_lim {
        bump(&REGIME_V_DORMANT);
        return Ok((c0, 0.0, Regime::Dormant));
    }
    let c1 = closer(-lim.v_max)?;
    if c1.base.phi_lp <= lim.phi_lim {
        bump(&REGIME_V_SATURATED);
        return Ok((c1, -lim.v_max, Regime::Saturated));
    }
    // Python's argument order is `(f, -v_max, 0.0, f(-v_max), f(0))` — the LOW end first, which
    // is `c1` and not `c0`. Transposing the two residuals is a wrong first secant that still
    // converges, to a root a few ulps away: the exact shape rung 68's own suite pins with
    // `test_authority_is_inert_on_the_triple_and_binds_on_the_lever_alone`'s comment.
    let v = try_illinois(|v| closer(v).map(|c| c.base.phi_lp - lim.phi_lim),
                         -lim.v_max, 0.0,
                         c1.base.phi_lp - lim.phi_lim, c0.base.phi_lp - lim.phi_lim,
                         1e-13, ILLINOIS_MAXIT)?;
    bump(&REGIME_V_RIDING);
    Ok((closer(v)?, v, Regime::Riding))
}

/// RUNG 68's `_manifold_v` — the base point § 2's algebra is STATED at.
///
/// At rung 68 all three laws hold ONE constraint, so **the stator's own root IS the shared
/// manifold** and the body is `V(g, q)[0]`. At rung 69 they do not and there is no point where
/// all three hold at once, which is why the four arguments this body ignores are carried.
#[allow(clippy::too_many_arguments)]
pub(crate) fn r68_manifold_v(
    _: &ScheduledStatorCore, _: &FlightCondition, _: f64, _: f64, _: f64, g: f64, q: f64,
    v_law: &dyn Fn(f64, f64) -> Result<(f64, Regime), Abort>,
) -> Result<f64, Abort> {
    Ok(v_law(g, q)?.0)
}

/// RUNG 68's `_closer_v` — [`r64_closer`]'s body one lever over, and the only public way to write
/// [`TwoSpoolTransientCore::v_forced`].
///
/// Python: *"A leaked trial setting would make the closure report a state the plant never visited
/// — rung 62's `_powers` failure mode, and the reason both overrides are always restored in a
/// `finally`."* Here the `finally` is [`ForcedStator`]'s `Drop`, which also runs on unwind.
pub(crate) fn closer_v<'a>(
    ft: &'a FuelTransientCore, a: f64, h: f64, mf: f64, tt2: f64, pt2: f64,
) -> impl Fn(f64) -> Result<FuelCloseState, Abort> + 'a {
    move |v: f64| {
        let _g = ForcedStator::set(&ft.inner, v);
        // **THE RUNG-62 PIN.** Python is `super(LimitedBleedTransient, self)._close_fuel`, which
        // resolves to `ScheduledBleedTransient`'s body from EVERY leaf in the chain — § 5.19 (ii)
        // measured all 16 such sites landing there regardless of depth. It is a static pin to one
        // ANCESTOR, not "the parent": at rung 68 the parent is rung 67 and the pin is rung 62.
        // The pin is on the FUNCTION and never on the TABLE — `ft` here is the LEAF core, so
        // every `self.X` inside rung 62's body still dispatches to the leaf.
        r62_try_close_fuel(ft, a, h, mf, tt2, pt2)
    }
}

/// RUNG 68's `_closer` — rung 64's, reproduced at this rung's call sites for the pin's sake.
pub(crate) fn closer_b<'a>(
    ft: &'a FuelTransientCore, a: f64, h: f64, mf: f64, tt2: f64, pt2: f64,
) -> impl Fn(f64) -> Result<FuelCloseState, Abort> + 'a {
    move |b: f64| {
        let _g = ForcedBleed::set(&ft.inner, b);
        r62_try_close_fuel(ft, a, h, mf, tt2, pt2)
    }
}

/// RUNG 68's `_triple_laws` — **§ 2's three control laws, and the whole point is that none of them
/// knows the others exist.**
///
/// Each solves `phi_lp = phi_lim` for ITS OWN actuator given the other two, each through a SHIPPED
/// closure. That mutual ignorance is what makes their products a MEASUREMENT of § 2's algebra
/// rather than a restatement of it.
///
/// # THE `b_state` / `v_state` BOUNDARY, WHICH IS THE RUNG'S OWN TRAP IN ITS FOURTH SHAPE
///
/// A law that TRIALS an actuator must not see that actuator's state, and MUST see the other two:
///
/// | law | trials | sets | reason |
/// |---|---|---|---|
/// | `R` (fuel) | neither | **both** | it solves no actuator, so it reads the plant as it is |
/// | `C` (valve) | `b` | `v_state` only | it solves against the plant as the STATORS actually are |
/// | `V` (stator) | `v` | `b_state` only | the exact mirror |
///
/// Getting the pair backwards converges a solver on a residual the plant never uses — rung 62's
/// `_powers` trap, and nothing raises.
fn r68_triple_laws<'a>(
    core: &'a ScheduledStatorCore, flight: &'a FlightCondition, a: f64, h: f64, mf_sched: f64,
    accel: Option<&'a AccelSchedule>, surge: Option<&'a Floor>,
) -> Result<TripleLaws<'a>, Abort> {
    bump(&TRIPLE_LAWS_CALLS);
    let ft = &core.fuel;
    let (tt2, pt2, _) = ft.inner.inlet(flight);

    // **R** — the FUEL law. Rung 52's leg is a `max(0, .)`, so it has a KINK at its own dormant
    // edge and a central difference straddling that kink returns the slope of neither branch;
    // the regime is therefore CARRIED and never re-derived from the float.
    let r = move |q: f64, v: f64| -> Result<(f64, LegRegime), Abort> {
        let _sb = MarchedBleed::set(&ft.inner, q);
        let _sv = MarchedStator::set(&ft.inner, v);
        let mut caps: Vec<f64> = Vec::new();
        if let Some(ac) = accel {
            caps.push(ft.try_sched_fuel(flight, a, h, mf_sched, ac)?);
        }
        if let Some(su) = surge {
            caps.push(ft.try_surge_fuel(flight, a, h, mf_sched, su)?);
        }
        let raw = match caps.iter().copied().reduce(f64::min) {
            Some(m) => mf_sched - m,
            None => 0.0,
        };
        Ok((0.0f64.max(raw),
            if raw > 0.0 { LegRegime::Riding } else { LegRegime::Dormant }))
    };

    // **C** — the VALVE law: it trials `b`, so NO `b_state`, but `v_state = v`.
    let c = move |g: f64, v: f64| -> Result<(f64, Regime), Abort> {
        let _sv = MarchedStator::set(&ft.inner, v);
        let bl = ft.inner.lever.lim.expect("rung-68's valve law on an unfloored machine");
        let (_, b, reg) = crate::limited_bleed::r64_solve_b(
            &bl, closer_b(ft, a, h, 1e-9f64.max(mf_sched - g), tt2, pt2))?;
        Ok((b, reg))
    };

    // **V** — the STATOR law: the exact mirror, trialling `v` with `b_state = q`.
    let v = move |g: f64, q: f64| -> Result<(f64, Regime), Abort> {
        let _sb = MarchedBleed::set(&ft.inner, q);
        let (_, vv, reg) = ft.inner.solve_v(
            &closer_v(ft, a, h, 1e-9f64.max(mf_sched - g), tt2, pt2))?;
        Ok((vv, reg))
    };

    Ok(TripleLaws { r: Box::new(r), c: Box::new(c), v: Box::new(v) })
}

/// RUNG 68's `_triple_rig` — **ONE constructor for every ledger cell.**
///
/// A cell can never differ from another by anything except which loops are armed — rung 63's
/// lesson, and the reason the credits are differenceable at all. Every floor comes from the SAME
/// `from_margin(cmap, ., sm)`, which is what makes this ONE set point rather than three numbers
/// that happen to agree (§ 2's scope).
fn r68_triple_rig(
    core: &ScheduledStatorCore, arm: &TripleRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    let cmap = core.arming().map_lp_design;
    let b_max = core.fuel.inner.lever.lim.map(|l| l.b_max).unwrap_or(0.10);
    let bl = if arm.valve {
        Some(BleedLimiter::from_margin_tau(&cmap, b_max, arm.sm, Some(arm.tau)))
    } else {
        None
    };
    let sl = if arm.stator {
        Some(StatorLimiter::from_margin(&cmap, arm.v_max, arm.sm, Some(arm.tau_s)))
    } else {
        None
    };
    let m = core.at_lever(&LeverArm { bleed_lim: bl, stator_lim: sl, ..Default::default() });
    let surge = if arm.fuel {
        Some(Floor::Phi(SurgeLimiter::from_margin(&cmap, Spool::Lp, arm.sm)))
    } else {
        None
    };
    let lag = if arm.fuel {
        Some(AsymmetricLag::new(arm.tau_att, arm.tau_rel))
    } else {
        None
    };
    (m, surge, lag)
}

// ---------------------------------------------------------------------------------------------
// THE THREE SWAPPED STATOR CELLS, `at_lever`, AND THE POINT READER
// ---------------------------------------------------------------------------------------------

/// RUNG 68's `_arm` — rung 57's schedule arming with ONE addition: **a live limiter position
/// overrides the LP map**, applied EXACTLY the way rung 53's constructor applies a constant
/// setting, so both derived channels move together.
///
/// `v_forced` wins over `v_state` for rung 65's reason one lever over: the stator's own command
/// solve trials settings on a plant whose live setting is the one being commanded away from.
/// **Neither set — every STEADY solve, and every reduce arm — leaves this a pure call to the
/// parent**, which is what keeps the initial running line identical to the machine this rung is
/// compared against.
fn r68_arm(t: &TwoSpoolTransientCore, nu_lp: f64, nu_hp: f64, tt2: f64) {
    r57_arm(t, nu_lp, nu_hp, tt2);
    if t.stator_leg().is_none() {
        bump(&ARM68_UNARMED);
        return;
    }
    let v = match t.v_forced.get().or_else(|| t.v_state.get()) {
        Some(v) => v,
        None => {
            bump(&ARM68_NO_POSITION);
            return;
        }
    };
    let design = t.stator.map_lp_design;
    if v == 0.0 {
        bump(&ARM68_ZERO);
        t.inner.set_map_lp(design);
    } else {
        bump(&ARM68_MOVED);
        t.inner.set_map_lp(design.with_vsv(v));
    }
}

/// RUNG 68's `v_of` — rung 57's reader with the live limiter position on top.
///
/// **A LAGGED SETTING IS NOT A FUNCTION OF THE STATE** — it carries history — so outside a march
/// this hands back the parent's answer and [`v_at_point`] is the only way to recover a marched
/// one. That is rung 65's `b_at_point` correction, one lever over.
fn r68_v_of(
    t: &TwoSpoolTransientCore, spool: Spool, nu_lp: f64, nu_hp: f64, tt2: Option<f64>,
) -> f64 {
    if spool == Spool::Lp && t.stator_leg().is_some() {
        if let Some(v) = t.v_forced.get().or_else(|| t.v_state.get()) {
            bump(&VOF68_LIVE);
            return v;
        }
    }
    r57_v_of(t, spool, nu_lp, nu_hp, tt2)
}

/// RUNG 68's `at_lever` — **the SIXTH instance of the sibling-constructor trap**, and the first
/// where the signature genuinely GROWS.
///
/// So the failure mode is no longer only *hands back the wrong class* but also *silently drops the
/// third loop*. Both are caught by the same gate, which asserts the sibling still marches the
/// five-state integrator.
fn r68_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    match build_three_loop_cascade(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    }
}

/// RUNG 68's `_stator_march` — rung 67's with TWO additions, both ISOLATION DIAGNOSTICS and
/// neither a control setting.
///
/// `v0` overrides the stator's initial position (rung 65's `b0`, one lever over) and `ic_order`
/// selects which member of the `s = 0` family the joint solve lands on. Both default to absent and
/// leave every inherited march bit-for-bit.
///
/// This rung consumes exactly TWO fields of the scope and passes the other three on, as rung 67
/// consumes one and passes two.
pub fn r68_stator_march(
    ft: &FuelTransientCore, flight: &FlightCondition, ramp: &Ramp, nu0: Option<(f64, f64)>,
    leg: &StatorLeg<'_>, scope: &MarchScope,
) -> (Vec<FuelPoint>, (f64, f64)) {
    let _gv = InitialStator::set(&ft.inner, scope.v0);
    let _go = DeclaredOrder::set(&ft.inner, scope.ic_order);
    crate::cross_loop::r67_stator_march(
        ft, flight, ramp, nu0,
        leg,
        &MarchScope { b0: scope.b0, lag: scope.lag, tau_gov: scope.tau_gov,
                      ..MarchScope::DEFAULT })
}

// ---------------------------------------------------------------------------------------------
// THE MARCH — a FIFTH state
// ---------------------------------------------------------------------------------------------

/// RUNG 68's `integrate_fuel` — the dispatch and **three refusals**.
///
/// | reduce arm | condition | lands on |
/// |---|---|---|
/// | rungs 67 / 66 / 65 / 64 / 52 | `_lagged_stator()` false | rung 67's `integrate_fuel`, untouched |
///
/// **THE REDUCE IS BY DISPATCH AND NOT BY A ZERO**: `stator_lim is None` means the five-state
/// integrator is not entered and `arm` never touches a map, so every inherited arm leaves through
/// the parent bit-for-bit.
pub fn r68_integrate_fuel(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
) -> Vec<FuelPoint> {
    let lag = lim.lag.or_else(|| ft.inner.lag.get());
    // Python: *"RUNG 67's clock rides on an instance attribute, and `ScheduledStatorTransient.
    // _stator_march` — the one every reader in this family actually calls — does NOT forward it
    // as a keyword. Reading only the argument would let a rung-68 march accept `tau_gov` and
    // SILENTLY IGNORE the governor, with the refusal below never firing."*
    let tau_gov = lim.tau_gov.or_else(|| ft.inner.tau_gov.get());
    if !ft.inner.lagged_stator() {
        bump(&INTEGRATE68_REDUCED);
        return crate::cross_loop::r67_integrate_fuel(
            ft, flight, fuel_schedule, nu0, s_end, ds,
            &FuelLimiters { tau_gov, lag, ..lim.clone() });
    }
    assert!(tau_gov.is_none(),
            "rung-68 is THREE LOOPS ON ONE VARIABLE: rung 52's phi fuel leg, rung 65's phi valve \
             and rung 68's phi stator, all on `phi_lim`. Rung 47's tau_gov watches `Tt4`, a \
             DIFFERENT variable -- adding it is THREE loops on TWO variables, which s 2's algebra \
             says superposes rung 67's P<0 block onto this rank-one one. That is rung 68's own \
             next seam, asserted against rather than run.");
    assert!(lim.s_off.is_none() && lim.tau_rel.is_none(),
            "rung-68: rungs 50/51's FORCED release edges are an isolation instrument for a leg \
             that could not pin its own trigger. All three legs here pin their own, so forcing \
             one would measure the forcing (rung 66's argument, one loop on).");
    assert!(ft.inner.lever.lim.is_none() || lagged(&ft.inner),
            "rung-68: an INSTANTANEOUS valve beside a lagged stator is not a control but a \
             different plant (rung 65 called the instantaneous limit singular, and rung 66 \
             refused the comparison for that reason). Give the valve a `tau` or leave it out.");
    r68_integrate_fuel_triple(ft, flight, fuel_schedule, nu0, s_end, ds, lim, lag)
}

/// One derivative evaluation's full return — Python's eleven-tuple out of `der`.
struct TripleDer {
    da: f64,
    dh: f64,
    dg: f64,
    dq: f64,
    dv: f64,
    mf: f64,
    inst: FuelInstant,
    req: f64,
    cmd: f64,
    vcmd: f64,
    vreg: Regime,
}

/// RUNG 68's MARCH — **rung 66's merged integrator with the stator setting as a FIFTH state**, and
/// the two optional legs genuinely optional so the SAME integrator produces the ledger's `S`,
/// `FS`, `VS` and `FVS` cells.
///
/// Every key rungs 52/65/66 record is recorded here byte-unchanged, plus `v`/`v_cmd`/`v_regime`,
/// so every rung-52/65/66/67 reader works on this trajectory too.
///
/// # THE JOINT INITIAL CONDITION IS A ONE-PARAMETER FAMILY, NOT A FIXED POINT
///
/// Rung 66's joint solve converged in ONE iteration at every corner it tested because
/// `required(0) == 0` there — the fuel leg opens dormant, `R_q == 0`, contraction 0. **That escape
/// is gone at `n = 3`**: the valve is live at `s = 0` (rung 66 measured `b0 = 0.037`) and so is the
/// stator, and those two SHARE the constraint, so their pairwise contraction is `|C_v V_q| = 1`
/// exactly — marginal. The set of joint fixed points is a CURVE, and a Gauss-Seidel sweep lands on
/// whichever member its ORDER selects.
///
/// **THE ORDER IS DECLARED, NEVER INFERRED**: `g -> q -> v`, rung 66's order with the new actuator
/// appended last, so the rung-66 arm is reached unchanged and the stator takes up only what the
/// pair leaves. `ic_family` reports the alternatives as the sensitivity they are.
///
/// **AND THE STARTING MEMBER IS LOAD-BEARING RATHER THAN COSMETIC.** Measured: initialising all
/// three at zero instead lands the sweep on a DIFFERENT member (the fuel leg takes the whole clip,
/// `g0 = 2.0e-3` against rung 66's exact 0) and moves `min phi_lp` in the fifth figure.
#[allow(clippy::too_many_arguments)]
pub fn r68_integrate_fuel_triple(
    ft: &FuelTransientCore, flight: &FlightCondition, fuel_schedule: &dyn Fn(f64) -> f64,
    nu0: (f64, f64), s_end: f64, ds: f64, lim: &FuelLimiters<'_>,
    lag: Option<AsymmetricLag>,
) -> Vec<FuelPoint> {
    let lim_s = ft.inner.stator_leg().expect("rung-68's march with no stator floor");
    let tau_s = lim_s.tau.expect("rung-68's march on an unlagged stator");
    let has_q = lagged(&ft.inner);
    let has_g = lag.is_some() && (lim.accel.is_some() || lim.floor().is_some());
    let freeze = lim.freeze;
    let tt4_max = lim.tt4_max;
    let accel = lim.accel;
    let surge = lim.floor();

    // THE MODELLING FLOOR, and it is TIGHTER AGAIN — see [`r68_rk4_floor`], which is a CELL so the
    // refusal can be measured rather than trusted.
    let mut rate = 1.0 / tau_s;
    if has_q {
        rate += 1.0 / ft.inner.lever.lim.expect("has_q").tau.expect("has_q");
    }
    if has_g {
        let l = lag.expect("has_g");
        rate += 1.0 / l.tau_att.min(l.tau_rel);
    }
    ft.inner.rk4_floor(ds, rate, 1 + usize::from(has_q) + usize::from(has_g), tau_s);
    let (tt2, pt2, _) = ft.inner.inlet(flight);

    // THE VALVE law. Rung 64's root over TRIAL positions, so NO `b_state` -- but `v_state` IS set,
    // because the valve solves its command against the plant as the STATORS actually are.
    let command = |a: f64, h: f64, mf: f64, v: f64| -> Result<f64, Abort> {
        if !has_q {
            return Ok(0.0);
        }
        let _sv = MarchedStator::set(&ft.inner, v);
        let bl = ft.inner.lever.lim.expect("has_q");
        Ok(crate::limited_bleed::r64_solve_b(&bl, closer_b(ft, a, h, mf, tt2, pt2))?.1)
    };

    // THE STATOR law, and the mirror image: it trials `v`, so NO `v_state`, but `b_state = q`.
    // Returns `(v, regime)` — the regime is CARRIED, never re-derived from the float.
    let stator = |a: f64, h: f64, mf: f64, q: f64| -> Result<(f64, Regime), Abort> {
        let _sb = MarchedBleed::set(&ft.inner, q);
        let (_, v, reg) = ft.inner.solve_v(&closer_v(ft, a, h, mf, tt2, pt2))?;
        Ok((v, reg))
    };

    // THE FUEL law. It trials NEITHER other actuator, so it sees BOTH states. Solved from the
    // SCHEDULED fuel (rung 52's discipline verbatim) so arming one leg cannot perturb another's
    // bracket.
    let required = |a: f64, h: f64, q: f64, v: f64, mf_sched: f64| -> Result<f64, Abort> {
        if !has_g {
            return Ok(0.0);
        }
        let _sb = MarchedBleed::set(&ft.inner, q);
        let _sv = MarchedStator::set(&ft.inner, v);
        let mut caps: Vec<f64> = Vec::new();
        if let Some(ac) = accel {
            caps.push(ft.try_sched_fuel(flight, a, h, mf_sched, ac)?);
        }
        if let Some(su) = surge.as_ref() {
            caps.push(ft.try_surge_fuel(flight, a, h, mf_sched, su)?);
        }
        Ok(match caps.iter().copied().reduce(f64::min) {
            Some(m) => 0.0f64.max(mf_sched - m),
            None => 0.0,
        })
    };

    let der = |a: f64, h: f64, g: f64, q: f64, v: f64, s: f64| -> Result<TripleDer, Abort> {
        let mf_sched = fuel_schedule(s);
        let req = required(a, h, q, v, mf_sched)?;
        let mut mf = 1e-9f64.max(mf_sched - g);
        let inst = {
            let _sb = MarchedBleed::set(&ft.inner, q);
            let _sv = MarchedStator::set(&ft.inner, v);
            if let Some(tmax) = tt4_max {
                // The UNLAGGED redline, rung 52's placement.
                if ft.try_instant_fuel(flight, a, h, mf)?.base.tt4 > tmax {
                    mf = mf.min(ft.try_topping_fuel(flight, a, h, tmax, mf)?);
                }
            }
            ft.try_instant_fuel(flight, a, h, mf)?
        };
        let cmd = command(a, h, mf, v)?;
        let (vcmd, vreg) = stator(a, h, mf, q)?;
        let da = if freeze == Some(Spool::Lp) { 0.0 } else { inst.base.phi_lp_dot / ft.rho() };
        let dh = if freeze == Some(Spool::Hp) { 0.0 } else { inst.base.phi_hp_dot };
        let dg = if has_g { (req - g) / lag.expect("has_g").tau(req, g) } else { 0.0 };
        let dq = if has_q {
            (cmd - q) / ft.inner.lever.lim.expect("has_q").tau.expect("has_q")
        } else {
            0.0
        };
        Ok(TripleDer { da, dh, dg, dq, dv: (vcmd - v) / tau_s, mf, inst, req, cmd, vcmd, vreg })
    };

    // --- THE JOINT INITIAL CONDITION ----------------------------------------------------------
    let (mut a, mut h) = nu0;
    let mf0 = fuel_schedule(0.0);
    let v0 = ft.inner.v0.get();
    if let Some(x) = v0 {
        ft.inner.check_v0(x, &lim_s);
    }
    // Python raises out of the whole method here — the initial solves sit BEFORE the loop's `try`.
    let raise = |e: Abort| -> ! { panic!("{}", e.0) };
    let mut g = 0.0f64;
    let mut q = command(a, h, mf0, 0.0).unwrap_or_else(|e| raise(e));
    let mut v = v0.unwrap_or(0.0);
    let b0 = ft.inner.b0.get();
    if let Some(x) = b0 {
        q = x;
    }
    let order = ft.inner.ic_order.get();
    assert!({
                let mut cs: Vec<char> = order.chars().collect();
                cs.sort_unstable();
                cs == ['g', 'q', 'v']
            },
            "rung-68 ic_order is a permutation of 'gqv'; got {order:?}");
    let mut res = f64::INFINITY;
    let mut its = 0usize;
    for i in 1..=60usize {
        its = i;
        let (mut gn, mut qn, mut vn) = (g, q, v);
        for k in order.chars() {
            match k {
                'g' => gn = required(a, h, qn, vn, mf0).unwrap_or_else(|e| raise(e)),
                'q' => {
                    if b0.is_none() {
                        qn = command(a, h, 1e-9f64.max(mf0 - gn), vn)
                            .unwrap_or_else(|e| raise(e));
                    }
                }
                'v' => {
                    if v0.is_none() {
                        vn = stator(a, h, 1e-9f64.max(mf0 - gn), qn)
                            .unwrap_or_else(|e| raise(e)).0;
                    }
                }
                _ => unreachable!("the permutation assert above admits only g/q/v"),
            }
        }
        res = py_max3((gn - g).abs(), (qn - q).abs(), (vn - v).abs());
        g = gn;
        q = qn;
        v = vn;
        if res <= 1e-12 {
            break;
        }
    }
    assert!(res <= 1e-9,
            "rung-68: the joint initial condition did not converge (residual {res:.3e} after \
             {its} iterations) in order {order:?}. s 2 makes the actuator block RANK ONE, so the \
             s = 0 fixed points are a CURVE and a sweep can only land on a member, never contract \
             to a point. This is the degeneracy at s = 0 and it is a FINDING: report the state \
             and the order, do not raise the iteration cap.");

    // --- THE RK4 LOOP -------------------------------------------------------------------------
    let mut pts: Vec<FuelPoint> = Vec::new();
    let mut s = 0.0f64;
    let n_steps = (s_end / ds).round_ties_even() as i64;
    for _ in 0..=n_steps {
        let Ok(k1) = der(a, h, g, q, v, s) else { break };
        pts.push(point(s, a, h, &k1.inst, k1.mf, fuel_schedule(s),
                       PointExtra::Triple { g, required: k1.req, b: q, b_cmd: k1.cmd,
                                            v, v_cmd: k1.vcmd, v_regime: k1.vreg,
                                            ic_iters: its, ic_res: res, ic_order: order }));
        let stages = (|| -> Result<[f64; 15], Abort> {
            let k2 = der(a + ds / 2.0 * k1.da, h + ds / 2.0 * k1.dh, g + ds / 2.0 * k1.dg,
                         q + ds / 2.0 * k1.dq, v + ds / 2.0 * k1.dv, s + ds / 2.0)?;
            let k3 = der(a + ds / 2.0 * k2.da, h + ds / 2.0 * k2.dh, g + ds / 2.0 * k2.dg,
                         q + ds / 2.0 * k2.dq, v + ds / 2.0 * k2.dv, s + ds / 2.0)?;
            let k4 = der(a + ds * k3.da, h + ds * k3.dh, g + ds * k3.dg, q + ds * k3.dq,
                         v + ds * k3.dv, s + ds)?;
            Ok([k2.da, k2.dh, k2.dg, k2.dq, k2.dv,
                k3.da, k3.dh, k3.dg, k3.dq, k3.dv,
                k4.da, k4.dh, k4.dg, k4.dq, k4.dv])
        })();
        let Ok([k2a, k2h, k2g, k2q, k2v, k3a, k3h, k3g, k3q, k3v, k4a, k4h, k4g, k4q, k4v]) =
            stages else { break };
        a += ds / 6.0 * (k1.da + 2.0 * k2a + 2.0 * k3a + k4a);
        h += ds / 6.0 * (k1.dh + 2.0 * k2h + 2.0 * k3h + k4h);
        g += ds / 6.0 * (k1.dg + 2.0 * k2g + 2.0 * k3g + k4g);
        q += ds / 6.0 * (k1.dq + 2.0 * k2q + 2.0 * k3q + k4q);
        v += ds / 6.0 * (k1.dv + 2.0 * k2v + 2.0 * k3v + k4v);
        // EVERY POSITION IS PHYSICAL (rung 65, verbatim): the actuators' own hardware stops,
        // applied to the STATE and never to a command. The stator's band is ONE-SIDED and its
        // dormant stop is ZERO — the design setting — which is why the clamp is asymmetric where
        // the valve's is not, and why it goes through a CELL.
        if has_q {
            let bmax = ft.inner.lever.lim.expect("has_q").b_max;
            q = bmax.min(0.0f64.max(q));
        }
        v = ft.inner.clamp_v(v, &lim_s);
        g = 0.0f64.max(g);
        s += ds;
    }
    pts
}

// ---------------------------------------------------------------------------------------------
// COUNTERS — the reduce, the regimes and the arming arms are ALL invisible to every value key
// ---------------------------------------------------------------------------------------------
//
// Three things this rung does are unreachable from a float a reader can print:
//
// * **THE REDUCE.** `stator_lim is None` hands straight back to rung 67's `integrate_fuel`, so a
//   reduce arm and a rung-68 march with a dormant stator emit the same numbers by construction.
// * **THE REGIME.** `solve_v` returns it as its third element and — exactly as at rung 64 —
//   nothing in the ladder reads it. `v_regime` reaches a trajectory point and `_riding` filters
//   on it, but the DISTRIBUTION over the three branches is what a gate has to assert on.
// * **`arm`'s FOUR ARMS.** Unarmed / no-position / zero / moved are four different bodies and
//   three of them leave the map exactly where they found it.

thread_local! {
    static SOLVE_V_CALLS: Cell<u64> = const { Cell::new(0) };
    static REGIME_V_DORMANT: Cell<u64> = const { Cell::new(0) };
    static REGIME_V_RIDING: Cell<u64> = const { Cell::new(0) };
    static REGIME_V_SATURATED: Cell<u64> = const { Cell::new(0) };
    static TRIPLE_LAWS_CALLS: Cell<u64> = const { Cell::new(0) };
    static INTEGRATE68_REDUCED: Cell<u64> = const { Cell::new(0) };
    static ARM68_UNARMED: Cell<u64> = const { Cell::new(0) };
    static ARM68_NO_POSITION: Cell<u64> = const { Cell::new(0) };
    static ARM68_ZERO: Cell<u64> = const { Cell::new(0) };
    static ARM68_MOVED: Cell<u64> = const { Cell::new(0) };
    static VOF68_LIVE: Cell<u64> = const { Cell::new(0) };
}

fn bump(c: &'static std::thread::LocalKey<Cell<u64>>) {
    c.with(|x| x.set(x.get() + 1));
}

/// What the counters above hold — read by `slice_aa_dispatch.rs`, which is the only file that can
/// see any of it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Census68 {
    pub solve_v_calls: u64,
    pub regime_dormant: u64,
    pub regime_riding: u64,
    pub regime_saturated: u64,
    pub triple_laws_calls: u64,
    pub integrate_reduced: u64,
    pub arm_unarmed: u64,
    pub arm_no_position: u64,
    pub arm_zero: u64,
    pub arm_moved: u64,
    pub v_of_live: u64,
}

impl Census68 {
    pub fn read() -> Self {
        Census68 {
            solve_v_calls: SOLVE_V_CALLS.with(|x| x.get()),
            regime_dormant: REGIME_V_DORMANT.with(|x| x.get()),
            regime_riding: REGIME_V_RIDING.with(|x| x.get()),
            regime_saturated: REGIME_V_SATURATED.with(|x| x.get()),
            triple_laws_calls: TRIPLE_LAWS_CALLS.with(|x| x.get()),
            integrate_reduced: INTEGRATE68_REDUCED.with(|x| x.get()),
            arm_unarmed: ARM68_UNARMED.with(|x| x.get()),
            arm_no_position: ARM68_NO_POSITION.with(|x| x.get()),
            arm_zero: ARM68_ZERO.with(|x| x.get()),
            arm_moved: ARM68_MOVED.with(|x| x.get()),
            v_of_live: VOF68_LIVE.with(|x| x.get()),
        }
    }

    /// Thread-local with no per-test reset, so every gate resets first. Cargo gives each `#[test]`
    /// its own thread today; the reset makes that irrelevant rather than relied upon.
    pub fn reset() {
        for c in [&SOLVE_V_CALLS, &REGIME_V_DORMANT, &REGIME_V_RIDING, &REGIME_V_SATURATED,
                  &TRIPLE_LAWS_CALLS, &INTEGRATE68_REDUCED, &ARM68_UNARMED, &ARM68_NO_POSITION,
                  &ARM68_ZERO, &ARM68_MOVED, &VOF68_LIVE] {
            c.with(|x| x.set(0));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// THE POINT READER
// ---------------------------------------------------------------------------------------------

/// RUNG 68's `v_at_point` — the marched stator setting at a recorded trajectory point.
///
/// **RECORDED, NEVER RE-SOLVED**: a lagged position carries history, and re-solving would silently
/// hand back the COMMAND. That is rung 65's `b_at_point` correction, one lever over, and § 5.22
/// (ii) measured what the re-solving version costs at rung 64 — both published ratios to exactly
/// zero, with all 111 gates still passing.
///
/// It PANICS on every other route because Python raises `AssertionError` there: a point that did
/// not record `v` came from a different integrator.
pub fn v_at_point(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Triple { v, .. }
        // SLICE AD (5 of 13): rung 72 records `v`, so refusing here would be stricter
        // than Python on a dict that carries the key.
        | PointExtra::Shared { v, .. } => v,
        _ => panic!(
            "rung-68: a lagged stator setting is a march STATE and cannot be recovered from a \
             trajectory point that did not record it. This point came from a different \
             integrator."),
    }
}

/// The `(ic_iters, ic_res, ic_order)` a rung-68 point carries — a PANIC on every other route.
pub fn ic_at_point(p: &FuelPoint) -> (usize, f64, &'static str) {
    match p.extra {
        PointExtra::Triple { ic_iters, ic_res, ic_order, .. }
        // SLICE AD (6 of 13): rung 72 runs a FOUR-way sweep and records all three.
        | PointExtra::Shared { ic_iters, ic_res, ic_order, .. } => (ic_iters, ic_res, ic_order),
        _ => panic!("rung-68: this trajectory carries no joint initial condition"),
    }
}

// ---------------------------------------------------------------------------------------------
// § 2 — THE SIX CROSS-GAINS, AND THE ONE INDEPENDENT PRODUCT
// ---------------------------------------------------------------------------------------------

/// The six central differences at one trajectory point, plus the four products § 2 is about.
#[derive(Clone, Debug, PartialEq)]
pub struct TripleGains {
    /// Was EVERY perturbed evaluation riding-interior? A dropped point is a COVERAGE CLAIM, so
    /// the caller reports the count rather than filtering silently.
    pub interior: bool,
    /// Which arms were off-regime, in Python's own key order.
    pub off_regime: Vec<&'static str>,
    pub r_q: f64,
    pub r_v: f64,
    pub c_g: f64,
    pub c_v: f64,
    pub v_g: f64,
    pub v_q: f64,
    pub v_base: f64,
    /// `R_q * C_v * V_g` — **the only product that tests JOINT collapse**, and § 2's genuinely
    /// new claim. Predicted `-1`.
    pub cyclic: f64,
    /// Rung 66's identity, three times. Each predicted `1`.
    pub pair_rc: f64,
    pub pair_rv: f64,
    pub pair_cv: f64,
    /// The `s` of the point this was taken at.
    pub s: f64,
}

/// RUNG 68's `_triple_gains_at` — **the six central differences at one trajectory point.**
///
/// `manifold = true` puts the stator ON the shared manifold (`v = V(g, q)`, optionally displaced
/// by `delta`) before differencing — the EXACT statement of § 2, which assumes all three laws
/// evaluated at one common point. `manifold = false` differences at the LIVE marched `v`, which is
/// rung 66's own choice and is OFF the manifold during a transient; rung 66 measured a ±3.5 %
/// residual departure there for exactly this reason.
///
/// The three step sizes differ by orders because the three arguments do: `g` is a fuel clip of
/// order 1e-3 kg/s, `q` a valve fraction on `[0, 0.1]`, `v` a stator setting of order 1e-2.
///
/// # EVERY PERTURBED EVALUATION IS REGIME-CHECKED, NOT JUST THE BASE POINT
///
/// **This is the rung's own trap in its third place.** A base point can be comfortably riding while
/// one arm of a central difference has already crossed into `dormant` or onto a stop; the
/// difference then measures the KINK, not the gain. Measured cost of ignoring it: `c1` — which § 2
/// predicts ≈ 0 — came back at `1.3e+2` on a handful of edge points while the interior ones sat at
/// `1e-8`.
#[allow(clippy::too_many_arguments)]
pub fn triple_gains_at(
    core: &ScheduledStatorCore, flight: &FlightCondition, p: &FuelPoint,
    accel: Option<&AccelSchedule>, surge: Option<&Floor>,
    dg: f64, dq: f64, dv: f64, manifold: bool, delta: f64, strict: bool,
) -> Result<TripleGains, Abort> {
    let (a, h, mf_sched) = (p.nu_lp, p.nu_hp, p.mf_sched);
    let g = match p.extra {
        PointExtra::Triple { g, .. }
        // SLICE AD (7 of 13): `g` is the APPLIED clip on a rung-72 point.
        | PointExtra::Shared { g, .. } => g,
        _ => panic!("rung-68's gains need a five-state trajectory"),
    };
    // `valve_of` returns `(b, b_cmd)`; the gains are differenced against the POSITION.
    let q = crate::lagged_bleed::valve_of(p).0;
    let laws = core.triple_laws(flight, a, h, mf_sched, accel, surge)?;
    let v = if manifold {
        core.manifold_v(flight, a, h, mf_sched, g, q, &*laws.v)? + delta
    } else {
        v_at_point(p)
    };

    // Python builds a dict of twelve `(key, value)` pairs in ONE literal, so every evaluation runs
    // BEFORE any regime is inspected. Reproduced as a `Vec` in the same order: short-circuiting on
    // the first off-regime arm would change how many closure calls the plant sees, which is a
    // difference the counters can read even where the floats agree.
    let ev: Vec<(&'static str, f64, bool)> = vec![
        leg_ev("R+q", (laws.r)(q + dq, v)?),
        leg_ev("R-q", (laws.r)(q - dq, v)?),
        leg_ev("R+v", (laws.r)(q, v + dv)?),
        leg_ev("R-v", (laws.r)(q, v - dv)?),
        reg_ev("C+g", (laws.c)(g + dg, v)?),
        reg_ev("C-g", (laws.c)(g - dg, v)?),
        reg_ev("C+v", (laws.c)(g, v + dv)?),
        reg_ev("C-v", (laws.c)(g, v - dv)?),
        reg_ev("V+g", (laws.v)(g + dg, q)?),
        reg_ev("V-g", (laws.v)(g - dg, q)?),
        reg_ev("V+q", (laws.v)(g, q + dq)?),
        reg_ev("V-q", (laws.v)(g, q - dq)?),
    ];
    let off: Vec<&'static str> = ev.iter().filter(|(_, _, r)| !r).map(|(k, _, _)| *k).collect();
    if !off.is_empty() && strict {
        return Ok(TripleGains {
            interior: false, off_regime: off, s: p.s, v_base: v,
            r_q: f64::NAN, r_v: f64::NAN, c_g: f64::NAN, c_v: f64::NAN,
            v_g: f64::NAN, v_q: f64::NAN, cyclic: f64::NAN,
            pair_rc: f64::NAN, pair_rv: f64::NAN, pair_cv: f64::NAN,
        });
    }
    let at = |k: &str| ev.iter().find(|(n, _, _)| *n == k).expect("the twelve keys above").1;
    let d = |kp: &str, km: &str, h2: f64| (at(kp) - at(km)) / (2.0 * h2);
    let (r_q, r_v) = (d("R+q", "R-q", dq), d("R+v", "R-v", dv));
    let (c_g, c_v) = (d("C+g", "C-g", dg), d("C+v", "C-v", dv));
    let (v_g, v_q) = (d("V+g", "V-g", dg), d("V+q", "V-q", dq));
    Ok(TripleGains {
        interior: off.is_empty(),
        off_regime: off,
        r_q, r_v, c_g, c_v, v_g, v_q,
        v_base: v,
        cyclic: r_q * c_v * v_g,
        pair_rc: r_q * c_g,
        pair_rv: r_v * v_g,
        pair_cv: c_v * v_q,
        s: p.s,
    })
}

fn leg_ev(k: &'static str, x: (f64, LegRegime)) -> (&'static str, f64, bool) {
    (k, x.0, x.1 == LegRegime::Riding)
}

fn reg_ev(k: &'static str, x: (f64, Regime)) -> (&'static str, f64, bool) {
    (k, x.0, x.1 == Regime::Riding)
}

/// RUNG 68's `_riding` — trajectory points where **ALL THREE loops are live and STRICTLY
/// INTERIOR.**
///
/// **THE FILTER IS THE INSTRUMENT, not bookkeeping.** Rung 67's lesson is that a zero cross-gain is
/// SATURATION and never decoupling; this rung's own trap is the inverse — a saturated loop
/// counterfeits INDEPENDENCE by removing its own row from the coupling. The stator is filtered on
/// the REGIME LABEL `solve_v` returned and **never on a float comparison against a stop**.
pub fn riding(traj: &[FuelPoint], b_max: f64) -> Vec<FuelPoint> {
    traj.iter()
        .filter(|p| match p.extra {
            PointExtra::Triple { required, b_cmd, v_regime, .. }
            // SLICE AD (8 of 13): a `false` fallback in a FILTER is the quietest failure
            // of the thirteen -- the reader returns an empty set and every downstream
            // statistic is computed over nothing at all.
            | PointExtra::Shared { required, b_cmd, v_regime: Some(v_regime), .. } =>
                required > 0.0 && 0.0 < b_cmd && b_cmd < b_max && v_regime == Regime::Riding,
            // A stator-less rung-72 point is not RIDING a stator it does not have. `false`
            // here is the same answer Python gives (`p.get("v_regime") == "riding"` on
            // `None`), not a fallback.
            PointExtra::Shared { v_regime: None, .. } => false,
            _ => false,
        })
        .copied()
        .collect()
}

// ---------------------------------------------------------------------------------------------
// § 2 — THE SPECTRUM: two zeros, and the rates add
// ---------------------------------------------------------------------------------------------

/// Roots of `l^3 - c2 l^2 + c1 l - c0`, by **Newton on the dominant root followed by exact
/// deflation.**
///
/// Adequate here because the predicted spectrum is `{0, 0, c2}` — one well-separated root and a
/// deflated quadratic — and it keeps the module free of a linear-algebra dependency.
///
/// # THE ARITHMETIC, AND THE RISK THAT MEASURED TO ZERO
///
/// This is the one body in slice AA with no precedent in the crate, and the risk registered
/// against it before it was read was **Cardano's trigonometric branch and a cube-root spelling** —
/// `x ** (1/3)` is a libm `pow`, `f64::cbrt` is a different instruction, and the two disagree in
/// the last bit under a rule `lib.rs`'s three-spellings note does not carry. **There is no cube
/// root.** `probe_aa6.py` swept the whole rung: ONE `**` site, exponent `0.5`, and no `math.*` call
/// anywhere. So the only spelling decision is [`powp`], per `tests/porting_rules.rs`.
///
/// What DOES need care is the sort. The complex branch returns `-0.5*p` **twice**, so the ties are
/// real and Python's `sorted` is stable — [`sort_by`](slice::sort_by), never `sort_unstable_by`.
pub fn cubic_roots(c2: f64, c1: f64, c0: f64) -> [f64; 3] {
    let f = |x: f64| ((x - c2) * x + c1) * x - c0;
    let fp = |x: f64| (3.0 * x - 2.0 * c2) * x + c1;
    let mut x = if c2 != 0.0 { c2 } else { 1.0 };
    for _ in 0..80 {
        let d = fp(x);
        if d == 0.0 {
            break;
        }
        let step = f(x) / d;
        x -= step;
        if step.abs() <= 1e-14 * 1.0f64.max(x.abs()) {
            break;
        }
    }
    // deflate: `l^3 - c2 l^2 + c1 l - c0 = (l - x)(l^2 + p l + q)`
    let (p, q) = (x - c2, c1 - (c2 - x) * x);
    let disc = p * p - 4.0 * q;
    let mut out = if disc >= 0.0 {
        let rt = powp(disc, 0.5);
        [x, 0.5 * (-p + rt), 0.5 * (-p - rt)]
    } else {
        // a complex pair: report Re twice.
        [x, -0.5 * p, -0.5 * p]
    };
    out.sort_by(|a, b| a.partial_cmp(b).expect("the roots of a real cubic are finite here"));
    out
}

/// Python's `round(x, 12)` — correctly rounded to twelve DECIMAL digits, ties to even.
///
/// Format-and-parse, not `(x*1e12).round()/1e12`: the latter is a DIFFERENT function (it rounds
/// the SCALED value, and the scaling is itself inexact). [`round6`](crate::two_spool::round6) and
/// [`round3`](crate::two_spool_transient::round3) are the same decision at other widths, each with
/// its divergence class closed by construction rather than by a passing sweep.
///
/// It exists for exactly one reader: `ic_family`'s `order_members` counts DISTINCT members of the
/// `s = 0` family, and its set key is a rounded triple. A wrong rounding there does not move a
/// float — it moves an INTEGER, from "the six orders land on one member" to "they land on six".
pub fn round12(x: f64) -> f64 {
    format!("{x:.12}").parse::<f64>().expect("a formatted finite double parses back")
}

/// One clock triple's arm of [`triple_modes`].
#[derive(Clone, Debug)]
pub struct ModesArm {
    /// `(tau_att, tau_v, tau_s)` — the `(g, q, v)` order of the state vector.
    pub taus: (f64, f64, f64),
    /// `-sum_i 1/tau_i`, which § 2 predicts IS the one non-zero eigenvalue.
    pub rate_sum: f64,
    pub n: usize,
    pub n_sampled: usize,
    /// DISCLOSED, never a silent truncation — a dropped point is a coverage claim.
    pub skipped: usize,
    pub rows: Vec<ModesRow>,
    pub dom_range: Option<(f64, f64)>,
    pub worst_zero: Option<f64>,
}

/// One sampled point's spectrum.
#[derive(Clone, Debug)]
pub struct ModesRow {
    pub s: f64,
    /// `tr J` — **NOT reported as a measurement.** The diagonal `-1/tau_i` is the ODE's own
    /// structure, so `tr J == -sum 1/tau_i` is a tautology of the instrument. It is the ROOTS that
    /// carry the claim.
    pub c2: f64,
    /// `sum_{i<j} (1 - a_ij a_ji)/(tau_i tau_j)` — zero iff every PAIRWISE product is 1, which is
    /// rung 66's result three times.
    pub c1: f64,
    /// `det J = (x+1)^2 / (x tau_g tau_v tau_s)` — zero iff the CYCLIC product is −1, **the
    /// genuinely NEW claim**.
    pub c0: f64,
    pub roots: [f64; 3],
    pub cyclic: f64,
    /// The two roots closest to zero, by magnitude.
    pub zeros: [f64; 2],
    /// The root furthest from zero.
    pub dom: f64,
}

/// RUNG 68's `triple_modes` — **§ 2's SPECTRUM, measured on the shipped closures across a clock
/// grid.**
///
/// Two zero eigenvalues is exactly `c0 == c1 == 0`, so the `n-1` rank deficiency DECOMPOSES into
/// the three pairwise identities plus the one cyclic identity.
#[allow(clippy::too_many_arguments)]
pub fn triple_modes(
    core: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, sm: f64,
    clocks: &[(f64, f64, f64)], v_max: f64, tau_rel_mult: f64, every: usize,
) -> Vec<ModesArm> {
    let mut out = Vec::new();
    for &(tau_v, tau_att, tau_s) in clocks {
        let arm = TripleRigArm { sm, tau: tau_v, tau_s, v_max, tau_att,
                                 tau_rel: tau_rel_mult * tau_att, ..TripleRigArm::default() };
        let (m, surge, lag) = core.triple_rig(&arm);
        let leg = StatorLeg { accel: None, surge, tt4_max: None };
        let (traj, _) = m.stator_march_scoped(
            flight, ramp, None, &leg, &MarchScope { lag, ..MarchScope::DEFAULT });
        let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
        let pts = riding(&traj, b_max);
        let taus = (tau_att, tau_v, tau_s);
        // Python's `sum(1.0 / t for t in taus)` — a three-term LEFT FOLD in the state vector's
        // own order. Probe 5 measured this site identical on both interpreters.
        let rate = 1.0 / taus.0 + 1.0 / taus.1 + 1.0 / taus.2;
        let (mut rows, mut skipped) = (Vec::new(), 0usize);
        let sampled: Vec<&FuelPoint> = pts.iter().step_by(every.max(1)).collect();
        for p in &sampled {
            let gg = triple_gains_at(&m, flight, p, None, leg.surge.as_ref(),
                                     1e-7, 1e-5, 1e-4, true, 0.0, true)
                .expect("rung-68's spectrum march does not abort");
            if !gg.interior {
                skipped += 1;
                continue;
            }
            let a3 = [[-1.0, gg.r_q, gg.r_v], [gg.c_g, -1.0, gg.c_v], [gg.v_g, gg.v_q, -1.0]];
            let td = [taus.0, taus.1, taus.2];
            let mut j = [[0.0f64; 3]; 3];
            for (i, row) in j.iter_mut().enumerate() {
                for (k, cell) in row.iter_mut().enumerate() {
                    *cell = a3[i][k] / td[i];
                }
            }
            let c2 = j[0][0] + j[1][1] + j[2][2];
            let c1 = (j[0][0] * j[1][1] - j[0][1] * j[1][0])
                + (j[0][0] * j[2][2] - j[0][2] * j[2][0])
                + (j[1][1] * j[2][2] - j[1][2] * j[2][1]);
            let c0 = j[0][0] * (j[1][1] * j[2][2] - j[1][2] * j[2][1])
                - j[0][1] * (j[1][0] * j[2][2] - j[1][2] * j[2][0])
                + j[0][2] * (j[1][0] * j[2][1] - j[1][1] * j[2][0]);
            let roots = cubic_roots(c2, c1, c0);
            // `sorted(roots, key=abs)` — Python's sort is STABLE, so equal magnitudes keep their
            // input order. `_cubic_roots`'s complex branch returns `-0.5*p` twice, so the tie is
            // REAL and `sort_unstable_by` would be a different function.
            let mut by_abs = roots;
            by_abs.sort_by(|a, b| a.abs().partial_cmp(&b.abs()).expect("finite roots"));
            rows.push(ModesRow { s: p.s, c2, c1, c0, roots, cyclic: gg.cyclic,
                                 zeros: [by_abs[0], by_abs[1]], dom: by_abs[2] });
        }
        let dom_range = if rows.is_empty() {
            None
        } else {
            Some((rows.iter().map(|x| x.dom).fold(f64::INFINITY, f64::min),
                  rows.iter().map(|x| x.dom).fold(f64::NEG_INFINITY, f64::max)))
        };
        let worst_zero = rows.iter()
            .flat_map(|x| x.zeros.iter().map(|z| z.abs()))
            .fold(None::<f64>, |acc, z| Some(match acc { Some(a) => a.max(z), None => z }));
        out.push(ModesArm { taus, rate_sum: -rate, n: pts.len(), n_sampled: sampled.len(),
                            skipped, rows, dom_range, worst_zero });
    }
    out
}

/// One sampled point of [`triple_gains`] — the SAME gains read two ways.
#[derive(Clone, Debug)]
pub struct GainsRow {
    pub s: f64,
    /// Taken ON the shared manifold — the exact statement of § 2.
    pub on: TripleGains,
    /// Taken at the MARCHED `v`, which is off-manifold during a transient and is rung 66's own
    /// choice.
    pub live: TripleGains,
}

/// RUNG 68's `triple_gains` return.
#[derive(Clone, Debug)]
pub struct TripleGainsReport {
    pub n_riding: usize,
    pub n_sampled: usize,
    pub rows: Vec<GainsRow>,
    /// DISCLOSED: a dropped point is a coverage claim.
    pub skipped: Vec<(f64, Vec<&'static str>, Vec<&'static str>)>,
    pub s_window: Option<(f64, f64)>,
    pub cyclic_on: Vec<f64>,
    pub cyclic_live: Vec<f64>,
    pub worst_on: Option<f64>,
    pub worst_live: Option<f64>,
}

/// RUNG 68's `triple_gains` — **§ 2 MEASURED**: the six cross-gains, the three PAIRWISE products
/// (rung 66's identity, three times) and the CYCLIC product, **the only one that tests JOINT
/// collapse.**
///
/// Both readings are returned, because they answer different questions: `on` is taken on the
/// shared manifold, which is the exact statement of § 2; `live` is taken at the marched `v`, which
/// is off-manifold during a transient and is rung 66's own choice.
#[allow(clippy::too_many_arguments)]
pub fn triple_gains(
    core: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, sm: f64,
    arm: &TripleRigArm, every: usize,
) -> TripleGainsReport {
    let (m, surge, lag) = core.triple_rig(&TripleRigArm { sm, ..*arm });
    let leg = StatorLeg { accel: None, surge, tt4_max: None };
    let (traj, _) = m.stator_march_scoped(
        flight, ramp, None, &leg, &MarchScope { lag, ..MarchScope::DEFAULT });
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding(&traj, b_max);
    let sampled: Vec<&FuelPoint> = pts.iter().step_by(every.max(1)).collect();
    let (mut rows, mut skipped) = (Vec::new(), Vec::new());
    for p in &sampled {
        let on = triple_gains_at(&m, flight, p, None, leg.surge.as_ref(),
                                 1e-7, 1e-5, 1e-4, true, 0.0, true)
            .expect("rung-68's gains march does not abort");
        let live = triple_gains_at(&m, flight, p, None, leg.surge.as_ref(),
                                   1e-7, 1e-5, 1e-4, false, 0.0, true)
            .expect("rung-68's gains march does not abort");
        if !(on.interior && live.interior) {
            skipped.push((p.s,
                          if on.interior { Vec::new() } else { on.off_regime.clone() },
                          if live.interior { Vec::new() } else { live.off_regime.clone() }));
            continue;
        }
        rows.push(GainsRow { s: p.s, on, live });
    }
    let worst = |f: fn(&GainsRow) -> f64| -> Option<f64> {
        rows.iter().map(|x| (f(x) + 1.0).abs())
            .fold(None::<f64>, |acc, z| Some(match acc { Some(a) => a.max(z), None => z }))
    };
    TripleGainsReport {
        n_riding: pts.len(),
        n_sampled: sampled.len(),
        s_window: if pts.is_empty() { None }
                  else { Some((pts[0].s, pts[pts.len() - 1].s)) },
        cyclic_on: rows.iter().map(|x| x.on.cyclic).collect(),
        cyclic_live: rows.iter().map(|x| x.live.cyclic).collect(),
        worst_on: worst(|x| x.on.cyclic),
        worst_live: worst(|x| x.live.cyclic),
        rows,
        skipped,
    }
}

/// One displacement of [`cyclic_sensitivity`].
#[derive(Clone, Debug)]
pub struct SensitivityRow {
    pub delta: f64,
    /// `cyclic + 1` — `None` when the displacement drove a loop onto a stop, which is the confound
    /// this rung exists to name and is RECORDED as the regime event it is, never differenced.
    pub dep: Option<f64>,
    pub cyclic: Option<f64>,
    pub off_regime: Vec<&'static str>,
    pub pairs: Option<(f64, f64, f64)>,
}

/// RUNG 68's `cyclic_sensitivity` — **THE DETECTOR'S SENSITIVITY, MEASURED, never asserted.**
///
/// The golden-gate lesson: *a null result is worth what its instrument can resolve, and no more.*
/// The stator is displaced off the shared manifold by `delta` and the departure `cyclic + 1` read
/// back. A useful instrument returns a departure LINEAR in `delta` against the noise floor at
/// `delta = 0`; the gain is what converts *"the cyclic product is −1"* into *"the three laws share
/// a manifold to within `x` in `v`"*.
#[allow(clippy::too_many_arguments)]
pub fn cyclic_sensitivity(
    core: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, sm: f64,
    arm: &TripleRigArm, deltas: &[f64],
) -> Sensitivity {
    let (m, surge, lag) = core.triple_rig(&TripleRigArm { sm, ..*arm });
    let leg = StatorLeg { accel: None, surge, tt4_max: None };
    let (traj, _) = m.stator_march_scoped(
        flight, ramp, None, &leg, &MarchScope { lag, ..MarchScope::DEFAULT });
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding(&traj, b_max);
    assert!(!pts.is_empty(), "rung-68 cyclic_sensitivity needs a riding-interior point");
    let p = pts[pts.len() / 2];
    let mut rows = Vec::new();
    for &d in deltas {
        let gg = triple_gains_at(&m, flight, &p, None, leg.surge.as_ref(),
                                 1e-7, 1e-5, 1e-4, true, d, true)
            .expect("rung-68's sensitivity march does not abort");
        if !gg.interior {
            rows.push(SensitivityRow { delta: d, dep: None, cyclic: None,
                                       off_regime: gg.off_regime, pairs: None });
            continue;
        }
        rows.push(SensitivityRow {
            delta: d,
            dep: Some(gg.cyclic + 1.0),
            cyclic: Some(gg.cyclic),
            off_regime: Vec::new(),
            pairs: Some((gg.pair_rc, gg.pair_rv, gg.pair_cv)),
        });
    }
    assert!(rows[0].dep.is_some(), "rung-68: the delta=0 base point must be interior");
    let floor = rows[0].dep.expect("asserted just above").abs();
    let gains: Vec<f64> = rows[1..].iter()
        .filter(|x| x.delta > 0.0 && x.dep.is_some())
        .map(|x| x.dep.expect("filtered").abs() / x.delta)
        .collect();
    let gain = if gains.is_empty() {
        None
    } else {
        // Python's `sum(gains) / len(gains)` — a four-term LEFT FOLD on this grid; probe 5
        // measured it identical on both interpreters.
        Some(gains.iter().fold(0.0f64, |a, b| a + b) / gains.len() as f64)
    };
    let resolves = gain.map(|g| floor / g);
    Sensitivity { s: p.s, floor, rows, gain, resolves }
}

/// RUNG 68's `cyclic_sensitivity` return.
#[derive(Clone, Debug)]
pub struct Sensitivity {
    pub s: f64,
    /// `|cyclic + 1|` at `delta = 0` — **the NOISE FLOOR, which is what the null result is worth.**
    pub floor: f64,
    pub rows: Vec<SensitivityRow>,
    /// The mean departure per unit displacement — what converts *"the cyclic product is -1"* into
    /// *"the three laws share a manifold to within `x` in `v`"*.
    pub gain: Option<f64>,
    /// `floor / gain` — the displacement this instrument can actually resolve.
    pub resolves: Option<f64>,
}

// ---------------------------------------------------------------------------------------------
// § 4 — WHAT THE TRIPLE DELIVERS: the 7-cell ledger (8 with the bare march)
// ---------------------------------------------------------------------------------------------

/// RUNG 68's `_violation_inc` — **the SAME area, in the INCIDENCE currency.**
///
/// `int max(0, m_lim - M_i) ds`, with `M_i = T_c - (1/phi - v)` read at the LIVE stator setting.
/// Rung 66's [`violation`](crate::two_lag::violation) is inherited UNCHANGED for the `phi`
/// currency — same trapezoid rule, same signature — so the two rungs' ledgers are DIFFERENCEABLE
/// rather than merely similar. This is its mirror against the wall the stator does NOT move, and
/// **the two disagree in SIGN on the stator's own credit.**
pub fn violation_inc(traj: &[FuelPoint], m_lim: f64, t_c: f64, s_hi: f64) -> f64 {
    let mi = |p: &FuelPoint| -> f64 {
        // Python's `p.get('v', 0.0)` — a trajectory with no stator state reads ZERO, which is the
        // design setting and makes the rung-66 ledger cells comparable rather than refused.
        // SLICE AD (9 of 13). This one is a SINGLE-LINE match, and probe O called it
        // `exhaustive` twice before its regex was repaired -- see § (b).
        let v = match p.extra {
            PointExtra::Triple { v, .. } | PointExtra::Shared { v, .. } => v,
            _ => 0.0,
        };
        t_c - (1.0 / p.phi_lp - v)
    };
    let mut out = 0.0f64;
    for i in 1..traj.len() {
        if traj[i].s > s_hi {
            break;
        }
        let h = traj[i].s - traj[i - 1].s;
        out += 0.5 * h * (0.0f64.max(m_lim - mi(&traj[i - 1])) + 0.0f64.max(m_lim - mi(&traj[i])));
    }
    out
}

/// One cell of the 8-cell ledger.
#[derive(Clone, Copy, Debug)]
pub struct BillCell {
    /// The violation integral in the `phi` currency — rung 66's, inherited unchanged.
    pub i: f64,
    /// The same area against the INCIDENCE wall, which the stator cannot move.
    pub i_inc: f64,
    pub npts: usize,
    pub min_phi: f64,
    pub end_s: f64,
    pub v_min: f64,
    /// **BOTH ENDS ARE RECORDED, and neither is "the amount used" on its own.** Rung 69's band is
    /// one-sided the OTHER way, so `v_min` alone reads `0.0` for an incidence-referenced loop that
    /// rode the whole ramp.
    pub v_max_used: f64,
    pub v_saturated: bool,
    pub b_max_used: f64,
    pub credit: f64,
    pub credit_inc: f64,
}

/// RUNG 68's `triple_bill` return — the FULL 8-cell ledger and its three marginal credits.
#[derive(Clone, Debug)]
pub struct TripleBill {
    pub phi_lim: f64,
    pub m_lim: f64,
    /// **INSERTION-ORDERED, and that is semantic**: `bare, F, V, S, FV, FS, VS, FVS`. Python's
    /// dict preserves it and `sum(singles.values())` folds in it, so a `HashMap` would be a
    /// different function.
    pub cells: Vec<(&'static str, BillCell)>,
    /// `(fuel, valve, stator)`.
    pub marginal: (f64, f64, f64),
    pub marginal_incidence: (f64, f64, f64),
    pub singles: (f64, f64, f64),
    pub sum_singles: f64,
    pub delivered: f64,
    pub erosion: (f64, f64, f64),
}

impl TripleBill {
    pub fn cell(&self, name: &str) -> &BillCell {
        &self.cells.iter().find(|(n, _)| *n == name).expect("an eight-cell ledger").1
    }
}

/// Python's `min(gen, default=0.0)` / `max(gen, default=0.0)` — reduce over the ELEMENTS, and
/// fall back to `0.0` **only when there are none**.
///
/// Spelled as a function because the seeded-fold spelling is both shorter and wrong, and the
/// difference is invisible whenever the extremum happens to sit on the seed's side of zero.
fn fold_or(mut it: impl Iterator<Item = f64>, f: fn(f64, f64) -> f64) -> f64 {
    match it.next() {
        None => 0.0,
        Some(first) => it.fold(first, f),
    }
}

/// RUNG 68's `triple_bill` — **THE FULL 7-CELL LEDGER (8 with the bare march)**: every subset of
/// the three loops, every loop LAGGED.
///
/// **ALL THREE MARGINAL CREDITS ARE QUOTED, and that is pre-registered rather than chosen after
/// the fact.** Rung 66 measured the `n = 2` marginals at 1.59 % (fuel onto valve) and 33.64 %
/// (valve onto fuel) — BOTH doubling the rate sum, yet differing by 21× — so credit is not a
/// function of `sum 1/tau` and *"the third loop buys least"* has no mechanism behind it. With
/// three loops there are six orders, and quoting one would be cherry-picking.
///
/// **THE WALL IS NAMED ON EVERY NUMBER.** The primary currency is referenced to the `phi` floor.
/// The stator MOVES that floor (rung 53) while leaving the metal one alone, and measurably
/// `dM_phi/dv = -0.115` against `dM_i/dv = +0.344` — **OPPOSITE SIGNS**. So the incidence-
/// referenced integral is reported beside it: a credit quoted without its wall is meaningless
/// here, because a margin is a DISTANCE.
pub fn triple_bill(
    core: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, sm: f64,
    arm: &TripleRigArm,
) -> TripleBill {
    const CELLS: [(&str, bool, bool, bool); 8] = [
        ("bare", false, false, false),
        ("F", true, false, false),
        ("V", false, true, false),
        ("S", false, false, true),
        ("FV", true, true, false),
        ("FS", true, false, true),
        ("VS", false, true, true),
        ("FVS", true, true, true),
    ];
    let cmap = core.arming().map_lp_design;
    let phi_lim = (1.0 + sm) * cmap.phi_surge;
    let t_c = cmap.tan_beta1_crit();
    // The SAME floor, read as an incidence.
    let m_lim = t_c - 1.0 / phi_lim;
    let mut cells: Vec<(&'static str, BillCell)> = Vec::new();
    for (name, fu, va, st) in CELLS {
        let (m, surge, lag) = core.triple_rig(&TripleRigArm {
            sm, fuel: fu, valve: va, stator: st, ..*arm });
        let leg = StatorLeg { accel: None, surge, tt4_max: None };
        let (traj, _) = m.stator_march_scoped(
            flight, ramp, None, &leg, &MarchScope { lag, ..MarchScope::DEFAULT });
        let v_of_point = |p: &FuelPoint| match p.extra {
            // SLICE AD (10 of 13).
            PointExtra::Triple { v, .. } | PointExtra::Shared { v, .. } => v,
            _ => 0.0,
        };
        let b_of_point = |p: &FuelPoint| match p.extra {
            // SLICE AD (11 of 13).
            PointExtra::Triple { b, .. } | PointExtra::Cascade { b, .. }
            | PointExtra::CrossCascade { b, .. } | PointExtra::Valve { b, .. }
            | PointExtra::Shared { b, .. } => b,
            _ => 0.0,
        };
        cells.push((name, BillCell {
            i: crate::two_lag::violation(&traj, phi_lim, ramp.r),
            i_inc: violation_inc(&traj, m_lim, t_c, ramp.r),
            npts: traj.len(),
            min_phi: traj.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min),
            end_s: traj[traj.len() - 1].s,
            // **`min(gen, default=0.0)` IS NOT `fold(0.0, min)`, AND THE ORACLE CAUGHT THE
            // DIFFERENCE ON ITS FIRST RUN.** Python's `default` is used only when the generator
            // is EMPTY; a seeded fold clamps the answer at the seed. On the `S` cell every `v` is
            // negative, so `v_max_used` came back `0.0` where Python reads `-1.08e-11` -- the
            // largest of the negatives. All 22 ported gates passed with the wrong value, because
            // the only assertion that reads `v_max_used` is RUNG 69's, one slice ahead: Python's
            // own comment beside this key says the band is one-sided the OTHER way there and
            // `v_min` alone would read 0.0. Both ends therefore fold over the ELEMENTS.
            v_min: fold_or(traj.iter().map(v_of_point), f64::min),
            v_max_used: fold_or(traj.iter().map(v_of_point), f64::max),
            v_saturated: traj.iter().any(|p| matches!(
                p.extra, PointExtra::Triple { v_regime: Regime::Saturated, .. })),
            b_max_used: fold_or(traj.iter().map(b_of_point), f64::max),
            credit: f64::NAN,
            credit_inc: f64::NAN,
        }));
    }
    let base = cells[0].1.i;
    let base_i = cells[0].1.i_inc;
    for (_, c) in cells.iter_mut() {
        c.credit = if base != 0.0 { 100.0 * (1.0 - c.i / base) } else { f64::NAN };
        c.credit_inc = if base_i != 0.0 { 100.0 * (1.0 - c.i_inc / base_i) } else { f64::NAN };
    }
    let get = |cs: &Vec<(&'static str, BillCell)>, n: &str| -> BillCell {
        cs.iter().find(|(k, _)| *k == n).expect("an eight-cell ledger").1
    };
    let (fvs, vs, fs, fv) = (get(&cells, "FVS"), get(&cells, "VS"),
                             get(&cells, "FS"), get(&cells, "FV"));
    let singles = (get(&cells, "F").credit, get(&cells, "V").credit, get(&cells, "S").credit);
    let marg = (fvs.credit - vs.credit, fvs.credit - fs.credit, fvs.credit - fv.credit);
    let marg_inc = (fvs.credit_inc - vs.credit_inc, fvs.credit_inc - fs.credit_inc,
                    fvs.credit_inc - fv.credit_inc);
    let inf_or = |s: f64, m: f64| if m != 0.0 { s / m } else { f64::INFINITY };
    TripleBill {
        phi_lim,
        m_lim,
        // Python's `sum(singles.values())` — a THREE-term left fold in `fuel, valve, stator`
        // insertion order. Probe 5 measured it identical on both interpreters; the order is kept
        // because it is the function, not because the difference is visible here.
        sum_singles: singles.0 + singles.1 + singles.2,
        delivered: fvs.credit,
        erosion: (inf_or(singles.0, marg.0), inf_or(singles.1, marg.1),
                  inf_or(singles.2, marg.2)),
        marginal: marg,
        marginal_incidence: marg_inc,
        singles,
        cells,
    }
}

/// One row of [`saturation_counterfeit`] — a saturated point and a riding one, read by the SAME
/// unfiltered instrument.
#[derive(Clone, Debug)]
pub struct CounterfeitRow {
    pub s: f64,
    pub regime: Regime,
    pub off_regime: Vec<&'static str>,
    pub v_g: f64,
    pub v_q: f64,
    pub pair_rc: f64,
    pub pair_rv: f64,
    pub pair_cv: f64,
    pub c1: f64,
    pub c0: f64,
    pub roots: [f64; 3],
    /// How many roots sit within `1e-3 * |c2|` of zero — Python's `sum(1 for …)`, an INTEGER
    /// count and therefore outside the `sum()` exemption entirely.
    pub n_zero: usize,
}

/// RUNG 68's `saturation_counterfeit` — **THE INSTRUMENT'S OWN FAILURE MODE, MEASURED RATHER THAN
/// ASSERTED.**
///
/// A loop on its stop has `dU/du_j == 0` for every `j`: it contributes a row of zeros to the
/// coupling, so **SATURATION COSTS THE BLOCK A ZERO** — the saturated state keeps only its own bare
/// `-1/tau`, and at most one zero can survive, from the remaining pair.
///
/// **WHAT THE OBSERVABLE IS DEPENDS ON WHERE THE POINT SITS**, and only one of the two is reachable
/// on a real march. EXACTLY on the shared manifold the surviving pair is exact (`a c = 1`), so the
/// triple reads as a degenerate PAIR — one zero instead of two. OFF the manifold, which is where a
/// transient always is, that pair's identity has degraded too, so the block reads as a FULLY
/// INDEPENDENT triple — **zero zeros**.
///
/// **SO THE PRACTICAL COUNTERFEIT IS INDEPENDENCE.** This is the INVERSE of rung 67's lesson (*a
/// zero cross-gain is saturation, never decoupling*): there a stop faked COUPLING's absence in one
/// entry; here a stop fakes the absence of REDUNDANCY in the whole block. The gains are measured
/// with the interior filter OFF **on purpose** — the subject is what the unfiltered instrument
/// reports.
pub fn saturation_counterfeit(
    core: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, sm: f64,
    arm: &TripleRigArm, v_max_sat: f64,
) -> Counterfeit {
    let (m, surge, lag) = core.triple_rig(&TripleRigArm { sm, v_max: v_max_sat, ..*arm });
    let leg = StatorLeg { accel: None, surge, tt4_max: None };
    let (traj, _) = m.stator_march_scoped(
        flight, ramp, None, &leg, &MarchScope { lag, ..MarchScope::DEFAULT });
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let sat: Vec<FuelPoint> = traj.iter().filter(|p| match p.extra {
        // SLICE AD (12 of 13): `riding`'s sibling, and the same silent-empty-set failure.
        PointExtra::Triple { required, b_cmd, v_regime, .. }
        | PointExtra::Shared { required, b_cmd, v_regime: Some(v_regime), .. } =>
            required > 0.0 && 0.0 < b_cmd && b_cmd < b_max && v_regime == Regime::Saturated,
        // As in `riding`: Python compares `None` against a string and gets `False`.
        PointExtra::Shared { v_regime: None, .. } => false,
        _ => false,
    }).copied().collect();
    let rid = riding(&traj, b_max);
    let taus = [arm.tau_att, arm.tau, arm.tau_s];
    let mut rows = Vec::new();
    let mid = |v: &Vec<FuelPoint>| -> Option<FuelPoint> {
        if v.is_empty() { None } else { Some(v[v.len() / 2]) }
    };
    for p in [mid(&sat), mid(&rid)].into_iter().flatten() {
        let gg = triple_gains_at(&m, flight, &p, None, leg.surge.as_ref(),
                                 1e-7, 1e-5, 1e-4, false, 0.0, false)
            .expect("rung-68's counterfeit march does not abort");
        let a3 = [[-1.0, gg.r_q, gg.r_v], [gg.c_g, -1.0, gg.c_v], [gg.v_g, gg.v_q, -1.0]];
        let mut j = [[0.0f64; 3]; 3];
        for (i, row) in j.iter_mut().enumerate() {
            for (k, cell) in row.iter_mut().enumerate() {
                *cell = a3[i][k] / taus[i];
            }
        }
        let c2 = j[0][0] + j[1][1] + j[2][2];
        let c1 = (j[0][0] * j[1][1] - j[0][1] * j[1][0])
            + (j[0][0] * j[2][2] - j[0][2] * j[2][0])
            + (j[1][1] * j[2][2] - j[1][2] * j[2][1]);
        let c0 = j[0][0] * (j[1][1] * j[2][2] - j[1][2] * j[2][1])
            - j[0][1] * (j[1][0] * j[2][2] - j[1][2] * j[2][0])
            + j[0][2] * (j[1][0] * j[2][1] - j[1][1] * j[2][0]);
        let roots = cubic_roots(c2, c1, c0);
        rows.push(CounterfeitRow {
            s: p.s,
            regime: match p.extra {
                // SLICE AD (13 of 13): the filter above now admits rung 72, so this
                // `unreachable!` would BECOME reachable without the matching arm --
                // the same pairing as sites 3 and 4.
                PointExtra::Triple { v_regime, .. }
                | PointExtra::Shared { v_regime: Some(v_regime), .. } => v_regime,
                _ => unreachable!(
                    "filtered to Triple or stator-carrying Shared points above"),
            },
            off_regime: gg.off_regime.clone(),
            v_g: gg.v_g, v_q: gg.v_q,
            pair_rc: gg.pair_rc, pair_rv: gg.pair_rv, pair_cv: gg.pair_cv,
            c1, c0, roots,
            n_zero: roots.iter().filter(|x| x.abs() < 1e-3 * c2.abs()).count(),
        });
    }
    Counterfeit { v_max: v_max_sat, n_saturated: sat.len(), n_riding: rid.len(), rows }
}

/// RUNG 68's `saturation_counterfeit` return.
#[derive(Clone, Debug)]
pub struct Counterfeit {
    /// The deliberately UNDER-set authority that drives the stator onto its stop.
    pub v_max: f64,
    pub n_saturated: usize,
    pub n_riding: usize,
    /// At most two: the middle SATURATED point and the middle RIDING one, read by the same
    /// unfiltered instrument.
    pub rows: Vec<CounterfeitRow>,
}

// ---------------------------------------------------------------------------------------------
// § 3 — THE INITIAL-CONDITION FAMILY
// ---------------------------------------------------------------------------------------------

/// One run of [`ic_family`] — the `s = 0` state the sweep landed on, and what the march made of it.
#[derive(Clone, Copy, Debug)]
pub struct IcRun {
    pub g0: f64,
    pub b0: f64,
    pub v0: f64,
    pub iters: usize,
    pub res: f64,
    pub i: f64,
    pub min_phi: f64,
    /// `int g ds` over the ramp — **the ONE key in this slice that the CPython arm cannot carry.**
    ///
    /// It is a `sum()` over 101 trajectory terms, and CPython 3.12+'s `sum` is Neumaier-compensated
    /// where PyPy's — and a Rust fold — are naive. Probe 5 measured **2 of 10** instances
    /// differing. Every other float `sum()` in rung 68 adds three or four numbers and agrees on
    /// both interpreters; the exemption is this reader, named rather than a tolerance tier.
    pub withheld: f64,
}

/// RUNG 68's `ic_family` — **§ 3: the `s = 0` fixed points are a CURVE, so the sweep lands on a
/// MEMBER.**
///
/// TWO instruments, because they answer different questions. `orders` varies the Gauss-Seidel
/// sweep order from the DECLARED starting member (rung 66's: `g = 0`, `q = b_cmd(0)`, `v = 0`);
/// `starts` varies the starting valve position itself, which is rung 65's own `b0` instrument
/// re-run at `n = 3`. **If the declared start is already a fixed point, every order lands on it in
/// one iteration and the family shows up only in the second sweep** — which is rung 66 § 0's own
/// diagnosis: the degeneracy at `s = 0` is NON-UNIQUENESS of the initial condition, not a stalled
/// solve.
#[allow(clippy::too_many_arguments)]
pub fn ic_family(
    core: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, sm: f64,
    arm: &TripleRigArm, orders: &[&'static str], starts: &[Option<f64>],
) -> IcFamily {
    let (m, surge, lag) = core.triple_rig(&TripleRigArm { sm, ..*arm });
    let cmap = core.arming().map_lp_design;
    let phi_lim = (1.0 + sm) * cmap.phi_surge;
    let leg = StatorLeg { accel: None, surge, tt4_max: None };
    let run = |scope: MarchScope| -> IcRun {
        let (t, _) = m.stator_march_scoped(flight, ramp, None, &leg, &scope);
        let z = &t[0];
        let (g0, b0) = (asym_extra(z).0, crate::lagged_bleed::valve_of(z).0);
        let (iters, res, _) = ic_at_point(z);
        IcRun {
            g0,
            b0,
            v0: v_at_point(z),
            iters,
            res,
            i: crate::two_lag::violation(&t, phi_lim, ramp.r),
            min_phi: t.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min),
            // **THE CPYTHON EXEMPTION LIVES HERE.** Python's
            // `sum(p['g'] * ds for p in t if p['s'] <= r + 1e-12)` over ~101 terms — a NAIVE left
            // fold on PyPy and in Rust, Neumaier-compensated on CPython 3.12+.
            withheld: t.iter()
                .filter(|p| p.s <= ramp.r + 1e-12)
                .map(|p| asym_extra(p).0 * ramp.ds)
                .fold(0.0f64, |a, b| a + b),
        }
    };
    let by_order: Vec<(&'static str, IcRun)> = orders.iter()
        .map(|&o| (o, run(MarchScope { lag, ic_order: Some(o), ..MarchScope::DEFAULT })))
        .collect();
    let by_start: Vec<(Option<f64>, IcRun)> = starts.iter()
        .map(|&b| (b, run(MarchScope { lag, b0: b, ..MarchScope::DEFAULT })))
        .collect();
    // Python's `len({(round(g0,12), round(b0,12), round(v0,12)) …})` — a SET over ROUNDED triples,
    // so the count is an INTEGER a wrong rounding moves outright. See [`round12`].
    //
    // **`to_bits` IS NOT PYTHON'S SET KEY, AND SIGNED ZERO IS WHERE THEY PART.** A Python set
    // compares floats with `==`, under which `-0.0 == 0.0`, so a member reached at `-0.0` and one
    // reached at `+0.0` are ONE member there and would be TWO here. No trajectory on the shipped
    // grid produces a negative zero — `v0` starts at exactly `+0.0` — so the difference is
    // unreachable and therefore invisible to all 12 084 oracle keys; `key12` normalises it anyway,
    // because the alternative is a latent off-by-one in a COUNT that no value gate can witness.
    // (PyPy's own `round(-0.0, 12)` returns `+0.0`, measured; Rust's format-and-parse keeps the
    // sign. Either way this closes it.)
    let key12 = |x: f64| (round12(x) + 0.0).to_bits();
    let mut members: Vec<[u64; 3]> = by_order.iter()
        .map(|(_, x)| [key12(x.g0), key12(x.b0), key12(x.v0)])
        .collect();
    members.sort_unstable();
    members.dedup();
    let is: Vec<f64> = by_start.iter().map(|(_, x)| x.i).collect();
    let ws: Vec<f64> = by_start.iter().map(|(_, x)| x.withheld).collect();
    let spread = |v: &[f64]| -> Option<f64> {
        let lo = v.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        if lo != 0.0 { Some((hi - lo) / lo) } else { None }
    };
    IcFamily {
        by_order,
        by_start,
        order_members: members.len(),
        start_spread_i: spread(&is),
        start_spread_withheld: spread(&ws),
    }
}

/// RUNG 68's `ic_family` return — Python's dict, one struct, per the crate's convention.
#[derive(Clone, Debug)]
pub struct IcFamily {
    /// The Gauss-Seidel sweep ORDER varied from the declared starting member. **Insertion-ordered
    /// and that is semantic**: Python's dict keeps the caller's `orders` sequence.
    pub by_order: Vec<(&'static str, IcRun)>,
    /// The starting VALVE position varied — rung 65's own `b0` instrument re-run at `n = 3`.
    pub by_start: Vec<(Option<f64>, IcRun)>,
    /// How many DISTINCT members of the `s = 0` family the six orders land on. An INTEGER, and
    /// the one number in this rung a wrong rounding moves outright — see [`round12`].
    pub order_members: usize,
    pub start_spread_i: Option<f64>,
    /// The spread of [`IcRun::withheld`], **and therefore the one derived key that inherits the
    /// CPython exemption.**
    pub start_spread_withheld: Option<f64>,
}
