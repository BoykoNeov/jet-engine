//! RUNG 76 — **THE FUEL-DEPENDENT CAP**: rung 73 § 11's second seam, deferred by rungs 73, 74
//! AND 75.
//!
//! Every cap in this family is a SET-POINT SOLVE, so it is a function of the STATE alone and
//! `d(cap)/d(mf) = 0` — which is what collapses rung 73's applied reference from a continuum to
//! three readings. Rung 48's `Wf/pt3` leg is the ONE whose law is not a solve: its own docstring
//! states it as an inequality ON THE FUEL, and a real limiter EVALUATES that from the delivery
//! pressure it senses.
//!
//! ```text
//! solve    cap = w*  with  w* = (1+margin)*kappa(n_H(w*))*pt3(w*)   -- rung 48, shipped
//! sensed   cap(w) = (1+margin)*kappa(n_H(w))*pt3(w)  at  w = mf_app -- AS WRITTEN
//! ```
//!
//! **THE HEADLINE: a device in a leg's LAW reaches only the MASKED leg; a device in the PLANT the
//! legs READ reaches only the AUTHORITATIVE one.** Min-select masks a law; it cannot mask a
//! plant, because the plant is shared. So this writes `c/tau_f` on the authoritative fuel
//! diagonal — the one entry rungs 73, 74 and 75 each measure as *moved 0.0 relative* — and leaves
//! the masked one alone. And `n_live` is STILL <= 3, a FIFTH time.
//!
//! Ported from `tests/test_rung76.py` — **16 collected tests, of which 12 carry `slow` there**,
//! both MEASURED with `pytest --collect-only -q` and reconciled against
//! `cargo test --test rung76 -- --list`. **This file has 19**, and the three extra are DECLARED
//! below rather than absorbed.
//!
//! # THE PYTHON↔RUST MAP — 1:1 IN ORDER, **3 ADDED**, 0 COLLAPSED, 0 SPLIT BY PARAMETER
//!
//! | # | `tests/test_rung76.py` | here |
//! |---|---|---|
//! | 1 | `reduces_to_rung75_bit_for_bit` | [`reduces_to_rung75_bit_for_bit`] |
//! | 2 | `the_reduce_is_not_vacuous` | [`the_reduce_is_not_vacuous`] |
//! | 3 | `the_refusals_are_refusals` | [`the_refusals_are_refusals`] |
//! | 4 | `the_knob_is_carried_by_at_lever` | [`the_knob_is_carried_by_at_lever`] |
//! | 5 | `c_is_strictly_inside_the_unit_interval` | [`c_is_strictly_inside_the_unit_interval`] |
//! | 6 | `the_authoritative_diagonal_moves_and_the_law_is_c_minus_one` | [`the_authoritative_diagonal_moves_and_the_law_is_c_minus_one`] |
//! | 7 | `the_move_is_identical_in_both_references` | [`the_move_is_identical_in_both_references`] |
//! | 8 | `the_masked_leg_is_untouched_and_the_rank_does_not_move` | [`the_masked_leg_is_untouched_and_the_rank_does_not_move`] |
//! | 9 | `the_governors_row_is_bit_identical` | [`the_governors_row_is_bit_identical`] |
//! | 10 | `det_J_scales_by_roughly_one_minus_c_and_the_residual_is_not_noise` | [`det_j_scales_by_roughly_one_minus_c_and_the_residual_is_not_noise`] |
//! | 11 | `the_masked_cell_is_structurally_unreachable_with_the_accel_leg_binding` | [`the_masked_cell_is_structurally_unreachable_with_the_accel_leg_binding`] |
//! | 12 | `the_two_laws_agree_at_the_solves_own_answer` | [`the_two_laws_agree_at_the_solves_own_answer`] |
//! | 13 | `the_solve_amplifies_the_cap_by_one_over_one_minus_c` | [`the_solve_amplifies_the_cap_by_one_over_one_minus_c`] |
//! | 14 | `the_sensed_leg_cuts_harder_over_the_whole_ramp` | [`the_sensed_leg_cuts_harder_over_the_whole_ramp`] |
//! | 15 | `the_trajectories_do_not_converge_at_the_tail` | [`the_trajectories_do_not_converge_at_the_tail`] |
//! | 16 | `the_cap_march_MOVES_but_TWO_OF_FOUR_LOOPS_ARE_INERT_at_this_wall` | [`the_cap_march_moves_but_two_of_four_loops_are_inert_at_this_wall`] |
//! | **+1** | **— none —** | [`the_cap_scope_restores_what_it_displaced_and_not_a_constant`] |
//! | **+2** | **— none —** | [`the_fixed_point_identity_is_scored_where_it_is_not_already_exact`] |
//! | **+3** | **— none —** | [`the_second_margin_reaches_the_two_expressions_the_suites_grid_leaves_dark`] |
//!
//! # THE THREE ADDED GATES, AND WHY EACH IS OWED RATHER THAN INVENTED
//!
//! Each was booked onto this step in writing, by the step that measured the hole.
//!
//! **+1 — `CapScope`'s RESTORE POLICY** (§ 5.31.4 (b)). The scope must be entered on a machine
//! whose RESTING law is not the one being armed, and `_cap_law` read WITHOUT being set first.
//! Driving the guard from a default machine through `_cap_fuel` — which is all any ported gate
//! does — sees neither of the two wrong policies: restore-to-a-constant and do-not-restore both
//! leave a default-`solve` machine reading `solve` afterwards, which is the right answer for the
//! wrong reason. Both directions are driven here, so the answer cannot be a constant.
//!
//! **+2 — `solve_gain`'s IDENTITY (1), ON A ROW WHERE IT IS NOT ALREADY EXACT** (§ 5.31.5 (d)).
//! Python asserts the AGGREGATE `sg["fixed_point"] < 1e-15`, and step 5 measured that field at
//! **`+0.0` bit for bit at 18 of the 36 driven rows** — 8 of 10 at the suite's own cell. At such
//! a row `sensed(q, w0)` returns `w0` itself, so a reader that had wrongly written
//! `solve(q) - w0` — comparing the solve with itself — reports the same `0.0` and is scored as
//! having verified the identity. **A gate that only ever lands on those rows certifies the
//! exactness with the exactness** ([[instrument-fed-by-what-it-certifies]]). Step 5 also supplied
//! the better population: rung 69's INCIDENCE arm, where the exact-zero rows are 1 of 1, 1 of 2
//! and 1 of 4. So this gate asserts on the NONZERO rows, counts them so it cannot go vacuous, and
//! takes its bar from ARITHMETIC (a few ULPs of `cap_solve`) rather than from any measurement the
//! port produced — which is [[rust-port-inside-outside-exactness]]'s rule.
//!
//! **+3 — THE SECOND MARGIN** (§ 5.31.5 (a), the widest of the inherited requirements). The
//! ported gates take their grid from `test_rung76.py` by construction, and that grid was measured
//! to leave two expressions unexercised: `cap_rows`' `accel_binds` filter never discards a row
//! (`n_inert = 0` of 10 at the `phi` arm's own margin) and `CapCellRead::row_err` is never
//! computed, because no cell is governor-authoritative there — which is gate 11's finding, and
//! therefore also gate 11's blind spot. **A ported gate that reproduces the suite exactly
//! reproduces its blind spots exactly.** Step 5's harness covered them and was deleted with the
//! step, so the second margin lands here. Its two bars are transcribed from § 5.31.5 (k)'s table,
//! which was measured on the PYTHON drive: `n_inert = 1` of 2 rows on the incidence arm at the
//! suite's own margin, and `row_err` reachable only at (`phi`, `0.20`).
//!
//! # WHAT IS DELIBERATELY *NOT* ADDED
//!
//! § 5.31.4 (c) and § 5.31.5 (c) between them disclose FIVE differences that **no value gate on
//! this plant can distinguish**: either `max` spelling in `c_at`, the two clock conditions, the
//! two scope orders, `masked_moved`'s two arms, and the two NaN folds. They are recorded in
//! `sensed_cap.rs`'s own doc comments as defences with no reader. A gate here asserting any of
//! them would be asserting a tautology, and step 5 ruled it out in advance rather than leaving it
//! to be attempted and quietly dropped.
//!
//! # THE READERS' UNSPELLED DEFAULTS COME FROM `sensed_cap.rs`'s CONSTANTS
//!
//! § 5.31.5 (l) named seven constants — [`CAP_GAINS_REFS`], [`CAP_GAINS_LAWS`],
//! [`CAP_BILL_TAU_T`], [`CAP_BILL_TAIL`], [`SOLVE_GAIN_REF`], [`SOLVE_GAIN_DQ`],
//! [`SOLVE_GAIN_EVERY`] — precisely because **this file is their only call site**: the suite does
//! not spell them, `main.py` does not, and `engine.py`'s own `Usage:` block passes four
//! positional arguments. They are used below rather than re-typed. The defaults the suite DOES
//! spell (`phi_lim`, `margin`, `taus`, `tau_t`, `r`, `s_settle`, `ds`, `v_max`, `inc`, `every`)
//! are typed from the suite here and deliberately NOT aliased to a const: a second home for a
//! number this file transcribes is how two copies of one value start to disagree.
//!
//! # THE SHARED `cap_gains` READING IS AN `OnceLock`, AND THE COST IS MEASURED
//!
//! Six Python gates take a module-scoped `gains` fixture. One [`cap_gains`] call costs **5.9 s**
//! in `--release` (measured, slice AG step 6), so recomputing it per gate would cost ~36 s for
//! six readings that Python computes once. It is shared through [`OnceLock`] exactly as
//! `rung48.rs` and `rung49.rs` share their sweeps. `OnceLock::get_or_init` is deliberate over a
//! `LazyLock`: a panic inside the initialiser leaves the cell UNSET and the next gate re-runs it,
//! so a real failure surfaces with its own message in every gate rather than as a poisoning
//! report in five of six.

