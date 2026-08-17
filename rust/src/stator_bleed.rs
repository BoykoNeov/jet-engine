//! RUNG 61 — the VARIABLE STATOR and the INTERSTAGE BLEED VALVE on one steady machine.
//!
//! **A compensating lever buys back the COORDINATE, not the BILL.**
//!
//! # What this module is, structurally
//!
//! Python's `StatorBleedMatcher(TwoSpoolBleedMatcher, VariableStatorMatcher)` is the port's ONE
//! diamond, and § 5.3's pre-flight discharged it before a line was written: the two parents
//! collide on exactly one name (`__init__`), rung 53 overrides nothing on the plant, and rung 61
//! opts out of the constructor chain by hand. The flattened order is
//! `61 → 42 → 53 → 39 → 38`, and here that is not an order at all — it is **two table pointers**:
//!
//! | pointer | value | what it buys |
//! |---|---|---|
//! | [`VariableStatorCore::hooks`] | [`R61`] | `at_setting` carries the valve position |
//! | [`crate::two_spool::TwoSpoolMapCore::hooks`] | [`R42`] | every rung-53 reader sees the BLED match |
//!
//! # § 5.11 (v) — THE PLAN PREDICTED A `Descendant` VARIANT AND IT IS NOT NEEDED
//!
//! [`crate::stator::Descendant`]'s own doc said *"slices N and O add a VARIANT and a TABLE
//! ENTRY"*, and § 5.9 said the same. **Slice O adds a table entry and no variant**, because the
//! one piece of state rung 61's `at_setting` reads — the bleed fraction — has lived on
//! [`crate::two_spool::TwoSpoolMapCore`] since slice L, where rung 42 needed it. A rung-61 core
//! is [`Descendant::Plain`], and that is also the right answer for
//! [`VariableStatorCore::inc_max`]: rung 61 declares no `_INC_MAX`, so it inherits rung 53's 80,
//! which is exactly what `Plain` returns.
//!
//! Slice N was burned twice by a carrier living one level below the hook that reads it. Here the
//! same fact arrives in the port's favour, and the lesson is symmetric: **the question is never
//! "does this hook need a variant", it is "where does this hook's state already live".**
//!
//! # THE CONSTRUCTION ORDER IS THE RUNG, AND THE HAZARD IS CHECKED ABSENT
//!
//! Rung 61 builds a rung-53 core (which captures the hardware and both design references from a
//! `v = 0` run) and only THEN opens the valve — Python's own sequence, which sets `self.bleed`
//! after `VariableStatorMatcher.__init__` returns. That is only safe because
//! [`VariableStatorCore::with_hooks`] never dispatches through the inner table: it calls
//! `TwoSpoolCore::new` and reads stations off the reference run. Had the capture matched even
//! once, `&R42` at `bleed = 0.0` is a *different function object* than `&R39` and every design
//! reference would shift — a value-level difference no signature inspection shows (§ 5.11 (vi)).
//!
//! # THE TRAP THIS MODULE EXISTS TO MAKE IMPOSSIBLE
//!
//! Rung 53's `at_setting` rebuilds through [`VariableStatorCore::with_hooks`], which initialises
//! `bleed: 0.0`. A rung-61 sibling built through **rung 53's** body therefore comes back with the
//! valve SHUT — every headline number plausible, on a machine that does not exist. § 5.3 item 7
//! measured the co-operative-`super()` version of the same failure and found the damage
//! concentrated in exactly the two quantities the rung is about (`φ` and `N`, 13–15 %) while
//! thrust moved 0.1 %. **No value gate can catch that**, which is why [`R61`] exists and why the
//! gate for it is structural.

use crate::bleed::R42;
use crate::engine::FlightCondition;
use crate::gas::Abort;
use crate::map::ComponentMap;
use crate::stator::{AuthorityCeiling, Descendant, SpoolMargin, StatorHooks, VariableStatorCore};
use crate::two_spool::{Spool, TwoSpoolEngine};

use std::cell::Cell;

// =========================================================================================
// THE CENSUS — one read-and-reset, eleven tallies
// =========================================================================================

thread_local! {
    static CENSUS: Cell<BleedCensus> = const { Cell::new(BleedCensus::ZERO) };
}

