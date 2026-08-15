//! RUNGS 53 + 54 — THE VARIABLE STATOR, and the stator-row THROAT.
//!
//! Port of `turbojet/engine.py`'s `VariableStatorMatcher` (phase 5 slice M of
//! `docs/plans/todo-rust-port.md`, § 5.9). Its two `ComponentMap` channels live in
//! [`crate::map`] beside the fields they read; everything that needs a MATCHED point is here.
//!
//! # Why the two rungs are one module
//!
//! They are inseparable in the source and the port keeps that. Rung 54's `throat_margin`
//! EXTENDS rung 53's `stator_margin` row in place, and rung 54's `_schedule_root` is the
//! documented immune replacement for rung 53's `incidence_schedule` ladder — the two docstrings
//! cross-reference each other. Splitting them would put a method and its own correction in
//! different files.
//!
//! # RUNG 53 — a margin is a DISTANCE
//!
//! The stator setting `v = tan(alpha_1)` is the first lever in the project that MOVES THE SURGE
//! FLOOR. Both channels it drives are derived from the map's own constants ([`ComponentMap::psi`]
//! from `l`, [`ComponentMap::phi_surge_at`] from rungs 36/41's imposed floor read as an
//! incidence), so the rung adds **zero new constants**.
//!
//! The finding is that rungs 36–52 measured surge exposure as a distance in `phi` — and a
//! distance is only meaningful if the wall stands still. Move the wall and the currency becomes
//! COORDINATE-DEPENDENT: `M_phi = phi_op - phi_surge(v)` and `M_i = T_c - tan(beta_1)` vanish
//! together, so as a STALL TEST they are equivalent, but their DERIVATIVES in `v` can disagree in
//! sign. `T_c` is a property of the blade metal and stator-invariant, which is why the incidence
//! currency is the one that survives the lever.
//!
//! # RUNG 54 — THE POINT OF ENTRY IS: THERE ISN'T ONE
//!
//! `v` enters the steady solve through [`ComponentMap::solve_n`] alone (rung 53's P1) and the
//! throat enters NO solver, so the throat loading `X` is a post-hoc functional of the
//! ALREADY-SOLVED state. An upstream throat therefore cannot change the map from setting to
//! incidence — it can only remove settings from the feasible set. **BIND, NEVER RELIEVE.** Hence
//! every rung-54 method here is a pure read, and the reduce is an INVARIANCE OVER `C` (every
//! matched field bit-identical at EVERY capacity, at a MOVED stator), which is strictly stronger
//! than rung 53's identity at one setting.
//!
//! # Three shape decisions, each of which would be a silent divergence
//!
//! **1. `at_setting` REBUILDS, and it is a HOOK.** Python's sibling constructor re-invokes the
//! class constructor, which re-runs the design capture; this does the same rather than cloning a
//! core and swapping its two maps. The rebuild is not arithmetic ceremony: rung 55's override
//! constructs a `StageStack` FROM the moved map, so a descendant's sibling is genuinely
//! `v`-dependent, and a copy-and-swap in rung 53's body would leave slice N with a stack built at
//! the wrong setting. It is a hook for the same reason — overridden at THREE levels (53, 55, 61),
//! and reached through `self` by `stator_sweep`, `currency_split`, `incidence_schedule`, `_scan`
//! and `schedule_throat`. Hardcoding rung 53's body would compile, return numbers, and return the
//! WRONG ones under slices N and O.
//!
//! **2. THE THROAT ROW EXTENDS THE MARGIN ROW IN PLACE.** Python's `throat_margin` mutates rung
//! 53's returned dict and hands back the same object, so the row carries **16 keys without a
//! throat model and 19 with one**. Rebuilding it as one flat struct with the union of both field
//! sets would look like a cleanup and would destroy the only instrument that can see the capacity
//! branch at all — a float dump is structurally blind to a MISSING value (slice L's P9). The
//! nesting here — [`SpoolMargin::throat`] is `Option`, and inside it [`ThroatRead::choke`] is
//! `Option` — makes the two absences unwritable rather than merely unwritten.
//!
//! **3. THE FLOOR ASSERT AND THE BRACKET ASSERT ARE DIFFERENT ANIMALS.** [`ComponentMap::solve_n`]
//! took a fallible twin in slice M step 1 because `_scan` catches it 100 cells out of 100. Rung
//! 53's own `phi_s < phi_op` floor assert did NOT: measured over 560 clean calls (7 settings × 5
//! shapes × 4 throttles × 2 gases × 2 spools) it raises **0** times, so by the zero-firing rule it
//! stays an `assert!`. Two raise sites in one call chain, two different verdicts, each measured.
//!
//! [`ComponentMap::psi`]: crate::map::ComponentMap::psi
//! [`ComponentMap::solve_n`]: crate::map::ComponentMap::solve_n
//! [`ComponentMap::phi_surge_at`]: crate::map::ComponentMap::phi_surge_at

