//! RUNG 69 — **THE REFERENCE SPLIT**: rung 68's stator, referenced to INCIDENCE.
//!
//! The SAME lever, the same plant, the same two other loops, the same clocks and the same set
//! point read at the design setting. **The only thing that moves is the COORDINATE the third loop
//! watches** — from rung 49/64's `phi_lim` to rung 60's incidence margin `M_i = T_c - (1/phi - v)`.
//!
//! Headline: *a loop's COORDINATE decides whether it adds a ZERO or a RANK* — `zeros = n - m`,
//! where `m` is the number of INDEPENDENT CONSTRAINTS and the loop count never enters. See
//! `docs/rung69-spec.md`.
//!
//! # What this module is
//!
//! [`StatorIncidenceLimiter`] and its four methods; `_ref`'s carrier and its RAII guard
//! [`RefScope`]; [`build_reference_split_cascade`] with rung 69's four `__init__` guards and every
//! inherited refusal re-listed in Python's order; the five `R69*` tables; **the one added cell
//! [`TripleHooks::with_ref`] and the nine swapped bodies**; and the six readers of §§ 1, 3 and 4,
//! with [`cubic_roots_c`] and [`invariants`] under them.
//!
//! **STEP 1 SHIPPED THE TABLE WITH NINE NAMED PANICS IN IT AND STEP 2 REPLACED THEM ONE AT A
//! TIME**, which is why `UNPORTED_AT_STEP1` and the gate that read those messages are gone from
//! `tests/slice_ab_cells.rs` — their whole content was *"not yet ported"*. Recorded here so a
//! reader does not restore a gate whose pass condition the port itself removed.
//!
//! **TWO TENS APPEAR IN THIS SLICE AND THEY ARE DIFFERENT TENS**, so the addition is written out
//! rather than left for a reader to reconcile: **10 SWAPS** = the 9 cells overridden below +
//! `__init__`, which is not a cell (no shipped table carries a constructor hook — it ports as
//! [`build_reference_split_cascade`]'s four `assert!`s); **10 TABLE CELLS** = those same 9 + the
//! one this rung ADDS, [`TripleHooks::with_ref`]. `tests/slice_ab_cells.rs` holds the arithmetic
//! and the compiler holds the width.
//!
//! # WHY THE FILE IS SHAPED THE WAY IT IS — THE SWAPS, NOT THE CELL COUNT (§ 5.26 (ii))
//!
//! Phase 7's rule is *step 1 of every slice is the cell addition*, so a slice that forgets a cell
//! fails at its own first gate rather than at a value key nine rungs downstream. **That rule buys
//! almost nothing at this slice.** Slice AA ADDED nine cells; this one adds **one** and SWAPS
//! **ten**, and a forgotten swap compiles, runs, and is caught by nothing the ladder does
//! automatically — the parent's body is a perfectly good function pointer.
//!
//! So the pre-flight asked step 5's question in advance (§ 5.26 (ii)): at every call site the
//! shipped suite reaches, would rung 68's body have returned something different? EMITTED, over
//! the whole rung-69 suite:
//!
//! | cell | calls | parent DISAGREES | how a gate can see it |
//! |---|---|---|---|
//! | `_stator_leg` | 2 297 263 | 1 276 121 | by value |
//! | `_clamp_v` | 40 354 | 25 364 | by value |
//! | `_lagged_stator` | 95 | 54 | by value |
//! | `_check_v0` | 4 | 2 | by value (the parent's assert fires where the child's passes) |
//! | **`_rk4_floor`** | 77 | **0** | **only through its PANIC STRING** |
//! | `_solve_v` | 162 869 | — | **by PANIC** (the parent reads `stator_lim`, `None` on 102 064) |
//! | `_manifold_v` | 291 | — | ~~**by PANIC**, on 122~~ **BY VALUE** — corrected § 5.26.5 (b) |
//! | `_triple_rig` | 60 | — | ~~**by PANIC**, on 60~~ **BY VALUE** — corrected § 5.26.5 (b) |
//! | `at_lever` | 61 | — | by value: 31 calls carry `stator_inc` IN and 0 lose it |
//!
//! **THE LAST TWO ROWS WERE WRONG AND STEP 5 MEASURED IT.** The prediction was that rung 68's
//! bodies dereference `self.stator_lim`, which an incidence arming leaves `None`. Neither does.
//! `r68_triple_rig` never READS that field — it BUILDS a `StatorLimiter` from the map, so the
//! parent hands back a well-formed sibling carrying the WRONG REFERENCE with nothing raising; and
//! `r68_manifold_v` is `V(g, q)[0]`, which reads no field at all and returns the stator's OWN root
//! where this rung returns the SHARED manifold (opposite SIGNS at the sampled point). Corrected
//! here rather than only in the plan, because this table is what a reader of the module sees, and
//! **the silent shape is the dangerous one**: a gate written to expect a panic would have left both
//! cells effectively ungated. `tests/slice_ab_dispatch.rs` carries the measurement.
//!
//! **`_rk4_floor` IS THE ONE SWAP NO VALUE KEY CAN SEE.** Its condition is `ds * rate <= 2.0` in
//! BOTH rungs, character for character; the entire difference is the assertion MESSAGE — rung 68
//! explains the bound by *"J has rank one"*, rung 69 by *"the block is rank TWO and the dominant
//! root is a COMPLEX pair"*. It is not unobservable: `test_rung69.py:530` does
//! `pytest.raises(AssertionError, match="rank TWO")`. It is gated by a `#[should_panic]`, never by
//! a value diff, and writing it as a value diff is how the cell ends up silently ungated.
//!
//! # THE BAND FLIP, and it is the rung's own declared silent failure
//!
//! `M_i` is INCREASING in `v` where rung 68's `phi_lp` was DECREASING, so `_solve_v`'s bracket
//! orientation and BOTH clamp tests flip BACK to `_solve_b`'s. Rung 69's own docstring says it
//! *"fails silently — a wrong orientation returns a wrong regime label with nothing raising"*, so
//! the evidence is EMITTED by an AST diff rather than read off the bodies:
//!
//! | cell | verdict | comparisons | unary minus |
//! |---|---|---|---|
//! | `_clamp_v` | RESTRUCTURED | `min(0, max(-v_max, v))` → `max(0, min(v_max, v))` | **1 → 0** |
//! | `_check_v0` | RESTRUCTURED | `-v_max <= v0 <= 0` → `0 <= v0 <= v_max` | **2 → 0** |
//! | `_solve_v` | RESTRUCTURED | bracket `[-v_max, 0]` → `[0, +v_max]` | **3 → 0** |
//! | `_rk4_floor` | SHAPE-EQUAL | identical condition; MESSAGE only | 0 → 0 |
//!
//! **The unary-minus column IS the band flip, and it reads to zero.** Six negations of `v_max`
//! across the three orientation-carrying cells, none surviving.
//!
//! # `_ref` — CONFIG-kind, and the cell is the SETTER rather than the call
//!
//! Python's `_with_ref(self, ref, fn, *a, **kw)` is higher-order over a return type that varies by
//! call site — a tuple from `_triple_rig`, a dict from `triple_bill`, and at rung 73 a tuple from
//! `_quad_gains_at`. A `fn` pointer in a `const` table cannot be generic over that, and `&dyn Fn`
//! does not rescue it, because it is the RETURN type that differs. Read against rung 73's override
//! (`_ref_law`), the only thing that override changes is **WHICH FIELD THE GUARD WRITES**. So the
//! cell is [`TripleHooks::with_ref`] — *set, and hand back the previous value* — the RAII guard
//! [`RefScope`] is shared, and each reader opens its own scope.
//!
//! **AND `_ref` NEVER NESTS — MEASURED, NOT INHERITED.** 58 sets, every one from `_with_ref`,
//! every one outside every march: `'inc'` 14 + `'phi'` 15 = 29 sets to a value against 29 restores
//! to `None`. A restore-to-`None` that is exact means the previous value was `None` every time.
//! The guard nevertheless restores the PREVIOUS value, because that is what Python's `finally`
//! does; the two spellings agree on every shipped path, which is why slice X's manufactured-nest
//! gate is the only instrument that can tell them apart.
//!
//! [`TripleHooks::with_ref`]: crate::three_loop::TripleHooks::with_ref

use std::cell::Cell;

use crate::bleed_transient::{LeverArm, LeverArming, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{
    AsymmetricLag, Floor, FuelCloseState, FuelPoint, FuelTransientHooks, PointExtra, SurgeLimiter,
};
use crate::gas::{powp, Abort};
use crate::lagged_bleed::py_max3;
use crate::limited_bleed::{BleedLimiter, Regime};
use crate::map::ComponentMap;
use crate::spool::{try_illinois, ILLINOIS_MAXIT};
use crate::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
    StatorTransientHooks,
};
use crate::three_loop::{
    riding, triple_bill, triple_gains_at, v_at_point, StatorLegArm, StatorLimiter, TripleBill,
    TripleGains, TripleHooks, TripleRigArm,
};
use crate::two_spool::{Spool, TwoSpoolEngine};
use crate::two_spool_transient::{MarchedBleed, TwoSpoolTransientCore, TwoSpoolTransientHooks};

// ---------------------------------------------------------------------------------------------
// THE DEVICE
// ---------------------------------------------------------------------------------------------

/// RUNG 69's control law: **the smallest `v` in `[0, +v_max]` that holds
/// `M_i = T_c - (1/phi - v) >= m_lim`.**
///
/// Rung 68's [`StatorLimiter`](crate::three_loop::StatorLimiter) with ONE thing changed — the wall
/// the loop watches. Every other thing is rung 68's: the same lever, the same plant, the same two
/// other loops, the same clocks.
///
/// **AND THE DIRECTION IS NOW THE PHYSICAL ONE.** Measured at the point rung 68 measured its own,
/// `dphi_lp/dv = -0.423` but `dM_i/dv = +0.335`: closing the stators LOWERS `phi` and RAISES
/// incidence margin, because closing lowers the WALL `phi_surge(v) = 1/(T_c+v)` faster than it
/// lowers `phi`. So THIS loop closes at low corrected flow, which is what a real VSV schedule
/// does and the exact opposite of rung 68's `phi`-referenced one. Rung 68 had to disclose an
/// ANTI-PHYSICAL lever; this rung does not.
///
/// The three regimes, and **two of them are inverted BACK** relative to `StatorLimiter`:
///
/// * `v = 0` — DORMANT: `M_i` already clears the floor; the DESIGN setting, and the closure
///   dispatches to the parent bit-for-bit.
/// * `0 < v < v_max` — RIDING: the only regime in which this loop is evidence of anything.
/// * `v = +v_max` — SATURATED: the floor is violated; the ceiling belongs to `v_max`.
///
/// `m_lim` is rung 60's currency and carries rung 36's disclaimed constant, not a new one: use
/// [`from_phi`](Self::from_phi) / [`from_margin`](Self::from_margin), which put the floor on the
/// SAME PHYSICAL WALL as rung 64's `phi_lim` at the design setting. That — not equality of two
/// floats in different units — is what "one set point" can mean across a change of coordinate.
///
/// `Copy` for [`LeverArm`]'s reason, which is what keeps *"the signature is never re-opened"* true
/// when this field is added to it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatorIncidenceLimiter {
    /// The floor, in rung 60's incidence-margin currency `M_i`.
    ///
    /// **Not called `phi_lim`, and the sibling method is not either** — see
    /// [`phi_lim_at`](Self::phi_lim_at).
    pub m_lim: f64,
    /// The AUTHORITY. The admissible band is `[0, +v_max]`, one-sided — the MIRROR of rung 68's.
    pub v_max: f64,
    /// The actuator's BANDWIDTH — hardware, like `v_max`. `None` is refused by the integrator,
    /// not silently dropped: rung 66's discipline, inherited from rung 68 verbatim.
    pub tau: Option<f64>,
}

impl StatorIncidenceLimiter {
    /// Python's `__post_init__` — **both asserts, in Python's order, and there are TWO where rung
    /// 68 has three.**
    ///
    /// `StatorLimiter` opens with `assert self.phi_lim > 0.0, "rung-68 phi floor is a flow
    /// coefficient"`. There is no counterpart here, and the reason is a MEASUREMENT rather than
    /// the *"a margin is signed"* argument that was written here first and is wrong on the shipped
    /// grid: `T_c = 1/phi_surge` exactly (rung 53, zero new constants), so
    /// `m_lim = T_c - 1/phi_lim = 1/phi_surge - 1/((1+sm) phi_surge)` is **non-negative for every
    /// floor this rung builds** — and **exactly zero at `sm = 0`**, the boundary
    /// [`from_margin`](Self::from_margin)'s own assert explicitly admits (*"sits AT or ABOVE the
    /// surge line"*). A copied-over `m_lim > 0` would refuse precisely that case. The absence is
    /// gated by that witness in `tests/slice_ab_cells.rs`, not by this paragraph.
    pub fn new(m_lim: f64, v_max: f64, tau: Option<f64>) -> Self {
        assert!(
            v_max > 0.0 && v_max < 1.0,
            "rung-69 needs stators with AUTHORITY: v_max = 0 is a limiter that cannot act, which \
             is a DIFFERENT object from an absent one (that is `stator_inc=None`); and |v| >= 1 \
             is far outside the setting range rungs 53-58 swept (V = 0.20). Got v_max = {v_max}"
        );
        assert!(
            tau.is_none_or(|t| t > 0.0),
            "rung-69 tau is a time constant on the march coordinate; an INSTANTANEOUS stator loop \
             is a different object and is not built (rung 66's discipline, inherited verbatim \
             from rung 68). Got {tau:?}"
        );
        StatorIncidenceLimiter { m_lim, v_max, tau }
    }

