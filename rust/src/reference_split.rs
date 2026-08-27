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
//! # What this module is, at STEP 1
//!
//! **The one added cell, the nine swapped cells' distinct function pointers, `_ref`'s carrier and
//! guard, `stator_inc` on the arming, and [`StatorIncidenceLimiter`]. Nothing is ported.** The
//! nine swapped bodies below PANIC with a message naming step 2, so a swap that step 2 forgets
//! cannot silently keep running rung 68's body.
//!
//! **TWO TENS APPEAR IN THIS SLICE AND THEY ARE DIFFERENT TENS**, so the addition is written out
//! rather than left for a reader to reconcile: **10 SWAPS** = the 9 cells overridden below +
//! `__init__`, which is not a cell (no shipped table carries a constructor hook — it ports as
//! [`build_reference_split_cascade`]'s four `assert!`s); **10 TABLE CELLS** = those same 9 + the
//! one this rung ADDS, [`TripleHooks::with_ref`]. `tests/slice_ab_cells.rs` holds the arithmetic
//! and the compiler holds the width.
//!
//! # WHY THAT IS THE STEP-1 GATE HERE, AND NOT THE CELL COUNT (§ 5.26 (ii))
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
//! | `_manifold_v` | 291 | — | **by PANIC**, on 122 |
//! | `_triple_rig` | 60 | — | **by PANIC**, on 60 |
//! | `at_lever` | 61 | — | by value: 31 calls carry `stator_inc` IN and 0 lose it |
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

use crate::bleed_transient::{LeverArm, LeverArming, LeverHooks};
use crate::engine::FlightCondition;
use crate::fuel_transient::{AsymmetricLag, Floor, FuelCloseState, FuelTransientHooks};
use crate::gas::Abort;
use crate::limited_bleed::Regime;
use crate::map::ComponentMap;
use crate::stator_transient::{
    ScheduledStatorCore, ScheduledStatorTransient, StatorTransientHooks,
};
use crate::three_loop::{StatorLegArm, TripleHooks, TripleRigArm};
use crate::two_spool::TwoSpoolEngine;
use crate::two_spool_transient::{TwoSpoolTransientCore, TwoSpoolTransientHooks};

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
    // GUARD C, hoisted — see the note above.
    assert!(arm.stator_inc.is_none() || !arm.stator.lp_disabled,
            "rung-69's incidence floor watches the LP, which a disabled LP spool does not have.");
    let built = ScheduledStatorTransient::with_ref_tables(
        design_engine, flight_design, mdot_design, map_lp, map_hp, rho, arm.stator,
        &R69_TWO, &R69_STATOR, &R69_FUEL, &R69,
        LeverArming { bleed: arm.bleed, sched: arm.bleed_sched, lim: arm.bleed_lim },
        &R69_TRIPLE, arm.stator_lim, arm.stator_inc);
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
// THE NINE SWAPPED CELLS — OPENED AT STEP 1, BODIES AT STEP 2
// ---------------------------------------------------------------------------------------------
//
// Each PANICS with a message naming itself, and `tests/slice_ab_cells.rs` reads those messages.
// That is the step-1 gate: nine distinct function pointers, none of them rung 68's.
//
// **A `todo!()` would have been the idiomatic spelling and it is the wrong one here.** The failure
// this slice actually risks is a swap left pointing at the parent, and `todo!()` and the parent's
// body are indistinguishable to a reader skimming the table — both are "something that compiles".
// A NAMED panic per cell makes the table's ten slots addressable by a gate, and step 2 replaces
// them one at a time with the panic count as its own progress bar.

/// The one string every unported cell below names itself with — so the gate can assert the SET of
/// open cells rather than nine unrelated messages.
const STEP2: &str = "rung-69 cell opened at slice AB step 1 and NOT YET PORTED";

fn r69_at_lever(_: &ScheduledStatorCore, _: &LeverArm) -> ScheduledStatorCore {
    panic!("{STEP2}: at_lever");
}

fn r69_stator_leg(_: &TwoSpoolTransientCore) -> Option<StatorLegArm> {
    panic!("{STEP2}: _stator_leg");
}

fn r69_lagged_stator(_: &TwoSpoolTransientCore) -> bool {
    panic!("{STEP2}: _lagged_stator");
}

fn r69_clamp_v(_: &TwoSpoolTransientCore, _: f64, _: &StatorLegArm) -> f64 {
    panic!("{STEP2}: _clamp_v");
}

fn r69_check_v0(_: &TwoSpoolTransientCore, _: f64, _: &StatorLegArm) {
    panic!("{STEP2}: _check_v0");
}

fn r69_rk4_floor(_: f64, _: f64, _: usize, _: f64) {
    panic!("{STEP2}: _rk4_floor");
}

fn r69_solve_v(
    _: &TwoSpoolTransientCore,
    _: &dyn Fn(f64) -> Result<FuelCloseState, Abort>,
) -> Result<(FuelCloseState, f64, Regime), Abort> {
    panic!("{STEP2}: _solve_v");
}

#[allow(clippy::too_many_arguments)]
fn r69_manifold_v(
    _: &ScheduledStatorCore,
    _: &FlightCondition,
    _: f64,
    _: f64,
    _: f64,
    _: f64,
    _: f64,
    _: &dyn Fn(f64, f64) -> Result<(f64, Regime), Abort>,
) -> Result<f64, Abort> {
    panic!("{STEP2}: _manifold_v");
}

fn r69_triple_rig(
    _: &ScheduledStatorCore,
    _: &TripleRigArm,
) -> (ScheduledStatorCore, Option<Floor>, Option<AsymmetricLag>) {
    panic!("{STEP2}: _triple_rig");
}

/// The nine names above, in the order [`R69_TRIPLE`] and [`R69`] list them — **the set a gate
/// asserts against, so "did step 2 port them all" is a count and not a reading.**
///
/// It shrinks to nothing by the end of step 2. Written here rather than in the test file for
/// [`Census68`](crate::three_loop::Census68)'s corrected lesson: a defence whose only reader is a
/// test can be deleted with the test and nobody notices.
pub const UNPORTED_AT_STEP1: [&str; 9] = [
    "at_lever", "_stator_leg", "_lagged_stator", "_clamp_v", "_check_v0", "_rk4_floor",
    "_solve_v", "_manifold_v", "_triple_rig",
];
