//! Rung 94 anchor § 0 — GIVENS on EXISTING code (rung 92's loaded matcher): along rig A's running line, the
//! responses of thrust, both shaft speeds and both flow coefficients to the burner temperature and — at a fixed
//! burner temperature — to a fixed POWER load on either shaft (Richardson from w 0.01 / 0.02). No hold-thrust or
//! hold-speed code exists.
use jet_rung94_probe::*;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};

#[derive(Clone, Copy)]
struct Pt { f: f64, nl: f64, nh: f64, pl: f64, ph: f64, tt3: f64 }

fn main() {
    for (name, ml, mh) in maps() {
        let mk = |so: Option<ShaftOfftake>| {
            let e = match so { Some(o) => design().with_shaft_offtake(o), None => design() };
            ShaftOfftakeMatcher::new(e, flight(), 1.0, ml, mh, 0.0)
        };
        let at = |m: &ShaftOfftakeMatcher, t: f64| -> Option<Pt> {
            m.try_match_point(&flight(), t).ok().map(|r| Pt {
                f: r.base.base.thrust, nl: r.base.n_lp_ratio, nh: r.base.n_hp_ratio, pl: r.base.phi_lp,
                ph: r.base.phi_hp, tt3: r.base.base.station("3").tt })
        };
        let m0 = mk(None);
        println!("\n== {name}: unloaded line and its Tt4 slopes (central, +-10 K)");
        println!("Tt4  | F(N)    N_L    N_H    phi_L  phi_H  Tt3   | dlnF/dT  dN_L/dT  dN_H/dT  dlnphiL/dT dlnphiH/dT (per K)");
        let tt4s = [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0, 900.0, 800.0, 750.0, 700.0];
        for &t in &tt4s {
            let (p, a, b) = (at(&m0, t).unwrap(), at(&m0, t + 10.0).unwrap(), at(&m0, t - 10.0).unwrap());
            let d = |x: f64, y: f64| (x - y) / 20.0;
            println!("{t:.0} | {:.3} {:.4} {:.4} {:.4} {:.4} {:.1} | {:.3e} {:.3e} {:.3e} {:.3e} {:.3e}",
                     p.f, p.nl, p.nh, p.pl, p.ph, p.tt3, d(a.f.ln(), b.f.ln()), d(a.nl, b.nl), d(a.nh, b.nh),
                     d(a.pl.ln(), b.pl.ln()), d(a.ph.ln(), b.ph.ln()));
        }
        for (shaft, hp) in [("HP", true), ("LP", false)] {
            let load = |w: f64| if hp { ShaftOfftake::hp(Demand::power(w)) } else { ShaftOfftake::lp(Demand::power(w)) };
            let (m1, m2) = (mk(Some(load(0.01))), mk(Some(load(0.02))));
            println!("\n-- {name}, {shaft}-shaft POWER load at fixed Tt4: per unit w (Richardson 0.01/0.02)");
            println!("Tt4  | dlnF/dw  dN_L/dw  dN_H/dw  dlnphiL/dw dlnphiH/dw dTt3/dw");
            for &t in &tt4s {
                let (p0, p1, p2) = (at(&m0, t).unwrap(), at(&m1, t), at(&m2, t));
                let (Some(p1), Some(p2)) = (p1, p2) else { println!("{t:.0} | (a load point refused)"); continue; };
                let r = |g: &dyn Fn(&Pt) -> f64| 2.0 * (g(&p1) - g(&p0)) / 0.01 - (g(&p2) - g(&p0)) / 0.02;
                println!("{t:.0} | {:+.4} {:+.4} {:+.4} {:+.4} {:+.4} {:+.2}", r(&|p| p.f.ln()), r(&|p| p.nl), r(&|p| p.nh),
                         r(&|p| p.pl.ln()), r(&|p| p.ph.ln()), r(&|p| p.tt3));
            }
        }
    }
}
