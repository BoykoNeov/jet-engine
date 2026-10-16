//! Rung 94 anchor pre-freeze checks (the advisor's), existing code only:
//! (a) H2 at 700 K: the HP flow coefficient's hold-minus-fixed shift to SECOND order, phi_T(load)·ΔT with the
//!     slope read AT the load (finite differences on the loaded matcher at T0 ± 10) and integrated over ΔT_lin
//!     (the average of the slope at T0 and T0 + ΔT, both at the load);
//! (b) H5: for each load, the LOWEST burner temperature that carries it (bisected) and the thrust there.
use jet_rung94_probe::*;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};

fn main() {
    for (name, ml, mh) in maps() {
        let mk = |so: Option<ShaftOfftake>| {
            let e = match so { Some(o) => design().with_shaft_offtake(o), None => design() };
            ShaftOfftakeMatcher::new(e, flight(), 1.0, ml, mh, 0.0)
        };
        let q = |m: &ShaftOfftakeMatcher, t: f64| m.try_match_point(&flight(), t).ok()
            .map(|r| (r.base.base.thrust, r.base.phi_hp.ln(), r.base.phi_lp.ln()));
        for (shaft, hp) in [("HP", true), ("LP", false)] {
            let load = |w: f64| if hp { ShaftOfftake::hp(Demand::power(w)) } else { ShaftOfftake::lp(Demand::power(w)) };
            // (a) the 700 K cell: w 0.02, ΔT_lin from the anchor's table
            let dt_lin = match (name, hp) { ("flat", true) => 41.5, ("flat", false) => 41.6, (_, true) => 45.6, (_, false) => 40.1 };
            let m = mk(Some(load(0.02)));
            let at = |t: f64| q(&m, t).unwrap();
            // exact: the loaded HP lnphi at T0+ΔT minus at T0 (both under the load) is the hold-minus-fixed shift
            let (a0, a1) = (at(700.0), at(700.0 + dt_lin));
            let (b0, b1) = (at(700.0), at(700.0 + dt_lin));
            let _ = (b0, b1);
            println!("{name} {shaft} 700/0.02: hold-minus-fixed d ln phi_H over dT_lin {dt_lin}: {:+.5} (phi_L {:+.5})",
                     a1.1 - a0.1, a1.2 - a0.2);
            // and at 1000/0.05, 1500/0.05 for the record
            for (t0, w, dtl) in [(1000.0, 0.05, 78.3), (1500.0, 0.05, 47.7)] {
                let m = mk(Some(load(w)));
                if let (Some(x0), Some(x1)) = (q(&m, t0), q(&m, t0 + dtl)) {
                    println!("   {t0}/{w}: hold-minus-fixed d ln phi_H {:+.5}, d ln phi_L {:+.5}", x1.1 - x0.1, x1.2 - x0.2);
                }
            }
            // (b) the lowest Tt4 that carries each H5 load, and its thrust
            for w in [0.05, 0.08, 0.10] {
                let m = mk(Some(load(w)));
                let ok = |t: f64| q(&m, t).is_some();
                if !ok(1500.0) { println!("   w {w}: not carried even at 1500 K"); continue; }
                let (mut lo, mut hi) = (600.0, 1500.0);
                if ok(lo) { println!("   w {w}: carried at 600 K, thrust {:.1}", q(&m, lo).unwrap().0); continue; }
                while hi - lo > 0.5 { let mid = 0.5 * (lo + hi); if ok(mid) { hi = mid } else { lo = mid } }
                println!("   w {w}: lowest Tt4 carrying it {hi:.1} K, thrust there {:.2} N", q(&m, hi).unwrap().0);
            }
        }
        println!("   {name}: unloaded thrust at 700 K {:.2} N, 800 K {:.2} N, 1000 K {:.2} N", q(&mk(None), 700.0).unwrap().0,
                 q(&mk(None), 800.0).unwrap().0, q(&mk(None), 1000.0).unwrap().0);
    }
}
