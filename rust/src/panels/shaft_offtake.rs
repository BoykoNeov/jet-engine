//! Rung 92's panel — **born in Rust**, held to its own capture
//! (`rust/oracle/rust_owned/print_shaft_offtake_table.txt`), a CHANGE detector only; its numbers are
//! checked by `tests/rung92.rs`. Six throttle points of the steady band, the two identities at one
//! point, and ONE valve cell (shaped maps, `r` = 0.5, `ds` = 0.005) — the scored grid is the spec's.

use super::airflow::{design13, hp_map, lp_map};
use super::Design;
use crate::bleed::TwoSpoolBleedMatcher;
use crate::bleed_transient::LeverArm;
use crate::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use crate::limited_bleed::{build_limited_bleed, BleedLimiter};
use crate::map::ComponentMap;
use crate::pyfmt::Printer;
use crate::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};
use crate::stator_transient::{Ramp, ScheduledStatorCore, ScheduledStatorTransient};
use crate::two_spool::{TwoSpoolEngine, TwoSpoolMapResult};

/// The anchor's linear model (`docs/plans/rung92-anchor-shaft-offtake.md` § 1) on the unloaded
/// point, per unit SHARE: `((HP shaft: d ln phi_L, d ln phi_H), (LP shaft: …))`.
fn model(r0: &TwoSpoolMapResult) -> ((f64, f64), (f64, f64)) {
    let s = &r0.base;
    let (tt2, tt25, tt3) = (s.station("2").tt, s.station("25").tt, s.station("3").tt);
    let a = |tau: f64, eta: f64| 3.5 * eta * (tau - 1.0) / (1.0 + eta * (tau - 1.0));
    let (tl, th) = (tt25 / tt2, tt3 / tt25);
    let (al, ah, t_l) = (a(tl, r0.eta_lpc), a(th, r0.eta_hpc), (tl - 1.0) / tl);
    ((-ah, 0.5 - ah), (0.5 - al + ah * t_l, t_l * (ah - 1.0)))
}

