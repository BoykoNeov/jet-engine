//! Rung 97 cost: the march with the branch vs with the pin, interleaved, best of 15, on the default move (choked) and
//! the +40 K 700 K move (unchoked start), plus the steady ends' build.
use std::time::Instant;
use turbojet::atmosphere::standard;
use turbojet::sandbox_controls::{build, ControlsSettings};

fn main() {
    let (t, p) = standard(0.0);
    for (lab, s) in [("default move", ControlsSettings::defaults()),
                     ("SL +40 K 700 K", ControlsSettings { t0: t + 40.0, p0: p, from: 700.0, ..ControlsSettings::defaults() })] {
        let sv = build(&s).unwrap();
        let base = &sv.core.fuel.inner.inner.base;
        let (mut best_new, mut best_old) = (f64::INFINITY, f64::INFINITY);
        for _ in 0..15 {
            base.lp_pinned.set(false); let c = Instant::now(); let _ = sv.march(); best_new = best_new.min(c.elapsed().as_secs_f64());
            base.lp_pinned.set(true); let c = Instant::now(); let _ = sv.march(); best_old = best_old.min(c.elapsed().as_secs_f64());
        }
        base.lp_pinned.set(false);
        println!("{lab}: march {:.1} ms with the branch, {:.1} ms on the pin ({:+.0} %)", best_new * 1e3, best_old * 1e3, (best_new / best_old - 1.0) * 100.0);
    }
}