use std::sync::OnceLock;

use turbojet::anti_windup::{build_anti_windup_cascade, WINDUP_LAW_NONE, WINDUP_LAW_TRACK};
use turbojet::applied_reference::REF_LAW_APPLIED;
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Authority, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::reference_split::StatorIncidenceLimiter;
use turbojet::sensed_cap::{
    accel_for, build_sensed_cap_cascade, cap_bill, cap_gains, cap_march, solve_gain, CapCell,
    CapCellRead, CapGains, CapScope, CAP_BILL_TAIL, CAP_BILL_TAU_T, CAP_GAINS_LAWS,
    CAP_GAINS_REFS, CAP_LAW_SENSED, CAP_LAW_SOLVE, SOLVE_GAIN_DQ, SOLVE_GAIN_EVERY,
    SOLVE_GAIN_REF,
};
use turbojet::shared_actuator::{SharedRigArm, REF_LAW_DEFAULT};
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

// ============================================================================== the grid
//
// `tests/test_rung76.py`'s module constants, verbatim.

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
const TT4_MAX: f64 = 1200.0;
const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;
const TAU_T: f64 = 0.05;

/// The two floors, inherited from rungs 74/75 rather than chosen: `0.80` is where all four legs
/// ride and every JACOBIAN is read; `0.76` is the both-legs-ride arm where every TRAJECTORY is
/// marched.
const PHI_JAC: f64 = 0.80;
const PHI_BOTH: f64 = 0.76;

/// **RUNG 48's OWN already-imposed scalar, and the ONE imposition this rung carries.** At `0.10`
/// the accel leg is the binding cap on this trajectory; above ~`0.20` the `phi` leg takes over
/// and the knob is INERT by construction (spec § 1.3).
///
/// **THAT SENTENCE IS ABOUT THE READERS' POINTS, NOT THE MARCH, AND THE TWO ANSWER OPPOSITELY AT
/// THE SAME MARGIN** — § 5.31 (i), and the Python file now carries both tables under it. The
/// consequence for this file is
/// [`the_second_margin_reaches_the_two_expressions_the_suites_grid_leaves_dark`].
const MARGIN: f64 = 0.10;