use crate::engine::FlightCondition;
use crate::gas::{powp, Abort};
use crate::map::ComponentMap;
use crate::two_spool::{Spool, TwoSpoolEngine, TwoSpoolMapCore, TwoSpoolMapResult, R39};

// =========================================================================================
// THE ROW — and the two absences a float dump cannot see
// =========================================================================================

/// Rung 53's per-spool reading: BOTH reference-free surge currencies at one operating point.
///
/// The two are equivalent as a STALL TEST (`m_phi > 0` ⟺ `m_i > 0`) and inequivalent as a
/// DISTANCE. That is the rung.
#[derive(Clone, Copy, Debug)]
pub struct SpoolMargin {
    /// This spool's stator setting.
    pub vsv: f64,
    /// Running-line flow coefficient at the matched point.
    pub phi_op: f64,
    /// Corrected speed at the matched point.
    pub n: f64,
    /// FACE-referred corrected flow, `phi_op * n` (design = 1).
    pub m: f64,
    /// The stall floor AT THIS SETTING — [`ComponentMap::phi_surge_at`], the LIVE wall.
    pub phi_surge: f64,
    /// The stall floor at the DESIGN setting — the `phi_surge` FIELD, which is the anchor rungs
    /// 36/41/44/45 read. Equal to `phi_surge` exactly when `vsv == 0`.
    pub phi_surge_design: f64,
    /// **CURRENCY A** — distance in `phi`. The wall MOVES with `v`.
    pub m_phi: f64,
    /// Rotor relative inlet angle at the operating point.
    pub tan_b1: f64,
    /// The critical angle — a property of the blade METAL, hence stator-INVARIANT.
    pub tan_b1_crit: f64,
    /// **CURRENCY B** — distance in incidence. The wall is the metal.
    pub m_i: f64,
    /// Pressure ratio at the operating point, via [`TwoSpoolMapCore::pi_c_spool`].
    pub pi_op: f64,
    /// Rung 41's constant-speed pressure-ratio margin, evaluated at the LIVE floor. Reported for
    /// definition-robustness: a third currency that could disagree with both.
    pub sm_n: f64,
    /// **RUNG 54's EXTENSION, PRESENT ONLY WHEN THE ROW CAME FROM
    /// [`throat_margin`](VariableStatorCore::throat_margin).** `None` from
    /// [`stator_margin`](VariableStatorCore::stator_margin) — 12 fields vs 16 vs 19, which is the
    /// discriminant § 5.9 (vi) exists to preserve.
    pub throat: Option<ThroatRead>,
}

/// Rung 54's throat read-offs, added to a margin row IN PLACE.
///
/// `c_min` is reported ALWAYS and needs no constant — that is how rung 54's claims stay free of
/// the one constant it adds. The three fields that DO need it live in [`Self::choke`].
#[derive(Clone, Copy, Debug)]
pub struct ThroatRead {
    /// `A_th(v)/A_th(0) = 1/sqrt(1+v^2)` — DERIVED, cascade cosine rule, EVEN in `v`.
    pub area: f64,
    /// `X(v) = m/area` — the throat-referred corrected flow.
    pub throat_loading: f64,
    /// `1/X` — the DERIVED threshold on `C`: the row chokes here iff `C >= c_min`. **The
    /// constant-free half of rung 54.**
    pub c_min: f64,
    /// The map's capacity fraction, echoed so a row states its own model.
    pub capacity: f64,
    /// Present only when `capacity > 0` — the three keys that need the throat model.
    pub choke: Option<ChokeRead>,
}

/// The three read-offs that exist only with a throat model (`C > 0`).
#[derive(Clone, Copy, Debug)]
pub struct ChokeRead {
    /// `M_c = 1 - C*X` — rung 54's THIRD reference-free currency.
    pub m_c: f64,
    /// `M_c <= 0`.
    pub choked: bool,
    /// The disclosed constant read physically, at `gamma = 1.4` — Python's default, which Rust
    /// has to spell at the call site.
    pub throat_mach_design: f64,
}

/// Both spools' rows at one operating point, plus the settings that produced them.
#[derive(Clone, Copy, Debug)]
pub struct StatorMargin {
    pub tt4: f64,
    pub vsv_lp: f64,
    pub vsv_hp: f64,
    pub lp: SpoolMargin,
    pub hp: SpoolMargin,
}

impl StatorMargin {
    pub fn spool(&self, spool: Spool) -> &SpoolMargin {
        match spool {
            Spool::Lp => &self.lp,
            Spool::Hp => &self.hp,
        }
    }
}

/// One row of [`stator_sweep`](VariableStatorCore::stator_sweep) — BOTH spools at each setting,
/// so the row simultaneously carries rung 53's P5 arrow measurement on the spool that did NOT
/// move.
#[derive(Clone, Copy, Debug)]
pub struct SweepRow {
    pub vsv: f64,
    pub swept: Spool,
    pub lp: SpoolMargin,
    pub hp: SpoolMargin,
}