/// What [`take_census`] returns. Every tally exists because § 5.11 registered a prediction the
/// VALUES cannot test — a dead cap, a dead arm of a compound condition, and a `try/except` that
/// swallows nothing on the shipped grid are all invisible to a bit-exact dump.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BleedCensus {
    /// [`StatorBleedCore::feasible`] calls — § 5.11 (i)'s **10 613** on the probe grid.
    pub feasible_calls: u64,
    /// …of which the plant REFUSED. § 5.11 (i)'s **0** on the suite's own grid, and the whole
    /// finding: the method exists to swallow refusals and swallows none.
    pub feasible_none: u64,
    /// [`StatorBleedCore::at_point`] constructions — one per `feasible` call, by construction.
    pub at_point_built: u64,
    /// `_B_STEP` walk steps taken, summed.
    pub walk_steps: u64,
    /// The largest single walk — against `_B_CAP / _B_STEP = 22.5`.
    pub walk_steps_max: u64,
    /// Bisection passes, summed.
    pub bisect_passes: u64,
    /// The largest single bisection — **against `_B_MAX = 80`, which § 5.11 (ii) measured DEAD
    /// at 22–30.**
    pub bisect_passes_max: u64,
    /// Bisections that ended on `|r| <= _B_TOL` — § 5.11 (ii)'s **196 of 196**.
    pub exit_tol: u64,
    /// Bisections that ended on `hi - lo <= 1e-15` — **the DEAD ARM**, § 5.11 (ii)'s **0**.
    pub exit_interval: u64,
    /// Bisections that ran out of `_B_MAX` without either exit clause. Distinct from both above:
    /// Python falls out of the `for` and returns the last mid anyway.
    pub exit_ran_out: u64,
    /// `"valve authority exhausted"` returns — § 5.11 (i)'s **124 of 320**, the ONE live refusal.
    pub exit_cap: u64,
    /// `"choked envelope closed"` returns — dead on the shipped grid, live past `b = 0.49` at
    /// `Tt4 = 700`.
    pub exit_envelope: u64,
    /// `"stator setting infeasible"` returns — dead on the shipped grid, live past `v ≈ 1.3`.
    pub exit_stator_infeasible: u64,
}

impl BleedCensus {
    pub const ZERO: Self = Self {
        feasible_calls: 0, feasible_none: 0, at_point_built: 0,
        walk_steps: 0, walk_steps_max: 0, bisect_passes: 0, bisect_passes_max: 0,
        exit_tol: 0, exit_interval: 0, exit_ran_out: 0,
        exit_cap: 0, exit_envelope: 0, exit_stator_infeasible: 0,
    };
}

/// Read the tallies and RESET them.
///
/// **Single-consumer, like [`crate::stage::take_census`].** Two gates reading the same run see
/// the second one empty; every consumer below therefore takes its own census immediately after
/// the call it is measuring.
pub fn take_census() -> BleedCensus {
    CENSUS.with(|c| c.replace(BleedCensus::ZERO))
}

fn bump(f: impl FnOnce(&mut BleedCensus)) {
    CENSUS.with(|c| {
        let mut v = c.get();
        f(&mut v);
        c.set(v);
    });
}

// =========================================================================================
// THE HOOK TABLE
// =========================================================================================

/// RUNG 61's table — the THIRD and last `at_setting` override (53 · 55 · 61).
///
/// Python's docstring calls the override *load-bearing* and names the six rung-53/54 readers that
/// route through it (`stator_sweep`, `currency_split`, `incidence_schedule`, `_scan`,
/// `authority_ceiling`, `schedule_throat`). In Rust the load-bearing part is one line —
/// [`r61_at_setting`] restores `core.bleed` after the rebuild — and everything else is rung 53's
/// own body, unchanged and still witnessed bit-for-bit by the rung-53/54 suites.
pub const R61: StatorHooks = StatorHooks { at_setting: r61_at_setting };

/// RUNG 61's sibling constructor: rung 53's rebuild, with the valve carried across.
///
/// **The `bleed` is set AFTER construction and that is not a shortcut** — it is Python's own
/// sequence (`VariableStatorMatcher.__init__` first, `self.bleed = float(bleed)` second), and it
/// is safe only because the capture never dispatches (see the module note).
fn r61_at_setting(core: &VariableStatorCore, vsv_lp: f64, vsv_hp: f64) -> VariableStatorCore {
    let bleed = core.core.bleed;
    let mut sib = VariableStatorCore::with_hooks(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        core.map_lp_design, core.map_hp_design, vsv_lp, vsv_hp, core.hooks, core.core.hooks,
        core.descendant.clone());
    sib.core.bleed = bleed;
    sib
}

// =========================================================================================
// THE TWO DISCLOSED CHOICES, AS TYPES
// =========================================================================================

/// Which quantity `b*` is asked to restore.
///
/// **Python asserts `target in ("phi", "m_phi")` and this makes that assert unrepresentable** —
/// slice N's precedent (`Split` / `CapProfile`), third use. The distinction is the rung: the
/// stator MOVED THE FLOOR between the two instructions, so they are different numbers and the
/// gap IS the floor motion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    /// Restore the POINT — `phi_op(v, b*) == phi_op(0, 0)`.
    Phi,
    /// Restore the REPORTED MARGIN — `M_phi(v, b*) == M_phi(0, 0)`.
    MPhi,
}