/// § 5.31.5 (a)'s second margin — **NOT the suite's, and that is the point.** See the module
/// header's `+3`.
const MARGIN_HI: f64 = 0.20;

/// **THE DIFFERENCING FLOOR, and it is arithmetic rather than taste**: `_rhs_gains_at`
/// central-differences at `dg = 1e-7`, so roundoff alone is `eps/dg ~ 2.2e-9`. The anchor asked
/// for `< 1e-9` and § 7 scores that tolerance as optimistic by exactly this much.
const JAC_FLOOR: f64 = 3e-9;

/// `PHI / FLOOR - 1.0` — the EXPRESSION Python spells, never a typed decimal.
fn sm_of(phi_lim: f64) -> f64 { phi_lim / FLOOR - 1.0 }

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

/// Python's `_rig(design, SensedCapTransient, …)` — five knobs, all by PLAIN ASSIGNMENT.
#[allow(clippy::too_many_arguments)]
fn cap_rig(sm: f64, inc: bool, coord: &'static str, ref_law: &'static str, law: &'static str,
           tau_t: Option<f64>, cap_law: &'static str) -> ScheduledStatorCore {
    let m = full_of(build_sensed_cap_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &suite_arm(sm, inc)));
    m.fuel.inner.lag_coord.set(coord);
    m.fuel.inner.ref_law.set(ref_law);
    m.fuel.inner.windup_law.set(law);
    m.fuel.inner.tau_t.set(tau_t);
    m.fuel.inner.cap_law.set(cap_law);
    m
}

/// The default arm of Python's `_rig`.
fn cap_default(sm: f64, inc: bool) -> ScheduledStatorCore {
    cap_rig(sm, inc, "demand", REF_LAW_DEFAULT, WINDUP_LAW_NONE, None, CAP_LAW_SOLVE)
}

/// Python's `_rig(design, AntiWindupTransient, …)` — the PARENT, gate 1's other side. Python sets
/// no `_cap_law` on this class, and neither does this.
fn windup_rig(sm: f64, coord: &'static str, ref_law: &'static str, law: &'static str,
              tau_t: Option<f64>) -> ScheduledStatorCore {
    let m = full_of(build_anti_windup_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &suite_arm(sm, false)));
    m.fuel.inner.lag_coord.set(coord);
    m.fuel.inner.ref_law.set(ref_law);
    m.fuel.inner.windup_law.set(law);
    m.fuel.inner.tau_t.set(tau_t);
    m
}

/// Python's `_accel` — **THE SCHEDULE IS BUILT ON THE RIG THAT WILL MARCH IT.** `kappa_ss` is read
/// off the plant's OWN equilibria, so a schedule built on a bare machine and marched on
/// `_shared_rig`'s would be a schedule for a DIFFERENT ENGINE — the trap rungs 61–75 hit on knobs,
/// wearing its other face (spec § 7).
fn accel(sm: f64, inc: bool, margin: f64) -> AccelSchedule {
    accel_for(&cap_default(sm, inc), &flight(), LO, HI, sm, TT4_MAX, TAUS, V_MAX, inc, margin)
}

fn surge_of(sm: f64) -> SurgeLimiter { SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm) }
fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }

/// Python's `_march(m, sm, acc)` — all four loops PLUS rung 48's schedule, which is the arming no
/// march in this family had ever carried.
fn march(m: &ScheduledStatorCore, sm: f64, acc: &AccelSchedule) -> Vec<FuelPoint> {
    let leg = StatorLeg {
        accel: Some(acc),
        surge: Some(Floor::Phi(surge_of(sm))),
        tt4_max: Some(TT4_MAX),
    };
    let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS };
    m.stator_march_scoped(&flight(), &ramp, None, &leg,
                          &MarchScope { lag: Some(lag()), tau_gov: Some(TAU_GOV),
                                        ..MarchScope::DEFAULT }).0
}

/// Python's `_keys(traj)`. Length is per point for [`rung75.rs`'s reason](../rung75/index.html) —
/// a `clip` arm records no `w_fuel`/`w_gov`, and gate 1 marches one.
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
            _ => panic!("rung-76's reduce compares marches that record `b` and `v`."),
        }
        v
    }).collect()
}

/// Python's `pytest.raises(AssertionError, match=…)` — the message, or `""` on a clean return.
fn message_of<F: FnOnce()>(f: F) -> String {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(()) => String::new(),
        Err(e) => match e.downcast_ref::<String>() {
            Some(s) => s.clone(),
            None => e.downcast_ref::<&str>().map(|s| (*s).to_string())
                     .unwrap_or_else(|| "<non-string panic>".into()),
        },
    }
}

/// Python's module-scoped `gains` fixture. See the module header for why `OnceLock`.
fn gains() -> &'static CapGains {
    static G: OnceLock<CapGains> = OnceLock::new();
    G.get_or_init(|| cap_gains(
        &cap_default(sm_of(PHI_JAC), false), &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS,
        TAU_T, &CAP_GAINS_REFS, &CAP_GAINS_LAWS, false, R, SETTLE, DS, V_MAX, 8))
}

/// Python's `_live(g)` — `{k: c for k, c in g["cells"].items() if c.get("n")}`. An
/// [`Empty`](CapCell::Empty) cell has no `n` key at all, which is what makes `.get` the right
/// spelling there and a `Read` with `n == 0` unreachable.
fn live(g: &CapGains) -> Vec<(&str, &CapCellRead)> {
    g.cells.iter().filter_map(|(k, c)| match c {
        CapCell::Read(b) if b.n > 0 => Some((k.as_str(), &**b)),
        _ => None,
    }).collect()
}

// ======================================================================================
// THE REDUCE SPINE — ONE arm, by DISPATCH, on FIVE cells. `_cap_law = 'solve'` is not a
// limit of anything: the hook's branch is simply not taken and the floats are rung 75's.
// ======================================================================================

