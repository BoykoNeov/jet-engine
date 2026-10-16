//! **THE NOZZLE PUSHES AGAINST THE FLIGHT'S AIR** (2026-10-16). Not a rung — a solver repair
//! (sandbox plan § 10.8, CLAUDE.md "Solver repairs").
//!
//! Every matcher captured the design run's ambient pressure and used it, for ever after, as the
//! nozzle's back-pressure — at whatever altitude it was then flown. The thrust's pressure term
//! read the flight's own `p0`, so the two disagreed whenever the flight left the design pressure.
//! No shipped caller ever did (a census of the whole gate found none, bit for bit), so nothing
//! moved; the sandbox was the first to fly frozen hardware at a new altitude, and carried an
//! override. Now every nozzle call and every choked/subsonic test reads the flight's `p0`.
//!
//! **The gate is a similarity law, not a stored number.** On a gas whose properties depend on
//! temperature only (the perfect gas, the thermally-perfect gas), an engine flown at the same
//! `T0` and Mach number but a different `p0` is the same engine scaled: every pressure goes as
//! `p0`, every ratio and temperature is unchanged, and airflow and thrust go as `p0`. With the
//! design back-pressure, the thin-air point is pushed against air four times as thick — the
//! nozzle reads unchoked and the two-shaft solve refuses (out of scope) or the one-shaft one moves
//! to the wrong branch. Removed once, this file went red (recorded in the commit).
//!
//! **Not on the equilibrium gas** (dissociation depends on pressure) and **not with a fixed-POWER
//! generator load** (`Demand::Power`: a fixed number of watts is a bigger share of a smaller
//! engine — real physics, not a defect). The share-type loads scale, so they are in.

use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::{ComponentMap, MapMatcher};
use turbojet::matcher::{Branch, OffDesignMatcher};
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses, TwoSpoolMapMatcher,
                          TwoSpoolMatcher};
use turbojet::two_spool_transient::TwoSpoolTransient;

/// Ratios agree to this, relatively. The scaling is exact in algebra; in arithmetic a solve's
/// tolerance and a few roundings separate the two points.
const SIMILAR: f64 = 1e-9;
const P_DESIGN: f64 = 50_000.0;
/// The discriminator is the quarter-pressure point (the old code pushed it against 4× its air);
/// double pressure is the other side, where the old code happened to agree on a choked nozzle.
const P_FLY: [f64; 3] = [12_500.0, 25_000.0, 100_000.0];

fn rel(a: f64, b: f64) -> f64 { ((a - b) / b).abs() }

fn at(p0: f64, m0: f64) -> FlightCondition { FlightCondition::new(250.0, p0, m0) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

/// The two temperature-only gases; the second is `cp(T)` from the NASA tables.
fn gases() -> [(&'static str, Gas); 2] { [("cpg", cpg()), ("tpg", Gas::thermally_perfect())] }

const ONE: Losses = Losses {
    pi_d: 0.97, eta_c: 0.88, e_c: None, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, e_t: None,
    eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const TWO: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};

fn lp() -> ComponentMap { ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() } }
fn hp() -> ComponentMap { ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() } }

fn two(gas: Gas) -> TwoSpoolEngine { build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, P_DESIGN, TWO) }

// --- one shaft --------------------------------------------------------------------------------