impl Target {
    fn read(self, row: &SpoolMargin) -> f64 {
        match self {
            Target::Phi => row.phi_op,
            Target::MPhi => row.m_phi,
        }
    }

    /// The dump's spelling, so the oracle compares Python's own `target` string.
    pub fn as_str(self) -> &'static str {
        match self {
            Target::Phi => "phi",
            Target::MPhi => "m_phi",
        }
    }
}

/// Why `b*` does not exist.
///
/// **Both of these are DEAD on the whole Python suite** (§ 5.11 (i)): across 10 613 `feasible`
/// calls on the shipped grid the plant refused zero times, so only [`Exhausted::ValveAuthority`]
/// — which needs no refusal at all, just a walk that runs out of valve — is ever reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exhausted {
    /// The walk reached `_B_CAP` without the residual crossing. **The live one**, 124 of 320.
    ValveAuthority,
    /// The plant refused a step of the walk. Reachable past `b = 0.49` near the throttle edge.
    EnvelopeClosed,
}

impl Exhausted {
    /// Python's own `reason` string, for the oracle.
    pub fn as_str(self) -> &'static str {
        match self {
            Exhausted::ValveAuthority => "valve authority exhausted (b >= cap)",
            Exhausted::EnvelopeClosed => "choked envelope closed before the target",
        }
    }
}

/// What [`StatorBleedCore::compensating_bleed`] returns.
///
/// **AN ENUM AND NOT A STRUCT OF `Option`s — § 5.11 (iv), registered as a port decision.** Python
/// returns three DIFFERENT key sets from this one function, and `compensability` reads
/// `c.get("resid_last")`, which tolerates the third shape. A struct with `Option` fields would let
/// a caller read a field Python would have raised `KeyError` on, which is the *ported test can go
/// VACUOUS* failure wearing a type.
#[derive(Clone, Copy, Debug)]
pub enum Compensating {
    /// The bracket closed. Carries the three BARE readings Python attaches only here.
    Solved {
        spool: Spool,
        tt4: f64,
        vsv: f64,
        target: Target,
        b_star: f64,
        goal: f64,
        resid: f64,
        bare_phi: f64,
        bare_m_phi: f64,
        bare_m_i: f64,
    },
    /// The walk ended without a crossing. Carries `b_last`/`resid_last`; Python's other None
    /// branch does not.
    Exhausted {
        spool: Spool,
        tt4: f64,
        vsv: f64,
        target: Target,
        reason: Exhausted,
        goal: f64,
        b_last: f64,
        resid_last: f64,
    },
    /// The stator setting alone is infeasible with the valve shut — Python's ONLY None branch
    /// that carries neither `b_last` nor `resid_last`.
    StatorInfeasible { spool: Spool, tt4: f64, vsv: f64, target: Target, goal: f64 },
}

impl Compensating {
    /// `b*`, or `None` — Python's `c["b_star"]`.
    pub fn b_star(&self) -> Option<f64> {
        match *self {
            Compensating::Solved { b_star, .. } => Some(b_star),
            _ => None,
        }
    }

    /// Python's `c.get("reason")` — `None` on the solved branch.
    pub fn reason(&self) -> Option<&'static str> {
        match *self {
            Compensating::Solved { .. } => None,
            Compensating::Exhausted { reason, .. } => Some(reason.as_str()),
            Compensating::StatorInfeasible { .. } => {
                Some("stator setting infeasible with the valve shut")
            }
        }
    }

    /// Python's `c.get("resid_last")` — **absent, not null, on the third branch**, which is why
    /// `compensability` uses `.get` and why this returns `Option`.
    pub fn resid_last(&self) -> Option<f64> {
        match *self {
            Compensating::Exhausted { resid_last, .. } => Some(resid_last),
            _ => None,
        }
    }

    pub fn goal(&self) -> f64 {
        match *self {
            Compensating::Solved { goal, .. }
            | Compensating::Exhausted { goal, .. }
            | Compensating::StatorInfeasible { goal, .. } => goal,
        }
    }
}

// =========================================================================================
// THE MATCHER
// =========================================================================================

/// RUNG 61. Two-spool map matching with BOTH rung 53's variable stators and rung 42's valve.
///
/// **THE REDUCE IS TWO-AXIS**, and it is stronger than either parent's alone:
///
/// | corner | reduces to | how |
/// |---|---|---|
/// | `(v = 0, b = 0)` | rung 39 | `R42`'s match forwards at `b == 0`; `R61`'s `at_setting` is unreached by `match` |
/// | `(v ≠ 0, b = 0)` | rung 53 | same, with the maps moved by rung 53's own constructor |
/// | `(v = 0, b ≠ 0)` | rung 42 | rung 53's `psi` returns early at `vsv == 0`, so the maps are the ones passed in |
pub struct StatorBleedCore {
    pub core: VariableStatorCore,
    /// THE VALVE'S AUTHORITY CEILING, AS AN INSTANCE VALUE — see [`with_b_cap`](Self::with_b_cap).
    b_cap: f64,
}