// =========================================================================================
// THE HOOK TABLE
// =========================================================================================

/// The virtual table rungs 53/54 ship, and its ONE slot is the reason it exists.
///
/// `at_setting` is overridden at THREE levels — rung 53 here, rung 55's `StageStackMatcher`
/// (slice N) and rung 61's `StatorBleedMatcher` (slice O) — and FIVE reading methods reach it
/// through `self`. Rung 61's override exists precisely so a sweep cannot silently run with the
/// bleed valve SHUT; rung 55's so a swept setting cannot silently drop the stage stack. Both
/// failures produce plausible numbers on the wrong machine, which no value gate would flag.
///
/// **SLICE M CANNOT WITNESS THE DISPATCH** — it has no descendant — so the gate is an IOU in
/// `rung53.rs::slice_m_deferrals`, not a silent omission (the `slice_j_deferrals` /
/// `slice_l_deferrals` precedent, fourth use).
pub struct StatorHooks {
    /// A sibling matcher: the SAME hardware and the same design references, stators moved.
    pub at_setting: fn(&VariableStatorCore, f64, f64) -> VariableStatorCore,
}

/// RUNGS 53/54's table. Rung 54 adds no override — every method it brings is a pure read on
/// rung 53's object, which is what "the throat enters no solver" means expressed as a table.
pub const R53: StatorHooks = StatorHooks { at_setting: r53_at_setting };

/// Descendant state carried on the core, so [`StatorHooks::at_setting`]'s signature does not
/// change when slices N and O arrive.
///
/// One variant today. It is here rather than added later because the alternative — a return type
/// that names a concrete rung — is the signature slices N and O would have to BREAK, turning two
/// cheap additive slices into two more gated-code refactors. Rung 55's and rung 61's `at_setting`
/// bodies were read before this was decided: both read only fields of `self` (`K_lp`/`K_hp`/
/// `split`/`vsv_stages_*`/`cap_profile`; `bleed`), so both arrive as a VARIANT plus a TABLE ENTRY.
#[derive(Clone, Copy, Debug)]
pub enum Descendant {
    /// Rung 53/54: no state beyond the two settings.
    Plain,
}

// =========================================================================================
// THE MATCHER
// =========================================================================================

/// RUNG 53. Two-spool map matching with a VARIABLE STATOR on each compressor.
///
/// The stators sit at their DESIGN setting at the design point BY CONSTRUCTION (rung 42's
/// valve-shut discipline): the hardware and both maps' design references are captured from a
/// `v = 0` design run, and only then are the stators moved.
///
/// **THE REDUCE IS AN IDENTITY, and it is stronger than rung 42's dispatch.** At
/// `vsv_lp == vsv_hp == 0.0` the stored maps ARE the maps that were passed in and the matching
/// hooks are rung 39's own — there is no rung-53 code path to skip. Rungs 38–52 are untouched
/// because `psi` returns early at `vsv == 0` and because the `phi_surge` FIELD still means the
/// design-setting anchor, so rung 41/44/45's readers are literally unchanged.
///
/// **How that identity SPLITS in Rust, decided in the plan rather than discovered here.** Python
/// gates it with two assertions and only one survives verbatim. `VariableStatorMatcher.match is
/// TwoSpoolMapMatcher.match` — *there is no rung-53 code path to skip* — ports EXACTLY, as raw
/// fn-pointer equality between the `R53` and `R39` table entries, and that is the half carrying
/// the claim. `m.map_lp is LP` ports WEAKER: [`ComponentMap`] is `Copy`, so it has no object
/// identity to compare and none would be meaningful for a value type. The honest Rust is
/// field-wise `==` plus `vsv == 0.0`, **and that weakening is stated in the gate's own text** —
/// a reduce gate that silently answers a smaller question is the *ported test can go VACUOUS*
/// failure.
pub struct VariableStatorCore {
    /// Rung 39's object, built at the DESIGN setting and then handed the moved maps.
    pub core: TwoSpoolMapCore,
    pub vsv_lp: f64,
    pub vsv_hp: f64,
    /// The maps AS PASSED IN — the design-setting references every sibling is rebuilt from.
    pub map_lp_design: ComponentMap,
    pub map_hp_design: ComponentMap,
    /// Python's `_ctor` tuple, held for the same reason: `at_setting` re-invokes the constructor.
    design_engine: TwoSpoolEngine,
    flight_design: FlightCondition,
    mdot_design: f64,
    pub hooks: &'static StatorHooks,
    pub descendant: Descendant,
}