    /// **THE MATCHED FLOOR**: the incidence set point that a given `phi` floor IS at the DESIGN
    /// stator setting, `m_lim = T_c - 1/phi_lim` (rung 60's `from_phi` at `vsv = 0`).
    ///
    /// The two walls then coincide at `v = 0` and diverge only as the lever moves — which is
    /// exactly the experiment. Matching them any other way would confound the REFERENCE SPLIT
    /// with a set-point offset: rung 66 measured a −2.5 % offset moving its product to 0.951.
    pub fn from_phi(cmap: &ComponentMap, v_max: f64, phi_lim: f64, tau: Option<f64>) -> Self {
        Self::new(cmap.tan_beta1_crit() - 1.0 / phi_lim, v_max, tau)
    }

    /// The incidence floor matched to rung 64's / 68's `from_margin(cmap, ., sm)` — the SAME
    /// physical wall, read in the other coordinate.
    pub fn from_margin(cmap: &ComponentMap, v_max: f64, sm: f64, tau: Option<f64>) -> Self {
        assert!(
            cmap.phi_surge > 0.0,
            "rung-69 from_margin needs a surge line: build the map with .with_phi_surge(.)"
        );
        assert!(sm >= 0.0, "the rung-69 floor sits AT or ABOVE the surge line");
        Self::from_phi(cmap, v_max, (1.0 + sm) * cmap.phi_surge, tau)
    }

    /// The `phi` floor this incidence floor IS at the design setting — the inverse of
    /// [`from_phi`](Self::from_phi), and the number rung 69's readers use to locate the SHARED
    /// manifold.
    ///
    /// **NAMED `phi_lim_at` AND NOT `phi_lim` ON PURPOSE, and the reason does not survive the port
    /// — the NAME does.** Both sibling limiters carry a FLOAT field called `phi_lim`
    /// ([`BleedLimiter`](crate::limited_bleed::BleedLimiter),
    /// [`StatorLimiter`](crate::three_loop::StatorLimiter)), so in Python a method of that name
    /// here would make a duck-typed `lim.phi_lim == other.phi_lim` compare a BOUND METHOD against
    /// a float — unequal, and raising nothing. Rust refuses that comparison at compile time, so
    /// the hazard is gone; the name is kept because a reader moving between the two languages
    /// should not have to translate it.
    pub fn phi_lim_at(&self, cmap: &ComponentMap) -> f64 {
        1.0 / (cmap.tan_beta1_crit() - self.m_lim)
    }

    /// `M_i = T_c - tan_beta1 = T_c - (1/phi - v)`, read at the LIVE stator setting.
    ///
    /// Rung 53's `tan_beta1`, negated onto rung 60's currency; no new physics. `static` in Python,
    /// so no receiver here either.
    pub fn margin(t_c: f64, phi: f64, v: f64) -> f64 {
        t_c - (1.0 / phi - v)
    }
}

/// **THE NARROWED RETURN SLICE AA BUILT FOR THIS SLICE, USED.**
///
/// [`StatorLegArm`]'s own doc comment states the design in advance: `_stator_leg`'s callers touch
/// exactly `.tau` and `.v_max`, never the limit itself, so the cell's return is narrowed to those
/// two fields rather than being an enum over the two limiter types. This impl is the whole cost of
/// that decision being right — one `From`, and no exhaustive `match` at any call site.
///
/// **The SIGN of the band is NOT in this struct**, and that is the point: rung 68's band is
/// `[-v_max, 0]` and rung 69's is `[0, +v_max]`, and the difference is carried by
/// [`TripleHooks::clamp_v`] and [`TripleHooks::check_v0`] — cells — rather than by a field a
/// shared body could read the wrong way round.
impl From<StatorIncidenceLimiter> for StatorLegArm {
    fn from(l: StatorIncidenceLimiter) -> Self {
        StatorLegArm { v_max: l.v_max, tau: l.tau }
    }
}

// ---------------------------------------------------------------------------------------------
// `_ref` — THE CARRIER'S GUARD
// ---------------------------------------------------------------------------------------------

/// The RAII form of Python's `_with_ref`'s `try/finally` — **the restore is `Drop`, so it survives
/// an unwind that a straight-line restore would skip.**
///
/// Python is `prev, self._ref = self._ref, ref` … `try: return fn(*a, **kw)` …
/// `finally: self._ref = prev`.
///
/// **THE GUARD IS SHARED AND THE SETTER IS THE CELL.** Rung 73 overrides `_with_ref` to write a
/// DIFFERENT field (`_ref_law`), and that is the only thing its override changes — so the field
/// choice is dispatched through [`TripleHooks::with_ref`] and this type is written once. Holding
/// the guard by value for the duration of one reader is the whole call: Rust's scoping IS the
/// `finally`, which is strictly stronger than the discipline the Python relies on.
///
/// **RESTORE-PREVIOUS, NOT RESTORE-`None`.** Measured over the rung-69 suite, `_ref`'s previous
/// value is `None` at every one of its 29 value-sets, so the two spellings agree on every shipped
/// path and no value key can tell them apart — [`InitialBleed`]'s situation exactly, and the
/// reason a manufactured nest is the only instrument that reaches it.
///
/// [`InitialBleed`]: crate::two_spool_transient::InitialBleed
pub struct RefScope<'a> {
    core: &'a TwoSpoolTransientCore,
    prev: Option<&'static str>,
}

impl<'a> RefScope<'a> {
    /// Set the reference for as long as the returned guard lives, **through the cell**.
    ///
    /// `None` is a real assignment and not a no-op: it is what `_triple_rig`'s `self._ref or (…)`
    /// fallback reads, so a reader that cleared an outer scope would build a different rig.
    /// Nothing else in the crate may write the carrier — this is the only public way in, which is
    /// what makes the pairing structural rather than a discipline.
    pub fn set(core: &'a TwoSpoolTransientCore, r: Option<&'static str>) -> Self {
        let prev = core.with_ref(r);
        RefScope { core, prev }
    }

    /// What this scope displaced — Python's `prev`, exposed so a gate can read the restore
    /// POLICY rather than only its effect.
    pub fn displaced(&self) -> Option<&'static str> {
        self.prev
    }
}

impl Drop for RefScope<'_> {
    fn drop(&mut self) {
        // Through the SAME cell, so a rung that moves the field moves both halves of the guard at
        // once. Writing `self.core.ref_.set(self.prev)` here would work at rung 69 and silently
        // restore the WRONG field at rung 73.
        (self.core.triple_hooks.with_ref)(self.core, self.prev);
    }
}

// ---------------------------------------------------------------------------------------------
// THE BUILDER — rung 69's four guards, and the inherited ones re-listed
// ---------------------------------------------------------------------------------------------

/// Rung 69's constructor: rung 68's machine with the third loop's REFERENCE as the only new axis.
///
/// **ALL FOUR OF ITS OWN GUARDS ARE REACHABLE — MEASURED, NOT ASSUMED.** Slice U's pre-flight
/// found three shipped asserts no input can reach by sweeping the ARMING COMBINATIONS, so a
/// 96-point grid over `stator_inc` × `stator_lim` × `vsv_lp` × `vsv_sched_lp` × `bleed_lim` ×
/// `lp_disabled` was swept before this function was written:
///
/// | guard | what it refuses | fired |
/// |---|---|---|
/// | A | ONE stator, ONE reference | 4 |
/// | B | constant / schedule / floor are exclusive on the LP | 20 |
/// | C | an incidence floor on a DISABLED LP spool | 10 |
/// | D | **ONE PHYSICAL WALL** — `m_lim` must BE the valve's `phi` floor at the design setting | 1 |
///
/// **13 of 96 points build.** No repeat of slice U's finding.
///
/// **GUARD C IS ASSERTED BEFORE THE BUILD AND THE OTHERS AFTER IT, AND THAT IS A DEVIATION WORTH
/// STATING.** Python runs `super().__init__` first and asserts C afterwards; the port cannot,
/// because rung 57's early return for `lp_disabled` is a SEPARATE CONSTRUCTOR here
/// ([`ScheduledStatorTransient::lp_disabled`]) and the shared one refuses the flag outright with a
/// rung-57 message. Asserting C after the build would leave rung 69's own refusal **unreachable**
/// — a defence with no reader, this phase's most-repeated defect — so it is asserted where it can
/// fire. The observable differs from Python only for a doubly-invalid arming, where Python reports
/// an inherited refusal and this reports rung 69's.
pub fn build_reference_split_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
) -> ScheduledStatorTransient {
    build_split_family_cascade(design_engine, flight_design, mdot_design, map_lp, map_hp, rho,
                               arm, &R69_TWO, &R69_STATOR, &R69_FUEL, &R69, &R69_TRIPLE)
}

/// **THE GUARD SEQUENCE ITSELF, TABLE-PARAMETERISED — and it is a port of INHERITANCE, not a
/// factoring-out of a duplication.**
///
/// Neither `CrossSplitTransient` (rung 70) nor `FullSplitTransient` (rung 71) defines `__init__`
/// in Python at all — measured, not assumed — so both run **this** constructor verbatim and the
/// only thing that differs is which five tables the object is built with. In Python that is what
/// `class Rung70(Rung69)` with no `__init__` MEANS; in a `const`-table architecture it is a
/// parameter. Re-spelling the eleven asserts once per rung would be the copy
/// [[rust-port-copy-vs-rederivation]] warns about in its other direction: a deliberate
/// duplication is one the SOURCE makes, and the source makes none here.
///
/// **GUARD C STAYS ON THE PRE-BUILD SIDE FOR ALL THREE CALLERS BY CONSTRUCTION**, because there
/// is one body. That placement is the deviation the public wrapper's doc comment discloses, and
/// having one body is what keeps the disclosure true for rungs 70 and 71 without restating it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_split_family_cascade(
    design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
    map_lp: Option<ComponentMap>, map_hp: Option<ComponentMap>, rho: f64, arm: &LeverArm,
    two_hooks: &'static TwoSpoolTransientHooks,
    stator_hooks: &'static StatorTransientHooks,
    fuel_hooks: &'static FuelTransientHooks,
    lever_hooks: &'static LeverHooks,
    triple_hooks: &'static TripleHooks,
) -> ScheduledStatorTransient {
    // GUARD C, hoisted — see the note on [`build_reference_split_cascade`], which is the whole
    // reason this body exists once rather than three times.
    assert!(arm.stator_inc.is_none() || !arm.stator.lp_disabled,
            "rung-69's incidence floor watches the LP, which a disabled LP spool does not have.");
    let built = ScheduledStatorTransient::with_ref_tables(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm.stator,
        two_hooks, stator_hooks, fuel_hooks, lever_hooks,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim },
        triple_hooks, arm.stator_lim, arm.stator_inc);
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
    // Rung 68's three, in Python's order.
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
    // RUNG 69's OWN FOUR, in Python's order — C already fired above if it was going to.
    assert!(arm.stator_lim.is_none() || arm.stator_inc.is_none(),
            "rung-69 is ONE stator with ONE reference: give it a phi floor (`stator_lim`, rung \
             68) or an INCIDENCE floor (`stator_inc`, rung 69). Arming both would be two loops \
             on one ACTUATOR, which is a different object again and not what the seam asked.");
    assert!(!(arm.stator_inc.is_some()
              && (arm.stator.vsv_lp != 0.0 || arm.stator.sched_lp.is_some())),
            "rung-69: the LP stators get a CONSTANT setting (53), a SCHEDULE (57) or a FLOOR \
             (68/69) -- exactly one. Rung 68's three-way assert, one reference over.");
    if let (Some(s), Some(b)) = (arm.stator_inc, arm.bleed_lim) {
        let want = built.core().arming().map_lp_design.tan_beta1_crit() - 1.0 / b.phi_lim;
        assert!((s.m_lim - want).abs() <= 1e-12 * 1.0f64.max(want.abs()),
                "rung-69 needs ONE PHYSICAL WALL, which across a change of coordinate is the only \
                 reading of 'one set point' that survives: the incidence floor must BE the \
                 valve's phi floor at the DESIGN setting, m_lim = T_c - 1/phi_lim = {want}, got \
                 {}. Build both from the same `from_margin(cmap, ., sm)`. An offset here would \
                 confound the REFERENCE SPLIT with a set-point offset -- rung 66 measured a -2.5 \
                 % offset moving its own product to 0.951.", s.m_lim);
    }
    built
}

// ---------------------------------------------------------------------------------------------
// THE TABLES — five, and TWO of them carry a cell of this rung's own
// ---------------------------------------------------------------------------------------------

/// RUNG 69's lever table — ONE cell, `at_lever`, whose keyword count reaches its NINTH and last.
pub const R69: LeverHooks = LeverHooks {
    at_lever: r69_at_lever,
    ..crate::three_loop::R68
};

/// RUNG 69's `TwoSpoolTransientHooks` — **ZERO cells swapped**, named for `R66_TWO`'s reason: a
/// spread of the parent would make the NEXT addition to that table silent here.
pub const R69_TWO: TwoSpoolTransientHooks = crate::three_loop::R68_TWO;

/// RUNG 69's fuel table — **ZERO cells swapped**, named for the same reason.
pub const R69_FUEL: FuelTransientHooks = crate::three_loop::R68_FUEL;

/// RUNG 69's stator table — **ZERO cells swapped**, named for the same reason.
pub const R69_STATOR: StatorTransientHooks = crate::three_loop::R68_STATOR;

