//! Rung 108's scoring on the SHIPPED code (anchor § 3, its reading rules frozen there): H1–H8 and the reported
//! quantities. Every scored run on the 0.005 step, governed and open alike. "Governed" = `two_speed_clip::march` with
//! rung 95's governor as the demand; "open" = the same march with the ramp; "106" = the same march at equal constants
//! (rung 106's march, by the G1 / construction gate); "instant" = rung 95's governed march with the instant redline.
//!
//! Vacuity rules as coded (from § 3, written before the first run):
//! * any cell whose clip never engages (g ≡ 0) ⇒ VACUOUS;
//! * H1: no overshoot at τ_att 0.1 ⇒ VACUOUS (ordering and ratio both);
//! * H6: "later than instant" scored where both settle; the τ_rel ordering VACUOUS when all three settle before their
//!   crossing; a run that never settles makes its cells VACUOUS;
//! * a pair where one run stops early is read over the stretch both reach, and the cell says so (CUT).
use std::time::Instant;
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::{AsymmetricLag, FuelPoint, PointExtra};
use turbojet::governor as gv;
use turbojet::governor_lag::Demand;
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver, SpeedLaw, ThrustBy, CONTROL_SHAPES, DS};
use turbojet::two_speed_clip::{self as tsc, TwoSpeedClip};

const DSF: f64 = DS / 4.0;
const ATT: [f64; 3] = [0.02, 0.05, 0.1];
const REL: [f64; 3] = [0.2, 0.4, 0.8];

fn base() -> ControlsSettings { ControlsSettings { govern: true, hold: Hold::Thrust, redline_on: true, ..ControlsSettings::defaults() } }
fn solver(s: &ControlsSettings) -> ControlsSolver {
    build(&ControlsSettings { gov_lag_on: false, gov_lag_two: false, release_on: false, ..*s }).unwrap()
}
fn lag(p: &FuelPoint) -> (f64, f64) { match p.extra { PointExtra::Asym { g, required } => (g, required), _ => panic!() } }
fn run(sv: &ControlsSolver, att: f64, rel: f64, open: bool) -> Vec<FuelPoint> {
    let ramp = sv.settings.schedule(sv.mf_lo, sv.mf_hi);
    let dem = if open { Demand::Ramp(&ramp) } else { Demand::Governor(sv.governor.as_ref().unwrap()) };
    tsc::march(&sv.core.fuel, &sv.flight, dem, TwoSpeedClip { tt4_max: sv.settings.redline, lag: AsymmetricLag::new(att, rel) },
               &sv.lag_limiters(), None, (sv.start.nu_lp, sv.start.nu_hp), sv.settings.s_end(), DSF).0
}
fn instant(sv: &ControlsSolver) -> Vec<FuelPoint> {
    gv::march(&sv.core.fuel, &sv.flight, sv.governor.as_ref().unwrap(), &sv.limiters(), None,
              (sv.start.nu_lp, sv.start.nu_hp), sv.settings.s_end(), DSF).0
}
fn crossing(p: &[FuelPoint]) -> Option<usize> { (1..p.len()).find(|&i| lag(&p[i - 1]).0 > 0.0 && { let (g, r) = lag(&p[i]); r <= g }) }
fn engaged(p: &[FuelPoint]) -> bool { p.iter().any(|x| lag(x).0 > 0.0) }
fn overshoot(p: &[FuelPoint], red: f64) -> (usize, f64) {
    let (k, t) = p.iter().enumerate().map(|(k, x)| (k, x.tt4)).fold((0, f64::NEG_INFINITY), |a, b| if b.1 > a.1 { b } else { a });
    (k, t - red)
}
fn shares(sv: &ControlsSolver, p: &[FuelPoint]) -> Vec<f64> {
    let (f0, f1) = (gv::thrust(&sv.start), gv::thrust(&sv.end));
    p.iter().map(|x| (gv::thrust(&sv.reread(x).unwrap().base) - f0) / (f1 - f0)).collect()
}
/// The frozen settle time: the last upward crossing of 0.95 after which every point stays ≥ 0.95; and the first reach.
fn settle(p: &[FuelPoint], fr: &[f64]) -> (Option<f64>, Option<f64>) {
    let interp = |i: usize| -> f64 {
        if i == 0 { return p[0].s; }
        let (s0, s1, a, b) = (p[i - 1].s, p[i].s, fr[i - 1], fr[i]);
        s0 + (s1 - s0) * (0.95 - a) / (b - a)
    };
    let first = (0..fr.len()).find(|&i| fr[i] >= 0.95).map(interp);
    let last = if *fr.last().unwrap() < 0.95 { None } else {
        Some(match (0..fr.len()).rev().find(|&i| fr[i] < 0.95) { None => p[0].s, Some(i) => interp(i + 1) })
    };
    (last, first)
}
fn drawdown(fr: &[f64], k0: usize) -> f64 {
    let (mut hi, mut dd) = (f64::NEG_INFINITY, 0.0f64);
    for &v in fr.iter().skip(k0) { hi = hi.max(v); dd = dd.max(hi - v); }
    dd
}
fn trapz(p: &[FuelPoint], f: impl Fn(&FuelPoint) -> f64) -> f64 {
    p.windows(2).map(|w| 0.5 * (w[1].s - w[0].s) * (f(&w[0]) + f(&w[1]))).sum()
}
fn lp_min(p: &[FuelPoint]) -> (usize, f64) {
    p.iter().enumerate().map(|(k, x)| (k, x.phi_lp)).fold((0, f64::INFINITY), |a, b| if b.1 < a.1 { b } else { a })
}
fn v(hit: bool, vac: bool) -> &'static str { if vac { "VACUOUS" } else if hit { "HIT" } else { "MISS" } }
fn add(t: &mut [usize; 3], hit: bool, vac: bool) { t[if vac { 2 } else if hit { 0 } else { 1 }] += 1; }
fn tally(name: &str, t: [usize; 3]) { println!("== {name}: {} HIT, {} MISS, {} VACUOUS", t[0], t[1], t[2]); }
fn n(x: Option<f64>) -> f64 { x.unwrap_or(f64::NAN) }
fn cut<'a>(runs: &[&'a [FuelPoint]]) -> (usize, bool) {
    let m = runs.iter().map(|p| p.len()).min().unwrap();
    (m, runs.iter().any(|p| p.len() != m))
}

