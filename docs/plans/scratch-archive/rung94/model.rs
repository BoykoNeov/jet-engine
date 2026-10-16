//! Rung 94 anchor § 1 — THE MODEL from § 0's measured partials (existing code only). For a hold of quantity Q
//! (ln thrust, or HP speed) against a fixed power load w on one shaft, the burner temperature rise ΔT solves
//! Q(T0 + ΔT, w) = Q(T0, 0). LINEAR: ΔT = −Q_w w / Q_T. QUADRATIC (all second partials measured: Q_TT along the
//! line, Q_ww from w 0.01/0.02/0.04, Q_Tw from Q_w at T0 ± 10): the correction, whose size sets the bar on the
//! linear model. Then each flow coefficient's shift under the hold, linear. Then REACHABILITY: the existing loaded
//! matcher at (T0 + ΔT_lin, w) — solve or refuse only.
use jet_rung94_probe::*;
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};

#[derive(Clone, Copy)]
struct P { lnf: f64, nh: f64, pl: f64, ph: f64 }

fn main() {
    for (name, ml, mh) in maps() {
        let mk = |so: Option<ShaftOfftake>| {
            let e = match so { Some(o) => design().with_shaft_offtake(o), None => design() };
            ShaftOfftakeMatcher::new(e, flight(), 1.0, ml, mh, 0.0)
        };
        let q = |m: &ShaftOfftakeMatcher, t: f64| m.try_match_point(&flight(), t).ok()
            .map(|r| P { lnf: r.base.base.thrust.ln(), nh: r.base.n_hp_ratio, pl: r.base.phi_lp.ln(), ph: r.base.phi_hp.ln() });
        let m0 = mk(None);
        for (shaft, hp) in [("HP", true), ("LP", false)] {
            let load = |w: f64| if hp { ShaftOfftake::hp(Demand::power(w)) } else { ShaftOfftake::lp(Demand::power(w)) };
            let (m1, m2, m4) = (mk(Some(load(0.01))), mk(Some(load(0.02))), mk(Some(load(0.04))));
            let (s1, s2, s4) = (mk(Some(load(0.005))), mk(Some(load(0.01))), mk(Some(load(0.02))));
            println!("\n== {name}, {shaft}-shaft power: T0 w | HOLD THRUST dT lin / quad (bar) dlnphiL dlnphiH (fixed-T: L H) reach | HOLD N_H dT lin / quad (bar) dlnF dlnphiL dlnphiH reach");
            for (t0, w) in [(1500.0, 0.05), (1200.0, 0.05), (1000.0, 0.05), (800.0, 0.02), (700.0, 0.02)] {
                // partials at (t0, 0)
                let at = |m: &ShaftOfftakeMatcher, t: f64| q(m, t).unwrap();
                let (b, bp, bm) = (at(&m0, t0), at(&m0, t0 + 10.0), at(&m0, t0 - 10.0));
                let (bpp, bmm) = (at(&m0, t0 + 25.0), at(&m0, t0 - 25.0));
                let dt = |g: &dyn Fn(&P) -> f64| (g(&bp) - g(&bm)) / 20.0;
                let dtt = |g: &dyn Fn(&P) -> f64| (g(&bpp) - 2.0 * g(&b) + g(&bmm)) / 625.0;
                // load partials (Richardson first, second difference) at t and t +- 10
                let dw_at = |t: f64, g: &dyn Fn(&P) -> f64| -> Option<(f64, f64)> {
                    let small = t0 <= 800.0;
                    let (h, a1, a2, a4) = if small { (0.005, &s1, &s2, &s4) } else { (0.01, &m1, &m2, &m4) };
                    let (p0, p1, p2, p4) = (q(&m0, t)?, q(a1, t)?, q(a2, t)?, q(a4, t)?);
                    let first = 2.0 * (g(&p1) - g(&p0)) / h - (g(&p2) - g(&p0)) / (2.0 * h);
                    let second = (g(&p4) - 2.0 * g(&p2) + g(&p0)) / (4.0 * h * h);
                    Some((first, second))
                };
                let hold = |g: &dyn Fn(&P) -> f64| -> Option<(f64, f64, f64)> {
                    let (qw, qww) = dw_at(t0, g)?;
                    let qwp = dw_at(t0 + 10.0, g)?.0;
                    let qwm = dw_at(t0 - 10.0, g)?.0;
                    let qtw = (qwp - qwm) / 20.0;
                    let (qt, qtt) = (dt(g), dtt(g));
                    let lin = -qw * w / qt;
                    // quadratic: qt d + qtt d^2/2 + qw w + qww w^2/2 + qtw w d = 0, the root nearest lin
                    let (a, bb, c) = (0.5 * qtt, qt + qtw * w, qw * w + 0.5 * qww * w * w);
                    let quad = if a.abs() < 1e-30 { -c / bb } else {
                        let disc = bb * bb - 4.0 * a * c;
                        let r1 = (-bb + disc.max(0.0).sqrt()) / (2.0 * a);
                        let r2 = (-bb - disc.max(0.0).sqrt()) / (2.0 * a);
                        if (r1 - lin).abs() < (r2 - lin).abs() { r1 } else { r2 }
                    };
                    Some((lin, quad, (3.0 * (quad - lin).abs()).max(1.0)))
                };
                let reach = |t: f64| -> &str { if q(&mk(Some(load(w))), t).is_some() { "solves" } else { "REFUSED" } };
                let (fw, _) = dw_at(t0, &|p: &P| p.lnf).unwrap_or((f64::NAN, 0.0));
                let (lw, _) = dw_at(t0, &|p: &P| p.pl).unwrap_or((f64::NAN, 0.0));
                let (hw, _) = dw_at(t0, &|p: &P| p.ph).unwrap_or((f64::NAN, 0.0));
                let (lt, ht, ft) = (dt(&|p: &P| p.pl), dt(&|p: &P| p.ph), dt(&|p: &P| p.lnf));
                let mut row = format!("{t0:.0} {w} |");
                match hold(&|p: &P| p.lnf) {
                    Some((lin, quad, bar)) => row += &format!(" F: {lin:+.1} / {quad:+.1} ({bar:.1}) {:+.4} {:+.4} (fixed {:+.4} {:+.4}) {} |",
                        lw * w + lt * lin, hw * w + ht * lin, lw * w, hw * w, reach(t0 + lin)),
                    None => row += " F: (a load partial refused) |",
                }
                match hold(&|p: &P| p.nh) {
                    Some((lin, quad, bar)) => row += &format!(" N_H: {lin:+.1} / {quad:+.1} ({bar:.1}) {:+.4} {:+.4} {:+.4} {}",
                        fw * w + ft * lin, lw * w + lt * lin, hw * w + ht * lin, reach(t0 + lin)),
                    None => row += " N_H: (a load partial refused)",
                }
                println!("{row}");
            }
        }
    }
}