/// **RUNG 69's THIRD-LOOP TABLE — EIGHT of rung 68's nine cells swapped, plus the one this rung
/// ADDS.**
///
/// `triple_laws` is the ninth and is NOT swapped here — rung **70** overrides it, alone among the
/// nine. It is spelled out below rather than reached through a `..R68_TRIPLE` spread, so that the
/// one INHERITED cell is a decision on the page rather than the residue of a spread. That is the
/// whole shape of this slice's risk: a swap that is silently the parent's compiles and runs.
pub const R69_TRIPLE: TripleHooks = TripleHooks {
    stator_leg: r69_stator_leg,
    lagged_stator: r69_lagged_stator,
    clamp_v: r69_clamp_v,
    check_v0: r69_check_v0,
    rk4_floor: r69_rk4_floor,
    solve_v: r69_solve_v,
    manifold_v: r69_manifold_v,
    // NOT rung 69's — rung 70's. The one cell of the nine this rung inherits.
    triple_laws: crate::three_loop::R68_TRIPLE.triple_laws,
    triple_rig: r69_triple_rig,
    with_ref: r69_with_ref,
    // NONE OF SLICE AD's THREE — all three names arrive at rung 72, so this rung carries
    // `NO_TRIPLE`'s refusal for each. Reached through rung 68's table, which is where the
    // panicking slots live.
    reference: crate::three_loop::R68_TRIPLE.reference,
    rk4_floor_shared: crate::three_loop::R68_TRIPLE.rk4_floor_shared,
    shared_rig: crate::three_loop::R68_TRIPLE.shared_rig,
    // AND SLICE AE STEP 2's FOURTEENTH CELL — `_quad_gains_at` also arrives at rung 72, so this
    // rung carries the same refusal, reached through rung 68's table.
    quad_gains_at: crate::three_loop::R68_TRIPLE.quad_gains_at,
    // NONE OF SLICE AF's FOUR — all four names arrive at rung 74, so this rung carries
    // `NO_TRIPLE`'s refusal for each. Reached through rung 68's table, which is where the
    // panicking slots live.
    cap_fuel: crate::three_loop::R68_TRIPLE.cap_fuel,
    sensed_cap: crate::three_loop::R68_TRIPLE.sensed_cap,
    windup_tau: crate::three_loop::R68_TRIPLE.windup_tau,
    with_coord: crate::three_loop::R68_TRIPLE.with_coord,
};

// ---------------------------------------------------------------------------------------------
// THE ONE ADDED CELL'S BODY — the only body step 1 fills
// ---------------------------------------------------------------------------------------------

/// RUNG 69's `_with_ref` — **the SETTER half of Python's higher-order guard.**
///
/// Writes `_ref` and hands back what it displaced. Rung 73's override writes `_ref_law` instead
/// and changes nothing else, which is exactly why this is the cell and the call is not.
pub fn r69_with_ref(t: &TwoSpoolTransientCore, r: Option<&'static str>) -> Option<&'static str> {
    let prev = t.ref_.get();
    t.ref_.set(r);
    prev
}

// ---------------------------------------------------------------------------------------------
// THE NINE SWAPPED CELLS — BODIES
// ---------------------------------------------------------------------------------------------
//
// **EIGHT OF THE NINE OPEN WITH A REDUCE ARM, AND THAT ARM IS THE RUNG'S OWN CONTRACT**:
// `stator_inc is None` ⇒ rung 68's body VERBATIM, by a direct call and never by a re-derivation.
// `_rk4_floor` is the ninth and has no arm at all — it is a `@staticmethod` in Python with no
// receiver to ask, so it re-derives the SAME constant for a DIFFERENT reason and fires on every
// machine that carries rung 69's table.
//
// **THE PARENT'S BODY IS CALLED, NOT COPIED.** `crate::three_loop::r68_*` are `pub(crate)` for
// exactly this: a reduce arm that re-spells rung 68's expression is a second copy that can drift,
// and the whole reduce contract is *bit-for-bit, by dispatch*.
//
// The counters below are why: a reduce arm and a rung-68 march are the same numbers BY
// CONSTRUCTION, so no value key can tell which arm ran. [`Census69`] is the only instrument that
// can, and it is written here rather than in a test file for [`Census68`]'s corrected lesson.

/// RUNG 69's `_stator_leg` — **the incidence limiter IN PREFERENCE, and the two are mutually
/// exclusive by construction anyway.**
///
/// Python is `self.stator_inc if self.stator_inc is not None else self.stator_lim`, so the
/// preference is written even though guard A already refuses both being armed. That belt-and-
/// braces spelling is transcribed rather than simplified to an `.or()`: the guard lives in the
/// BUILDER, and this cell is reachable from objects the builder never saw.
///
/// The return is [`StatorLegArm`] — slice AA's narrowing, built for this slice — so a caller reads
/// `.tau` and `.v_max` and cannot reach the wall or its sign. **The band's SIGN lives in
/// [`r69_clamp_v`] and [`r69_check_v0`], never in a value a shared body could read backwards.**
fn r69_stator_leg(t: &TwoSpoolTransientCore) -> Option<StatorLegArm> {
    match t.stator.inc {
        Some(l) => {
            bump(&LEG_INC);
            Some(StatorLegArm::from(l))
        }
        None => {
            bump(&LEG_PARENT);
            crate::three_loop::r68_stator_leg(t)
        }
    }
}

/// RUNG 69's `_lagged_stator` — the incidence limiter's own `tau`, else rung 68's answer.
fn r69_lagged_stator(t: &TwoSpoolTransientCore) -> bool {
    match t.stator.inc {
        Some(l) => l.tau.is_some(),
        None => {
            bump(&LAGGED_PARENT);
            crate::three_loop::r68_lagged_stator(t)
        }
    }
}

/// RUNG 69's `_clamp_v` — **`max(0, min(v_max, v))`: THE BAND FLIPS BACK.**
///
/// `M_i` is INCREASING in `v`, so the incidence loop's admissible band is `[0, +v_max]` where rung
/// 68's was `[-v_max, 0]`. Same dormant stop (`v = 0`, the design setting), opposite open side.
///
/// **THE UNARY MINUS IS THE WHOLE DIFFERENCE AND IT READS TO ZERO** — § 5.26 (iv)'s emitted AST
/// diff, not a body read: rung 68 spells one negation of `v_max` here and rung 69 spells none.
/// The pre-flight also measured why *"the clamp never binds"* would have been the wrong
/// instrument: `v > 0` on 25 364 of 25 371 inputs, and on every one of those rung 68's
/// `min(0, max(-v_max, v))` returns **0** where this returns `v`.
fn r69_clamp_v(t: &TwoSpoolTransientCore, v: f64, lim_s: &StatorLegArm) -> f64 {
    if t.stator.inc.is_none() {
        bump(&CLAMP_PARENT);
        return crate::three_loop::r68_clamp_v(t, v, lim_s);
    }
    0.0f64.max(lim_s.v_max.min(v))
}

/// RUNG 69's `_check_v0` — the same band, asserted on an OVERRIDDEN initial position.
///
/// `test_rung69.py` drives it with `pytest.raises(AssertionError, match="stator POSITION")` on a
/// `v0` rung **68** would have accepted, which is why the two halves of the band flip are separate
/// cells: the clamp is silent and this is not.
fn r69_check_v0(t: &TwoSpoolTransientCore, v0: f64, lim_s: &StatorLegArm) {
    if t.stator.inc.is_none() {
        bump(&CHECK_PARENT);
        return crate::three_loop::r68_check_v0(t, v0, lim_s);
    }
    assert!((0.0..=lim_s.v_max).contains(&v0),
            "rung-69 v0 is a stator POSITION on the one-sided band: {v0} is outside [0, {}] -- \
             and note the band is the MIRROR of rung 68's.", lim_s.v_max);
}

/// RUNG 69's `_rk4_floor` — **THE ONE SWAP NO VALUE KEY CAN SEE, and it is not unobservable.**
///
/// The condition is `ds * rate <= 2.0` in BOTH rungs, character for character; § 5.26 (ii)
/// measured 0 disagreements in 77 calls. **The entire difference is the assertion MESSAGE** — rung
/// 68 justifies the constant by *"J has rank one, so the non-zero eigenvalue is exactly
/// −sum(1/tau_i)"*, and under the split that reason is GONE even though the constant survives:
/// `J` is rank TWO and the dominant root is a COMPLEX pair of modulus `sqrt(A z (1-k))`, which by
/// AM-GM is at most `sqrt(1-k)/2` times the rate sum. The inherited `2.0` is therefore
/// conservative for every plant with `k >= -3`, and the measured `k` on this arc is −1.67…−2.01.
///
/// **SO THE GATE IS A `#[should_panic(expected = "rank TWO")]` AND NEVER A VALUE DIFF** — writing
/// it as a value diff is exactly how this cell would end up silently ungated.
///
/// `static` in Python, so no receiver — and therefore **no reduce arm**: it fires on every machine
/// carrying rung 69's table, including one armed with a rung-68 `phi` floor instead.
fn r69_rk4_floor(ds: f64, rate: f64, n_states: usize, tau_s: f64) {
    assert!(ds * rate <= 2.0,
            "rung-69: ds*sum(1/tau_i) = {:.3} is outside the explicit RK4 stability region for \
             the {n_states} actuator states (ds = {ds}, tau_s = {tau_s}). Under the REFERENCE \
             SPLIT the rates no longer simply add -- the block is rank TWO and the dominant root \
             is a COMPLEX pair of modulus sqrt(A*z*(1-k)) -- but that modulus is bounded by \
             sqrt(1-k)/2 times this sum, so the sum is still the conservative guard for k >= -3. \
             Refine the grid or slow a clock.", ds * rate);
}

/// RUNG 69's `_solve_v` — **the smallest `v` in `[0, +v_max]` holding `M_i >= m_lim`.**
///
/// [`r64_solve_b`](crate::limited_bleed::r64_solve_b)'s structure **AND ORIENTATION RESTORED**:
/// `M_i` is INCREASING in `v` (measured `dM_i/dv = +0.335`) exactly as `phi_lp` is increasing in
/// `b`, where rung 68's `_solve_v` had to invert both clamp tests and the bracket because `phi_lp`
/// DECREASES in `v`. Getting the orientation wrong returns a wrong regime label with nothing
/// raising — rung 62's `_powers` trap, and this is its FIFTH reload.
///
/// **THE REGIME IS CARRIED, never re-derived from the float** — rung 68's saturation counterfeit
/// applies here verbatim, and the suite gates it directly.
fn r69_solve_v(
    t: &TwoSpoolTransientCore,
    closer: &dyn Fn(f64) -> Result<FuelCloseState, Abort>,
) -> Result<(FuelCloseState, f64, Regime), Abort> {
    let lim = match t.stator.inc {
        None => {
            bump(&SOLVE_PARENT);
            return crate::three_loop::r68_solve_v(t, closer);
        }
        Some(l) => l,
    };
    bump(&SOLVE_V_CALLS);
    let t_c = t.stator.map_lp_design.tan_beta1_crit();
    let m_of = |v: f64, c: &FuelCloseState| StatorIncidenceLimiter::margin(t_c, c.base.phi_lp, v);
    let c0 = closer(0.0)?;
    let f0 = m_of(0.0, &c0) - lim.m_lim;
    if f0 >= 0.0 {
        bump(&REGIME_V_DORMANT);
        return Ok((c0, 0.0, Regime::Dormant));
    }
    let c1 = closer(lim.v_max)?;
    let f1 = m_of(lim.v_max, &c1) - lim.m_lim;
    if f1 <= 0.0 {
        bump(&REGIME_V_SATURATED);
        return Ok((c1, lim.v_max, Regime::Saturated));
    }
    // Python's argument order is `(f, 0.0, v_max, f0, f1)` — the LOW end first, which here is the
    // DORMANT stop where rung 68's was the saturated one. Transposing the two residuals is a
    // wrong first secant that still converges, to a root a few ulps away.
    let v = try_illinois(|v| closer(v).map(|c| m_of(v, &c) - lim.m_lim),
                         0.0, lim.v_max, f0, f1, 1e-13, ILLINOIS_MAXIT)?;
    bump(&REGIME_V_RIDING);
    Ok((closer(v)?, v, Regime::Riding))
}

/// RUNG 69's `_manifold_v` — **the SHARED constraint's manifold, and there is no longer a point
/// where all three laws hold at once.**
///
/// At rung 68 the stator's OWN root IS the shared manifold, so that body is `V(g, q)[0]` and the
/// four arguments it ignores are carried only for this rung. Under the split, `phi = phi_lim` and
/// `M_i = m_lim` together force `v = 0` — the stator's own dormant stop — so the only base point
/// at which ANY row-pair of the block is exactly parallel is the wall the FUEL leg and the VALVE
/// both hold.
///
/// **ROOTED UNCLAMPED, ON `[-0.6, +0.6]`** — it is a diagnostic base point and not a state. The
/// incidence limiter's own band is `[0, v_max]` and the shared manifold sits at `v < 0` wherever
/// the two `phi` loops are still lagging their commands, so clamping it to the band would return
/// the stop and call it a manifold.
///
/// `_b_state = q` is held for the whole body, as Python's `try/finally` does — the mirror of
/// `_triple_laws`'s `V` law, which trials `v` and so must see the VALVE as it actually is.
#[allow(clippy::too_many_arguments)]
fn r69_manifold_v(
    core: &ScheduledStatorCore, flight: &FlightCondition, a: f64, h: f64, mf_sched: f64, g: f64,
    q: f64, v_law: &dyn Fn(f64, f64) -> Result<(f64, Regime), Abort>,
) -> Result<f64, Abort> {
    let inc = match core.fuel.inner.stator.inc {
        None => {
            bump(&MANIFOLD_PARENT);
            return crate::three_loop::r68_manifold_v(core, flight, a, h, mf_sched, g, q, v_law);
        }
        Some(l) => l,
    };
    let ft = &core.fuel;
    let phi_lim = inc.phi_lim_at(&ft.inner.stator.map_lp_design);
    let (tt2, pt2, _) = ft.inner.inlet(flight);
    let _sb = MarchedBleed::set(&ft.inner, q);
    // **THE RUNG-62 PIN**, through rung 68's own `_closer_v`: the closure is the parent's, CALLED
    // rather than re-spelled, so `self._closer_v(base_close, …)` stays one function.
    let closer = crate::three_loop::closer_v(ft, a, h, 1e-9f64.max(mf_sched - g), tt2, pt2);
    let f = |v: f64| closer(v).map(|c| c.base.phi_lp - phi_lim);
    let (lo, hi) = (-0.6f64, 0.6f64);
    let (flo, fhi) = (f(lo)?, f(hi)?);
    // The two `{:.4e}` fields print `4.2e-3` where Python prints `4.2000e-03`. Nothing matches on
    // this message — it is the one rung-69 assert with no `pytest.raises` — and reproducing
    // Python's exponent padding would be a formatter of its own for a string only a human reads.
    assert!(flo * fhi < 0.0,
            "rung-69: the SHARED manifold (phi_lp = phi_lim) is not bracketed by the LP stator \
             on [{lo}, {hi}] at ({a:.4}, {h:.4}): phi - phi_lim = ({flo:.4e}, {fhi:.4e}). s 1's \
             identities are stated at that base point and under the split there is no substitute \
             for it.");
    try_illinois(f, lo, hi, flo, fhi, 1e-14, ILLINOIS_MAXIT)
}

/// RUNG 69's `_triple_rig` — **rung 68's rig with the stator's REFERENCE as the only new axis.**
///
/// Every cell of every table in this rung comes from here, so a cell can differ from another ONLY
/// by which loops are armed and which coordinate the third one watches — rung 63's lesson, and the
/// reason the two references' ledgers are differenceable at all.
///
/// **THE REFERENCE IS CONSUMED AT CONSTRUCTION AND NEVER RE-READ.** Python is
/// `self._ref or ("phi" if self.stator_lim is not None else "inc")`, and three of the six readers
/// build the rig inside the scope and march the returned machine after it has CLOSED. A body that
/// consulted the carrier later would see `None`, fall through to the fallback, and label the wrong
/// arm — and § 5.26.1 (j) records why the obvious ledger key cannot see that: `reference_bill`'s
/// `bare`/`F`/`V`/`FV` cells carry no stator and are identical between the references BY
/// CONSTRUCTION, so `common_max_rel` reads ~0 with the defect live.
fn r69_triple_rig(
    core: &ScheduledStatorCore, arm: &TripleRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    let t = &core.fuel.inner;
    let reference =
        t.ref_.get().unwrap_or(if t.stator.lim.is_some() { "phi" } else { "inc" });
    assert!(reference == "inc" || reference == "phi",
            "rung-69 reference is 'inc' or 'phi'; got {reference:?}");
    let cmap = core.arming().map_lp_design;
    let b_max = t.lever.lim.map(|l| l.b_max).unwrap_or(0.10);
    let bl = if arm.valve {
        Some(BleedLimiter::from_margin_tau(&cmap, b_max, arm.sm, Some(arm.tau)))
    } else {
        None
    };
    let (sl, si) = match (arm.stator, reference == "phi") {
        (false, _) => (None, None),
        (true, true) => {
            bump(&RIG_PHI);
            (Some(StatorLimiter::from_margin(&cmap, arm.v_max, arm.sm, Some(arm.tau_s))), None)
        }
        (true, false) => {
            bump(&RIG_INC);
            (None,
             Some(StatorIncidenceLimiter::from_margin(&cmap, arm.v_max, arm.sm, Some(arm.tau_s))))
        }
    };
    let m = core.at_lever(&LeverArm {
        bleed_lim: bl, stator_lim: sl, stator_inc: si, ..Default::default() });
    let surge = if arm.fuel {
        Some(Floor::Phi(SurgeLimiter::from_margin(&cmap, Spool::Lp, arm.sm)))
    } else {
        None
    };
    let lag = if arm.fuel { Some(AsymmetricLag::new(arm.tau_att, arm.tau_rel)) } else { None };
    (m, surge, lag)
}

/// RUNG 69's `at_lever` — **the SEVENTH instance of the sibling-constructor trap, and the second
/// in a row where the signature GROWS.**
///
/// So *"silently drops the third loop"* now has a sibling failure mode, *"silently swaps its
/// REFERENCE"*, and **no float would reveal it**: a machine handed back with `stator_lim` where
/// `stator_inc` was asked for still marches five states and still reports a stator credit. The
/// pre-flight measured the traffic — 31 of 61 calls carry `stator_inc` IN and **0 lose it** — so
/// the keyword is load-bearing on half the call sites.
///
/// It routes through [`build_reference_split_cascade`] and therefore re-asserts all four of rung
/// 69's guards on every sibling, which is what keeps a rig from being built past guard D.
fn r69_at_lever(core: &ScheduledStatorCore, arm: &LeverArm) -> ScheduledStatorCore {
    match build_reference_split_cascade(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(), arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("at_lever never disables LP"),
    }
}

// ---------------------------------------------------------------------------------------------
// COUNTERS — the reduce arms and the rig's reference are invisible to every value key
// ---------------------------------------------------------------------------------------------
//
// Two things this rung does cannot be reached from a float a reader can print:
//
// * **THE REDUCE.** Eight cells open with `stator_inc is None ⇒ the parent`, and a reduce arm
//   emits rung 68's numbers BY CONSTRUCTION. That is the contract, so agreement proves nothing
//   about WHICH body ran — a cell wired to rung 68 unconditionally passes every reduce gate.
// * **THE REFERENCE.** `_triple_rig` picks `phi` or `inc`, and § 5.26.1 (j) measured that the
//   ledger cells a wrong pick would move are exactly the ones identical between the arms.
//
// Both are what `slice_ab_dispatch.rs` reads at step 5.

thread_local! {
    static SOLVE_V_CALLS: Cell<u64> = const { Cell::new(0) };
    static REGIME_V_DORMANT: Cell<u64> = const { Cell::new(0) };
    static REGIME_V_RIDING: Cell<u64> = const { Cell::new(0) };
    static REGIME_V_SATURATED: Cell<u64> = const { Cell::new(0) };
    static LEG_INC: Cell<u64> = const { Cell::new(0) };
    static LEG_PARENT: Cell<u64> = const { Cell::new(0) };
    static LAGGED_PARENT: Cell<u64> = const { Cell::new(0) };
    static CLAMP_PARENT: Cell<u64> = const { Cell::new(0) };
    static CHECK_PARENT: Cell<u64> = const { Cell::new(0) };
    static SOLVE_PARENT: Cell<u64> = const { Cell::new(0) };
    static MANIFOLD_PARENT: Cell<u64> = const { Cell::new(0) };
    static RIG_INC: Cell<u64> = const { Cell::new(0) };
    static RIG_PHI: Cell<u64> = const { Cell::new(0) };
}

fn bump(c: &'static std::thread::LocalKey<Cell<u64>>) {
    c.with(|x| x.set(x.get() + 1));
}

/// What the counters above hold. [`Census68`](crate::three_loop::Census68)'s shape one rung on,
/// and its corrected lesson: a defence whose only reader is a test can be deleted with the test
/// and nobody notices, so the declaration lives beside the bodies it counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Census69 {
    /// Calls to rung 69's OWN `_solve_v` body — the parent arm is counted separately.
    pub solve_v_calls: u64,
    pub regime_dormant: u64,
    pub regime_riding: u64,
    pub regime_saturated: u64,
    /// `_stator_leg` returning the INCIDENCE limiter.
    pub leg_inc: u64,
    /// The six reduce arms — every one of them rung 68's body, reached by a direct call.
    pub leg_parent: u64,
    pub lagged_parent: u64,
    pub clamp_parent: u64,
    pub check_parent: u64,
    pub solve_parent: u64,
    pub manifold_parent: u64,
    /// Which reference `_triple_rig` armed, which is the ONE thing this rung adds and the one
    /// thing § 5.26.1 (j) says no ledger key can see.
    pub rig_inc: u64,
    pub rig_phi: u64,
}