impl StatorBleedCore {
    /// Absolute tolerance on the compensated coordinate. **LIVE** — § 5.11 (ii) measured every
    /// one of 196 bisections ending here.
    pub const B_TOL: f64 = 1e-11;
    /// Bisection cap.
    ///
    /// **MEASURED DEAD** (§ 5.11 (ii)): 22–30 passes over all 196 solved calls on the probe grid.
    /// Ported as written and recorded dead — `_INC_MAX`'s precedent (§ 5.9 (iv)), and slice L's
    /// `Tt4_lo` before it. A dead cap's SPELLING still has to be right.
    pub const B_MAX: usize = 80;
    /// Rung 42's own bound, minus a hair — the valve's authority ceiling.
    pub const B_CAP: f64 = 0.45;
    /// Walk step.
    pub const B_STEP: f64 = 0.02;

    /// Build a rung-61 matcher.
    ///
    /// **`lp_disabled` IS NOT A PARAMETER, AND THAT IS § 5.11's P7.** Python carries it and
    /// asserts `not (lp_disabled and bleed != 0.0)`. Here [`VariableStatorCore`] holds a
    /// [`crate::two_spool::TwoSpoolMapCore`] directly rather than the degenerate-or-not enum, so
    /// **there is no `lp_disabled` rung-61 object for the assert to reject** — the Python guard is
    /// unrepresentable rather than ported, which is a stronger statement than a runtime check.
    /// Recorded in `rung61.rs::slice_o_deferrals`, not silently dropped.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap, vsv_lp: f64, vsv_hp: f64, bleed: f64,
    ) -> Self {
        // THE ORDER IS THE RUNG (module note): a rung-53 core first — which captures the hardware
        // and both design references from a v = 0, b = 0 run — and only then the valve.
        let mut core = VariableStatorCore::with_hooks(
            design_engine, flight_design, mdot_design, map_lp, map_hp, vsv_lp, vsv_hp,
            &R61, &R42, Descendant::Plain);
        assert!((0.0..0.5).contains(&bleed),
                "rung-61 bleed fraction must be in [0, 0.5) (rung 42's bound).");
        core.core.bleed = bleed;
        Self { core, b_cap: Self::B_CAP }
    }

    /// Override `_B_CAP` on THIS object only.
    ///
    /// **A CLASS CONSTANT A SHIPPED TEST REACHES INSIDE AND REBINDS PER INSTANCE**, which an
    /// associated const cannot express — `test_rung61.py:370` writes `m._B_CAP = cap` to run the
    /// NEGATIVE CONTROL that keeps this rung from claiming a derived compensability ceiling. It
    /// is slice N's `_V_SCAN` exactly ([`crate::stage::StageStackCore::with_v_scan`]), and it was
    /// found the way that lesson prescribes: `grep '\\._[A-Z_]* *='` over the suite, run at the
    /// START of the port and not at the end.
    ///
    /// **THE OVERRIDE DELIBERATELY DOES NOT PROPAGATE**, and that is faithful rather than lazy.
    /// Python sets an INSTANCE attribute; [`at_point`](Self::at_point) then constructs a fresh
    /// object which reads the CLASS attribute — and the siblings only ever serve
    /// `stator_margin`, never a walk, so no sibling's cap is read at all.
    pub fn with_b_cap(mut self, b_cap: f64) -> Self {
        self.b_cap = b_cap;
        self
    }

    pub fn bleed(&self) -> f64 {
        self.core.core.bleed
    }

    // --- sibling constructors: rung 42's controlled comparison, in TWO coordinates ---------

    /// The same hardware and the same design references at an arbitrary `(v, b)`.
    pub fn at_point(&self, vsv_lp: f64, vsv_hp: f64, bleed: f64) -> StatorBleedCore {
        bump(|c| c.at_point_built += 1);
        Self::new(
            self.core.design_engine().clone(), *self.core.flight_design(),
            self.core.mdot_design(), self.core.map_lp_design, self.core.map_hp_design,
            vsv_lp, vsv_hp, bleed)
    }

    pub fn at_bleed(&self, bleed: f64) -> StatorBleedCore {
        self.at_point(self.core.vsv_lp, self.core.vsv_hp, bleed)
    }

    /// Python's `at_setting` override, at the concrete type. The *dispatching* copy — the one
    /// rung 53's own readers reach through `self` — is [`r61_at_setting`] in the [`R61`] table.
    pub fn at_setting(&self, vsv_lp: f64, vsv_hp: f64) -> StatorBleedCore {
        self.at_point(vsv_lp, vsv_hp, self.bleed())
    }

    // --- the price of the stator's phi-debit ----------------------------------------------

    /// One trial: the margin row at `(v, b)`, or `None` if the plant refuses it.
    ///
    /// **THE `None` IS THE POINT AND IT NEVER HAPPENS ON THE SHIPPED GRID** — § 5.11 (i): 10 613
    /// calls, 10 613 rows. Python's docstring says the feasible set is bounded on both axes by
    /// different mechanisms, and a 1 760-cell wide sweep CONFIRMS that (`v ≈ 1.3` via the
    /// speed-line bracket; `b = 0.49` via the choked envelope, and only near the throttle edge) —
    /// what is corrected is the SCOPE: both bounds sit entirely outside every shipped test.
    ///
    /// **One fidelity gap, measured dead.** Python's bare `except AssertionError` would also
    /// swallow rung 53's floor assert (`phi_s < phi_op`); here that stays a panic, because
    /// [`VariableStatorCore::try_stator_margin`] makes only the MATCH fallible. Slice M measured
    /// that assert raising 0 times in 560 calls (§ 5.9 (iii)) and the wide sweep above found only
    /// TWO message classes in 756 refusals — neither of them the floor. Recorded, not hidden.
    pub fn feasible(
        &self, flight: &FlightCondition, tt4: f64, v: f64, spool: Spool, b: f64,
    ) -> Option<SpoolMargin> {
        bump(|c| c.feasible_calls += 1);
        let sib = match spool {
            Spool::Lp => self.at_point(v, 0.0, b),
            Spool::Hp => self.at_point(0.0, v, b),
        };
        match sib.core.try_stator_margin(flight, tt4) {
            Ok(m) => Some(*m.spool(spool)),
            Err(Abort(_)) => {
                bump(|c| c.feasible_none += 1);
                None
            }
        }
    }

    /// `b*(v)`: the bleed that BUYS BACK what closing the stator to `v` spent.
    ///
    /// See [`Target`] for why the two instructions give different numbers — the gap IS the floor
    /// motion the stator caused, which is rung 53's headline reached from a third direction.
    ///
    /// Returns [`Compensating::Exhausted`] with `reason` when the plant cannot deliver it — which
    /// is the HP spool's NORMAL answer, because rung 42's `dphi_H/db` passes through zero at `π*`
    /// and reverses below it.
    pub fn compensating_bleed(
        &self, flight: &FlightCondition, tt4: f64, v: f64, spool: Spool, target: Target,
    ) -> Compensating {
        let bare = self.feasible(flight, tt4, 0.0, spool, 0.0)
            .unwrap_or_else(|| panic!(
                "rung-61: the BARE machine is already infeasible at Tt4={tt4:.1}."));
        let goal = target.read(&bare);

        let at0 = match self.feasible(flight, tt4, v, spool, 0.0) {
            Some(r) => r,
            None => {
                bump(|c| c.exit_stator_infeasible += 1);
                return Compensating::StatorInfeasible { spool, tt4, vsv: v, target, goal };
            }
        };
        // < 0 when the stator spent something to buy back.
        let r0 = target.read(&at0) - goal;

        // Walk the valve open until the residual crosses or the plant refuses. Rung 42's envelope
        // guard raises, so "ran out of valve" and "ran out of envelope" are DIFFERENT answers and
        // are reported as such.
        let (mut lo, mut r_lo) = (0.0_f64, r0);
        let (mut hi, mut r_hi) = (None, 0.0_f64);
        let mut b = 0.0_f64;
        let mut nwalk = 0_u64;
        while b < self.b_cap {
            b = (b + Self::B_STEP).min(self.b_cap);
            nwalk += 1;
            let row = match self.feasible(flight, tt4, v, spool, b) {
                Some(r) => r,
                None => {
                    bump(|c| {
                        c.walk_steps += nwalk;
                        c.walk_steps_max = c.walk_steps_max.max(nwalk);
                        c.exit_envelope += 1;
                    });
                    return Compensating::Exhausted {
                        spool, tt4, vsv: v, target, reason: Exhausted::EnvelopeClosed,
                        goal, b_last: lo, resid_last: r_lo,
                    };
                }
            };
            let r = target.read(&row) - goal;
            if (r_lo < 0.0 && 0.0 <= r) || (r_lo > 0.0 && 0.0 >= r) {
                hi = Some(b);
                r_hi = r;
                break;
            }
            lo = b;
            r_lo = r;
        }
        bump(|c| {
            c.walk_steps += nwalk;
            c.walk_steps_max = c.walk_steps_max.max(nwalk);
        });
        let mut hi = match hi {
            Some(h) => h,
            None => {
                bump(|c| c.exit_cap += 1);
                return Compensating::Exhausted {
                    spool, tt4, vsv: v, target, reason: Exhausted::ValveAuthority,
                    goal, b_last: lo, resid_last: r_lo,
                };
            }
        };

        let mut r = r_hi;
        let mut npass = 0_u64;
        // THE EXIT IS A DISJUNCTION AND ITS SECOND ARM IS DEAD (§ 5.11 (ii)): 196 of 196
        // bisections end on `|r| <= B_TOL`. Spelled as written anyway — a `||` whose right side is
        // dropped is a different function wherever the grid later moves.
        let mut exit = 2_u8;  // 0 = tol, 1 = interval, 2 = ran out of B_MAX
        for _ in 0..Self::B_MAX {
            npass += 1;
            let mid = 0.5 * (lo + hi);
            let row = self.feasible(flight, tt4, v, spool, mid)
                .expect("rung-61 bisection stepped outside a bracketed interval");
            r = target.read(&row) - goal;
            if r.abs() <= Self::B_TOL || hi - lo <= 1e-15 {
                exit = if r.abs() <= Self::B_TOL { 0 } else { 1 };
                lo = mid;
                hi = mid;
                break;
            }
            if (r < 0.0) == (r_lo < 0.0) {
                lo = mid;
                r_lo = r;
            } else {
                hi = mid;
            }
        }
        bump(|c| {
            c.bisect_passes += npass;
            c.bisect_passes_max = c.bisect_passes_max.max(npass);
            match exit {
                0 => c.exit_tol += 1,
                1 => c.exit_interval += 1,
                _ => c.exit_ran_out += 1,
            }
        });
        Compensating::Solved {
            spool, tt4, vsv: v, target,
            b_star: 0.5 * (lo + hi),
            goal,
            resid: r,
            bare_phi: bare.phi_op,
            bare_m_phi: bare.m_phi,
            bare_m_i: bare.m_i,
        }
    }
}

