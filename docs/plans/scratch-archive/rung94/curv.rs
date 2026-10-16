//! Rung 94 anchor § 0 — GIVENS 2, existing code: (a) the responses at FINITE fixed-power loads at a fixed Tt4
//! (curvature in w), (b) the line's curvature in Tt4, (c) the largest fixed power the engine carries at a fixed
//! Tt4 (rung 92's fold), both shafts, flat and flow/press.
use jet_rung94_probe::*;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};

fn main() {
    for (name, ml, mh) in maps() {
        let mk = |so: Option<ShaftOfftake>| {
            let e = match so { Some(o) => design().with_shaft_offtake(o), None => design() };
            ShaftOfftakeMatcher::new(e, flight(), 1.0, ml, mh, 0.0)
        };
        let m0 = mk(None);
        let q = |m: &ShaftOfftakeMatcher, t: f64| m.try_match_point(&flight(), t).ok()
            .map(|r| (r.base.base.thrust, r.base.n_hp_ratio, r.base.n_lp_ratio, r.base.phi_lp, r.base.phi_hp, r.base.base.station("3").tt));
        println!("\n== {name} (b): the line's second derivatives at Tt4 (+-25 K): d2lnF/dT2, d2N_H/dT2, d2lnphiL/dT2, d2lnphiH/dT2");
        for t in [1500.0, 1200.0, 1000.0, 900.0, 800.0, 750.0, 700.0] {
            let (a, b, c) = (q(&m0, t + 25.0).unwrap(), q(&m0, t).unwrap(), q(&m0, t - 25.0).unwrap());
            let d2 = |x: f64, y: f64, z: f64| (x - 2.0 * y + z) / 625.0;
            println!("{t:.0} | {:.3e} {:.3e} {:.3e} {:.3e}", d2(a.0.ln(), b.0.ln(), c.0.ln()), d2(a.1, b.1, c.1),
                     d2(a.3.ln(), b.3.ln(), c.3.ln()), d2(a.4.ln(), b.4.ln(), c.4.ln()));
        }
        for (shaft, hp) in [("HP", true), ("LP", false)] {
            let load = |w: f64| if hp { ShaftOfftake::hp(Demand::power(w)) } else { ShaftOfftake::lp(Demand::power(w)) };
            println!("\n-- {name} (a), {shaft}-shaft power at fixed Tt4: w | lnF-lnF0, N_H-N_H0, lnphiL-.., lnphiH-.., Tt3-Tt3_0");
            for t in [1500.0, 1000.0, 800.0, 700.0] {
                let b = q(&m0, t).unwrap();
                let mut row = format!("{t:.0}:");
                for w in [0.01, 0.02, 0.04, 0.05, 0.10] {
                    match q(&mk(Some(load(w))), t) {
                        Some(p) => row += &format!(" [{w}: {:+.5} {:+.5} {:+.5} {:+.5} {:+.2}]", (p.0 / b.0).ln(), p.1 - b.1,
                                                   (p.3 / b.3).ln(), (p.4 / b.4).ln(), p.5 - b.5),
                        None => row += &format!(" [{w}: refused]"),
                    }
                }
                println!("{row}");
            }
            println!("-- {name} (c), {shaft}-shaft: the largest fixed power carried at fixed Tt4 (bisected to 1e-4)");
            for t in [700.0, 800.0, 900.0, 1000.0, 1100.0, 1200.0, 1300.0, 1400.0, 1500.0] {
                let ok = |w: f64| q(&mk(Some(load(w))), t).is_some();
                let (mut lo, mut hi) = (0.0, 0.49);
                if ok(hi) { println!("{t:.0}: carries 0.49"); continue; }
                while hi - lo > 1e-4 { let mid = 0.5 * (lo + hi); if ok(mid) { lo = mid } else { hi = mid } }
                println!("{t:.0}: w_max {lo:.4}");
            }
        }
    }
}