/// The accel-armed plant is one this ladder has ALWAYS supported and never marched, so the reduce
/// runs on it and not on rungs 72–75's `phi`-only rig (spec § 0.2).
#[test]
fn reduces_to_rung75_bit_for_bit() {
    let sm = sm_of(PHI_BOTH);
    let acc = accel(sm, false, MARGIN);
    for (coord, ref_law, law, tt) in [
        ("clip", REF_LAW_APPLIED, WINDUP_LAW_NONE, None),
        ("demand", REF_LAW_DEFAULT, WINDUP_LAW_NONE, None),
        ("demand", REF_LAW_DEFAULT, WINDUP_LAW_TRACK, Some(TAU_T)),
        ("demand", REF_LAW_APPLIED, WINDUP_LAW_TRACK, Some(TAU_T)),
        ("demand-latched", REF_LAW_APPLIED, WINDUP_LAW_NONE, None),
    ] {
        let a = keys(&march(&cap_rig(sm, false, coord, ref_law, law, tt, CAP_LAW_SOLVE),
                            sm, &acc));
        let b = keys(&march(&windup_rig(sm, coord, ref_law, law, tt), sm, &acc));
        assert_eq!(a, b, "{coord}|{ref_law}|{law}");
    }
}

/// **ARM 1 MUST BE A TEST, NOT A TAUTOLOGY** (rung 73's `charpoly_selftest` discipline, rungs
/// 74/75's own): if `_cap_law` were ignored, the reduce above would compare rung 75 with rung 75
/// and pass. The SAME machine under `sensed` must DIFFER.
#[test]
fn the_reduce_is_not_vacuous() {
    let sm = sm_of(PHI_BOTH);
    let acc = accel(sm, false, MARGIN);
    let a = keys(&march(&cap_default(sm, false), sm, &acc));
    let b = keys(&march(&cap_rig(sm, false, "demand", REF_LAW_DEFAULT, WINDUP_LAW_NONE, None,
                                 CAP_LAW_SENSED), sm, &acc));
    assert_ne!(a, b);
}

/// `clip × sensed` is refused because `clip` DISPATCHES OUT of this ladder before `_cap_fuel` is
/// ever called — the march would silently be rung 73 and be reported as this rung. And `sensed`
/// without a schedule has nothing to re-read.
///
/// The three needles are `turbojet/engine.py:19228`, `:19233` and `:19223`'s literals —
/// transcribed from THERE and never from `sensed_cap.rs`, because slice AF step 4 § (c) measured
/// the port's own refusal text four formatting divergences wide against Python's. § 5.31 (v)
/// measured each of these three matching exactly ONE message file-wide, so unlike rung 75's
/// `"did not converge"` all three pin a SITE.
#[test]
fn the_refusals_are_refusals() {
    let sm = sm_of(PHI_BOTH);
    let acc = accel(sm, false, MARGIN);

    let msg = message_of(|| {
        let m = cap_rig(sm, false, "clip", REF_LAW_APPLIED, WINDUP_LAW_NONE, None,
                        CAP_LAW_SENSED);
        march(&m, sm, &acc);
    });
    assert!(msg.contains("REFUSED in the CLIP coordinate"), "{msg}");

    // AND THIS ONE MARCHES WITHOUT A SCHEDULE, which is why it is spelled out rather than
    // routed through `march` — the arming under test is the ABSENCE of the accel leg.
    let msg = message_of(|| {
        let m = cap_rig(sm, false, "demand", REF_LAW_DEFAULT, WINDUP_LAW_NONE, None,
                        CAP_LAW_SENSED);
        let leg = StatorLeg { accel: None::<&AccelSchedule>,
                              surge: Some(Floor::Phi(surge_of(sm))), tt4_max: Some(TT4_MAX) };
        let ramp = Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds: DS };
        m.stator_march_scoped(&flight(), &ramp, None, &leg,
                              &MarchScope { lag: Some(lag()), tau_gov: Some(TAU_GOV),
                                            ..MarchScope::DEFAULT });
    });
    assert!(msg.contains("there must BE one"), "{msg}");

    let msg = message_of(|| {
        let m = cap_rig(sm, false, "demand", REF_LAW_DEFAULT, WINDUP_LAW_NONE, None, "fitted");
        march(&m, sm, &acc);
    });
    assert!(msg.contains("CAP LAW is this rung's subject"), "{msg}");
}

/// **THE FOURTEENTH INSTANCE of the trap rungs 61–75 each hit** — and the first rung that wrote
/// the carry BEFORE the first reader ran rather than after a reader lied.
#[test]
fn the_knob_is_carried_by_at_lever() {
    let sm = sm_of(PHI_BOTH);
    let m = cap_rig(sm, false, "demand", REF_LAW_DEFAULT, WINDUP_LAW_NONE, None,
                    CAP_LAW_SENSED);
    // Python passes `bleed_lim=m.bleed_lim, stator_lim=m.stator_lim` — the machine's OWN two
    // devices, which is what `suite_arm` builds.
    assert_eq!(m.at_lever(&suite_arm(sm, false)).fuel.inner.cap_law.get(), CAP_LAW_SENSED);
    let arm = SharedRigArm {
        sm, tau: TAU, tau_s: TAU_S, v_max: V_MAX, tt4_max: TT4_MAX,
        ..SharedRigArm::default()
    };
    let (rig, _, _) = (m.triple_hooks().shared_rig)(&m, &arm);
    assert_eq!(rig.fuel.inner.cap_law.get(), CAP_LAW_SENSED);
}

// ======================================================================================
// § 0.3 — `c` IS MEASURED, and `c < 1` is NOT implied by the shipped solver working
// ======================================================================================