impl Census69 {
    pub fn read() -> Self {
        Census69 {
            solve_v_calls: SOLVE_V_CALLS.with(|x| x.get()),
            regime_dormant: REGIME_V_DORMANT.with(|x| x.get()),
            regime_riding: REGIME_V_RIDING.with(|x| x.get()),
            regime_saturated: REGIME_V_SATURATED.with(|x| x.get()),
            leg_inc: LEG_INC.with(|x| x.get()),
            leg_parent: LEG_PARENT.with(|x| x.get()),
            lagged_parent: LAGGED_PARENT.with(|x| x.get()),
            clamp_parent: CLAMP_PARENT.with(|x| x.get()),
            check_parent: CHECK_PARENT.with(|x| x.get()),
            solve_parent: SOLVE_PARENT.with(|x| x.get()),
            manifold_parent: MANIFOLD_PARENT.with(|x| x.get()),
            rig_inc: RIG_INC.with(|x| x.get()),
            rig_phi: RIG_PHI.with(|x| x.get()),
        }
    }

    /// Thread-local with no per-test reset, so every gate resets first. Cargo gives each `#[test]`
    /// its own thread today; the reset makes that irrelevant rather than relied upon.
    pub fn reset() {
        for c in [&SOLVE_V_CALLS, &REGIME_V_DORMANT, &REGIME_V_RIDING, &REGIME_V_SATURATED,
                  &LEG_INC, &LEG_PARENT, &LAGGED_PARENT, &CLAMP_PARENT, &CHECK_PARENT,
                  &SOLVE_PARENT, &MANIFOLD_PARENT, &RIG_INC, &RIG_PHI] {
            c.with(|x| x.set(0));
        }
    }
}

// ---------------------------------------------------------------------------------------------
// § 1/3 — THE SPECTRUM: a COMPLEX pair, and a Newton march that does not converge
// ---------------------------------------------------------------------------------------------

/// A complex double, for [`cubic_roots_c`] and its readers alone.
///
/// **THE PORT'S FIRST COMPLEX NUMBER, AND IT EXISTS BECAUSE RUNG 68's ROOT FINDER THREW AWAY THE
/// INFORMATION THIS RUNG NEEDS.** [`cubic_roots`](crate::three_loop::cubic_roots) deflates on the
/// DOMINANT root and reports a complex pair's real part TWICE; rung 69's claim is that the freed
/// root does NOT land on the real axis, so a reader that could not see an imaginary part could not
/// state it.
///
/// # WHICH OPERATIONS EXIST HERE IS A CENSUS OF PYTHON's CALL SITES, AND SLICE AC's STEP 2 GREW IT
///
/// **This comment used to read *"three operations are needed … a fuller one would invite a reader
/// to compose operations whose Python counterpart was never called"*, and rung 70 falsified it.**
/// [`_zeta_pair`](crate::cross_split::zeta_pair) spells `nz[0]+nz[1]`, `nz[0]*nz[1]`,
/// `cmath.sqrt(p)` and `-s / (2.0*rt)` — a complex sum, a complex PRODUCT and a complex
/// DIVISION, none of which rung 69 called. So the type grows, and the ORIGINAL RULE IS KEPT
/// RATHER THAN DROPPED: every operation below is here because a Python line calls it, spelled the
/// way CPython and PyPy spell it, and nothing is added for symmetry. The corrected form of the
/// sentence is *a census is only as wide as the rungs that have been read* — the same shape as
/// [`csqrt_real`]'s *"the only form this rung calls"*, which slice AC's pre-flight flagged as
/// exactly the kind of sentence this slice had already falsified once.
///
/// The one operation with a genuine numerical choice is [`c_div`]: § 5.27 (iv) priced a schoolbook
/// spelling against Python's over all 18 captured `_zeta_pair` calls and it agrees on only
/// **13 of 18**, worst gap 4.44e−16.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct C64 {
    pub re: f64,
    pub im: f64,
}

impl C64 {
    /// Python's `abs(z)` — `hypot`, and **not** `sqrt(re*re + im*im)`.
    ///
    /// CPython's `_Py_c_abs` and PyPy's `rcomplex.c_abs` both call the platform `hypot`, and so
    /// does Rust's [`f64::hypot`]; the naive spelling is a different function (it overflows and
    /// loses half the significand where `hypot` does not). Every root with `im == 0.0` — which is
    /// all three on the real branch — reduces to `|re|` exactly on any conforming `hypot`, so the
    /// only rows where the library is load-bearing are the genuinely complex pairs. **Registered
    /// as the slice's one platform-library exposure; step 4's oracle is what measures it.**
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
}

/// Python's `cmath.sqrt(complex(d, 0.0))` for a REAL argument — the only form this rung calls.
///
/// CPython's `c_sqrt` computes `s = 2*sqrt(ax/8 + hypot(ax/8, ay/8))`, and with `ay = 0` every
/// step is exact: `ax/8` is a power-of-two scaling, `hypot(x, 0) = |x|`, the sum is `ax/4`, and
/// `sqrt(ax/4) = sqrt(ax)/2` exactly because both operations shift the exponent. So the answer is
/// `sqrt(|d|)` on the appropriate axis, and no `hypot` survives into the result. PyPy's
/// `rcomplex.c_sqrt` is the same algorithm.
///
/// **THE ZERO CASE IS CPython's OWN EARLY RETURN, NOT A SIMPLIFICATION**: `complex(-0.0, 0.0)`
/// hits `z.real == 0 && z.imag == 0` and returns `+0 + 0j`, where `(-0.0).sqrt()` would hand back
/// `-0.0` and change the sign of a root's real part when `p` is also zero.
///
/// Restricted to normal magnitudes: the subnormal rescaling branch is not reproduced, and `d` here
/// is `p*p - 4q` with `p`, `q` of order 1e2…1e4.
fn csqrt_real(d: f64) -> C64 {
    if d == 0.0 {
        return C64 { re: 0.0, im: 0.0 };
    }
    if d > 0.0 {
        C64 { re: d.sqrt(), im: 0.0 }
    } else {
        C64 { re: 0.0, im: (-d).sqrt() }
    }
}

