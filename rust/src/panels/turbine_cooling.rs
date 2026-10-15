//! Rung 91's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_turbine_cooling_table.txt`), a CHANGE detector only; its numbers
//! are checked by `tests/rung91.rs`. Six throttle points of the steady band and ONE valve cell
//! (shaped maps, `r` = 0.5, `ds` = 0.005) — the scored grid is the spec's.

use super::airflow::{design13, hp_map, lp_map};
use super::Design;
use crate::bleed_transient::LeverArm;
use crate::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use crate::limited_bleed::{build_limited_bleed, BleedLimiter};
use crate::map::ComponentMap;
use crate::pyfmt::Printer;
use crate::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient};
use crate::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
use crate::two_spool::{TwoSpoolEngine, TwoSpoolMapResult};

/// The anchor's linear model (`docs/plans/rung91-anchor-turbine-cooling.md` § 1), evaluated on the
/// uncooled point for ONE unit of the stream `(e1, e2)`: `(d ln pi_HPT, d ln phi_L, d ln phi_H)`.
fn model(r0: &TwoSpoolMapResult, kappa: f64, g_t: f64, e1: f64, e2: f64) -> (f64, f64, f64) {
    let s = &r0.base;
    let (tt2, tt25, tt3, tt4, tt45) = (s.station("2").tt, s.station("25").tt, s.station("3").tt,
                                       s.tt4, s.station("45").tt);
    let f = s.station("4").far;
    let a = |tau: f64, eta: f64| 3.5 * eta * (tau - 1.0) / (1.0 + eta * (tau - 1.0));
    let (tl, th) = (tt25 / tt2, tt3 / tt25);
    let (al, ah, t_l) = (a(tl, r0.eta_lpc), a(th, r0.eta_hpc), (tl - 1.0) / tl);
    let (tau, pi) = (s.tau_hpt, s.pi_hpt);
    let sigma = r0.eta_hpt * g_t * pi.powf(g_t) / tau;
    let rho = tau * sigma / (1.0 - tau);
    let (kt, ktp) = (kappa * tt3 / tt4, kappa * tt3 / tt45);
    let (m1, m2) = (e1 / (1.0 + f), e2 / (1.0 + f));
    let e = e1 + e2;
    let p = (e / (1.0 + f) + 0.5 * m1 * (kt - 1.0) + 0.5 * m2 * (ktp - 1.0)) / (1.0 - 0.5 * sigma);
    let h = -e + m1 * kt - rho * p;
    let l = -e + m1 * kt + sigma * p + m2 * ktp;
    (p, al * l + ah * (h - t_l * l) + e - 0.5 * l, (ah - 0.5) * (h - t_l * l) + 0.5 * t_l * l + e)
}