impl VariableStatorCore {
    /// Central-difference step in the stator setting — a PURE COORDINATE, so the step is a
    /// numerical choice and not a physical one.
    pub const DV: f64 = 5e-4;
    /// Incidence residual tolerance for the schedule root.
    pub const INC_TOL: f64 = 1e-12;
    /// Bisection cap on both schedule root-finders.
    ///
    /// **MEASURED DEAD** (§ 5.9 (iv)): rung 53's ladder uses 30–36 passes and rung 54's bracketed
    /// root 26–33, on every one of the 42 + 54 roots the probe grid produced. Ported as written
    /// and recorded dead, so no reader infers it is load-bearing.
    ///
    /// Rung 61 SHADOWS it at 200 (slice O). That shadow is live — it is read by these inherited
    /// solver loops — which is the one place a Python class constant is not a constant.
    pub const INC_MAX: usize = 80;
    /// Stator-setting scan step for the ceiling walk.
    pub const V_STEP: f64 = 0.04;
    /// Scan ceiling.
    ///
    /// **MEASURED DEAD** (§ 5.9 (ii)): no walk reaches it. Every one of the 100 probe cells ends
    /// on [`ComponentMap::solve_n`](crate::map::ComponentMap::solve_n)'s bracket instead, at
    /// settings spanning 1.16–3.36. Ported as written and recorded dead — slice L's
    /// `Tt4_lo = 350.0` precedent, second instance.
    pub const V_MAX: f64 = 8.0;

    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap, vsv_lp: f64, vsv_hp: f64,
    ) -> Self {
        Self::with_hooks(design_engine, flight_design, mdot_design, map_lp, map_hp,
                         vsv_lp, vsv_hp, &R53, Descendant::Plain)
    }

    /// The constructor descendants call, so a rung-55/61 core is built by the same capture-then-
    /// move sequence rather than by a second one.
    #[allow(clippy::too_many_arguments)]
    pub fn with_hooks(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap, vsv_lp: f64, vsv_hp: f64,
        hooks: &'static StatorHooks, descendant: Descendant,
    ) -> Self {
        assert!(map_lp.vsv == 0.0 && map_hp.vsv == 0.0,
                "rung-53 VariableStatorMatcher takes the DESIGN-SETTING maps and moves the \
                 stators itself (the design references must be captured at v=0). Pass \
                 vsv_lp/vsv_hp, not a map that already carries .with_vsv(.).");
        // THE ORDER IS THE RUNG: capture the hardware and both design references from the v=0
        // maps FIRST, and only then move the stators. Building the core from already-moved maps
        // would re-reference every corrected coordinate onto a machine that does not exist.
        let mut core = TwoSpoolMapCore::with_hooks(
            design_engine.clone(), flight_design, mdot_design, map_lp, map_hp, &R39);
        // At v == 0 the maps are LEFT ALONE, so the reduce is an identity and not a
        // re-construction — `with_vsv(0.0)` would be a different object carrying the same bits,
        // and Python's gate asserts the object.
        if vsv_lp != 0.0 {
            core.map_lp = map_lp.with_vsv(vsv_lp);
        }
        if vsv_hp != 0.0 {
            core.map_hp = map_hp.with_vsv(vsv_hp);
        }
        VariableStatorCore {
            core, vsv_lp, vsv_hp,
            map_lp_design: map_lp, map_hp_design: map_hp,
            design_engine, flight_design, mdot_design, hooks, descendant,
        }
    }

    /// The design throttle the schedules read their target incidence at.
    pub fn tt4_design(&self) -> f64 {
        self.core.base.tt4_design
    }

    /// The design flight condition — Python reads it back out of `_ctor[1]`.
    pub fn flight_design(&self) -> &FlightCondition {
        &self.flight_design
    }

    // --- THE DISPATCH POINT ---------------------------------------------------------------

    /// A sibling matcher on the SAME hardware and the same design references, stators moved —
    /// **through the virtual table.**
    ///
    /// Every sweep below goes through this, so a swept setting can never be confused with a
    /// re-designed engine (rung 42's controlled comparison, at fixed `Tt4`).
    pub fn at_setting(&self, vsv_lp: f64, vsv_hp: f64) -> VariableStatorCore {
        (self.hooks.at_setting)(self, vsv_lp, vsv_hp)
    }

    /// The sibling with ONE spool moved and the other at design — the shape every rung-53/54
    /// sweep actually wants. Python spells this inline, five times, as
    /// `at_setting(v, 0.0) if spool == "lp" else at_setting(0.0, v)`.
    fn at_one(&self, spool: Spool, v: f64) -> VariableStatorCore {
        match spool {
            Spool::Lp => self.at_setting(v, 0.0),
            Spool::Hp => self.at_setting(0.0, v),
        }
    }

    fn spool_bits(&self, spool: Spool) -> (ComponentMap, f64, f64, f64) {
        match spool {
            Spool::Lp => (self.core.map_lp, self.core.tau_lpc_d, self.core.base.eta_lpc,
                          self.vsv_lp),
            Spool::Hp => (self.core.map_hp, self.core.tau_hpc_d, self.core.base.eta_hpc,
                          self.vsv_hp),
        }
    }

    // --- RUNG 53's reading instrument -------------------------------------------------------

    /// RUNG 53's reading instrument: BOTH reference-free surge currencies, per spool.
    ///
    /// ```text
    ///     phi-margin       M_phi = phi_op - phi_surge(v)     [the wall MOVES with v]
    ///     incidence margin M_i   = T_c - tan_beta1(phi_op,v) [the wall is the METAL]
    /// ```
    ///
    /// Both vanish together, so as a STALL TEST they are equivalent; as a DISTANCE they are not,
    /// and that is the rung. `sm_n` is rung 41's constant-speed pressure-ratio margin evaluated
    /// at the LIVE floor, reported for definition-robustness.
    ///
    /// Needs a surge line on both maps (`phi_surge > 0`) — it is the incidence anchor too.
    pub fn stator_margin(&self, flight: &FlightCondition, tt4: f64) -> StatorMargin {
        self.try_stator_margin(flight, tt4).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE twin — see [`Abort`]. What propagates is the MATCH's failure, which is what
    /// rung 54's `_scan` walks until.
    ///
    /// **The two asserts inside stay asserts, and both were measured.** `phi_surge > 0` is
    /// reachable by CONSTRUCTION rather than by throttle (an unarmed map pair), and the floor
    /// assert `phi_s < phi_op` raises **0** times over 560 clean calls (§ 5.9 (iii)). The raise
    /// `_scan` actually catches is neither of them — it is `solve_n`'s bracket, three frames down,
    /// on 100 cells out of 100.
    pub fn try_stator_margin(
        &self, flight: &FlightCondition, tt4: f64,
    ) -> Result<StatorMargin, Abort> {
        let (ml, mh) = (&self.core.map_lp, &self.core.map_hp);
        assert!(ml.phi_surge > 0.0 && mh.phi_surge > 0.0,
                "rung-53 stator_margin needs the rung-36 floor as its incidence anchor on BOTH \
                 maps: build them with .with_phi_surge(phi_surge).");
        let od = self.core.try_match_point(flight, tt4)?;
        Ok(StatorMargin {
            tt4,
            vsv_lp: self.vsv_lp,
            vsv_hp: self.vsv_hp,
            lp: self.spool_row(&od, Spool::Lp, tt4),
            hp: self.spool_row(&od, Spool::Hp, tt4),
        })
    }

    fn spool_row(&self, od: &TwoSpoolMapResult, spool: Spool, tt4: f64) -> SpoolMargin {
        let (phi_op, n_op, tt_in) = match spool {
            Spool::Lp => (od.phi_lp, od.n_lp, od.base.station("2").tt),
            Spool::Hp => (od.phi_hp, od.n_hp, od.base.station("25").tt),
        };
        let (cmap, tau_d, eta_base, v) = self.spool_bits(spool);
        let (phi_s, t_c) = (cmap.phi_surge_at(), cmap.tan_beta1_crit());
        assert!(phi_s < phi_op,
                "rung-53 {} running line has crossed its OWN floor at Tt4={tt4:.1}, v={v:+.3}: \
                 phi_op={phi_op:.4} vs phi_surge(v)={phi_s:.4}.",
                match spool { Spool::Lp => "LP", Spool::Hp => "HP" });
        let pi_op = self.core.pi_c_spool(&cmap, tau_d, eta_base, n_op, phi_op, tt_in);
        let pi_s = self.core.pi_c_spool(&cmap, tau_d, eta_base, n_op, phi_s, tt_in);
        SpoolMargin {
            vsv: v, phi_op, n: n_op, m: phi_op * n_op,
            phi_surge: phi_s, phi_surge_design: cmap.phi_surge,
            m_phi: phi_op - phi_s,
            tan_b1: cmap.tan_beta1(phi_op), tan_b1_crit: t_c,
            m_i: t_c - cmap.tan_beta1(phi_op),
            pi_op, sm_n: pi_s / pi_op - 1.0,
            throat: None,
        }
    }

    /// Two-sided sweep of ONE spool's stator setting at FIXED throttle.
    ///
    /// Rung 50's lesson, reused: an edge is measured two-sided or not at all. Each row carries
    /// both currencies on BOTH spools, so the other spool's row is simultaneously rung 53's P5
    /// arrow measurement — how far a stator on one shaft reaches the other.
    pub fn stator_sweep(
        &self, flight: &FlightCondition, tt4: f64, vsv_grid: &[f64], spool: Spool,
    ) -> Vec<SweepRow> {
        vsv_grid.iter().map(|&v| {
            let r = self.at_one(spool, v).stator_margin(flight, tt4);
            SweepRow { vsv: v, swept: spool, lp: r.lp, hp: r.hp }
        }).collect()
    }
}