// =========================================================================================
// THE COMPENSATED POINT, IN EVERY CURRENCY
// =========================================================================================

/// One row of [`StatorBleedCore::compensated_point`] — bare `(0,0)`, bare-stator `(v,0)` and
/// compensated `(v,b*)`, in all four currencies at once.
///
/// The compensated half is `None` exactly when `b*` does not exist, which is the HP spool's
/// normal answer. Python returns a dict that is simply MISSING those keys; here the absence is
/// one `Option` rather than fourteen, which is the same information with one place to check.
#[derive(Clone, Copy, Debug)]
pub struct CompensatedPoint {
    pub spool: Spool,
    pub tt4: f64,
    pub vsv: f64,
    pub b_star: Option<f64>,
    pub reason: Option<&'static str>,
    pub phi_bare: f64,
    pub phi_stator: f64,
    pub m_i_bare: f64,
    pub m_i_stator: f64,
    pub m_phi_bare: f64,
    pub m_phi_stator: f64,
    pub n_bare: f64,
    pub n_stator: f64,
    pub thrust_bare: f64,
    pub thrust_stator: f64,
    pub phi_other_bare: f64,
    /// P6 — rung 53's exact per-spool zero, measured on the OTHER spool under the lever alone.
    pub d_phi_other_stator: f64,
    pub comp: Option<Compensated>,
}