/// `print_turbine_cooling_table` — rung 91.
pub fn turbine_cooling_table(p: &mut Printer, d: &Design) {
    p.print("\nTURBINE COOLING AIR (rung 91): HP-compressor air that rejoins the gas at the turbines - the");
    p.print("VANE stream before the HP turbine rotor (it still works there), the BLADE stream after it (LP");
    p.print("turbine only). Two-spool engine, pi 3 x 6, flat maps, hardware sized with cooling off.");
    let flat = ComponentMap::default();
    let de = |tc: Option<TurbineCooling>| -> TwoSpoolEngine {
        match tc { Some(t) => design13(d).with_turbine_cooling(t), None => design13(d) }
    };
    let spec = &design13(d).gas.spec;
    let (kappa, g_t) = (spec.cp_c / spec.cp_t, (spec.gamma_t - 1.0) / spec.gamma_t);
    let m0 = TurbineCoolingMatcher::new(de(None), d.flight, 1.0, flat, flat, 0.0);
    let at = |tc: TurbineCooling| TurbineCoolingMatcher::new(de(Some(tc)), d.flight, 1.0, flat, flat, 0.0);

    p.print("\n  1. Per unit cooling share: the HP turbine's expansion ratio and each compressor's flow");
    p.print("     coefficient, measured (extrapolated to zero share) against the anchor's model. Fixed");
    p.print("     burner-exit temperature; + is away from surge (and, for the turbine, LESS expansion).");
    let band = [1500.0, 1200.0, 1100.0, 1000.0, 800.0, 700.0];
    for (name, unit) in [("VANE", (1.0, 0.0)), ("BLADE", (0.0, 1.0))] {
        let (m1, m2) = (at(TurbineCooling::new(0.01 * unit.0, 0.01 * unit.1)),
                        at(TurbineCooling::new(0.02 * unit.0, 0.02 * unit.1)));
        p.print(format!("\n  {name:<6}{:>6}{:>9}{:>9}{:>9}{:>9}{:>9}{:>9}", "Tt4", "piHPT", "model", "LP",
                        "model", "HP", "model"));
        for tt4 in band {
            let r0 = m0.match_point(&d.flight, tt4).base;
            let (r1, r2) = (m1.match_point(&d.flight, tt4).base, m2.match_point(&d.flight, tt4).base);
            let rich = |x0: f64, x1: f64, x2: f64| 2.0 * (x1 / x0).ln() / 0.01 - (x2 / x0).ln() / 0.02;
            let (mp, ml, mh) = model(&r0, kappa, g_t, unit.0, unit.1);
            p.print(format!("  {:<6}{tt4:>6.0}{:>+9.4}{mp:>+9.4}{:>+9.4}{ml:>+9.4}{:>+9.4}{mh:>+9.4}", "",
                            rich(r0.base.pi_hpt, r1.base.pi_hpt, r2.base.pi_hpt),
                            rich(r0.phi_lp, r1.phi_lp, r2.phi_lp), rich(r0.phi_hp, r1.phi_hp, r2.phi_hp)));
        }
    }

    p.print("\n  2. What 5 % of the HP compressor's air costs, at the same burner-exit temperature: thrust and");
    p.print("     fuel per unit thrust, against rung 90's customer bleed (the same air thrown overboard),");
    p.print("     and the rotor-inlet temperature Tt41 the vane air buys.");
    p.print(format!("\n  {:>6}{:>10}{:>10}{:>10}{:>11}{:>11}{:>11}", "Tt4", "Tt41 vane", "dF vane", "dF blade",
                    "dF cust", "dTSFC vane", "dTSFC cust"));
    let (v5, b5) = (at(TurbineCooling::new(0.05, 0.0)), at(TurbineCooling::new(0.0, 0.05)));
    let c5 = CustomerBleedMatcher::new(design13(d).with_customer_bleed(CustomerBleed::share(0.05)), d.flight,
                                       1.0, flat, flat, 0.0);
    for tt4 in [1500.0, 1100.0, 800.0] {
        let r0 = m0.match_point(&d.flight, tt4).base.base.performance;
        let kv = v5.match_point(&d.flight, tt4).booking.expect("active");
        let kb = b5.match_point(&d.flight, tt4).booking.expect("active");
        let kc = c5.match_point(&d.flight, tt4).booking.expect("active");
        let df = |st: f64| 100.0 * (st / r0.specific_thrust - 1.0);
        let dt = |ts: f64| 100.0 * (ts / r0.tsfc - 1.0);
        p.print(format!("  {tt4:>6.0}{:>10.1}{:>+10.3}{:>+10.3}{:>+11.3}{:>+11.3}{:>+11.3}", kv.tt41,
                        df(kv.st_inlet), df(kb.st_inlet), df(kc.st_inlet), dt(kv.tsfc_inlet), dt(kc.tsfc_inlet)));
    }

    p.print("\n  3. The throttle slam with rung 64's protection valve (LP flow-coefficient floor, b_max 0.10):");
    p.print("     shaped maps, r = 0.5, Tt4 1000 -> 1400 K. Closest approach to surge with the valve shut,");
    p.print("     the valve's ceiling (fully open), and the air it bleeds holding two set points.");
    p.print(format!("\n  {:>10}{:>10}{:>10}{:>10}{:>12}{:>12}", "demand", "minphiL", "minphiH", "ceiling",
                    "bleed .38", "bleed .42"));
    let rp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r: 0.5, s_settle: 1.2, ds: 0.005 };
    let rows: [(&str, TwoSpoolEngine); 4] = [
        ("none", de(None)),
        ("vane 5%", de(Some(TurbineCooling::new(0.05, 0.0)))),
        ("blade 5%", de(Some(TurbineCooling::new(0.0, 0.05)))),
        ("cust 5%", design13(d).with_customer_bleed(CustomerBleed::share(0.05))),
    ];
    for (name, e) in rows {
        let m: ScheduledStatorCore = match build_limited_bleed(e, d.flight, 1.0, Some(lp_map()), Some(hp_map()),
                                                               1.0, &LeverArm::default()) {
            ScheduledStatorTransient::Full(c) => c,
            ScheduledStatorTransient::Degenerate(_) => unreachable!("both maps are passed"),
        };
        let shut = m.bill_cell(&d.flight, &rp, false);
        let full = m.at_lever(&LeverArm::constant(0.10)).bill_cell(&d.flight, &rp, false);
        let bill = |sm: f64| m.at_lever(&LeverArm::floored(BleedLimiter::new((1.0 + sm) * 0.55, 0.10)))
            .bill_cell(&d.flight, &rp, false).b_int;
        p.print(format!("  {name:>10}{:>10.5}{:>10.5}{:>10.5}{:>12.3e}{:>12.3e}", shut.min_phi_lp,
                        shut.min_phi_hp, full.min_phi_lp, bill(0.38), bill(0.42)));
    }
    for line in NARRATIVE {
        p.print(*line);
    }
}

const NARRATIVE: &[&str] = &[
    "\n  Air that comes back is CHEAP in thrust and DEAR in surge margin. It rejoins ahead of the LP",
    "  turbine's choked throat, so that throat must pass more gas: its back-pressure rises, the HP turbine",
    "  expands LESS, and the HP spool loses more work than if the air had been thrown overboard. So the",
    "  customer bleed's arrow REVERSES: the HP spool is pushed toward surge at high power (below ~1115 K",
    "  for vane air, ~795 K for blade air, it is helped), and the LP spool toward surge at EVERY power -",
    "  the customer bleed's low-power relief is gone. Blade air is worse than vane air on both spools: it",
    "  back-pressures the HP turbine as much and does no work there. In the slam the LP protection valve",
    "  bleeds 2-12 times (vane) and 3-29 times (blade) what it bleeds for the same customer share, its",
    "  ceiling falls, and a valve that held its set point without cooling can no longer hold it at the",
    "  highest set points. The thrust bill runs the other way: 5 % vane air costs about a third of the",
    "  thrust the same customer share does.",
];
