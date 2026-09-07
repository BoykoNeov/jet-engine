//! RUNG 75 — **THE DECLARED ANTI-WINDUP DEVICE**: rung 74 § 10's own seam, and the cell rung 74
//! § 4 reports as having no plant.
//!
//! Rung 74 § 4 found rung 52's `max(0, ·)` to be this family's anti-windup device *by accident*,
//! and found that removing it leaves the masked applied-referenced leg with
//! `dw/ds = (cap - mf_app)/tau > 0` and nothing in its path. This rung declares the device the
//! accident was standing in for — back-calculation onto the fuel actually applied:
//!
//! ```text
//! dw/ds = ( target - w ) / tau  +  ( mf_app - w ) / tau_t
//! ```
//!
//! **THE HEADLINE: AN ANTI-WINDUP DEVICE IS DECISIVE ON THE SPECTRUM AND INERT ON THE RANK — the
//! exact inverse of rung 74's coordinate.** The term is STATE-DEPENDENT, so unlike rung 74's
//! forcing it is *in* the Jacobian: it writes `-1/tau_t` onto the masked leg's own diagonal, the
//! one rung 73's applied reference had cancelled to exactly zero. The masked pole LEAVES THE
//! ORIGIN, `zeros` loses `n_masked`, and `det J` — dead since rung 73 — REVIVES. And `n_live`
//! does not move, because the term sits in the masked leg's ROW while the masked COLUMN stays
//! zero.
//!
//! Ported from `tests/test_rung75.py` — **17 collected tests, of which 8 carry `slow` there.**
//! Both numbers are MEASURED (`pytest --collect-only -q`, once plain and once with `-m slow`),
//! never read off a sentence: `rung74.rs`'s header records that a `grep` over-counted by one in
//! slices AC, AD and AE because the extra sat inside a doc comment, and names itself the fourth
//! instance waiting to happen. **Here the `grep` and the collector AGREE at 17**, which is not
//! the same claim as the `grep` being reliable — it is one measurement confirming another. The
//! count below is reconciled against `cargo test --test rung75 -- --list`. The `slow` marker is
//! dropped per slice M's rule; `#[ignore]` is re-introduced only against a MEASURED Rust cost,
//! and the measured cost of this binary's most expensive reader is 4.7 s ([`windup_bill`], five
//! clocks) against a whole-file budget that needs none.
//!
//! # THE PYTHON↔RUST MAP — 1:1 IN ORDER, 0 ADDED, 0 COLLAPSED, 0 SPLIT BY PARAMETER
//!
//! The Python file carries no `@pytest.mark.parametrize`, so nothing splits. Nothing is added
//! either: unlike rung 74, **every rung-75 reader has a caller in the shipped suite** — the six
//! are `_rhs_gains_at` (through `windup_gains`), `_windup_rows`, `windup_gains`,
//! `contraction_law`, `device_control` and `windup_bill`, and gates 4–11 below reach all six. So
//! there is no `latch_discriminator`-shaped hole here and no doc-sourced bar to declare.
//!
//! | # | `tests/test_rung75.py` | here |
//! |---|---|---|
//! | 1 | `reduces_to_rung74_bit_for_bit` | [`reduces_to_rung74_bit_for_bit`] |
//! | 2 | `the_reduce_is_not_vacuous` | [`the_reduce_is_not_vacuous`] |
//! | 3 | `the_cell_rung74_has_no_plant_for_is_reached` | [`the_cell_rung74_has_no_plant_for_is_reached`] |
//! | 4 | `the_pole_leaves_the_origin_and_det_j_revives` | [`the_pole_leaves_the_origin_and_det_j_revives`] |
//! | 5 | `the_incidence_stator_arm_carries_it_too` | [`the_incidence_stator_arm_carries_it_too`] |
//! | 6 | `the_revival_is_applied_only` | [`the_revival_is_applied_only`] |
//! | 7 | `the_masked_rows_coupling_vanishes_at_tau_t_equals_tau` | [`the_masked_rows_coupling_vanishes_at_tau_t_equals_tau`] |
//! | 8 | `the_ic_sweep_converges_at_the_derived_iteration_count` | [`the_ic_sweep_converges_at_the_derived_iteration_count`] |
//! | 9 | `the_ic_cap_is_the_inherited_one_on_every_plant` | [`the_ic_cap_is_the_inherited_one_on_every_plant`] |
//! | 10 | `the_two_devices_burn_identically_and_never_share_a_state` | [`the_two_devices_burn_identically_and_never_share_a_state`] |
//! | 11 | `holding_the_redline_is_a_threshold_on_the_tracking_clock` | [`holding_the_redline_is_a_threshold_on_the_tracking_clock`] |
//! | 12 | `the_device_is_declared` | [`the_device_is_declared`] |
//! | 13 | `track_is_refused_where_a_second_device_is_already_present` | [`track_is_refused_where_a_second_device_is_already_present`] |
//! | 14 | `the_tracking_clock_is_never_defaulted` | [`the_tracking_clock_is_never_defaulted`] |
//! | 15 | `the_rk4_floor_bounds_the_fast_end` | [`the_rk4_floor_bounds_the_fast_end`] |
//! | 16 | `at_lever_carries_all_four_knobs` | [`at_lever_carries_all_four_knobs`] |
//! | 17 | `the_march_MOVES_but_TWO_OF_FOUR_LOOPS_ARE_INERT_at_this_wall` | [`the_march_moves_but_two_of_four_loops_are_inert_at_this_wall`] |
//!
//! # **THIS FILE OVERRIDES READER DEFAULTS AT SIX CALL SITES — THE EXACT OPPOSITE OF RUNG 74's**
//!
//! `rung74.rs`'s header ends *"There is no overridden default anywhere in the Python file —
//! measured by reading all four call sites, and recorded because rung 73's file had exactly
//! one."* Read the same way here, `tests/test_rung75.py` overrides at **six of its eight** reader
//! calls, and the overrides are the SWEEP AXES themselves:
//!
//! | reader | `engine.py` default | what the suite passes | gate |
//! |---|---|---|---|
//! | `windup_gains` | `refs=("applied", "sched")` | `("applied",)` | 4 |
//! | `windup_gains` | `refs=("applied", "sched")` | both, `inc=True` | 5 |
//! | `windup_gains` | `refs=("applied", "sched")` | `("sched",)` | 6 |
//! | `windup_gains` | `tau_ts=(0.05, 0.0125)` | `(0.0125, 0.05)` — **REVERSED** | 7 |
//! | `contraction_law` | `tau_ts=` six clocks | the four SLOWEST | 8 |
//! | `device_control` | `refs=("sched", "applied")` | `("applied",)` | 10 |
//! | `windup_bill` | `tau_ts=` eight clocks | five of them | 11 |
//!
//! **Gate 7's reversal is the one that would be invisible if transcribed as a set.** `tau_ts` is
//! swept in ORDER and [`WindupGains::ratios`] divides the FIRST cell's reading by the second, so
//! `(0.0125, 0.05)` and `(0.05, 0.0125)` produce reciprocal ratios — `2.5` against `0.4`. The
//! suite reads no ratio in that gate, which is exactly why a port could reverse it and stay
//! green while the cell contents swapped underneath the two named lookups. Every tuple below is
//! written in the suite's order, and this table is where that is checked.
//!
//! Everything BELOW the swept argument is still the reader's own default, read off
//! `turbojet/engine.py`'s `def` lines and not off this module's [`DS`]: `taus` is passed at every
//! call and agrees with the default, and `inc`, `r`, `s_settle`, `ds`, `v_max`, `every`, `res0`,
//! `tol` and `ic_cap` are never named. § 5.27.6 (i) burned a slice on transcribing one reader's
//! stride into another, so they are spelled out as [`RDR_*`](RDR_DS) constants here.
//!
//! # THE FIVE NEEDLES ARE PYTHON's LITERALS, AND ONE OF THEM PINS A RUNG AND NOT A SITE
//!
//! `tests/test_rung75.py` uses five `match=` needles. Each is transcribed from
//! `turbojet/engine.py`'s own string — never from `anti_windup.rs` — because slice AF step 4
//! § (c) measured the port's refusal text **four formatting divergences wide** against Python's
//! (`{:.3e}` vs `%e`, `{:?}` vs `!r`), so a needle copied from the Rust side would certify the
//! divergence instead of catching it. The sources are `engine.py:18645` (`ANTI-WINDUP LAW`),
//! `:18630` (`REFUSED outside the plain DEMAND`), `:18636` (`DECLARED, never defaulted`) and
//! `:10538` (rung 65's `RK4 stability region`).
//!
//! **AND `"did not converge"` IS DISCLOSED, NOT REPAIRED.** § 5.31 (v) measured it matching **18
//! messages across `engine.py`**, so gate 3 — the gate that proves rung 74's plantless cell is
//! reached — can say that A refusal fired and cannot say WHICH. That is where a ported refusal
//! can drift while this file stays green. It is left as Python leaves it: narrowing the needle
//! here would make this file stop being a 1:1 map of the suite, and the site question belongs to
//! `slice_ag_cells.rs`, which drives the refusals directly. The other four needles match exactly
//! one message each.
//!
//! # WHAT OVERLAPS `slice_ag_cells.rs` / `slice_ag_laws.rs`, AND WHY IT IS PORTED ANYWAY
//!
//! Steps 1 and 2 already gate close relatives of **12, 13, 14, 16** and of gate 9's `ic_cap`
//! carry. They are ported here regardless, on `rung72.rs`/`rung73.rs`/`rung74.rs`'s precedent:
//! this file's contract is a **1:1 map of the shipped suite**, and a hole in it is invisible to
//! every count on both sides. The files also gate different things — the step files drive the
//! cell directly through `R75_TRIPLE`, these drive it through `stator_march_scoped` with the
//! suite's own arming, which is the entry the suite actually uses.