fn main() {
    let t0 = Instant::now();
    let k_move = (base().ramp / DSF).round_ties_even() as usize;
    {
        let sv = solver(&base());
        let g = run(&sv, 0.02, 0.4, false);
        let bad = g.iter().filter(|p| sv.reread(p).unwrap().base.tt4.to_bits() != p.tt4.to_bits()).count();
        println!("reread check: {bad} of {} points differ in Tt4; move ends at index {k_move}", g.len());
        assert_eq!(bad, 0);
    }
    let mut t = [[0usize; 3]; 16];
    let names = ["H1 overshoot falls with tau_att (gov)", "H1 overshoot falls with tau_att (open)",
                 "H1 ratio 0.02/0.1 in [0.2,0.5] (gov)", "H1 ratio 0.02/0.1 in [0.2,0.5] (open)",
                 "H2 governed overshoot > open", "H3 g_c falls as tau_att rises",
                 "H4 governed sags (> 0.002)", "H4 sag rises with tau_rel", "H5 fast cut sags less than 106",
                 "H6 settle later than instant", "H6 settle later with tau_rel", "H7 open never sags",
                 "H8 LP min before the crossing", "H8 LP min rises as tau_att falls"];
    for shape in 0..4 {
        let name = CONTROL_SHAPES[shape];
        let sv = solver(&ControlsSettings { shape, ..base() });
        let red = sv.settings.redline;
        println!("==== {name}");
        let inst = instant(&sv);
        let fi = shares(&sv, &inst);
        // H1 / H2 / H3 / H8 at tau_rel 0.4, tau_att over {0.02, 0.05, 0.1, 0.2}
        let atts4 = [0.02, 0.05, 0.1, 0.2];
        let g4: Vec<Vec<FuelPoint>> = atts4.iter().map(|&a| run(&sv, a, 0.4, false)).collect();
        let o4: Vec<Vec<FuelPoint>> = atts4.iter().map(|&a| run(&sv, a, 0.4, true)).collect();
        let (oi_k, oi) = overshoot(&inst, red);
        for (route, rs, ti, tr) in [("gov", &g4, 0usize, 2usize), ("open", &o4, 1, 3)] {
            let ov: Vec<(usize, f64)> = rs.iter().map(|p| overshoot(p, red)).collect();
            let vac = ov[2].1 <= 0.0 || rs.iter().any(|p| !engaged(p));
            let falls = ov.windows(2).all(|w| w[0].1 < w[1].1);
            let ratio = ov[0].1 / ov[2].1;
            let inb = (0.2..=0.5).contains(&ratio);
            add(&mut t[ti], falls, vac);
            add(&mut t[tr], inb, vac);
            println!("  H1 {route}: overshoot K by tau_att {:?} at points {:?} -> falls {} | ratio {ratio:.4} {}",
                     ov.iter().map(|x| format!("{:.3}", x.1)).collect::<Vec<_>>(), ov.iter().map(|x| x.0).collect::<Vec<_>>(),
                     v(falls, vac), v(inb, vac));
        }
        println!("  reported: instant overshoot {oi:.4} K at point {oi_k}");
        for i in 0..3 {
            let (og, oo) = (overshoot(&g4[i], red).1, overshoot(&o4[i], red).1);
            let vac = !engaged(&g4[i]) || !engaged(&o4[i]);
            add(&mut t[4], og > oo, vac);
            println!("  H2 tau_att {}: governed {og:.3} K open {oo:.3} K {}", atts4[i], v(og > oo, vac));
        }
        let gc: Vec<Option<f64>> = g4.iter().map(|p| crossing(p).map(|k| lag(&p[k]).0)).collect();
        let falls = gc.windows(2).all(|w| n(w[0]) > n(w[1]));
        let vac3 = gc.iter().any(|x| x.is_none());
        add(&mut t[5], falls, vac3);
        println!("  H3 g_c by tau_att {:?} (crossing s {:?}) -> {}", gc.iter().map(|x| format!("{:.5e}", n(*x))).collect::<Vec<_>>(),
                 g4.iter().map(|p| crossing(p).map(|k| format!("{:.3}", p[k].s)).unwrap_or("-".into())).collect::<Vec<_>>(), v(falls, vac3));
        // H8 (tau_att 0.02 / 0.05 / 0.1 at tau_rel 0.4)
        let (_, li) = lp_min(&inst);
        let mut lps = Vec::new();
        for i in 0..3 {
            let p = &g4[i];
            let (kl, ml) = lp_min(p);
            let k = crossing(p);
            let before = k.map_or(true, |k| kl < k);
            add(&mut t[12], before, !engaged(p));
            lps.push(ml);
            println!("  H8 tau_att {}: LP min {ml:.6} at s={:.3}, crossing s={} -> before {} | relief vs instant {:+.6}",
                     atts4[i], p[kl].s, k.map(|k| format!("{:.3}", p[k].s)).unwrap_or("-".into()), v(before, !engaged(p)), ml - li);
        }
        let rises = lps[0] > lps[1] && lps[1] > lps[2];
        add(&mut t[13], rises, false);
        println!("  H8 LP min by tau_att {:?} -> rises as tau_att falls {} (instant {li:.6}; 0.2: {:.6})",
                 lps.iter().map(|x| format!("{x:.6}")).collect::<Vec<_>>(), v(rises, false), lp_min(&g4[3]).1);
        // H4 / H5 / H6 / H7: the 3 x 3 grid
        let gg: Vec<Vec<Vec<FuelPoint>>> = ATT.iter().map(|&a| REL.iter().map(|&r| run(&sv, a, r, false)).collect()).collect();
        let og: Vec<Vec<Vec<FuelPoint>>> = ATT.iter().map(|&a| REL.iter().map(|&r| run(&sv, a, r, true)).collect()).collect();
        let s106: Vec<Vec<FuelPoint>> = REL.iter().map(|&tau| run(&sv, tau, tau, false)).collect();
        let all: Vec<&[FuelPoint]> = gg.iter().flatten().chain(og.iter().flatten()).chain(s106.iter()).map(|p| p.as_slice())
            .chain([inst.as_slice()]).collect();
        let (nmin, was_cut) = cut(&all);
        if was_cut { println!("  CUT: the grid read over {nmin} points (lengths {:?})", all.iter().map(|p| p.len()).collect::<Vec<_>>()); }
        let dd = |sv: &ControlsSolver, p: &[FuelPoint]| drawdown(&shares(sv, &p[..nmin]), k_move);
        for (ia, &a) in ATT.iter().enumerate() {
            let d: Vec<f64> = gg[ia].iter().map(|p| dd(&sv, p)).collect();
            for (ir, &r) in REL.iter().enumerate() {
                let p = &gg[ia][ir];
                let vac = !engaged(p);
                add(&mut t[6], d[ir] > 0.002, vac);
                let k = crossing(p);
                let gend = lag(&p[nmin - 1]).0;
                println!("  H4 ({a}, {r}): drawdown {:.5} {} | g_end/g_c {:.4}", d[ir], v(d[ir] > 0.002, vac),
                         k.map(|k| gend / lag(&p[k]).0).unwrap_or(f64::NAN));
            }
            let rises = d[0] < d[1] && d[1] < d[2];
            add(&mut t[7], rises, gg[ia].iter().any(|p| !engaged(p)));
            println!("  H4 tau_att {a}: drawdown rises with tau_rel {}", v(rises, gg[ia].iter().any(|p| !engaged(p))));
            for (ir, &r) in REL.iter().enumerate() {
                let p = &og[ia][ir];
                let d = dd(&sv, p);
                add(&mut t[11], d <= 0.002, !engaged(p));
                println!("  H7 open ({a}, {r}): drawdown {d:.5} {}", v(d <= 0.002, !engaged(p)));
            }
        }
        for (ir, &tau) in REL.iter().enumerate() {
            let (d2, d106) = (dd(&sv, &gg[0][ir]), dd(&sv, &s106[ir]));
            let vac = !engaged(&gg[0][ir]) || !engaged(&s106[ir]);
            add(&mut t[8], d2 < d106, vac);
            println!("  H5 tau {tau}: (0.02, {tau}) {d2:.5} vs 106 {d106:.5} {} | overshoot 106 {:.3} K",
                     v(d2 < d106, vac), overshoot(&s106[ir], red).1);
        }
        // H6 at tau_att 0.02
        let (xi, xi1) = settle(&inst[..nmin], &fi[..nmin]);
        let xs: Vec<(Option<f64>, Option<f64>, Option<usize>)> = gg[0].iter().map(|p| {
            let f = shares(&sv, &p[..nmin]);
            let (a, b) = settle(&p[..nmin], &f);
            (a, b, crossing(&p[..nmin]))
        }).collect();
        for (ir, &r) in REL.iter().enumerate() {
            let vac = xs[ir].0.is_none() || xi.is_none() || !engaged(&gg[0][ir]);
            let later = n(xs[ir].0) > n(xi);
            add(&mut t[9], later, vac);
            println!("  H6 (0.02, {r}): settle {:.4} (first reach {:.4}; crossing s={}) vs instant {:.4} (first {:.4}) {}",
                     n(xs[ir].0), n(xs[ir].1), xs[ir].2.map(|k| format!("{:.3}", gg[0][ir][k].s)).unwrap_or("-".into()),
                     n(xi), n(xi1), v(later, vac));
        }
        let never = xs.iter().any(|x| x.0.is_none());
        let all_before = xs.iter().enumerate().all(|(ir, x)| match (x.0, x.2) {
            (Some(t), Some(k)) => t <= gg[0][ir][k].s, (Some(_), None) => true, _ => false });
        let mono = n(xs[0].0) < n(xs[1].0) && n(xs[1].0) < n(xs[2].0);
        add(&mut t[10], mono, never || all_before);
        println!("  H6 settle rises with tau_rel {}{}", v(mono, never || all_before), if all_before { " (all settle before the crossing)" } else { "" });
        // Reported: g_c, let-through, withheld against D1 (governed and open, tau_att 0.02, every tau_rel)
        for (route, rs) in [("gov", &gg[0]), ("open", &og[0])] {
            for (ir, &r) in REL.iter().enumerate() {
                let p = &rs[ir];
                if let Some(k) = crossing(p) {
                    let gc = lag(&p[k]).0;
                    let thru = trapz(&p[..k], |x| lag(x).1 - lag(x).0);
                    let held = trapz(&p[k..], |x| lag(x).0 - lag(x).1);
                    let gend = lag(p.last().unwrap()).0;
                    println!("  reported {route} (0.02, {r}): g_c {gc:.5e} at s={:.3} | let through {thru:.4e} (tau_att g_c {:.4e}) | withheld {held:.4e} (tau_rel (g_c - g_end) {:.4e})",
                             p[k].s, 0.02 * gc, r * (gc - gend));
                }
            }
        }
        println!("  ({:.0} s so far)", t0.elapsed().as_secs_f64());
    }
    for (i, nm) in names.iter().enumerate() { tally(nm, t[i]); }

    println!("-- reported: other laws at (0.02, 0.4): overshoot, H4 drawdown, H5 against 106 at 0.4");
    let laws: [(&str, ControlsSettings); 5] = [
        ("HP rate .1", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.1, ..base() }),
        ("HP prop 10", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 10.0, ..base() }),
        ("EPR", ControlsSettings { thrust_by: ThrustBy::Epr, ..base() }),
        ("N1 rate .15", ControlsSettings { thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.15, ..base() }),
        ("N1 prop 7", ControlsSettings { thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 7.0, ..base() }),
    ];
    for (lname, ls) in laws {
        for shape in 0..4 {
            let sv = solver(&ControlsSettings { shape, ..ls });
            let red = sv.settings.redline;
            let p = run(&sv, 0.02, 0.4, false);
            let q = run(&sv, 0.4, 0.4, false);
            let inst = instant(&sv);
            let (nmin, c) = cut(&[&p, &q, &inst]);
            let (d, d106) = (drawdown(&shares(&sv, &p[..nmin]), k_move), drawdown(&shares(&sv, &q[..nmin]), k_move));
            println!("{lname:11} {:10}: pts {}/{}/{}{} overshoot {:.3} K (106 at 0.4 {:.3}, instant {:.3}) | drawdown {d:.5} sags {} | vs 106 {d106:.5} less {} | engaged {}",
                     CONTROL_SHAPES[shape], p.len(), q.len(), inst.len(), if c { " CUT" } else { "" }, overshoot(&p, red).1,
                     overshoot(&q, red).1, overshoot(&inst, red).1, d > 0.002, d < d106, engaged(&p));
        }
    }
    println!("-- reported: the burner-temperature hold to 1600 K (crossing the redline), (0.02, 0.4) and 106 at 0.4");
    for shape in 0..4 {
        let sv = solver(&ControlsSettings { shape, hold: Hold::BurnerTemp, to: 1600.0, ..base() });
        let red = sv.settings.redline;
        let p = run(&sv, 0.02, 0.4, false);
        let q = run(&sv, 0.4, 0.4, false);
        let inst = instant(&sv);
        let (nmin, c) = cut(&[&p, &q, &inst]);
        let (d, d106) = (drawdown(&shares(&sv, &p[..nmin]), k_move), drawdown(&shares(&sv, &q[..nmin]), k_move));
        println!("{:10}: pts {}/{}/{}{} overshoot {:.3} K (106 {:.3}, instant {:.3}) | drawdown {d:.5} (106 {d106:.5}) | crossing {:?} | end Tt4 {:.2}",
                 CONTROL_SHAPES[shape], p.len(), q.len(), inst.len(), if c { " CUT" } else { "" }, overshoot(&p, red).1,
                 overshoot(&q, red).1, overshoot(&inst, red).1, crossing(&p).map(|k| p[k].s), p.last().unwrap().tt4);
    }
    println!("-- scoring took {:.0} s", t0.elapsed().as_secs_f64());
}
