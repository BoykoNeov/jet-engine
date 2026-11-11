//! Off protocol, after the scoring: H5 MISS at 0.8 and H6 VACUOUS read with a longer settle (4 instead of 2), and the
//! share reached in the frozen window; rung 106 store (g_c) and crossing at each lag. Helpers copied from score.rs.
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
    let k_move = (base().ramp / DSF).round_ties_even() as usize;
    for shape in 0..4 {
        println!("==== {}", CONTROL_SHAPES[shape]);
        for settle_len in [2.0, 4.0] {
            let sv = solver(&ControlsSettings { shape, settle: settle_len, ..base() });
            let inst = instant(&sv);
            let fi = shares(&sv, &inst);
            let (xi, _) = settle(&inst, &fi);
            let mut row = Vec::new();
            for &r in &REL {
                let p = run(&sv, 0.02, r, false);
                let q = run(&sv, r, r, false);
                let (fp, fq) = (shares(&sv, &p), shares(&sv, &q));
                let (xp, xp1) = settle(&p, &fp);
                let mx = fp.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let (kp, kq) = (crossing(&p), crossing(&q));
                row.push(format!("tau_rel {r}: two-speed dd {:.5} settle {:.4} (first {:.4}) max share {mx:.4} end share {:.4} | 106 dd {:.5} g_c {:.4e} crossing s={} g_end/g_c {:.3} max share {:.4} | two-speed g_c {:.4e} g_end/g_c {:.3}",
                    drawdown(&fp, k_move), n(xp), n(xp1), fp.last().unwrap(), drawdown(&fq, k_move),
                    kq.map(|k| lag(&q[k]).0).unwrap_or(f64::NAN), kq.map(|k| format!("{:.3}", q[k].s)).unwrap_or("-".into()),
                    kq.map(|k| lag(q.last().unwrap()).0 / lag(&q[k]).0).unwrap_or(f64::NAN),
                    fq.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                    kp.map(|k| lag(&p[k]).0).unwrap_or(f64::NAN), kp.map(|k| lag(p.last().unwrap()).0 / lag(&p[k]).0).unwrap_or(f64::NAN)));
            }
            println!("  settle {settle_len}: instant settle {:.4}", n(xi));
            for l in row { println!("    {l}"); }
        }
    }
}