use std::panic::catch_unwind;

use turbojet::anti_windup::{
    build_anti_windup_cascade, contraction_law, device_control, windup_bill, windup_gains,
    windup_march, ContractionLaw, DeviceCell, DeviceControl, WindupBill, WindupCell,
    WindupCellRead, WindupGains, WindupRatio, WINDUP_LAW_NONE, WINDUP_LAW_TRACK,
};
use turbojet::applied_reference::REF_LAW_APPLIED;
use turbojet::bleed_transient::LeverArm;
use turbojet::demand_coordinate::{build_demand_coordinate_cascade, IC_CAP_DECLARED};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::reference_split::StatorIncidenceLimiter;
use turbojet::shared_actuator::{SharedRigArm, REF_LAW_DEFAULT};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ============================================================================== the grid
//
// `tests/test_rung75.py`'s module constants, verbatim.

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
/// **RUNG 67's imposed redline, VERBATIM through rungs 70–74.**
const TT4_MAX: f64 = 1200.0;
const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;

/// **THE TWO FLOORS, and the split is INHERITED rather than chosen.** `0.80` is rung 74 § 1.3's
/// disclosure one rung on — the only floor at which all four legs ride, so it is where every
/// JACOBIAN is read; `0.76` is rung 74's own both-legs-ride arm, where every TRAJECTORY is
/// marched.
const PHI_JAC: f64 = 0.80;
const PHI_BOTH: f64 = 0.76;

/// **THE ONE NEW CONSTANT, and it is swept, never quoted alone.** `0.0125` is four times faster
/// than the plant's clocks, which is what makes gate 4's `1/tau_t` scaling a ratio of exactly
/// `4.0` rather than a fitted number.
const TAU_T: f64 = 0.05;
const TAU_T_FAST: f64 = 0.0125;

/// `PHI / FLOOR - 1.0` — the EXPRESSION Python spells, never a typed decimal. Rung 69's
/// constructor asserts `m_lim == T_c - 1/phi_lim`, so a rounded constant breaks a wall identity.
fn sm_of(phi_lim: f64) -> f64 { phi_lim / FLOOR - 1.0 }

// ----------------------------------------------------------- THE READERS' OWN DEFAULTS
//
// See the module header's table for what the suite OVERRIDES. These are what it does not.