// --- the hook body -------------------------------------------------------------------------

/// RUNG 53's sibling constructor. Rebuilds from the DESIGN maps, exactly as Python re-invokes
/// its own constructor — see the module note's decision 1 for why this is not a copy-and-swap.
fn r53_at_setting(core: &VariableStatorCore, vsv_lp: f64, vsv_hp: f64) -> VariableStatorCore {
    VariableStatorCore::with_hooks(
        core.design_engine.clone(), core.flight_design, core.mdot_design,
        core.map_lp_design, core.map_hp_design, vsv_lp, vsv_hp, core.hooks, core.descendant)
}

// =========================================================================================
// THE HEADLINE, MEASURED — and its control
// =========================================================================================

/// What [`currency_split`](VariableStatorCore::currency_split) returns: the two currencies'
/// derivatives in the stator setting, on the spool whose stators move.
#[derive(Clone, Copy, Debug)]
pub struct CurrencySplit {
    pub spool: Spool,
    pub tt4: f64,
    /// The setting the difference was centred on.
    pub vsv: f64,
    /// The central-difference step actually used.
    pub dv: f64,
    pub phi_op: f64,
    pub phi_surge: f64,
    pub d_phi_op: f64,
    pub d_m: f64,
    pub d_n: f64,
    /// **RUNG 53's P1, as one number.** The stator is a SPEED lever: `m` moves only through the
    /// efficiency island, so this ratio is a machine zero on a flat island.
    pub flow_vs_speed: f64,
    /// The closed form the derivation predicts at the design point:
    /// `-(1+l)*phi^2/D`, with `D = 2 + 2*sigma*(phi-1) + l*(2-phi)`.
    pub d_phi_op_closed: f64,
    pub d_m_phi: f64,
    pub d_m_i: f64,
    pub d_sm_n: f64,
    /// `1/(2+l)` — the closed form for `dM_i/dv` at `phi = 1`.
    pub d_m_i_closed_design: f64,
    /// **THE HEADLINE**: do the two currencies disagree in SIGN?
    pub split: bool,
    /// `-dphi_op/dv`, the quantity the interval law tests.
    pub ratio: f64,
    /// `(phi_surge^2, phi_op^2)` — the interval whose WIDTH is the open margin.
    pub interval: (f64, f64),
    /// The interval law: the currencies disagree IFF `ratio` lies strictly inside `interval`.
    pub in_interval: bool,
    /// `sqrt((1+l)/(2+l))` — the floor value at which that interval closes.
    pub floor_boundary: f64,
}