/// A bracketing root-finder converges on a SIGN CHANGE whether or not `G = w - cap(w)` is
/// monotone, so `_sched_fuel` bracketing buys *a root exists*, never `G' > 0`. `c` is therefore
/// measured, on both stator arms and across margins.
///
/// **THE `accel` CALL IS KEPT THOUGH ITS VALUE IS DISCARDED.** Python builds `acc` and then
/// `del acc`s it: `solve_gain` builds its own schedule internally, so the binding is unused — but
/// the CALL is a real `accel_for` on this arm and margin, and it can raise. Dropping the line as
/// dead code would delete an execution the suite performs at six (arm, margin) points.
#[test]
fn c_is_strictly_inside_the_unit_interval() {
    for inc in [false, true] {
        for margin in [0.05, MARGIN, 0.40] {
            let sm = sm_of(PHI_JAC);
            let acc = accel(sm, inc, margin);
            let m = cap_default(sm, inc);
            let sg = solve_gain(&m, &flight(), LO, HI, TT4_MAX, PHI_JAC, margin, TAUS,
                                SOLVE_GAIN_REF, inc, R, SETTLE, DS, V_MAX, SOLVE_GAIN_DQ,
                                SOLVE_GAIN_EVERY);
            assert!(sg.n > 0, "{inc} {margin}");
            let lo = sg.rows.iter().map(|x| x.c).fold(f64::INFINITY, f64::min);
            let hi = sg.rows.iter().map(|x| x.c).fold(f64::NEG_INFINITY, f64::max);
            assert!(0.0 < lo && hi < 1.0, "{inc} {margin} {lo} {hi}");
            drop(acc);
        }
    }
}

// ======================================================================================
// § 1 — THE AUTHORITATIVE DIAGONAL, which nothing in this family has ever moved
// ======================================================================================

/// **THE RUNG.** `d(mf_app)/dw_auth = 1` where the leg holds, so `d(cap)/dw_auth = c` and the
/// diagonal is `(c-1)/tau_f` — against `-1/tau_f`, which rungs 73, 74 AND 75 each report as
/// *moved 0.0 relative*. Scored PER POINT against THAT point's own `c`, never pooled.
#[test]
fn the_authoritative_diagonal_moves_and_the_law_is_c_minus_one() {
    let l = live(gains());
    assert!(!l.is_empty(), "no interior riding cell");
    for (k, c) in &l {
        assert!(c.auth_err < JAC_FLOOR, "{k} {}", c.auth_err);
        assert!(c.auth_moved > 0.15, "{k} {}", c.auth_moved);
        // AND WHAT IT MOVED FROM IS `-1/tau_f`, TO THE SAME DIFFERENCING FLOOR AND NOT BETTER —
        // because rung 73 WEAKENED `_jac4` to *measure* this diagonal rather than construct it,
        // and § 1.3 of that rung priced the weakening at five orders of magnitude. A gate here
        // at `1e-12` would be asserting against the construction rung 73 removed.
        assert!((c.auth_diag0.0 + 1.0 / TAUS.0).abs() < JAC_FLOOR, "{k} {:?}", c.auth_diag0);
    }
}

/// `_demand_reference` returns `cap` ITSELF when `mf_app == w_own`, i.e. the applied reference is
/// the IDENTITY on the leg that holds — so it cannot change what a plant-side gain does there. A
/// sharp asymmetry against rung 75, whose masked diagonal is reference-DEPENDENT.
#[test]
fn the_move_is_identical_in_both_references() {
    let l = live(gains());
    let mut sched: Vec<f64> = Vec::new();
    let mut applied: Vec<f64> = Vec::new();
    for (_, c) in &l {
        if c.ref_law == REF_LAW_APPLIED { applied.push(c.auth_err) } else { sched.push(c.auth_err) }
    }
    assert!(!sched.is_empty() && !applied.is_empty(), "both references must be represented");
    for (name, errs) in [("sched", &sched), ("applied", &applied)] {
        let worst = errs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!(worst < JAC_FLOOR, "{name} {worst}");
    }
}

/// `min()` is flat in what the masked leg holds, so `d(mf_app)/dw_masked = 0` and the masked
/// diagonal cannot move — and the masked COLUMN stays zero, which is `n_live <= 3` a FIFTH
/// running.
#[test]
fn the_masked_leg_is_untouched_and_the_rank_does_not_move() {
    for (k, c) in &live(gains()) {
        assert_eq!(c.masked_moved, 0.0, "{k}");
        assert_eq!(c.mask_leak, 0.0, "{k}");
        assert_eq!(c.mask_leak0, 0.0, "{k}");
        assert_eq!(c.zeros, c.zeros0, "{k}");
    }
}

/// `_cap_gov` has NO sensed branch — a floor on a STATE is not a formula for a FUEL — so nothing
/// in that row can move in any cell. The knob's DOMAIN, measured.
#[test]
fn the_governors_row_is_bit_identical() {
    for (k, c) in &live(gains()) {
        assert_eq!(c.gov_row, 0.0, "{k}");
    }
}

/// The masked column is zero, so `det J` = masked diagonal × det(live 3×3) and only the
/// authoritative fuel row moves. Anchor P7 asked for `1-c` EXACTLY and § 7 scores it REFUTED: the
/// whole row scales by `1-c` only when both laws are read at the SAME `w`, and they are not
/// (`mf_app` against `cap_solve`). The residual is bounded and § 3 names it.
///
/// `applied × solve` is excluded because `det J == 0` there (rung 73's dead determinant) and the
/// ratio is 0/0 — which is exactly what the anchor excluded in advance.
#[test]
fn det_j_scales_by_roughly_one_minus_c_and_the_residual_is_not_noise() {
    let mut checked = 0;
    for (k, c) in &live(gains()) {
        if c.ref_law == REF_LAW_APPLIED && c.law == WINDUP_LAW_NONE {
            assert!(c.det0.0.abs() < 1e-6, "{k} {:?}", c.det0);      // still dead
            continue;
        }
        let err = c.det_err.unwrap_or_else(|| panic!("{k} has no det ratio"));
        assert!(err < 0.06, "{k} {err}");
        let rr = c.det_ratio.expect("a det ratio beside a det_err");
        assert!(0.75 < rr.0 && rr.1 < 0.90, "{k} {rr:?}");
        checked += 1;
    }
    assert!(checked >= 2, "{checked}");
}