/// Shared by all four readers, none of them named at any call site: `inc = False`, `r = 0.5`,
/// `s_settle = 1.2`, `ds = 0.005`, `v_max = 0.20`.
const RDR_INC: bool = false;
const RDR_R: f64 = 0.5;
const RDR_SETTLE: f64 = 1.2;
const RDR_DS: f64 = 0.005;
const RDR_V_MAX: f64 = 0.20;
/// `windup_gains(..., every=8)` — the stride, and the ONE argument of the six that changes how
/// many rows a cell holds. `test_rung76.py` names it explicitly and this file never does.
const RDR_EVERY: usize = 8;
/// `contraction_law(..., res0=2.898e-3, tol=1e-12, ic_cap=400)`. **`res0` is RUNG 74's OWN
/// REPORTED RESIDUAL**, not a fit, and `ic_cap = 400` is the only raised cap in the family —
/// gate 9 is what keeps the raise confined to this one reader.
const CL_RES0: f64 = 2.898e-3;
const CL_TOL: f64 = 1e-12;
const CL_IC_CAP: usize = 400;

// ============================================================================== the fixtures

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), PI_LPC, PI_HPC, TT4, 50_000.0, REAL)
}

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// Python's `_rig`'s arming half — the valve always, and EITHER the `phi` stator OR rung 69's
/// incidence one, never both.
fn suite_arm(sm: f64, inc: bool) -> LeverArm {
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: if inc { None }
                    else { Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))) },
        stator_inc: if inc {
            Some(StatorIncidenceLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S)))
        } else { None },
        ..Default::default()
    }
}

/// Python's `_rig(design, AntiWindupTransient, …)`.
///
/// The four knobs are set by PLAIN ASSIGNMENT (`m._lag_coord, m._ref_law = coord, ref` and
/// `m._windup_law, m._tau_t = law, tau_t`), which is what the fixture does — not through
/// `_with_windup`. Routing them through the table here would test a different line, and slice AF
/// step 6's leading finding was four production call sites that had done exactly that.
fn windup_rig(sm: f64, inc: bool, coord: &'static str, ref_law: &'static str,
              law: &'static str, tau_t: Option<f64>) -> ScheduledStatorCore {
    let m = full_of(build_anti_windup_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &suite_arm(sm, inc)));
    m.fuel.inner.lag_coord.set(coord);
    m.fuel.inner.ref_law.set(ref_law);
    m.fuel.inner.windup_law.set(law);
    m.fuel.inner.tau_t.set(tau_t);
    m
}

/// The default arm of Python's `_rig`: `coord="demand"`, `ref="sched"`, `law="none"`,
/// `tau_t=None`.
fn windup_default(sm: f64, inc: bool) -> ScheduledStatorCore {
    windup_rig(sm, inc, "demand", REF_LAW_DEFAULT, WINDUP_LAW_NONE, None)
}

/// Python's `_rig(design, DemandCoordinateTransient, …)` — the PARENT, gate 1's other side.
/// Python sets no windup knob on this class, and neither does this.
fn demand_rig(sm: f64, coord: &'static str, ref_law: &'static str) -> ScheduledStatorCore {
    let m = full_of(build_demand_coordinate_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &suite_arm(sm, false)));
    m.fuel.inner.lag_coord.set(coord);
    m.fuel.inner.ref_law.set(ref_law);
    m
}

fn surge_of(sm: f64) -> SurgeLimiter { SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm) }
fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }

/// Python's `_march(m, sm)` — all four loops, at the suite's own ramp, with NO accel schedule.
fn march(m: &ScheduledStatorCore, sm: f64) -> Vec<FuelPoint> {
    let leg = StatorLeg {
        accel: None::<&AccelSchedule>,
        surge: Some(Floor::Phi(surge_of(sm))),
        tt4_max: Some(TT4_MAX),
    };
    let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS };
    m.stator_march_scoped(&flight(), &ramp, None, &leg,
                          &MarchScope { lag: Some(lag()), tau_gov: Some(TAU_GOV),
                                        ..MarchScope::DEFAULT }).0
}

/// Python's `_keys(traj)` — compared BIT for bit.
///
/// **THE LENGTH IS PER POINT AND THAT IS PYTHON's LINE, NOT A CONVENIENCE.** The comprehension is
/// `tuple(p[k] for k in ks if k in p)`, so a `clip` trajectory — which dispatches out of this
/// ladder into rung 73 and records no `w_fuel`/`w_gov` — yields NINE where a `demand` one yields
/// ELEVEN. A fixed-width port would either panic on the clip arm or pad it with a value the
/// integrator never produced, and gate 1 compares a clip arm against a clip arm.
fn keys(traj: &[FuelPoint]) -> Vec<Vec<u64>> {
    traj.iter().map(|p| {
        let mut v = vec![p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(),
                         p.phi_lp.to_bits(), p.phi_hp.to_bits(), p.tt4.to_bits(),
                         p.mf.to_bits()];
        match p.extra {
            PointExtra::Shared { b, v: vv, .. } => { v.push(b.to_bits()); v.push(vv.to_bits()); }
            PointExtra::Demand { b, v: vv, w_fuel, w_gov, .. } => {
                v.push(b.to_bits());
                v.push(vv.to_bits());
                v.push(w_fuel.to_bits());
                v.push(w_gov.to_bits());
            }
            _ => panic!("rung-75's reduce compares marches that record `b` and `v`."),
        }
        v
    }).collect()
}

/// Python's `pytest.raises(AssertionError, match=…)` — the message, or `""` on a clean return.
fn message_of<F: FnOnce()>(f: F) -> String {
    match catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(()) => String::new(),
        Err(e) => match e.downcast_ref::<String>() {
            Some(s) => s.clone(),
            None => e.downcast_ref::<&str>().map(|s| (*s).to_string())
                     .unwrap_or_else(|| "<non-string panic>".into()),
        },
    }
}

/// Python's `out["cells"][f"{ref}|{tau_t}"]`, resolved off the PAIR the port carries.
///
/// A cell that came back [`WindupCell::Empty`] panics naming its `n_riding`, rather than being
/// skipped: every gate below asserts `n >= 3` or more, and an empty cell silently treated as
/// "nothing to check" is how a gate over a filtered sample goes vacuous.
fn cell<'a>(g: &'a WindupGains, ref_law: &str, tau_t: f64) -> &'a WindupCellRead {
    let (_, c) = g.cells.iter()
        .find(|((r, t), _)| *r == ref_law && *t == tau_t)
        .unwrap_or_else(|| panic!("`windup_gains` published no cell {ref_law}|{tau_t}"));
    match c {
        WindupCell::Read(b) => b,
        WindupCell::Empty { n_riding } =>
            panic!("cell {ref_law}|{tau_t} is EMPTY ({n_riding} riding points)"),
    }
}