/// One row of [`throttle_currency`](VariableStatorCore::throttle_currency) — a CONSECUTIVE
/// DIFFERENCE, so a grid of `k` throttles yields `k-1` rows.
#[derive(Clone, Copy, Debug)]
pub struct ThrottleRow {
    pub tt4: f64,
    pub spool: Spool,
    pub d_m_phi: f64,
    pub d_m_i: f64,
    pub d_sm_n: f64,
    pub signs_agree: bool,
    pub all_three_agree: bool,
    /// `dM_i/dM_phi`, which the derivation says must equal `jacobian`.
    pub ratio: f64,
    /// `1/phi_mid^2` — STRICTLY POSITIVE, which is the whole content of the control.
    pub jacobian: f64,
    pub phi_mid: f64,
}

/// One row of [`incidence_schedule`](VariableStatorCore::incidence_schedule): the setting that
/// holds the design incidence at this throttle, and what both currencies read there.
#[derive(Clone, Copy, Debug)]
pub struct ScheduleRow {
    pub tt4: f64,
    pub spool: Spool,
    pub vsv_star: f64,
    /// The residual at the returned root — `<= INC_TOL` by construction unless the bisection
    /// exhausted its cap or the bracket was already zero.
    pub residual: f64,
    pub tan_b1: f64,
    pub tan_b1_design: f64,
    pub phi_op: f64,
    /// The same throttle at the DESIGN setting — the counterfactual the schedule is read against.
    pub phi_op_bare: f64,
    pub phi_surge: f64,
    pub m_i: f64,
    pub m_i_bare: f64,
    pub m_phi: f64,
    pub m_phi_bare: f64,
    pub sm_n: f64,
    pub sm_n_bare: f64,
    pub n: f64,
}

