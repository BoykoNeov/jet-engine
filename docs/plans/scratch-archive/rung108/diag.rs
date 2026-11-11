//! Diagnose the pinned-trigger miss: march the shipped two-speed clip at (0.02, 0.1), governed, map 0, step 0.005, and
//! log every evaluation (recorded or inner stage) with g > 0 and r <= g before the first recorded crossing.
use std::cell::RefCell;
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::AsymmetricLag;
use turbojet::governor_lag::Demand;
use turbojet::sandbox_controls::{build, ControlsSettings, DS};
use turbojet::two_speed_clip::{self as tsc, TwoSpeedClip};
fn main() {
    let s = ControlsSettings { shape: 0, redline_on: true, govern: true, hold: Hold::Thrust, ..ControlsSettings::defaults() };
    let sv = build(&s).unwrap();
    let g0 = sv.governor.as_ref().unwrap();
    let clip = TwoSpeedClip { tt4_max: s.redline, lag: AsymmetricLag::new(0.02, 0.1) };
    let lim = sv.lag_limiters();
    let ds = DS / 4.0;
    let log = RefCell::new(Vec::new());
    let der = |a: f64, b: f64, g: f64, t: f64, tag: &str| {
        let d = tsc::try_der(&sv.core.fuel, &sv.flight, Demand::Governor(g0), clip, &lim, None, a, b, g, t).unwrap();
        if g > 0.0 && d.required <= g { log.borrow_mut().push(format!("{tag} s={t:.5} g={g:e} r={:e} r-g={:e}", d.required, d.required - g)); }
        d
    };
    let ((mut a, mut b), mut g, mut t) = ((sv.start.nu_lp, sv.start.nu_hp), 0.0f64, 0.0f64);
    let mut crossed = false;
    for k in 0..=((s.s_end() / ds).round() as usize) {
        let k1 = der(a, b, g, t, "k1");
        if g > 0.0 && k1.required <= g { println!("first recorded crossing at point {k} s={t}"); crossed = true; }
        if crossed { break; }
        let k2 = der(a + ds / 2.0 * k1.da, b + ds / 2.0 * k1.db, g + ds / 2.0 * k1.dg, t + ds / 2.0, "k2");
        let k3 = der(a + ds / 2.0 * k2.da, b + ds / 2.0 * k2.db, g + ds / 2.0 * k2.dg, t + ds / 2.0, "k3");
        let k4 = der(a + ds * k3.da, b + ds * k3.db, g + ds * k3.dg, t + ds, "k4");
        a += ds / 6.0 * (k1.da + 2.0 * k2.da + 2.0 * k3.da + k4.da);
        b += ds / 6.0 * (k1.db + 2.0 * k2.db + 2.0 * k3.db + k4.db);
        g += ds / 6.0 * (k1.dg + 2.0 * k2.dg + 2.0 * k3.dg + k4.dg);
        t += ds;
    }
    for l in log.borrow().iter() { println!("{l}"); }
}
