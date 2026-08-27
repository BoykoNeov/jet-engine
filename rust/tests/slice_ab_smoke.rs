//! SLICE AB — **THE RUNG-69 PORT, RUN ONCE END TO END.**
//!
//! Step 2 shipped nine cell bodies and six readers, and until something calls them the only thing
//! measured is that they COMPILE. This file is the cheapest instrument that is not that: it builds
//! the machine the ladder really builds, runs all six readers, and asserts the STRUCTURAL facts a
//! broken port cannot produce — a non-empty riding window, finite roots, both references actually
//! constructed.
//!
//! **EVERY ASSERTION HERE IS AN EXISTENCE OR A FINITENESS, AND THAT IS DELIBERATE.** The rung's
//! claims — `zeros = n - m`, `det J` blind, `c1` the discriminator, the damping floor, the sign
//! table — are `tests/rung69.rs`'s at step 3, ported from `test_rung69.py`. Asserting a claim here
//! too would give this file a pass condition it cannot justify and would duplicate a gate whose
//! Python original states it better. What it CAN see, and what nothing else in the crate could see
//! before step 3, is a body that panics, aborts, or silently produces no rows at all.
//!
//! It also reads [`Census69`], which is the only instrument that can tell a rung-69 cell from the
//! rung-68 body it reduces to: eight of the nine cells open with `stator_inc is None ⇒ the
//! parent`, and a reduce arm emits rung 68's numbers BY CONSTRUCTION.

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::{
    build_reference_split_cascade, damping_floor, reference_bill, reference_gains,
    reference_modes, ring_visibility, rk4_margin, Census69, StatorIncidenceLimiter,
};
use turbojet::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::TripleRigArm;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ---------------------------------------------------------------- the suite's own grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const PHI: f64 = 0.80;
const B: f64 = 0.10;
const TAU: f64 = 0.05;
const V_MAX: f64 = 0.20;
const SM: f64 = PHI / FLOOR - 1.0;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const R: f64 = 0.5;

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
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

/// The rung-69 machine `test_rung69.py`'s `split` fixture builds: rung 65's lagged valve under a
/// floor, plus rung 69's incidence stator on the SAME physical wall.
fn split_machine() -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::with_tau(PHI, B, Some(TAU))),
        stator_inc: Some(StatorIncidenceLimiter::from_margin(&lp_map(), V_MAX, SM, Some(TAU))),
        ..Default::default()
    };
    match build_reference_split_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("the LP spool is live"),
    }
}

/// `LO → HI` at `r = 0.5`, the suite's own ramp.
fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: 1.2, ds } }

fn arm() -> TripleRigArm { TripleRigArm { sm: SM, v_max: V_MAX, ..TripleRigArm::default() } }

// =============================================================================================
// § 1 — THE GAINS READER, AND THE ONE THING NO VALUE KEY CAN SEE
// =============================================================================================

/// `reference_gains` reaches riding-interior points under BOTH references, and the census says
/// **both rigs were actually built** — `rig_inc` and `rig_phi` are the only evidence anywhere that
/// `_triple_rig` read the carrier rather than falling through to its own fallback.
#[test]
fn reference_gains_runs_and_builds_both_rigs() {
    let m = split_machine();
    Census69::reset();
    let g = reference_gains(&m, &flight(), &ramp(0.005), SM, &arm(), 10);
    println!("n_riding={} n_sampled={} rows={} skipped={}",
             g.n_riding, g.n_sampled, g.rows.len(), g.skipped.len());
    println!("k_range={:?} worst_pair_gap={:?}", g.k_range, g.worst_pair_gap);
    let c = Census69::read();
    println!("census {c:?}");
    assert!(g.n_riding > 0, "the incidence march reached no riding-interior point at all");
    assert!(!g.rows.is_empty(), "every sampled point was skipped: {:?}", g.skipped);
    assert!(g.rows.iter().all(|x| x.k.is_finite() && x.pair_gap.is_finite()));
    assert!(c.rig_inc > 0 && c.rig_phi > 0,
            "both references must be CONSTRUCTED -- `rig_phi == 0` means the `_with_ref` scope \
             never reached `_triple_rig`, which is the defect s 5.26.1 (j) registered and which \
             no ledger key can see. Got {c:?}");

    // **THE FIRST DRAFT OF THE REST OF THIS GATE ASSERTED THE PARENT ARMS WERE NEVER TAKEN, AND
    // IT WAS WRONG.** This reader builds TWO rigs, and the `phi` one is a rung-68 machine BY
    // ARMING -- `stator_inc` is `None` on it -- so every cell called on it MUST take the reduce
    // arm. That is the contract, not a defect, and the count is 943 + 35 + 7 rather than 0.
    //
    // What the counts can pin instead is the READER's own shape, which is derivable and is the
    // reason the two references are differenced on ONE trajectory:
    //
    // * only `m_i` is MARCHED, so the march-only cells never see the parent at all;
    // * `m_p` is only DIFFERENCED, at `n_sampled` points, and each such point costs exactly one
    //   `manifold_v` plus the four `V` evaluations of a central difference in `g` and `q`.
    assert_eq!(c.lagged_parent + c.clamp_parent + c.check_parent, 0,
               "`_lagged_stator`, `_clamp_v` and `_check_v0` are reached only from the MARCH, and \
                only the INCIDENCE rig is marched here -- that is what makes this one trajectory \
                rather than two. A non-zero count means the `phi` rig was marched. Got {c:?}");
    assert_eq!(c.manifold_parent, g.n_sampled as u64,
               "the `phi` rig is evaluated ON THE SHARED MANIFOLD at every sampled point, before \
                any regime is inspected -- so exactly one parent `_manifold_v` per sampled point. \
                Got {c:?} against n_sampled = {}", g.n_sampled);
    assert_eq!(c.solve_parent, 5 * g.n_sampled as u64,
               "each `phi` point costs FIVE parent `_solve_v` calls: the manifold's own `V(g,q)` \
                plus the four `V+-g` / `V+-q` arms of the central difference. Got {c:?}");
}