impl VariableStatorCore {
    /// **THE HEADLINE, MEASURED**: the two currencies' derivatives in the stator setting, by
    /// central difference about THIS matcher's setting.
    ///
    /// Also returns the closed forms the derivation predicts at the design point, and the
    /// INTERVAL test: the currencies disagree iff `-phi_op'/v'` lies in
    /// `(phi_surge^2, phi_op^2)` — an interval whose WIDTH is the open margin. So the split is not
    /// a curiosity of one operating point; it is a statement about how much margin is open.
    ///
    /// **THE OTHER SPOOL STAYS AT THIS MATCHER'S SETTING, NOT AT DESIGN.** Python spells the two
    /// legs `at_setting(v, self.vsv_hp)` / `at_setting(self.vsv_lp, v)`, which differs from every
    /// other sweep in this file — those move one spool and pin the other at zero. Reusing
    /// [`at_one`](Self::at_one) here would silently reset the unswept spool and measure a
    /// derivative on a different machine.
    pub fn currency_split(
        &self, flight: &FlightCondition, tt4: f64, spool: Spool, dv: Option<f64>,
    ) -> CurrencySplit {
        let h = dv.unwrap_or(Self::DV);
        let v0 = match spool { Spool::Lp => self.vsv_lp, Spool::Hp => self.vsv_hp };
        let base = *self.stator_margin(flight, tt4).spool(spool);

        let leg = |v: f64| -> SpoolMargin {
            let sib = match spool {
                Spool::Lp => self.at_setting(v, self.vsv_hp),
                Spool::Hp => self.at_setting(self.vsv_lp, v),
            };
            *sib.stator_margin(flight, tt4).spool(spool)
        };
        let (lo, hi) = (leg(v0 - h), leg(v0 + h));
        let d_phi = (hi.phi_op - lo.phi_op) / (2.0 * h);
        let d_m = (hi.m - lo.m) / (2.0 * h);
        let d_n = (hi.n - lo.n) / (2.0 * h);
        let dm_phi = (hi.m_phi - lo.m_phi) / (2.0 * h);
        let dm_i = (hi.m_i - lo.m_i) / (2.0 * h);
        let dsm_n = (hi.sm_n - lo.sm_n) / (2.0 * h);

        let (cmap, _, _, _) = self.spool_bits(spool);
        let (l, sg) = (cmap.l, cmap.sigma);
        let (phi, phi_s) = (base.phi_op, base.phi_surge);
        let d = 2.0 + 2.0 * sg * (phi - 1.0) + l * (2.0 - phi);
        CurrencySplit {
            spool, tt4, vsv: v0, dv: h,
            phi_op: phi, phi_surge: phi_s,
            d_phi_op: d_phi, d_m, d_n,
            flow_vs_speed: (d_m / base.m).abs() / (d_n / base.n).abs(),
            d_phi_op_closed: -(1.0 + l) * phi * phi / d,
            d_m_phi: dm_phi, d_m_i: dm_i, d_sm_n: dsm_n,
            d_m_i_closed_design: 1.0 / (2.0 + l),
            split: (dm_phi < 0.0) != (dm_i < 0.0),
            ratio: -d_phi,
            interval: (phi_s * phi_s, phi * phi),
            in_interval: phi_s * phi_s < -d_phi && -d_phi < phi * phi,
            floor_boundary: powp((1.0 + l) / (2.0 + l), 0.5),
        }
    }

    /// **THE CONTROL for the headline, and the gate that could kill it.**
    ///
    /// At the DESIGN stator setting the only live lever is the THROTTLE, which moves `phi_op` and
    /// leaves the floor alone. Then `M_i = T_c - 1/phi_op` is a monotone reparameterisation of
    /// `M_phi = phi_op - phi_s0`, so `dM_i = dM_phi/phi_op^2` with a STRICTLY POSITIVE Jacobian:
    /// the two currencies MUST agree in sign and differ only by that factor. **A sign
    /// disagreement here would mean the moving floor is NOT the split's mechanism.**
    pub fn throttle_currency(
        &self, flight: &FlightCondition, tt4_grid: &[f64], spool: Spool,
    ) -> Vec<ThrottleRow> {
        assert!(self.vsv_lp == 0.0 && self.vsv_hp == 0.0,
                "rung-53 throttle_currency is the v=0 control: run it on a design-setting \
                 matcher.");
        let pts: Vec<SpoolMargin> = tt4_grid.iter()
            .map(|&t| *self.stator_margin(flight, t).spool(spool)).collect();
        pts.windows(2).zip(tt4_grid.iter().skip(1)).map(|(w, &t)| {
            let (a, b) = (&w[0], &w[1]);
            let (d_phi, d_i) = (b.m_phi - a.m_phi, b.m_i - a.m_i);
            let phi_mid = 0.5 * (a.phi_op + b.phi_op);
            let d_sm = b.sm_n - a.sm_n;
            ThrottleRow {
                tt4: t, spool, d_m_phi: d_phi, d_m_i: d_i, d_sm_n: d_sm,
                signs_agree: (d_phi > 0.0) == (d_i > 0.0),
                all_three_agree: (d_phi > 0.0) == (d_i > 0.0) && (d_i > 0.0) == (d_sm > 0.0),
                ratio: if d_phi != 0.0 { d_i / d_phi } else { f64::NAN },
                jacobian: 1.0 / (phi_mid * phi_mid),
                phi_mid,
            }
        }).collect()
    }