/// The half of [`CompensatedPoint`] that exists only when `b*` does.
#[derive(Clone, Copy, Debug)]
pub struct Compensated {
    pub phi_comp: f64,
    pub m_i_comp: f64,
    pub m_phi_comp: f64,
    pub n_comp: f64,
    pub thrust_comp: f64,
    /// **P3 — THE TWO EXACT IDENTITIES the iso-φ locus forces**, rung 60's tautology reached by a
    /// THIRD route (restoration, not pinning): `dM_i = v` and
    /// `dM_phi = v·φ_s0²/(1 + v·φ_s0)`. Gated as identities, explicitly NOT as findings.
    pub d_m_i: f64,
    pub d_m_i_pred: f64,
    pub d_m_phi: f64,
    pub d_m_phi_pred: f64,
    pub d_m_i_resid: f64,
    pub d_m_phi_resid: f64,
    /// P5 — the bill, RELOCATED: rung 53's overspeed against rung 42's thrust one.
    pub dn_stator: f64,
    pub dn_comp: f64,
    pub d_f_stator: f64,
    pub d_f_comp: f64,
    pub phi_other_comp: f64,
    pub d_phi_other_comp: f64,
}

/// One row of [`StatorBleedCore::compensability`] — **rung 61's headline object**.
#[derive(Clone, Copy, Debug)]
pub struct Compensability {
    pub tt4: f64,
    pub vsv: f64,
    pub pi_hpc: f64,
    pub pi_lpc: f64,
    pub b_lp: Option<f64>,
    pub b_hp: Option<f64>,
    pub why_lp: Option<&'static str>,
    pub why_hp: Option<&'static str>,
    pub resid_lp: Option<f64>,
    pub resid_hp: Option<f64>,
    /// `b_hp / b_lp`, **through Python's TRUTHINESS** — see the method note.
    pub ratio: Option<f64>,
}