// =============================================================================================
// § 1/3 — THE SPECTRUM, AND THE FIRST COMPLEX ARITHMETIC IN THE PORT
// =============================================================================================

/// `reference_modes` runs the four-clock grid under both references and every root is finite.
///
/// The 80-step Newton march inside `cubic_roots_c` **exhausts its budget on 72 of 256 calls** and
/// wanders over two decades before it does (§ 5.26 (iii)), so *"it produced numbers at all"* is
/// worth one assertion of its own before step 3 asks what they mean.
#[test]
fn reference_modes_runs_both_references_over_the_clock_grid() {
    let m = split_machine();
    let clocks = [(0.05, 0.05, 0.05), (0.05, 0.005, 0.05), (0.05, 0.5, 0.05),
                  (0.02, 0.05, 0.10)];
    let r = reference_modes(&m, &flight(), &ramp(0.002), SM, &clocks, V_MAX, 3.0, 20);
    assert_eq!(r.arms.len(), 4);
    for a in &r.arms {
        for (name, x) in a.refs() {
            println!("taus={:?} {name}: n={} rows={} skipped={} zeros={:?} zeta={:?}",
                     a.taus, x.n, x.rows.len(), x.skipped, x.zeros, x.zeta_range);
            assert!(!x.rows.is_empty(), "{name} produced no rows at {:?}", a.taus);
            assert!(x.rows.iter().all(|y| y.roots.iter()
                        .all(|z| z.re.is_finite() && z.im.is_finite())),
                    "{name} produced a non-finite root at {:?}", a.taus);
            assert!(x.rows.iter().all(|y| y.c1.is_finite() && y.c0.is_finite()));
        }
    }
}

/// `damping_floor` and `rk4_margin` — the two readers that quote the cubic's dominant root against
/// a closed form, run on one grid point each.
#[test]
fn the_damping_floor_and_the_rk4_margin_run() {
    let m = split_machine();
    let grid = [(0.05, 0.05, 0.05)];
    let d = damping_floor(&m, &flight(), &ramp(0.005), SM, &grid, V_MAX, 3.0);
    println!("damping rows={} holds={} tightest={:?}", d.rows.len(), d.holds, d.tightest);
    assert_eq!(d.rows.len(), 1);
    let live = d.tightest.expect("the mid-trajectory point must be riding-interior");
    assert!(live.zeta.is_finite() && live.floor.is_finite() && live.det2.is_finite());

    let k = rk4_margin(&m, &flight(), &ramp(0.005), SM, &arm(), 10);
    println!("rk4 n={} max_ratio={:?} max_bound={:?} ds_lambda={}",
             k.n, k.max_ratio, k.max_bound, k.ds_lambda);
    assert!(k.n > 0, "the rk4 margin reader found no interior point");
    assert!(k.rows.iter().all(|x| x.ratio.is_finite() && x.bound.is_finite()));
}

// =============================================================================================
// § 4 — THE LEDGER AND THE RING PROBE
// =============================================================================================

/// `reference_bill` runs rung 68's 8-cell ledger TWICE and the four stator-free cells agree —
/// **which is the one thing that CANNOT be evidence of a live reference**, and is asserted here
/// only because a drift in a cell that cannot have one would mean the rig itself moved.
#[test]
fn reference_bill_runs_both_ledgers_and_the_stator_free_cells_agree() {
    let m = split_machine();
    let b = reference_bill(&m, &flight(), &ramp(0.005), SM, &arm());
    println!("common_max_rel={:e} delivered={:?} delivered_inc={:?}",
             b.common_max_rel, b.delivered, b.delivered_inc);
    println!("credit inc={:?}\ncredit phi={:?}", b.stator_credit_inc, b.stator_credit_phi);
    assert_eq!(b.common.len(), 4);
    assert!(b.common_max_rel < 1e-12,
            "`bare`/`F`/`V`/`FV` carry no stator and are identical between the references BY \
             CONSTRUCTION; a difference here is the RIG moving, not the reference. Got {:e}",
            b.common_max_rel);
    assert!(b.delivered.0.is_finite() && b.delivered.1.is_finite());
}

/// `ring_visibility` marches four trajectories — base and displaced, under both references — and
/// the displaced arms really are displaced.
#[test]
fn ring_visibility_runs_all_four_arms() {
    let m = split_machine();
    let v = ring_visibility(&m, &flight(), &ramp(0.002), SM, &arm(), 0.05);
    for (name, r) in [("inc", v.inc), ("phi", v.phi)] {
        println!("{name} base={:?}", r.base);
        println!("{name} disp={:?}", r.displaced);
        assert!(r.base.n_riding > 0, "{name}: the base march never rode");
        assert!(r.base.survives.is_none(), "{name}: the base arm has no displacement");
        assert!(r.displaced.survives.is_some(), "{name}: the displaced arm must report one");
    }
}