    /// RUNG 53's payoff object, and one the `phi`-currency cannot even express: the stator
    /// schedule `v*(Tt4)` that holds the rotor INCIDENCE at its design value — which is what a
    /// real VSV schedule is FOR.
    ///
    /// `T_design` is READ (not assumed) off this matcher at the design setting and design
    /// throttle, so the schedule inherits no constant of its own. Along the returned schedule
    /// `M_i` is constant BY CONSTRUCTION while `M_phi` is not — **that contrast IS the headline,
    /// made operational**: the `phi`-currency reports a margin LOSS along a schedule that changes
    /// the true margin not at all.
    ///
    /// # The premise, and the condition the source's own docstring never names
    ///
    /// The ladder is justified by "closing the stators lowers `tan(beta_1)` monotonically", and
    /// rung 54 BOUNDS that premise: where the incidence peak is INTERIOR, `tan(beta_1)` turns back
    /// UP past the peak, so a doubling ladder can step OVER the root and out the far side —
    /// reporting the schedule unreachable when it exists. **At this method's own default
    /// `v_hi = 1.0` it does no such thing.** Measured (§ 5.9 (viii)): the walk-over reproduces on
    /// **1 of 6** shape×spool cells and there only at caps ≥ 1.725, because an interior peak is
    /// NECESSARY BUT NOT SUFFICIENT — the peak must lie *between the root and the cap*.
    /// `steep`/HP has an interior peak and never walks over at ANY cap. The source names neither
    /// condition.
    ///
    /// The port reproduces BOTH SIDES and repairs neither: a more careful Rust bracket would look
    /// like an improvement and be a silent divergence. Prefer
    /// [`schedule_throat`](Self::schedule_throat) on an unfamiliar map shape.
    ///
    /// The bracket assert PROPAGATES UNCAUGHT — 18 of the probe grid's 80 cells raise it —
    /// because nothing wraps this method in a catch. That is why it is an `assert!` and not an
    /// [`Abort`].
    pub fn incidence_schedule(
        &self, flight: &FlightCondition, tt4_grid: &[f64], spool: Spool, v_hi: f64,
    ) -> Vec<ScheduleRow> {
        let t_design = self.at_setting(0.0, 0.0)
            .stator_margin(&self.flight_design, self.tt4_design()).spool(spool).tan_b1;
        let read = |v: f64, tt4: f64| -> SpoolMargin {
            *self.at_one(spool, v).stator_margin(flight, tt4).spool(spool)
        };

        tt4_grid.iter().map(|&tt4| {
            let bare = read(0.0, tt4);
            let mut lo = 0.0f64;
            let mut r_lo = bare.tan_b1 - t_design;      // > 0 below design power
            let (mut v, mut r) = (lo, r_lo);
            if r_lo.abs() > Self::INC_TOL {
                // Ladder the upper bracket UP rather than starting at `v_hi`: a large trial
                // setting unloads the speed line so far that `solve_n`'s own n-bracket fails —
                // a map-validity edge, not a root-finding failure — so walk out gently.
                let (mut hi, cap) = (0.05f64, v_hi);
                // Python guards with `r_hi is not None and r_hi < 0.0`, initialising `r_hi = None`
                // before the loop. **That sentinel is UNREACHABLE** — the body assigns before
                // every exit — so the `is not None` half can never be the reason the assert
                // fires. Here the loop BREAKS WITH the value, which makes the unset state
                // unrepresentable rather than merely unreached: the guard that survives is the one
                // that can actually fail.
                let r_hi = loop {
                    hi = hi.min(cap);
                    let rh = read(hi, tt4).tan_b1 - t_design;
                    if rh < 0.0 || hi >= cap {
                        break rh;
                    }
                    lo = hi;
                    r_lo = rh;
                    hi = 2.0 * hi;
                };
                assert!(r_hi < 0.0,
                        "rung-53 incidence schedule does not bracket at Tt4={tt4:.0} within \
                         v <= {v_hi:.2}: residual {r_lo:+.4e} at v={lo:.4}. The design incidence \
                         is unreachable this far off design — raise v_hi or narrow the throttle \
                         grid.");
                for _ in 0..Self::INC_MAX {
                    v = 0.5 * (lo + hi);
                    r = read(v, tt4).tan_b1 - t_design;
                    if r.abs() <= Self::INC_TOL || hi - lo <= 1e-14 {
                        break;
                    }
                    if r * r_lo > 0.0 {
                        lo = v;
                        r_lo = r;
                    } else {
                        hi = v;
                    }
                }
            }
            let at = read(v, tt4);
            ScheduleRow {
                tt4, spool, vsv_star: v, residual: r,
                tan_b1: at.tan_b1, tan_b1_design: t_design,
                phi_op: at.phi_op, phi_op_bare: bare.phi_op, phi_surge: at.phi_surge,
                m_i: at.m_i, m_i_bare: bare.m_i,
                m_phi: at.m_phi, m_phi_bare: bare.m_phi,
                sm_n: at.sm_n, sm_n_bare: bare.sm_n, n: at.n,
            }
        }).collect()
    }
}
