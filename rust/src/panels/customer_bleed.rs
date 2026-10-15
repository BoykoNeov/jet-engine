//! Rung 90's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_customer_bleed_table.txt`), a CHANGE detector only; its numbers
//! are checked by `tests/rung90.rs`. Six throttle points of the steady band and ONE valve cell
//! (shaped maps, `r` = 0.5, `ds` = 0.005) — the scored grid is the spec's.

use super::airflow::{design13, hp_map, lp_map};
use super::Design;
use crate::bleed::TwoSpoolBleedMatcher;
use crate::bleed_transient::LeverArm;
use crate::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use crate::limited_bleed::{build_limited_bleed, BleedLimiter};
use crate::map::ComponentMap;
use crate::pyfmt::Printer;
use crate::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient};

/// The anchor's linear model (`docs/plans/rung90-anchor-customer-bleed.md` § 1), evaluated on the
/// customer-free point: `(d ln phi_L/dc, d ln phi_H/dc)`.
fn model(tt2: f64, tt25: f64, tt3: f64, eta_lpc: f64, eta_hpc: f64) -> (f64, f64) {
    let k = 3.5;
    let a = |tau: f64, eta: f64| k * eta * (tau - 1.0) / (1.0 + eta * (tau - 1.0));
    let (tl, th) = (tt25 / tt2, tt3 / tt25);
    let (al, ah, t_l) = (a(tl, eta_lpc), a(th, eta_hpc), (tl - 1.0) / tl);
    (1.5 - al - ah * (1.0 - t_l), 0.5 + (1.0 - t_l) * (1.0 - ah))
}

/// `print_customer_bleed_table` — rung 90.
pub fn customer_bleed_table(p: &mut Printer, d: &Design) {
    p.print("\nTHE CUSTOMER BLEED (rung 90): air the aircraft takes off the HP compressor's delivery");
    p.print("(station 3) for its cabin - it never comes back. Two-spool engine, pi 3 x 6, flat maps.");
    let flat = ComponentMap::default();
    let de = |cb: Option<CustomerBleed>| match cb {
        Some(c) => design13(d).with_customer_bleed(c),
        None => design13(d),
    };
    let m0 = CustomerBleedMatcher::new(de(None), d.flight, 1.0, flat, flat, 0.0);
    let m2 = CustomerBleedMatcher::new(de(Some(CustomerBleed::share(0.02))), d.flight, 1.0, flat, flat, 0.0);
    let m5 = CustomerBleedMatcher::new(de(Some(CustomerBleed::share(0.05))), d.flight, 1.0, flat, flat, 0.0);
    let b5 = TwoSpoolBleedMatcher::new(design13(d), d.flight, 1.0, flat, flat, 0.05);

    p.print("\n  1. How each compressor's flow coefficient moves per unit customer share, measured against");
    p.print("     the anchor's model (fixed turbine-inlet temperature; + is away from surge).");
    p.print(format!("\n  {:>6}{:>9}{:>9}{:>9}{:>9}", "Tt4", "LP", "model", "HP", "model"));
    let band = [1500.0, 1200.0, 1000.0, 900.0, 800.0, 700.0];
    for tt4 in band {
        let r0 = m0.match_point(&d.flight, tt4).base;
        let r2 = m2.match_point(&d.flight, tt4).base;
        let s = &r0.base;
        let (ml, mh) = model(s.station("2").tt, s.station("25").tt, s.station("3").tt, r0.eta_lpc, r0.eta_hpc);
        p.print(format!("  {tt4:>6.0}{:>+9.4}{ml:>+9.4}{:>+9.4}{mh:>+9.4}",
                        (r2.phi_lp / r0.phi_lp).ln() / 0.02, (r2.phi_hp / r0.phi_hp).ln() / 0.02));
    }
    p.print("\n  2. What 5 % of the air costs: the customer bleed (station 3) against rung 42's interstage");
    p.print("     bleed (station 25), same turbine-inlet temperature, thrust and fuel per unit thrust.");
    p.print(format!("\n  {:>6}{:>12}{:>12}{:>12}{:>12}", "Tt4", "dF cust %", "dF b %", "dTSFC cust", "dTSFC b"));
    for tt4 in [1500.0, 1100.0, 800.0] {
        let r0 = m0.match_point(&d.flight, tt4).base.base.performance;
        let k5 = m5.match_point(&d.flight, tt4).booking.expect("active");
        let kb = b5.match_point(&d.flight, tt4).booking.expect("open");
        p.print(format!("  {tt4:>6.0}{:>+12.3}{:>+12.3}{:>+12.3}{:>+12.3}",
                        100.0 * (k5.st_inlet / r0.specific_thrust - 1.0),
                        100.0 * (kb.st_inlet / r0.specific_thrust - 1.0),
                        100.0 * (k5.tsfc_inlet / r0.tsfc - 1.0), 100.0 * (kb.tsfc_inlet / r0.tsfc - 1.0)));
    }

    p.print("\n  3. The throttle slam with rung 64's protection valve (LP flow-coefficient floor, b_max 0.10):");
    p.print("     shaped maps, r = 0.5, Tt4 1000 -> 1400 K. Closest approach to surge with the valve shut,");
    p.print("     the valve's ceiling (fully open), and the air it bleeds holding two set points.");
    p.print(format!("\n  {:>10}{:>10}{:>10}{:>10}{:>12}{:>12}", "demand", "minphiL", "minphiH", "ceiling",
                    "bleed .38", "bleed .42"));
    let rp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r: 0.5, s_settle: 1.2, ds: 0.005 };
    for (name, cb) in [("none", None), ("share 5%", Some(CustomerBleed::share(0.05))),
                       ("5% amount", Some(CustomerBleed::amount(0.05)))] {
        let m: ScheduledStatorCore = match build_limited_bleed(de(cb), d.flight, 1.0, Some(lp_map()), Some(hp_map()),
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
    p.print("\n  A different arrow from rung 42's. The HP turbine now drives its compressor with less gas than");
    p.print("  the compressor pumps, so the HP spool gets a new freedom - pushed AWAY from surge at every power,");
    p.print("  with no sign change. The LP spool is pushed TOWARD surge at high power (the HP's pressure-ratio");
    p.print("  drop starves the LP face more than the missing air feeds it) and helped below ~910 K. The slam's");
    p.print("  closest approach sits on the high-power side, so the LP protection valve must bleed MORE to hold");
    p.print("  the same margin, its ceiling falls, and a fixed AMOUNT of air - a larger share at the slam's");
    p.print("  low-power start - costs more than the same share. Customer air also costs more thrust and fuel");
    p.print("  than interstage air: it carries the HP compressor's work as well.");
}