/// § 1.4, and it is a CONSEQUENCE rather than a gap. Rung 48's `Wf/pt3` leg is FEEDFORWARD ON THE
/// CAUSE and fires early; the topping governor is FEEDBACK ON A CONSEQUENCE and fires late. So
/// where the accel leg BINDS the cap it binds early, and the leg that binds the cap is then also
/// the leg that HOLDS the actuator. Anchor P6 is scored UNREACHED on this.
#[test]
fn the_masked_cell_is_structurally_unreachable_with_the_accel_leg_binding() {
    let g = gains();
    let mut seen = 0;
    for (k, c) in &g.cells {
        let (auth, n) = match c {
            CapCell::Read(b) => (b.auth, b.n),
            CapCell::Empty { auth, .. } => (*auth, 0),
        };
        if auth == Authority::Gov {
            assert_eq!(n, 0, "{k}");
            seen += 1;
        }
    }
    // A GOVERNOR CELL MUST EXIST FOR THE CLAIM TO BE ABOUT ANYTHING. Python's loop over an empty
    // dict would pass silently; here the reader publishes both authorities for every (ref, law),
    // so the count is a fact and is asserted as one.
    assert_eq!(seen, 4, "two references x two windup laws, each publishing a `gov` cell");
}

// ======================================================================================
// § 3 — WHAT THE SOLVE WAS BUYING: a GAIN, not a relocation
// ======================================================================================

/// D2, where it actually lives. `cap_solve` is BY CONSTRUCTION the fixed point of `cap_sensed`, so
/// the two laws agree there EXACTLY — and that is why the equilibrium of a leg that holds does not
/// move. The march's TAIL is not that equilibrium (the spools are still spinning up), which is why
/// anchor P10's trajectory form is scored REFUTED.
#[test]
fn the_two_laws_agree_at_the_solves_own_answer() {
    for inc in [false, true] {
        let m = cap_default(sm_of(PHI_JAC), inc);
        let sg = solve_gain(&m, &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS,
                            SOLVE_GAIN_REF, inc, R, SETTLE, DS, V_MAX, SOLVE_GAIN_DQ,
                            SOLVE_GAIN_EVERY);
        assert!(sg.n > 0, "{inc}");
        let fp = sg.fixed_point.expect("a non-empty row set publishes it");
        assert!(fp < 1e-15, "{inc} {fp}");
    }
}

/// **THE FINDING THAT WAS NOT PREDICTED AT ALL**, and the correction P7's refutation bought.
/// Differentiating the fixed point `cap = cap_sensed(cap, q)` gives
/// `d(cap_solve)/dq = (d(cap_sensed)/dq)/(1-c)` in one line — so writing a limiter as a SOLVE
/// multiplies its sensitivity to every other state by `1/(1-c)`. A limiter written as a solve is a
/// STIFFER limiter than the schedule it claims to implement.
#[test]
fn the_solve_amplifies_the_cap_by_one_over_one_minus_c() {
    for inc in [false, true] {
        let m = cap_default(sm_of(PHI_JAC), inc);
        let sg = solve_gain(&m, &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS,
                            SOLVE_GAIN_REF, inc, R, SETTLE, DS, V_MAX, SOLVE_GAIN_DQ,
                            SOLVE_GAIN_EVERY);
        let err = sg.gain_err.expect("a non-empty row set publishes it");
        assert!(err < 1e-7, "{inc} {err}");
        let gain = sg.gain.expect("a non-empty row set publishes it");
        assert!(gain.0 > 1.0, "{inc} {gain:?}");
    }
}

// ======================================================================================
// § 2 — THE BILL: the path moves, the destination does not
// ======================================================================================

/// During the ramp `mf_app < cap_solve`, so the droop identity gives
/// `cap_sensed = cap_solve + c*(mf_app - cap_solve) < cap_solve`. **Rung 48's set-point solve has
/// been granting the engine the fuel it would be self-consistent WITH, which is more fuel than the
/// schedule it implements allows.** The SIGN is the claim.
#[test]
fn the_sensed_leg_cuts_harder_over_the_whole_ramp() {
    let m = cap_default(sm_of(PHI_BOTH), false);
    let b = cap_bill(&m, &flight(), LO, HI, TT4_MAX, PHI_BOTH, MARGIN, TAUS, CAP_BILL_TAU_T,
                     REF_LAW_DEFAULT, WINDUP_LAW_NONE, false, R, SETTLE, DS, V_MAX,
                     CAP_BILL_TAIL);
    assert!(b.cuts_harder);
    assert!(b.max_tt4.1 < b.max_tt4.0, "{:?}", b.max_tt4);   // peak TIT falls
    assert!(b.min_phi.1 > b.min_phi.0, "{:?}", b.min_phi);   // surge margin rises
    assert!(b.fuel_int.1 < b.fuel_int.0, "{:?}", b.fuel_int); // and it burns less
}

/// **ANCHOR P10, SCORED REFUTED AND GATED AS SUCH.** The prediction was the right claim in the
/// wrong coordinate: D2 is about an EQUILIBRIUM and this march never reaches one — the schedule
/// stops at `s = r` but the spools are still spinning up, so the cap keeps moving and both legs
/// keep chasing it. Gating the refutation is what stops a later rung from quietly re-deriving the
/// claim that failed.
#[test]
fn the_trajectories_do_not_converge_at_the_tail() {
    let m = cap_default(sm_of(PHI_BOTH), false);
    let b = cap_bill(&m, &flight(), LO, HI, TT4_MAX, PHI_BOTH, MARGIN, TAUS, CAP_BILL_TAU_T,
                     REF_LAW_DEFAULT, WINDUP_LAW_NONE, false, R, SETTLE, DS, V_MAX,
                     CAP_BILL_TAIL);
    let tail = b.wf_tail.expect("the tail window is non-empty");
    let ramp = b.wf_ramp.expect("the ramp window is non-empty");
    assert!(tail > 1e-3, "{tail}");
    assert!(tail > ramp, "{tail} {ramp}");
}

// ======================================================================================
// THE THREE ADDED GATES — see the module header for what booked each one here.
// ======================================================================================