#[test]
fn one_shaft_matcher_is_the_same_engine_scaled_on_both_branches() {
    for (g, gas) in gases() {
        let (mut choked, mut subsonic) = (0, 0);
        let m = OffDesignMatcher::new(build_turbojet(gas, 10.0, 1500.0, P_DESIGN, ONE), at(P_DESIGN, 0.85), 1.0);
        for (tt4, m0) in [(1500.0, 0.85), (1100.0, 0.3), (560.0, 0.85), (500.0, 0.85)] {
            let base = m.match_point(&at(P_DESIGN, m0), tt4);
            match base.branch { Branch::Choked => choked += 1, Branch::Subsonic => subsonic += 1 }
            for p0 in P_FLY {
                let o = m.match_point(&at(p0, m0), tt4);
                let tag = format!("{g} Tt4={tt4} M0={m0} p0={p0}");
                assert_eq!(o.branch, base.branch, "{tag}: branch");
                for (a, b, what) in [(o.pi_c, base.pi_c, "pi_c"), (o.pi_t, base.pi_t, "pi_t"),
                                     (o.tau_t, base.tau_t, "tau_t"), (o.m9, base.m9, "M9"),
                                     (o.p9 / p0, base.p9 / P_DESIGN, "p9/p0"),
                                     (o.mdot_air / p0, base.mdot_air / P_DESIGN, "airflow/p0"),
                                     (o.thrust / p0, base.thrust / P_DESIGN, "thrust/p0")] {
                    assert!(rel(a, b) < SIMILAR, "{tag}: {what} {a} vs {b}");
                }
            }
        }
        // Both branches must be exercised, or half the repair is untested.
        assert!(choked > 0 && subsonic > 0, "{g}: choked {choked}, subsonic {subsonic}");
    }
}

#[test]
fn one_shaft_map_matcher_is_the_same_engine_scaled() {
    for (g, gas) in gases() {
        let m = MapMatcher::new(build_turbojet(gas, 10.0, 1500.0, P_DESIGN, ONE), at(P_DESIGN, 0.85), 1.0, hp());
        let base = m.match_point(&at(P_DESIGN, 0.85), 1300.0);
        for p0 in P_FLY {
            let o = m.match_point(&at(p0, 0.85), 1300.0);
            for (a, b, what) in [(o.base.pi_c, base.base.pi_c, "pi_c"), (o.eta_c, base.eta_c, "eta_c"),
                                 (o.base.thrust / p0, base.base.thrust / P_DESIGN, "thrust/p0")] {
                assert!(rel(a, b) < SIMILAR, "{g} p0={p0}: {what} {a} vs {b}");
            }
        }
    }
}

// --- two shafts -------------------------------------------------------------------------------