/// RUNG 69's `_cubic_roots_c` — roots of `l^3 - c2 l^2 + c1 l - c0` **as complex numbers, deflating
/// on the root nearest ZERO.**
///
/// [`cubic_roots`](crate::three_loop::cubic_roots) deflates on the DOMINANT root, which discards
/// exactly what this rung measures. Here the predicted spectrum is one near-zero root and a
/// genuinely complex pair, and `l ~ c0/c1` is its own first Newton step from `x = 0`.
///
/// # THE SURFACE IS SHARP, AND IT WAS MEASURED BEFORE A LINE WAS WRITTEN
///
/// Over the 256 calls the shipped suite makes (§ 5.26 (iii)):
///
/// | | measured |
/// |---|---|
/// | iterations | `{2: 103, 8: 9, 9: 18, 10: 15, 11: 21, 12: 12, 13: 3, 19: 3, `**`80: 72`**`}` |
/// | exit taken | `tol` 184 · **`EXHAUSTED-80` 72** · `d == 0` 0 |
/// | discriminant | REAL pair 192 · COMPLEX pair 64 |
///
/// **72 of 256 calls run out of budget rather than converging, and none of them settles into an
/// ulp limit cycle** — the last six iterates wander over two decades and change sign, with
/// `|f(x)|` at exit up to 1.3e−10 against a tolerance of 6e−14. The reason is in the coefficients:
/// the near-zero pair is COMPLEX, so there is no real root near the start point and Newton on the
/// real line cannot find one. **The exit value is an arbitrary point of a chaotic march, and the
/// port owes all 80 steps of it bit-for-bit.**
///
/// That is not a hope: probe 7c replayed all 256 coefficient triples under both interpreters and
/// the exit value AND the iteration count agree on 256/256. Plain IEEE multiply/add throughout —
/// no `sum()`, no library call — so a Rust translation that does not fuse reproduces it. The one
/// derived key, `reference_modes`'s `n_zero`, sits 3.5 decades from its own threshold with **0**
/// triples within a decade of flipping it.
///
/// # AND NOTHING IN THIS SLICE'S TEST FILES CAN SEE THE BUDGET — MEASURED AT STEP 3
///
/// The `80` above is load-bearing and **unreachable by any value gate**. Cutting it to 20 and
/// re-running every slice-AB binary (`rung69`, `slice_ab_smoke`, `slice_ab_cells`) leaves all of
/// them GREEN, and the injection is emphatically not inert: it moves **56 of 243** root components
/// and **24 of 81** `worst_zero` values on the shipped clock grid. What it does not move is
/// `n_zero` — **0 of 81** — which is the only derived key a gate reads, and § 5.26 (iii)
/// measured why: the threshold sits 3.5 decades away.
///
/// So the exhausted arm's exit value is REPRODUCIBLE-BY-CONTRACT and GATED-BY-NOTHING here.
/// **The instrument is step 4's oracle**, which dumps the roots themselves; this note exists so
/// that a future thinning of that dump is a decision rather than an accident.
///
/// The tolerance is `1e-15 * max(|c2|, |x|, 1.0)` — a THREE-argument Python `max`, so
/// [`py_max3`](crate::lagged_bleed::py_max3) and not a chain of [`f64::max`], which differ on NaN
/// in the first position.
pub fn cubic_roots_c(c2: f64, c1: f64, c0: f64) -> [C64; 3] {
    let f = |x: f64| ((x - c2) * x + c1) * x - c0;
    let fp = |x: f64| (3.0 * x - 2.0 * c2) * x + c1;
    let mut x = 0.0f64;
    for _ in 0..80 {
        let d = fp(x);
        if d == 0.0 {
            break;
        }
        let step = f(x) / d;
        x -= step;
        if step.abs() <= 1e-15 * py_max3(c2.abs(), x.abs(), 1.0) {
            break;
        }
    }
    // deflate: `l^3 - c2 l^2 + c1 l - c0 = (l - x)(l^2 + p l + q)`
    let (p, q) = (x - c2, c1 - (c2 - x) * x);
    let rt = csqrt_real(p * p - 4.0 * q);
    // Python promotes `-p` to `complex(-p, 0.0)` before adding, so the imaginary parts are
    // `0.0 + rt.im` and `0.0 - rt.im` — spelled out, because `-rt.im` hands back `-0.0` on the
    // real branch where Python has `+0.0`. The `0.5 *` is [`py_half`], NOT a scaling.
    [C64 { re: x, im: 0.0 },
     py_half(C64 { re: -p + rt.re, im: 0.0 + rt.im }),
     py_half(C64 { re: -p - rt.re, im: 0.0 - rt.im })]
}

/// Python's `0.5 * z` — **A FULL COMPLEX PRODUCT, NOT A SCALING OF TWO FLOATS.**
///
/// `float * complex` returns `NotImplemented` and Python falls through to `complex.__rmul__`,
/// which promotes `0.5` to `complex(0.5, 0.0)` and runs the four-multiply product
/// (`_Py_c_prod` in CPython, `rcomplex.c_mul` in PyPy):
///
/// ```text
/// re = 0.5*z.re - 0.0*z.im        im = 0.5*z.im + 0.0*z.re
/// ```
///
/// The cross terms are `0.0 * something`, which is **not** an identity on a signed zero: on the
/// `(-2, 5, -10)` triple the deflated pair is `-0 ± 4.472j`, so the third root's real part is
/// `0.5*(-0.0) - 0.0*(-4.472j.im)` = `(-0.0) - (-0.0)` = **`+0.0`**, where the naive `0.5 * z.re`
/// hands back `-0.0`.
///
/// **STEP 2 SPELLED OUT THE SIGN-OF-ZERO DECISION FOR THE ADDITION AND MISSED IT FOR THE
/// SCALING** (§ 5.26.2 (e)). The shipped suite's own grid never reaches it — every root sections
/// D, E and F compute agrees bit-for-bit either way — and it was step 4's DECLARED EXTRA table of
/// direct calls that caught it, in one key of 15 957. The lesson is the phase's own: a reader that
/// reasons about one operation of an expression has said nothing about the next one.
///
/// Not folded by the optimiser: Rust guarantees IEEE-754 semantics with no fast-math, and
/// `x - 0.0*y` is not `x` under those rules.
fn py_half(z: C64) -> C64 {
    C64 { re: 0.5 * z.re - 0.0 * z.im, im: 0.5 * z.im + 0.0 * z.re }
}

/// Python's `2.0 * z` — **[`py_half`]'s trap with the other constant**, and it is in the very
/// expression slice AC ports.
///
/// `float * complex` returns `NotImplemented`, Python promotes `2.0` to `complex(2.0, 0.0)` and
/// runs the four-multiply product, so the cross terms are `0.0 * something` and are **not** an
/// identity on a signed zero. § 5.27 (iv) measured it: `2.0 * complex(3.0, -0.0)` is `(6+0j)`
/// where a component-wise scaling gives `(6-0j)`. Slice AB found ONE port defect in 15 957 keys
/// and it was exactly this class at `0.5`; the reader who fixed it there had said nothing about
/// the next constant.
pub fn py_two(z: C64) -> C64 {
    C64 { re: 2.0 * z.re - 0.0 * z.im, im: 2.0 * z.im + 0.0 * z.re }
}

/// Python's `z1 + z2` — componentwise, `_Py_c_sum` / `rcomplex.c_add`.
///
/// No sign-of-zero subtlety of its own (an addition of two componentwise sums is exactly the
/// componentwise sum), so this is the one new operation with nothing to disclose. It is spelled
/// out anyway because [`c_mul`]'s and [`c_div`]'s are next to it and a reader comparing the three
/// should see all three bodies.
pub fn c_add(a: C64, b: C64) -> C64 {
    C64 { re: a.re + b.re, im: a.im + b.im }
}

/// Python's `z1 * z2` — the FOUR-multiply product, `_Py_c_prod` / `rcomplex.c_mul`.
///
/// § 5.27 (iv) replayed all 18 captured `nz[0]*nz[1]` calls against this spelling: **18 of 18**
/// agree, so unlike [`c_div`] the schoolbook form is free HERE — measured on this plant's sample,
/// not free in principle.
pub fn c_mul(a: C64, b: C64) -> C64 {
    C64 { re: a.re * b.re - a.im * b.im, im: a.re * b.im + a.im * b.re }
}

/// Python's unary `-z` — `complex.__neg__`, componentwise negation.
///
/// Spelled rather than folded into the division's caller because `-s / (2.0*rt)` negates BEFORE
/// dividing, and `-(s/d)` and `(-s)/d` are not the same expression under [`c_div`]'s branch test
/// (the branch is chosen on the DENOMINATOR, but the numerator's signs ride through three
/// different sums).
pub fn c_neg(z: C64) -> C64 {
    C64 { re: -z.re, im: -z.im }
}

/// Python's `z1 - z2` — componentwise, `_Py_c_diff` / `rcomplex.c_sub`.
///
/// **SLICE AD GREW THE CENSUS A SECOND TIME, AND THE RULE ABOVE IS WHY THAT IS NOT A SURPRISE.**
/// Rung 72's `_quartic_roots_c` spells `z[i] - z[j]` (Durand–Kerner's denominator) and
/// `z[i] -= d`, neither of which rungs 69 or 70 called. The type's own doc already records that
/// *a census is only as wide as the rungs that have been read*; this is the third reading.
pub fn c_sub(a: C64, b: C64) -> C64 {
    C64 { re: a.re - b.re, im: a.im - b.im }
}

/// Python's promotion of a `float` to a `complex` before a mixed-type arithmetic op.
///
/// `complex.__mul__` / `__add__` return `NotImplemented` for a `float` operand only after
/// `TO_COMPLEX` has widened it to `(x, 0.0)`; every mixed expression in `_quartic_roots_c`
/// (`z + a3`, `x * scale`, `den == 0`) therefore runs the FULL complex op with a zero imaginary
/// part, not a componentwise shortcut. [`py_half`] and [`py_two`] are this function specialised to
/// the two constants slice AB and AC met; rung 72 multiplies by a *variable*, so the promotion is
/// spelled once and handed to [`c_mul`] / [`c_add`] rather than open-coded per call site.
pub fn c_real(x: f64) -> C64 {
    C64 { re: x, im: 0.0 }
}

/// Python's `complex == 0` — **BOTH components, and the `int` is promoted, not the complex
/// demoted.**
///
/// `_quartic_roots_c` guards its Durand–Kerner denominator with `if den == 0:`. Python widens the
/// `int` to `complex(0.0, 0.0)` and compares componentwise, so a denominator of `0 + 1e-30j` is
/// **not** zero and the guard does not fire. A port that tested `den.re == 0.0` alone would fire
/// the guard where Python does not.
///
/// § 5.28 (iii) measured the guard **never firing** on any of the 1 068 shipped calls, so this is
/// ported for faithfulness and **disclosed rather than gated** — a gate on an unreachable branch
/// passes forever and says nothing.
pub fn c_is_zero(z: C64) -> bool {
    z.re == 0.0 && z.im == 0.0
}

/// Python's `z ** k` for a small non-negative `int` — **CPython's `c_powu`, which is BINARY
/// exponentiation and not a naive repeated product.**
///
/// `complex_pow` takes the integer fast path when the exponent is real, integral and `|k| <= 100`,
/// and `c_powi` then calls `c_powu`: a `mask`-doubling loop that squares `p` each round and
/// multiplies the accumulator `r` (which starts at `1 + 0j`) by `p` on each set bit. PyPy's
/// `rcomplex` takes the same path.
///
/// **THE ASSOCIATION ORDER IS OBSERVABLE, WHICH IS WHY THE LOOP IS COPIED RATHER THAN
/// SUMMARISED.** At `k = 3` this computes `z * (z*z)`; a naive `((z*z)*z)` is a different
/// expression in floating point. § 5.28 (iii)'s probe E measured a naive repeated product agreeing
/// with both interpreters on all four exponents *for the one base rung 72 uses*
/// (`complex(0.4, 0.9)`) — a measurement on one value, not a licence, and `exp(k log z)` and the
/// polar form both DIFFER at `k = 1, 2, 3`. The algorithm below cannot be wrong on any base.
///
/// The `r = c_1 * p` products are kept: `_Py_c_prod(1+0j, p)` is `(1*p.re - 0*p.im, 1*p.im +
/// 0*p.re)`, whose cross terms are `0.0 * something` and are therefore **not** an identity on a
/// signed zero — [`py_half`]'s trap, at the multiplicative unit.
pub fn c_powu(z: C64, k: u32) -> C64 {
    let mut r = C64 { re: 1.0, im: 0.0 };
    let mut p = z;
    let mut mask: u32 = 1;
    while mask > 0 && k >= mask {
        if k & mask != 0 {
            r = c_mul(r, p);
        }
        mask <<= 1;
        p = c_mul(p, p);
    }
    r
}

/// Python's `z1 / z2` — **SMITH's ALGORITHM, and this is the one operation in slice AC that does
/// not survive a schoolbook spelling.**
///
/// CPython's `_Py_c_quot` and PyPy's `rcomplex.c_div` are the same body: divide top and bottom by
/// whichever component of the denominator is LARGER in magnitude, which is what keeps the
/// intermediate `|b|^2` from overflowing or underflowing. The textbook
/// `(a * conj(b)) / (b.re^2 + b.im^2)` is a different function in floating point, and § 5.27 (iv)
/// priced the difference on this rung's own sample:
///
/// | operation | schoolbook == Python | worst gap |
/// |---|---|---|
/// | complex × complex | 18 / 18 | 0 |
/// | `cmath.sqrt(p)` | 18 / 18 | 0 |
/// | **complex division** | **13 / 18** | **4.44e−16 abs**, and `.real` differs on the same 5 |
///
/// So this is not a precaution — five of eighteen shipped `_zeta_pair` values move if it is
/// written the obvious way.
///
/// **THE NaN ARM IS CPython's OWN, NOT A DEFENCE.** `abs_br >= abs_bi` and `abs_bi >= abs_br` are
/// BOTH false only when a component is NaN, and both interpreters then return `NaN + NaN j`
/// rather than falling into either branch. Reproduced because it is a branch of the source; the
/// zero-denominator case raises `ZeroDivisionError` in Python and its only caller here guards it
/// with `abs(rt) == 0.0` first, so it panics rather than inventing a value.
pub fn c_div(a: C64, b: C64) -> C64 {
    let abs_br = if b.re < 0.0 { -b.re } else { b.re };
    let abs_bi = if b.im < 0.0 { -b.im } else { b.im };
    if abs_br >= abs_bi {
        assert!(abs_br != 0.0,
                "complex division by zero -- Python raises ZeroDivisionError here, and rung 70's                  `_zeta_pair` guards it with `abs(rt) == 0.0` BEFORE dividing. Reaching this means                  the guard was lost.");
        let ratio = b.im / b.re;
        let denom = b.re + b.im * ratio;
        C64 { re: (a.re + a.im * ratio) / denom, im: (a.im - a.re * ratio) / denom }
    } else if abs_bi >= abs_br {
        let ratio = b.re / b.im;
        let denom = b.re * ratio + b.im;
        C64 { re: (a.re * ratio + a.im) / denom, im: (a.im * ratio - a.re) / denom }
    } else {
        // Neither comparison holds => a component is NaN. CPython's own third arm.
        C64 { re: f64::NAN, im: f64::NAN }
    }
}