/// **+1 — § 5.31.4 (b), still owed at step 6.** A gate on [`CapScope`] must read `_cap_law`
/// WITHOUT setting it first, on a machine whose RESTING law is not the one the guard arms.
///
/// Driving the guard through `_cap_fuel` from a default machine — which is all the sixteen ported
/// gates do — cannot see either wrong restore policy. Restore-to-a-constant (`set(SOLVE)` on drop)
/// and do-not-restore both leave a default machine reading `solve` afterwards, and a machine that
/// starts at `solve` is the only machine those gates ever build. **So this one starts at
/// `sensed`**, and runs BOTH directions so the answer cannot be a constant either way.
#[test]
fn the_cap_scope_restores_what_it_displaced_and_not_a_constant() {
    let sm = sm_of(PHI_BOTH);
    for (resting, armed) in [(CAP_LAW_SENSED, CAP_LAW_SOLVE), (CAP_LAW_SOLVE, CAP_LAW_SENSED)] {
        let m = cap_rig(sm, false, "demand", REF_LAW_DEFAULT, WINDUP_LAW_NONE, None, resting);
        // READ FIRST, SET NOTHING: the resting law is the machine's, not the scope's.
        assert_eq!(m.fuel.inner.cap_law.get(), resting);
        {
            let g = CapScope::set(&m.fuel.inner, armed);
            assert_eq!(m.fuel.inner.cap_law.get(), armed, "the scope arms its argument");
            assert_eq!(g.displaced(), resting,
                       "and it reports what it DISPLACED, not what it restores to");
        }
        assert_eq!(m.fuel.inner.cap_law.get(), resting,
                   "on drop it restores the DISPLACED law — a restore to a constant would read \
                    {armed:?} here on one of these two arms");
    }
}

/// **+2 — § 5.31.5 (d): the identity has to be scored where it is not ALREADY exact.**
///
/// [`the_two_laws_agree_at_the_solves_own_answer`] asserts the aggregate `fixed_point < 1e-15`,
/// and step 5 measured that field at `+0.0` BIT FOR BIT at 18 of 36 driven rows. At such a row
/// `sensed(q, w0)` returns `w0` itself, so a reader that had wrongly written `solve(q) - w0` —
/// the solve compared with itself — publishes the same `0.0` and is scored as having verified the
/// identity. This gate therefore:
///
/// * finds the rows where the residual is NONZERO and asserts there is at least one, so it cannot
///   pass by landing only on the exact ones;
/// * scores each against a bar taken from ARITHMETIC, not from any number the port produced —
///   identity (1) is exact in algebra, so its residual can only be the last bits of `cap_solve`,
///   and `8 ULP` is the bar ([[rust-port-inside-outside-exactness]]: a residual needs an ABSOLUTE
///   bar, and `1e-15` on a quantity of order `0.05` is not one);
/// * runs on rung 69's INCIDENCE arm, which step 5 measured as the sharper population (1 of 1,
///   1 of 2 and 1 of 4 rows exactly zero, against 8 of 10 at the suite's own cell).
///
/// **THE PORT REPRODUCES STEP 5's PYTHON-SIDE SPLIT ROW FOR ROW** — 4 of 10, 8 of 10 and 3 of 9
/// on the `phi` arm at margins `0.05 / 0.10 / 0.40`, and 1 of 1, 1 of 2, 1 of 4 on the incidence
/// arm. That was measured on this binary before this gate was written, so the population the
/// assertion below relies on is a fact about the port and not an inherited hope.
///
/// **AND THE BAR IS SCOPED, BECAUSE IT IS NOT UNIVERSAL.** The worst residual on the cell this
/// gate drives is **2.87 ULP**, so `8` is tight to a factor of ~3 rather than a formality — but
/// on the `phi` arm at `margin = 0.40` the same quantity reaches **8.81 ULP** and would fail it.
/// The bar therefore belongs to THIS cell and is not to be generalised to the reader's domain;
/// what the wider domain needs is a bar somebody measures there, not this one relaxed until it
/// passes.
#[test]
fn the_fixed_point_identity_is_scored_where_it_is_not_already_exact() {
    let m = cap_default(sm_of(PHI_JAC), true);
    let sg = solve_gain(&m, &flight(), LO, HI, TT4_MAX, PHI_JAC, MARGIN, TAUS, SOLVE_GAIN_REF,
                        true, R, SETTLE, DS, V_MAX, SOLVE_GAIN_DQ, SOLVE_GAIN_EVERY);
    assert!(sg.n > 0);
    let nonzero: Vec<&turbojet::sensed_cap::SolveGainRow> =
        sg.rows.iter().filter(|x| x.fixed_point != 0.0).collect();
    assert!(!nonzero.is_empty(),
            "every row's residual is exactly zero, so this gate would be certifying the \
             exactness with the exactness — the population step 5 measured has moved");
    for x in &nonzero {
        let ulps = 8.0 * f64::EPSILON * x.cap_solve.abs();
        assert!(x.fixed_point <= ulps,
                "s={} residual {} exceeds 8 ULP of cap_solve={} ({ulps})",
                x.s, x.fixed_point, x.cap_solve);
    }
}