/// Python's `out["ratios"][ref]`.
fn ratio<'a>(g: &'a WindupGains, ref_law: &str) -> &'a WindupRatio {
    g.ratios.iter().find(|x| x.ref_law == ref_law)
        .unwrap_or_else(|| panic!("`windup_gains` published no ratio for {ref_law}"))
}

/// Python's `out["cells"][f"{ref}|{tau_t}"]` on [`device_control`]'s flat list.
fn dev_cell<'a>(d: &'a DeviceControl, ref_law: &str, tau_t: f64) -> &'a DeviceCell {
    d.cells.iter().find(|c| c.ref_law == ref_law && c.tau_t == tau_t)
        .unwrap_or_else(|| panic!("`device_control` published no cell {ref_law}|{tau_t}"))
}

// ============================================================================== the readers
//
// One wrapper per reader, so the six defaults above are written ONCE and every gate below reads
// as its Python twin does.

fn gains(m: &ScheduledStatorCore, phi_lim: f64, tau_ts: &[f64], refs: &[&'static str],
         inc: bool) -> WindupGains {
    windup_gains(m, &flight(), LO, HI, TT4_MAX, phi_lim, TAUS, tau_ts, refs, inc,
                 RDR_R, RDR_SETTLE, RDR_DS, RDR_V_MAX, RDR_EVERY)
}

fn contraction(m: &ScheduledStatorCore, phi_lim: f64, tau_ts: &[f64]) -> ContractionLaw {
    contraction_law(m, &flight(), LO, HI, TT4_MAX, phi_lim, TAUS, tau_ts, CL_RES0, CL_TOL,
                    CL_IC_CAP, RDR_INC, RDR_R, RDR_SETTLE, RDR_DS, RDR_V_MAX)
}

fn control(m: &ScheduledStatorCore, phi_lim: f64, tau_ts: &[f64],
           refs: &[&'static str]) -> DeviceControl {
    device_control(m, &flight(), LO, HI, TT4_MAX, phi_lim, TAUS, tau_ts, refs, RDR_INC,
                   RDR_R, RDR_SETTLE, RDR_DS, RDR_V_MAX)
}

fn bill(m: &ScheduledStatorCore, phi_lim: f64, tau_ts: &[f64], ref_law: &'static str)
        -> WindupBill {
    windup_bill(m, &flight(), LO, HI, TT4_MAX, phi_lim, TAUS, tau_ts, ref_law, RDR_INC,
                RDR_R, RDR_SETTLE, RDR_DS, RDR_V_MAX)
}

// ======================================================================================
// THE REDUCE SPINE — TWO arms, both by DISPATCH, because `_windup_law = 'none'` is not a
// limit of anything: the hook's branch is simply not taken and the floats are rung 74's.
// That is a STRONGER reduce than rung 74's own second arm (a tolerance), and it is only
// available because this rung reuses its parent's march instead of siring one.
// ======================================================================================

/// **ARM 1**: the same machine, `_windup_law = 'none'`, on the two cells rung 74 HAS.
#[test]
fn reduces_to_rung74_bit_for_bit() {
    for (coord, ref_law) in [("clip", REF_LAW_APPLIED),
                             ("demand", REF_LAW_DEFAULT),
                             ("demand-latched", REF_LAW_APPLIED)] {
        let sm = sm_of(PHI_BOTH);
        let a = keys(&march(&windup_rig(sm, false, coord, ref_law, WINDUP_LAW_NONE, None), sm));
        let b = keys(&march(&demand_rig(sm, coord, ref_law), sm));
        assert_eq!(a, b, "{coord}|{ref_law}");
    }
}

/// **ARM 1 MUST BE A TEST, NOT A TAUTOLOGY** (rung 73's `charpoly_selftest` discipline, rung 74's
/// `the_clip_reduce_is_not_vacuous`): if `_windup_law` were ignored, the reduce above would
/// compare rung 74 with rung 74 and pass. The SAME machine under `track` must DIFFER.
#[test]
fn the_reduce_is_not_vacuous() {
    let sm = sm_of(PHI_BOTH);
    let a = keys(&march(&windup_default(sm, false), sm));
    let b = keys(&march(
        &windup_rig(sm, false, "demand", REF_LAW_DEFAULT, WINDUP_LAW_TRACK, Some(TAU_T)), sm));
    assert_ne!(a, b);
}

/// **ARM 2 — and it is the REASON this rung exists.** `demand × applied` has no interior
/// equilibrium without a stop (rung 74 § 4); with the device declared, it marches.
///
/// The needle is `"did not converge"`, and the module header records what it can and cannot say:
/// it matches 18 messages across `engine.py`, so this gate pins the RUNG and not the SITE.
#[test]
fn the_cell_rung74_has_no_plant_for_is_reached() {
    let sm = sm_of(PHI_BOTH);
    let msg = message_of(|| {
        let m = windup_rig(sm, false, "demand", REF_LAW_APPLIED, WINDUP_LAW_NONE, None);
        march(&m, sm);
    });
    assert!(msg.contains("did not converge"), "{msg}");

    let m = windup_rig(sm, false, "demand", REF_LAW_APPLIED, WINDUP_LAW_TRACK, Some(TAU_T));
    let traj = march(&m, sm);
    assert!(traj.len() > 300, "{}", traj.len());
    assert!(ic_res_of(&traj[0]) <= 1e-12, "{}", ic_res_of(&traj[0]));
}

/// Python's `traj[0]["ic_res"]` — a `KeyError` off a trajectory that records no joint solve,
/// which is a panic here.
fn ic_res_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Shared { ic_res, .. } | PointExtra::Demand { ic_res, .. } => ic_res,
        _ => panic!("rung-75's ARM 2 reads `ic_res` off the first marched point."),
    }
}

// ======================================================================================
// § 1 — THE JACOBIAN. Read through `_rhs_laws` (the DERIVATIVE), never `_jac4` (the
// TARGET): the anchor measures that a target-differencing reader is BLIND to this rung's
// whole subject and would have returned a perfect refutation of its headline.
// ======================================================================================

/// **P1/P2/P3/P4 — the headline, all four faces, from ONE reader call.**
///
/// P1 the masked diagonal IS `-1/tau_t`; P2 the authoritative one is UNMOVED and the device is
/// the zero FUNCTION there; P3 the masked COLUMN is untouched so `n_live <= 3` a FOURTH time;
/// P4 `det J` dead → alive, `zeros` 1 → 0, and both scale as `1/tau_t`.
#[test]
fn the_pole_leaves_the_origin_and_det_j_revives() {
    let m = windup_default(sm_of(PHI_JAC), false);
    let out = gains(&m, PHI_JAC, &[TAU_T, TAU_T_FAST], &[REF_LAW_APPLIED], RDR_INC);
    for tau_t in [TAU_T, TAU_T_FAST] {
        let c = cell(&out, REF_LAW_APPLIED, tau_t);
        assert!(c.n >= 5, "{}", c.n);
        // P1 — the diagonal the device writes, and the one rung 73 sent to the ORIGIN.
        // `diag_err` is the residual against `-1/tau_t`, SCALED BY `tau_t` so a fast clock's
        // larger entry is not flattered. It is a CENTRAL DIFFERENCE and lands at
        // -19.9999999999 against -20, which is the instrument measuring rather than asserting.
        assert!(c.diag_err < 1e-9, "{:?}", c.masked_diag);
        for x in [c.masked_diag.0, c.masked_diag.1] {
            assert!((x * tau_t + 1.0).abs() < 1e-9, "{x}");
        }
        assert!(c.masked_diag0.0.abs().max(c.masked_diag0.1.abs()) < 1e-9,
                "{:?}", c.masked_diag0);
        // P2 — the leg that HOLDS is untouched, and the device is exactly zero on it
        assert_eq!(c.auth_diag_moved, 0.0);
        assert_eq!(c.track_leak, 0.0);
        // P3 — `n_live <= 3` stands, a FOURTH time
        assert_eq!(c.mask_leak, 0.0);
        assert_eq!(c.mask_leak0, 0.0);
        // P4 — the determinant, and the zero count
        assert!(c.det0_alive < 1e-9, "{}", c.det0_alive);
        assert!(c.det_alive > 1.0, "{}", c.det_alive);
        assert_eq!(c.zeros.as_slice(), [0]);
        assert_eq!(c.zeros0.as_slice(), [1]);
    }
    // and BOTH scale as `1/tau_t`, which is what block-triangularity means
    let rr = ratio(&out, REF_LAW_APPLIED);
    for (name, (lo, hi)) in [("diag", rr.diag), ("det", rr.det)] {
        assert!((lo - 4.0).abs() < 1e-6 && (hi - 4.0).abs() < 1e-6, "{name} {lo} {hi}");
    }
}

/// **AND ON RUNG 69's INCIDENCE STATOR**, which was going to be a concession until measuring it
/// turned out cheaper than writing one. The device acts on the two FUEL-SIDE legs, whose laws
/// never mention the stator's coordinate — now measured rather than argued.
#[test]
fn the_incidence_stator_arm_carries_it_too() {
    let m = windup_default(sm_of(PHI_JAC), true);
    let out = gains(&m, PHI_JAC, &[TAU_T, TAU_T_FAST],
                    &[REF_LAW_APPLIED, REF_LAW_DEFAULT], true);
    for (ref_law, zeros0) in [(REF_LAW_APPLIED, 1usize), (REF_LAW_DEFAULT, 0usize)] {
        for tau_t in [TAU_T, TAU_T_FAST] {
            let c = cell(&out, ref_law, tau_t);
            assert!(c.n >= 3, "{ref_law}|{tau_t} n={}", c.n);
            assert!(c.diag_err < 1e-9, "{}", c.diag_err);
            assert!(c.row_err < 1e-9, "{}", c.row_err);
            assert_eq!(c.mask_leak, 0.0);
            assert_eq!(c.track_leak, 0.0);
            assert_eq!(c.auth_diag_moved, 0.0);
            assert_eq!(c.zeros0.as_slice(), [zeros0]);
            assert_eq!(c.zeros.as_slice(), [0]);
        }
    }
    for (ref_law, want) in [(REF_LAW_APPLIED, 4.0), (REF_LAW_DEFAULT, 2.5)] {
        let rr = ratio(&out, ref_law);
        for (name, pair) in [("diag", rr.diag), ("det", rr.det)] {
            assert!((pair.0 - want).abs() < 1e-6, "{ref_law} {name} {} vs {want}", pair.0);
        }
    }
}

/// **P5 — ONE mechanism with TWO faces, not two findings.** Under `sched` the masked diagonal was
/// ALREADY `-1/tau` (the target is `cap`, which contains no `w`), so nothing was ever dead there
/// and the device merely ADDS THE RATES — rung 66's identity in a fifth shape.
#[test]
fn the_revival_is_applied_only() {
    let m = windup_default(sm_of(PHI_JAC), false);
    let out = gains(&m, PHI_JAC, &[TAU_T, TAU_T_FAST], &[REF_LAW_DEFAULT], RDR_INC);
    for tau_t in [TAU_T, TAU_T_FAST] {
        let c = cell(&out, REF_LAW_DEFAULT, tau_t);
        assert!(c.diag_err < 1e-9, "{}", c.diag_err);
        assert_eq!(c.zeros.as_slice(), [0]);          // alive in BOTH
        assert_eq!(c.zeros0.as_slice(), [0]);
        assert!(c.det0_alive > 1.0, "{}", c.det0_alive);
        assert!(c.det_alive > 1.0, "{}", c.det_alive);
        assert_eq!(c.mask_leak, 0.0);
        assert_eq!(c.track_leak, 0.0);
    }
    // NOT 4.0 here, and that is the point: the diagonal is `-(1/tau + 1/tau_t)`, so the ratio
    // is 100/40 = 2.5 — and `det J` follows it exactly, block-triangularity again.
    let rr = ratio(&out, REF_LAW_DEFAULT);
    for (name, (lo, hi)) in [("diag", rr.diag), ("det", rr.det)] {
        assert!((lo - 2.5).abs() < 1e-6 && (hi - 2.5).abs() < 1e-6, "{name} {lo} {hi}");
    }
}

/// **P6** — `dRHS_masked/dw_auth = 1/tau_t - 1/tau_masked` under `applied`, so the masked leg
/// stops reading the authoritative one EXACTLY when the two clocks match, and reads it with the
/// OTHER SIGN on either side. Under `sched` it is `+1/tau_t` where rung 74 measured exactly `0`.
///
/// **THE `tau_ts` TUPLE IS REVERSED HERE** — `(TAU_T_FAST, TAU_T)`, the suite's own order. See
/// the module header: the cells are looked up by name so the reversal changes no assertion in
/// this gate, which is exactly why it has to be transcribed rather than normalised.
#[test]
fn the_masked_rows_coupling_vanishes_at_tau_t_equals_tau() {
    let m = windup_default(sm_of(PHI_JAC), false);
    let out = gains(&m, PHI_JAC, &[TAU_T_FAST, TAU_T],
                    &[REF_LAW_APPLIED, REF_LAW_DEFAULT], RDR_INC);
    // **AN EMPTY CELL IS A FAILURE HERE, NOT A SKIP.** Python's loop reads `c["row_err"]` off
    // every cell and an EMPTY one carries no such key, so it raises — an `if let` that walked
    // past the `Empty` variant would be strictly weaker than the suite on exactly the arm where
    // a port defect empties a sample. The count is asserted first for the same reason: a loop
    // over nothing passes.
    assert_eq!(out.cells.len(), 4, "2 refs x 2 clocks");
    for ((ref_law, tau_t), c) in &out.cells {
        match c {
            WindupCell::Read(c) => assert!(c.row_err < 1e-9, "{ref_law}|{tau_t} {}", c.row_err),
            WindupCell::Empty { n_riding } =>
                panic!("cell {ref_law}|{tau_t} is EMPTY ({n_riding} riding)"),
        }
    }
    // the SIGN CHANGE: fast tracking reads the authoritative leg POSITIVE, and at
    // `tau_t = tau_masked` the entry is exactly zero
    let fast = cell(&out, REF_LAW_APPLIED, TAU_T_FAST);
    let same = cell(&out, REF_LAW_APPLIED, TAU_T);
    let min_fast = fast.rows.iter().map(|r| r.row_auth).fold(f64::INFINITY, f64::min);
    assert!(min_fast > 0.0, "{min_fast}");
    let min_same = same.rows.iter().map(|r| r.row_auth.abs()).fold(f64::INFINITY, f64::min);
    assert_eq!(min_same, 0.0);
    // under `sched` rung 74 had NO coupling at all there
    assert_eq!(cell(&out, REF_LAW_DEFAULT, TAU_T).row_auth0, (0.0, 0.0));
}

// ======================================================================================
// § 2 — THE CONTRACTION: rung 74's own residual, EXPLAINED and CORRECTED.
// ======================================================================================

/// **P7** — `ceil(ln(tol/res0)/ln sigma)` with `sigma = tau_t/(tau + tau_t)`, `res0` RUNG 74's
/// OWN REPORTED RESIDUAL and `tol` the inherited one. **Zero fitted constants.**
///
/// So rung 74 § 4's `2.898e-3` was never a solver failing to find a plant — it was this same
/// geometric contraction at `sigma = 1`, where the residual has nowhere to go. Rung 74's VERDICT
/// stands (`tau_t -> inf` gives `w* -> inf`, no finite equilibrium); its NUMBER is explained, and
/// the `exists / does not exist` boundary is the 60-iteration cap cutting a geometric sequence.
#[test]
fn the_ic_sweep_converges_at_the_derived_iteration_count() {
    let m = windup_default(sm_of(PHI_BOTH), false);
    let out = contraction(&m, PHI_BOTH, &[0.4, 0.2, 0.1, 0.05]);
    assert_eq!(out.n, 4);
    assert!(out.all_exact, "{:?}", out.rows);
    // and the two SLOWEST arms are exactly the ones the inherited cap cannot reach — the
    // boundary is the solver's, and it is now a NUMBER rather than an artifact
    let within: Vec<bool> = out.rows.iter().map(|x| x.within_inherited_cap).collect();
    assert_eq!(within, vec![false, false, true, true]);
}

/// **AND THE CAP IS RAISED IN A READER ONLY.** `_ic_cap` is a class default of 60 everywhere; if
/// a plant ever carried a raised one, § 2's boundary would be this rung's choice rather than the
/// inherited solver's.
#[test]
fn the_ic_cap_is_the_inherited_one_on_every_plant() {
    assert_eq!(IC_CAP_DECLARED, 60);
    let sm = sm_of(PHI_BOTH);
    assert_eq!(demand_rig(sm, "demand", REF_LAW_DEFAULT).fuel.inner.ic_cap.get(),
               IC_CAP_DECLARED);
    let t = windup_default(sm, false);
    assert_eq!(t.fuel.inner.ic_cap.get(), IC_CAP_DECLARED);
    assert_eq!(t.bare_lever().fuel.inner.ic_cap.get(), IC_CAP_DECLARED);
    let arm = SharedRigArm {
        sm, tau: TAU, tau_s: TAU_S, v_max: V_MAX, tt4_max: TT4_MAX,
        ..SharedRigArm::default()
    };
    let (rig, _, _) = (t.triple_hooks().shared_rig)(&t, &arm);
    assert_eq!(rig.fuel.inner.ic_cap.get(), IC_CAP_DECLARED);
}

// ======================================================================================
// § 3 — THE ACCIDENT AND THE DEVICE. This is where this rung's own P8 died.
// ======================================================================================

/// **P8, REFUTED AS STATED AND GATED AS MEASURED.**
///
/// The anchor predicted the two devices COINCIDE where no leg is cutting, because there
/// `mf_app = mf_sched` and the tracker pulls to where the latch clamps. That is wrong on the
/// state and right on the output: the tracking term pulls toward `mf_app`, but the TARGET term
/// still pushes toward `cap`, and `cap > mf_sched` (rung 74 § 0.2 measures `1.303x` at `s = 0`),
/// so the balance sits ABOVE the schedule while the latch clamps AT it.
///
/// What is true is a DISTINCTION: the OUTPUT agrees to 0.0 exactly, the STATE never agrees at
/// all, and the state gap follows the park law's `tau_t/tau`.
#[test]
fn the_two_devices_burn_identically_and_never_share_a_state() {
    let m = windup_default(sm_of(PHI_BOTH), false);
    let out = control(&m, PHI_BOTH, &[TAU_T, TAU_T_FAST], &[REF_LAW_APPLIED]);
    let mut gaps = Vec::new();
    for tau_t in [TAU_T, TAU_T_FAST] {
        let c = dev_cell(&out, REF_LAW_APPLIED, tau_t);
        assert!(c.n_dormant >= 1, "{}", c.n_dormant);
        assert!(c.n_cutting > 100, "{}", c.n_cutting);
        let dormant_output = c.dormant_output.expect("a dormant sample exists");
        let dormant_state = c.dormant_state.expect("a dormant sample exists");
        let cutting_output = c.cutting_output.expect("a cutting sample exists");
        assert_eq!(dormant_output, 0.0);              // HELD, exactly
        assert!(dormant_state > 1e-6, "{dormant_state}");   // REFUTED, and by the park law
        assert!(cutting_output > 100.0, "{cutting_output}");  // different plants
        gaps.push(dormant_state);
    }
    // THE REFUTATION'S OWN MECHANISM: the gap IS `(tau_t/tau) * (cap - mf_app)`, so halving
    // the clock four times over quarters it
    assert!((gaps[0] / gaps[1] - 4.0).abs() < 1e-6, "{:?}", gaps);
}

// ======================================================================================
// § 4 — THE BILL. A threshold on the one constant this rung adds.
// ======================================================================================

/// **P9/P10 — rung 47's headline concession, third layer.**
///
/// Rung 47: *a lagged governor breaks the redline hold.* Rung 74: that is a property of the
/// COORDINATE, not the lag. Rung 75: **within the demand coordinate it is a THRESHOLD ON
/// `tau_t`** — the one constant this rung adds and cannot derive. Rung 54's shape, on a clock.
///
/// And the hand-over is MONOTONE INCREASING in `tau_t` with the fast end earliest (P9a). The span
/// inside the sweep is two grid cells; the statement with magnitude is against the ACCIDENT,
/// which hands over at 1.065 against 0.695–0.705.
#[test]
fn holding_the_redline_is_a_threshold_on_the_tracking_clock() {
    let m = windup_default(sm_of(PHI_BOTH), false);
    let out = bill(&m, PHI_BOTH, &[0.0125, 0.05, 0.0625, 0.075, 0.1], REF_LAW_APPLIED);
    // THE THRESHOLD, and it is bracketed rather than quoted
    assert_eq!(out.tau_t_holds, Some(0.0625));
    assert_eq!(out.tau_t_breaks, Some(0.075));
    let rh = out.ratio_holds.expect("a holding clock exists");
    let rb = out.ratio_breaks.expect("a breaking clock exists");
    assert!(1.0 < rh && rh < rb && rb < 2.0, "{rh} {rb}");
    assert!(out.tt4_monotone);
    // P9a — monotone, fast end earliest
    assert!(out.handover_monotone);
    let (lo_h, hi_h) = out.handover_span.expect("some row hands over");
    assert!(lo_h <= hi_h, "{lo_h} {hi_h}");
    // AND THE DEVICE BEATS THE ACCIDENT ON BOTH CURRENCIES BY MUCH MORE THAN `tau_t` MOVES
    // EITHER: ~160 K of redline and ~0.36 of hand-over
    let acc = &out.accident;
    assert!(acc.over > 100.0, "{}", acc.over);
    let acc_hand = acc.handover.expect("the accident hands over");
    assert!(acc_hand > hi_h + 0.3, "{acc_hand} vs {hi_h}");
    for x in &out.rows {
        assert!(x.max_tt4 < acc.max_tt4 - 100.0, "{} vs {}", x.max_tt4, acc.max_tt4);
    }
}

// ======================================================================================
// THE KNOB IS DECLARED, AND THE TWO REFUSALS ARE REFUSALS.
// ======================================================================================

/// The needle `"ANTI-WINDUP LAW"` is Python's `match=`, and the message it matches is
/// `turbojet/engine.py:18645`'s literal — transcribed from THERE, not from `anti_windup.rs`, for
/// the module header's reason.
#[test]
fn the_device_is_declared() {
    let sm = sm_of(PHI_BOTH);
    let msg = message_of(|| {
        let t = windup_rig(sm, false, "demand", REF_LAW_DEFAULT, "reset", Some(TAU_T));
        march(&t, sm);
    });
    assert!(msg.contains("ANTI-WINDUP LAW"), "{msg}");
}

/// `clip` still carries rung 52's `max(0, ·)` and `demand-latched` carries the latch, so either
/// cell would run TWO anti-windup devices at once — rung 63's change-one-law-at-a-time, which
/// rung 74 § 2 records itself breaking in a `for` loop.
///
/// The needle is `turbojet/engine.py:18630`'s literal.
#[test]
fn track_is_refused_where_a_second_device_is_already_present() {
    let sm = sm_of(PHI_BOTH);
    for coord in ["clip", "demand-latched"] {
        let msg = message_of(|| {
            let t = windup_rig(sm, false, coord, REF_LAW_APPLIED, WINDUP_LAW_TRACK, Some(TAU_T));
            march(&t, sm);
        });
        assert!(msg.contains("REFUSED outside the plain DEMAND"), "{coord}: {msg}");
    }
}

/// The needle is `turbojet/engine.py:18636`'s literal.
///
/// **THE THIRD BAD VALUE IS `None`, AND IT IS NOT THE SAME TEST AS THE OTHER TWO.** Python sweeps
/// `(None, 0.0, -0.05)`; here `tau_t` is `Option<f64>`, so `None` exercises the presence half of
/// the refusal and the two floats its positivity half. A port that folded `None` into a sentinel
/// `0.0` would collapse the two into one and the sweep would still pass.
#[test]
fn the_tracking_clock_is_never_defaulted() {
    let sm = sm_of(PHI_BOTH);
    for bad in [None, Some(0.0), Some(-0.05)] {
        let msg = message_of(|| {
            let t = windup_rig(sm, false, "demand", REF_LAW_DEFAULT, WINDUP_LAW_TRACK, bad);
            march(&t, sm);
        });
        assert!(msg.contains("DECLARED, never defaulted"), "{bad:?}: {msg}");
    }
}

/// **ANCHOR § 0.4**: the device adds `1/tau_t` to each of TWO fuel-side diagonals, so the
/// inherited `ds*sum(1/tau_i) <= 2` admits `tau_t >= 2*ds/(2 - ds*sum) = 0.00625` at this grid.
/// Perfect tracking is not reachable here and is not claimed — and the constant is not loosened
/// to reach it (rung 65's lesson).
///
/// The needle is rung 65's own message at `turbojet/engine.py:10538`, and § 5.31 (v) measured it
/// matching exactly ONE message file-wide — so unlike gate 3's, this one pins a site.
///
/// **THE ADMITTED VALUE IS SPELLED AS THE DERIVATION**, not as `0.00625`, because the grid runs
/// ON its own admissibility boundary and the two sides of that comparison must not drift apart.
#[test]
fn the_rk4_floor_bounds_the_fast_end() {
    let sm = sm_of(PHI_BOTH);
    let floor = 2.0 * DS / (2.0 - DS * (4.0 / TAUS.0));
    assert_eq!(floor, turbojet::anti_windup::WINDUP_TAU_GRID_FLOOR);
    let t = windup_rig(sm, false, "demand", REF_LAW_APPLIED, WINDUP_LAW_TRACK, Some(floor));
    assert!(!march(&t, sm).is_empty(), "the floor itself is ADMITTED");
    let msg = message_of(|| {
        let t = windup_rig(sm, false, "demand", REF_LAW_APPLIED, WINDUP_LAW_TRACK, Some(0.005));
        march(&t, sm);
    });
    assert!(msg.contains("RK4 stability region"), "{msg}");
}

/// **THE THIRTEENTH INSTANCE of the trap rungs 61–74 each hit** — and it bit again during this
/// rung's build: `_ic_cap` was set on the outer rig, `_shared_rig` returned a fresh machine
/// without it, and § 2's two slowest arms reported ASSERT instead of 185 and 98.
#[test]
fn at_lever_carries_all_four_knobs() {
    let sm = sm_of(PHI_BOTH);
    let t = windup_rig(sm, false, "demand", REF_LAW_APPLIED, WINDUP_LAW_TRACK, Some(TAU_T_FAST));
    let arm = SharedRigArm {
        sm, tau: TAU, tau_s: TAU_S, v_max: V_MAX, tt4_max: TT4_MAX,
        ..SharedRigArm::default()
    };
    let (rig, _, _) = (t.triple_hooks().shared_rig)(&t, &arm);
    for m in [t.bare_lever(), rig] {
        let i = &m.fuel.inner;
        assert_eq!((i.share_law.get(), i.ref_law.get(), i.lag_coord.get()),
                   ("max", REF_LAW_APPLIED, "demand"));
        assert_eq!((i.windup_law.get(), i.tau_t.get()),
                   (WINDUP_LAW_TRACK, Some(TAU_T_FAST)));
    }
}

// --- THE MARCH AUDIT: rung 79's gap seam, checked from the other end ------------------------
//
// `docs/rungs72-77-march-audit.md`. A CONFIRMATION's gate, not this rung's anchor.

/// `docs/rung79-gap-margin.md` proved rungs 78/79's marches never leave their initial state and
/// flagged that this rung SHARES THE RIG. **It does not stand still** — it marches at
/// `PHI_BOTH = 0.76`, and the arrest is confined to `(demand, 0.80)`, rung 74 § 2.2's arm.
///
/// **THE SECOND HALF IS THE FINDING AND IT IS PINNED AS AN EQUALITY.** At this wall the VALVE and
/// the STATOR never act, 0 of 341 steps each: [`windup_bill`]'s trajectories are a TWO-loop plant
/// (rung 47's governor + rung 49's `phi` leg), not the four-loop one this rung's other sections
/// read at `PHI_JAC`. Nothing shipped is voided — this rung's claims are about the fuel-side
/// device — but the scope is now recorded rather than assumed.
///
/// **THE CELL IS `windup_bill`'s OWN**, not the one an audit reaches for by default: the bill
/// marches `("demand", "applied", "track", tau_t)`. `("demand", "sched", "none")` is this rung's
/// REDUCE cell — it IS rung 74 — and gating there would have measured the parent.
#[test]
fn the_march_moves_but_two_of_four_loops_are_inert_at_this_wall() {
    let sm = sm_of(PHI_BOTH);
    let m = windup_default(sm, false);
    let traj = windup_march(&m, &flight(), LO, HI, TT4_MAX, sm, TAUS, R, SETTLE, DS, V_MAX,
                            false, "demand", REF_LAW_APPLIED, WINDUP_LAW_TRACK, Some(TAU_T),
                            None).3;
    assert!(traj.len() > 300, "{}", traj.len());
    let nu: Vec<f64> = traj.iter().map(|p| p.nu_lp).collect();
    let (nu_lo, nu_hi) = (nu.iter().copied().fold(f64::INFINITY, f64::min),
                          nu.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    assert!((nu_hi - nu_lo) / nu_lo > 1e-2, "{nu_lo} {nu_hi}");
    let t4: Vec<f64> = traj.iter().map(|p| p.tt4).collect();
    let (t4_lo, t4_hi) = (t4.iter().copied().fold(f64::INFINITY, f64::min),
                          t4.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    assert!(t4_hi - t4_lo > 100.0, "{t4_lo} {t4_hi}");
    // THE TWO INERT LOOPS — an equality, because the dormancy is what is being recorded
    let b_max = m.fuel.inner.lever.lim.expect("the suite arms a valve").b_max;
    assert_eq!(traj.iter().filter(|p| { let b = b_cmd_of(p); b > 0.0 && b < b_max }).count(), 0);
    assert_eq!(traj.iter().filter(|p| v_regime_of(p) == Some(Regime::Riding)).count(), 0);
    // THE TWO LIVE ONES, the second against the free droop
    assert!(traj.iter().filter(|p| required_of(p) > 0.0).count() > 300);
    let min_phi = traj.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min);
    assert!(min_phi > PHI_BOTH, "{min_phi}");
    assert!(min_phi > 0.755, "the phi leg would be dormant below the droop: {min_phi}");
}

fn b_cmd_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Shared { b_cmd, .. } | PointExtra::Demand { b_cmd, .. } => b_cmd,
        _ => panic!("rung-75's audit reads `b_cmd` off every marched point."),
    }
}

fn v_regime_of(p: &FuelPoint) -> Option<Regime> {
    match p.extra {
        PointExtra::Shared { v_regime, .. } | PointExtra::Demand { v_regime, .. } => v_regime,
        _ => panic!("rung-75's audit reads `v_regime` off every marched point."),
    }
}

fn required_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Shared { required, .. } | PointExtra::Demand { required, .. } => required,
        _ => panic!("rung-75's audit reads `required` off every marched point."),
    }
}