/// `print_shaft_offtake_table` — rung 92.
pub fn shaft_offtake_table(p: &mut Printer, d: &Design) {
    p.print("\nSHAFT-POWER OFFTAKE (rung 92): the aircraft's generator load on the HP or the LP shaft - power,");
    p.print("no air. Two-spool engine, pi 3 x 6, flat maps, hardware sized with no load.");
    let flat = ComponentMap::default();
    let de = |so: Option<ShaftOfftake>| -> TwoSpoolEngine {
        match so { Some(o) => design13(d).with_shaft_offtake(o), None => design13(d) }
    };
    let m0 = ShaftOfftakeMatcher::new(de(None), d.flight, 1.0, flat, flat, 0.0);
    let at = |so: ShaftOfftake| ShaftOfftakeMatcher::new(de(Some(so)), d.flight, 1.0, flat, flat, 0.0);
    let one = |hp: bool, dm: Demand| if hp { ShaftOfftake::hp(dm) } else { ShaftOfftake::lp(dm) };
    let rich = |x0: f64, x1: f64, x2: f64| 2.0 * (x1 / x0).ln() / 0.01 - (x2 / x0).ln() / 0.02;

    p.print("\n  1. Per unit share of that shaft's turbine work: each compressor's flow coefficient, measured");
    p.print("     (extrapolated to zero load) against the anchor's model. Fixed burner-exit temperature;");
    p.print("     + is away from surge.");
    let band = [1500.0, 1200.0, 1100.0, 1000.0, 800.0, 700.0];
    for (name, hp) in [("HP", true), ("LP", false)] {
        let (m1, m2) = (at(one(hp, Demand::share(0.01))), at(one(hp, Demand::share(0.02))));
        p.print(format!("\n  {name:<6}{:>6}{:>9}{:>9}{:>9}{:>9}", "Tt4", "LP", "model", "HP", "model"));
        for tt4 in band {
            let r0 = m0.match_point(&d.flight, tt4).base;
            let (r1, r2) = (m1.match_point(&d.flight, tt4).base, m2.match_point(&d.flight, tt4).base);
            let (mh, ml) = model(&r0);
            let (xl, xh) = if hp { mh } else { ml };
            p.print(format!("  {:<6}{tt4:>6.0}{:>+9.4}{xl:>+9.4}{:>+9.4}{xh:>+9.4}", "",
                            rich(r0.phi_lp, r1.phi_lp, r2.phi_lp), rich(r0.phi_hp, r1.phi_hp, r2.phi_hp)));
        }
    }

    p.print("\n  2. A bleed is a power offtake plus air thrown away (flat maps, exact): at 1200 K, a 5 % share");
    p.print("     against rung 42's interstage bleed (LP shaft) and rung 90's customer bleed (both shafts).");
    p.print("     Speeds and pressure ratios agree; each face's flow coefficient differs by the dumped air.");
    let s = 0.05;
    let lp = at(ShaftOfftake::lp(Demand::share(s))).match_point(&d.flight, 1200.0).base;
    let both = at(ShaftOfftake::both(Demand::share(s), Demand::share(s))).match_point(&d.flight, 1200.0).base;
    let b42 = TwoSpoolBleedMatcher::new(design13(d), d.flight, 1.0, flat, flat, s).match_point(&d.flight, 1200.0).base;
    let c90 = CustomerBleedMatcher::new(design13(d).with_customer_bleed(CustomerBleed::share(s)), d.flight, 1.0,
                                        flat, flat, 0.0).match_point(&d.flight, 1200.0).base;
    p.print(format!("\n  {:>16}{:>12}{:>12}{:>12}{:>12}", "", "N_HP ratio", "piHPC ratio", "phiL ratio", "phiH ratio"));
    for (name, a, z) in [("LP vs rung 42", &lp, &b42), ("both vs rung 90", &both, &c90)] {
        p.print(format!("  {name:>16}{:>12.6}{:>12.6}{:>12.6}{:>12.6}", a.n_hp / z.n_hp,
                        a.base.pi_hpc / z.base.pi_hpc, a.phi_lp / z.phi_lp, a.phi_hp / z.phi_hp));
    }

    p.print("\n  3. The SAME power on either shaft (0.05 of the HP compressor's design power): the share of");
    p.print("     each turbine it is, the flow-coefficient shifts, and the thrust and fuel per unit thrust.");
    p.print(format!("\n  {:>6}{:>7}{:>10}{:>10}{:>10}{:>10}{:>10}", "Tt4", "shaft", "share", "dlnphiL", "dlnphiH",
                    "dF %", "dTSFC %"));
    for tt4 in [1500.0, 1200.0, 1000.0] {
        let r0 = m0.match_point(&d.flight, tt4).base;
        for (name, hp) in [("HP", true), ("LP", false)] {
            let r = at(one(hp, Demand::power(0.05))).match_point(&d.flight, tt4);
            let k = r.booking.expect("active");
            p.print(format!("  {tt4:>6.0}{name:>7}{:>10.4}{:>+10.4}{:>+10.4}{:>+10.3}{:>+10.3}",
                            if hp { k.share_hp } else { k.share_lp }, (r.base.phi_lp / r0.phi_lp).ln(),
                            (r.base.phi_hp / r0.phi_hp).ln(),
                            100.0 * (k.st_inlet / (r0.base.thrust / r0.base.mdot_air) - 1.0),
                            100.0 * (k.tsfc_inlet / r0.base.performance.tsfc - 1.0)));
        }
    }

    p.print("\n  4. The throttle slam with rung 64's protection valve (LP flow-coefficient floor, b_max 0.10):");
    p.print("     shaped maps, r = 0.5, Tt4 1000 -> 1400 K, 0.02 of the HP compressor's design power. Closest");
    p.print("     approach to surge with the valve shut, the valve's ceiling, the air it bleeds holding two set");
    p.print("     points, and when the HP speed covers 90 % of its rise.");
    p.print(format!("\n  {:>8}{:>10}{:>10}{:>10}{:>12}{:>12}{:>9}", "load", "minphiL", "minphiH", "ceiling",
                    "bleed .38", "bleed .42", "t90 HP"));
    let rp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r: 0.5, s_settle: 1.2, ds: 0.005 };
    let rows: [(&str, Option<ShaftOfftake>); 3] = [
        ("none", None),
        ("HP .02", Some(ShaftOfftake::hp(Demand::power(0.02)))),
        ("LP .02", Some(ShaftOfftake::lp(Demand::power(0.02)))),
    ];
    for (name, so) in rows {
        let m: ScheduledStatorCore = match build_limited_bleed(de(so), d.flight, 1.0, Some(lp_map()), Some(hp_map()),
                                                               1.0, &LeverArm::default()) {
            ScheduledStatorTransient::Full(c) => c,
            ScheduledStatorTransient::Degenerate(_) => unreachable!("both maps are passed"),
        };
        let shut = m.bill_cell(&d.flight, &rp, true);
        let tr = shut.traj.as_ref().expect("kept");
        let (n0, n1) = (tr[0].nu_hp, tr[tr.len() - 1].nu_hp);
        let target = n0 + 0.9 * (n1 - n0);
        let t90 = tr.windows(2).find(|w| w[0].nu_hp < target && w[1].nu_hp >= target)
            .map_or(f64::NAN, |w| w[0].s + (w[1].s - w[0].s) * (target - w[0].nu_hp) / (w[1].nu_hp - w[0].nu_hp));
        let full = m.at_lever(&LeverArm::constant(0.10)).bill_cell(&d.flight, &rp, false);
        let bill = |sm: f64| m.at_lever(&LeverArm::floored(BleedLimiter::new((1.0 + sm) * 0.55, 0.10)))
            .bill_cell(&d.flight, &rp, false).b_int;
        p.print(format!("  {name:>8}{:>10.5}{:>10.5}{:>10.5}{:>12.3e}{:>12.3e}{t90:>9.4}", shut.min_phi_lp,
                        shut.min_phi_hp, full.min_phi_lp, bill(0.38), bill(0.42)));
    }
    for line in NARRATIVE {
        p.print(*line);
    }
}

const NARRATIVE: &[&str] = &[
    "\n  A bleed is a power offtake plus air thrown away - exactly, on these maps - so a generator is the",
    "  half of a bleed that never leaves. On the HP shaft it pushes BOTH compressors toward surge at every",
    "  power: the HP compressor gets less work, its pressure ratio falls, and that starves the LP face too.",
    "  That is what the customer bleed's HP relief hid: its HP spool was helped only because its air LEFT.",
    "  On the LP shaft the same kilowatts cost the compressors almost nothing in the steady state - the LP's",
    "  own loss is offset by the HP's pressure rise, and the HP sees rung 42's response exactly. The thrust",
    "  and fuel bill is the same on either shaft - the kilowatts cost the same fuel; only which compressor",
    "  pays in surge margin differs. In the slam the LP valve bleeds several times more for an HP load and",
    "  less for an LP one; the HP load also slows the HP spool's rise. A fixed power is a growing share as",
    "  the throttle falls, and past a fold the engine has no steady point at all.",
];