/// **+3 — § 5.31.5 (a), the widest inherited requirement.** The sixteen ported gates take their
/// grid from `test_rung76.py`, and that grid leaves two expressions in `sensed_cap.rs`
/// unexercised. **A ported gate that reproduces the suite exactly reproduces its blind spots
/// exactly**, so the second margin belongs in this file and not only in step 5's deleted harness.
///
/// The two bars are transcribed from § 5.31.5 (k)'s table, measured on the PYTHON drive:
///
/// * **`cap_rows`' `accel_binds` filter.** At the `phi` arm's own margin it discards nothing —
///   `n_inert = 0` of 10 rows — so the branch is present and inert, and every ported gate reads
///   its output without ever seeing it fire. On the INCIDENCE arm at the same margin it is LIVE.
///   **The COUNT below is a sum over all eight cells and § 5.31.5 (k)'s `1 of its 2 rows` is a
///   per-cell reading, so the two numbers are not the same statistic and are not asserted equal**
///   — driven here, this arm has 2 surviving rows and 2 filtered ones. What the gate pins is the
///   direction both readings agree on: nonzero here, exactly zero at the suite's own cell.
/// * **[`CapCellRead::row_err`].** It is `Some` only on a governor-authoritative cell, and
///   [`the_masked_cell_is_structurally_unreachable_with_the_accel_leg_binding`] measures that
///   there are none at the suite's margin — which is that gate's finding AND its blind spot.
///   Step 5 measured `row_err` reachable at (`phi`, `0.20`) and nowhere else.
#[test]
fn the_second_margin_reaches_the_two_expressions_the_suites_grid_leaves_dark() {
    // (a) THE FILTER FIRES — on the incidence arm, at the suite's OWN margin.
    let g_inc = cap_gains(&cap_default(sm_of(PHI_JAC), true), &flight(), LO, HI, TT4_MAX,
                          PHI_JAC, MARGIN, TAUS, TAU_T, &CAP_GAINS_REFS, &CAP_GAINS_LAWS, true,
                          R, SETTLE, DS, V_MAX, 8);
    let inert: usize = g_inc.cells.iter().map(|(_, c)| match c {
        CapCell::Read(b) => b.n_inert,
        CapCell::Empty { n_inert, .. } => *n_inert,
    }).sum();
    assert!(inert > 0,
            "`accel_binds` discards nothing here either, so the filter is inert on BOTH arms \
             this file drives and no gate in it scores the expression");
    // AND THE CONTROL: the suite's own arm, where it does NOT fire. Without this the assertion
    // above is a fact about the reader rather than about the grid.
    let inert_suite: usize = gains().cells.iter().map(|(_, c)| match c {
        CapCell::Read(b) => b.n_inert,
        CapCell::Empty { n_inert, .. } => *n_inert,
    }).sum();
    assert_eq!(inert_suite, 0,
               "the suite's own cell is where the filter is measured INERT — if it fires here \
                the two populations have merged and this gate is no longer a second one");

    // (b) `row_err` IS COMPUTED — at the second margin, on the `phi` arm, where a cell is
    // governor-authoritative and the suite's margin has none.
    let g_hi = cap_gains(&cap_default(sm_of(PHI_JAC), false), &flight(), LO, HI, TT4_MAX,
                         PHI_JAC, MARGIN_HI, TAUS, TAU_T, &CAP_GAINS_REFS, &CAP_GAINS_LAWS,
                         false, R, SETTLE, DS, V_MAX, 8);
    let gov: Vec<(&str, &CapCellRead)> = live(&g_hi).into_iter()
        .filter(|(_, c)| c.auth == Authority::Gov).collect();
    assert!(!gov.is_empty(),
            "no governor-authoritative cell at margin {MARGIN_HI}, so `row_err` is `None` \
             everywhere this file looks and the expression is still unscored");
    for (k, c) in &gov {
        let err = c.row_err.unwrap_or_else(|| panic!("{k} is `gov`-authoritative and must \
                                                      publish a `row_err`"));
        assert!(err.is_finite(), "{k} {err}");
    }
}

// --- THE MARCH AUDIT: rung 79's gap seam, checked from the other end ------------------------
//
// `docs/rungs72-77-march-audit.md`. A CONFIRMATION's gate, not this rung's anchor.

/// Rungs 78/79 stand still at `(demand, PHI_JAC = 0.80)` — rung 74 § 2.2's arrest arm.
/// [`cap_bill`] marches the same coordinate at `PHI_BOTH = 0.76` and the plant DOES accelerate, so
/// the arrest is the CELL and not the rig this rung shares with them.
///
/// **AND THE VALVE AND THE STATOR ARE INERT HERE, 0/341 each** — pinned as an equality, since the
/// dormancy is the record. § 2's `cuts_harder` bill is therefore read on a TWO-loop plant; it is a
/// claim about the fuel-side cap law, which is what makes that admissible, but the scope was never
/// stated.
#[test]
fn the_cap_march_moves_but_two_of_four_loops_are_inert_at_this_wall() {
    let sm = sm_of(PHI_BOTH);
    let m = cap_default(sm, false);
    let acc = accel(sm, false, MARGIN);
    let traj = cap_march(&m, &flight(), LO, HI, TT4_MAX, sm, TAUS, R, SETTLE, DS, V_MAX, false,
                         "demand", REF_LAW_DEFAULT, WINDUP_LAW_NONE, None, CAP_LAW_SOLVE, &acc,
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
    let b_max = m.fuel.inner.lever.lim.expect("the suite arms a valve").b_max;
    assert_eq!(traj.iter().filter(|p| { let b = b_cmd_of(p); b > 0.0 && b < b_max }).count(), 0);
    assert_eq!(traj.iter().filter(|p| v_regime_of(p) == Some(Regime::Riding)).count(), 0);
    assert!(traj.iter().filter(|p| required_of(p) > 0.0).count() > 300);
    let min_phi = traj.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min);
    assert!(min_phi > PHI_BOTH, "{min_phi}");
    assert!(min_phi > 0.755, "the phi leg would be dormant below the droop: {min_phi}");
}

fn b_cmd_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Shared { b_cmd, .. } | PointExtra::Demand { b_cmd, .. } => b_cmd,
        _ => panic!("rung-76's audit reads `b_cmd` off every marched point."),
    }
}

fn v_regime_of(p: &FuelPoint) -> Option<Regime> {
    match p.extra {
        PointExtra::Shared { v_regime, .. } | PointExtra::Demand { v_regime, .. } => v_regime,
        _ => panic!("rung-76's audit reads `v_regime` off every marched point."),
    }
}

fn required_of(p: &FuelPoint) -> f64 {
    match p.extra {
        PointExtra::Shared { required, .. } | PointExtra::Demand { required, .. } => required,
        _ => panic!("rung-76's audit reads `required` off every marched point."),
    }
}