/// Python's `cmath.sqrt(z)` for a GENERAL complex argument — CPython's `c_sqrt`, in full.
///
/// # THE GATED CONDITION THIS REPLACES WAS FALSIFIED BY A SHIPPED TEST ONE FILE OVER
///
/// § 5.27 (iv) registered `p.im == 0` as a **gated condition rather than an assumption**: `p` is
/// the product of the two largest-modulus roots of a real cubic, so it is real whenever those two
/// are a conjugate pair or both real — measured positive-real on **18 of 18** calls of the shipped
/// rung-70 READERS. Step 3 shipped it as an `assert!`, with the honest caveat that a near-zero
/// root inside the complex pair leaves `nz` holding one real and one complex root and makes `p`
/// genuinely complex.
///
/// **That caveat is not hypothetical — `tests/test_rung71.py` DRIVES it on purpose.** Rung 71's
/// damping-reader gate hands rung 70's `_zeta_pair` the constructed spectrum
/// `[-194, -23 ± 25.5i]`, whose two largest moduli are one real root and one member of the pair:
/// `p = 4462 + 4947i`, and Python answers `1.27809528556979`. The assertion refused it.
///
/// **AND THE NUMBER WAS ALREADY WRITTEN DOWN IN THIS PORT.** [`zeta_ring`]'s own doc comment
/// quotes the four arms where the two readers disagree, *"1.279 vs 0.670"* among them — so the
/// port published a value that its own `sqrt` could not produce, in a file committed at the same
/// step. The measurement was of the READERS; the claim was about the RUNG. Those are different
/// sets, and the shipped suite is in the second one.
///
/// # THE `assert!` CAUGHT WHAT THE GATE COULD NOT, AND THAT IS THE HALF WORTH KEEPING
///
/// The real-only spelling does not merely refuse this call — driven past its own assertion it
/// returns `1.624295178664163` where Python returns `1.278095…`. **And rung 71's shipped gate
/// passes on both**: it asks `|zeta_pair(bad) − ring| > 0.5`, and the two candidates are `0.608`
/// and `0.954` from `ring`. A one-sided bar cannot tell a right answer from a wrong one on the
/// same side of it, so a port that had shipped the fast path WITHOUT the assertion would have
/// been green and wrong. Step 4 found gates too weak to catch an injection; this is the mirror
/// — a defensive assertion catching what the ported gate structurally could not.
///
/// # THE SPELLING IS CPython's, AND THE REAL BRANCH IS UNMOVED IN VALUE
///
/// ```text
/// ax /= 8;   s = 2*sqrt(ax + hypot(ax, ay/8));   d = ay/(2*s)
/// re >= 0 ? (s, copysign(d, im)) : (d, copysign(s, im))
/// ```
///
/// On a real argument every arithmetic step is exact — `ax/8` is a power-of-two scaling,
/// `hypot(x, 0) = |x|`, the sum is `ax/4`, and `2*sqrt(ax/4) = sqrt(ax)` because both operations
/// only shift the exponent. The one difference is the SIGN OF A ZERO: `copysign(d, im)` carries
/// `im`'s sign, and the real-only spelling always returned `+0.0`.
///
/// **THAT IS NOT THE RARE CASE — IT IS THE COMMON ONE, AND THE FIRST WRITING OF THIS PARAGRAPH
/// SAID THE OPPOSITE.** Intercepting all 96 `cmath.sqrt(p)` calls the two shipped suites make:
///
/// | | measured |
/// |---|---|
/// | `p.im == -0.0` | **90** of 96 — and `p.re < 0` on **0** of those |
/// | `p.im == +0.0` | 5 |
/// | genuinely complex `p` | **1** (rung 71's gate) |
/// | `sqrt` differs BIT-WISE from the real-only spelling | **91** |
/// | the RETURNED `zeta` differs | **1** |
///
/// So the divergence reaches 90 shipped calls and changes nothing on any of them: `p.re >= 0`
/// throughout, which confines it to the imaginary component's zero, and [`c_div`] washes that out
/// of the `.real` the reader takes. Had any of the 90 carried `p.re < 0` the same `copysign` would
/// have flipped the sign of a NON-zero component instead — which is why the `re < 0` count is
/// measured here and not reasoned about. `tests/porting_rules.rs` RULE 4 holds both halves.
///
/// Restricted to normal magnitudes, exactly as [`csqrt_real`] declares: CPython's subnormal
/// rescaling branch (`CM_SCALE_UP`/`CM_SCALE_DOWN`) is not reproduced, and `p` here is a product
/// of roots of order 1e1…1e4.
pub fn csqrt(z: C64) -> C64 {
    // CPython's own early return, and it PRESERVES the sign of the zero imaginary part.
    if z.re == 0.0 && z.im == 0.0 {
        return C64 { re: 0.0, im: z.im };
    }
    let ax = z.re.abs() / 8.0;
    let ay = z.im.abs();
    let s = 2.0 * (ax + ax.hypot(ay / 8.0)).sqrt();
    let d = ay / (2.0 * s);
    if z.re >= 0.0 {
        C64 { re: s, im: d.copysign(z.im) }
    } else {
        C64 { re: d, im: s.copysign(z.im) }
    }
}

/// Python's `sorted(roots, key=abs)` — **STABLE**, which is load-bearing: the real branch returns
/// a conjugate pair with equal magnitudes, and an unstable sort would swap them.
///
/// `pub(crate)` since slice AC: rung 70's `_zeta_pair`, `split_modes` and `split_floor` all call
/// the SAME `sorted(roots, key=abs)`, so re-spelling a stable sort in `cross_split.rs` would be a
/// duplication the source does not make.
pub(crate) fn sorted_by_abs(roots: [C64; 3]) -> [C64; 3] {
    let mut out = roots;
    out.sort_by(|a, b| a.abs().partial_cmp(&b.abs()).expect("the roots here are finite"));
    out
}

/// RUNG 69's `_invariants` — the characteristic polynomial's three coefficients from the six
/// cross-gains and the three clocks.
///
/// `J = D A` with `D = diag(1/tau_i)` and `A` the gain matrix with `-1` on the diagonal, so
/// `c2 = tr J`, `c1 = sum of the three 2x2 principal minors`, `c0 = det J`.
///
/// **`c1` IS THE ONE KEY IN THIS SLICE WHOSE INTERPRETERS DISAGREE.** Python spells both `c2` and
/// `c1` as `sum(...)` over a three-element generator, and § 5.26 (i) measured CPython's
/// Neumaier-compensated `sum` diverging from the naive fold on **23 of 256** instances of `c1`
/// while `c2` agrees on all 256. Length is not the discriminator (slice AA's explanation, refuted)
/// and neither is cancellation (my replacement, refuted by the same probe): whether the
/// compensation survives the final rounding is a bit-pattern property of the particular summands.
/// PyPy folds left from `0.0`, which is what this is, and adding `0.0` first is exact for every
/// non-`-0.0` first term — the diagonal is `-1/tau_i`, so it never is one.
pub fn invariants(gg: &TripleGains, taus: (f64, f64, f64)) -> (f64, f64, f64) {
    let a = [[-1.0, gg.r_q, gg.r_v], [gg.c_g, -1.0, gg.c_v], [gg.v_g, gg.v_q, -1.0]];
    let td = [taus.0, taus.1, taus.2];
    let mut j = [[0.0f64; 3]; 3];
    for (i, row) in j.iter_mut().enumerate() {
        for (k, cell) in row.iter_mut().enumerate() {
            *cell = a[i][k] / td[i];
        }
    }
    let c2 = j[0][0] + j[1][1] + j[2][2];
    let c1 = (j[0][0] * j[1][1] - j[0][1] * j[1][0])
        + (j[0][0] * j[2][2] - j[0][2] * j[2][0])
        + (j[1][1] * j[2][2] - j[1][2] * j[2][1]);
    let c0 = j[0][0] * (j[1][1] * j[2][2] - j[1][2] * j[2][1])
        - j[0][1] * (j[1][0] * j[2][2] - j[1][2] * j[2][0])
        + j[0][2] * (j[1][0] * j[2][1] - j[1][1] * j[2][0]);
    (c2, c1, c0)
}

// ---------------------------------------------------------------------------------------------
// THE SIX READERS — and THREE of them march a machine after the scope has CLOSED
// ---------------------------------------------------------------------------------------------
//
// § 5.26.1 (j), registered before a line of this section was written:
//
// * `reference_bill` runs `triple_bill` ENTIRELY inside the scope, so `_ref` may be read
//   throughout.
// * `reference_gains`, `reference_modes` and `ring_visibility` build the rig inside the scope and
//   march the returned machine AFTER it has closed.
//
// So the reference must be CONSUMED AT CONSTRUCTION — baked into which limiter `_triple_rig`
// armed — and never re-read from the carrier by a downstream reader. Every `RefScope` below is
// therefore scoped to the `triple_rig` call alone and not to the march, which makes the
// requirement structural rather than a discipline: a reader that wanted the carrier later would
// have to widen a `let` and say so.

/// One sampled point of [`reference_gains`] — the SAME point read under BOTH references.
#[derive(Clone, Debug)]
pub struct RefGainsRow {
    pub s: f64,
    /// The INCIDENCE rig, on the shared manifold — the plant that was marched.
    pub inc: TripleGains,
    /// The `phi` rig, at the INCIDENCE march's own points, so the two references are differenced
    /// on ONE trajectory rather than on two.
    pub phi: TripleGains,
    /// The incidence rig at the LIVE marched `v` — the alternative base point, REPORTED and never
    /// gated on (§ 0.3 degrades `pair_RC` to 0.94–0.98 there).
    pub own: TripleGains,
    /// `(pair_RV + pair_CV)/2` — the ONE scalar that sets the split, the cyclic product and the
    /// damping floor.
    pub k: f64,
    /// `|pair_RV - pair_CV| / |k|` — the two split pairs taking the SAME value is a MEASUREMENT,
    /// so its residual is a key.
    pub pair_gap: f64,
    pub v_base: f64,
}

/// RUNG 69's `reference_gains` return.
#[derive(Clone, Debug)]
pub struct ReferenceGains {
    pub n_riding: usize,
    pub n_sampled: usize,
    pub rows: Vec<RefGainsRow>,
    /// DISCLOSED, never a silent truncation — `(s, inc off-regime, phi off-regime)`.
    pub skipped: Vec<(f64, Vec<&'static str>, Vec<&'static str>)>,
    pub s_window: Option<(f64, f64)>,
    pub k_range: (Option<f64>, Option<f64>),
    pub worst_rc_inc: Option<f64>,
    pub worst_rc_phi: Option<f64>,
    pub worst_pair_gap: Option<f64>,
    pub worst_rc_own: Option<f64>,
}

/// Python's `max(gen, default=None)` / `min(gen, default=None)` over a possibly-empty iterator.
pub(crate) fn opt_fold(mut it: impl Iterator<Item = f64>, f: fn(f64, f64) -> f64) -> Option<f64> {
    let first = it.next()?;
    Some(it.fold(first, f))
}

/// RUNG 69 § 1 — **the six cross-gains under BOTH references at the SAME base points.**
///
/// THE INSTRUMENT IS THE SPLIT, not any single scalar. `pair_RC` — the two loops that still share
/// `phi` — must stay at 1 while `pair_RV` and `pair_CV` BOTH move to `k`, so **which pairs keep
/// rung 66's identity reads off WHICH LOOPS SHARE A CONSTRAINT.** `cyclic` is reported because
/// rung 68 quotes it, and `k` because it sets the split, the cyclic product AND the damping floor.
///
/// The march is the INCIDENCE one (the new plant); the rung-68 rig is evaluated at ITS points, so
/// the two references are differenced on ONE trajectory rather than on two.
pub fn reference_gains(
    core: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, sm: f64,
    arm: &TripleRigArm, every: usize,
) -> ReferenceGains {
    let a = TripleRigArm { sm, ..*arm };
    let (m_i, surge, lag) = core.triple_rig(&a);
    // The `phi` rig, built INSIDE the scope and used outside it — the whole content of § (j).
    let m_p = {
        let _r = RefScope::set(&core.fuel.inner, Some("phi"));
        core.triple_rig(&a).0
    };
    let leg = StatorLeg { accel: None, surge, tt4_max: None };
    let (traj, _) = m_i.stator_march_scoped(
        flight, ramp, None, &leg, &MarchScope { lag, ..MarchScope::DEFAULT });
    let b_max = m_i.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding(&traj, b_max);
    let sampled: Vec<&FuelPoint> = pts.iter().step_by(every.max(1)).collect();
    let (mut rows, mut skipped) = (Vec::new(), Vec::new());
    for p in &sampled {
        // Python evaluates all THREE before inspecting any regime, so the closure-call count a
        // counter can read does not depend on which arm was off.
        let inc = triple_gains_at(&m_i, flight, p, None, leg.surge.as_ref(),
                                  1e-7, 1e-5, 1e-4, true, 0.0, true)
            .expect("rung-69's gains march does not abort");
        let own = triple_gains_at(&m_i, flight, p, None, leg.surge.as_ref(),
                                  1e-7, 1e-5, 1e-4, false, 0.0, true)
            .expect("rung-69's gains march does not abort");
        let phi = triple_gains_at(&m_p, flight, p, None, leg.surge.as_ref(),
                                  1e-7, 1e-5, 1e-4, true, 0.0, true)
            .expect("rung-69's gains march does not abort");
        if !(inc.interior && phi.interior) {
            skipped.push((p.s, inc.off_regime.clone(), phi.off_regime.clone()));
            continue;
        }
        let k = 0.5 * (inc.pair_rv + inc.pair_cv);
        rows.push(RefGainsRow {
            s: p.s,
            pair_gap: (inc.pair_rv - inc.pair_cv).abs() / k.abs(),
            v_base: inc.v_base,
            k,
            inc,
            own,
            phi,
        });
    }
    ReferenceGains {
        n_riding: pts.len(),
        n_sampled: sampled.len(),
        s_window: if pts.is_empty() { None }
                  else { Some((pts[0].s, pts[pts.len() - 1].s)) },
        k_range: (opt_fold(rows.iter().map(|x| x.k), f64::min),
                  opt_fold(rows.iter().map(|x| x.k), f64::max)),
        worst_rc_inc: opt_fold(rows.iter().map(|x| (x.inc.pair_rc - 1.0).abs()), f64::max),
        worst_rc_phi: opt_fold(rows.iter().map(|x| (x.phi.pair_rc - 1.0).abs()), f64::max),
        worst_pair_gap: opt_fold(rows.iter().map(|x| x.pair_gap), f64::max),
        worst_rc_own: opt_fold(rows.iter().filter(|x| x.own.interior)
                                   .map(|x| (x.own.pair_rc - 1.0).abs()), f64::max),
        rows,
        skipped,
    }
}

