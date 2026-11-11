//! The open route's pinned-trigger miss: (1) every inner RK4 stage with g > 0 and r <= g before the first recorded
//! crossing, open, shape 0, (0.02, 0.1), step 0.02; (2) per open cell (shape x att), the first point where the two-speed
//! march at give-back 0.1 departs from rung 47's at the cut-in time, the crossing, and the largest relative gap before it.
use std::cell::RefCell;
use turbojet::fuel_transient::{AsymmetricLag, FuelPoint, PointExtra};
use turbojet::governor_lag::{self as gl, ClipLag, Demand};
use turbojet::sandbox_controls::{build, ControlsSettings, DS};
use turbojet::two_speed_clip::{self as tsc, TwoSpeedClip};
fn g_of(p: &FuelPoint) -> (f64, f64) { match p.extra { PointExtra::Asym { g, required } => (g, required), _ => panic!() } }
fn main() {
    // (1) the stages
    {
        let s = ControlsSettings { shape: 0, redline_on: true, ..ControlsSettings::defaults() };
        let sv = build(&s).unwrap();
        let ramp = s.schedule(sv.mf_lo, sv.mf_hi);
        let clip = TwoSpeedClip { tt4_max: s.redline, lag: AsymmetricLag::new(0.02, 0.1) };
        let lim = sv.lag_limiters();
        let ds = DS;
        let log = RefCell::new(Vec::new());
        let der = |a: f64, b: f64, g: f64, t: f64, tag: &str| {
            let d = tsc::try_der(&sv.core.fuel, &sv.flight, Demand::Ramp(&ramp), clip, &lim, None, a, b, g, t).unwrap();
            if g > 0.0 && d.required <= g { log.borrow_mut().push(format!("{tag} s={t:.4} g={g:e} r={:e} (g-r)/r={:e}", d.required, (g - d.required) / d.required)); }
            d
        };
        let ((mut a, mut b), mut g, mut t) = ((sv.start.nu_lp, sv.start.nu_hp), 0.0f64, 0.0f64);
        for k in 0..=((s.s_end() / ds).round() as usize) {
            let k1 = der(a, b, g, t, "k1");
            if g > 0.0 && k1.required <= g { println!("open: first recorded crossing at point {k} s={t:.4}"); break; }
            let k2 = der(a + ds / 2.0 * k1.da, b + ds / 2.0 * k1.db, g + ds / 2.0 * k1.dg, t + ds / 2.0, "k2");
            let k3 = der(a + ds / 2.0 * k2.da, b + ds / 2.0 * k2.db, g + ds / 2.0 * k2.dg, t + ds / 2.0, "k3");
            let k4 = der(a + ds * k3.da, b + ds * k3.db, g + ds * k3.dg, t + ds, "k4");
            a += ds / 6.0 * (k1.da + 2.0 * k2.da + 2.0 * k3.da + k4.da);
            b += ds / 6.0 * (k1.db + 2.0 * k2.db + 2.0 * k3.db + k4.db);
            g += ds / 6.0 * (k1.dg + 2.0 * k2.dg + 2.0 * k3.dg + k4.dg);
            t += ds;
        }
        for l in log.borrow().iter() { println!("  {l}"); }
    }
    // (2) the cells
    for shape in 0..4 {
        for att in [0.02, 0.05, 0.1] {
            let s = ControlsSettings { shape, redline_on: true, ..ControlsSettings::defaults() };
            let sv = build(&s).unwrap();
            let ramp = s.schedule(sv.mf_lo, sv.mf_hi);
            let lim = sv.lag_limiters();
            let st = (sv.start.nu_lp, sv.start.nu_hp);
            let two = tsc::march(&sv.core.fuel, &sv.flight, Demand::Ramp(&ramp),
                                 TwoSpeedClip { tt4_max: s.redline, lag: AsymmetricLag::new(att, 0.1) }, &lim, None, st, s.s_end(), DS).0;
            let one = gl::march(&sv.core.fuel, &sv.flight, Demand::Ramp(&ramp), ClipLag { tt4_max: s.redline, tau: att },
                                &lim, None, st, s.s_end(), DS).0;
            let k = (1..two.len()).find(|&i| g_of(&two[i - 1]).0 > 0.0 && { let (g, r) = g_of(&two[i]); r <= g }).unwrap();
            let first = (0..k).find(|&i| two[i].tt4.to_bits() != one[i].tt4.to_bits() || g_of(&two[i]).0.to_bits() != g_of(&one[i]).0.to_bits());
            let worst = (0..k).map(|i| ((two[i].tt4 - one[i].tt4) / one[i].tt4).abs().max(((two[i].mf - one[i].mf) / one[i].mf).abs()))
                .fold(0.0f64, f64::max);
            println!("open shape {shape} att {att}: crossing point {k} (s={:.2}); first departure {:?}; worst rel gap before it {worst:e}",
                     two[k].s, first.map(|i| (i, two[i].s)));
            // The overshoot across give-back times, and where it lies against the first departure.
            let peak = |p: &[FuelPoint]| p.iter().enumerate().map(|(k, x)| (k, x.tt4)).fold((0, f64::NEG_INFINITY), |a, b| if b.1 > a.1 { b } else { a });
            let runs: Vec<Vec<FuelPoint>> = [0.1, 0.4, 0.8].iter().map(|&rel| tsc::march(&sv.core.fuel, &sv.flight, Demand::Ramp(&ramp),
                TwoSpeedClip { tt4_max: s.redline, lag: AsymmetricLag::new(att, rel) }, &lim, None, st, s.s_end(), DS).0).collect();
            let split = (0..runs[0].len()).find(|&i| runs[0][i].tt4.to_bits() != runs[2][i].tt4.to_bits());
            let p1 = peak(&one);
            let pk: Vec<(usize, f64)> = runs.iter().map(|r| peak(r)).collect();
            let over = |t: f64| t - s.redline;
            let spread = pk.iter().map(|p| over(p.1)).fold(f64::NEG_INFINITY, f64::max) - pk.iter().map(|p| over(p.1)).fold(f64::INFINITY, f64::min);
            println!("    rel 0.1 vs 0.8 first split {:?}; peak point (rel 0.1/0.4/0.8) {:?}; overshoot K {:?}; spread {spread:e} K; rung 47 at att: point {} overshoot {:e} K",
                     split, pk.iter().map(|p| p.0).collect::<Vec<_>>(), pk.iter().map(|p| over(p.1)).collect::<Vec<_>>(), p1.0, over(p1.1));
        }
    }
}