/// The numbers every two-shaft result carries, as `(value, what)` — pressures divided by `p0`.
fn two_spool_reading(r: &turbojet::two_spool::TwoSpoolResult, p0: f64) -> [(f64, &'static str); 6] {
    [(r.pi_lpc, "pi_lpc"), (r.pi_hpc, "pi_hpc"), (r.pi_hpt, "pi_hpt"), (r.pi_lpt, "pi_lpt"),
     (r.mdot_air / p0, "airflow/p0"), (r.thrust / p0, "thrust/p0")]
}

fn assert_similar(tag: &str, a: &[(f64, &'static str)], b: &[(f64, &'static str)]) {
    for ((x, what), (y, _)) in a.iter().zip(b) {
        assert!(rel(*x, *y) < SIMILAR, "{tag}: {what} {x} vs {y}");
    }
}

#[test]
fn two_shaft_matchers_are_the_same_engine_scaled() {
    for (g, gas) in gases() {
        let m38 = TwoSpoolMatcher::new(two(gas.clone()), at(P_DESIGN, 0.85), 1.0);
        let m39 = TwoSpoolMapMatcher::new(two(gas.clone()), at(P_DESIGN, 0.85), 1.0, lp(), hp());
        let m42 = TwoSpoolBleedMatcher::new(two(gas), at(P_DESIGN, 0.85), 1.0, lp(), hp(), 0.05);
        for tt4 in [1500.0, 1300.0] {
            let b38 = m38.match_point(&at(P_DESIGN, 0.85), tt4).two();
            let b39 = m39.match_point(&at(P_DESIGN, 0.85), tt4).two();
            let b42 = m42.match_point(&at(P_DESIGN, 0.85), tt4);
            for p0 in P_FLY {
                let f = at(p0, 0.85);
                let tag = format!("{g} Tt4={tt4} p0={p0}");
                assert_similar(&format!("38 {tag}"), &two_spool_reading(&m38.match_point(&f, tt4).two(), p0),
                               &two_spool_reading(&b38, P_DESIGN));
                assert_similar(&format!("39 {tag}"), &two_spool_reading(&m39.match_point(&f, tt4).two().base, p0),
                               &two_spool_reading(&b39.base, P_DESIGN));
                assert_similar(&format!("42 {tag}"), &two_spool_reading(&m42.match_point(&f, tt4).base.base, p0),
                               &two_spool_reading(&b42.base.base, P_DESIGN));
            }
        }
    }
}

#[test]
fn two_shaft_loads_given_as_shares_scale_too() {
    // Rungs 90–92 are modelled on the perfect gas only.
    let design = || two(cpg());
    let m90 = CustomerBleedMatcher::new(design().with_customer_bleed(CustomerBleed::share(0.04)),
                                        at(P_DESIGN, 0.85), 1.0, lp(), hp(), 0.0);
    let m91 = TurbineCoolingMatcher::new(design().with_turbine_cooling(TurbineCooling::new(0.05, 0.03)),
                                         at(P_DESIGN, 0.85), 1.0, lp(), hp(), 0.0);
    let m92 = ShaftOfftakeMatcher::new(design().with_shaft_offtake(ShaftOfftake::hp(Demand::Share(0.05))),
                                       at(P_DESIGN, 0.85), 1.0, lp(), hp(), 0.0);
    for tt4 in [1500.0, 1300.0] {
        let d = at(P_DESIGN, 0.85);
        let (b90, b91, b92) = (m90.match_point(&d, tt4), m91.match_point(&d, tt4), m92.match_point(&d, tt4));
        assert!(b90.booking.is_some() && b91.booking.is_some() && b92.booking.is_some(),
                "Tt4={tt4}: every load must be ACTIVE, or this tests rung 42's body three times");
        for p0 in P_FLY {
            let f = at(p0, 0.85);
            assert_similar(&format!("90 Tt4={tt4} p0={p0}"), &two_spool_reading(&m90.match_point(&f, tt4).base.base, p0),
                           &two_spool_reading(&b90.base.base, P_DESIGN));
            assert_similar(&format!("91 Tt4={tt4} p0={p0}"), &two_spool_reading(&m91.match_point(&f, tt4).base.base, p0),
                           &two_spool_reading(&b91.base.base, P_DESIGN));
            assert_similar(&format!("92 Tt4={tt4} p0={p0}"), &two_spool_reading(&m92.match_point(&f, tt4).base.base, p0),
                           &two_spool_reading(&b92.base.base, P_DESIGN));
        }
    }
}

#[test]
fn two_shaft_transient_instant_is_the_same_engine_scaled() {
    // An UNBALANCED instant (rung 40), off both design speeds — the thrust tail every two-shaft
    // march steps through.
    for (g, gas) in gases() {
        let t = TwoSpoolTransient::new(two(gas), at(P_DESIGN, 0.85), 1.0, lp(), hp(), 1.0);
        let c = t.core();
        let base = c.instant(&at(P_DESIGN, 0.85), 0.97, 0.98, 1400.0);
        for p0 in P_FLY {
            let o = c.instant(&at(p0, 0.85), 0.97, 0.98, 1400.0);
            for (a, b, what) in [(o.pi_hpt, base.pi_hpt, "pi_hpt"), (o.pi_lpt, base.pi_lpt, "pi_lpt"),
                                 (o.sp_thrust, base.sp_thrust, "specific thrust"),
                                 (o.tau_hpt, base.tau_hpt, "tau_hpt"), (o.tau_lpt, base.tau_lpt, "tau_lpt"),
                                 (o.m9, base.m9, "M9")] {
                assert!(rel(a, b) < SIMILAR, "{g} p0={p0}: {what} {a} vs {b}");
            }
        }
    }
}