/// One row of [`StatorBleedCore::authority_with_bleed`] — the seam AS POSED, scored.
#[derive(Clone, Copy, Debug)]
pub struct AuthorityRow {
    pub bleed: f64,
    pub v_edge: f64,
    pub v_peak: f64,
    pub peak_interior: bool,
    pub m_i_0: f64,
    pub m_i_peak: f64,
    pub m_i_edge: f64,
    pub span: f64,
    pub n_scan: usize,
}

/// One row of [`StatorBleedCore::price_split`] — P4's two loci.
#[derive(Clone, Copy, Debug)]
pub struct PriceSplit {
    pub vsv: f64,
    pub b_phi: Option<f64>,
    pub b_m_phi: Option<f64>,
    pub gap: Option<f64>,
    pub floor_motion: f64,
    pub why_phi: Option<&'static str>,
    pub why_m_phi: Option<&'static str>,
}

impl StatorBleedCore {
    fn phi_surge_design(&self, spool: Spool) -> f64 {
        match spool {
            Spool::Lp => self.core.map_lp_design.phi_surge,
            Spool::Hp => self.core.map_hp_design.phi_surge,
        }
    }

    fn moved(&self, v: f64, spool: Spool, b: f64) -> StatorBleedCore {
        match spool {
            Spool::Lp => self.at_point(v, 0.0, b),
            Spool::Hp => self.at_point(0.0, v, b),
        }
    }

    /// THE ROW: bare vs bare-stator vs compensated, carrying the two identities, the trade
    /// relocation, and the other spool's arrow.
    pub fn compensated_point(
        &self, flight: &FlightCondition, tt4: f64, v: f64, spool: Spool,
    ) -> CompensatedPoint {
        let c = self.compensating_bleed(flight, tt4, v, spool, Target::Phi);
        let other = match spool { Spool::Lp => Spool::Hp, Spool::Hp => Spool::Lp };

        let m0 = self.at_point(0.0, 0.0, 0.0);
        let r0 = m0.core.stator_margin(flight, tt4);
        let od0 = m0.core.core.match_point(flight, tt4);
        let sv = self.moved(v, spool, 0.0);
        let rv = sv.core.stator_margin(flight, tt4);
        let odv = sv.core.core.match_point(flight, tt4);

        let mut out = CompensatedPoint {
            spool, tt4, vsv: v,
            b_star: c.b_star(),
            reason: c.reason(),
            phi_bare: r0.spool(spool).phi_op,
            phi_stator: rv.spool(spool).phi_op,
            m_i_bare: r0.spool(spool).m_i,
            m_i_stator: rv.spool(spool).m_i,
            m_phi_bare: r0.spool(spool).m_phi,
            m_phi_stator: rv.spool(spool).m_phi,
            n_bare: r0.spool(spool).n,
            n_stator: rv.spool(spool).n,
            thrust_bare: od0.base.thrust,
            thrust_stator: odv.base.thrust,
            phi_other_bare: r0.spool(other).phi_op,
            d_phi_other_stator: rv.spool(other).phi_op - r0.spool(other).phi_op,
            comp: None,
        };
        let b_star = match c.b_star() {
            Some(b) => b,
            None => return out,
        };
        let cm = self.moved(v, spool, b_star);
        let rc = cm.core.stator_margin(flight, tt4);
        let odc = cm.core.core.match_point(flight, tt4);
        let phi_s0 = self.phi_surge_design(spool);
        let d_m_i = rc.spool(spool).m_i - r0.spool(spool).m_i;
        let d_m_i_pred = v;
        let d_m_phi = rc.spool(spool).m_phi - r0.spool(spool).m_phi;
        let d_m_phi_pred = v * phi_s0 * phi_s0 / (1.0 + v * phi_s0);
        out.comp = Some(Compensated {
            phi_comp: rc.spool(spool).phi_op,
            m_i_comp: rc.spool(spool).m_i,
            m_phi_comp: rc.spool(spool).m_phi,
            n_comp: rc.spool(spool).n,
            thrust_comp: odc.base.thrust,
            d_m_i, d_m_i_pred, d_m_phi, d_m_phi_pred,
            d_m_i_resid: d_m_i - d_m_i_pred,
            d_m_phi_resid: d_m_phi - d_m_phi_pred,
            dn_stator: rv.spool(spool).n / r0.spool(spool).n - 1.0,
            dn_comp: rc.spool(spool).n / r0.spool(spool).n - 1.0,
            d_f_stator: odv.base.thrust / od0.base.thrust - 1.0,
            d_f_comp: odc.base.thrust / od0.base.thrust - 1.0,
            phi_other_comp: rc.spool(other).phi_op,
            d_phi_other_comp: rc.spool(other).phi_op - r0.spool(other).phi_op,
        });
        out
    }