/// One sampled point's spectrum under ONE reference.
#[derive(Clone, Debug)]
pub struct RefModesRow {
    pub s: f64,
    /// `(1-k)(1/(tau_g tau_s) + 1/(tau_q tau_s))` — **THE DISCRIMINATOR**, ~0 under `phi` and
    /// decisively non-zero here, and NOT the invariant rung 68 used.
    pub c1: f64,
    /// `det J` — ZERO under BOTH references, because the two `phi` loops keep exactly parallel
    /// rows whatever the third one watches. **BLIND to the split.**
    pub c0: f64,
    /// `tr J` — the ODE's own diagonal, not a measurement.
    pub c2: f64,
    pub k: f64,
    pub pair_rc: f64,
    pub cyclic: f64,
    pub roots: [C64; 3],
    /// `-Re(dom)/|dom|`, `None` only if the dominant root is exactly zero.
    pub zeta: Option<f64>,
    pub complex_pair: bool,
    /// Roots under `1e-4 * rate` — **`n - m`**: TWO under `phi` (rung 68), ONE here. The rung.
    pub n_zero: usize,
    pub worst_zero: f64,
    /// Both invariants RELATIVE to the rate sum's own power, because "zero" without its scale is
    /// not a measurement either.
    pub c1_rel: f64,
    pub c0_rel: f64,
}

/// One reference's arm of [`reference_modes`].
#[derive(Clone, Debug)]
pub struct RefModesArm {
    pub rate_sum: f64,
    pub n: usize,
    pub n_sampled: usize,
    /// DISCLOSED below, never a silent truncation.
    pub skipped: usize,
    pub rows: Vec<RefModesRow>,
    /// The DISTINCT `n_zero` values, sorted — Python's `sorted({...})`.
    pub zeros: Vec<usize>,
    pub max_c0_rel: Option<f64>,
    pub min_c1_rel: Option<f64>,
    pub all_complex: Option<bool>,
    pub zeta_range: (Option<f64>, Option<f64>),
}

/// One clock triple, both references.
#[derive(Clone, Debug)]
pub struct RefModesClock {
    /// `(tau_att, tau_v, tau_s)` — the `(g, q, v)` order of the state vector, which is NOT the
    /// order the clock grid is written in.
    pub taus: (f64, f64, f64),
    pub inc: RefModesArm,
    pub phi: RefModesArm,
}

impl RefModesClock {
    /// The two arms in Python's own key order, for a reader that iterates.
    pub fn refs(&self) -> [(&'static str, &RefModesArm); 2] {
        [("inc", &self.inc), ("phi", &self.phi)]
    }
}

/// RUNG 69's `reference_modes` return.
#[derive(Clone, Debug)]
pub struct ReferenceModes {
    pub clocks: Vec<(f64, f64, f64)>,
    pub ds: f64,
    pub arms: Vec<RefModesClock>,
}

/// RUNG 69 § 1's SPECTRUM under BOTH references, on the shipped closures, across a clock grid.
///
/// THE THREE OBSERVABLES DO NOT CARRY THE SAME CONTENT: `zeros` is `n - m` and is the rung; `c0`
/// is `det J` and is **blind** to the split; `c1` is the discriminator, and it is not the
/// invariant rung 68 used. A reader that inherited rung 68's determinant test would report rank
/// one and see nothing.
#[allow(clippy::too_many_arguments)]
pub fn reference_modes(
    core: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, sm: f64,
    clocks: &[(f64, f64, f64)], v_max: f64, tau_rel_mult: f64, every: usize,
) -> ReferenceModes {
    let mut arms = Vec::new();
    for &(tau_v, tau_att, tau_s) in clocks {
        let taus = (tau_att, tau_v, tau_s);
        let mut built: Vec<RefModesArm> = Vec::new();
        for reference in ["inc", "phi"] {
            let a = TripleRigArm { sm, tau: tau_v, tau_s, v_max, tau_att,
                                   tau_rel: tau_rel_mult * tau_att, ..TripleRigArm::default() };
            let (m, surge, lag) = {
                let _r = RefScope::set(&core.fuel.inner, Some(reference));
                core.triple_rig(&a)
            };
            let leg = StatorLeg { accel: None, surge, tt4_max: None };
            let (traj, _) = m.stator_march_scoped(
                flight, ramp, None, &leg, &MarchScope { lag, ..MarchScope::DEFAULT });
            let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
            let pts = riding(&traj, b_max);
            // Python's `sum(1.0 / t for t in taus)` — a three-term LEFT FOLD in the state
            // vector's own order; probe 6 measured this site identical on both interpreters.
            let rate = 1.0 / taus.0 + 1.0 / taus.1 + 1.0 / taus.2;
            let sampled: Vec<&FuelPoint> = pts.iter().step_by(every.max(1)).collect();
            let (mut rows, mut skipped) = (Vec::new(), 0usize);
            for p in &sampled {
                let gg = triple_gains_at(&m, flight, p, None, leg.surge.as_ref(),
                                         1e-7, 1e-5, 1e-4, true, 0.0, true)
                    .expect("rung-69's spectrum march does not abort");
                if !gg.interior {
                    skipped += 1;
                    continue;
                }
                let (c2, c1, c0) = invariants(&gg, taus);
                let roots = cubic_roots_c(c2, c1, c0);
                let nz = sorted_by_abs(roots);
                let dom = nz[2];
                rows.push(RefModesRow {
                    s: p.s, c1, c0, c2,
                    k: 0.5 * (gg.pair_rv + gg.pair_cv),
                    pair_rc: gg.pair_rc,
                    cyclic: gg.cyclic,
                    roots,
                    zeta: if dom.abs() > 0.0 { Some(-dom.re / dom.abs()) } else { None },
                    complex_pair: dom.im.abs() > 1e-6 * dom.abs(),
                    n_zero: roots.iter().filter(|x| x.abs() < 1e-4 * rate).count(),
                    worst_zero: nz[0].abs(),
                    // `rate ** 2` MULTIPLIES and `rate ** 3` calls `pow` — PyPy's JIT rewrites
                    // the square and not the cube, so reproducing PyPy means doing the same
                    // (`tests/porting_rules.rs` RULE 2).
                    c1_rel: c1.abs() / (rate * rate),
                    c0_rel: c0.abs() / powp(rate, 3.0),
                });
            }
            let mut zeros: Vec<usize> = rows.iter().map(|x| x.n_zero).collect();
            zeros.sort_unstable();
            zeros.dedup();
            built.push(RefModesArm {
                rate_sum: -rate,
                n: pts.len(),
                n_sampled: sampled.len(),
                skipped,
                zeros,
                max_c0_rel: opt_fold(rows.iter().map(|x| x.c0_rel), f64::max),
                min_c1_rel: opt_fold(rows.iter().map(|x| x.c1_rel), f64::min),
                all_complex: if rows.is_empty() { None }
                             else { Some(rows.iter().all(|x| x.complex_pair)) },
                zeta_range: (opt_fold(rows.iter().filter_map(|x| x.zeta), f64::min),
                             opt_fold(rows.iter().filter_map(|x| x.zeta), f64::max)),
                rows,
            });
        }
        let phi = built.pop().expect("two arms were pushed");
        let inc = built.pop().expect("two arms were pushed");
        arms.push(RefModesClock { taus, inc, phi });
    }
    ReferenceModes { clocks: clocks.to_vec(), ds: ramp.ds, arms }
}

/// The live half of a [`DampingRow`] — present only when the mid-trajectory point was interior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DampingLive {
    pub s: f64,
    pub k: f64,
    /// `A = 1/tau_g + 1/tau_q` — the two loops that still share `phi`.
    pub a: f64,
    /// `z = 1/tau_s` — the split loop.
    pub z: f64,
    pub a_over_z: f64,
    /// `A z (1-k)` — the rank-2 block's determinant, i.e. `lam1 lam2`.
    pub det2: f64,
    pub zeta_pred: f64,
    /// The SHIPPED cubic's own dominant root, not the closed form — the two are reported side by
    /// side rather than one being asserted from the other.
    pub zeta: f64,
    /// `1/sqrt(1-k)` — the AM-GM floor, and **bandwidth-independent**.
    pub floor: f64,
    pub modulus: f64,
    pub mod_pred: f64,
    pub rate_sum: f64,
    pub complex_pair: bool,
}

/// One grid point of [`damping_floor`] — Python's row dict, whose KEYS differ by arm.
#[derive(Clone, Debug)]
pub struct DampingRow {
    pub taus: (f64, f64, f64),
    pub n: usize,
    /// Non-empty exactly when the mid point was off-regime, which is the arm Python spells by
    /// putting an `off_regime` key in and leaving `zeta` out.
    pub off_regime: Vec<&'static str>,
    pub live: Option<DampingLive>,
}

/// RUNG 69's `damping_floor` return.
#[derive(Clone, Debug)]
pub struct DampingFloor {
    pub rows: Vec<DampingRow>,
    pub holds: bool,
    /// The `zeta/floor`-minimal live row — Python's `min(..., key=...)`, which returns the FIRST
    /// minimum.
    pub tightest: Option<DampingLive>,
    pub worst_pred_err: Option<f64>,
}

/// RUNG 69 § 3 — **`zeta >= 1/sqrt(1-k)` OVER EVERY BANDWIDTH, with equality at `A = z`.**
///
/// The gains do not depend on the clocks at all — `R`, `C` and `V` are control LAWS, and the
/// clocks enter only through `D = diag(1/tau_i)`. So the honest instrument measures the gains once
/// per grid point ON THAT POINT'S OWN MARCH and reports both the closed-form `zeta` and the
/// shipped cubic's own dominant root, rather than pretending each clock arm is an independent
/// measurement of `k`.
///
/// **NO `_with_ref` HERE** — this reader runs on whatever the machine is armed with, which on a
/// rung-69 object is the incidence loop.
#[allow(clippy::too_many_arguments)]
pub fn damping_floor(
    core: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, sm: f64,
    grid: &[(f64, f64, f64)], v_max: f64, tau_rel_mult: f64,
) -> DampingFloor {
    let mut rows = Vec::new();
    for &(tau_v, tau_att, tau_s) in grid {
        let taus = (tau_att, tau_v, tau_s);
        let a_arm = TripleRigArm { sm, tau: tau_v, tau_s, v_max, tau_att,
                                   tau_rel: tau_rel_mult * tau_att, ..TripleRigArm::default() };
        let (m, surge, lag) = core.triple_rig(&a_arm);
        let leg = StatorLeg { accel: None, surge, tt4_max: None };
        let (traj, _) = m.stator_march_scoped(
            flight, ramp, None, &leg, &MarchScope { lag, ..MarchScope::DEFAULT });
        let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
        let pts = riding(&traj, b_max);
        if pts.is_empty() {
            rows.push(DampingRow { taus, n: 0, off_regime: Vec::new(), live: None });
            continue;
        }
        let p = pts[pts.len() / 2];
        let gg = triple_gains_at(&m, flight, &p, None, leg.surge.as_ref(),
                                 1e-7, 1e-5, 1e-4, true, 0.0, true)
            .expect("rung-69's damping march does not abort");
        if !gg.interior {
            rows.push(DampingRow { taus, n: pts.len(), off_regime: gg.off_regime, live: None });
            continue;
        }
        let k = 0.5 * (gg.pair_rv + gg.pair_cv);
        let (a, z) = (1.0 / tau_att + 1.0 / tau_v, 1.0 / tau_s);
        let det2 = a * z * (1.0 - k);
        let (c2, c1, c0) = invariants(&gg, taus);
        let dom = sorted_by_abs(cubic_roots_c(c2, c1, c0))[2];
        rows.push(DampingRow {
            taus, n: pts.len(), off_regime: Vec::new(),
            live: Some(DampingLive {
                s: p.s, k, a, z, a_over_z: a / z, det2,
                zeta_pred: (a + z) / (2.0 * powp(det2, 0.5)),
                zeta: -dom.re / dom.abs(),
                floor: powp(1.0 - k, -0.5),
                modulus: dom.abs(),
                mod_pred: powp(det2, 0.5),
                rate_sum: a + z,
                complex_pair: dom.im.abs() > 1e-6 * dom.abs(),
            }),
        });
    }
    let live: Vec<DampingLive> = rows.iter().filter_map(|x| x.live).collect();
    let mut tightest: Option<DampingLive> = None;
    for x in &live {
        // Python's `min(..., key=...)` keeps the FIRST minimum, so the comparison is STRICT.
        if tightest.is_none_or(|b| x.zeta / x.floor < b.zeta / b.floor) {
            tightest = Some(*x);
        }
    }
    DampingFloor {
        holds: live.iter().all(|x| x.zeta >= x.floor - 1e-9),
        worst_pred_err: opt_fold(live.iter().map(|x| (x.zeta / x.zeta_pred - 1.0).abs()),
                                 f64::max),
        tightest,
        rows,
    }
}

