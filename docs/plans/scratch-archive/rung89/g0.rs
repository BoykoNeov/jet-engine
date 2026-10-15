// G0: the reduces (tau_close Some(tau) == None bit for bit) and one march's wall time. Symmetric only.
use r89probe::*;
use turbojet::limited_bleed::BleedLimiter;
use turbojet::stator_transient::Ramp;
use turbojet::three_levers::march;
use turbojet::valve_reference::sibling;

fn main() {
    let fl = flight();
    let ms = bare(lp_map(), hp_map());
    let mt = bare(tilt_map(), tilt_map());
    let st = stat();
    let mut n_pts = 0usize; let mut n_runs = 0usize;
    for (shape, r, sm) in [("shaped", 0.5, 0.46), ("tilted", 1.0, 0.54), ("shaped", 0.25, 0.34)] {
        let m = if shape == "shaped" { &ms } else { &mt };
        let ramp = Ramp { tt4_lo: 1000.0, tt4_hi: 1400.0, r, s_settle: 1.2, ds: 0.005 };
        for tau in [0.01, 0.05, 0.2] {
            for inc in [false, true] {
                let v = BleedLimiter::with_tau(phi_of(sm), BMAX, Some(tau));
                let v = if inc { v.on_incidence() } else { v };
                let t0 = std::time::Instant::now();
                let a = march(&sibling(m, Some(&st), Some(v)), &fl, &ramp, None).0;
                let dt = t0.elapsed().as_secs_f64();
                let b = march(&sibling(m, Some(&st), Some(v.closing(tau))), &fl, &ramp, None).0;
                assert_eq!(a.len(), b.len());
                for (p, q) in a.iter().zip(&b) { assert_eq!(format!("{p:?}"), format!("{q:?}")); }
                // and on the stators-off machine
                let c = march(&sibling(m, None, Some(v)), &fl, &ramp, None).0;
                let d = march(&sibling(m, None, Some(v.closing(tau))), &fl, &ramp, None).0;
                assert_eq!(c.len(), d.len());
                for (p, q) in c.iter().zip(&d) { assert_eq!(format!("{p:?}"), format!("{q:?}")); }
                n_pts += a.len() + c.len(); n_runs += 2;
                println!("{shape} r={r} sm={sm} tau={tau} inc={inc}: n={} march {:.2}s  OK", a.len(), dt);
            }
        }
    }
    println!("G0: closing(tau) == symmetric, bit for bit: {n_runs} run pairs, {n_pts} points");
}