    /// **RUNG 61's HEADLINE OBJECT**: `b*(v)` on BOTH spools across the throttle band.
    ///
    /// The LP spool's valve authority is large and near-constant, so `b*_LP` is finite and mild.
    /// The HP spool's passes through ZERO at `π* = γc^(γc/(γc−1))` and REVERSES below it, so
    /// `b*_HP` is unreachable: the HP stator's φ-debit cannot be bought back at all. NOT a fourth
    /// independent appearance of `π*` — it is rung 42's OWN crossing read in a new currency, which
    /// is why every row carries `pi_hpc`.
    ///
    /// **THE RATIO IS PYTHON'S TRUTHINESS, PORTED AS WRITTEN.** `(bh / bl) if (bl and bh)` treats
    /// an exact `0.0` as absent, which `is not None` would not. § 5.11 (iii) measured no
    /// `b* == 0.0` on any grid swept (196 values, min 8.54e-3), and on the shipped band every row
    /// is mixed — `b_lp` finite, `b_hp` absent — so the trap never decides anything. Spelled
    /// faithfully and recorded LATENT: **fourth instance of a dead thing's spelling still having
    /// to be right.**
    pub fn compensability(
        &self, flight: &FlightCondition, tt4_grid: &[f64], v: f64,
    ) -> Vec<Compensability> {
        let mut rows = Vec::new();
        for &tt4 in tt4_grid {
            // Python's `except AssertionError: continue` — the throttle point is SKIPPED, not
            // reported, so a refused row never reaches the caller at all.
            let od = match self.at_point(0.0, 0.0, 0.0).core.core.try_match_point(flight, tt4) {
                Ok(od) => od,
                Err(Abort(_)) => continue,
            };
            let lp = self.compensating_bleed(flight, tt4, v, Spool::Lp, Target::Phi);
            let hp = self.compensating_bleed(flight, tt4, v, Spool::Hp, Target::Phi);
            let (b_lp, b_hp) = (lp.b_star(), hp.b_star());
            let ratio = match (b_lp, b_hp) {
                // `if (bl and bh)` — BOTH present AND both non-zero.
                (Some(l), Some(h)) if l != 0.0 && h != 0.0 => Some(h / l),
                _ => None,
            };
            rows.push(Compensability {
                tt4, vsv: v, pi_hpc: od.base.pi_hpc, pi_lpc: od.base.pi_lpc,
                b_lp, b_hp,
                why_lp: lp.reason(), why_hp: hp.reason(),
                resid_lp: lp.resid_last(), resid_hp: hp.resid_last(),
                ratio,
            });
        }
        rows
    }

    /// THE SEAM AS WRITTEN, SCORED. Six specs say *"the bleed takes over where the stator's
    /// authority ends"*. Rung 54's [`AuthorityCeiling`] is the instrument for the stator's end;
    /// this runs it at several valve positions. TAKEOVER predicts the ceiling is INDIFFERENT to
    /// the valve — anything else refutes the sequencing picture.
    pub fn authority_with_bleed(
        &self, flight: &FlightCondition, tt4: f64, bleeds: &[f64], spool: Spool,
    ) -> Vec<AuthorityRow> {
        bleeds.iter().map(|&b| {
            let a: AuthorityCeiling =
                self.at_bleed(b).core.authority_ceiling(flight, tt4, spool, None);
            AuthorityRow {
                bleed: b, v_edge: a.v_edge, v_peak: a.v_peak, peak_interior: a.peak_interior,
                m_i_0: a.m_i_0, m_i_peak: a.m_i_peak, m_i_edge: a.m_i_edge,
                span: a.m_i_peak - a.m_i_0, n_scan: a.n_scan,
            }
        }).collect()
    }

    /// P4: *restore the point* and *restore the reported margin* are different instructions, and
    /// the gap between their prices is the floor motion the stator caused. Rung 54 found a
    /// CONSTRAINT'S SEVERITY coordinate-dependent, rung 56 a LEVER'S COST; this asks it of the
    /// PRICE OF UNDOING ONE LEVER WITH ANOTHER.
    pub fn price_split(
        &self, flight: &FlightCondition, tt4: f64, v_grid: &[f64], spool: Spool,
    ) -> Vec<PriceSplit> {
        let phi_s0 = self.phi_surge_design(spool);
        v_grid.iter().map(|&v| {
            let a = self.compensating_bleed(flight, tt4, v, spool, Target::Phi);
            let c = self.compensating_bleed(flight, tt4, v, spool, Target::MPhi);
            let gap = match (a.b_star(), c.b_star()) {
                (Some(x), Some(y)) => Some(x - y),
                _ => None,
            };
            PriceSplit {
                vsv: v, b_phi: a.b_star(), b_m_phi: c.b_star(), gap,
                floor_motion: v * phi_s0 * phi_s0 / (1.0 + v * phi_s0),
                why_phi: a.reason(), why_m_phi: c.reason(),
            }
        }).collect()
    }
}