/// One sampled point of [`rk4_margin`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rk4MarginRow {
    pub s: f64,
    pub modulus: f64,
    pub k: f64,
    /// `|lam| / sum(1/tau)` — the ratio the derivation bounds.
    pub ratio: f64,
    /// `sqrt(1-k)/2` — that bound, which must stay below 1 for the inherited constant to be
    /// conservative.
    pub bound: f64,
}

/// RUNG 69's `rk4_margin` return.
#[derive(Clone, Debug)]
pub struct Rk4Margin {
    pub rate_sum: f64,
    /// The number of INTERIOR rows, not the number of riding points.
    pub n: usize,
    pub rows: Vec<Rk4MarginRow>,
    pub max_mod: Option<f64>,
    pub max_ratio: Option<f64>,
    pub max_bound: Option<f64>,
    pub ds_lambda: f64,
}

/// **THE GUARD, MEASURED AGAINST THE PLANT rather than trusted.**
///
/// [`r69_rk4_floor`] keeps rung 68's constant on a DIFFERENT argument — the dominant root is now a
/// complex pair — so what must be checked is the ratio the derivation bounds,
/// `|lam| / sum(1/tau) <= sqrt(1-k)/2`, and that it stays below 1. Rung 65 published a retraction
/// for exactly the failure mode of a trusted stability argument, which is why this reader exists
/// at all rather than the inequality being asserted.
#[allow(clippy::too_many_arguments)]
pub fn rk4_margin(
    core: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, sm: f64,
    arm: &TripleRigArm, every: usize,
) -> Rk4Margin {
    let a_arm = TripleRigArm { sm, ..*arm };
    let (m, surge, lag) = core.triple_rig(&a_arm);
    let leg = StatorLeg { accel: None, surge, tt4_max: None };
    let (traj, _) = m.stator_march_scoped(
        flight, ramp, None, &leg, &MarchScope { lag, ..MarchScope::DEFAULT });
    let b_max = m.fuel.inner.lever.lim.expect("the rig arms a valve").b_max;
    let pts = riding(&traj, b_max);
    let taus = (a_arm.tau_att, a_arm.tau, a_arm.tau_s);
    let rate = 1.0 / taus.0 + 1.0 / taus.1 + 1.0 / taus.2;
    let mut rows = Vec::new();
    for p in pts.iter().step_by(every.max(1)) {
        let gg = triple_gains_at(&m, flight, p, None, leg.surge.as_ref(),
                                 1e-7, 1e-5, 1e-4, true, 0.0, true)
            .expect("rung-69's rk4 march does not abort");
        if !gg.interior {
            continue;
        }
        let (c2, c1, c0) = invariants(&gg, taus);
        let dom = sorted_by_abs(cubic_roots_c(c2, c1, c0))[2];
        let k = 0.5 * (gg.pair_rv + gg.pair_cv);
        rows.push(Rk4MarginRow { s: p.s, modulus: dom.abs(), k, ratio: dom.abs() / rate,
                                 bound: powp(1.0 - k, 0.5) / 2.0 });
    }
    Rk4Margin {
        rate_sum: rate,
        n: rows.len(),
        max_mod: opt_fold(rows.iter().map(|x| x.modulus), f64::max),
        max_ratio: opt_fold(rows.iter().map(|x| x.ratio), f64::max),
        max_bound: opt_fold(rows.iter().map(|x| x.bound), f64::max),
        // Python's `max(gen, default=0.0)` — a ZERO fallback here, not `None`.
        ds_lambda: ramp.ds * opt_fold(rows.iter().map(|x| x.modulus), f64::max).unwrap_or(0.0),
        rows,
    }
}

/// One reference's stator credit, all four ways [`reference_bill`] quotes it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatorCredit {
    /// The `S` cell, against the `phi` wall.
    pub alone: f64,
    /// The `S` cell, against the INCIDENCE wall — the one the stator cannot move.
    pub alone_inc: f64,
    /// `FVS - FV`, against `phi`.
    pub marginal: f64,
    pub marginal_inc: f64,
}

/// RUNG 69's `reference_bill` return.
#[derive(Clone, Debug)]
pub struct ReferenceBill {
    pub inc: TripleBill,
    pub phi: TripleBill,
    /// The four STATOR-FREE cells, `(inc, phi)` each — identical BY CONSTRUCTION, and recomputed
    /// so that any drift shows up in a cell that CANNOT have one.
    pub common: Vec<(&'static str, (f64, f64))>,
    pub common_max_rel: f64,
    pub stator_credit_inc: StatorCredit,
    pub stator_credit_phi: StatorCredit,
    /// `(inc, phi)`.
    pub delivered: (f64, f64),
    pub delivered_inc: (f64, f64),
}

/// RUNG 69 § 4 — **rung 68's 8-cell ledger run TWICE, once per reference, one rig, both walls.**
///
/// The `bare`, `F`, `V` and `FV` cells carry no stator and are therefore IDENTICAL between the two
/// references by construction; they are recomputed rather than shared so that any drift would show
/// up as a difference in a cell that CANNOT have one — a free check on the rig, and rung 63's
/// lesson about differenceable cells.
///
/// **AND THAT IS EXACTLY WHY `common_max_rel` IS NOT THE KEY THAT WOULD CATCH A LOST REFERENCE**
/// (§ 5.26.1 (j)): those four cells agree whichever arm ran. The discriminating keys are the ones
/// that MUST differ — `pair_RV`, `pair_CV`, `c1`, `zeros` — and they live in the other readers.
///
/// Unlike the three readers above, `triple_bill` runs ENTIRELY inside the scope, so the guard here
/// spans the whole call.
pub fn reference_bill(
    core: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, sm: f64,
    arm: &TripleRigArm,
) -> ReferenceBill {
    let inc = {
        let _r = RefScope::set(&core.fuel.inner, Some("inc"));
        triple_bill(core, flight, ramp, sm, arm)
    };
    let phi = {
        let _r = RefScope::set(&core.fuel.inner, Some("phi"));
        triple_bill(core, flight, ramp, sm, arm)
    };
    let common: Vec<(&'static str, (f64, f64))> = ["bare", "F", "V", "FV"]
        .iter()
        .map(|c| (*c, (inc.cell(c).i, phi.cell(c).i)))
        .collect();
    let credit = |b: &TripleBill| StatorCredit {
        alone: b.cell("S").credit,
        alone_inc: b.cell("S").credit_inc,
        marginal: b.marginal.2,
        marginal_inc: b.marginal_incidence.2,
    };
    ReferenceBill {
        common_max_rel: opt_fold(common.iter().map(|(_, (a, b))| (a / b - 1.0).abs()), f64::max)
            .expect("the four stator-free cells"),
        stator_credit_inc: credit(&inc),
        stator_credit_phi: credit(&phi),
        delivered: (inc.delivered, phi.delivered),
        delivered_inc: (inc.cell("FVS").credit_inc, phi.cell("FVS").credit_inc),
        common,
        inc,
        phi,
    }
}

/// One displacement arm of [`ring_visibility`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RingArm {
    pub n: usize,
    pub n_riding: usize,
    /// The tracking error at the FIRST riding point.
    pub e0: f64,
    /// Sign changes of `e = v - v_cmd` over the riding points, zeros skipped.
    pub crossings: usize,
    /// `|e0| / |v0|` — **WHAT FRACTION OF THE DISPLACEMENT SURVIVES AS AN ERROR AT ALL.** Under a
    /// SHARED constraint the other loops absorb it EXACTLY — the `s = 0` fixed points are a family
    /// and a displaced stator just selects a different member — so there is nothing left to ring.
    /// Under the split they cannot.
    pub survives: Option<f64>,
    /// `max(-e/e0)` — the largest counter-swing as a fraction of the initial error.
    pub counter: Option<f64>,
    pub v_range: (f64, f64),
}

/// One reference's pair of arms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RingRef {
    pub base: RingArm,
    pub displaced: RingArm,
}

/// RUNG 69's `ring_visibility` return.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RingVisibility {
    pub inc: RingRef,
    /// Rung 68's `phi` reference on the same rig, as a **NEGATIVE CONTROL**: its spectrum is
    /// provably real, so any crossing it shows is not a ring and sets the count's noise floor.
    pub phi: RingRef,
}

/// **IS THE MODE OBSERVABLE?** — rung 67's question, asked of a different mechanism.
///
/// § 3 says `zeta >= 1/sqrt(1-k) ~ 0.58`, which allows AT MOST ONE overshoot of ~11 % of a
/// displacement. So the probe is the textbook one: DISPLACE the stator's initial position off its
/// own command (rung 68's `v0`, an isolation instrument) and count ZERO CROSSINGS of the tracking
/// error while the loop is RIDING.
///
/// THREE THINGS MAKE IT AN INSTRUMENT RATHER THAN A PLOT: the `phi` reference runs on the same rig
/// as a negative control; the ERROR and not the position is the signal, because `v_cmd` moves
/// under the ramp; and the count is restricted to RIDING points, because the band is ONE-SIDED and
/// its dormant stop would CLAMP an undershoot away. That clamp is DISCLOSED rather than worked
/// around — an unobservable-because-clamped mode is still unobservable, but it is a different
/// sentence from an unobservable-because-damped one.
///
/// **THE DISPLACEMENT'S SIGN FOLLOWS THE BAND**: `+disp` under `inc` and `-disp` under `phi`, since
/// the two bands are mirrors and a displacement out of the band would be refused by
/// [`r69_check_v0`].
#[allow(clippy::too_many_arguments)]
pub fn ring_visibility(
    core: &ScheduledStatorCore, flight: &FlightCondition, ramp: &Ramp, sm: f64,
    arm: &TripleRigArm, disp: f64,
) -> RingVisibility {
    let mut out: Vec<RingRef> = Vec::new();
    for reference in ["inc", "phi"] {
        let a = TripleRigArm { sm, ..*arm };
        let (m, surge, lag) = {
            let _r = RefScope::set(&core.fuel.inner, Some(reference));
            core.triple_rig(&a)
        };
        let leg = StatorLeg { accel: None, surge, tt4_max: None };
        let mut arms: Vec<RingArm> = Vec::new();
        for v0 in [None, Some(if reference == "inc" { disp } else { -disp })] {
            let (traj, _) = m.stator_march_scoped(
                flight, ramp, None, &leg, &MarchScope { lag, v0, ..MarchScope::DEFAULT });
            // Python filters on `p.get("v_regime") == "riding"` — the RAW label, not `_riding`'s
            // three-loop filter, because the question is about the STATOR's own tracking.
            let rid: Vec<&FuelPoint> = traj.iter()
                // SLICE AD (3 of 13): a `matches!` is a wildcard by construction, and
                // without the second pattern every rung-72 point is filtered OUT -- the
                // reader would return an EMPTY riding set and report perfect tracking.
                .filter(|p| matches!(p.extra,
                                     PointExtra::Triple { v_regime: Regime::Riding, .. }
                                     | PointExtra::Shared {
                                         v_regime: Some(Regime::Riding), .. }
                                     // SLICE AF (11 of 31).
                                     | PointExtra::Demand {
                                         v_regime: Some(Regime::Riding), .. }))
                .collect();
            let e: Vec<f64> = rid.iter().map(|p| match p.extra {
                PointExtra::Triple { v, v_cmd, .. }
                // SLICE AD (4 of 13): the filter above now admits rung 72, so this
                // `unreachable!` would BECOME reachable without the matching arm.
                | PointExtra::Shared { v, v_cmd, .. }
                // SLICE AF (12 of 31): the filter above now admits rung 74, so this
                // `unreachable!` would BECOME reachable without the matching arm.
                | PointExtra::Demand { v, v_cmd, .. } => v - v_cmd,
                _ => unreachable!("filtered to the five-state march"),
            }).collect();
            let nz: Vec<f64> = e.iter().copied().filter(|x| x.abs() > 1e-12).collect();
            let e0 = e.first().copied().unwrap_or(0.0);
            let big = e0.abs() > 1e-9;
            arms.push(RingArm {
                n: traj.len(),
                n_riding: rid.len(),
                e0,
                crossings: (1..nz.len()).filter(|&i| nz[i] * nz[i - 1] < 0.0).count(),
                // Python is `abs(e0)/abs(v0) if v0 else None`, and `if v0` is falsy for
                // **0.0** as well as for `None`. `v0.map(..)` would divide, handing back
                // `inf`/`NaN` where Python reports "no displacement". Unreachable at the
                // shipped `disp = 0.05` -- which is exactly why it is spelled rather than
                // left to a caller to discover.
                survives: v0.filter(|x| *x != 0.0).map(|x| e0.abs() / x.abs()),
                counter: if big { opt_fold(e.iter().map(|x| -x / e0), f64::max) } else { None },
                v_range: (opt_fold(traj.iter().map(v_at_point), f64::min)
                              .expect("rung-69's ring march is non-empty"),
                          opt_fold(traj.iter().map(v_at_point), f64::max)
                              .expect("rung-69's ring march is non-empty")),
            });
        }
        let displaced = arms.pop().expect("two arms were pushed");
        let base = arms.pop().expect("two arms were pushed");
        out.push(RingRef { base, displaced });
    }
    let phi = out.pop().expect("two references were pushed");
    let inc = out.pop().expect("two references were pushed");
    RingVisibility { inc, phi }
}
